/**
 * 知识导图编辑器包装组件
 *
 * 将 MindMapContentView 包装为符合 DSTU EditorProps 接口的组件。
 * 
 * 设计原则：
 * - 使用 DSTU 原生的 MindMapContentView
 * - 从 DSTU path 获取 DstuNode
 * - 可在 Learning Hub 或其他地方复用
 */

import React, { lazy, Suspense, useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { CircleNotch, WarningCircle, ArrowClockwise } from '@phosphor-icons/react';
import type { EditorProps } from '../editorTypes';
import { dstu } from '../index';
import { cn } from '@/lib/utils';
import { NotionButton } from '@/components/ui/NotionButton';
import type { DstuNode } from '../types';
import { showGlobalNotification } from '@/components/UnifiedNotification';

// 懒加载 MindMapContentView
const MindMapContentView = lazy(() => 
  import('@/features/mindmap/MindMapContentView').then(m => ({ default: m.MindMapContentView }))
);

/**
 * 知识导图编辑器包装组件
 */
export const MindMapEditorWrapper: React.FC<EditorProps> = (props) => {
  const { t } = useTranslation(['dstu', 'mindmap', 'common']);
  const [node, setNode] = useState<DstuNode | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 解析路径获取 mindmapId
  const path = props.path;

  // 获取回调
  const onClose = props.onClose;

  // 从路径提取资源 ID
  const extractResourceId = (dstuPath: string): string | null => {
    // 路径格式: /mm_xxx 或 /folder/mm_xxx
    const match = dstuPath.match(/\/(mm_[a-zA-Z0-9_-]+)$/);
    return match ? match[1] : null;
  };

  // 加载 DstuNode
  const loadNode = useCallback(async () => {
    if (!path) {
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    setError(null);

    const result = await dstu.get(path);
    setIsLoading(false);

    if (result.ok) {
      if (result.value) {
        setNode(result.value);
      } else {
        const errMsg = t('mindmap:errors.notFound');
        setError(errMsg);
        showGlobalNotification('error', errMsg);
      }
    } else {
      const errMsg = result.error.toUserMessage();
      setError(errMsg);
      showGlobalNotification('error', errMsg);
    }
  }, [path, t]);

  useEffect(() => {
    void loadNode();
  }, [loadNode]);

  // 加载中
  if (isLoading) {
    return (
      <div className={cn('flex items-center justify-center h-full', props.className)}>
        <CircleNotch size={24} className="animate-spin text-muted-foreground" />
      </div>
    );
  }

  // 错误状态
  if (error) {
    return (
      <div className={cn('flex flex-col items-center justify-center h-full py-8 gap-4', props.className)}>
        <WarningCircle size={48} className="text-destructive/50" />
        <span className="text-destructive text-center max-w-md">{error}</span>
        <NotionButton variant="ghost"
          className="flex items-center gap-2 px-4 py-2 border rounded-md hover:bg-[var(--interactive-hover)]"
          onClick={loadNode}
        >
          <ArrowClockwise size={16} />
          {t('common:actions.retry')}
        </NotionButton>
      </div>
    );
  }

  // 节点不存在
  if (!node) {
    return (
      <div className={cn('flex flex-col items-center justify-center h-full py-8 gap-4', props.className)}>
        <WarningCircle size={48} className="text-muted-foreground/50" />
        <span className="text-muted-foreground text-center">
          {t('mindmap:errors.notFound')}
        </span>
      </div>
    );
  }

  // 提取资源 ID
  const resourceId = extractResourceId(path) || node.id;

  return (
    <Suspense
      fallback={
        <div className={cn('flex items-center justify-center h-full', props.className)}>
          <CircleNotch size={24} className="animate-spin text-muted-foreground" />
        </div>
      }
    >
      <MindMapContentView
        resourceId={resourceId}
        className={props.className}
      />
    </Suspense>
  );
};
