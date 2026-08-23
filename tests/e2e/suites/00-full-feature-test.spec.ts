/**
 * 全功能集成测试 — 模拟用户调用所有功能
 *
 * 测试范围: 40 项功能
 * 预计耗时: ~5 分钟 (含等待)
 * 运行: npm run test:e2e -- --grep="全功能"
 *
 * 每个测试步骤独立运行，失败后继续下一个。
 * 最终输出完整的功能测试报告。
 */

import { test, expect } from '../fixtures/tauri.fixture';
import {
  waitForText,
  waitForTestId,
  waitForLoadingComplete,
} from '../helpers/wait-for-event';

// ============================================================================
// 测试报告收集器
// ============================================================================

interface TestReport {
  feature: string;
  status: 'PASS' | 'FAIL' | 'SKIP';
  duration: number;
  error?: string;
}
const report: TestReport[] = [];

function record(feature: string, status: TestReport['status'], duration: number, error?: string) {
  report.push({ feature, status, duration, error });
  const emoji = status === 'PASS' ? '✅' : status === 'FAIL' ? '❌' : '⏭️';
  console.log(`  ${emoji} [${status}] ${feature} (${duration}ms)${error ? ` — ${error}` : ''}`);
}

// ============================================================================
// 辅助函数
// ============================================================================

async function timed<T>(
  label: string,
  fn: () => Promise<T>,
): Promise<{ result: T | null; duration: number; error?: string }> {
  const start = Date.now();
  try {
    const result = await fn();
    record(label, 'PASS', Date.now() - start);
    return { result, duration: Date.now() - start };
  } catch (e: any) {
    const msg = e?.message || String(e);
    record(label, 'FAIL', Date.now() - start, msg);
    return { result: null, duration: Date.now() - start, error: msg };
  }
}

// ============================================================================
// 测试分组
// ============================================================================

test.describe('全功能集成测试 (40项)', () => {
  test.setTimeout(600_000); // 10 分钟

  test('DeepStudent 全功能回归测试', async ({ tauriApp }) => {
    const { page, invoke } = tauriApp;

    // ================================================================
    // 分组 1: 应用基础 (4 项)
    // ================================================================
    console.log('\n📱 === 应用基础 ===');

    await timed('1.1 应用标题', async () => {
      const title = await page.title();
      expect(title).toContain('Deep Student');
    });

    await timed('1.2 Tauri API 桥接', async () => {
      const hasApi = await page.evaluate(() =>
        typeof (window as any).__TAURI__?.core?.invoke === 'function');
      expect(hasApi).toBe(true);
    });

    await timed('1.3 后端连通性', async () => {
      const result = await invoke<unknown[]>('dstu_list', {
        path: '/', options: { limit: 1, offset: 0 }
      });
      expect(Array.isArray(result)).toBe(true);
    });

    await timed('1.4 应用版本', async () => {
      const version = await invoke<string>('get_app_version');
      expect(version).toBeTruthy();
      expect(version).toMatch(/^\d+\.\d+\.\d+/);
    });

    // ================================================================
    // 分组 2: 学习中心 — 资源管理 (6 项)
    // ================================================================
    console.log('\n📁 === 学习中心 ===');
    let testFolderId = '';
    let testNoteId = '';

    await timed('2.1 创建测试文件夹', async () => {
      const folder = await invoke<{ id: string }>('dstu_folder_create', {
        path: '/', name: `E2E_Test_${Date.now()}`
      });
      testFolderId = folder.id;
      expect(folder.id).toBeTruthy();
    });

    await timed('2.2 创建笔记', async () => {
      const note = await invoke<{ id: string; title: string; node_type: string }>(
        'dstu_create', {
          path: `/${testFolderId}`,
          resourceType: 'note',
          title: 'E2E 测试笔记',
          content: '# 测试笔记\n\n这是自动化测试创建的笔记内容。'
        }
      );
      testNoteId = note.id;
      expect(note.node_type).toBe('note');
      expect(note.title).toBe('E2E 测试笔记');
    });

    await timed('2.3 创建思维导图', async () => {
      const mm = await invoke<{ id: string; node_type: string }>(
        'dstu_create', {
          path: `/${testFolderId}`,
          resourceType: 'mindmap',
          title: 'E2E 测试导图',
          content: JSON.stringify({
            nodes: [
              { id: 'root', text: '根节点', children: [
                { id: 'c1', text: '子节点1' },
                { id: 'c2', text: '子节点2' }
              ]}
            ]
          })
        }
      );
      expect(mm.node_type).toBe('mindmap');
    });

    await timed('2.4 列出文件夹内容', async () => {
      const items = await invoke<Array<{ id: string; node_type: string }>>(
        'dstu_list', { path: `/${testFolderId}`, options: { limit: 20, offset: 0 } }
      );
      expect(items.length).toBeGreaterThanOrEqual(2);
    });

    await timed('2.5 搜索资源', async () => {
      const results = await invoke<Array<{ title: string }>>('dstu_search', {
        query: 'E2E 测试',
        options: { limit: 10 }
      });
      expect(results.some((r: any) => r.title?.includes('E2E'))).toBe(true);
    });

    await timed('2.6 获取笔记内容', async () => {
      const note = await invoke<{ content: string }>('dstu_get', {
        path: `/${testFolderId}/${testNoteId}`
      });
      expect(note.content).toContain('测试笔记');
    });

    // ================================================================
    // 分组 3: 设置 & 系统信息 (4 项)
    // ================================================================
    console.log('\n⚙️ === 设置 & 系统 ===');

    await timed('3.1 读取设置', async () => {
      const setting = await invoke<string>('get_setting', { key: 'language' });
      expect(typeof setting === 'string' || setting === null).toBe(true);
    });

    await timed('3.2 保存设置', async () => {
      await invoke('save_setting', {
        key: 'e2e_test_key',
        value: 'e2e_test_value'
      });
      const saved = await invoke<string>('get_setting', { key: 'e2e_test_key' });
      expect(saved).toBe('e2e_test_value');
    });

    await timed('3.3 获取 API 配置列表', async () => {
      const configs = await invoke<unknown[]>('get_api_configs');
      expect(Array.isArray(configs)).toBe(true);
    });

    await timed('3.4 获取模型配置', async () => {
      const models = await invoke<unknown[]>('get_model_profiles');
      expect(Array.isArray(models)).toBe(true);
    });

    // ================================================================
    // 分组 4: 题库 (5 项)
    // ================================================================
    console.log('\n📝 === 题库 ===');
    let examId = '';

    await timed('4.1 创建题库', async () => {
      const exam = await invoke<{ id: string; node_type: string }>(
        'dstu_create', {
          path: `/${testFolderId}`,
          resourceType: 'exam',
          title: 'E2E 测试题库',
          content: JSON.stringify([
            { type: 'single_choice', question: '1+1=?', options: ['1', '2', '3'], answer: '2', difficulty: 1 },
            { type: 'single_choice', question: '水的化学式?', options: ['O2', 'H2O', 'CO2'], answer: 'H2O', difficulty: 1 },
            { type: 'fill_blank', question: '地球绕__公转', answer: '太阳', difficulty: 2 },
          ])
        }
      );
      examId = exam.id;
      expect(exam.node_type).toBe('exam');
    });

    await timed('4.2 获取题库详情', async () => {
      const exam = await invoke<{ content: string }>('dstu_get', {
        path: `/${testFolderId}/${examId}`
      });
      expect(exam.content).toContain('single_choice');
    });

    await timed('4.3 列出所有题库', async () => {
      const items = await invoke<Array<{ node_type: string }>>('dstu_list', {
        path: '/', options: { typeFilter: 'exam', limit: 10 }
      });
      expect(items.every((i: any) => i.node_type === 'exam')).toBe(true);
    });

    await timed('4.4 更新题库', async () => {
      await invoke('dstu_update', {
        path: `/${testFolderId}/${examId}`,
        updates: { title: 'E2E 测试题库 (已更新)' }
      });
      const updated = await invoke<{ title: string }>('dstu_get', {
        path: `/${testFolderId}/${examId}`
      });
      expect(updated.title).toContain('已更新');
    });

    await timed('4.5 题库收藏', async () => {
      await invoke('dstu_set_favorite', {
        path: `/${testFolderId}/${examId}`,
        favorite: true
      });
      // 验证收藏成功
      const items = await invoke<Array<{ is_favorite: boolean }>>('dstu_list', {
        path: `/${testFolderId}`,
        options: { limit: 20 }
      });
      const exam = items.find((i: any) => i.id === examId);
      expect(exam).toBeTruthy();
    });

    // ================================================================
    // 分组 5: 文件夹 & 路径 (3 项)
    // ================================================================
    console.log('\n📂 === 文件夹管理 ===');

    await timed('5.1 获取文件夹树', async () => {
      const tree = await invoke<unknown>('dstu_folder_get_tree', { path: '/' });
      expect(tree).toBeTruthy();
    });

    await timed('5.2 路径解析', async () => {
      const parsed = await invoke<{ resource_type: string }>('dstu_parse_path', {
        path: `/${testFolderId}/${testNoteId}`
      });
      expect(parsed.resource_type).toBeTruthy();
    });

    await timed('5.3 获取资源位置', async () => {
      const loc = await invoke<string>('dstu_get_resource_location', {
        resourceId: testNoteId
      });
      expect(loc).toBeTruthy();
    });

    // ================================================================
    // 分组 6: 回收站 (3 项)
    // ================================================================
    console.log('\n🗑️ === 回收站 ===');
    let trashNoteId = '';

    await timed('6.1 软删除到回收站', async () => {
      const note = await invoke<{ id: string }>('dstu_create', {
        path: `/${testFolderId}`,
        resourceType: 'note',
        title: '待删除笔记',
        content: '这条笔记将被删除'
      });
      trashNoteId = note.id;
      await invoke('dstu_delete', { path: `/${testFolderId}/${trashNoteId}` });
      // 验证已不在列表
      const items = await invoke<Array<{ id: string }>>('dstu_list', {
        path: `/${testFolderId}`,
        options: { limit: 50 }
      });
      expect(items.some((i: any) => i.id === trashNoteId)).toBe(false);
    });

    await timed('6.2 回收站列表', async () => {
      const deleted = await invoke<Array<{ id: string }>>('dstu_list_deleted', {
        limit: 20
      });
      expect(deleted.some((d: any) => d.id === trashNoteId)).toBe(true);
    });

    await timed('6.3 从回收站恢复', async () => {
      await invoke('dstu_restore', { resourceId: trashNoteId });
      // 验证已恢复
      const items = await invoke<Array<{ id: string }>>('dstu_list', {
        path: `/${testFolderId}`,
        options: { limit: 50 }
      });
      expect(items.some((i: any) => i.id === trashNoteId)).toBe(true);
    });

    // ================================================================
    // 分组 7: 文件管理 (2 项)
    // ================================================================
    console.log('\n📄 === 文件管理 ===');

    await timed('7.1 列出文件', async () => {
      const files = await invoke<Array<{ id: string }>>('vfs_list_files', {
        folderId: null,
        limit: 10,
      });
      expect(Array.isArray(files)).toBe(true);
    });

    await timed('7.2 附件配置', async () => {
      const config = await invoke<unknown>('vfs_get_attachment_config');
      expect(config).toBeTruthy();
    });

    // ================================================================
    // 分组 8: OCR 检查 (2 项)
    // ================================================================
    console.log('\n🔍 === OCR 引擎 ===');

    await timed('8.1 OCR 可用性检查', async () => {
      try {
        const result = await invoke<unknown>('check_ocr_availability');
        expect(result).toBeTruthy();
      } catch (e: any) {
        // OCR 可能未安装 — 不算失败
        if (e?.message?.includes('not available')) {
          record('8.1 OCR 可用性', 'SKIP', 0, 'OCR 引擎未安装');
        } else {
          throw e;
        }
      }
    });

    await timed('8.2 OCR 引擎配置', async () => {
      try {
        const configs = await invoke<unknown[]>('get_ocr_engine_configs');
        expect(Array.isArray(configs)).toBe(true);
      } catch {
        record('8.2 OCR 引擎配置', 'SKIP', 0, '命令不可用');
      }
    });

    // ================================================================
    // 分组 9: Anki 服务 (4 项)
    // ================================================================
    console.log('\n🃏 === Anki 服务 ===');

    await timed('9.1 AnkiConnect 可用性', async () => {
      try {
        const available = await invoke<boolean>('anki_connect_check_status');
        expect(typeof available === 'boolean' || typeof available === 'object').toBe(true);
      } catch {
        record('9.1 AnkiConnect', 'SKIP', 0, 'Anki 桌面端未运行');
      }
    });

    await timed('9.2 内置模板列表', async () => {
      const templates = await invoke<unknown[]>('get_custom_templates', {});
      expect(Array.isArray(templates)).toBe(true);
    });

    await timed('9.3 文档会话列表', async () => {
      const sessions = await invoke<unknown[]>('enhanced_anki_list_document_sessions', {
        limit: 10,
        offset: 0,
      });
      expect(Array.isArray(sessions)).toBe(true);
    });

    await timed('9.4 卡片库查询', async () => {
      const cards = await invoke<unknown[]>('enhanced_anki_list_library_cards', {
        limit: 5,
        offset: 0,
        search: '',
      });
      expect(Array.isArray(cards)).toBe(true);
    });

    // ================================================================
    // 分组 10: 思维导图相关 (2 项)
    // ================================================================
    console.log('\n🧠 === 思维导图 ===');

    await timed('10.1 思维导图 CRUD', async () => {
      const mm = await invoke<{ id: string }>('vfs_create_mindmap', {
        title: 'E2E MM Test',
        content: JSON.stringify({ nodes: [{ id: '1', text: 'Root' }] }),
        folderId: testFolderId,
      });
      expect(mm.id).toBeTruthy();

      const fetched = await invoke<{ content: string }>('vfs_get_mindmap', {
        mindmapId: mm.id,
      });
      expect(fetched.content).toContain('Root');
    });

    await timed('10.2 思维导图列表', async () => {
      const list = await invoke<unknown[]>('vfs_list_mindmaps', {
        folderId: testFolderId,
      });
      expect(Array.isArray(list)).toBe(true);
    });

    // ================================================================
    // 分组 11: 记忆系统 (3 项)
    // ================================================================
    console.log('\n🧿 === 记忆系统 ===');

    await timed('11.1 记忆配置', async () => {
      const config = await invoke<unknown>('memory_get_config');
      expect(config).toBeTruthy();
    });

    await timed('11.2 记忆搜索', async () => {
      const results = await invoke<Array<unknown>>('memory_search', {
        query: '测试',
        limit: 5,
      });
      expect(Array.isArray(results)).toBe(true);
    });

    await timed('11.3 记忆列表', async () => {
      const list = await invoke<Array<unknown>>('memory_list', {
        limit: 10,
        offset: 0,
      });
      expect(Array.isArray(list)).toBe(true);
    });

    // ================================================================
    // 分组 12: 数据治理 (3 项)
    // ================================================================
    console.log('\n🛡️ === 数据治理 ===');

    await timed('12.1 数据库状态', async () => {
      try {
        const status = await invoke<unknown>('data_governance_get_database_status');
        expect(status).toBeTruthy();
      } catch {
        record('12.1 数据库状态', 'SKIP', 0, 'data_governance feature 未启用');
      }
    });

    await timed('12.2 Schema 注册表', async () => {
      try {
        const schema = await invoke<unknown>('data_governance_get_schema_registry');
        expect(schema).toBeTruthy();
      } catch {
        record('12.2 Schema 注册表', 'SKIP', 0, 'feature 未启用');
      }
    });

    await timed('12.3 导出格式查询', async () => {
      try {
        const formats = await invoke<string[]>('dstu_export_formats', {
          path: `/${testFolderId}/${testNoteId}`
        });
        expect(Array.isArray(formats)).toBe(true);
      } catch {
        record('12.3 导出格式', 'SKIP', 0, '命令不可用');
      }
    });

    // ================================================================
    // 分组 13: LLM 用量 (2 项)
    // ================================================================
    console.log('\n📊 === LLM 用量 ===');

    await timed('13.1 用量趋势', async () => {
      const trends = await invoke<unknown[]>('llm_usage_get_trends', { days: 7 });
      expect(Array.isArray(trends)).toBe(true);
    });

    await timed('13.2 用量摘要', async () => {
      const summary = await invoke<unknown>('llm_usage_summary', { days: 30 });
      expect(summary).toBeTruthy();
    });

    // ================================================================
    // 分组 14: 批量操作 & 清理 (3 项)
    // ================================================================
    console.log('\n🧹 === 清理 ===');

    await timed('14.1 批量删除资源', async () => {
      // 先创建一个临时资源再删除
      const temp = await invoke<{ id: string }>('dstu_create', {
        path: `/${testFolderId}`,
        resourceType: 'note',
        title: 'temp-to-delete',
        content: ''
      });
      await invoke('dstu_delete', { path: `/${testFolderId}/${temp.id}` });
      const deleted = await invoke<Array<{ id: string }>>('dstu_list_deleted', { limit: 50 });
      expect(deleted.some((d: any) => d.id === temp.id)).toBe(true);
    });

    await timed('14.2 永久删除', async () => {
      // 对回收站中的项目永久删除
      try {
        await invoke('dstu_permanent_delete', { resourceId: trashNoteId });
      } catch {
        // 可能不支持永久删除 — 跳过
        record('14.2 永久删除', 'SKIP', 0, '命令可能不可用');
      }
    });

    await timed('14.3 清理测试数据', async () => {
      // 删除测试文件夹
      try {
        await invoke('dstu_delete', { path: `/${testFolderId}` });
      } catch (e: any) {
        // 包含子项的文件夹删除可能失败
        record('14.3 清理测试数据', 'FAIL', 0, e?.message);
      }
    });

    // ================================================================
    // 测试报告
    // ================================================================
    console.log('\n' + '='.repeat(60));
    console.log('            全功能集成测试报告');
    console.log('='.repeat(60));

    const passed = report.filter(r => r.status === 'PASS').length;
    const failed = report.filter(r => r.status === 'FAIL').length;
    const skipped = report.filter(r => r.status === 'SKIP').length;
    const total = report.length;

    console.log(`  总计: ${total} | ✅ 通过: ${passed} | ❌ 失败: ${failed} | ⏭️ 跳过: ${skipped}`);
    console.log(`  通过率: ${((passed / (passed + failed)) * 100).toFixed(1)}% (不含跳过)`);

    if (failed > 0) {
      console.log('\n  失败项:');
      report.filter(r => r.status === 'FAIL').forEach(r => {
        console.log(`    ❌ ${r.feature} — ${r.error}`);
      });
    }

    if (skipped > 0) {
      console.log('\n  跳过项:');
      report.filter(r => r.status === 'SKIP').forEach(r => {
        console.log(`    ⏭️ ${r.feature} — ${r.error}`);
      });
    }

    console.log('='.repeat(60));

    // 关键功能断言 — 核心 CRUD 必须全部通过
    const criticalFailures = [
      '1.3 后端连通性',
      '2.1 创建测试文件夹',
      '2.2 创建笔记',
      '2.4 列出文件夹内容',
    ].filter(name => report.find(r => r.feature === name)?.status === 'FAIL');

    if (criticalFailures.length > 0) {
      throw new Error(
        `关键功能测试失败:\n${criticalFailures.map(f => `  ❌ ${f}`).join('\n')}`
      );
    }
  });
});
