---
phase: 66-commit-selection
reviewed: 2026-05-25T13:18:29Z
depth: deep
files_reviewed: 9
files_reviewed_list:
  - src-tauri/src/commands/review.rs
  - src-tauri/src/lib.rs
  - src/app.css
  - src/components/CommitGraph.svelte
  - src/components/CommitRow.svelte
  - src/components/CommitRow.test.ts
  - src/components/ReviewPanel.svelte
  - src/components/ReviewPanel.test.ts
  - src/lib/types.ts
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: issues_found
---

# Phase 66: Code Review Report

**Reviewed:** 2026-05-25T13:18:29Z
**Depth:** deep
**Files Reviewed:** 9
**Status:** issues_found

## Summary

Phase 66 adds a pure Rust selection core (`compute_range_oids` / `validate_range` / set
ops / `intersect_graph_order`) plus four mutex-serialized RMW commands, a `SessionCommit`
TS type, in-graph membership/pending-base markers on `CommitRow`, a minimal in-session
list with per-row remove in `ReviewPanel`, and the two-right-click range gesture in
`CommitGraph`.

The concurrency design is sound — the RMW genuinely holds the `ReviewSessionsState` mutex
across read → mutate → `save_session` → map-write, and the `selection_rmw_serialized` test
proves no lost writes under 50 concurrent adds. The git2-only and theme-CSS-only project
rules are respected; the review menu items correctly omit the `is_merge` gate (D-08).

However there is one **BLOCKER**: `compute_range_oids` only hides `base.parent_id(0)`, so
selecting a **merge commit as the range base** leaks its second-parent side branch into the
selection. This was confirmed empirically — `[M..N]` returns `{M, N, side}` instead of
`{M, N}`. The existing `merge_commit_selectable` test only covers merge-as-tip, so the
suite does not catch it. Four warnings concern event-listener filtering, error swallowing,
and an unvalidated session precondition.

## Critical Issues

### CR-01: Merge commit as range base leaks the second-parent side branch into the selection

**File:** `src-tauri/src/commands/review.rs:197-219` (`compute_range_oids`)
**Issue:**
The walk hides only the first parent of `base`:

```rust
if base_commit.parent_count() > 0 {
    revwalk.hide(base_commit.parent_id(0)?)?;
}
```

When `base` is a merge commit (≥2 parents), only the first-parent ancestry is hidden. The
second (and later) parents' subtrees remain reachable from `tip` and are pulled into the
range, even though they are ancestors of the inclusive base and must be excluded.

This is not theoretical — D-08 explicitly requires merge commits to remain selectable, and
the range gesture lets a merge be picked as base. Verified empirically against the existing
test topology by extending it with `N = commit on top of M (the merge of D and side)`:

- `validate_range(M, N)` passes (`merge_base(M,N)=M`, `N` is a descendant of `M`).
- `compute_range_oids(M, N)` returns `{N, M, side}` — `side` leaked.
- Correct `[M..N]` inclusive set is `{N, M}`.

The probe test failed with:
`LEAK: side branch leaked into [M..N]; full set = [N, M, side]`

Effect: the user seeds a range from a merge base and silently gets extra unrelated commits
in their review session (incorrect selection / data the user did not pick). The existing
`merge_commit_selectable` test (review.rs:771) only exercises merge-as-**tip**, so the gap
is invisible to the suite.

**Fix:** hide every parent of the base, not just `parent_id(0)`:

```rust
let base_commit = repo.find_commit(base).map_err(TrunkError::from)?;
for i in 0..base_commit.parent_count() {
    revwalk
        .hide(base_commit.parent_id(i).map_err(TrunkError::from)?)
        .map_err(TrunkError::from)?;
}
// Root commit base (parent_count == 0): hide nothing.
```

Add a merge-as-base regression test (the probe above, asserting `side` and `D` are absent
from `[M..N]`).

## Warnings

### WR-01: `session-changed` listener reloads on ANY repo's event while `sessionStatus`/`status` is null

**File:** `src/components/CommitGraph.svelte:1381` and `src/components/ReviewPanel.svelte:84`
**Issue:**
The cross-repo filter is null-permissive:

```ts
if (sessionStatus && event.payload !== sessionStatus.canonical_path) return;
```

When `sessionStatus` (CommitGraph) / `status` (ReviewPanel) is `null` — the initial state,
and also the state `reloadSession`'s `catch` resets to on any failure (see WR-02) — the
guard short-circuits and **every** `session-changed` event from any open repo triggers a
reload here. In a multi-window / multi-repo setup this causes spurious reloads of the wrong
repo's panel. Not data-corrupting, but wasteful and can produce confusing UI churn.
**Fix:** capture the canonical path independently of `sessionStatus` (e.g. resolve it once
on mount and store it), or skip filtering only on the very first load:

```ts
if (event.payload !== canonicalPath) return; // canonicalPath resolved on mount
```

### WR-02: `reloadSession` catch swallows all errors as "no session", hiding `not_open`/IPC failures

**File:** `src/components/CommitGraph.svelte:334-339`
**Issue:**
The comment claims the catch only handles the normal `no_session` case, but it catches
**every** rejection — including `list_session_commits` returning `not_open` when the
`CommitCache` has no entry for the repo yet (the realistic first-run path, before
`refresh_commit_graph` populates the cache) and any transport/IPC failure. All of these
silently reset `sessionStatus = null` and `sessionOids = new Set()`, so a genuinely active
session can render as "no session" (no membership markers, review menu items suppressed
because `sessionActive` is derived from the now-null status).
**Fix:** branch on the error code; only treat `no_session` (and arguably `not_open`) as the
benign empty state, and surface/log unexpected errors instead of clearing UI state:

```ts
} catch (e) {
    const code = (e as TrunkError).code;
    if (code === "no_session" || code === "not_open") {
        sessionStatus = null;
        sessionOids = new Set();
    } else {
        showToast((e as TrunkError).message ?? "Failed to load review session", "error");
    }
}
```

### WR-03: `seed_review_range` / `add_review_commit` do expensive git2 work before checking the session exists

**File:** `src-tauri/src/commands/review.rs:362-393` (`seed_review_range`)
**Issue:**
`seed_review_range` opens the repo, parses OIDs, validates the range, and runs a full
revwalk in `spawn_blocking` **before** `seed_review_range_rmw` checks the in-memory session
(the `no_session` guard lives inside `mutate_session_rmw`). If no session is active, the
user pays for a full walk and only then gets `no_session`. More importantly the ordering
means a `bad_range`/`unrelated_history` error can be returned to a caller that has no
session at all — the error the frontend shows ("Failed to seed range") is misleading versus
the real precondition failure. The gesture is only reachable when `sessionActive` is true
in the UI, so this is a robustness/clarity issue rather than a correctness bug, but the
command is a public IPC surface and should not assume the UI gate.
**Fix:** cheaply confirm the in-memory session exists (lock, `contains_key(&canonical)`,
drop) before the `spawn_blocking` walk, returning `no_session` early; or document that the
walk-before-check ordering is intentional.

### WR-04: `app.emit("session-changed", ...)` failures are silently discarded

**File:** `src-tauri/src/commands/review.rs:391, 411, 431` (and the Phase 65 lifecycle emits)
**Issue:**
Every mutation does `let _ = app.emit("session-changed", ...)`. If the emit fails, the
disk + in-memory state has already changed but no listener is notified, so the panel/graph
silently show stale membership until the next manual reload. `emit` failing is rare, but
discarding the result with `let _ =` means a desync is undetectable. This mirrors existing
Phase 65 style, so it is consistent — flagged as a robustness gap, not a regression.
**Fix:** at minimum log the emit error (e.g. `if let Err(e) = app.emit(...) {
log::warn!(...) }`) so a notification failure is observable.

## Info

### IN-01: `apply_add` dedupes but `apply_remove` removes all occurrences — asymmetric invariant

**File:** `src-tauri/src/commands/review.rs:221-231`
**Issue:** `apply_add` guarantees no duplicate is ever inserted, while `apply_remove` uses
`retain` to strip every occurrence. The set is therefore assumed dup-free on the add path
but defensively de-duped on remove. This only diverges if duplicates enter the vec by some
path other than `apply_add` (e.g. a hand-edited session file or a future writer). Harmless
today; worth a one-line note that the stored `commits` is a set-by-convention.
**Fix:** none required; optionally add a comment documenting the set invariant, or normalize
on load.

### IN-02: `intersect_graph_order` short-OID fallback uses raw OID truncation, not git's short-id

**File:** `src-tauri/src/commands/review.rs:278`
**Issue:** For orphaned/unresolvable OIDs the fallback computes `short_oid` as
`oid_str.chars().take(7).collect()`. For a real-but-not-in-graph commit this differs from
git's ambiguity-aware short id and from how in-graph commits get their `short_oid` (from the
graph). Cosmetic only; the panel just displays it.
**Fix:** acceptable as-is for a fallback; optionally use `find_commit` +
`as_object().short_id()` when the commit resolves.

### IN-03: `validate_range` `base == tip` short-circuit skips existence validation

**File:** `src-tauri/src/commands/review.rs:175-177`
**Issue:** When `base == tip`, `validate_range` returns `Ok(())` before any repo lookup, so
it never confirms the OID actually exists. `compute_range_oids` then calls
`repo.find_commit(base)`, which surfaces a proper error for a nonexistent OID — so the
command path is safe — but `validate_range` in isolation reports a non-existent
`base==tip` pair as valid. Low impact because the two are always called together in the
command.
**Fix:** none required given the call-site pairing; note the contract that `validate_range`
assumes resolvable OIDs for the equality case.

---

_Reviewed: 2026-05-25T13:18:29Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: deep_
