/**
 * 事件等待辅助函数
 * 用于在 E2E 测试中等待特定的 Tauri 事件
 */

import type { Page } from '@playwright/test';

/** 等待 Tauri emit 的特定事件 */
export async function waitForTauriEvent(
  page: Page,
  eventName: string,
  timeoutMs = 30_000
): Promise<unknown> {
  return page.evaluate(
    ({ eventName, timeoutMs }) => {
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          reject(new Error(`Timeout waiting for event: ${eventName} (${timeoutMs}ms)`));
        }, timeoutMs);

        const { listen } = (window as any).__TAURI__?.event || {};
        if (!listen) {
          clearTimeout(timer);
          reject(new Error('Tauri event API not available'));
          return;
        }

        // Use a flag to prevent multiple resolves
        let resolved = false;
        listen(eventName, (event: unknown) => {
          if (!resolved) {
            resolved = true;
            clearTimeout(timer);
            resolve(event);
          }
        }).catch(reject);
      });
    },
    { eventName, timeoutMs }
  );
}

/** 等待 DOM 中出现指定文本 */
export async function waitForText(
  page: Page,
  text: string,
  timeoutMs = 10_000
): Promise<void> {
  await page.waitForFunction(
    (text) => document.body.innerText.includes(text),
    text,
    { timeout: timeoutMs }
  );
}

/** 等待指定 data-testid 元素可见 */
export async function waitForTestId(
  page: Page,
  testId: string,
  timeoutMs = 10_000
): Promise<void> {
  await page.waitForSelector(`[data-testid="${testId}"]`, {
    state: 'visible',
    timeout: timeoutMs,
  });
}

/** 等待加载状态消失 (spinner/loading 文字) */
export async function waitForLoadingComplete(
  page: Page,
  timeoutMs = 30_000
): Promise<void> {
  try {
    await page.waitForFunction(
      () => {
        const spinners = document.querySelectorAll(
          '[class*="spinner"], [class*="Spinner"], [class*="loading"], [class*="Loading"]'
        );
        const loadingTexts = document.body.innerText.match(/加载中|loading|Loading\.\.\./i);
        return spinners.length === 0 && !loadingTexts;
      },
      { timeout: timeoutMs }
    );
  } catch {
    // 超时不失败 — 有些页面可能持续显示加载指示器
  }
}

/** 截图并附加时间戳 (调试用) */
export async function debugScreenshot(page: Page, label: string): Promise<void> {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  await page.screenshot({
    path: `test-results/debug-${label}-${timestamp}.png`,
    fullPage: true,
  });
}
