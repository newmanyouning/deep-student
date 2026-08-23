/**
 * Chat V2 - ExpandableText 可展开文本组件
 *
 * 根据内容实际大小判断显示方式，保证显示完整不截断：
 * - 内容实际高度 ≤ 阈值：完整显示，无截断
 * - 内容实际高度 > 阈值：显示预览（限制高度 + 渐隐），并提供"展开/收起"按钮查看全部
 *
 * 参考 UserMessageBubble 的"预览 + 可展开"模式（scrollHeight 实测 + ResizeObserver），
 * 与"硬截断丢弃内容"（slice + overflow-hidden 无展开入口）的根本区别是：
 * 用户始终可以通过展开查看完整内容。
 *
 * 使用场景：变体预览卡片、长文本摘要、工具输出等可能过长的文本展示。
 */

import React, { useRef, useState, useEffect, useCallback } from 'react';
import { CaretDown, CaretUp } from '@phosphor-icons/react';
import { cn } from '@/utils/cn';
import { useTranslation } from 'react-i18next';

// ============================================================================
// Props
// ============================================================================

export interface ExpandableTextProps {
  /** 完整文本内容（组件内部不会预截断） */
  content: string;
  /** 预览高度阈值（px），内容实际高度超过此值才折叠。默认 120 */
  maxHeight?: number;
  /** 容器自定义类名 */
  className?: string;
  /** 文本段落自定义类名 */
  textClassName?: string;
  /** 展开/收起按钮自定义类名 */
  toggleClassName?: string;
  /** 是否默认展开（默认 false，超阈值时折叠） */
  defaultExpanded?: boolean;
}

// ============================================================================
// 组件
// ============================================================================

export const ExpandableText: React.FC<ExpandableTextProps> = ({
  content,
  maxHeight = 120,
  className,
  textClassName,
  toggleClassName,
  defaultExpanded = false,
}) => {
  const { t } = useTranslation(['chatV2', 'common']);
  const contentRef = useRef<HTMLDivElement>(null);
  const [isOverflow, setIsOverflow] = useState(false);
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  // 根据内容实际大小判断：测量 scrollHeight 是否超过阈值
  useEffect(() => {
    const el = contentRef.current;
    if (!el) return;

    const check = () => {
      setIsOverflow(el.scrollHeight > maxHeight);
    };

    check();

    // 监听内容/尺寸变化（流式追加、图片加载等可能改变高度）
    const observer = new ResizeObserver(check);
    observer.observe(el);
    return () => observer.disconnect();
  }, [content, maxHeight]);

  const toggleExpand = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  // 内容实际高度 ≤ 阈值时，shouldCollapse 为 false，完整显示
  const shouldCollapse = isOverflow && !isExpanded;

  return (
    <div className={cn('expandable-text', className)}>
      {/* 内容区域：超阈值且未展开时限制高度 + 渐隐 */}
      <div
        ref={contentRef}
        className={cn(
          'expandable-text__content relative',
          shouldCollapse && 'expandable-text__content--collapsed overflow-hidden'
        )}
        style={shouldCollapse ? { maxHeight: `${maxHeight}px` } : undefined}
      >
        <p className={cn('whitespace-pre-wrap break-words', textClassName)}>
          {content}
        </p>
        {/* 折叠时底部渐隐遮罩，提示有更多内容 */}
        {shouldCollapse && (
          <div
            aria-hidden="true"
            className="pointer-events-none absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-card to-transparent dark:from-card/50"
          />
        )}
      </div>

      {/* 展开/收起按钮：仅在内容超阈值时显示 */}
      {isOverflow && (
        <button
          type="button"
          onClick={toggleExpand}
          aria-expanded={isExpanded}
          className={cn(
            'mt-1 inline-flex items-center gap-1 text-xs text-primary hover:underline',
            toggleClassName
          )}
        >
          {isExpanded ? (
            <>
              <CaretUp size={12} />
              <span>{t('common:collapse', '收起')}</span>
            </>
          ) : (
            <>
              <CaretDown size={12} />
              <span>{t('common:expand', '展开')}</span>
            </>
          )}
        </button>
      )}
    </div>
  );
};

export default ExpandableText;
