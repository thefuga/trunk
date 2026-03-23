---
phase: 37-conflict-detection-operation-state
plan: 03
subsystem: backend, ui
tags: [rust, git2, tauri, svelte, diff, conflict-detection, merge]

requires:
  - phase: 37-conflict-detection-operation-state
    provides: "Plan 01: get_operation_state, merge/rebase commands; Plan 02: conflict UI, diffKind='commit' for read-only diffs"
provides:
  - "diff_conflicted Tauri command using repo.diff_tree_to_workdir() for conflicted file diffs"
  - "Frontend wiring to call diff_conflicted for conflicted files instead of diff_unstaged"
affects: [conflict-resolution-ui, merge-workflow, rebase-workflow]

tech-stack:
  added: []
  patterns: ["diff_tree_to_workdir for bypassing conflicted index entries (stages 1/2/3)"]

key-files:
  created: []
  modified:
    - "src-tauri/src/commands/diff.rs"
    - "src-tauri/src/lib.rs"
    - "src/App.svelte"

key-decisions:
  - "Used diff_tree_to_workdir (HEAD vs workdir) to bypass conflicted index entries that lack stage-0"

patterns-established:
  - "diff_conflicted pattern: when index is in conflict state, compare HEAD tree directly to workdir to show conflict markers"

requirements-completed: [CONF-01]

duration: 5min
completed: 2026-03-20
---

# Phase 37 Plan 03: Conflicted File Diff Summary

**diff_conflicted backend command using git2 diff_tree_to_workdir to show conflict markers when clicking conflicted files**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-20T23:41:32Z
- **Completed:** 2026-03-20T23:46:36Z
- **Tasks:** 2 (1 TDD + 1 auto)
- **Files modified:** 3

## Accomplishments
- New diff_conflicted_inner function compares HEAD tree to working directory, bypassing the index entirely so conflicted files (which only have stage 1/2/3 entries, not stage 0) produce valid diffs with conflict markers visible
- Frontend handleFileSelect and refetchFileDiff now call diff_conflicted instead of diff_unstaged for conflicted files
- Two new tests validate: conflict markers appear in diff output, and non-conflicted modified files still produce valid diffs
- All 11 existing diff tests continue to pass with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Add diff_conflicted backend command** - `726dc95` (test: RED), `f6f7544` (feat: GREEN)
2. **Task 2: Wire frontend to call diff_conflicted** - `7e87c24` (feat)

## Files Created/Modified
- `src-tauri/src/commands/diff.rs` - Added diff_conflicted_inner (HEAD tree to workdir diff) and diff_conflicted Tauri command wrapper, plus two tests
- `src-tauri/src/lib.rs` - Registered diff_conflicted in generate_handler! macro
- `src/App.svelte` - Changed handleFileSelect and refetchFileDiff to call diff_conflicted for conflicted kind

## Decisions Made
- Used diff_tree_to_workdir instead of diff_index_to_workdir because conflicted files have no stage-0 index entry (only stages 1/2/3 for base/ours/theirs), so index-based diffs return empty results

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Rust borrow checker required scoped blocks in the merge conflict test to avoid lifetime issues with git2 Commit objects holding references to Repository -- resolved by restructuring test with inner blocks for each commit operation

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Conflicted file diff display is now fully functional (closes UAT gap #4)
- Phase 37 all 3 plans complete: operation state detection, conflict UI, conflicted file diffs
- Ready for UAT re-verification of the conflicted file diff scenario

## Self-Check: PASSED

All files and commits verified.

---
*Phase: 37-conflict-detection-operation-state*
*Completed: 2026-03-20*
