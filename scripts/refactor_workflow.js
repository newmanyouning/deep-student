export const meta = {
  name: 'backend-refactor-analysis',
  description: '修复编译错误 → 模块依赖分析 → 功能评估 → 依赖膨胀根因 → 单版本可行性 → 迁移计划 → 试点重写',
  phases: [
    { title: 'Phase 1: Compile Error Fix', detail: '修复所有E0308/E0382编译错误，Rust专家审查' },
    { title: 'Phase 2: Module Dependency Graph', detail: '分析397个Rust文件间的完整依赖关系' },
    { title: 'Phase 3: Functional Group Analysis', detail: '按功能分组分析模块边界和耦合' },
    { title: 'Phase 4: Dependency Bloat RCA', detail: '根因分析：多版本依赖膨胀的根本原因' },
    { title: 'Phase 5: Single-Version Feasibility', detail: '评估统一版本工具的可行性' },
    { title: 'Phase 6: Migration Plan + Pilot', detail: '制定迁移计划并试点重写一个模块' }
  ]
};

// ============================================================
// Phase 1: Fix Compile Errors
// ============================================================
phase('Phase 1: Compile Error Fix');

log('=== Phase 1: Fixing compilation errors ===');
log('Errors from cargo_errors.txt may be partially stale. Verifying each file...');

const errorFiles = [
  'C:\\deep-student\\src-tauri\\src\\chat_v2\\tools\\chatanki_executor.rs',
  'C:\\deep-student\\src-tauri\\src\\chat_v2\\tools\\paper_save_executor.rs',
  'C:\\deep-student\\src-tauri\\src\\dstu\\folder_handlers.rs',
  'C:\\deep-student\\src-tauri\\src\\dstu\\handlers.rs',
  'C:\\deep-student\\src-tauri\\src\\essay_grading\\mod.rs',
  'C:\\deep-student\\src-tauri\\src\\paddleocr_api.rs',
  'C:\\deep-student\\src-tauri\\src\\vfs\\handlers.rs',
];

const FIX_SCHEMA = {
  type: 'object',
  properties: {
    file: { type: 'string' },
    already_fixed_count: { type: 'number' },
    newly_fixed_count: { type: 'number' },
    fixes_applied: { type: 'array', items: { type: 'object', properties: {
      line: { type: 'number' },
      error_type: { type: 'string' },
      fix_description: { type: 'string' },
      old_code_snippet: { type: 'string' },
      new_code_snippet: { type: 'string' }
    }}},
    remaining_issues: { type: 'array', items: { type: 'string' } },
    syntax_review_ok: { type: 'boolean' }
  },
  required: ['file', 'already_fixed_count', 'newly_fixed_count', 'fixes_applied', 'syntax_review_ok']
};

const fixResults = await pipeline(
  errorFiles,
  function(file) {
    var shortName = file.split('\\').pop().replace('.rs', '');
    var prompt = 'Read the Rust source file at ' + file + '.\n\n' +
      'The cargo build log (cargo_errors.txt at C:\\deep-student\\src-tauri\\cargo_errors.txt) ' +
      'reports E0308 (type mismatch) and E0382 (borrow-after-move) errors in this file. ' +
      'However, the error log is from a PREVIOUS build and was partially fixed.\n\n' +
      'YOUR TASK:\n' +
      '1. Read the file and identify ANY remaining type errors where:\n' +
      '   - A function returns String but the context expects a typed error (ToolError, DstuError, VfsError, EssayGradingError, etc.)\n' +
      '   - A typed error is passed where String is expected\n' +
      '   - A borrow-after-move pattern exists (like reqwest::Response used after .text())\n' +
      '   - A ? operator is used on a function returning () where Result is expected\n\n' +
      '2. For EACH error found, apply the correct fix:\n' +
      '   - String -> typed error: wrap with ErrorType::Variant(string) or ErrorType::from(string)\n' +
      '   - Typed error -> String: use .to_string() or format!("{}", err)\n' +
      '   - Borrow-after-move: restructure to avoid the move, or clone before consuming\n' +
      '   - ? on (): Change to proper error propagation or remove ?\n\n' +
      '3. After all fixes, do a complete Rust syntax review of the entire file:\n' +
      '   - Verify all match arms are exhaustive\n' +
      '   - Check all return types match function signatures\n' +
      '   - Verify no other compilation issues\n\n' +
      'IMPORTANT RULES:\n' +
      '- Preserve ALL existing logic, error handling, and behavior\n' +
      '- Only fix type mismatches — do not refactor or change logic\n' +
      '- If the error is already fixed (shows .to_string() or .into() or ::from()), DO NOT modify\n' +
      '- Do NOT change function signatures\n\n' +
      'Return a structured report of what you fixed (if anything) and what was already fixed.';

    return agent(prompt, { label: shortName, schema: FIX_SCHEMA, isolation: 'worktree' });
  }
);

var totalFixed = 0;
var totalAlreadyFixed = 0;
for (var i = 0; i < fixResults.length; i++) {
  if (fixResults[i]) {
    totalFixed += fixResults[i].newly_fixed_count || 0;
    totalAlreadyFixed += fixResults[i].already_fixed_count || 0;
  }
}
log('Phase 1 complete: ' + totalAlreadyFixed + ' errors already fixed, ' + totalFixed + ' newly fixed');

// ============================================================
// Phase 2: Module Dependency Graph
// ============================================================
phase('Phase 2: Module Dependency Graph');

log('=== Phase 2: Building complete module dependency graph ===');

const DEP_SCHEMA = {
  type: 'object',
  properties: {
    total_modules: { type: 'number' },
    total_import_edges: { type: 'number' },
    circular_deps: { type: 'array', items: { type: 'object', properties: {
      cycle: { type: 'array', items: { type: 'string' } },
      severity: { type: 'string' }
    }}},
    most_depended_on: { type: 'array', items: { type: 'object', properties: {
      module: { type: 'string' },
      fan_in: { type: 'number' }
    }}},
    excessive_deps: { type: 'array', items: { type: 'object', properties: {
      module: { type: 'string' },
      fan_out: { type: 'number' },
      suggestion: { type: 'string' }
    }}}
  },
  required: ['total_modules', 'total_import_edges', 'circular_deps']
};

const CMD_SCHEMA = {
  type: 'object',
  properties: {
    total_commands: { type: 'number' },
    registered_only: { type: 'array', items: { type: 'string' } },
    issues: { type: 'array', items: { type: 'object', properties: {
      command: { type: 'string' },
      issue: { type: 'string' },
      severity: { type: 'string' }
    }}}
  },
  required: ['total_commands']
};

const dependencyAnalysis = await parallel([
  function() {
    return agent(
      'Build a COMPLETE dependency graph of ALL Rust source files under C:\\deep-student\\src-tauri\\src.\n\n' +
      'For each module (directory or standalone file), list:\n' +
      '1. All modules it imports (use/import statements)\n' +
      '2. All modules that import it (reverse dependencies)\n' +
      '3. Identify circular dependency chains\n' +
      '4. Count fan-in (how many modules depend on this one) and fan-out (how many modules this depends on)\n' +
      '5. Flag any module with excessive dependencies (>20 imports from other internal modules)\n\n' +
      'Focus especially on:\n' +
      '- llm_manager/ and its adapters/ submodule\n' +
      '- chat_v2/ and its tools/, handlers/, pipeline/, workspace/ submodules\n' +
      '- vfs/ and its repos/ submodule\n' +
      '- dstu/ and its submodules\n' +
      '- data_governance/ and its submodules\n\n' +
      'Write the complete analysis to: C:\\deep-student\\DEPENDENCY_GRAPH.md',
      { label: 'dep_graph_builder', schema: DEP_SCHEMA }
    );
  },

  function() {
    return agent(
      'Map all Tauri command registrations to their implementations.\n\n' +
      '1. Read C:\\deep-student\\src-tauri\\src\\lib.rs — find ALL .invoke_handler() or tauri::generate_handler![] calls to list every registered command\n' +
      '2. For each command, trace it to its Rust implementation function\n' +
      '3. Check if the command has frontend TypeScript type definitions\n' +
      '4. Report any:\n' +
      '   - Commands registered but never called from frontend\n' +
      '   - Commands called from frontend but not registered\n' +
      '   - Mismatched parameter types between Rust and TypeScript\n\n' +
      'Write report to: C:\\deep-student\\COMMAND_REGISTRY.md',
      { label: 'command_audit', schema: CMD_SCHEMA }
    );
  }
]);

var depGraph = dependencyAnalysis[0];
var cmdAudit = dependencyAnalysis[1];
if (depGraph) log('Phase 2 complete: ' + depGraph.total_modules + ' modules, ' + depGraph.total_import_edges + ' edges, ' + (depGraph.circular_deps ? depGraph.circular_deps.length : 0) + ' circular deps');
if (cmdAudit) log('  Commands: ' + cmdAudit.total_commands + ' registered, ' + (cmdAudit.issues ? cmdAudit.issues.length : 0) + ' issues');

// ============================================================
// Phase 3: Functional Group Analysis
// ============================================================
phase('Phase 3: Functional Group Analysis');

log('=== Phase 3: Analyzing each functional group ===');

const GROUP_SCHEMA = {
  type: 'object',
  properties: {
    group_name: { type: 'string' },
    total_loc_estimate: { type: 'number' },
    health_score: { type: 'number' },
    external_deps_count: { type: 'number' },
    essential_deps: { type: 'array', items: { type: 'string' } },
    replaceable_deps: { type: 'array', items: { type: 'string' } },
    optional_deps: { type: 'array', items: { type: 'string' } },
    redundant_deps: { type: 'array', items: { type: 'string' } },
    internal_coupling_issues: { type: 'array', items: { type: 'string' } },
    extracted_report_path: { type: 'string' }
  },
  required: ['group_name', 'health_score', 'external_deps_count']
};

const functionalGroups = [
  {
    name: 'llm_manager',
    desc: 'LLM Manager + adapters (14 adapters, model routing, provider protocols)',
    files: 'src/llm_manager/, src/llm_manager/adapters/, src/providers/, src/vendors/',
    questions: 'How many crates/APIs does this group depend on? Can the adapter system be simplified? Which external deps are essential vs optional?'
  },
  {
    name: 'chat_v2',
    desc: 'Chat V2 (handlers, tools, pipeline, workspace)',
    files: 'src/chat_v2/',
    questions: 'What is the internal coupling between tools/ and handlers/? Can any tools be extracted to standalone modules? Are there duplicate utility functions across tools?'
  },
  {
    name: 'vfs',
    desc: 'VFS (Virtual File System — repos, handlers, unit_builder)',
    files: 'src/vfs/',
    questions: 'Is the repo abstraction layer clean or leaky? Can any repos be merged or simplified? What external deps does VFS bring in?'
  },
  {
    name: 'dstu',
    desc: 'DSTU (Deep Study — handlers, export, handler_utils)',
    files: 'src/dstu/',
    questions: 'How tightly coupled is DSTU to VFS? Can DSTU exist independently? What is the error propagation pattern?'
  },
  {
    name: 'data_governance',
    desc: 'Data Governance (backup, sync, audit, migration)',
    files: 'src/data_governance/',
    questions: 'Is this module self-contained or spread across others? Can backup/sync be extracted? Does it depend on too many heavy deps?'
  },
  {
    name: 'other_services',
    desc: 'Remaining services: multimodal, memory, essay_grading, translation, ocr_adapters, cloud_storage, mcp, crypto, database, tools',
    files: 'src/multimodal/, src/memory/, src/essay_grading/, src/translation/, src/ocr_adapters/, src/cloud_storage/, src/mcp/, src/crypto/, src/database/, src/tools/',
    questions: 'Which of these services are independent? Which have heavy external dependency chains? Can any be feature-gated or optional?'
  }
];

const groupAnalyses = await pipeline(
  functionalGroups,
  function(group) {
    var prompt = 'Analyze the "' + group.name + '" functional group in the Tauri backend.\n\n' +
      '## Group: ' + group.name + '\n' +
      '**Description**: ' + group.desc + '\n' +
      '**Files to analyze**: ' + group.files + '\n\n' +
      '## Key Questions:\n' + group.questions + '\n\n' +
      '## Analysis Tasks:\n' +
      '1. List ALL external crate dependencies this group pulls in (directly and transitively)\n' +
      '2. Measure internal coupling: count cross-module function calls/imports between sub-modules\n' +
      '3. Identify duplicated or overlapping functionality within this group\n' +
      '4. Evaluate architectural soundness: SOLID principles, separation of concerns\n' +
      '5. Estimate LOC (lines of code) in this group\n' +
      '6. For each external dependency, classify as:\n' +
      '   - ESSENTIAL (core functionality depends on it)\n' +
      '   - REPLACEABLE (could use a different crate)\n' +
      '   - OPTIONAL (could be feature-gated)\n' +
      '   - REDUNDANT (functionality exists elsewhere in the project)\n' +
      '7. Rate overall health: 1-10 (10 = perfectly modular, 1 = spaghetti)\n\n' +
      'Write analysis to: C:\\deep-student\\FUNCTIONAL_ANALYSIS_' + group.name.toUpperCase() + '.md';

    return agent(prompt, { label: 'group_' + group.name, schema: GROUP_SCHEMA });
  }
);

var sum = 0;
var count = 0;
for (var i = 0; i < groupAnalyses.length; i++) {
  if (groupAnalyses[i]) {
    sum += groupAnalyses[i].health_score || 5;
    count++;
  }
}
var avgHealth = count > 0 ? sum / count : 0;
log('Phase 3 complete: ' + count + ' groups analyzed, avg health score: ' + avgHealth.toFixed(1) + '/10');

// ============================================================
// Phase 4: Dependency Bloat RCA
// ============================================================
phase('Phase 4: Dependency Bloat RCA');

log('=== Phase 4: Root cause analysis of dependency bloat ===');

const RCA_SCHEMA = {
  type: 'object',
  properties: {
    multi_version_count: { type: 'number' },
    root_causes: { type: 'array', items: { type: 'object', properties: {
      crate_name: { type: 'string' },
      versions_found: { type: 'array', items: { type: 'string' } },
      pullers: { type: 'array', items: { type: 'string' } },
      root_cause: { type: 'string' },
      fix_feasible: { type: 'boolean' },
      estimated_effort: { type: 'string' }
    }}},
    vendor_necessity: { type: 'string' },
    quick_wins: { type: 'array', items: { type: 'string' } },
    report_path: { type: 'string' }
  },
  required: ['multi_version_count', 'root_causes']
};

const rcaResult = await agent(
  'Perform a ROOT CAUSE ANALYSIS of dependency version bloat in C:\\deep-student\\src-tauri.\n\n' +
  '## Known Issues (from previous audit):\n' +
  '1. **rustls**: 3 versions (0.21, 0.22, 0.23) - security risk\n' +
  '2. **reqwest**: 3 versions (0.11, 0.12, 0.13)\n' +
  '3. **zip**: 3 versions (0.6, 2.4, 4.6)\n' +
  '4. **hyper**: 2 versions (0.14, 1.x)\n' +
  '5. **thiserror**: 2 versions (1.0, 2.0) - from vendored object_store\n' +
  '6. **125 packages** appear at multiple versions\n\n' +
  '## Tasks:\n\n' +
  '### A. Root Cause Mapping\n' +
  'For EACH multi-version dependency, trace the EXACT dependency chain that pulls in each version:\n' +
  '- Which crate explicitly depends on version X?\n' +
  '- Through which intermediate crates?\n' +
  '- Is it a direct dep or transitive?\n' +
  '- Can the chain be broken?\n\n' +
  '### B. Vendor Analysis\n' +
  'Two crates are vendored (lancedb, object_store). These are the MAIN sources of newer-version transitive deps.\n' +
  '1. Why were they vendored? (Read vendor directory comments)\n' +
  '2. What specific versions are pinned?\n' +
  '3. Can we upgrade the vendors to match our direct deps, or upgrade our direct deps to match vendors?\n' +
  '4. Is vendoring still necessary, or can we switch to published crates?\n\n' +
  '### C. Upgrade Feasibility Matrix\n' +
  'For each multi-version crate, create a matrix:\n' +
  '| Crate | Current | Target | Blockers | Effort |\n\n' +
  '### D. Full Cargo.toml Audit\n' +
  'Read the EXACT Cargo.toml (C:\\deep-student\\src-tauri\\Cargo.toml):\n' +
  '1. List every direct dependency with its specified version constraints\n' +
  '2. Compare against Cargo.lock resolved versions\n' +
  '3. Identify where loosening/tightening version constraints could reduce duplication\n\n' +
  'Write complete analysis to: C:\\deep-student\\DEPENDENCY_BLOAT_RCA.md',
  { label: 'dep_bloat_rca', schema: RCA_SCHEMA }
);

if (rcaResult) {
  log('Phase 4 complete: ' + rcaResult.multi_version_count + ' multi-version crates analyzed');
  var qw = rcaResult.quick_wins ? rcaResult.quick_wins.length : 0;
  log('  Quick wins identified: ' + qw);
  log('  Vendored deps: ' + rcaResult.vendor_necessity);
}

// ============================================================
// Phase 5: Single-Version Feasibility
// ============================================================
phase('Phase 5: Single-Version Feasibility');

log('=== Phase 5: Evaluating single-version tool feasibility ===');

const FEAS_SCHEMA = {
  type: 'object',
  properties: {
    can_unify_all: { type: 'boolean' },
    can_unify_most: { type: 'boolean' },
    crates_analysed: { type: 'number' },
    unifiable_now: { type: 'array', items: { type: 'string' } },
    needs_vendor_update: { type: 'array', items: { type: 'string' } },
    needs_upstream_change: { type: 'array', items: { type: 'string' } },
    estimated_total_effort: { type: 'string' },
    recommended_strategy: { type: 'string' },
    report_path: { type: 'string' }
  },
  required: ['can_unify_all', 'can_unify_most', 'recommended_strategy']
};

const feasibilityResult = await agent(
  'Based on the dependency analysis from phases 2-4, evaluate whether this project can be migrated to use a SINGLE version of each key tool/library.\n\n' +
  '## Key Libraries to Evaluate:\n\n' +
  '1. **HTTP Client**: reqwest (currently 0.11 + 0.12 + 0.13) -> Can we use only one version?\n' +
  '2. **TLS**: rustls (0.21 + 0.22 + 0.23) -> Can we use only 0.23?\n' +
  '3. **Serialization**: serde/serde_json (check if multiple versions exist)\n' +
  '4. **Async Runtime**: tokio (check version consistency)\n' +
  '5. **Compression**: zip (0.6 + 2.4 + 4.6) -> Can we use only one version?\n' +
  '6. **HTTP Framework**: hyper (0.14 + 1.x) -> Can we use only 1.x?\n' +
  '7. **Error handling**: thiserror (1.0 + 2.0) -> Can we use only one?\n' +
  '8. **Tauri Framework**: Is it on latest stable?\n\n' +
  '## Tasks:\n\n' +
  '### A. Tauri Version Check\n' +
  'Check if Tauri 2.x is the latest. If not, what is the upgrade path?\n' +
  '- Read Cargo.toml for tauri version\n' +
  '- Check if upgrading Tauri would resolve any version conflicts\n\n' +
  '### B. reqwest Unification Feasibility\n' +
  '- Why do we need 0.11? (Is it a direct dep or transitive?)\n' +
  '- What Cargo.toml changes would consolidate to 0.12?\n' +
  '- Would Tauri 2.x accept reqwest 0.12?\n\n' +
  '### C. Vendor Removal Feasibility\n' +
  '- Can lancedb be upgraded to a version that uses reqwest 0.12 + hyper 1.x?\n' +
  '- Can object_store be upgraded to match our other deps?\n' +
  '- Is there an alternative to vendoring these crates?\n\n' +
  '### D. Complete Migration Roadmap\n' +
  'If SINGLE-VERSION is feasible, produce:\n' +
  '1. Order of operations (which to upgrade first)\n' +
  '2. Estimated changes needed per crate\n' +
  '3. Risk assessment per change\n' +
  '4. Which modules would need code changes\n\n' +
  'Write to: C:\\deep-student\\SINGLE_VERSION_FEASIBILITY.md',
  { label: 'single_version_study', schema: FEAS_SCHEMA }
);

if (feasibilityResult) {
  log('Phase 5 complete: Can unify all = ' + feasibilityResult.can_unify_all + ', Can unify most = ' + feasibilityResult.can_unify_most);
  var unifiable = feasibilityResult.unifiable_now ? feasibilityResult.unifiable_now.join(', ') : 'none';
  log('  Unifiable now: ' + unifiable);
  log('  Strategy: ' + feasibilityResult.recommended_strategy);
}

// ============================================================
// Phase 6: Migration Plan + Pilot Module Rewrite
// ============================================================
phase('Phase 6: Migration Plan + Pilot');

log('=== Phase 6: Creating migration plan and executing pilot rewrite ===');

var moduleToRewrite = 'providers/mod.rs';
if (feasibilityResult && feasibilityResult.recommended_strategy) {
  if (feasibilityResult.recommended_strategy.indexOf('vendor') >= 0) {
    moduleToRewrite = 'vendors/siliconflow.rs';
  } else if (feasibilityResult.unifiable_now && feasibilityResult.unifiable_now.length > 0) {
    moduleToRewrite = 'providers/mod.rs';
  }
}

const PILOT_SCHEMA = {
  type: 'object',
  properties: {
    module_rewritten: { type: 'string' },
    functions_preserved: { type: 'number' },
    functions_modified: { type: 'number' },
    deps_removed: { type: 'array', items: { type: 'string' } },
    deps_upgraded: { type: 'array', items: { type: 'string' } },
    error_handling_preserved: { type: 'array', items: { type: 'string' } },
    error_handling_removed: { type: 'array', items: { type: 'string' } },
    compilation_verified: { type: 'boolean' },
    summary_path: { type: 'string' }
  },
  required: ['module_rewritten', 'functions_preserved']
};

var pilotPrompt = '## Pilot Module Rewrite\n\n' +
  'Based on the single-version feasibility study, execute a PILOT REWRITE of one module using unified dependency versions.\n\n' +
  '### Module to Rewrite: ' + moduleToRewrite + '\n\n' +
  '### Rules (MANDATORY):\n' +
  '1. **PRESERVE ALL EXISTING FUNCTIONALITY** — every function, every error variant, every behavior MUST remain identical\n' +
  '2. **PRESERVE ALL ERROR HANDLING** — if the original propagates specific errors, the rewrite must do the same\n' +
  '3. **ONLY SKIP ERROR HANDLING** that becomes impossible (e.g., if a specific error type no longer exists because an old crate version is replaced)\n' +
  '4. **USE ONLY UNIFIED VERSIONS** — use the target single version of each dependency\n' +
  '5. **KEEP ALL FUNCTION SIGNATURES** — no public API changes\n' +
  '6. **KEEP ALL TESTS PASSING** — if there are existing tests, adapt them but do not change their assertions\n\n' +
  '### Before Rewrite:\n' +
  '1. Read the current module implementation completely\n' +
  '2. List all its dependencies and their versions\n' +
  '3. List all function signatures\n' +
  '4. List all error types used\n\n' +
  '### After Rewrite:\n' +
  '1. Write the new implementation preserving exact behavior\n' +
  '2. Add a comment block at the top explaining what was changed and why\n' +
  '3. Verify: every public function has identical signature\n' +
  '4. Verify: all error paths are preserved\n\n' +
  '### Output:\n' +
  'Write the rewritten module to the same file path (after backing up).\n\n' +
  'Write a migration summary to: C:\\deep-student\\PILOT_REWRITE_SUMMARY.md';

const pilotRewrite = await agent(pilotPrompt, { label: 'pilot_rewrite', isolation: 'worktree', schema: PILOT_SCHEMA });

// 6b: Final synthesis report
var fixSummary = '';
for (var i = 0; i < fixResults.length; i++) {
  if (fixResults[i]) {
    fixSummary += fixResults[i].file + ': ' + fixResults[i].already_fixed_count + ' already fixed, ' + fixResults[i].newly_fixed_count + ' newly fixed\n';
  }
}

const synthesisReport = await agent(
  'Synthesize ALL findings into a master refactoring plan.\n\n' +
  'Read from the generated reports:\n' +
  '- C:\\deep-student\\DEPENDENCY_GRAPH.md\n' +
  '- C:\\deep-student\\COMMAND_REGISTRY.md\n' +
  '- C:\\deep-student\\FUNCTIONAL_ANALYSIS_llm_manager.md\n' +
  '- C:\\deep-student\\FUNCTIONAL_ANALYSIS_chat_v2.md\n' +
  '- C:\\deep-student\\FUNCTIONAL_ANALYSIS_vfs.md\n' +
  '- C:\\deep-student\\FUNCTIONAL_ANALYSIS_dstu.md\n' +
  '- C:\\deep-student\\FUNCTIONAL_ANALYSIS_data_governance.md\n' +
  '- C:\\deep-student\\FUNCTIONAL_ANALYSIS_other_services.md\n' +
  '- C:\\deep-student\\DEPENDENCY_BLOAT_RCA.md\n' +
  '- C:\\deep-student\\SINGLE_VERSION_FEASIBILITY.md\n' +
  '- C:\\deep-student\\PILOT_REWRITE_SUMMARY.md\n\n' +
  '## Phase 1 Compile Error Fix Summary:\n' + fixSummary + '\n\n' +
  '## Final Report Structure:\n\n' +
  '### 1. Executive Summary\n' +
  'One-page overview of the entire refactoring operation\n\n' +
  '### 2. Current State\n' +
  '- Compilation status (what is fixed, what remains)\n' +
  '- Module dependency health\n' +
  '- Dependency bloat severity\n\n' +
  '### 3. Functional Group Health Scores\n' +
  '| Group | Score | LOC | Key Issues |\n\n' +
  '### 4. Single-Version Migration Roadmap\n' +
  '- Phase-by-phase plan with estimated effort\n' +
  '- Risk assessment\n' +
  '- Quick wins (things that can be done immediately)\n\n' +
  '### 5. Pilot Rewrite Results\n' +
  '- What was rewritten\n' +
  '- What was learned\n' +
  '- Applicability to other modules\n\n' +
  '### 6. Recommended Next Steps\n' +
  'Priority-ordered action items\n\n' +
  'Write the master plan to: C:\\deep-student\\MASTER_REFACTORING_PLAN.md',
  { label: 'master_plan' }
);

log('');
log('========================================');
log('BACKEND REFACTORING WORKFLOW COMPLETE');
log('========================================');
log('');
log('Generated reports:');
log('  1. DEPENDENCY_GRAPH.md — Module dependency analysis');
log('  2. COMMAND_REGISTRY.md — Tauri command audit');
log('  3. FUNCTIONAL_ANALYSIS_*.md — Per-group analysis (6 files)');
log('  4. DEPENDENCY_BLOAT_RCA.md — Root cause analysis');
log('  5. SINGLE_VERSION_FEASIBILITY.md — Unification feasibility');
log('  6. PILOT_REWRITE_SUMMARY.md — Pilot rewrite results');
log('  7. MASTER_REFACTORING_PLAN.md — Master plan');
log('');
log('Compile errors: ' + totalAlreadyFixed + ' already fixed, ' + totalFixed + ' newly fixed');

return {
  phase1_fixes: fixResults,
  phase2_depGraph: depGraph,
  phase2_cmdAudit: cmdAudit,
  phase3_groups: groupAnalyses,
  phase4_rca: rcaResult,
  phase5_feasibility: feasibilityResult,
  phase6_pilot: pilotRewrite
};
