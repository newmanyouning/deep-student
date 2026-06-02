/**
 * VFS OCR 存储 API
 *
 * 封装 VFS 统一知识管理架构的 OCR 结果存储功能。
 * OCR 识别结果存储管理，包括存储、查询、删除和导出。
 *
 * ## 主要功能
 * - `vfsOcrStoreResult` - 存储 OCR 识别结果
 * - `vfsOcrListResults` - 列出指定资源的 OCR 结果
 * - `vfsOcrDeleteResult` - 删除指定的 OCR 结果
 * - `vfsOcrMarkExported` - 标记 OCR 结果为已导出
 * - `vfsOcrListForExport` - 列出待导出的 OCR 结果
 *
 * @module vfsOcrStorageApi
 */

import { invoke } from '@tauri-apps/api/core';
import { debugLog } from '../debug-panel/debugMasterSwitch';

const console = debugLog as Pick<typeof debugLog, 'log' | 'warn' | 'error' | 'info' | 'debug'>;

// ============================================================================
// 类型定义
// ============================================================================

/** OCR 存储条目 */
export interface OcrStorageEntry {
  /** 记录 ID */
  id: string;
  /** 关联的资源 ID */
  resource_id: string;
  /** OCR 识别文本 */
  text: string;
  /** 置信度 (0.0 - 1.0) */
  confidence: number;
  /** 来源（如 "paddleocr", "tesseract"） */
  source: string;
  /** 是否已导出 */
  exported: boolean;
  /** 创建时间 */
  created_at: string;
}

// ============================================================================
// API 函数
// ============================================================================

const LOG_PREFIX = '[VfsOcrStorageApi]';

/**
 * 存储 OCR 识别结果
 *
 * 将 OCR 识别结果存入数据库。
 *
 * @param resourceId 资源 ID
 * @param text OCR 识别文本
 * @param confidence 置信度 (0.0 - 1.0)
 * @param source 来源（如 "paddleocr", "tesseract"）
 * @returns 存储的记录 ID
 */
export async function vfsOcrStoreResult(
  resourceId: string,
  text: string,
  confidence: number,
  source: string
): Promise<string> {
  console.log(LOG_PREFIX, 'vfsOcrStoreResult:', { resourceId, confidence, source });

  try {
    const result = await invoke<string>('vfs_ocr_store_result', {
      resourceId,
      text,
      confidence,
      source,
    });

    console.log(LOG_PREFIX, `vfsOcrStoreResult: stored with id ${result}`);
    return result;
  } catch (error: unknown) {
    console.error(LOG_PREFIX, 'vfsOcrStoreResult failed:', error);
    throw error;
  }
}

/**
 * 列出指定资源的 OCR 结果
 *
 * @param resourceId 资源 ID
 * @returns OCR 结果列表
 */
export async function vfsOcrListResults(
  resourceId: string
): Promise<OcrStorageEntry[]> {
  console.log(LOG_PREFIX, 'vfsOcrListResults:', resourceId);

  try {
    const result = await invoke<OcrStorageEntry[]>('vfs_ocr_list_results', {
      resourceId,
    });

    console.log(LOG_PREFIX, `vfsOcrListResults: ${result.length} entries`);
    return result;
  } catch (error: unknown) {
    console.error(LOG_PREFIX, 'vfsOcrListResults failed:', error);
    throw error;
  }
}

/**
 * 删除指定的 OCR 结果
 *
 * @param id 记录 ID
 */
export async function vfsOcrDeleteResult(id: string): Promise<void> {
  console.log(LOG_PREFIX, 'vfsOcrDeleteResult:', id);

  try {
    await invoke<void>('vfs_ocr_delete_result', { id });
    console.log(LOG_PREFIX, 'vfsOcrDeleteResult: success');
  } catch (error: unknown) {
    console.error(LOG_PREFIX, 'vfsOcrDeleteResult failed:', error);
    throw error;
  }
}

/**
 * 标记 OCR 结果为已导出
 *
 * @param id 记录 ID
 */
export async function vfsOcrMarkExported(id: string): Promise<void> {
  console.log(LOG_PREFIX, 'vfsOcrMarkExported:', id);

  try {
    await invoke<void>('vfs_ocr_mark_exported', { id });
    console.log(LOG_PREFIX, 'vfsOcrMarkExported: success');
  } catch (error: unknown) {
    console.error(LOG_PREFIX, 'vfsOcrMarkExported failed:', error);
    throw error;
  }
}

/**
 * 列出待导出的 OCR 结果
 *
 * 返回所有 exported=false 的 OCR 结果。
 *
 * @returns 待导出的 OCR 结果列表
 */
export async function vfsOcrListForExport(): Promise<OcrStorageEntry[]> {
  console.log(LOG_PREFIX, 'vfsOcrListForExport');

  try {
    const result = await invoke<OcrStorageEntry[]>('vfs_ocr_list_for_export');

    console.log(LOG_PREFIX, `vfsOcrListForExport: ${result.length} entries`);
    return result;
  } catch (error: unknown) {
    console.error(LOG_PREFIX, 'vfsOcrListForExport failed:', error);
    throw error;
  }
}
