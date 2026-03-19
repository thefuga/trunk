---
gsd_state_version: 1.0
milestone: v0.7
milestone_name: Hunk Staging & Search
status: unknown
stopped_at: Completed 36-01-PLAN.md
last_updated: "2026-03-19T02:40:31.026Z"
progress:
  total_phases: 5
  completed_phases: 4
  total_plans: 8
  completed_plans: 7
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-17 after v0.7 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** Phase 36 — search-ui

## Current Position

Phase: 36 (search-ui) — EXECUTING
Plan: 2 of 2

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
| Phase 32 P01 | 4min | 3 tasks | 2 files |
| Phase 33 P01 | 2min | 2 tasks | 2 files |
| Phase 34 P01 | 11min | 3 tasks | 2 files |
| Phase 34 P02 | 2min | 2 tasks | 2 files |
| Phase 34 P03 | 1min | 1 tasks | 1 files |
| Phase 35-search-backend P01 | 2min | 2 tasks | 4 files |
| Phase 36-search-ui P01 | 5min | 2 tasks | 2 files |

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
- [Phase 32]: Used ApplyOptions::hunk_callback instead of Patch::from_diff + to_buf roundtrip -- simpler, fewer allocations
- [Phase 32]: Kept hunk commands in staging.rs rather than new file -- shares helpers, natural grouping
- [Phase 33]: No new dependencies -- all patterns reused from existing codebase (safeInvoke, showToast, ask())
- [Phase 33]: Single hunkOperationInFlight boolean disables ALL hunk buttons to prevent stale-index races
- [Phase 34]: Used forward diff + reverse patch construction instead of git2 .reverse(true) -- ensures line indices from frontend match the diff the user sees
- [Phase 34]: Single build_partial_patch_text with reverse flag instead of separate forward/reverse builders -- less code duplication
- [Phase 34]: No new dependencies -- all patterns reused from existing codebase (safeInvoke, showToast, ask(), CSS custom properties)
- [Phase 34]: Selection state uses Set<number> reassigned on mutation for Svelte 5 reactivity
- [Phase 34]: onmousedown handler placed before onclick on same div -- mousedown fires before click, preventing text selection before it starts
- [Phase 35-search-backend]: Pure in-memory scan over CommitCache — no spawn_blocking needed
- [Phase 35-search-backend]: search_commits_inner as testable pure fn, search_commits as thin Tauri wrapper
- [Phase 36-search-ui]: SearchBar is a pure presentation component -- no IPC calls, parent manages all state
- [Phase 36-search-ui]: Cmd+F uses capture:true addEventListener to intercept before WebView native find
- [Phase 36-search-ui]: Search navigation (Enter/Shift+Enter) both scrolls and selects the commit

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

Last session: 2026-03-19T02:40:18.840Z
Stopped at: Completed 36-01-PLAN.md
Resume file: None
Next action: Execute Phase 34 Plan 02 (line-level staging frontend)
