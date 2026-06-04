# Round 09: 学习资源中心 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 模块规模

```
src/features/learning-hub/    80 文件
├── LearningHubPage.tsx        1078 行 — 主页面 (三屏滑动布局)
├── LearningHubSidebar.tsx     2803 行 — 侧边栏 (最大文件!)
├── LearningHubSidebarV2.tsx   222 行 — 新版侧边栏 (迁移中?)
├── PreviewRouter.tsx          — 统一预览路由器 (11种格式)
├── ResourceGridView.tsx       384 行 — 网格/列表视图
├── stores/
│   ├── finderStore.ts         878 行 — 主 Store (文件浏览状态)
│   ├── desktopStore.ts        — 桌面端布局
│   └── recentStore.ts         — 最近访问
├── components/
│   ├── finder/                10 文件 — macOS Finder 风格 UI
│   └── ...                    8 文件 — 通用组件
├── apps/views/                14 文件 — 11 种内容预览器
├── hooks/                     6 文件 — 专用 Hooks
└── views/                     3 文件 — 索引状态/诊断/记忆
```

### 关键指标

| 文件 | 行数 | 评级 |
|------|------|------|
| `LearningHubSidebar.tsx` | **2803** | 🔴 God Component |
| `LearningHubPage.tsx` | 1078 | 🟡 偏大 |
| `finderStore.ts` | 878 | 🟡 偏大 |

---

## 架构设计

### 数据流

```
DSTU Protocol (后端 VFS)
    ↓
finderStore (Zustand + persist)
    ↓
LearningHubPage (三屏滑动布局)
├── LearningHubSidebar (左侧: 快捷访问 + 文件夹树)
│   └── finder/ (DesktopView, FileList, QuickAccess, SearchBar)
├── ResourceGridView (中间: 文件网格/列表)
└── PreviewRouter → apps/views/ (右侧: 内容预览)
    ├── NoteContentView, TextbookContentView
    ├── ExamContentView, EssayContentView
    ├── DocxPreview, PptxPreview, XlsxPreview
    ├── ImageContentView, TranslationContentView
    └── FileContentView (兜底)
```

### 设计特点

- **Finder 风格 UI**: macOS Finder 的快捷访问 + 文件夹树 + 内容区三栏布局
- **多 Tab 支持**: `MAX_TABS` 上限，TabBar + TabPanelContainer
- **统一预览路由**: `PreviewRouter` 根据 `previewType` 路由到 11 种预览组件
- **DSTU 协议驱动**: 所有文件操作通过 `dstu/api` 和 `dstu/types` 进行
- **移动端三屏滑动**: 左(入口)←中(文件)→右(应用内容)，手势滑动切换

---

## 发现的问题

- [ ] **P1** — `LearningHubSidebar.tsx` **2803 行**。全项目第二大单一组件文件（仅次于 TauriAdapter 4104行）。需要拆分为独立的功能块
- [ ] **P1** — `LearningHubSidebar.tsx` 和 `LearningHubSidebarV2.tsx` 并存，暗示有重构/迁移在进行中但未完成（类似 contextHelper/contextHelperOptimized 模式）
- [ ] **P2** — `LearningHubPage.tsx` 1078 行，混合了布局管理、事件处理、Tab 管理、拖拽调整大小等职责
- [ ] **P2** — `finderStore.ts` 878 行，包含了视图模式、排序、面包屑、错误处理、多选操作等过多职责
- [ ] **P3** — `PreviewRouter` 中的 `cn()` 从 `@/lib/utils` 导入（延续遗留 cn 模式）
- [ ] **P3** — 6 个 hooks 中仅 `useVfsContextInject` 有 'Vfs' 前缀，其余命名不一致

---

## 建议优先处理

1. 拆分 `LearningHubSidebar.tsx` (2803行) — 将 finder 部分、快捷访问、文件夹树分离为独立组件文件
2. 决策 Sidebar/SidebarV2 的去留 — 完成迁移或删除旧版
3. 从 `finderStore` 中提取 `viewMode`、`sortConfig` 等 UI 状态到独立 store
