# API 重构实施计划

**日期**: 2026-05-29
**最后更新**: 2026-05-29 (执行中)
**进度**: ~148/697 命令 (21.2%)
**原则**: 最小改动、向后兼容、单文件修改、逐步提交

---

## 实施策略

由于 Rust 模块间引用复杂（移动文件需要更新所有 `use` 路径），采用**逻辑拆分 + 物理逐步迁移**策略：

| 阶段 | 改动 | 风险 | 状态 |
|------|------|------|------|
| 1 | 添加错误类型（新增代码，不修改现有逻辑） | 无 | ✅ 核心模块完成 |
| 2 | 封装 MemoryContext（减少 State 参数） | 低 | ❌ 未开始 |
| 3 | 重命名 lib.rs 注册路径（逻辑拆分） | 中 | ❌ 未开始 |
| 4 | 物理移动文件 | 高（需全量测试） | ❌ 未开始 |

---

## 阶段 1 执行状态 (2026-05-29)

### 已完成模块

| 模块 | 文件 | 命令 | 错误类型 | 状态 |
|------|------|------|---------|------|
| Essay Grading | essay_grading/error.rs, mod.rs | 20 | EssayGradingError | ✅ |
| Memory | memory/error.rs, handlers.rs | 27 | MemoryError | ✅ |
| VFS Todo/Pomodoro | vfs/todo_handlers.rs | 25 | VfsError + Serialize | ✅ |
| Review Plan | review_plan_service.rs | 17 | ReviewPlanError | ✅ |
| DSTU folder | dstu/folder_handlers.rs | 14 | DstuError | ✅ |
| DSTU trash | dstu/trash_handlers.rs | 5 | DstuError | ✅ (已有) |
| Chat V2 variant | chat_v2/handlers/variant_handlers.rs | 8 | ChatV2Error | ✅ (已有) |
| Chat V2 search | chat_v2/handlers/search_handlers.rs | 3 | ChatV2Error | ✅ |
| Chat V2 OCR | chat_v2/handlers/ocr.rs | 1 | ChatV2Error | ✅ |
| Chat V2 ask_user | chat_v2/handlers/ask_user_handlers.rs | 1 | ChatV2Error | ✅ |
| Chat V2 canvas | chat_v2/handlers/canvas_handlers.rs | 1 | ChatV2Error | ✅ |
| Chat V2 approval | chat_v2/handlers/approval_handlers.rs | 3 | ChatV2Error | ✅ |
| Chat V2 group | chat_v2/handlers/group_handlers.rs | 7 | ChatV2Error | ✅ |
| Chat V2 load_session | chat_v2/handlers/load_session.rs | 1 | ChatV2Error | ✅ |
| Chat V2 manage_session | chat_v2/handlers/manage_session.rs | 14 | ChatV2Error | ✅ |
| DSTU error | dstu/error.rs | — | +From<VfsError> +From<JoinError> | ✅ |
| VFS error | vfs/error.rs | — | +Serialize derive | ✅ |

### 待完成模块 (按优先级)

| 优先级 | 模块 | 文件 | 命令 | 状态 |
|--------|------|------|------|------|
| P1 | Chat V2 | handlers/block_actions.rs | ~6 | ChatV2Error 已就绪 |
| P1 | Chat V2 | handlers/migration.rs | ~2 | ChatV2Error 已就绪 |
| P1 | Chat V2 | handlers/send_message.rs | ~2 | ChatV2Error 已就绪 |
| P1 | Chat V2 | handlers/workspace_handlers.rs | ~10 | ChatV2Error 已就绪 |
| P1 | Chat V2 | handlers/resource_handlers.rs | ~5 | ⚠️ 已废弃模块 |
| P1 | DSTU | handlers.rs | ~30 | DstuError 已就绪 |
| P2 | Notes | cmd/notes.rs | 39 | 已用 AppError |
| P2 | Enhanced Anki | enhanced_anki_service.rs | 22 | 已用 AppError |
| P2 | Web Search | cmd/web_search.rs | 17 | 已用 AppError |
| P0 | Legacy | commands.rs | 137 | 重命名+重组 |

---

## 已添加的基础设施

### Error From 转换链

```
VfsError ──From──> MemoryError
VfsError ──From──> EssayGradingError
VfsError ──From──> DstuError
AppError ──From──> EssayGradingError
anyhow   ──From──> ReviewPlanError
anyhow   ──From──> ChatV2Error
JoinError──From──> DstuError
rusqlite ──From──> ChatV2Error
rusqlite ──From──> DstuError
serde_json──From──> ChatV2Error
serde_json──From──> DstuError
std::io  ──From──> DstuError
```

### 迁移规则（重要！）

1. **移除 `.map_err(|e| e.to_string())`** → 改用 `?` + From trait
2. **移除 `ChatV2Error::XXX(...).into()`** → 直接返回 `ChatV2Error::XXX(...)`
3. **移除 `.to_string()` after error constructors** — `?` 操作符自动使用 From impls
4. **类型别名用 `std::result::Result<T, E>`** — 避免 `anyhow::Result` 阴影
5. **如果模块 error 没有 `From<String>`，绝不能返回 String error**
6. **spawn_blocking 闭包标注返回类型** — `-> DstuResult<T>` 确保 `?` 正确推断
7. **ChatV2Database::get_conn_safe() 已返回 ChatV2Result** — 直接 `?` 即可

---
*此报告由 deps.db 数据自动生成 + 人工更新*
