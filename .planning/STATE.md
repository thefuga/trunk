---
gsd_state_version: 1.0
milestone: v0.10
milestone_name: CI/CD & Releases
status: Ready to plan
stopped_at: Completed 51-01-PLAN.md
last_updated: "2026-03-25T23:58:05.791Z"
last_activity: 2026-03-25
progress:
  total_phases: 3
  completed_phases: 2
  total_plans: 3
  completed_plans: 3
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-25 after v0.10 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** Phase 51 — cross-platform-release-pipeline

## Current Position

Phase: 52
Plan: Not started

## Performance Metrics

| Metric | v0.1 | v0.2 | v0.3 | v0.4 | v0.5 | v0.6 | v0.7 | v0.8 | v0.9 |
|--------|------|------|------|------|------|------|------|------|------|
| Phases | 6 | 4 | 4 | 3 | 7 | 5 | 5 | 7 | 6 |
| Plans | 27 | 9 | 14 | 5 | 12 | 16 | 8 | 19 | 13 |
| Days | 7 | 2 | 3 | 1 | 2 | 2 | 2 | 4 | 3 |
| Phase 50 P01 | 20min | 2 tasks | 96 files |
| Phase 50 P02 | 1min | 1 tasks | 1 files |
| Phase 51 P01 | 2min | 2 tasks | 1 files |

## Accumulated Context

### Decisions

- v0.10 scope: No automated release creation -- artifacts uploaded as workflow artifacts, release created manually
- v0.10 scope: No changelog tooling -- release notes written manually with GSD context
- v0.10 scope: Prettier needs initial format pass before CI can enforce it
- Research: ubuntu-22.04 for Linux, setup-bun@v2 required, rust-cache@v2 for caching, tauri-action@v0 for builds
- Research: Homebrew cask to joaofnds/homebrew-tap for macOS distribution
- [Phase 50]: Biome v2 config uses assist.actions.source.organizeImports (not top-level organizeImports)
- [Phase 50]: Biome scoped to src/ via files.includes; vendored virtual-list excluded from lint/format
- [Phase 50]: Vendored JS uses @ts-nocheck (tsconfig exclude insufficient for imported files)
- [Phase 50]: Clippy alone satisfies both cargo check and cargo clippy requirements (superset)
- [Phase 50]: rust-cache save-if restricted to main branch to prevent PR cache pollution
- [Phase 50]: Two-gate CI pipeline: fast checks (biome, fmt, svelte-check) gate heavy checks (clippy, test, vitest)
- [Phase 51]: macos-15-intel replaces deprecated macos-13 for Intel builds
- [Phase 51]: Build-only tauri-action (no tagName/releaseName) for artifact-only workflow
- [Phase 51]: rust-cache save-if on tag pushes for infrequent release builds

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
Last session: 2026-03-25T23:54:20.444Z
Stopped at: Completed 51-01-PLAN.md
Resume file: None
Next action: /gsd:plan-phase 50
