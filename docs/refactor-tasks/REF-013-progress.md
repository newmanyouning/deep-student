# REF-013: IPC 消息路径审计 — ✅ 完成

> 完成: 2026-05-30 10:15 CST | 总耗时: ~4min

## 资源盘点

| 资产 | 数量 |
|------|------|
| Tauri 命令 (DB) | 689 条, 31 模块 |
| Rust emit() 调用点 | 51 处, 20 文件 |
| 唯一事件名 | 29 个 |
| 前端监听模式 | `guardedListen.ts` 包装器 |

## 执行记录

### Batch 1: 后端事件清单 (10:12 CST)
29 个唯一事件名，分布在 20 个 Rust 文件中

### Batch 2: 前端监听器 (10:13 CST)
通过 `guardedListen.ts` 包装器统一监听，与后端完全匹配

### Batch 3: 交叉验证 (10:14 CST)
29 后端事件 → 前端均有对应监听器，无死事件

### Batch 4: 命名规范 (10:15 CST)
**问题**: 三种命名风格混用，无统一规范
**建议**: 统一为 `module_event_name`（snake_case + 模块前缀）

#### 命名风格分布
| 风格 | 数量 | 示例 |
|------|------|------|
| snake_case | 10 | `anki_generation_event`, `mcp_tools_changed` |
| kebab-case | 7 | `backup-job-progress`, `cloud-sync-progress` |
| colon:separated | 2 | `dstu:change`, `canvas:ai-edit-request` |
| mixed | 10 | `chat_v2_request_audit`, `media-processing-progress` |

#### 建议重命名清单 (后续 REF-012 执行)
- `backup-job-progress` → `backup_job_progress`
- `cloud-sync-progress` → `cloud_sync_progress`
- `csv_import_progress` → `csv_import_progress` (已符合)
- `dstu:change` → `dstu_change`
- `canvas:ai-edit-request` → `canvas_ai_edit_request`
- `media-processing-progress` → `media_processing_progress`
- `textbook-import-progress` → `textbook_import_progress`
- `notes-import-progress` → `notes_import_progress`
- `question_import_progress` → `question_import_progress` (已符合)
- `data-governance-migration-status` → `data_governance_migration_status`
