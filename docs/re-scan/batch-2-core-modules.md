# Batch 2: Rust 核心业务模块 — 重新扫描报告

> 扫描时间: 2026-05-30 15:45 CST | 80+30+10=120 文件 | 状态: ✅ 完成

## 2.1 chat_v2/handlers/ — 14 文件, 80 Tauri 命令

| Handler 文件 | 命令数 | 前缀 | 命名符合 |
|-------------|--------|------|---------|
| `approval_handlers.rs` | 3 | `chat_v2_` | ✅ |
| `ask_user_handlers.rs` | 1 | `chat_v2_` | ✅ |
| `block_actions.rs` | 7 | `chat_v2_` | ✅ (文件名例外) |
| `canvas_handlers.rs` | 1 | `chat_v2_` | ✅ |
| `group_handlers.rs` | 7 | `chat_v2_` | ✅ |
| `load_session.rs` | 1 | `chat_v2_` | ✅ |
| `manage_session.rs` | 14 | `chat_v2_` | ✅ |
| `migration.rs` | 3 | `chat_v2_` | ✅ |
| `ocr.rs` | 1 | `chat_v2_` | ✅ |
| `search_handlers.rs` | 6 | `chat_v2_` | ✅ |
| `send_message.rs` | 5 | `chat_v2_` | ✅ |
| `variant_handlers.rs` | 5 | `chat_v2_` | ✅ |
| `workspace_handlers.rs` | 18 | `workspace_*` | ❌ |
| `resource_handlers.rs` | 8 | `resource_*` | ❌ 已废弃 |

### 命名冲突

| ID | 严重度 | 位置 | 问题 | 建议 |
|----|--------|------|------|------|
| N2-01 | **HIGH** | `workspace_handlers.rs` | 18 命令用 `workspace_*` 而非 `chat_v2_*` | 统一为 `chat_v2_workspace_*` 或独立为顶层模块 |
| N2-02 | **MEDIUM** | 6 文件 | 文件名混用 `_handler.rs` 后缀 vs 裸名 | 统一为 `{name}_handlers.rs` |
| N2-03 | **MEDIUM** | `block_actions.rs` | 唯一使用 `_actions` 模式的文件 | 重命名为 `block_handlers.rs` |

### 跨 Handler 依赖
- `block_actions.rs` → `manage_session.rs::rebuild_session_skill_state_from_surviving_history`
- `variant_handlers.rs` → `send_message.rs::apply_original_skill_snapshot_overrides`

## 2.2 vfs/ — 5 handler 文件

| Handler | 命令数 | 前缀 | 符合 |
|---------|--------|------|------|
| `handlers.rs` | 87+ | `vfs_` | ✅ |
| `ref_handlers.rs` | 3 | `vfs_` | ✅ |
| `index_handlers.rs` | 7 | `vfs_` | ✅ |
| `todo_handlers.rs` | ~30 | `todo_*`/`pomodoro_*` | ❌ |
| `ocr_storage_handlers.rs` | 5 | `ocr_*` | ❌ |

| ID | 严重度 | 位置 | 问题 |
|----|--------|------|------|
| N2-04 | **HIGH** | `todo_handlers.rs` | 30+ 命令缺 `vfs_` 前缀 |
| N2-05 | **HIGH** | `ocr_storage_handlers.rs` | 5 命令缺 `vfs_` 前缀 |

## 2.3 dstu/ — 3 handler 文件 ✅

所有命令使用 `dstu_*` 前缀，文件名统一为 `{name}_handlers.rs`。最一致的模块。

---

*Batch 2 完成。文件: 120 | 冲突: 5 (N2-01..N2-05)*
