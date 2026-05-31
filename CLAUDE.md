# 项目重构进度

> 最后更新: 2026-05-31 | PaddleOCR 全栈集成 + URL模式补充完成 ✅
> 当前阶段: 语法扫描 — 进行中 | 详细计划: SYNTAX_SCAN_PLAN.md

## 0. 重构协议

### 时间戳铁律 🔴
- **唯一合法来源**: `date +"%Y-%m-%d %H:%M CST"` (当前) + `stat -c "%y %n" <file>` (文件)
- **严禁臆造**: 禁止手写时间、估算、复制旧时间戳

### 编译纪律 ✅ (阶段已升级)
- ~~禁止编译验证~~ → 重构基本完成，允许 cargo check
- 静态分析阶段结束，进入编译验证阶段

---

## 1. 错误类型状态矩阵 (全部完成)

| 模块 | 文件 | Serialize | Display | Error | From<VfsError> | From<anyhow> | From<String> | From<模块> for String |
|------|------|-----------|---------|-------|----------------|--------------|--------------|----------------------|
| VfsError | vfs/error.rs | ✅ | ✅ | ✅ | — | — | ✅(新增) | ✅ |
| DstuError | dstu/error.rs | ✅ | ✅(thiserror) | ✅ | ✅(新增) | — | ✅(新增) | ✅ |
| ChatV2Error | chat_v2/error.rs | ✅ | ✅(thiserror) | ✅ | — | ✅ | — | ✅ (JSON) |
| ReviewPlanError | review_plan_error.rs | ✅ | ✅ | — | ✅ | ✅ | ✅ | — |
| DataGovernanceError | data_governance/mod.rs | ✅(manual) | ✅(thiserror) | ✅ | ✅(新增) | ✅(新增) | ✅(新增) | ✅ (JSON) |
| EssayGradingError | essay_grading/error.rs | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| MemoryError | memory/error.rs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| ToolError | tools/executor.rs | — | ✅ | ✅ | ✅ | — | ✅ | — |
| AnkiConnectError | anki_connect_service.rs | — | ✅ | ✅ | — | — | ✅ | ✅ |

### From 转换链（完整）
```
VfsError → MemoryError, EssayGradingError, DstuError, ReviewPlanError, DataGovernanceError, ToolError
anyhowError → ReviewPlanError, ChatV2Error, EssayGradingError, MemoryError, DataGovernanceError
AppError → EssayGradingError
rusqlite → ChatV2Error, DstuError, VfsError, DataGovernanceError
io::Error → VfsError, DstuError, DataGovernanceError
serde_json → ChatV2Error, DstuError, VfsError, DataGovernanceError
ChatV2Error → ToolError
VfsError → ToolError
String → VfsError, DstuError, DataGovernanceError, EssayGradingError, MemoryError, ToolError, AnkiConnectError
```

---

## 2. 模块命令转换状态

| 模块 | 命令/函数数 | 当前错误类型 | 状态 |
|------|------------|-------------|------|
| VFS (handlers + index) | ~32 | VfsResult<T> | ✅ |
| VFS (todo/pomodoro) | 25 | VfsResult<T> | ✅ |
| Chat V2 (handlers) | ~85 | ChatV2Result<T> | ✅ |
| Chat V2 (tools) | ~150 | ToolResult<T> | ✅ |
| Review Plan | 17 | ReviewPlanResult<T> | ✅ |
| DSTU (全模块) | ~59 | DstuResult<T> | ✅ |
| Essay Grading | 20 | EssayGradingResult<T> | ✅ |
| Memory | 27 | MemoryResult<T> | ✅ |
| Data Governance | ~55 | DataGovernanceResult<T> | ✅ |
| Commands (遗留) | ~140 | Result<T, AppError> | ✅ |
| 内部服务 | ~27 | AnkiConnectError/DataGovError/AppError | ✅ |

**总计: ~637 命令/函数已标准化**

---

## 3. 新增错误类型 (本次重构创建/完善)

| 错误类型 | 文件 | 变体 | 用途 |
|---------|------|------|------|
| ToolError | tools/executor.rs | InvalidArgs, Execution, Timeout, NotFound, Cancelled, Internal | LLM 工具调用 |
| AnkiConnectError | anki_connect_service.rs | Request, Parse, Other | Anki 服务通信 |
| EssayGradingError | essay_grading/error.rs | Database, Validation, NotFound, Internal, Other | 作文批改 |
| MemoryError | memory/error.rs | Database, Validation, NotFound, Other | 记忆系统 |
| ReviewPlanError | review_plan_error.rs | Database, Validation, NotFound, Other | 复习计划 |

---

## 4. 冲突任务进度

### P0 ✅ | P1 ✅ | P2 ✅ — 全部完成

---

## 5. 已完成的跨领域工作

| 工作项 | 日期 | 内容 |
|--------|------|------|
| ToolError 统一化 | 05-30 | 新建 ToolError, ToolExecutor trait 升级, 33 文件 ~150 函数转换 |
| VFS + ChatV2 handlers | 05-30 | ~42 命令 String→VfsResult/ChatV2Result |
| 内部服务修复 | 05-30 | 27 函数 String→typed (anki/apkg/backup/config) |
| 废弃模块删除 | 05-30 | adapters/, resource_repo.rs, resource_handlers.rs (6文件+7lib.rs注册) |
| Data Governance | 05-30 | 55 命令 String→DataGovernanceResult (6文件) |
| DSTU 模块 | 05-30 | 58 命令 String→DstuResult (3文件, From<VfsError>+From<String>新增) |
| EssayGrading + Memory | 05-30 | 47 命令 AppError→EssayGradingError / String→MemoryError |
| 错误类型基础设施 | 05-30 | VfsError Serialize, DataGovernanceError 手动 Serialize, 9个From impl |
| REF-008 chat_v2 | 05-30 | 循环依赖分析+shared.rs+废弃模块删除+通配符显式化+导入统一+ContentBlock去重 |
| 命名标准化 P1 | 05-30 | workspace/todo/pomodoro/ocr 前缀统一 (48命令) |

---

## 6. 下一步

- **语法扫描**: 分批扫描全部 ~1,972 源文件，修复语法错误 (~44 批次)
- **编译验证**: 统一编译 (cargo check + npm run build)
- **前端验证**: 检查 TypeScript 类型与后端命令一致性

---

## 7. 语法扫描进度

> 详细批次定义: SYNTAX_SCAN_PLAN.md
> 每批次完成后更新此表

### 第一阶段: Rust 源文件 (397 files, ~340K lines)

| 批次 | 文件数 | 状态 | 发现问题 | 完成时间 |
|------|--------|------|----------|----------|
| R1: 根级小文件 + adapters | 20 | ✅ 复核通过 | 1 (database.debug.rs) | 2026-05-31 |
| R2: 核心入口 (lib/commands/models) | 3 | ✅ 复核通过 | 1 (lib.rs) | 2026-05-31 |
| R3: 内部服务 Part 1 | 11 | ✅ 完成 | 0 | 2026-05-31 |
| R4: 内部服务 Part 2 | 14 | ✅ 完成 | 0 | 2026-05-31 |
| R5: 更多服务 + 工具 | 20 | ✅ 完成 | 0 | 2026-05-31 |
| R6: VFS 核心 | 14 | ✅ 完成 | 19 (vfs/handlers.rs: VfsError::Other 缺闭括号) | 2026-05-31 |
| R7: VFS repos Part 1 | 10 | ✅ 完成 | 0 | 2026-05-31 |
| R8: VFS repos Part 2 + 其他 | 17 | ✅ 完成 | 0 | 2026-05-31 |
| R9: Chat V2 核心 | 14 | ✅ 完成 | 0 | 2026-05-31 |
| R10: Chat V2 pipeline + handlers | 20 | ✅ 完成 | 0 | 2026-05-31 |
| R11: Chat V2 tools Part 1 | 16 | ✅ 完成 | 0 | 2026-05-31 |
| R12: Chat V2 tools Part 2 + workspace | 19 | ✅ 完成 | 0 | 2026-05-31 |
| R13: DSTU 模块 | 23 | ✅ 完成 | 0 | 2026-05-31 |
| R14: Data Governance | 28 | ✅ 完成 | 0 | 2026-05-31 |
| R15: LLM Manager | 17 | ✅ 完成 | 0 | 2026-05-31 |
| R16: LLM Usage + Memory + Essay + QBank | 20 | ✅ 完成 | 0 | 2026-05-31 |
| R17: Database + Providers + Tools | 8 | ✅ 完成 | 0 | 2026-05-31 |
| R18: Multimodal + OCR + Translation | 17 | ✅ 完成 | 0 | 2026-05-31 |
| R19: CMD + Cloud + Crypto + MCP | 22 | ✅ 完成 | 0 | 2026-05-31 |
| R20: Utils + 剩余 | 15 | ✅ 完成 | 0 | 2026-05-31 |

### 第二阶段: TypeScript/TSX 源文件 (~1,575 files)

| 批次 | 文件数 | 状态 | 发现问题 | 完成时间 |
|------|--------|------|----------|----------|
| T1: 入口 + API + Config | 25 | ⏳ 待开始 | — | — |
| T2: Stores | 14 | ⏳ 待开始 | — | — |
| T3: Hooks Part 1 | 20 | ⏳ 待开始 | — | — |
| T4: Hooks Part 2 + Lib + Services | 25 | ⏳ 待开始 | — | — |
| T5: Types + Utils Part 1 | 25 | ⏳ 待开始 | — | — |
| T6: Utils Part 2 | 30 | ⏳ 待开始 | — | — |
| T7: MCP + Voice + Debug Panel | 40 | ⏳ 待开始 | — | — |
| T8: DSTU + Essay + Components shared | 40 | ⏳ 待开始 | — | — |
| T9-T20: Features 按子模块 | ~760 | ⏳ 待开始 | — | — |
| T21: Study UI + 配置 | 120 | ⏳ 待开始 | — | — |

### 第三阶段: 交叉检查

| 批次 | 内容 | 状态 | 发现问题 | 完成时间 |
|------|------|------|----------|----------|
| C1: Rust mod vs 文件系统 | — | ⏳ 待开始 | — | — |
| C2: TS import vs 文件系统 | — | ⏳ 待开始 | — | — |
| C3: Tauri command 注册 vs 前端 | — | ⏳ 待开始 | — | — |

### 整体进度: 20/44 批次 (45.5%) — Rust 阶段完成 ✅

> ⚠️ CI 反馈补充修复 (2026-05-31): 24 个 Rust 文件中发现全角引号（U+201C/U+201D "like this"）替代 ASCII 引号，Rust 编译器不识别。已全局替换修复。另 2 个文件含全角单引号（U+2018/U+2019）也已修复。

---

## 附录 A: 技术栈
0.9.40 | Rust 1.96 | React 18 | Tauri 2 | ~3000 文件

## 附录 B: 重构统计
- **新增错误类型**: 5 个 (ToolError, AnkiConnectError, EssayGradingError, MemoryError, ReviewPlanError)
- **新增 From 转换**: 16 个
- **新增 error.rs 文件**: 4 个 (essay_grading, memory, dstu(impls), vfs(impls))
- **删除废弃模块**: 3 个模块 (6 文件)
- **转换命令/函数**: ~637 个
- **涉及文件数**: ~100+ Rust 文件
