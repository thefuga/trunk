---
phase: 43-tech-debt-cleanup
plan: 01
subsystem: cleanup
tags: [dead-code, tech-debt, diff, rebase, type-safety]

requires:
  - phase: 37-conflict-detection-operation-state
    provides: "diff_conflicted command (now orphaned)"
  - phase: 41-interactive-rebase-editor
    provides: "rebaseBaseName lookup and handleOpenRebaseEditor"
provides:
  - "Clean codebase with no orphaned diff_conflicted references"
  - "Working rebaseBaseName resolution via resolve_ref IPC"
  - "Type-safe diffKind prop excluding 'conflicted'"
affects: []

tech-stack:
  added: []
  patterns: []

key-files:
  created: []
  modified:
    - src-tauri/src/lib.rs
    - src-tauri/src/commands/diff.rs
    - src/App.svelte

key-decisions:
  - "No code changes needed for D-04 (submit_rebase_message) -- already absent from source"

patterns-established: []

requirements-completed: []

duration: 4min
completed: 2026-03-23
---

# Phase 43 Plan 01: Tech Debt Cleanup Summary

**Removed orphaned diff_conflicted command/tests, fixed rebaseBaseName branch resolution via resolve_ref IPC, cleaned dead InputDialog import, narrowed diffKind type**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-23T15:33:02Z
- **Completed:** 2026-03-23T15:37:32Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Removed diff_conflicted command (inner function, async wrapper, invoke_handler registration, 2 tests) -- all 9 remaining diff tests pass
- Fixed rebaseBaseName lookup to actually resolve branch OIDs via resolve_ref IPC instead of always falling back to short OID
- Removed dead InputDialog import from App.svelte
- Narrowed diffKind type to exclude 'conflicted' before passing to DiffPanel

## Task Commits

Each task was committed atomically:

1. **Task 1: Remove diff_conflicted backend command (D-01, D-07)** - `a5f2562` (fix)
2. **Task 2: Fix App.svelte dead import, rebaseBaseName lookup, and diffKind type (D-02, D-03, D-04, D-05)** - `ad82b0e` (fix)

## Files Created/Modified
- `src-tauri/src/lib.rs` - Removed diff_conflicted from invoke_handler registration
- `src-tauri/src/commands/diff.rs` - Removed diff_conflicted_inner, async wrapper, and 2 tests (tests 10-11)
- `src/App.svelte` - Removed InputDialog import, fixed rebaseBaseName resolution, narrowed diffKind type

## Decisions Made
- No code changes needed for D-04 (submit_rebase_message) -- grep confirmed zero references in source, already clean

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- v0.8 milestone tech debt is clean
- All orphaned code from conflict/rebase phases removed
- Ready for milestone closure

## Self-Check: PASSED

All files exist, all commits verified.

---
*Phase: 43-tech-debt-cleanup*
*Completed: 2026-03-23*
