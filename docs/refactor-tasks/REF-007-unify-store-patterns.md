# REF-007: Unify Store Patterns (Slice-Based vs Flat)

## Meta
- **Layer**: Layer 2 -- Data Models & State
- **Priority**: P2
- **Est. effort**: L
- **Predecessors**: REF-006
- **Scope**: `src/stores/`, `src/**/stores/`
- **Related reports**: `.planning/exploration/reports/cumulative-issues.md` (P2-07, P2-08), R04, R05

## Problem Description

Zustand stores in the codebase use **inconsistent patterns**:

**Pattern A -- Flat store (no slices):**
Most stores (uiStore, networkStore, ankiUIStore, etc.) are flat objects with all state and actions in a single object. For small stores this is fine, but for complex stores like `questionBankStore` (1630 lines), it creates an unmaintainable monolith.

**Pattern B -- Store with slices (questionsSlice, practiceSlice, etc.):**
Some feature stores have started adopting slice patterns, but inconsistently. Slices are sometimes in separate files, sometimes inline.

**Additional inconsistencies:**
- **`store/` vs `stores/` dual directory** (P2-07): `ResourceStateManager` uses a custom Pub/Sub pattern in `store/`, while everything else uses Zustand in `stores/`
- **Custom Persist** (P2-08): Some stores use custom versioning instead of Zustand's built-in `persist` middleware
- **`CHAT_HOST_FLAGS`** (P3-04): 12 feature flags hardcoded to `true`, with no runtime toggling mechanism
- **`NAV_ITEMS_COUNT=7`** hardcoded (P3-05)

## Target State

- All stores follow a consistent pattern:
  - **Small stores (< 200 lines)**: Flat pattern is acceptable
  - **Medium stores (200-600 lines)**: Slice pattern recommended
  - **Large stores (> 600 lines)**: MUST use slice pattern with slices in separate files
- `store/` directory eliminated (content migrated to `stores/`)
- All stores use Zustand built-in `persist` middleware
- Feature flags use a runtime-configurable mechanism
- Hardcoded constants (NAV_ITEMS_COUNT, CHAT_HOST_FLAGS) are centralized in a config module

## Steps

1. Document the canonical store pattern in CODE_STYLE.md or CONTRIBUTING.md
2. Define the slice pattern template (file organization, types, combine pattern)
3. For each store, classify its complexity tier (small/medium/large):
   - Small (< 200 lines): leave as-is, ensure consistency
   - Medium (200-600 lines): optionally refactor to slices
   - Large (> 600 lines): must refactor to slices
4. Migrate `store/ResourceStateManager` to Zustand
5. Standardize persist middleware usage across all stores
6. Centralize `CHAT_HOST_FLAGS` and `NAV_ITEMS_COUNT` into `src/config/features.ts`
7. Clean up the dual `store/` vs `stores/` directory structure

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src/stores/questionBankStore.ts` | Refactor | Enforce slice pattern |
| `src/stores/mindmapStore.ts` | Refactor | Enforce slice pattern |
| `src/stores/ankiQueueStore.ts` | Refactor | Standardize persist |
| `src/store/ResourceStateManager.ts` | Migrate | Move to `src/stores/` as Zustand store |
| `src/store/` | Remove | After migration |
| `src/config/features.ts` | Create or modify | Centralize feature flags |
| `src/stores/*.ts` (small stores) | Minor | Minor structural fixes if needed |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| `ResourceStateManager` | Class | `src/store/` Pub/Sub | `src/stores/` Zustand store |
| Store exports | Various | Inconsistent patterns | Consistent slice/flat pattern |

## Static Verification

- [ ] Run `npx tsc --noEmit` before and after
- [ ] Verify backward-compatible exports
- [ ] Check that no code imports from `src/store/` after migration
- [ ] Manual review of store consumers

## Completion Criteria

- [ ] `src/store/` directory fully removed
- [ ] All stores in `src/stores/` use consistent pattern
- [ ] All stores use Zustand built-in `persist`
- [ ] Feature flags and constants centralized in `src/config/`
- [ ] `npx tsc --noEmit` passes with zero errors
