# 补充扫描: 遗漏模块诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 遗漏汇总

| 组 | 位置 | 文件数 | 总行数 |
|----|------|--------|--------|
| A | 前端遗漏 (assets/data/menu/promptkit) | 10 | ~500 |
| B | 后端大型单文件 (>1500行) | 15 | ~38,000 |
| C | 后端中型单文件 (500-1500行) | 20 | ~16,000 |
| D | 后端小型单文件 (<500行) | 30 | ~5,800 |
| **合计** | — | **75** | **~60,000** |

---

## A 组：前端遗漏

### promptkit/ — ⚠️ 第三套 cn() 实现

```
promptkit/
├── chat-container.tsx  72 行
├── prompt-input.tsx    167 行
├── lib/cn.ts           14 行 — 第三套 cn()!
│   (使用 twMerge 直接，不依赖 clsx)
└── ui/tooltip.tsx      14 行
```

**关键发现**: `promptkit/lib/cn.ts` 是项目中**第三套** cn() 实现：

| # | 实现 | 位置 |
|---|------|------|
| 1 | `clsx + twMerge` | `@/utils/cn` (推荐, 仅2文件使用) |
| 2 | 手写 clsx 克隆 | `@/lib/utils` (事实标准, 75文件) |
| 3 | `twMerge` 直接 | `promptkit/lib/cn.ts` (新发现!) |

### menu/ — macOS 菜单桥接

`menuEventBridge.ts` — 2026-05-14 新增，监听 Rust 原生菜单事件并转换为命令面板事件。Phase D2 of native-feel migration。

### data/ — Anki 模板管理

- `builtin-templates.json` — 内置 Anki 模板数据
- `ankiTemplates.ts` — `TemplateManager` 类（从数据库加载模板）

### assets/ — 静态资源

`react.svg`, `siliconflowlogo.svg`, `siliconflowlogo-dark.svg` — 仅 3 个文件。

---

## B 组：后端大型单文件 (11 文件, 33,746 行)

| 文件 | 行数 | 职责 | 问题 |
|------|------|------|------|
| `commands.rs` | **5,839** | 原有巨量 Tauri 命令 | 🔴 与 `cmd/` 并存，代码重复 |
| `lance_vector_store.rs` | **4,491** | LanceDB 向量存储 + 索引 | 🔴 应拆分为子模块 |
| `question_import_service.rs` | **3,953** | 题目批量导入 (CSV/JSON) | 🔴 与 question_bank_service 职责重叠 |
| `document_parser.rs` | **3,620** | 文档解析 (PDF/DOCX/图片) | 🔴 |
| `notes_exporter.rs` | **3,052** | 笔记批量导出 | 🟡 |
| `streaming_anki_service.rs` | **2,380** | 流式 Anki 制卡 | 🟡 |
| `question_bank_service.rs` | **2,275** | 题库 CRUD | 🟡 |
| `notes_manager.rs` | **2,205** | 笔记 CRUD | 🟡 |
| `question_sync_service.rs` | **2,133** | 题目云同步 | 🟡 |
| `models.rs` | **2,104** | 全局数据模型定义 | 🟡 应与 types 统一 |
| `exam_sheet_service.rs` | **1,694** | 试卷处理 | ✅ |

### 关键发现: commands.rs vs cmd/

```
commands.rs  (5839 行, 单文件) ← 旧架构
cmd/         (11 文件, 6629 行) ← 新架构 (按域拆分)
```

`commands.rs` 和 `cmd/` **并存**，形成新旧两套命令架构。`commands.rs` 5839 行可能包含 `cmd/` 中已迁移命令的旧版本或重复。

---

## C+D 组：中/小型单文件 (53 文件, 27,779 行)

### 按功能域分类

| 域 | 文件 | 代表 |
|----|------|------|
| **Anki 生态** | 5 个 | `anki_connect_service`, `apkg_exporter_service`, `streaming_anki_service`, `enhanced_anki_service`, `spaced_repetition` |
| **PDF/文档** | 7 个 | `document_parser`, `pdf_ocr_service`, `pdf_protocol`, `pdfium_utils`, `document_processing_service`, `page_rasterizer`, `deepseek_ocr_parser` |
| **题目系统** | 6 个 | `question_bank_service`, `question_import_service`, `question_export_service`, `question_sync_service`, `exam_sheet_service`, `review_plan_service` |
| **数据管理** | 5 个 | `data_space`, `file_manager`, `unified_file_manager`, `notes_manager`, `notes_exporter` |
| **备份/恢复** | 5 个 | `backup_job_manager`, `backup_common`, `backup_config`, `config_recovery`, `error_recovery` |
| **开发/调试** | 6 个 | `debug_commands`, `debug_log_service`, `debug_logger`, `crash_logger`, `anr_watchdog`, `metrics_server` |
| **向量/搜索** | 3 个 | `lance_vector_store`, `vector_store`, `vlm_grounding_service` |
| **基础设施** | 16 个 | `models`, `secure_store`, `session_manager`, `persistent_message_queue`, `feature_flags`, `background_tasks`, `workflow_error_handler`... |

### 设计评估

- ✅ 大部分中/小型文件职责单一，组织合理
- ⚠️ Anki 相关有 5 个独立服务文件，可考虑合并为 `anki_services/` 目录
- ⚠️ PDF/文档处理 7 个文件散落，可集中到 `document_processing/`
- ⚠️ 题目系统 6 个文件，与 `qbank_grading/` 目录的关系不明确

---

## 最终 God File 排行榜 (整合前后端)

| 排名 | 文件 | 行数 | 语言 |
|------|------|------|------|
| 🥇 | `vfs/handlers.rs` | 7,431 | Rust |
| 🥈 | `data_governance/sync/mod.rs` | 7,463 | Rust |
| 🥉 | `dstu/handlers.rs` | 6,379 | Rust |
| 4 | `llm_manager/mod.rs` | 5,994 | Rust |
| 5 | `commands.rs` | **5,839** | Rust ⭐ 新上榜 |
| 6 | `chat_v2/tools/chatanki_executor.rs` | 5,758 | Rust |
| 7 | `llm_manager/model2_pipeline.rs` | 5,567 | Rust |
| 8 | `database/mod.rs` | 6,043 | Rust |
| 9 | `vfs/indexing.rs` | 4,632 | Rust |
| 10 | `lance_vector_store.rs` | **4,491** | Rust ⭐ 新上榜 |
| 11 | `chat_v2/repo.rs` | 4,371 | Rust |
| 12 | `TauriAdapter.ts` | **4,104** | TS |
| 13 | `question_import_service.rs` | **3,953** | Rust ⭐ 新上榜 |
| 14 | `chat_v2/types.rs` | 3,670 | Rust |
| 15 | `document_parser.rs` | **3,620** | Rust ⭐ 新上榜 |

---

## 补充发现的关键问题

- [ ] **P1** — `commands.rs` (5839行) 与 `cmd/` 目录并存，形成新旧两套命令架构，存在代码重复风险
- [ ] **P1** — 存在**第三套** cn() 实现 (`promptkit/lib/cn.ts`)，使用 twMerge 直接调用。现在项目有 3 套 cn() 并存
- [ ] **P2** — 65 个单文件 Rust 模块在 `src-tauri/src/` 根目录，应组织到子目录中
- [ ] **P2** — Anki 相关 5 个独立服务 + PDF 处理 7 个文件散落，应按域合并为目录
- [ ] **P2** — `models.rs` (2104行) — 全局数据模型定义应拆分为 domain models
- [ ] **P3** — `database.debug.rs` 是调试文件但放在生产目录中
