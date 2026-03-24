---
phase: 45-frontend-tab-architecture
plan: 03
subsystem: ui
tags: [svelte5, tab-bar, dirty-detection, css-custom-properties, multi-tab]

requires:
  - phase: 45-frontend-tab-architecture
    provides: App.svelte tab manager with keep-alive, RepoView, tab CRUD, keyboard shortcuts, persistence

provides:
  - Multi-tab TabBar.svelte with dirty dots, close buttons, + new tab button, horizontal scroll
  - Dirty detection via repo-changed watcher events updating tab.dirty per D-04/D-05
  - --color-tab-hover CSS custom property for tab hover states
  - Initial dirty check on tab restore for accurate state on app launch

affects: [48-polish-differentiators, tab-context-menu]

tech-stack:
  added: []
  patterns: [css-custom-property-for-all-colors, role-tab-accessibility, scroll-into-view-on-active-change]

key-files:
  created: []
  modified:
    - src/components/TabBar.svelte
    - src/App.svelte
    - src/app.css

key-decisions:
  - "TabBar uses div[role=tab] instead of nested buttons to satisfy HTML validation (button-in-button forbidden)"
  - "Dirty detection via repo-changed watcher events (D-05: no polling) with staged+unstaged>0 threshold (D-04)"
  - "Initial dirty check on tab restore ensures accurate dirty state on app launch"

patterns-established:
  - "All color values via CSS custom properties (--color-tab-hover for hover, --color-accent for dirty dot)"
  - "Programmatic scrollIntoView on activeTabId change for overflow tab bars"

requirements-completed: [TAB-01, TAB-03, TAB-04, TAB-06, TAB-07]

duration: 5min
completed: 2026-03-24
---

# Phase 45 Plan 03: TabBar Rewrite & Dirty Detection Summary

**Multi-tab TabBar.svelte with 144 lines: dirty indicator dots, close buttons, + new tab button, horizontal scroll, and repo-changed watcher-driven dirty detection in App.svelte**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-24T03:55:00Z
- **Completed:** 2026-03-24T04:00:26Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Rewrote TabBar.svelte (144 lines) as full multi-tab component: renders all tabs with dirty dots, close buttons (Shift+click for force close), + new tab button, hidden scrollbar overflow
- Added --color-tab-hover CSS custom property to app.css (no inline colors per CLAUDE.md rules)
- Wired repo-changed listener in App.svelte for tab dirty detection: updates tab.dirty = staged+unstaged > 0 per D-04/D-05
- Added initial dirty check on tab restore so dirty state is accurate on app launch
- Human-verified all 7 TAB requirements (TAB-01 through TAB-07) working correctly

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewrite TabBar.svelte, add hover token, and wire dirty detection in App.svelte** - `ef196de` (feat)
2. **Task 2: Verify complete tab experience** - checkpoint:human-verify (approved)

## Files Created/Modified
- `src/components/TabBar.svelte` - Rewritten: multi-tab rendering with Props (tabs, activeTabId, onactivate, onclose, onnew), dirty-dot class, close-btn, new-tab-btn, scroll-into-view effect
- `src/App.svelte` - Updated: repo-changed listener for dirty detection, initial dirty check on restore, TabBar integration with full props
- `src/app.css` - Added: --color-tab-hover CSS custom property

## Decisions Made
- Used `div[role=tab]` instead of nested `<button>` elements to avoid HTML validation error (button-in-button is forbidden). Close button is a separate `<button>` inside the tab div.
- Dirty detection threshold: `staged + unstaged > 0` (conflicted files tracked separately per D-04).
- Dirty detection via watcher events only (no polling) per D-05, plus initial check on tab restore for accurate state on launch.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None -- all data paths are fully wired. Dirty detection connected to backend get_dirty_counts. No placeholder text or empty values.

## Next Phase Readiness
- Phase 45 (Frontend Tab Architecture) is now complete: all 7 TAB requirements verified
- Phase 46 (Tree View Data Layer) can proceed independently
- Phase 48 (Polish & Differentiators) can build on this tab architecture for context menu, middle-click close, duplicate detection

## Self-Check: PASSED

- All modified files exist (TabBar.svelte, App.svelte, app.css)
- Task 1 commit found (ef196de)
- Summary file created (45-03-SUMMARY.md)
- Task 2 checkpoint approved by user

---
*Phase: 45-frontend-tab-architecture*
*Completed: 2026-03-24*
