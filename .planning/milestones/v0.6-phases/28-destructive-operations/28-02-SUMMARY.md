---
phase: 28-destructive-operations
plan: 02
subsystem: ui
tags: [svelte, tauri-menu, dialog, context-menu, discard]

# Dependency graph
requires:
  - phase: 28-destructive-operations
    provides: Backend discard_file and discard_all IPC commands
provides:
  - Right-click context menu on unstaged files for single-file discard
  - Discard All button in unstaged header for bulk discard
  - Confirmation dialogs with differentiated warnings for tracked vs untracked files
  - Toast feedback on discard operations
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dynamic import of @tauri-apps/plugin-dialog and @tauri-apps/api/menu inside handlers"
    - "Differentiated confirmation dialogs based on file status (tracked vs untracked)"

key-files:
  created: []
  modified:
    - src/components/FileRow.svelte
    - src/components/StagingPanel.svelte

key-decisions:
  - "oncontextmenu prop added to FileRow but only wired on unstaged files (not staged) — discard only applies to unstaged"
  - "Context menu and dialog imports are dynamic (inside handlers) following BranchSidebar pattern"

patterns-established:
  - "FileRow oncontextmenu prop pattern: parent controls right-click behavior per-section"

requirements-completed: [GITOP-01, GITOP-02]

# Metrics
duration: 2min
completed: 2026-03-15
---

# Phase 28 Plan 02: Frontend Discard Operations Summary

**Right-click context menu for single-file discard and Discard All button with confirmation dialogs and toast feedback in StagingPanel**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-15T20:17:16Z
- **Completed:** 2026-03-15T20:20:14Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- FileRow gains oncontextmenu prop for right-click handling on unstaged files
- StagingPanel wires discard single file via native Tauri context menu with differentiated warnings (tracked: "Discard Changes", untracked: "Delete File")
- Discard All button in unstaged header shows count-based confirmation before invoking discard_all IPC
- Toast notifications confirm successful discard operations or report errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add oncontextmenu to FileRow + wire discard context menu in StagingPanel** - `e9bb052` (feat)

## Files Created/Modified
- `src/components/FileRow.svelte` - Added oncontextmenu prop to Props interface and outer div
- `src/components/StagingPanel.svelte` - Added handleDiscardFile, showFileContextMenu, handleDiscardAll functions, Discard All button, toast imports

## Decisions Made
- oncontextmenu prop wired only on unstaged FileRow instances (not staged) — discard only applies to unstaged changes
- Dynamic imports for dialog and menu APIs inside handlers, following established BranchSidebar pattern

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `bun run check` exits with code 1 due to pre-existing errors in virtual-list module (unrelated to this plan's changes). FileRow and StagingPanel have zero errors. This is a known pre-existing condition.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Ready for 28-03 (branch/tag delete, rename operations in sidebar and graph pills)
- discard operations fully wired end-to-end (backend from 28-01 + frontend from 28-02)

---
*Phase: 28-destructive-operations*
*Completed: 2026-03-15*
