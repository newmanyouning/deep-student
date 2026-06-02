export const meta = {
  name: 'ci-config-check-and-cargo-check',
  description: '检查CI配置 → 清理编译产物 → 运行cargo check验证累积变更',
  phases: [
    { title: 'Phase 1: CI Config Audit', detail: '检查GitHub CI workflow配置是否适合测试构建' },
    { title: 'Phase 2: Clean Build Artifacts', detail: '清理46GB target目录 + stale error文件' },
    { title: 'Phase 3: Cargo Check', detail: '运行cargo check，按模块组收集并修复编译错误' }
  ]
};

// ============================================================
// Phase 1: CI Configuration Audit
// ============================================================
phase('Phase 1: CI Config Audit');

log('=== Phase 1: Checking CI configuration ===');

var ciResult = await agent(
  [
    '## CI Configuration Audit for Test Build',
    '',
    'Read and audit the following CI workflow files for test build suitability:',
    '',
    '1. C:\\deep-student\\.github\\workflows\\ci.yml',
    '2. C:\\deep-student\\.github\\workflows\\build-test.yml',
    '3. C:\\deep-student\\src-tauri\\Cargo.toml (dev-dependencies, features, profiles)',
    '',
    '### Current state:',
    '- ci.yml: backend job runs `cargo check` on ubuntu, has sync tests matrix',
    '- build-test.yml: runs FULL `npx tauri build` (not cargo check) for 4 platforms + android',
    '- There is NO dedicated test-build workflow that just does `cargo check`',
    '',
    '### Tasks:',
    '',
    '### A: Fix build-test.yml — Add cargo check step BEFORE full build',
    'The build-test.yml runs `npx tauri build` which takes 30-60 min. If compilation fails, it wastes CI minutes.',
    'Add a `cargo check` step BEFORE the build step so syntax/type errors fail fast (<5 min vs 60 min).',
    '',
    'Add after the "Disable updater signing" step (around line 46):',
    '```yaml',
    '      - name: Fast compile check (fail early on syntax/type errors)',
    '        run: cargo check',
    '        working-directory: src-tauri',
    '```',
    'Do this for ALL 5 platform jobs (windows, macos-arm64, macos-x64, linux, android).',
    '',
    '### B: Check Cargo.toml for test profile',
    'Read C:\\deep-student\\src-tauri\\Cargo.toml — look for [profile.test] or [profile.dev] sections.',
    'If [profile.dev] exists, check opt-level and debug settings.',
    'If no test profile, add one:',
    '```toml',
    '[profile.dev]',
    'opt-level = 0     # fast compile, no optimization',
    'debug = 1         # minimal debug info for faster compile',
    '',
    '[profile.test]',
    'opt-level = 0',
    'debug = 1',
    '```',
    '',
    '### C: Verify CI env has all needed system deps for cargo check',
    'Check that CI backend job has libwebkit2gtk-4.1-dev, protobuf-compiler, etc.',
    'These are already present in ci.yml but verify they are correct.',
    '',
    '### D: Add CARGO_TERM_COLOR=always to CI for readable output',
    '',
    'Apply all changes. Report what was added/modified.'
  ].join('\n'),
  { label: 'ci_audit', schema: {
    type: 'object',
    properties: {
      build_test_updated: { type: 'boolean' },
      cargo_toml_updated: { type: 'boolean' },
      ci_updated: { type: 'boolean' },
      changes_summary: { type: 'array', items: { type: 'string' } },
      files_modified: { type: 'array', items: { type: 'string' } }
    },
    required: ['build_test_updated', 'changes_summary']
  }}
);

if (ciResult) {
  log('CI audit complete:');
  log('  build-test.yml updated: ' + ciResult.build_test_updated);
  log('  Cargo.toml updated: ' + ciResult.cargo_toml_updated);
  for (var i = 0; i < (ciResult.changes_summary || []).length; i++) {
    log('  - ' + ciResult.changes_summary[i]);
  }
}

// ============================================================
// Phase 2: Clean Build Artifacts (local)
// ============================================================
phase('Phase 2: Clean Build Artifacts');

log('=== Phase 2: Cleaning build artifacts ===');

// We need to run bash commands locally to clean up
// Use a sequential agent approach for deterministic cleanup

var cleanupResult = await agent(
  [
    '## Clean Build Artifacts',
    '',
    'The current src-tauri/target directory is 46GB. Clean it completely.',
    'Also delete stale error files.',
    '',
    '### Commands to run (use Bash tool, run sequentially):',
    '',
    '1. Delete the entire target directory:',
    '   rm -rf C:/deep-student/src-tauri/target',
    '   (WARNING: this frees ~46GB of disk space)',
    '',
    '2. Delete stale error files:',
    '   rm -f C:/deep-student/src-tauri/cargo_errors.txt',
    '   rm -f C:/deep-student/src-tauri/cargo_full.txt',
    '',
    '3. Verify cleanup:',
    '   ls C:/deep-student/src-tauri/target 2>&1 || echo "target/ deleted successfully"',
    '   ls C:/deep-student/src-tauri/cargo_errors.txt 2>&1 || echo "cargo_errors.txt deleted"',
    '   ls C:/deep-student/src-tauri/cargo_full.txt 2>&1 || echo "cargo_full.txt deleted"',
    '',
    '4. Report freed disk space:',
    '   (target was ~46G, now freed)',
    '',
    'IMPORTANT: Run each command one at a time. Verify each step before proceeding.'
  ].join('\n'),
  { label: 'clean_build_artifacts', schema: {
    type: 'object',
    properties: {
      target_deleted: { type: 'boolean' },
      error_files_deleted: { type: 'boolean' },
      disk_space_freed: { type: 'string' },
      errors_encountered: { type: 'array', items: { type: 'string' } }
    },
    required: ['target_deleted']
  }}
);

if (cleanupResult) {
  log('Cleanup: target deleted=' + cleanupResult.target_deleted + ', error files deleted=' + cleanupResult.error_files_deleted);
}

// ============================================================
// Phase 3: Cargo Check (sequential, single compilation)
// ============================================================
phase('Phase 3: Cargo Check');

log('=== Phase 3: Running cargo check ===');
log('IMPORTANT: Only ONE cargo check runs at a time. No parallel compilation.');
log('This will take 5-15 minutes on first clean build.');

var checkResult = await agent(
  [
    '## Run Cargo Check to Verify All Cumulative Changes',
    '',
    '### Context:',
    'Over P0-P3 refactoring, ~100+ files were modified across the Tauri backend:',
    '- P0: Dead code deletion, 30 unused imports removed, Cargo features cleaned, 5 dep upgrades',
    '- P1: 4 god files split into 43 domain files, DSTU dedup',
    '- P2: VFS coordinator, Memory trait, pipeline merge, executor DI, adapter merge, MemoryService dual-constructor',
    '- P3: reqwest 0.11→0.12, oauth2 4.4→5.0, reqwest-eventsource 0.5→0.6',
    '',
    'ALL of these changes need verification.',
    '',
    '### Instructions:',
    '',
    '1. FIRST: run cargo check and capture ALL output:',
    '   cd C:/deep-student/src-tauri && cargo check 2>&1 | tee cargo_check_output.txt',
    '   This will take a while on first run (clean target, no cache).',
    '   Run with: timeout 600000 (10 minute timeout)',
    '',
    '2. IF cargo check SUCCEEDS (exit code 0):',
    '   - Report: "All P0-P3 changes verified — cargo check passed"',
    '   - List any warnings (unused imports, deprecation notices)',
    '   - Save output to C:/deep-student/src-tauri/cargo_check_output.txt',
    '',
    '3. IF cargo check FAILS (exit code != 0):',
    '   - Parse the error output CAREFULLY',
    '   - Categorize errors by FILE and ERROR TYPE',
    '   - For EACH error file, FIX the error:',
    '     a. Read the file at the error location',
    '     b. Understand what broke (type mismatch? missing import? module not found?)',
    '     c. Apply the MINIMAL fix — preserve all logic and function signatures',
    '     d. Re-run cargo check to verify the fix',
    '   - Continue this fix-and-check loop until cargo check passes or all fixable errors are resolved',
    '',
    '4. After all fixes, save the final output to C:/deep-student/src-tauri/cargo_check_output.txt',
    '',
    '### CRITICAL RULES:',
    '- ONLY run ONE cargo check at a time (single compilation process)',
    '- Fix errors in dependency order: module structure errors first, then type errors, then warnings',
    '- Module structure: if "module not found" errors appear, fix mod.rs declarations first',
    '- Type errors: if E0308 appears, check if error types changed (String vs typed Error)',
    '- Import errors: if "unresolved import" appears, fix use statements',
    '- NEVER change function logic — only fix compilation issues',
    '- If an error requires significant refactoring, report it rather than attempting a risky fix',
    '',
    '### Expected common issues from our refactoring:',
    '- mod.rs declarations may need updating after god file splits (handlers directories)',
    '- pub use re-exports may have missed some items',
    '- oauth2 5.0 API changes in mcp/auth.rs (async_http_client → &reqwest::Client)',
    '- Unused import warnings (clean up if trivial)',
    '',
    'Return a DETAILED report of every error found and fixed, or confirmation that cargo check passed.'
  ].join('\n'),
  { label: 'cargo_check', schema: {
    type: 'object',
    properties: {
      cargo_check_passed: { type: 'boolean' },
      total_errors_found: { type: 'number' },
      total_errors_fixed: { type: 'number' },
      errors_remaining: { type: 'number' },
      warnings_count: { type: 'number' },
      errors_by_file: { type: 'array', items: { type: 'object', properties: {
        file: { type: 'string' },
        error_count: { type: 'number' },
        fixed_count: { type: 'number' },
        error_types: { type: 'array', items: { type: 'string' } }
      }}},
      fix_iterations: { type: 'number' },
      final_output_saved: { type: 'boolean' },
      summary: { type: 'string' }
    },
    required: ['cargo_check_passed', 'total_errors_found', 'total_errors_fixed']
  }}
);

if (checkResult) {
  log('');
  log('========================================');
  log('CARGO CHECK RESULTS');
  log('========================================');
  log('Passed: ' + checkResult.cargo_check_passed);
  log('Errors found: ' + checkResult.total_errors_found);
  log('Errors fixed: ' + checkResult.total_errors_fixed);
  log('Errors remaining: ' + checkResult.errors_remaining);
  log('Warnings: ' + checkResult.warnings_count);
  log('Fix iterations: ' + checkResult.fix_iterations);
  log('');
  log('Summary: ' + checkResult.summary);

  if (checkResult.errors_by_file && checkResult.errors_by_file.length > 0) {
    log('');
    log('Errors by file:');
    for (var i = 0; i < checkResult.errors_by_file.length; i++) {
      var ef = checkResult.errors_by_file[i];
      log('  ' + ef.file + ': ' + ef.error_count + ' errors, ' + ef.fixed_count + ' fixed');
    }
  }
}

log('');
log('========================================');
log('CI CHECK + CARGO VERIFICATION COMPLETE');
log('========================================');

return {
  ci: ciResult,
  cleanup: cleanupResult,
  cargo_check: checkResult
};