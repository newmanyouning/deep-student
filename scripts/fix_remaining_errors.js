export const meta = {
  name: 'fix-remaining-cargo-errors',
  description: '修复cargo check剩余108错误：DSTU handlers + MemoryStorage + Tauri2 API。严格Rust语法验证',
  phases: [
    { title: 'Phase 1: Scan All Errors', detail: '读取cargo check输出，按根因分类所有错误' },
    { title: 'Phase 2: Parallel Fix by Category', detail: '按错误类别并行修复：DSTU类型、MemoryStorage、Tauri2 API、杂项' },
    { title: 'Phase 3: Final Cargo Check', detail: '运行cargo check验证所有修复' }
  ]
};

// ============================================================
// Phase 1: Read error output and categorize
// ============================================================
phase('Phase 1: Scan All Errors');

log('=== Phase 1: Categorizing remaining 108 errors ===');

// Agent reads the cargo check output file and categorizes
var errorAnalysis = await agent(
  [
    '## Read cargo check output and build an error-by-file catalog.',
    '',
    'Read C:\\deep-student\\src-tauri\\cargo_check_output.txt',
    '',
    'For EVERY error, extract: file path, line number, error code, and error message.',
    'Group by FILE and by ROOT CAUSE.',
    '',
    'Expected root causes from previous run:',
    '1. DstuCreateOptions struct: folder_id/tags/file_data fields removed',
    '2. Missing repo functions: update_exam_content, update_translation, rename_folder, etc.',
    '3. MemoryStorage trait: get_conn/get_conn_safe not on Arc<dyn MemoryStorage>',
    '4. Type mismatches: various E0308 in DSTU handlers',
    '5. Tauri 2 API: emit(), try_state(), base64 encode, BytesText, i64 unwrap_or',
    '6. VfsUpdateMindMapParams: expected_updated_at, settings, version_source missing',
    '7. String as i64 casts',
    '',
    'Write a complete catalog to: C:\\deep-student\\docs\\analysis\\ERROR_CATALOG.md',
    '',
    'Return: total errors per category, and the exact file:line for every error.'
  ].join('\n'),
  { label: 'error_catalog', schema: {
    type: 'object',
    properties: {
      total_errors: { type: 'number' },
      categories: { type: 'array', items: { type: 'object', properties: {
        root_cause: { type: 'string' },
        error_count: { type: 'number' },
        files_affected: { type: 'array', items: { type: 'string' } },
        error_examples: { type: 'array', items: { type: 'string' } }
      }}}
    },
    required: ['total_errors', 'categories']
  }}
);

if (errorAnalysis) {
  log('Errors categorized into ' + errorAnalysis.categories.length + ' root causes:');
  for (var i = 0; i < errorAnalysis.categories.length; i++) {
    var cat = errorAnalysis.categories[i];
    log('  ' + cat.root_cause + ': ' + cat.error_count + ' errors in ' + cat.files_affected.length + ' files');
  }
}

// ============================================================
// Phase 2: Parallel Fix by Category (4 agents)
// ============================================================
phase('Phase 2: Parallel Fix by Category');

log('=== Phase 2: Fixing errors in parallel by category ===');

var FIX_SCHEMA = {
  type: 'object',
  properties: {
    category: { type: 'string' },
    errors_fixed: { type: 'number' },
    errors_unfixable: { type: 'number' },
    files_modified: { type: 'array', items: { type: 'string' } },
    syntax_verified: { type: 'boolean' },
    unfixable_reasons: { type: 'array', items: { type: 'string' } }
  },
  required: ['category', 'errors_fixed', 'syntax_verified']
};

var fixResults = await parallel([
  // Agent 1: Fix DstuCreateOptions + type mismatches in DSTU handlers
  function() {
    var prompt = [
      '## Fix DSTU Handler Errors: DstuCreateOptions + Type Mismatches',
      '',
      '### Context:',
      'During P1 god file decomposition, dstu/handlers.rs was split into 10 domain files.',
      'The new handler files reference DstuCreateOptions and repo functions that changed.',
      '',
      '### Root Cause 1: DstuCreateOptions field changes (~26 E0609 errors)',
      'The struct DstuCreateOptions may have lost fields (folder_id, tags, file_data) during refactoring.',
      '',
      'STEPS:',
      '1. Read C:\\deep-student\\src-tauri\\src\\dstu\\types.rs — find the DstuCreateOptions struct definition',
      '2. Note ALL fields it currently has',
      '3. Read ALL dstu/handlers/*.rs files — find every construction of DstuCreateOptions',
      '4. If fields are missing from the struct, ADD them back to DstuCreateOptions',
      '5. If the handler code uses old field names, UPDATE the handler code',
      '',
      'PREFERENCE: Add missing fields TO the struct rather than removing from handlers.',
      'This is backward-compatible and safer.',
      '',
      '### Root Cause 2: Missing repo functions (~30 E0599 errors)',
      'Handlers call functions like: update_exam_content, update_translation, rename_folder',
      'These may have moved to sub-modules or changed names.',
      '',
      'STEPS:',
      '1. For EACH missing function, search the dstu/ and vfs/ directories for where it is defined',
      '2. If found in a different module: add proper use import',
      '3. If renamed: use new name',
      '4. If truly gone: find the replacement (e.g., VfsCommandDb::generic_update)',
      '',
      '### Root Cause 3: Type mismatches (~19 E0308 errors)',
      'Various return type and argument type mismatches in DSTU handlers.',
      '',
      'STEPS:',
      '1. For EACH E0308 error, read the function signature',
      '2. Match the return type to what the function ACTUALLY returns',
      '3. Use DstuError::from() or .map_err() to bridge type gaps',
      '4. Prefer returning DstuResult<T> over Result<T, String>',
      '',
      '### STRICT RUST SYNTAX RULES:',
      '- Every fix must be valid Rust syntax',
      '- Check: all match arms exhaustive',
      '- Check: all return types match function signature',
      '- Check: all use imports resolve to valid paths',
      '- Check: no unused mut, no dead code warnings introduced',
      '- After ALL fixes, verify syntax: read each modified file and confirm valid Rust'
    ].join('\n');
    return agent(prompt, { label: 'dstu_type_fixes', schema: FIX_SCHEMA });
  },

  // Agent 2: Fix MemoryStorage trait issues
  function() {
    var prompt = [
      '## Fix MemoryStorage Trait Errors (~20 E0277 errors)',
      '',
      '### Context:',
      'P2 introduced MemoryStorage trait for memory VFS decoupling.',
      'Some DSTU handlers call .get_conn() / .get_conn_safe() on Arc<dyn MemoryStorage> but these methods are NOT on the trait.',
      '',
      '### STEPS:',
      '',
      '1. Read C:\\deep-student\\src-tauri\\src\\memory\\storage_trait.rs',
      '   Note: the trait already has conn() and conn_unchecked() methods.',
      '   But handlers are calling .get_conn() or .get_conn_safe() which are VfsDatabase methods.',
      '',
      '2. For EACH error site (E0277 on Arc<dyn MemoryStorage>):',
      '   - Replace vfs_db.get_conn() → storage.conn_unchecked()',
      '   - Replace vfs_db.get_conn_safe() → storage.conn()',
      '   - Ensure the returned connection type matches expected pattern',
      '',
      '3. Check if the storage trait needs additional methods:',
      '   - Read dstu/handlers/*.rs for ALL MemoryStorage method calls',
      '   - If a needed method is missing from the MemoryStorage trait, ADD it',
      '   - Then implement it in VfsMemoryStorage (thin wrapper)',
      '',
      '4. Verify all MemoryStorage::conn() calls are used correctly:',
      '   - conn() returns MemoryStorageConn which derefs to &Connection',
      '   - Handlers should be able to use it for SQL queries directly',
      '',
      '### STRICT RUST SYNTAX RULES:',
      '- Verify each modified function compiles (valid Rust)',
      '- All trait method signatures must be object-safe (no generics with impl Trait)',
      '- VfsMemoryStorage impl must match trait exactly',
      '- No type mismatches introduced'
    ].join('\n');
    return agent(prompt, { label: 'memory_storage_fixes', schema: FIX_SCHEMA });
  },

  // Agent 3: Fix Tauri 2 API + misc errors
  function() {
    var prompt = [
      '## Fix Tauri 2 API Changes + Misc Syntax Errors (~15 errors)',
      '',
      '### Error categories to fix:',
      '',
      '### 1. Tauri 2 emit() API (~5 errors)',
      'OLD: window.emit("event", payload).ok();',
      'NEW: window.emit("event", payload)?;  (in Tauri 2, emit returns Result)',
      '',
      'STEPS:',
      '- Search dstu/handlers/*.rs for .emit( — fix any that ignore the Result',
      '- Replace .ok() patterns with ? propagation or proper error handling',
      '',
      '### 2. Tauri 2 try_state() (~3 errors)',
      'OLD: app_handle.state::<Type>()',
      'NEW: app_handle.try_state::<Type>().ok_or(...)?',
      '',
      'STEPS:',
      '- Search for .state() calls in dstu/ and vfs/ handlers',
      '- Convert to try_state() pattern',
      '',
      '### 3. base64 encode API (P0 upgrade 0.21→0.22) (~3 errors)',
      'OLD: base64::encode(data)',
      'NEW: base64::engine::general_purpose::STANDARD.encode(data)',
      '',
      'STEPS:',
      '- Search for base64::encode and base64::decode calls',
      '- Convert to 0.22 API',
      '- Add use base64::Engine; where needed',
      '',
      '### 4. BytesText unescape (~2 errors)',
      'Method may have been renamed or removed in a newer crate version.',
      'Search for BytesText usage and find current API.',
      '',
      '### 5. i64 unwrap_or on String (~2 E0605 errors)',
      'Cannot cast String to i64. These are type errors in handler code.',
      'Find the actual expected type and fix the conversion.',
      '',
      '### 6. VfsUpdateMindMapParams missing fields (~2 E0063)',
      'The struct may have gained expected_updated_at, settings, version_source fields.',
      'Read the struct definition and add default values for missing fields.',
      '',
      '### STRICT RUST SYNTAX RULES:',
      '- Every fix MUST be valid Rust',
      '- Verify match exhaustiveness after changes',
      '- Check function signatures for correctness',
      '- No unwrap() without clear justification'
    ].join('\n');
    return agent(prompt, { label: 'tauri2_and_misc', schema: FIX_SCHEMA });
  }
]);

var totalFixed = 0;
var allSyntaxVerified = true;
for (var k = 0; k < fixResults.length; k++) {
  var r = fixResults[k];
  if (r) {
    log('Category [' + r.category + ']: ' + r.errors_fixed + ' fixed, ' + (r.errors_unfixable || 0) + ' unfixable, syntax verified: ' + r.syntax_verified);
    totalFixed += r.errors_fixed || 0;
    if (!r.syntax_verified) allSyntaxVerified = false;
    if (r.unfixable_reasons && r.unfixable_reasons.length > 0) {
      for (var j = 0; j < r.unfixable_reasons.length; j++) {
        log('  UNFIXABLE: ' + r.unfixable_reasons[j]);
      }
    }
  }
}
log('Phase 2 complete: ' + totalFixed + ' total errors fixed');

// ============================================================
// Phase 3: Final Cargo Check
// ============================================================
phase('Phase 3: Final Cargo Check');

log('=== Phase 3: Running final cargo check ===');

var finalCheck = await agent(
  [
    '## Final Cargo Check — Verify All Fixes',
    '',
    'All error categories have been fixed. Now run cargo check to verify.',
    '',
    '### Instructions:',
    '1. Run: cd C:/deep-student/src-tauri && cargo check 2>&1',
    '   timeout: 600000ms',
    '   This should be FAST since target/ was cleaned and incremental compilation works.',
    '',
    '2. If cargo check PASSES:',
    '   - Report success with warning count',
    '   - Save output to C:/deep-student/src-tauri/cargo_check_final.txt',
    '',
    '3. If cargo check FAILS:',
    '   - Count remaining errors by file',
    '   - For each error file (<5 remaining per file):',
    '     a. Read the file at error location',
    '     b. Apply minimal fix (valid Rust syntax)',
    '     c. The fix must be a single correct change',
    '   - Re-run cargo check after ALL fixes applied',
    '   - Loop until cargo check passes or <3 unfixable errors remain',
    '',
    '### STRICT RULES:',
    '- Single cargo check process at a time',
    '- Every fix must be valid Rust syntax',
    '- Report final error count precisely'
  ].join('\n'),
  { label: 'final_cargo_check', schema: {
    type: 'object',
    properties: {
      cargo_check_passed: { type: 'boolean' },
      errors_before: { type: 'number' },
      errors_after: { type: 'number' },
      errors_fixed_this_round: { type: 'number' },
      warnings: { type: 'number' },
      iterations: { type: 'number' },
      final_status: { type: 'string' }
    },
    required: ['cargo_check_passed', 'errors_before', 'errors_after', 'final_status']
  }}
);

if (finalCheck) {
  log('');
  log('========================================');
  log('FINAL CARGO CHECK RESULT');
  log('========================================');
  log('Passed: ' + finalCheck.cargo_check_passed);
  log('Errors before: ' + finalCheck.errors_before);
  log('Errors after: ' + finalCheck.errors_after);
  log('Fixed this round: ' + finalCheck.errors_fixed_this_round);
  log('Warnings: ' + finalCheck.warnings);
  log('Iterations: ' + finalCheck.iterations);
  log('Status: ' + finalCheck.final_status);
}

log('');
log('========================================');
log('ERROR FIX WORKFLOW COMPLETE');
log('========================================');

return {
  error_catalog: errorAnalysis,
  fixes: fixResults,
  final_check: finalCheck
};