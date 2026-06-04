/**
 * Phase 4: 交叉分析 — 死代码 / 循环依赖 / 边界违规 / 关键比率
 */

import { DatabaseSync } from 'node:sqlite';
import { join } from 'node:path';

const ROOT = 'C:/deep-student';
const DB_PATH = join(ROOT, '.planning/dependency-db/deps.db');
const db = new DatabaseSync(DB_PATH);

console.log('══════════════════════════════════════════════');
console.log('  Phase 4: 交叉分析报告');
console.log('══════════════════════════════════════════════\n');

// ── 1. cn() 精确比例 ────────────────────────────────────
console.log('【1】 cn() 实现使用比例');
const cnLibCount = db.prepare(`
  SELECT COUNT(DISTINCT importer_file_id) as cnt
  FROM imports WHERE imported_file_path = 'src/lib/utils.ts' AND imported_name = 'cn'
`).get().cnt;
const cnUtilCount = db.prepare(`
  SELECT COUNT(DISTINCT importer_file_id) as cnt
  FROM imports WHERE imported_file_path = 'src/utils/cn.ts' AND imported_name = 'cn'
`).get().cnt;
console.log(`  @/lib/utils  (遗留, 无twMerge): ${cnLibCount} 个文件`);
console.log(`  @/utils/cn   (推荐, clsx+twMerge): ${cnUtilCount} 个文件`);
console.log(`  遗留比例: ${(cnLibCount/(cnLibCount+cnUtilCount)*100).toFixed(1)}%`);

// ── 2. MistakeItem 完整引用链 ────────────────────────────
console.log('\n【2】 MistakeItem 引用链');
const mistakeImporters = db.prepare(`
  SELECT DISTINCT f.path, f.lines
  FROM imports i JOIN files f ON i.importer_file_id = f.id
  WHERE i.imported_file_path = 'src/types/index.ts' AND i.imported_name = 'MistakeItem'
  ORDER BY f.path
`).all();
console.log(`  从 types/index.ts 导入 MistakeItem 的文件: ${mistakeImporters.length} 个`);
for (const r of mistakeImporters) {
  const reExported = db.prepare(`
    SELECT COUNT(*) as cnt FROM exports e JOIN files f2 ON e.file_id = f2.id
    WHERE f2.path = ? AND e.name = 'MistakeSummary'
  `).get(r.path).cnt;
  const tag = reExported > 0 ? ' [re-exports as MistakeSummary]' : '';
  console.log(`    ${r.path} (${r.lines}l)${tag}`);
}

// ── 3. API 层绕过 ────────────────────────────────────────
console.log('\n【3】 Store 直接调用 invoke (绕过 API 层)');
const bypassStores = db.prepare(`
  SELECT DISTINCT f.path
  FROM imports i JOIN files f ON i.importer_file_id = f.id
  WHERE f.category = 'store'
    AND f.language = 'typescript'
    AND i.imported_file_path LIKE 'external:%@tauri-apps%'
  ORDER BY f.path
`).all();
console.log(`  直接 import @tauri-apps 的 Store: ${bypassStores.length} 个`);
for (const r of bypassStores) console.log(`    ${r.path}`);

// 对比: 正确使用 API 层的
const correctStores = db.prepare(`
  SELECT DISTINCT f.path
  FROM imports i JOIN files f ON i.importer_file_id = f.id
  WHERE f.category = 'store'
    AND f.language = 'typescript'
    AND i.imported_file_path LIKE 'src/api/%'
  ORDER BY f.path
`).all();
console.log(`  正确使用 api/ 层的 Store: ${correctStores.length} 个`);
for (const r of correctStores) console.log(`    ${r.path}`);

// ── 4. 空壳 Feature 目录 ──────────────────────────────────
console.log('\n【4】 空壳 Feature 目录验证');
const emptyFeatures = [
  'src/features/practice',
  'src/features/template-management',
];
for (const feat of emptyFeatures) {
  const filesInDir = db.prepare('SELECT COUNT(*) as cnt FROM files WHERE path LIKE ?').get(feat + '/%').cnt;
  const importsFrom = db.prepare('SELECT COUNT(*) as cnt FROM imports WHERE imported_file_path LIKE ?').get(feat + '/%').cnt;
  const importsBy = db.prepare(`
    SELECT COUNT(*) as cnt FROM imports i JOIN files f ON i.importer_file_id = f.id
    WHERE f.path LIKE ?
  `).get(feat + '/%').cnt;
  const exportsFrom = db.prepare(`
    SELECT COUNT(*) as cnt FROM exports e JOIN files f ON e.file_id = f.id
    WHERE f.path LIKE ?
  `).get(feat + '/%').cnt;

  console.log(`  ${feat}:`);
  console.log(`    文件: ${filesInDir}  被导入: ${importsFrom}次  导入外部: ${importsBy}次  导出: ${exportsFrom}个`);
  if (importsFrom === 0 && exportsFrom === 0) console.log(`    → 确认死目录`);
}

// ── 5. 最大依赖文件 ──────────────────────────────────────
console.log('\n【5】 被依赖最多的文件 Top 15');
const topDepended = db.prepare(`
  SELECT f.path, f.language, COUNT(DISTINCT i.importer_file_id) as refs
  FROM imports i JOIN files f ON i.imported_file_path = f.path
  WHERE i.imported_file_path NOT LIKE 'external:%'
    AND i.imported_file_path NOT LIKE 'crate:%'
    AND i.imported_file_path NOT LIKE 'crate-unresolved:%'
  GROUP BY f.path
  ORDER BY refs DESC LIMIT 15
`).all();
for (const r of topDepended) {
  console.log(`  ${String(r.refs).padStart(4)} refs  [${r.language}] ${r.path}`);
}

// ── 6. 依赖最多的文件 (God File 的依赖面) ──────────────────
console.log('\n【6】 外部依赖最多的文件 Top 10');
const topDependents = db.prepare(`
  SELECT f.path, f.lines, COUNT(DISTINCT i.imported_file_path) as deps
  FROM imports i JOIN files f ON i.importer_file_id = f.id
  WHERE i.imported_file_path NOT LIKE 'external:%'
    AND i.imported_file_path NOT LIKE 'crate-unresolved:%'
    AND f.language = 'typescript'
  GROUP BY f.id ORDER BY deps DESC LIMIT 10
`).all();
for (const r of topDependents) {
  console.log(`  ${String(r.deps).padStart(3)} deps  ${r.path} (${r.lines}l)`);
}

// ── 7. 边界违规 ──────────────────────────────────────────
console.log('\n【7】 Feature 模块边界违规 (feature → feature 直接依赖)');
const boundaryViolations = db.prepare(`
  SELECT DISTINCT f.path as importer, i.imported_file_path as target
  FROM imports i JOIN files f ON i.importer_file_id = f.id
  WHERE f.path LIKE 'src/features/%'
    AND i.imported_file_path LIKE 'src/features/%'
    AND SUBSTR(f.path, 15, INSTR(SUBSTR(f.path, 15), '/') - 1)
        != SUBSTR(i.imported_file_path, 15, INSTR(SUBSTR(i.imported_file_path, 15), '/') - 1)
    AND f.path NOT LIKE '%index.ts'
  LIMIT 30
`).all();
console.log(`  跨 feature 依赖数: ${boundaryViolations.length}`);
const grouped = {};
for (const v of boundaryViolations) {
  const from = v.importer.replace('src/features/', '').split('/')[0];
  const to = v.target.replace('src/features/', '').split('/')[0];
  const key = `${from} → ${to}`;
  grouped[key] = (grouped[key] || 0) + 1;
}
const sorted = Object.entries(grouped).sort((a,b) => b[1]-a[1]).slice(0, 15);
for (const [k, v] of sorted) {
  console.log(`  ${String(v).padStart(3)}x  ${k}`);
}

// ── 8. 文件大小分布 ──────────────────────────────────────
console.log('\n【8】 God File 分布');
const sizeDist = db.prepare(`
  SELECT
    CASE WHEN lines > 5000 THEN '5000+'
         WHEN lines > 3000 THEN '3000-5000'
         WHEN lines > 2000 THEN '2000-3000'
         WHEN lines > 1000 THEN '1000-2000'
         WHEN lines > 500  THEN '500-1000'
         ELSE '<500' END as bucket,
    COUNT(*) as cnt
  FROM files WHERE language IN ('typescript', 'rust')
  GROUP BY bucket
  ORDER BY MIN(lines) DESC
`).all();
for (const r of sizeDist) {
  console.log(`  ${r.bucket.padEnd(12)} ${String(r.cnt).padStart(4)} 文件`);
}

// ── 9. 绝对死代码 (零导入 + 零导出) ──────────────────────
console.log('\n【9】 完全孤立文件 (0导入 0导出, >100行, 非测试)');
const deadFiles = db.prepare(`
  SELECT f.path, f.lines, f.category
  FROM files f
  WHERE f.language = 'typescript'
    AND f.lines > 100
    AND f.path NOT LIKE '%__tests__%'
    AND f.path NOT LIKE '%.test.%'
    AND f.path NOT LIKE '%.css'
    AND f.path NOT LIKE '%.d.ts'
    AND f.path NOT LIKE '%index.ts'
    AND f.path NOT LIKE '%index.tsx'
    AND f.category NOT IN ('entry', 'config')
    AND f.id NOT IN (SELECT DISTINCT importer_file_id FROM imports)
    AND f.id NOT IN (SELECT DISTINCT file_id FROM exports)
  ORDER BY f.lines DESC LIMIT 10
`).all();
if (deadFiles.length === 0) {
  console.log('  (无 — 所有大文件都至少被引用或导出)');
} else {
  for (const r of deadFiles) {
    console.log(`  ${r.lines}l  ${r.category}  ${r.path}`);
  }
}

db.close();
console.log('\nPhase 4 complete.');
