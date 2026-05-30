# REF-001: Audit and Prune Unused TypeScript Types

## Meta
- **Layer**: Layer 0 -- Base Types & Constants
- **Priority**: P1
- **Est. effort**: M
- **Predecessors**: None
- **Scope**: `src/types/`, `src/**/*.ts`, `src/**/*.tsx`
- **Related reports**: `.planning/exploration/reports/cumulative-issues.md` (P1-05: `types/index.ts` 1036-line God File), `R03`
- **Diagnostic reports**: `.planning/exploration/reports/round-03-types-shared.md`

## Problem Description

`src/types/index.ts` is a 1036-line God File containing:

- **300+ exported types/interfaces** with no internal organization or module separation
- **Multiple deprecated types** still exported and potentially referenced:
  - `MistakeItem` (deprecated 2026-01) has a 4-layer dependency chain (`types/index.ts` -> `types/api.ts` -> `stores/anki/types.ts` -> `stores/anki/useAnkiUIStore.ts` -> `app/services/saveRequestHandler.ts`) with one file still using it at runtime
- **Dead exports** with zero imports across the codebase
- **Mixed concerns**: API response types, component prop types, event types, store state types all in one file
- **Vue macro remnants**: `defineEmits`, `defineProps` type patterns left from a Vue era (P2-01 in cumulative issues)
- **Lack of `strict: true`** in tsconfig (P1-01): many types that would fail under strict mode coexist silently

Additionally, `types/api.ts`, `types/ui.ts`, and `types/hooks.ts` are pure re-export files (P2-11) that add no value -- they just re-export from `index.ts`.

## Target State

- `types/index.ts` split into domain-specific modules under `src/types/`:
  - `src/types/api.ts` -- API request/response types
  - `src/types/domain.ts` -- Domain models (ChatMessage, AnkiCard, etc.)
  - `src/types/events.ts` -- Event types
  - `src/types/ui.ts` -- Component prop and UI state types
- All truly unused types removed (confirmed by ts-prune or manual audit)
- Deprecated types (`MistakeItem` and chain) fully removed or replaced
- `types/api.ts`, `types/ui.ts`, `types/hooks.ts` eliminated (content merged or removed)
- `strict: true` enabled in tsconfig to catch latent type issues

## Steps

1. Run `npx ts-prune` (or `npx ts-unused-exports`) to identify unused exports
2. Categorize all types in `types/index.ts` by domain (API, Domain, Events, UI)
3. Split into domain modules under `src/types/` (breaking the God File)
4. Verify each re-export file (`types/api.ts`, `types/ui.ts`, `types/hooks.ts`) -- if pure re-exports, eliminate them
5. Trace the `MistakeItem` dependency chain end-to-end; remove or replace the deprecated type
6. Remove unused types confirmed by ts-prune
7. Enable `strict: true` in `tsconfig.json` and fix any new errors
8. Run `npx tsc --noEmit` to verify zero errors

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src/types/index.ts` | Refactor | Split into domain modules, remove unused exports |
| `src/types/api.ts` | Eliminate | Pure re-export layer -- merge into api types module |
| `src/types/ui.ts` | Eliminate | Pure re-export layer -- merge into ui types module |
| `src/types/hooks.ts` | Eliminate | Pure re-export layer -- merge or remove |
| `src/types/domain.ts` | Create | Domain model types from extracted content |
| `src/types/events.ts` | Create | Event type definitions from extracted content |
| `tsconfig.json` | Modify | Enable `strict: true` |
| `stores/anki/types.ts` | Modify | Remove `MistakeSummary = MistakeItem` alias |
| `stores/anki/useAnkiUIStore.ts` | Modify | Replace MistakeItem usage with current equivalent |
| `app/services/saveRequestHandler.ts` | Modify | Replace MistakeItem runtime usage |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| `MistakeItem` | Interface | Deprecated, in `types/index.ts` | Removed |
| `MistakeSummary` | Type alias | `= MistakeItem` (in `stores/anki/types.ts`) | Inline the fields or use current type |
| All re-exports from `types/api.ts` | Module | Re-exports from `index.ts` | Direct exports from domain modules |

## Static Verification

- [ ] Run `npx tsc --noEmit` after changes
- [ ] Run `npx ts-prune` to confirm zero unused exports
- [ ] Verify no broken imports with `npx tsc --noEmit --traceResolution`
- [ ] Manual code review for API contract consistency

## Completion Criteria

- [ ] `types/index.ts` reduced from 1036 lines to < 200 lines (just re-exports from domain modules)
- [ ] `types/api.ts`, `types/ui.ts`, `types/hooks.ts` eliminated
- [ ] `MistakeItem` fully removed from codebase
- [ ] `strict: true` enabled in tsconfig
- [ ] `npx tsc --noEmit` passes with zero errors
- [ ] No runtime regressions (smoke test key flows)
