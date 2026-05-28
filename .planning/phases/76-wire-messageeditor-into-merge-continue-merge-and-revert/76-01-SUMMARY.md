---
phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
plan: 01
subsystem: api
tags: [tauri, git, git2, subprocess, serde, merge, rust]

# Dependency graph
requires:
  - phase: 75-message-editor
    provides: MessageEditor modal (frozen) that Plan 03 wires merge_branch_begin's Ready{message} into
provides:
  - "MergeBeginResult tagged enum (serde tag=\"kind\", snake_case) — frozen contract Plan 03 discriminates on"
  - "get_merge_message query command (reads .git/MERGE_MSG verbatim, no cache/emit)"
  - "merge_branch_begin two-step command (ff probe -> --no-commit -> tagged result; emits repo-changed on every outcome)"
  - "merge_continue finish path with --cleanup=strip, GIT_EDITOR=true else-branch removed"
  - "First #[cfg(test)] temp-repo suite in operation_state.rs (Wave-0 gap closed)"
affects: [76-02 revert backend, 76-03 frontend merge/revert wiring, 76-04 UAT checkpoint]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "serde internally-tagged enum (#[serde(tag = \"kind\", rename_all = \"snake_case\")]) — first in codebase, frozen IPC discriminant"
    - "begin command wrapper emits repo-changed unconditionally after an all-arm match that extracts the graph from every variant"
    - "extracted *_inner for query commands (get_merge_message_inner) so unit tests bypass tauri State"
    - "temp-repo TDD harness in operation_state.rs (git2 + tempfile, real git subprocess, no mocks)"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/operation_state.rs
    - src-tauri/src/lib.rs
    - src-tauri/tests/common/drivers/operation_state.rs
    - src-tauri/tests/test_operation_state.rs
    - src-tauri/tests/test_integ_workflows.rs

key-decisions:
  - "merge_continue_inner returns Err on message=None instead of the old GIT_EDITOR=true bypass — None is now a contract violation, not a silent fallback"
  - "Conflict detection scans BOTH stdout and stderr — git writes the CONFLICT notice to stdout, not stderr (verified git 2.54.0); the old stderr-only check would have misclassified conflicts as Err"
  - "test commit_file seeds the treebuilder from the first parent's tree (carry-forward) so fixtures are faithful linear history, not implicit deletes"
  - "divergent fixtures force-checkout HEAD so the worktree/index match the object-DB commits before subprocess git merge"

patterns-established:
  - "Pattern: serde tag enum as the frozen frontend discriminant — assert the serialized `kind` string in tests, not just the Rust variant"
  - "Pattern: begin-mutation wrapper emits repo-changed for EVERY outcome (the emit sits after the all-arm graph-extraction match, at the same nesting level)"

requirements-completed: [MSG-01, MSG-02]

# Metrics
duration: 22min
completed: 2026-05-29
---

# Phase 76 Plan 01: Merge-side backend (begin/continue) Summary

**Two-step `merge_branch_begin` (`--ff-only` probe -> `--no-commit` -> `MergeBeginResult` tagged enum) plus a verbatim `get_merge_message` query and a `--cleanup=strip` merge-continue finish path — all proven against temp git repos, with the GIT_EDITOR/`--no-edit` bypasses removed.**

## Performance

- **Duration:** ~22 min
- **Started:** 2026-05-29
- **Completed:** 2026-05-29
- **Tasks:** 2 (TDD: RED + GREEN) + 1 ownership follow-up commit (integration-suite migration)
- **Files modified:** 5

## Accomplishments
- `MergeBeginResult` serde tagged enum — the frozen contract Plan 03's frontend discriminates against (`fast_forwarded` / `conflicts` / `ready`).
- `merge_branch_begin`: `git merge --ff-only` probe distinguishes fast-forward (no editor) from non-ff; on non-ff it runs `--no-commit` and returns `Conflicts` (never an `Err`) or `Ready { message }` with the verbatim `.git/MERGE_MSG`.
- The `merge_branch_begin` async wrapper emits `repo-changed` on **every** variant so a cancelled ff/clean/conflict merge still surfaces the in-progress UI (RESEARCH finding 7 / Pitfall 4) — structurally guaranteed by a single unconditional emit after the all-arm graph-extraction match.
- `get_merge_message` query reads `.git/MERGE_MSG` verbatim (including `# Conflicts:` lines) — no cache insert, no emit.
- `merge_continue` finish commit now uses `--cleanup=strip` so conflicted-merge bodies drop the `# Conflicts:` block; the `GIT_EDITOR=true` else-branch is gone.
- First `#[cfg(test)]` temp-repo suite in `operation_state.rs` (6 tests), closing the Wave-0 gap.

## Task Commits

1. **Task 0: RED — failing tests for merge_branch_begin + get_merge_message** — `4bb0056` (test)
2. **Task 1: GREEN — implement get_merge_message, MergeBeginResult, merge_branch_begin; cleanup=strip; drop GIT_EDITOR; register in lib.rs** — `f5e4c21` (feat)
3. **Ownership follow-up: migrate the `src-tauri/tests/` integration suite to the two-step `merge_branch_begin` semantics** — `166feb1` (test)

_TDD plan: RED commit established the failing compile gate; GREEN commit made all 6 unit tests pass. The follow-up `test` commit repaired the pre-existing integration suite that referenced the renamed `merge_branch_inner` (ownership.md: caused-by-this-change, fixed before done)._

**Plan metadata:** (this docs commit)

## Files Created/Modified
- `src-tauri/src/commands/operation_state.rs` — `MergeBeginResult` enum; `get_merge_message[_inner]` query; `merge_branch_begin[_inner]` (replaces `merge_branch[_inner]`); `merge_continue_inner` `--cleanup=strip` + else-branch removal; `#[cfg(test)]` temp-repo suite.
- `src-tauri/src/lib.rs` — registered `get_merge_message` + `merge_branch_begin`; removed `merge_branch` from the invoke handler list.

## Decisions Made
- **`merge_continue_inner` rejects `message=None` with `Err`** rather than the old `GIT_EDITOR=true` `git merge --continue` fallback. Under the new flow the frontend always supplies `Some` (it aborts on null and never invokes). Making `None` an explicit error deletes the silent-bypass branch (coding-style: "delete the fallback you control").
- **Conflict detection scans stdout AND stderr.** Empirically (git 2.54.0) `git merge --no-commit` writes the `CONFLICT` notice to **stdout**, not stderr; the original `merge_branch_inner` only checked stderr. Surfaced by the RED `merge_branch_begin_conflict_returns_conflicts_not_err` test failing with empty stderr — fixed by scanning both streams.
- **Test `commit_file` carries the first parent's tree forward.** Avoids each commit implicitly deleting every other path (a fresh treebuilder would), keeping the divergent/clean/conflict fixtures faithful to real linear history.

## Deviations from Plan

None affecting scope — the plan was executed as written. Two within-task auto-fixes (Rule 1) were applied to the **test harness**, not the production contract:

### Auto-fixed Issues

**1. [Rule 1 - Bug] Conflict notice read from stdout, not just stderr (+ stale integration tests encoding the old bug)**
- **Found during:** Task 1 (GREEN — `merge_branch_begin_inner`), and again when running the full `just check` gate.
- **Issue:** The restructured inner copied the original `merge_branch_inner`'s stderr-only conflict check. `git merge --no-commit` emits `CONFLICT` on stdout (verified git 2.54.0), so a conflicted merge returned `Err(merge_error, "")` instead of the `Conflicts` variant — two unit tests failed. The pre-existing integration suite (`src-tauri/tests/`) had a `merge_branch_with_conflict_returns_error` test plus two code comments that explicitly **documented this bug as expected behavior**, and its driver referenced the renamed `merge_branch_inner` (compile break).
- **Fix:** Scan both `stdout` and `stderr` (lowercased) for `conflict` before classifying as an error. Migrated the integration driver to `merge_branch_begin` (returns `MergeBeginResult`); rewrote the three affected tests to the two-step begin+continue semantics; **inverted** `merge_branch_with_conflict_returns_error` → `merge_branch_begin_with_conflict_returns_conflicts`; deleted the two stale "known bug" comments.
- **Files modified:** src-tauri/src/commands/operation_state.rs, src-tauri/tests/common/drivers/operation_state.rs, src-tauri/tests/test_operation_state.rs, src-tauri/tests/test_integ_workflows.rs
- **Verification:** `merge_branch_begin_conflict_returns_conflicts_not_err` + the `--cleanup=strip` unit test pass; the full integration suite (14 + 8 tests) passes with the inverted conflict assertion confirming `Conflicts` against the TestContext builder; `just check` green.
- **Committed in:** f5e4c21 (inner fix, GREEN commit) + 166feb1 (integration-suite migration, ownership follow-up)

**2. [Rule 1 - Bug] Test fixtures left the worktree out of sync with HEAD**
- **Found during:** Task 1 (GREEN — fixture setup)
- **Issue:** `commit_file` writes commits through the object DB only; after the divergent fixtures' trailing commit the worktree/index still matched the reset target, so subprocess `git merge` refused with "local changes would be overwritten." Also the redundant force-create of `main` clashed with the unborn-default-branch HEAD.
- **Fix:** Removed the redundant `repo.branch("main", ...)`/`set_head` calls (init.defaultBranch already establishes `main` on first commit); force-checkout HEAD at the end of the divergent fixtures; seed `commit_file`'s treebuilder from the first parent.
- **Files modified:** src-tauri/src/commands/operation_state.rs (test module only)
- **Verification:** All 6 `operation_state` tests pass.
- **Committed in:** f5e4c21 (Task 1 GREEN commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1). One touches production (`merge_branch_begin_inner` stdout-conflict scan, which the old `merge_branch` path also needed); the other is the test harness/fixtures. Migrating the pre-existing integration suite was required by ownership.md (the rename broke its compile) — same root cause as the stdout fix. The frozen `MergeBeginResult` contract and the production command signatures match the plan's `<interfaces>` block exactly.
**Impact on plan:** No scope creep; no contract drift. The stdout-conflict fix is a real correctness improvement, and inverting the stale integration test removed a test that asserted the old wrong behavior.

## Wave-ordering Note (expected, not a defect)

`lib.rs` no longer registers `merge_branch`, but the frontend still calls `invoke("merge_branch", …)` at `CommitGraph.svelte:592` and `BranchSidebar.svelte:397`. These two sites are exactly Plan 03's edit targets (per 76-RESEARCH); they repoint to `merge_branch_begin`. Between this plan and Plan 03 the frontend merge button is wired to an unregistered command — this is the planned wave seam (PLAN Task 1 step 6 makes the removal an explicit decision), not a regression introduced here.

## Issues Encountered
- The two test-harness bugs above (stdout-conflict, dirty-worktree fixtures). Both resolved within Task 1's GREEN cycle; no fix-attempt limit reached.

## Known Stubs
None — this plan ships complete backend behavior. (`editor.rs` remains intentionally unused per D-01a — not imported, not deleted, confirmed by grep: no `editor::prepare`/`EditorHandle` reference.)

## Acceptance Gate Results
- `cargo test --lib operation_state` — 6 passed, 0 failed.
- `grep GIT_EDITOR operation_state.rs` — none.
- `grep no-edit operation_state.rs` — none.
- `grep -c cleanup=strip operation_state.rs` — 4 (commit arg + test assertions).
- `grep 'editor::prepare\|EditorHandle' operation_state.rs` — none.
- `cargo build` — succeeds (no orphan `merge_branch` reference).
- `grep -rn 'merge_branch\b' src-tauri/` (excluding `merge_branch_begin`) — only frontend Svelte callers remain (Plan 03 targets); no surviving Rust reference.
- **`just check` — fully green** (cargo fmt, biome, svelte-check, clippy, all cargo tests incl. integration suites, 566 vitest tests).

## Next Phase Readiness
- `MergeBeginResult` is the frozen contract for Plan 03's frontend (`result.kind === "ready" | "fast_forwarded" | "conflicts"`).
- `get_merge_message` is ready for the StagingPanel merge-continue editor pre-fill.
- The `repo-changed`-on-every-begin emit is structural; its visible effect (banner-after-cancel) is verified by manual UAT in 76-04, not a Rust unit test.
- Plan 02 (revert backend) mirrors this shape: `revert_commit_begin` / `revert_continue` (`--cleanup=strip`) / `revert_abort`, same begin-emit guarantee.

## Self-Check: PASSED
- Commits `4bb0056` (RED) and `f5e4c21` (GREEN) exist in git history.
- `MergeBeginResult`, `merge_branch_begin_inner`, `get_merge_message_inner` present in operation_state.rs.
- `lib.rs` registers `get_merge_message` and `merge_branch_begin`.
- SUMMARY file present on disk.

---
*Phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert*
*Completed: 2026-05-29*
