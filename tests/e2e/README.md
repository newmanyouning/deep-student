# DeepStudent E2E 测试框架

## 快速开始

```bash
# 1. 构建 + 启动 Tauri app (CDP 模式)
npm run test:e2e:setup

# 2. 运行 Smoke 测试 (快速验证)
npm run test:e2e:smoke

# 3. 运行全部 E2E 测试
npm run test:e2e

# 4. 运行 UI 模式 (交互式调试)
npm run test:e2e:ui

# 5. 运行用户旅程测试
npm run test:e2e:journeys
```

## 架构

```
tests/e2e/
├── fixtures/
│   └── tauri.fixture.ts    ← CDP 连接 + Tauri 命令桥接
├── helpers/
│   └── wait-for-event.ts   ← 事件等待 + DOM 断言
├── pages/                   ← 页面对象 (PO pattern)
├── suites/                  ← 按功能分类的 E2E 测试
│   ├── 01-app-startup.spec.ts
│   ├── 02-learning-hub.spec.ts
│   └── ...
└── journeys/                ← 完整用户旅程
    ├── j1-textbook-learning.spec.ts
    └── ...
```

## 新增测试

```typescript
import { test, expect } from '../fixtures/tauri.fixture';

test('我的新功能测试', async ({ tauriApp }) => {
  const { page, invoke } = tauriApp;

  // 调用后端命令
  const result = await invoke('dstu_list', { path: '/' });
  expect(Array.isArray(result)).toBe(true);

  // 操作 DOM
  await page.click('[data-testid="my-button"]');
  await expect(page.locator('.my-result')).toBeVisible();
});
```

## 模拟 LLM 响应

```typescript
await tauriApp.mockLLM('这是 AI 的回复内容', {
  thinking: '思考过程...',
  toolCalls: [{ name: 'builtin-todo_create', result: { id: '123' } }]
});
```

## CI 运行

```yaml
# PR 时自动运行 smoke tests
# main push 时运行全量 E2E
# 手动触发: Actions → E2E Tests → Run workflow
```
