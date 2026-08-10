# 全软件统一测试模块设计

> 日期: 2026-08-10 21:50 CST | 分支: pr/3-pdf-reading
> 来源: 功能精简计划决策点 2 — 用户指示: "设计全软件统一测试模块, 不单独在某个模块使用测试"
> 调研依据: 测试基建全量盘点 (vitest/Playwright CT/e2e/Rust/CI 五路事实收集, 结论均带文件:行号)

---

## 1. 现状盘点

### 1.1 四套测试体系并存

| 体系 | 位置 | 规模 | 配置 |
|------|------|------|------|
| Vitest 单测 | `tests/vitest/` (232 文件) **+ `src/**/__tests__/` (101 文件, 26 个目录)** | 333 | `vitest.config.ts`: jsdom, forks 单进程, include 双轨 |
| Playwright CT | `tests/ct/` | 5 spec | `playwright-ct.config.ts`, alias 到 `tests/ct/mocks/tauri-core-mock.ts` (只返回 null) |
| E2E | `tests/e2e/` | 5 文件 | Playwright + CDP 连 Tauri WebView |
| Rust | 211 个 `#[cfg(test)]` 文件 + `src-tauri/tests/` 21 个集成测试 | — | cargo test (已统一, 不动) |

### 1.2 问题诊断 (按严重度)

| # | 问题 | 证据 |
|---|------|------|
| P1 | **CI 主流程不跑前端单测** — `ci.yml` frontend job 只有 `tsc --noEmit`; vitest 只在 `e2e-tests.yml` 的 component-tests job 跑 | `ci.yml:41` vs `e2e-tests.yml:59` |
| P2 | **Tauri mock 散装** — 无共享 invoke mock; 每个测试文件手工 `vi.fn()` + `vi.mock('@tauri-apps/api/core')` 重复造轮子; CT 的 alias mock 只返回 null, 与单测的手工 mock 行为不一致 | `AppearanceTab.pointerCursor.test.tsx:9` 等数十处 |
| P3 | **双轨分布** — 同一被测对象的测试散在 `tests/vitest/<领域>/` 和 `src/features/<模块>/__tests__/` 两处, 无归属规则 | 如 chat 测试两边都有 |
| P4 | **无统一数据工厂** — 只有散落的局部 makeItem()/createThinkingBlock() | `useTodoStore.test.ts:30` 等 |
| P5 | **无统一 render helper** — 每个组件测试各自 mock react-i18next/SubjectContext | 无 `renderWithProviders` 命中 |
| P6 | **源码契约测试脆性** — `readFileSync` + 字符串断言, 重构即误炸 (本轮精简已更新 2 个) | `secondarySurfaceShellContract.test.ts:6-12` |
| P7 | i18n 在测试里靠 setup 强制 zh-CN, 无按测试覆盖机制 | `vitest.setup.ts:6-10` |

---

## 2. 目标架构: tests/ 单一事实源

```
tests/
├── kit/                    ★ 统一测试工具包 (本设计核心新增)
│   ├── index.ts            统一出口
│   ├── tauri.ts            统一 invoke mock (见 3.1)
│   ├── factories/          统一数据工厂 (makeNote/makeTemplate/makeSession/makeQuestion...)
│   ├── render.tsx          renderWithProviders (i18n + 各 Context 统一包裹)
│   └── fixtures/           共享静态数据 (模板 JSON/对话快照样例...)
├── unit/                   ★ 全部 Vitest 单测的唯一归属 (领域镜像目录)
│   ├── chat-v2/  learning-hub/  settings/  dstu/  anki/  todo/  utils/ ...
│   └── (由 tests/vitest/ 与 src/**/__tests__/ 合并而来)
├── contract/               源码契约测试 (从 tests/vitest 根级 *Contract.test.ts 归拢)
├── ct/                     Playwright 组件测试 (现状保留)
└── e2e/                    E2E (现状保留)
```

原则:
1. **测试不依附模块目录** — `src/` 下不再新增 `__tests__/`; 测试按**领域**在 `tests/unit/` 镜像组织, 与被测文件路径一一对应可寻址
2. **kit 是唯一允许的基础设施来源** — invoke mock/工厂/render helper 禁止在测试文件内手工重造
3. **Rust 侧不动** — cargo test 已是统一体系, CI 已覆盖

---

## 3. 核心件设计

### 3.1 kit/tauri.ts — 统一 invoke mock

```ts
// 命令路由式 mock: 注册处理器, 未注册命令走默认桩并记录调用
export function mockTauri(handlers: Record<string, (args: any) => any>) {
  const calls: Array<{ cmd: string; args: any }> = [];
  const invoke = vi.fn(async (cmd: string, args?: any) => {
    calls.push({ cmd, args });
    if (cmd in handlers) return handlers[cmd](args);
    throw new Error(`[mockTauri] 未注册的命令: ${cmd}`);  // 显式失败, 防止静默 null
  });
  vi.mocked(invoke); // 经 vi.mock('@tauri-apps/api/core') 注入
  return { invoke, calls };
}
```

要点:
- **未注册命令显式抛错** (而非 CT mock 的静默 null) — 让"组件悄悄多调了一个命令"在测试期暴露
- 提供 `installTauriMock()` 在 `vitest.setup.ts` 全局挂载 `vi.mock('@tauri-apps/api/core')`, 各测试只需声明 handlers
- CT 侧 `tests/ct/mocks/tauri-core-mock.ts` 复用同一路由实现, 消除单测/CT 行为分叉

### 3.2 kit/factories — 统一数据工厂

首批 (按测试存量需求排序): `makeChatSession/makeChatMessage/makeContentBlock` (chat-v2 测试最多, 77 文件) / `makeNote` / `makeTemplate` (Anki 模板) / `makeQuestion` / `makeDstuNode` / `makeTodoItem`。每个工厂支持 overrides 参数, 类型与被测代码的生产类型同源 (import 自 src/types)。

### 3.3 kit/render.tsx — renderWithProviders

统一包裹: i18n (真实 i18n 实例, 默认 zh-CN, 可 per-test 切 en-US) + SubjectContext (现 vitest.setup.ts 的全局 mock 迁入选配参数) + 必要的 UI Provider。签名: `renderWithProviders(ui, { i18nLang?, subject?, route? })`。

### 3.4 契约测试去脆性约定

`tests/contract/` 的 readFileSync 断言改为只断言**公开行为锚点** (如导出的函数名/CSS 类名存在), 并不断言实现细节字符串; 被精简删除的文件必须从契约测试同步移除 (本轮已建立先例: secondarySurfaceShellContract/segmentedControlMigrationContract 只读现役文件)。

---

## 4. 迁移计划 (分 4 阶段, 每阶段独立可验收)

| 阶段 | 内容 | 验收 |
|------|------|------|
| **T1 kit 落地** | 建 tests/kit/ (tauri mock + render helper + 首批 7 个工厂); vitest.setup.ts 接入 installTauriMock | kit 自测通过; 不影响现有 333 测试 |
| **T2 新测试强制规范** | eslint 规则/评审约定: 新增测试必须位于 tests/unit/, 必须经 kit mock; src/**/__tests__ 冻结新增 | 规范写入 CODE_STYLE.md |
| **T3 存量搬迁** | 26 个 `src/**/__tests__/` 目录 (101 文件) 按领域分批迁至 `tests/unit/<领域>/`, 同步把手工 invokeMock 换成 kit; tests/vitest/ 根级 73 个散落文件归入 unit/ 或 contract/ | 每批 `npm run test:unit` 全绿; 搬迁是纯移动+mock 替换, 不改断言 |
| **T4 CI 接入** | ci.yml frontend job 增加 `npm run test:unit` (现在只 tsc); vitest include 收敛为 `tests/unit/**` + `tests/contract/**` | CI 前端单测可见; 双轨 include 删除 |

注意: T3 搬迁期间 `vitest.config.ts` 的 include 保持双轨, 迁完后才收敛 — 避免中间态测试丢失。

---

## 5. 与本轮精简的关系

- 本轮已删除 2 个"测试死组件"的成对测试 (NoteTagsEditor/NotesSidebarSearch) 和 2 个契约测试引用 — 存量 333 → 329 左右
- 精简中建立的纪律 (import 级验证、barrel 排除) 同样适用于 T3 搬迁: 搬迁后必须 `npm run test:unit` 全绿才算完成
- `tests/e2e/README.md` 描述的 `pages/`、`suites/02-*` 尚不存在, e2e 扩充不在本设计范围

---

> 调研: 测试基建代理 (vitest/CT/e2e/Rust/CI 五路) + 主会话设计
