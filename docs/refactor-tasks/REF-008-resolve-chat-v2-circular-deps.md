# REF-008: Resolve Circular Dependencies in chat_v2 Module

## Meta
- **Layer**: Layer 3 -- Core Business Logic
- **Priority**: P1
- **Est. effort**: L
- **Predecessors**: None
- **Scope**: `src-tauri/src/chat_v2/` (entire module tree)
- **Related reports**: `.planning/exploration/reports/round-21-23-chat-v2-backend.md`, `round-24-26-backend-merged.md`

## Problem Description

The `chat_v2` module has known circular dependencies between its submodules:

**14 handler files** in `src-tauri/src/chat_v2/handlers/` reference each other and the core modules in ways that create potential cycles:

| File | Lines | Commands | Concern |
|------|-------|----------|---------|
| `send_message.rs` | ~600 | 5 | Message send/cancel/retry |
| `block_actions.rs` | ~300 | 7 | Delete/copy/update blocks |
| `manage_session.rs` | ~600+ | 14 | Session CRUD |
| `workspace_handlers.rs` | ~1200+ | 18 | Agent workspace management |
| `group_handlers.rs` | ~300 | 7 | Session grouping |

**Known cycle patterns:**
1. `manage_session.rs` calls `rebuild_session_skill_state_from_surviving_history` from `block_actions.rs`
2. `send_message.rs` imports from `ChatV2Pipeline`, `ChatV2State`, `ChatV2Repo`, `VfsResourceRepo`, and event emitters
3. `workspace_handlers.rs` imports `ChatV2Pipeline` and `ChatV2State` for agent execution
4. `handlers/mod.rs` re-exports everything, potentially creating a flat namespace that masks cycles

**Infrastructure modules with cross-cutting imports:**

| Module | Function | Imported By |
|--------|----------|-------------|
| `ChatV2Pipeline` | Message pipeline | `send_message.rs`, `workspace_handlers.rs` |
| `ChatV2State` | Stream management | `send_message.rs`, `workspace_handlers.rs` |
| `ChatV2Repo` | Database operations | All handler files |
| `ChatV2Database` | DB connection | All handler files |

## Target State

- Zero circular dependencies in `chat_v2` module tree
- Handler files depend only on:
  - Infrastructure modules (database, error, events, types)
  - Service/domain modules (repo, pipeline, state)
  - NOT on other handler files
- Cross-cutting concerns extracted to shared service modules
- `handlers/mod.rs` only re-exports individual handler symbols (no resolved function dependencies between handlers)

## Steps

1. Map the current dependency graph using `cargo modules` or manual `use` statement analysis
2. Identify all cycles, especially:
   - Handler-to-handler dependencies
   - Handler-to-pipeline dependencies that should be inverted
3. For each cycle, determine the fix:
   - Extract shared logic to a new service module under `src-tauri/src/chat_v2/services/`
   - Use trait objects to invert dependencies
   - Move utility functions to `src-tauri/src/chat_v2/utils/`
4. Extract `rebuild_session_skill_state_from_surviving_history` from `manage_session.rs` to a shared service
5. Extract workspace common helpers from `workspace_handlers.rs` to a shared workspace service
6. Restructure `handlers/mod.rs` to avoid flat re-exports
7. Verify with `cargo check` and `cargo clippy`
8. Check for any remaining cycles with `cargo modules`

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src-tauri/src/chat_v2/handlers/manage_session.rs` | Modify | Extract shared functions |
| `src-tauri/src/chat_v2/handlers/block_actions.rs` | Modify | Remove direct dependency on manage_session |
| `src-tauri/src/chat_v2/handlers/workspace_handlers.rs` | Modify | Extract shared helpers |
| `src-tauri/src/chat_v2/handlers/mod.rs` | Refactor | Re-export without circular references |
| `src-tauri/src/chat_v2/services/` | Create | New service module for shared logic |
| `src-tauri/src/chat_v2/utils.rs` | Create | Utility functions for session state rebuild etc. |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| `rebuild_session_skill_state_from_surviving_history` | Function | In `manage_session.rs` | Extracted to `services/session_skill.rs` |
| Workspace helpers | Various | In `workspace_handlers.rs` | Extracted to `services/workspace_service.rs` |

## Static Verification

- [ ] Run `cargo check` before and after
- [ ] Run `cargo clippy --no-deps` before and after
- [ ] Verify zero cycles using `cargo modules` or `cargo check` (Rust compiler rejects cycles)
- [ ] Manual code review for dependency inversion patterns

## Completion Criteria

- [ ] Zero circular dependencies in `chat_v2` module
- [ ] Handler files have no direct dependencies on other handler files
- [ ] All tests pass
- [ ] `cargo clippy --no-deps` passes with zero new warnings
