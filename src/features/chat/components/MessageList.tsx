/**
 * Chat V2 - MessageList 消息列表组件
 *
 * 职责：虚拟滚动，订阅 messageOrder，渲染 MessageItem
 * 
 * 🚀 P1 优化（冷启动与虚拟化）：
 * 1. 首帧直接渲染少量可见项，不初始化虚拟化
 * 2. 虚拟化延迟初始化（requestIdleCallback）
 * 3. 首帧禁用 measureElement，滚动稳定后开启
 * 4. 滚动逻辑简化：rAF + 条件触发
 * 5. 移除 flushSync，异步状态更新
 */

import React, { useRef, useEffect, useLayoutEffect, useCallback, memo, useMemo, useState } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { useTranslation } from 'react-i18next';
import type { StoreApi } from 'zustand';
import { motion, AnimatePresence } from 'framer-motion';
import { cn } from '@/utils/cn';
import { newMessageVariants } from '@/styles/motion-variants';
import { CustomScrollArea } from '@/components/custom-scroll-area';
import { MessageItem } from './MessageItem';
import { useMessageOrder, useSessionStatus, useIsDataLoaded } from '../hooks/useChatStore';
import type { ChatStore } from '../core/types';
import { sessionSwitchPerf } from '../debug/sessionSwitchPerf';
import { useBreakpoint } from '@/hooks/useBreakpoint';
import Z_INDEX from '@/config/zIndex';
import { useSmoothWheel } from '../hooks/useSmoothWheel';
import { ArrowDown } from '@phosphor-icons/react';
import { ThreadEmptyStateShell } from './ui/ThreadEmptyStateShell';
import { ThreadContentShell } from './ui/ThreadContentShell';

// ============================================================================
// 常量定义
// ============================================================================

/** 首帧直接渲染的消息数量（不使用虚拟化） */
const INITIAL_RENDER_COUNT = 10;

/** 虚拟化初始化延迟（ms）- 使用 requestIdleCallback 或 setTimeout */
const VIRTUALIZER_INIT_DELAY = 50;

/** 默认估算消息高度（设置为合理值，测量会覆盖）*/
const DEFAULT_ESTIMATED_ITEM_SIZE = 120;
/** 超过该数量后启用虚拟滚动，避免长会话全量渲染 */
const VIRTUALIZATION_THRESHOLD = 80;

// ============================================================================
// Props 定义
// ============================================================================

export interface MessageListProps {
  /** Store 实例 */
  store: StoreApi<ChatStore>;
  /** 自定义类名 */
  className?: string;
  /** 空态中显示的当前分组名；未分组时不显示 */
  emptyStateGroupName?: string | null;
  /** 预估消息高度 */
  estimatedItemSize?: number;
  /** 过滤空消息 */
  overscan?: number;
  /** 🆕 强制显示空态（用于空态预览） */
  forceEmptyPreview?: boolean;
}

// ============================================================================
// 组件实现
// ============================================================================

/**
 * MessageList 消息列表组件
 *
 * 功能：
 * 1. 虚拟滚动优化性能
 * 2. 自动滚动到底部（流式生成时）
 * 3. 空状态展示
 */
const MessageListInner: React.FC<MessageListProps> = ({
  store,
  className,
  emptyStateGroupName = null,
  estimatedItemSize = DEFAULT_ESTIMATED_ITEM_SIZE,
  overscan = 5,
  forceEmptyPreview = false,
}) => {
  // 📊 细粒度打点：组件函数开始执行
  const instanceIdRef = useRef(Math.random().toString(36).slice(2, 8));
  const renderCountRef = useRef(0);
  renderCountRef.current++;

  sessionSwitchPerf.mark('ml_mount', {
    instanceId: instanceIdRef.current,
    renderCount: renderCountRef.current,
  });

  const { t } = useTranslation('chatV2');
  const scrollToBottomLabel = t('messageList.scrollToBottom', {
    defaultValue: 'Scroll to bottom',
  });

  // 📱 移动端适配：检测屏幕尺寸
  const { isSmallScreen } = useBreakpoint();

  // 容器 ref - CustomScrollArea 的外层容器
  const containerRef = useRef<HTMLDivElement>(null);

  // 🚀 P1优化：viewport 状态管理
  // 使用 useState 替代 useReducer + flushSync，避免强制同步刷新
  const [viewportElement, setViewportElement] = useState<HTMLDivElement | null>(null);

  // 🚀 虚拟滚动状态管理
  const [virtualizerReady, setVirtualizerReady] = useState(false);

  // viewport callback ref - 异步更新状态，不使用 flushSync
  const viewportCallbackRef = useCallback((node: HTMLDivElement | null) => {
    if (node) {
      // 异步设置 viewport，不阻塞首帧渲染
      setViewportElement(node);
    }
  }, []);

  // 订阅消息顺序（已通过 useMessageOrder 内部的引用缓存优化）
  const messageOrder = useMessageOrder(store);

  // WCAG: 屏幕阅读器新消息通知（适用于虚拟化模式）
  const prevSrCountRef = useRef(messageOrder.length);
  const isFirstSrRender = useRef(true);
  const [srAnnouncement, setSrAnnouncement] = useState('');
  useEffect(() => {
    if (isFirstSrRender.current) {
      isFirstSrRender.current = false;
      prevSrCountRef.current = messageOrder.length;
      return;
    }
    if (messageOrder.length > prevSrCountRef.current) {
      setSrAnnouncement(
        t('messageList.srNewMessages', {
          count: messageOrder.length,
          defaultValue: `New messages received, total {{count}} messages`,
        })
      );
    }
    prevSrCountRef.current = messageOrder.length;
  }, [messageOrder.length, t]);

  // 订阅会话状态
  const sessionStatus = useSessionStatus(store);

  // 订阅数据是否已加载
  const isDataLoaded = useIsDataLoaded(store);

  // 📊 细粒度打点：hooks 执行完成
  sessionSwitchPerf.mark('ml_hooks_done', {
    messageCount: messageOrder.length,
    isDataLoaded
  });

  // 📊 性能打点：追踪首次渲染完成
  const hasMarkedFirstRenderRef = useRef(false);
  const hasMarkedFirstRenderScheduledRef = useRef(false);
  const lastStoreRef = useRef<StoreApi<ChatStore> | null>(null);

  // 🚀 性能优化：使用 useMemo 计算 scrollAreaKey
  // 当 store 变化时，key 变化，CustomScrollArea 重新挂载，callback ref 被调用
  const scrollAreaKey = useMemo(() => Math.random(), [store]);

  const storeChanged = lastStoreRef.current !== store;
  if (storeChanged) {
    hasMarkedFirstRenderRef.current = false;
    hasMarkedFirstRenderScheduledRef.current = false;
    lastStoreRef.current = store;
  }

  // 🔧 追踪是否为历史会话首次渲染（用于跳过入场动画 + 强制布局）
  // 每次切换会话时重置为 true，确保新会话正确滚动到底部
  const isInitialRenderRef = useRef(true);
  if (storeChanged) {
    isInitialRenderRef.current = true;
  }

  // 是否正在流式生成
  const isStreaming = sessionStatus === 'streaming';
  // 超长会话启用虚拟滚动，短会话保持直接渲染以降低复杂度
  const useDirectRender = messageOrder.length <= VIRTUALIZATION_THRESHOLD;

  const virtualRowCount = messageOrder.length;

  // 🚀 虚拟化延迟初始化
  useEffect(() => {
    if (!viewportElement) return;

    const timeoutId = setTimeout(() => {
      setVirtualizerReady(true);
      sessionSwitchPerf.mark('ml_virtualizer_ready', { delayed: true });
    }, VIRTUALIZER_INIT_DELAY);

    return () => clearTimeout(timeoutId);
  }, [viewportElement]);

  // 虚拟化初始化耗时记录
  const hasLoggedVirtualizerRef = useRef(false);
  const virtualizerInitStart = performance.now();

  // 虚拟滚动配置
  const virtualizer = useVirtualizer({
    count: virtualizerReady && !useDirectRender ? virtualRowCount : 0,
    getScrollElement: () => viewportElement,
    estimateSize: () => estimatedItemSize,
    overscan,
    // 🔧 修复消息重叠：始终启用测量，不再延迟
    // 延迟测量会导致虚拟化器使用估算高度定位消息，造成重叠
    measureElement: (element) => element?.getBoundingClientRect().height ?? estimatedItemSize,
  });

  if (!hasLoggedVirtualizerRef.current && virtualizerReady) {
    const virtualizerInitMs = performance.now() - virtualizerInitStart;
    sessionSwitchPerf.mark('ml_virtualizer_done', {
      ms: virtualizerInitMs,
      messageCount: messageOrder.length,
    });
    hasLoggedVirtualizerRef.current = true;
  }

  // 🔧 程序化滚动锁 refs（需在 useLayoutEffect 之前声明，避免 TDZ）
  const programmaticScrollLockRef = useRef(false);
  const programmaticScrollUnlockTimerRef = useRef<number | null>(null);

  // useLayoutEffect 中移除 virtualizer.measure() 调用
  // ref callback (measureElement) 在提交阶段执行，此时 DOM 已创建但浏览器尚未 paint。
  // getBoundingClientRect().height 已返回正确值，measure() 会清空 itemSizeCache，
  // 导致所有回退到 estimateSize(120px) → 首帧位置错误。
  // ResizeObserver 自动处理所有动态高度变化，无需显式 measure()。

  // 仅在消息数量变化（新消息到达）时调用 measure()
  // 这是唯一需要重算缓存的情况：新虚拟项无缓存条目，measureElement ref
  // 回调在提交阶段捕获其实际高度 → resizeItem 填充缓存。
  // 与旧的每帧重测不同：不随 isStreaming 触发，不随每帧 rAF 触发。
  useEffect(() => {
    if (useDirectRender || !virtualizerReady) return;
    virtualizer.measure();
  }, [virtualRowCount]);

  // 🔧 历史会话首次加载：强制浏览器完成布局计算后再展示
  // 问题：大量消息同时渲染时，浏览器可能在 layout 完成前就 paint，
  // 导致消息高度计算不准确，出现视觉堆叠。展开思考/滚动触发 re-layout 后恢复。
  // 解决：useLayoutEffect 在 paint 前同步执行，强制 reflow 确保所有高度已计算。
  useLayoutEffect(() => {
    if (!isDataLoaded || !viewportElement) return;
    if (!isInitialRenderRef.current) return;
    isInitialRenderRef.current = false;

    // 强制同步 reflow：读取 offsetHeight 会触发浏览器完成所有待处理的 layout
    void viewportElement.offsetHeight;

    // 已移除 virtualizer.measure():
    // 该调用在首次渲染时清空缓存，导致所有项回退至 estimateSize(120px)，
    // 破坏了 measureElement ref callback 在提交阶段已捕获的正确高度。
    // ResizeObserver + measureElement ref 已覆盖所有测量场景。

    // 🔧 历史会话打开后自动滚动到底部
    // 使用 virtualizer.getTotalSize() 而非 scrollHeight：
    // measureElement ref callbacks 在 commit 阶段已填充 itemSizeCache，
    // getTotalSize() 实时重算返回真实高度，而 scrollHeight 仍是 render 阶段旧值
    programmaticScrollLockRef.current = true;
    const realTotal = virtualizerReady && !useDirectRender
      ? virtualizer.getTotalSize()
      : viewportElement.scrollHeight;
    viewportElement.scrollTop = realTotal;
    // 短延迟后释放程序化滚动锁
    if (programmaticScrollUnlockTimerRef.current !== null) {
      window.clearTimeout(programmaticScrollUnlockTimerRef.current);
    }
    programmaticScrollUnlockTimerRef.current = window.setTimeout(() => {
      programmaticScrollLockRef.current = false;
      programmaticScrollUnlockTimerRef.current = null;
    }, 100);
  }, [isDataLoaded, viewportElement, useDirectRender, virtualizerReady, virtualizer]);

  // 🔧 优化：使用 ref 追踪上一次消息数量和滚动状态
  const [showScrollToBottom, setShowScrollToBottom] = useState(false);

  /** 检查当前是否在底部附近（阈值 50px，ChatGPT/Claude 同级灵敏度） */
  const isNearBottom = useCallback(() => {
    if (!viewportElement) return true;
    const { scrollTop, scrollHeight, clientHeight } = viewportElement;
    return scrollHeight - scrollTop - clientHeight < 50;
  }, [viewportElement]);

  const scheduleProgrammaticScrollUnlock = useCallback((delayMs: number) => {
    if (programmaticScrollUnlockTimerRef.current !== null) {
      window.clearTimeout(programmaticScrollUnlockTimerRef.current);
    }
    programmaticScrollUnlockTimerRef.current = window.setTimeout(() => {
      programmaticScrollLockRef.current = false;
      programmaticScrollUnlockTimerRef.current = null;
    }, delayMs);
  }, []);

  // 滚动到底部（仅用户点击按钮时触发）
  const scrollToBottom = useCallback((behavior: ScrollBehavior = 'auto') => {
    if (!viewportElement) return;
    const totalHeight = useDirectRender
      ? viewportElement.scrollHeight
      : virtualizer.getTotalSize();
    const maxScroll = totalHeight - viewportElement.clientHeight;
    if (behavior === 'smooth') {
      programmaticScrollLockRef.current = true;
      viewportElement.scrollTo({ top: Math.max(maxScroll * 2, totalHeight * 2), behavior: 'smooth' });
      scheduleProgrammaticScrollUnlock(500);
    } else {
      programmaticScrollLockRef.current = true;
      viewportElement.scrollTop = Math.max(0, maxScroll);
      scheduleProgrammaticScrollUnlock(100);
    }
  }, [scheduleProgrammaticScrollUnlock, viewportElement, useDirectRender, virtualizer]);

  /** 点击"回到底部"按钮 */
  const handleScrollToBottomClick = useCallback((event: React.MouseEvent<HTMLButtonElement>) => {
    event.currentTarget.blur();
    setShowScrollToBottom(false);
    scrollToBottom('smooth');
  }, [scrollToBottom]);

  // 滚动事件 → 仅控制按钮显隐，不触发任何自动滚动
  useEffect(() => {
    if (!viewportElement) return;
    const syncScrollState = () => {
      if (programmaticScrollLockRef.current) return;
      setShowScrollToBottom(!isNearBottom());
    };
    syncScrollState();
    viewportElement.addEventListener('scroll', syncScrollState, { passive: true });
    return () => viewportElement.removeEventListener('scroll', syncScrollState);
  }, [viewportElement, isNearBottom]);

  // 清理 programmaticScrollUnlock 计时器
  useEffect(() => {
    return () => {
      if (programmaticScrollUnlockTimerRef.current !== null) {
        window.clearTimeout(programmaticScrollUnlockTimerRef.current);
      }
    };
  }, []);

  // 🖱️ 平滑滚轮 + 第一时间显示"回到底部"按钮
  useSmoothWheel(containerRef.current, {
    onUserScrollUp: () => {
      setShowScrollToBottom(true);
    },
  });

  // 📊 性能打点：首次渲染完成
  // 只有当 isDataLoaded 为 true 时才触发 first_render，避免 race condition
  useEffect(() => {
    // 📊 细粒度打点：useEffect 触发
    sessionSwitchPerf.mark('ml_effect_trigger', { isDataLoaded });

    if (hasMarkedFirstRenderRef.current) return;
    if (!isDataLoaded) return; // 等待数据加载完成

    // 使用 requestAnimationFrame 确保 DOM 已经渲染
    requestAnimationFrame(() => {
      if (hasMarkedFirstRenderRef.current) return; // 双重检查

      sessionSwitchPerf.mark('first_render', {
        messageCount: messageOrder.length,
        isEmpty: messageOrder.length === 0,
      });
      sessionSwitchPerf.endTrace(); // 结束追踪
      hasMarkedFirstRenderRef.current = true;
    });
  }, [isDataLoaded, messageOrder.length]);

  // 📊 细粒度打点：render 开始
  const getVirtualItemsStart = performance.now();
  const virtualItems = virtualizerReady ? virtualizer.getVirtualItems() : [];
  const getVirtualItemsMs = performance.now() - getVirtualItemsStart;
  sessionSwitchPerf.mark('ml_get_virtual_items', { ms: getVirtualItemsMs, count: virtualItems.length });
  const hasViewport = !!viewportElement;

  // 说明：短会话直渲避免虚拟化成本，长会话启用虚拟滚动以控制 DOM 规模。

  sessionSwitchPerf.mark('ml_render_start', {
    messageCount: messageOrder.length,
    virtualItemCount: virtualItems.length,
    hasViewport,
    useDirectRender,
    virtualizerReady,
  });

  // 📊 细粒度打点：首帧在 render 路径上被调度（避免仅依赖 effect/rAF）
  if (!hasMarkedFirstRenderScheduledRef.current && isDataLoaded) {
    sessionSwitchPerf.mark('first_render_scheduled', {
      messageCount: messageOrder.length,
      hasViewport,
      useDirectRender,
    });
    hasMarkedFirstRenderScheduledRef.current = true;
  }

  // 空状态
  if (forceEmptyPreview || messageOrder.length === 0) {
    const emptyStatePrimaryAction = emptyStateGroupName
      ? t('messageList.empty.primaryActionInGroup', {
          groupName: emptyStateGroupName,
          defaultValue: '在「{{groupName}}」里学点什么？',
        })
      : t('messageList.empty.primaryAction', { defaultValue: '今天想学点什么？' });

    return (
      <div
        className={cn(
          'flex h-full w-full flex-col',
          className
        )}
      >
        <div className="custom-scrollbar min-h-0 flex-1 overflow-y-auto px-4 pb-6 pt-3 md:px-8 md:pb-8 md:pt-4">
          <ThreadEmptyStateShell
            title={emptyStatePrimaryAction}
            contentClassName={isSmallScreen ? 'py-10' : 'py-16'}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="relative h-full">
    {/* WCAG 4.1.3: 屏幕阅读器通知区域（虚拟化模式下不能在容器上用 aria-live） */}
    <div
      role="status"
      aria-live="polite"
      aria-atomic="true"
      className="sr-only"
    >
      {srAnnouncement}
    </div>
    <CustomScrollArea
      key={scrollAreaKey}
      ref={containerRef}
      viewportRef={viewportCallbackRef}
      className={cn('h-full', className)}
      viewportClassName="scroll-smooth"
      viewportProps={{
        // 无需底部 padding，布局已分离
      }}
      hideTrackWhenIdle
    >
      {useDirectRender ? (
        // 直接渲染模式(禁用虚拟化)
        <div
          role="log"
          aria-live="polite"
          aria-relevant="additions"
          style={{ width: '100%' }}
        >
          <AnimatePresence>
            {messageOrder.map((messageId, messageIndex) => {
              const isUserMessage = store.getState().getMessage(messageId)?.role === 'user';
              const content = (
                <MessageItem
                  messageId={messageId}
                  store={store}
                  isFirst={messageIndex === 0}
                  isLatest={messageIndex === messageOrder.length - 1}
                />
              );
              if (isUserMessage) {
                return (
                  <motion.div
                    key={messageId}
                    variants={newMessageVariants}
                    initial={isInitialRenderRef.current ? "animate" : "initial"}
                    animate="animate"
                    exit="exit"
                  >
                    {content}
                  </motion.div>
                );
              }
              return <div key={messageId}>{content}</div>;
            })}
          </AnimatePresence>
        </div>
      ) : (
        // 虚拟滚动模式
        <div
          role="log"
          aria-live="polite"
          aria-relevant="additions"
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {virtualItems.map((virtualRow) => {
            const messageId = messageOrder[virtualRow.index];
            if (!messageId) return null;

            const isUserMessage = store.getState().getMessage(messageId)?.role === 'user';

            return (
              <div
                key={messageId}
                data-index={virtualRow.index}
                ref={virtualizer.measureElement}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                {isUserMessage ? (
                  <motion.div
                    variants={newMessageVariants}
                    initial={isInitialRenderRef.current ? "animate" : "initial"}
                    animate="animate"
                  >
                    <MessageItem
                      messageId={messageId}
                      store={store}
                      isFirst={virtualRow.index === 0}
                      isLatest={virtualRow.index === messageOrder.length - 1}
                    />
                  </motion.div>
                ) : (
                  <MessageItem
                    messageId={messageId}
                    store={store}
                    isFirst={virtualRow.index === 0}
                    isLatest={virtualRow.index === messageOrder.length - 1}
                  />
                )}
              </div>
            );
          })}
        </div>
      )}
    </CustomScrollArea>
    {/* 回到底部浮动按钮 */}
    <div
      className="pointer-events-none absolute inset-x-0 bottom-2 px-4 md:bottom-3 md:px-8"
      style={{ zIndex: Z_INDEX.inputBar - 10 }}
    >
      <ThreadContentShell className="pointer-events-none overflow-visible">
        <div
          className="t-panel-slide ml-auto w-fit"
          data-open={showScrollToBottom ? 'true' : 'false'}
          aria-hidden={!showScrollToBottom}
          style={{
            ['--panel-translate-y' as string]: '12px',
            ['--panel-open-dur' as string]: '300ms',
            ['--panel-close-dur' as string]: '220ms',
          }}
        >
          <button
            type="button"
            onClick={handleScrollToBottomClick}
            title={scrollToBottomLabel}
            data-slot="message-list-scroll-to-bottom"
            tabIndex={showScrollToBottom ? 0 : -1}
            className={cn(
              'pointer-events-auto ml-auto flex h-10 w-10 items-center justify-center rounded-full',
              'border border-[color:var(--button-utility-border)] bg-[color:var(--button-utility-surface)]',
              'text-[color:var(--button-utility-foreground)] transition-colors duration-150',
              'hover:border-[color:var(--button-utility-border)] hover:bg-[color:var(--button-utility-hover)] hover:text-[color:var(--button-utility-foreground)]',
              'active:bg-[color:var(--button-utility-active)]',
              'focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/30',
              'cursor-pointer'
            )}
            aria-label={scrollToBottomLabel}
          >
            <ArrowDown size={16} weight="bold" />
          </button>
        </div>
      </ThreadContentShell>
    </div>
    </div>
  );
};

// 🚀 性能优化：使用 React.memo 防止父组件重渲染导致的不必要重渲染
// 自定义比较函数：只有当 store 引用或其他 props 真正变化时才重渲染
export const MessageList = memo(MessageListInner, (prevProps, nextProps) => {
  // 如果 store 引用相同，认为 props 没有变化
  // store 内部状态变化通过订阅机制处理，不需要组件重渲染
  return (
    prevProps.store === nextProps.store &&
    prevProps.className === nextProps.className &&
    prevProps.emptyStateGroupName === nextProps.emptyStateGroupName &&
    prevProps.estimatedItemSize === nextProps.estimatedItemSize &&
    prevProps.overscan === nextProps.overscan &&
    prevProps.forceEmptyPreview === nextProps.forceEmptyPreview
  );
});

export default MessageList;
