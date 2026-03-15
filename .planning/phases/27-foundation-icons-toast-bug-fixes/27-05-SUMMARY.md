---
phase: 27-foundation-icons-toast-bug-fixes
plan: 05
subsystem: ui
tags: [svelte, commit-graph, column-resize, bug-fix]

requires:
  - phase: 27-foundation-icons-toast-bug-fixes
    provides: lastVisibleColumn derived state already implemented for other columns

provides:
  - message column resize handle guarded by lastVisibleColumn check — no trailing divider when message is rightmost visible column

affects: [commit-graph, column-visibility, column-resize]

tech-stack:
  added: []
  patterns:
    - "{#if 'col' !== lastVisibleColumn} pattern applied consistently to all column resize handles"

key-files:
  created: []
  modified:
    - src/components/CommitGraph.svelte

key-decisions:
  - "Preserved startColumnResize('author', e, true) handler inside the guard — it resizes author column from the left edge of the message cell, so it must remain when author is visible"

patterns-established:
  - "All column resize handles now follow identical lastVisibleColumn guard pattern"

requirements-completed: [FIX-02]

duration: 3min
completed: 2026-03-15
---

# Phase 27 Plan 05: Guard Message Column Resize Handle Summary

**Added `{#if 'message' !== lastVisibleColumn}` guard to message column's trailing resize handle, eliminating the spurious right-edge divider when author/date/sha columns are all hidden**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-15T04:52:00Z
- **Completed:** 2026-03-15T04:55:04Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Wrapped message column's `col-resize-handle` div in `{#if 'message' !== lastVisibleColumn}...{/if}` guard
- Removed misleading comment claiming the handle was "the LEFT edge of the author column" (it's the right edge of message)
- All other column guards (ref, graph, author, date, sha) remain intact and unchanged
- UAT test 7 fix: when message is the last visible column, no trailing resize divider appears on its right edge

## Task Commits

Each task was committed atomically:

1. **Task 1: Guard message column resize handle with lastVisibleColumn check** - `d22d591` (fix)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified
- `src/components/CommitGraph.svelte` - Added `{#if 'message' !== lastVisibleColumn}` guard around the message column's resize handle div (lines 490-493)

## Decisions Made
- Preserved the `startColumnResize('author', e, true)` handler inside the guard — this handler is correct (it resizes author from message's right edge) and should still fire whenever author is visible. Only suppressed when message is truly the last visible column.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- FIX-02 complete: message column trailing resize divider suppressed when message is the rightmost visible column
- All column resize handle guards now use consistent `{#if 'col' !== lastVisibleColumn}` pattern
- Ready for remaining plans in phase 27

---
*Phase: 27-foundation-icons-toast-bug-fixes*
*Completed: 2026-03-15*
