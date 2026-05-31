/**
 * Settings API — 统一 settings 读写操作
 *
 * P1-04 (2026-05-30): 130+ 文件直接调用 invoke('save_setting')/invoke('get_setting')
 * 现在通过此模块集中化，替代分散的 invoke() 调用。
 */

import { invoke } from '@tauri-apps/api/core';

/** 读取设置值 */
export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>('get_setting', { key });
}

/** 保存设置值 */
export async function saveSetting(key: string, value: string): Promise<void> {
  await invoke<void>('save_setting', { key, value });
}

/** 删除设置值 */
export async function deleteSetting(key: string): Promise<void> {
  await invoke<void>('delete_setting', { key });
}

/** 按前缀读取设置 */
export async function getSettingsByPrefix(prefix: string): Promise<Record<string, string>> {
  return invoke<Record<string, string>>('get_settings_by_prefix', { prefix });
}

export const settingsApi = {
  get: getSetting,
  save: saveSetting,
  delete: deleteSetting,
  getByPrefix: getSettingsByPrefix,
} as const;
