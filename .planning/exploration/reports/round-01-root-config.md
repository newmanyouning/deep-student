# Round 01: 根配置与入口文件 — 诊断报告

**日期**: 2026-05-29
**状态**: ✅ 完成

---

## 构建系统 (Vite 6)

### 插件列表

| 插件 | 状态 | 备注 |
|------|------|------|
| `@vitejs/plugin-react` | 活跃 | React Fast Refresh |
| `vite-plugin-static-copy` | 活跃 | 复制 PDF.js cmaps/fonts/wasm 到 dist |
| `rollup-plugin-visualizer` | 条件 | 仅 `ANALYZE=1` 时启用，输出 treemap |
| `exclude-mcp-debug` (自定义) | 条件 | 仅 production，将 4573 行调试代码替换为空实现 |

### resolve.alias

| 别名 | 路径 |
|------|------|
| `@` | `./src` |

### 关键配置项

- **base**: dev 用 `/`，build 用 `./`（适配 Tauri 移动端协议资源加载）
- **server.port**: 固定 `1422`，`strictPort: true`
- **server.watch**: 使用 polling 模式（`interval: 300`），注释说明为 "解决路径含空格时 FSEvents 不工作"
- **build.sourcemap**: 显式设为 `false`（注释："防止生产包意外暴露源码；请勿移除此行"）
- **build.target**: `esnext`（支持 top-level await）
- **build.manualChunks**: 仅 3 个分组 — `vendor-i18n`, `vendor-pdfjs`, `vendor-mermaid`（路线图 Phase 11 计划扩展到 6 个分组）
- **dedupe**: ProseMirror 全家桶 + CodeMirror 全套 + Lezer — 防止 Milkdown/Crepe 的多实例问题

### Dev Proxy (ModelScope MCP)

4 条代理规则指向 `https://mcp.api-inference.modelscope.net`：
- `/sse-proxy` → SSE 连接
- `/messages` → POST 请求
- `/ws-proxy` → WebSocket
- `/http-proxy` → Streamable HTTP

每个代理都有详细的 header 修复逻辑。

### 异常点

1. **Vue 宏定义**: `define` 中设置了 `__VUE_OPTIONS_API__`, `__VUE_PROD_DEVTOOLS__`, `__VUE_PROD_HYDRATION_MISMATCH_DETAILS__` — React 项目不需要这些，疑似从某处复制配置遗留
2. **manualChunks 不完整**: 当前仅 3 个 vendor 分组，与路线图 v1.2 Phase 11 目标差距较大
3. **PostCSS 显式导入**: `postcss.config.js` 使用简单配置，但 `vite.config.ts` 中又显式 `import tailwindcss from "tailwindcss"` 并在 `css.postcss.plugins` 中重复声明，形成双重配置

---

## TypeScript 配置

### 编译选项

| 选项 | 值 | 影响 |
|------|----|------|
| `target` | ES2022 | — |
| `module` | ESNext | — |
| `strict` | **false** | 关闭所有严格检查 |
| `noImplicitAny` | **false** | 允许隐式 any |
| `noUnusedLocals` | false | 未用变量不报错 |
| `noUnusedParameters` | false | 未用参数不报错 |
| `skipLibCheck` | true | 跳过 .d.ts 检查 |
| `jsx` | react-jsx | — |
| `paths` | `@/*` → `src/*` | — |

### 排除范围

- 所有 `__tests__` 目录
- 所有 `*.test.ts/tsx`, `*.spec.ts/tsx` 文件

### 异常点

1. **非严格模式**: `strict: false` + `noImplicitAny: false` — 与路线图 v1.2 Phase 8 目标直接相关，该阶段目标是 "Fix all 17 pre-existing TS errors"
2. **测试排除**: 测试文件完全排除在 TS 编译之外，意味着测试代码不受类型检查保护

---

## 代码规范 (ESLint v9 Flat Config)

### 核心规则

| 规则 | 级别 | 说明 |
|------|------|------|
| `no-restricted-imports` | error | 禁止 shadcn Button/Tooltip，强制使用 NotionButton/CommonTooltip |
| `no-alert` | error | 禁止 window.alert |
| `no-console` | warn | 禁止 console.log（允许 warn/error），注释提到 "1142 处历史 console.log" |
| `no-restricted-syntax` | warn | 禁止直接 window/document.addEventListener |
| `ds-components/no-native-button` | warn | 禁止原生 `<button>` |
| `boundaries/element-types` | warn | Feature 模块边界检查 |

### TypeScript 规则（全部关闭）

- `@typescript-eslint/no-explicit-any`: off
- `@typescript-eslint/no-unused-vars`: off
- `@typescript-eslint/no-require-imports`: off

### 例外目录

- `debug-panel/plugins/` — 完全关闭 import 限制
- `components/dev/` — 完全关闭 import 限制
- `components/ui/shad/` — 原生元素允许
- `promptkit/` — 原生元素允许

### 异常点

1. **AGENTS.md 不存在**: ESLint 配置中 3 次引用 `参见 AGENTS.md 规范`，但该文件在 `.gitignore` 中被排除，不在仓库中
2. **no-console 为 warn**: 1142 处历史 console.log 待清理，仅 warn 不阻断 CI
3. **大量 TS 规则关闭**: `no-explicit-any`, `no-unused-vars` 均应逐步启用

---

## 样式规范 (Stylelint)

### 核心规则

| 规则 | 值 |
|------|----|
| `declaration-no-important` | true（2 个文件例外） |
| `max-nesting-depth` | 3 |
| `selector-max-specificity` | "0,4,2" |

### 忽略的样式文件

- `src/shared/styles/_legacy-app.css`
- `src/shared/styles/_legacy-deepstudent.css`

**注释说明**: "Legacy safety-net stylesheets — will be retired in chunks once decomposed feature files reach visual parity."

### 异常点

1. **_legacy 文件**: 两个遗留 CSS 文件被 Stylelint 忽略，是 CSS 迁移计划的核心目标

---

## 测试配置 (Vitest)

### 环境

- **环境**: jsdom
- **Pool**: forks + singleFork（Node 22 稳定性 workaround，注释提到 "偶发 Channel closed 崩溃"）
- **CSS 处理**: 禁用
- **静默模式**: `silent: true`

### Mock 别名（11 个）

测试环境中大量 Tauri API 和内部模块被 mock：
- Tauri core/event/window/webviewWindow/webview
- react-i18next
- chat-core 多个子模块
- SubjectContext
- heic2any

### 异常点

1. **单进程 fork pool**: 牺牲启动速度换取稳定性，说明测试基础设施存在底层问题

---

## 其他配置

### .release-channel

值为 `experimental` — 表明当前发布频道为实验性。

### Release Please

- Release type: `node`
- 同时更新 `package.json`, `tauri.conf.json`, `Cargo.toml` 的版本号

### .gitignore (189 行)

选择性忽略：
- `AGENTS.md`, `UI-REVIEW.md`, `FRONTEND_ANALYSIS.md` — AI 制品
- `.planning/` — 计划工作流产物（包括本探究文件夹！）
- `docs/design/`, `docs/features/`, `docs/plans/` 等 — 内部设计文档
- 大量一次性 SQL 脚本和测试文件

**注意**: `.planning/` 在 .gitignore 中，本探究计划不会被 git 追踪。

### .vsconfig

Windows Visual Studio 构建依赖：VC Tools, Windows 11 SDK 26100, CMake, ATL, ASAN。

---

## 发现的问题汇总

- [ ] **P1** — `strict: false` + `noImplicitAny: false`，TS 类型安全缺失（v1.2 Phase 8 目标）
- [ ] **P1** — `AGENTS.md` 在 .gitignore 中被排除但 ESLint 多次引用，规范文档丢失
- [ ] **P2** — `manualChunks` 配置不完整，仅 3 个 vendor 分组，与路线图目标差距大
- [ ] **P2** — Vue 宏定义 (`__VUE_*`) 残留在 React 项目的 vite.config.ts 中
- [ ] **P2** — PostCSS 配置存在双重声明（postcss.config.js + vite.config.ts 内联）
- [ ] **P2** — 测试文件 (`*.test.ts`) 被 tsconfig 排除，测试代码无类型检查
- [ ] **P3** — `no-console` 为 warn，1142 处历史 console.log 待清理
- [ ] **P3** — `@typescript-eslint/no-explicit-any` 和 `no-unused-vars` 关闭
- [ ] **P3** — `_legacy-app.css` 和 `_legacy-deepstudent.css` 被 Stylelint 忽略，待迁移
- [ ] **P4** — Vitest 使用 forks + singleFork 稳定性 workaround
- [ ] **P4** — `.planning/` 在 .gitignore 中，探究计划不会入 git 仓库

---

## 建议优先处理

1. 恢复 `AGENTS.md` 到仓库中（或从 .gitignore 移除该项）
2. 开启 `strict: true`（v1.2 Phase 8 计划中）
3. 清理 vite.config.ts 中的 Vue 遗留定义
4. 统一 PostCSS 配置方式（保留 postcss.config.js，移除 vite.config.ts 中的内联声明）
