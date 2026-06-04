# 遗漏单文件模块 — 完整诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成 (B+C+D 三组全部扫描)

---

## B 组: 大型单文件 (11 文件, 33,746 行)

### 1. commands.rs (5,839 行) — 旧版命令文件

**架构角色**: 旧版 Tauri 命令集合，与 `cmd/` 目录并存形成双轨。

```
commands.rs 结构:
├── pub use cmd::* (re-export 新架构命令, 第46-54行)
├── 自定义 parse_bool_flag 等辅助函数
├── #[tauri::command] 函数 — 剩余的旧版命令
│   (绝大部分业务命令已迁移到 cmd/，commands.rs 保留的是
│    尚未迁移的旧命令或与 cmd/ 共享的命令集)
└── 全局常量 (OPTIMIZE_MIN_INTERVAL_KG_SECS)
```

**文件头显示**: `pub use crate::cmd::*` — re-export 了 cmd/ 的所有命令，确保旧路径仍然工作。这意味着 `commands.rs` 现在是一个**兼容层**而非主入口。

**实际注册**: `lib.rs` 的 `generate_handler!` 中同时使用了 `crate::commands::*` 和直接来自 `cmd/` 的命令。新旧混合注册。

### 2. lance_vector_store.rs (4,491 行) — LanceDB 向量存储

**架构角色**: LanceDB 向量数据库的完整实现。

```
├── extract_plain_text() — 聊天内容提取
├── LanceVectorStore struct
│   ├── 向量 CRUD (add/delete/search/batch)
│   ├── Table 管理 (create/compact/optimize)
│   ├── FTS 全文搜索索引
│   ├── Embedding 维度管理
│   └── 移动端兼容 (tmpdir 处理)
└── #[cfg(feature = "lance")] 条件编译
```

**依赖**: arrow_array, lancedb, lance-encoding, 通过 feature flag 控制。

### 3. question_import_service.rs (3,953 行) — Visual-First 题目导入

**架构角色**: 6 阶段 VLM 驱动的题目导入管线。

```
Stage 1: PageRasterizer — PDF/DOCX → 高清页面图片
Stage 2: VlmAnalyzer   — VLM 逐页分析 → 题目 + 配图 bbox
Stage 3: CrossPageMerger — 跨页题目检测合并
Stage 4: FigureExtractor — 按 bbox 裁切配图 → VFS
Stage 5: LlmStructurer   — VLM raw_text → 标准题目 JSON
Stage 6: Persistence      — SAVEPOINT 事务写入
+ CSV 导入 (CsvImportService)
```

**依赖**: page_rasterizer, vlm_grounding_service, figure_extractor, cross_page_merger, llm_structurer, vfs

### 4. document_parser.rs (3,620 行) — 多格式文档解析

**架构角色**: 统一的文档解析引擎。

```
支持格式:
├── PDF (pdfium-render + poppler 回退)
├── DOCX (calamine + docx-rs)
├── PPTX (pptx-to-md)
├── EPUB (zip + quick-xml 自实现, 避免 GPL-3.0)
├── RTF (rtf-parser)
├── TXT/Markdown/图片
└── 安全防护:
    ├── ZIP Bomb 检测 (最大压缩比 100:1)
    ├── 嵌套 ZIP 深度限制 (max 3 层)
    ├── 单条目大小上限 (100MB)
    └── 文件数上限 (10000)
```

### 5. notes_exporter.rs (3,052 行) — 笔记导出

**架构角色**: 笔记批量导出为 ZIP (Markdown + 元数据)。

```
NotesExporter struct
├── 批量导出 (export_all)
├── 单笔记导出 (export_single)
├── 版本历史 (include_versions)
├── 附件打包
└── ZIP 格式: .md 文件 + _versions/ + _preferences/
Schema v2
```

### 6. streaming_anki_service.rs (2,380 行) — 流式 Anki 制卡

**架构角色**: LLM 流式生成 Anki 卡片的服务。

```
StreamingAnkiService
├── 分段处理文档
├── LLM 流式调用
├── 取消信号 (watch channel)
├── 暂停/恢复
├── 卡片提取/去重
└── 断点续传
```

### 7. question_bank_service.rs (2,275 行) — 题库服务

**架构角色**: 题目 CRUD + 答题评分 + 统计聚合。

委托给 VfsQuestionRepo 进行数据操作，自身负责业务逻辑。

### 8. notes_manager.rs (2,205 行) — 笔记管理

**架构角色**: 笔记 CRUD + 文本提取 + 向量索引触发。

支持 ProseMirror JSON 和 Markdown 格式的内容提取。

### 9. question_sync_service.rs (2,133 行) — 题目云同步

**架构角色**: 本地-远程题目同步 + 冲突解决。

```
同步策略: KeepLocal / KeepRemote / KeepNewer / Merge / Manual
冲突检测: 基于 updated_at + content_hash
```

### 10. models.rs (2,104 行) — 全局数据模型

**架构角色**: 所有 Rust 结构体的集中定义。

包含: ChatMessage, AnkiCard, Template, ExamSheet, ApiConfig, ModelAssignments, AppError 等。与前端 `types/index.ts` 的 God File 问题对称。

### 11. exam_sheet_service.rs (1,694 行) — 试卷处理

**架构角色**: 试卷上传 → OCR → 题目识别 → 预览生成。

---

## B 组关键问题汇总

| # | 文件 | 行数 | 核心问题 |
|---|------|------|---------|
| 1 | `commands.rs` | 5,839 | 与 `cmd/` 双轨并存，实际为兼容层 |
| 2 | `lance_vector_store.rs` | 4,491 | 单文件过大，应拆分 table/config/search |
| 3 | `question_import_service.rs` | 3,953 | 6 阶段管线集中，Stage 应拆分为独立模块 |
| 4 | `document_parser.rs` | 3,620 | 7 种格式解析集中，每格式应独立文件 |
| 5 | `notes_exporter.rs` | 3,052 | 导出逻辑 + ZIP 打包 + 版本历史混合 |
| 6 | `streaming_anki_service.rs` | 2,380 | LLM 调用 + 卡片解析 + 队列管理混合 |
| 7 | `question_bank_service.rs` | 2,275 | 委托给 VFS，自身代码可精简 |
| 8 | `notes_manager.rs` | 2,205 | CRUD + 文本提取 + 向量索引混合 |
| 9 | `question_sync_service.rs` | 2,133 | 同步 + 冲突检测 + 本地 DB 操作混合 |
| 10 | `models.rs` | 2,104 | 全局类型 God File，应按域拆分 |
| 11 | `exam_sheet_service.rs` | 1,694 | 业务逻辑 + VLM + 图片裁剪混合 |

---

## C 组: 中型单文件 (23 文件, ~16,084 行)

| 文件 | 行数 | 职责 | 问题 |
|------|------|------|------|
| `pdf_ocr_service.rs` | 1,444 | PDF OCR 处理 (pdfium + LLM) | 🟡 |
| `apkg_exporter_service.rs` | 1,436 | Anki APKG 格式导出 | 🟡 |
| `data_space.rs` | 1,401 | A/B/C/D 四槽位数据空间 | ✅ 设计清晰 |
| `file_manager.rs` | 1,375 | 文件系统操作 + 图片处理 | ✅ |
| `backup_job_manager.rs` | 1,301 | 备份任务调度 (DashMap) | 🟡 |
| `vlm_grounding_service.rs` | 1,174 | VLM 页面分析 (Stage 2) | ✅ |
| `enhanced_anki_service.rs` | 1,016 | 增强 Anki 制卡 | 🟡 |
| `backup_common.rs` | 920 | 备份公共函数 | ✅ |
| `review_plan_service.rs` | 911 | SM-2 间隔重复 | ✅ |
| `debug_commands.rs` | 885 | 调试命令集合 | ⚠️ 生产代码含调试 |
| `document_processing_service.rs` | 722 | 文档处理编排 | ✅ |
| `backup_config.rs` | 720 | 备份配置 | ✅ |
| `unified_file_manager.rs` | 718 | 统一文件管理 | ⚠️ 与 file_manager 可能重叠 |
| `debug_logger.rs` | 715 | 调试日志 (JSON + 多级过滤) | ⚠️ 生产代码 |
| `reasoning_policy.rs` | 691 | 思维链回传策略 | ✅ |
| `anki_connect_service.rs` | 611 | AnkiConnect 本地 API | ✅ |
| `injection_budget.rs` | 569 | 上下文注入预算 | ✅ |
| `voice_input.rs` | 543 | 语音输入 (ASR) | ✅ |
| `secure_store.rs` | 538 | AES-256-GCM 安全存储 | ✅ |
| `textbooks_db.rs` | 519 | 教材独立数据库 | ✅ |
| `question_export_service.rs` | 505 | 题目导出 (CSV/JSON) | ✅ |
| `spaced_repetition.rs` | 500 | SM-2 算法实现 | ✅ |
| `workflow_error_handler.rs` | 490 | 工作流错误处理 | ✅ |

---

## D 组: 小型单文件 (29 文件, ~6,706 行)

### 按功能域分类

| 域 | 文件列表 | 总计 |
|----|---------|------|
| **开发/调试** | `debug_log_service`(417), `crash_logger`(182), `anr_watchdog`(72), `metrics_server`(93), `database.debug`(47), `startup_cleanup`(102) | 6 |
| **PDF/文档** | `page_rasterizer`(459), `deepseek_ocr_parser`(405), `pdf_protocol`(372), `pdfium_utils`(304) | 4 |
| **配置/特性** | `feature_flags`(489), `backup_config`(720), `config_recovery`(204), `canonical_tools`(261) | 4 |
| **基础设施** | `secure_store`(538), `session_manager`(84), `persistent_message_queue`(195), `background_tasks`(56), `json_validator`(119), `tts`(203), `vector_store`(85) | 7 |
| **错误处理** | `error_details`(353), `error_recovery`(359), `workflow_error_handler`(490) | 3 |
| **导入管线** (Stage 3-5) | `cross_page_merger`(90), `figure_extractor`(170), `llm_structurer`(309) | 3 |
| **包管理** | `package_manager`(337) | 1 |
| **OCR** | `ocr_circuit_breaker`(283) | 1 |

---

## 综合评估

### God File 排行榜 (后端完整版)

| 排名 | 文件 | 行数 | 状态 |
|------|------|------|------|
| 1 | `vfs/handlers.rs` | 7,431 | R23 已报告 |
| 2 | `data_governance/sync/mod.rs` | 7,463 | R24 已报告 |
| 3 | `dstu/handlers.rs` | 6,379 | R24 已报告 |
| 4 | `database/mod.rs` | 6,043 | R20 已报告 |
| 5 | `llm_manager/mod.rs` | 5,994 | R22 已报告 |
| 6 | **`commands.rs`** | **5,839** | ⭐ 新 |
| 7 | `chat_v2/tools/chatanki_executor.rs` | 5,758 | R21 已报告 |
| 8 | `llm_manager/model2_pipeline.rs` | 5,567 | R22 已报告 |
| 9 | `vfs/indexing.rs` | 4,632 | R23 已报告 |
| 10 | **`lance_vector_store.rs`** | **4,491** | ⭐ 新 |
| 11 | `chat_v2/repo.rs` | 4,371 | R21 已报告 |
| 12 | **`question_import_service.rs`** | **3,953** | ⭐ 新 |
| 13 | `chat_v2/types.rs` | 3,670 | R21 已报告 |
| 14 | **`document_parser.rs`** | **3,620** | ⭐ 新 |
| 15 | **`notes_exporter.rs`** | **3,052** | ⭐ 新 |

### 架构评估

| 评估项 | 结论 |
|--------|------|
| `commands.rs` vs `cmd/` | 双轨并存。`commands.rs` 现在是兼容层（re-export cmd/），但 5839 行表明仍有大量代码未迁移 |
| 题目导入管线 | 设计优雅（6 Stage pipeline），但 Stage 1-5 全在一个文件 |
| 文档解析 | 7 种格式解析集中在一个文件，结构良好但过大 |
| 数据模型 | `models.rs` 与前端 `types/index.ts` 完全对称的 God File |
| 向量存储 | LanceDB 实现完整，但单文件 4491 行需要模块化 |
| 中/小型文件 | 23+29=52 个文件，大部分职责清晰，组织合理 |

### 建议

1. `commands.rs` — 确定退役计划，将剩余命令迁移到 `cmd/`
2. `lance_vector_store.rs` — 拆分为 table.rs / search.rs / index.rs / config.rs
3. `question_import_service.rs` — 每个 Stage 独立文件，主文件为编排层
4. `document_parser.rs` — 每种格式独立文件 (pdf.rs / docx.rs / pptx.rs...)
5. `models.rs` — 按域拆分为 models/chat.rs / models/anki.rs / models/api.rs
