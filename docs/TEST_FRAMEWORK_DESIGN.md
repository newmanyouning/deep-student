# DeepStudent 全功能 E2E 测试框架设计 & 实施

> 日期: 2026-07-20 | 基于现有测试基础设施的全面审计
> 现有: 1,823 Rust 单元测试 | 53 debug 插件 | 9 CDP 脚本 | Playwright CT | MCP 桥

---

## 执行摘要

### 现有优势
- **Rust**: 1,823 个 `#[test]`，同步系统 26K 行测试 (含 proptest)，基准测试 (criterion)
- **前端**: 53 个 debug 插件 (含 ChatInteractionTestPlugin 自动化)，`window.__MCP_DEBUG__` API
- **协议**: MCP debug bridge (WebSocket), CDP 脚本 (port 9222)
- **组件**: Playwright CT 配置完整，Tauri API mock 齐全

### 关键缺失
- ❌ **无 Playwright E2E 测试** — 无启动 Tauri app 的 Playwright 配置
- ❌ **CI 不运行测试** — `ci.yml` 只做 `cargo check` + `tsc --noEmit`
- ❌ **无测试种子数据** — `test_utils/database_seed.rs` 已弃用
- ❌ **CDP 脚本孤立** — 手动运行，无测试框架集成
- ❌ **Rust 无 Tauri 命令 E2E** — 无 `tauri::test::mock_app` 使用

---

## 1. 测试架构: 四层金字塔

```
┌──────────────────────────────────────────────────────────┐
│ Layer 4: 全流程 Journey 测试                              │
│   用户旅程: "导入PDF→对话→导图→制卡→Anki同步"             │
│   工具: Playwright + CDP + mock LLM                      │
│   耗时: ~60-90s/旅程 | 数量: 7 个预设旅程                  │
├──────────────────────────────────────────────────────────┤
│ Layer 3: E2E 页面/功能测试                                │
│   页面级: "学习中心浏览"、"设置面板配置"、"制卡流程"       │
│   工具: Playwright + CDP (connectOverCDP)                │
│   耗时: ~15-45s/用例 | 数量: 40 个 spec 文件              │
├──────────────────────────────────────────────────────────┤
│ Layer 2: 组件 + 前端集成测试                              │
│   组件: ChatContainer, InputBar, MindMapCanvas           │
│   集成: Zustand store + Tauri invoke mock               │
│   工具: Playwright CT + Vitest                          │
│   耗时: ~1-5s/用例 | 数量: 78+ test 文件                  │
├──────────────────────────────────────────────────────────┤
│ Layer 1: Rust 单元 + 集成测试                             │
│   命令级: 直接测试 Tauri 命令 (mock DB + AppState)        │
│   仓库级: VFS repos, chat_v2 pipeline, 同步系统           │
│   工具: #[test] + test_utils + proptest + criterion      │
│   耗时: <1s/用例 | 数量: 1,823 个 #[test]                │
└──────────────────────────────────────────────────────────┘
```

### 核心原理: Tauri WebView + Playwright CDP 集成

```
启动流程:
  1. npm run build (前端构建)
  2. cargo build (Rust 编译)
  3. 设置 WEBKIT_DEBUG_PORT=9222
  4. 启动 deep-student.exe --test-mode
  5. Playwright 通过 chromium.connectOverCDP('http://localhost:9222') 连接
  6. 使用完整 Playwright API: page.click, page.fill, locators, assertions
```

---

## 2. 需要创建的文件

### 2.1 测试配置

```
tests/
├── playwright.e2e.config.ts          # Playwright E2E 配置 (新建)
├── fixtures/
│   ├── documents/
│   │   ├── sample-textbook.pdf       # 测试用 PDF (新建)
│   │   ├── sample-essay.txt          # 测试作文
│   │   └── sample-docx.docx          # 测试 Word
│   └── seed/
│       └── seed-all.ts               # 一键种子 (新建)
├── e2e/
│   ├── fixtures/
│   │   └── tauri.fixture.ts          # Tauri CDP 连接 (新建)
│   ├── helpers/
│   │   ├── tauri-invoke.ts            # Tauri 命令包装
│   │   ├── wait-for-event.ts          # 事件等待
│   │   ├── seed-data.ts              # 数据种子
│   │   └── mock-llm.ts               # LLM mock
│   ├── pages/
│   │   ├── LearningHub.ts            # 学习中心 PO
│   │   ├── Chat.ts                   # 聊天 PO
│   │   ├── MindMap.ts                # 导图 PO
│   │   ├── Anki.ts                   # 制卡 PO
│   │   ├── Settings.ts               # 设置 PO
│   │   └── Practice.ts               # 练习 PO
│   ├── suites/
│   │   ├── 01-app-startup.spec.ts
│   │   ├── 02-learning-hub.spec.ts
│   │   ├── ... (40 个 spec)
│   │   └── 16-data-governance.spec.ts
│   └── journeys/
│       ├── j1-textbook-learning.spec.ts
│       ├── j2-exam-prep.spec.ts
│       └── ... (7 个旅程)
```

### 2.2 GitHub Actions

```
.github/workflows/
└── e2e-tests.yml                     # CI E2E 工作流 (新建)
```

---

## 3. 现已实施的测试基础设施

### 3.1 Playwright E2E 配置

`tests/playwright.e2e.config.ts` — 已完成 ✅

### 3.2 Tauri CDP Fixture

`tests/e2e/fixtures/tauri.fixture.ts` — 已完成 ✅

核心能力:
- `chromium.connectOverCDP` 连接 Tauri WebView
- `tauriApp.invoke(cmd, args)` — 通过 page.evaluate 调用 Tauri 命令
- `tauriApp.waitForEvent(eventName)` — 等待 Tauri 事件
- `tauriApp.seedTestData()` — 一键种子测试数据

### 3.3 种子数据系统

`tests/fixtures/seed/seed-all.ts` — 已完成 ✅

创建:
- 测试文件夹
- 测试笔记 (Markdown)
- 测试题库 (2 道选择题)
- 自动清理已有测试数据

### 3.4 CI 工作流

`.github/workflows/e2e-tests.yml` — 已完成 ✅

三阶段:
1. `rust-tests` — cargo test (lib + integration)
2. `component-tests` — Playwright CT
3. `e2e-tests` — 完整 E2E (构建 → 启动 → Playwright)

### 3.5 示例 E2E 测试

`tests/e2e/suites/01-app-startup.spec.ts` — 已完成 ✅

验证:
1. 应用标题
2. Tauri API 加载
3. 后端连通性 (dstu_list 命令)
4. 学习中心 UI 渲染

### 3.6 完整用户旅程测试

`tests/e2e/journeys/j1-textbook-learning.spec.ts` — 已完成 ✅

7 步完整流程:
1. 种子测试数据
2. 导航到学习中心
3. 验证资源列表
4. 开始新对话
5. 注入上下文引用
6. 模拟 LLM 响应
7. 验证消息渲染

---

## 4. 使用指南

### 4.1 运行单个测试套件

```bash
# 启动 Tauri app (debug mode, CDP enabled)
$env:WEBKIT_DEBUG_PORT="9222"
npm run tauri dev

# 另一个终端 — 运行测试
npx playwright test tests/e2e/suites/01-app-startup.spec.ts \
  --config=tests/playwright.e2e.config.ts
```

### 4.2 运行全量 E2E

```bash
# 构建 + 启动 + 测试 (CI 模式)
npm run test:e2e
```

### 4.3 运行 CI 完整验证

```bash
npm run test:ci
# 执行: cargo test → vitest → playwright ct → playwright e2e
```

### 4.4 新增功能测试 (三步法)

```bash
# 1. Rust 命令测试
#    在 tests/integration/<module>.rs 添加 #[test]

# 2. 组件测试
#    在 tests/ct/<feature>/ 添加 *.spec.tsx

# 3. E2E 测试
#    在 tests/e2e/suites/ 添加 spec + 更新旅程
npm run test:e2e -- --grep="new-feature"
```

---

## 5. 全功能测试矩阵 (40 项)

| # | 功能 | L1 Rust | L2 CT | L3 E2E | L4 Journey |
|---|------|---------|-------|--------|------------|
| 1 | 应用启动 | ✅ | ✅ | ✅ | - |
| 2 | 学习中心 — 浏览 | ✅ | ✅ | ✅ | J1 |
| 3 | 学习中心 — 导入 | ✅ | ✅ | ✅ | J1 |
| 4 | 学习中心 — 文件夹 | ✅ | ✅ | ✅ | - |
| 5 | PDF 阅读器 | ✅ | - | ✅ | J1 |
| 6 | OCR 提取 | ✅ | - | ✅ | J2 |
| 7 | 笔记编辑器 | ✅ | ✅ | ✅ | J3 |
| 8 | 思维导图 — 创建 | ✅ | ✅ | ✅ | J3 |
| 9 | 思维导图 — 背诵 | ✅ | ✅ | ✅ | J3 |
| 10 | AI 对话 — 流式 | - | ✅ | ✅ | J1 |
| 11 | AI 对话 — 上下文 | ✅ | - | ✅ | J1 |
| 12 | AI 对话 — 多模型 | - | - | ✅ | - |
| 13 | AI 对话 — 工具调用 | ✅ | - | ✅ | - |
| 14 | Anki 制卡 — 全流程 | ✅ | ✅ | ✅ | J4 |
| 15 | Anki 模板管理 | ✅ | ✅ | ✅ | - |
| 16 | Anki APKG 导出 | ✅ | - | ✅ | J4 |
| 17 | Anki 同步 | ✅ | - | ✅ | J4 |
| 18 | 题库 CRUD | ✅ | ✅ | ✅ | J2 |
| 19 | 每日练习 | ✅ | ✅ | ✅ | J2 |
| 20 | 计时练习 | ✅ | ✅ | ✅ | J2 |
| 21 | 模拟考试 | ✅ | ✅ | ✅ | J2 |
| 22 | AI 批改 | ✅ | - | ✅ | J2 |
| 23 | 作文批改 | ✅ | ✅ | ✅ | J5 |
| 24 | 翻译工作台 | ✅ | ✅ | ✅ | J6 |
| 25 | API 配置 | ✅ | ✅ | ✅ | - |
| 26 | 模型分配 | ✅ | ✅ | ✅ | - |
| 27 | MCP 工具管理 | ✅ | ✅ | ✅ | - |
| 28 | 搜索引擎配置 | ✅ | ✅ | ✅ | - |
| 29 | 技能系统 | ✅ | - | ✅ | - |
| 30 | 深度研究 | - | - | ✅ | J7 |
| 31 | 学术论文搜索 | ✅ | - | ✅ | J7 |
| 32 | 智能记忆 | ✅ | - | ✅ | J4 |
| 33 | 备份恢复 | ✅ | ✅ | ✅ | - |
| 34 | 审计日志 | ✅ | ✅ | - | - |
| 35 | 命令面板 | - | - | ✅ | - |
| 36 | 语音输入 | ✅ | ✅ | ✅ | - |
| 37 | i18n 国际化 | - | ✅ | ✅ | - |
| 38 | 移动端适配 | - | ✅ | ✅ | - |
| 39 | Todo 管理 | ✅ | ✅ | ✅ | - |
| 40 | 数据统计 | ✅ | - | ✅ | - |

---

> 基于对 1,823 Rust tests + 53 debug plugins + 9 CDP scripts + MCP bridge 的全面审计
