/**
 * Phase 2b: Rust 导入关系提取 (v2)
 * 解析所有 .rs 文件的 use 语句，写入 imports 表。
 */

import { DatabaseSync } from 'node:sqlite';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = 'C:/deep-student';
const DB_PATH = join(ROOT, '.planning/dependency-db/deps.db');
const db = new DatabaseSync(DB_PATH);

const rustFiles = db.prepare(`
  SELECT id, path FROM files WHERE language = 'rust' ORDER BY path
`).all();

console.log(`Processing ${rustFiles.length} Rust files...`);

// ── Crate 路径 → 文件 ID 映射 ──────────────────────────
const moduleToFileId = new Map();

for (const f of rustFiles) {
  let rel = f.path.replace('src-tauri/src/', '').replace(/\.rs$/, '').replace(/\//g, '::');
  rel = rel.replace(/::mod$/, '');
  if (rel) moduleToFileId.set(rel, f.id);

  // 父路径: chat_v2::tools::chatanki_executor → chat_v2::tools → chat_v2
  const parts = rel.split('::');
  for (let i = parts.length - 1; i >= 1; i--) {
    const prefix = parts.slice(0, i).join('::');
    moduleToFileId.set(prefix, f.id);
  }
}

// ── 提取 use 语句 ────────────────────────────────────────
function extractUseStatements(content) {
  let cleaned = content.replace(/\/\/.*$/gm, '').replace(/\/\*[\s\S]*?\*\//g, '');
  const lines = cleaned.split('\n');
  const statements = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i].trim();
    if (line.startsWith('use ') || line.startsWith('pub use ')) {
      let stmt = line;
      let depth = (stmt.match(/\{/g) || []).length - (stmt.match(/\}/g) || []).length;
      while (depth > 0 && i + 1 < lines.length) {
        i++;
        stmt += ' ' + lines[i].trim();
        depth = (stmt.match(/\{/g) || []).length - (stmt.match(/\}/g) || []).length;
      }
      statements.push(stmt);
    }
    i++;
  }
  return statements;
}

function parseUseItems(stmt) {
  const clean = stmt.replace(/^(pub )?use /, '').replace(/;$/, '').trim();
  const lastCC = clean.lastIndexOf('::');
  if (lastCC === -1) return [];

  const pathPart = clean.substring(0, lastCC);
  const itemPart = clean.substring(lastCC + 2);
  const firstSeg = pathPart.split('::')[0];

  // 分类
  let kind;
  if (firstSeg === 'crate') kind = 'crate';
  else if (['std', 'alloc', 'core'].includes(firstSeg)) kind = 'std';
  else kind = 'external';

  // 提取导入项
  let items = [];
  if (itemPart.startsWith('{') && itemPart.endsWith('}')) {
    items = itemPart.slice(1, -1).split(',')
      .map(s => s.trim()).filter(Boolean)
      .map(s => {
        const asM = s.match(/^(\w+)\s+as\s+(\w+)$/);
        return asM ? { n: asM[1], a: asM[2] } : { n: s, a: null };
      });
  } else if (itemPart === '*') {
    items = [{ n: '*', a: null }];
  } else {
    const asM = itemPart.match(/^(\w+)\s+as\s+(\w+)$/);
    items = asM ? [{ n: asM[1], a: asM[2] }] : [{ n: itemPart, a: null }];
  }

  return items.map(it => ({ kind, path: pathPart, name: it.n, alias: it.a }));
}

function resolveCrateTarget(cratePath) {
  const cleanPath = cratePath.replace(/^crate::/, '');
  // 精确匹配
  if (moduleToFileId.has(cleanPath)) return `crate:${cleanPath}`;
  // 父路径匹配: chat_v2::types::ChatMessage → chat_v2::types
  const parts = cleanPath.split('::');
  for (let len = parts.length - 1; len >= 1; len--) {
    const prefix = parts.slice(0, len).join('::');
    if (moduleToFileId.has(prefix)) return `crate:${cleanPath}`;
  }
  return `crate-unresolved:${cleanPath}`;
}

// ── 写入 ────────────────────────────────────────────────
const insertImport = db.prepare(`
  INSERT OR IGNORE INTO imports (importer_file_id, imported_file_path, imported_name, is_type_only)
  VALUES (?, ?, ?, 0)
`);

let total = 0, crateCount = 0, extCount = 0, stdCount = 0;
let processed = 0;

db.exec('BEGIN');

for (const file of rustFiles) {
  processed++;
  let content;
  try { content = readFileSync(join(ROOT, file.path), 'utf-8'); } catch { continue; }

  const stmts = extractUseStatements(content);
  for (const stmt of stmts) {
    for (const item of parseUseItems(stmt)) {
      total++;
      if (item.kind === 'std') { stdCount++; continue; }
      if (item.kind === 'external') {
        extCount++;
        const pkg = item.path.split('::')[0];
        try { insertImport.run(file.id, `external:${pkg}`, item.name, 0); } catch {}
        continue;
      }
      crateCount++;
      const target = resolveCrateTarget(item.path);
      try { insertImport.run(file.id, target, item.name, 0); } catch {}
    }
  }
}

db.exec('COMMIT');

console.log(`\nDone.`);
console.log(`  Total use items: ${total}`);
console.log(`  Internal (crate): ${crateCount}`);
console.log(`  External crates: ${extCount}`);
console.log(`  std/core/alloc: ${stdCount}`);

// ── 统计 (仅限 Rust 文件) ───────────────────────────────
const rustIds = rustFiles.map(f => f.id);
const rustIdSet = new Set(rustIds);

console.log('\n=== Top 15 most-used internal crate paths ===');
const topInternal = db.prepare(`
  SELECT i.imported_file_path as target, COUNT(*) as cnt
  FROM imports i
  WHERE i.imported_file_path LIKE 'crate:%'
    AND i.importer_file_id IN (${[...rustIdSet].join(',')})
  GROUP BY target ORDER BY cnt DESC LIMIT 15
`).all();
for (const r of topInternal) console.log(`  ${String(r.cnt).padStart(4)} refs  ${r.target.replace('crate:', '')}`);

console.log('\n=== Top 10 external Rust crates ===');
const topExt = db.prepare(`
  SELECT i.imported_file_path as pkg, COUNT(*) as cnt
  FROM imports i
  WHERE i.imported_file_path LIKE 'external:%'
    AND i.importer_file_id IN (${[...rustIdSet].join(',')})
  GROUP BY pkg ORDER BY cnt DESC LIMIT 10
`).all();
for (const r of topExt) console.log(`  ${String(r.cnt).padStart(4)} refs  ${r.pkg.replace('external:', '')}`);

console.log('\n=== Rust files with most crate deps ===');
const topDep = db.prepare(`
  SELECT f.path, f.lines, COUNT(*) as deps
  FROM imports i JOIN files f ON i.importer_file_id = f.id
  WHERE i.imported_file_path LIKE 'crate:%' AND i.importer_file_id IN (${[...rustIdSet].join(',')})
  GROUP BY f.id ORDER BY deps DESC LIMIT 10
`).all();
for (const r of topDep) console.log(`  ${String(r.deps).padStart(3)} deps  ${r.path} (${r.lines}l)`);

db.close();
console.log('\nPhase 2b complete.');
