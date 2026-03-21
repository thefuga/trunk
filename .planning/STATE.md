---
gsd_state_version: 1.0
milestone: v0.8
milestone_name: Conflict & Rebase
status: unknown
stopped_at: Phase 38 context gathered
last_updated: "2026-03-21T01:05:30.582Z"
progress:
  total_phases: 5
  completed_phases: 1
  total_plans: 3
  completed_plans: 3
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-20 after v0.8 milestone started)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** Phase 37 — conflict-detection-operation-state

## Current Position

Phase: 37 (conflict-detection-operation-state) — COMPLETE
Plan: 3 of 3

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

## Accumulated Context

### Decisions

(-- v0.8 milestone)

- [Phase 37]: Used git2 repo.state() for operation detection instead of manual filesystem checks
- [Phase 37]: Set GIT_EDITOR=true on merge --continue to prevent interactive editor prompts
- [Phase 37]: Conflicted files in dedicated section above unstaged/staged with max-height 40% cap
- [Phase 37]: Read-only diff for conflicted files via diffKind=commit reuse
- [Phase 37]: Abort requires confirmation dialog; Continue and Skip do not
- [Phase 37]: Used diff_tree_to_workdir for conflicted files to bypass stage-0-less index entries

### Pending Todos

1 pending todo.

- Fix merge commit line bend direction (ui)

### Known Limitations

- SSH_AUTH_SOCK absent when app launched from Finder (not `cargo tauri dev`). Documented as known limitation.

### Blockers/Concerns

(None)

## Session Continuity

Last session: 2026-03-21T01:05:30.577Z
Stopped at: Phase 38 context gathered
Resume file: .planning/phases/38-merge-editor/38-CONTEXT.md
Next action: Phase 37 complete -- all 3 plans done
