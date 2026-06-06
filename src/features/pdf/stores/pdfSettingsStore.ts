/**
 * PDF 阅读器设置 Store
 *
 * 提供 PDF 阅读器的性能和功能设置，支持持久化到 localStorage
 */

import { create } from 'zustand';
import { subscribeWithSelector, persist } from 'zustand/middleware';
import { createThrottledStorage } from '@/utils/throttledStorage';

/** PDF 阅读器设置接口 */
export interface PdfSettings {
  // ========== 渲染性能 ==========
  /** 最大设备像素比（1.0-3.0，默认 1.5） */
  maxDevicePixelRatio: number;
  /** 滚动时降级渲染（滚动中使用低 DPR，停止后恢复） */
  enableScrollDprDowngrade: boolean;
  /** 滚动降级时的 DPR（默认 1.0） */
  scrollDpr: number;
  /** 虚拟化 overscan 行数（默认 2） */
  virtualizerOverscan: number;

  // ========== 文本层 ==========
  /** 默认启用文本选择（关闭可提升性能） */
  enableTextLayerByDefault: boolean;
  /** 文本层范围：仅在当前页 ± N 页启用（0 = 仅当前页） */
  textLayerRange: number;

  // ========== 批注层 ==========
  /** 默认启用批注层 */
  enableAnnotationLayerByDefault: boolean;
  /** 批注层范围：仅在当前页 ± N 页启用 */
  annotationLayerRange: number;

  // ========== 缩略图 ==========
  /** 缩略图宽度（px） */
  thumbnailWidth: number;
  /** 缩略图 DPR */
  thumbnailDpr: number;
  /** 缩略图 overscan */
  thumbnailOverscan: number;

  // ========== 默认视图 ==========
  /** 默认缩放比例（0.5-3.0） */
  defaultScale: number;
  /** 默认视图模式 */
  defaultViewMode: 'single' | 'dual';
}

/** 默认设置 */
export const DEFAULT_PDF_SETTINGS: PdfSettings = {
  // 渲染性能
  maxDevicePixelRatio: 1.5,
  enableScrollDprDowngrade: true,
  scrollDpr: 1.0,
  virtualizerOverscan: 2,

  // 文本层
  enableTextLayerByDefault: true, // 默认启用，支持扫描 PDF 文本选择
  textLayerRange: 1,

  // 批注层
  enableAnnotationLayerByDefault: false, // 默认关闭
  annotationLayerRange: 0,

  // 缩略图
  thumbnailWidth: 100,
  thumbnailDpr: 1.0,
  thumbnailOverscan: 4,

  // 默认视图
  defaultScale: 1.0,
  defaultViewMode: 'single',
};

/** 数值设置的合法范围约束（模块级常量，避免每次调用重建） */
const PDF_SETTING_CONSTRAINTS: Record<string, [number, number]> = {
  maxDevicePixelRatio: [1.0, 3.0],
  scrollDpr: [0.5, 3.0],
  virtualizerOverscan: [1, 6],
  textLayerRange: [0, 5],
  annotationLayerRange: [0, 5],
  thumbnailWidth: [60, 160],
  thumbnailDpr: [0.5, 2.0],
  thumbnailOverscan: [1, 10],
  defaultScale: [0.5, 3.0],
};

interface PdfSettingsStore {
  settings: PdfSettings;

  /** 更新单个设置 */
  updateSetting: <K extends keyof PdfSettings>(key: K, value: PdfSettings[K]) => void;

  /** 批量更新设置 */
  updateSettings: (partial: Partial<PdfSettings>) => void;

  /** 重置为默认设置 */
  resetSettings: () => void;

  /** 获取当前渲染 DPR（考虑滚动状态） */
  getRenderDpr: (isScrolling: boolean) => number;
}

export const usePdfSettingsStore = create<PdfSettingsStore>()(
  subscribeWithSelector(
    persist(
      (set, get) => ({
        settings: { ...DEFAULT_PDF_SETTINGS },

        updateSetting: (key, value) => {
          let validated = value;
          if (typeof value === 'number') {
            const range = PDF_SETTING_CONSTRAINTS[key as string];
            if (range) {
              validated = Math.max(range[0], Math.min(range[1], value)) as typeof value;
            }
          }
          set((state) => ({
            settings: { ...state.settings, [key]: validated },
          }));
        },

        updateSettings: (partial) => {
          // 🔒 审计修复: 批量更新也需要验证数值范围约束
          // 原代码绕过了 PDF_SETTING_CONSTRAINTS 验证
          const validated = { ...partial };
          for (const [key, value] of Object.entries(validated)) {
            if (typeof value === 'number') {
              const range = PDF_SETTING_CONSTRAINTS[key];
              if (range) {
                (validated as Record<string, unknown>)[key] = Math.max(range[0], Math.min(range[1], value));
              }
            }
          }
          set((state) => ({
            settings: { ...state.settings, ...validated },
          }));
        },

        resetSettings: () => {
          set({ settings: { ...DEFAULT_PDF_SETTINGS } });
        },

        getRenderDpr: (isScrolling) => {
          const { settings } = get();
          if (isScrolling && settings.enableScrollDprDowngrade) {
            return Math.min(settings.scrollDpr, settings.maxDevicePixelRatio);
          }
          const deviceDpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
          return Math.min(deviceDpr, settings.maxDevicePixelRatio);
        },
      }),
      {
        name: 'pdf-settings',
        version: 1,
        partialize: (state) => ({ settings: state.settings }),
        storage: createThrottledStorage() as any,
      }
    )
  )
);

/** 便捷 Hook：直接获取设置对象 */
export const usePdfSettings = () => usePdfSettingsStore((s) => s.settings);

/** 便捷 Hook：获取单个 action（避免不必要的重渲染） */
export const usePdfUpdateSetting = () => usePdfSettingsStore((s) => s.updateSetting);
export const usePdfResetSettings = () => usePdfSettingsStore((s) => s.resetSettings);
export const usePdfGetRenderDpr = () => usePdfSettingsStore((s) => s.getRenderDpr);

