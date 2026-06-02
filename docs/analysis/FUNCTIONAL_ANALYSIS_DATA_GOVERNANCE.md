# Functional Group Analysis: data_governance

## 1. Group Overview

**Description:** Data Governance (backup, sync, audit, migration)
**Root path:** `src-tauri/src/data_governance/`
**Feature flag:** `data_governance` (enabled by default, adds `refinery` crate)

### Architecture
```
data_governance/
├── mod.rs                 (262 LOC)  — Module declaration, DataGovernanceError, re-exports
├── plugin.rs              (16 LOC)   — Empty Tauri plugin placeholder
├── init.rs                (429 LOC)  — Initialization orchestration
├── schema_registry.rs     (878 LOC)  — Database version state tracking
├── dto/mod.rs             (126 LOC)  — DTO re-exports
│
├── audit/mod.rs           (838 LOC)  — Audit logging (AuditDatabase, AuditRepository)
│
├── backup/
│   ├── mod.rs            (3822 LOC)  — BackupManager, BackupManifest, BackupTier
│   ├── assets.rs         (1474 LOC)  — Asset file backup/restore
│   └── zip_export.rs     (862 LOC)   — ZIP export/import
│
├── sync/
│   ├── mod.rs            (7463 LOC)  — SyncManager, sync orchestration
│   ├── classification.rs (212 LOC)   — Sync data classification
│   ├── conflict_resolver.rs (502 LOC) — Record-level conflict detection
│   ├── emitter.rs        (379 LOC)   — Progress event emitter
│   ├── field_merge.rs    (890 LOC)   — Field-level merge strategies
│   ├── hlc.rs            (389 LOC)   — Hybrid Logical Clocks
│   ├── progress.rs       (545 LOC)   — Sync progress tracking
│   └── tombstone.rs      (425 LOC)   — Tombstone deletion handling
│
├── migration/
│   ├── mod.rs            (330 LOC)   — Migration framework, ALL_MIGRATION_SETS
│   ├── coordinator.rs    (4347 LOC)  — Multi-database migration coordinator
│   ├── definitions.rs    (137 LOC)   — MigrationDef struct
│   ├── verifier.rs       (194 LOC)   — Post-migration verification
│   ├── script_checker.rs (789 LOC)   — Static migration script analysis
│   ├── vfs.rs            (757 LOC)   — VFS database migrations
│   ├── chat_v2.rs        (504 LOC)   — Chat V2 database migrations
│   ├── mistakes.rs       (377 LOC)   — Mistakes database migrations
│   └── llm_usage.rs      (256 LOC)   — LLM usage database migrations
│
├── commands.rs           (1424 LOC)  — Core commands (schema, audit, health, migration diagnostic)
├── commands_backup.rs    (2272 LOC)  — Backup Tauri commands
├── commands_asset.rs     (361 LOC)   — Asset backup Tauri commands
├── commands_restore.rs   (1365 LOC)  — Restore Tauri commands
├── commands_sync.rs      (3149 LOC)  — Sync Tauri commands
├── commands_types.rs     (171 LOC)   — Shared response types
├── commands_zip.rs       (1471 LOC)  — ZIP export/import commands
│
├── tests.rs              (1089 LOC)  — Integration tests
├── migration_tests.rs    (1801 LOC)  — Migration-specific integration tests
└── critical_audit_tests.rs (1659 LOC) — Audit-focused tests
```

## 2. Lines of Code

| Component | LOC | Percentage |
|-----------|-----|------------|
| `sync/` | 10,433 | 24.9% |
| `migration/` | 7,591 | 18.1% |
| `backup/` | 6,158 | 14.7% |
| Command files (7 files) | 10,213 | 24.3% |
| Test files (3 files) | 4,549 | 10.8% |
| Root + support | 3,021 | 7.2% |
| **Total** | **41,965** | **100%** |

The module is **41,965 LOC** across 36 source files.

## 3. External Dependencies

### 3.1 Feature-gate dependencies (data_governance feature)
| Dependency | Classification | Reason |
|-----------|---------------|--------|
| `refinery 0.9` | ESSENTIAL | Migration framework that all database versioning depends on |

### 3.2 Direct runtime dependencies (in Cargo.toml, not optional)
| Dependency | Classification | Reason |
|-----------|---------------|--------|
| `rusqlite 0.29` (with `backup` feature) | ESSENTIAL | Core database operations; SQLite Backup API used for atomic backup/restore |
| `serde 1.0 + serde_json 1.0` | ESSENTIAL | Serialization for backup manifests, audit logs, Tauri command responses |
| `sha2 0.10 + hex 0.4` | REPLACEABLE | File integrity checksums; could use blake3 (already in comment-dep) |
| `uuid 1.7` | REPLACEABLE | Backup ID generation; nanoid/ulid already in project deps |
| `chrono 0.4` (with `serde`) | ESSENTIAL | Timestamps throughout all data structures |
| `walkdir 2` | ESSENTIAL | Directory tree traversal for asset backup |
| `zip 0.6` | ESSENTIAL | ZIP export/import commands |
| `tokio 1.35` | ESSENTIAL | Async runtime for backup/sync Tauri commands |
| `tracing 0.1` | ESSENTIAL | Structured logging |
| `thiserror 1.0` | ESSENTIAL | Error derives for DataGovernanceError and sub-errors |
| `anyhow 1.0` | OPTIONAL | Used only for error conversion (From<anyhow> impl); could be removed |
| `libc 0.2` | OPTIONAL | Disk space check via statvfs; only for backup disk check |
| `tempfile 3.0` | OPTIONAL | Test-only dependency |

### 3.3 Indirect-but-used dependencies
| Dependency | Classification | Reason |
|-----------|---------------|--------|
| `aws-sdk-s3 1.65` (gated by `cloud_storage_s3`) | OPTIONAL | S3 cloud storage in sync module |
| `aws-config 1.5` (gated by `cloud_storage_s3`) | OPTIONAL | S3 cloud storage configuration |
| `tauri 2` | ESSENTIAL | AppHandle, State, command attributes, event emitting |
| `dashmap 5.5` | REPLACEABLE | Used in backup_job_manager (outside module) |

### Dependency Summary
- **ESSENTIAL:** 9 (rusqlite, serde, serde_json, chrono, walkdir, zip, tokio, tracing, tauri, refinery)
- **REPLACEABLE:** 3 (sha2/hex, uuid, dashmap)
- **OPTIONAL:** 3 (anyhow, libc, aws-sdk-s3/aws-config)
- **REDUNDANT:** 0

## 4. Internal Coupling Analysis

### 4.1 Cross-submodule imports within data_governance

| Source | Depends on |
|--------|-----------|
| `commands.rs` | audit, backup, migration, schema_registry, commands_backup, commands_restore, commands_types |
| `commands_backup.rs` | audit, backup, schema_registry, sync/classification, sync (SyncManager), commands_restore |
| `commands_asset.rs` | backup (assets), schema_registry |
| `init.rs` | audit, migration, schema_registry |
| `migration/coordinator.rs` | schema_registry, audit |
| `backup/mod.rs` | schema_registry, migration (ALL_MIGRATION_SETS) |
| `backup/assets.rs` | (standalone within backup) |
| `backup/zip_export.rs` | backup (BackupManager) |
| `sync/mod.rs` | sync/* (all sub-modules) |
| `sync/tombstone.rs` | cloud_storage (external) |

### 4.2 External (non-data_governance) module coupling

| External Module | Used by | Nature |
|----------------|---------|--------|
| `backup_common` | commands, commands_backup, commands_restore, commands_sync, commands_zip | **Heavy**: global semaphore, shared constants, helper functions |
| `backup_job_manager` | commands, commands_backup, commands_restore, commands_sync, commands_zip | **Heavy**: job lifecycle management tightly coupled to backup logic |
| `cloud_storage` | commands_sync, sync/tombstone, sync/mod | **Heavy**: sync depends on CloudStorage trait |
| `data_space` | init, backup/mod, commands_backup, commands_restore | **Medium**: active directory path resolution |
| `commands::AppState` | commands | **Medium**: maintenance mode check |
| `utils::text` | commands, commands_backup, commands_zip | **Light**: safe_truncate_chars |
| `models::AppError` | backup_common | **Light** (outside data_governance but coupled to it) |

### 4.3 Who depends on data_governance

| External module | What it imports |
|----------------|-----------------|
| `lib.rs` | **Heavy**: init.rs, all commands, error types, SchemaRegistry, AuditState |
| `backup_job_manager.rs` | DataGovernanceError |

## 5. Duplicated Functionality

1. **`log_and_skip_err`** function is identically defined in:
   - `backup/mod.rs` (lines 50-58): `fn log_and_skip_err<T, E: std::fmt::Display>(result: Result<T, E>) -> Option<T>`
   - `sync/mod.rs` (lines 92-100): identical signature and body
   - `backup_common.rs` has `log_and_skip_entry_err` — slightly different but semantically identical
   - **Verdict:** Should be unified into `backup_common` or a shared utility.

2. **Database path resolution** appears in at least 3 places:
   - `BackupManager::resolve_database_path_in_dir()` in `backup/mod.rs`
   - `MigrationCoordinator::get_database_path()` in `migration/coordinator.rs`
   - `resolve_database_path()` in `commands_backup.rs`
   - **Verdict:** Three implementations of the same mapping (DatabaseId -> file path).

3. **WAL checkpoint + Backup API pattern** duplicated across:
   - `backup_single_database` (backup/mod.rs)
   - `restore_single_database_to_path` (backup/mod.rs)
   - `backup_db_at_path` (backup/mod.rs)
   - `restore_db_at_path` (backup/mod.rs)
   - `backup_audit_db` (backup/mod.rs)
   - `restore_audit_db` (backup/mod.rs)
   - **Verdict:** The WAL+Backup+integrity pattern is repeated ~6 times with minor variations.

4. **Schema version reading** (querying `refinery_schema_history` table) exists in both `schema_registry.rs` and `backup/mod.rs` (`get_schema_version` method).
   - **Verdict:** Minor duplication, but schema_registry should be the single source.

## 6. Architectural Soundness (SOLID)

### Single Responsibility Principle
- **Violated**: `sync/mod.rs` at 7,463 LOC handles sync orchestration, conflict detection, change log export/import, retry logic, and manifest generation.
- **Violated**: `migration/coordinator.rs` at 4,347 LOC handles coordination, path resolution, audit logging, and migration execution.
- **Violated**: `backup/mod.rs` at 3,822 LOC handles backup execution, restore, verification, tiered logic, workspace databackup, crypto key backup, and audit backup.

### Open/Closed Principle
- **Adequate**: New databases can be added by extending `DatabaseId` enum, adding corresponding migration files, and adding to `ALL_MIGRATION_SETS`. The governance boundary is explicit via `DatabaseId` and `database_ids`.

### Liskov Substitution
- **Not heavily applicable**: The error type hierarchy is well-structured with `DataGovernanceError` wrapping sub-errors like `BackupError`, `SyncError`, `MigrationError`.

### Interface Segregation
- **Could be better**: `BackupManager` is a god struct with ~35 public methods handling backup, restore, verification, incremental backup, crypto keys, workspace databases, and audit databases.

### Dependency Inversion
- **Weak**: Direct calls to `crate::data_space::get_data_space_manager()` are sprinkled across commands_backup, backup/mod, init, commands_restore. Sync module directly imports `crate::cloud_storage::CloudStorage` concrete trait instead of accepting it as a generic parameter.
- **Verdict**: Extraction to a separate crate would require inverting these dependencies via traits.

## 7. Can backup/sync be extracted to a separate crate?

**Backup extraction feasibility: moderate**
- Would need to extract: `DatabaseId`, `schema_registry` (or a minimal interface), `BackupTier` definitions
- Would need to abstract: `data_space` path resolution, `backup_common` global limiter, `backup_job_manager` job persistence
- The backup submodule at 6,158 LOC could be extracted but the coupling to job manager (1,300 LOC) and common helpers (920 LOC) means ~2,200 LOC of supporting code would need to move or be abstracted.

**Sync extraction feasibility: difficult**
- Tightly coupled to `cloud_storage` trait (S3/WebDAV implementations)
- Depends on backup module's constants, schema_registry, global limiter
- The 10,433 LOC sync module would need significant refactoring to accept CloudStorage as a generic parameter

## 8. Health Score: **6/10**

### Strengths (+1 each)
- Well-documented with Chinese and English inline docs
- Comprehensive test coverage: 3 dedicated test files (4,549 LOC), unit tests inline
- Feature-gated behind `data_governance` flag
- Proper error types with thiserror derives, From impls, and Serialize
- Explicit governance boundary via `DatabaseId` enum with documented exemptions
- Clean public API via mod.rs re-exports (commands are centrally registered in lib.rs)

### Weaknesses (-1 each)
- `sync/mod.rs` at 7,463 LOC is severely oversized (SRP violation)
- Duplicated `log_and_skip_err` in 3 places
- Heavy coupling to `backup_job_manager` (1,300 LOC outside the module with data_governance-specific logic)
- 6 separate command files is scattered; response types mixed across command files and dto/mod.rs
- Database path resolution duplicated in 3 places
- Direct coupling to `data_space` and `cloud_storage` concrete types (no DIP)
- `BackupManager` god struct with ~35 public methods
- Cross-module dependency on `commands::AppState` for maintenance mode check

### Verdict
The module is structurally sound for a monorepo but would need significant refactoring (dependency inversion, file splitting, deduplication) before extraction to a separate crate. The sync system is the most problematic area due to its size and coupling.

## 9. Key Files

| File | Path | Role |
|------|------|------|
| Module root | `/src-tauri/src/data_governance/mod.rs` | Error types, re-exports |
| Init | `/src-tauri/src/data_governance/init.rs` | Startup orchestration |
| Schema registry | `/src-tauri/src/data_governance/schema_registry.rs` | Database version tracking |
| Audit | `/src-tauri/src/data_governance/audit/mod.rs` | Audit logging |
| Backup | `/src-tauri/src/data_governance/backup/mod.rs` | Backup/restore manager |
| Sync | `/src-tauri/src/data_governance/sync/mod.rs` | Cloud sync manager |
| Coordinator | `/src-tauri/src/data_governance/migration/coordinator.rs` | Migration executor |
| Shared types | `/src-tauri/src/data_governance/commands_types.rs` | Tauri response types |
| Job manager | `/src-tauri/src/backup_job_manager.rs` | Backup job persistence (outside module) |
| Common helpers | `/src-tauri/src/backup_common.rs` | Global semaphore, shared constants (outside module) |
