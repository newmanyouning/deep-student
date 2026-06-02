export const meta = {
  name: 'p2-cycle-resolution-and-di',
  description: 'P2: 修复P1残余 → VFS索引三元循环解除 → 内存VFS解耦 → 管道合并 → 执行器反转 → 瘦适配器合并',
  phases: [
    { title: 'Phase 1: Residual Fixes', detail: '修复P1残留的4个问题 + 评估P2代码规模' },
    { title: 'Phase 2: Parallel P2 Execution', detail: '5项独立任务并行执行，各代理<50%上下文窗口' }
  ]
};

// ============================================================
// Phase 1: Fix P1 residual issues (1 agent, small)
// ============================================================
phase('Phase 1: Residual Fixes');

log('=== Phase 1: Fixing 4 P1 residual issues ===');

var residualResult = await agent(
  [
    '## Fix 4 residual issues from P1 god file decomposition.',
    '',
    '### Issue 1: vfs/todo_handlers.rs — 25 commands return Result<T, String> instead of VfsResult<T>',
    'File: C:\\deep-student\\src-tauri\\src\\vfs\\handlers\\todo_handlers.rs',
    'Find ALL 25 functions returning Result<T, String>. Convert each to use VfsError.',
    'Pattern: return Err("message".to_string()) → return Err(VfsError::Other("message".to_string()))',
    'Also fix the Error type in the function signatures.',
    '',
    '### Issue 2: Empty placeholder files',
    'Check C:\\deep-student\\src-tauri\\src\\vfs\\handlers\\todo_handlers.rs and pomodoro_handlers.rs',
    'If empty, populate with their handlers from the original vfs/handlers.rs (which was split).',
    'If they contain code but are just missing imports, fix imports.',
    '',
    '### Issue 3: lib.rs stale/merged comments',
    'File: C:\\deep-student\\src-tauri\\src\\lib.rs',
    'Look around lines 89, 92 for stale comments (VFS comment including DSTU text; SM-2/question sync mention).',
    'Clean up or update these comments.',
    '',
    '### Issue 4: dstu/mod.rs missing re-exports',
    'File: C:\\deep-student\\src-tauri\\src\\dstu\\mod.rs',
    'Add missing pub use re-exports for: dstu_rename, dstu_unwatch, dstu_watch from handlers::common.',
    'Read dstu/handlers/common.rs to find exact function signatures and add corresponding re-exports.',
    '',
    '### Verify:',
    'After all fixes, verify each file is syntactically correct Rust.'
  ].join('\n'),
  { label: 'fix_residuals', schema: {
    type: 'object',
    properties: {
      issues_fixed: { type: 'number' },
      issue1_string_to_vfserror: { type: 'number' },
      issue2_placeholders_fixed: { type: 'boolean' },
      issue3_comments_cleaned: { type: 'boolean' },
      issue4_reexports_added: { type: 'array', items: { type: 'string' } },
      files_modified: { type: 'array', items: { type: 'string' } }
    },
    required: ['issues_fixed', 'files_modified']
  }}
);

if (residualResult) {
  log('Residual fixes: ' + residualResult.issues_fixed + '/4 issues fixed');
  log('  Issue 1: ' + residualResult.issue1_string_to_vfserror + ' String->VfsError conversions');
  log('  Issue 2: placeholders ' + (residualResult.issue2_placeholders_fixed ? 'fixed' : 'pending'));
  log('  Issue 3: comments ' + (residualResult.issue3_comments_cleaned ? 'cleaned' : 'pending'));
  log('  Issue 4: ' + (residualResult.issue4_reexports_added ? residualResult.issue4_reexports_added.length : 0) + ' re-exports added');
}

// ============================================================
// Phase 2: Parallel P2 Execution (5 independent tasks)
// ============================================================
phase('Phase 2: Parallel P2 Execution');

log('=== Phase 2: 5 P2 tasks in parallel, each <50% context window ===');

var P2_SCHEMA = {
  type: 'object',
  properties: {
    task_key: { type: 'string' },
    status: { type: 'string', enum: ['completed', 'partial', 'blocked'] },
    files_created: { type: 'array', items: { type: 'string' } },
    files_modified: { type: 'array', items: { type: 'string' } },
    loc_affected: { type: 'number' },
    backward_compatible: { type: 'boolean' },
    issues: { type: 'array', items: { type: 'string' } }
  },
  required: ['task_key', 'status', 'backward_compatible']
};

var p2Results = await parallel([
  // Task 2.1: VFS Indexing Triad — Cycle 3 (HIGH) — Coordinator pattern
  function() {
    var prompt = [
      '## P2 Task 2.1: Break VFS Indexing Triad Cycle (HIGH severity)',
      '',
      '### Problem',
      'Three VFS services form a 3-node circular dependency:',
      '  indexing.rs (4,632 LOC) <-> embedding_service <-> pdf_processing_service',
      'This is the most complex cycle in the codebase.',
      '',
      '### Solution: Coordinator Pattern (Option B — lowest risk)',
      'Extract orchestration into a coordinator that owns references to both services.',
      'Services no longer import each other directly.',
      '',
      '### Files involved:',
      '- C:\\deep-student\\src-tauri\\src\\vfs\\indexing.rs (~4,632 LOC)',
      '- C:\\deep-student\\src-tauri\\src\\vfs\\embedding_service.rs',
      '- C:\\deep-student\\src-tauri\\src\\vfs\\pdf_processing_service.rs',
      '',
      '### Step 1: Read all 3 files to understand the dependency pattern',
      '### Step 2: Create vfs/indexing/ directory and coordinator.rs',
      '### Step 3: Move orchestration logic from embedding_service and pdf_processing_service into coordinator',
      '### Step 4: Remove reverse deps (embedding_service -> indexing, pdf_processing_service -> indexing)',
      '### Step 5: Update handlers to use coordinator',
      '',
      '### CRITICAL RULES:',
      '- PRESERVE ALL FUNCTIONALITY — every index operation must work identically',
      '- Coordinator pattern: coordinator.rs owns Arc<EmbeddingService> and Arc<PdfProcessingService>',
      '- Services call coordinator, not each other',
      '- Public API through coordinator, not direct service calls',
      '- Keep all existing public function signatures as delegation wrappers in old locations',
      '- No logic changes — only ownership and call direction changes'
    ].join('\n');
    return agent(prompt, { label: 'vfs_indexing_coordinator', schema: P2_SCHEMA });
  },

  // Task 2.2: Memory VFS decoupling — MemoryStorage trait
  function() {
    var prompt = [
      '## P2 Task 2.2: Decouple Memory Module from VFS Internals',
      '',
      '### Problem',
      'memory/service.rs imports 8+ concrete VFS types directly:',
      'VfsDatabase, VfsLanceStore, VfsIndexStateRepo, VfsNoteRepo, VfsFolderRepo, etc.',
      'This creates tight coupling — every VFS change forces memory recompilation.',
      '',
      '### Solution: MemoryStorage Trait',
      'Define a trait exposing only the VFS operations that memory actually needs.',
      '',
      '### Step 1: Audit memory VFS usage',
      'Read C:\\deep-student\\src-tauri\\src\\memory\\service.rs — list every VFS function/method called',
      'Read other files in memory/ — any VFS imports?',
      '',
      '### Step 2: Define MemoryStorage trait',
      'Create C:\\deep-student\\src-tauri\\src\\memory\\storage_trait.rs',
      'Define a trait with only the VFS operations memory needs (not all of VFS):',
      '- fn get_note(&self, id: &str) -> MemoryResult<Note>;',
      '- fn search_notes(&self, query: &str) -> MemoryResult<Vec<Note>>;',
      '- fn insert_memory(&self, entry: &MemoryEntry) -> MemoryResult<()>;',
      '- ... (exact set from step 1 audit)',
      '',
      '### Step 3: Implement trait for VFS types',
      'Add impl MemoryStorage for the VFS types in a new file or in an existing one',
      'These are THIN wrappers — just delegate to existing VFS methods',
      '',
      '### Step 4: Update memory service',
      'Replace all concrete VFS type parameters with Box<dyn MemoryStorage>',
      'Update function signatures to accept the trait instead of concrete types',
      'Update the wiring in lib.rs to inject the VFS impl',
      '',
      '### CRITICAL RULES:',
      '- Memory module behavior must be IDENTICAL',
      '- Trait methods should be async if the original calls are async',
      '- Keep all existing MemoryError variants',
      '- The trait should be object-safe (use Box<dyn MemoryStorage> or Arc<dyn MemoryStorage>)'
    ].join('\n');
    return agent(prompt, { label: 'memory_vfs_decouple', schema: P2_SCHEMA });
  },

  // Task 2.3: Merge essay_grading + translation pipelines
  function() {
    var prompt = [
      '## P2 Task 2.3: Merge essay_grading + translation Pipelines',
      '',
      '### Problem',
      'essay_grading/pipeline.rs and translation/pipeline.rs share 60-70% structural overlap.',
      '~600 LOC of near-identical streaming LLM call code in two places.',
      '',
      '### Solution: Shared StreamingLLMPipeline abstraction',
      '',
      '### Files:',
      '- C:\\deep-student\\src-tauri\\src\\essay_grading\\pipeline.rs',
      '- C:\\deep-student\\src-tauri\\src\\translation\\pipeline.rs',
      '',
      '### Step 1: Read both files completely. Identify the common pattern:',
      '- Streaming HTTP request setup (base_url, api_key, headers)',
      '- SSE/stream chunk parsing',
      '- Error handling during streaming',
      '- Progress event emission',
      '- Cancellation support',
      '',
      '### Step 2: Extract shared pipeline',
      'Create C:\\deep-student\\src-tauri\\src\\utils\\streaming_llm_pipeline.rs',
      'Extract common code into a StreamingLLMPipeline struct with:',
      '- Generic prompt building callback: FnOnce(&Context) -> String',
      '- Generic response chunk parser: FnMut(&str) -> ParsedChunk',
      '- Common streaming HTTP logic (reqwest Client, SSE parsing, cancellation)',
      '- Common progress emission pattern',
      '',
      '### Step 3: Update essay_grading/pipeline.rs',
      'Replace direct streaming logic with calls to StreamingLLMPipeline',
      'Keep essay-specific prompt building and result parsing',
      '',
      '### Step 4: Update translation/pipeline.rs',
      'Same pattern as step 3',
      '',
      '### CRITICAL RULES:',
      '- BOTH pipelines must produce IDENTICAL output to current behavior',
      '- Keep existing function signatures as public API',
      '- Callbacks should be simple — no complex trait hierarchies',
      '- Error propagation must match current behavior exactly'
    ].join('\n');
    return agent(prompt, { label: 'merge_pipelines', schema: P2_SCHEMA });
  },

  // Task 2.4: Pipeline/executor dependency inversion
  function() {
    var prompt = [
      '## P2 Task 2.4: Chat V2 Pipeline/Executor Dependency Inversion',
      '',
      '### Problem',
      'chat_v2/pipeline.rs directly imports 13+ concrete executor types from tools/.',
      'Tight coupling: every new tool executor requires pipeline.rs modification.',
      '',
      '### Solution: Depend only on ExecutorRegistry trait',
      '',
      '### Files:',
      '- C:\\deep-student\\src-tauri\\src\\chat_v2\\pipeline.rs',
      '- C:\\deep-student\\src-tauri\\src\\chat_v2\\tools\\executor.rs',
      '',
      '### Step 1: Read pipeline.rs — identify all concrete executor imports',
      'List every import like: use crate::chat_v2::tools::some_executor::SomeExecutor;',
      '',
      '### Step 2: Read tools/executor.rs — understand the ExecutorRegistry trait',
      'Check if it already has a resolve_executor(name) -> Box<dyn ToolExecutor> method.',
      'If not, add it.',
      '',
      '### Step 3: Update pipeline.rs',
      'Remove ALL concrete executor type imports.',
      'Replace with ExecutorRegistry usage:',
      '  OLD: let executor = SomeExecutor::new(ctx);',
      '  NEW: let executor = registry.resolve("some_executor")?;',
      '',
      '### Step 4: Verify',
      'Search for any remaining concrete executor references in pipeline.rs',
      'Ensure the ExecutorRegistry implementation provides all needed executors',
      '',
      '### CRITICAL RULES:',
      '- Pipeline behavior must be IDENTICAL',
      '- Tool resolution by name must match exact previous behavior',
      '- Keep error handling for unknown tool names'
    ].join('\n');
    return agent(prompt, { label: 'pipeline_di', schema: P2_SCHEMA });
  },

  // Task 2.5: Consolidate thin LLM adapters
  function() {
    var prompt = [
      '## P2 Task 2.5: Consolidate Thin LLM Adapters',
      '',
      '### Problem',
      '5 of 14 LLM adapters (Grok, Mimo, Ernie, Doubao, Mistral) each add < 30 lines of unique logic.',
      'They exist as separate files but are essentially GenericOpenAIAdapter with tiny tweaks.',
      '',
      '### Solution: ProviderOverrides config in GenericOpenAIAdapter',
      '',
      '### Files to read (the 5 thin adapters):',
      '- C:\\deep-student\\src-tauri\\src\\llm_manager\\adapters\\grok.rs',
      '- C:\\deep-student\\src-tauri\\src\\llm_manager\\adapters\\mimo.rs',
      '- C:\\deep-student\\src-tauri\\src\\llm_manager\\adapters\\ernie.rs',
      '- C:\\deep-student\\src-tauri\\src\\llm_manager\\adapters\\doubao.rs',
      '- C:\\deep-student\\src-tauri\\src\\llm_manager\\adapters\\mistral.rs',
      '- C:\\deep-student\\src-tauri\\src\\llm_manager\\adapters\\generic_openai.rs',
      '- C:\\deep-student\\src-tauri\\src\\llm_manager\\adapters\\mod.rs (adapter registry)',
      '',
      '### Step 1: Read all 7 files. For each thin adapter, identify its UNIQUE logic:',
      '- What parameters does it add/remove/modify vs GenericOpenAIAdapter?',
      '- What is unique about its reasoning config?',
      '- What is unique about its response parsing?',
      '',
      '### Step 2: Define ProviderOverrides config',
      'In generic_openai.rs, add a ProviderOverrides struct:',
      '- params_to_remove: Vec<String> (e.g., ["frequency_penalty", "presence_penalty"])',
      '- params_to_add: HashMap<String, serde_json::Value>',
      '- reasoning_config_fn: Option<fn(&mut Map, &ApiConfig)>',
      '- tool_choice_override: Option<String>',
      '- sampling_removal: bool (remove temperature/top_p for reasoning)',
      '',
      '### Step 3: Implement override-based transformation',
      'Add apply_overrides(body: &mut Map, config: &ApiConfig, overrides: &ProviderOverrides) method',
      'GenericOpenAIAdapter.apply_reasoning_config() checks overrides first, then falls back to default',
      '',
      '### Step 4: Create override configs for each thin adapter',
      'For each of the 5 adapters, create a const PROVIDER_OVERRIDES: ProviderOverrides',
      'Move their unique logic into these configs',
      '',
      '### Step 5: Update adapter registry',
      'In adapters/mod.rs, remove the 5 thin adapter structs',
      'Register them as GenericOpenAIAdapter with provider-specific overrides',
      'Keep their ID strings for backward compatibility (e.g., "grok", "mimo", etc.)',
      '',
      '### Step 6: Remove thin adapter files',
      'Delete grok.rs, mimo.rs, ernie.rs, doubao.rs, mistral.rs',
      'Remove their pub use lines from adapters/mod.rs',
      'Remove their mod declarations',
      '',
      '### CRITICAL RULES:',
      '- EVERY unique behavior from thin adapters MUST be preserved via overrides',
      '- Adapter selection by provider_type string MUST still work',
      '- All existing tests for these adapters MUST be adapted or preserved as integration tests',
      '- GenericOpenAIAdapter behavior for OPENAI must NOT change',
      '- ProviderOverrides is an INTERNAL detail — public API unchanged'
    ].join('\n');
    return agent(prompt, { label: 'thin_adapter_merge', schema: P2_SCHEMA });
  }
]);

var completed = 0;
var totalFilesCreated = 0;
var totalFilesModified = 0;
var allBC = true;

for (var i = 0; i < p2Results.length; i++) {
  var r = p2Results[i];
  if (r) {
    log('');
    log('Task ' + r.task_key + ': ' + r.status);
    if (r.files_created) totalFilesCreated += r.files_created.length;
    if (r.files_modified) totalFilesModified += r.files_modified.length;
    if (r.status === 'completed') completed++;
    if (!r.backward_compatible) allBC = false;
    log('  Files created: ' + (r.files_created ? r.files_created.length : 0) + ', modified: ' + (r.files_modified ? r.files_modified.length : 0));
    log('  LOC affected: ' + r.loc_affected + ', BC: ' + r.backward_compatible);
    if (r.issues && r.issues.length > 0) {
      for (var j = 0; j < r.issues.length; j++) {
        log('  Issue: ' + r.issues[j]);
      }
    }
  }
}

log('');
log('========================================');
log('P2 CYCLE RESOLUTION + DI COMPLETE');
log('========================================');
log('');
log('Tasks completed: ' + completed + '/5');
log('Total files created: ' + totalFilesCreated);
log('Total files modified: ' + totalFilesModified);
log('All backward compatible: ' + allBC);

return {
  residuals: residualResult,
  p2_tasks: p2Results
};
