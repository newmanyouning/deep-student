import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { AnkiCard, AnkiGenerationOptions } from '../types';
import { createTauriPersistStorage } from '../utils/tauriPersistStorage';

export type AnkiMaterialSourceType = 'mistake' | 'chat';

export interface ChatQueueSnapshotMetadata {
  trimmed?: boolean;
  originMessageStableId?: string | null;
  focusCardId?: string | null;
}

export interface ChatQueueSnapshot {
  type: 'chat';
  templateId: string;
  cards: AnkiCard[];
  options: AnkiGenerationOptions;
  metadata?: ChatQueueSnapshotMetadata;
}

export type AnkiMaterialSnapshot = Record<string, unknown> | ChatQueueSnapshot | null;

export interface QueuedAnkiMaterial {
  queueId: string;
  sourceId: string;
  sourceType: AnkiMaterialSourceType;
  title: string;
  tags?: string[];
  createdAt: string;
  sourceCreatedAt?: string | null;
  summary?: string | null;
  content: string;
  contentLength: number;
  snapshot?: AnkiMaterialSnapshot;
}

interface AnkiQueueState {
  materials: QueuedAnkiMaterial[];
  addMaterial: (material: QueuedAnkiMaterial, options?: { replaceExisting?: boolean }) => void;
  removeMaterial: (queueId: string) => void;
  clearMaterials: () => void;
}

const STORAGE_KEY = 'anki.material.queue';

const isSameSource = (a: QueuedAnkiMaterial, b: QueuedAnkiMaterial) =>
  a.sourceType === b.sourceType && a.sourceId === b.sourceId;

const parseTimestamp = (value?: string | null): number => {
  if (!value) return 0;
  const ts = Date.parse(value);
  return Number.isNaN(ts) ? 0 : ts;
};

const mergeMaterials = (
  current: QueuedAnkiMaterial[],
  incoming: unknown,
): QueuedAnkiMaterial[] => {
  if (!Array.isArray(incoming) || incoming.length === 0) return current;
  const dedupe = new Map<string, QueuedAnkiMaterial>();
  const upsert = (item: QueuedAnkiMaterial | null | undefined) => {
    if (!item) return;
    if (typeof item.sourceId !== 'string' || typeof item.sourceType !== 'string') return;
    const key = `${item.sourceType}:${item.sourceId}`;
    const existing = dedupe.get(key);
    if (!existing) { dedupe.set(key, item); return; }
    if (parseTimestamp(item.createdAt) > parseTimestamp(existing.createdAt)) dedupe.set(key, item);
  };
  current.forEach(upsert);
  (incoming as QueuedAnkiMaterial[]).forEach(upsert);
  const merged = Array.from(dedupe.values());
  merged.sort((a, b) => (b.createdAt || '').localeCompare(a.createdAt || ''));
  return merged;
};

export const useAnkiQueueStore = create<AnkiQueueState>()(
  persist(
    (set) => ({
      materials: [],
      addMaterial: (material, options) =>
        set((state) => {
          const existing = state.materials.find((item) => isSameSource(item, material));
          const others = state.materials.filter((item) => !isSameSource(item, material));
          const next = existing
            ? options?.replaceExisting ? [material, ...others] : [existing, ...others]
            : [material, ...others];
          return { materials: next.sort((a, b) => (b.createdAt || '').localeCompare(a.createdAt || '')) };
        }),
      removeMaterial: (queueId) =>
        set((state) => ({ materials: state.materials.filter((item) => item.queueId !== queueId) })),
      clearMaterials: () => set({ materials: [] }),
    }),
    {
      name: STORAGE_KEY,
      storage: createJSONStorage(createTauriPersistStorage),
      merge: (persisted, current) => ({
        ...current,
        materials: mergeMaterials(current.materials, (persisted as AnkiQueueState)?.materials),
      }),
    },
  ),
);

export const createQueueId = () => {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `anki-queue-${Date.now()}-${Math.random().toString(16).slice(2)}`;
};
