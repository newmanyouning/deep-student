/**
 * NoteContentView - 笔记内容视图
 *
 * 统一应用面板中的笔记编辑视图。
 * 通过 DSTU 协议获取笔记数据，直接传递给编辑器组件。
 * 
 * 改造后移除了对 NotesProvider/NotesContext 的依赖，
 * 所有数据通过 DSTU 节点和 API 获取。
 */

import React, { useEffect, useState, useCallback, useMemo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { CircleNotch, WarningCircle, ArrowCounterClockwise } from '@phosphor-icons/react';
import { NotionButton } from '@/components/ui/NotionButton';
import { NotesCrepeEditor } from '@/features/notes/NotesCrepeEditor';
import { NotesContextPanel } from '@/features/notes/NotesContextPanel';
import { MarkdownPreview } from '@/features/notes/preview/MarkdownPreview';
import { Eye, PencilSimple } from '@phosphor-icons/react';
import { reportError, type VfsError, VfsErrorCode } from '@/shared/result';
import { dstu } from '@/dstu';
import { useSystemStatusStore } from '@/stores/systemStatusStore';
import { showGlobalNotification } from '@/components/UnifiedNotification';
import type { ContentViewProps } from '../UnifiedAppPanel';
import { PanelGroup, Panel, PanelResizeHandle, type ImperativePanelHandle } from 'react-resizable-panels';
import { cn } from '@/lib/utils';
import { useMediaQuery } from '@/hooks/useMediaQuery';
import { DotsSixVertical, SidebarSimple } from '@phosphor-icons/react';
import { CommonTooltip } from '@/components/shared/CommonTooltip';
import { COMMAND_EVENTS, useCommandEvents } from '@/command-palette/hooks/useCommandEvents';
import type { CrepeEditorApi } from '@/components/crepe';
import {
  consumePendingOcrPage,
  getLastOcrPage,
  dispatchOcrPageSync,
  OCR_PAGE_SYNC_EVENT,
  saveLastOcrPage,
  type OcrPageSyncEvent,
} from '@/features/learning-hub/ocrPageSync';

/**
 * 笔记内容视图
 * 
 * 直接使用 DSTU 协议获取和保存笔记数据，
 * 不再依赖 NotesProvider/NotesContext。
 */
const NoteContentView: React.FC<ContentViewProps> = ({
  node,
  onClose,
  onTitleChange,
  readOnly = false,
  isActive = false,
}) => {
  const { t } = useTranslation(['notes', 'common']);
  const isSmallScreen = useMediaQuery("(max-width: 768px)");

  // ========== 右侧面板状态 ==========
  const [rightPanelVisible, setRightPanelVisible] = useState(true);
  const rightPanelRef = useRef<ImperativePanelHandle>(null);

  const toggleRightPanel = useCallback(() => {
    const panel = rightPanelRef.current;
    if (!panel) return;
    if (rightPanelVisible) {
      panel.collapse();
    } else {
      panel.expand();
    }
  }, [rightPanelVisible]);

  // ========== 状态 ==========
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<VfsError | null>(null);
  
  // 笔记内容状态
  // 🔧 修复：使用 null 表示"未加载"，空字符串表示"已加载但内容为空"
  const [content, setContent] = useState<string | null>(null);
  const [title, setTitle] = useState<string>(node.name || '');
  const [tags, setTags] = useState<string[]>((node.metadata?.tags as string[]) || []);
  const editorApiRef = useRef<CrepeEditorApi | null>(null);
  
  // 🔧 追踪当前加载的笔记 ID，用于防止竞态条件
  const loadingNoteIdRef = React.useRef<string | null>(null);

  // ★ 判断是否为 OCR 笔记（tag 中包含 'ocr'）
  const isOcrNote = (node.metadata?.tags as string[])?.includes('ocr') ?? false;
  // ★ 内容大小阈值：超过 100KB 默认用预览模式
  const isLargeContent = (content?.length ?? 0) > 100_000;
  const shouldDefaultPreview = isOcrNote || isLargeContent;

  // ★ 提取 OCR 源文件 ID（从 tag 中解析 source:<file_id>）
  const ocrSourceId = useMemo(() => {
    if (!isOcrNote) return null;
    const tags = (node.metadata?.tags as string[]) || [];
    const sourceTag = tags.find(t => t.startsWith('source:'));
    return sourceTag ? sourceTag.slice(7) : null; // 去掉 "source:" 前缀
  }, [isOcrNote, node.metadata?.tags]);

  // ★ OCR 逐页阅读：页码状态（优先用 PDF 传递的页码，回退 tag 提取，再回退持久化记录）
  const initialOcrPage = useMemo(() => {
    if (!isOcrNote || !ocrSourceId) return 1;
    // 1) PDF "打开笔记" 传来的页码
    const pending = consumePendingOcrPage(ocrSourceId);
    if (pending !== null && pending > 0) return pending;
    // 2) 笔记 tag 中的 page:<N>
    const tags = (node.metadata?.tags as string[]) || [];
    const pageTag = tags.find(t => t.startsWith('page:'));
    const pageNum = pageTag ? parseInt(pageTag.slice(5), 10) : 0;
    if (pageNum > 0) return pageNum;
    // 3) 最后打开页码（sessionStorage）
    return getLastOcrPage(ocrSourceId) ?? 1;
  }, [isOcrNote, ocrSourceId, node.metadata?.tags]);

  const [ocrPageNumber, setOcrPageNumber] = useState(initialOcrPage);
  const [ocrPageMd, setOcrPageMd] = useState<string | null>(null);
  const [ocrPageLoading, setOcrPageLoading] = useState(false);
  const [ocrTotalPages, setOcrTotalPages] = useState<number | null>(null);

  // ★ 已编辑页面缓存：key=pageNumber, value=编辑后的内容
  const editedPagesRef = useRef<Map<number, string>>(new Map());

  // ★ 加载指定页的 OCR MD（优先使用已编辑的内容）
  const loadOcrPage = useCallback(async (pageNum: number) => {
    if (!ocrSourceId) return;
    // 1) 优先使用已编辑缓存
    const edited = editedPagesRef.current.get(pageNum);
    if (edited !== undefined) {
      setOcrPageMd(edited);
      return;
    }
    // 2) 加载 OCR 原始数据
    setOcrPageLoading(true);
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const md = await invoke<string>('get_ocr_page_md', {
        fileId: ocrSourceId,
        pageIndex: pageNum - 1,
      });
      setOcrPageMd(md);
    } catch (err: any) {
      // 从错误消息中提取总页数（如 "Page 5 not found (total: 300)"）
      const msg = String(err?.message || err || '');
      const totalMatch = msg.match(/total:\s*(\d+)/);
      if (totalMatch) setOcrTotalPages(parseInt(totalMatch[1], 10));
      setOcrPageMd(null);
    } finally {
      setOcrPageLoading(false);
    }
  }, [ocrSourceId]);

  // ★ OCR 笔记：初始化时加载对应页码
  useEffect(() => {
    if (isOcrNote && ocrSourceId) {
      setOcrPageNumber(initialOcrPage);
      void loadOcrPage(initialOcrPage);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOcrNote, ocrSourceId, node.id]);

  /** ★★ MD → PDF 页码同步：MD 翻页时通知 PDF 查看器 */
  // 使用 ref 防止 onTitleChange 每次渲染变化触发无限循环
  const onTitleChangeRef = useRef(onTitleChange);
  onTitleChangeRef.current = onTitleChange;
  useEffect(() => {
    if (!isOcrNote || !ocrSourceId) return;
    dispatchOcrPageSync({ fileId: ocrSourceId, pageNumber: ocrPageNumber, source: 'md' });
    saveLastOcrPage(ocrSourceId, ocrPageNumber);
    // ★ 更新标签页标题为当前页
    onTitleChangeRef.current?.(`${node.name} - 第 ${ocrPageNumber} 页`);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOcrNote, ocrSourceId, ocrPageNumber, node.name]);

  /** ★★ PDF → MD 页码同步：PDF 翻页时同步 MD 页码 */
  useEffect(() => {
    if (!isOcrNote || !ocrSourceId) return;
    const handler = (e: Event) => {
      const detail = (e as CustomEvent<OcrPageSyncEvent>).detail;
      if (!detail || detail.source !== 'pdf') return;
      if (detail.fileId !== ocrSourceId) return;
      if (detail.pageNumber > 0) {
        setOcrPageNumber(detail.pageNumber);
        void loadOcrPage(detail.pageNumber);
      }
    };
    window.addEventListener(OCR_PAGE_SYNC_EVENT, handler);
    return () => window.removeEventListener(OCR_PAGE_SYNC_EVENT, handler);
  }, [isOcrNote, ocrSourceId, loadOcrPage]);

  // ★ 视图模式：preview（轻量 react-markdown）vs editor（ProseMirror/Milkdown）
  const [viewMode, setViewMode] = useState<'preview' | 'editor'>(
    shouldDefaultPreview ? 'preview' : 'editor'
  );
  // 当切换到不同的 OCR 笔记或超大笔记时，重置为预览模式
  useEffect(() => {
    setViewMode(shouldDefaultPreview ? 'preview' : 'editor');
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [node.id, isOcrNote]);

  const noteId = node.id;

  // ★ 判断是否为旧版巨型 OCR 笔记（无 page:<N> tag 的原始全量笔记）
  const isOldMonolithicOcr = isOcrNote && !(node.metadata?.tags as string[])?.some(t => t.startsWith('page:'));

  // ========== 加载笔记内容（提取为可复用函数，支持重试） ==========
  const loadNoteContent = useCallback(async () => {
    // ★ 仅跳过旧版巨型 OCR 笔记（无 page:<N> tag），逐页笔记正常加载
    if (isOldMonolithicOcr) {
      setIsLoading(false);
      return;
    }
    // 🔧 修复：记录当前加载的笔记 ID
    const currentNoteId = node.id;
    loadingNoteIdRef.current = currentNoteId;
    
    setIsLoading(true);
    setError(null);
    // ★ 优化体验：不再粗暴地 setContent(null)，保留旧内容（Stale-While-Revalidate），
    // 配合顶部的透明 Loading 指示器，实现无缝切换

    // 通过 DSTU 获取笔记内容
    const result = await dstu.getContent(node.path);

    // 🔧 修复：检查是否仍在加载同一笔记（防止竞态条件）
    if (loadingNoteIdRef.current !== currentNoteId) {
      return;
    }

    if (!result.ok) {
      console.error('[NoteContentView] ❌ 加载笔记内容失败:', result.error);
      if (result.error.code !== VfsErrorCode.NOT_FOUND) {
        reportError(result.error, '加载笔记内容');
      }
      setError(result.error);
      setIsLoading(false);
      return;
    }

    const contentStr = typeof result.value === 'string' ? result.value : '';
    
    setContent(contentStr);
    setTitle(node.name || '');
    // 重新加载时同步最新的 tags（node 可能已更新）
    setTags((node.metadata?.tags as string[]) || []);
    setIsLoading(false);
  }, [node.id, node.path, node.name, isOldMonolithicOcr]);

  useEffect(() => {
    void loadNoteContent();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [node.id, isOldMonolithicOcr]);

  // ========== 保存回调 ==========
  // 内容保存
  const handleSave = useCallback(async (newContent: string) => {
    if (readOnly) return;
    // S-003: 维护模式拦截，防止 Learning Hub 入口绕过写入
    if (useSystemStatusStore.getState().maintenanceMode) {
      showGlobalNotification('warning', t('common:maintenance.blocked_note_save', '维护模式下无法保存笔记'));
      return;
    }
    const result = await dstu.update(node.path, newContent, node.type);
    if (!result.ok) {
      console.error('[NoteContentView] ❌ 保存笔记失败:', result.error);
      reportError(result.error, '保存笔记');
      throw new Error(result.error.toUserMessage());
    }
    setContent(newContent);
    // ★ OCR 笔记：保存后同步更新预览的逐页 MD，确保切回预览显示变更
    if (isOcrNote && ocrSourceId) {
      setOcrPageMd(newContent);
      // 缓存编辑过的页面，翻页回来时优先显示已编辑内容
      editedPagesRef.current.set(ocrPageNumber, newContent);
    }
  }, [node.path, node.type, readOnly, t, isOcrNote, ocrSourceId, ocrPageNumber]);

  // ★★ 手动保存 + 放弃编辑
  const [isSaving, setIsSaving] = useState(false);
  const [lastSavedAt, setLastSavedAt] = useState<Date | null>(null);

  const handleManualSave = useCallback(async () => {
    const editor = editorApiRef.current;
    if (!editor || editor.isReadonly()) return;
    setIsSaving(true);
    try {
      const markdown = editor.getMarkdown();
      await handleSave(markdown);
      setLastSavedAt(new Date());
      showGlobalNotification('success', t('notes:actions.save_success', '保存成功'));
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : t('notes:actions.save_failed', '保存失败');
      showGlobalNotification('error', msg);
    } finally {
      setIsSaving(false);
    }
  }, [handleSave, t]);

  const handleDiscardEdit = useCallback(() => {
    setViewMode('preview');
    setLastSavedAt(null);
  }, []);
  const handleTitleChange = useCallback(async (newTitle: string) => {
    if (readOnly) return;
    // S-003: 维护模式拦截
    if (useSystemStatusStore.getState().maintenanceMode) {
      showGlobalNotification('warning', t('common:maintenance.blocked_note_save', '维护模式下无法保存笔记'));
      return;
    }
    const result = await dstu.setMetadata(node.path, { title: newTitle });
    if (!result.ok) {
      console.error('[NoteContentView] Failed to update title:', result.error);
      reportError(result.error, '更新标题');
      throw new Error(result.error.toUserMessage());
    }
    setTitle(newTitle);
    // 通知父级面板标题已更新
    onTitleChange?.(newTitle);
  }, [node.path, readOnly, onTitleChange, t]);

  // 标签变更
  const handleTagsChange = useCallback(async (newTags: string[]) => {
    if (readOnly) return;
    const result = await dstu.setMetadata(node.path, { tags: newTags });
    if (!result.ok) {
      console.error('[NoteContentView] Failed to update tags:', result.error);
      reportError(result.error, '更新标签');
      throw new Error(result.error.toUserMessage());
    }
    setTags(newTags);
  }, [node.path, readOnly]);

  useCommandEvents(
    {
      [COMMAND_EVENTS.NOTES_FORCE_SAVE]: () => {
        if (!isActive || readOnly) return;
        const editor = editorApiRef.current;
        if (!editor || editor.isReadonly()) return;
        void handleSave(editor.getMarkdown())
          .then(() => {
            showGlobalNotification('success', t('notes:actions.save_success', '保存成功'));
          })
          .catch((err) => {
            const msg = err instanceof Error ? err.message : t('notes:actions.save_failed', '保存失败');
            showGlobalNotification('error', msg);
          });
      },
      [COMMAND_EVENTS.NOTES_TOGGLE_OUTLINE]: () => {
        if (!isActive || isSmallScreen) return;
        toggleRightPanel();
      },
      [COMMAND_EVENTS.NOTES_INSERT_MATH]: () => {
        if (!isActive || readOnly || editorApiRef.current?.isReadonly()) return;
        editorApiRef.current?.insertAtCursor('\n$$\n\n$$\n');
      },
      [COMMAND_EVENTS.NOTES_INSERT_TABLE]: () => {
        if (!isActive || readOnly || editorApiRef.current?.isReadonly()) return;
        editorApiRef.current?.insertTable();
      },
      [COMMAND_EVENTS.NOTES_INSERT_CODEBLOCK]: () => {
        if (!isActive || readOnly || editorApiRef.current?.isReadonly()) return;
        editorApiRef.current?.insertCodeBlock();
      },
      [COMMAND_EVENTS.NOTES_INSERT_LINK]: () => {
        if (!isActive || readOnly || editorApiRef.current?.isReadonly()) return;
        editorApiRef.current?.insertLink('https://', '');
      },
      [COMMAND_EVENTS.NOTES_INSERT_IMAGE]: () => {
        if (!isActive || readOnly || editorApiRef.current?.isReadonly()) return;
        editorApiRef.current?.insertImage('https://', '');
      },
      [COMMAND_EVENTS.AI_CONTINUE_WRITING]: () => {
        if (!isActive || readOnly || editorApiRef.current?.isReadonly()) return;
        showGlobalNotification('info', t('notes:ai.continue_not_available', 'AI 续写命令暂不可用，请使用聊天面板发起编辑。'));
      },
    },
    true
  );

  // ========== 渲染 ==========
  // 🔧 优化：Stale-While-Revalidate
  // 当有旧内容 (content !== null) 但正在加载新内容 (isLoading) 时，不要白屏，而是保留旧内容+顶部透明进度条
  
  if (isLoading && content === null) {
    return (
      <div className="flex items-center justify-center h-full">
        <CircleNotch size={24} className="animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">
          {t('common:loading', '加载中...')}
        </span>
      </div>
    );
  }

  if (error) {
    const message = error.code === VfsErrorCode.NOT_FOUND
      ? t('notes:error.notFound', '笔记不存在或已被删除')
      : error.toUserMessage();
    return (
      <div className="flex flex-col items-center justify-center h-full">
        <WarningCircle size={32} className="text-destructive mb-2" />
        <span className="text-destructive">{message}</span>
        <div className="flex gap-2 mt-3">
          <NotionButton variant="primary" onClick={() => loadNoteContent()}>
            {t('common:retry', '重试')}
          </NotionButton>
          {onClose && (
            <NotionButton variant="ghost" onClick={onClose}>
              {t('common:close', '关闭')}
            </NotionButton>
          )}
        </div>
      </div>
    );
  }
  
  return (
    <div className="flex flex-col h-full bg-background relative overflow-hidden">
      {isLoading && content !== null && (
        <div className="absolute top-0 left-0 right-0 h-1 bg-primary/20 z-50 overflow-hidden">
          <div className="h-full bg-primary animate-[indeterminate_1.5s_infinite_linear]" />
        </div>
      )}
      {/* 右侧栏开关按钮 - 置于 PanelGroup 之上，避免被编辑器 sticky header 遮挡 */}
      {!isSmallScreen && (
        <div className="flex items-center justify-end px-2 py-0.5 flex-shrink-0">
          <CommonTooltip
            content={rightPanelVisible ? t('notes:context.collapse_panel', '收起侧边栏') : t('notes:context.expand_panel', '展开侧边栏')}
            position="bottom"
          >
            <NotionButton
              variant="ghost"
              iconOnly
              size="sm"
              className={cn(
                "h-6 w-6 text-muted-foreground/50 hover:text-foreground hover:bg-[var(--interactive-hover)] transition-colors",
                !rightPanelVisible && "text-muted-foreground/70"
              )}
              onClick={toggleRightPanel}
            >
              <SidebarSimple size={14} />
            </NotionButton>
          </CommonTooltip>
        </div>
      )}
      <PanelGroup direction="horizontal" autoSaveId="learning-hub-note-layout" className="flex-1 min-h-0">
        <Panel
          defaultSize={80}
          minSize={50}
          id="learning-hub-note-editor"
          order={1}
          className="flex flex-col min-h-0"
        >
          {/* ★ 视图模式切换栏：OCR/大笔记默认预览模式，避免 ProseMirror 卡死 */}
          {!readOnly && (
            <div className="flex items-center justify-end gap-1 px-2 py-0.5 flex-shrink-0 bg-muted/20 border-b border-border/30">
              <span className="text-[10px] text-muted-foreground mr-1">
                {isOldMonolithicOcr ? '⚠️ 旧版OCR(只读)' : isOcrNote ? '🔍 OCR 笔记' : isLargeContent ? '📄 大文件' : ''}
              </span>
              {/* ★ 旧版巨型 OCR 笔记不允许编辑（会导致编辑器卡死） */}
              {!isOldMonolithicOcr && (
                <CommonTooltip content={viewMode === 'preview' ? t('notes:switchToEditor', '切换到编辑模式') : t('notes:switchToPreview', '切换到预览模式')} position="bottom">
                  <NotionButton
                    variant="ghost"
                    size="sm"
                    className="h-6 text-[11px] gap-1 text-muted-foreground hover:text-foreground"
                    onClick={() => setViewMode(v => v === 'preview' ? 'editor' : 'preview')}
                  >
                    {viewMode === 'preview' ? (
                      <><PencilSimple size={12} /> 编辑</>
                    ) : (
                      <><Eye size={12} /> 预览</>
                    )}
                  </NotionButton>
                </CommonTooltip>
              )}
              {/* ★★ 编辑模式下显示 保存/放弃 按钮 */}
              {viewMode === 'editor' && !readOnly && (
                <>
                  <span className="text-slate-300 dark:text-slate-600 mx-0.5">|</span>
                  <span className="text-[10px] text-muted-foreground">
                    {lastSavedAt
                      ? `已保存 ${lastSavedAt.toLocaleTimeString()}`
                      : isSaving ? '保存中...' : ''}
                  </span>
                  <NotionButton
                    variant="primary"
                    size="sm"
                    className="h-6 text-[11px] gap-1"
                    disabled={isSaving}
                    onClick={handleManualSave}
                  >
                    {isSaving ? (
                      <CircleNotch size={12} className="animate-spin" />
                    ) : (
                      '💾 保存'
                    )}
                  </NotionButton>
                  <NotionButton
                    variant="ghost"
                    size="sm"
                    className="h-6 text-[11px] gap-1 text-muted-foreground hover:text-destructive"
                    onClick={handleDiscardEdit}
                  >
                    ↩ 放弃
                  </NotionButton>
                </>
              )}
            </div>
          )}
          {viewMode === 'preview' ? (
            isOcrNote && ocrSourceId ? (
              // ★ OCR 笔记：逐页阅读器（避免加载完整 MD 卡死）
              <div className="flex-1 min-h-0 flex flex-col">
                {/* 页码导航栏 */}
                <div className="flex items-center justify-center gap-2 px-2 py-1 bg-muted/30 border-b text-[11px] select-none">
                  <NotionButton
                    variant="ghost" size="sm" className="h-6 px-1"
                    disabled={ocrPageNumber <= 1 || ocrPageLoading}
                    onClick={() => {
                      const prev = Math.max(1, ocrPageNumber - 1);
                      setOcrPageNumber(prev);
                      void loadOcrPage(prev);
                    }}
                  >◀</NotionButton>
                  <span className="text-muted-foreground min-w-[80px] text-center">
                    第 {ocrPageNumber} 页
                    {ocrTotalPages ? ` / ${ocrTotalPages}` : ''}
                  </span>
                  <NotionButton
                    variant="ghost" size="sm" className="h-6 px-1"
                    disabled={ocrPageLoading || (ocrTotalPages !== null && ocrPageNumber >= ocrTotalPages)}
                    onClick={() => {
                      const next = ocrPageNumber + 1;
                      setOcrPageNumber(next);
                      void loadOcrPage(next);
                    }}
                  >▶</NotionButton>
                  {ocrPageLoading && <CircleNotch size={12} className="animate-spin text-muted-foreground ml-1" />}
                </div>
                {/* 逐页 MD 内容 */}
                <MarkdownPreview
                  content={ocrPageMd || (ocrPageLoading ? '' : '*(此页无文字内容)*')}
                  loading={ocrPageLoading}
                  className="flex-1 min-h-0"
                />
              </div>
            ) : (
              <MarkdownPreview
                content={content || ''}
                loading={isLoading}
                className="flex-1 min-h-0"
              />
            )
          ) : (
            <NotesCrepeEditor
              key={`editor-${noteId}-${isOcrNote ? ocrPageNumber : 0}`}
              initialContent={
                isOcrNote && ocrPageMd
                  ? ocrPageMd // OCR 编辑当前页内容（get_ocr_page_md 已含 # Page N 标题）
                  : content
              }
              initialTitle={
                isOcrNote
                  ? `${node.name} - 第 ${ocrPageNumber} 页`
                  : title
              }
              onSave={readOnly ? undefined : handleSave}
              onTitleChange={readOnly ? undefined : handleTitleChange}
              noteId={noteId}
              className="flex-1 min-h-0"
              readOnly={readOnly}
              onEditorReady={(api) => {
                editorApiRef.current = api;
              }}
            />
          )}
        </Panel>

        {!isSmallScreen && (
          <>
            <PanelResizeHandle className={cn(
              "w-1 bg-border/40 hover:bg-primary/20 transition-colors flex items-center justify-center group",
              !rightPanelVisible && "pointer-events-none opacity-0 !w-0"
            )}>
              <DotsSixVertical size={12} className="text-muted-foreground/30 group-hover:text-muted-foreground/60 transition-colors" />
            </PanelResizeHandle>
            <Panel
              ref={rightPanelRef}
              defaultSize={20}
              minSize={15}
              maxSize={30}
              collapsedSize={0}
              id="learning-hub-note-outline"
              order={2}
              collapsible
              onCollapse={() => setRightPanelVisible(false)}
              onExpand={() => setRightPanelVisible(true)}
              className={cn(
                "flex flex-col min-h-0 bg-muted/5 transition-all",
                rightPanelVisible ? "border-l border-border/40" : "border-l-0"
              )}
            >
              {rightPanelVisible && (
                <NotesContextPanel
                  noteId={noteId}
                  title={title}
                  createdAt={node.createdAt}
                  updatedAt={node.updatedAt}
                  tags={tags}
                  content={content || ''}
                  onTagsChange={readOnly ? undefined : handleTagsChange}
                />
              )}
            </Panel>
          </>
        )}
      </PanelGroup>

    </div>
  );
};

export default NoteContentView;
