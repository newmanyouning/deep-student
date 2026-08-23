/**
 * DeepStudent E2E 测试配置
 *
 * 功能:
 * - 通过 CDP 连接 Tauri WebView (port 9222)
 * - 完整 Playwright API: page.click, fill, locators, assertions
 * - 自动截图 + 视频录制 + trace (失败时)
 * - CI 集成 (retries, reporter)
 *
 * 使用:
 *   WEBKIT_DEBUG_PORT=9222 npm run tauri dev &
 *   npx playwright test --config=tests/playwright.e2e.config.ts
 */

import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  timeout: 120_000,           // 2 分钟/测试 (含 UI 等待)
  expect: { timeout: 15_000 },
  retries: process.env.CI ? 2 : 0,
  workers: 1,                 // Tauri app 单实例
  fullyParallel: false,

  reporter: [
    ['html', { outputFolder: 'test-results/html' }],
    ['json', { outputFile: 'test-results/results.json' }],
    ['list'],
    // GitHub Actions 注释
    ...(process.env.CI ? [['github']] as const : []),
  ],

  use: {
    // 截图 + 视频 + trace 仅在失败时
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    trace: 'retain-on-failure',
    // CDP 连接
    ...(process.env.CI ? {} : {
      // 本地: 连接已运行的 Tauri app
      // CI: 启动脚本已设置 WEBKIT_DEBUG_PORT
    }),
  },

  projects: [
    {
      name: 'tauri-windows',
      testMatch: '**/*.spec.ts',
    },
  ],
});
