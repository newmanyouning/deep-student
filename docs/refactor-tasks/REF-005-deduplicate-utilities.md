# REF-005: Audit and Deduplicate Utility Functions

## Meta
- **Layer**: Layer 1 -- Utility Functions & Pure Logic
- **Priority**: P2
- **Est. effort**: M
- **Predecessors**: REF-004
- **Scope**: `src/utils/`, `src/lib/`, `src/shared/utils/`, `src/hooks/`
- **Related reports**: `.planning/exploration/reports/cumulative-issues.md` (P2-10, P2-12), R03, R05, R06
- **Diagnostic reports**: `.planning/exploration/reports/round-03-types-shared.md`, `round-05-api-services.md`

## Problem Description

The codebase has scattered utility functions across multiple directories with overlapping concerns:

**Known duplication patterns:**

| Utility | Location 1 | Location 2 | Notes |
|---------|-----------|-----------|-------|
| `cn()` class merge | `src/utils/cn.ts` | `src/lib/utils.ts` | Resolved in REF-004 |
| `getErrorMessage` | `src/utils/error.ts` (try-catch pattern) | Inline in stores (Rust-style `Result<T,E>`) | Two error handling styles coexist (P2-06) |
| `formatDate` / `formatDateTime` | Inline across ~8 components | `src/utils/date.ts`? | Not centralized |
| `uuid` / `generateId` | Inline | `crypto.randomUUID()` native | Mixed patterns |
| Storage helpers | Inline in stores | Zustand persist middleware | Some stores use custom persist, some use Zustand built-in |

**Additional concerns:**
- **Service layer has 4 architecture patterns** (P2-12): Singleton, static class, object literal, module function
- **`src/shared/index.ts`** only exports 3 of 11 components in the shared directory (P2-13)
- **`events/chat.ts`** has 3 of 6 events that are test stubs (P2-13)

## Target State

- Each utility category has a single canonical file/export
- `src/utils/` is the canonical location for all utility functions:
  - `src/utils/cn.ts` -- class merging
  - `src/utils/error.ts` -- error handling helpers
  - `src/utils/date.ts` -- date formatting
  - `src/utils/id.ts` -- ID generation
  - `src/utils/storage.ts` -- storage helpers
- Service layer uses consistent pattern (preferably module functions)
- Dead exports in `src/shared/` removed
- Test stubs in `events/chat.ts` removed

## Steps

1. Inventory all utility functions across `src/utils/`, `src/lib/`, `src/shared/utils/`, and inline in components
2. Categorize by function (date, error, id, storage, etc.)
3. For each category, pick the canonical implementation and consolidate
4. Update all imports across the codebase
5. Remove duplicated implementations
6. Clean up `src/shared/index.ts` exports
7. Remove test stubs from `events/chat.ts`
8. Run `npx tsc --noEmit` to verify

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src/utils/date.ts` | Create or populate | Centralize date formatting |
| `src/utils/id.ts` | Create or populate | Centralize ID generation |
| `src/utils/storage.ts` | Create or populate | Centralize storage helpers |
| `src/utils/error.ts` | Modify | Ensure single error handling pattern |
| `src/lib/utils.ts` | Modify | After REF-004, keep only non-cn utilities or remove |
| `src/shared/index.ts` | Modify | Fix exports to match actual components |
| `src/events/chat.ts` | Modify | Remove test stubs |
| ~10-20 component files | Modify | Update imports to canonical utility locations |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| `formatDate` | Function | Inline in N components | `src/utils/date.ts` |
| `generateId` | Function | Inline | `src/utils/id.ts` |
| Various storage helpers | Function | Scattered | `src/utils/storage.ts` |

## Static Verification

- [ ] Run `npx tsc --noEmit` before and after
- [ ] Run `npx ts-prune` to detect new dead exports
- [ ] Manual review for import consistency

## Completion Criteria

- [ ] All utility functions in canonical locations under `src/utils/`
- [ ] Zero duplicate implementations of the same utility
- [ ] `src/shared/index.ts` exports match reality
- [ ] `src/events/chat.ts` has zero test stubs
- [ ] `npx tsc --noEmit` passes with zero errors
