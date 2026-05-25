---
phase: 66-commit-selection
plan: 01
subsystem: api
tags: [git2, revwalk, rust, review-session, set-semantics, tdd]

# Dependency graph
requires:
  - phase: 65-review-session-keystone
    provides: "ReviewSession.commits: Vec<String> schema, review_store atomic persistence, review.rs command/_inner pattern, GraphResult/GraphCommit DTOs"
provides:
  - "Pure selection-core helpers in review.rs: validate_range, compute_range_oids, apply_add, apply_remove, union_dedup, intersect_graph_order"
  - "SessionCommit Serialize struct (oid/short_oid/summary) — the frozen frontend interface shape"
  - "In-process git2 test-repo helper (linear chain + merge + unrelated root) reusable by future Rust tests in review.rs"
affects: [66-02-command-wiring, 66-03-frontend-types, 67-anchor-capture]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure git2 helpers take &git2::Repository (no Tauri State/AppHandle) so logic is unit-testable against an in-process repo"
    - "In-process test repo via tempfile::TempDir + Repository::init with empty-tree commits and a deterministic zero-time signature"
    - "Set-as-list semantics: store order irrelevant on disk; graph order re-imposed at read by intersect_graph_order"

key-files:
  created: []
  modified:
    - "src-tauri/src/commands/review.rs — selection core helpers + SessionCommit struct + #[cfg(test)] test-repo helper and 11 unit tests"

key-decisions:
  - "validate_range orders base==tip early-return BEFORE graph_descendant_of (which returns false for x,x) so the inclusive {base} case is valid"
  - "compute_range_oids guards parent_count()>0 before parent_id(0) so a root-commit base hides nothing instead of panicking (interactive_rebase.rs fallback)"
  - "No is_merge gate in either range helper (D-08): merge commits are selectable as base or tip"
  - "intersect_graph_order appends find_commit fallbacks and uses '(unavailable)' for truly-unresolvable OIDs — never silently drops a selected commit (SEL-04)"
  - "SessionCommit defined inline in review.rs (not types.rs) with Serialize-default snake_case to match GraphCommit"

patterns-established:
  - "Pure-helper testability wedge: separate set/walk logic (no Tauri state) from the mutex+IO orchestration deferred to Plan 02"
  - "Reusable in-process git2 test-repo fixture for revwalk/validation tests"

requirements-completed: [SEL-01, SEL-02, SEL-03, SEL-04]

# Metrics
duration: ~25min
completed: 2026-05-25
---

# Phase 66 Plan 01: Selection Core (Range Walk, Set Semantics, Graph-Ordered List) Summary

**Pure, unit-tested Rust selection core for review-session commit sets: git2 revwalk range computation with inclusive [base..tip] + root/merge handling, invalid-range validation (bad_range/unrelated_history), set union/add/remove/dedup, and a graph-ordered intersection that never drops a selected commit.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-25T14:34Z (approx)
- **Completed:** 2026-05-25
- **Tasks:** 3 (all TDD: RED → GREEN)
- **Files modified:** 1

## Accomplishments
- `validate_range` + `compute_range_oids`: inclusive [base..tip] via revwalk push(tip)/hide(base.parent), root-commit fallback (no panic on parentless base), base==tip valid, non-ancestor → `bad_range`, unrelated history → `unrelated_history`, merge commits selectable (D-08).
- `apply_add` (idempotent), `apply_remove` (exact + safe-on-missing), `union_dedup` (HashSet union preserving hand-picked commits) — D-03 set semantics.
- `intersect_graph_order` + `SessionCommit` struct: session set intersected with the full cached graph order, deduped, with a `find_commit` fallback that appends orphans and includes truly-unresolvable OIDs as `(unavailable)` (SEL-04 never-drop).
- In-process git2 test-repo helper (linear A→B→C→D + side branch + merge + a separate unrelated-root repo) backing 11 passing unit tests.

## Task Commits

Each task followed strict TDD (RED test commit → GREEN impl commit):

1. **Task 1: Range walk + validation** - `ff04c61` (test) → `85bc10f` (feat)
2. **Task 2: Set union/add/remove/dedup** - `ced9aab` (test) → `3ae2a5e` (feat)
3. **Task 3: Graph-ordered intersection (SEL-04)** - `659aa59` (test) → `6f0f08a` (feat)

No REFACTOR commits were needed — helpers were minimal and clean on first GREEN. rustfmt formatting was folded into the Task 3 GREEN commit (it reformatted the new helper bodies project-wide-consistently; tests stayed green).

## Files Created/Modified
- `src-tauri/src/commands/review.rs` — Added `SessionCommit` struct; the pure helpers `validate_range`, `compute_range_oids`, `apply_add`, `apply_remove`, `union_dedup`, `intersect_graph_order`; and a `#[cfg(test)]` in-process test-repo helper with 11 selection unit tests.

## Decisions Made
- Followed the plan's prescribed helper signatures and error codes exactly (`bad_range`, `unrelated_history`).
- `base == tip` short-circuits to `Ok(())` before the descendant check because git2's `graph_descendant_of(x, x)` is `false` — without the early return the valid single-commit range would be misclassified as a bad range.
- The orphan fallback in `intersect_graph_order` covers two branches in one test: a selected OID absent from the graph but resolvable via `find_commit` (appended with its real summary), and the all-zero null OID that even `find_commit` can't resolve (appended with `(unavailable)`).

## Deviations from Plan

None — plan executed exactly as written. All six Task-1 tests, three Task-2 tests, and two Task-3 tests pass; the named acceptance greps (`fn validate_range`/`fn compute_range_oids`, no `is_merge` in the helper bodies, `struct SessionCommit` with snake_case fields) all hold; helpers take `&git2::Repository` only (no Tauri state).

## Issues Encountered
- **Worktree cwd drift (recovered):** An early `cd /Users/joaofnds/code/trunk/src-tauri` (an absolute path to the MAIN repo, not the worktree) caused the first Task-1 RED edit to land in the main checkout. Detected via the per-commit branch assertion (HEAD was `main`, not `worktree-agent-*`). Recovered by capturing the change as a patch, reverting the main-repo file with `git checkout -- <file>` (single-file, sanctioned), and re-applying the patch inside the worktree. All subsequent work used relative paths from the worktree cwd. No commits ever landed on a protected branch; no work was lost.

## Deferred Issues
- **Pre-existing clippy warnings in `tests/test_graph.rs`** (2: "returning the result of a `let` binding from a block", "unneeded late initialization"). These are in an integration-test file I did not touch and predate this plan (last modified in `5aa735d`). Out of scope per the scope boundary — not fixed here. The lib (`cargo clippy --lib`) is warning-free, including all new helpers.

## Verification
- `cd src-tauri && cargo test --lib review` → 15 passed, 0 failed (11 new selection tests + 4 pre-existing merge-status tests).
- `cd src-tauri && cargo clippy --lib` → no warnings on the new helpers.
- `cargo fmt --check` → clean (applied in the Task 3 GREEN commit).

## Next Phase Readiness
- Plan 02 can now wrap these pure helpers in `#[tauri::command]`s (`seed_review_range`, `add_review_commit`, `remove_review_commit`, `list_session_commits`) with the `ReviewSessionsState` mutex + `review_store::save_session` RMW orchestration (Pitfall 2).
- `SessionCommit`'s field shape (`oid`/`short_oid`/`summary`) is frozen for Plan 03's `src/lib/types.ts` interface.
- Note for Plan 02: these helpers are currently exercised only by tests, so they may surface `dead_code` warnings until the commands consume them — wiring the commands resolves that.

---
*Phase: 66-commit-selection*
*Completed: 2026-05-25*
