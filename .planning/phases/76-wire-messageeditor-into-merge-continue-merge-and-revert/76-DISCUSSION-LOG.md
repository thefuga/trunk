# Phase 76: Wire MessageEditor into merge/continue, merge, and revert - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-28
**Phase:** 76-wire-messageeditor-into-merge-continue-merge-and-revert
**Areas discussed:** Mechanism & editor.rs fate, Cancel state (clean merge/revert), Modal titles & host

---

## Mechanism & editor.rs fate

| Option | Description | Selected |
|--------|-------------|----------|
| Direct/unified | `git X --no-commit` → read .git/MERGE_MSG → `git commit -m msg`. Reuses operation_state.rs:158. Defaults verbatim from git. editor.rs goes unused (flagged as tech-debt removal, not deleted this phase). | ✓ |
| GIT_EDITOR script (editor.rs) | Build default in backend → editor → run git with GIT_EDITOR=<editor.rs script> injecting the message. Honors Phase 75 helper but adds indirection. | |

**User's choice:** Direct/unified.
**Notes:** Driven by a verified scratch-repo finding — `git revert --no-commit` and `git merge --no-commit --no-ff` both set `REVERT_HEAD`/`MERGE_HEAD` (the MSG-06 recoverable states) AND write `.git/MERGE_MSG` with the exact spec'd default messages. `git revert` has no `-m`-for-message flag. Direct approach is simpler, unified across all 3 ops, and the default is more CLI-faithful (git's own MERGE_MSG). Consequence: editor.rs is intentionally unused — must be acknowledged so a reviewer doesn't wire it in.

---

## Cancel state (clean merge/revert)

| Option | Description | Selected |
|--------|-------------|----------|
| Leave recoverable | Repo stays mid-operation (MERGE_HEAD/REVERT_HEAD set, changes staged); recover via OperationBanner. Matches CLI. Uniform with MSG-06 conflict case. | ✓ |
| Auto-abort to clean | On cancel of a clean op, run `git merge --abort`/`git revert --abort` so 'cancel = nothing happened'. More GUI-intuitive but diverges from CLI, needs a second code path. | |

**User's choice:** Leave recoverable.
**Notes:** Milestone goal is to match git's terminal `$EDITOR` behavior; CLI leaves you mid-operation when you abort the editor. One uniform cancel path for both clean and conflict cases.

---

## Modal titles & host

| Option | Description | Selected |
|--------|-------------|----------|
| Per-op titles | "Merge commit message" (both merge ops) / "Revert commit message". Uses Phase 75 `title` prop. | ✓ |
| Single generic title | One "Commit message" title for all three. | |

**User's choice:** Per-op titles.
**Notes:** Host decision (single MessageEditor in RepoView with callbacks threaded to the 4 trigger sites, mirroring `onopenrebaseeditor`) was decided by Claude as a pure codebase-pattern match, not asked.

---

## Claude's Discretion

- **Editor host architecture** — single `MessageEditor` in `RepoView.svelte`, callback prop threaded to the 4 trigger sites (mirrors existing `onopenrebaseeditor`).
- **Sub-mechanics left to the researcher** (mechanism itself is decided): fast-forward detection approach (OQ-1), default-surfacing IPC shape / begin-finish command structure (OQ-2), confirming `git commit -m` clears `REVERT_HEAD`/`MERGE_HEAD` (OQ-3), exact removal of each bypass flag (OQ-4). See CONTEXT.md.

## Deferred Ideas

- Remove `editor.rs` + revert the `tempfile` runtime-dep promotion (dead code after this phase) — future cleanup, not Phase 76.
- Consolidate `interactive_rebase.rs` queue script onto a shared helper (already deferred in Phase 75; moot if editor.rs is removed).
- `commit.template` / commit signing / hook-output streaming / `--no-verify` — deferred per REQUIREMENTS.md v2 + Out of Scope.
</content>
