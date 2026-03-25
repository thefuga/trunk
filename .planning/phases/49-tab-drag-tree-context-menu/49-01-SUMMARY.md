---
phase: 49-tab-drag-tree-context-menu
plan: 01
subsystem: ui
tags: [sortablejs, drag-and-drop, tabs, svelte5, reorder]

# Dependency graph
requires:
  - phase: 45-multi-tab-architecture
    provides: TabBar component with tab management, persistTabs() mechanism
provides:
  - SortableJS drag-and-drop tab reordering in TabBar
  - Persisted tab order via existing persistTabs() mechanism
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [SortableJS horizontal drag with {#key} reconciliation pattern]

key-files:
  created: []
  modified:
    - src/components/TabBar.svelte
    - src/App.svelte

key-decisions:
  - "Reused existing SortableJS pattern from RebaseEditor with horizontal direction"
  - "filter: '.new-tab-btn' excludes + button from draggable items while keeping it clickable"

patterns-established:
  - "SortableJS horizontal drag: direction:'horizontal' + scroll:true + scrollSensitivity:50 for tab bars"

requirements-completed: [TAB-11]

# Metrics
duration: 2min
completed: 2026-03-25
---

# Phase 49 Plan 01: Tab Drag Reorder Summary

**SortableJS drag-and-drop tab reordering with auto-scroll, new-tab exclusion, and persisted order**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-25T03:19:20Z
- **Completed:** 2026-03-25T03:21:50Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- SortableJS integrated into TabBar with horizontal drag, smooth animation, and auto-scroll at edges
- New-tab (+) button excluded from draggable items via CSS class filter
- {#key tabs} reconciliation prevents Svelte/SortableJS DOM conflicts after reorder
- Tab reorder persists across app relaunch via existing persistTabs() mechanism

## Task Commits

Each task was committed atomically:

1. **Task 1: Integrate SortableJS into TabBar with drag styling** - `b7bd6d2` (feat)
2. **Task 2: Wire onreorder in App.svelte to update tabs and persist** - `ccb0ad5` (feat)

## Files Created/Modified
- `src/components/TabBar.svelte` - Added SortableJS import, onreorder prop, $effect with Sortable.create, {#key tabs} wrapper, and global drag CSS classes
- `src/App.svelte` - Added onreorder callback that updates tabs array and calls persistTabs()

## Decisions Made
- Reused established SortableJS pattern from RebaseEditor (forceFallback, animation:150, {#key} reconciliation)
- Used `filter: '.new-tab-btn'` with `preventOnFilter: false` to keep the + button clickable but non-draggable
- Used theme CSS custom properties (var(--color-selected-row)) for drag styling per project rules

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Tab drag reorder complete, ready for Plan 02 (tree view context menu)
- No blockers or concerns

## Self-Check: PASSED

- All files exist (TabBar.svelte, App.svelte, 49-01-SUMMARY.md)
- All commits verified (b7bd6d2, ccb0ad5)

---
*Phase: 49-tab-drag-tree-context-menu*
*Completed: 2026-03-25*
