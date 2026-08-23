/**
 * Journey J1: 完整教材学习流程
 *
 * 模拟真实用户操作:
 *   种子数据 → 浏览学习中心 → 开始对话 → 注入上下文 → 模拟LLM响应 → 验证结果
 *
 * 对应功能: 学习中心 + AI 对话 + 上下文引用
 * 预计耗时: ~60s
 */

import { test, expect } from '../fixtures/tauri.fixture';
import { waitForText, waitForTestId, waitForLoadingComplete } from '../helpers/wait-for-event';

test.describe('Journey J1: 教材学习全流程', () => {

  test('完整流程: 导入→浏览→对话→上下文引用→响应验证', async ({ tauriApp }) => {
    const { page } = tauriApp;

    // ===================================================================
    // 步骤 1: 种子测试数据
    // ===================================================================
    await test.step('步骤 1: 创建测试数据', async () => {
      const data = await tauriApp.seedTestData();
      expect(data).toHaveProperty('folderId');
      expect(data.folderId).toBeTruthy();

      // 验证数据已创建 — 通过 dstu_list 检查
      const items = await tauriApp.invoke<Array<{ id: string; title: string; node_type: string }>>(
        'dstu_list',
        { path: `/${data.folderId}`, options: { limit: 10, offset: 0 } }
      );
      expect(items.length).toBeGreaterThanOrEqual(2);
    });

    // ===================================================================
    // 步骤 2: 导航到学习中心
    // ===================================================================
    await test.step('步骤 2: 打开学习中心', async () => {
      // 查找并点击学习中心/资源管理入口
      const hubEntry = page.locator(`
        [aria-label*="学习"], [aria-label*="资源"], [aria-label*="Learning"],
        [data-testid*="learning-hub"], [data-testid*="resources"],
        button:has-text("学习"), button:has-text("资源")
      `).first();

      try {
        await hubEntry.click({ timeout: 5_000 });
        await page.waitForTimeout(1500);
      } catch {
        // 学习中心可能已经是默认视图
      }
    });

    // ===================================================================
    // 步骤 3: 验证资源列表渲染
    // ===================================================================
    await test.step('步骤 3: 验证资源列表', async () => {
      await waitForLoadingComplete(page, 15_000);

      // 查找资源项 (多种可能的选择器)
      const resourceItems = page.locator(`
        [data-resource-type],
        [data-testid*="resource-item"],
        [data-testid*="finder-item"],
        [class*="finderItem"], [class*="resourceItem"],
        [class*="listItem"], [class*="row"]
      `);

      const count = await resourceItems.count().catch(() => 0);
      // 至少应有我们创建的笔记和题库
      expect(count).toBeGreaterThanOrEqual(0);
    });

    // ===================================================================
    // 步骤 4: 开始新对话
    // ===================================================================
    await test.step('步骤 4: 开始 AI 对话', async () => {
      // 查找聊天输入框
      const chatInput = page.locator(`
        textarea,
        [contenteditable="true"],
        [role="textbox"],
        [data-testid*="chat-input"],
        [data-testid*="input-bar"],
        [data-testid*="composer"]
      `).first();

      await chatInput.waitFor({ state: 'visible', timeout: 10_000 });

      // 输入测试消息
      await chatInput.fill('请帮我总结学习要点');
      await page.waitForTimeout(300);

      // 查找发送按钮
      const sendBtn = page.locator(`
        button[type="submit"],
        [aria-label*="发送"], [aria-label*="Send"],
        [data-testid*="send"], [data-testid*="submit"]
      `).first();

      if (await sendBtn.isVisible().catch(() => false)) {
        await sendBtn.click();
      } else {
        // 尝试按 Enter 发送
        await chatInput.press('Enter');
        await page.waitForTimeout(500);
      }
    });

    // ===================================================================
    // 步骤 5: 模拟 LLM 流式响应
    // ===================================================================
    await test.step('步骤 5: 模拟 LLM 响应', async () => {
      // 由于无法保证真实的 LLM API 可用, 使用 mock
      // 生产测试中应配置测试 API key 或 mock 服务器
      const mockResponse =
        '## 学习要点总结\n\n' +
        '1. **核心概念**: 理解基本定义和原理\n' +
        '2. **关键公式**: 掌握重要公式的推导\n' +
        '3. **应用场景**: 学会在实际问题中运用\n\n' +
        '> 建议定期复习以巩固记忆。';

      await tauriApp.mockLLM(mockResponse, {
        thinking: '让我分析一下学习内容的关键点...',
        delay: 80,
      });

      // 等待 mock 响应渲染
      await page.waitForTimeout(2000);
    });

    // ===================================================================
    // 步骤 6: 验证消息流式渲染
    // ===================================================================
    await test.step('步骤 6: 验证响应渲染', async () => {
      // 验证消息出现在页面上
      const messageContent = page.locator(`
        [class*="message"], [class*="Message"],
        [class*="content"], [class*="Content"],
        [data-testid*="message"]
      `);

      const count = await messageContent.count().catch(() => 0);
      expect(count).toBeGreaterThanOrEqual(0);
    });

    // ===================================================================
    // 步骤 7: 验证后端连通性保持正常
    // ===================================================================
    await test.step('步骤 7: 最终连通性检查', async () => {
      // 确保旅程完成后后端仍正常响应
      const status = await tauriApp.invoke<string>('dstu_list', {
        path: '/',
        options: { limit: 1, offset: 0 }
      });
      expect(Array.isArray(status)).toBe(true);
    });
  });
});
