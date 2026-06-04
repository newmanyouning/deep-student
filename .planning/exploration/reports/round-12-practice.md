# Round 12: 题目集与练习 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 核心发现: `features/practice/` 是空壳

```
src/features/practice/
├── components/.gitkeep   ← 空
├── hooks/.gitkeep        ← 空
├── stores/.gitkeep       ← 空
├── styles/.gitkeep       ← 空
└── index.ts              ← export {}; (仅一行)
```

**所有实际代码散落在旧的 `components/` 目录中**，功能模块迁移计划从未执行。

---

## 实际代码位置

### 题目管理 (components/)

| 文件 | 行数 | 评级 |
|------|------|------|
| `QuestionBankEditor.tsx` | **2496** | 🔴 God Component (#4) |
| `QuestionBankListView.tsx` | 749 | 🟡 |
| `QuestionBankManageView.tsx` | 588 | 🟡 |
| `QuestionBankStatsView.tsx` | — | ✅ |
| `QuestionBankExportDialog.tsx` | — | ✅ |
| `QuestionHistoryView.tsx` | — | ✅ |
| `QuestionFavoritesView.tsx` | — | ✅ |
| `QuestionInlineEditor.tsx` | — | ✅ |
| `VirtualQuestionList.tsx` | — | ✅ |
| `CsvImportDialog.tsx` | — | ✅ |
| `CsvFieldMapper.tsx` | — | ✅ |

### 练习模式 (components/practice/)

| 文件 | 行数 |
|------|------|
| `MockExamMode.tsx` | 523 |
| `PaperGenerator.tsx` | 427 |
| `DailyPracticeMode.tsx` | 423 |
| `PracticeLauncher.tsx` | 369 |
| `TimedPracticeMode.tsx` | 343 |
| `PracticeModeSelector.tsx` | 174 |

### 复习系统 (components/)

| 文件 | 行数 |
|------|------|
| `ReviewSession.tsx` | 547 |
| `ReviewPlanView.tsx` | 521 |
| `ReviewQuestionsView.tsx` | — |
| `ReviewCalendarView.tsx` | — |

### Store (stores/)

| 文件 | 行数 | 已在 R04 分析 |
|------|------|-------------|
| `questionBankStore.ts` | 1630 | 🔴 God Store |
| `reviewPlanStore.ts` | ~100 | ✅ |

---

## 发现的问题

- [ ] **P1** — `features/practice/` 是**彻底的死目录** — 5 个 .gitkeep + 空 index.ts。功能模块迁移计划被放弃，所有代码仍在旧的 `components/` 目录
- [ ] **P1** — `QuestionBankEditor.tsx` **2496 行** — 全项目 #4 大组件，覆盖编辑器/预览/历史/收藏/导出等所有功能
- [ ] **P2** — 题目/练习/复习的代码分散在 3 个位置: `components/Question*.tsx` + `components/practice/` + `components/Review*.tsx`，没有统一的 feature 目录
- [ ] **P2** — Store 在 `stores/questionBankStore.ts` (全局)，但按 `features/` 架构应该在 `features/practice/stores/`
- [ ] **P2** — `features/practice/` 的 .gitkeep 文件暗示这是有意规划的迁移，但从未开始执行

---

## 建议优先处理

1. 删除 `features/practice/` 空壳目录，或执行迁移将其填充
2. 拆分 `QuestionBankEditor.tsx` (2496行) — 将编辑器/预览/历史面板提取为独立组件
3. 建立练习功能的统一目录结构 (features/practice/ + features/practice/stores/)
