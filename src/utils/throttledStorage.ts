/**
 * Throttled async localStorage wrapper for Zustand persist middleware.
 *
 * Batches writes with a configurable delay to avoid synchronous I/O
 * blocking the main thread during rapid state changes.
 *
 * Usage in a Zustand store:
 *   persist(storeCreator, {
 *     name: 'my-store',
 *     storage: createThrottledStorage() as any,
 *   })
 *
 * The `as any` cast on the storage option is necessary because Zustand's
 * PersistStorage<S> generic infers S from the full store type (including
 * action methods via partialize), which makes Promise<string | null>
 * incompatible with Promise<StorageValue<S>> due to Promise invariance.
 * Since the runtime behavior (JSON.parse/stringify) handles string values
 * correctly, this cast is safe.
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
      } catch { /* quota exceeded or private browsing — silently ignore */ }
    }
  };

  return {
    getItem: async (name: string): Promise<string | null> => localStorage.getItem(name),
    setItem: async (_name: string, value: string): Promise<void> => {
      pending[_name] = value;
      if (timer === null) timer = setTimeout(flush, delayMs);
    },
    removeItem: async (name: string): Promise<void> => {
      pending[name] = null;
      if (timer === null) timer = setTimeout(flush, delayMs);
    },
  };
}
