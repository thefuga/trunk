---
status: complete
phase: quick
plan: 260325-up9
subsystem: ui
tags: [svelte, keyboard-navigation, commit-graph, a11y]

provides:
  - "Keyboard ArrowUp/ArrowDown navigation in commit graph pane"
affects: [commit-graph, commit-selection]

tech-stack:
  added: []
  patterns: ["Container tabindex + onkeydown for keyboard-driven list navigation"]

key-files:
  created: []
  modified:
    - src/components/CommitGraph.svelte

key-decisions:
  - "Used VirtualList scroll with align:auto instead of scrollToOid for instant minimal scrolling (no smooth scroll, no centering)"
  - "Added svelte-ignore for a11y_no_noninteractive_tabindex since listbox role with tabindex is semantically correct"
  - "Auto-focus container only when document.body is active (no stealing focus from search bar or other elements)"

requirements-completed: []

duration: 1min
completed: 2026-03-26
---

# Quick Task 260325-up9: Keyboard Arrow Navigation for Commit Graph

**ArrowUp/ArrowDown keyboard navigation in commit graph pane with auto-scroll and search bar exemption**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-26T01:09:18Z
- **Completed:** 2026-03-26T01:10:33Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- ArrowDown/ArrowUp moves selected commit in the graph pane with clamping at boundaries
- Selected row auto-scrolls into view using VirtualList's scroll method (instant, minimal movement)
- If no commit is selected, ArrowDown selects first commit, ArrowUp selects last
- WIP row selection correctly triggers onWipClick callback
- Arrow keys are ignored when search bar input has focus
- Container auto-focuses on mount for immediate keyboard use without clicking

## Task Commits

1. **Task 1: Add keyboard arrow navigation to CommitGraph** - `a048c3f` (feat)

## Files Created/Modified

- `src/components/CommitGraph.svelte` - Added containerRef, handleKeydown handler, auto-focus effect, tabindex/role/onkeydown on outer div

## Decisions Made

- Used `listRef.scroll({ align: "auto" })` instead of `scrollToOid` for keyboard nav -- scrollToOid centers the row with smooth scroll which feels sluggish for rapid key presses; align:auto does minimal instant scrolling
- Added `outline: none` inline style since the selected row highlight already communicates focus state
- Guarded auto-focus with `document.activeElement === document.body` check to avoid stealing focus from search bar or other inputs

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## Self-Check: PASSED
