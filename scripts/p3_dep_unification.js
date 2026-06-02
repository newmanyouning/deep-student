export const meta = {
  name: 'p3-dependency-unification',
  description: 'P3：联网检查依赖升级影响 → 分批升级 → 语法验证。每代理<50%上下文窗口',
  phases: [
    { title: 'Phase 1: Web Research', detail: '联网搜索reqwest/oauth2/tungstenite升级的breaking changes' },
    { title: 'Phase 2: Staged Upgrades', detail: '分3批升级依赖，每批独立代理验证语法' },
    { title: 'Phase 3: Final Verification', detail: '全局语法检查 + 导入完整性验证' }
  ]
};

// ============================================================
// Phase 1: Web Research — Breaking changes for each upgrade
// ============================================================
phase('Phase 1: Web Research');

log('=== Phase 1: Web search for upgrade impacts ===');

var researchResults = await parallel([
  function() {
    return agent(
      [
        'WebSearch for the following and compile a structured report:',
        '',
        '1. "reqwest 0.11 to 0.12 migration guide breaking changes"',
        '   - What API changes between reqwest 0.11 and 0.12?',
        '   - Is reqwest::Client::new() API identical?',
        '   - Does reqwest::ClientBuilder change?',
        '   - Does the Error type remain compatible?',
        '   - Does the header module (reqwest::header) change?',
        '   - Does multipart API change?',
        '   - Does streaming/response API change?',
        '   - Any TLS/rustls implications?',
        '',
        '2. "oauth2 crate 4.4 to 5.0 migration"',
        '   - What breaking changes in oauth2 5.x?',
        '   - Does it support reqwest 0.12?',
        '   - API changes in token exchange?',
        '',
        '3. "reqwest-eventsource 0.5 to 0.6 breaking changes"',
        '   - Does 0.6 support reqwest 0.12?',
        '   - API changes in EventSource?',
        '',
        'For each: list ALL breaking changes, with before/after code examples.',
        'Return structured data.'
      ].join('\n'),
      { label: 'web_research', schema: {
        type: 'object',
        properties: {
          reqwest_changes: { type: 'array', items: { type: 'object', properties: {
            area: { type: 'string' },
            old_api: { type: 'string' },
            new_api: { type: 'string' },
            severity: { type: 'string' }
          }}},
          oauth2_changes: { type: 'array', items: { type: 'object', properties: {
            area: { type: 'string' },
            old_api: { type: 'string' },
            new_api: { type: 'string' },
            severity: { type: 'string' }
          }}},
          eventsource_changes: { type: 'array', items: { type: 'object', properties: {
            area: { type: 'string' },
            old_api: { type: 'string' },
            new_api: { type: 'string' },
            severity: { type: 'string' }
          }}},
          overall_risk: { type: 'string', enum: ['low', 'medium', 'high'] },
          recommendation: { type: 'string' }
        },
        required: ['reqwest_changes', 'overall_risk', 'recommendation']
      }}
    );
  },

  function() {
    return agent(
      [
        '### Task: Scan current codebase for ALL reqwest usage patterns',
        '',
        'Search ALL .rs files under C:\\deep-student\\src-tauri\\src\\ for reqwest imports and usage.',
        'Categorize every usage into:',
        '',
        '1. Client construction: reqwest::Client::new(), reqwest::Client::builder()',
        '2. Request building: client.get(url), client.post(url), .header(), .json(), .body()',
        '3. Response handling: .send().await, .text().await, .json().await',
        '4. Header usage: reqwest::header::*, HeaderMap, HeaderValue',
        '5. Error handling: reqwest::Error in From impls or match arms',
        '6. Multipart: reqwest::multipart',
        '7. Streaming: .bytes_stream(), chunk()',
        '8. Blocking: reqwest::blocking',
        '9. TLS config: any custom TLS setup',
        '10. Url: reqwest::Url (re-exported from url crate)',
        '',
        'For each category, count occurrences and list file:line.',
        'This tells us EXACTLY which files need changes and for what reason.',
        '',
        'Return a detailed breakdown by category and file.'
      ].join('\n'),
      { label: 'reqwest_usage_scan', schema: {
        type: 'object',
        properties: {
          total_files_using_reqwest: { type: 'number' },
          categories: { type: 'array', items: { type: 'object', properties: {
            category: { type: 'string' },
            count: { type: 'number' },
            files: { type: 'array', items: { type: 'string' } },
            api_change_expected: { type: 'boolean' }
          }}},
          oauth2_files: { type: 'array', items: { type: 'string' } },
          eventsource_files: { type: 'array', items: { type: 'string' } },
          riskiest_files: { type: 'array', items: { type: 'object', properties: {
            file: { type: 'string' },
            reason: { type: 'string' }
          }}}
        },
        required: ['total_files_using_reqwest', 'categories']
      }}
    );
  }
]);

var research = researchResults[0];
var usage = researchResults[1];

if (research) {
  log('Web research: ' + research.reqwest_changes.length + ' reqwest changes, ' + research.oauth2_changes.length + ' oauth2 changes, risk: ' + research.overall_risk);
  log('Recommendation: ' + research.recommendation);
}
if (usage) {
  log('Usage scan: ' + usage.total_files_using_reqwest + ' files use reqwest across ' + usage.categories.length + ' categories');
}

// ============================================================
// Phase 2: Staged Upgrades (sequential — order matters)
// ============================================================
phase('Phase 2: Staged Upgrades');

log('=== Phase 2: 3-stage sequential dependency upgrade ===');

var UPGRADE_SCHEMA = {
  type: 'object',
  properties: {
    stage: { type: 'string' },
    status: { type: 'string', enum: ['upgraded', 'deferred', 'failed'] },
    files_changed: { type: 'number' },
    crates_upgraded: { type: 'array', items: { type: 'object', properties: {
      crate_name: { type: 'string' },
      old_version: { type: 'string' },
      new_version: { type: 'string' },
      code_changes_needed: { type: 'boolean' }
    }}},
    api_breakages_found: { type: 'number' },
    api_breakages_fixed: { type: 'number' },
    syntax_verified: { type: 'boolean' },
    issues: { type: 'array', items: { type: 'string' } }
  },
  required: ['stage', 'status', 'syntax_verified']
};

// Stage 1: Upgrade oauth2 + reqwest-eventsource (enablers for reqwest 0.12)
log('Stage 1: Upgrading oauth2 + reqwest-eventsource...');
var stage1Result = await agent(
  [
    '## Stage 1: Upgrade oauth2 + reqwest-eventsource (reqwest 0.12 enablers)',
    '',
    'These two crates lock reqwest 0.11. Upgrading them unblocks reqwest 0.12.',
    '',
    '### 1a. Upgrade oauth2 4.x → 5.x',
    'Read C:\\deep-student\\src-tauri\\Cargo.toml — find oauth2 dependency.',
    'Change version to 5.0.',
    '',
    'Search ALL .rs files for oauth2 imports. Check API compatibility:',
    '- The main oauth2 API (Client::new, exchange_code, etc.) typically stable across major versions',
    '- If code uses BasicClient or standard OAuth2 flow, migration is straightforward',
    '- Apply fixes if any API method signature changed',
    '',
    '### 1b. Upgrade reqwest-eventsource 0.5 → 0.6',
    'Change reqwest-eventsource version in Cargo.toml to 0.6.',
    'Search ALL .rs files for reqwest_eventsource imports.',
    '- EventSource API generally stable',
    '- Apply fixes if needed',
    '',
    '### CRITICAL:',
    '- Do NOT upgrade reqwest — only the enablers',
    '- Verify Cargo.toml is consistent after changes',
    '- Report exact files changed and any API breakage found'
  ].join('\n'),
  { label: 'stage1_enablers', schema: UPGRADE_SCHEMA }
);

if (stage1Result) {
  log('  Stage 1: ' + stage1Result.status + ', ' + stage1Result.files_changed + ' files, ' + (stage1Result.crates_upgraded ? stage1Result.crates_upgraded.length : 0) + ' crates');
}

// Stage 2: Upgrade reqwest 0.11 → 0.12
log('Stage 2: Upgrading reqwest 0.11→0.12...');
var reqwestChanges = research ? research.reqwest_changes || [] : [];
var changesStr = '';
for (var i = 0; i < reqwestChanges.length; i++) {
  changesStr += '- ' + reqwestChanges[i].area + ': ' + reqwestChanges[i].old_api + ' -> ' + reqwestChanges[i].new_api + ' (severity: ' + reqwestChanges[i].severity + ')\n';
}

var stage2Result = await agent(
  [
    '## Stage 2: Upgrade reqwest 0.11 → 0.12',
    '',
    '### Step 1: Update Cargo.toml',
    'Read C:\\deep-student\\src-tauri\\Cargo.toml. Change reqwest from 0.11 to 0.12.',
    'Keep ALL feature flags identical: json, rustls-tls, stream, blocking, multipart',
    '',
    '### Step 2: Known breaking changes from web research:',
    changesStr,
    '',
    '### Step 3: Fix ALL files using reqwest',
    'Based on Phase 1 usage scan, approximately ' + (usage ? usage.total_files_using_reqwest : '~20') + ' files use reqwest.',
    '',
    'For EACH file using reqwest under src-tauri/src/:',
    '1. Read the file',
    '2. Identify reqwest API usage',
    '3. Apply API migration if needed:',
    '   - Client::new(), Client::builder() — UNCHANGED in 0.12',
    '   - reqwest::Error type — UNCHANGED',
    '   - reqwest::header module — UNCHANGED',
    '   - reqwest::multipart — UNCHANGED',
    '   - Response::text(), ::json() — UNCHANGED',
    '   - reqwest::Url (re-export from url crate) — UNCHANGED',
    '   - Streaming: ::bytes_stream() — minor change: returns Stream of Bytes directly',
    '4. Verify each change is syntactically correct Rust',
    '',
    '### Step 4: Post-upgrade verification',
    'Search for any remaining reqwest 0.11 API patterns that would fail in 0.12.',
    'Ensure no file imports from reqwest 0.11-specific paths.',
    '',
    '### CRITICAL: Preserve ALL existing error handling, function signatures, and behavior.'
  ].join('\n'),
  { label: 'stage2_reqwest', schema: UPGRADE_SCHEMA }
);

if (stage2Result) {
  log('  Stage 2: ' + stage2Result.status + ', ' + stage2Result.files_changed + ' files, ' + (stage2Result.crates_upgraded ? stage2Result.crates_upgraded.length : 0) + ' crates');
}

// Stage 3: Verify consolidation
log('Stage 3: Final consolidation...');
var stage3Result = await agent(
  [
    '## Stage 3: Final consolidation and verification',
    '',
    'After reqwest upgrade, verify the dependency graph and code integrity.',
    '',
    '### Task 1: Cargo.toml consistency',
    'Read C:\\deep-student\\src-tauri\\Cargo.toml:',
    '- Verify reqwest is at 0.12',
    '- Verify oauth2 is at 5.0',
    '- Verify reqwest-eventsource is at 0.6',
    '- Check no stale old-version comments or references',
    '',
    '### Task 2: Code syntax verification',
    'Scan ALL .rs files that reference reqwest for correct 0.12 API usage:',
    '- Search for reqwest:: — verify all usage matches 0.12 API',
    '- Search for any oauth2:: — verify 5.0 compatible',
    '- Search for reqwest_eventsource:: — verify 0.6 compatible',
    '- Verify no 0.11-specific API patterns remain',
    '',
    '### Task 3: Consolidation check',
    'These crates should now have FEWER duplicate versions:',
    '- hyper: oauth2 4.x locked hyper 0.14 — now should be single version',
    '- http: similar consolidation expected',
    '- base64: oauth2 4.x locked base64 0.13 — should reduce',
    '',
    '### Task 4: Functional verification',
    'Verify key modules are intact:',
    '- providers/mod.rs: ProviderAdapter uses reqwest for HTTP',
    '- cloud_storage/: uses reqwest + oauth2',
    '- mcp/: SSE transport uses reqwest_eventsource',
    '- All #[tauri::command] functions that use reqwest',
    '',
    'Report all findings.'
  ].join('\n'),
  { label: 'stage3_verify', schema: UPGRADE_SCHEMA }
);

if (stage3Result) {
  log('  Stage 3: ' + stage3Result.status + ', ' + stage3Result.files_changed + ' files checked');
}

var upgradeResults = [stage1Result, stage2Result, stage3Result];

var totalUpgraded = 0;
var totalFilesChanged = 0;
var allVerified = true;
for (var k = 0; k < upgradeResults.length; k++) {
  var r = upgradeResults[k];
  if (r) {
    log('Stage ' + r.stage + ': ' + r.status + ', ' + r.files_changed + ' files, ' + (r.crates_upgraded ? r.crates_upgraded.length : 0) + ' crates');
    totalFilesChanged += r.files_changed || 0;
    if (r.crates_upgraded) totalUpgraded += r.crates_upgraded.length;
    if (!r.syntax_verified) allVerified = false;
    if (r.issues && r.issues.length > 0) {
      for (var j = 0; j < r.issues.length; j++) log('  ⚠ ' + r.issues[j]);
    }
  }
}

log('Phase 2 complete: ' + totalUpgraded + ' crates upgraded, ' + totalFilesChanged + ' files adapted');

// ============================================================
// Phase 3: Final verification
// ============================================================
phase('Phase 3: Final Verification');

log('=== Phase 3: Global syntax and import verification ===');

var finalVerify = await agent(
  [
    '## P3 Final Verification',
    '',
    'After dependency upgrades across 3 stages, perform a final comprehensive check:',
    '',
    '### 1. Import integrity',
    'Search all modified files for any broken use statements.',
    'Key modules that may have import changes:',
    '- All files using reqwest (the ~' + (usage ? usage.total_files_using_reqwest : '~20') + ' files identified in Phase 1)',
    '- All files that import from oauth2',
    '- All files that import from reqwest_eventsource',
    '',
    '### 2. Error type compatibility',
    'Verify reqwest::Error is still compatible with all From impls:',
    '- Check error.rs files that have From<reqwest::Error> impls',
    '- Verify no error type paths changed',
    '',
    '### 3. Function signature preservation',
    'Scan all modified files — every public function must have identical signature to before.',
    'Compare with git diff to verify no unintended signature changes.',
    '',
    '### 4. Cargo.toml consistency',
    'Read C:\\deep-student\\src-tauri\\Cargo.toml — verify:',
    '- All upgraded crates have correct version',
    '- No leftover old version references',
    '- Features are correct for new versions',
    '',
    '### 5. Report',
    'Output a structured verification report.',
    'If any issues found, fix them or flag them.'
  ].join('\n'),
  { label: 'final_verify', schema: {
    type: 'object',
    properties: {
      all_imports_ok: { type: 'boolean' },
      all_errors_compatible: { type: 'boolean' },
      all_signatures_preserved: { type: 'boolean' },
      cargo_toml_consistent: { type: 'boolean' },
      issues_found: { type: 'number' },
      issues_fixed: { type: 'number' },
      remaining_concerns: { type: 'array', items: { type: 'string' } }
    },
    required: ['all_imports_ok', 'all_signatures_preserved', 'cargo_toml_consistent']
  }}
);

if (finalVerify) {
  log('Final verification:');
  log('  Imports OK: ' + finalVerify.all_imports_ok);
  log('  Errors compatible: ' + finalVerify.all_errors_compatible);
  log('  Signatures preserved: ' + finalVerify.all_signatures_preserved);
  log('  Cargo.toml consistent: ' + finalVerify.cargo_toml_consistent);
  log('  Issues found/fixed: ' + finalVerify.issues_found + '/' + finalVerify.issues_fixed);
}

log('');
log('========================================');
log('P3 DEPENDENCY UNIFICATION COMPLETE');
log('========================================');
log('');
log('Crates upgraded: ' + totalUpgraded);
log('Files adapted: ' + totalFilesChanged);
log('All syntax verified: ' + allVerified);

return {
  research: research,
  usage_scan: usage,
  upgrades: upgradeResults,
  verification: finalVerify
};