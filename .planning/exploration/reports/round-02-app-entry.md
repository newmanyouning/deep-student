# Round 02: 前端入口与路由 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## React 入口分析 (main.tsx — 682 行)

### Provider 嵌套层级

```
ErrorBoundary(name="TopLevel")
  └── OverlayCoordinatorProvider
      └── DialogControlProvider
          └── App (根组件)
```

**注意**: 开发环境不使用 `<React.StrictMode>`（注释称 "避免 effect/事件监听的二次执行"），这是**颠倒的做法** — 生产环境反而包裹了 StrictMode。

### 初始化顺序 (共 16 步)

| 步骤 | 操作 | 位置 |
|------|------|------|
| 1 | 平台检测类初始化 (`initPlatformClasses`) | 同步 |
| 2 | OverlayScrollbars ClickScrollPlugin 注册 | 同步 |
| 3 | React Grab 条件加载 | 异步 |
| 4 | DSTU Logger 注入 | 同步 |
| 5 | 旧事件清理 (HMR re-attach 保护) | 同步 |
| 6 | Console.warn 过滤 (Tauri callback id) | 同步 |
| 7 | Promise.withResolvers polyfill | 同步 |
| 8 | React.PropTypes shim | 同步 |
| 9 | ReactDOM.createRoot 挂载 | 同步 |
| 10 | Sentry 条件初始化 (需用户同意) | 异步 |
| 11 | MCP Bootstrap | 异步 |
| 12 | Tauri log plugin + 全局错误/拒绝上报 | 异步 (仅 Tauri 环境) |
| 13 | MCP Debug 模块 | 异步 |
| 14 | Chat V2 紧急保存 (beforeunload + visibilitychange) | 同步注册 |
| 15 | HMR dispose handler (cleanupRegistry) | 条件 |
| 16 | Sentry 同意的 key 导出 | — |

### 全局错误处理 (3 层过滤)

1. **IIFE 早期过滤** (第 56-89 行): 捕获 Tauri HTTP 插件的 `fetch_cancel_body`、`fetch_read_body+streamChannel`、`resource id invalid` 错误
2. **Console.error 拦截**: 过滤 Tauri IPC 同步触发的 stale resource 错误
3. **Window error + unhandledrejection**: 上报到后端 `report_frontend_log`（含 10 秒节流去重）

### 关键发现

- **StrictMode 颠倒**: 开发环境**禁用** StrictMode（原注释："避免 effect/事件监听的二次执行造成噪声与性能影响"），但生产环境**启用**
- **CleanupRegistry 模式**: 全局清理函数注册表，支持 HMR 热重载时清理事件监听

---

## 路由系统

### 路由模式：无 React Router

使用自定义 **CurrentView 状态机** 替代传统路由：

```
CurrentView 类型 = 'chat-v2' | 'sandbox-workbench' | 'settings' | 'dashboard'
  | 'data-management' | 'task-dashboard' | 'template-management' | 'ui-lab'
  | 'template-json-preview' | 'pdf-reader' | 'learning-hub'
  | 'skills-management' | 'todo'
  (+ DEV: 'crepe-demo' | 'chat-v2-test' | 'tree-test' | 'llm-playground')
```

### 废弃视图重定向 (canonicalView.ts)

17 个旧视图被重定向到新目标：
- `analysis`, `chat`, `batch`, `review` → `chat-v2`
- `notes`, `markdown-editor`, `textbook-library`, `exam-sheet`, `library` → `learning-hub`
- `anki-generation` → `task-dashboard`
- `mistake-detail`, `irec`, `irec-management` 等 → `chat-v2`

### LRU 视图淘汰

- **固定保活**: `chat-v2`
- **最大保活数**: 8 个视图
- 超出限制时驱逐最久未访问的非固定视图

---

## App Shell 结构

### 布局组件树

```
App
├── [桌面端] DesktopShell
│   ├── 标题栏 (titlebar: 40px)
│   │   ├── MacOS 红绿灯占位 (68px)
│   │   ├── 侧边栏折叠按钮 + 更新徽章
│   │   ├── 导航按钮 (前进/后退/新建会话)
│   │   ├── 学习资源面包屑
│   │   ├── 命令面板按钮
│   │   └── 窗口控制按钮
│   ├── 左侧面板 (ModernSidebar)
│   │   └── 导航项 (chat-v2 → learning-hub → todo → skills → task-dashboard → template → settings)
│   ├── 主内容区 (ViewLayerRenderer - 多视图叠加层)
│   │   ├── 隐藏保活层 (visibility:hidden)
│   │   └── 活跃视图层 (z-10)
│   └── 全局组件层
│       ├── NotificationContainer
│       └── CommandPalette
│
└── [移动端] MobileLayout
    ├── UnifiedMobileHeader (56px)
    ├── 主内容区 (MobileSlidingLayout)
    ├── BottomTabBar (56/48px)
    └── 安全区适配 (env(safe-area-inset-*))
```

### 关键 Shell 常量

| 常量 | 值 |
|------|----|
| 桌面侧边栏宽度 | 272px |
| 移动侧边栏宽度 | 110px |
| 标题栏高度 | 40px |
| Mac 红绿灯占位 | 68px |
| 移动端顶栏 | 56px |
| 底部 TabBar（有标签） | 56px |
| 底部 TabBar（无标签） | 48px |

### 视图渲染策略

使用 `ViewLayerRenderer` 组件实现**全部保活 + CSS 显隐切换**：
- 已访问的视图保持挂载（不卸载 DOM）
- 当前视图：`opacity-100 z-10 pointer-events-auto`
- 后台视图：`opacity-0 z-0 pointer-events-none visibility:hidden`
- 未访问视图：不渲染（`return null`）

---

## 配置常量

| 文件 | 内容 | 职责 |
|------|------|------|
| `breakpoints.ts` | 5 级响应式断点 + 语义化尺寸别名 | 响应式布局 |
| `featureFlags.ts` | ChatHost 重构开关 (12个) + MultiVariant 开关 (2个) | 功能开关 |
| `fontConfig.ts` | 18 种字体预设（6组）+ 字号缩放 (0.85-1.3) | 字体系统 |
| `mobileLayout.ts` | 移动端布局高度常量 | 移动端布局 |
| `navigation.ts` | 7 个导航项配置 + i18n 标题 | 导航定义 |
| `zIndex.ts` | 15 级 z-index 层级规范 | 层级管理 |
| `debugPanel.ts` | 调试时间线配置 | 调试面板 |
| `rag.ts` | RAG 阈值从后端设置读取 | RAG 配置 |

---

## Polyfills / Shims

| 文件 | 内容 |
|------|------|
| `promiseWithResolvers.ts` | `Promise.withResolvers()` polyfill（ES2024 特性） |
| `react-proptypes-shim.ts` | `React.PropTypes` 兼容 shim（React 18+ 移除了 PropTypes） |

---

## 发现的问题

- [ ] **P1** — StrictMode 颠倒：开发环境不使用但生产环境使用，与 React 最佳实践完全相反。注释说为避免 effect 双重执行，但这隐藏了副作用 bug
- [ ] **P1** — App.tsx 使用 `cn()` from `@/lib/utils`（第 12 行），是**历史遗留实现**，仅做简单字符串拼接，不支持 Tailwind 类名冲突解决。CODE_STYLE.md 明确规定必须使用 `@/utils/cn`
- [ ] **P1** — App.tsx 是 "God Component"：850+ 行，混合了导航状态、设置加载、维护模式、网络状态、用户协议、主题、Mac 字体平滑、侧边栏半透明、指针光标、命令面板注册等数十种关注点
- [ ] **P2** — 无 React Router：自定义 CurrentView 状态机无法提供 URL 可分享的导航，且路由逻辑分散在 canonicalView、navigation.ts 和 App.tsx 中
- [ ] **P2** — main.tsx 中的 `saveRequestHandler.ts` (472 行) 引用 `MistakeItem` 类型，该类型注释声明已在 2026-01 清理，但类型导入和逻辑仍存在
- [ ] **P2** — `app/services/types.ts` 中 `ChatMessage` 类型仅 20 行，`saveRequestHandler.ts` 却高达 472 行，职责严重不均
- [ ] **P2** — 全局错误处理在 main.tsx 中重复了 index.html 的内联过滤逻辑（两者都过滤 `fetch_cancel_body`）
- [ ] **P3** — `CHAT_HOST_FLAGS` 中所有 12 个 flag 都为 `true`，功能开关失去开关意义（应该是已完成的迁移里程碑）
- [ ] **P3** — `NAV_ITEMS_COUNT = 7` 硬编码，如果添加新导航项需手动更新
- [ ] **P4** — `canonicalView.ts` 中 `DEV_ONLY_VIEWS` 使用 `import.meta.env.DEV` 而非 `import.meta as any`

---

## 建议优先处理

1. 将 App.tsx 中的 `@/lib/utils` cn() 替换为 `@/utils/cn`
2. 拆分 App.tsx：将设置加载逻辑提取到独立 hooks，将导航状态机提取到独立模块
3. 修复 StrictMode：开发环境启用，生产环境可保留（或至少一致）
4. 考虑引入 React Router 或至少将 CurrentView 同步到 URL hash
5. 评估 CHAT_HOST_FLAGS 是否可清理（所有为 true 意味着迁移已完成）
