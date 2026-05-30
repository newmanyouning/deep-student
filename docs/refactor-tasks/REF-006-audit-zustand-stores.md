# REF-006: Audit Zustand Stores for Unused State

## Meta
- **Layer**: Layer 2 -- Data Models & State
- **Priority**: P2
- **Est. effort**: M
- **Predecessors**: REF-001
- **Scope**: `src/stores/`, `src/**/stores/`
- **Related reports**: `.planning/exploration/reports/cumulative-issues.md` (P1-06: `questionBankStore` 1630-line God Store), R04, R05

## Problem Description

Zustand stores have accumulated unused or rarely-used state over time:

**God Stores:**

| Store | Lines | Issues |
|-------|-------|--------|
| `questionBankStore.ts` | 1630 | P1-06 -- CRUD, CSV import/export, practice modes, sync, stats, check-in calendar |
| `mindmapStore.ts` | 1526 | R11 -- multiple concerns mixed |
| `ankiQueueStore.ts` | ~800 | Custom persist instead of Zustand built-in (P2-08) |

**Pattern problems:**
- **Direct `invoke()` calls** bypassing the API layer (P1-07): `questionBankStore` calls 20+ Tauri commands directly instead of through the API service layer
- **Custom persist implementations** (P2-08): `ankiQueueStore` has a hand-rolled version-numbered persistence instead of using Zustand's built-in `persist` middleware
- **`store/` vs `stores/` dual directory** (P2-07): `ResourceStateManager` lives in `store/` (custom Pub/Sub) while all other stores are in `stores/` (Zustand)
- **Store calls to `invoke()`** bypass the `app/services/` API service layer, making it impossible to swap or mock the backend

## Target State

- Each store has:
  - Clear single responsibility
  - No direct `invoke()` calls (all go through API service layer)
  - Consistent pattern (Zustand with slices for complex stores)
  - Only actively used state fields
- `questionBankStore` split into ~3-4 focused stores or slice modules
- `store/` directory (Pub/Sub `ResourceStateManager`) migrated to Zustand or removed
- All stores use either Zustand built-in `persist` or a uniform custom persistence pattern

## Steps

1. Inventory all Zustand stores in `src/stores/` and feature `stores/` directories
2. For each store, trace all state fields to their consumers (components/hooks)
3. Identify unused or rarely-accessed state fields
4. Identify stores that directly call `invoke()` and plan migration to API layer
5. Design `questionBankStore` decomposition into focused slices
6. Implement splits, ensuring backward-compatible selectors during transition
7. Migrate `store/ResourceStateManager` to Zustand
8. Consolidate persistence strategy

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src/stores/questionBankStore.ts` | Refactor | Split into slices or focused stores |
| `src/stores/mindmapStore.ts` | Refactor | Remove unused state, clarify responsibility |
| `src/stores/ankiQueueStore.ts` | Refactor | Use Zustand built-in `persist` |
| `src/store/ResourceStateManager.ts` | Refactor | Migrate to Zustand |
| `src/stores/` (feature stores) | Modify | Remove unused state fields |
| `src/app/services/` (API layer) | Modify | Add missing API functions for store invoke calls |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| `questionBankStore` | Zustand store | 1630-line monolith | Split into focused stores with composed exports |
| `ResourceStateManager` | Pub/Sub class | `store/` directory | Zustand store in `stores/` |
| `ankiQueueStore.persist` | Middleware | Custom versioned | Zustand built-in `persist` |

## Static Verification

- [ ] Run `npx tsc --noEmit` before and after
- [ ] Verify store exports remain backward-compatible (re-export from original file)
- [ ] Manual review of store consumers (components) for regression

## Completion Criteria

- [ ] `questionBankStore.ts` reduced from 1630 to < 600 lines
- [ ] Zero direct `invoke()` calls from stores (all through API layer)
- [ ] `store/` directory removed (content migrated to `stores/`)
- [ ] All stores use consistent persistence pattern
- [ ] `npx tsc --noEmit` passes with zero errors
