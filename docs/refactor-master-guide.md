# Deep Student — Refactoring Master Guide

- **Version**: 1.0
- **Generated**: 2026-05-30
- **Project**: Deep Student v0.9.40 — AI-powered learning assistant
- **Tech Stack**: Tauri 2 (Rust 1.96 backend) + React 18 / TypeScript (Vite 6 frontend)
- **Repository**: `C:\deep-student` (https://github.com/user/deep-student)

## Purpose

This document is the single authoritative entry point for all refactoring work on the Deep Student project. It defines the architectural layers, dependency constraints, API standards, validation strategy, and remaining work inventory. Every refactoring decision — from error type unification to module splitting to CSS migration — traces back to this guide.

## Intended Audience

- Developers contributing refactoring changes to the Deep Student codebase
- Code reviewers evaluating architectural compliance
- Project maintainers planning sprints and tracking technical debt reduction
- Automated tools (linters, dependency checkers, CI pipelines) that enforce the rules defined herein

---

## 1. Refactoring Layer Definitions

The codebase is organized into six layers (0 through 5). Each layer has strict dependency rules: **a module in layer N may only depend on modules in layer N-1 or below**. No upward or lateral dependencies are permitted unless explicitly exempted (see Section 3).

### Layer 0 — Base Types & Constants

Foundation types, error enums, shared constants, and primitive data structures. Zero dependencies on other project modules.

| Module | Path | Key Contents |
|--------|------|-------------|
| Shared models | `src-tauri/src/models.rs` | `AppError` enum, shared type definitions |
| TS types | `src/types/` | TypeScript type definitions, interfaces |
| TS shared | `src/shared/` | Shared frontend constants and utilities |
| Rust lib | `src-tauri/src/lib.rs` | Module registration, Tauri plugin wiring |
| Theme/config | `src/config/` | Frontend configuration, theme tokens |

### Layer 1 — Utility Functions & Pure Logic

Stateless utilities, helper functions, database connection management, and pure computations. May depend on Layer 0 only.

| Module | Path | Key Contents |
|--------|------|-------------|
| Database | `src-tauri/src/database/` | Connection pooling, migrations, query helpers |
| Utils (Rust) | `src-tauri/src/utils/` | General-purpose Rust utilities |
| Utils (TS) | `src/utils/` | Frontend utility functions |
| Lib (TS) | `src/lib/` | Frontend library code |
| Services (TS) | `src/services/` | Frontend service layer |
| i18n | `src/locales/` `src/i18n.ts` | Internationalization strings (zh-CN) |
| Shims | `src/shims/` | Browser compatibility shims |
| Polyfills | `src/polyfills/` | Polyfill imports |

### Layer 2 — Data Models & State

State management stores (Zustand), type definitions for domain entities, data transfer objects. May depend on Layers 0-1.

| Module | Path | Key Contents |
|--------|------|-------------|
| Stores (TS) | `src/stores/` | Zustand state stores |
| Store (TS) | `src/store/` | Additional store definitions |
| VFS types | `src-tauri/src/vfs/types.rs` | Virtual filesystem entity types |
| VFS repos | `src-tauri/src/vfs/repos/` | VFS repository implementations |
| Chat V2 types | `src-tauri/src/chat_v2/types.rs` | Chat entity types |
| Chat V2 resource types | `src-tauri/src/chat_v2/resource_types.rs` | Resource type definitions |
| DSTU types | `src-tauri/src/dstu/types.rs` | DSTU resource protocol types |
| DSTU path types | `src-tauri/src/dstu/path_types.rs` | DSTU path representation types |
| Memory config | `src-tauri/src/memory/config.rs` | Memory module configuration |
| Memory types | `src-tauri/src/memory/` | Memory entity types |
| Essay grading types | `src-tauri/src/essay_grading/types.rs` | Essay grading entity types |
| Error definitions | `src-tauri/src/*/error.rs` | All module-level error enums |
| Data governance DTO | `src-tauri/src/data_governance/dto/` | Data governance transfer objects |
| Cloud storage config | `src-tauri/src/cloud_storage/config.rs` | S3/WebDAV configuration types |
| Secure store | `src-tauri/src/secure_store.rs` | Encrypted credential storage |
| Config recovery | `src-tauri/src/config_recovery.rs` | Configuration backup/recovery |
| Space (TS) | `src/data/` | Static data files |
| Events (TS) | `src/events/` | Frontend event type definitions |

### Layer 3 — Core Business Logic

Domain services, pipelines, algorithm implementations, repository patterns, and engine code. May depend on Layers 0-2.

| Module | Path | Key Contents |
|--------|------|-------------|
| Chat V2 pipeline | `src-tauri/src/chat_v2/pipeline/` | Chat processing pipeline stages |
| Chat V2 tools | `src-tauri/src/chat_v2/tools/` | Chat tool implementations |
| Chat V2 workspace | `src-tauri/src/chat_v2/workspace/` | Workspace management logic |
| Chat V2 context | `src-tauri/src/chat_v2/context.rs` | Chat context management |
| Chat V2 adapters | `src-tauri/src/chat_v2/adapters/` | External adapter implementations |
| Chat V2 state | `src-tauri/src/chat_v2/state.rs` | Chat state management |
| Chat V2 events | `src-tauri/src/chat_v2/events.rs` | Chat event handling |
| Chat V2 skills | `src-tauri/src/chat_v2/skills.rs` | Chat skill definitions |
| Chat V2 variant | `src-tauri/src/chat_v2/variant_context.rs` | Variant context logic |
| Chat V2 user message | `src-tauri/src/chat_v2/user_message_builder.rs` | Message construction |
| Memory service | `src-tauri/src/memory/service.rs` | Smart memory service |
| Memory auto-extractor | `src-tauri/src/memory/auto_extractor.rs` | Automatic memory extraction |
| Memory category manager | `src-tauri/src/memory/category_manager.rs` | Memory categorization |
| Memory compressor | `src-tauri/src/memory/compressor.rs` | Memory compression logic |
| Memory evolution | `src-tauri/src/memory/evolution.rs` | Memory evolution algorithms |
| Memory query rewriter | `src-tauri/src/memory/query_rewriter.rs` | Query rewriting for memory search |
| Memory reranker | `src-tauri/src/memory/reranker.rs` | Result reranking |
| Memory LLM decision | `src-tauri/src/memory/llm_decision.rs` | LLM-based memory decisions |
| Essay grading pipeline | `src-tauri/src/essay_grading/pipeline.rs` | Essay grading pipeline |
| Essay grading events | `src-tauri/src/essay_grading/events.rs` | Essay grading event types |
| Essay grading custom modes | `src-tauri/src/essay_grading/custom_modes.rs` | Custom grading modes |
| Essay grading text stats | `src-tauri/src/essay_grading/text_stats.rs` | Text statistics computation |
| VFS repos | `src-tauri/src/vfs/repos/` | VFS data access repositories |
| VFS services | `src-tauri/src/vfs/*_service.rs` | VFS indexing, embedding, OCR services |
| VFS unit builder | `src-tauri/src/vfs/unit_builder/` | VFS unit construction logic |
| VFS lance store | `src-tauri/src/vfs/lance_store.rs` | LanceDB integration |
| DSTU exam formatter | `src-tauri/src/dstu/exam_formatter.rs` | Exam formatting logic |
| DSTU path parser | `src-tauri/src/dstu/path_parser.rs` | DSTU path parsing |
| LLM manager | `src-tauri/src/llm_manager/` | LLM provider management (9 providers) |
| LLM manager adapters | `src-tauri/src/llm_manager/adapters/` | Provider adapter implementations |
| LLM manager model pipeline | `src-tauri/src/llm_manager/model2_pipeline.rs` | Model execution pipeline |
| LLM manager parser | `src-tauri/src/llm_manager/parser.rs` | LLM response parsing |
| LLM manager RAG extension | `src-tauri/src/llm_manager/rag_extension.rs` | RAG extension logic |
| LLM manager exam engine | `src-tauri/src/llm_manager/exam_engine.rs` | Exam generation engine |
| Data governance | `src-tauri/src/data_governance/` | Backup, sync, audit, migration logic |
| Cloud storage | `src-tauri/src/cloud_storage/` | S3, WebDAV, sync manager |
| Review plan service | `src-tauri/src/review_plan_service.rs` | Review scheduling logic |
| Question sync service | `src-tauri/src/question_sync_service.rs` | Question bank synchronization |
| QBank grading | `src-tauri/src/qbank_grading/` | Question bank grading logic |
| LLM usage | `src-tauri/src/llm_usage/` | LLM usage tracking |
| Enhanced Anki service | `src-tauri/src/enhanced_anki_service.rs` | Enhanced Anki card generation |
| Anki connect service | `src-tauri/src/anki_connect_service.rs` | AnkiConnect integration |
| Streaming Anki service | `src-tauri/src/streaming_anki_service.rs` | Streaming Anki operations |
| Notes manager | `src-tauri/src/notes_manager.rs` | Notes management logic |
| Notes exporter | `src-tauri/src/notes_exporter.rs` | Notes export functionality |
| Document parser | `src-tauri/src/document_parser.rs` | Document parsing logic |
| Document processing | `src-tauri/src/document_processing_service.rs` | Document processing pipeline |
| DeepSeek OCR parser | `src-tauri/src/deepseek_ocr_parser.rs` | DeepSeek OCR integration |
| PDF OCR service | `src-tauri/src/pdf_ocr_service.rs` | PDF OCR processing |
| PDFium utils | `src-tauri/src/pdfium_utils.rs` | PDFium rendering utilities |
| Page rasterizer | `src-tauri/src/page_rasterizer.rs` | Page rasterization |
| Translation | `src-tauri/src/translation/` | Translation pipeline |
| Figure extractor | `src-tauri/src/figure_extractor.rs` | Figure extraction from documents |
| JSON validator | `src-tauri/src/json_validator.rs` | JSON validation utilities |
| Lance vector store | `src-tauri/src/lance_vector_store.rs` | Vector store implementation |
| Vector store | `src-tauri/src/vector_store.rs` | Alternative vector storage |
| Spaced repetition | `src-tauri/src/spaced_repetition.rs` | Spaced repetition algorithm |
| Cross-page merger | `src-tauri/src/cross_page_merger.rs` | Cross-page content merging |
| Multi-modal | `src-tauri/src/multimodal/` | Multi-modal processing |
| OCR adapters | `src-tauri/src/ocr_adapters/` | OCR adapter implementations |
| OCR circuit breaker | `src-tauri/src/ocr_circuit_breaker.rs` | OCR circuit breaker pattern |
| Providers | `src-tauri/src/providers/` | External provider integrations |
| Tools | `src-tauri/src/tools/` | Generic tool implementations |
| Vendors | `src-tauri/src/vendors/` | Vendor-specific integrations |
| Services | `src-tauri/src/services/` | General backend services |
| Batch operations | `src-tauri/src/batch_operations.rs` | Batch processing logic |
| File manager | `src-tauri/src/file_manager.rs` | File management operations |
| Unified file manager | `src-tauri/src/unified_file_manager.rs` | Unified file operations |
| MCP | `src-tauri/src/mcp/` | MCP client implementation |
| Crypto | `src-tauri/src/crypto/` | Cryptographic operations |
| Engines (TS) | `src/engines/` | Frontend rendering engines (Markdown, code highlight) |
| Hooks (TS) | `src/hooks/` | React custom hooks |
| API (TS) | `src/api/` | Tauri invoke wrappers |
| DSTU (TS) | `src/dstu/` | DSTU frontend API |
| MCP (TS) | `src/mcp/` | MCP client frontend |
| Essays (TS) | `src/essay-grading/` | Essay grading frontend |
| Voice input (TS) | `src/voice-input/` | Voice input frontend |
| Translation (TS) | `src/translation/` | Translation frontend |
| Services (TS) | `src/services/` | Frontend service modules |
| Menu (TS) | `src/menu/` | Menu configuration |
| Config (TS) | `src/config/` | Application configuration |

### Layer 4 — API Handlers & Message Paths

Tauri command handlers, IPC message handlers, event emitters, and frontend-backend bridge code. May depend on Layers 0-3.

| Module | Path | Key Contents |
|--------|------|-------------|
| Chat V2 handlers | `src-tauri/src/chat_v2/handlers/` | 14 handler files (~165 Tauri commands) |
| VFS handlers | `src-tauri/src/vfs/handlers.rs` | VFS command handlers |
| VFS todo handlers | `src-tauri/src/vfs/todo_handlers.rs` | Todo/pomodoro command handlers |
| VFS ref handlers | `src-tauri/src/vfs/ref_handlers.rs` | Reference command handlers |
| VFS index handlers | `src-tauri/src/vfs/index_handlers.rs` | Index command handlers |
| DSTU handlers | `src-tauri/src/dstu/handlers.rs` | DSTU main command handlers |
| DSTU folder handlers | `src-tauri/src/dstu/folder_handlers.rs` | DSTU folder command handlers |
| DSTU trash handlers | `src-tauri/src/dstu/trash_handlers.rs` | DSTU trash command handlers |
| DSTU export | `src-tauri/src/dstu/export/` | DSTU export handlers |
| Memory handlers | `src-tauri/src/memory/handlers.rs` | Memory command handlers |
| Essay grading handlers | `src-tauri/src/essay_grading/mod.rs` | Essay grading commands (20) |
| Review plan service | `src-tauri/src/review_plan_service.rs` | Review plan commands (17) |
| Data governance | `src-tauri/src/data_governance/commands*.rs` | Data governance commands |
| LLM usage | `src-tauri/src/llm_usage/` | LLM usage tracking handlers |
| CMD modules | `src-tauri/src/cmd/` | Split command files (notes, web_search, ocr, mcp, anki_connect, anki_cards, enhanced_anki, textbooks, translation) |
| Legacy commands | `src-tauri/src/commands.rs` | Legacy file (137 commands, AppError) |
| Debug commands | `src-tauri/src/debug_commands.rs` | Debug/inspection commands |
| Backup config | `src-tauri/src/backup_config.rs` | Backup configuration commands |
| Feature flags | `src-tauri/src/feature_flags.rs` | Feature flag commands |
| Error recovery | `src-tauri/src/error_recovery.rs` | Error recovery commands |
| Workflow error handler | `src-tauri/src/workflow_error_handler.rs` | Workflow error handling |
| QBank grading | `src-tauri/src/qbank_grading/` | Question bank grading commands |
| Question sync service | `src-tauri/src/question_sync_service.rs` | Question sync commands |
| Config recovery | `src-tauri/src/config_recovery.rs` | Config recovery commands |
| Secure store | `src-tauri/src/secure_store.rs` | Secure store commands |
| Cloud storage | `src-tauri/src/cloud_storage/sync_manager.rs` | Cloud sync commands |
| Data space | `src-tauri/src/data_space.rs` | Data space commands |
| Debug logger | `src-tauri/src/debug_logger.rs` | Debug logging commands |
| Debug log service | `src-tauri/src/debug_log_service.rs` | Debug log service commands |
| Frontend features | `src/features/` | 14 feature modules with UI + handlers |
| Frontend stores | `src/stores/` | Zustand stores bridging to Tauri invoke |
| Frontend components | `src/components/` | UI components (ui, shared, layout, icons, crepe, anki, previews, practice) |
| Frontend API | `src/api/` | Tauri invoke wrapper functions |
| Frontend context | `src/contexts/` | React context providers |

### Layer 5 — Entry Points & Wiring

Application entry points, module registration, build configuration, and dependency injection setup. May depend on all lower layers (0-4).

| Module | Path | Key Contents |
|--------|------|-------------|
| Rust main | `src-tauri/src/main.rs` | Tauri application entry point |
| Rust lib | `src-tauri/src/lib.rs` | Module registration, plugin setup, command registration |
| Tauri config | `src-tauri/tauri.conf.json` | Tauri configuration |
| Cargo manifest | `src-tauri/Cargo.toml` | Rust dependency declaration |
| React entry | `src/main.tsx` | Frontend React entry point |
| App root | `src/App.tsx` | Root React component |
| Vite config | `vite.config.ts` | Vite build configuration |
| Tauri build script | `src-tauri/build.rs` | Build-time code generation |
| Menu | `src-tauri/src/menu.rs` | Application menu definition |
| Start-up cleanup | `src-tauri/src/startup_cleanup.rs` | Start-up maintenance tasks |
| Background tasks | `src-tauri/src/background_tasks.rs` | Background task registration |
| Injection budget | `src-tauri/src/injection_budget.rs` | Resource budget management |
| Metrics server | `src-tauri/src/metrics_server.rs` | Metrics endpoint |
| ANR watchdog | `src-tauri/src/anr_watchdog.rs` | Application Not Responding watchdog |
| Crash logger | `src-tauri/src/crash_logger.rs` | Crash reporting |
| Error details | `src-tauri/src/error_details.rs` | Detailed error formatting |
| Style entry | `src/styles/` | CSS entry points, App.css (12K lines) |
| Lazy components | `src/lazyComponents.tsx` | Lazy-loaded component registry |

---

## 2. Module Dependency Constraints

### 2.1 Allowed Dependency Direction

```
Layer 5 (Entry Points)
    |
    v
Layer 4 (API Handlers)
    |
    v
Layer 3 (Business Logic)
    |
    v
Layer 2 (Data Models)
    |
    v
Layer 1 (Utilities)
    |
    v
Layer 0 (Base Types)
```

**Rule**: A module at layer N may only depend on modules at layer N-1 or lower. Cross-layer dependencies that skip a layer are allowed (e.g., Layer 4 may depend on Layer 2 directly), but **upward dependencies (layer N depending on layer N+1) are forbidden**.

### 2.2 Currently-Violating Module Pairs

The following known violations exist and must be addressed during refactoring:

| Source Module | Layer | Target Module | Layer | Violation Type | Notes |
|---------------|-------|---------------|-------|----------------|-------|
| `chat_v2/handlers/` | 4 | `chat_v2/pipeline/` (internal) | 3 | Allowed (4->3) | OK — handlers call pipeline |
| `memory/handlers.rs` | 4 | `memory/service.rs` | 3 | Allowed (4->3) | OK — handlers call service |
| `commands.rs` | 4 | `models.rs` (AppError) | 0 | Allowed (4->0) | OK — uses AppError |
| `chat_v2/handlers/load_session.rs` | 4 | `commands.rs` | 4 | **Lateral (4->4)** | Circular dependency risk: handlers importing from legacy commands |
| `cmd/notes.rs` | 4 | `models.rs` | 0 | Allowed (4->0) | OK |
| `chat_v2/pipeline/` | 3 | `chat_v2/types.rs` | 2 | Allowed (3->2) | OK |
| `vfs/handlers.rs` | 4 | `commands.rs` | 4 | **Lateral (4->4)** | Shared imports across handler files |
| `dstu/handlers.rs` | 4 | `dstu/folder_handlers.rs` | 4 | **Lateral (4->4)** | Cross-file handler dependencies |
| `dstu/handlers.rs` | 4 | `commands.rs` | 4 | **Lateral (4->4)** | Legacy import leakage |
| `chat_v2/` | 3 | `vfs/` | 2-3 | Cross-module (3->3) | Chat depends on VFS — acceptable architectural dependency |
| `essay_grading/` | 3 | `models.rs` (AppError) | 0 | Allowed (3->0) | Has `From<AppError>` impl |
| Various handlers | 4 | `models.rs` | 0 | Allowed (4->0) | OK |

### 2.3 Resolution Strategy for Violations

| Violation | Severity | Resolution |
|-----------|----------|------------|
| Lateral 4->4 imports across handler files | **High** | Extract shared logic into Layer 3 service modules; handlers should only call services |
| `handlers/load_session.rs` → `commands.rs` | **High** | Move shared session loading logic into `chat_v2/` service layer |
| `dstu/handlers.rs` → `commands.rs` | **High** | Extract DSTU-related helpers from `commands.rs` into DSTU module |
| Circular dependencies | **Critical** | Break cycle by introducing a shared types crate or extracting the circular path into a new module at a lower layer |
| `chat_v2/` → `vfs/` | Low (intentional) | Keep as-is; document as an allowed cross-module dependency |

---

## 3. API Unification Standards

### 3.1 Error Handling

**All Tauri commands must return `ModuleResult<T>` using the module's own error type.** The migration from `Result<T, String>` is tracked as Phase 1 of the refactoring.

**Status**: 378/697 Tauri commands completed (54.2%).

**Error types in use**:

| Error Type | Module | Status |
|------------|--------|--------|
| `ChatV2Error` | `chat_v2/` | Active — 39 commands migrated |
| `VfsError` | `vfs/` | Active — 25 commands migrated |
| `DstuError` | `dstu/` | Active — 19 commands migrated |
| `MemoryError` | `memory/` | Active — 27 commands migrated |
| `EssayGradingError` | `essay_grading/` | Active — 20 commands migrated |
| `ReviewPlanError` | `review_plan_service.rs` | Active — 17 commands migrated |
| `DataGovernanceError` | `data_governance/` | Active — new |
| `AppError` | `models.rs` | Legacy — 137+ commands |

**From implementation matrix** (known conversions):

| Source | Target | Notes |
|--------|--------|-------|
| `VfsError` | `MemoryError`, `EssayGradingError`, `DstuError` | Cross-module conversion |
| `AppError` | `EssayGradingError` | Legacy bridge |
| `anyhow::Error` | `ReviewPlanError`, `ChatV2Error` | Third-party error wrapping |
| `rusqlite::Error` | `ChatV2Error`, `DstuError` | Database error conversion |
| `JoinError` | `DstuError` | Threading error conversion |
| `serde_json::Error` | `ChatV2Error`, `DstuError` | Serialization error conversion |

**Conversion pattern**:

```rust
// Correct — use ? with From trait
fn my_command(db: tauri::State<Database>) -> ChatV2Result<Session> {
    let conn = db.get_conn_safe()?;  // ChatV2Error via From
    Ok(ChatV2Repo::find_session(&conn, id)?)  // ChatV2Error via From
}

// Incorrect — do not use .map_err(|e| e.to_string())
fn my_command(db: tauri::State<Database>) -> Result<Session, String> {  // Wrong return type
    let conn = db.get_conn_safe().map_err(|e| e.to_string())?;  // String conversion
    ChatV2Repo::find_session(&conn, id).map_err(|e| e.to_string())  // String conversion
}
```

### 3.2 Naming Conventions

| Scope | Convention | Example |
|-------|-----------|---------|
| Tauri commands (Rust) | `{module}_{action}` snake_case | `memory_search`, `vfs_create_file` |
| Frontend invoke calls | camelCase | `memorySearch`, `vfsCreateFile` |
| Events (emit/listen) | `{module}_{event_type}` | `chat_message_received`, `vfs_file_updated` |
| Modules (Rust) | snake_case | `chat_v2`, `essay_grading` |
| Directories (frontend) | kebab-case | `learning-hub`, `essay-grading` |
| Error enum variants | PascalCase | `ChatV2Error::SessionNotFound` |
| Type aliases | `{Module}Result<T>` | `ChatV2Result<T>`, `VfsResult<T>` |
| Type aliases (full) | `std::result::Result<T, E>` | Avoid anyhow shadowing |

### 3.3 Message Path Format

All IPC event names follow the format `{module}_{event_type}` in snake_case:

```
chat_message_received        Chat module, message received
vfs_file_updated             VFS module, file updated
memory_sync_complete         Memory module, sync complete
dstu_resource_changed        DSTU module, resource changed
essay_grading_progress       Essay grading, progress update
data_governance_sync_done    Data governance, sync complete
```

### 3.4 Dependency Injection

- **Singletons** are provided via `tauri::State` (Rust) and accessed through `tauri::Manager`
- **Database connections** use `get_conn_safe()` returning `ChatV2Result<impl Deref<Target = Connection>>`
- **Service instances** are registered at startup in `src-tauri/src/lib.rs` using `app.manage()`
- **Frontend** accesses backend through `@tauri-apps/api/core` `invoke()` calls and event listeners

---

## 4. Validation Strategy

### 4.1 Static Analysis Phase (NO COMPILATION)

Before any compilation is attempted, all refactoring changes must pass the following static checks:

| Check | Tool | Scope |
|-------|------|-------|
| TypeScript type checking | `tsc --noEmit` | All `src/` files |
| Rust lint | `cargo check` | All `src-tauri/src/` files |
| Dependency graph analysis | `cargo metadata` + custom script | Module import graph, layer violations |
| Unused code detection | `rustc` dead_code warnings, `ts-prune` | Both Rust and TS |
| Import cycle detection | Manual review + cargo-deny | Module import graph |

**Gate**: All static checks must pass before proceeding to compilation.

### 4.2 Chunked Verification

Refactoring is verified per layer, bottom-up:

1. **Layer 0**: Verify all type definitions, error enums, and constants are self-consistent
2. **Layer 1**: Verify utilities and database layer have no upward dependencies
3. **Layer 2**: Verify state models and type definitions depend only on Layers 0-1
4. **Layer 3**: Verify business logic depends only on Layers 0-2
5. **Layer 4**: Verify handlers depend only on Layers 0-3, with no lateral dependencies between handler modules
6. **Layer 5**: Verify entry points wire everything correctly

Each layer verification must pass before work on the next layer begins.

### 4.3 Compilation Gate

Full compilation (`cargo build` / `npm run build`) is only attempted after:

- All static analysis checks pass
- All layer verifications pass
- Human sign-off is obtained from the lead maintainer

### 4.4 CI Integration

Planned CI pipeline stages (to be added in a future phase):

1. **Lint**: `cargo clippy`, `eslint`, `tsc --noEmit`
2. **Dependency**: Custom script checking layer constraints
3. **Build**: `cargo build`, `vite build`
4. **Test**: `cargo test`, `vitest`
5. **Package**: Tauri bundle

---

## 5. Remaining Work Inventory

### 5.1 API Error Unification (Phase 1)

**Overall progress**: 378/697 Tauri commands (54.2%)

**Remaining modules to migrate from `Result<T, String>`** (319 commands):

| Module | File(s) | Est. Commands | Priority | Notes |
|--------|---------|--------------|----------|-------|
| Chat V2 block_actions | `chat_v2/handlers/block_actions.rs` | 6 | P0 | ChatV2Error ready |
| Chat V2 migration | `chat_v2/handlers/migration.rs` | 2 | P0 | ChatV2Error ready |
| Chat V2 send_message | `chat_v2/handlers/send_message.rs` | 2 | P0 | ChatV2Error ready |
| Chat V2 workspace | `chat_v2/handlers/workspace_handlers.rs` | ~10 | P0 | ChatV2Error ready |
| Chat V2 resource | `chat_v2/handlers/resource_handlers.rs` | ~6 | P0 | Deprecated module? |
| DSTU handlers | `dstu/handlers.rs` | ~30 | P1 | DstuError ready |
| Data governance | `data_governance/commands*.rs` | ~30 | P1 | DataGovernanceError ready |
| LLM usage | `llm_usage/` | ~10 | P1 | Needs error type |
| QBank grading | `qbank_grading/` | ~10 | P2 | Needs error type |
| Cloud storage | `cloud_storage/` | ~10 | P2 | Needs error type |
| CMD modules | `cmd/notes.rs`, `cmd/enhanced_anki.rs` | ~40 | P2 | AppError available |
| Legacy commands | `commands.rs` | 137 | P3 | AppError, low priority |
| Other internal services | Various `.rs` files (spaced_repetition, config_recovery, etc.) | ~36 | P3 | Internal methods, not Tauri commands |

### 5.2 CSS Architecture Migration

| Item | Current State | Target State | Status |
|------|--------------|-------------|--------|
| Main CSS file | `src/styles/App.css` (~12,000 lines) | Split into component-scoped files | Plan exists, not executed |
| CSS paradigm | Global CSS | Tailwind v4 utility classes | Not started |
| Scattered CSS | 60+ scattered `.css` files | Consolidation + Tailwind migration | Not started |
| Phase | — | Gradual migration across features | Not started |

### 5.3 Frontend Architecture Cleanup

| Item | Count | Status |
|------|-------|--------|
| TypeScript errors | 17 | Phase 8 of v1.2 roadmap |
| Bundle size issues | Unknown | Phase 9: bundle analysis needed |
| Lazy loading gaps | Heavy features not deferred | Phase 10: lazy loading |
| `manualChunks` config | Not optimized | Phase 11: Vite chunk reconfiguration |
| Performance baseline | Not established | Phase 12: baseline validation |

### 5.4 Circular Dependency Resolution

| Dependency Cycle | Location | Severity | Resolution |
|-----------------|----------|----------|------------|
| `handlers/` ↔ `commands.rs` | Chat V2 + DSTU + VFS handlers | **High** | Extract shared types to Layer 2 |
| Module-level cross-imports | Various handler files | **Medium** | Abstract service layer between handlers |
| Tauri State → Handler → Service | App bootstrap | Low (intentional) | Keep as framework pattern |

### 5.5 Input Struct Encapsulation (Phase 2)

**Status**: Not started.

Tauri commands with 3+ parameters should bundle them into input structs:

```rust
// Before
#[tauri::command]
async fn search_memories(
    app: tauri::AppHandle,
    query: String,
    limit: Option<usize>,
    threshold: Option<f64>,
    category: Option<String>,
) -> MemoryResult<Vec<Memory>> { ... }

// After
#[derive(Deserialize)]
struct SearchMemoriesInput {
    query: String,
    limit: Option<usize>,
    threshold: Option<f64>,
    category: Option<String>,
}

#[tauri::command]
async fn search_memories(
    app: tauri::AppHandle,
    input: SearchMemoriesInput,
) -> MemoryResult<Vec<Memory>> { ... }
```

### 5.6 State Redundancy Elimination (Phase 3)

**Status**: Not started.

Audit Zustand stores and Rust `State` structs for duplicated or stale data. Common patterns to eliminate:

- Frontend stores caching data that is already in the backend database
- Multiple stores holding overlapping slices of the same entity
- Race conditions between store updates and backend events

### 5.7 Module Splitting & Dead Code Removal (Phase 4)

**Status**: Not started.

| Item | Action |
|------|--------|
| `commands.rs` (137 commands) | Split into per-module handler files; retire legacy file |
| `cmd/` modules | Integrate into proper module handlers or keep as-is with AppError |
| Deprecated chat v2 resource handlers | Verify and remove if unused |
| Unused error variants | Audit all error enums for unused variants |
| Dead CSS | Audit with PurgeCSS or similar tool |
| Unused TS components | Audit with ts-prune |

### 5.8 v1.2 Roadmap (Phases 8-12)

The following phases are on the active roadmap and are tracked separately from the main refactoring:

| Phase | Description | Target |
|-------|-------------|--------|
| Phase 8 | Fix 17 pre-existing TypeScript errors | Before v1.2 |
| Phase 9 | Bundle analysis (vite-bundle-analyzer) | Before v1.2 |
| Phase 10 | Lazy loading heavy features | Before v1.2 |
| Phase 11 | manualChunks reconfiguration | Before v1.2 |
| Phase 12 | Performance baseline validation | v1.2 release |

---

## 6. Existing Diagnostic Assets

### 6.1 Diagnostic Reports (24 reports)

Location: `.planning/exploration/reports/`

```text
Coverage:
  Frontend: root config, app entry, types/shared, stores, API services,
            UI components, hooks/engines, chat-v2, learning-hub,
            notes (Milkdown), mindmap (React Flow), practice/question-bank,
            anki-template (Mustache), pdf/docx/translation/essay/pomodoro/
            todo/sandbox/voice merged, settings, system-features,
            mcp-client, dstu, study-ui
  Backend:  backend-entry + cmd + database, chat-v2-backend,
            llm-manager (9 providers), vfs,
            backend-merged (tools/search/memory/datagov/cloud)
  Cross-cutting: tests, build, CI, i18n, styles
  Summary:   cumulative-issues.md (55+ issues)
  Scans:     supplement scans, root scan, study-ui scan
```

### 6.2 API Refactor Reports (31 reports)

Location: `.planning/exploration/dependency-db/reports/api-refactor/`

```text
Per-module reports:
  chat_v2, vfs, dstu, memory, essay_grading, review_plan_service,
  data_governance, llm_usage, qbank_grading, question_sync_service,
  cloud_storage, cmd/* (9 files), commands, translation, tts,
  pdfium_utils, anki_connect_service, config_recovery, data_space,
  debug_commands, debug_logger, secure_store, backup_config
Supporting: INDEX.md, IMPLEMENTATION-PLAN.md
```

---

## 7. Quick Reference

### 7.1 Module-to-Layer Mapping (Backend)

| Module | Layer | Error Type | Has Handlers? | Status |
|--------|-------|-----------|--------------|--------|
| `models.rs` | 0 | `AppError` | No | Stable |
| `database/` | 1 | — | No | Stable |
| `utils/` | 1 | — | No | Stable |
| `vfs/types.rs` | 2 | `VfsError` | No | Stable |
| `chat_v2/types.rs` | 2 | `ChatV2Error` | No | Stable |
| `dstu/types.rs` | 2 | `DstuError` | No | Stable |
| `chat_v2/pipeline/` | 3 | `ChatV2Error` | No | Stable |
| `memory/service.rs` | 3 | `MemoryError` | No | Stable |
| `essay_grading/pipeline.rs` | 3 | `EssayGradingError` | No | Stable |
| `chat_v2/handlers/` | 4 | `ChatV2Error` | Yes (14 files) | Migrating |
| `vfs/todo_handlers.rs` | 4 | `VfsError` | Yes | Done |
| `memory/handlers.rs` | 4 | `MemoryError` | Yes | Done |
| `essay_grading/mod.rs` | 4 | `EssayGradingError` | Yes | Done |
| `commands.rs` | 4 | `AppError` | Yes (137) | Legacy |
| `main.rs` | 5 | — | No | Stable |
| `lib.rs` | 5 | — | No | Stable |

### 7.2 Migration Quick Checklist

When converting a handler from `Result<T, String>` to `ModuleResult<T>`:

- [ ] Change return type from `Result<T, String>` to `ModuleResult<T>`
- [ ] Remove all `.map_err(|e| e.to_string())` calls
- [ ] Replace `.map_err(|e| format!(...))` with `.map_err(|_| ModuleError::Variant(...))` or add a `From` impl
- [ ] Remove redundant `.into()` on error values being returned directly
- [ ] In `spawn_blocking` closures, annotate the closure return type as `-> ModuleResult<T>`
- [ ] Verify all `From` impls exist for error types used inside the handler
- [ ] Run `cargo check` on the module

### 7.3 Layer Compliance Checklist

When adding or modifying a module:

- [ ] Identify the correct layer for the module
- [ ] Verify all imports only reference modules at the same or lower layer
- [ ] No imports from a higher layer
- [ ] No circular dependencies with sibling modules
- [ ] If a cross-layer dependency is required, document it in Section 2.2
