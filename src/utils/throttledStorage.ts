/**
 * Throttled async localStorage wrapper for Zustand persist.
 * Batches writes to avoid synchronous I/O blocking the main thread.
 */
export function createThrottledStorage(delayMs = 500) {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let pending: Record<string, string | null> = {};
  const flush = () => {
    const batch = pending;
    pending = {};
    timer = null;
    for (const [key, value] of Object.entries(batch)) {
      try {
        if (value === null) localStorage.removeItem(key);
        else localStorage.setItem(key, value);
      } catch {}
    }
  };
  return {
    getItem: async (name: string) => localStorage.getItem(name),
    setItem: async (name: string, value: string) => {
      pending[name] = value;
      if (timer === null) timer = setTimeout(flush, delayMs);
    },
    removeItem: async (name: string) => {
      pending[name] = null;
      if (timer === null) timer = setTimeout(flush, delayMs);
    },
  };
}
