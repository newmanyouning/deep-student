/**
 * Chat V2 API — 统一 Chat V2 命令调用
 *
 * P1-04 (2026-05-30): chat_v2 域中有 ~50 处分散的 invoke() 调用
 * 此模块提供统一的 API 入口，替代 TauriAdapter 之外的直接调用。
 */

import { invoke } from '@tauri-apps/api/core';

// Session management
export async function deleteSession(sessionId: string) { await invoke('chat_v2_delete_session', { sessionId }); }
export async function updateSessionSettings(sessionId: string, settings: Record<string, unknown>) { await invoke('chat_v2_update_session_settings', { sessionId, settings }); }
export async function archiveSession(sessionId: string) { await invoke('chat_v2_archive_session', { sessionId }); }
export async function saveSession(sessionId: string) { await invoke('chat_v2_save_session', { sessionId }); }

// Streaming
export async function upsertStreamingBlock(blockId: string, messageId: string, sessionId: string | null, blockType: string, content: string, status: string | null) { await invoke('chat_v2_upsert_streaming_block', { blockId, messageId, sessionId, blockType, content, status }); }
export async function updateBlockToolOutput(blockId: string, toolOutputJson: string) { await invoke('chat_v2_update_block_tool_output', { blockId, toolOutputJson }); }
export async function cancelStream(sessionId: string, messageId: string) { await invoke('chat_v2_cancel_stream', { sessionId, messageId }); }

// Tags
export async function addTag(sessionId: string, tag: string) { await invoke('chat_v2_add_tag', { sessionId, tag }); }
export async function removeTag(sessionId: string, tag: string) { await invoke('chat_v2_remove_tag', { sessionId, tag }); }

// Groups
export async function reorderGroups(sessionId: string, groupIds: string[]) { await invoke('chat_v2_reorder_groups', { sessionId, groupIds }); }

// Variants
export async function askUserRespond(sessionId: string, response: string) { await invoke('chat_v2_ask_user_respond', { sessionId, response }); }
export async function toolApprovalRespond(sessionId: string, approved: boolean) { await invoke('chat_v2_tool_approval_respond', { sessionId, approved }); }

// ==================== Conversation Snapshot (export/import) ====================

/** 快照元信息（chat_v2_export_session_meta 返回，camelCase） */
export interface SnapshotExportMeta {
  format: string;
  version: number;
  exportedAt: string;
  appVersion: string;
  session: Record<string, unknown>;
  sessionState: Record<string, unknown> | null;
  messageCount: number;
  pageSize: number;
}

/** 消息分块（chat_v2_export_session_messages 返回） */
export interface SnapshotMessagesChunk {
  messages: Array<Record<string, unknown>>;
  blocks: Array<Record<string, unknown>>;
  nextOffset: number | null;
}

/** 导入结果（chat_v2_import_session 返回） */
export interface ImportSessionSnapshotResult {
  sessionId: string;
  messageCount: number;
  blockCount: number;
  warnings: string[];
}

export async function exportSessionMeta(sessionId: string) {
  return invoke<SnapshotExportMeta>('chat_v2_export_session_meta', { sessionId });
}
export async function exportSessionMessages(sessionId: string, offset: number, limit?: number) {
  return invoke<SnapshotMessagesChunk>('chat_v2_export_session_messages', { sessionId, offset, limit });
}
export async function importSessionSnapshot(snapshotJson: string) {
  return invoke<ImportSessionSnapshotResult>('chat_v2_import_session', { snapshotJson });
}

export const chatV2Api = {
  deleteSession, updateSessionSettings, archiveSession, saveSession,
  upsertStreamingBlock, updateBlockToolOutput, cancelStream,
  addTag, removeTag, reorderGroups,
  askUserRespond, toolApprovalRespond,
  exportSessionMeta, exportSessionMessages, importSessionSnapshot,
} as const;
