# DeepStudent 项目分层探究计划

**创建日期**: 2026-05-29
**目的**: 在不过载上下文的前提下，分批次、分模块系统诊断项目现状，整理出一份准确的项目档案，替代可能的过时文档。

## 分层策略

采用 **5 层渐进式** 扫描，每层按模块拆分，单次扫描文件数控制在 15-30 个以内：

| 层 | 名称 | 说明 | 预计轮次 |
|----|------|------|----------|
| 0 | 文档汇总 | 汇总已有文档，产出已知事实清单 | 1 轮（已完成） |
| 1 | 项目骨架 | 配置、入口、根结构 | 2 轮 |
| 2 | 核心基础设施 | 共享模块、状态管理、API层、路由 | 5 轮 |
| 3 | 功能模块（前端） | 逐个 feature 扫描 | 12 轮 |
| 4 | 后端模块（Rust） | 逐个 backend crate 扫描 | 10 轮 |
| 5 | 横切关注点 | 测试、构建、CI、i18n、样式 | 4 轮 |

**总计约 34 轮，每轮扫描 10-25 个文件。**

---

## 执行约定

1. **每轮独立**: 每一轮产出一份 `round-XX-模块名.md` 诊断报告
2. **先读 README**: 如模块有 README.md，优先阅读
3. **记录关键信息**: 模块职责、核心文件、对外接口、已知问题
4. **发现即记录**: 发现代码规范不一致、过时注释、TODO/FIXME 即记录
5. **不修改代码**: 探究阶段只读不写

---

## 层 0：文档汇总 ✅

### 0.1 已读文档清单

| 文档 | 状态 | 关键信息 |
|------|------|----------|
| README.md / README_CN.md | ✅ | 项目定位、功能列表、架构概览、技术栈 |
| package.json | ✅ | v0.9.40, React 18 + Vite 6 + Tauri 2 |
| CODE_STYLE.md | ✅ | 命名规范、i18n要求、组件规范、Rust规范 |
| ROADMAP.md | ✅ | v1.1 已发布, v1.2 进行中 (性能/代码健康) |
| BUILD-CONFIG.md | ✅ | 跨平台构建配置 (Win/Mac/Linux/Android/iOS) |
| cloud-sync-compatibility-analysis-2026-05-23.md | ✅ | 云同步能力现状及风险点 |
| css-architecture-migration-design.md | ✅ | CSS 架构迁移计划 (App.css 12K行 -> Tailwind v4) |

### 0.2 从文档已获知的问题

- **CSS 架构混乱**: App.css 12K行 + DeepStudent.css 3K行 + 60+ scattered CSS files，迁移计划已批准但未完成
- **云同步不完整**: 13 张表的行级增量同步，大量表未覆盖
- **TS 错误**: 路线图中提到 "17 pre-existing TS errors" 待修复 (Phase 8)
- **代码规范不统一**: 存在两个 cn() 实现 (`@/utils/cn` 推荐 vs `@/lib/utils` 历史遗留)
- **文档可能过时**: 多次 AI 开发，缺乏统一标准

---

## 层 1：项目骨架

### 1.1 根配置与入口文件 [round-01-root-config.md]
- `vite.config.ts` — 构建配置
- `tsconfig.json` / `tsconfig.node.json` — TS 配置
- `tailwind.config.js` — Tailwind 配置
- `eslint.config.js` — ESLint 规则
- `postcss.config.js` — PostCSS
- `index.html` — 入口 HTML
- `.env.example` — 环境变量模板
- `.release-channel` / `release-please-config.json` — 发布配置

### 1.2 前端入口与路由 [round-02-app-entry.md]
- `src/main.tsx` — React 挂载点
- `src/App.tsx` — 根组件
- `src/app/` — App shell 全部文件
- `src/config/` — 前端配置文件
- `src/polyfills/` — Polyfills
- `src/shims/` — 类型垫片

---

## 层 2：核心基础设施

### 2.1 类型系统与共享层 [round-03-types-shared.md]
- `src/types/` — 全局类型定义
- `src/shared/` — 共享模块
- `src/lib/` — 基础库
- `src/utils/` — 工具函数
- `src/contexts/` — React Contexts

### 2.2 状态管理 [round-04-stores.md]
- `src/stores/` — Zustand stores 全部
- `src/store/` — 旧版 store（若存在）

### 2.3 API 层与服务层 [round-05-api-services.md]
- `src/api/` — Tauri invoke 封装
- `src/services/` — 前端服务层
- `src/events/` — 事件系统

### 2.4 通用 UI 组件 [round-06-ui-components.md]
- `src/components/ui/` — 基础 UI 组件
- `src/components/shared/` — 共享业务组件
- `src/components/layout/` — 布局组件
- `src/components/icons/` — 图标组件

### 2.5 Hooks 与引擎 [round-07-hooks-engines.md]
- `src/hooks/` — 全部 React Hooks
- `src/engines/` — 渲染引擎 (Markdown/代码高亮等)
- `src/data/` — 前端数据层

---

## 层 3：功能模块（前端）

### 3.1 Chat V2 对话引擎 [round-08-chat-v2.md]
路径: `src/features/chat/`
关注: 对话架构、适配器、技能加载、消息渲染

### 3.2 学习资源中心 [round-09-learning-hub.md]
路径: `src/features/learning-hub/`
关注: 资源管理、文件导入、向量化状态

### 3.3 笔记系统 [round-10-notes.md]
路径: `src/features/notes/` + `src/components/crepe/`
关注: Milkdown 编辑器集成、富文本编辑

### 3.4 知识导图 [round-11-mindmap.md]
路径: `src/features/mindmap/`
关注: React Flow 集成、导图/大纲切换

### 3.5 题目集与练习 [round-12-practice.md]
路径: `src/features/practice/` + `src/components/practice/`
关注: 题库、每日练习、模拟考试、AI评分

### 3.6 Anki 闪卡与模板 [round-13-anki-template.md]
路径: `src/components/anki/` + `src/features/template-management/`
关注: Mustache 模板引擎、3D 预览、制卡任务

### 3.7 PDF/DOCX 阅读 [round-14-pdf-docx.md]
路径: `src/features/pdf/` + `src/components/previews/`
关注: PDF.js、docx-preview、分屏阅读

### 3.8 翻译工作台 [round-15-translation.md]
路径: `src/features/*translation*/` + `src/translation/`
关注: 翻译 Pipeline、领域预设

### 3.9 作文批改与其他功能 [round-16-essay-others.md]
路径: `src/essay-grading/` + `src/features/pomodoro/` + `src/features/todo/` + `src/features/sandbox/` + `src/voice-input/`
关注: 作文评分、番茄钟、待办、沙箱、语音输入

### 3.10 系统功能（设置/命令面板/技能管理/调试面板）[round-17-system-features.md]
路径: `src/features/settings/` + `src/command-palette/` + `src/features/skills-management/` + `src/debug-panel/`
关注: 设置页、命令面板、技能管理UI、调试工具

### 3.11 MCP 客户端 [round-18-mcp-client.md]
路径: `src/mcp/` + `src/mcp-debug/`
关注: MCP 协议客户端实现、调试面板

### 3.12 DSTU 资源协议 [round-19-dstu.md]
路径: `src/dstu/`
关注: 统一资源协议、VFS API 前端封装

---

## 层 4：后端模块（Rust）

### 4.1 后端入口与命令路由 [round-20-backend-entry.md]
路径: `src-tauri/src/main.rs` / `lib.rs` + `src-tauri/src/cmd/` + `src-tauri/src/data/`
关注: Tauri 命令注册、应用初始化

### 4.2 数据库与迁移 [round-21-database.md]
路径: `src-tauri/src/database/` + `src-tauri/migrations/`
关注: SQLite schema、迁移历史、数据表清单

### 4.3 Chat V2 Pipeline [round-22-chat-v2-backend.md]
路径: `src-tauri/src/chat_v2/`
关注: 对话 Pipeline、工具执行器、workspace 管理

### 4.4 LLM 管理与适配 [round-23-llm-manager.md]
路径: `src-tauri/src/llm_manager/` + `src-tauri/src/llm_usage/` + `src-tauri/src/adapters/` + `src-tauri/src/providers/`
关注: 9家模型供应商适配、Token追踪

### 4.5 VFS 虚拟文件系统 [round-24-vfs.md]
路径: `src-tauri/src/vfs/`
关注: 文件存储、向量化索引、Blob管理

### 4.6 搜索引擎与工具 [round-25-tools-search.md]
路径: `src-tauri/src/tools/` + `src-tauri/src/services/`
关注: 7种搜索引擎适配器

### 4.7 智能记忆 [round-26-memory.md]
路径: `src-tauri/src/memory/`
关注: 三层架构、LLM决策、向量比对

### 4.8 数据治理与云同步 [round-27-data-governance.md]
路径: `src-tauri/src/data_governance/` + `src-tauri/src/cloud_storage/`
关注: 备份恢复、增量同步、冲突解决、S3/WebDAV

### 4.9 翻译/作文/评分后端 [round-28-content-backend.md]
路径: `src-tauri/src/translation/` + `src-tauri/src/essay_grading/` + `src-tauri/src/qbank_grading/`
关注: 翻译 Pipeline、作文评分、题库评分

### 4.10 安全与基础设施 [round-29-security-infra.md]
路径: `src-tauri/src/crypto/` + `src-tauri/src/multimodal/` + `src-tauri/src/ocr_adapters/` + `src-tauri/src/mcp/` + `src-tauri/src/utils/` + `src-tauri/src/dstu/`
关注: AES加密、多模态处理、OCR适配、MCP后端、DSTU后端

---

## 层 5：横切关注点

### 5.1 测试体系 [round-30-tests.md]
路径: `tests/` 全部（vitest/ct/perf/security/visual）
关注: 测试覆盖范围、测试框架、CI 集成

### 5.2 构建与脚本 [round-31-build-scripts.md]
路径: `scripts/` 全部
关注: 构建脚本、i18n检查、模型注册表

### 5.3 CI/CD [round-32-cicd.md]
路径: `.github/workflows/`
关注: CI 流程、构建矩阵、Release 自动化

### 5.4 国际化与样式资产 [round-33-i18n-styles.md]
路径: `src/locales/` + `src/styles/` + `src/assets/` + `public/`
关注: i18n 结构、CSS 架构现状、静态资源

---

## 执行状态

| 轮次 | 模块 | 状态 | 产出文件 |
|------|------|------|----------|
| 0.1 | 文档汇总 | ✅ 完成 | README.md (本文档) |
| 1.1 | 根配置与入口 | ✅ 完成 | round-01-root-config.md |
| 1.2 | 前端入口与路由 | ✅ 完成 | round-02-app-entry.md |
| 2.1 | 类型系统与共享层 | ✅ 完成 | round-03-types-shared.md |
| 2.2 | 状态管理 (Stores) | ✅ 完成 | round-04-stores.md |
| 2.3 | API 层与服务层 | ✅ 完成 | round-05-api-services.md |
| 2.4 | 通用 UI 组件 | ✅ 完成 | round-06-ui-components.md |
| 2.5 | Hooks 与引擎 | ✅ 完成 | round-07-hooks-engines.md |
| 3.1 | Chat V2 对话引擎 | ✅ 完成 | round-08-chat-v2.md |
| 3.2 | 学习资源中心 | ✅ 完成 | round-09-learning-hub.md |
| 3.3 | 笔记系统 | ✅ 完成 | round-10-notes.md |
| 3.4 | 知识导图 | ✅ 完成 | round-11-mindmap.md |
| 3.5 | 题目集与练习 | ✅ 完成 | round-12-practice.md |
| 3.6 | Anki 闪卡与模板 | ✅ 完成 | round-13-anki-template.md |
| 3.7-3.9 | PDF+翻译+作文+番茄+待办+沙箱+语音 | ✅ 完成 | round-14-16-merged.md (合并) |
| 3.10a | 设置模块 | ✅ 完成 | round-17a-settings.md |
| 3.10b | 命令面板+技能管理+调试面板 | ✅ 完成 | round-17b-system-features.md |
| 3.11 | MCP 客户端 | ✅ 完成 | round-18-mcp-client.md |
| 3.12 | DSTU 资源协议 | ✅ 完成 | round-19-dstu.md |
| 4.1 | 后端入口+cmd+database | ✅ 完成 | round-20-backend-entry.md |
| 4.2 | Chat V2 Pipeline 后端 | ✅ 完成 | round-21-chat-v2-backend.md |
| 4.3 | LLM Manager | ✅ 完成 | round-22-llm-manager.md |
| 4.4 | VFS 虚拟文件系统 | ✅ 完成 | round-23-vfs.md |
| 4.5 | DSTU+Tools+Memory+MCP | ✅ 完成 | round-24-26-backend-merged.md |
| 4.6 | DataGov+Cloud+Essay+QBank+Trans | ✅ 完成 | round-24-26-backend-merged.md |
| 4.7 | Crypto+Multimodal+OCR+Usage+Infra | ✅ 完成 | round-24-26-backend-merged.md |
| 5.1-5.4 | 测试+构建+CI+i18n+样式 | ✅ 完成 | round-30-33-cross-cutting.md |
| 补充 | 遗漏模块 (75文件, ~60K行) | ✅ 完成 | round-supplement-complete.md |
| 根目录 | 根目录完整扫描 (所有非src文件夹/文件) | ✅ 完成 | round-root-full-scan.md |
| .study-ui | 独立实验项目扫描 (190文件) | ✅ 完成 | round-study-ui-scan.md |
| 单文件 | 65个遗漏Rust单文件逐文件扫描 | ✅ 完成 | round-single-file-scan.md |

---

## ✅ 全部探究完成

**日期**: 2026-05-29
**诊断报告**: **24 份**
**扫描覆盖**: ~1,305 文件
**累积问题**: 55+
**.gitignore 更新**: 排除 .kiro/ .roundtable/ docs/.deepseek/ .study-ui/docs/

## ✅ 全部探究计划 + 全部补充扫描执行完毕

**日期**: 2026-05-29
**诊断报告**: 23 份
**扫描覆盖**: ~1,240 文件 (前端~600 + 后端~450 + .study-ui~190)
**累积问题**: 50+ (详见 `reports/cumulative-issues.md`)
**.gitignore 更新**: 排除 .kiro/ .roundtable/ docs/.deepseek/ .study-ui/docs/
**实际执行**: 25+2 轮 → **22 份诊断报告**
**扫描覆盖**: ~1,050 源文件 + 根目录全部 30+ 个文件/文件夹
**累积问题**: 45+ (详见 `reports/cumulative-issues.md`)

*计划生成日期: 2026-05-29。全部完成。*
