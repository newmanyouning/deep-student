# Round 20: 后端入口与命令路由 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 规模概要

| 文件 | 行数 | 职责 |
|------|------|------|
| `main.rs` | 6 | 仅调用 `deep_student_lib::run()` |
| `lib.rs` | **2275** | 🔴 90+ mod 声明 + run() + 760 命令注册 |
| `cmd/mod.rs` | 19 | 命令子模块索引（11 个域） |
| `database/mod.rs` | **6043** | 🔴 数据库操作（SQL + 迁移 + CRUD） |
| `database/manager.rs` | 2559 | 数据库连接管理 |
| `Cargo.toml` | 248 | 依赖配置 |

---

## 模块声明 (lib.rs — 90+ 个 pub mod)

lib.rs 声明了 90+ 个子模块，覆盖了:
- 核心功能: `chat_v2`, `vfs`, `dstu`, `llm_manager`, `memory`, `mcp`
- 业务功能: `essay_grading`, `translation`, `qbank_grading`, `tools`
- 基础设施: `crypto`, `database`, `cloud_storage`, `secure_store`
- 服务层: `exam_sheet_service`, `question_bank_service`, `document_processing_service`
- 条件编译: `data_governance` (feature gated), `menu` (macOS only), `mcp` (feature gated)

### 命令注册 (760 个 invoke_handler)

`tauri::generate_handler!` 宏中包含约 **760 个命令**，按域分布:
- PDF OCR: ~15 命令
- 题目集: ~15 命令
- 设置/配置: ~30 命令
- OCR 引擎: ~12 命令
- Anki/卡片: ~20 命令
- Chat V2 / MCP / VFS / Data Governance: 其余 ~660 命令

---

## cmd/ 目录 (11 文件, 6,629 行)

| 文件 | 行数 | 域 |
|------|------|------|
| `notes.rs` | **2019** | 笔记命令 |
| `anki_connect.rs` | 884 | Anki 连接 |
| `textbooks.rs` | 818 | 教材 |
| `ocr.rs` | 766 | OCR |
| `mcp.rs` | 725 | MCP |
| `enhanced_anki.rs` | 616 | 增强制卡 |
| `web_search.rs` | 556 | 搜索 |
| `translation.rs` | 135 | 翻译 |
| `anki_cards.rs` | 74 | 卡片 |
| `helpers.rs` | 36 | 辅助函数 |
| `mod.rs` | 19 | 索引 |

---

## database/ 模块 (2 文件, 8,602 行)

- `mod.rs` (6043行): 包含所有表的 CRUD 操作 — SQL 查询、迁移函数、表 Schema 定义
- `manager.rs` (2559行): 连接池管理、路径解析、备份/恢复

---

## 发现的问题

- [ ] **P1** — `lib.rs` 2275 行，混合了 90+ mod 声明 + run() 初始化 + 760 命令注册。应将命令注册分离到独立文件
- [ ] **P1** — `database/mod.rs` **6043 行** — 全项目最大 Rust 文件。所有表的 SQL 操作集中在一个文件中，应按表或域拆分
- [ ] **P2** — `cmd/notes.rs` 2019 行，cmd 目录中还有独立模块(lib.rs 中的 `commands::`)未合并到 cmd/
- [ ] **P2** — 760 个 Tauri 命令 — 这是一个极其庞大的 API surface。许多命令可能未被前端使用（需要 R05 发现的 API 层调用来交叉验证）
- [ ] **P3** — `cmd/` (11个文件, 6.6K行) 和 `commands/` (lib.rs 中独立声明) 并存，形成两个命令目录

---

## 建议优先处理

1. 拆分 `database/mod.rs` (6043行) — 按表创建独立文件 (sessions.rs, messages.rs, anki_cards.rs...)
2. 将 760 命令注册从 lib.rs 提取到 `cmd/` 目录的注册函数
3. 统一 `cmd/` 和 `commands/` 为一个命令目录
