---
gsd_state_version: 1.0
milestone: v0.9
milestone_name: Multi-tab & Tree View
status: v0.9 milestone complete
stopped_at: Completed 49-02-PLAN.md
last_updated: "2026-03-25T03:39:53.940Z"
last_activity: 2026-03-25
progress:
  total_phases: 6
  completed_phases: 5
  total_plans: 13
  completed_plans: 12
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-23 after v0.9 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** Phase 49 — tab-drag-tree-context-menu

## Current Position

Phase: 49
Plan: Not started

## Performance Metrics

| Metric | v0.1 | v0.2 | v0.3 | v0.4 | v0.5 | v0.6 | v0.7 | v0.8 |
|--------|------|------|------|------|------|------|------|------|
| Phases | 6 | 4 | 4 | 3 | 7 | 5 | 5 | 7 |
| Plans | 27 | 9 | 14 | 5 | 12 | 16 | 8 | 19 |
| Commits | 155 | 76 | 88 | ~30 | 111 | ~129 | -- | 61 |
| Days | 7 | 2 | 3 | 1 | 2 | 2 | 2 | 4 |
| Phase 44 P01 | 8m | 2 tasks | 4 files |
| Phase 45 P01 | 4m | 2 tasks | 7 files |
| Phase 45 P02 | 7m | 2 tasks | 7 files |
| Phase 45 P03 | 5m | 2 tasks | 3 files |
| Phase 46 P01 | 4m | 2 tasks | 2 files |
| Phase 47 P01 | 3min | 2 tasks | 6 files |
| Phase 47 P02 | 4min | 2 tasks | 4 files |
| Phase 48 P01 | 4min | 2 tasks | 2 files |
| Phase 48 P02 | 7min | 2 tasks | 4 files |
| Phase 49 P01 | 2min | 2 tasks | 2 files |
| Phase 49 P02 | 2min | 2 tasks | 3 files |

## Accumulated Context

### Decisions

- Phase 44 scope: Only RunningOp needs multi-repo scoping; RepoState, CommitCache, WatcherState are already HashMap<String, _> keyed by repo path
- Research recommends destroy/recreate over keep-alive for tab switching (simpler, Rust cache makes remount fast)
- Per-repo context scoping (remoteState, undoRedoState) needs design spike in Phase 45 planning
- [Phase 44]: RunningOp uses HashMap<String, u32> keyed by repo path for per-repo remote op isolation
- [Phase 44]: force_close_repo cancels running op via SIGTERM before cleaning state (D-03)
- [Phase 44]: close_repo intentionally does NOT touch RunningOp (D-02 graceful behavior)
- [Phase 45]: $state() must be assigned to variable declaration — factory uses const state = $state({...}); return state
- [Phase 45]: Backward-compat singleton aliases call factory at module scope; consumers compile without changes until Plan 02
- [Phase 45]: App.svelte owns per-tab state creation via getOrCreateTabState; RepoView receives as props, never creates its own
- [Phase 45]: Keep-alive rendering uses display:contents/none for zero-cost hidden tabs per D-08
- [Phase 45]: StagingPanel threads clearRedoStack prop to CommitForm (prop chain for per-tab state)
- [Phase 45]: TabBar uses div[role=tab] instead of nested buttons for HTML validation (button-in-button forbidden)
- [Phase 45]: Dirty detection via repo-changed watcher (no polling) with staged+unstaged>0 threshold and initial check on restore
- [Phase 46]: Trie-based algorithm for O(n) flat-to-tree conversion with path compression
- [Phase 46]: Compression guard checks child type === directory to avoid collapsing dir with single file child
- [Phase 47]: flattenTree uses recursive approach matching buildTree, producing FlatRow[] in one pass
- [Phase 47]: findFocusIndex returns 0 (not -1) when path not found for safe array indexing
- [Phase 47]: color-tree-focus reuses color-selected-row via CSS custom property reference
- [Phase 47]: TreeFileList owns expanded Set and focusIndex per-instance for per-section state
- [Phase 47]: CommitDetail uses DIFF_STATUS_MAP adapter to bridge FileDiff and FileStatus types for unified tree rendering
- [Phase 48]: Dynamic imports for @tauri-apps/api/menu and clipboard-manager in App.svelte for on-demand tab context menu
- [Phase 48]: Duplicate tab detection normalizes trailing slashes before repo path comparison
- [Phase 48]: Directory staging uses flat file list prefix matching rather than tree traversal
- [Phase 48]: Expand/collapse uses signal counter pattern (increment  counter, child  detects change)
- [Phase 49]: Reused SortableJS pattern from RebaseEditor with horizontal direction for tab drag reorder
- [Phase 49]: filter: '.new-tab-btn' with preventOnFilter: false excludes + button from drag while keeping it clickable
- [Phase 49]: Directory context menus use native Tauri menus with file count display matching existing file context menu pattern

### Pending Todos

1 pending todo.

- Fix merge commit line bend direction (ui)

### Known Limitations

- SSH_AUTH_SOCK absent when app launched from Finder (not `cargo tauri dev`). Documented as known limitation.

### Blockers/Concerns

(None)

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260320-wz2 | Simplify splash screen project cards to single-line items, save 10 projects instead of 5 | 2026-03-21 | ca345bf | [260320-wz2-simplify-splash-screen-project-cards-to-](./quick/260320-wz2-simplify-splash-screen-project-cards-to-/) |
| 260325-j3y | Remove left tab offset when window is maximized/fullscreen | 2026-03-25 | 13597f5 | [260325-j3y-remove-left-tab-offset-when-window-is-ma](./quick/260325-j3y-remove-left-tab-offset-when-window-is-ma/) |
| 260325-jb3 | Prevent Ctrl+Cmd+F from being captured by Cmd+F search handler | 2026-03-25 | 8b3dcd9 | [260325-jb3-prevent-ctrl-cmd-f-from-being-captured-b](./quick/260325-jb3-prevent-ctrl-cmd-f-from-being-captured-b/) |
| 260325-lkj | Fix graph column width too wide for linear repos (auto-fit to content) | 2026-03-25 | 51add5e | [260325-lkj-fix-graph-column-width-too-wide-for-line](./quick/260325-lkj-fix-graph-column-width-too-wide-for-line/) |

## Session Continuity

Last activity: 2026-03-25
Last session: 2026-03-25T18:47:02Z
Stopped at: Completed quick task 260325-lkj
Resume file: None
Next action: Re-verify TREE-01, TREE-02, TREE-05, TREE-06
