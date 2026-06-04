# Round 11: 知识导图 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 模块规模: 97 文件 — 架构质量最好的功能模块

```
src/features/mindmap/
├── registry/          4 文件 — 组件/布局/预设/样式注册表
├── layouts/           9 文件 — 3 家族 × 6 引擎 (Tree/Balanced/Logic/OrgChart)
├── components/mindmap/ 14 文件 — 节点(4) + 边(5) + 画布(1)
├── components/shared/  5 文件 — EmojiPicker, InlineLatex, BlankedText
├── components/toolbar/  1 文件 — StylePanel
├── store/             5 文件 — mindmap(1526行), document, history, ui
├── presets/           4 文件 — 7 主题预设 × 2 模式 (亮/暗)
├── styles/themes/     8 文件 — 7 个视觉主题
├── constants/         4 文件 — 颜色/布局/快捷键/主题
├── hooks/             2 文件 — 键盘/剪贴板
├── utils/             15 文件 — 布局/节点操作/导出导入
├── views/             2 文件 — MindMapView(10行) + OutlineView(1290行)
└── api/               2 文件 — 后端 API 封装
```

### 关键文件大小

| 文件 | 行数 | 内容 |
|------|------|------|
| `mindmapStore.ts` | **1526** | 核心 Store — 节点增删改、展开折叠、选择、撤销/重做 |
| `OutlineView.tsx` | **1290** | 大纲视图 — 右侧面板，树形文本编辑 |
| `MindMapCanvas.tsx` | 641 | 画布 — React Flow 集成 |
| `BalancedLayoutEngine.ts` | 327 | 均衡布局算法 |
| `TreeLayoutEngine.ts` | 207 | 树形布局算法 |

---

## 架构亮点

### 注册表模式 (Registry Pattern)

```
ComponentRegistry → 节点/边组件注册
LayoutRegistry    → 布局引擎注册
PresetRegistry    → 预设主题注册
StyleRegistry     → 样式注册
```

与 Chat V2 的插件注册表类似，但更专注于 UI 组件。

### 布局引擎 (3 家族 × 6 引擎)

| 家族 | 引擎 |
|------|------|
| **Mindmap** | TreeLayoutEngine, BalancedLayoutEngine |
| **Logic** | LogicTreeLayoutEngine, LogicBalancedLayoutEngine |
| **OrgChart** | HorizontalOrgChartEngine, VerticalOrgChartEngine |

### 7 个可视化主题

default, dark, colorful, colorfulDark, minimal, minimalDark — 每个含亮/暗双模式。

---

## 发现的问题

- [ ] **P2** — `mindmapStore.ts` 1526 行，包含了节点操作、视图状态、选择管理、撤销/重做等混合职责。建议拆分为 nodeStore + uiStore + historyStore（已有独立的 historyStore 和 uiStore 文件，可能未充分利用）
- [ ] **P2** — `OutlineView.tsx` 1290 行 — 大纲视图过重。树形编辑、拖拽排序、内联编辑应拆分为子组件
- [ ] **P3** — `MindMapViewNew.tsx` (12行) vs `MindMapView.tsx` (10行) — "New" 后缀暗示迁移但两者都极小，可能都是包装器
- [ ] **P3** — `types.ts` (414行) 和 `types/index.ts` 同时存在，形成冗余

### 正面评价

- **架构质量最好**的功能模块：注册表模式 + 清晰的目录分类 + 完善的工具函数
- 布局引擎设计优秀：插件化的布局算法，易于扩展新布局
- 节点/边组件化：5 种边类型 + 4 种节点类型，通过 ComponentRegistry 注册

---

## 建议优先处理

1. 进一步拆分 `mindmapStore.ts` (1526行) — 利用已有的独立 store 文件
2. 拆分 `OutlineView.tsx` (1290行) 为可复用的子组件
3. 统一 `types.ts` 和 `types/index.ts` 的导出策略
