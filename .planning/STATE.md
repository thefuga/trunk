---
gsd_state_version: 1.0
milestone: v0.6
milestone_name: UI Polish & Core Ops
status: executing
stopped_at: Completed 28-01-PLAN.md
last_updated: "2026-03-15T20:14:22Z"
last_activity: 2026-03-15 - Completed 28-01 backend destructive operations
progress:
  total_phases: 6
  completed_phases: 2
  total_plans: 6
  completed_plans: 6
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-15 after v0.6 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** v0.6 UI Polish & Core Ops

## Current Position

Phase: 28 — Destructive Operations
Plan: 1 of 3 complete
Status: Executing phase 28
Last activity: 2026-03-15 - Completed 28-01 backend destructive operations

```
v0.6 Progress: [░░░░░░░░░░░░░░░░░░░░] 0/5 phases
```

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

### Roadmap Evolution

- Phase 27.1 inserted after Phase 27: Add icons to commit graph pills (URGENT)

### Pending Todos

4 pending todos carried from v0.2:
1. **Make commit dot bigger and lanes thinner** (ui) — 2026-03-10
2. **WIP HEAD row background covers dotted line on hover** (ui) — 2026-03-10
3. **Second commit connector line disconnected from first commit** (ui) — 2026-03-10
4. **Persist left and right pane open/close state** (ui) — 2026-03-10

### Known Limitations

- SSH_AUTH_SOCK absent when app launched from Finder (not `cargo tauri dev`). Documented as known limitation.

### Blockers/Concerns

(None)

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 9 | Revert commit graph row height to more packed style, keep lane width, increase commit dot radius by 2px | 2026-03-14 | 901f73a | [9-revert-commit-graph-row-height-to-more-p](./quick/9-revert-commit-graph-row-height-to-more-p/) |
| 10 | Make the pill connector line thinner (1.5px → 1px) | 2026-03-15 | a2b7b95 | [10-make-the-pill-line-thinner](./quick/10-make-the-pill-line-thinner/) |

## Session Continuity

Last session: 2026-03-15T20:14:22Z
Stopped at: Completed 28-01-PLAN.md
Resume file: None
Next action: Execute 28-02-PLAN.md
