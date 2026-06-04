# 累积问题追踪

**更新**: 2026-05-29 (Round 01-33 全部完成) ✅ 最终版本

---

## 重复出现的模式问题

### 模式 1: God File / God Component

| 文件 | 行数 | 发现轮次 |
|------|------|---------|
| `TauriAdapter.ts` | **4104** | R08 |
| `CrepeEditor.tsx` | **2859** | R10 |
| `LearningHubSidebar.tsx` | **2803** | R09 |
| `QuestionBankEditor.tsx` | **2496** | R12 |
| `EnhancedPdfViewer.tsx` | 1783 | R14 |
| `questionBankStore.ts` | 1630 | R04 |
| `CardAgent.ts` | 1575 | R13 |
| `mindmapStore.ts` | 1526 | R11 |
| `TaskDashboardPage.tsx` | 1356 | R13 |
| `OutlineView.tsx` | 1290 | R11 |
| `types/index.ts` | 1036 | R03 |
| `App.tsx` | 850+ | R02 |
| `main.tsx` | 682 | R02 |

### 模式 5: 空壳 Feature 目录

`features/` 目录下多个子目录为空壳，只有 .gitkeep + `export {}`：

| 目录 | 发现轮次 |
|------|---------|
| `features/practice/` | R12 |
| `features/template-management/` | R13 |
| (其他 feature 目录正常) | — |

这表明 `features/` 模块化迁移被启动但未完成。
| `types/index.ts` | 1036 | ChatMessage, ApiConfig, AnkiCard, Template, ExamSheet, Theme, Statistics, Events |
| `questionBankStore.ts` | 1630 | 题目CRUD, CSV导入导出, 练习模式, 数据同步, 时间统计, 打卡日历 |
| `App.tsx` | 850+ | 导航, 设置加载, 维护模式, 网络状态, 用户协议, 主题, 字体平滑... |
| `main.tsx` | 682 | Tauri噪声过滤, Sentry, Log plugin, MCP Debug, 错误上报, 紧急保存 |

### 模式 2: 双重实现

同一功能有两套代码：

| 功能 | 推荐方案 | 遗留方案 | 影响文件 |
|------|---------|---------|---------|
| `cn()` 类名合并 | `@/utils/cn` (clsx+twMerge) | `@/lib/utils` (手写,无twMerge) | App.tsx 使用错误版本 |
| 错误处理 | `try-catch + getErrorMessage` | `Result<T,E>` (Rust风格) | 仅VFS模块使用Result |
| Store 目录 | `stores/` (Zustand) | `store/` (自定义Pub/Sub) | ResourceStateManager |
| 持久化 | Zustand persist | 自实现 | ankiQueueStore 自实现版本号去重 |

### 模式 3: 废弃代码存活

`MistakeItem` (2026-01 标记废弃) 的完整依赖链（2026-05-30 更新：types/api.ts 已删除，从 4 层减至 3 层）：

```
types/index.ts (定义, @deprecated)
  → stores/anki/types.ts (MistakeSummary = MistakeItem)
    → stores/anki/useAnkiUIStore.ts (ImportSlice使用)
      → app/services/saveRequestHandler.ts (运行时逻辑)
```

4 个确认的引用文件（原 5 个，types/api.ts 已删除），1 个仍在使用运行时逻辑。

### 模式 4: 架构层违规

Store 直接调用 `invoke()` 而绕过 API 层：

| Store | 直接 invoke 调用 |
|-------|-----------------|
| questionBankStore | `qbank_list_questions`, `qbank_submit_answer`, `qbank_get_stats` 等 20+ 命令 |
| researchStore | 通过事件驱动，非直接 invoke |
| unifiedIndexStore | ✅ 使用 `vfsUnifiedIndexApi` 封装 (唯一遵循规范) |
| ankiQueueStore | 通过 `TauriAPI` 封装 |

---

## 按优先级汇总

### P1 — 必须修复

| ID | 问题 | 发现轮次 | 影响范围 |
|----|------|---------|---------|
| P1-01 | `strict: false`, TypeScript 非严格模式 | R01 | 全局类型安全 |
| P1-02 | `AGENTS.md` 被 .gitignore 排除，规范文档缺失 | R01 | 开发规范一致性 |
| P1-03 | StrictMode 颠倒（开发关/生产开） | R02 | React 最佳实践 |
| ~~P1-04~~ | ~~**cn() 使用比例颠倒**~~ ✅ 已解决 (2026-05-30, REF-004) — `lib/utils.ts` 改为从 `utils/cn` 重导出，所有 ~250 文件自动获得 twMerge 支持 | R02, R03, **R06** | 所有 UI 组件 Tailwind 冲突 |
| P1-05 | `types/index.ts` 1036行 God File | R03 | 类型可维护性 |
| P1-06 | `questionBankStore` 1630行 God Store | R04 | Store 可维护性 |
| P1-07 | Store 直接调用 invoke() 绕过 API 层（API 层存在但未被使用） | R04, **R05** | 前后端分离 |
| P1-08 | ESLint 强制使用的 NotionButton/CommonTooltip 实际使用遗留 cn()，形成规范矛盾 | **R06** | 规范一致性 |

### P2 — 应该修复

| ID | 问题 | 发现轮次 |
|----|------|---------|
| P2-01 | Vue 宏定义残留在 React 项目 vite 配置 | R01 |
| P2-02 | PostCSS 双重声明 | R01 |
| P2-03 | 无 React Router，自定义视图状态机 | R02 |
| P2-04 | App.tsx God Component | R02 |
| P2-05 | `DialogControlContext` (628行) 放错目录 | R03 |
| P2-06 | 两种错误处理风格并存 | R03 |
| ~~P2-07~~ | ~~`store/` 与 `stores/` 双目录并存~~ ✅ 已解决 (2026-05-30, REF-006) — store/ 目录已删除 | R04 |
| ~~P2-08~~ | ~~持久化策略不统一~~ ✅ 已解决 (2026-05-30, REF-007) — tauriPersistStorage 适配器统一 | R04 |
| ~~P2-09~~ | ~~`MistakeItem` 废弃链~~ ✅ 已解决 (2026-05-31, REF-001-D) — 9 文件协调清理, 类型定义已删除 | R02, R03, R04 |
| ~~P2-10~~ | ~~`cn()` 双实现~~ ✅ 已解决 (2026-05-30, REF-004) | R03 |
| ~~P2-11~~ | ~~`types/ui.ts`, `api.ts`, `hooks.ts` 纯 re-export 无意义层~~ ✅ 已解决 (2026-05-30) | R03 |
| P2-12 | 服务层 4 种架构模式混用 (Singleton/静态类/对象/模块函数) | **R05** |
| P2-13 | `shared/index.ts` 仅导出 3/11 组件；`events/chat.ts` 3/6 事件为测试 | R05, **R06** |

### P3 — 建议修复

| ID | 问题 | 发现轮次 |
|----|------|---------|
| P3-01 | 1142 处历史 console.log | R01 |
| P3-02 | ESLint TS 规则全部关闭 | R01 |
| P3-03 | CSS 遗留文件 `_legacy-app.css` | R01 |
| P3-04 | `CHAT_HOST_FLAGS` 12个开关全为 true | R02 |
| P3-05 | `NAV_ITEMS_COUNT=7` 硬编码 | R02 |
| P3-06 | `HpiasEvent` union 40+ variants | R04 |
| P3-07 | Settings tab 映射表硬编码 | R04 |
| P3-08 | shad/ui 28文件，Button/Tooltip 被禁止但保留 | **R06** |
| P3-09 | 10 个布局组件中 7 个为移动端，桌面端仅 2 个 | **R06** |

### P4 — 低优先级

| ID | 问题 | 发现轮次 |
|----|------|---------|
| P4-01 | Vitest forks + singleFork 稳定性 workaround | R01 |
| P4-02 | `.planning/` 在 .gitignore 中 | R01 |
| P4-03 | AnkiCard 有废弃字段别名 | R03 |
| P4-04 | `.d.ts` 和 `.ts` 文件混放 types/ | R03 |
| P4-05 | `debugDatabase.ts` 存在暗示 API 层缺少正常查询接口 | **R05** |

---

*更新: 2026-05-30 — REF-009 LLM 适配器架构调研完成 (10 LLM + 9 OCR + 6 TTS + 11 Image)。
现有代码分析: llm_manager/adapters 已有 14 个 RequestAdapter, ocr_adapters 已有 OcrAdapterFactory。
新 ProviderDiscovery 设计为包装层，直接对接现有 AdapterRegistry + LLMManager, 无需替换。
