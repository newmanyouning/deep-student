# REF-010: Unify VFS Repo Access Patterns

## Meta
- **Layer**: Layer 3 -- Core Business Logic
- **Priority**: P2
- **Est. effort**: M
- **Predecessors**: REF-011
- **Scope**: `src-tauri/src/vfs/repos/`, `src-tauri/src/vfs/handlers.rs`, `src-tauri/src/vfs/database.rs`
- **Related reports**: `.planning/exploration/reports/round-24-26-backend-merged.md` (VFS section), `docs/refactor-progress-summary.md`

## Problem Description

The VFS (Virtual File System) module has **inconsistent repository access patterns**:

**Current state:**
All VFS repos are public structs in `src-tauri/src/vfs/repos/` with static methods:

```rust
// Pattern A: Static method taking &VfsDatabase
VfsNoteRepo::get_note(vfs_db, note_id)
VfsExamRepo::get_exam_sheet(vfs_db, exam_id)

// Pattern B: Static method that constructs connection internally
// (less common but exists)
```

**Inconsistencies:**

1. **Some handlers create connections manually**, others use repo methods -- no uniform pattern
2. **Error handling varies**: some handlers use `.map_err(|e| ChatV2Error::IoError(e.to_string()))` even after the error unification (REF-401 target), others use `?` with VfsError
3. **Cross-module VFS calls**: `chat_v2/handlers/` imports VFS repos directly (e.g., `send_message.rs` imports `VfsResourceRepo`), bypassing any service layer
4. **VFS access from non-VFS modules**: Memory module, DSTU module, and essay_grading module all access VFS repos directly with different patterns
5. **State management duplication**: `Arc<VfsDatabase>` is passed as Tauri State, but repos don't encapsulate the state management

**Impact:**
- Scattered VFS access patterns make it hard to change the database layer
- Direct repo access from non-VFS modules creates tight coupling
- Error handling inconsistencies persisted after error type migration

## Target State

- **Service layer** between handlers and repos:
  ```
  Handlers -> VfsService (encapsulates db + repos) -> Repos
  ```
- `VfsService` manages `Arc<VfsDatabase>` internally and exposes domain methods
- All non-VFS modules access VFS through `VfsService` only (not repos directly)
- Consistent error propagation using `VfsError` throughout
- Repos become package-private (pub(crate) with limited visibility)

## Steps

1. Define `VfsService` struct with constructor and domain methods
2. Move handler-level orchestration logic from `handlers.rs` into `VfsService` where applicable
3. Update `chat_v2/handlers/` to use `VfsService` instead of direct repo access
4. Update `memory/`, `dstu/`, `essay_grading/` modules to use `VfsService`
5. Ensure all VFS error paths use `VfsError` consistently
6. Register `VfsService` as Tauri State (Arc<VfsService>) or have handlers construct it from VfsDatabase state
7. Reduce repo visibility where possible

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src-tauri/src/vfs/service.rs` | Create | VfsService struct with domain methods |
| `src-tauri/src/vfs/mod.rs` | Modify | Export VfsService |
| `src-tauri/src/vfs/handlers.rs` | Refactor | Use VfsService internally |
| `src-tauri/src/vfs/repos/*.rs` | Modify | Reduce visibility to pub(crate) where possible |
| `src-tauri/src/chat_v2/handlers/send_message.rs` | Modify | Use VfsService instead of direct VfsResourceRepo |
| `src-tauri/src/chat_v2/handlers/block_actions.rs` | Modify | Use VfsService |
| `src-tauri/src/memory/service.rs` | Modify | Use VfsService |
| `src-tauri/src/dstu/handlers.rs` | Modify | Use VfsService |
| `src-tauri/src/essay_grading/mod.rs` | Modify | Use VfsService |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| VFS access pattern | Architecture | Direct repo + db | VfsService layer |
| VfsService | Struct | Doesn't exist | Encapsulates db + all repos |
| Repo visibility | Module | `pub` | `pub(crate)` |

## Static Verification

- [ ] Run `cargo check` before and after
- [ ] Run `cargo clippy --no-deps` before and after
- [ ] Run `cargo test` (unit + integration)
- [ ] Verify that no non-VFS module directly imports from `vfs::repos::*`

## Completion Criteria

- [ ] `VfsService` exists and exposes all VFS domain operations
- [ ] No non-VFS module directly imports VFS repos (only `VfsService`)
- [ ] Consistent error handling with `VfsError` across all VFS access points
- [ ] All tests pass
- [ ] `cargo clippy --no-deps` passes with zero new warnings
