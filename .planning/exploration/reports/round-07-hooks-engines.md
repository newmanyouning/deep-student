# Round 07: Hooks 与引擎 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## Hooks 清单 (35 个)

### 按职责分类

| 类别 | Hooks | 数量 |
|------|-------|------|
| **通用工具** | useDebounce, useCountdown, useMediaQuery, useBreakpoint, useFocusTrap, useDocumentTitle, usePreventScroll | 7 |
| **系统/平台** | useTheme, useAppInitialization, useAppUpdater, useWindowDrag, useNetworkStatus | 5 |
| **事件与导航** | useEventRegistry, useNavigationHistory, useNavigationShortcuts, useTauriEventListener | 4 |
| **学习功能** | usePdfLoader, usePdfProcessingProgress, useQuestionBankSession, useQbankAiGrading, useExamSheetProgress, useLearningHeatmap, useMultimodalSearch, useChatV2Stats, useStatisticsData | 9 |
| **通知与UI** | useNotification, useUnifiedNotification, useViewVisibility | 3 |
| **后台任务** | useBackupJobListener, useMigrationStatusListener, useConflictResolution, useAttachmentSettings, useSystemSettings, useTauriDragAndDrop, useVendorModels | 7 |

### 设计评价

整体质量不错。多数 hooks 小而专注：

| Hook | 行数 | 设计特点 |
|------|------|---------|
| `useDebounce` | 18 | 标准实现，干净 |
| `useMediaQuery` | 47 | SSR-safe + 旧浏览器兼容 |
| `useBreakpoint` | 89 | 5 个 media query + useMemo，提供语义别名 |
| `useViewVisibility` | 25 | 超薄封装，从 viewStore 读取 |
| `useNetworkStatus` | 39 | delegate 到 networkStore，事件→Store 桥接 |
| `useEventRegistry` | 47 | 集中管理 addEventListener/removeEventListener |
| `useWindowDrag` | 20 | Tauri startDragging 的 React 封装 |
| `usePreventScroll` | 44 | 阻止编程滚动，恢复原始样式 |

### 值得关注的大 Hook

| Hook | 估计行数 | 复杂度来源 |
|------|---------|-----------|
| `useAppInitialization` | ~120 | 多步骤异步初始化流程 |
| `useAppUpdater` | ~180 | 更新渠道/频率/跳过/静默检查全链路 |
| `useNavigationHistory` | ~100 | 历史栈管理（前进/后退/替换/中转页过滤） |
| `useTheme` | — | 8 种调色板 + 自选色号 + 历史遗留废弃导出 |

### 发现

1. **useTheme 有废弃导出**: `COLOR_PALETTES` 和 `SPECIAL_PALETTES` 标记 `@deprecated`，但未说明迁移路径
2. **useNotification 有重复**: hooks 和 stores 中均有独立的 `NotificationMessage` 类型定义（与 `types/index.ts` 中的定义重复）
3. **useAppInitialization 直接 invoke**: 延续了 "绕过 API 层" 的模式，直接 `invoke('get_setting')` 而不是通过 settings API

---

## 渲染引擎 (src/engines/)

### 仅 1 个文件: TemplateAIEngine.ts

- 监听 Tauri 事件 `template_ai_stream_{sessionId}_*` 的流式生成
- 使用 Store 的 `getState()` 直接操作状态（非 React 组件内使用）
- 事件驱动架构：start → content → error → complete
- 类模式（需要实例化）

**问题**: `engines/` 目录名暗示应该有多个渲染引擎（Markdown、代码高亮、Mermaid 等），但实际只有一个 AI 引擎。README 中提到的 Markdown/code/Mermaid 渲染可能散布在组件中而非此处。

---

## 样式文件 (16 个)

### 组织方式

```
styles/
├── tailwind.css              # 主入口: @tailwind + @import + @layer
├── shadcn-variables.css      # 设计令牌 (CSS variables)
├── theme-colors.css          # 主题色
├── typography.css            # 排版
├── ios-safe-area.css         # iOS 安全区
├── modern-buttons.css        # 现代化按钮
├── responsive-utilities.css  # 响应式工具
├── shadcn-overrides.css      # shadcn 覆盖
├── notion-animations.css     # Notion 风格动画
├── transitions-dev.css       # 开发环境过渡
├── motion-variants.ts        # Framer Motion 变体 (TS文件!)
└── native-feel/
    ├── cursors.css           # 光标策略
    ├── interaction.css       # 交互反馈
    ├── scrollbars.css        # macOS 风格滚动条
    ├── selection.css         # 文本选择白名单
```

### 关键发现

1. **全局 `user-select: none`**: `tailwind.css` 第 53-60 行设置所有元素默认不可选中，这是 VS Code/Linear/Notion 风格。通过 `native-feel/selection.css` 白名单恢复文本选择
2. **native-feel 是最近的迁移**: 注释提到 "Phase A — 2026-05-14 native-feel migration"，CSS 中有设计文档引用
3. **`motion-variants.ts` 混在 CSS 中**: 这是 TypeScript 文件（Framer Motion 动画变体），不应出现在 `styles/` 目录
4. **CSS 架构迁移状态**: tailwind.css 使用 `@import` 串联 6 个文件 + `@tailwind` 指令。根据 CSS 架构迁移计划，目标是 Tailwind v4，而当前仍为 v3

### 与已知遗留 CSS 的关系

<parameter name="new_string" string="true">_legacy-app.css</parameter>
文件中仍有 `src/shared/styles/_legacy-app.css` 和 `_legacy-deepstudent.css`（被 Stylelint 忽略），这两者与当前的 `styles/` 目录并存，形成新老 CSS 共存的过渡状态。

---

## 发现的问题

- [ ] **P2** — `engines/` 目录仅含 1 个文件（TemplateAIEngine），目录名暗示应有多个渲染引擎但实际缺失。README 中描述的 Markdown/Code/Mermaid 渲染逻辑散布在组件中
- [ ] **P2** — `motion-variants.ts` 是 TypeScript 文件但放在 `styles/` 目录中，应移入 `utils/` 或 `config/`
- [ ] **P2** — `useNotification` 和 `types/index.ts` 中有重复的 `NotificationMessage` 类型定义
- [ ] **P3** — `useTheme` 有标记 `@deprecated` 的导出但缺少迁移路径说明
- [ ] **P3** — `useAppInitialization` 直接调用 `invoke('get_setting')` 绕过 API 层
- [ ] **P3** — 新 CSS (`styles/`) 和遗留 CSS (`shared/styles/_legacy-*.css`) 并存，迁移进行中但未完成
- [ ] **P4** — `tailwind.css` 中的 `stylelint-disable-next-line` 注释暗示 `-webkit-user-select: none` 违反了项目的 `property-no-vendor-prefix` 规则

---

## 层 2 总结: 核心基础设施扫描完成

层 2（R03-R07）覆盖了类型系统、状态管理、API/服务层、UI 组件、Hooks/引擎。整体评估：

- **Hooks 层** — 质量最好的层，小而专注
- **API 层** — 设计良好但未被 Store 充分使用
- **Store 层** — 最大的重构需求，God Store + 绕过 API 层
- **UI 组件层** — cn() 问题贯穿全部 75 个文件
- **类型层** — God File 问题待拆分

## 建议优先处理

1. 将 `motion-variants.ts` 移出 `styles/` 目录
2. 统一 `NotificationMessage` 类型定义到 `types/index.ts`
3. 补充 `engines/` 目录或重命名为 `engines/ai/` 更准确反映内容
4. 清理 `useTheme` 中的废弃导出
