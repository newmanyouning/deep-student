# REF-004: Resolve Duplicate `cn()` Function

## Meta
- **Layer**: Layer 1 -- Utility Functions & Pure Logic
- **Priority**: P1
- **Est. effort**: S
- **Predecessors**: None
- **Scope**: `src/utils/cn.ts`, `src/lib/utils.ts`, `src/App.tsx`, and all files importing either `cn()`
- **Related reports**: `.planning/exploration/reports/cumulative-issues.md` (P1-04, P2-10), `R02`, `R03`, `R06`

## Problem Description

There are **two implementations** of the `cn()` (className merge) function:

**Implementation A** -- `src/utils/cn.ts` (used by 75 files):
```typescript
import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```
- Has Tailwind class conflict resolution via `twMerge`
- Requires `clsx` and `tailwind-merge` dependencies
- Used by the **majority** of the codebase (75 import sites)

**Implementation B** -- `src/lib/utils.ts` (used by 2 files including `App.tsx`):
```typescript
export function cn(...inputs: ClassValue[]) {
  return inputs.map(toVal).filter(Boolean).join(' ');
}
```
- Hand-rolled implementation, zero dependencies
- Does **NOT** resolve Tailwind class conflicts (no `twMerge`)
- Used by `App.tsx` and one other file

**Impact:**
- `App.tsx` uses the hand-rolled `cn()` which does not merge Tailwind classes, causing potential class conflicts in the root component's styles (P1-04 in cumulative issues)
- ESLint rule R06 forces use of `NotionButton`/`CommonTooltip` components that use the main `cn()`, but App.tsx styles bypass this
- Two implementations is confusing for new contributors

## Target State

- Single canonical `cn()` implementation that supports Tailwind class merging
- All imports point to the same source
- `App.tsx` uses the standard `cn()`

## Steps

1. Adopt `src/utils/cn.ts` as the canonical implementation (it has `twMerge` support)
2. Update all imports from `@/lib/utils` to `@/utils/cn`
3. Verify `App.tsx` imports from `@/utils/cn` instead of `@/lib/utils`
4. After confirming no remaining imports, remove `src/lib/utils.ts` (or re-purpose it for non-cn utilities)
5. Run `npx tsc --noEmit` to verify no broken imports
6. Visual check that no style regressions occur from Tailwind class merging change

## Files Affected

| Path | Change | Description |
|------|--------|-------------|
| `src/utils/cn.ts` | Keep | Canonical implementation (already correct) |
| `src/lib/utils.ts` | Remove or re-purpose | Remove `cn()` or keep only non-cn utilities |
| `src/App.tsx` | Modify | Change import from `@/lib/utils` to `@/utils/cn` |
| Any file importing `@/lib/utils` just for `cn()` | Modify | Change import to `@/utils/cn` |
| `src/utils/cn.ts` may need `export type { ClassValue }` | Modify | Add re-export if needed by existing consumers |

## Interface Changes

| Symbol | Type | Old | New |
|--------|------|-----|-----|
| `cn()` in `@/lib/utils` | Function | Hand-rolled, no twMerge | Removed (redirect to `@/utils/cn`) |
| `cn()` in `@/utils/cn` | Function | Already complete | Remains canonical |

## Static Verification

- [ ] Run `npx tsc --noEmit` after changes
- [ ] Run `grep -r "from '@/lib/utils'" src/` to confirm zero `cn()` imports remain
- [ ] Visual regression check on `App.tsx` rendered output

## Completion Criteria

- [ ] Single `cn()` implementation at `src/utils/cn.ts`
- [ ] Zero imports of `cn()` from `@/lib/utils`
- [ ] `App.tsx` uses `twMerge`-enabled `cn()`
- [ ] `npx tsc --noEmit` passes with zero errors
