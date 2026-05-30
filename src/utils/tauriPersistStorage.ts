/**
 * Tauri persist storage adapter for Zustand `persist` middleware.
 *
 * P2-08 / REF-007: Unifies persistence strategy across all stores.
 * Previously ankiQueueStore had ~120 lines of custom persistence logic;
 * now any store can use Zustand's standard `persist()` with this adapter.
 *
 * Uses Tauri's secure store as primary backend with localStorage fallback.
 */

import type { PersistStorage, StorageValue } from 'zustand/middleware';

const hasLocalStorage = (): boolean =>
  typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';

export function createTauriPersistStorage<S>(): PersistStorage<S> {
  return {
    async getItem(name: string): Promise<StorageValue<S> | null> {
      try {
        const { TauriAPI } = await import('@/utils/tauriApi');
        const stored = await TauriAPI.getSetting(name);
        if (stored) return JSON.parse(stored) as StorageValue<S>;
      } catch {
        // Tauri not available — fall through to localStorage
      }
      if (hasLocalStorage()) {
        try {
          const stored = window.localStorage.getItem(name);
          if (stored) return JSON.parse(stored) as StorageValue<S>;
        } catch { /* ignore */ }
      }
      return null;
    },

    async setItem(name: string, value: StorageValue<S>): Promise<void> {
      const serialized = JSON.stringify(value);
      try {
        const { TauriAPI } = await import('@/utils/tauriApi');
        await TauriAPI.saveSetting(name, serialized);
        return;
      } catch { /* fall through */ }
      if (hasLocalStorage()) {
        try {
          window.localStorage.setItem(name, serialized);
        } catch { /* ignore */ }
      }
    },

    async removeItem(name: string): Promise<void> {
      try {
        const { TauriAPI } = await import('@/utils/tauriApi');
        // Tauri doesn't have a direct "removeSetting", but we can save empty
        await TauriAPI.saveSetting(name, '');
      } catch { /* fall through */ }
      if (hasLocalStorage()) {
        try {
          window.localStorage.removeItem(name);
        } catch { /* ignore */ }
      }
    },
  };
}
