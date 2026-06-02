# Cross-Module Interface Consistency Check Report

> Date: 2026-06-02
> Source: INTERFACE_DB.json + source code scan (src-tauri/src/)

## Summary

| Category | Status | Issues Found |
|----------|--------|-------------|
| P1: God file decomposition | PASS | 0 |
| P2: Trait extraction (MemoryStorage) | FAIL | 1 |
| P2: Trait extraction (QuestionSyncCallback) | PASS | 0 |
| P2: Adapter consolidation | PASS | 0 |
| P3: Dependency API changes | PASS | 0 |

---

## P1: God File Decomposition (old single-file -> multi-file paths)

### vfs/handlers.rs -> vfs/handlers/* (13 files)

The old `vfs/handlers.rs` single file no longer exists. The `vfs/handlers/` directory module with `mod.rs` + 13 sub-modules is the sole structure.

- No `mod handlers;` declaration points to a single file -- it resolves to the directory.
- All imports in `lib.rs` and elsewhere use the correct paths:
  - `crate::vfs::handlers::resource_handlers::vfs_create_or_reuse`
  - `crate::vfs::handlers::note_handlers::vfs_create_note`
  - etc.
- The mod.rs re-exports all public functions from sub-modules, maintaining backward compatibility.

**Verdict: PASS**

### dstu/handlers.rs -> dstu/handlers/* (10 files)

Old single file gone. Directory module with `mod.rs` + 10 sub-modules is sole structure.

- `lib.rs` imports use `crate::dstu::handlers::common::dstu_list`, `crate::dstu::handlers::search_handlers::dstu_search`, etc.
- The `dstu/mod.rs` also re-exports `pub use handlers::*` for top-level convenience.

**Verdict: PASS**

### data_governance/sync/mod.rs -> sync/* (5+ files)

Already decomposed into `orchestrator, manifest, changeset, conflict_resolver, emitter, field_merge, hlc, progress, tombstone, retry, classification` (11 files).

- Imports reference `crate::data_governance::sync::ConflictPolicy` (re-exported from mod.rs) or `crate::data_governance::sync::orchestrator::SyncManager`.
- No old single-sync-file references detected.

**Verdict: PASS**

### llm_manager/mod.rs -> (6 domain files)

The single `mod.rs` still exists as a directory module with sub-modules: `adapters, builtin_vendors, config_types, exam_engine, image_processing, model2_pipeline, model_profile_service, parser, rag_extension, streaming, tool_call, vendor_config_service`.

- No code imports from non-existent sub-module paths.
- LLMManager struct and core methods remain in `mod.rs`, domain types split into separate files.

**Verdict: PASS**

---

## P2: Trait Extraction

### MemoryStorage trait

**Trait definition**: `memory/storage_trait.rs` defines `MemoryStorage` with `conn()`, `conn_unchecked()`, and ~20 abstract methods.

**Implementation**: `VfsMemoryStorage` in the same file wraps `Arc<VfsDatabase>`, `Arc<VfsLanceStore>`, `Arc<LLMManager>`, implementing trait methods by delegating to VFS repos.

**Consumers**:
- `memory/service.rs` uses `Arc<dyn MemoryStorage>` internally (field `storage`), calls `storage.conn()` for DB access.
- `memory/audit_log.rs` uses `self.storage.conn()` through `MemoryAuditLogger`.

**Issue found (FIXED)**: `memory/handlers.rs` had a concrete bypass of the `MemoryStorage` trait.
- File: `src/memory/handlers.rs`
- The helper `get_memory_service()` (line 73-78) takes concrete `&Arc<VfsDatabase>` and constructs a `MemoryService` from it, consistent with Tauri's State injection pattern.
- All 29 Tauri command functions take `vfs_db: State<'_, Arc<VfsDatabase>>` (concrete VFS type) as required by Tauri's DI, but immediately wrap into `Arc<dyn MemoryStorage>` via the service constructor.
- **FIXED**: `memory_get_audit_logs` (was line 823) directly called `vfs_db.get_conn_safe()` instead of using `storage.conn()`. Now creates a `MemoryService` and uses `service.storage_ref().conn()` to access the DB through the MemoryStorage trait.

**Impact**: Handlers necessarily use concrete types for Tauri State injection, but all business logic goes through `MemoryService` which uses the trait. The one specific trait bypass in `memory_get_audit_logs` has been resolved.

**Verdict: PASS (fix applied)**

### QuestionSyncCallback trait

**Trait definition**: `vfs/repos/question_repo.rs` line 38.
**Implementation**: `question_sync_service.rs` implements `QuestionSyncCallback` for `QuestionSyncService`.
**Registration**: Via `register_question_sync_callback()` storing in `OnceLock<Box<dyn QuestionSyncCallback>>`.
**Usage**: Callers use `with_question_sync_callback(|cb| cb.some_method(...))`.

- No direct concrete-type references outside the registration/implementation boundary.
- This pattern correctly hides `QuestionSyncService` behind the trait.

**Verdict: PASS**

---

## P2: Adapter Consolidation

The 5 thin adapter modules (grok, mimo, ernie, doubao, mistral) that previously existed as separate files have been removed:

- `llm_manager/adapters/grok.rs` -- does not exist
- `llm_manager/adapters/mimo.rs` -- does not exist
- `llm_manager/adapters/ernie.rs` -- does not exist
- `llm_manager/adapters/doubao.rs` -- does not exist
- `llm_manager/adapters/mistral.rs` -- does not exist

Their adapter logic is now embedded inside `llm_manager/adapters/generic_openai.rs` as inline functions (e.g., `grok_reasoning_config`, `mimo_reasoning_config`, `ernie_reasoning_config`, `doubao_reasoning_config`, `mistral_reasoning_config`) within the `GenericOpenaiAdapter` framework.

- No `mod grok;` / `mod mimo;` etc. declarations exist in `llm_manager/adapters/mod.rs`.
- No import of these modules from any other file.

**Verdict: PASS**

---

## P3: Dependency API Changes

### reqwest 0.13 -- `is_dns()`

Searched entire `src/` for `is_dns()`. Zero results.

**Verdict: PASS**

### oauth2 5.0 -- `async_http_client()`

Searched entire `src/` for `async_http_client()`. Zero results.

**Verdict: PASS**

### base64 0.22 -- `base64::encode` / `base64::decode` (old API)

Searched entire `src/` for `base64::encode` and `base64::decode`. Zero results.

All base64 usage uses the 0.22+ API:
- `base64::engine::general_purpose::STANDARD.encode(...)`
- `base64::engine::general_purpose::STANDARD.decode(...)`
- `use base64::{engine::general_purpose, Engine as _};`

**Verdict: PASS**

---

## Edge Cases Examined

### Dual `ref_handlers` modules
`vfs/ref_handlers.rs` (2543 lines, reference mode commands) and `vfs/handlers/ref_handlers.rs` (293 lines, path cache commands) are genuinely different modules serving different purposes. Not an inconsistency.

### Dual `index_handlers` modules
`vfs/index_handlers.rs` (8.7KB, unified index commands) and `vfs/handlers/index_handlers.rs` (71KB, search/dimension/RAG commands) serve different command sets. Not an inconsistency.

### Thin re-export wrappers
`vfs/handlers/todo_handlers.rs` and `vfs/handlers/pomodoro_handlers.rs` simply re-export from `vfs::todo_handlers`. These are dead code (not imported by `lib.rs`) but do not cause compilation errors. Minor code smell only.

---

## Conclusions

1. **MemoryStorage trait bypass (FIXED)**: `memory_get_audit_logs` in `src/memory/handlers.rs` was directly calling `vfs_db.get_conn_safe()` instead of using the `MemoryStorage` trait. Fixed to use `service.storage_ref().conn()` through the service layer.

2. **All other categories clean**. No old import paths, no removed-adapter references, no deprecated dependency APIs found.

3. **INTERFACE_DB.json** is consistent with the current module structure across all 432 modules checked.
