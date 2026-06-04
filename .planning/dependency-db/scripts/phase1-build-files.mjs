/**
 * Phase 1: 建立 files 表
 * 遍历 src/ 和 src-tauri/src/ 所有源文件，填充文件清单。
 */

import { DatabaseSync } from 'node:sqlite';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative, extname } from 'node:path';

const ROOT = 'C:/deep-student';
const DB_PATH = join(ROOT, '.planning/dependency-db/deps.db');

// ── 创建数据库 + Schema ──────────────────────────────────
const db = new DatabaseSync(DB_PATH);

db.exec(`
  -- 文件清单
  CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    language TEXT NOT NULL,
    category TEXT,
    subcategory TEXT,
    lines INTEGER DEFAULT 0,
    is_dead INTEGER DEFAULT 0,
    notes TEXT
  );

  -- 导出
  CREATE TABLE IF NOT EXISTS exports (
    id INTEGER PRIMARY KEY,
    file_id INTEGER REFERENCES files(id),
    name TEXT NOT NULL,
    kind TEXT,
    is_default INTEGER DEFAULT 0,
    UNIQUE(file_id, name)
  );

  -- 导入
  CREATE TABLE IF NOT EXISTS imports (
    id INTEGER PRIMARY KEY,
    importer_file_id INTEGER REFERENCES files(id),
    imported_file_path TEXT NOT NULL,
    imported_name TEXT,
    is_type_only INTEGER DEFAULT 0,
    UNIQUE(importer_file_id, imported_file_path, imported_name)
  );

  -- 函数调用
  CREATE TABLE IF NOT EXISTS calls (
    id INTEGER PRIMARY KEY,
    caller_file_id INTEGER REFERENCES files(id),
    callee_name TEXT NOT NULL,
    callee_file_id INTEGER,
    call_site_line INTEGER,
    kind TEXT
  );

  -- 诊断发现
  CREATE TABLE IF NOT EXISTS findings (
    id INTEGER PRIMARY KEY,
    finding_id TEXT UNIQUE,
    severity TEXT,
    title TEXT,
    description TEXT,
    affected_files TEXT,
    round TEXT
  );

  -- 死代码候选视图
  CREATE VIEW IF NOT EXISTS dead_code_candidates AS
  SELECT f.id, f.path, f.language, f.category, f.lines
  FROM files f
  WHERE f.category NOT IN ('entry', 'config', 'test', 'types-shared', 'assets')
    AND f.id NOT IN (SELECT DISTINCT importer_file_id FROM imports);
`);

// ── 文件分类逻辑 ────────────────────────────────────────
function classifyFile(filepath) {
  const p = filepath.replace(/\\/g, '/');

  // src-tauri (Rust)
  if (p.startsWith('src-tauri/src/')) {
    const sub = p.replace('src-tauri/src/', '');
    if (sub.includes('/')) {
      const dir = sub.split('/')[0];
      return { language: 'rust', category: mapRustDir(dir), subcategory: dir };
    }
    if (sub.endsWith('.rs') && sub !== 'lib.rs' && sub !== 'main.rs') {
      const name = sub.replace('.rs', '');
      return { language: 'rust', category: 'service-single', subcategory: name };
    }
    if (sub === 'lib.rs') return { language: 'rust', category: 'entry', subcategory: 'root' };
    if (sub === 'main.rs') return { language: 'rust', category: 'entry', subcategory: 'root' };
    return { language: 'rust', category: 'other' };
  }

  // src/ (TypeScript/CSS)
  if (p.startsWith('src/')) {
    const sub = p.replace('src/', '');
    const ext = extname(p);

    // 特殊根文件
    if (sub === 'App.tsx' || sub === 'main.tsx' || sub === 'i18n.ts' || sub === 'lazyComponents.tsx')
      return { language: 'typescript', category: 'entry', subcategory: 'root' };

    // 子目录
    if (sub.includes('/')) {
      const dir = sub.split('/')[0];
      if (ext === '.css')
        return { language: 'css', category: 'style', subcategory: dir };
      return { language: 'typescript', category: mapTSDir(dir), subcategory: dir };
    }

    // src/ 根级别的文件
    return { language: 'typescript', category: 'other', subcategory: 'root-file' };
  }

  // 根目录配置等
  if (p.startsWith('.') || !p.includes('/'))
    return { language: extname(p).replace('.', ''), category: 'config' };

  return { language: 'unknown', category: 'other' };
}

function mapTSDir(dir) {
  const map = {
    'api': 'api-layer',
    'app': 'app-shell',
    'assets': 'assets',
    'command-palette': 'system-feature',
    'components': 'component',
    'config': 'config',
    'contexts': 'context',
    'data': 'data-layer',
    'debug-panel': 'system-feature',
    'dstu': 'protocol',
    'engines': 'engine',
    'essay-grading': 'feature',
    'events': 'event-system',
    'features': 'feature',
    'hooks': 'hook',
    'lib': 'library',
    'locales': 'i18n',
    'mcp': 'protocol',
    'mcp-debug': 'system-feature',
    'menu': 'app-shell',
    'polyfills': 'library',
    'promptkit': 'component',
    'services': 'service-layer',
    'shared': 'shared',
    'shims': 'library',
    'store': 'store',
    'stores': 'store',
    'styles': 'style',
    'translation': 'feature',
    'types': 'types',
    'utils': 'utility',
    'voice-input': 'feature',
  };
  return map[dir] || 'other';
}

function mapRustDir(dir) {
  const map = {
    'chat_v2': 'chat-engine',
    'llm_manager': 'llm',
    'vfs': 'vfs',
    'dstu': 'protocol',
    'tools': 'search',
    'memory': 'memory',
    'mcp': 'protocol',
    'translation': 'feature',
    'cloud_storage': 'cloud',
    'data_governance': 'data-gov',
    'essay_grading': 'feature',
    'qbank_grading': 'feature',
    'crypto': 'security',
    'multimodal': 'multimodal',
    'ocr_adapters': 'ocr',
    'llm_usage': 'llm',
    'cmd': 'command',
    'database': 'database',
    'adapters': 'adapter',
    'providers': 'adapter',
    'services': 'service',
    'utils': 'utility',
    'vendors': 'adapter',
  };
  return map[dir] || 'service-single';
}

// ── 遍历文件 ────────────────────────────────────────────
function walkDir(dir, extensions, fileList = []) {
  try {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const full = join(dir, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === 'node_modules' || entry.name === 'target' || entry.name === '.git')
          continue;
        walkDir(full, extensions, fileList);
      } else if (extensions.some(e => entry.name.endsWith(e))) {
        fileList.push(full);
      }
    }
  } catch { /* skip inaccessible dirs */ }
  return fileList;
}

console.log('Walking src/...');
const tsFiles = walkDir(join(ROOT, 'src'), ['.ts', '.tsx', '.css']);
console.log(`  Found ${tsFiles.length} files`);

console.log('Walking src-tauri/src/...');
const rustFiles = walkDir(join(ROOT, 'src-tauri/src'), ['.rs']);
console.log(`  Found ${rustFiles.length} files`);

const allFiles = [...tsFiles, ...rustFiles];

// ── 写入数据库 ──────────────────────────────────────────
const insert = db.prepare(
  'INSERT INTO files (path, language, category, subcategory, lines) VALUES (?, ?, ?, ?, ?)'
);

const tx = db.exec('BEGIN');
let count = 0;
for (const absPath of allFiles) {
  const rel = relative(ROOT, absPath).replace(/\\/g, '/');
  const { language, category, subcategory } = classifyFile(rel);
  let lines = 0;
  try { lines = readFileSync(absPath, 'utf-8').split('\n').length; } catch { /* binary */ }

  insert.run(rel, language, category, subcategory || null, lines);
  count++;
}
db.exec('COMMIT');

console.log(`Inserted ${count} files into files table.`);

// ── 快速统计 ─────────────────────────────────────────────
const stats = db.prepare(`
  SELECT language, category, COUNT(*) as cnt, SUM(lines) as total_lines
  FROM files GROUP BY language, category ORDER BY language, cnt DESC
`).all();

console.log('\n=== Files by category ===');
for (const r of stats) {
  console.log(`  ${r.language.padEnd(12)} ${r.category.padEnd(20)} ${String(r.cnt).padStart(5)} files  ${String(r.total_lines).padStart(8)} lines`);
}

const total = db.prepare('SELECT COUNT(*) as cnt, SUM(lines) as total_lines FROM files').get();
console.log(`\n  TOTAL: ${total.cnt} files, ${total.total_lines} lines`);

db.close();
console.log('\nPhase 1 complete. Database at:', DB_PATH);
