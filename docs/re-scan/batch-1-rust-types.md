# Batch 1: Rust Error Type Scan Report

## 1. Error Enum Table

| Error Enum | File | Variants | Type Alias | Uses `thiserror` | Uses `Serialize` |
|---|---|---|---|---|---|
| `ChatV2Error` | `src-tauri/src/chat_v2/error.rs` | 19 | `ChatV2Result<T>` | Yes | Yes |
| `VfsError` | `src-tauri/src/vfs/error.rs` | 20 | `VfsResult<T>` | No (manual Display) | No |
| `DstuError` | `src-tauri/src/dstu/error.rs` | 11 | `DstuResult<T>` | Yes | Yes |
| `MemoryError` | `src-tauri/src/memory/error.rs` | 4 | `MemoryResult<T>` | No (manual Display) | Yes |
| `EssayGradingError` | `src-tauri/src/essay_grading/error.rs` | 4 | `EssayGradingResult<T>` | No (manual Display) | Yes |
| `DataGovernanceError` | `src-tauri/src/data_governance/error.rs` | 9 | `DataGovernanceResult<T>` | Yes | Yes |
| `DataGovernanceError` | `src-tauri/src/data_governance/mod.rs` | 5 | `DataGovernanceResult<T>` | Yes | No |
| `AppError` + `AppErrorType` | `src-tauri/src/models.rs` | 8+1 (struct+enum) | None | No (manual Display) | Yes |
| `ReviewPlanError` | **DOES NOT EXIST** | 0 | None | N/A | N/A |
| `LlmUsageError` | `src-tauri/src/llm_usage/database.rs` | 5 | `LlmUsageResult<T>` | Yes | No (noted) |
| `McpError` | `src-tauri/src/mcp/client.rs` | 7 | `McpResult<T>` | Yes | No (noted) |

## 2. From Conversion Chains

### ChatV2Error (chat_v2/error.rs)
```
rusqlite::Error       → ChatV2Error::Database(String)
serde_json::Error     → ChatV2Error::Serialization(String)
anyhow::Error         → ChatV2Error::Other(String)
ChatV2Error           → String  (JSON-encoded with code + message)
```

### VfsError (vfs/error.rs)
```
std::io::Error        → VfsError::Io(String)
serde_json::Error     → VfsError::Serialization(String)
rusqlite::Error       → VfsError::Database(String)
VfsError              → String
```

### DstuError (dstu/error.rs)
```
DstuError             → String
std::io::Error        → DstuError::IoError(String)
serde_json::Error     → DstuError::SerializationError(String)
rusqlite::Error       → DstuError::DatabaseError(String)
```

### MemoryError (memory/error.rs)
```
crate::vfs::error::VfsError  → MemoryError::Database(String)
```
NOTE: No `From<String>`, no `From<rusqlite::Error>`, no `From<serde_json::Error>`.

### EssayGradingError (essay_grading/error.rs)
```
crate::models::AppError  → EssayGradingError (maps Database/Validation/NotFound/fallback)
String                   → EssayGradingError::Internal(String)
crate::vfs::error::VfsError → EssayGradingError::Database(String)
```

### DataGovernanceError (data_governance/error.rs)
```
String                   → DataGovernanceError::Internal(String)
rusqlite::Error          → DataGovernanceError::Database(String)
serde_json::Error        → DataGovernanceError::Serialization(String)
std::io::Error           → DataGovernanceError::Io(String)
```

### DataGovernanceError (data_governance/mod.rs, line 110)
```
migration::MigrationError       → (via #[from])
schema_registry::SchemaRegistryError → (via #[from])
```
No `From<String>`, no `From<rusqlite::Error>`, no `From<serde_json::Error>`, no `From<std::io::Error>`.

### AppError (models.rs)
```
String                   → AppError (calls .validation())
&str                     → AppError (calls .validation())
zip::result::ZipError    → AppError::file_system(...)
anyhow::Error            → AppError::unknown(...)
serde_json::Error        → AppError::validation(...)
std::io::Error           → AppError::file_system(...)
AcquireError             → AppError::new(Unknown, ...)
rusqlite::Error          → AppError::database(...)
```

### Cross-module VfsError conversion targets:
```
VfsError  →  MemoryError       (via From<VfsError> in memory/error.rs)
VfsError  →  EssayGradingError (via From<VfsError> in essay_grading/error.rs)
```

## 3. Naming Conflicts Found

### CRITICAL: Two `DataGovernanceError` definitions

| Location | Variants |
|---|---|
| `src-tauri/src/data_governance/error.rs` (line 7) | `Internal`, `Database`, `NotFound`, `InvalidArgument`, `Serialization`, `Io`, `Backup`, `Restore`, `Sync` |
| `src-tauri/src/data_governance/mod.rs` (line 110) | `Migration(#[from] MigrationError)`, `SchemaRegistry(#[from] SchemaRegistryError)`, `Backup`, `Sync`, `NotImplemented` |

Both define their own `pub enum DataGovernanceError` and `pub type DataGovernanceResult<T>`.

- The `mod.rs` version is at path `crate::data_governance::DataGovernanceError`.
- The `error.rs` version is at path `crate::data_governance::error::DataGovernanceError`.
- They have **different** variant sets and **incompatible** `From` implementations.
- The `mod.rs` version uses `#[from]` derive for `MigrationError` and `SchemaRegistryError`.
- The `error.rs` version has `From<String>`, `From<rusqlite::Error>`, `From<serde_json::Error>`, `From<std::io::Error>`.
- **Neither version has `From` impls for the other**, so code that uses one cannot be directly adapted to the other.

**Impact**: Importing `use crate::data_governance::DataGovernanceError` gives the `mod.rs` 5-variant version. Importing `use crate::data_governance::error::DataGovernanceError` gives the `error.rs` 9-variant version. This is likely a **bug/oversight from an incomplete refactoring**.

### WARNING: `ChatV2Error` defines `VariantCannotRetry(String, String)` -- inconsistent display prefix

Most error messages use one pattern, but `VariantCannotRetry` uses `"Cannot retry variant: {0}, current status: {1}"` while `VariantCannotActivateFailed` uses `"Cannot activate failed variant: {0}"`. Inconsistency: one uses "variant" after the verb, one doesn't.

### WARNING: VfsError uses uppercase `SCREAMING_SNAKE_CASE` in Display for folder errors, but normal sentence case for other errors

Folder-related error displays use prefixes like `"FOLDER_NOT_FOUND: {}"`, `"FOLDER_ALREADY_EXISTS: {}"`, etc. while other variants like `Database`, `NotFound`, `Io` use sentence case like `"Database error: {}"`, `"{} not found: {}"`, `"IO error: {}"`.

## 4. Module Declaration Inconsistencies

### review_plan_service.rs -- CLAUDE.md tracking error

- `CLAUDE.md` claims `ReviewPlanError` exists and 17 commands are migrated. **This is false.**
- `src-tauri/src/review_plan_service.rs` has **zero** code referencing `ReviewPlanError`.
- All 17 Tauri commands in that file return `Result<T, String>` with `.map_err(|e| e.to_string())`.
- All service methods use `anyhow::Result<T>` (external `anyhow::Result`).
- `ReviewPlanError` is not defined anywhere in the entire codebase.
- The module is declared as `pub mod review_plan_service;` in lib.rs -- it is not its own folder, just a single file.

### essay_grading/mod.rs -- commands still use AppError, not EssayGradingError

- The `EssayGradingError` type exists in `essay_grading/error.rs` with 4 variants and `EssayGradingResult<T>` type alias.
- However, all Tauri commands in `essay_grading/mod.rs` (20+ commands) return `Result<T, AppError>` -- **not** `Result<T, EssayGradingError>`.
- This means `EssayGradingError` is **defined but unused** by the actual command handlers.

### data_governance/mod.rs -- dual error definition

As described above in Section 3, `data_governance/mod.rs` defines its own `DataGovernanceError` inline (line 110-125) while also declaring `pub mod error;` (line 45) which pulls in the conflicting `DataGovernanceError` from `data_governance/error.rs`.

### Additional error types not tracked in CLAUDE.md

Two error types exist in the codebase but are **not listed** in the CLAUDE.md progress tracking:

1. **`LlmUsageError`** (5 variants) in `src-tauri/src/llm_usage/database.rs` (line 32)
   - Type alias: `LlmUsageResult<T>`
   - Uses `thiserror` + `#[from]` for `rusqlite::Error` and `std::io::Error`
   - Not listed in any module's mod.rs as a submodule -- it's a standalone database error

2. **`McpError`** (7 variants) in `src-tauri/src/mcp/client.rs` (line 25)
   - Type alias: `McpResult<T>`
   - Uses `thiserror` + `#[from]` for `serde_json::Error`
   - MCP (Model Context Protocol) module

## 5. Summary of Issues Found

| Severity | Issue | Location |
|---|---|---|
| **HIGH** | Two conflicting `DataGovernanceError` definitions | `data_governance/error.rs` vs `data_governance/mod.rs` |
| **HIGH** | `ReviewPlanError` claimed as completed in CLAUDE.md but does not exist | `review_plan_service.rs` still uses `Result<T, String>` |
| **MEDIUM** | `EssayGradingError` defined but unused by commands | `essay_grading/error.rs` vs `essay_grading/mod.rs` (uses `AppError`) |
| **LOW** | `LlmUsageError` and `McpError` not tracked in API refactor plan | `llm_usage/database.rs`, `mcp/client.rs` |
| **LOW** | No `AppResult<T>` type alias for `AppError` | `models.rs` |
| **LOW** | Inconsistent Display formatting in `VfsError` (sentence vs SCREAMING_SNAKE_CASE) | `vfs/error.rs` |
