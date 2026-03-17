---
gsd_state_version: 1.0
milestone: v0.7
milestone_name: Hunk Staging & Search
status: planning
stopped_at: Phase 32 context gathered
last_updated: "2026-03-17T04:46:54.419Z"
last_activity: 2026-03-17 — v0.7 roadmap created (5 phases, 20 requirements)
progress:
  total_phases: 1
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-17 after v0.7 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** v0.7 Hunk Staging & Search

## Current Position

Phase: 32 (Hunk Staging Backend) — not started
Plan: —
Status: Roadmap created, ready to plan Phase 32
Last activity: 2026-03-17 — v0.7 roadmap created (5 phases, 20 requirements)

## Performance Metrics

| Metric | v0.1 | v0.2 | v0.3 | v0.4 | v0.5 | v0.6 |
|--------|------|------|------|------|------|------|
| Phases | 6 | 4 | 4 | 3 | 7 | 5 |
| Plans | 27 | 9 | 14 | 5 | 12 | — |
| Commits | 155 | 76 | 88 | ~30 | 111 | — |
| Days | 7 | 2 | 3 | 1 | 2 | — |
| Phase 27-foundation-icons-toast-bug-fixes P03 | 3min | 2 tasks | 9 files |
| Phase 27-foundation-icons-toast-bug-fixes P04 | 8min | 2 tasks | 2 files |
| Phase 27-foundation-icons-toast-bug-fixes P02 | 6min | 2 tasks | 6 files |
| Phase 27-foundation-icons-toast-bug-fixes P01 | 8 min | 2 tasks | 2 files |
| Phase 27-foundation-icons-toast-bug-fixes P05 | 3min | 1 tasks | 1 files |
| Phase 27.1-add-icons-to-commit-graph-pills P01 | 2min | 2 tasks | 3 files |
| Phase 28-destructive-operations P01 | 4min | 2 tasks | 4 files |
| Phase 28-destructive-operations P02 | 2min | 1 tasks | 2 files |
| Phase 28-destructive-operations P03 | 3min | 2 tasks | 4 files |
| Phase 28-destructive-operations P04 | 1min | 1 tasks | 1 files |
| Phase 29-staging-commit-ux P02 | 2min | 2 tasks | 2 files |
| Phase 29-staging-commit-ux P01 | 2min | 1 tasks | 1 files |

## Accumulated Context

### Decisions

| Decision | Rationale |
|----------|-----------|
| LAYOUT-01 in Phase 30 (not 31) | Right pane auto-open is triggered by ref navigation (GRAPH-03) — natural companion |
| 5 phases (standard granularity) | 20 requirements cluster into 5 natural delivery boundaries; no artificial splits |
| Phase 27 includes bug fixes | FIX-01/FIX-02 are trivial (1-line fixes), high-value, and unblock clean testing |
- [Phase 27-foundation-icons-toast-bug-fixes]: Used @lucide/svelte (Svelte 5 package) not lucide-svelte for Svelte 5 compatibility — Svelte 5 requires @lucide/svelte; lucide-svelte causes SvelteComponent type errors
- [Phase 27-foundation-icons-toast-bug-fixes]: Extracted get_dirty_counts_inner as sync fn for testability (mirrors get_status_inner pattern) — Enables unit tests to call inner fn directly without Tauri async runtime
- [Phase 27-foundation-icons-toast-bug-fixes]: message column resize handle is never guarded by lastVisibleColumn — It targets author column width from the left edge; suppressing it would break author column resizing
- [Phase 27-foundation-icons-toast-bug-fixes]: _resetToasts() helper added to toast store for Vitest test isolation of Svelte 5 module-level state
- [Phase 27-foundation-icons-toast-bug-fixes]: Wave 0 test scaffolds — tests written before implementation for TOAST-01 and FIX-01 — Verifies Nyquist compliance: RED tests exist before Wave 1 implementations
- [Phase 27-foundation-icons-toast-bug-fixes]: Preserved startColumnResize('author', e, true) handler inside the guard — Handler resizes author column from message's right edge; must remain when author is visible
- [Phase 27.1-add-icons-to-commit-graph-pills]: All ref types unconditionally use ICON_WIDTH — removes hasIcon() branching and makes pill widths uniform — Removes conditional branching (Tag/Stash only) in favor of uniform icon handling for all ref types
- [Phase 28-destructive-operations]: discard_file uses git2 checkout for tracked files and std::fs::remove_file for untracked — no git CLI subprocess needed
- [Phase 28-destructive-operations]: discard commands skip cache rebuild + repo-changed emit — FS watcher handles workdir change detection
- [Phase 28-destructive-operations]: Branch/tag mutation commands rebuild graph cache before emitting repo-changed — matches existing create_branch/create_tag pattern
- [Phase 28-destructive-operations]: oncontextmenu prop wired only on unstaged FileRow instances — discard only applies to unstaged changes
- [Phase 28-destructive-operations]: InputDialog gains backward-compatible defaultValue field for pre-filling rename input
- [Phase 28-destructive-operations]: Pill context menus wired on rect, icon g, and text span for full click coverage
- [Phase 28-destructive-operations]: showOverflowRefContextMenu as separate function from showPillContextMenu — uses RefLabel fields instead of OverlayRefPill to avoid union type refactor
- [Phase 29-staging-commit-ux]: Used inline conditional flex: 1 for 50/50 split instead of CSS classes
- [Phase 29-staging-commit-ux]: Mode state as string union instead of boolean flags — cleaner discrimination, extensible
- [Phase 29-staging-commit-ux]: clearRedoStack skipped for stash — stash doesnt modify commit history

### Roadmap Evolution

- Phase 27.1 inserted after Phase 27: Add icons to commit graph pills (URGENT)

### Pending Todos

0 pending todos.

### Known Limitations

- SSH_AUTH_SOCK absent when app launched from Finder (not `cargo tauri dev`). Documented as known limitation.

### Blockers/Concerns

(None)

### Quick Tasks Completed

| # | Description | Date | Commit | Status | Directory |
|---|-------------|------|--------|--------|-----------|
| 9 | Revert commit graph row height to more packed style, keep lane width, increase commit dot radius by 2px | 2026-03-14 | 901f73a | | [9-revert-commit-graph-row-height-to-more-p](./quick/9-revert-commit-graph-row-height-to-more-p/) |
| 10 | Make the pill connector line thinner (1.5px → 1px) | 2026-03-15 | a2b7b95 | | [10-make-the-pill-line-thinner](./quick/10-make-the-pill-line-thinner/) |
| 260316-1j6 | Remove the bottom bar and use the new notification system for state updates | 2026-03-16 | 5b7b6fa | Verified | [260316-1j6-remove-the-bottom-bar-and-use-the-new-no](./quick/260316-1j6-remove-the-bottom-bar-and-use-the-new-no/) |

## Session Continuity

Last session: 2026-03-17T04:46:54.415Z
Stopped at: Phase 32 context gathered
Resume file: .planning/phases/32-hunk-staging-backend/32-CONTEXT.md
Next action: Discuss Phase 29
