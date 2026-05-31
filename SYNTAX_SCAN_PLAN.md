# 语法错误扫描计划

> 创建时间: 2026-05-31 | 项目: C:\deep-student
> 目标: 分批扫描所有源文件，仅检查语法错误（不编译），逐批修复
> 约束: 每批次加载文件量 ≤ 上下文窗口的 50%，避免上下文溢出

---

## 项目文件总览

| 类别 | 文件数 | 预估行数 |
|------|--------|----------|
| Rust 源文件 (*.rs) | 397 | ~340,000 |
| TypeScript/TSX (*.ts, *.tsx) | 1,487 | ~428,000 |
| Study-UI TS/TSX | 88 | ~11,000 |
| TOML/JSON/YAML 配置 | ~30 | ~35,000 |
| **总计** | **~2,002** | **~814,000** |

---

## 扫描策略

1. **Rust 优先**: Rust 文件数量少但代码行多，语法错误影响编译，优先处理
2. **按模块分批**: 同一模块的文件放在同一批次，保持上下文连贯
3. **大小均衡**: 每批次控制在 ~30-50 个文件或 ~15,000-25,000 行
4. **先核心后外围**: lib.rs → 核心模块 → 工具模块 → 前端
5. **检查方式**: 静态阅读代码，识别 Rust 语法错误（缺少分号、括号不匹配、类型错误、use 路径错误等）

---

## 第一阶段: Rust 源文件 (src-tauri/src/)

### 批次 R1: 根级小文件 + adapters
- **预估行数**: ~6,500 | **文件数**: ~20
- `main.rs` (6)
- `batch_operations.rs` (63)
- `background_tasks.rs` (56)
- `anr_watchdog.rs` (72)
- `crash_logger.rs` (182)
- `config_recovery.rs` (204)
- `cross_page_merger.rs` (90)
- `database_optimizations.rs` (66)
- `database.debug.rs` (47)
- `canonical_tools.rs` (261)
- `feature_flags.rs` (489)
- `injection_budget.rs` (569)
- `json_validator.rs` (119)
- `reasoning_policy.rs` (691)
- `session_manager.rs` (84)
- `startup_cleanup.rs` (102)
- `menu.rs` (202)
- `adapters/mod.rs` (4)
- `adapters/gemini-openai-converter.rs` (2,431)
- `secure_store.rs` (538)

### 批次 R2: 核心入口文件
- **预估行数**: ~10,200 | **文件数**: 3
- `lib.rs` (2,281)
- `commands.rs` (5,839)
- `models.rs` (2,120)

### 批次 R3: 内部服务文件 (Part 1)
- **预估行数**: ~17,000 | **文件数**: ~12
- `anki_connect_service.rs` (2,431)
- `apkg_exporter_service.rs` (1,439)
- `document_parser.rs` (3,620)
- `document_processing_service.rs` (722)
- `enhanced_anki_service.rs` (1,016)
- `exam_sheet_service.rs` (1,694)
- `file_manager.rs` (1,375)
- `notes_exporter.rs` (3,052)
- `notes_manager.rs` (2,205)
- `error_details.rs` (353)
- `error_recovery.rs` (359)

### 批次 R4: 内部服务文件 (Part 2)
- **预估行数**: ~19,000 | **文件数**: ~13
- `question_bank_service.rs` (2,275)
- `question_export_service.rs` (505)
- `question_import_service.rs` (3,953)
- `question_sync_service.rs` (2,133)
- `streaming_anki_service.rs` (2,380)
- `pdf_ocr_service.rs` (1,444)
- `paddleocr_api.rs` (455)
- `deepseek_ocr_parser.rs` (405)
- `page_rasterizer.rs` (459)
- `pdfium_utils.rs` (304)
- `pdf_protocol.rs` (372)
- `ocr_circuit_breaker.rs` (283)
- `unified_file_manager.rs` (718)
- `vlm_grounding_service.rs` (1,174)

### 批次 R5: 更多服务 + 工具
- **预估行数**: ~17,000 | **文件数**: ~15
- `debug_log_service.rs` (417)
- `debug_logger.rs` (715)
- `debug_commands.rs` (885)
- `persistent_message_queue.rs` (195)
- `tts.rs` (203)
- `voice_input.rs` (543)
- `vector_store.rs` (85)
- `lance_vector_store.rs` (4,491)
- `metrics_server.rs` (93)
- `package_manager.rs` (337)
- `spaced_repetition.rs` (500)
- `review_plan_error.rs` (50)
- `review_plan_service.rs` (911)
- `workflow_error_handler.rs` (490)
- `llm_structurer.rs` (309)
- `figure_extractor.rs` (170)
- `backup_common.rs` (920)
- `backup_config.rs` (720)
- `backup_job_manager.rs` (1,300)
- `data_space.rs` (1,401)

### 批次 R6: VFS 核心模块
- **预估行数**: ~18,000 | **文件数**: ~15
- `vfs/mod.rs` (137)
- `vfs/types.rs` (3,147)
- `vfs/error.rs` (239)
- `vfs/database.rs` (1,315)
- `vfs/handlers.rs` (7,342)
- `vfs/index_handlers.rs` (291)
- `vfs/index_service.rs` (399)
- `vfs/indexing.rs` (4,632)
- `vfs/ref_handlers.rs` (2,543)
- `vfs/todo_handlers.rs` (372)
- `vfs/ocr_storage.rs` (96)
- `vfs/ocr_storage_handlers.rs` (59)
- `vfs/ocr_utils.rs` (181)
- `vfs/attachment_config.rs` (138)

### 批次 R7: VFS repos (Part 1)
- **预估行数**: ~18,000 | **文件数**: ~12
- `vfs/repos/mod.rs` (94)
- `vfs/repos/attachment_repo.rs` (2,704)
- `vfs/repos/file_repo.rs` (1,739)
- `vfs/repos/folder_repo.rs` (3,072)
- `vfs/repos/note_repo.rs` (1,633)
- `vfs/repos/exam_repo.rs` (1,636)
- `vfs/repos/essay_repo.rs` (1,539)
- `vfs/repos/textbook_repo.rs` (1,701)
- `vfs/repos/question_repo.rs` (2,394)
- `vfs/repos/translation_repo.rs` (974)

### 批次 R8: VFS repos (Part 2) + 其他 VFS
- **预估行数**: ~17,000 | **文件数**: ~15
- `vfs/repos/mindmap_repo.rs` (1,366)
- `vfs/repos/resource_repo.rs` (1,075)
- `vfs/repos/todo_repo.rs` (1,772)
- `vfs/repos/pomodoro_repo.rs` (195)
- `vfs/repos/review_plan_repo.rs` (1,097)
- `vfs/repos/blob_repo.rs` (605)
- `vfs/repos/embedding_repo.rs` (841)
- `vfs/repos/embedding_dim_repo.rs` (561)
- `vfs/repos/index_segment_repo.rs` (297)
- `vfs/repos/index_unit_repo.rs` (571)
- `vfs/repos/path_cache_repo.rs` (1,254)
- `vfs/repos/pdf_preview.rs` (351)
- `vfs/lance_store.rs` (1,492)
- `vfs/multimodal_service.rs` (979)
- `vfs/pdf_processing_service.rs` (3,421)
- `vfs/embedding_service.rs` (620)
- `vfs/unit_builder/` 全部 (4 files, ~741)

### 批次 R9: Chat V2 核心
- **预估行数**: ~18,000 | **文件数**: ~15
- `chat_v2/mod.rs` (219)
- `chat_v2/types.rs` (3,670)
- `chat_v2/context.rs` (1,013)
- `chat_v2/database.rs` (658)
- `chat_v2/error.rs` (215)
- `chat_v2/events.rs` (1,834)
- `chat_v2/repo.rs` (4,371)
- `chat_v2/resource_types.rs` (663)
- `chat_v2/skills.rs` (520)
- `chat_v2/state.rs` (457)
- `chat_v2/prompt_builder.rs` (883)
- `chat_v2/user_message_builder.rs` (436)
- `chat_v2/variant_context.rs` (1,410)
- `chat_v2/vfs_resolver.rs` (2,662)

### 批次 R10: Chat V2 pipeline + handlers
- **预估行数**: ~16,000 | **文件数**: ~20
- `chat_v2/approval_manager.rs` (568)
- `chat_v2/approval_scope.rs` (689)
- `chat_v2/pipeline.rs` (839)
- `chat_v2/pipeline_tests.rs` (2,094)
- `chat_v2/pipeline/compaction.rs` (1,417)
- `chat_v2/pipeline/constants.rs` (95)
- `chat_v2/pipeline/helpers.rs` (878)
- `chat_v2/pipeline/history.rs` (582)
- `chat_v2/pipeline/llm_adapter.rs` (968)
- `chat_v2/pipeline/multi_variant.rs` (3,045)
- `chat_v2/pipeline/persistence.rs` (1,085)
- `chat_v2/pipeline/prompt.rs` (225)
- `chat_v2/pipeline/retrieval.rs` (497)
- `chat_v2/pipeline/summary.rs` (437)
- `chat_v2/pipeline/token_resources.rs` (355)
- `chat_v2/pipeline/tool_loop.rs` (2,178)
- `chat_v2/pipeline/variant_adapter.rs` (553)
- `chat_v2/handlers/` (14 files, ~8,900)

### 批次 R11: Chat V2 tools (Part 1)
- **预估行数**: ~18,000 | **文件数**: ~18
- `chat_v2/tools/mod.rs` (212)
- `chat_v2/tools/arg_utils.rs` (69)
- `chat_v2/tools/types.rs` (244)
- `chat_v2/tools/injector.rs` (116)
- `chat_v2/tools/registry.rs` (261)
- `chat_v2/tools/executor.rs` (636)
- `chat_v2/tools/executor_registry.rs` (417)
- `chat_v2/tools/academic_search_executor.rs` (1,093)
- `chat_v2/tools/anki_executor.rs` (478)
- `chat_v2/tools/ask_user_executor.rs` (291)
- `chat_v2/tools/attachment_executor.rs` (541)
- `chat_v2/tools/builtin_resource_executor.rs` (3,627)
- `chat_v2/tools/builtin_retrieval_executor.rs` (2,064)
- `chat_v2/tools/canvas_executor.rs` (1,025)
- `chat_v2/tools/canvas_tools.rs` (850)
- `chat_v2/tools/chatanki_executor.rs` (5,758)

### 批次 R12: Chat V2 tools (Part 2) + workspace + migration
- **预估行数**: ~18,000 | **文件数**: ~18
- `chat_v2/tools/docx_executor.rs` (542)
- `chat_v2/tools/fetch_executor.rs` (1,151)
- `chat_v2/tools/general_executor.rs` (247)
- `chat_v2/tools/image_generation_executor.rs` (906)
- `chat_v2/tools/knowledge_executor.rs` (428)
- `chat_v2/tools/memory_executor.rs` (1,030)
- `chat_v2/tools/paper_save_executor.rs` (1,424)
- `chat_v2/tools/pptx_executor.rs` (639)
- `chat_v2/tools/qbank_executor.rs` (1,994)
- `chat_v2/tools/session_executor.rs` (1,299)
- `chat_v2/tools/skills_executor.rs` (479)
- `chat_v2/tools/sleep_executor.rs` (454)
- `chat_v2/tools/subagent_executor.rs` (356)
- `chat_v2/tools/template_executor.rs` (2,361)
- `chat_v2/tools/todo_executor.rs` (858)
- `chat_v2/tools/user_todo_executor.rs` (573)
- `chat_v2/tools/workspace_executor.rs` (908)
- `chat_v2/tools/xlsx_executor.rs` (628)
- `chat_v2/workspace/` (13 files, ~4,670)
- `chat_v2/migration/` (3 files, ~1,239)

### 批次 R13: DSTU 模块
- **预估行数**: ~14,000 | **文件数**: ~23
- `dstu/mod.rs` (143)
- `dstu/error.rs` (174)
- `dstu/types.rs` (1,007)
- `dstu/exam_formatter.rs` (350)
- `dstu/handlers.rs` (6,379)
- `dstu/folder_handlers.rs` (1,097)
- `dstu/path_parser.rs` (741)
- `dstu/path_types.rs` (412)
- `dstu/trash_handlers.rs` (798)
- `dstu/export/` (9 files, ~1,352)
- `dstu/handler_utils/` (8 files, ~3,578)

### 批次 R14: Data Governance
- **预估行数**: ~18,000 | **文件数**: ~28
- `data_governance/mod.rs` (262)
- `data_governance/commands.rs` (1,424)
- `data_governance/commands_asset.rs` (362)
- `data_governance/commands_backup.rs` (2,272)
- `data_governance/commands_restore.rs` (1,365)
- `data_governance/commands_sync.rs` (3,149)
- `data_governance/commands_types.rs` (171)
- `data_governance/commands_zip.rs` (1,472)
- `data_governance/init.rs` (429)
- `data_governance/plugin.rs` (16)
- `data_governance/schema_registry.rs` (878)
- `data_governance/critical_audit_tests.rs` (1,659)
- `data_governance/tests.rs` (1,089)
- `data_governance/migration_tests.rs` (1,801)
- `data_governance/audit/mod.rs` (838)
- `data_governance/backup/mod.rs` (3,822)
- `data_governance/backup/assets.rs` (1,474)
- `data_governance/backup/zip_export.rs` (862)
- `data_governance/dto/mod.rs` (126)
- `data_governance/migration/` (9 files, ~7,431)
- `data_governance/sync/` (8 files, ~9,652)

### 批次 R15: LLM Manager + LLM Usage
- **预估行数**: ~18,000 | **文件数**: ~17
- `llm_manager/mod.rs` (5,994)
- `llm_manager/builtin_vendors.rs` (1,398)
- `llm_manager/exam_engine.rs` (1,128)
- `llm_manager/model2_pipeline.rs` (5,567)
- `llm_manager/parser.rs` (600)
- `llm_manager/rag_extension.rs` (1,390)
- `llm_manager/adapters/` (15 files, ~5,800)

### 批次 R16: LLM Usage + Memory + Essay Grading + QBank
- **预估行数**: ~10,000 | **文件数**: ~20
- `llm_usage/` (6 files, ~3,160)
- `memory/` (12 files, ~6,838)
- `essay_grading/` (7 files, ~3,019)
- `review_plan_error.rs` (50)
- `review_plan_service.rs` (911)

### 批次 R17: Database + Providers + Vendors + Tools
- **预估行数**: ~14,000 | **文件数**: ~8
- `database/mod.rs` (6,043)
- `database/manager.rs` (2,559)
- `providers/mod.rs` (2,507)
- `tools/mod.rs` (1,292)
- `tools/web_search.rs` (2,991)
- `vendors/mod.rs` (11)
- `vendors/siliconflow.rs` (165)

### 批次 R18: Multimodal + OCR Adapters + Translation
- **预估行数**: ~12,000 | **文件数**: ~17
- `multimodal/` (8 files, ~6,553)
- `ocr_adapters/` (8 files, ~2,306)
- `translation/` (5 files, ~983)
- `qbank_grading/` (4 files, ~908)

### 批次 R19: CMD + Cloud Storage + Crypto + Test Utils
- **预估行数**: ~13,000 | **文件数**: ~22
- `cmd/` (11 files, ~6,628)
- `cloud_storage/` (6 files, ~2,999)
- `crypto/` (3 files, ~492)
- `test_utils/` (3 files, ~171)
- `mcp/` (12 files, ~2,600)

### 批次 R20: Utils + 剩余小文件
- **预估行数**: ~5,000 | **文件数**: ~15
- `utils/` (10 files, ~1,413)
- `services/mod.rs` (6)
- 其他遗漏的独立文件

---

## 第二阶段: TypeScript/TSX 源文件 (src/)

### 批次 T1: 入口 + API + Config + Contexts
- **预估行数**: ~8,000 | **文件数**: ~25
- `App.tsx` (2,672)
- `main.tsx` (682)
- `lazyComponents.tsx` (145)
- `i18n.ts` (141)
- `vite-env.d.ts` (17)
- `api/` 全部 (12 files, ~4,000)
- `config/` 全部 (8 files)
- `contexts/DialogControlContext.tsx` (627)

### 批次 T2: Stores (全部)
- **预估行数**: ~8,000 | **文件数**: ~14
- `stores/` 全部 (14 files)

### 批次 T3: Hooks (Part 1)
- **预估行数**: ~8,000 | **文件数**: ~20
- `hooks/` 前20个文件

### 批次 T4: Hooks (Part 2) + Lib + Services + Events
- **预估行数**: ~8,000 | **文件数**: ~25
- `hooks/` 剩余
- `lib/` (4 files)
- `services/` (6 files)
- `events/chat.ts`

### 批次 T5: Types + Utils (Part 1)
- **预估行数**: ~10,000 | **文件数**: ~25
- `types/` 全部 (17 files)
- `utils/` 前15个文件

### 批次 T6: Utils (Part 2)
- **预估行数**: ~10,000 | **文件数**: ~30
- `utils/` 剩余文件

### 批次 T7: MCP + Voice Input + Debug Panel
- **预估行数**: ~12,000 | **文件数**: ~40
- `mcp/` (6 files)
- `mcp-debug/` (9 files)
- `voice-input/` (15 files)
- `debug-panel/` 核心文件

### 批次 T8: DSTU + Essay Grading + Components (shared)
- **预估行数**: ~12,000 | **文件数**: ~40
- `dstu/` 全部 (25 files)
- `essay-grading/` (10 files)
- `components/shared/` (12 files)
- `components/ui/shad/` 部分

### 批次 T9-T20: Features 模块 (按子模块分批)
- **预估行数**: 每批 ~15,000-20,000 | **文件数**: 每批 ~30-60
- T9: `features/chat/` Part 1 (adapters, hooks, core)
- T10: `features/chat/` Part 2 (components Part 1)
- T11: `features/chat/` Part 3 (components Part 2, renderers, panels)
- T12: `features/chat/` Part 4 (plugins, pages, debug, dev)
- T13: `features/settings/` (60 files)
- T14: `features/notes/` (40 files)
- T15: `features/mindmap/` (30 files) + `features/pdf/` (5 files) + `features/sandbox/` (5 files) + `features/pomodoro/` (5 files) + `features/todo/` (8 files)
- T16: `components/` Part 1 (anki/cardforge/)
- T17: `components/` Part 2 (crepe, dev, essay-grading, layout)
- T18: `components/` Part 3 (skills-management, translation, practice, stats, llm-usage, style-lab)
- T19: `components/` Part 4 (root-level components: Dashboard, ModernSidebar, TemplateManager, etc.)
- T20: `components/` Part 5 (root-level 剩余)

### 批次 T21: Study UI + 配置文件
- **预估行数**: ~15,000 | **文件数**: ~120
- `.study-ui/src/` 全部 (88 TS/TSX + CSS)
- 配置文件 (TOML, JSON, YAML)

---

## 第三阶段: 跨文件一致性检查

### 批次 C1-C3: 交叉引用验证
- Rust module 声明与文件系统一致性
- TypeScript import 路径与文件系统一致性
- Tauri command 注册与前端调用一致性

---

## 批次汇总

| 阶段 | 批次 | 文件数 | 预估行数 |
|------|------|--------|----------|
| Rust | R1-R20 | 397 | ~340,000 |
| TypeScript | T1-T21 | 1,575 | ~440,000 |
| 交叉检查 | C1-C3 | — | — |
| **总计** | **~44 批次** | **~1,972** | **~780,000** |

---

## 检查标准

每个文件检查以下语法问题：

### Rust 文件
- [ ] 缺少分号 (;)
- [ ] 括号/大括号不匹配
- [ ] 错误的 use 路径 (模块不存在或路径错误)
- [ ] 类型不匹配 (函数签名 vs 实际使用)
- [ ] 未定义的变量/函数/类型引用
- [ ] match 表达式缺少分支
- [ ] 生命周期标注错误
- [ ] 泛型约束错误
- [ ] 宏调用语法错误
- [ ] mod 声明与实际文件不匹配
- [ ] 错误的可见性修饰符
- [ ] trait 实现缺失必需方法

### TypeScript/TSX 文件
- [ ] 缺少分号/括号
- [ ] import 路径不存在
- [ ] 类型引用错误 (不存在的类型/接口)
- [ ] JSX 语法错误
- [ ] 未定义变量/函数引用
- [ ] 函数参数类型不匹配
- [ ] 泛型使用错误
- [ ] 导出/导入不匹配 (default vs named)

---

## 使用说明

1. 每批次开始前，将本文件中的对应批次标记为 `🔄 进行中`
2. 批次完成后标记为 `✅ 完成`，记录完成时间和发现的问题数
3. 在 CLAUDE.md 中更新整体进度百分比
4. 如发现跨批次影响的错误，在问题追踪区记录
