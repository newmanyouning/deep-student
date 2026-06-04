# Round 10: 笔记系统 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 模块规模

```
src/features/notes/    59 文件  +  src/components/crepe/    11 文件  =  70 文件
```

---

## 关键文件大小

| 文件 | 行数 | 评级 |
|------|------|------|
| `CrepeEditor.tsx` | **2859** | 🔴 God Component (全项目 #3) |
| `NotesCrepeEditor.tsx` | 888 | 🟡 偏大 |
| `NotesSidebar.tsx` | 789 | 🟡 |
| `NotesSidebarV2.tsx` | 728 | 🟡 |
| `useCrepeEditor.ts` | 685 | 🟡 |
| `notesTreeStore.ts` | 465 | ✅ |

---

## 架构

### 编辑器层

```
CrepeEditor (Milkdown 封装, 2859行)
├── useCrepeEditor (Hook, 685行)
├── features/imageUpload.ts (图片上传)
├── features/mermaidPreview.ts (Mermaid 渲染)
├── plugins/ (自定义 Milkdown 插件)
└── types.ts

NotesCrepeEditor (888行)
└── 项目特定的编辑器包装 (工具栏、内容保存、快捷键)

NoteEditorPortal (104行)
└── ⚠️ 白板功能已移除，当前始终返回 null
    (保留组件仅为兼容 App.tsx 引用)
```

### 数据层

```
NotesContext (React Context)
├── notes: 笔记列表
├── loadedContentIds: 已加载内容
├── saveNoteContent / ensureNoteContent
└── editorPortalNoteId

notesTreeStore (Zustand + immer)
├── 树形结构 (DndFileTree)
├── 拖拽状态 (activeId/draggedIds/overId)
├── 筛选状态 (filter/matches/expanded)
└── 持久化快照 (expandedIds/selectedIds)
```

### 组件层

```
NotesHome → NotesSidebar + NotesCrepeEditor
NotesSidebar → DndFileTree (拖拽文件树)
├── TreeContext + TreeNode + DndKitTreeAdapter
└── reference-selector/ (引用选择器)
```

---

## 发现的问题

- [ ] **P1** — `CrepeEditor.tsx` **2859 行**。全项目 #3 大文件，Milkdown 完整集成在一个文件中。应拆分为 plugins/features/hooks 子模块
- [ ] **P1** — `NoteEditorPortal.tsx` 是**死代码**。组件结构保留但始终 `return null`（白板功能移除后的骨架）。注释称"保留以兼容 App.tsx 引用"，应直接清理
- [ ] **P1** — `NotesSidebar.tsx` / `NotesSidebarV2.tsx` 是第 **3 个** V1/V2 并存模式（继 contextHelper, LearningHubSidebar 之后）
- [ ] **P2** — `CrepeEditor.tsx` 和 `useCrepeEditor.ts` 存在**代码重复**：两者都独立配置了 Milkdown（Crepe/CrepeFeature 导入、imageBlockConfig 等）
- [ ] **P2** — `notesTreeStore` 使用 `immer` middleware（通过 `enableMapSet`），但项目其他 store 均不使用 immer — 形成不一致
- [ ] **P3** — `CrepeEditor` 位于 `components/crepe/`（通用组件目录），`NotesCrepeEditor` 位于 `features/notes/`（功能模块），两者的职责边界模糊

---

## 建议优先处理

1. 清理 `NoteEditorPortal.tsx` — 死代码移除，同时清理 App.tsx 中的引用
2. 拆分 `CrepeEditor.tsx` (2859行) — 将 imageUpload、mermaidPreview、plugins 提取为独立文件
3. 决策 NotesSidebar/SidebarV2 的去留 — 完成迁移或删除旧版
