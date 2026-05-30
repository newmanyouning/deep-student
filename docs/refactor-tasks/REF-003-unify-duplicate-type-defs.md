# REF-003: Unify Duplicate Type Definitions Between Frontend/Backend

## Meta
- **Layer**: Layer 0 -- Base Types & Constants
- **Priority**: P2
- **Est. effort**: L
- **Predecessors**: REF-001, REF-002
- **Scope**: `src/types/` (frontend), `src-tauri/src/models.rs`, `src-tauri/src/**/types.rs` (backend)
- **Related reports**: `.planning/exploration/reports/round-03-types-shared.md`, `.planning/exploration/reports/round-20-backend-entry.md`

## Problem Description

The frontend `types/index.ts` and backend `src-tauri/src/models.rs` have overlapping domain type definitions that are maintained independently:

**Known duplicate or near-duplicate definitions:**

| Concept | Frontend (`types/index.ts`) | Backend (`models.rs` or module `types.rs`) |
|---------|---------------------------|-------------------------------------------|
| `ChatMessage` | `src/types/index.ts` | `src-tauri/src/chat_v2/types.rs` |
| `ApiConfig` | `src/types/index.ts` | `src-tauri/src/models.rs` (ModelAssignments, ApiConfig structs) |
| `AnkiCard` | `src/types/index.ts` | `src-tauri/src/models.rs` |
| `ExamSheetSessionDetail` | `src/types/index.ts` | `src-tauri/src/models.rs` |
| `Template`/`CustomAnkiTemplate` | `src/types/index.ts` | `src-tauri/src/models.rs` |
| `Theme` | `src/types/index.ts` | No backend equivalent (pure frontend -- OK) |
| `Events` | `src/types/index.ts` | `src-tauri/src/chat_v2/events/` |

The duplication causes:

- **Serialization mismatches**: Frontend sends fields that the backend doesn't expect, or vice versa
- **Inconsistent renaming**: The frontend uses camelCase (via `#[serde(rename_all = "camelCase")]`) but there's no automated check that frontend TS types match backend Rust structs
- **Manual sync burden**: Every time a backend type changes, the frontend type must be manually updated with no compiler enforcement
- **`MistakeItem` deprecation chain** (P2-09) is a concrete example of drift between frontend/backend

## Target State

- **Source of truth identified for each shared type**: either auto-generated from Rust (via ts-rs crate) or a manually maintained schema file with CI diff checks
- All `#[derive(Serialize, Deserialize)]` structs that cross the IPC boundary have corresponding frontend types that are verifiably in sync
- Option A (recommended): Add `ts-rs` crate to generate TypeScript definitions from Rust structs at build time
- Option B (fallback): Create a JSON Schema bridge with CI validation

## Steps

1. Inventory all `#[derive(Serialize, Deserialize)]` structs used in Tauri command return types and parameter types
2. Map each to its frontend TypeScript equivalent
3. Evaluate `ts-rs` integration: add `#[derive(TS)]` to key IPC structs, generate TS types
4. Create a build step (`build.rs` or script) to regenerate TS types from Rust
5. Remove manually maintained duplicate TS types, replace with generated ones
6. For structs that cannot use ts-rs (complex generic types), add JSON Schema tests in CI
7. Add a CI gate that fails if generated types differ from committed types

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src-tauri/Cargo.toml` | Modify | Add `ts-rs` dependency (optional, for generation) |
| `src-tauri/build.rs` | Create (or modify) | Add ts-rs codegen step |
| `src/types/index.ts` | Refactor | Remove/update manually maintained duplicate types |
| `src-tauri/src/models.rs` | Modify | Add `#[derive(TS)]` to IPC types |
| `src-tauri/src/chat_v2/types.rs` | Modify | Add `#[derive(TS)]` to IPC types |
| Various `handlers/*.rs` | Modify | Add `#[derive(TS)]` to request/response types |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| Frontend types matching Rust IPC structs | Various | Manually maintained | Auto-generated from Rust |

## Static Verification

- [ ] Run `npx tsc --noEmit` before and after
- [ ] Run `cargo check` before and after
- [ ] Verify generated TS types match existing manual types (diff check)
- [ ] Smoke test IPC calls that use shared types

## Completion Criteria

- [ ] Auto-generated TypeScript types from Rust structs for all IPC types
- [ ] CI gate enforcing generated types are up to date
- [ ] Zero manual type drift between frontend and backend for IPC types
- [ ] All existing tests pass
