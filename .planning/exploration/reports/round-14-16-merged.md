# Round 14-16: PDF / 翻译 / 作文 / 其他小模块 — 合并诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成 (R14+R15+R16 合并)

---

## R14: PDF/DOCX 阅读 — 6 文件, ~2,900 行

```
src/features/pdf/
├── components/
│   ├── EnhancedPdfViewer.tsx    1783 行 🔴
│   ├── PdfReader.tsx            339 行
│   └── TextbookPdfViewer.tsx    319 行
├── stores/
│   ├── pdfProcessingStore.ts    294 行
│   └── pdfSettingsStore.ts       —
├── styles/ (3 CSS)
└── index.ts
```

| 问题 | 说明 |
|------|------|
| **P2** | `EnhancedPdfViewer.tsx` **1783 行** — 增强型 PDF 查看器过大，应拆分工具栏/侧栏/页面渲染 |
| ✅ | 模块结构清晰: components/stores/styles/index 四层分离 |
| ✅ | PDF 处理状态有自己的 store (pdfProcessingStore) |

---

## R15: 翻译工作台 — 9 文件, ~2,300 行

```
src/translation/
├── TranslationStreamRenderer.tsx
└── useTranslationStream.ts      352 行

src/components/translation/
├── TranslationMain.tsx          602 行
├── PromptPanel.tsx              412 行
├── ComparisonView.tsx           —
├── SourcePanel.tsx              —
├── TargetPanel.tsx              260 行
├── TranslationHeader.tsx        —
└── TranslationHistory.tsx       —
```

| 问题 | 说明 |
|------|------|
| **P2** | 代码分散在 `translation/` (2文件) 和 `components/translation/` (7文件) 两个位置 |
| **P3** | `TranslationMain.tsx` 602 行，可拆分 |

---

## R16a: 作文批改 — 13 文件, ~3,700 行

```
src/essay-grading/ (2 文件) + src/components/essay-grading/ (11 文件)
├── SettingsDrawer.tsx           752 行 (最大)
├── GradingMain.tsx              605 行
├── InputPanel.tsx               591 行
├── ResultPanel.tsx               —
├── StreamingAnnotatedText.tsx   280 行
├── ScoreCard.tsx / PolishSectionView.tsx / SentenceDetailView.tsx / ...
```

| 问题 | 说明 |
|------|------|
| **P2** | 代码分散在 essay-grading/ (2) + components/essay-grading/ (11) |
| **P3** | 无 features/essay-grading/ 目录 — 未开始模块化迁移 |

---

## R16b: 番茄钟 — 9 文件

```
src/features/pomodoro/  ✅ 完整的 feature 目录结构
├── components/ (3 组件: Widget, FocusMode, Panel)
├── stores/usePomodoroStore.ts
├── api.ts / types.ts / index.ts
```

✅ 目录结构完整，与其他空壳 feature 不同。

## R16c: 待办事项 — 11 文件

```
src/features/todo/  ✅ 完整的 feature 目录结构
├── components/ (5 组件: Page, Sidebar, MainPanel, ContentView, ShellSidebar)
├── stores/useTodoStore.ts
├── api.ts / types.ts / index.ts
```

✅ 目录结构完整。注意有 TodoSidebar 和 TodoShellSidebar — 需确认是否重复。

## R16d: 沙箱工作台 — 8 文件

```
src/features/sandbox/
├── store/useSandboxWorkbenchStore.ts
├── pages/SandboxWorkbenchPage.tsx
├── components/ (4 组件: Toolbar, StatusRail, Inspector, Surface)
├── launchSandboxWorkbench.ts / types.ts
```

✅ 结构清晰，小而专注。

## R16e: 语音输入 — 20 文件

```
src/voice-input/ (10 文件 — 核心实现)
├── audio.ts / controller.ts / hooks.ts
├── config.ts / runtimeConfig.ts / modelSelection.ts
├── history.ts / providerRegistry.ts / support.ts
└── index.ts

src/features/voice-input/index.ts (仅 re-export)
```

✅ 模块设计良好：providerRegistry 暗示可扩展的语音供应商架构。

---

## 汇总发现

### 空壳 feature 目录统计 (截至目前)

| 目录 | 状态 |
|------|------|
| `features/practice/` | 🔴 空壳 (.gitkeep) |
| `features/template-management/` | 🔴 空壳 (.gitkeep) |
| `features/pdf/` | ✅ 完整 |
| `features/pomodoro/` | ✅ 完整 |
| `features/todo/` | ✅ 完整 |
| `features/sandbox/` | ✅ 完整 |

### 模块代码分散统计

| 功能 | 代码位置 | 分散度 |
|------|---------|--------|
| 翻译 | `translation/` + `components/translation/` | 2 处 |
| 作文 | `essay-grading/` + `components/essay-grading/` | 2 处 |
| 语音 | `voice-input/` + `features/voice-input/` | 2 处 (re-export 模式，可接受) |

### 问题汇总

- [ ] **P2** — `EnhancedPdfViewer.tsx` 1783 行，应拆分
- [ ] **P2** — 翻译和作文代码分两处存放，应统一到 features/ 目录
- [ ] **P3** — `TranslationMain.tsx` 602 行，SettingsDrawer 752 行，可拆分
- [ ] **P4** — `features/pomodoro/components/` 仍有 .gitkeep（表示原本空目录，后来才添加组件）
