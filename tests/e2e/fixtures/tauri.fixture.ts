/**
 * Tauri E2E 测试 Fixture
 *
 * 通过 CDP 连接 Tauri WebView，注入自定义 fixture:
 *   - tauriApp.page        — Playwright Page (完整 API)
 *   - tauriApp.invoke()    — 调用 Tauri 后端命令
 *   - tauriApp.waitForEvent() — 等待 Tauri 事件
 *   - tauriApp.seedTestData() — 一键种子
 *   - tauriApp.mockLLM()   — 模拟 LLM 响应
 */

import { test as base, chromium, type Page } from '@playwright/test';

// ============================================================================
// 类型定义
// ============================================================================

export interface TauriApp {
  page: Page;
  /** 调用 Tauri 后端命令 */
  invoke<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T>;
  /** 等待 Tauri 事件 */
  waitForEvent(eventName: string, timeoutMs?: number): Promise<unknown>;
  /** 一键种子测试数据 */
  seedTestData(): Promise<{ folderId: string }>;
  /** Mock LLM 响应 (避免实际 API 调用) */
  mockLLM(content: string, opts?: MockLLMOptions): Promise<void>;
}

export interface MockLLMOptions {
  thinking?: string;
  toolCalls?: Array<{ name: string; result: unknown }>;
  delay?: number;
}

export type TauriFixtures = {
  tauriApp: TauriApp;
};

// ============================================================================
// CDP 连接辅助
// ============================================================================

const CDP_URL = process.env.CDP_URL || 'http://localhost:9222';

async function connectToTauri(): Promise<{ browser: unknown; page: Page }> {
  const maxRetries = process.env.CI ? 30 : 15;
  const retryInterval = 2000;

  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch(`${CDP_URL}/json`);
      const pages = await response.json() as Array<{ type: string; url: string }>;
      const page = pages.find(p => p.type === 'page');
      if (page) {
        const browser = await chromium.connectOverCDP(CDP_URL);
        const contexts = browser.contexts();
        const defaultContext = contexts[0];
        const appPage = defaultContext.pages()[0];
        await appPage.waitForLoadState('domcontentloaded');
        return { browser, page: appPage };
      }
    } catch {
      // CDP 尚未就绪，重试
    }
    await new Promise(r => setTimeout(r, retryInterval));
  }
  throw new Error(`无法连接到 Tauri CDP: ${CDP_URL} (${maxRetries} 次重试后)`);
}

// ============================================================================
// Fixture 定义
// ============================================================================

export const test = base.extend<TauriFixtures>({
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  tauriApp: [async ({}, use: any) => {
    const { browser, page } = await connectToTauri();

    // 验证 Tauri API 已加载
    const tauriLoaded = await page.evaluate(() => {
      return !!(window as any).__TAURI__?.core?.invoke;
    });
    if (!tauriLoaded) {
      throw new Error('Tauri API 未加载 — 请确保 app 在 debug 模式运行');
    }

    const tauriApp: TauriApp = {
      page,

      // ---- Tauri 命令调用 ----
      invoke: async <T>(cmd: string, args?: Record<string, unknown>) => {
        return page.evaluate(
          ({ cmd, args }) => (window as any).__TAURI__.core.invoke(cmd, args),
          { cmd, args }
        ) as Promise<T>;
      },

      // ---- 事件等待 ----
      waitForEvent: async (eventName: string, timeoutMs = 30_000) => {
        return page.evaluate(
          ({ eventName, timeoutMs }) => {
            return new Promise((resolve, reject) => {
              const timer = setTimeout(() => {
                reject(new Error(`等待事件超时: ${eventName} (${timeoutMs}ms)`));
              }, timeoutMs);

              const { listen } = (window as any).__TAURI__?.event || {};
              if (!listen) {
                clearTimeout(timer);
                reject(new Error('Tauri event API 不可用'));
                return;
              }

              listen(eventName, (event: unknown) => {
                clearTimeout(timer);
                resolve(event);
              }).catch(reject);
            });
          },
          { eventName, timeoutMs }
        );
      },

      // ---- 种子数据 ----
      seedTestData: async () => {
        const result = await page.evaluate(async () => {
          const invoke = (window as any).__TAURI__.core.invoke;

          // 创建测试文件夹
          const folder = await invoke('dstu_folder_create', {
            path: '/',
            name: 'E2E_Test_Data'
          });

          // 创建测试笔记
          await invoke('dstu_create', {
            path: `/${folder.id}`,
            resourceType: 'note',
            title: 'E2E Test Note',
            content: '# Test Note\n\nThis note was created by the E2E test framework.'
          });

          // 创建测试题库
          await invoke('dstu_create', {
            path: `/${folder.id}`,
            resourceType: 'exam',
            title: 'E2E Test Exam',
            questions: JSON.stringify([
              { type: 'single_choice', question: 'What is 2+2?', options: ['3', '4', '5'], answer: '4' },
              { type: 'single_choice', question: 'What is H2O?', options: ['Oxygen', 'Water', 'Hydrogen'], answer: 'Water' }
            ])
          });

          return { folderId: folder.id };
        });
        return result;
      },

      // ---- LLM Mock ----
      mockLLM: async (content: string, opts: MockLLMOptions = {}) => {
        const { thinking, toolCalls, delay = 50 } = opts;

        await page.evaluate(
          async ({ content, thinking, toolCalls, delay }) => {
            const chunks = content.match(/.{1,10}/g) || [content];

            // Mock thinking block
            if (thinking) {
              window.dispatchEvent(new CustomEvent('mock:thinking', {
                detail: { text: thinking }
              }));
              await new Promise(r => setTimeout(r, 100));
            }

            // Mock streaming content
            for (const chunk of chunks) {
              window.dispatchEvent(new CustomEvent('mock:stream-chunk', {
                detail: { type: 'content', text: chunk }
              }));
              await new Promise(r => setTimeout(r, delay));
            }

            // Mock tool call results
            if (toolCalls) {
              for (const tc of toolCalls) {
                window.dispatchEvent(new CustomEvent('mock:tool-result', {
                  detail: { name: tc.name, result: tc.result }
                }));
                await new Promise(r => setTimeout(r, 50));
              }
            }

            // Signal completion
            window.dispatchEvent(new CustomEvent('mock:stream-complete'));
          },
          { content, thinking, toolCalls, delay }
        );
      },
    };

    await use(tauriApp);

    // Cleanup
    await page.close();
  }, { scope: 'worker' as const }], // 整个 worker 共享一个 app 实例
});

export { expect } from '@playwright/test';
