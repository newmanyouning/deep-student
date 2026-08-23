/**
 * 阅读进度持久化 — 全局 flush 注册表
 *
 * 解决应用关闭/标签页失活时防抖进度丢失的问题。
 * 各组件注册 flush 回调，main.tsx 的 beforeunload/visibilitychange 调用 flushAll。
 */

type FlushFn = () => void | Promise<void>;

const flushCallbacks = new Set<FlushFn>();

/** 注册一个 flush 回调，返回取消注册函数 */
export const registerFlush = (fn: FlushFn): (() => void) => {
  flushCallbacks.add(fn);
  return () => { flushCallbacks.delete(fn); };
};

/** 同步触发所有已注册的 flush（用于 beforeunload） */
export const flushAllSync = () => {
  for (const fn of flushCallbacks) {
    try { fn(); } catch { /* 不阻塞关闭 */ }
  }
};

/** 异步触发所有已注册的 flush（用于 visibilitychange） */
export const flushAllAsync = async () => {
  const promises: Promise<void>[] = [];
  for (const fn of flushCallbacks) {
    try {
      const result = fn();
      if (result instanceof Promise) {
        promises.push(result.catch(() => {}));
      }
    } catch { /* 不阻塞 */ }
  }
  if (promises.length > 0) {
    await Promise.allSettled(promises);
  }
};
