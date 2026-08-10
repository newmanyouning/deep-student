# 实验功能与未完成功能盘点

> 生成时间: 2026-08-06 21:28 CST
> 扫描方式: 四路并行扫描 (Rust 后端 / 前端 TS / 文档计划 / 配置文件)
> 范围: 全仓库 ~3,000 文件
> 用途: 挑选下一个实验功能的候选清单

---

## 一、总览

| 类别 | 数量 | 说明 |
|------|------|------|
| 🧩 完整功能桩 (可用性为 0) | 12 | 前后端已接线,但实现返回 "not implemented" / "即将上线" |
| 🔩 部分实现 (功能残缺) | 18 | 主流程可用,某个能力缺失或返回空数据 |
| 🚫 已禁用 / 隐藏 | 10 | 代码在但被 flag / 注释 / 权限关闭 |
| 📋 规划中未执行 | 9 | 设计文档已写,代码未动 |
| 🏷️ 已发货但标"实验性" | 9 | 正在用,UI/文档里带实验徽章 |
| 🧹 预留 / 死代码 / 清理项 | 14 | 占位符、幽灵依赖、遗留注释 |

---

## 二、🧩 完整功能桩 (前后端已接线,调用即报错)

### 1. Research 深度研究模块 — 26 个后端命令全是桩 🔥
- **位置**: `src-tauri/src/cmd/research_stubs.rs` (整文件), 注册于 `lib.rs:1711-1736`
- **现状**: 26 个 `research_*` 命令全部返回 `not_implemented`; 前端 `src/utils/settingsApi.ts:85-208` 已封装调用; `researchStore.ts` (506 行) 无任何 UI 消费者
- **已建好的部分**: `research_reports` 表 + 报告 CRUD (`commands.rs:4111-4175`, `database/mod.rs:3637-3714`) — **报告存储已就绪,只缺引擎**
- **完整设计**: `docs/RESEARCH_MODULE_INTENT_ANALYSIS.md` — HPIAS (分层渐进信息获取系统), 多轮研究、plan→审批、并行子代理、批判评审、宏洞察
- **另一条路**: 文档建议 Option B — 用现有 `chat_v2/tools/subagent_executor.rs` 的 `subagent_call` 增强 research-mode 技能,实现"多子代理"效果,不必写 26 个命令
- **决策点**: 实现 or 删除 (MASTER_REFACTORING_PLAN Task 1.2 同样在等这个决定)

### 2. 命令面板 ~40 个"幽灵命令" 🔥
- **位置**: `src/command-palette/registry/capabilityRegistry.ts` — 所有未实现命令标 `'hidden'`
- **内容**: 学习事件 22 个 (`learning.commands.ts`, 每个都 `// TODO: 未实现` — 发事件但无监听者)、聊天 (分享/导出/语音输入/AI 续写/分支对话)、全局 (快速搜索/通知/同步/lock-app) 等
- **现状**: 已从面板过滤掉,强制调用也是 no-op
- **注意**: registry 里的 `'experimental'` 状态类型定义了但从未使用

### 3. 导入对话快照 — 永远失败的对话框
- **位置**: `src/components/ImportConversationDialog.tsx:69` → `src/utils/systemApi.ts:443-458`
- **现状**: 完整对话框 UI (文件选择、警告列表、成功面板) 都存在,但调用 `importConversationSnapshot` 总是返回 `{ success: false, message: '导入功能尚未实现' }`
- **附带**: `saveImageToImagesDir` (systemApi.ts:460-470) 同样总是返回空路径

### 4. DSTU 编辑器 create 模式 — 全是 "coming soon"
- **位置**: `src/dstu/editors/` — `NoteEditorWrapper.tsx:80-96`, `PDFViewerWrapper`, `ImageViewerWrapper`, `FileViewerWrapper`, `TranslationViewerWrapper`, `MindMapEditorWrapper`, `ExamEditorWrapper`
- **现状**: 打开/编辑已有文件可用,但新建笔记/考试/思维导图走 create 模式时渲染 "创建模式暂不支持" 或空壳
- **头注释**: "当前实现为占位组件,等待 VFS 后端完成后将连接到实际组件"

### 5. 统计面板占位卡 — "即将推出"
- **位置**: `src/components/SOTADashboardLite.tsx:359-376` (科目统计), `:408-425` (统一回顾统计)
- **现状**: 两张卡 opacity-60, 值显示 "—", 徽章 "即将推出"; 后端对应统计字段已被移除

### 6. Windows 系统 TTS — 桩 + 谎报可用
- **位置**: `src-tauri/src/tts.rs:105-117` (`speak_windows`), `:93-99` (`tts_stop` 只打印一行)
- **现状**: `speak_windows` 返回 "Windows TTS 需要额外配置,请使用 Web Speech API"; 但 `tts_check_available` 声称可用 — 用户在 Windows 上调用静默无效果
- **成本极低**: 依赖注释写着"需要额外的 crate 依赖",但 `windows` crate 已在 Cargo.toml;macOS (`say`) / Linux (`espeak`) 都已实现

### 7. MCP OAuth 2.1 交互式授权 — 缺最后一步
- **位置**: `src-tauri/src/mcp/sse_transport.rs:313-328` (`perform_oauth_authentication`)
- **现状**: 返回 `AuthenticationError`, 注释 "OAuth 2.1 interactive flow 尚未完整实现(需要打开浏览器 + 处理回调)"
- **已建好**: `mcp/auth.rs` — PKCE challenge 生成、授权 URL、CSRF/verifier 校验、code exchange **全部已实现**,只缺"打开系统浏览器 + 本地回调接收"胶水
- **价值**: 解锁只支持 OAuth 的 MCP 服务器

### 8. 向量库 `load_document_chunks` — trait 默认未实现, Lance 已覆盖
- **位置**: `src-tauri/src/vector_store.rs:60-65` (trait 默认), `lance_vector_store.rs:3036-3100` (**已实现**: 跨候选维度表按 document_id 过滤 + chunk_index 排序)
- **现状**: 非 Lance 后端会命中 not_implemented 默认实现; Lance 主路径已可用
- **用途**: 增量重索引 / 全文操作需要读取文档全部 chunk

### 9. `backfillUserMessageEmbeddings` — 前端桩
- **位置**: `src/utils/chatApi.ts:202-214`
- **现状**: 返回 0, 注释 "后端命令尚未实现"; 用途: 用户消息 embedding 回填 (语义搜索聊天记录)

### 10. iOS 系统 OCR — 待实现
- **位置**: `src-tauri/src/ocr_adapters/system_ocr/mod.rs:5-9, 55-60`
- **现状**: macos / windows 已实现;iOS 13+ Apple Vision (`VNRecognizeTextRequest`) 标注"待实现",移动端走 `Unsupported`

### 11. Workspace Agent Skills — 空壳模块
- **位置**: `src-tauri/src/chat_v2/workspace/skills.rs` (33 行)
- **现状**: `get_skill` → `None`, `list_skills` → `[]`, 注释 "空壳模块… 兼容性占位符,将在后续版本移除"; 前端注入 skills 数据

### 12. Find & Replace 面板 — no-op UI
- **位置**: `src/features/notes/components/FindReplacePanel.tsx:40-55`
- **现状**: `handleFind` 空实现 (注释: 依赖浏览器原生 Ctrl+F), 替换按钮禁用; `NotesCrepeEditor.tsx:741` "TODO: 待 Crepe 支持后重新实现"

---

## 三、🔩 部分实现 (主流程可用,某能力缺失)

| # | 功能 | 位置 | 缺失部分 |
|---|------|------|---------|
| 1 | 知识点统计历史对比 | `question_bank_service.rs:1180-1195` | `previous` 恒为空, "TODO: 实现历史快照对比" |
| 2 | 作文批改会话搜索 | `essay_grading/mod.rs:170-184` | `_query` 参数被忽略, "TODO: 添加搜索支持" |
| 3 | Markdown 笔记导入冲突策略 | `notes_exporter.rs:2813` | `overwritten_count: 0`, 导入直接覆盖无冲突处理 |
| 4 | 云同步 `last_sync_at` | `data_governance/commands_sync.rs:341, 768-775` | 状态永远显示无同步时间; 同步命令要求调用方自供云配置, 无法一键同步 |
| 5 | Pipeline 记忆检索块 | `chat_v2/pipeline/retrieval.rs:334-357` | 占位返回空 (已被 builtin-memory_search 工具取代, 死分支仍发 MEMORY 块) |
| 6 | 回顾分析统计 | `commands.rs:3761-3777` | "回顾分析功能已移除", 返回硬编码全零 |
| 7 | 多变体 chat token 汇总 | `multi_variant.rs:560-564` | `stream_complete` 事件 usage 为 None, 各变体统计未合并 |
| 8 | 滚动上下文摘要 | `utils/token_budget.rs:179-184` | 占位字符串 "⋯context trimmed⋯" (真实压缩在 compaction.rs, 已完成) |
| 9 | 试卷导出 PDF/Word | `src/components/practice/PaperGenerator.tsx:147-151` | 按钮已渲染, `handleExport` 只 console.log |
| 10 | 图像生成重试 | `src/features/chat/plugins/blocks/imageGen.tsx:262-265` | `handleRetry` 只打日志 |
| 11 | 压缩摘要"查看原始对话" | `compactionSummary.tsx:15` | 按钮 + 后端 `chat_v2_get_compacted_range` 均缺 (P3) |
| 12 | Anki 文档处理状态 | `cardforge/engines/TaskController.ts:425-459` | 命令缺失, 从任务列表降级推导 |
| 13 | 磁盘空间检查 | `src/api/dataGovernance.ts:1144-1160` | 后端命令缺失时静默返回"空间足够" |
| 14 | 云同步合并策略 | `SyncSettingsSection.tsx:245` | "keep_latest" 硬编码, "TODO: 从配置中获取" |
| 15 | 学习中心删除确认 | `useLearningHub.ts:415` | 直接删除无确认对话框 |
| 16 | 命令面板确认对话框 | `commandRegistry.ts:397` | `requireConfirm` 命令跳过确认 |
| 17 | 番茄钟系统通知 | `usePomodoroStore.ts:8` | Phase 3 TODO: 目前用 WebAudio 蜂鸣 |
| 18 | 教科书模式插件 | `chat/plugins/modes/textbook.ts:108-149` | 已禁用, 残留 mock 加载器 (mock:// 占位 URL) |

---

## 四、🚫 已禁用 / 隐藏的功能

### 1. 后端 MCP 客户端子系统 — 完整构建但默认禁用
- **位置**: `src-tauri/src/mcp/` (client.rs, global.rs, 4 种 transport, auth.rs PKCE), `lib.rs:710-724`
- **现状**: 完整子系统编译进包 (`mcp` 是默认 feature), 但 DB 设置 `mcp.mode=backend` 才初始化, 默认 `"frontend"` (前端 SDK 模式); `get_mcp_status` 返回 `"backend_mcp_disabled"`
- **决策点**: 完成前端 SDK 迁移的清理, 或重新启用后端模式

### 2. 多模态 (VL) 索引与搜索 — flag 翻转不一致
- **位置**: `src/mcp/builtinMcpServer.ts:108-139` (`multimodal_search` 工具整体注释掉)、`src/utils/chatApi.ts:166-179` (教材导入后自动索引被注释)、`src/services/multimodalRagService.ts`
- **现状**: `MULTIMODAL_INDEX_ENABLED = true` 但头注释说 "当前禁用", 守卫处静默返回空; 调试面板插件也被注释
- **意图**: 用 VL embedding 模型做图片/扫描 PDF 视觉搜索

### 3. `http` Cargo feature — 定义但编译不过
- **位置**: `Cargo.toml:233`, `tools/web_search.rs:2877-2913`
- **现状**: web_search 工具的可选 HTTP 服务器模式, `run_http` 分支依赖 axum, 但 axum **不在依赖里** — 启用即编译失败; 不启用则返回 "HTTP feature not enabled"
- **选择**: 加 axum 或删掉分支

### 4. 运行时 feature flags — 4 个功能默认关闭
- **位置**: `src-tauri/src/feature_flags.rs:253-338` (DB 支撑, 有前端命令)
- **默认关闭**: `search.reranker` (需要 LLM 配置)、`ui.engine_status_panel`、`ui.mcp_tool_hover`、`ui.error_dialog_actions`; `observability.performance_metrics` 30% 渐进灰度
- **注意**: 这套系统本身是完整的, 直接打开 flag 即可上线

### 5. 前端 mcp-debug 模块 — 生产构建替换为 no-op
- **位置**: `vite.config.ts:33-43` (4573 行模块), Cargo feature `mcp-debug`
- **现状**: 开发专用 AI 调试桥 (截图/DOM/IPC/输入模拟), 正确 opt-in; `VITE_ENABLE_MCP_DEBUG` 可强制打开

### 6. 多模态视频输入 — 类型预留, 无消费方
- **位置**: `src-tauri/src/multimodal/types.rs:23, 40-42`
- **现状**: `MultimodalVideo` 字段已定义 ("预留扩展(未来支持)"), 无 pipeline 读取

### 7. 统一 OCR 服务类型 — 预留死代码
- **位置**: `src-tauri/src/ocr_adapters/types.rs:210-289`
- **现状**: `OcrRequest` / `ImageSource` / `OcrAdapterConfig` 标 "预留, 用于未来的统一 OCR 服务", `#[allow(dead_code)]`

### 8. 非 lance 构建的向量库 — 占位
- **位置**: `src-tauri/src/multimodal/vector_store.rs:84, 114-121`
- **现状**: 所有操作返回 "Lance feature 未启用"; 仅当 `lance` feature 关闭时走这里

### 9. 全局 UI 缩放 / 增强渲染模式 — 实验开关
- **位置**: 设置页 `SystemSettingsSection.tsx:354, 406-412`; 缩放标 "测试", 增强模式标 "实验", 调试模式标 "开发中"
- **现状**: 功能真实存在, 只是官方标注未充分验证

### 10. `.env.example` 未启用开关
- `VITE_ENABLE_REACT_GRAB` (React DevTools 集成)、`VITE_ENABLE_SMOOTH_SCROLL` — 注释掉, 从未启用

---

## 五、📋 规划中未执行 (文档已写, 代码未动)

### 1. PDF 自动分片 (PaddleOCR) — "Implementation Ready" 但文件不存在 🔥
- **位置**: `docs/analysis/PDF_AUTO_SPLIT_DESIGN.md` (2026-06-02, 状态: Final)
- **内容**: 大 PDF 提交 AI Studio 作业 API 前自动分片 (20MB / 50 页阈值、并发上传、结果合并), 估算 ~321 LoC, 指定新文件 `paddleocr_split.rs` — **不存在**
- **附带 5 个未来方向**: 自然分节边界分片、按页结果缓存、流式上传、可配置阈值、非 PDF (TIFF/DJVU) 分片

### 2. 轻量翻译 Popover — backlog 项, 目录是空的
- **位置**: `.planning/ROADMAP.md` Backlog; `.planning/phases/999.1-lightweight-translation-popover/` (**空目录**)
- **内容**: 选中文本点 Translate → 原位弹出小卡片流式译文, 附带复制/收藏/发到 Chat; TranslateWorkbench 保留为长文入口
- **与当前分支的关系**: 当前分支 pr/3-pdf-reading 已改过 `translation/chat_popover.rs` — 可能是该功能的雏形, 值得核对

### 3. v1.2 性能基线 (Phases 8-12) — 工具都没装
- **位置**: `.planning/ROADMAP.md` Active 区 (全 `[ ]`)
- **内容**: 修 17 个 TS 错误 + tsc CI 门槛; 装 `vite-bundle-visualizer` (**不在 devDependencies**); `React.lazy` 拆分 Milkdown/Pptx/Xlsx/Settings/调试面板; `manualChunks` 分 6 个 vendor 块, 主包 1.2MB → ≤500KB; iOS 冷启动基线
- **已有一半**: package.json 有 `analyze` script, vite.config 有 1 处 manualChunks

### 4. CSS 架构迁移到 Tailwind v4 — 已批准未执行
- **位置**: `docs/plans/2026-05-13-css-architecture-migration-{design,plan}.md` (状态: Approved, 分支 study-ui-migration)
- **内容**: 迁移 12K 行 App.css + 3K 行 DeepStudent.css + 60+ 散落 CSS → Tailwind v4 + CSS Modules + 语义设计令牌
- **现状**: package.json 仍锁 `tailwindcss ^3.4.19`

### 5. 云同步架构增强 — 分析完成
- **位置**: `docs/cloud-sync-compatibility-analysis-2026-05-23.md`
- **现状**: 仅 13 张表有行级同步; 未覆盖 `mindmaps`、`review_history`、`todo_items`、`pomodoro_records`、`memory_*`、`chat_v2_attachments`、`settings` 等
- **目标形态**: 本地 SQLite 主存储 + 领域变更流 + 内容寻址对象存储 + 派生数据本地重建 + CRDT; 参考 Replicache/Zero mutation 思路 + PowerSync + Electric
- **规模**: 最大工程项之一

### 6. Master Refactoring Plan — 6 个阶段全 "Planned"
- **位置**: `docs/plans/MASTER_REFACTORING_PLAN.md` (~70-90 人日)
- **内容**: 8 个 god-file 拆分 (vfs/handlers.rs 7,324 行等)、模块提取 (question_repo → qbank/)、依赖升级 (thiserror 1→2, reqwest 0.11→0.12, hyper, oauth2 4.4→5.x, zip 0.6→2.x)、Tauri `specta` 自动 TS 类型
- **注**: 错误类型标准化部分已被 CLAUDE.md 记录的工作覆盖, 结构部分未动

### 7. Feature Audit 路线图 — 98 项功能打分后的方向
- **位置**: `docs/FEATURE_AUDIT_REPORT.md` (2026-07-12)
- **v1.0 P0**: O-1 协议统一 (DSTU 唯一前端入口, VFS 降为内部服务); O-2 工具统一 (所有工具经 MCP 协议注册); 删 8 个 F/D 级功能; 合并 7 组碎片模块
- **v1.1**: 云同步 GA (实验→稳定)、记忆系统并入 VFS 作为一等资源、多模态知识库提升到可用
- **P2 忠告**: "不要继续增加功能 — 98 项中超过一半需要改进而非增加"
- **AI-06 子代理执行被评 F 级 (实验性/利基)**

### 8. 功能合并提案 — 未来选项
- **位置**: `docs/FEATURE_MERGE_REPORT.md`
- 复习计划 vs Anki 未来共享 SM-2 模块; Todo vs 会话待办桥接 ("保存 AI 计划为用户 Todo"); 作文批改管线暴露为 chat_v2 skill (仿 research-mode); 统一练习入口 `start_practice_session(mode: daily|timed|mock)`

### 9. 记忆架构 L1/L3 层 — 建议未建
- **位置**: `docs/MEMORY_ARCHITECTURE_COMPARISON.md` (2026-07-26)
- **建议**: 不动现有架构, 加 **L1 会话工作上下文** 和 **L3 事件日志** 两层, MemoryService 作为 L2+L3 事实层保留

---

## 六、🏷️ 已发货但标为"实验性"

| 功能 | 位置 | 标注 |
|------|------|------|
| 云同步 (S3/WebDAV) | `SyncTab.tsx:141-143, 220-222` | UI 琥珀色"实验版"徽章; README "△ experimental"; FEATURE_AUDIT v1.1 目标转 GA |
| 实验版更新渠道 | `useAppUpdater.ts:30`, `AboutTab.tsx:189-199` | 用户可选接收实验版更新 |
| 多模型对比 | README | 标记 experimental |
| 子代理执行 (subagent_call) | README + FEATURE_AUDIT | 标记 experimental, 评分 F 级但**是 research-mode 的现成底座** |
| 全局 UI 缩放 | settings.json | "全局界面缩放(实验)", 桌面端专用 |
| Markdown 增强渲染模式 | SystemSettingsSection.tsx:354 | "增强模式(实验)" |
| 调试模式 | SystemSettingsSection.tsx:406 | "仍在开发中" 徽章 |
| data_governance 模块 | Cargo feature 默认开启 | UI 标实验, 功能已默认发货 |
| Gemini 预览模型 | `scripts/gemini-model-registry.json` | `gemini-omni-flash-preview` (视频生成)、`gemini-3.1-pro-preview` 作为内置默认模型发货; `gemini-3.5-pro` 注释"尚未发布" |

---

## 七、🧹 预留 / 死代码 / 清理项

### 已实现但从未接到 UI (~93 个命令) 🔥
- **位置**: `docs/architecture/DIAGNOSTIC_REPORT.md` §W1 — "部分命令可能是未来功能预留,但缺乏标记"
- **未接线组**: PDF/考试会话 (~20)、题库导入 (~10)、CSV 导入导出 (4)、语音/调试 (~8)、安全/功能开关 (~12)、LLM 配置 CRUD (~15)、OCR 引擎配置 (~14)、Anki Connect (~10)
- **动作**: 要么接线要么标 `UNUSED`; 其中部分就是第五节 7 里"删 8 个 F/D 级功能"的素材

### 幽灵依赖与权限
- `@tauri-apps/plugin-os` — package.json 声明 + shim, 前后端都没用, Rust 端未注册 → 可删
- `capabilities/test.json` — **生产构建**给主窗口永久 `https://*/*` + `http://*/*` 无限制出网权限 (无 platforms 限制) → 建议加 `platforms: ["windows"]` 或移出生产合并
- `capabilities/default.json:21` — 生产能力含 `core:webview:allow-internal-toggle-devtools` (正式版可切 devtools) → 建议移除
- `vite.config.ts:60-64` — React 项目里定义 Vue 的 `__VUE_OPTIONS_API__` 等常量 (无意义残留)

### Rust 端保留代码
- `pdf_ocr_service.rs:859-915` — `init_pdfium` / `render_page_to_image` "保留供未来使用"
- `question_import_service.rs:1379-1389` — `import_docx_with_images` 已废弃 (VLM 导入取代)
- `model2_pipeline.rs:5062-5068` — `convert_image_to_markdown` 已废弃
- `database/manager.rs:2553-2558` — `migrate_builtin_templates_to_db` 禁用
- `vfs/indexing/mod.rs:1615-1619` — 废弃 `index_resource` 写假 `placeholder_no_lance_*` row id (复活会损坏数据)
- `chat_v2/pipeline/persistence.rs:512-516` — `generated_blocks` 兼容分支恒空
- `data_governance/plugin.rs` — 整文件是未实现插件模式的文档
- `mcp/client.rs:1975` — `examples` 模块仅 feature 开启时编译

### 前端残留
- 孤儿 locale 字符串: `notes.json` 导入/导出"开发中,敬请期待"、`learningHub.json` "新建功能即将上线"、`anki.json` "该操作即将上线" — 无组件引用
- `common.json` `dev_features_title/desc` ("控制实验与开发功能") — 无组件引用
- 图谱模块废弃占位类型 (`notesUtils.ts`, `shared.ts`, `ankiSourceBuilder.ts`)
- `PdfReader.tsx` 临时保留 react-pdf 实现, 待迁移 BasePdfViewer
- `deepseek_ocr_parser.rs` 部分废弃, 迁移到 `ocr_adapters/deepseek.rs` 未完成
- 过时注释: `chat_v2/mod.rs:13` "pipeline 待实现" (已实现)、`lance_vector_store.rs:151` "占位骨架" (已 4500 行)、`vfs/handlers/mod.rs` "占位" (已实现)
- `NotesSidebar.tsx:380` 手动输入教材 ID 提示"即将实现选择器", 但 ReferenceSelector 组件已存在 (遗留路径)
- 事务缺口: `attachment_repo.rs:632`, `file_handlers.rs:289` — 多步上传未包事务

---

## 八、📌 推荐实现顺序 (按 价值/成本 比)

### 🟢 P0 — 小成本高价值 (1-3 天级别)
| 功能 | 成本 | 价值 |
|------|------|------|
| **Windows TTS** — `windows` crate 已在依赖里, SAPI 调用 ~50 行 | 极低 | 修复谎报可用的功能 |
| **PDF 自动分片** — 设计 Final, 明确 ~321 LoC, 新文件 | 低 | 解锁 >20MB/50 页的教材 OCR |
| **知识点历史对比 / 作文搜索 / last_sync_at / Markdown 导入冲突** | 极低 | 4 个 "TODO 缺一行" 型补全 |
| **导入对话快照** — 对话框 UI 全在, 补后端命令或删除入口 | 低 | 消除永远失败的 UI |
| **幽灵依赖 + 权限清理** (plugin-os, test.json, devtools 权限) | 极低 | 安全 + 减包 |

### 🟡 P1 — 中等工程量 (1-2 周)
| 功能 | 说明 |
|------|------|
| **MCP OAuth 2.1 流程** | PKCE 全实现, 只缺浏览器 + 回调胶水 |
| **DSTU create 模式** | 7 个编辑器壳, 逐个接 VFS 后端 |
| **Find & Replace** | 待 Crepe 支持, 或浏览器原生方案 |
| **试卷导出 PDF/Word** | 生成已工作, 补导出 |
| **轻量翻译 Popover** | backlog 项 + 当前分支已有 chat_popover 雏形 |
| **v1.2 性能 Phases 8-9** | 修 17 TS 错误 + 装 visualizer 出基线, 见效快 |
| **Research 模块: implement-or-delete 决策** | Option B (subagent_call 增强 research-mode) 比写 26 个命令现实得多 |
| **幽灵命令清理** | 40 个命令逐个实现 or 删注册 |

### 🔴 P2 — 大工程 (需要专门立项)
| 功能 | 说明 |
|------|------|
| **多模态索引恢复** | VL 搜索, flag 一致性先修好, 工具恢复 |
| **后端 MCP 模式** | 决策: 清理 or 复活 |
| **云同步架构增强** | CRDT + 变更流, 最大工程 |
| **CSS Tailwind v4 迁移** | 已批准, 需要专门 branch |
| **记忆 L1/L3 层** | 会话上下文 + 事件日志 |
| **HPIAS 完整实现** | 26 命令 + researchStore 接线 |
| **Master Refactoring 结构阶段** | 8 个 god-file 拆分 + 依赖升级 |

---

## 九、相关背景备注

- 分支 `release-experimental-0.9.39` 存在, 历史上做过实验性发布渠道
- `.study-ui/` 是独立的设计实验项目 (gitignored), 其 docs/plans 里的 UI 设计 (侧边栏 SOTA 重构等) 是否移植到主应用**未验证**
- `.planning/ROADMAP.md` 引用的 `milestones/` 目录不存在 (文档损坏项)
- 当前分支 `pr/3-pdf-reading` 已修改 `translation/chat_popover.rs` — 与第五节 2 的 Popover backlog 高度相关, 可能已在推进
