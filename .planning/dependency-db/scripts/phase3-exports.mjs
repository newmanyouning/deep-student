/**
 * Phase 3: 导出关系提取 (TypeScript + Rust)
 * 解析所有文件的 export 语句，写入 exports 表。
 */

import { DatabaseSync } from 'node:sqlite';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = 'C:/deep-student';
const DB_PATH = join(ROOT, '.planning/dependency-db/deps.db');
const db = new DatabaseSync(DB_PATH);

// ── TypeScript 导出 ──────────────────────────────────────
function extractTSExports(fileId, filePath) {
  let content;
  try { content = readFileSync(join(ROOT, filePath), 'utf-8'); } catch { return []; }

  const results = [];

  // exports with declaration: export const/function/class/interface/type/enum Name
  const declRe = /export\s+(?:default\s+)?(?:const|function|class|interface|type|enum)\s+(\w+)/g;
  let m;
  while ((m = declRe.exec(content)) !== null) {
    const isDefault = m[0].includes('default');
    // 提取 kind
    const kindMatch = m[0].match(/export\s+(?:default\s+)?(const|function|class|interface|type|enum)/);
    results.push({ name: m[1], kind: kindMatch ? kindMatch[1] : 'unknown', isDefault: isDefault ? 1 : 0 });
  }

  // Named exports: export { X, Y, Z as W }
  const namedRe = /export\s+(?:type\s+)?\{([^}]+)\}/g;
  while ((m = namedRe.exec(content)) !== null) {
    // 检查前面是否有 from — 如果是 barrel re-export，跳过
    const beforeM = content.substring(Math.max(0, m.index - 10), m.index);
    if (beforeM.includes('= require') || beforeM.includes('import(')) continue;

    const items = m[1].split(',').map(s => s.trim()).filter(Boolean);
    for (const item of items) {
      const asMatch = item.match(/^(\w+)\s+as\s+(\w+)$/);
      if (asMatch) {
        results.push({ name: asMatch[1], kind: 're-export', isDefault: 0 });
        results.push({ name: asMatch[2], kind: 're-export-alias', isDefault: 0 });
      } else {
        results.push({ name: item, kind: 're-export', isDefault: 0 });
      }
    }
  }

  // export default (anonymous functions/classes)
  if (/export\s+default\s+(function|class)/.test(content)) {
    results.push({ name: 'default', kind: 'default', isDefault: 1 });
  }

  // export * from '...' → mark as wildcard
  const wildcardRe = /export\s+\*\s+from\s+['"]([^'"]+)['"]/g;
  while ((m = wildcardRe.exec(content)) !== null) {
    results.push({ name: `*→${m[1]}`, kind: 'wildcard-reexport', isDefault: 0 });
  }

  return results;
}

// ── Rust 导出 ────────────────────────────────────────────
function extractRustExports(fileId, filePath) {
  let content;
  try { content = readFileSync(join(ROOT, filePath), 'utf-8'); } catch { return []; }

  // 移除注释
  const cleaned = content.replace(/\/\/.*$/gm, '').replace(/\/\*[\s\S]*?\*\//g, '');

  const results = [];
  // pub fn/struct/enum/trait/mod/type/const/static Name
  const pubRe = /pub(?:\s*\(\s*(?:crate|super|self)\s*\))?\s+(fn|struct|enum|trait|mod|type|const|static|async\s+fn|unsafe\s+fn)\s+(\w+)/g;
  let m;
  while ((m = pubRe.exec(cleaned)) !== null) {
    const kind = m[1].replace('async ', '').replace('unsafe ', '');
    results.push({ name: m[2], kind, isDefault: 0 });
  }

  // pub use re-exports
  const pubUseRe = /pub\s+use\s+([^;]+);/g;
  while ((m = pubUseRe.exec(cleaned)) !== null) {
    results.push({ name: `reuse→${m[1].trim()}`, kind: 'pub-use', isDefault: 0 });
  }

  return results;
}

// ── 写入数据库 ──────────────────────────────────────────

// 清空旧数据
db.exec('DELETE FROM exports');

const insertExport = db.prepare(`
  INSERT OR IGNORE INTO exports (file_id, name, kind, is_default) VALUES (?, ?, ?, ?)
`);

const allFiles = db.prepare('SELECT id, path, language FROM files ORDER BY language, path').all();

let totalExports = 0;
let processed = 0;

db.exec('BEGIN');
for (const file of allFiles) {
  processed++;
  let exports;
  if (file.language === 'typescript' || file.language === 'css') {
    exports = extractTSExports(file.id, file.path);
  } else if (file.language === 'rust') {
    exports = extractRustExports(file.id, file.path);
  } else {
    continue;
  }

  for (const exp of exports) {
    try {
      insertExport.run(file.id, exp.name, exp.kind, exp.isDefault);
      totalExports++;
    } catch {}
  }
}
db.exec('COMMIT');

console.log(`Processed ${processed} files.`);
console.log(`Total exports: ${totalExports}`);

// ── 统计 ─────────────────────────────────────────────────
console.log('\n=== Exports by language/kind ===');
const kindStats = db.prepare(`
  SELECT f.language, e.kind, COUNT(*) as cnt
  FROM exports e JOIN files f ON e.file_id = f.id
  GROUP BY f.language, e.kind ORDER BY cnt DESC LIMIT 15
`).all();
for (const r of kindStats) {
  console.log(`  ${r.language.padEnd(12)} ${r.kind.padEnd(20)} ${r.cnt}`);
}

console.log('\n=== Files with most exports (TS) ===');
const topTS = db.prepare(`
  SELECT f.path, COUNT(*) as cnt, f.lines
  FROM exports e JOIN files f ON e.file_id = f.id
  WHERE f.language = 'typescript'
  GROUP BY f.id ORDER BY cnt DESC LIMIT 10
`).all();
for (const r of topTS) {
  console.log(`  ${String(r.cnt).padStart(4)} exports  ${r.path} (${r.lines}l)`);
}

console.log('\n=== Files with most exports (Rust) ===');
const topRust = db.prepare(`
  SELECT f.path, COUNT(*) as cnt, f.lines
  FROM exports e JOIN files f ON e.file_id = f.id
  WHERE f.language = 'rust'
  GROUP BY f.id ORDER BY cnt DESC LIMIT 10
`).all();
for (const r of topRust) {
  console.log(`  ${String(r.cnt).padStart(4)} exports  ${r.path} (${r.lines}l)`);
}

// ── 死代码检测: 导出但未被导入 ──────────────────────────
console.log('\n=== Exported but never imported (potential dead code, top 15) ===');
const deadExports = db.prepare(`
  SELECT f.path, e.name, e.kind
  FROM exports e JOIN files f ON e.file_id = f.id
  WHERE f.language = 'typescript'
    AND f.path NOT LIKE '%__tests__%'
    AND f.path NOT LIKE '%.test.%'
    AND e.name NOT LIKE '*%'
    AND e.name NOT LIKE 'default'
    AND e.name NOT LIKE 'reuse%'
    AND e.name NOT IN (
      SELECT i.imported_name FROM imports i
      WHERE i.importer_file_id IS NOT NULL
        AND (i.imported_file_path = f.path
             OR i.imported_file_path LIKE '%/' || REPLACE(REPLACE(f.path, 'src/', ''), '.tsx', '') || '%'
             OR i.imported_file_path LIKE '%/' || REPLACE(REPLACE(f.path, 'src/', ''), '.ts', '') || '%')
    )
  LIMIT 20
`).all();
for (const r of deadExports) {
  console.log(`  ${r.kind.padEnd(12)} ${r.name.padEnd(30)} ${r.path}`);
}

db.close();
console.log('\nPhase 3 complete.');
