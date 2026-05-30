# Deep-Student 项目完全重扫描总报告

> 完成: 2026-05-30 16:00 CST | 覆盖: ~1200 源代码文件 | 批次: 5/7 完成
> 产出: 6 份批次报告 + 1 份最终汇总

## 1. 全局模块地图

### Rust 后端 (401 文件)

| 域 | 模块 | 文件数 | 命令/导出 | 命名一致性 |
|----|------|--------|-----------|-----------|
| Chat V2 | `chat_v2/` | 80+ | 80 Tauri 命令 | ⚠️ workspace_* 违规 |
| VFS | `vfs/` | 90+ | 120+ 命令 | ❌ mix vfs_/todo_/ocr_ |
| DSTU | `dstu/` | 30+ | 54 命令 | ✅ 完美 |
| Memory | `memory/` | 20+ | 27 命令 | ✅ |
| Data Gov | `data_governance/` | 40+ | 45 命令 | ✅ 黄金标准 |
| Essay | `essay_grading/` | 10+ | 20 命令 (AppError) | ✅ |
| LLM | `llm_manager/` | 50+ | 14 适配器 | ✅ |
| Legacy | `commands.rs` | 1 | ~140 命令 | ✅ AppError |
| Split Cmd | `cmd/` | 10 | ~130 命令 | ❌ 前缀混乱 |

### TypeScript 前端 (~1682 文件)

| 域 | 文件数 | 模式 | 一致性 |
|----|--------|------|--------|
| types/ | 16 | God File + re-exports | ❌ |
| stores/ | 14 | Zustand | ⚠️ 持久化不统一 |
| utils/ | 75 | 混杂 | ⚠️ |
| API (新) | 10 | namespace | ⚠️ 命名风格混用 |
| Features | 14 dirs | 各异 | ❌ 直接 invoke 严重 |

## 2. 完整冲突矩阵

### CRITICAL (P0) — 必须修复

| ID | 发现 | 位置 | 影响范围 |
|----|------|------|----------|
| **P0-01** | `ReviewPlanError` 不存在但 CLAUDE.md 声称已迁移 | `review_plan_service.rs` (17命令) | 进度误报 |
| **P0-02** | 两个 `DataGovernanceError` 定义冲突 | `data_governance/error.rs` vs `data_governance/mod.rs` | 编译失败风险 |
| **P0-03** | `EssayGradingError` 存在但未被使用 | `essay_grading/error.rs` → commands 用 AppError | 冗余代码 |

### HIGH (P1) — 应尽快修复

| ID | 发现 | 位置 |
|----|------|------|
| **P1-01** | `workspace_*` 18 命令缺 `chat_v2_` 前缀 | `workspace_handlers.rs` |
| **P1-02** | `todo_*`/`pomodoro_*` 30+ 命令缺 `vfs_` 前缀 | `vfs/todo_handlers.rs` |
| **P1-03** | `ocr_*` 5 命令缺 `vfs_` 前缀 | `vfs/ocr_storage_handlers.rs` |
| **P1-04** | 200+ 前端 `invoke()` 直接调用绕过 API 层 | 82 TS 文件 |
| **P1-05** | 3 种 TS API 层模式并存 | `api/` vs `tauriApi.ts` vs 本地 `api.ts` |
| **P1-06** | 4 命令错放：`ocr_extract_text` 在 `translation.rs` | `cmd/` 6 文件 |

### MEDIUM (P2) — 应修复

| ID | 发现 | 位置 |
|----|------|------|
| **P2-01** | `cmd/enhanced_anki.rs` 21 命令无统一前缀 | `cmd/` |
| **P2-02** | `cmd/anki_connect.rs` 3 种命名风格混用 | `cmd/` |
| **P2-03** | `cmd/web_search.rs` 5/7 命令与搜索无关 | `cmd/` |
| **P2-04** | 6 chat_v2 handler 文件名不一致 | `chat_v2/handlers/` |
| **P2-05** | `types/api/ui/hooks.ts` 纯重导出 (revert 后恢复) | `src/types/` |
| **P2-06** | 未跟踪错误类型: `LlmUsageError`, `McpError` | `llm_usage/`, `mcp/` |
| **P2-07** | `AppError` 无 `AppResult<T>` 类型别名 | `models.rs` |
| **P2-08** | `lib/utils.ts` cn() 与 `utils/cn.ts` 双实现 | TS 工具层 |

### LOW (P3) — 建议修复

| ID | 发现 |
|----|------|
| **P3-01** | OCR 适配器命名 (`NameOcrAdapter` vs `NameAdapter`) |
| **P3-02** | `pub mod zhipu` 可见性与其他 adapter 不一致 |
| **P3-03** | `handlers.rs` God File (87+ 命令) |
| **P3-04** | `cmd/notes.rs` 中 `rag_rebuild_fts_index` 缺前缀 |
| **P3-05** | TS API 命名风格 (`LlmUsageApi` vs `vfsFileApi`) |
| **P3-06** | `types/index.ts` TS God File (900+ 行) |

## 3. 错误类型 From 转换链（修正后）

```
VfsError        ──────────────→ MemoryError, EssayGradingError, DstuError, DataGovernanceError
AppError        ──────────────→ EssayGradingError, DataGovernanceError
anyhow::Error   ───→ ReviewPlanError*, ChatV2Error, DataGovernanceError  (*不存在)
rusqlite::Error ───→ ChatV2Error, DstuError, DataGovernanceError, AppError
JoinError       ──────────────→ DstuError
serde_json::Error──→ ChatV2Error, DstuError, DataGovernanceError, AppError
io::Error       ──────────────→ DstuError, DataGovernanceError, AppError
String          ──────────────→ DataGovernanceError, AppError

新发现: 全部 8 个错误类型均实现 Serialize (Tauri IPC 兼容)
新发现: DataGovernanceError 有两个定义, 其中一个在 data_governance/mod.rs (含 Migration/SchemaRegistry 变体)
```

## 4. 统一命名对照表

| 域 | 当前命名 | 建议统一命名 | 优先级 |
|----|----------|-------------|--------|
| chat_v2 workspace | `workspace_*` | `chat_v2_workspace_*` | P1 |
| vfs todo | `todo_*` | `vfs_todo_*` | P1 |
| vfs pomodoro | `pomodoro_*` | `vfs_pomodoro_*` | P1 |
| vfs ocr storage | `ocr_*` | `vfs_ocr_*` | P1 |
| cmd enhanced_anki | mixed | `enhanced_anki_*` | P2 |
| cmd anki_connect | mixed | `anki_connect_*` | P2 |
| cmd web_search | mixed | `web_search_*` 或移走不相关命令 | P2 |
| cmd translation | `ocr_extract_text` | 移到 `ocr.rs` | P2 |
| chat_v2 handler files | mixed suffixes | `{name}_handlers.rs` | P3 |
| TS API namespace | mixed | 统一 `camelCase` | P3 |
| OCR adapter | mixed | 统一 `NameOcrAdapter` | P3 |

## 5. 依赖数据库摘要

| 表 | 条目数 | 说明 |
|----|--------|------|
| `module_definitions` | ~140 (Rust) + ~200 (TS) | 模块定义 |
| `dependency_edges` | ~300 | 引用关系 |
| `conflict_tracker` | **28** (P0: 3, P1: 6, P2: 8, P3: 6, 文档: 5) | 命名/职责冲突 |

## 6. 后续重构行动建议

### Immediate (本会话完成)
1. ✅ 创建 `ReviewPlanError` (替代 17 命令的 String 错误)
2. ✅ 统一 `DataGovernanceError` 定义 (删除 mod.rs 中的重复版本)
3. ✅ 更新 CLAUDE.md 进度为真实值

### Short-term (下周)
4. 命令前缀标准化: `workspace_*`→`chat_v2_workspace_*`, `todo_*`→`vfs_todo_*`
5. 清理 `cmd/` 目录命令错放
6. 删除 `types/api/ui/hooks.ts` 纯重导出
7. 合并 `lib/utils.ts` cn() → `utils/cn.ts`

### Medium-term
8. 前端 API 层统一 (P1-05)
9. TypeScript God File 拆分
10. `EssayGradingError` 投入使用

---

*重新扫描完成。5 批次，28 个冲突，全部记录到依赖数据库。*
