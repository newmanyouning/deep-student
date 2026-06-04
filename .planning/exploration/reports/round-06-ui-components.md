# Round 06: 通用 UI 组件 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## cn() 使用现状 (决定性发现)

| 实现 | 导入路径 | 使用文件数 | 占比 |
|------|---------|-----------|------|
| **遗留** (无 twMerge) | `@/lib/utils` | **75** | 97.4% |
| **推荐** (clsx+twMerge) | `@/utils/cn` | **2** | 2.6% |

**结论**: `@/lib/utils` 是事实标准，`@/utils/cn` 是名义标准。CODE_STYLE.md 的规定与实际代码完全相反。如果要迁移，需要修改 75 个文件。

---

## 基础 UI 组件 (src/components/ui/)

### 自定义组件

| 组件 | 文件 | 行数 | 关键特征 |
|------|------|------|---------|
| **NotionButton** | `ui/NotionButton.tsx` | 90 | 项目核心按钮，映射到 `buttonPrimitiveContract`，支持 variant/size/iconOnly |
| **NotionDialog** | `ui/NotionDialog.tsx` | 359 | 通用模态框 + NotionAlertDialog，Framer Motion 动画，Portal 渲染 |
| **ScrollArea** | `ui/scroll-area.tsx` | 205 | 平台感知滚动（iOS 原生/其他 OverlayScrollbars），从 study-ui 迁移 |
| **SegmentedControl** | `ui/SegmentedControl.tsx` | — | iOS 风格分段控件，多选项切换 |
| **CommandPalette** | `ui/CommandPalette.tsx` | — | 基于 cmdk 的命令面板 UI |
| **SnappySlider** | `ui/SnappySlider.tsx` | — | 带吸附效果的滑块 |
| **TextSwap** | `ui/TextSwap.tsx` | — | 文字交换动画 |
| **IconSwap** | `ui/IconSwap.tsx` | — | 图标交换动画 |
| **PulseDot** | `ui/PulseDot.tsx` | — | 脉冲指示灯 |
| **ProviderIcon** | `ui/ProviderIcon.tsx` | — | AI 供应商图标 |
| **DeepStudentLogo** | `ui/DeepStudentLogo.tsx` | — | Logo 组件 |
| **SiliconFlowLogo** | `ui/SiliconFlowLogo.tsx` | — | 供应商 Logo |
| **app-menu/** | `ui/app-menu/` | 4 文件 | AppMenu + AppSelect + Demo |

### shadcn/ui 组件 (src/components/ui/shad/)

| 组件 | 文件 | 行数 | 组件 | 文件 | 行数 |
|------|------|------|------|------|------|
| Popover | Popover.tsx | **263** | Tooltip | Tooltip.tsx | 155 |
| Slider | Slider.tsx | 209 | Sheet | Sheet.tsx | 149 |
| Select | Select.tsx | 178 | Tabs | Tabs.tsx | 101 |
| Collapsible | Collapsible.tsx | — | Combobox | Combobox.tsx | — |
| Dialog | Dialog.tsx | — | Command | Command.tsx | — |
| TagInput | TagInput.tsx | 104 | ScrollArea | ScrollArea.tsx | 66 |
| Table | Table.tsx | 69 | Input | Input.tsx | — |
| Switch | Switch.tsx | 44 | Label | Label.tsx | — |
| Progress | Progress.tsx | 37 | Skeleton | Skeleton.tsx | 44 |
| Badge | Badge.tsx | — | Button | Button.tsx | — |
| Textarea | Textarea.tsx | 24 | Separator | Separator.tsx | 12 |
| Alert | Alert.tsx | — | Checkbox | Checkbox.tsx | — |
| Card | Card.tsx | — | Breadcrumb | Breadcrumb.tsx | — |

**shad 组件总计**: 28 个文件，约 2684 行

### 统一侧边栏 (ui/unified-sidebar/)

| 文件 | 职责 |
|------|------|
| `UnifiedSidebar.tsx` | 主侧边栏容器 |
| `SidebarDrawer.tsx` | 移动端抽屉式侧边栏 |
| `SidebarSheet.tsx` | 底部 Sheet 式侧边栏 |
| `UnifiedSidebarSection.tsx` | 侧边栏分区 |
| `MobileSidebarLayout.tsx` | 移动端侧边栏布局 |
| `types.ts` | 类型定义 |

### 关键发现: ESLint 规则的对象

ESLint 禁止使用 `ui/shad/Button` 和 `ui/shad/Tooltip`，强制使用 `NotionButton` 和 `CommonTooltip`。但 Tooltip.tsx (155行) 和 Button.tsx 仍然存在（作为 shad 基础，可能被内部引用）。

**矛盾**: `NotionButton` 使用 `@/lib/utils` cn()（遗留版本），ESLint 规则要求使用它，但它的 cn() 实现不符合 CODE_STYLE.md 规范。

---

## 共享业务组件 (src/components/shared/)

| 组件 | 行数 | 职责 |
|------|------|------|
| **CommonTooltip** | 281 | 通用 Tooltip（Portal + 位置计算 + OverlayCoordinator） |
| **UnifiedDragDropZone** | **716** | 拖拽上传区（最大文件） |
| **UnifiedModelSelector** | 455 | 模型选择器 |
| **UnifiedCodeEditor** | 371 | 代码编辑器包装 |
| **MultimodalIndexButton** | 232 | 多模态索引按钮 |
| **ChatCollapsible** | 109 | 对话折叠面板 |
| **ModelCapabilityIcons** | 124 | 模型能力图标 |
| **Resizable** | 135 | 可调整大小面板 |
| **AiContentLabel** | 99 | AI 生成内容标签 |
| **OverlayCoordinator** | 59 | 覆盖层协调器 |
| **OverlayLayer** | 97 | 覆盖层 |

### 共享组件导出索引

`shared/index.ts` 仅导出 3 个组件: UnifiedDragDropZone, MultimodalIndexButton, AiContentLabel。其余 8 个组件未通过 barrel export 导出。

---

## 布局组件 (src/components/layout/)

| 组件 | 职责 |
|------|------|
| **MobileSlidingLayout** | 三屏滑动布局（左栏←中→右） |
| **BottomTabBar** | 底部导航栏 |
| **UnifiedMobileHeader** | 统一移动端顶栏 |
| **MobileHeader** | 移动端顶栏 |
| **MobileSidebarNavigation** | 移动端侧边栏导航 |
| **Topbar** | 桌面端顶栏 |
| **MacTopSafeDragZone** | macOS 标题栏拖拽区 |
| **TopSafeDragZone** | 通用顶栏拖拽区 |

---

## 发现的问题

- [ ] **P1** — **cn() 使用比例 75:2 颠倒**: 75 个组件使用遗留 `@/lib/utils`，仅 2 个使用推荐的 `@/utils/cn`。CODE_STYLE.md 的规定与实际情况完全相反
- [ ] **P1** — **ESLint 规则强制执行了使用遗留 cn 的组件**: 规则要求使用 NotionButton 和 CommonTooltip，但它们都用 `@/lib/utils`
- [ ] **P2** — `shared/index.ts` 仅导出 3/11 个组件，其余组件无 barrel export，导致导入路径不一致
- [ ] **P2** — `UnifiedDragDropZone` 716 行，是最大组件，应检查是否可拆分
- [ ] **P2** — `ScrollArea` 组件使用了从 study-ui 迁移的 `lib/scroll-platform.ts` 和 `lib/scroll-theme.ts`，形成组件→lib 的跨层依赖
- [ ] **P3** — shad/ui 28 个文件，其中 Button/Tooltip 被 ESLint 禁止直接使用，但文件仍在仓库中（作为内部依赖保留）
- [ ] **P3** — 布局组件 10 个文件中 7 个是移动端相关的，桌面端仅为 Topbar + MacTopSafeDragZone

---

## 建议优先处理

1. **决定 cn() 策略** — 要么更新 CODE_STYLE.md 承认 `@/lib/utils` 为标准，要么执行 75 个文件的批量迁移
2. 完善 `shared/index.ts` 的 barrel export
3. 将 `lib/scroll-*` 移回 components 目录或确认 lib/ 的定位
