/**
 * OCR 逐页阅读 — PDF ↔ MD 页码同步模块
 *
 * 解决 PDF 查看器和 MD 笔记阅读器之间的页码同步问题：
 * 1. PDF 翻页 → 打开 MD 笔记 → 直接跳到对应页
 * 2. MD 翻页 → 切回 PDF → PDF 同步到对应页
 * 3. 追踪上次打开页面，下次恢复
 *
 * ## 页码记录数据结构
 *
 * PdfPageRecord 记录完整翻页信息，确保在 sessionStorage / DSTU / 事件
 * 三条路径上都携带一致的 page + timestamp + source 数据。
 */

// ======== 页码记录参数 ========

/**
 * PDF 页码记录参数 — 所有翻页操作携带此结构，
 * 确认为准记录翻页来源和发生时间。
 */
export interface PdfPageRecord {
  /** 页码（1-based） */
  page: number;
  /** 翻页时间戳（Unix 毫秒） */
  timestamp: number;
  /** 翻页来源 */
  source: 'pdf' | 'md' | 'restore' | 'user';
  /** 文件 ID（PDF 的 sourceId） */
  fileId: string;
}

/** 创建页码记录 */
export const createPageRecord = (
  page: number, fileId: string, source: PdfPageRecord['source'] = 'user',
): PdfPageRecord => ({
  page, fileId, source,
  timestamp: Date.now(),
});

// ======== 初始页码传递（跨组件，不经事件链） ========

let pendingOcrPage: number | null = null;
let pendingOcrFileId: string | null = null;

/** TextbookContentView 调用：设置下次 NoteContentView 打开时的起始页码 */
export const setPendingOcrPage = (fileId: string, page: number) => {
  pendingOcrPage = page;
  pendingOcrFileId = fileId;
};

/** NoteContentView 调用：消费已设置的起始页码（仅消费一次） */
export const consumePendingOcrPage = (fileId: string): number | null => {
  if (pendingOcrFileId === fileId && pendingOcrPage !== null) {
    const page = pendingOcrPage;
    pendingOcrPage = null;
    pendingOcrFileId = null;
    return page;
  }
  return null;
};

// ======== 双向同步事件 ========

export interface OcrPageSyncEvent {
  fileId: string;
  pageNumber: number;
  source: 'pdf' | 'md';
}

export const OCR_PAGE_SYNC_EVENT = 'ocr:page-sync';

export const dispatchOcrPageSync = (detail: OcrPageSyncEvent) => {
  window.dispatchEvent(new CustomEvent<OcrPageSyncEvent>(OCR_PAGE_SYNC_EVENT, { detail }));
};

// ======== 页码持久化（sessionStorage） ========

const OCR_LAST_KEY_PREFIX = 'ocr_last_v2:';
const OCR_RECORD_KEY_PREFIX = 'ocr_record:';

/** 上次保存的页码（防重复写入） */
const lastWrittenPage = new Map<string, number>();

/**
 * 保存页码记录（完整结构 + 简单页码双写）
 * 简单页码用于快速读取（向后兼容），完整记录用于精确追踪。
 */
export const savePageRecord = (record: PdfPageRecord) => {
  try {
    const pageKey = OCR_LAST_KEY_PREFIX + record.fileId;
    const recordKey = OCR_RECORD_KEY_PREFIX + record.fileId;

    // 防重复：相同页面不重复写
    const prev = lastWrittenPage.get(record.fileId);
    if (prev === record.page) return;
    lastWrittenPage.set(record.fileId, record.page);

    // 双写：简单页码 + 完整记录
    sessionStorage.setItem(pageKey, String(record.page));
    sessionStorage.setItem(recordKey, JSON.stringify(record));
  } catch { /* sessionStorage 不可用 */ }
};

/** 获取保存的页码记录（返回完整结构，回退简单页码） */
export const getPageRecord = (fileId: string): PdfPageRecord | null => {
  try {
    const recordKey = OCR_RECORD_KEY_PREFIX + fileId;
    const raw = sessionStorage.getItem(recordKey);
    if (raw) {
      const parsed = JSON.parse(raw) as PdfPageRecord;
      if (parsed && typeof parsed.page === 'number' && parsed.page > 0) {
        return parsed;
      }
    }
    // 回退：从简单页码构造记录
    const pageKey = OCR_LAST_KEY_PREFIX + fileId;
    const val = sessionStorage.getItem(pageKey);
    if (val) {
      const page = parseInt(val, 10);
      if (page > 0) {
        return createPageRecord(page, fileId, 'restore');
      }
    }
    return null;
  } catch { return null; }
};

/**
 * 轻量保存 + 读取（向后兼容旧接口）
 */
export const saveLastOcrPage = (fileId: string, page: number) => {
  savePageRecord(createPageRecord(page, fileId, 'user'));
};

export const getLastOcrPage = (fileId: string): number | null => {
  const record = getPageRecord(fileId);
  return record?.page ?? null;
};
