/**
 * Chat V2 - Deprecated Tool 块渲染插件
 *
 * 渲染已弃用/已移除工具的历史数据块
 * 当软件更新后旧工具被重命名或移除，旧会话中的工具块会以此组件展示。
 *
 * 功能：
 * 1. 显示工具名称
 * 2. 警告标识和 "deprecated" 标签
 * 3. 用户友好提示文案
 * 4. 可折叠的输入/输出数据展示
 * 5. 保留完整历史数据
 */

import React, { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { cn } from '@/utils/cn';
import {
  WarningCircle,
  CaretDown,
  CaretRight,
  Wrench,
} from '@phosphor-icons/react';
import { blockRegistry, type BlockComponentProps } from '../../registry';

// ============================================================================
// 子组件：可折叠数据展示
// ============================================================================

interface CollapsibleSectionProps {
  label: string;
  children: React.ReactNode;
  defaultExpanded?: boolean;
}

const CollapsibleSection: React.FC<CollapsibleSectionProps> = ({
  label,
  children,
  defaultExpanded = false,
}) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  return (
    <div className="border-b border-border/20 last:border-b-0">
      <button
        type="button"
        className={cn(
          'flex items-center gap-1.5 w-full px-3 py-2 text-xs',
          'text-muted-foreground hover:text-foreground transition-colors',
          'hover:bg-muted/20'
        )}
        onClick={() => setIsExpanded(!isExpanded)}
      >
        {isExpanded ? <CaretDown size={12} /> : <CaretRight size={12} />}
        <span className="font-medium">{label}</span>
      </button>
      {isExpanded && (
        <div className="px-3 pb-2">
          {children}
        </div>
      )}
    </div>
  );
};

// ============================================================================
// 主组件：Deprecated Tool Block
// ============================================================================

const DeprecatedToolBlock: React.FC<BlockComponentProps> = React.memo(({
  block,
}) => {
  const { t } = useTranslation('chatV2');
  const toolName = block.toolName || t('blocks.mcpTool.unknownTool');

  return (
    <div
      className={cn(
        'deprecated-tool-block',
        'rounded-lg border overflow-hidden',
        'border-amber-200 dark:border-amber-800',
        'bg-amber-50/60 dark:bg-amber-950/30'
      )}
    >
      {/* 头部：工具名称 + deprecated 标签 */}
      <div className="flex items-center justify-between px-3 py-2.5 border-b border-amber-200/50 dark:border-amber-800/50">
        <div className="flex items-center gap-2 min-w-0">
          {/* 工具图标 */}
          <div className="p-1.5 rounded-md bg-amber-100 dark:bg-amber-900/50 shrink-0">
            <Wrench size={16} className="text-amber-600 dark:text-amber-400" />
          </div>

          {/* 工具名称 */}
          <div className="flex flex-col min-w-0">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium text-foreground truncate">
                {toolName}
              </span>
              {/* deprecated 标签 */}
              <span
                className={cn(
                  'inline-flex items-center gap-1 text-[10px] font-semibold',
                  'px-1.5 py-0.5 rounded-full',
                  'bg-amber-200 dark:bg-amber-800',
                  'text-amber-800 dark:text-amber-200'
                )}
              >
                <WarningCircle size={10} weight="fill" />
                deprecated
              </span>
            </div>
            {/* 用户友好提示 */}
            <span className="text-xs text-muted-foreground mt-0.5">
              {t('blocks.deprecatedTool.toolDeprecated', {
                toolName,
                defaultValue: `工具「${toolName}」已不再可用，但历史数据已保留`,
              })}
            </span>
          </div>
        </div>

        {/* 状态标记 */}
        <span
          className={cn(
            'text-[10px] font-medium px-1.5 py-0.5 rounded shrink-0',
            block.status === 'success' && 'bg-green-500/10 text-green-600 dark:text-green-400',
            block.status === 'error' && 'bg-red-500/10 text-red-600 dark:text-red-400',
            block.status === 'running' && 'bg-blue-500/10 text-blue-600 dark:text-blue-400',
            block.status === 'pending' && 'bg-gray-500/10 text-gray-600 dark:text-gray-400'
          )}
        >
          {block.status}
        </span>
      </div>

      {/* 输入参数 */}
      {block.toolInput && Object.keys(block.toolInput).length > 0 && (
        <CollapsibleSection
          label={t('blocks.mcpTool.input', '输入参数')}
          defaultExpanded={false}
        >
          <pre
            className={cn(
              'text-xs font-mono whitespace-pre-wrap break-words',
              'text-muted-foreground bg-background/50 p-2 rounded',
              'max-h-60 overflow-auto'
            )}
          >
            {JSON.stringify(block.toolInput, null, 2)}
          </pre>
        </CollapsibleSection>
      )}

      {/* 输出结果 */}
      {block.toolOutput !== undefined && (
        <CollapsibleSection
          label={t('blocks.mcpTool.output', '输出结果')}
          defaultExpanded={true}
        >
          <pre
            className={cn(
              'text-xs font-mono whitespace-pre-wrap break-words',
              'text-foreground bg-background/50 p-2 rounded',
              'max-h-60 overflow-auto'
            )}
          >
            {typeof block.toolOutput === 'string'
              ? block.toolOutput
              : JSON.stringify(block.toolOutput, null, 2)}
          </pre>
        </CollapsibleSection>
      )}

      {/* 错误信息 */}
      {block.error && (
        <div className="px-3 py-2">
          <div
            className={cn(
              'p-2 rounded-md text-xs',
              'bg-destructive/10 border border-destructive/30 text-destructive/90'
            )}
          >
            {block.error}
          </div>
        </div>
      )}
    </div>
  );
});

// ============================================================================
// 自动注册
// ============================================================================

blockRegistry.register('deprecated_tool', {
  type: 'deprecated_tool',
  component: DeprecatedToolBlock,
  onAbort: 'mark-error',
});

// 导出组件（可选，用于测试）
export { DeprecatedToolBlock };
