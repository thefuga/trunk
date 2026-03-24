---
phase: 45-frontend-tab-architecture
plan: 01
subsystem: ui
tags: [svelte5, state-management, factory-pattern, persistence, tabs]

requires:
  - phase: 44-backend-state-scoping
    provides: per-repo RunningOp HashMap pattern

provides:
  - TabInfo and PersistedTab type contracts for multi-tab architecture
  - createRemoteState() factory for per-tab remote operation state
  - createUndoRedoState() factory for per-tab undo/redo state
  - Tab persistence helpers (getOpenTabs, setOpenTabs, getActiveTabId, setActiveTabId)
  - Backward-compatible singleton exports for existing consumers

affects: [45-02, 45-03, tab-bar, repo-view-extraction]

tech-stack:
  added: []
  patterns: [factory-function-for-per-instance-state, backward-compat-singleton-alias]

key-files:
  created:
    - src/lib/tab-types.ts
    - src/lib/remote-state.svelte.test.ts
    - src/lib/undo-redo.svelte.test.ts
    - src/lib/store.test.ts
  modified:
    - src/lib/remote-state.svelte.ts
    - src/lib/undo-redo.svelte.ts
    - src/lib/store.ts

key-decisions:
  - "$state() must be assigned to variable declaration, not returned inline from factory -- Svelte 5 compiler constraint"
  - "Backward-compat singleton aliases kept as module-level constants calling factory, so consumers compile without changes until Plan 02"

patterns-established:
  - "Factory function for per-instance $state: declare const inside function, return it"
  - "Backward-compat singleton: const _compat = createFactory(); export const old = _compat.prop"

requirements-completed: [TAB-05, TAB-06]

duration: 4min
completed: 2026-03-24
---

# Phase 45 Plan 01: Tab Foundation Summary

**Factory functions replacing global singletons for per-tab remote/undo-redo state, TabInfo/PersistedTab type contracts, and LazyStore tab persistence helpers with 14 unit tests**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-24T03:32:31Z
- **Completed:** 2026-03-24T03:36:34Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Created TabInfo, PersistedTab interfaces and createTabId() UUID generator in new tab-types.ts
- Refactored remote-state.svelte.ts from singleton to createRemoteState() factory with backward-compat export
- Refactored undo-redo.svelte.ts from singleton to createUndoRedoState() factory returning UndoRedoManager with push/pop/clear, plus backward-compat exports
- Added tab persistence to store.ts: getOpenTabs, setOpenTabs, getActiveTabId, setActiveTabId
- 14 unit tests proving factory instance independence, correct defaults, LIFO ordering, UUID format

## Task Commits

Each task was committed atomically:

1. **Task 1: Create tab types, refactor singletons to factories, add tab persistence**
   - `afbf8e3` (test: add failing tests for tab types, factory isolation, and persistence)
   - `761ffe9` (feat: create tab types, refactor singletons to factories, add tab persistence)
2. **Task 2: Unit tests for factory isolation and persistence helpers**
   - `8f8f596` (test: comprehensive unit tests for factory isolation and type contracts)

## Files Created/Modified
- `src/lib/tab-types.ts` - TabInfo, PersistedTab interfaces, createTabId() UUID generator
- `src/lib/remote-state.svelte.ts` - createRemoteState() factory + RemoteState interface + backward-compat singleton
- `src/lib/undo-redo.svelte.ts` - createUndoRedoState() factory returning UndoRedoManager + backward-compat exports
- `src/lib/store.ts` - Tab persistence functions (getOpenTabs, setOpenTabs, getActiveTabId, setActiveTabId)
- `src/lib/remote-state.svelte.test.ts` - 4 tests for defaults and instance independence
- `src/lib/undo-redo.svelte.test.ts` - 6 tests for push/pop/clear/independence
- `src/lib/store.test.ts` - 4 tests for createTabId UUID and type contracts

## Decisions Made
- `$state()` placement constraint: Svelte 5 compiler requires `$state(...)` as variable declaration initializer, not in return statement. Factory pattern uses `const state = $state({...}); return state;` instead of `return $state({...});`
- Backward-compat singleton aliases call factory at module scope so existing Toolbar, PullDropdown, CommitForm, CommitGraph consumers compile without changes until Plan 02 migrates them

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed $state() placement in createRemoteState factory**
- **Found during:** Task 1 (GREEN phase)
- **Issue:** Plan showed `return $state({...})` but Svelte 5 compiler rejects `$state()` outside variable declaration initializer
- **Fix:** Changed to `const state = $state({...}); return state;`
- **Files modified:** src/lib/remote-state.svelte.ts
- **Verification:** `bun run test` passes, `bun run check` shows no new errors
- **Committed in:** 761ffe9

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial syntactic adjustment required by Svelte 5 compiler. No scope creep.

## Issues Encountered
None beyond the $state() placement issue documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Tab type contracts and factory functions ready for Plan 02 (RepoView extraction and consumer migration)
- Plan 02 can import createRemoteState/createUndoRedoState in RepoView.svelte and pass instances as props
- Backward-compat exports ensure zero breakage during incremental migration

---
*Phase: 45-frontend-tab-architecture*
*Completed: 2026-03-24*
