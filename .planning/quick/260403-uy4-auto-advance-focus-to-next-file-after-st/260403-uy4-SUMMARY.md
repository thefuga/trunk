---
phase: quick-260403-uy4
plan: 01
subsystem: ui
tags: [svelte, staging, diff, auto-advance, ux]

requires: []
provides:
  - advanceToNextFile shared function in RepoView for auto-advancing file selection
  - onfileadvance callback pattern from StagingPanel to RepoView
affects: [staging, diff-panel, merge-editor]

tech-stack:
  added: []
  patterns:
    - "advanceToNextFile pattern: fetch fresh status, pick next file in same section, or clear diff"
    - "onfileadvance callback: StagingPanel notifies parent after single-file actions, parent decides whether to advance"

key-files:
  created: []
  modified:
    - src/components/RepoView.svelte
    - src/components/StagingPanel.svelte

key-decisions:
  - "Auto-advance picks first remaining file when current file leaves section (not index-based) for simplicity"
  - "Bulk operations (Stage All, Unstage All, Discard All) do NOT trigger onfileadvance -- only single-file actions do"
  - "handleFileResolved refactored to reuse advanceToNextFile instead of inline logic"

patterns-established:
  - "advanceToNextFile: centralized auto-advance logic reusable for unstaged, staged, and conflicted sections"

requirements-completed: [AUTO-ADVANCE]

duration: 5min
completed: 2026-04-03
---

# Quick 260403-uy4: Auto-advance File Selection Summary

**Auto-advance to next file in same section after staging, unstaging, or discarding via DiffPanel toolbar, StagingPanel buttons, or hunk/line actions**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-04T04:02:04Z
- **Completed:** 2026-04-04T04:07:43Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `advanceToNextFile` function to RepoView that fetches fresh status and selects the next file in the same section
- Wired DiffPanel `onhunkaction` to detect empty diffs (all hunks gone) and trigger auto-advance
- Added `onfileadvance` callback prop to StagingPanel, called after stageFile, unstageFile, handleDiscardFile, and resolveConflictedFile
- Refactored `handleFileResolved` to reuse `advanceToNextFile` instead of duplicating logic
- Bulk operations (Stage All, Unstage All, etc.) intentionally do NOT auto-advance

## Task Commits

Each task was committed atomically:

1. **Task 1: Add advanceToNextFile to RepoView and wire onhunkaction** - `73eb560` (feat)
2. **Task 2: Wire StagingPanel stageFile/unstageFile to call onfileadvance** - `753358e` (feat)

## Files Created/Modified
- `src/components/RepoView.svelte` - Added advanceToNextFile function, wired onhunkaction empty-diff detection, refactored handleFileResolved, added onfileadvance prop to StagingPanel instance
- `src/components/StagingPanel.svelte` - Added onfileadvance to Props interface, called it after stageFile, unstageFile, handleDiscardFile, resolveConflictedFile

## Decisions Made
- Auto-advance picks the first remaining file in the section when the current file leaves (rather than trying to guess the original index position) -- simpler and matches most Git GUI behavior
- Bulk operations intentionally excluded from auto-advance to avoid jarring UX when multiple files move at once
- Conflict resolution from context menu ("Take All Current/Incoming") now also triggers auto-advance via onfileadvance callback

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Self-Check: PASSED

- All modified files exist on disk
- All commit hashes verified in git log (73eb560, 753358e)
- SUMMARY.md created at expected path
- `just check` passes all 6 checks with 0 errors

---
*Phase: quick-260403-uy4*
*Completed: 2026-04-03*
