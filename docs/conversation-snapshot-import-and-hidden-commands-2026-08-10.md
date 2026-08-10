# 对话快照导入功能设计 + hidden 命令功能调研

> 日期: 2026-08-10 21:50 CST | 分支: pr/3-pdf-reading
> 来源: 功能精简计划决策点 4 (ImportConversationDialog 意图调研 + 设计文档) 与决策点 5 (hidden 命令调研), 用户指示合写一份
> **状态更新 (2026-08-10): 已实现** — 后端 `snapshot_handlers.rs` 3 命令 (meta/分页 messages/import 全量 ID 重映射), 前端 5 命令接线完成 (export/import/copy-last-response/read-aloud/sync-now 转 ready); 按"有成熟替代才删"标准删除 14 个命令注册, 保留 22 个无替代命令。实现差异: 导出采用 meta+消息分页的分块 invoke (替代单一大 JSON), 导入经前端 fileManager 读文件后单 invoke 提交 (≤32MB)。

---

## 第一部分: 对话快照导入 — 调研结论与功能设计

### 1.1 现状调研 (全部经代码验证)

**这个导入想导入什么**: UI 文案明确 — "请选择通过「导出对话」功能导出的 JSON 文件" (`chat_host.json:33`)。即: **把本软件导出的对话 JSON 在另一处导入恢复**, 是"对话快照的导出→导入闭环"的导入端。

**现状: 闭环两端都不存在, 导入对话框是纯占位**:

| 环节 | 状态 | 证据 |
|------|------|------|
| 导入对话框 UI | ✅ 完整 (文件选择/警告/成功面板) | `ImportConversationDialog.tsx` |
| 前端 API | ❌ **纯占位, 无 invoke**, 固定返回 "导入功能尚未实现" | `systemApi.ts:442-458` |
| 后端命令 | ❌ **完全没有** (连 stub 都不是) | 全库 grep `import_conversation` 零命中 |
| 导出对话命令 | ❌ 不存在 | `chat.export` 命令只派发事件, 无监听者 (`chat.commands.ts:165-176`) |
| 入口 | `DataImportExport.tsx:1846-1871` "导入对话"卡片 → `DSTU_OPEN_IMPORT_CONVERSATION` 事件 → `App.tsx:1005-1011` 打开对话框 | 可达但不是一级入口 |
| 全量备份 | `export_unified_backup_data` 只含设置/API 配置, **不含对话数据** (`commands.rs:3622-3709`) | 对话数据无任何导出途径 |

**意图推断**: 对话是用户的核心学习资产 (含 AI 讲解/制卡上下文), 设计意图是支持 ①跨设备迁移 ②对话分享 ③单会话级备份恢复 — 与全量备份互补的细粒度快照。

### 1.2 功能设计

#### 数据格式 (Conversation Snapshot v1)

```jsonc
{
  "format": "deepstudent-conversation-snapshot",
  "version": 1,
  "exported_at": "2026-08-10T21:50:00+08:00",
  "app_version": "0.9.40",
  "session": { /* chat_v2_sessions 行 (去除本机内部字段) */ },
  "messages": [ /* chat_v2_messages 行, 按 seq 排序 */ ],
  "blocks": [ /* chat_v2_blocks 行, 关联 message_id */ ],
  "attachments": [ /* chat_v2_attachments 元数据; blob 可选内联 base64 或省略 */ ],
  "session_state": { /* chat_v2_session_state, 可选 */ }
}
```

落点: **chat_v2.db** 五表 (schema 见 `migrations/chat_v2/V20260130__init.sql:25-176`)。

#### 后端命令 (2 个新命令)

| 命令 | 职责 | 要点 |
|------|------|------|
| `chat_v2_export_session(session_id, include_attachments?) → { snapshot_json }` | 导出 | 按 session 拉取 messages/blocks/attachments, 序列化为快照 JSON; 前端经 fileManager.saveTextFile 落盘 |
| `chat_v2_import_session(snapshot_json, conflict_strategy?) → { session_id, warnings[] }` | 导入 | ① 校验 format/version; ② **全部 id 重映射** (session/message/block id 生成新 UUID, 防止与库内现有行或同步 change_log 冲突); ③ 事务写入; ④ 返回新 session_id + 警告 (如版本不兼容/附件缺失) |

关键决策:
- **id 必须重映射** — 直接保留原 id 会与 data_governance 同步的 change_log 冲突 (同一 id 在两台设备各自演化)。导入即"新会话副本"
- 导入的会话标题加后缀 "(导入)" 或保留原名由用户选; `created_at` 保留原值, 新增 `imported_at` 元数据
- 附件: v1 省略 blob 内联 (仅元数据 + 缺失警告), v2 再考虑内联
- 同步: 导入的会话作为全新行进入 change_log, 正常参与同步

#### 前端改造

1. `systemApi.importConversationSnapshot` 从占位改为真调用 `chat_v2_import_session`
2. 对话框增加"冲突策略"选项 (默认重映射为新会话)
3. 会话列表 (chat 侧边栏) 加"导出"右键项 + 命令面板 `chat.export` 接入同一导出命令
4. 导入成功后跳转新会话

#### 实施阶段

| 阶段 | 内容 | 量 |
|------|------|----|
| P1 | 导出命令 + 导入命令 (id 重映射 + 校验) + 前端接通 | ~600 行 Rust + ~150 行 TS |
| P2 | 会话列表导出入口 + 命令面板 chat.export/chat.import 激活 | 小 |
| P3 | 附件 blob 内联选项; 批量导出 | 中 |

---

## 第二部分: hidden 命令功能调研 (47 个)

### 2.1 总览

`capabilityRegistry.ts` 中 47 个 `'hidden'` 命令 (learning 19 / chat 12 / global 16), 用户从面板不可见。**实质: 44 个是纯占位 (只派发无人监听的事件), 3 个有真实监听者。**

### 2.2 有真实实现的 3 个 (应转 ready 或保留 hidden 但可用)

| 命令 | 实现 | 证据 |
|------|------|------|
| `global.shortcut-settings` | 跳设置-shortcuts 标签 | `App.tsx:1388-1400` |
| `chat.voice-input` | 语音输入开关 | `voice-input/hooks.ts:31-62` |
| `chat.toggle-learn-mode` |  stub: 提示"已迁移到 Skills, 用 /skill tutor-mode" | `chat.commands.ts:257-258` |

### 2.3 纯占位命令分组 (44 个, 事件零监听)

| 组 | 命令 | 意图 | 建议处置 |
|----|------|------|----------|
| 复习控制 | learning.start-review/pause-review/next-item/show-answer/schedule-review/mark-mastered (6) | 复习会话的键盘控制 | ⚠️ 与归档的 review_plan 功能同源 — **若未来恢复复习计划, 这组命令是其键盘层**, 建议随复习功能去留 |
| 学习激励 | learning.achievements/streak/daily-goal/show-progress/statistics/calendar/history/export-progress (8) | 成就/打卡/统计 | 游戏化系统, 无后端; 删 |
| 阅读辅助 | learning.read-aloud/focus-mode/take-notes/highlight (4) | TTS/专注/笔记/高亮 | TTS 后端在 (tts.rs); read-aloud 可接线, 余删 |
| 翻译快捷 | learning.translate-selection/switch-language-pair (2) | 选区翻译 | 翻译工作台已是一等资源; 删 |
| 对话内容 | chat.copy-last-response/share/export/import (4) | 复制/分享/导出/导入 | export/import 见第一部分设计 (P2 激活); copy-last-response 可简单实现; share 需分享服务, 删 |
| 对话进阶 | chat.ai-continue/quick-prompt/multi-turn-edit/branch-conversation (4) | AI 续写/快捷提示/编辑重生成/分支 | multi-turn-edit 与 branch 在 chat-v2 已有 UI 变体功能 (variant handlers); 其余删 |
| 对话设置 | chat.model-settings/show-history (2) | 模型参数/历史 | 均已有 UI 入口; 删 |
| 全局搜索 | global.quick-search (1) | 全局搜索 | learning-hub 有搜索; 命令面板本身即搜索; 删 |
| 全局设置 | global.theme-system/toggle-notifications/mute-sounds (3) | 主题/通知/声音 | 设置页均有; 删 |
| 网络 | global.check-connection/sync-now (2) | 连接检查/手动同步 | sync-now 可接 data_governance; check-connection 删 |
| 剪贴板 | global.paste-from-clipboard (1) | 读取剪贴板 | 删 (输入框原生支持) |
| 帮助 | global.show-help/about/changelog/report-bug (4) | 帮助/关于/日志/报错 | about 设置页有; 余删 |
| 数据 | global.export-all/import-data (2) | 全量导出/导入 | 数据治理页已有; 删 |
| 安全 | global.lock-app (1) | 应用锁 | 无后端; 删 |
| 状态 | global.show-loading (1) | 任务指示 | 删 |

### 2.4 处置建议汇总

- **删注册 (约 36 个)**: 无实现且无独特价值的占位
- **接线激活 (约 5 个)**: `chat.export`/`chat.import` (本设计 P2), `chat.copy-last-response`, `learning.read-aloud` (TTS 已有后端), `global.sync-now`
- **随复习功能去留 (6 个)**: learning 复习控制组 — 与 `_archive` 的 review_plan UI 同命运
- **已可用 3 个**: shortcut-settings / voice-input / toggle-learn-mode (转 ready 或维持)

> 注意: `learning.commands.ts` 的 22 个 `TODO: 未实现` 事件常量中, `OPEN_TRANSLATE`/`OPEN_ESSAY_GRADING` 已接入 `useLearningHubEvents.ts:172-173` — 清理时这两个常量保留。

---

> 调研: 2 个并行代理 + 主会话复核; 全部结论带文件:行号, 见文内引用
