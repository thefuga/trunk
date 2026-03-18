---
phase: 34-line-level-staging
plan: 02
subsystem: ui
tags: [svelte5, line-selection, toolbar-mode-switching, css-custom-properties, tauri-ipc]

# Dependency graph
requires:
  - phase: 34-line-level-staging
    provides: stage_lines, unstage_lines, discard_lines backend IPC commands
provides:
  - Line selection UI in DiffPanel with click/shift-click handlers
  - Toolbar mode switching between hunk-mode and selection-mode buttons
  - IPC calls from frontend to backend line-level commands
  - CSS tokens for selected line highlight backgrounds
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [toolbar mode switching based on selection state, per-hunk selection scoping]

key-files:
  created: []
  modified:
    - src/components/DiffPanel.svelte
    - src/app.css

key-decisions:
  - "No new dependencies -- all patterns reused from existing codebase (safeInvoke, showToast, ask(), CSS custom properties)"
  - "Selection state uses Set<number> for O(1) lookup on line indices, reassigned on mutation for Svelte 5 reactivity"

patterns-established:
  - "Toolbar mode switching: conditional rendering based on selectedHunkKey matching current hunk + selectedCount > 0"
  - "Line selection scoped per-hunk: clicking a line in a different hunk clears previous selection"

requirements-completed: [HUNK-07, HUNK-08]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 34 Plan 02: Line-Level Staging Frontend Summary

**Click-to-select diff lines with shift-click range selection, toolbar mode switching between hunk/line buttons, and IPC handlers for stage/unstage/discard lines**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-18T06:53:58Z
- **Completed:** 2026-03-18T06:56:28Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- DiffPanel renders clickable add/delete lines with pointer cursor, toggles selection on click, range-selects on shift+click
- Toolbar dynamically switches between "Stage Hunk"/"Discard Hunk" and "Stage Lines (N)"/"Discard Lines (N)" based on selection state
- Three IPC handlers (handleStageLines, handleUnstageLines, handleDiscardLines) wired to backend commands with error handling and toast feedback
- Selected lines use brighter background colors via CSS custom properties (0.25 alpha vs 0.1 unselected)
- Escape key clears selection, file navigation clears selection, cross-hunk click clears previous selection
- All colors use CSS custom properties -- zero inline hex or rgba values

## Task Commits

Each task was committed atomically:

1. **Task 1: Add CSS tokens and implement line selection state + click handlers + toolbar mode switching + IPC handlers** - `5ea7389` (feat)

## Files Created/Modified
- `src/components/DiffPanel.svelte` - Added line selection state ($state), click/shift-click handlers, toolbar mode switching, Escape key handler, pointer cursor on add/delete lines, selected-line background via lineBackground() extension, IPC handlers for stage/unstage/discard lines
- `src/app.css` - Added --color-diff-add-bg-selected and --color-diff-delete-bg-selected CSS custom properties

## Decisions Made
- No new dependencies needed -- all patterns reused from existing codebase (safeInvoke, showToast, ask(), CSS custom properties)
- Selection state uses Set<number> reassigned on each mutation for Svelte 5 reactivity (Svelte 5 tracks $state assignments, not mutations)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Line-level staging feature is complete end-to-end (backend + frontend)
- Phase 34 is fully complete -- both plans executed successfully
- Ready for visual verification via `cargo tauri dev`

---
*Phase: 34-line-level-staging*
*Completed: 2026-03-18*
