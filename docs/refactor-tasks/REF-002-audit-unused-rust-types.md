# REF-002: Audit and Prune Unused Rust Types/Enums

## Meta
- **Layer**: Layer 0 -- Base Types & Constants
- **Priority**: P1
- **Est. effort**: M
- **Predecessors**: None
- **Scope**: `src-tauri/src/**/*.rs`
- **Related reports**: `.planning/exploration/dependency-db/reports/api-refactor/INDEX.md`, `docs/refactor-progress-summary.md`

## Problem Description

The Rust backend at `src-tauri/src/` has 2275 lines in `lib.rs` alone with 93 module declarations. Across all modules, there are patterns of dead or vestigial code:

- **Deprecated Tauri commands** still registered in `invoke_handler` (e.g., `resource_*` commands in `resource_handlers.rs` -- 7 commands marked `#[deprecated]`, still exported and registered)
- **Unused structs/enums** in `commands.rs` (5804 lines, 140+ commands) that were superseded by `cmd/` module commands
- **`#[allow(...)]` annotations** suppressing warnings about dead code (e.g., `#![allow(non_snake_case)]` at line 1 of `commands.rs`)
- **`models.rs` (AppError)** contains 74+ error variant constructors -- some may be unused since modules migrated to their own error types (VfsError, DstuError, MemoryError, etc.)
- **Anyhow shadows**: many files use `anyhow::Result` which masks unused error variants

The refactoring progress summary shows that error unification is at 54.2% -- as modules migrated away from `Result<T, String>` to `Result<T, ModuleError>`, code paths in the original error types may have become dead.

## Target State

- Zero compiler warnings for dead code (`#[allow(dead_code)]` removed or justified)
- Deprecated commands fully removed (not just `#[deprecated]` annotated)
- All unused struct fields, enum variants, and functions identified and removed
- 100% of `#[allow(...)]` annotations for dead code eliminated

## Steps

1. Run `cargo clippy --no-deps -W clippy::dead_code` to get baseline dead code warnings
2. Run `cargo +nightly deadlinks` or `cargo udeps` (if available) to find unused dependencies
3. Categorize dead code by module:
   - Unused enum variants in error types (after migration)
   - Unused struct fields
   - Unused functions (especially in `commands.rs`)
   - `#[deprecated]` items past their removal window
4. Remove or flag each category with appropriate disposition
5. Remove deprecated `resource_handlers.rs` commands (tracked in REF-014)
6. Remove `#[allow(dead_code)]` annotations that are no longer needed
7. Verify `cargo clippy --no-deps` passes cleanly
8. Verify `cargo check` compiles without warnings

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src-tauri/src/models.rs` | Modify | Remove unused AppError variants |
| `src-tauri/src/commands.rs` | Modify | Remove dead functions and `#[allow(...)]` |
| `src-tauri/src/chat_v2/error.rs` | Review | Check for unused ChatV2Error variants |
| `src-tauri/src/vfs/error.rs` | Review | Check for unused VfsError variants |
| `src-tauri/src/dstu/error.rs` | Review | Check for unused DstuError variants |
| `src-tauri/src/memory/error.rs` | Review | Check for unused MemoryError variants |
| `src-tauri/src/essay_grading/error.rs` | Review | Check for unused EssayGradingError variants |
| `src-tauri/src/data_governance/error.rs` | Review | Check for unused DataGovernanceError variants |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| Various `*Error` variants | Enum | Present (possibly unused) | Removed if confirmed unused |
| Various struct fields | Field | Present | Removed if unused |
| `resource_*` commands | Function | 7 deprecated commands | Removed |

## Static Verification

- [ ] Run `cargo clippy --no-deps` before and after (compare warnings count)
- [ ] Run `cargo check` with all features enabled
- [ ] Manual code review for safety (removing error variants may affect pattern matches)

## Completion Criteria

- [ ] Zero `cargo clippy --no-deps` warnings for dead code
- [ ] All `#[deprecated]` Tauri commands removed
- [ ] All `#[allow(dead_code)]` annotations removed or justified with comments
- [ ] No functional regressions (all tests pass)
