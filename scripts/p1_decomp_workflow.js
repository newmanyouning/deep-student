export const meta = {
  name: 'p1-god-file-decomposition',
  description: 'P1 God File分解: vfs/handlers.rs + sync/mod.rs + llm_manager/mod.rs + dstu/handlers.rs -> 36+ files',
  phases: [
    { title: 'Phase 1: Big 3 Parallel Decomp', detail: 'VFS handlers + data_gov sync + llm_manager 并行分解' },
    { title: 'Phase 2: DSTU Decomposition', detail: 'DSTU handlers.rs 分解 + 去重' },
    { title: 'Phase 3: Imports Update and Verify', detail: '全局更新导入路径，验证向后兼容' }
  ]
};

var DECOMP_SCHEMA = {
  type: 'object',
  properties: {
    module: { type: 'string' },
    original_loc: { type: 'number' },
    new_mod_loc: { type: 'number' },
    files_created: { type: 'array', items: { type: 'string' } },
    total_new_loc: { type: 'number' },
    functions_moved: { type: 'number' },
    backward_compatible: { type: 'boolean' },
    issues: { type: 'array', items: { type: 'string' } }
  },
  required: ['module', 'files_created', 'backward_compatible']
};

// ============================================================
// Phase 1: Decompose 3 biggest god files in parallel
// ============================================================
phase('Phase 1: Big 3 Parallel Decomp');

log('=== Phase 1: Decomposing VFS handlers + sync/mod + llm_manager in parallel ===');

var bigThree = await parallel([
  function() {
    var prompt = [
      '## GOD FILE DECOMPOSITION: VFS handlers.rs',
      '',
      'Current file: C:\\deep-student\\src-tauri\\src\\vfs\\handlers.rs (~7,324 LOC)',
      'Module: C:\\deep-student\\src-tauri\\src\\vfs\\',
      '',
      '### Strategy',
      'Create a vfs/handlers/ directory. Original handlers.rs becomes handlers/mod.rs.',
      '',
      '### Step 1: READ the ENTIRE vfs/handlers.rs file thoroughly',
      '',
      '### Step 2: Create directory C:\\deep-student\\src-tauri\\src\\vfs\\handlers\\',
      '',
      '### Step 3: Extract domain groups into these files:',
      '',
      '1. vfs/handlers/resource_handlers.rs — ALL vfs_create_*, vfs_get_resource*, vfs_resource_* functions',
      '2. vfs/handlers/note_handlers.rs — ALL vfs_note_*, vfs_create_note* functions',
      '3. vfs/handlers/file_handlers.rs — ALL vfs_file_*, vfs_upload_file*, vfs_get_file* functions',
      '4. vfs/handlers/attachment_handlers.rs — ALL vfs_attachment_*, vfs_upload_attachment* functions',
      '5. vfs/handlers/index_handlers.rs — ALL vfs_search*, vfs_reindex*, vfs_index_* functions',
      '6. vfs/handlers/mindmap_handlers.rs — ALL vfs_mindmap_*, vfs_create_mindmap* functions',
      '7. vfs/handlers/todo_handlers.rs — ALL vfs_todo_* functions',
      '8. vfs/handlers/pomodoro_handlers.rs — ALL vfs_pomodoro_* functions',
      '9. vfs/handlers/ocr_handlers.rs — ALL vfs_ocr_* functions',
      '10. vfs/handlers/multimodal_handlers.rs — ALL vfs_multimodal_* functions',
      '11. vfs/handlers/pdf_handlers.rs — ALL vfs_pdf_*, vfs_get_pdf_* functions',
      '12. vfs/handlers/ref_handlers.rs — ALL vfs_ref_*, vfs_get_resource_refs* functions',
      '13. vfs/handlers/debug_handlers.rs — ALL vfs_debug_*, vfs_reset_*, vfs_diagnose_* functions',
      '14. vfs/handlers/mod.rs — module declarations + pub use re-exports of ALL public items',
      '',
      '### CRITICAL RULES:',
      '1. PRESERVE EVERY FUNCTION EXACTLY — copy body verbatim, no logic changes',
      '2. PRESERVE ALL #[tauri::command] attributes on functions',
      '3. Each file gets only the imports it actually needs',
      '4. mod.rs MUST re-export every public item with pub use for backward compatibility',
      '5. ALL function signatures remain IDENTICAL — params, return types, attributes',
      '6. Do NOT refactor or restructure any function logic',
      '7. After creating all new files, DELETE the original vfs/handlers.rs',
      '8. Update vfs/mod.rs if needed for the handlers directory structure',
      '',
      '### Verification:',
      '- Count functions in original vs all new files — must match exactly',
      '- Every #[tauri::command] from original must appear in one new file',
      '- mod.rs re-exports must cover all public functions'
    ].join('\n');
    return agent(prompt, { label: 'vfs_handlers_split', schema: DECOMP_SCHEMA });
  },

  function() {
    var prompt = [
      '## GOD FILE DECOMPOSITION: data_governance/sync/mod.rs',
      '',
      'Current file: C:\\deep-student\\src-tauri\\src\\data_governance\\sync\\mod.rs (~7,463 LOC)',
      'Directory: C:\\deep-student\\src-tauri\\src\\data_governance\\sync\\',
      '',
      '### Strategy: Split mod.rs into domain files, keep mod.rs as re-exports only.',
      '',
      '### Step 1: READ the ENTIRE sync/mod.rs file thoroughly',
      '',
      '### Step 2: Extract into:',
      '',
      'sync/orchestrator.rs (~2,000 LOC):',
      '- SyncManager struct definition and all impl blocks',
      '- Orchestration: start_sync, stop_sync, sync_all, sync_resource',
      '- Sync state management, status tracking',
      '- Public API for sync operations',
      '',
      'sync/changeset.rs (~1,500 LOC):',
      '- Change log management: create_changeset, apply_changeset',
      '- Export/import logic for change data',
      '- Change serialization/deserialization',
      '- Delta computation between local and remote states',
      '',
      'sync/retry.rs (~500 LOC):',
      '- Retry logic, backoff strategies',
      '- Exponential backoff configuration',
      '- RetryableError trait or retry wrappers',
      '- Transient failure handling',
      '',
      'sync/manifest.rs (~500 LOC):',
      '- Manifest generation and parsing',
      '- Sync metadata tracking',
      '- Version tracking for synced resources',
      '',
      'sync/conflict.rs (if conflict logic exists in mod.rs beyond conflict_resolver.rs)',
      '',
      'sync/mod.rs (~50 LOC):',
      '- pub mod declarations + pub use re-exports',
      '- Shared constants only',
      '',
      '### CRITICAL RULES:',
      '1. PRESERVE EVERYTHING — copy code verbatim',
      '2. All imports move with their code to the correct file',
      '3. mod.rs re-exports MUST cover all public items',
      '4. Do NOT change function signatures or logic',
      '5. SyncManager must remain accessible as sync::SyncManager'
    ].join('\n');
    return agent(prompt, { label: 'dg_sync_split', schema: DECOMP_SCHEMA });
  },

  function() {
    var prompt = [
      '## GOD FILE DECOMPOSITION: llm_manager/mod.rs',
      '',
      'Current file: C:\\deep-student\\src-tauri\\src\\llm_manager\\mod.rs (~5,994 LOC)',
      'Directory: C:\\deep-student\\src-tauri\\src\\llm_manager\\',
      '',
      'WARNING: llm_manager has fan-in 51. Changes affect the entire codebase.',
      '',
      '### Strategy: Extract domain files, lean mod.rs. Sub-modules exist: adapters/, exam_engine.rs, model2_pipeline.rs, parser.rs, rag_extension.rs, builtin_vendors.rs',
      '',
      '### Step 1: READ the ENTIRE llm_manager/mod.rs multiple times',
      '',
      '### Step 2: Extract into:',
      '',
      'llm_manager/config_types.rs (~200 LOC):',
      '- ApiConfig struct (43 fields) + Default impl',
      '- VendorConfig struct + impl',
      '- ModelProfile struct + impl',
      '- RegistryCapabilityFlags, RegistryModelRecord, RegistrySeriesRecord, RegistryDocument',
      '- ProviderProtocolRegistryRecord',
      '- All supporting config type definitions',
      '',
      'llm_manager/vendor_config_service.rs (~500 LOC):',
      '- ALL VendorConfig CRUD functions',
      '- load_builtin_vendors, save_vendor_configs, get_vendor_configs',
      '- Vendor config validation and normalization',
      '',
      'llm_manager/model_profile_service.rs (~800 LOC):',
      '- ALL ModelProfile CRUD functions',
      '- load_builtin_models, save_model_profiles, get_model_profiles',
      '- Model profile validation, merging, defaults',
      '- merge_vendor_profile and helpers',
      '- Model adapter resolution',
      '',
      'llm_manager/streaming.rs (~1,500 LOC):',
      '- Streaming request logic: stream_chat, stream_completion',
      '- Cancellation token management',
      '- Stream event handling and forwarding',
      '- Response chunk processing',
      '- All async streaming functions',
      '',
      'llm_manager/tool_call.rs (~400 LOC):',
      '- Tool call processing and dispatch',
      '- Tool result formatting',
      '- Tool call validation',
      '- Function calling protocol handling',
      '',
      'llm_manager/image_processing.rs (~300 LOC):',
      '- Image encoding/decoding for multimodal',
      '- Base64 image handling for LLM requests',
      '- Image size/resolution validation',
      '- Multimodal content block construction',
      '',
      'llm_manager/mod.rs (~1,500 LOC remaining):',
      '- LLMManager struct definition + core impl',
      '- Module declarations for all sub-modules',
      '- Re-exports of public API',
      '- Core configuration and initialization',
      '',
      '### CRITICAL RULES:',
      '1. PRESERVE ALL CODE VERBATIM',
      '2. Imports move with their code',
      '3. mod.rs re-exports match existing public API exactly',
      '4. ApiConfig, VendorConfig, ModelProfile accessible from llm_manager::',
      '5. Do NOT change function signatures or behavior',
      '6. 51 modules depend on llm_manager — break NOTHING'
    ].join('\n');
    return agent(prompt, { label: 'llm_manager_split', schema: DECOMP_SCHEMA });
  }
]);

var b1 = bigThree[0];
var b2 = bigThree[1];
var b3 = bigThree[2];
if (b1) log('VFS handlers: ' + b1.functions_moved + ' functions to ' + b1.files_created.length + ' files, BC: ' + b1.backward_compatible);
if (b2) log('DG sync: ' + b2.functions_moved + ' functions to ' + b2.files_created.length + ' files, BC: ' + b2.backward_compatible);
if (b3) log('LLM mgr: ' + b3.functions_moved + ' functions to ' + b3.files_created.length + ' files, BC: ' + b3.backward_compatible);

// ============================================================
// Phase 2: DSTU Handlers Decomposition + Dedup
// ============================================================
phase('Phase 2: DSTU Decomposition');

log('=== Phase 2: DSTU handlers.rs decomposition + dedup ===');

var DSTU_SCHEMA = {
  type: 'object',
  properties: {
    module: { type: 'string' },
    files_created: { type: 'array', items: { type: 'string' } },
    functions_moved: { type: 'number' },
    dedup_type_inference_removed: { type: 'number' },
    duplicate_parsed_path_removed: { type: 'boolean' },
    string_errors_converted: { type: 'number' },
    backward_compatible: { type: 'boolean' }
  },
  required: ['module', 'files_created', 'backward_compatible']
};

var dstuResult = await agent(
  [
    '## GOD FILE DECOMPOSITION + DEDUP: dstu/handlers.rs (6,382 LOC)',
    '',
    'Current file: C:\\deep-student\\src-tauri\\src\\dstu\\handlers.rs',
    'Module: C:\\deep-student\\src-tauri\\src\\dstu\\',
    'Health score: 5/10 — worst in codebase. 40+ duplicated switch statements.',
    '',
    '### Part A: Split handlers.rs into per-resource-type files (60% of effort)',
    '',
    'Create C:\\deep-student\\src-tauri\\src\\dstu\\handlers\\ directory.',
    'Replace dstu/handlers.rs with dstu/handlers/mod.rs',
    '',
    'Extract into:',
    '1. dstu/handlers/mod.rs (~100 LOC) — module declarations + re-exports',
    '2. dstu/handlers/common.rs (~800 LOC) — SHARED cross-type utilities: move, copy, rename, delete, set_favorite, toggle_pin, generic CRUD wrappers. Functions used by 3+ handlers.',
    '3. dstu/handlers/note_handlers.rs (~500 LOC) — ALL note-specific handlers',
    '4. dstu/handlers/textbook_handlers.rs (~400 LOC) — ALL textbook-specific handlers',
    '5. dstu/handlers/exam_handlers.rs (~500 LOC) — ALL exam-specific handlers + formatting',
    '6. dstu/handlers/translation_handlers.rs (~400 LOC) — ALL translation-specific handlers',
    '7. dstu/handlers/essay_handlers.rs (~400 LOC) — ALL essay-specific handlers',
    '8. dstu/handlers/image_handlers.rs (~300 LOC) — ALL image-specific handlers',
    '9. dstu/handlers/file_handlers.rs (~300 LOC) — ALL file-specific handlers',
    '10. dstu/handlers/mindmap_handlers.rs (~400 LOC) — ALL mindmap handlers + favorites',
    '11. dstu/handlers/search_handlers.rs (~800 LOC) — Cross-type search, search_in_folder, global search',
    '',
    '### Part B: DSTU Dedup (40% of effort)',
    '',
    'B1: Consolidate type-inference functions.',
    'There are 5 functions inferring DSTU resource types. Keep DstuNodeType::from_str() as the ONE canonical. Remove the other 4. Update all callers.',
    'Search for: infer_resource_type, guess_type, parse_type, type_from_name, from_string in dstu/',
    '',
    'B2: Remove duplicate ParsedPath.',
    'Two structs: path_types::ParsedPath and types::DstuParsedPath. Pick path_types::ParsedPath, remove the other. Update all references.',
    '',
    'B3: Normalize error returns.',
    'Convert Result<_, String> helpers to DstuResult<T> using DstuError variants.',
    'Search for functions returning Result<..., String> in dstu/ and convert.',
    '',
    '### CRITICAL RULES:',
    '1. PRESERVE ALL FUNCTION LOGIC VERBATIM — only reorganize files',
    '2. All #[tauri::command] functions keep their attributes',
    '3. mod.rs re-exports EVERYTHING for backward compatibility',
    '4. Dedup changes MUST maintain backward compatibility',
    '5. Update dstu/mod.rs for new handler structure',
    '6. Delete original dstu/handlers.rs after creating all new files'
  ].join('\n'),
  { label: 'dstu_split_and_dedup', schema: DSTU_SCHEMA }
);

if (dstuResult) {
  log('DSTU: ' + dstuResult.functions_moved + ' functions to ' + dstuResult.files_created.length + ' files');
  log('  Dedup: ' + dstuResult.dedup_type_inference_removed + ' type fns removed, ' + dstuResult.string_errors_converted + ' errors converted');
}

// ============================================================
// Phase 3: Import fixes and module verification
// ============================================================
phase('Phase 3: Imports Update and Verify');

log('=== Phase 3: Updating cross-module imports ===');

var IMPORT_SCHEMA = {
  type: 'object',
  properties: {
    broken_imports_found: { type: 'number' },
    broken_imports_fixed: { type: 'number' },
    files_updated: { type: 'array', items: { type: 'string' } },
    all_compatible: { type: 'boolean' }
  },
  required: ['broken_imports_found', 'broken_imports_fixed', 'all_compatible']
};

var VERIFY_SCHEMA = {
  type: 'object',
  properties: {
    modules_verified: { type: 'number' },
    issues_found: { type: 'number' },
    issues_fixed: { type: 'number' },
    remaining_issues: { type: 'array', items: { type: 'string' } }
  },
  required: ['modules_verified', 'issues_found', 'issues_fixed']
};

var importFix = await parallel([
  function() {
    var prompt = [
      '## GLOBAL IMPORT UPDATE AFTER GOD FILE DECOMPOSITION',
      '',
      '4 god files were split into 36+ sub-modules. All should maintain backward compatibility via mod.rs re-exports.',
      'Verify and fix any broken import paths across the entire codebase.',
      '',
      '### Modules that changed:',
      '1. vfs/handlers.rs -> vfs/handlers/mod.rs + 13 domain files',
      '2. data_governance/sync/mod.rs -> sync/ sub-modules (orchestrator, changeset, retry, manifest)',
      '3. llm_manager/mod.rs -> llm_manager/ sub-modules (config_types, vendor_config_service, streaming, tool_call, image_processing, model_profile_service)',
      '4. dstu/handlers.rs -> dstu/handlers/mod.rs + 10 domain files',
      '',
      '### Tasks:',
      '1. Verify vfs/mod.rs correctly declares handlers sub-module for the new directory structure',
      '2. Verify data_governance/sync/mod.rs declares new sub-modules and re-exports correctly',
      '3. Verify llm_manager/mod.rs declares new sub-modules and re-exports correctly',
      '4. Verify dstu/mod.rs correctly declares handlers for new directory structure',
      '5. Search for crate::vfs::handlers:: imports — any breaks?',
      '6. Search for crate::data_governance::sync:: imports — any breaks?',
      '7. Search for crate::llm_manager:: imports — any breaks?',
      '8. Search for crate::dstu::handlers:: imports — any breaks?',
      '9. Fix ALL broken imports found',
      '10. Verify lib.rs still correctly references modules',
      '',
      'IMPORTANT: If re-exports are correct in mod.rs files, no external imports should need changing.'
    ].join('\n');
    return agent(prompt, { label: 'import_fix', schema: IMPORT_SCHEMA });
  },

  function() {
    var prompt = [
      '## VERIFICATION: Rust Module Structure Integrity',
      '',
      'After god file decomposition, verify module structure:',
      '',
      '1. Read C:\\deep-student\\src-tauri\\src\\lib.rs — all top-level modules still declared correctly',
      '2. Read C:\\deep-student\\src-tauri\\src\\vfs\\mod.rs — handlers module declaration works',
      '3. Read C:\\deep-student\\src-tauri\\src\\data_governance\\mod.rs — sync sub-module correct',
      '4. Read C:\\deep-student\\src-tauri\\src\\llm_manager\\mod.rs — new sub-modules declared',
      '5. Read C:\\deep-student\\src-tauri\\src\\dstu\\mod.rs — handlers declaration works',
      '',
      '### Check for:',
      '- Missing mod declarations',
      '- Incorrect or missing pub use re-exports',
      '- Stale file references (pointing to deleted .rs files)',
      '- Any visible compile-breaking issues',
      '',
      'Fix all issues found. Report every change.'
    ].join('\n');
    return agent(prompt, { label: 'module_verify', schema: VERIFY_SCHEMA });
  }
]);

var impFix = importFix[0];
var modVer = importFix[1];

if (impFix) log('Import fix: ' + impFix.broken_imports_found + ' broken found, ' + impFix.broken_imports_fixed + ' fixed, all OK: ' + impFix.all_compatible);
if (modVer) log('Module verify: ' + modVer.modules_verified + ' modules checked, ' + modVer.issues_found + ' issues, ' + modVer.issues_fixed + ' fixed');

// Summary
log('');
log('========================================');
log('P1 GOD FILE DECOMPOSITION COMPLETE');
log('========================================');
log('');

var totalFilesCreated = 0;
if (b1 && b1.files_created) totalFilesCreated += b1.files_created.length;
if (b2 && b2.files_created) totalFilesCreated += b2.files_created.length;
if (b3 && b3.files_created) totalFilesCreated += b3.files_created.length;
if (dstuResult && dstuResult.files_created) totalFilesCreated += dstuResult.files_created.length;
log('New files created: ' + totalFilesCreated);

var allBC = true;
if (b1 && !b1.backward_compatible) allBC = false;
if (b2 && !b2.backward_compatible) allBC = false;
if (b3 && !b3.backward_compatible) allBC = false;
if (dstuResult && !dstuResult.backward_compatible) allBC = false;
log('All backward compatible: ' + allBC);

return {
  vfs: b1,
  dg_sync: b2,
  llm_manager: b3,
  dstu: dstuResult,
  import_fixes: importFix
};
