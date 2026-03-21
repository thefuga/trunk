---
phase: 40-rebase-workflow
plan: 01
subsystem: ui
tags: [svelte, context-menu, rebase, git]

# Dependency graph
requires:
  - phase: 39-merge-workflow
    provides: "Merge context menu items on all branch surfaces (pattern to replicate)"
  - phase: 37-operation-detection
    provides: "rebase_branch IPC command, OperationBanner for mid-rebase conflict resolution"
provides:
  - "Rebase context menu items on all 6 branch surfaces (pill, overflow, sidebar)"
  - "Toast-free rebase handler matching merge pattern"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Rebase menu items adjacent to merge items in all context menus"

key-files:
  created: []
  modified:
    - src/components/CommitGraph.svelte
    - src/components/BranchSidebar.svelte

key-decisions:
  - "No success toast on rebase -- graph refresh via repo-changed event is sufficient (matches merge pattern)"
  - "Rebase items always adjacent to merge items -- no separator between them"

patterns-established:
  - "Merge+Rebase grouped together in context menus, followed by separator"

requirements-completed: [REB-01, REB-02, REB-04, REB-05, REB-06]

# Metrics
duration: 2min
completed: 2026-03-21
---

# Phase 40 Plan 01: Rebase Workflow Summary

**Rebase context menu items on all 6 branch surfaces (pills, overflow refs, sidebar) with toast-free success handling**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-21T17:40:35Z
- **Completed:** 2026-03-21T17:42:30Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added "Rebase [HEAD] onto [branch]" to 4 CommitGraph surfaces (local pill, remote pill, local overflow, remote overflow)
- Added handleRebaseBranch handler and rebase items to 2 BranchSidebar surfaces (local and remote)
- Removed success toast from CommitGraph rebase handler (matches merge pattern)
- All 7 branch surfaces now have rebase alongside merge (commit menu was already done)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add rebase to CommitGraph pill and overflow ref menus, remove success toast** - `294ac53` (feat)
2. **Task 2: Add rebase handler and menu items to BranchSidebar** - `4a9f7c3` (feat)

## Files Created/Modified
- `src/components/CommitGraph.svelte` - Added rebase items to 4 menu surfaces, removed success toast from handler
- `src/components/BranchSidebar.svelte` - Added handleRebaseBranch handler, rebase items in local and remote branch menus

## Decisions Made
- No success toast on rebase -- graph refresh via repo-changed event is sufficient (matches merge pattern from Phase 39)
- Rebase items always adjacent to merge items with no separator between them (separator after the group)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All rebase context menu surfaces complete
- Mid-rebase conflict resolution handled by existing Phase 37-38 infrastructure (OperationBanner, StagingPanel, MergeEditor)
- Abort/Skip during rebase handled by existing OperationBanner buttons

## Self-Check: PASSED

All files found. All commits verified.

---
*Phase: 40-rebase-workflow*
*Completed: 2026-03-21*
