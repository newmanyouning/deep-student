# 补充扫描计划 — 遗漏模块诊断

**日期**: 2026-05-29

## 遗漏原因

原始计划按**目录**组织，但后端有 **65 个单文件模块** (`src-tauri/src/*.rs`) 不属于任何子目录，因此被遗漏。前端也有 4 个小目录未被覆盖。

---

## 补充扫描清单

### A 组: 前端遗漏 (4 目录, 10 文件)

| # | 目录/文件 | 内容 |
|---|---------|------|
| S1 | `src/assets/` | 图标/字体等静态资源 (3 文件) |
| S2 | `src/data/` | Anki 内置模板 JSON + 模板管理器 TS |
| S3 | `src/menu/` | macOS 菜单事件桥接 |
| S4 | `src/promptkit/` | Prompt 输入 UI 工具包 (4 文件, 有独立 cn.ts!) |

### B 组: 后端大型单文件 (>2000 行, 11 个)

| # | 文件 | 行数 | 职责 |
|---|------|------|------|
| S5 | `commands.rs` | **5,839** | 🔴 原有巨量命令文件 |
| S6 | `lance_vector_store.rs` | **4,491** | 🔴 LanceDB 向量存储 |
| S7 | `question_import_service.rs` | **3,953** | 🔴 题目批量导入 |
| S8 | `document_parser.rs` | **3,620** | 🔴 文档解析 (PDF/DOCX) |
| S9 | `notes_exporter.rs` | **3,052** | 🟡 笔记导出 |
| S10 | `streaming_anki_service.rs` | **2,380** | 🟡 流式制卡 |
| S11 | `question_bank_service.rs` | **2,275** | 🟡 题库服务 |
| S12 | `notes_manager.rs` | **2,205** | 🟡 笔记管理 |
| S13 | `question_sync_service.rs` | **2,133** | 🟡 题目同步 |
| S14 | `models.rs` | **2,104** | 🟡 数据模型定义 |
| S15 | `exam_sheet_service.rs` | **1,694** | 试卷处理 |

### C 组: 后端中型单文件 (500-2000 行, 20+ 个)

| 文件 | 行数 |
|------|------|
| `pdf_ocr_service.rs` | 1,444 |
| `apkg_exporter_service.rs` | 1,436 |
| `data_space.rs` | 1,401 |
| `file_manager.rs` | 1,375 |
| `backup_job_manager.rs` | 1,301 |
| `vlm_grounding_service.rs` | 1,174 |
| `enhanced_anki_service.rs` | 1,016 |
| `backup_common.rs` | 920 |
| `page_rasterizer.rs` | — |
| `pdf_protocol.rs` | — |
| `persistent_message_queue.rs` | — |
| `document_processing_service.rs` | — |
| `debug_commands.rs` | — |
| `debug_log_service.rs` | — |
| `batch_operations.rs` | — |
| `question_export_service.rs` | — |
| `unified_file_manager.rs` | — |
| `review_plan_service.rs` | — |
| `secure_store.rs` | — |
| `session_manager.rs` | — |
| `spaced_repetition.rs` | — |

### D 组: 后端小型单文件 (<500 行)

`error_details`, `error_recovery`, `config_recovery`, `crash_logger`, `debug_logger`, `anr_watchdog`, `background_tasks`, `backup_config`, `cross_page_merger`, `deepseek_ocr_parser`, `feature_flags`, `figure_extractor`, `injection_budget`, `json_validator`, `llm_structurer`, `metrics_server`, `ocr_circuit_breaker`, `package_manager`, `pdfium_utils`, `reasoning_policy`, `startup_cleanup`, `tts`, `voice_input`, `workflow_error_handler`, `canonical_tools`, `textbooks_db`, `vector_store`, `database_optimizations`, `database.debug.rs`

---

## 建议扫描策略

由于遗漏模块数量多（65 个 Rust + 10 个前端），建议：

1. **A 组 (前端遗漏)** — 1 轮快速扫描
2. **B 组 (后端大型文件)** — 3 轮分步 (每轮 3-4 个文件)
3. **C+D 组 (中/小型文件)** — 1 轮批量摘要

**共 5 轮补充扫描**
