---
gsd_state_version: 1.0
milestone: v0.8
milestone_name: Conflict & Rebase
status: unknown
stopped_at: Phase 41 context gathered
last_updated: "2026-03-21T22:07:05.589Z"
last_activity: 2026-03-21
progress:
  total_phases: 5
  completed_phases: 4
  total_plans: 11
  completed_plans: 11
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-20 after v0.8 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** Phase 40 — rebase-workflow

## Current Position

Phase: 40 (rebase-workflow) — COMPLETE
Plan: 1 of 1 (done)

## Performance Metrics

| Metric | v0.1 | v0.2 | v0.3 | v0.4 | v0.5 | v0.6 | v0.7 |
|--------|------|------|------|------|------|------|------|
| Phases | 6 | 4 | 4 | 3 | 7 | 5 | 5 |
| Plans | 27 | 9 | 14 | 5 | 12 | 16 | 8 |
| Commits | 155 | 76 | 88 | ~30 | 111 | ~129 | -- |
| Days | 7 | 2 | 3 | 1 | 2 | 2 | 2 |
| Phase 37 P01 | 4min | 2 tasks | 4 files |
| Phase 37 P02 | 4min | 3 tasks | 6 files |
| Phase 37 P03 | 5min | 2 tasks | 3 files |
| Phase 38 P02 | 2min | 1 tasks | 2 files |
| Phase 38 P01 | 5min | 2 tasks | 5 files |
| Phase 38 P03 | 2min | 2 tasks | 2 files |
| Phase 38 P04 | 2min | 2 tasks | 2 files |
| Phase 38 P05 | 1min | 1 tasks | 1 files |
| Phase 38 P06 | 1min | 1 tasks | 1 files |
| Phase 39 P01 | 3min | 2 tasks | 3 files |
| Phase 40 P01 | 2min | 2 tasks | 2 files |

## Accumulated Context

### Decisions

(-- v0.8 milestone)

- [Phase 37]: Used git2 repo.state() for operation detection instead of manual filesystem checks
- [Phase 37]: Set GIT_EDITOR=true on merge --continue to prevent interactive editor prompts
- [Phase 37]: Conflicted files in dedicated section above unstaged/staged with max-height 40% cap
- [Phase 37]: Read-only diff for conflicted files via diffKind=commit reuse
- [Phase 37]: Abort requires confirmation dialog; Continue and Skip do not
- [Phase 37]: Used diff_tree_to_workdir for conflicted files to bypass stage-0-less index entries
- [Phase 38]: Simple sequential scan with sync-point search for three-way comparison instead of full LCS/Myers diff
- [Phase 38]: Immutable Set<string> with ours/theirs-regionIdx-lineIdx keys for selection state
- [Phase 38]: Used git2 Index::conflicts() iterator instead of non-existent conflict_get() -- git2 0.19 only exposes the iterator API for conflict entry lookup
- [Phase 38]: Output textarea (not contenteditable) for reliable plain-text merge editing
- [Phase 38]: Scroll sync via guard-flag + requestAnimationFrame to prevent feedback loops
- [Phase 38]: Removed diff_conflicted from App.svelte since MergeEditor loads its own data via get_merge_sides
- [Phase 38]: handleFileResolved calls clearStagingDiff to return center pane to CommitGraph after conflict resolution
- [Phase 38]: Changed panelRefs type from HTMLDivElement[] to HTMLElement[] to accommodate textarea in scroll sync
- [Phase 38]: Query get_status after resolution to find remaining conflicts rather than tracking conflict list in local state
- [Phase 39]: No success toast on merge -- graph refresh via repo-changed event is sufficient feedback
- [Phase 39]: Remote branch context menu is single-item (merge only) since checkout is handled via click
- [Phase 39]: Detached HEAD hides merge items entirely rather than showing disabled items
- [Phase 40]: No success toast on rebase -- graph refresh via repo-changed event is sufficient (matches merge pattern)
- [Phase 40]: Rebase items always adjacent to merge items in context menus, no separator between them

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

## Session Continuity

Last activity: 2026-03-21
Last session: 2026-03-21T22:07:05.585Z
Stopped at: Phase 41 context gathered
Resume file: .planning/phases/41-interactive-rebase-editor/41-CONTEXT.md
Next action: Phase 40 complete -- all plans done. v0.8 milestone complete.
