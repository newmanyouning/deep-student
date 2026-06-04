/**
 * Phase 2a: TypeScript 导入关系提取
 * 解析所有 .ts/.tsx 文件的 import 语句，写入 imports 表。
 */

import { DatabaseSync } from 'node:sqlite';
import { readFileSync } from 'node:fs';
import { join, dirname, relative, normalize } from 'node:path';

const ROOT = 'C:/deep-student';
const DB_PATH = join(ROOT, '.planning/dependency-db/deps.db');
const db = new DatabaseSync(DB_PATH);

// ── 获取所有 TS 文件 ─────────────────────────────────────
const tsFiles = db.prepare(`
  SELECT id, path FROM files
  WHERE language IN ('typescript', 'css')
  ORDER BY path
`).all();

console.log(`Processing ${tsFiles.length} TypeScript/CSS files...`);

// ── 路径解析 ────────────────────────────────────────────
const SRC_ROOT = join(ROOT, 'src');

function resolveImportPath(importPath, currentFilePath) {
  // 跳过外部包和 Node 内置模块
  if (!importPath.startsWith('.') && !importPath.startsWith('@/')) {
    return { resolved: null, isExternal: true, package: importPath.split('/')[0] };
  }

  // @/ 别名 → src/
  if (importPath.startsWith('@/')) {
    const relPath = importPath.replace('@/', 'src/');
    return { resolved: normalize(relPath).replace(/\\/g, '/'), isExternal: false };
  }

  // 相对路径
  const currentDir = dirname(join(ROOT, currentFilePath));
  const absImport = normalize(join(currentDir, importPath));
  const relToRoot = relative(ROOT, absImport).replace(/\\/g, '/');

  // 检查是否在 src/ 或 src-tauri/ 内
  if (relToRoot.startsWith('src/') || relToRoot.startsWith('src-tauri/')) {
    return { resolved: relToRoot, isExternal: false };
  }

  // 超出项目范围
  return { resolved: null, isExternal: true, package: importPath };
}

function resolveToFileId(importPath, currentFilePath) {
  const result = resolveImportPath(importPath, currentFilePath);

  if (result.isExternal) return { fileId: null, ...result };

  // 尝试精确匹配
  let resolved = result.resolved;

  // 去掉扩展名尝试匹配
  const exts = ['.ts', '.tsx', '.css', '.json', '/index.ts', '/index.tsx'];
  for (const ext of exts) {
    const candidate = resolved.endsWith(ext) ? resolved : resolved + ext;
    const row = db.prepare('SELECT id FROM files WHERE path = ?').get(candidate);
    if (row) return { fileId: row.id, resolved: candidate, isExternal: false };
  }

  // 如果已带扩展名，尝试直接匹配
  const row = db.prepare('SELECT id FROM files WHERE path = ?').get(resolved);
  if (row) return { fileId: row.id, resolved, isExternal: false };

  // 未解析到具体文件（可能是路径错误或跨模块引用）
  return { fileId: null, resolved, isExternal: false, unresolved: true };
}

// ── 解析 import 语句 ─────────────────────────────────────
// 正则：匹配各种 import 形式
const IMPORT_RE = /import\s+(?:type\s+)?(?:(?:\{[^}]*\}|[\w*]+)(?:\s*,\s*(?:\{[^}]*\}|[\w*]+))*\s+from\s+)?['"]([^'"]+)['"]/g;
const NAMED_IMPORT_RE = /import\s+(?:type\s+)?\{([^}]+)\}\s+from\s+['"]([^'"]+)['"]/g;
const DEFAULT_IMPORT_RE = /import\s+(?:type\s+)?(\w+)\s+from\s+['"]([^'"]+)['"]/g;
const NAMESPACE_IMPORT_RE = /import\s+(?:type\s+)?\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]/g;
const DYNAMIC_IMPORT_RE = /import\s*\(\s*['"]([^'"]+)['"]\s*\)/g;
const LAZY_IMPORT_RE = /React\.lazy\s*\(\s*\(\)\s*=>\s*import\s*\(\s*['"]([^'"]+)['"]\s*\)/g;
const SIDE_EFFECT_RE = /import\s+['"]([^'"]+)['"]\s*;?/g;

// ── 提取导入 ────────────────────────────────────────────
const insertImport = db.prepare(`
  INSERT OR IGNORE INTO imports (importer_file_id, imported_file_path, imported_name, is_type_only)
  VALUES (?, ?, ?, ?)
`);

let totalImports = 0;
let externalCount = 0;
let unresolvedCount = 0;
let processed = 0;

// 批量事务
db.exec('BEGIN');

for (const file of tsFiles) {
  processed++;
  if (processed % 200 === 0) {
    db.exec('COMMIT; BEGIN');
    console.log(`  ${processed}/${tsFiles.length} files...`);
  }

  let content;
  try {
    content = readFileSync(join(ROOT, file.path), 'utf-8');
  } catch {
    continue;
  }

  // 跳过空内容的行（提速）
  const imports = new Set(); // 去重

  // 1. Named imports: import { X, Y } from 'path'
  // 2. import { type X, Y } from 'path' — inline type
  const namedImportRe = /import\s+(type\s+)?\{([^}]+)\}\s+from\s+['"]([^'"]+)['"]/g;
  let match;
  while ((match = namedImportRe.exec(content)) !== null) {
    const isType = !!match[1];
    const names = match[2].split(',').map(n => n.trim()).filter(n => n && !n.startsWith('type '));
    const typeNames = match[2].split(',').map(n => n.trim()).filter(n => n.startsWith('type ')).map(n => n.replace('type ', '').trim());
    const importPath = match[3];

    const { fileId, resolved, isExternal, unresolved } = resolveToFileId(importPath, file.path);

    for (const name of names) {
      const key = `${importPath}|${name}|${isType ? 'type' : 'value'}`;
      if (imports.has(key)) continue;
      imports.add(key);
      const targetPath = (isExternal ? `external:${importPath}` : (resolved || importPath));
      insertImport.run(file.id, targetPath, name, isType ? 1 : 0);
      totalImports++;
      if (isExternal) externalCount++;
      if (unresolved) unresolvedCount++;
    }
    for (const name of typeNames) {
      const key = `${importPath}|${name}|type`;
      if (imports.has(key)) continue;
      imports.add(key);
      const targetPath = (isExternal ? `external:${importPath}` : (resolved || importPath));
      insertImport.run(file.id, targetPath, name, 1);
      totalImports++;
      if (isExternal) externalCount++;
      if (unresolved) unresolvedCount++;
    }
  }

  // 3. Default imports: import X from 'path'
  const defaultRe = /import\s+(?:type\s+)?(\w+)\s+from\s+['"]([^'"]+)['"]/g;
  while ((match = defaultRe.exec(content)) !== null) {
    const name = match[1];
    if (name === 'type') continue; // "import type {X}" already handled
    const importPath = match[2];

    const key = `${importPath}|${name}|default`;
    if (imports.has(key)) continue;
    imports.add(key);

    const { fileId, resolved, isExternal, unresolved } = resolveToFileId(importPath, file.path);
    const targetPath = (isExternal ? `external:${importPath}` : (resolved || importPath));
    insertImport.run(file.id, targetPath, name, 0);
    totalImports++;
    if (isExternal) externalCount++;
    if (unresolved) unresolvedCount++;
  }

  // 4. Namespace imports: import * as X from 'path'
  const nsRe = /import\s+\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]/g;
  while ((match = nsRe.exec(content)) !== null) {
    const name = `*:${match[1]}`;
    const importPath = match[2];

    const key = `${importPath}|${name}|namespace`;
    if (imports.has(key)) continue;
    imports.add(key);

    const { fileId, resolved, isExternal, unresolved } = resolveToFileId(importPath, file.path);
    const targetPath = (isExternal ? `external:${importPath}` : (resolved || importPath));
    insertImport.run(file.id, targetPath, name, 0);
    totalImports++;
    if (isExternal) externalCount++;
    if (unresolved) unresolvedCount++;
  }

  // 5. Dynamic imports: import('path')
  while ((match = DYNAMIC_IMPORT_RE.exec(content)) !== null) {
    const importPath = match[1];
    const key = `${importPath}|*dynamic*|dynamic`;
    if (imports.has(key)) continue;
    imports.add(key);

    const { fileId, resolved, isExternal, unresolved } = resolveToFileId(importPath, file.path);
    const targetPath = (isExternal ? `external:${importPath}` : (resolved || importPath));
    insertImport.run(file.id, targetPath, '*dynamic*', 0);
    totalImports++;
    if (isExternal) externalCount++;
    if (unresolved) unresolvedCount++;
  }

  // 6. Side-effect imports: import 'path'; or import 'path'
  const sideRe = /import\s+['"]([^'"]+\.(?:css|json))['"]\s*;?/g;
  while ((match = sideRe.exec(content)) !== null) {
    const importPath = match[1];
    const key = `${importPath}|*side*|side`;
    if (imports.has(key)) continue;
    imports.add(key);

    const { fileId, resolved, isExternal, unresolved } = resolveToFileId(importPath, file.path);
    const targetPath = (isExternal ? `external:${importPath}` : (resolved || importPath));
    insertImport.run(file.id, targetPath, '*side*', 0);
    totalImports++;
    if (isExternal) externalCount++;
    if (unresolved) unresolvedCount++;
  }
}

db.exec('COMMIT');

console.log(`\nDone. Processed ${processed} files.`);
console.log(`  Total imports: ${totalImports}`);
console.log(`  External (npm): ${externalCount}`);
console.log(`  Unresolved (broken?): ${unresolvedCount}`);
console.log(`  Internal (project): ${totalImports - externalCount}`);

// ── 快速统计 ─────────────────────────────────────────────
console.log('\n=== Top 20 most-imported internal files ===');
const topImported = db.prepare(`
  SELECT i.imported_file_path as target, COUNT(*) as cnt
  FROM imports i
  WHERE i.imported_file_path NOT LIKE 'external:%'
    AND i.imported_file_path NOT LIKE '%.css'
  GROUP BY i.imported_file_path
  ORDER BY cnt DESC
  LIMIT 20
`).all();
for (const r of topImported) {
  console.log(`  ${String(r.cnt).padStart(4)} refs  ${r.target}`);
}

console.log('\n=== Top 20 most-imported external packages ===');
const topExternal = db.prepare(`
  SELECT i.imported_file_path as pkg, COUNT(*) as cnt
  FROM imports i
  WHERE i.imported_file_path LIKE 'external:%'
  GROUP BY i.imported_file_path
  ORDER BY cnt DESC
  LIMIT 20
`).all();
for (const r of topExternal) {
  const name = r.pkg.replace('external:', '').split('/')[0];
  console.log(`  ${String(r.cnt).padStart(4)} refs  ${name}`);
}

console.log('\n=== Dead code candidates (not imported by anyone yet) ===');
const dead = db.prepare(`
  SELECT f.path, f.lines, f.category FROM files f
  WHERE f.language = 'typescript'
    AND f.category NOT IN ('entry', 'config', 'style', 'assets')
    AND f.path NOT LIKE '%__tests__%'
    AND f.path NOT LIKE '%.test.%'
    AND f.path NOT LIKE '%.css'
    AND f.id NOT IN (
      SELECT DISTINCT ii.id FROM files ff
      JOIN imports ii ON ii.imported_file_path = ff.path
      WHERE ff.id = f.id
    )
    AND f.path NOT IN (
      SELECT imported_file_path FROM imports WHERE imported_file_path = f.path
    )
  ORDER BY f.lines DESC
  LIMIT 15
`).all();
if (dead.length === 0) {
  console.log('  (none found — all files have at least one import reference)');
} else {
  for (const r of dead) {
    console.log(`  ${String(r.lines).padStart(5)}l  ${r.path}`);
  }
}

db.close();
console.log('\nPhase 2a complete.');
