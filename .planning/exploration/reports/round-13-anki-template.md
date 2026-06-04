# Round 13: Anki 闪卡与模板管理 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 模块规模

| 位置 | 文件数 | 总行数 | 说明 |
|------|--------|--------|------|
| `components/anki/` | 30 | ~7,800 | 实际代码所在地 |
| `features/template-management/` | **1** | 1 (export {}) | 🔴 空壳目录 |
| `components/RealTimeTemplateEditor/` | 3 | ~500 | 实时模板编辑器 |
| `data/anki/` | 1 | JSON | 内置模板数据 |

---

## 架构: CardForge 2.0 引擎

```
components/anki/
├── cardforge/           ← CardForge 2.0 核心引擎 (微插件架构)
│   ├── engines/         5 文件 — CardAgent/CardEngine/SegmentEngine/TaskController
│   ├── hooks/           useCardForge (499行)
│   ├── adapters/        chatV2Adapter — 与 Chat V2 的桥接
│   ├── prompts/         生成提示词
│   └── types/           类型定义
├── panels/              3 面板 — 文档上传/材料队列/导出
├── services/            streamEventHandler (358行)
├── hooks/               Anki 专用 Hooks
├── utils/               cardHelpers/formatters/exportNormalize
└── TaskDashboardPage.tsx  1356行 — 任务看板主页
```

### 关键文件

| 文件 | 行数 | 职责 |
|------|------|------|
| `CardAgent.ts` | **1575** | AI 卡片生成代理 — 最大引擎文件 |
| `TaskDashboardPage.tsx` | **1356** | 制卡任务看板 |
| `CardEngine.ts` | 838 | 卡片生成引擎核心 |
| `TaskController.ts` | 771 | 批量任务调度 |
| `SegmentEngine.ts` | 525 | 文档分段处理 |
| `useCardForge.ts` | 499 | CardForge React Hook |
| `RealTimeTemplateEditor/index.tsx` | 472 | Mustache 模板实时编辑器 |

---

## 发现的问题

- [ ] **P1** — `features/template-management/` 是第 **3 个**空壳目录（继 practice 之后），只有 .gitkeep + `export {}`
- [ ] **P1** — `CardAgent.ts` 1575 行 — AI 卡片生成的核心逻辑集中在一个文件中，应拆分
- [ ] **P2** — `TaskDashboardPage.tsx` 1356 行 — 任务看板过重
- [ ] **P2** — 数据分散在 `components/anki/`、`components/RealTimeTemplateEditor/`、`data/anki/`、`stores/anki/`（已在 R04 分析）四个位置
- [ ] **P3** — CardForge 引擎使用类模式（CardAgent, TaskController 等），与项目主流的 Zustand + Hook 模式不一致
- [ ] **P3** — `TaskController.examples.ts` (479行) 是示例代码，不应在生产目录中

### 正面评价

- CardForge 引擎有自己的适配器/引擎/Hook/类型分层，架构清晰
- 明确的 Chat V2 适配器 (`chatV2Adapter`) 实现了模块间解耦
- 注释明确记录了清理历史 (MistakeImportDialog 等已删除)

---

## 建议优先处理

1. 删除 `features/template-management/` 空壳或执行迁移
2. 拆分 `CardAgent.ts` (1575行) — 将提示词构建/API调用/结果解析分离
3. 将 `TaskController.examples.ts` 移出到 `__tests__/` 或 `dev/`
