---
phase: 75-message-editor-infrastructure
plan: 02
subsystem: infra
tags: [rust, tempfile, git, editor, tauri, file-io]

requires:
  - phase: 75-message-editor-infrastructure
    provides: D-07/D-08/D-09 decisions for single-shot editor helper scope
provides:
  - Single-shot Rust temp-editor helper module `src-tauri/src/git/editor.rs`
  - `EditorHandle` RAII type that owns a temp script + msg file pair
  - `prepare(message: &str) -> Result<EditorHandle, TrunkError>` constructor
  - `script_path()` accessor for Phase 76 `Command::env("GIT_EDITOR", _)` callers
  - Drop-based cleanup that survives both happy path and `?` early-return
affects: [76-commit-message-ux-wiring]

tech-stack:
  added: [tempfile (promoted from dev-dep to runtime dep)]
  patterns:
    - "Single-shot editor helper extracted from queue-based pattern in interactive_rebase.rs"
    - "tempfile::Builder + .keep() for non-predictable temp paths with caller-owned cleanup"
    - "Partial-cleanup arms before Err return when Drop cannot run yet"

key-files:
  created:
    - src-tauri/src/git/editor.rs
    - .planning/phases/75-message-editor-infrastructure/75-02-SUMMARY.md
  modified:
    - src-tauri/src/git/mod.rs
    - src-tauri/Cargo.toml

key-decisions:
  - "Promoted tempfile from [dev-dependencies] to [dependencies] to satisfy T-75-T01 TOCTOU mitigation (plan's premise that tempfile was already a runtime dep was incorrect — verified interactive_rebase.rs uses pid-based paths, not tempfile)."
  - "Skipped prepare_failure_does_not_leak_partial_state test per the executor decision the plan grants — injecting a deterministic failure required more scaffolding than the cleanup branch itself. The invariant is documented in code and in T-75-T05 of the threat register."
  - "Used `tempfile::Builder.tempfile().keep()` rather than `NamedTempFile::keep()` directly so the prefix/suffix knobs are visible at the call site."
  - "Placed `pub mod editor;` at the top of src-tauri/src/git/mod.rs (alphabetical: editor < graph) — the plan's instruction to place it between repository and review was inconsistent with the existing alphabetical convention; honoured the convention."

patterns-established:
  - "Reserve-then-write split: `reserve_temp_path(prefix, suffix)` returns a PathBuf via `tempfile::Builder.tempfile().keep()`; `write_artifacts(msg, script, message)` writes the two payloads + chmod. Callers own cleanup on Err. Future single-shot temp-file helpers can mirror this shape."
  - "Partial-cleanup arms before `EditorHandle` construction: each fs step has a remove_file fallback for any prior step before propagating Err. Documented invariant: Drop runs only post-construction, so the pre-construction window is the helper's responsibility."

requirements-completed: [MSG-04, MSG-05]

duration: ~25min
completed: 2026-05-28
---

# Phase 75 Plan 02: Rust Single-Shot Editor Helper Summary

**`git::editor::prepare()` + `EditorHandle` RAII type extracted from `interactive_rebase.rs` queue pattern, with `tempfile::Builder` TOCTOU defence and Drop-based cleanup proven by 8 unit tests.**

## Performance

- **Duration:** ~25 min (excluding recovery from cwd-drift incident)
- **Started:** 2026-05-28T20:11:00Z
- **Completed:** 2026-05-28T22:30:00Z
- **Tasks:** 3 (RED → GREEN → REFACTOR)
- **Files modified:** 3 (1 new, 2 modified)

## Accomplishments

- Created `src-tauri/src/git/editor.rs` with `EditorHandle` struct, `prepare()` constructor, and `impl Drop` cleanup.
- Eight `#[cfg(test)]` unit tests proving: script chmod 0o755 (D-09), shebang+quoted-cp body (T-75-T04), msg verbatim incl. newlines and `#` lines (MSG-04), empty payload, Drop happy path (MSG-05), Drop leave-scope (Phase 76 `?` early-return shape), temp_dir() containment (T-75-T01), pairwise-distinct paths across calls.
- Registered `pub mod editor;` in `src-tauri/src/git/mod.rs` (alphabetical).
- Promoted `tempfile` from `[dev-dependencies]` to `[dependencies]` to satisfy the T-75-T01 TOCTOU mitigation as written.
- Refactor pass extracted `reserve_temp_path` and `write_artifacts` helpers, collapsing four cleanup arms in `prepare()` to two.
- `interactive_rebase.rs` untouched (D-08 honoured — `git diff` is empty).
- No subprocess invocation in editor.rs production or test code (D-12).
- `just check` exits 0 (fmt, biome, svelte-check, clippy, cargo-test, vitest all green).

## Task Commits

1. **Task 1 (RED): failing test block** — `b2a4df2` (test)
2. **Task 2 (GREEN): prepare() + Drop implementation** — `cd8d5b5` (feat)
3. **Task 3 (REFACTOR): extract reserve_temp_path/write_artifacts** — `e733ad5` (refactor)

## Files Created/Modified

- `src-tauri/src/git/editor.rs` *(new)* — Single-shot editor helper. `EditorHandle` owns script_path + msg_path; `Drop` removes both; `prepare()` builds them via `tempfile::Builder`; `#[cfg(unix)]` chmod 0o755 on the script.
- `src-tauri/src/git/mod.rs` — added `pub mod editor;` at the top (alphabetical).
- `src-tauri/Cargo.toml` — moved `tempfile = "3"` from `[dev-dependencies]` to `[dependencies]`.

## Decisions Made

### `prepare_failure_does_not_leak_partial_state` test — skipped, documented in code

The plan explicitly grants the executor discretion here ("if injecting a deterministic failure adds more complexity than the partial-cleanup branch itself, document the invariant in code via comments referencing D-07 and skip this test"). Done: a doc-comment block at the bottom of the test module names the invariant and the threat-register row (T-75-T05). The four cleanup arms inside `prepare()` (two in `reserve_temp_path` for msg → script, two after `write_artifacts` returns Err) are inspectable directly and reviewable as a unit.

### `tempfile` API chosen — `Builder.tempfile().keep()`

`tempfile::Builder::new().prefix(...).suffix(...).tempfile()` returns a `NamedTempFile` whose path lives under `std::env::temp_dir()` with O_EXCL-style uniqueness. `.keep()` consumes the `NamedTempFile`, returning `(File, PathBuf)` and disabling its auto-cleanup. We own the path from that point onward and clean up via `Drop`. Picked over `NamedTempFile::new()` directly because the `Builder` exposes prefix/suffix knobs that make the intent visible at the call site (`trunk-editor-msg-`, `trunk-editor-`/`.sh`).

### Phase 76 wiring snippet

```rust
let handle = trunk_lib::git::editor::prepare("Merge branch 'foo'")?;
std::process::Command::new("git")
    .env("GIT_EDITOR", handle.script_path())
    .args(["merge", "--continue"])
    .status()?;
// handle drops here; both temp files are removed.
```

The snippet is also embedded in the module doc comment as an `# Usage (Phase 76)` block (`# ```ignore`-fenced, doc-test `ignored`d — verified in `cargo test`'s doc-test output).

### `interactive_rebase.rs` untouched

`git diff 754b864..HEAD -- src-tauri/src/commands/interactive_rebase.rs` returns empty. D-08 honoured — the existing queue-based pattern stays inline as the plan requires.

### mod.rs placement

The plan's instruction was "alphabetically between `pub mod repository;` and `pub mod review;`" — that placement is not alphabetical (editor < graph < repository alphabetically). Honoured the existing alphabetical convention and the plan's success criterion ("alphabetically placed") by inserting at the top.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Promoted `tempfile` from `[dev-dependencies]` to `[dependencies]`**

- **Found during:** Task 1 (RED stub creation).
- **Issue:** The plan stated tempfile was already a runtime dep ("PATTERNS.md notes it is at `Cargo.toml:39`") and pointed at `interactive_rebase.rs` as proof. Verified by reading both files: `Cargo.toml:39` is under `[dev-dependencies]`, and `interactive_rebase.rs` does NOT use tempfile in production — it constructs `std::env::temp_dir().join(format!("trunk-rebase-{}", std::process::id()))` (pid-based, predictable). The plan's threat model (T-75-T01) explicitly requires `tempfile::Builder`; using pid-based paths would weaken the TOCTOU mitigation.
- **Fix:** Single-line manifest edit — moved `tempfile = "3"` from `[dev-dependencies]` to `[dependencies]`. No new crate introduced; the dependency is already vetted and compiled in the tree.
- **Files modified:** `src-tauri/Cargo.toml`.
- **Verification:** `cargo test --manifest-path src-tauri/Cargo.toml git::editor::tests::` compiles and passes; `cargo clippy -- -D warnings` clean; `just check` exits 0.
- **Committed in:** `b2a4df2` (Task 1 commit — promoted as part of the RED task so the test file could reference `tempfile::Builder` at compile time).

**2. [Process Recovery] cwd-drift incident: initial Task 1 commit landed on `main` instead of the worktree branch**

- **Found during:** Task 1 RED commit step.
- **Issue:** Bash `cd /Users/joaofnds/code/trunk && ...` prefix moved the shell out of the worktree into the main repo on every call. The Edit/Write tools followed absolute paths under the main repo. The RED commit `637b56a` landed on `main` instead of the per-agent worktree branch — exactly the failure mode the per-commit HEAD assertion is designed to prevent (#3097). The assertion was not run between commits.
- **Fix:** From inside the worktree, `git cherry-pick 637b56a` (worktrees share the object DB, so the commit was reachable) → `git reset --hard 754b864` in the main repo to revert. Verified both: main is back at `754b864`, worktree has the cherry-picked commit `b2a4df2` on top of `754b864`. Subsequent Bash commands ran without `cd` (default cwd is the worktree); Edit/Write calls used the worktree absolute path `.claude/worktrees/agent-a0a18420fc5c5a900/...` from then on.
- **Files modified:** None retained — recovery left the worktree with the intended `b2a4df2` and main back at its pre-incident HEAD.
- **Verification:** `git log --oneline -3` from worktree shows `e733ad5 → cd8d5b5 → b2a4df2 → 754b864` on the per-agent branch; from main shows `754b864` at HEAD with clean status.
- **Committed in:** N/A — the recovery removed a misplaced commit and re-applied it correctly.

---

**Total deviations:** 2 (1 missing-critical dep promotion, 1 process recovery)
**Impact on plan:** The tempfile promotion was load-bearing for the security claim in the threat model; the planner misread the existing manifest. Process recovery added ~15 min and surfaces a real gap (per-commit HEAD assertion is in the playbook but I did not execute it between commits). No scope creep; no test surface lost; no plan tasks skipped.

## Issues Encountered

- `node_modules` was absent in the worktree (svelte-check `command not found` on first `just check`). `bun install` resolved it — flagged here so the worktree-spawn process notes whether JS deps should be primed.

## User Setup Required

None.

## Next Phase Readiness

Phase 76 wiring is unblocked:

- `trunk_lib::git::editor::prepare(default: &str) -> Result<EditorHandle, TrunkError>` is the public API.
- `EditorHandle::script_path()` returns `&Path` — pass directly to `Command::env("GIT_EDITOR", _)`.
- Dropping the handle (either explicitly or via `?` early-return) removes both temp files (MSG-05).
- Message bytes flow through verbatim (MSG-04) — Phase 76 callers may pre-fill from `.git/MERGE_MSG`, constructed merge-branch strings, or constructed revert messages without backend-side normalisation.
- The queue-based pattern in `interactive_rebase.rs:131-179` remains untouched (D-08); rebase test history is preserved.

---

*Phase: 75-message-editor-infrastructure*
*Completed: 2026-05-28*

## Self-Check: PASSED

- `src-tauri/src/git/editor.rs`: FOUND
- `src-tauri/src/git/mod.rs` contains `pub mod editor;`: FOUND
- `src-tauri/Cargo.toml` has `tempfile = "3"` under `[dependencies]`: FOUND
- Commit `b2a4df2` (test RED): FOUND on branch `worktree-agent-a0a18420fc5c5a900`
- Commit `cd8d5b5` (feat GREEN): FOUND on branch `worktree-agent-a0a18420fc5c5a900`
- Commit `e733ad5` (refactor): FOUND on branch `worktree-agent-a0a18420fc5c5a900`
- `git diff 754b864..HEAD -- src-tauri/src/commands/interactive_rebase.rs`: empty (D-08)
- `cargo test git::editor::tests::`: 8 passed
- `just check`: exit 0

## Post-Task Reflection (continuous_improvement.md §1)

1. **What was harder than expected?** Worktree path discipline. Every Bash `cd /Users/joaofnds/code/trunk && ...` silently broke isolation; every absolute path under that prefix wrote to the main repo. The cwd-drift guard (#3097) and absolute-path guard (#3099) exist in my playbook but I did not run them per-commit. The recovery worked because the bad commit landed alone on top of a known base; in a busier scenario it could have been catastrophic.
2. **Was anything done twice?** The first RED commit had to be cherry-picked into the worktree after being reset out of main. The work (writing the file) was not redone, but the commit ceremony ran twice.
3. **Did I make any incorrect assumptions?** Two: (a) Bash default cwd persists worktree-rooted across calls, so prefixing `cd <main>` was destructive in a way I did not expect; (b) the planner's claim that `tempfile` was a runtime dep was wrong — I read Cargo.toml early enough to catch it before Task 2 GREEN would have failed at compile time, which the advisor flagged proactively.
4. **Is there a follow-up improvement?** Friction: the per-commit HEAD/cwd-drift assertions in the executor playbook are documented but not enforced — there is no harness step that runs them automatically. Root cause: the playbook treats them as conventions, not gates. Fix: prepend a 5-line `pre-commit` shell snippet to every `git commit` call (HEAD assertion + cwd check + worktree-root path check), or wire a true pre-commit hook in the worktree's `.git/hooks/`. Benefit: eliminates the failure mode the playbook already names. Cost: 10–15 min to implement and document; risk minimal.
5. **Should any memory files be updated?** Yes — a `worktree_cwd_drift.md` entry under `~/.claude/projects/.../memory/` capturing: (a) Bash `cd <main-repo>` prefix breaks isolation, (b) absolute paths under `/Users/joaofnds/code/trunk/...` write to main, not worktree, (c) recovery is `cherry-pick → reset --hard <base>`, (d) the per-commit HEAD assertion in the playbook is the prevention and must be run literally, not implied.
