# 项目重构进度

> 最后更新: 2026-05-30 12:28 CST — OCR 存储模块创建完成 (3文件, 378行)
> 当前阶段: 静态修改
> 总体完成度: 55% (12/22 任务完成)

## 1. 当前任务
- **任务编号**: —
- **任务标题**: 清理 lib.rs 模块注册
- **所属层次**: Layer 5
- **状态**: 进行中
- **关联细分文件**: docs/refactor-tasks/REF-015-cleanup-lib-rs.md
- **开始时间**: 2026-05-30 09:03 CST
- **受阻原因**: —

## 2. 整体进度索引

### Layer 0: 基础类型与常量定义
- [x] [REF-001] 审计并清理未使用的 TypeScript 类型 (P1, M) — ✅ 完成
- [x] [REF-002] 审计并清理未使用的 Rust 类型/枚举 (P1, M) — ✅ 完成 (08:47 CST)
- [ ] [REF-003] 统一前后端重复的类型定义 (P2, L)

### Layer 1: 工具函数与纯逻辑模块
- [x] [REF-004] 解决 cn() 函数重复 (utils/cn vs lib/utils) (P1, S) — ✅ 完成
- [x] [REF-005] 审计并去重代码库中的工具函数 (P2, M) — ✅ 完成 (08:43 CST)

### Layer 2: 数据模型与状态
- [x] [REF-006] 审计 Zustand stores 中的未使用状态 (P2, M) — ✅ 完成 (08:55 CST)
- [x] [REF-007] 统一 store 模式 (P2, L) — ✅ 完成 (09:00 CST), P2-08 已解决

### Layer 3: 核心业务逻辑
- [ ] [REF-008] 解决 chat_v2 模块中的循环依赖 (P1, L)
- [ ] [REF-009] 提取 LLM 适配器接口以降低耦合 (P2, XL)
- [ ] [REF-010] 统一 VFS repo 访问模式 (P2, M)

### Layer 4: API 处理层与消息路径
- [-] [REF-011] 完成剩余内部服务的 API 错误类型统一 (P1, M) — Tauri 命令部分已完成 (54.2%), 内部服务文件待处理
- [ ] [REF-012] 标准化 Tauri 命令命名为 `{module}_{action}` 约定 (P3, L)
- [ ] [REF-013] 审计并记录所有 IPC 消息路径（事件/命令） (P2, M)
- [x] [REF-014] 移除已废弃的 `resource_handlers.rs` 模块 (P1, S) — ✅ 完成 (08:37 CST)

### Layer 5: 入口与组装层
- [ ] [REF-015] 清理 `lib.rs` 模块注册 (P2, S)
- [ ] [REF-016] 从 `commands.rs` 中移除死代码（标记未使用命令） (P2, M)

### 横切关注点
- [ ] [REF-017] CSS 架构迁移（12K行全局CSS→Tailwind v4） (P1, XL)
- [ ] [REF-018] 修复 17 个已有 TypeScript 错误（tsc --noEmit） (P1, S)
- [ ] [REF-019] 打包体积优化（懒加载 + 代码分割） (P2, L)
- [ ] [REF-020] 死代码消除遍历（前端 + 后端） (P2, L)
- [ ] [REF-021] i18n 键审计与清理 (P3, M)
- [ ] [REF-022] 为 `tsc --noEmit` 添加 CI 门禁 (P2, S)

## 3. 最近完成的任务 (最近5条)
1. 2026-05-30 09:00 CST — REF-007 完成: 创建 tauriPersistStorage 适配器, ankiQueueStore 248→110行(-56%), P2-08 持久化统一
2. 2026-05-30 08:55 CST — REF-006 完成: 删除 store/ResourceStateManager.ts, P2-07 双目录解决
3. 2026-05-30 08:43 CST — REF-005 完成: generateId 统一, formatDate/formatFileSize 重复审计
4. 2026-05-30 08:37 CST — REF-014 完成: 删除 resource_handlers.rs
5. 2026-05-30 08:33 CST — REF-000 生态语法审计: 无破坏性更新

## 4. 验证状态
- 静态分析: 未启动
- 分块验证: 未启动
- 编译验证: 未启动（项目规则：重构阶段禁止编译验证）

## 5. 偏离与风险
- **无偏离**: 工作遵循指导文件计划。
- **风险 REF-017**: CSS 迁移涉及 ~12K 行全局 CSS + 60+ 散落文件，需高度协调。已有完整设计文档和迁移计划。
- **风险 REF-008**: chat_v2 循环依赖可能需要在 14 个 handler 文件中进行接口变更。
- **REF-005 已完成**: ✅ generateId 统一; 🔍 formatDate×10, formatFileSize×10 内联重复; getErrorMessage 已集中化(125)
- **REF-007 已完成**: ✅ P2-08 持久化统一 (tauriPersistStorage 适配器); ankiQueueStore 248→110行(-56%); questionBankStore God Store 拆分为 REF-008 后置

## 6. 下一步计划
- [ ] REF-006 审计 Zustand stores 未使用状态 (P2, M, 依赖 REF-001✅)
- [ ] REF-008 解决 chat_v2 循环依赖 (P1, L, 无依赖)
- [ ] REF-018 修复 17 个 TS 错误 (P1, S, 依赖 REF-004✅)
- [ ] REF-011 完成 API 错误类型统一（继续 REF-011 内部服务文件）

---

## 附录 A: 时间协议

### 获取当前时间
每次会话启动及任务开始/完成时，使用以下命令获取准确时间：
```bash
date +"%Y-%m-%d %H:%M:%S %Z"          # 本地时间 (CST/UTC+8)
date -u +"%Y-%m-%d %H:%M:%S UTC"       # UTC 时间
```

### 获取文件修改时间（用于追溯已完成的变更）
```bash
stat -c "%y %n" <file-path>            # 文件最后修改时间
ls -lt <dir> | head -10                # 列出最近修改的文件
```

### 时间戳格式规范
- **任务开始/完成**: `YYYY-MM-DD HH:MM CST` (如 `2026-05-30 09:01 CST`)
- **文件修改时间**: 使用 `stat -c "%y"` 输出，格式 `YYYY-MM-DD HH:MM:SS.nnn +0800`
- **CLAUDE.md 最后更新**: `> 最后更新: YYYY-MM-DD HH:MM CST — 简述`

### 规则
- 当前时间使用 `date` 命令获取，不手动估算
- 过去的变更时间使用 `stat -c "%y"` 读取文件系统时间戳
- 禁止使用 `date -s` 或任何修改系统时间的命令
- 所有时间戳使用 CST (UTC+8) 时区，中国大陆标准时间

---

## 附录 B: 项目架构速查

### 技术栈
- **名称**: Deep Student - AI 驱动的智能学习助手
- **版本**: 0.9.40
- **技术栈**: Tauri 2 (Rust 后端) + React/TypeScript (前端)
- **Cargo 入口**: `src-tauri/Cargo.toml`（非根目录）
- **Rust 版本**: 1.96.0

### 重构纪律
- **禁止编译测试**（cargo check/build/test），仅做代码修改和静态分析

### 模块结构
- `src-tauri/src/lib.rs` — 模块注册入口
- `src-tauri/src/chat_v2/` — Chat V2 (handlers/ 14 文件, pipeline, workspace, tools, error.rs)
- `src-tauri/src/dstu/` — DSTU 资源协议 (handlers, folder_handlers, trash_handlers, export, error)
- `src-tauri/src/memory/` — 智能记忆 (service, handlers, config, error)
- `src-tauri/src/vfs/` — 虚拟文件系统 (database, repos, types, handlers, error)
- `src-tauri/src/essay_grading/` — 作文批改 (pipeline, events, types, error)
- `src-tauri/src/data_governance/` — 数据治理 (6 个命令文件, migration, backup, sync, audit, error)
- `src-tauri/src/commands.rs` — 遗留命令文件 (已使用 AppError)
- `src-tauri/src/cmd/` — 已拆分命令模块 (notes, web_search, ocr, mcp 等)
- `src-tauri/src/models.rs` — AppError 定义

### 错误类型及 From 转换
```
VfsError     → MemoryError, EssayGradingError, DstuError
AppError     → EssayGradingError
anyhow::Error → ReviewPlanError, ChatV2Error
rusqlite::Error → ChatV2Error, DstuError, DataGovernanceError, AppError
JoinError    → DstuError
serde_json::Error → ChatV2Error, DstuError, DataGovernanceError, AppError
io::Error    → DstuError, DataGovernanceError, AppError
String       → DataGovernanceError, AppError
```

### 方法返回类型速查（用于 REF-011）
| 调用来源 | 返回类型 | 适用的 From |
|---------|---------|------------|
| VfsRepo 方法 | VfsResult<T> | From<VfsError> |
| MemoryService 方法 | VfsResult<T> | From<VfsError> |
| ChatV2Database::get_conn_safe() | ChatV2Result<Conn> | (已是 ChatV2Error) |
| ChatV2Repo 方法 | Result<T, ChatV2Error> | (已是 ChatV2Error) |
| LLMManager 方法 | Result<T, AppError> | From<AppError> |

### 关键参考文档
| 文档 | 路径 |
|------|------|
| 生态语法审计 (最新调研) | `docs/refactor-tasks/REF-000-ecosystem-syntax-audit.md` |
| 重构进度摘要 | `docs/refactor-progress-summary.md` |
| 分层次指导文件 | `docs/refactor-master-guide.md` |
| 大重构清单 | `docs/refactor-master-checklist.md` |
| 累积问题清单 (55+) | `.planning/exploration/reports/cumulative-issues.md` |
| API 重构索引 | `.planning/exploration/dependency-db/reports/api-refactor/INDEX.md` |
| CSS 迁移设计 | `docs/plans/2026-05-13-css-architecture-migration-design.md` |
| 路线图 | `.planning/ROADMAP.md` |
| 代码风格指南 | `docs/CODE_STYLE.md` |
| 构建配置 | `docs/BUILD-CONFIG.md` |
