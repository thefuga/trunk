---
gsd_state_version: 1.0
milestone: v0.12
milestone_name: Better Diffs
status: verifying
stopped_at: Phase 60 context gathered
last_updated: "2026-03-28T22:40:09.822Z"
last_activity: 2026-03-28
progress:
  total_phases: 6
  completed_phases: 1
  total_plans: 2
  completed_plans: 2
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-28 after v0.12 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** Phase 59 — Backend Data Model & Diff Options

## Current Position

Phase: 60
Plan: Not started
Status: Phase complete — ready for verification
Last activity: 2026-03-28

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

| Metric | v0.1 | v0.2 | v0.3 | v0.4 | v0.5 | v0.6 | v0.7 | v0.8 | v0.9 | v0.10 | v0.11 |
|--------|------|------|------|------|------|------|------|------|------|-------|-------|
| Phases | 6 | 4 | 4 | 3 | 7 | 5 | 5 | 7 | 6 | 3 | 6 |
| Plans | 27 | 9 | 14 | 5 | 12 | 16 | 8 | 19 | 13 | 4 | 16 |
| Days | 7 | 2 | 3 | 1 | 2 | 2 | 2 | 4 | 3 | 2 | 2 |
| Phase 59 P01 | 5min | 3 tasks | 7 files |
| Phase 59 P02 | 5min | 3 tasks | 5 files |

## Accumulated Context

### Decisions

- syntect (Rust) chosen over Shiki (JS) for syntax highlighting -- CSS custom properties compliance, no inline styles
- similar crate chosen for word-level diff -- runs on Rust thread pool, purpose-built iter_inline_changes() API
- Whitespace-ignored diffs disable staging (GitHub Desktop pattern) -- never attempt hunk index remapping
- [Phase 59]: Byte offset ranges (u32 start/end) for WordSpan/SyntaxToken enrichment fields -- compact IPC, frontend slices content
- [Phase 59]: 100,000 context_lines cap for show_full_file instead of u32::MAX -- avoids IPC payload issues
- [Phase 59]: DiffLine enrichment fields use snake_case matching Rust Serialize default; DiffRequestOptions uses camelCase matching serde rename_all
- [Phase 59]: Default diff preferences: contextLines=3, ignoreWhitespace=false, showFullFile=false matching Rust defaults

### Pending Todos

None.

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
| 260325-up9 | Add keyboard arrow navigation to commit graph pane | 2026-03-26 | a048c3f | [260325-up9-when-the-commit-graph-pane-has-focus-the](./quick/260325-up9-when-the-commit-graph-pane-has-focus-the/) |

## Session Continuity

Last activity: 2026-03-28
Last session: 2026-03-28T22:40:09.817Z
Stopped at: Phase 60 context gathered
Resume file: .planning/phases/60-word-level-diff/60-CONTEXT.md
Next action: /gsd:plan-phase 59
