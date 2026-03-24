---
phase: 45-frontend-tab-architecture
plan: 02
subsystem: ui
tags: [svelte5, tab-architecture, keep-alive, component-extraction, state-management]

requires:
  - phase: 45-frontend-tab-architecture
    provides: Factory functions (createRemoteState, createUndoRedoState), TabInfo/PersistedTab types, tab persistence helpers

provides:
  - RepoView.svelte component with all per-repo state and handlers
  - App.svelte as thin tab manager with keep-alive rendering
  - Tab CRUD (addNewTab, closeTab, forceCloseTab, openRepoInTab)
  - Keyboard shortcuts for tab management (Cmd+T/W/1-9, Ctrl+Tab)
  - Tab persistence with debounced save/restore and legacy migration
  - Per-tab state isolation via getOrCreateTabState factory in App.svelte
  - All singleton consumers (Toolbar, PullDropdown, CommitForm, CommitGraph) accept per-tab state as props

affects: [45-03, tab-bar-multi-tab, dirty-indicator]

tech-stack:
  added: []
  patterns: [keep-alive-display-contents-none, per-tab-state-factory-in-orchestrator, prop-threading-over-singletons]

key-files:
  created:
    - src/components/RepoView.svelte
  modified:
    - src/App.svelte
    - src/components/Toolbar.svelte
    - src/components/PullDropdown.svelte
    - src/components/CommitForm.svelte
    - src/components/CommitGraph.svelte
    - src/components/StagingPanel.svelte

key-decisions:
  - "App.svelte owns per-tab state creation via getOrCreateTabState; RepoView receives as props, never creates its own"
  - "Keep-alive uses display:contents/none pattern for zero-cost hidden tabs per D-08"
  - "Escape key handler moved to RepoView (per-repo scope), tab shortcuts stay in App.svelte (global scope)"
  - "StagingPanel threaded clearRedoStack prop to CommitForm (Rule 2 deviation: missing prop chain)"

patterns-established:
  - "Keep-alive rendering: {#each tabs} with display:{tab.id === activeTabId ? 'contents' : 'none'}"
  - "Per-tab state factory: App.svelte creates RemoteState+UndoRedoManager per tab, passes as props"
  - "Prop threading replaces singleton imports for per-instance state"

requirements-completed: [TAB-01, TAB-02, TAB-03, TAB-05]

duration: 7min
completed: 2026-03-24
---

# Phase 45 Plan 02: RepoView Extraction & Tab Manager Summary

**RepoView.svelte extracted with 563 lines of per-repo state, App.svelte rewritten as 321-line tab orchestrator with keep-alive rendering and Cmd+T/W/1-9 keyboard shortcuts**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-24T03:43:11Z
- **Completed:** 2026-03-24T03:50:31Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Extracted all per-repo state (~30 $state variables, ~15 handler functions) from App.svelte into new RepoView.svelte (563 lines)
- Rewrote App.svelte as thin tab manager (321 lines): tabs[], activeTabId, tab CRUD, keep-alive rendering, keyboard shortcuts, tab persistence with legacy migration
- Updated 5 components (Toolbar, PullDropdown, CommitForm, CommitGraph, StagingPanel) to receive per-tab state as props instead of importing singletons
- Per-tab state isolation: getOrCreateTabState in App.svelte creates RemoteState + UndoRedoManager per tab

## Task Commits

Each task was committed atomically:

1. **Task 1: Create RepoView.svelte and update singleton consumers** - `88d738e` (feat)
2. **Task 2: Rewrite App.svelte as tab manager with keep-alive rendering** - `5f001a8` (feat)

## Files Created/Modified
- `src/components/RepoView.svelte` - New: all per-repo state, handlers, main layout with BranchSidebar/CommitGraph/StagingPanel/DiffPanel/MergeEditor/RebaseEditor/CommitDetail
- `src/App.svelte` - Rewritten: thin tab orchestrator with tabs[], keep-alive, CRUD, keyboard shortcuts, persistence
- `src/components/Toolbar.svelte` - Updated: remoteState + undoRedo received as props, PullDropdown gets remoteState prop
- `src/components/PullDropdown.svelte` - Updated: remoteState received as prop instead of singleton import
- `src/components/CommitForm.svelte` - Updated: clearRedoStack received as prop instead of singleton import
- `src/components/CommitGraph.svelte` - Updated: clearRedoStack received as prop instead of singleton import
- `src/components/StagingPanel.svelte` - Updated: clearRedoStack threaded through to CommitForm

## Decisions Made
- App.svelte owns per-tab state creation (getOrCreateTabState) and passes it as props to Toolbar (in title bar) and RepoView. RepoView never creates RemoteState or UndoRedoManager internally.
- Keep-alive rendering uses `display: contents` (active) / `display: none` (inactive) per D-08 decision.
- Escape key handler for closing diffs moved to RepoView (per-repo context), while tab shortcuts (Cmd+T/W/1-9, Ctrl+Tab) stay in App.svelte (global scope).
- Tab persistence debounced at 500ms per RESEARCH recommendation.
- Legacy migration: auto-converts old `open_repo` store key to tabs format on first load.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Threaded clearRedoStack through StagingPanel**
- **Found during:** Task 1 (Prop threading)
- **Issue:** Plan specified CommitForm receives clearRedoStack as prop, but CommitForm is rendered inside StagingPanel, not directly in RepoView. StagingPanel needed to pass it through.
- **Fix:** Added clearRedoStack prop to StagingPanel's Props interface and threaded it to CommitForm usage.
- **Files modified:** src/components/StagingPanel.svelte
- **Verification:** `bun run check` shows no type errors in StagingPanel or CommitForm
- **Committed in:** 88d738e (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 missing critical prop chain)
**Impact on plan:** Essential for correctness -- without threading, CommitForm would not receive clearRedoStack. No scope creep.

## Issues Encountered
None -- plan executed cleanly after the StagingPanel prop threading deviation.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None -- all data paths are fully wired. No placeholder text or empty values.

## Next Phase Readiness
- RepoView and tab manager architecture ready for Plan 03 (TabBar multi-tab rewrite)
- TabBar currently shows single active tab name -- Plan 03 will rewrite it with full multi-tab UI (scroll, drag, dirty indicators)
- Per-tab state factories and prop threading patterns established for any future component additions

## Self-Check: PASSED

- All created files exist (RepoView.svelte, 45-02-SUMMARY.md)
- All commits found (88d738e, 5f001a8)
- All modified files exist (App.svelte, Toolbar.svelte, PullDropdown.svelte, CommitForm.svelte, CommitGraph.svelte, StagingPanel.svelte)
- `bun run test`: 139 tests passed
- No new errors in changed files from `bun run check`

---
*Phase: 45-frontend-tab-architecture*
*Completed: 2026-03-24*
