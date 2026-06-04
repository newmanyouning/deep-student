/**
 * Generate individual API refactor reports for each module.
 */

import { DatabaseSync } from 'node:sqlite';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = 'C:/deep-student';
const DB = join(ROOT, '.planning/dependency-db/deps.db');
const OUT = join(ROOT, '.planning/exploration/dependency-db/reports/api-refactor');
const DATADIR = join(OUT, '_data');

const db = new DatabaseSync(DB);

// ── Module-specific error types ──────────────────────────
const ERROR_TYPES = {
  chat_v2: 'ChatV2Error',
  vfs: 'VfsError',
  dstu: 'DstuError',
  memory: 'MemoryError',
  data_governance: 'DataGovernanceError',
  cmd__notes: 'NotesError',
  cmd__enhanced_anki: 'AnkiError',
  essay_grading: 'EssayGradingError',
  review_plan_service: 'ReviewPlanError',
  cloud_storage: 'CloudStorageError',
  cmd__web_search: 'SearchError',
  cmd__ocr: 'OcrError',
  cmd__mcp: 'McpError',
  cmd__anki_connect: 'AnkiConnectError',
  cmd__textbooks: 'TextbookError',
  data_space: 'DataSpaceError',
  question_sync_service: 'SyncError',
  translation: 'TranslationError',
  qbank_grading: 'QbankGradingError',
  llm_usage: 'UsageError',
  tts: 'TtsError',
  commands: 'AppError',
};

// ── Module descriptions ──────────────────────────────────
const MODULE_INFO = {
  commands: {
    title: 'commands.rs — 遗留命令文件',
    issue: '这是项目的旧版命令集合，与 cmd/ 目录并存。137 个命令中许多已被 cmd/ 子模块接管（通过 pub use re-export），剩余的命令应迁移到对应子模块然后退役此文件。',
    action: '退役计划：将剩余命令迁移到 cmd/ 子模块，commands.rs 保留为兼容 re-export 层（或直接删除）',
  },
  vfs: {
    title: 'VFS — 虚拟文件系统',
    issue: '119 个命令混合了 5 种不同的职责域：文件CRUD、PDF处理、番茄钟(Pomodoro)、待办(Todo)、语音输入(Voice)。Pomodoro/Todo/Voice 不应属于 VFS 模块。',
    action: '拆分为 VFS Core + Pomodoro + Todo + Voice Input 四个独立模块',
  },
  chat_v2: {
    title: 'Chat V2 — 对话引擎',
    issue: '78 个命令，返回类型用 Result<T, String> 而非统一的错误类型。部分命令（anki_cards_result, canvas_edit_result）通过 AppHandle 发事件而非返回值。',
    action: '统一错误类型为 ChatV2Error，将事件驱动命令改为返回值模式',
  },
  dstu: {
    title: 'DSTU — 资源协议',
    issue: '54 个命令，命名一致性最好（全部 dstu_ 前缀）。但错误类型用 String。',
    action: '统一错误类型为 DstuError，优化参数封装',
  },
  data_governance: {
    title: 'Data Governance — 数据治理',
    issue: '43 个命令，覆盖备份/恢复/同步/审计/迁移 5 个子域。返回类型用 String。',
    action: '统一错误类型为 DataGovernanceError，按子域组织命令',
  },
  memory: {
    title: 'Memory — 智能记忆',
    issue: '27 个命令，每个命令都需要 3 个 State 参数 (VfsDatabase + VfsLanceStore + LLMManager)。应该封装为 MemoryContext。',
    action: '引入 MemoryContext 封装 3 个 State，统一错误类型为 MemoryError',
  },
  'cmd__notes': {
    title: 'cmd::notes — 笔记命令',
    issue: '39 个命令，部分用 canvas_note_ 前缀（旧白板功能），部分用 notes_ 前缀。',
    action: '统一为 notes_ 前缀，移除废弃的 canvas_note_ 命令',
  },
  review_plan_service: {
    title: 'Review Plan — 复习计划',
    issue: '17 个命令，命名一致（review_plan_ 前缀），但返回类型用 String。每个命令都需要 VfsDatabase State。',
    action: '统一错误类型为 ReviewPlanError',
  },
  cloud_storage: {
    title: 'Cloud Storage — 云存储',
    issue: '14 个命令，每个命令都重复 CloudStorageConfig 参数。应封装为 State。',
    action: 'CloudStorageConfig 封装为 State，统一错误类型',
  },
  'cmd__enhanced_anki': {
    title: 'cmd::enhanced_anki — Anki 制卡',
    issue: '22 个命令，命名不一致（delete_anki_card vs generate_anki_cards_from_document）。',
    action: '统一为 anki_ 前缀，统一错误类型',
  },
  essay_grading: {
    title: 'Essay Grading — 作文批改',
    issue: '20 个命令，返回类型直接用具体类型而非 Result 包裹。',
    action: '全部包装为 Result<T, EssayGradingError>',
  },
  'cmd__web_search': {
    title: 'cmd::web_search — 搜索与设置',
    issue: '17 个命令，混合了搜索引擎命令和通用设置命令（get_setting, delete_setting）。',
    action: '拆分：搜索命令保留，通用设置命令移到独立 settings 模块',
  },
  'cmd__ocr': { title: 'cmd::ocr — OCR 引擎', issue: '14 个命令，命名一致', action: '统一错误类型' },
  'cmd__mcp': { title: 'cmd::mcp — MCP 协议', issue: '13 个命令', action: '统一错误类型' },
  'cmd__anki_connect': { title: 'cmd::anki_connect — Anki Connect', issue: '13 个命令', action: '统一错误类型' },
  'cmd__textbooks': { title: 'cmd::textbooks — 教材', issue: '11 个命令', action: '统一错误类型' },
  data_space: { title: 'Data Space — 数据空间 (A/B 槽位)', issue: '10 个命令', action: '统一错误类型' },
  debug_commands: { title: 'Debug Commands — 调试命令', issue: '7 个命令，生产代码含调试', action: '标记为 dev-only' },
  question_sync_service: { title: 'Question Sync — 题目同步', issue: '6 个命令', action: '统一错误类型' },
  backup_config: { title: 'Backup Config — 备份配置', issue: '5 个命令', action: '统一错误类型' },
  secure_store: { title: 'Secure Store — 安全存储', issue: '4 个命令', action: '保持现状' },
  tts: { title: 'TTS — 文本转语音', issue: '3 个命令', action: '保持现状' },
  translation: { title: 'Translation — 翻译', issue: '3 个命令', action: '统一错误类型' },
  'cmd__anki_cards': { title: 'cmd::anki_cards — Anki 卡片', issue: '3 个命令', action: '合并到 enhanced_anki' },
  qbank_grading: { title: 'QBank Grading — 题库评分', issue: '2 个命令', action: '统一错误类型' },
  llm_usage: { title: 'LLM Usage — 用量统计', issue: '2 个命令', action: '保持现状' },
  config_recovery: { title: 'Config Recovery — 配置恢复', issue: '2 个命令', action: '保持现状' },
  pdfium_utils: { title: 'Pdfium Utils — PDF引擎工具', issue: '1 个命令', action: '保持现状' },
  debug_logger: { title: 'Debug Logger — 调试日志', issue: '1 个命令', action: '标记为 dev-only' },
  'cmd__translation': { title: 'cmd::translation — 翻译命令', issue: '1 个命令', action: '合并到 translation 模块' },
  anki_connect_service: { title: 'Anki Connect Service', issue: '1 个命令', action: '合并到 cmd::anki_connect' },
};

// ── Generate reports ─────────────────────────────────────
const modules = db.prepare('SELECT module, COUNT(*) as cnt FROM api_functions GROUP BY module ORDER BY cnt DESC').all();

for (const mod of modules) {
  const safeName = mod.module.replace(/::/g, '__');
  const dataFile = join(DATADIR, safeName + '.json');
  let funcs;
  try { funcs = JSON.parse(readFileSync(dataFile, 'utf-8')); } catch { continue; }

  const info = MODULE_INFO[mod.module] || MODULE_INFO[safeName] || { title: mod.module, issue: '', action: '统一错误类型' };
  const errType = ERROR_TYPES[mod.module] || ERROR_TYPES[safeName] || 'AppError';

  // Build report
  let report = '';
  report += `# API 重构: ${info.title}\n\n`;
  report += `**日期**: 2026-05-29 | **命令数**: ${mod.cnt} | **对应诊断**: round-20~26\n\n`;
  report += `---\n\n`;
  report += `## 当前问题\n\n${info.issue}\n\n`;

  // 参数统计
  const paramTypes = {};
  const returnTypes = {};
  for (const f of funcs) {
    for (const p of f.params) {
      const type = p.includes(':') ? p.split(':').slice(1).join(':').trim() : 'unknown';
      const simple = type.replace(/State<'_,?\s*Arc<([^>]+)>>/g, 'State<$1>').replace(/State<'_,\s*([^>]+)>/g, 'State<$1>').replace(/tauri::/g, '').replace(/crate::/g, '');
      paramTypes[simple] = (paramTypes[simple] || 0) + 1;
    }
    const rt = f.returns.replace(/Result<(.+?),.*/, '$1').replace(/->\s*/, '').trim();
    returnTypes[rt] = (returnTypes[rt] || 0) + 1;
  }

  report += `## 当前参数模式\n\n`;
  const topParams = Object.entries(paramTypes).sort((a,b) => b[1]-a[1]).slice(0, 6);
  report += `| 参数类型 | 出现次数 |\n|---------|--------|\n`;
  for (const [t, c] of topParams) report += `| \`${t}\` | ${c} |\n`;

  report += `\n## 当前返回类型\n\n`;
  const topReturns = Object.entries(returnTypes).sort((a,b) => b[1]-a[1]).slice(0, 6);
  report += `| 返回类型 | 出现次数 |\n|---------|--------|\n`;
  for (const [t, c] of topReturns) report += `| \`${t}\` | ${c} |\n`;

  report += `\n## 命令清单与变更\n\n`;
  report += `| 当前命令 | 改为 | 参数变更 | 返回变更 |\n|---------|------|---------|--------|\n`;

  for (const f of funcs) {
    const newName = normalizeName(f.name, mod.module);
    const paramChange = describeParamChange(f.params, mod.module);
    const returnChange = describeReturnChange(f.returns, errType);

    const same = newName === f.name && paramChange === '' && returnChange === '';
    if (same) {
      report += `| \`${f.name}\` | *(保持)* | — | — |\n`;
    } else {
      // For space, show only if changed
      if (newName !== f.name) {
        report += `| \`${f.name}\` | \`${newName}\` | ${paramChange || '—'} | ${returnChange || '—'} |\n`;
      } else if (paramChange || returnChange) {
        report += `| \`${f.name}\` | *(保持)* | ${paramChange || '—'} | ${returnChange || '—'} |\n`;
      } else {
        report += `| \`${f.name}\` | *(保持)* | — | — |\n`;
      }
    }
  }

  report += `\n## 改进操作\n\n${info.action}\n\n`;
  report += `## 统一错误类型\n\n\`${errType}\` — 替换当前使用的 \`String\` / \`AppError\`\n\n`;
  report += `---\n*此报告由 deps.db 数据自动生成，对应模块原始数据见 \`_data/${safeName}.json\`*\n`;

  writeFileSync(join(OUT, `${safeName}.md`), report);
  console.log(`  ${safeName}.md (${mod.cnt} commands)`);
}

console.log(`\nGenerated ${modules.length} reports in api-refactor/`);
db.close();

// ── Helper functions ─────────────────────────────────────
function normalizeName(name, module) {
  // commands.rs → 分配到对应子模块
  if (module === 'commands') {
    const map = {
      'chat_v2': /^chat_v2_/,
      'anki': /^(generate_anki|save_anki|check_anki|get_anki|create_anki|add_cards|import_anki|export_cards|call_llm_for_boundary)/,
      'qbank': /^(qbank_|import_question|export_question|get_csv|resume_question|list_importing|pin_images|unpin_images)/,
      'settings': /^(save_setting|get_setting|delete_setting|get_settings_by_prefix|delete_settings_by_prefix|save_api_config|get_api_config|get_model_assign|save_model_assign|get_vendor|save_vendor|get_model_profiles|save_model_profiles|test_api_connection|get_model_adapter|save_model_adapter|reset_model_adapter|estimate_tokens)/,
      'ocr': /^(get_ocr|set_ocr|infer_ocr|validate_ocr|test_ocr|update_ocr|add_ocr|remove_ocr|save_available_ocr|get_available_ocr)/,
      'exam': /^(list_exam|get_exam|update_exam|rename_exam|inspect_pdf|import_question_bank)/,
      'vfs': /^(process_pdf|init_pdf|upload_pdf|cancel_pdf|pause_pdf|resume_pdf|skip_pdf|start_pdf|save_pdf|get_pdf)/,
      'debug': /^(optimize_chat|create_performance|analyze_query|clear_message)/,
      'search': /^(test_search|get_image_as_base64|get_security|get_cn_whitelist|detect_tool|get_tools_namespace|get_provider_strategies|save_provider_strategies|get_feature_flags|update_feature|is_feature|get_injection|simulate_budget)/,
    };
    for (const [target, regex] of Object.entries(map)) {
      if (regex.test(name)) return target + '__' + name;
    }
  }
  return name;
}

function describeParamChange(params, module) {
  // Detect String-heavy params that should be structs
  if (params.length >= 4) return '→ Input struct';
  // Detect State redundancy
  const states = params.filter(p => p.includes('State<'));
  if (states.length >= 3) return '→ Context struct';
  // Detect missing State
  if (module === 'cloud_storage' && params.some(p => p.includes('CloudStorageConfig')))
    return '→ State<CloudConfig>';
  return '';
}

function describeReturnChange(ret, errType) {
  if (ret.includes('Result<') && ret.includes(', String>'))
    return `Result<T, ${errType}>`;
  if (!ret.includes('Result<') && ret !== '()')
    return `Result<${ret}, ${errType}>`;
  return '';
}
