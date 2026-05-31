# Adversarial review — working-tree comment feature (0e38384..HEAD)

Scope: snapshot/anchor model, get-or-create snapshot, the in-diff Comment
affordances, the lag report. Findings ordered by severity. Confirmed bugs and
suspicions are separated.

---

## CONFIRMED — the click lag has two distinct causes; do not conflate them

### C1 (CRITICAL). The composer is gated behind 2–3 serial IPCs, the heaviest a full-workdir hash

`DiffPanel.svelte:212-235` (`openDiffComposer`): the composer is only shown when
`composerOpen` and `diffCaptured` are set, and both are set AFTER:
1. `await ensureActiveSession()` — `get_review_session_status` IPC, plus
   `start_review_session` / `resume_review_session` IPC when not already active
   (`DiffPanel.svelte:151-183, 212`).
2. `await safeInvoke("ensure_working_tree_snapshot")` — `DiffPanel.svelte:221`.

That second call is the dominant cost. It runs `decide_snapshot`
(`workdir_snapshot.rs:55`) → `workdir_tree_oid` (`:25`) →
`idx.add_all(["*"], DEFAULT, None)` (`:37`) which hashes the **entire working
tree** before any commit/compare. On a large or cold-cache repo this is the
visible "long time before the box appears."

The box-appearance latency is exactly the sum of those awaits — nothing else is
on that synchronous path. This is the user's report.

**Remediation (remove the cause, not the symptom):** the composer needs only
line numbers, which come from the hunk (`buildDiffAnchor` uses `new_lineno` /
`old_lineno` off `fd.hunks[hunkIndex].lines`, `diff-anchor.ts:92-96`) — NOT the
snapshot OID. Open the composer synchronously on click; defer the snapshot to
submit. Either await it in `handleSubmit`, or fire it un-awaited at click and
await the stored promise at submit. This makes open-then-cancel instant and free,
and (see C2) removes the spurious reload entirely.

### C2 (CRITICAL, headline). The feature defends against a reload it triggers itself

`snapshot_working_tree` writes a real commit to `.git/objects`
(`workdir_snapshot.rs:93`). The fs watcher watches the repo root with
`RecursiveMode::Recursive` and **no `.git` filter** (`watcher.rs:32`), so that
object write fires `repo-changed` (`watcher.rs:24`). The frontend listener
(`RepoView.svelte:530-541`) debounces 200ms then runs `handleRefresh()`,
`loadDirtyCounts()`, `loadHeadBranch()`, and `refetchFileDiff(...)`.
`refetchFileDiff` reassigns `stagingDiffFiles` (`RepoView.svelte:478`), which is
`currentDiffFiles` (`:193-197`), which is DiffPanel's `fileDiffs`. DiffPanel's
`$effect(() => { fileDiffs; ...; clearSelection(); collapsedFiles = new Set(); })`
(`DiffPanel.svelte:429-435`) then fires: selection cleared, collapsed-state and
hunk-element map wiped, focused hunk reset — under the user, mid-compose, ~500ms
after the click (300ms debouncer + 200ms frontend).

The entire up-front-capture apparatus — the `diffCaptured` snapshot state
(`DiffPanel.svelte:112`), the 8-line rationale comment (`:105-111`), and the
regression test — exists only to survive a reload the feature itself causes.
That is compensating for an effect instead of removing the cause
(engineering_judgment §5). The same deferral in C1 removes this: if no commit is
written until submit, merely composing produces no object write, no
`repo-changed`, no reload, and the defensive capture becomes unnecessary.

Note this is the SECOND disruption, not the box-appearance lag — it lands after
the box is already open. Two findings, one shared fix.

A narrower, independent fix worth noting regardless: the watcher should ignore
`.git/objects` (or `.git` entirely, using libgit2/status for index/ref state).
Recursively watching `.git` means every internal object write churns the UI.

---

## CONFIRMED — correctness / performance

### C3 (HIGH). Dangling snapshot commits are GC bait; every working-tree comment can orphan

`snapshot_working_tree` commits with `None` target ref (`workdir_snapshot.rs:93`)
— deliberately dangling. Comments anchor to that commit OID
(`DiffPanel.svelte:221-234`, persisted via `add_comment`). `git gc` (manual, or
git's own auto-gc which triggers on ordinary operations once loose objects pass
the threshold) prunes unreachable commits. After that, `classify_anchor`
(`review.rs:348-351`) can't `find_commit` the snapshot → every working-tree
comment resolves `CommitGone`. For a feature whose whole point is durable review
notes, "GC degradation is the accepted locked tradeoff" (`:91-92` comment) does
not make this not-a-bug. At minimum the snapshot commits need a keep-ref (e.g.
`refs/trunk/snapshots/<oid>`) for the session's lifetime; the dangling approach
trades a little ref litter for silent data loss.

### C4 (HIGH). "Get-or-create reuse" does NOT avoid the expensive work on the hot path

The reuse decision in `decide_snapshot` (`workdir_snapshot.rs:59-64`) calls
`workdir_tree_oid(repo)` FIRST (full `add_all` hash of the whole workdir), THEN
compares against the prior commit's tree to decide whether to skip the commit
write. So "reuse" only skips `repo.commit(...)` — the cheap part. The dominant
cost (hashing the entire workdir) runs on every single click regardless. The
docstring's framing that reuse "avoids redundant work" is misleading; the
redundant work is the hash, and it is not avoided. Combined with C1 this is why
every Comment click is slow, not just the first.

### C5 (MEDIUM). No in-flight guard on the Comment path → double-click = two snapshots/commits

Staging actions guard with `hunkOperationInFlight` (`DiffPanel.svelte:477-478`
etc.). The comment-open path (`openDiffComposer`, `handleCommentHunk`,
`handleCommentLines`, `handleCommentFullFile`) has no such guard. A double-click
on the hunk Comment button fires two `ensure_working_tree_snapshot` calls →
potentially two distinct snapshot commits (the second `decide_snapshot` may see
the first not yet recorded in the session field; the read-prior TOCTOU is
explicitly left unguarded, `review.rs:806-808`). Extra object writes, extra
`repo-changed` storms. Add an in-flight flag mirroring staging.

---

## SUSPICIONS / latent landmines

### S1 (latent). Snapshot Old side = HEAD tree, but the unstaged diff's Old coordinates are the INDEX

`diff_unstaged_inner` uses `repo.diff_index_to_workdir(...)`
(`commands/diff.rs:374`): the diff's Old side is the **index**, and `old_lineno`
is an index coordinate. But the snapshot commit's parent is **HEAD**
(`workdir_snapshot.rs:79-83`), so `classify_anchor` resolves an `Old` anchor
against `commit.parent(0).tree()` = HEAD tree (`review.rs:357-362`). For a
partially-staged file these trees differ, so an Old-side anchor's line numbers
would index into the wrong blob → wrong excerpt or spurious `LineOutOfRange`.
Currently masked because `openDiffComposer` hard-rejects Old-side working-tree
selections (`DiffPanel.svelte:201-210`). It becomes a real bug the moment
Old-side commenting is enabled. If the model is "New = workdir, Old = before",
the snapshot's parent should arguably be the index, not HEAD — or the diff and
the snapshot should share a base. Flag explicitly; do not ship Old-side support
on top of this without fixing it.

### S2 (latent). `commitOid` can be the empty string and is silently embedded in the anchor

When `diffKind === "unstaged"` and `workingTreeSnapshotOid` is null, the derived
`commitOid` is `""` (`DiffPanel.svelte:98-102`). The flow sets it before building
the anchor on the happy path, but `CommentComposer` also derives its anchor from
the `commitOid` prop directly (`CommentComposer.svelte:54`), and the full-file
path builds `fullFileCaptured` from `commitOid` reactively (`DiffPanel.svelte:125-134`).
There is no assertion that `commitOid` is a real OID before persistence. An empty
or stale OID would be written into a comment anchor and later resolve `CommitGone`.
A guard (`if (!oid) return;` after the snapshot call) would harden it.

---

## TESTS — what they actually prove

### T1. The l02 regression test is partly hollow

`DiffPanel.test.ts:310-336` ("keeps the whole-hunk comment range finite when the
diff reloads mid-compose"):
- It mocks `ensure_working_tree_snapshot` to return `undefined` (the default mock,
  `:29-36`), so `buildDiffAnchor(undefined, ...)` is exercised — a **real snapshot
  OID is never tested** end to end.
- It simulates the reload with `rerender({ fileDiffs: [testDiff] })`, NOT the
  actual watcher → `repo-changed` → `refetchFileDiff` → `clearSelection` path. It
  proves the captured range survives a `fileDiffs` identity change; it does not
  prove the real loop is handled.
- For the whole-hunk path specifically, the indices come from
  `hunkSelectableIndices(hunk)` (`DiffPanel.svelte:254`), not from
  `selectedLineIndices`. The cleared selection was never an input to that path, so
  "survives the clear" is largely tautological for the case the test name targets.
  The Infinity bug it guards against only ever existed for the *line-selection*
  path, which this test does not drive.

The test is not wrong, but it asserts less than its name implies. If C1/C2 are
fixed (no snapshot before open, no reload), this test and the capture machinery
it guards should be deleted, not kept.

### T2. Rust snapshot tests are solid and non-vacuous

`workdir_snapshot.rs` tests genuinely exercise behavior against real temp repos:
ignored-file exclusion, untracked inclusion, real-`.git/index`-untouched
(`:176-192`, a meaningful invariant), determinism, and the reuse-vs-create
decision asserting the returned OID identity (catches a tree-vs-commit mixup).
These are good. They do NOT cover the GC-orphan scenario (C3) or the
HEAD-vs-index Old-side mismatch (S1) — both untested.

---

## Dead code / coupling notes

- `seed_review_range_inner` is `#[cfg(test)]`-only and duplicates the live
  command's precheck (`review.rs:675-707`), with a comment admitting the live
  command can't call it (bare `Mutex` vs `spawn_blocking` `'static` bound). This
  is a test-only mirror that must be hand-kept in sync — a known orthogonality
  smell, acknowledged in-code but still a maintenance liability.
- The two CommentComposer mount branches (`DiffPanel.svelte:722-738`) and the
  dual capture contracts (injected `captured` vs derived diff-path) add a second
  code path that only the full-file feature uses; if C1/C2 are addressed the
  diff-path `captured` injection can likely collapse back to deriving in the
  composer.

---

## Biggest-risk verdict

The approach is the risk. Snapshotting the whole working tree into a `.git`
object **before** the composer can open does three bad things at once: it gates
the UI on a full-workdir hash (the reported lag, C1/C4), it trips the recursive
`.git` watcher into a self-inflicted mid-compose reload that the feature then
spends real code and a test defending against (C2), and it persists comments
against dangling commits that GC can silently prune (C3). All three dissolve
under one change: do not write the snapshot until submit, and open the composer
synchronously from hunk-local line numbers. Secondary must-fix before any
expansion: the HEAD-vs-index Old-side base mismatch (S1) and a `.git`-ignoring
watcher. Do not build further on the current snapshot-on-open design.
