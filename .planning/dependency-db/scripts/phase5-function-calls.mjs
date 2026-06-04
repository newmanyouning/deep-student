/**
 * Phase 5: 函数级调用图 — 关键模块深度扫描
 * 先聚焦: TauriAdapter, ChatStore, VFS handlers
 */

import { DatabaseSync } from 'node:sqlite';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = 'C:/deep-student';
const DB_PATH = join(ROOT, '.planning/dependency-db/deps.db');
const db = new DatabaseSync(DB_PATH);

// 清空旧数据
db.exec('DELETE FROM calls');

const insertCall = db.prepare(`
  INSERT INTO calls (caller_file_id, callee_name, callee_file_id, call_site_line, kind)
  VALUES (?, ?, ?, ?, ?)
`);

// ── 类型定义: 已知的全局函数/组件/Hook 清单 ─────────────
// 从 exports 表构建符号 → 文件 ID 映射
console.log('Building symbol map...');
const symbolMap = new Map(); // callee_name → [{fileId, kind}]
const allExports = db.prepare(`
  SELECT e.name, e.kind, e.file_id, f.path
  FROM exports e JOIN files f ON e.file_id = f.id
  WHERE e.name NOT LIKE '*%' AND e.name NOT LIKE 'reuse%' AND e.name != 'default'
`).all();
for (const exp of allExports) {
  if (!symbolMap.has(exp.name)) symbolMap.set(exp.name, []);
  symbolMap.get(exp.name).push({ fileId: exp.file_id, kind: exp.kind, path: exp.path });
}

// ── TypeScript 函数调用提取 ─────────────────────────────
function extractTSCalls(fileId, filePath) {
  let content;
  try { content = readFileSync(join(ROOT, filePath), 'utf-8'); } catch { return []; }

  const calls = [];
  const lines = content.split('\n');

  // 1. 函数调用: identifier(args) — 跳过关键字和常见内置
  const skipWords = new Set(['if','for','while','switch','catch','return','throw','new','import','export',
    'typeof','instanceof','void','delete','await','yield','console','JSON','Math','Object','Array',
    'String','Number','Boolean','Promise','Error','Map','Set','Date','RegExp','parseInt','parseFloat',
    'setTimeout','setInterval','clearTimeout','clearInterval','require','process','global','window',
    'document','navigator','localStorage','sessionStorage','fetch','alert','confirm','prompt']);
  const skipPre = new Set(['.',':','(','!','+','-','*','/','%','&','|','^','~','?','=','>','<',',','[','{']);

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (line.startsWith('//') || line.startsWith('import ') || line.startsWith('export ')) continue;

    // 函数调用: word( (不在 import/export 中)
    const callRe = /\b([A-Za-z_]\w*)\s*\(/g;
    let m;
    while ((m = callRe.exec(line)) !== null) {
      const name = m[1];
      if (skipWords.has(name)) continue;
      if (name[0] === '_' && name.length < 4) continue; // skip _e, _t etc
      if (name === name.toUpperCase() && name.length > 5) continue; // UPPER_CASE constants

      // 检查前面字符是否属于跳过集合
      const before = line[m.index - 1];
      if (before && skipPre.has(before)) continue;

      const syms = symbolMap.get(name);
      if (!syms || syms.length === 0) continue;

      // 判断类型: use前缀 → hook, 大写开头 → component, 否则 → function
      let kind = 'function_call';
      if (name.startsWith('use') && name[3] === name[3]?.toUpperCase()) kind = 'hook_call';
      else if (name[0] === name[0]?.toUpperCase() && name[0] >= 'A' && name[0] <= 'Z') kind = 'component_or_function';

      // 取第一个匹配的文件
      calls.push({
        callee: name,
        calleeFileId: syms[0].fileId,
        line: i + 1,
        kind,
      });
    }

    // 2. JSX 组件: <ComponentName ...>
    const jsxRe = /<([A-Z]\w*)[\s\/>]/g;
    while ((m = jsxRe.exec(line)) !== null) {
      const name = m[1];
      const syms = symbolMap.get(name);
      if (!syms || syms.length === 0) continue;
      calls.push({
        callee: name,
        calleeFileId: syms[0].fileId,
        line: i + 1,
        kind: 'jsx_component',
      });
    }
  }
  return calls;
}

// ── 选取关键模块 ─────────────────────────────────────────
const targetPatterns = [
  'src/features/chat/adapters/TauriAdapter.ts',
  'src/features/chat/core/store/%',
  'src/features/chat/core/session/%',
  'src/features/learning-hub/stores/%',
  'src/components/anki/cardforge/%',
  'src-tauri/src/chat_v2/%',
  'src-tauri/src/vfs/%',
  'src-tauri/src/llm_manager/%',
];

// 构建目标文件列表
let targetFiles = [];
for (const pattern of targetPatterns) {
  if (pattern.includes('%')) {
    const likePattern = pattern.replace(/%/g, '%');
    const files = db.prepare('SELECT id, path, language, lines FROM files WHERE path LIKE ?').all(likePattern);
    targetFiles.push(...files);
  } else {
    const file = db.prepare('SELECT id, path, language, lines FROM files WHERE path = ?').get(pattern);
    if (file) targetFiles.push(file);
  }
}

// 去重
const seen = new Set();
targetFiles = targetFiles.filter(f => { const k = f.id; if (seen.has(k)) return false; seen.add(k); return true; });
console.log(`Analyzing ${targetFiles.length} key files...`);

// ── 提取调用 ─────────────────────────────────────────────
let totalCalls = 0;
let processed = 0;

db.exec('BEGIN');
for (const file of targetFiles) {
  processed++;
  let calls;
  if (file.language === 'typescript') {
    calls = extractTSCalls(file.id, file.path);
  } else {
    continue; // Rust 调用图暂时跳过
  }

  for (const c of calls) {
    try {
      insertCall.run(file.id, c.callee, c.calleeFileId, c.line, c.kind);
      totalCalls++;
    } catch {}
  }
}
db.exec('COMMIT');

console.log(`Extracted ${totalCalls} function calls from ${processed} TypeScript files.`);

// ── 调用统计 ─────────────────────────────────────────────
console.log('\n=== Most-called functions (top 20) ===');
const topCalled = db.prepare(`
  SELECT c.callee_name, COUNT(*) as cnt, c.kind, f.path as defined_in
  FROM calls c LEFT JOIN files f ON c.callee_file_id = f.id
  GROUP BY c.callee_name ORDER BY cnt DESC LIMIT 25
`).all();
for (const r of topCalled) {
  const file = r.defined_in ? r.defined_in.replace('src/', '') : '(external)';
  console.log(`  ${String(r.cnt).padStart(4)}x  ${r.kind.padEnd(18)} ${r.callee_name.padEnd(30)}  ${file}`);
}

console.log('\n=== Files with most internal function calls ===');
const topCallers = db.prepare(`
  SELECT f.path, f.lines, COUNT(*) as cnt
  FROM calls c JOIN files f ON c.caller_file_id = f.id
  GROUP BY f.id ORDER BY cnt DESC LIMIT 10
`).all();
for (const r of topCallers) {
  console.log(`  ${String(r.cnt).padStart(4)} calls  ${r.path} (${r.lines}l)`);
}

// ── TauriAdapter.ts 调用分析 ──────────────────────────────
console.log('\n=== TauriAdapter.ts — 函数调用分布 ===');
const taId = db.prepare("SELECT id FROM files WHERE path LIKE '%TauriAdapter.ts'").get()?.id;
if (taId) {
  const taCalls = db.prepare(`
    SELECT c.callee_name, f.path as defined_in, COUNT(*) as cnt
    FROM calls c LEFT JOIN files f ON c.callee_file_id = f.id
    WHERE c.caller_file_id = ?
    GROUP BY c.callee_name ORDER BY cnt DESC LIMIT 20
  `).all(taId);
  for (const r of taCalls) {
    const src = r.defined_in ? r.defined_in.replace('src/', '') : '(ext)';
    console.log(`  ${String(r.cnt).padStart(3)}x  ${r.callee_name.padEnd(35)}  ${src}`);
  }

  console.log('\n  TauriAdapter 调用的模块:');
  const taModules = db.prepare(`
    SELECT f.path as module, COUNT(*) as cnt
    FROM calls c JOIN files f ON c.callee_file_id = f.id
    WHERE c.caller_file_id = ?
    GROUP BY f.path ORDER BY cnt DESC LIMIT 15
  `).all(taId);
  for (const r of taModules) {
    console.log(`  ${String(r.cnt).padStart(3)}x  ${r.module.replace('src/', '')}`);
  }
}

db.close();
console.log('\nPhase 5 complete.');
