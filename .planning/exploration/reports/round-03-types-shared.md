# Round 03: 类型系统与共享层 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 类型定义概况

### 文件清单

| 文件 | 行数 | 实际作用 |
|------|------|---------|
| `types/index.ts` | **1036** | 核心类型定义（所有类型集中在一个文件） |
| `types/navigation.ts` | 53 | CurrentView 联合类型 + 导航历史类型 |
| `types/i18n.ts` | 81 | react-i18next 类型增强 + 支持语言列表 |
| `types/global.d.ts` | 17 | Google Analytics gtag 类型 |
| ~~`types/ui.ts`~~ | ~~36~~ | ✅ 已删除 (2026-05-30, REF-001) |
| ~~`types/api.ts`~~ | ~~40~~ | ✅ 已删除 (2026-05-30, REF-001) |
| ~~`types/hooks.ts`~~ | ~~22~~ | ✅ 已删除 (2026-05-30, REF-001) |
| `types/dataGovernance.ts` | — | 数据治理相关 |
| `types/database.ts` | — | 数据库相关 |
| `types/dragDrop.ts` | — | 拖拽相关 |
| `types/enhanced-field-types.ts` | — | 增强字段类型 |
| `types/sentry.d.ts` | — | Sentry 类型声明 |
| `types/shims-tauri-plugins.d.ts` | — | Tauri 插件类型垫片 |
| `types/tauri-window.d.ts` | — | Tauri 窗口声明 |
| `types/textbook.ts` | — | 教材相关 |
| `types/vfs-unified-index.ts` | — | VFS 统一索引 |

### 关键发现

1. **`types/index.ts` 是 God File**: 1036 行包含所有核心类型 — ChatMessage (110+ 字段), ApiConfig, AnkiCard, Template, ExamSheet, Statistics, Theme 等，没有按领域拆分
2. **ui.ts / api.ts / hooks.ts 是纯 re-export**: 从 index.ts 重新导出类型，增加了不必要的文件数和导入路径复杂度
3. **MistakeItem 明确废弃但仍然存在**: 类型上标记 `@deprecated 2026-01`，代码注释列出 4 个仍在使用它的文件
4. **Anki 相关类型超过 300 行**（AnkiCard, CustomAnkiTemplate, FieldExtractionRule 等），应独立文件

---

## 工具函数清单

### cn() 双实现对比

| 维度 | `@/utils/cn` (**推荐**) | `@/lib/utils` (**遗留**) |
|------|------------------------|------------------------|
| 行数 | 6 | 29 |
| 依赖 | `clsx` + `tailwind-merge` | 无外部依赖（手写 clsx 克隆） |
| Tailwind 冲突解决 | ✅ `twMerge` | ❌ 仅简单拼接 |
| 使用量 | 待 Round 06 统计 | **App.tsx 直接使用** |

**结论**: 项目有两套 `cn()` 实现，且核心文件 App.tsx 使用了错误的遗留版本。这是 R01 中报告的遗留代码问题的具体体现。

### 核心工具文件

| 文件 | 行数 | 用途 |
|------|------|------|
| `errorUtils.ts` | 102 | `getErrorMessage()` 统一错误处理 + 路径脱敏 |
| `shared/result.ts` | 588 | Rust 风格 `Result<T,E>` + `VfsError` + `ok/err` + `map/andThen` |
| `platform.ts` | — | 平台检测 (isWindows/isMacOS) |
| `tauriApi.ts` | — | Tauri API 封装（Round 05 详查） |

### errorUtils 特点

- 路径脱敏：自动移除 Cargo 源码路径、用户主目录、工作区绝对路径
- 后端 JSON 错误解析：`{"code":"X","message":"..."}` 格式自动提取 message

### shared/result.ts 特点

- 完整实现了 Rust 的 Result 模式：`Ok<T>`, `Err<E>`, `ok()`, `err()`, `map()`, `andThen()`, `unwrapOr()`
- `VfsError` 类：11 种错误码 (`NOT_FOUND`, `NETWORK`, `TIMEOUT`…)，支持 `recoverable` 标志
- 关键字分类函数 `classifyErrorMessage()`: 从字符串错误消息推断错误码
- 但：仅在 VFS 模块使用，未被其他地方采用 — 有价值但影响有限

---

## 基础库 (lib/)

| 文件 | 行数 | 用途 |
|------|------|------|
| `utils.ts` | 29 | **遗留 cn()** 实现 |
| `scroll-platform.ts` | 49 | 运行时检测滚动平台 (iOS/Tauri/Touch) |
| `scroll-theme.ts` | 47 | OverlayScrollbars 主题 (useSyncExternalStore) |
| `template-parser.ts` | — | 流式模板 XML Action 解析器 |

lib/ 是从 `@study-ui` 迁移来的小型模块集。scroll-platform 和 scroll-theme 注释中提到 "Moved from study-ui into the main app so DeepStudent no longer depends on the `@study-ui` alias"。

---

## React Contexts

| Context | 文件 | 行数 | 职责 |
|---------|------|------|------|
| DialogControlContext | `contexts/DialogControlContext.tsx` | **628** | MCP 工具选择 + 搜索引擎选择 + 持久化 + 实时同步 |

### DialogControlContext 分析

这是项目中**唯一的 React Context**（除 shared 的 portal contexts），但极其复杂：

- 管理 MCP 工具选择（从数据库持久化）
- 管理搜索引擎选择（从数据库持久化）
- 实时连接状态同步（McpService.onStatus 订阅）
- 选择清洗（cleanSelectionsAgainstAvailability）
- `systemSettingsChanged` 事件响应
- MCP bootstrap 完成事件响应
- 1200ms 重载冷却

这个 Context 应该在 MCP 模块内部，而非放在全局 `contexts/` 目录。

---

## 共享图标系统

```
src/shared/icons/
├── index.ts      — 统一 Icon 导出入口 (Phosphor)
├── adapter.ts    — Lucide → Phosphor 迁移适配器 (Tailwind size → pixel)
└── mapping.ts    — LUCIDE_TO_PHOSPHOR_MAP 映射表
```

项目正在从 Lucide 图标库迁移到 Phosphor。`extractSizeFromClassName()` 和 `stripSizeClasses()` 是迁移辅助工具。

---

## 发现的问题

- [ ] **P1** — `types/index.ts` 1036 行 God File，混合了 ChatMessage、ApiConfig、AnkiCard、Theme 等完全不相关的类型。应按领域拆分（chat、anki、api、theme、exam）
- [x] **P1** ✅ `types/ui.ts`, `types/api.ts`, `types/hooks.ts` 已删除 (2026-05-30, REF-001)
- [ ] **P1** — App.tsx 使用 `cn()` from `@/lib/utils`（遗留实现），不支持 Tailwind 类名冲突解决
- [ ] **P2** — `DialogControlContext` 628 行放在 `contexts/` 目录，但它是 MCP/搜索引擎专用，应移入 `features/chat/` 或 `mcp/`
- [ ] **P2** — `shared/result.ts` 的 Rust 风格 Result 模式仅在 VFS 模块使用，与项目主流的 try-catch + getErrorMessage 模式不一致，形成两种错误处理风格并存
- [x] **P2** ~~`MistakeItem` 类型在 4 个文件中仍被引用~~ ✅ 已解决 (2026-05-31, REF-001-D)
- [ ] **P3** — `src/lib/utils.ts` 的遗留 cn() 实现应标记为 deprecated 并逐步替换引用
- [ ] **P3** — `types/` 目录包含 `.d.ts` 文件 (`sentry.d.ts`, `shims-tauri-plugins.d.ts`, `tauri-window.d.ts`) 和 `.ts` 文件混放，应按约定分离声明文件和类型定义文件
- [ ] **P4** — `AnkiCard` 有 `max_cards_per_mistake`（已废弃别名），且和 `AnkiLibraryCard` 存在大量字段重复

---

## 建议优先处理

1. 将 `types/index.ts` 拆分为按领域的子文件（chat.ts, anki.ts, api.ts 等），index.ts 仅做统一 re-export
2. ~~删除 `types/ui.ts`, `types/api.ts`, `types/hooks.ts`~~ ✅ 已完成 (2026-05-30, REF-001)
3. 批量替换 `@/lib/utils` cn() → `@/utils/cn`（可用 codemod 或脚本）
4. 将 `DialogControlContext` 移入 `mcp/` 或 `features/chat/` 目录
5. 制定 MistakeItem 移除计划，先清理 4 个引用文件再删除类型定义（已从 4 层减至 3 层）
