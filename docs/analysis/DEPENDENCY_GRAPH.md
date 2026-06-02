# Rust Dependency Graph Analysis

> Generated: 2026-06-01
> Source: `C:\deep-student\src-tauri\src`
> Total modules: **396** | Total internal import edges: **973**

---

## 1. Summary Statistics

| Metric | Value |
|--------|-------|
| Total Rust source files | 396 |
| Total internal import edges (crate::) | 973 |
| Circular dependency chains found | 4 |
| Modules with zero internal deps | 139 |
| Leaf modules (zero dependents) | 167 |
| Maximum fan-in | 82 (`crate::models`) |
| Maximum fan-out (internal) | 17 (`crate::vfs::handlers`) |
| Modules exceeding 20 internal imports | **0** |

---

## 2. Module Hierarchy Overview

### 2.1 Top-Level Module Structure

```
src-tauri/src/
├── lib.rs                          # Crate root: declares all top-level modules
├── main.rs                         # Binary entry point
├── commands.rs                     # Legacy command dispatcher (fan-out: 16)
├── models.rs                       # Shared data types (fan-in: 82 -- HIGHEST)
├── database/
├── chat_v2/                        # ~56 files -- core chat engine
│   ├── handlers/                   #   14 files -- Tauri command handlers
│   ├── pipeline/                   #   14 files -- message processing pipeline
│   ├── tools/                      #   24 files -- LLM tool executors
│   └── workspace/                  #   12 files -- workspace subagent system
├── vfs/                            # ~31 files -- Virtual File System
│   ├── repos/                      #   16 files -- database repositories
│   └── unit_builder/               #    4 files
├── llm_manager/                    # ~21 files -- LLM dispatch & adapters
│   └── adapters/                   #   14 files -- per-vendor LLM adapters
├── data_governance/                # ~28 files -- backup/sync/migration
│   ├── backup/                     #
│   ├── migration/                  #
│   └── sync/                       #
├── dstu/                           # ~23 files -- Document Structure Tree
│   ├── export/                     #
│   └── handler_utils/              #
├── memory/                         # 11 files -- memory (VFS-based)
├── mcp/                            # 11 files -- Model Context Protocol
├── multimodal/                     #  7 files -- multimodal embedding
├── ocr_adapters/                   #  8 files -- OCR backends
├── cloud_storage/                  #  6 files
├── cmd/                            # 11 files -- legacy command helpers
├── essay_grading/                  #  6 files
├── llm_usage/                      #  5 files
├── translation/                    #  5 files
├── utils/                          #  9 files -- shared utilities
├── crypto/                         #  3 files
├── providers/                      #  1 file  -- provider abstractions
├── tools/                          #  2 files
├── vendors/                        #  2 files
├── qbank_grading/                  #  3 files
├── test_utils/                     #  3 files
├── services/                       #  1 file
└── 40+ standalone root-level files #
```

---

## 3. Fan-In Analysis (Most Depended-On Modules)

These modules are the most widely imported across the codebase. High fan-in suggests they serve as shared infrastructure.

| Rank | Fan-In | Module | File | Role |
|------|--------|--------|------|------|
| 1 | **82** | `crate::models` | `models.rs` | Shared data models / DTOs |
| 2 | **57** | `crate::vfs::database` | `vfs/database.rs` | VFS database handle |
| 3 | **51** | `crate::llm_manager` | `llm_manager/mod.rs` | LLM dispatch root |
| 4 | **46** | `crate::chat_v2::types` | `chat_v2/types.rs` | Chat types/structs |
| 5 | **43** | `crate::vfs::repos` | `vfs/repos/mod.rs` | VFS repo re-exports |
| 6 | **42** | `crate::vfs::types` | `vfs/types.rs` | VFS type definitions |
| 7 | **39** | `crate::vfs::error` | `vfs/error.rs` | VFS error types |
| 8 | **31** | `crate::chat_v2::events` | `chat_v2/events.rs` | Chat event system |
| 9 | **29** | `crate::chat_v2::tools::executor` | `chat_v2/tools/executor.rs` | Tool executor trait |
| 10 | **28** | `crate::vfs` | `vfs/mod.rs` | VFS module root |
| 11 | **23** | `crate::database` | `database/mod.rs` | Database pool/manager |
| 12 | **17** | `crate::chat_v2::repo` | `chat_v2/repo.rs` | Chat repository |
| 13 | **16** | `crate::dstu::types` | `dstu/types.rs` | DSTU type defs |
| 14 | **15** | `crate::commands` | `commands.rs` | Legacy command glue |
| 15 | **15** | `crate::dstu::error` | `dstu/error.rs` | DSTU error types |
| 16 | **15** | `crate::vfs::lance_store` | `vfs/lance_store.rs` | LanceDB vector store |
| 17 | **13** | `crate::chat_v2::error` | `chat_v2/error.rs` | Chat error types |
| 18 | **13** | `crate::vfs::repos::folder_repo` | `vfs/repos/folder_repo.rs` | Folder repo (in cycle) |
| 19 | **12** | `crate::document_parser` | `document_parser.rs` | Document parsing |
| 20 | **12** | `crate::chat_v2::database` | `chat_v2/database.rs` | Chat database layer |

### Key Observation

`crate::models` (82 dependents) acts as the **universal type hub** -- nearly every module in the project imports types from `models.rs`. This is expected but carries high coupling risk: any change to `models.rs` triggers recompilation of 20%+ of the codebase.

---

## 4. Fan-Out Analysis (Modules with Most Internal Dependencies)

| Rank | Fan-Out | Module | File |
|------|---------|--------|------|
| 1 | **17** | `crate::vfs::handlers` | `vfs/handlers.rs` |
| 2 | **16** | `crate::commands` | `commands.rs` |
| 3 | **16** | `crate::chat_v2::handlers::send_message` | `chat_v2/handlers/send_message.rs` |
| 4 | **13** | `crate::chat_v2::handlers::variant_handlers` | `chat_v2/handlers/variant_handlers.rs` |
| 5 | **13** | `crate::chat_v2::tools::builtin_retrieval_executor` | `chat_v2/tools/builtin_retrieval_executor.rs` |
| 6 | **13** | `crate::chat_v2::tools::chatanki_executor` | `chat_v2/tools/chatanki_executor.rs` |
| 7 | **13** | `crate::vfs::pdf_processing_service` | `vfs/pdf_processing_service.rs` |
| 8 | **12** | `crate::question_import_service` | `question_import_service.rs` |
| 9 | **12** | `crate::memory::service` | `memory/service.rs` |
| 10 | **12** | `crate::vfs::indexing` | `vfs/indexing.rs` |

### Excessive Dependencies Check

**No module exceeds 20 internal imports.** The highest is `vfs::handlers` at 17. This threshold was chosen as a warning sign of a module trying to do too much; the codebase is within healthy bounds.

---

## 5. Circular Dependency Chains

### Cycle 1: VFS Folder Repo <-> Path Cache Repo
**Severity: MEDIUM**

```
crate::vfs::repos::folder_repo
  -> use crate::vfs::repos::path_cache_repo::VfsPathCacheRepo;
  <- use crate::vfs::repos::folder_repo::VfsFolderRepo;
crate::vfs::repos::path_cache_repo
```

**Files:**
- `vfs/repos/folder_repo.rs` (fan-in: 13)
- `vfs/repos/path_cache_repo.rs` (fan-in: 1)

**Root cause:** `folder_repo` calls into `path_cache_repo` for path operations, while `path_cache_repo` references `folder_repo` to resolve folder type IDs.

**Recommendation:** Extract the shared type (folder item type) into `vfs::types` or a shared base module (`vfs::repos::common` or similar), so both repos depend on a common type rather than each other.

---

### Cycle 2: Question Sync Service <-> Question Repo
**Severity: LOW -- cross-boundary dependency**

```
crate::question_sync_service (question_sync_service.rs)
  -> use crate::vfs::repos::question_repo::...;
crate::vfs::repos::question_repo (vfs/repos/question_repo.rs)
  -> use crate::question_sync_service::QuestionSyncService;
```

**Files:**
- `question_sync_service.rs` (root-level service)
- `vfs/repos/question_repo.rs` (VFS repo)

**Root cause:** `question_repo` directly imports `QuestionSyncService` to trigger sync operations after question mutations. The sync service in turn relies on `question_repo` for data access.

**Recommendation:** Introduce a callback/trait interface in the VFS layer. Have `question_repo` accept an optional sync callback (via `Box<dyn Fn>`) injected at initialization time, so the repo does not need a compile-time dependency on `question_sync_service`.

---

### Cycle 3: VFS Indexing Triad (3-node cycle)
**Severity: HIGH -- most complex cycle in system**

```
crate::vfs::embedding_service
  -> use crate::vfs::indexing::...
crate::vfs::indexing
  -> use crate::vfs::pdf_processing_service::...
  -> use crate::vfs::embedding_service::...
crate::vfs::pdf_processing_service
  -> use crate::vfs::indexing::...
```

**Files:**
- `vfs/embedding_service.rs`
- `vfs/indexing.rs` (fan-in: 8, fan-out: 12)
- `vfs/pdf_processing_service.rs` (fan-in: 4, fan-out: 13)

**Root cause:** All three services inter-depend: indexing orchestrates both embedding and PDF processing; embedding service reports progress back to indexing; PDF processing triggers indexing steps.

**Recommendation:** This is the most architecturally significant cycle. Mitigation options:
1. **Event-based decoupling**: Replace direct function calls with an event bus. `embedding_service` emits progress events; `pdf_processing_service` emits completion events; `indexing` subscribes to both.
2. **Introduce a coordinator**: Create `vfs::indexing::coordinator` that owns references to both `embedding_service` and `pdf_processing_service`, removing their need to know about each other.
3. **Trait extraction**: Define `EmbeddingProvider` and `PdfProcessor` traits in `vfs/mod.rs`. Have each service implement the trait; the indexing service depends on trait definitions only.

---

### Cycle 4: Data Governance Backup <-> Restore
**Severity: LOW**

```
crate::data_governance::commands_backup
  -> use super::commands_restore::...;
crate::data_governance::commands_restore
  -> use super::commands_backup::...;
```

**Files:**
- `data_governance/commands_backup.rs`
- `data_governance/commands_restore.rs`

**Root cause:** Backup commands reference restore types for partial restore operations; restore commands reference backup types for rollback capability.

**Recommendation:** Extract the shared types (restore-from-backup params, rollback spec) into a `commands_shared.rs` module under `data_governance/`.

---

## 6. Module Group Deep Dives

### 6.1 `llm_manager/` (21 files, 1 adapter per vendor)

```
llm_manager/
├── mod.rs                    # Root: fans out to 8 modules
├── builtin_vendors.rs        # Isolated: no internal deps
├── parser.rs                 # Isolated: no internal deps
├── exam_engine.rs            # Depends on: deepseek_ocr_parser, models, ocr_adapters, providers
├── model2_pipeline.rs        # Depends on: llm_manager, models, providers, reasoning_policy
├── rag_extension.rs          # Depends on: models, multimodal, providers, utils::text
└── adapters/                 # 14 files, one per vendor
    ├── mod.rs                # Re-exports: depends on crate::llm_manager
    ├── anthropic.rs          # Depends on crate::llm_manager
    ├── deepseek.rs           # Depends on crate::llm_manager
    ├── gemini.rs             # Depends on crate::llm_manager
    ├── ... (one per vendor)
    └── streaming_harness.rs  # Depends on: crate::llm_manager::adapters, crate::providers
```

**Key observations:**
- Each vendor adapter depends only on `crate::llm_manager` (the parent). No cross-adapter dependencies. This is a clean architecture.
- `streaming_harness.rs` is the only adapter with a non-parent dependency (`crate::providers`).
- `llm_manager/mod.rs` is the 3rd most depended-on module (fan-in 51) -- it is imported by chat_v2, vfs, memory, and many other services to make LLM calls.
- `exam_engine` and `model2_pipeline` are self-contained sub-engines within llm_manager.

---

### 6.2 `chat_v2/` (56 files)

```
chat_v2/
├── mod.rs                    # Root: depends on 4 internal modules
├── types.rs                  # Fan-in 46: shared chat data models
├── events.rs                 # Fan-in 31: event system
├── error.rs                  # Fan-in 13: error types
├── database.rs               # Fan-in 12: SQL database layer
├── repo.rs                   # Fan-in 17: repository
├── context.rs                # Context management
├── state.rs                  # App state
├── skills.rs                 # Skill system
├── resource_types.rs         # Content/resource types
├── prompt_builder.rs         # Prompt construction
├── user_message_builder.rs   # User message construction
├── variant_context.rs        # Variant context management
├── vfs_resolver.rs           # VFS cross-resolution
├── pipeline_tests.rs         # Integration tests
├── approval_manager.rs       # Approval flow
├── approval_scope.rs         # Approval scoping
├── handlers/                 # 14 files: Tauri command implementations
│   ├── send_message.rs       # Fan-out 16: message dispatch hub
│   ├── variant_handlers.rs   # Fan-out 13: variant management
│   ├── block_actions.rs      # Fan-out 9
│   ├── manage_session.rs     # Fan-out 8
│   ├── workspace_handlers.rs # Fan-out 6
│   ├── approval_handlers.rs  # Fan-out 5
│   ├── ...
├── pipeline/                 # 14 files: message processing pipeline
│   ├── compaction.rs
│   ├── history.rs
│   ├── prompt.rs
│   ├── tool_loop.rs
│   ├── multi_variant.rs
│   ├── llm_adapter.rs
│   └── ...
├── tools/                    # 24 files: LLM tool executors
│   ├── executor.rs           # Fan-in 29: ToolError + ToolExecutor trait
│   ├── registry.rs           # Tool registry
│   ├── builtin_retrieval_executor.rs  # Fan-out 13
│   ├── chatanki_executor.rs           # Fan-out 13
│   ├── qbank_executor.rs             # Fan-out 9
│   └── ... (20 more executors)
├── workspace/                # 12 files: subagent workspace system
│   ├── coordinator.rs
│   ├── inbox.rs
│   ├── router.rs
│   ├── emitter.rs
│   └── ...
└── migration/                # Migration support
```

**Key observations:**
- `types.rs` and `events.rs` are the two primary shared modules, used extensively across chat_v2 and into other modules.
- `send_message.rs` has the highest fan-out (16) in chat_v2, acting as the central orchestrator for message dispatch.
- `executor.rs` (tools) has high fan-in (29) -- the `ToolExecutor` trait is the interface all tool implementations adhere to.
- No module in chat_v2 exceeds 20 internal imports; the internal structure is well-partitioned.
- The tools directory has many small, focused executor files -- each typically depending on 3-8 internal modules.

---

### 6.3 `vfs/` (31 files)

```
vfs/
├── mod.rs                    # Fan-in 28: root
├── database.rs               # Fan-in 57: VFS database handle (VfsDatabase)
├── error.rs                  # Fan-in 39: VfsError type
├── types.rs                  # Fan-in 42: shared VFS types
├── handlers.rs               # Fan-out 17 (highest in codebase)
├── index_handlers.rs         # Fan-out 7
├── indexing.rs               # Fan-out 12 (in cycle 3)
├── index_service.rs          # Fan-out 6
├── embedding_service.rs      # (in cycle 3)
├── pdf_processing_service.rs # Fan-out 13 (in cycle 3)
├── multimodal_service.rs     # Fan-out 9
├── lance_store.rs            # Fan-in 15: LanceDB vector store
├── attachment_config.rs
├── ocr_storage.rs
├── ocr_storage_handlers.rs
├── ocr_utils.rs
├── ref_handlers.rs
├── todo_handlers.rs
├── repos/                    # 16 files: Database repositories
│   ├── mod.rs                # Fan-in 43: re-exports all repos
│   ├── folder_repo.rs        # Fan-in 13 (in cycle 1)
│   ├── file_repo.rs
│   ├── note_repo.rs
│   ├── resource_repo.rs
│   ├── question_repo.rs      # (in cycle 2)
│   ├── path_cache_repo.rs    # (in cycle 1)
│   └── ... (9 more repos)
└── unit_builder/             # 4 files
    ├── mod.rs
    ├── trait_def.rs
    ├── builders.rs
    └── registry.rs
```

**Key observations:**
- `vfs::database` has fan-in 57, making it the 2nd most depended-on module overall. `VfsDatabase` is the primary handle for accessing VFS storage.
- `vfs::handlers` has the highest fan-out in the entire codebase (17) -- it orchestrates all VFS operations and depends on many sub-modules.
- VFS repos re-export through `mod.rs` (fan-in 43) -- consumers import `crate::vfs::repos::VfsFooRepo` without needing to know which specific file a repo lives in.
- The VFS module contains 2 of the 4 circular dependency chains (cycles 1 and 3).
- `repos/mod.rs` serves as a bulking re-export module, with fan-in 43 but fan-out 0.

---

### 6.4 `dstu/` (23 files)

```
dstu/
├── mod.rs                    # Root (no internal deps)
├── types.rs                  # Fan-in 16: DSTU type definitions
├── error.rs                  # Fan-in 15: DstuError
├── path_parser.rs            # Path parsing
├── path_types.rs             # Path type definitions
├── handlers.rs               # Main handlers (fan-out 4)
├── folder_handlers.rs        # Folder operations
├── trash_handlers.rs         # Trash operations (fan-out 6)
├── exam_formatter.rs         # Exam formatting
├── export/                   # 8 files: export adapters
│   ├── mod.rs
│   ├── essay_adapter.rs      # Each depends on crate::dstu + crate::models
│   ├── exam_adapter.rs
│   ├── file_adapter.rs
│   ├── image_adapter.rs
│   ├── mindmap_adapter.rs
│   ├── note_adapter.rs
│   ├── textbook_adapter.rs
│   └── translation_adapter.rs
└── handler_utils/            # 7 files: shared handler logic
    ├── mod.rs                # Re-exports
    ├── crud.rs
    ├── content_helpers.rs
    ├── delete_helpers.rs
    ├── list_helpers.rs
    ├── node_converters.rs
    ├── path_utils.rs
    └── search_helpers.rs
```

**Key observations:**
- `dstu::types` (fan-in 16) and `dstu::error` (fan-in 15) are the shared infrastructure for the DSTU module.
- The export adapters are cleanly separated, each depending on `crate::dstu` and `crate::models` -- no cross-adapter dependencies.
- No circular dependencies within dstu.
- Maximum fan-out in dstu is 6 (`trash_handlers`).

---

### 6.5 `data_governance/` (28 files)

```
data_governance/
├── mod.rs                    # Root
├── init.rs                   # Initialization
├── plugin.rs                 # Tauri plugin
├── commands.rs               # Fan-out 8: main commands
├── commands_asset.rs
├── commands_backup.rs        # (in cycle 4)
├── commands_restore.rs       # (in cycle 4)
├── commands_sync.rs
├── commands_types.rs
├── commands_zip.rs
├── schema_registry.rs        # Fan-in 10
├── critical_audit_tests.rs
├── migration_tests.rs
├── tests.rs
├── audit/                    # Audit logging
│   └── mod.rs                # Fan-in 11
├── backup/                   # Backup system
│   ├── mod.rs
│   ├── assets.rs
│   └── zip_export.rs
├── dto/                      # DTOs
│   └── mod.rs
├── migration/                # Data migration
│   ├── mod.rs
│   ├── chat_v2.rs
│   ├── coordinator.rs
│   ├── definitions.rs
│   ├── llm_usage.rs
│   ├── mistakes.rs
│   ├── script_checker.rs
│   ├── verifier.rs
│   └── vfs.rs
└── sync/                     # Sync system
    ├── mod.rs
    ├── classification.rs
    ├── conflict_resolver.rs
    ├── emitter.rs
    ├── field_merge.rs
    ├── hlc.rs
    ├── progress.rs
    └── tombstone.rs
```

**Key observations:**
- `data_governance::migration::definitions` has fan-in 4 (used by multiple migration units).
- `schema_registry` has fan-in 10 -- used broadly to register/manage schemas.
- The `commands_backup` <-> `commands_restore` cycle is the smallest cycle and easiest to fix.
- The sync subsystem is cleanly separated, with `sync/mod.rs` having fan-out 1 and each sync file being largely independent.

---

## 7. Cross-Module Dependency Map

Below is a high-level view of how the major modules relate to each other (arrows show "depends on"):

```
                    ┌──────────┐
                    │  models  │◄────────────────────────────────────┐
                    │ (82 fi)  │◄──────────────────┐                 │
                    └────┬─────┘                   │                 │
                         │                         │                 │
                    ┌────▼─────┐           ┌───────▼──────┐         │
                    │vfs::db   │           │ vfs::types   │         │
                    │ (57 fi)  │           │  (42 fi)     │         │
                    └────┬─────┘           └───────┬──────┘         │
                         │                         │                 │
         ┌───────────────┤              ┌──────────┤                 │
         │               │              │          │                 │
    ┌────▼────┐    ┌─────▼──────┐  ┌────▼───┐  ┌──▼───────┐        │
    │ chat_v2 │    │    vfs/    │  │ memory │  │ dstu/    │        │
    │ (56 ff) │◄───┤  (31 ff)   │  │ (11 ff)│  │ (23 ff)  │        │
    └──┬──┬───┘    └─────┬──────┘  └──┬─────┘  └──┬───────┘        │
       │  │              │            │           │                 │
       │  │         ┌────▼────┐  ┌────▼─────┐  ┌──▼───────┐        │
       │  └─────────► llm_    │  │ data_    │  │ essay_   │        │
       │            │ manager │  │ gov.     │  │ grading  │        │
       │            │ (51 fi) │  │ (28 ff)  │  │ (6 ff)   │        │
       │            └────┬────┘  └──────────┘  └──────────┘        │
       │                 │                                          │
       │            ┌────▼────┐                                     │
       └────────────►vendors/ │                                     │
                    │providers│                                     │
                    └─────────┘                                     │
                                                                    │
                    Everyone imports ───►  models (82 dependents) ──┘
```

---

## 8. Dependency Quality Assessment

### 8.1 Strengths

1. **Clean adapter pattern in `llm_manager/adapters/`**: Each vendor adapter depends only on its parent module. Easy to add new vendors without touching existing code.

2. **Well-factored tool executors in `chat_v2/tools/`**: Each executor is a small file (typical fan-out 3-8) with focused responsibility, all implementing a shared `ToolExecutor` trait.

3. **Re-export pattern via `vfs::repos::mod`**: Consumers use `crate::vfs::repos::VfsFooRepo` without coupling to the specific file path, enabling repo file reorganization without consumer changes.

4. **No module exceeds 20 internal imports**: The codebase has a healthy cap on how many modules any single file depends on.

5. **DSU module has zero cycles**: Despite having 23 files across submodules, the DSTU module is architecturally clean.

### 8.2 Weaknesses / Risks

1. **4 circular dependency chains exist** -- 2 of them in the critical VFS indexing path (cycle 3 is the most architecturally significant).

2. **`crate::models` is a single-point coupling hub** (82 dependents). Any change risks widespread recompilation and potential API breakage. Consider splitting `models.rs` into domain-specific sub-modules (e.g., `models::ocr`, `models::chat`, `models::vfs`, `models::anki`).

3. **`vfs::handlers` (fan-out 17) is a "god handler"**: It depends on nearly every VFS sub-module. Consider breaking it into domain-specific handler files (e.g., `ocr_handlers.rs`, `search_handlers.rs`, `file_handlers.rs`).

4. **`question_sync_service` and `question_repo` cross-boundary cycle**: A non-VFS service (`question_sync_service`) and a VFS repository (`question_repo`) form a cycle -- suggesting an architectural boundary violation.

5. **Many root-level files**: There are ~40+ standalone `.rs` files at the root of `src/`. Some could be grouped into sub-modules (e.g., all Anki-related services into a `services/anki/` module).

---

## 9. Recommendations (Prioritized)

| Priority | Issue | Recommendation | Effort |
|----------|-------|---------------|--------|
| P0 | Cycle 3: VFS indexing triad | Extract event-based interface or coordinator | Medium |
| P1 | Cycle 1: folder/path_cache repo | Extract shared type into vfs::types | Small |
| P2 | Cycle 2: question_sync/question_repo | Introduce sync callback trait | Small |
| P2 | Cycle 4: backup/restore | Extract shared types into commands_shared | Small |
| P2 | models.rs (82 dependents) | Split into domain sub-modules | Medium |
| P3 | vfs::handlers (fan-out 17) | Split into domain-specific handler files | Large |

---

## Appendix A: Methodology

- **Scope**: All 396 Rust source files under `C:\deep-student\src-tauri\src`
- **Analysis method**: Python script (custom) parsing `use crate::...` and `use super::...` statements
- **External crate dependencies excluded**: `std`, `serde`, `tokio`, `tauri`, `chrono`, `rusqlite`, `regex`, `reqwest`, and 60+ other external crates are excluded from edge counting. Only intra-crate dependencies (`crate::`) are tracked.
- **Fan-in**: Number of modules that import a given module
- **Fan-out**: Number of unique internal modules a given module imports
- **Circular detection**: Tarjan's strongly connected components algorithm, filtering for SCCs with >= 2 nodes

## Appendix B: Raw Edge Count by Module Group

| Module Group | Files | Internal Edges | Avg Edges/File |
|-------------|-------|---------------|----------------|
| chat_v2/ | 56 | 224 | 4.0 |
| vfs/ | 31 | 154 | 5.0 |
| data_governance/ | 28 | 61 | 2.2 |
| dstu/ | 23 | 52 | 2.3 |
| llm_manager/ | 21 | 34 | 1.6 |
| mcp/ | 11 | 13 | 1.2 |
| memory/ | 11 | 32 | 2.9 |
| utils/ | 9 | 5 | 0.6 |
| ocr_adapters/ | 8 | 2 | 0.3 |
| multimodal/ | 7 | 19 | 2.7 |
| cloud_storage/ | 6 | 9 | 1.5 |
| cmd/ | 11 | 29 | 2.6 |
| essay_grading/ | 6 | 10 | 1.7 |
| llm_usage/ | 5 | 3 | 0.6 |
| translation/ | 5 | 10 | 2.0 |
| crypto/ | 3 | 0 | 0.0 |
| providers/ | 1 | 0 | 0.0 |
| tools/ | 2 | 0 | 0.0 |
| vendors/ | 2 | 1 | 0.5 |
| qbank_grading/ | 3 | 6 | 2.0 |
| test_utils/ | 3 | 3 | 1.0 |
| Root-level files | 40+ | 307 | 7.7 |
