/**
 * 通用工具函数
 */

/**
 * 防抖函数
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  
  return function debounced(...args: Parameters<T>) {
    if (timeout) {
      clearTimeout(timeout);
    }
    
    timeout = setTimeout(() => {
      func(...args);
      timeout = null;
    }, wait);
  };
}

let _idCounter = 0;
const ID_COUNTER_RESET = 1_000_000;

/**
 * 生成唯一 ID，可选前缀。
 * 合并自 createChatStore::generateId — 现在作为全局统一实现 (REF-005)。
 */
export function generateId(prefix?: string): string {
  const ts = Date.now().toString(36);
  const rnd = Math.random().toString(36).substring(2, 8);
  _idCounter = (_idCounter + 1) % ID_COUNTER_RESET;
  const ctr = _idCounter.toString(36);
  const body = `${ts}_${rnd}_${ctr}`;
  return prefix ? `${prefix}_${body}` : body;
}

/**
 * 格式化文件大小
 */
export function formatFileSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;
  
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  
  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

/**
 * 格式化时间戳
 */
export function formatTimestamp(timestamp: string | number): string {
  const date = new Date(timestamp);
  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
}