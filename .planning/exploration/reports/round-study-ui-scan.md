# .study-ui/ 独立实验项目 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 项目身份

```
name: study-ui
type: 独立 Tauri + React 实验项目
角色: 主项目的 UI 组件实验室 / 原型验证环境
状态: 部分已迁移到主项目，剩余为实验性代码
```

### 与主项目的关键差异

| 维度 | .study-ui | 主项目 (deep-student) |
|------|----------|---------------------|
| React | **19.2.4** | 18.3.1 |
| Vite | **7.3.1** | 6.0.3 |
| Tailwind | **v4** (CSS-first config) | v3 (JS config) |
| Tauri | 2.7.0 | 2.8.4 |
| TypeScript | **5.9.3** | 5.6.2 |
| cn() 实现 | ✅ `clsx + twMerge` (正确) | ⚠️ `@/lib/utils` (遗留) |

**.study-ui 是主项目的 "先行版本"** — 使用更新的技术栈，部分已验证的代码已迁移到主项目。

---

## 目录结构 (190 文件)

```
.study-ui/
├── 配置文件 (8)
│   ├── package.json (61行) — 独立 npm 项目
│   ├── tsconfig.json / eslint.config.mjs / postcss.config.mjs
│   ├── vite.config.ts / vitest.config.ts / vitest.setup.ts
│   ├── components.json — shadcn/ui 组件配置
│   └── kumo.json — Kumo UI 框架配置
│
├── src/ (93 文件)
│   ├── components/
│   │   ├── shell/ (8) — AppChrome, Sidebar, Titlebar, WindowControls
│   │   ├── content/ (15) — SettingsDemoPanel, ThreadCanvas
│   │   ├── settings/ (2) — AppSettingsProvider
│   │   ├── theme/ (1) — theme-provider
│   │   └── ui/ (12) — button, card, dialog, dropdown-menu, input,
│   │                scroll-area, sheet, surface, switch, tabs, textarea, tooltip
│   ├── lib/ (13) — app-shell, app-settings, scroll-platform,
│   │              scroll-theme, native-window, theme, utils(cn), ...
│   ├── styles/ (1) — app.css
│   ├── App.tsx / main.tsx
│   └── vite-env.d.ts
│
├── scripts/ (21 文件)
│   └── 19 个 contract 测试 (.test.mjs/.test.ts) + 2 个工具脚本
│       覆盖: scroll-area, switch, button, input, shell icons,
│              window background, macos titlebar, visual language...
│
├── src-tauri/ (24 文件)
│   ├── src/ (3) — lib.rs(22行), main.rs(6行), window_background.rs(338行)
│   ├── icons/ (16) — 全平台应用图标
│   ├── capabilities/default.json
│   ├── tauri.conf.json + tauri.macos.conf.json + tauri.windows.conf.json
│   └── Cargo.toml / Cargo.lock / build.rs
│
├── docs/ (37 文件)
│   ├── plans/ (30) — UI 设计计划 (2026-03 ~ 2026-07)
│   └── research/tmp/ (5) — 调研轮次记录
│
└── 根文档 (3)
    ├── README.md — ⚠️ 错误的 Next.js 模板内容
    ├── UI优化方案.md — 真实的 UI 审计报告
    └── test.md
```

---

## 已迁移到主项目的代码

| 文件 | 迁移状态 | 主项目位置 |
|------|---------|-----------|
| `scroll-platform.ts` | ✅ 已迁移 | `src/lib/scroll-platform.ts` |
| `scroll-theme.ts` | ✅ 已迁移 | `src/lib/scroll-theme.ts` |
| `cn()` (utils.ts) | ⚠️ 正确实现但主项目未采用 | `src/utils/cn.ts` (仅2文件使用) |

主项目中的 `scroll-platform.ts` 和 `scroll-theme.ts` 注释明确写道：
> "Moved from study-ui into the main app so DeepStudent no longer depends on the `@study-ui` alias"

---

## 未迁移的 UI 组件 (潜在可复用)

### Shell 组件组 (最可能迁移)

| 组件 | 行数 | 说明 |
|------|------|------|
| `AppChrome.tsx` | 615 | 应用主框架 (标题栏 + 侧边栏 + 内容区) |
| `Titlebar.tsx` | — | macOS 风格标题栏 |
| `WindowControls.tsx` | — | 窗口控制按钮 (红绿灯) |
| `Sidebar.tsx` | — | 侧边栏实现 |
| `SidebarUpdateBadge.tsx` | — | 更新徽章 |
| `FramelessResizeHandles.tsx` | — | 无边框窗口调整手柄 |

### UI 组件组

| 组件 | 说明 |
|------|------|
| `button.tsx` | 基于 CVA 的按钮变体 |
| `scroll-area.tsx` | Tailwind v4 版本滚动区域 |
| `input.tsx` / `textarea.tsx` | 表单输入 |
| `switch.tsx` / `tabs.tsx` | 交互控件 |
| `dialog.tsx` / `sheet.tsx` / `tooltip.tsx` | Radix UI 封装 |
| `dropdown-menu.tsx` | 下拉菜单 |
| `card.tsx` / `surface.tsx` | 布局容器 |

### 内容组件组

| 组件 | 说明 |
|------|------|
| `SettingsDemoPanel.tsx` | 设置面板原型 |
| `SettingsPanel.tsx` | 设置面板组件 |
| `ThreadCanvas.tsx` | 对话线程画布 |

### lib/ 模块 (已部分迁移)

| 模块 | 状态 |
|------|------|
| `app-shell.ts` (209行) | ❌ 未迁移 — 应用布局策略 |
| `app-settings.ts` | ❌ 未迁移 — 设置管理 |
| `native-window.ts` | ❌ 未迁移 — 原生窗口偏好 |
| `theme.ts` | ❌ 未迁移 — 主题管理 |
| `macos-titlebar-geometry.ts` | ❌ 未迁移 — Mac 标题栏几何 |

---

## 测试体系

.study-ui 有 **23 个 `.source.test.ts` 文件** — 一种 "源码级契约测试" 模式：
- 测试文件与源文件放在同一目录
- 使用 Node.js 原生 test runner (`node --test --experimental-strip-types`)
- 验证 UI 组件的行为契约

scripts/ 目录还有 19 个 contract 测试 (.test.mjs)，覆盖视觉语言、交互状态、窗口背景等。

---

## Rust 后端 (366 行)

```
src-tauri/src/
├── lib.rs (22行) — Tauri 入口
├── main.rs (6行) — 二进制入口
└── window_background.rs (338行) — 窗口背景效果
    (NSVisualEffectView / SetWindowCompositionAttribute / blur)
```

专为**原生窗口背景效果**（模糊、透明、 vibrancy）设计，支持 macOS/Windows/Linux 三平台。

---

## 发现问题

- [ ] **P1** — README.md 是**错误的 Next.js 模板**，与项目完全无关（`create-next-app` 默认内容），应替换为 study-ui 的实际说明
- [ ] **P2** — `.study-ui/` 使用 React 19 + Vite 7 + Tailwind v4，主项目使用 React 18 + Vite 6 + Tailwind v3。技术栈升级路径不明确
- [ ] **P2** — 部分代码已迁移（scroll-platform/theme），但还有大量未迁移的 Shell/UI/lib 代码。迁移策略不清晰：全量迁移 vs 按需迁移 vs 独立维护？
- [ ] **P2** — `UI优化方案.md` 中的 P0 可访问性问题（skip link、触摸目标、键盘支持）修复状态未知
- [ ] **P3** — 23 个 `.source.test.ts` 测试文件与源文件混放，主项目采用 `tests/` 集中管理模式
- [ ] **P3** — `kumo.json` — Kumo UI 框架配置，但项目注释中提到 "no-kumo-runtime" contract 测试，暗示 Kumo 可能已被放弃
- [ ] **P3** — `test.md` 位于根目录，内容未知

---

## 建议

1. **更新 README.md** — 替换为 study-ui 项目的真实描述
2. **明确迁移策略** — 三种选项：
   - A) 全量迁移到主项目（利用更新的技术栈）
   - B) 删除 .study-ui/（已迁移的部分保留，其余丢弃）
   - C) 保持独立实验室但建立明确的迁移 pipeline
3. **跟进 UI优化方案.md** 中的 P0 可访问性问题
4. 评估 Kumo UI 的使用状态，如已放弃则清理 `kumo.json` 和相关配置
