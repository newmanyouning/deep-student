/**
 * TextbookContentView - 教材内容视图
 *
 * 统一应用面板中的教材阅读视图。
 * 根据 previewType 路由到不同的预览组件：
 * - pdf: PDF 查看器
 * - docx: DOCX 富文本预览
 * - xlsx: Excel 表格预览
 * - text: 纯文本预览
 * 
 * 元数据字段：
 * - filePath: string - 文件路径
 * - readingProgress: { page: number; lastReadAt?: number } - 阅读进度（PDF专用）
 * - pageCount: number - 总页数
 */

import React, { useState, useCallback, useMemo, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { WarningCircle, FileText, CircleNotch, ArrowClockwise } from '@phosphor-icons/react';
import { NotionButton } from '@/components/ui/NotionButton';
import { TextbookPdfViewer, type ReadingProgress, type Bookmark } from '@/features/pdf/components/TextbookPdfViewer';
import type { ContentViewProps } from '../UnifiedAppPanel';
import { dstu } from '@/dstu';
import { reportError } from '@/shared/result';
import { showGlobalNotification } from '@/components/UnifiedNotification';
import { invoke } from '@tauri-apps/api/core';
import { CustomScrollArea } from '@/components/custom-scroll-area';
import { vfsFileApi } from '@/api/vfsFileApi';
import { usePdfLoader } from '@/hooks/usePdfLoader';
import {
  decodeBase64ToText,
  estimateBase64Size,
  LARGE_FILE_THRESHOLD,
  uint8ArrayToBase64,
} from '@/utils/base64FileUtils';
import { PreviewProvider, usePreviewContext } from './PreviewContext';
import type { ToolbarPreviewType } from './UnifiedPreviewToolbar';
import { resolveTextbookPreviewType } from './textbookPreviewResolver';
import { RichDocumentPreview } from './RichDocumentPreview';
import { usePdfFocusListener } from './usePdfFocusListener';
import { usePdfProcessingStore, getProcessingHint } from '@/features/pdf/stores/pdfProcessingStore';
import { Scan } from '@phosphor-icons/react';

const toToolbarPreviewType = (type: string | null): ToolbarPreviewType => {
  if (type === 'docx' || type === 'xlsx' || type === 'pptx' || type === 'text') {
    return type;
  }
  return 'other' as const;
};

/**
 * 教材内容视图
 */
const TextbookContentViewInner: React.FC<ContentViewProps> = ({
  node,
}) => {
  const { t } = useTranslation(['textbook', 'common', 'learningHub']);
  const {
    zoomScale,
    fontScale,
    previewType,
    setZoomScale,
    setFontScale,
    resetZoom,
    resetFont,
    setPreviewType,
  } = usePreviewContext();

  // 页面选择状态
  const [selectedPages, setSelectedPages] = useState<Set<number>>(new Set());

  // 保存进度的防抖引用
  const saveProgressTimerRef = useRef<number | null>(null);

  // ★ 追踪最新值的 ref（用于 cleanup flush，避免闭包捕获过期值）
  const nodePathRef = useRef(node.path);
  const nodeIdRef = useRef(node.id);
  const nodeMetadataRef = useRef(node.metadata);
  const pendingProgressRef = useRef<ReadingProgress | null>(null);
  const pendingBookmarksRef = useRef<Bookmark[] | null>(null);

  // 同步最新值到 ref
  useEffect(() => {
    nodePathRef.current = node.path;
    nodeIdRef.current = node.id;
    nodeMetadataRef.current = node.metadata;
  }, [node.path, node.id, node.metadata]);
  
  // 非 PDF 文件的内容状态
  const [fileContent, setFileContent] = useState<string | null>(null);
  const [contentLoading, setContentLoading] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);
  
  // ★ 非 PDF 内容重新加载的触发计数器
  const [contentRetryCount, setContentRetryCount] = useState(0);

  // ★ PDF 初始态 spinner 超时检测（防止无限旋转）
  const [pdfInitTimedOut, setPdfInitTimedOut] = useState(false);

  // ======== OCR 相关状态 ========

  /** OCR 可用性检查结果（null=未检查, {configured:false}=未配置, {configured:true}=已配置） */
  const [ocrAvailability, setOcrAvailability] = useState<{ configured: boolean; modelName?: string } | null>(null);
  /** 是否显示 OCR 文本视图（true=OCR文本, false=原始PDF图像） */
  const [showOcrText, setShowOcrText] = useState(false);
  /** OCR 识别文本内容 */
  const [ocrTextContent, setOcrTextContent] = useState<string | null>(null);

  // 处理页面选择变化 + 广播给 Chat InputBar
  const handlePageSelectionChange = useCallback((pages: Set<number>) => {
    setSelectedPages(pages);
    // 广播选中页码到 Chat InputBar（通过自定义 DOM 事件）
    document.dispatchEvent(new CustomEvent('pdf-page-refs:update', {
      detail: {
        sourceId: node.sourceId,
        sourceName: node.name,
        pages: Array.from(pages).sort((a, b) => a - b),
      },
    }));
  }, [node.sourceId, node.name]);

  // 监听 Chat 侧发来的清除/移除选择事件
  // ★ 标签页：通过 sourceId 过滤，避免多个 PDF tab 互相干扰
  useEffect(() => {
    const handleClear = (event: Event) => {
      const detail = (event as CustomEvent<{ sourceId?: string }>).detail;
      if (detail?.sourceId && detail.sourceId !== node.sourceId) return;
      setSelectedPages(new Set());
    };
    const handleRemove = (event: Event) => {
      const detail = (event as CustomEvent<{ page: number; sourceId?: string }>).detail;
      if (detail?.sourceId && detail.sourceId !== node.sourceId) return;
      setSelectedPages((prev) => {
        const next = new Set(prev);
        next.delete(detail.page);
        return next;
      });
    };
    document.addEventListener('pdf-page-refs:clear', handleClear);
    document.addEventListener('pdf-page-refs:remove', handleRemove);
    return () => {
      document.removeEventListener('pdf-page-refs:clear', handleClear);
      document.removeEventListener('pdf-page-refs:remove', handleRemove);
    };
  }, [node.sourceId]);

  // 处理导出选中页面（已废弃，保留空回调以兼容 TextbookPdfViewer 接口）
  const handleExportSelectedPages = useCallback(() => {}, []);

  // 从 node.metadata.filePath 获取文件路径
  const filePath = node.metadata?.filePath as string | undefined;
  const [filePathStat, setFilePathStat] = useState<{ available: boolean; size?: number } | null>(
    filePath ? { available: true } : { available: false }
  );
  
  // 根据 previewType 确定渲染模式（优先使用数据库值，若为 none 则根据扩展名推断）
  const resolvedPreviewType = resolveTextbookPreviewType(node.previewType, node.name);
  const isPdf = resolvedPreviewType === 'pdf';
  const isDocx = resolvedPreviewType === 'docx';
  const isXlsx = resolvedPreviewType === 'xlsx';
  const isPptx = resolvedPreviewType === 'pptx';
  const isText = resolvedPreviewType === 'text';
  const isUnsupported = resolvedPreviewType === 'none';
  const needsFileContent = isDocx || isXlsx || isPptx || isText;

  /** 订阅 PDF 处理状态 Store（响应 OCR/文本提取等处理进度） */
  const processingStatus = usePdfProcessingStore(
    useCallback(
      (state) => (isPdf ? state.statusMap.get(node.sourceId) : undefined),
      [isPdf, node.sourceId],
    ),
  );

  // ★ 使用共享 Hook 监听 PDF 页码跳转事件
  const [focusRequest, handleFocusHandled] = usePdfFocusListener({
    enabled: isPdf,
    nodeId: node.id,
    nodeSourceId: node.sourceId,
    nodePath: node.path,
    nodeName: node.name,
  });

  useEffect(() => {
    const contextPreviewType = (isDocx || isXlsx || isPptx || isText)
      ? resolvedPreviewType
      : null;
    setPreviewType(contextPreviewType);
  }, [isDocx, isPptx, isText, isXlsx, resolvedPreviewType, setPreviewType]);

  // 校验 filePath 是否可访问（用于失效回退）
  useEffect(() => {
    let isActive = true;
    if (!filePath) {
      setFilePathStat({ available: false });
      return;
    }

    const checkFilePath = async () => {
      try {
        const size = await invoke<number>('get_file_size', { path: filePath });
        if (!isActive) return;
        setFilePathStat({ available: true, size });
      } catch (err: unknown) {
        if (!isActive) return;
        console.warn('[TextbookContentView] filePath not accessible, fallback to DB:', filePath, err);
        setFilePathStat({ available: false });
      }
    };

    void checkFilePath();
    return () => {
      isActive = false;
    };
  }, [filePath]);

  // ★ PDF-403 修复：教材 PDF 不传 filePath，避免触发 pdfstream:// 协议的目录白名单限制导致 403。
  // 改为始终通过 usePdfLoader → vfs_get_attachment_content 从 VFS blob/DB 加载。
  const effectiveFilePath = isPdf ? undefined : (filePathStat?.available ? filePath : undefined);
  const effectiveFileSize = isPdf ? undefined : (filePathStat?.available ? filePathStat.size : undefined);

  // 使用统一的 PDF 加载 Hook（支持缓存、去重、大文件检测）
  const {
    file: pdfFile,
    loading: pdfLoading,
    error: pdfError,
    isLargeFile: isPdfLargeFile,
    retry: retryPdfLoad,
  } = usePdfLoader({
    nodeId: node.id,
    fileName: node.name,
    filePath: effectiveFilePath,
    cacheKey: `${node.id}:${node.updatedAt || ''}`,
    enabled: isPdf && !effectiveFilePath, // 只有当是 PDF 且没有可用 filePath 时才从数据库加载
  });
  
  // 加载非 PDF 文件内容
  useEffect(() => {
    if (!needsFileContent) return;
    
    let isMounted = true;
    setContentLoading(true);
    setContentError(null);
    
    const loadContent = async () => {
      try {
        let base64Content: string | null = null;
        const knownSize = typeof node.size === 'number' ? node.size : null;
        if (knownSize && knownSize > LARGE_FILE_THRESHOLD) {
          setContentError(t('learningHub:file.previewTooLarge', '文件过大，无法预览'));
          setContentLoading(false);
          return;
        }

        const loadFromVfs = async () => {
          const result = await invoke<{ content: string | null; found: boolean }>('vfs_get_attachment_content', {
            attachmentId: node.id,
          });
          if (!isMounted) return null;

          if (result?.found && result?.content) {
            const estimatedSize = estimateBase64Size(result.content);
            if (estimatedSize > LARGE_FILE_THRESHOLD) {
              setContentError(t('learningHub:file.previewTooLarge', '文件过大，无法预览'));
              setContentLoading(false);
              return null;
            }
            return result.content;
          }
          return null;
        };
        
        // ★ 优先使用可用的 filePath 读取本地文件，失败则回退到 VFS
        if (effectiveFilePath) {
          try {
            const fileSize = effectiveFileSize ?? await invoke<number>('get_file_size', { path: effectiveFilePath });
            if (!isMounted) return;
            if (fileSize > LARGE_FILE_THRESHOLD) {
              setContentError(t('learningHub:file.previewTooLarge', '文件过大，无法预览'));
              setContentLoading(false);
              return;
            }

            const bytes = await invoke<number[]>('read_file_bytes', { path: effectiveFilePath });
            if (!isMounted) return;
            // 转换为 base64（分块，避免大数组字符串拼接造成卡顿）
            base64Content = uint8ArrayToBase64(new Uint8Array(bytes));
          } catch (err: unknown) {
            console.warn('[TextbookContentView] Failed to read filePath, fallback to VFS:', err);
            if (!isMounted) return;
            base64Content = await loadFromVfs();
          }
        } else {
          base64Content = await loadFromVfs();
        }
        
        if (base64Content) {
          setFileContent(base64Content);
          setContentLoading(false);
        } else {
          setContentError(t('learningHub:file.contentNotFound', '未找到文件内容 (id: {{id}})', { id: node.id }));
          setContentLoading(false);
        }
      } catch (err: unknown) {
        console.error('[TextbookContentView] Failed to load file:', err);
        if (isMounted) {
          setContentError(err instanceof Error ? err.message : t('learningHub:file.loadFailed', '加载文件失败'));
          setContentLoading(false);
        }
      }
    };
    
    void loadContent();
    
    return () => {
      isMounted = false;
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [needsFileContent, effectiveFilePath, effectiveFileSize, node.id, node.size, t, contentRetryCount]);
  
  // 从 node.metadata 提取阅读进度
  const readingProgress = useMemo<ReadingProgress | undefined>(() => {
    const progress = node.metadata?.readingProgress as { page?: number; lastReadAt?: number } | undefined;
    if (progress && typeof progress.page === 'number' && progress.page > 0) {
      return {
        page: progress.page,
        lastReadAt: progress.lastReadAt,
      };
    }
    return undefined;
  }, [node.metadata?.readingProgress]);
  
  // 从 node.metadata 提取书签列表
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([]);
  
  // 书签保存的防抖引用
  const saveBookmarksTimerRef = useRef<number | null>(null);
  
  // 初始化书签数据
  useEffect(() => {
    const savedBookmarks = node.metadata?.bookmarks as Bookmark[] | undefined;
    if (savedBookmarks && Array.isArray(savedBookmarks)) {
      setBookmarks(savedBookmarks);
    } else {
      setBookmarks([]);
    }
  }, [node.metadata?.bookmarks]);
  
  // 保存阅读进度到 DSTU
  const handleProgressChange = useCallback((progress: ReadingProgress) => {
    // ★ 记录 pending 值，供 unmount flush 使用
    pendingProgressRef.current = progress;

    // 防抖：清理之前的定时器
    if (saveProgressTimerRef.current) {
      window.clearTimeout(saveProgressTimerRef.current);
    }
    
    // 延迟保存，避免频繁写入
    saveProgressTimerRef.current = window.setTimeout(async () => {
      saveProgressTimerRef.current = null;
      pendingProgressRef.current = null; // 已提交，清除 pending
      
      // 构建新的元数据（保留原有字段）
      const newMetadata = {
        ...nodeMetadataRef.current,
        readingProgress: {
          page: progress.page,
          lastReadAt: progress.lastReadAt,
        },
      };

      // 通过 DSTU 保存元数据 (Result模式)
      const result = await dstu.setMetadata(nodePathRef.current, newMetadata);
      if (!result.ok) {
        reportError(result.error, '保存阅读进度');
        console.warn('[TextbookContentView] Failed to save reading progress:', result.error.toUserMessage());
      }
    }, 2000); // 2秒防抖，避免频繁保存
  }, []);
  
  // 保存书签到后端（通过 VFS API）
  const handleBookmarksChange = useCallback((newBookmarks: Bookmark[]) => {
    // 更新本地状态
    setBookmarks(newBookmarks);
    // ★ 记录 pending 值，供 unmount flush 使用
    pendingBookmarksRef.current = newBookmarks;
    
    // 防抖：清理之前的定时器
    if (saveBookmarksTimerRef.current) {
      window.clearTimeout(saveBookmarksTimerRef.current);
    }
    
    // 延迟保存，避免频繁写入
    saveBookmarksTimerRef.current = window.setTimeout(async () => {
      saveBookmarksTimerRef.current = null;
      pendingBookmarksRef.current = null; // 已提交，清除 pending
      
      try {
        const fileId = nodeIdRef.current;
        
        // 调用后端 API 保存书签
        await vfsFileApi.updateBookmarks(fileId, newBookmarks);
        
        // 同时更新 DSTU 元数据，保持数据一致性
        const newMetadata = {
          ...nodeMetadataRef.current,
          bookmarks: newBookmarks,
        };
        await dstu.setMetadata(nodePathRef.current, newMetadata);
      } catch (err: unknown) {
        console.error('[TextbookContentView] Failed to save bookmarks:', err);
        showGlobalNotification('error', t('textbook:bookmarkSaveFailed', '书签保存失败'));
      }
    }, 1000); // 1秒防抖
  }, [t]);
  
  // ★ 清理定时器并 flush 未保存的数据（防止卸载丢失）
  React.useEffect(() => {
    return () => {
      // 清除定时器
      if (saveProgressTimerRef.current) {
        window.clearTimeout(saveProgressTimerRef.current);
        saveProgressTimerRef.current = null;
      }
      if (saveBookmarksTimerRef.current) {
        window.clearTimeout(saveBookmarksTimerRef.current);
        saveBookmarksTimerRef.current = null;
      }

      // ★ 合并 flush 未保存的阅读进度和书签（单次 setMetadata，避免竞态覆盖）
      const pendingProgress = pendingProgressRef.current;
      const pendingBookmarks = pendingBookmarksRef.current;
      pendingProgressRef.current = null;
      pendingBookmarksRef.current = null;

      if (pendingProgress || pendingBookmarks) {
        const mergedMetadata = { ...nodeMetadataRef.current };
        if (pendingProgress) {
          mergedMetadata.readingProgress = {
            page: pendingProgress.page,
            lastReadAt: pendingProgress.lastReadAt,
          };
        }
        if (pendingBookmarks) {
          mergedMetadata.bookmarks = pendingBookmarks;
          // 书签同时保存到 VFS API
          void vfsFileApi.updateBookmarks(nodeIdRef.current, pendingBookmarks);
        }
        dstu.setMetadata(nodePathRef.current, mergedMetadata).then(result => {
          if (!result.ok) {
            reportError(result.error, '保存未持久化的阅读进度/书签');
            console.warn('[TextbookContentView] flush setMetadata failed:', result.error.toUserMessage());
          }
        }).catch(err => {
          console.error('[TextbookContentView] flush setMetadata error:', err);
        });
      }
    };
  }, []);

  // ======== OCR 检测与处理流水线 ========

  /** 加载 OCR 文本（复用函数，避免 useEffect 闭包陷阱） */
  const loadOcrText = useCallback(async () => {
    if (!node.sourceId && !node.resourceId) return;
    const resourceId = node.resourceId || node.sourceId;
    try {
      const ocrInfo = await invoke<{
        hasOcr: boolean;
        ocrText: string | null;
      }>('vfs_get_resource_ocr_info', { resourceId });
      if (ocrInfo.hasOcr && ocrInfo.ocrText) {
        setOcrTextContent(ocrInfo.ocrText);
      }
    } catch (err: unknown) {
      console.warn('[TextbookContentView] Failed to load OCR text:', err);
    }
  }, [node.resourceId, node.sourceId]);

  /** 检测 OCR 配置并启动处理流水线 */
  useEffect(() => {
    if (!isPdf || !node.sourceId || !node.sourceId.startsWith('tb_')) return;

    let cancelled = false;

    const initOcr = async () => {
      // 1. 检查 OCR 是否已配置
      let configured = false;
      try {
        const avail = await invoke<{ configured: boolean; modelName?: string }>('check_ocr_availability');
        if (cancelled) return;
        setOcrAvailability(avail);
        configured = avail.configured;
      } catch {
        if (!cancelled) setOcrAvailability({ configured: false });
        return;
      }

      if (!configured) return;

      // 2. 先查询是否已有 OCR 文本（之前已完成处理）
      try {
        const ocrInfo = await invoke<{ hasOcr: boolean; ocrText: string | null }>(
          'vfs_get_resource_ocr_info',
          { resourceId: node.resourceId || node.sourceId },
        );
        if (cancelled) return;
        if (ocrInfo.hasOcr && ocrInfo.ocrText) {
          setOcrTextContent(ocrInfo.ocrText);
          return; // 已有 OCR 文本，无需启动流水线
        }
      } catch {
        // 资源不存在或其他错误，继续启动流水线
      }

      // 3. 检查当前处理状态（可能正在后台处理中）
      try {
        const status = await invoke<{
          stage: string;
          progress: { readyModes: string[] };
        } | null>('vfs_get_pdf_processing_status', { fileId: node.sourceId });
        if (cancelled) return;

        // 已完成且包含 OCR 模式则跳过
        if (status && (status.stage === 'completed' || status.stage === 'completed_with_issues')
          && status.progress.readyModes.includes('ocr')) {
          return;
        }

        // 未完成或出错 — 启动流水线（从 OCR 阶段开始）
        if (!status || status.stage === 'error' || status.stage === 'pending') {
          await invoke('vfs_start_pdf_processing', { fileId: node.sourceId });
        }
      } catch {
        // 状态检查失败，尝试直接启动
        try {
          await invoke('vfs_start_pdf_processing', { fileId: node.sourceId });
        } catch { /* 可能已在运行中，忽略 */ }
      }
    };

    void initOcr();
    return () => { cancelled = true; };
  }, [isPdf, node.sourceId, node.resourceId]);

  /** 监听处理状态变化：处理完成时加载 OCR 文本 */
  useEffect(() => {
    if (!processingStatus || ocrTextContent) return;

    const { stage, readyModes } = processingStatus;
    if ((stage === 'completed' || stage === 'completed_with_issues') && readyModes.includes('ocr')) {
      void loadOcrText();
    }
  }, [processingStatus, ocrTextContent, loadOcrText]);

  // ★ 非 PDF 文件重试加载
  const retryContentLoad = useCallback(() => {
    setFileContent(null);
    setContentError(null);
    setContentRetryCount((c) => c + 1);
  }, []);

  // ★ PDF 初始态 spinner 超时检测（10 秒后显示提示 + 重试按钮，避免无限旋转）
  useEffect(() => {
    if (!isPdf || effectiveFilePath || pdfFile || pdfLoading || pdfError) {
      setPdfInitTimedOut(false);
      return;
    }
    const timer = window.setTimeout(() => {
      setPdfInitTimedOut(true);
    }, 10_000);
    return () => window.clearTimeout(timer);
  }, [isPdf, effectiveFilePath, pdfFile, pdfLoading, pdfError]);

  // ★ 移除 filePath 为空时的硬性错误，改为在内容加载失败时显示错误
  // 因为从 attachments 迁移的文件可能没有 filePath，但可以通过 vfs_get_attachment_content 获取内容
  
  // PDF 文件：如果没有 filePath 且没有 pdfFile，显示加载中或错误
  if (isPdf && !effectiveFilePath && !pdfFile) {
    if (pdfLoading) {
      return (
        <div className="flex flex-col items-center justify-center h-full gap-4">
          <CircleNotch className="h-8 w-8 animate-spin text-primary" />
          {isPdfLargeFile && (
            <p className="text-sm text-muted-foreground">
              {t('textbook:loading.largeFile', '正在加载大文件，请稍候...')}
            </p>
          )}
        </div>
      );
    }
    if (pdfError) {
      return (
        <div className="flex flex-col items-center justify-center h-full gap-4">
          <WarningCircle className="w-12 h-12 text-destructive" />
          <p className="text-destructive text-center">{pdfError}</p>
          <NotionButton
            variant="default"
            size="sm"
            onClick={retryPdfLoad}
          >
            <ArrowClockwise className="h-3.5 w-3.5 mr-1.5" />
            {t('common:retry', '重试')}
          </NotionButton>
        </div>
      );
    }
    // 初始状态，等待加载（超时后显示提示 + 重试按钮）
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <CircleNotch className="h-8 w-8 animate-spin text-primary" />
        {pdfInitTimedOut && (
          <>
            <p className="text-sm text-muted-foreground text-center">
              {t('textbook:loading.timeout', '加载时间较长，可能遇到问题')}
            </p>
            <NotionButton
              variant="default"
              size="sm"
              onClick={retryPdfLoad}
            >
              <ArrowClockwise className="h-3.5 w-3.5 mr-1.5" />
              {t('common:retry', '重试')}
            </NotionButton>
          </>
        )}
      </div>
    );
  }
  
  // 加载中状态
  const LoadingSpinner = () => (
    <div className="flex items-center justify-center h-full">
      <CircleNotch className="h-8 w-8 animate-spin text-primary" />
    </div>
  );
  
  // 错误状态
  if (contentError) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <WarningCircle className="w-12 h-12 text-destructive" />
        <p className="text-destructive text-center">{contentError}</p>
        <NotionButton
          variant="default"
          size="sm"
          onClick={retryContentLoad}
        >
          <ArrowClockwise className="h-3.5 w-3.5 mr-1.5" />
          {t('common:retry', '重试')}
        </NotionButton>
      </div>
    );
  }
  
  const showRichToolbar = (isDocx || isXlsx || isPptx) && !!fileContent && !!previewType;
  const renderRichDocumentPreview = (
    kind: 'docx' | 'xlsx' | 'pptx',
    content: string
  ) => (
    <RichDocumentPreview
      kind={kind}
      base64Content={content}
      fileName={node.name}
      showToolbar={showRichToolbar}
      previewType={toToolbarPreviewType(previewType)}
      zoomScale={zoomScale}
      fontScale={fontScale}
      onZoomChange={setZoomScale}
      onFontChange={setFontScale}
      onZoomReset={resetZoom}
      onFontReset={resetFont}
      fallback={<LoadingSpinner />}
      rootClassName="bg-background"
    />
  );

  // DOCX 预览
  if (isDocx) {
    if (contentLoading || !fileContent) {
      return <LoadingSpinner />;
    }
    return renderRichDocumentPreview('docx', fileContent);
  }
  
  // XLSX 预览
  if (isXlsx) {
    if (contentLoading || !fileContent) {
      return <LoadingSpinner />;
    }
    return renderRichDocumentPreview('xlsx', fileContent);
  }
  
  // PPTX 预览
  if (isPptx) {
    if (contentLoading || !fileContent) {
      return <LoadingSpinner />;
    }
    return renderRichDocumentPreview('pptx', fileContent);
  }

  // 纯文本预览
  if (isText) {
    if (contentLoading || !fileContent) {
      return <LoadingSpinner />;
    }
    const textContent = decodeBase64ToText(fileContent) ?? fileContent;
    return (
      <div className="flex flex-col h-full bg-background overflow-hidden">
        <CustomScrollArea className="flex-1">
          <pre className="whitespace-pre-wrap text-sm p-4 m-0 min-h-full font-mono">
            {textContent}
          </pre>
        </CustomScrollArea>
      </div>
    );
  }
  
  // 不支持预览的文件类型（如 PPTX）
  if (isUnsupported) {
    // 从文件名获取扩展名
    const ext = node.name.split('.').pop()?.toUpperCase() || '';
    return (
      <div className="flex flex-col items-center justify-center h-full gap-4">
        <FileText className="w-16 h-16 text-muted-foreground" />
        <div className="text-center space-y-2">
          <p className="text-lg font-medium text-foreground">{node.name}</p>
          <p className="text-muted-foreground">
            {t('learningHub:textbook.unsupportedPreview', { ext })}
          </p>
        </div>
      </div>
    );
  }

  // ======== OCR 状态栏渲染函数 ========

  const renderOcrStatusBar = () => {
    if (ocrAvailability === null) return null;

    // ① OCR 未配置 — 提示用户前往设置
    if (!ocrAvailability.configured) {
      return (
        <div className="flex items-center gap-2 px-4 py-2 bg-amber-50 dark:bg-amber-950/20 border-b border-amber-200 dark:border-amber-800/40">
          <Scan className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0" />
          <span className="text-xs text-amber-700 dark:text-amber-300 leading-relaxed">
            {t('textbook:ocr.notConfigured', '这是一份 PDF 文件，如需 OCR 文本提取，请前往「设置 > OCR 引擎」配置模型')}
          </span>
        </div>
      );
    }

    // ② 处理中 — 显示进度条
    if (processingStatus
      && processingStatus.stage !== 'completed'
      && processingStatus.stage !== 'completed_with_issues'
      && processingStatus.stage !== 'error'
    ) {
      const hint = getProcessingHint(processingStatus);
      const progressPercent = Math.min(100, Math.max(0, processingStatus.percent));
      const currentPage = processingStatus.currentPage;
      const totalPages = processingStatus.totalPages;
      return (
        <div className="px-4 py-2.5 border-b border-border bg-muted/30">
          <div className="flex items-center gap-2 mb-1.5">
            <CircleNotch className="h-3.5 w-3.5 animate-spin text-primary shrink-0" />
            <span className="text-xs text-muted-foreground">{hint}</span>
            {(typeof currentPage === 'number' && typeof totalPages === 'number') && (
              <span className="text-xs text-muted-foreground ml-auto">
                {currentPage}/{totalPages}
              </span>
            )}
          </div>
          <div className="w-full h-1.5 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-primary rounded-full transition-all duration-300 ease-out"
              style={{ width: `${progressPercent}%` }}
            />
          </div>
        </div>
      );
    }

    // ③ OCR 文本就绪（completed / completed_with_issues）— 显示切换开关
    if ((processingStatus?.stage === 'completed' || processingStatus?.stage === 'completed_with_issues' || ocrTextContent)
      && ocrTextContent
    ) {
      return (
        <div className="flex items-center justify-between px-4 py-1.5 border-b border-border bg-muted/20">
          <div className="flex items-center gap-2">
            {processingStatus?.stage === 'completed_with_issues' && (
              <WarningCircle className="h-3.5 w-3.5 text-amber-500 shrink-0" />
            )}
            <span className="text-xs text-muted-foreground">
              {t('textbook:ocr.ready', 'OCR 文本已就绪')}
            </span>
          </div>
          <div className="flex items-center bg-muted rounded-lg p-0.5 gap-0.5">
            <button
              type="button"
              className={`px-2.5 py-1 rounded-md text-xs transition-colors ${
                !showOcrText
                  ? 'bg-background shadow-sm font-medium text-foreground'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
              onClick={() => setShowOcrText(false)}
            >
              {t('textbook:ocr.imageView', '图像')}
            </button>
            <button
              type="button"
              className={`px-2.5 py-1 rounded-md text-xs transition-colors ${
                showOcrText
                  ? 'bg-background shadow-sm font-medium text-foreground'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
              onClick={() => setShowOcrText(true)}
            >
              {t('textbook:ocr.textView', 'OCR 文本')}
            </button>
          </div>
        </div>
      );
    }

    return null;
  };

  // PDF 预览
  return (
    <div className="flex flex-col h-full bg-background">
      {isPdf && renderOcrStatusBar()}
      {showOcrText && ocrTextContent ? (
        <div className="flex-1 overflow-hidden">
          <CustomScrollArea className="h-full">
            <div className="p-4">
              <pre className="whitespace-pre-wrap text-sm leading-relaxed font-sans text-foreground/90">
                {ocrTextContent}
              </pre>
            </div>
          </CustomScrollArea>
        </div>
      ) : (
        <TextbookPdfViewer
          file={pdfFile}
          // ★ PDF-403 修复：不传 filePath 给教材 PDF，避免触发 pdfstream:// 协议导致 403
          // file 优先于 filePath（TextbookPdfViewer 内部逻辑：file 存在时使用 Blob URL，
          // 仅 file 为 null 时才回退到 filePath 的 pdfstream://）
          filePath={''}
          fileName={node.name}
          selectedPages={selectedPages}
          onPageSelectionChange={handlePageSelectionChange}
          onExportSelectedPages={handleExportSelectedPages}
          focusRequest={focusRequest}
          onFocusHandled={handleFocusHandled}
          readingProgress={readingProgress}
          onProgressChange={handleProgressChange}
          resourcePath={node.path}
          bookmarks={bookmarks}
          onBookmarksChange={handleBookmarksChange}
        />
      )}
    </div>
  );
};

const TextbookContentView: React.FC<ContentViewProps> = (props) => (
  <PreviewProvider>
    <TextbookContentViewInner {...props} />
  </PreviewProvider>
);

export default TextbookContentView;
