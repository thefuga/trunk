# Phase 43: Tech Debt Cleanup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 43-tech-debt-cleanup
**Areas discussed:** (auto mode — no interactive discussion)

---

## Auto Mode

This phase was processed with `--auto` mode. No gray areas were identified requiring user input — the phase scope is entirely defined by the ROADMAP.md success criteria (5 specific cleanup items).

### Codebase verification performed:
- `diff_conflicted`: confirmed present in `lib.rs` (registration) and `commands/diff.rs` (implementation + tests)
- `InputDialog` dead import: confirmed at `App.svelte:11`, component used elsewhere (CommitGraph, Toolbar, BranchSidebar)
- `rebaseBaseName` lookup: confirmed dead at `App.svelte:411-416` — `find()` always returns false
- `submit_rebase_message`: confirmed already cleaned — no references in src-tauri/src/
- Type mismatch: confirmed `'conflicted'` kind can flow to DiffPanel's `diffKind` prop which doesn't include it in its type union

## Claude's Discretion

- Approach for fixing `rebaseBaseName` lookup
- Type narrowing strategy for DiffPanel diffKind

## Deferred Ideas

None — phase is pure cleanup with defined scope.
