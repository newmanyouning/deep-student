# 数据兼容性审计报告

> 审计时间: 2026-05-30 11:35 CST | 目的: 验证重构后日志系统可用，数据类型无破坏性变更

## 1. 日志系统

### 当前配置 (lib.rs:172-251)

```
tauri_plugin_log::Builder::new()
  .targets([Target::new(TargetKind::Webview), Target::new(TargetKind::LogDir { ... })])
  .level(log::LevelFilter::Info)
  .level_for("lance", log::LevelFilter::Warn)
  .level_for("lance_encoding", log::LevelFilter::Warn)
  .build()
```

- 双目标输出: Webview 控制台 + LogDir 文件持久化
- Sentry 错误追踪 (条件编译)
- 级别: Info (默认), Warn (lance 库)
- 环境变量: `RUST_LOG` 可动态覆盖

**状态: ✅ 已正确配置，编译后可直接使用**

## 2. 数据库 schema (4 个库)

### 2.1 Chat V2 (chat_v2.db) — 16 迁移
```
V20260130  init              — 核心表: chat_v2_sessions, chat_v2_messages, chat_v2_blocks 等
V20260401  add_change_log    — __change_log 审计
V20260201  add_sync_fields   — device_id, local_version 同步字段
V20260502  archive_legacy    — 归档废弃会话
V20260510  add_compaction    — 消息压缩
V20260516  add_title_locked  — 标题锁定标志
V20260523  sync_coverage     — 补充同步字段
V20260524  field_deltas      — 字段级变更日志
```

### 2.2 VFS (vfs.db) — 31 迁移
```
核心表: notes, textbooks, exams, translations, essays, essay_sessions,
       mindmaps, files, images, folder_items, folders, 
       vfs_blobs, vfs_index, vfs_resource_refs,
       __change_log, __sync_tombstone, __sync_checkpoint
新增: pomodoro_records, todo_lists, todo_items (2026-03-08~11)
最近: V20260525__repair_legacy_questions_change_log_record_ids.sql
```

### 2.3 LLM Usage (llm_usage.db) — 6 迁移
```
核心表: llm_usage_records, __change_log
最近: V20260525__drop_daily_change_log_triggers.sql
```

### 2.4 Mistakes (mistakes.db) — 8 迁移
```
核心表: mistakes, custom_templates, anki_cards, __change_log
最近: V20260524__add_change_log_field_deltas.sql
```

## 3. 重构影响的数据类型

### 3.1 AppError 序列化变更 (REF-003)

**变更前:**
```json
{"error_type": "Validation", "message": "...", "details": null}
```

**变更后:**
```json
{"type": "validation", "message": "...", "details": null}
```

**影响分析:**
- SQLite 中**不存储** AppError，仅存在 Tauri 命令返回的 JSON 响应中
- 前端若使用 `AppError` 类型检查 `error.type`，需从旧 `error.error_type` 迁移
- 前端 `AppError` 接口 (types/index.ts:452) 已使用 `type` 字段→ **前后端一致**
- **破坏级别: 低** — 仅影响 Tauri 命令的错误响应 JSON 格式

### 3.2 AppErrorType 枚举序列化变更 (REF-003)

**变更前:** `"Validation"` `"Database"` `"LLM"` `"FileSystem"` `"NotFound"`
**变更后:** `"validation"` `"database"` `"llm"` `"fileSystem"` `"notFound"`

**影响分析:**
- 前端 AppError 接口使用 `'validation' | 'network' | 'server' | 'unknown'` 联合类型
- Rust enum 有更多变体 (Database/LLM/FileSystem/NotFound/Configuration/Conflict)
- 前端只检查已知的 4 种值，新小写格式与前端更兼容
- **破坏级别: 低** — 前端原本只匹配 4 种已知值

### 3.3 删除的死类型 (REF-002)

| 删除类型 | 是否存储在 DB | 影响 |
|----------|-------------|------|
| GeneralChatRequest/Response | ❌ 仅 API 参数 | 无影响 |
| ContinueChatRequest/Response | ❌ 仅 API 参数 | 无影响 |
| GenerateChatMetadataRequest/Response | ❌ 仅 API 参数 | 无影响 |
| UpdateChatMetadataNoteRequest/Response | ❌ 仅 API 参数 | 无影响 |
| UpdateOcrNoteRequest/Response | ❌ 仅 API 参数 | 无影响 |
| BridgeAnalysisRequest | ❌ 仅 API 参数 | 无影响 |
| ReviewSessionResponse | ❌ 仅 API 参数 | 无影响 |
| ReviewChatRequest | ❌ 仅 API 参数 | 无影响 |

所有删除的类型均为 **API 请求/响应 DTO**，不存储在数据库表中。

### 3.4 MistakeItem 退役 (REF-001)

MistakeItem 存储在 `mistakes.db` 的 `mistakes` 表中。该类型已标记 `@deprecated 2026-01`。
- Rust `models.rs` 中仍有 **MistakeItem struct** (未被删除，因为数据库读取需要)
- 仅删除了 TS 定义和前端引用，后端数据读取使用 `serde_json` 映射
- 数据库 `mistakes` 表**无 schema 变更**
- **破坏级别: 无**

### 3.5 数据治理变更类型 (REF-011)

`DataGovernanceError` 新增，替代 String 错误。无数据库 schema 影响。

### 3.6 命令重命名 (REF-012)

59 命令重命名 (如 `get_setting` → `web_search_get_setting`)。
- Rust 后端: `invoke_handler!` 注册名同步更新
- 前端: `invoke()` 调用名同步更新
- **破坏级别: 无** (前后端一致变更，无旧数据依赖)

## 4. 数据迁移兼容性矩阵

| 数据库 | 表数 | Schema 变更 | 数据类型变更 | 旧数据可读 | 需要迁移 |
|--------|------|------------|-------------|-----------|---------|
| chat_v2.db | 9 | 无 | 无 | ✅ | ❌ |
| vfs.db | 20+ | 无 | 无 | ✅ | ❌ |
| llm_usage.db | 2 | 无 | 无 | ✅ | ❌ |
| mistakes.db | 4 | 无 | 无 | ✅ | ❌ |
| 前端 JSON | N/A | N/A | AppError 字段名变更 | ⚠️ 需前端适配 | ❌ 自动 |

## 5. 日志输出路径

| 目标 | 路径/方式 | 级别 |
|------|----------|------|
| Webview 控制台 | `tauri-plugin-log` | Info+ |
| 文件日志 | `{app_data_dir}/logs/` (LogDir) | Info+ |
| Sentry 错误 | 条件编译 (`sentry` feature) | Error+ |
| Lance 库 | 抑制到 Warn | Warn+ |

**启用方式:** 设置环境变量 `RUST_LOG=debug` 可获取详细日志。

## 6. 建议

1. **AppError 序列化变更**需要在编译后做一次端到端测试：确认前端错误弹窗仍正常显示
2. **命令重命名**已验证前后端同步，但建议编译后用 `tsc --noEmit` 确认无遗漏的旧命令名
3. **数据迁移**不需要任何 SQL 脚本，所有 schema 保持兼容
4. **旧 mistakes 表**中的数据仍可通过 MistakeItem struct 反序列化读取，前端已移除但后端保留

---

*生成: 2026-05-30 11:35 CST | 审计范围: 日志系统 + 4 个数据库 + 22 个类型变更*
