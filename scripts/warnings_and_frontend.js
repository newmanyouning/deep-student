export const meta = {
  name: 'warnings-cleanup-and-frontend-sync',
  description: '清理222 Rust warnings → 建立变更接口数据库 → 扫描前端→ 修复前端 → 严格TS语法验证',
  phases: [
    { title: 'Phase 1: Warning Cleanup', detail: '分类清理222个Rust warnings：unused imports + deprecation' },
    { title: 'Phase 2: Change Interface Database', detail: '扫描git diff建立所有P0-P3接口变更数据库' },
    { title: 'Phase 3: Frontend Impact Scan', detail: '扫描~1575 TS/TSX文件，标记受变更影响的文件' },
    { title: 'Phase 4: Frontend Fixes', detail: '按变更数据库修复前端代码，严格TS语法验证' },
    { title: 'Phase 5: Final Verification', detail: 'cargo check + tsc验证全部修复' }
  ]
};

// ============================================================
// Phase 1: Warning Cleanup (2 parallel agents)
// ============================================================
phase('Phase 1: Warning Cleanup');

log('=== Phase 1: Cleaning 222 warnings ===');

var WARN_SCHEMA = {
  type: 'object',
  properties: {
    warnings_category: { type: 'string' },
    warnings_fixed: { type: 'number' },
    warnings_remaining: { type: 'number' },
    files_modified: { type: 'array', items: { type: 'string' } }
  },
  required: ['warnings_fixed', 'files_modified']
};

var warnResults = await parallel([
  // Agent 1a: Fix unused imports (the bulk - ~150 warnings)
  function() {
    var prompt = [
      '## Fix Unused Import Warnings in Rust Backend',
      '',
      'Read C:\\deep-student\\src-tauri\\cargo_check_final.txt — extract ALL "unused import" and "unused variable" warnings.',
      '',
      '### Strategy:',
      'Fix unused imports BY FILE. For each file with warnings:',
      '1. Read the file',
      '2. Remove ONLY the unused imports/variables',
      '3. Verify the file is still valid Rust after removal',
      '',
      '### Priority order:',
      '1. Files with 5+ unused import warnings (batch fix)',
      '2. Files with 1-4 warnings (quick fix)',
      '3. Warning categories to fix:',
      '   - "unused import" — remove the import line',
      '   - "unused variable" — prefix with _ or remove',
      '   - "unused mut" — remove mut keyword',
      '   - "unused `use`" — remove the use statement',
      '',
      '### CRITICAL RULES:',
      '- NEVER remove an import that IS used (double-check)',
      '- If an import appears unused but is needed for trait methods (like .encode() from Engine trait), KEEP IT',
      '- Use _ prefix for variables that must stay in scope but are unused',
      '- Do NOT change any logic or function bodies — only imports/mut/names',
      '',
      '### Expected fix count: ~150 warnings across ~50-80 files',
      'Process files in batches. Read, fix, verify syntax, move to next.'
    ].join('\n');
    return agent(prompt, { label: 'fix_unused_imports', schema: WARN_SCHEMA });
  },

  // Agent 1b: Fix deprecation warnings (~50 warnings)
  function() {
    var prompt = [
      '## Fix Deprecation Warnings in Rust Backend',
      '',
      'Read C:\\deep-student\\src-tauri\\cargo_check_final.txt — extract ALL deprecation warnings.',
      '',
      'Known deprecation patterns from our upgrades:',
      '',
      '1. image::image_dimensions → image::io::Reader::open(path)?.into_dimensions()',
      '   Files: pdf_ocr_service.rs, exam_sheet_service.rs, exam_engine.rs, model_profile_service.rs',
      '',
      '2. chrono::NaiveDateTime::from_timestamp_millis → DateTime::from_timestamp_millis',
      '   File: exam_sheet_service.rs:23',
      '',
      '3. Any other deprecation warnings — find the CURRENT API and migrate',
      '',
      '### Strategy:',
      'For EACH deprecation warning:',
      '1. Read the file at the warning line',
      '2. Look up the CURRENT API for the deprecated method (check the crate docs or changelog)',
      '3. Apply the minimal migration (change only the deprecated call)',
      '4. Verify the new call is valid Rust syntax',
      '',
      '### CRITICAL RULES:',
      '- Preserve all functionality — the replacement must do the same thing',
      '- Test: after fix, the deprecation warning should disappear',
      '- If a replacement is complex or risky, SKIP it and report why',
      '- Only fix deprecation warnings — no other changes'
    ].join('\n');
    return agent(prompt, { label: 'fix_deprecations', schema: WARN_SCHEMA });
  }
]);

var w1 = warnResults[0];
var w2 = warnResults[1];
if (w1) log('Unused imports: ' + w1.warnings_fixed + ' fixed in ' + (w1.files_modified ? w1.files_modified.length : 0) + ' files');
if (w2) log('Deprecations: ' + w2.warnings_fixed + ' fixed in ' + (w2.files_modified ? w2.files_modified.length : 0) + ' files');

// ============================================================
// Phase 2: Build Change Interface Database
// ============================================================
phase('Phase 2: Change Interface Database');

log('=== Phase 2: Building change interface database ===');

var changeDb = await agent(
  [
    '## Build P0-P3 Refactoring Change Interface Database',
    '',
    'Scan the git diff between the current branch and the pre-refactoring state to catalog ALL interface changes.',
    '',
    '### Step 1: Get git log of changed files',
    'Run: git diff --name-only HEAD~10..HEAD  (approximately last 10 commits)',
    'Or: git diff --stat origin/main..HEAD if available',
    'List ALL changed Rust files.',
    '',
    '### Step 2: Categorize changes',
    'For each changed file, determine the TYPE of change:',
    '',
    'A. Tauri Command Changes (frontend-visible):',
    '   - Commands added: new #[tauri::command] functions',
    '   - Commands removed: deleted #[tauri::command] functions',
    '   - Commands renamed: changed function names',
    '   - Command signature changes: parameter types or return types changed',
    '',
    'B. Module Structure Changes (import-path visible):',
    '   - Files moved: old_path → new_path',
    '   - Files split: single_file.rs → module/ directory',
    '   - Re-exports changed: pub use paths',
    '',
    'C. Type Changes:',
    '   - Struct field additions/removals/renames',
    '   - Trait method changes',
    '   - Error type changes (new/removed variants)',
    '',
    'D. Dependency Changes:',
    '   - Crate version upgrades',
    '   - API migrations required',
    '',
    '### Step 3: Build the database file',
    'Write a structured JSON database to: C:\\deep-student\\docs\\analysis\\CHANGE_INTERFACE_DB.json',
    '',
    'Format:',
    '{',
    '  "version": "1.0",',
    '  "generated": "2026-06-01",',
    '  "changed_commands": {',
    '    "added": [{"name": "...", "module": "...", "params": "...", "returns": "..."}],',
    '    "removed": [{"name": "...", "was_in": "..."}],',
    '    "renamed": [{"old": "...", "new": "...", "module": "..."}],',
    '    "signature_changed": [{"name": "...", "old_sig": "...", "new_sig": "..."}]',
    '  },',
    '  "changed_types": [{"type": "...", "change": "...", "old": "...", "new": "..."}],',
    '  "moved_modules": [{"old_path": "...", "new_path": "..."}],',
    '  "dependency_changes": [{"crate": "...", "old_ver": "...", "new_ver": "...", "breaking_changes": ["..."]}]',
    '}',
    '',
    '### Step 4: Also write a human-readable summary',
    'Write: C:\\deep-student\\docs\\analysis\\CHANGE_INTERFACE_SUMMARY.md'
  ].join('\n'),
  { label: 'change_database', schema: {
    type: 'object',
    properties: {
      total_changes: { type: 'number' },
      commands_added: { type: 'number' },
      commands_removed: { type: 'number' },
      commands_renamed: { type: 'number' },
      signature_changes: { type: 'number' },
      type_changes: { type: 'number' },
      module_moves: { type: 'number' },
      dep_changes: { type: 'number' },
      db_path: { type: 'string' },
      summary_path: { type: 'string' }
    },
    required: ['total_changes', 'db_path']
  }}
);

if (changeDb) {
  log('Change DB: ' + changeDb.total_changes + ' interface changes cataloged');
  log('  Commands: +' + changeDb.commands_added + ' / -' + changeDb.commands_removed + ' / renamed:' + changeDb.commands_renamed);
  log('  Types: ' + changeDb.type_changes + ' / Modules: ' + changeDb.module_moves + ' / Deps: ' + changeDb.dep_changes);
  log('  DB: ' + changeDb.db_path);
}

// ============================================================
// Phase 3: Frontend Impact Scan
// ============================================================
phase('Phase 3: Frontend Impact Scan');

log('=== Phase 3: Scanning frontend for impacted interfaces ===');

var frontendScan = await agent(
  [
    '## Scan TypeScript/TSX Frontend Against Change Database',
    '',
    'Read the change database: C:\\deep-student\\docs\\analysis\\CHANGE_INTERFACE_DB.json',
    '',
    '### For EACH changed Tauri command that was REMOVED or RENAMED:',
    '',
    '1. Search ALL .ts and .tsx files under C:\\deep-student\\src\\ for the OLD command name',
    '   - grep for the command string (e.g., "unified_search_cards", "chat_v2_send")',
    '   - The commands are typically called via invoke("command_name", {params})',
    '   - or via wrapper functions in src/utils/*Api.ts files',
    '',
    '2. Flag every file that references a removed/renamed command',
    '',
    '### For EACH changed Tauri command that was ADDED:',
    '',
    '1. Check if the frontend already has TypeScript bindings for it',
    '2. Flag commands that need new bindings',
    '',
    '### For EACH type change:',
    '',
    '1. Check if the changed Rust struct has a TypeScript interface counterpart',
    '2. Search for usage of the old field names in .ts/.tsx files',
    '',
    '### Output:',
    'Write a detailed impact report to: C:\\deep-student\\docs\\analysis\\FRONTEND_IMPACT_REPORT.md',
    'List EVERY file that needs changes, with the exact change needed.'
  ].join('\n'),
  { label: 'frontend_scan', schema: {
    type: 'object',
    properties: {
      total_files_scanned: { type: 'number' },
      files_impacted: { type: 'number' },
      commands_broken: { type: 'number' },
      types_broken: { type: 'number' },
      impact_by_file: { type: 'array', items: { type: 'object', properties: {
        file: { type: 'string' },
        issue_count: { type: 'number' },
        issues: { type: 'array', items: { type: 'string' } }
      }}},
      report_path: { type: 'string' }
    },
    required: ['total_files_scanned', 'files_impacted']
  }}
);

if (frontendScan) {
  log('Frontend scan: ' + frontendScan.total_files_scanned + ' files scanned, ' + frontendScan.files_impacted + ' impacted');
  log('  Broken commands: ' + frontendScan.commands_broken + ' / Broken types: ' + frontendScan.types_broken);
}

// ============================================================
// Phase 4: Frontend Fixes (with strict TS syntax verification)
// ============================================================
phase('Phase 4: Frontend Fixes');

log('=== Phase 4: Fixing frontend code with strict TS syntax ===');

var frontendFixes = await agent(
  [
    '## Fix Frontend Code Against Change Database',
    '',
    'Read the impact report: C:\\deep-student\\docs\\analysis\\FRONTEND_IMPACT_REPORT.md',
    '',
    '### Task: Fix ALL impacted frontend files',
    '',
    'For EACH impacted file:',
    '1. Read the file',
    '2. Apply the correct fix based on the change database:',
    '   - Removed command: either remove the call site, add a stub, or use the replacement',
    '   - Renamed command: update invoke() call to new name',
    '   - Changed type: update TypeScript interface to match new Rust struct fields',
    '   - Added command: add TypeScript binding wrapper function if needed',
    '',
    '3. STRICT TypeScript SYNTAX VERIFICATION after EACH fix:',
    '   - Verify imports resolve to existing files/modules',
    '   - Verify function parameter types are consistent',
    '   - Verify return type annotations match',
    '   - Verify no undefined variables or types',
    '   - Check for dangling references to removed functions',
    '   - Verify JSX/TSX syntax is valid (no unclosed tags)',
    '   - Verify template literal syntax is valid',
    '',
    '### Priority:',
    '1. Fix BROKEN commands (runtime errors) — highest priority',
    '2. Fix type mismatches (compile warnings) — medium priority',
    '3. Add bindings for new commands — low priority',
    '',
    '### CRITICAL RULES:',
    '- Every fix must produce valid TypeScript',
    '- Check: imports exist, types are defined, no circular imports introduced',
    '- Do NOT change component logic — only update API calls and type definitions',
    '- If an invoke() call target truly no longer exists and has no replacement, wrap in try/catch with console.warn',
    '',
    'Report every file changed and the exact fix applied.'
  ].join('\n'),
  { label: 'frontend_fixes', schema: {
    type: 'object',
    properties: {
      files_fixed: { type: 'number' },
      commands_restored: { type: 'number' },
      types_updated: { type: 'number' },
      bindings_added: { type: 'number' },
      strict_syntax_verified: { type: 'boolean' },
      files_modified: { type: 'array', items: { type: 'string' } },
      issues_remaining: { type: 'array', items: { type: 'string' } }
    },
    required: ['files_fixed', 'strict_syntax_verified']
  }}
);

if (frontendFixes) {
  log('Frontend fixes: ' + frontendFixes.files_fixed + ' files fixed');
  log('  Commands restored: ' + frontendFixes.commands_restored);
  log('  Types updated: ' + frontendFixes.types_updated);
  log('  Bindings added: ' + frontendFixes.bindings_added);
  log('  Strict TS syntax verified: ' + frontendFixes.strict_syntax_verified);
}

// ============================================================
// Phase 5: Final Verification
// ============================================================
phase('Phase 5: Final Verification');

log('=== Phase 5: Final cargo check + tsc verification ===');

var finalCheck = await parallel([
  // 5a: Rust cargo check
  function() {
    return agent(
      [
        '## Final Rust Cargo Check',
        '',
        'After warning cleanup, run cargo check:',
        'cd C:/deep-student/src-tauri && cargo check 2>&1',
        'timeout: 600000ms',
        '',
        'Expected: 0 errors, significantly reduced warnings.',
        'If errors appear, fix them (minimal fix, valid Rust).',
        'Re-run until clean.',
        '',
        'Save output to: C:/deep-student/src-tauri/cargo_check_final.txt (overwrite)',
        '',
        'Return: error count, warning count, pass/fail status.'
      ].join('\n'),
      { label: 'rust_final_check', schema: {
        type: 'object',
        properties: {
          passed: { type: 'boolean' },
          errors: { type: 'number' },
          warnings: { type: 'number' },
          fixed_this_round: { type: 'number' }
        },
        required: ['passed', 'errors', 'warnings']
      }}
    );
  },

  // 5b: TypeScript type check (if tsc is available)
  function() {
    return agent(
      [
        '## TypeScript Type Check for Frontend Changes',
        '',
        'Run TypeScript type check on the entire frontend:',
        '',
        '1. Check if npx tsc --noEmit is available (read tsconfig.json for config)',
        '2. Run: cd C:/deep-student && npx tsc --noEmit 2>&1 | head -500',
        '   timeout: 300000ms',
        '',
        '3. Parse output:',
        '   - Count TS errors by file',
        '   - Identify errors caused by OUR changes (not pre-existing)',
        '   - Fix any errors caused by our refactoring',
        '',
        '4. Report:',
        '   - Total TS errors before our fixes',
        '   - Errors caused by our changes',
        '   - Errors fixed',
        '   - Errors remaining (pre-existing, not ours)',
        '',
        'Save output to: C:/deep-student/tsc_check_final.txt',
        '',
        'CRITICAL: Only fix errors introduced by our changes. Do NOT fix pre-existing TS errors.'
      ].join('\n'),
      { label: 'tsc_check', schema: {
        type: 'object',
        properties: {
          tsc_available: { type: 'boolean' },
          total_errors: { type: 'number' },
          our_errors_found: { type: 'number' },
          our_errors_fixed: { type: 'number' },
          pre_existing_errors: { type: 'number' },
          status: { type: 'string' }
        },
        required: ['tsc_available', 'total_errors', 'status']
      }}
    );
  }
]);

var rustCheck = finalCheck[0];
var tsCheck = finalCheck[1];

log('');
log('========================================');
log('FINAL VERIFICATION RESULTS');
log('========================================');
if (rustCheck) {
  log('Rust: ' + (rustCheck.passed ? 'PASSED' : 'FAILED') + ' | errors:' + rustCheck.errors + ' | warnings:' + rustCheck.warnings);
}
if (tsCheck) {
  log('TypeScript: ' + tsCheck.status + ' | total errors:' + tsCheck.total_errors + ' | our errors fixed:' + tsCheck.our_errors_fixed);
}

log('');
log('========================================');
log('WARNINGS CLEANUP + FRONTEND SYNC COMPLETE');
log('========================================');

return {
  warnings: warnResults,
  change_db: changeDb,
  frontend_scan: frontendScan,
  frontend_fixes: frontendFixes,
  final: finalCheck
};