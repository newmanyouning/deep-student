/**
 * 统一资源类型（前端共享定义）
 *
 * 与 src-tauri/src/vfs/resource_kind.rs 的 ResourceKind 枚举一一对应。
 * 11 个小写字符串是持久化契约（数据库列、云端 payload 按原值读写），
 * 序列化值一个都不能改；别名只允许进入 parseResourceKind，禁止进入联合类型。
 *
 * 本文件是前端资源类型的唯一事实源，各模块的字符串字面量联合应逐步收敛至此。
 */

// ============================================================================
// 资源类型联合（11 值，持久化契约）
// ============================================================================

/**
 * 统一资源类型（11 个值，禁止改名）
 *
 * | 值         | 描述             |
 * |------------|------------------|
 * | note       | 笔记             |
 * | textbook   | 教材             |
 * | exam       | 题目集识别       |
 * | translation| 翻译             |
 * | essay      | 作文批改         |
 * | image      | 图片             |
 * | file       | 文件附件         |
 * | retrieval  | 检索结果（RAG）  |
 * | mindmap    | 知识导图         |
 * | card       | 题目卡片快照     |
 * | folder     | 文件夹（虚拟节点）|
 */
export type ResourceKind =
  | 'note'
  | 'textbook'
  | 'exam'
  | 'translation'
  | 'essay'
  | 'image'
  | 'file'
  | 'retrieval'
  | 'mindmap'
  | 'card'
  | 'folder';

/**
 * 全部资源类型（11 个值，顺序与后端 ResourceKind::all() 一致）
 */
export const RESOURCE_KIND_VALUES: readonly ResourceKind[] = [
  'note',
  'textbook',
  'exam',
  'translation',
  'essay',
  'image',
  'file',
  'retrieval',
  'mindmap',
  'card',
  'folder',
] as const;

// ============================================================================
// 宽容解析（对齐后端 ResourceKind::from_str）
// ============================================================================

/**
 * 从字符串解析资源类型（宽容层，对齐后端 from_str）
 *
 * 合并全部现有别名语义（单数/复数/中文），外加标准 11 个 lowercase 值：
 * - `attachment`/`attachments`/`附件` → image（附件按图片附件语义处理；
 *   文档附件以 `att_` 前缀 ID 走 file，见 resourceKindFromIdPrefix）
 * - `img` → image（图片的常见缩写）
 *
 * @param s 类型字符串（如 "note", "notes", "教材"）
 * @returns 解析成功返回 ResourceKind，失败返回 null
 */
export function parseResourceKind(s: string): ResourceKind | null {
  switch (s.toLowerCase()) {
    // 标准 11 个 lowercase 值
    case 'note': return 'note';
    case 'textbook': return 'textbook';
    case 'exam': return 'exam';
    case 'translation': return 'translation';
    case 'essay': return 'essay';
    case 'image': return 'image';
    case 'file': return 'file';
    case 'retrieval': return 'retrieval';
    case 'mindmap': return 'mindmap';
    case 'card': return 'card';
    case 'folder': return 'folder';
    // 别名（复数 + 中文）
    case 'notes':
    case '笔记': return 'note';
    case 'textbooks':
    case '教材': return 'textbook';
    case 'exams':
    case '题目集':
    case '试卷': return 'exam';
    case 'translations':
    case '翻译': return 'translation';
    case 'essays':
    case '作文':
    case '作文批改': return 'essay';
    case 'images':
    case '图片':
    case 'img': return 'image';
    case 'files':
    case '文件': return 'file';
    case 'retrievals':
    case '检索':
    case '检索结果': return 'retrieval';
    case 'mindmaps':
    case '知识导图':
    case '导图': return 'mindmap';
    case 'folders':
    case '文件夹': return 'folder';
    // 附件类型映射到 image（沿用 DSTU 现有语义）
    case 'attachment':
    case 'attachments':
    case '附件': return 'image';
    default: return null;
  }
}

// ============================================================================
// ID 前缀推断（对齐后端 ResourceKind::from_id_prefix）
// ============================================================================

/**
 * 从资源 ID 前缀推断资源类型（对齐后端 from_id_prefix）
 *
 * 说明：
 * - `img_` → image（修复：现状被归为 file）
 * - `att_` → file（attachment 是遗留 file 实体）
 * - `essay_session_` / `es_` → essay
 * - `card_` → card（题目卡片持久化 ID 格式 `card_{nanoid}`）
 * - retrieval 无专用前缀（VFS resources 表通用 `res_` ID 不携带类型信息），
 *   未知前缀一律返回 null
 *
 * @param id 资源 ID（如 "note_abc123", "tb_xyz"）
 * @returns 推断成功返回 ResourceKind，无法识别返回 null
 */
export function resourceKindFromIdPrefix(id: string): ResourceKind | null {
  if (id.startsWith('note_')) {
    return 'note';
  } else if (id.startsWith('tb_')) {
    return 'textbook';
  } else if (id.startsWith('file_') || id.startsWith('att_')) {
    // att_ 是遗留 file 实体
    return 'file';
  } else if (id.startsWith('img_')) {
    return 'image';
  } else if (id.startsWith('exam_')) {
    return 'exam';
  } else if (id.startsWith('tr_')) {
    return 'translation';
  } else if (id.startsWith('essay_session_') || id.startsWith('essay_') || id.startsWith('es_')) {
    return 'essay';
  } else if (id.startsWith('fld_')) {
    return 'folder';
  } else if (id.startsWith('mm_')) {
    return 'mindmap';
  } else if (id.startsWith('card_')) {
    return 'card';
  } else {
    return null;
  }
}

// ============================================================================
// 复数路径段（对齐后端 ResourceKind::to_path_segment）
// ============================================================================

/**
 * 转换为路径段字符串（复数形式，对齐后端 to_path_segment）
 *
 * 用于构建 DSTU 路径，如 "/高考复习/notes"
 *
 * @param k 资源类型
 * @returns 复数路径段，如 "notes", "textbooks", "cards"
 */
export function resourceKindToPathSegment(k: ResourceKind): string {
  switch (k) {
    case 'note': return 'notes';
    case 'textbook': return 'textbooks';
    case 'exam': return 'exams';
    case 'translation': return 'translations';
    case 'essay': return 'essays';
    case 'image': return 'images';
    case 'file': return 'files';
    case 'retrieval': return 'retrievals';
    case 'mindmap': return 'mindmaps';
    case 'card': return 'cards';
    case 'folder': return 'folders';
  }
}
