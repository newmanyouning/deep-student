/**
 * 对话快照导出助手
 *
 * 设计文档: docs/conversation-snapshot-import-and-hidden-commands-2026-08-10.md
 *
 * 分块导出: meta 命令拿会话行与分页参数, 然后按页拉取 messages+blocks,
 * 前端拼装为完整快照 JSON 后经 fileManager 落盘。
 * 多模型返回结果 (variants_json) 随消息行原样带出。
 */

import i18next from 'i18next';
import { exportSessionMeta, exportSessionMessages } from '@/api/chatV2Api';
import { fileManager } from '@/utils/fileManager';
import { showGlobalNotification } from '@/components/UnifiedNotification';
import { getErrorMessage } from '@/utils/errorUtils';

/**
 * 导出会话为快照 JSON 文件
 *
 * @param sessionId 会话 ID
 * @param title 会话标题（用于生成默认文件名）
 * @returns 是否真正保存（用户取消返回 false）
 */
export async function exportSessionToSnapshotFile(
  sessionId: string,
  title?: string
): Promise<boolean> {
  try {
    const meta = await exportSessionMeta(sessionId);

    // 分块拉取全部消息与块
    const messages: Array<Record<string, unknown>> = [];
    const blocks: Array<Record<string, unknown>> = [];
    let offset = 0;
    for (;;) {
      const chunk = await exportSessionMessages(sessionId, offset, meta.pageSize);
      messages.push(...chunk.messages);
      blocks.push(...chunk.blocks);
      if (chunk.nextOffset == null) break;
      offset = chunk.nextOffset;
    }

    const snapshot = {
      format: meta.format,
      version: meta.version,
      exportedAt: meta.exportedAt,
      appVersion: meta.appVersion,
      session: meta.session,
      sessionState: meta.sessionState,
      messages,
      blocks,
    };

    const safeName = (title || 'conversation')
      .replace(/[\\/:*?"<>|]/g, '_')
      .trim()
      .slice(0, 60) || 'conversation';

    const result = await fileManager.saveTextFile({
      title: i18next.t('command_palette:commands.chat.export', 'Export Conversation'),
      defaultFileName: `${safeName}.snapshot.json`,
      content: JSON.stringify(snapshot),
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });

    if (result.canceled) return false;

    showGlobalNotification(
      'success',
      i18next.t('chatV2:exportSession.success', '对话已导出'),
      i18next.t('chatV2:exportSession.successDetail', {
        count: messages.length,
        defaultValue: '已导出 {{count}} 条消息',
      })
    );
    return true;
  } catch (error: unknown) {
    console.error('[exportSessionSnapshot] Export failed:', error);
    showGlobalNotification(
      'error',
      i18next.t('chatV2:exportSession.failed', '导出对话失败'),
      getErrorMessage(error)
    );
    return false;
  }
}
