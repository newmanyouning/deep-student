# Round 04: 状态管理 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## Store 清单

| Store | 文件 | 行数 | 职责 | 持久化 | 中间件 |
|-------|------|------|------|--------|--------|
| `useQuestionBankStore` | `questionBankStore.ts` | **1630** | 题库：题目CRUD、答题、统计、CSV、练习、同步 | ❌ | devtools + subscribeWithSelector |
| `useHpiasStore` | `researchStore.ts` | **506** | 深度调研：多轮执行、子代理、事件处理 | ❌ | devtools + subscribeWithSelector |
| `useAnkiUIStore` | `anki/useAnkiUIStore.ts` | **486** | Anki制卡UI：文档、模板、卡片、AnkiConnect | ❌ | subscribeWithSelector |
| `useAnkiQueueStore` | `ankiQueueStore.ts` | **249** | Anki制卡队列：材料入队/出队/持久化 | ✅ (自实现) | — |
| `useUnifiedIndexStore` | `unifiedIndexStore.ts` | **170** | VFS统一索引状态总览 | ❌ | — |
| `useSettingsShellStore` | `settingsShellStore.ts` | **87** | 设置页Tab路由 | ❌ | — |
| `useSystemStatusStore` | `systemStatusStore.ts` | **60** | 迁移状态 + 维护模式 | ❌ | — |
| `useUIStore` | `uiStore.ts` | **23** | 左侧面板折叠 | ✅ (zustand persist) | persist |
| `useViewStore` | `viewStore.ts` | **28** | 当前视图 + 前一视图 | ❌ | — |
| `useNetworkStore` | `networkStore.ts` | **22** | 在线/离线状态 | ❌ | — |
| `useTemplateAIStore` | `templateAiStore.ts` | ~150 | 模板AI生成对话 | ❌ | — |
| `useReviewPlanStore` | `reviewPlanStore.ts` | ~100 | 复习计划 (SM-2算法) | ❌ | devtools + subscribeWithSelector |
| `useFinderStore` | `features/learning-hub/stores/` | — | 学习资源导航 (Round 09详查) | — | — |
| `ResourceStateManager` | `store/ResourceStateManager.ts` | ~80 | **遗留**: 发布-订阅资源状态同步 | ❌ | 非Zustand |

### store/ vs stores/ 

- `src/store/ResourceStateManager.ts` — 唯一的 `store/` 目录文件，使用自定义 Pub/Sub 模式而非 Zustand
- `src/stores/` — 所有 Zustand stores 的标准目录
- `store/` 是旧约定，`stores/` 是新约定，两者并存

---

## Store 间依赖关系

```
App.tsx
├── useUIStore (侧边栏折叠)
├── useSystemStatusStore (迁移/维护)
├── useViewStore (当前视图)
└── useFinderStore (学习资源导航)

功能模块
├── Practice → useQuestionBankStore
├── Research → useHpiasStore
├── Anki → useAnkiUIStore + useAnkiQueueStore
├── Settings → useSettingsShellStore
└── Network → useNetworkStore
```

**无循环依赖**，但各 store 之间是平的，没有层级架构。

---

## 关键发现

### 1. questionBankStore — God Store (1630 行)

这是项目中最复杂的 store，混合了：
- 题目 CRUD（loadQuestions, updateQuestion, deleteQuestion）
- 答题评分（submitAnswer, toggleFavorite）
- FTS5 全文搜索（searchQuestions, rebuildFtsIndex）
- CSV 导入导出（getCsvPreview, importCsv, exportCsv）
- 练习模式（startTimedPractice, generateMockExam, getDailyPractice, generatePaper, getCheckInCalendar）
- 数据同步（checkSyncStatus, getSyncConflicts, resolveSyncConflict）
- 时间统计（loadLearningTrend, loadActivityHeatmap, loadKnowledgeStats）
- 状态管理（60+ state fields, 50+ actions）

**问题**: 应该拆分为多个独立 store（questionsStore, practiceStore, syncStore, statsStore）。

### 2. HpiasStore — 事件驱动 + 性能优化

独特的架构：
- 40+ 种事件类型（`HpiasEvent` union type）
- 模块级 eventsLog（非 React state）避免高频事件触发渲染
- 2000 条滑动窗口 + 100 条截断
- 支持 `hydrateRoundFromVisualSummary`（从后端单条摘要恢复多轮状态）

**问题**: `HpiasEvent` union 过于庞大（40+ variants），event handler 的 switch 语句超长。

### 3. useAnkiUIStore — 切片模式（最佳实践）

唯一使用结构化切片（Slice Pattern）的 store：
- DocumentSlice, TemplateSlice, CardsSlice, AnkiConnectSlice, ImportSlice, UISlice, OptionsSlice
- 7 个切片，每个有独立的 State + Actions 接口
- 提供细粒度 selector hooks（`useDocumentState`, `useCardsState` 等）

**值得推广的模式**。

### 4. 持久化策略不一致

| Store | 方式 |
|-------|------|
| `useUIStore` | Zustand `persist` middleware (localStorage) |
| `useAnkiQueueStore` | 自实现：TauriAPI.saveSetting + localStorage 回退 + 版本号去重 |
| 其他 stores | 不持久化 |

### 5. 废弃类型链追踪

| 层级 | 文件 | 内容 |
|------|------|------|
| 根定义 | `types/index.ts` | `MistakeItem` (标记 @deprecated) |
| ~~重导出~~ | ~~`types/api.ts`~~ | ✅ 已删除 (2026-05-30, REF-001) |
| ~~旧 Store 目录~~ | ~~`store/ResourceStateManager.ts`~~ | ✅ 已删除 (2026-05-30, REF-006) |
| Store 引用 | `stores/anki/types.ts` | `import { MistakeItem }` → 定义 `MistakeSummary = MistakeItem` |
| Store 使用 | `stores/anki/useAnkiUIStore.ts` | `ImportSlice` 使用 `MistakeSummary[]` |

MistakeItem 的废弃影响链（已从 4 层减至 3 层）：types → anki store types → anki store → UI。

### 6. API 调用模式

大部分 store 直接通过 `invoke('command_name', ...)` 调用后端，而不是通过 `src/api/` 层。这违反了前后端分离原则。

**例外**: `useUnifiedIndexStore` 使用 `vfsUnifiedIndexApi` 封装层。

---

## 发现的问题

- [ ] **P1** — `questionBankStore` (1630 行) 是 God Store，混合了 7 种不同的职责域。应拆分为 questionsStore、practiceStore、syncStore、statsStore
- [ ] **P1** — Store 直接调用 `invoke()` 绕过 API 层，后端命令名称硬编码在前端 store 中
- [ ] **P2** — `store/` 和 `stores/` 目录并存，`ResourceStateManager` 使用自定义 Pub/Sub 而非 Zustand
- [ ] **P2** — 持久化策略不统一：Zustand persist vs 自实现 vs 不持久化
- [ ] **P2** — `MistakeItem` 废弃链条已确认：从 types → anki types → anki store → UI 组件（4 层依赖）
- [ ] **P2** — `HpiasEvent` union 有 40+ variants，event handler 的 switch 包含 40+ case 分支
- [ ] **P3** — `useAnkiUIStore` 使用的切片模式值得推广到其他大型 store
- [ ] **P3** — `useSettingsShellStore` 中硬编码了 settings tab 映射表 (18 个条目)

---

## 建议优先处理

1. 拆分 `questionBankStore` — 优先级最高（1630 行，对可维护性影响最大）
2. 将 `ResourceStateManager` 迁移到 Zustand 或移入 `stores/`
3. 统一持久化策略 — 选择 Zustand persist 或统一的自实现方案
4. 建立 "Store → API 层 → invoke" 的调用规范（当前只有 1 个 store 遵循此模式）
5. 清理 MistakeItem 废弃链 — 从 Anki ImportSlice 开始移除依赖
