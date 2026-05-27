# Phase 74: Address v0.13 tech debt — Research

**Researched:** 2026-05-27
**Domain:** Tech-debt cleanup (Tauri 2 + Svelte 5 + git2/Rust)
**Confidence:** HIGH

## Summary

Phase 74 cleans up the seven deferred findings from the v0.13 milestone audit. Six items still apply to current code (re-verified by reading the actual files); one (70/WR-01 — `previewMarkdown` staleness) is fully resolved by Phase 72's preview-pane deletion and should be dropped from scope. The remaining six split into four small Rust/Svelte logic fixes (66/WR-01, WR-02, WR-03, WR-04), one moderate Rust correctness fix with the most behavioral risk (70/CR-01 multi-hunk `slice_diff` leak), and one tiny lint-only annotation (3× `noNonNullAssertion` in `CommentComposer.svelte:43`).

Five of seven items are isolated single-file edits; only 66/WR-01 ripples across both `CommitGraph.svelte` and `ReviewPanel.svelte` listeners (and `ReviewPanel`'s listener is **already** fixed during Phase 73 — only `CommitGraph` carries the bug, see per-item §1). TDD is feasible for 70/CR-01 (multi-hunk reproducer fixture), 66/WR-01 (listener filter unit test), and 66/WR-03 (no_session-before-walk assertion). The remaining items are either pure refactors (biome warnings, app.emit logging) or hard to unit-test without IPC mocking gymnastics.

**Primary recommendation:** Plan as three waves: Wave 1 = parallelizable independent fixes (66/WR-01, 66/WR-04, biome); Wave 2 = the two Rust-side items that share `commands/review.rs` (66/WR-02 swallowing, 66/WR-03 reordering); Wave 3 = 70/CR-01 (the only correctness-with-behavioral-risk item, kept isolated). Drop 70/WR-01 with explicit verification evidence. Confirm INT-W1 and INT-W2 out of scope.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Event-listener cross-repo filtering | Browser / Client (Svelte) | — | Tauri events arrive on the frontend; filter at listener |
| IPC error classification (no_session vs other) | Browser / Client (Svelte) | — | Same as above — only frontend can render a toast |
| Session preconditions on backend commands | API / Backend (Rust) | — | `commands/review.rs` is the IPC surface; defense in depth |
| Diff range slicing | API / Backend (Rust) | — | `git/review.rs` pure renderer owns this |
| Lint-noise suppression | Browser / Client (Svelte) | — | Compile-time annotation, no runtime tier |

## Scope Summary

| ID | Item | Still applies? | File:line | Complexity | TDD-eligible? |
|----|------|----------------|-----------|------------|----------------|
| 66/WR-01 | session-changed listener null-permissive | YES (CommitGraph only — ReviewPanel already fixed in Phase 73) | `src/components/CommitGraph.svelte:1380-1389` | small | YES |
| 66/WR-02 | reloadSession swallows all IPC failures | YES | `src/components/CommitGraph.svelte:319-340` | small | YES (but jsdom IPC mocking required) |
| 66/WR-03 | seed_review_range walks before session check | YES | `src-tauri/src/commands/review.rs:632-663` | small | YES (Rust unit test asserts `no_session` early) |
| 66/WR-04 | app.emit failures discarded | YES (10 call sites in `commands/review.rs`) | `src-tauri/src/commands/review.rs:661, 681, 701, 735, 789, 815, 839, 1065, 1120, 1142` | 1-line × 10 | NO (rare in test; refactor only) |
| 70/CR-01 | slice_diff multi-hunk opposing-side leak | YES | `src-tauri/src/git/review.rs:175-204` | medium | YES (RED test with multi-hunk fixture) |
| 70/WR-01 | previewMarkdown not cleared on repoPath change | **NO — drop from scope** | (no remaining references) | — | — |
| Biome warnings | 3× noNonNullAssertion | YES | `src/components/diff/CommentComposer.svelte:43` | 1-line annotation | NO (refactor only) |

---

## Per-item research

### 1. 66/WR-01 — Null-permissive session-changed listener (CommitGraph only)

**Still applicable?** YES, but narrower than the audit implies. **ReviewPanel.svelte:447-461 already fixed during Phase 73-01** — it tracks a separate `canonicalPath` (line 241) resolved synchronously on `reload()`, and the listener filter at line 451 checks `if (canonicalPath && event.payload !== canonicalPath) return;`. The 73-01 RESEARCH explicitly cites this as "Pitfall 1: fail-closed when canonicalPath is null." Only `CommitGraph.svelte` still has the original null-permissive pattern.

**Exact file path + line range:** `src/components/CommitGraph.svelte:1378-1389` (the `$effect` that installs the listener). The buggy check is line 1381: `if (sessionStatus && event.payload !== sessionStatus.canonical_path) return;`.

**Recommended fix approach:** Copy ReviewPanel's `canonicalPath` pattern verbatim. Specifically:
1. Add `let canonicalPath = $state<string | null>(null);` near line 311.
2. In `reloadSession()` (line 319), capture `canonicalPath = status.canonical_path;` immediately after the `get_review_session_status` call succeeds — BEFORE the `if (status.state === "active")` branch. (This way, even cold sessions get the canonical path tracked.)
3. Change the listener guard to `if (canonicalPath && event.payload !== canonicalPath) return;`.
4. Reset `canonicalPath = null` in the `catch` (matching the existing `sessionStatus = null` line, but **only after** WR-02 is also fixed — see §2).

Use the same cancellation pattern ReviewPanel adopted at line 449-460 (`let cancelled = false;` + `if (cancelled) fn();` in `.then`) — the original CommitGraph `$effect` lacks this, so a teardown during the unresolved `listen()` promise leaks listeners across remounts. This is a small piggyback fix; Plan it explicitly.

**Risk / blast radius:** Localized to CommitGraph. The only consumer of `sessionStatus.canonical_path` is line 1381; nothing reads CommitGraph's `sessionStatus` from outside. Zero ripple.

**Test strategy:** Vitest. Mock `safeInvoke` for `get_review_session_status` and `list_session_commits`; use the existing `installReads` helper pattern from ReviewPanel tests if accessible, otherwise replicate. Two tests:
- **RED → GREEN:** When `canonicalPath` is `null` (initial state, before first `reloadSession()` resolves), a `session-changed` event with payload `/some/other/repo` should NOT trigger reload. Today it does (asserts `safeInvoke` not called); after fix it doesn't.
- **GREEN regression:** After `reloadSession()` resolves with canonical_path `/this/repo`, a `session-changed` for `/this/repo` triggers reload; for `/other/repo` does not.

TDD-eligible: YES.

---

### 2. 66/WR-02 — reloadSession swallows all IPC failures as "no session"

**Still applicable?** YES. `src/components/CommitGraph.svelte:334-339` is verbatim the code the audit cites — a bare `catch { sessionStatus = null; sessionOids = new Set(); }`. ReviewPanel's `reload()` (line 281-292) does the right thing already: branches on `isTrunkError(e) && e.code === "no_session"` and surfaces a toast for other errors. CommitGraph has not received that treatment.

**Exact file path + line range:** `src/components/CommitGraph.svelte:319-340` (the whole `reloadSession` function).

**Recommended fix approach:** Copy ReviewPanel's pattern. Wrap the `get_review_session_status` and `list_session_commits` calls in distinct try/catches, OR keep the single try/catch but branch on error code:

```ts
} catch (e) {
    if (isTrunkError(e) && (e.code === "no_session" || e.code === "not_open")) {
        sessionStatus = null;
        canonicalPath = null;     // tied to §1 fix
        sessionOids = new Set();
        return;
    }
    showToast("Failed to load review session. Try again or reopen the repo.", "error");
}
```

Treat `not_open` as a benign empty state per the audit's recommendation — it can fire during the first-run window before `refresh_commit_graph` populates `CommitCache`. Anything else (network/IPC failure, panic) surfaces a toast.

**Imports needed:** `isTrunkError` from `$lib/invoke.js` (already used elsewhere; check for existing import in this file — grep showed none, so add it). `showToast` from `$lib/toast.svelte.js` (already used in this file? — check during planning).

**Risk / blast radius:** Localized. Adds a toast on previously-silent error paths; users who never saw the bug see no change.

**Test strategy:** Vitest. Mock `safeInvoke` to reject with three shapes: `{code: "no_session"}`, `{code: "not_open"}`, `{code: "internal"}`. Assert toast called only on the third. TDD-eligible: YES, but requires the same IPC-mocking harness as §1.

---

### 3. 66/WR-03 — seed_review_range walks before checking session exists

**Still applicable?** YES. `src-tauri/src/commands/review.rs:632-663` — `seed_review_range` clones state_map outside spawn_blocking (line 640), then inside spawn_blocking does canonical_repo_path + Repository::open + Oid::from_str (×2) + validate_range + compute_range_oids. The `no_session` check lives inside `mutate_session_rmw` → `sessions.lock().get(canonical)` which only runs AFTER `spawn_blocking` returns (line 659). Empty sessions HashMap → expensive walk → useless `no_session` error.

**Exact file path + line range:** `src-tauri/src/commands/review.rs:639-660`.

**Recommended fix approach:** Add a cheap session-existence probe BEFORE the spawn_blocking walk. Two options:

**Option A (minimal):** Resolve canonical OUTSIDE spawn_blocking (matching sibling commands like `add_review_commit:676-677`), then lock the sessions mutex briefly to check `contains_key`. This **also closes INT-W1** (the audit's stylistic inconsistency — see §Out-of-Scope below) because canonical now resolves consistently with siblings.

```rust
let state_map = state.0.lock().unwrap().clone();
let data_dir = resolve_data_dir(&app)?;
let canonical = canonical_repo_path(&path, &state_map)
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
{
    let map = sessions.0.lock().unwrap();
    if !map.contains_key(&canonical) {
        return Err(serde_json::to_string(&TrunkError::new(
            "no_session", "No active review session for this repository"
        )).unwrap());
    }
}
// THEN spawn_blocking for the walk
```

Note the second lock acquisition inside `seed_review_range_rmw` is fine — Rust's `Mutex` is not reentrant, but the early lock is dropped before spawn_blocking starts. The TOCTOU window between the precheck and the RMW is acceptable: if End-review fires between them, the RMW lock will surface `no_session` and the user-visible result is identical.

**Option B (more invasive):** Add a `with_session_or_err(sessions, canonical, |session| -> Result<...>)` helper that locks once, runs validate-then-walk, then writes — but this defeats the spawn_blocking design (libgit2 walks under a tokio runtime mutex). Reject Option B.

**Risk / blast radius:** Localized to `seed_review_range`. The 2-line precheck doesn't change correctness; it only converts "did expensive work then said no_session" into "said no_session immediately." Three sibling commands (`add_review_commit`, `remove_review_commit`, etc.) already resolve canonical outside spawn_blocking and have no walk — they don't need this fix.

**Test strategy:** Cargo. Add a Rust unit test in the existing `tests` mod of `commands/review.rs`: create a temp data_dir, do NOT start a session, call `seed_review_range_rmw` (or the public command via a thin wrapper); assert `Err(TrunkError { code: "no_session", .. })` is returned without touching git2 (use a fake repo path that would fail to open). The fact that the test runs in microseconds proves no walk happened.

TDD-eligible: YES. Write the assertion-no_session-fast test first.

---

### 4. 66/WR-04 — app.emit failures silently discarded

**Still applicable?** YES — verified by grep: 10 call sites in `commands/review.rs` use `let _ = app.emit("session-changed", ...)`. Lines: 661, 681, 701, 735, 789, 815, 839, 1065, 1120, 1142. The audit cites only the first three but the pattern is project-wide for this command file.

**Exact file path + line range:** `src-tauri/src/commands/review.rs` — the 10 sites listed above.

**Recommended fix approach:** Replace each `let _ = app.emit(...)` with:

```rust
if let Err(e) = app.emit("session-changed", canonical.to_string_lossy().into_owned()) {
    log::warn!("session-changed emit failed for {}: {}", canonical.display(), e);
}
```

Check whether the project uses `log`, `tracing`, or `eprintln!` — grep the codebase. (Note: this is a Tauri app — `log::warn!` requires a logger to be initialized. If `tracing` is the project convention, use `tracing::warn!` instead.)

Consider extracting a helper to avoid 10 copies:

```rust
fn emit_session_changed(app: &AppHandle, canonical: &Path) {
    if let Err(e) = app.emit("session-changed", canonical.to_string_lossy().into_owned()) {
        log::warn!("session-changed emit failed for {}: {}", canonical.display(), e);
    }
}
```

then `emit_session_changed(&app, &canonical);` at each site. The helper is self-documenting and reduces visual noise.

**Risk / blast radius:** Mechanical refactor. Zero behavioral change on the happy path (emit succeeds). On the never-seen-in-practice failure path, the log line appears; nothing else changes. Same Phase 65 "never silently destroy" stance the audit invokes.

**Test strategy:** None — `app.emit` failures are essentially untestable without a Tauri integration test harness. This is a refactor with logging; verify by reading the diff and running `cargo clippy` + `cargo test`.

TDD-eligible: NO. This is pure refactor + logging.

---

### 5. 70/CR-01 — slice_diff multi-hunk opposing-side leak (the only correctness item)

**Still applicable?** YES. `src-tauri/src/git/review.rs:175-204` is verbatim the code the audit cites. The line callback's `None => matches!((side, line.origin()), (Side::New, '-') | (Side::Old, '+'))` keeps every opposing-side line from every hunk with no positional check.

**Exact file path + line range:** `src-tauri/src/git/review.rs:175-204` (the `diff.foreach` block inside `slice_diff`).

**Recommended fix approach:** Thread a `hunk_overlaps` flag through the foreach closures so opposing-side lines only emit when the current hunk overlaps `[start_line, end_line]`. The audit's pseudocode is correct; tactical detail:

The closure signatures in `git2::Diff::foreach` are `FnMut`, and the line callback fires after the hunk callback for the same hunk. Capturing `hunk_overlaps` by closure across two callbacks requires `RefCell` (or `Cell`) wrapping the bool, OR a different shape — `git2::Patch` iteration is cleaner: walk each `Patch` (one per file), iterate its hunks via `patch.num_hunks()` + `patch.hunk(i)` + `patch.num_lines_in_hunk(i)` + `patch.line_in_hunk(i, j)`, and decide per-hunk whether to slice. The `Patch` shape avoids the closure-coupling problem entirely.

Recommended: refactor `slice_diff` to use `git2::Patch::from_diff(&diff, 0)` (single-file diff because `opts.pathspec(&anchor.file_path)` constrains it), then iterate hunks and only emit lines from overlapping ones. The body remains short and readable.

**Sketch (verify against current git2 0.19 API):**
```rust
let mut patch = git2::Patch::from_diff(&diff, 0)
    .map_err(|_| ExcerptError::ResolutionFailed)?
    .ok_or(ExcerptError::NoHunks)?;
let mut out = String::new();
for h_idx in 0..patch.num_hunks() {
    let (hunk, _hunk_lines) = patch.hunk(h_idx).map_err(|_| ExcerptError::ResolutionFailed)?;
    let (h_start, h_count) = match side {
        Side::New => (hunk.new_start(), hunk.new_lines()),
        Side::Old => (hunk.old_start(), hunk.old_lines()),
    };
    let h_end = h_start + h_count.saturating_sub(1);
    let overlaps = h_start <= end_line && h_end >= start_line;
    for l_idx in 0..patch.num_lines_in_hunk(h_idx).map_err(|_| ExcerptError::ResolutionFailed)? {
        let line = patch.line_in_hunk(h_idx, l_idx).map_err(|_| ExcerptError::ResolutionFailed)?;
        let lineno = match side {
            Side::New => line.new_lineno(),
            Side::Old => line.old_lineno(),
        };
        let in_range = match lineno {
            Some(n) => n >= start_line && n <= end_line,
            None => overlaps && matches!(
                (side.clone(), line.origin()),
                (Side::New, '-') | (Side::Old, '+')
            ),
        };
        if !in_range { continue; }
        let prefix = match line.origin() { '+' | '-' | ' ' => line.origin(), _ => ' ' };
        out.push(prefix);
        out.push_str(&String::from_utf8_lossy(line.content()));
    }
}
```

The shape is `[CITED: git2 0.19 docs at https://docs.rs/git2/0.19/git2/struct.Patch.html]`. **The planner MUST verify `Patch::from_diff` and `Patch::line_in_hunk` are stable in git2 0.19** — check `src-tauri/Cargo.toml` for the pinned version, then `cargo doc --open` or fetch docs.rs. The capture-time adapter `buildDiffAnchor` in `src/lib/diff-anchor.ts` does NOT need to change — this is render-time only.

**Risk / blast radius:** Localized to `slice_diff`. Three existing tests touch it: `slice_diff_returns_requested_range:760`, `slice_diff_returns_no_hunks_when_file_unchanged:782`, `slice_diff_handles_root_commit:814`. All are single-hunk fixtures; they SHOULD still pass after the fix (single hunk always overlaps itself). The behavioral change is invisible to today's tests — the new test below catches it.

**Test strategy:** Cargo. **TDD-eligible: YES.** Write the failing test FIRST:

```rust
#[test]
fn slice_diff_multi_hunk_excludes_unrelated_hunks() {
    let TestRepo { repo, _tmp } = make_repo();
    // Parent: 50 lines, "L1\n...L50\n"
    let parent_oid = commit_with_file(&repo, "foo.rs", parent_content_50_lines());
    // Child: edits line 5 AND line 45 (forces two hunks).
    let child_oid = commit_with_file_on_top(&repo, parent_oid, "foo.rs",
        child_content_edits_at_5_and_45());
    let anchor = Anchor {
        commit_oid: child_oid.to_string(),
        file_path: "foo.rs".into(),
        side: Side::New, source: Source::Diff,
        start_line: 45, end_line: 45,
    };
    let out = slice_diff(&repo, &anchor).expect("slice_diff");
    // Must contain the line-45 hunk's content
    assert!(out.contains("L45_NEW"));
    // Must NOT leak the line-5 hunk's deletion
    assert!(!out.contains("L5_OLD"), "leaked unrelated hunk:\n{out}");
}
```

This test fails today (the line-5 `-` line leaks); passes after the fix. **This is the strongest TDD candidate in the phase.**

TDD-eligible: YES (mandatory — this is the only correctness fix).

---

### 6. 70/WR-01 — previewMarkdown not cleared on repoPath change

**Still applicable?** **NO. DROP FROM SCOPE.**

**Evidence:** `grep -rn "previewMarkdown\|panelMode\|ReviewDocPreview" src/` returns ZERO matches. Phase 72-01 dropped `panelMode` and `previewMarkdown` from the rune (per 72-01-SUMMARY); Phase 72-04 deleted `ReviewDocPreview.svelte` and its test. The audit doc itself flagged this — "preview pane was deleted in Phase 72; verify if still applicable" — and the answer is no.

**Action:** The plan must contain a one-line note documenting that this audit item was checked and is fully resolved by Phase 72, with the grep evidence above as the citation. No code change.

**Test strategy:** N/A.

TDD-eligible: N/A.

---

### 7. Biome warnings — 3× noNonNullAssertion in CommentComposer.svelte:43

**Still applicable?** YES. `bunx biome ci src/components/diff/CommentComposer.svelte` confirms 3 warnings at line 43 columns 41, 48, 58 (`file!`, `hunkIdx!`, `selectedLineIndices!`). Path corrected: the audit said `src/lib/.../CommentComposer.svelte` but the actual file is `src/components/diff/CommentComposer.svelte:43`.

**Note:** `bunx biome ci` returns exit code **0** despite warnings — `just check` is green today. These are noise, not gates. The cleanup is about silencing the warnings so future real warnings aren't lost in the existing three.

**Exact file path + line range:** `src/components/diff/CommentComposer.svelte:43`.

**Recommended fix approach:** The non-null assertions document a real caller contract (DiffPanel.svelte:625 guards `composerOpen && composerFile && composerHunkIdx !== null` before passing props; selectedLineIndices is bound from DiffPanel's `$state` and always non-null). The component lives in two contracts: a diff-path contract (all three optional props supplied) and a full-file-path contract (`captured` supplied, three optional props omitted). The `??` choice runs the assertions only on the diff path where they hold.

Three options ranked by quality:

**Option A (recommended): Refactor to discriminated union via Svelte's type narrowing.** Split `Props` into two interfaces, one per contract; choose at the call site. This is invasive (DiffPanel.svelte:625-642 also changes) and not worth it for three warnings.

**Option B (recommended for THIS phase): Move the `buildDiffAnchor` call into a typed helper that narrows via a single check.** Replace line 42-44 with:

```ts
function deriveDiffCapture(): { anchor: Anchor; cachedExcerpt: string } {
    if (file === undefined || hunkIdx === undefined || selectedLineIndices === undefined) {
        throw new Error(
            "CommentComposer: diff-path props missing — caller contract violated"
        );
    }
    return buildDiffAnchor(commitOid, file, hunkIdx, selectedLineIndices);
}
const capturedResult = $derived(captured ?? deriveDiffCapture());
```

The runtime check is dead in practice (DiffPanel guards before render) but turns the implicit "assert non-null" into an explicit guard. Cost: one short function, zero biome warnings, slightly more code but clearer intent. **This is the most aligned with project tone** (the `coding_style.md` rules' "Correctness over comfort" — be explicit about what you're asserting).

**Option C (cheapest): Add `// biome-ignore lint/style/noNonNullAssertion: caller contract (DiffPanel:625 guards)` comment above line 43.** Three suppressions per warning. The project has zero existing `biome-ignore` directives — verified by grep — so introducing one sets a precedent worth a brief team note in the SUMMARY. Use only if Option B is rejected.

**Risk / blast radius:** Option B is localized to lines 42-44 + the new function. Option C is a single annotation. Either is trivial.

**Test strategy:** Existing tests (`src/components/diff/CommentComposer.test.ts`) cover both contracts. Re-run vitest after the change; the assertion-throw branch is unreachable in practice but harmless if hit. No new test needed unless Option B's runtime guard is treated as a real invariant.

TDD-eligible: NO. This is a lint refactor, not a behavior change.

---

## Out-of-Scope Confirmations

### INT-W1 — `seed_review_range` resolves canonical inside spawn_blocking

**Audit verdict:** "Inconsistent ordering, not a bug." **Verified:** Confirmed by reading `commands/review.rs:640-657` vs `:674-678`. Siblings (`add_review_commit:676-678`, etc.) call `canonical_repo_path` outside spawn_blocking; `seed_review_range:646` does it inside. No correctness consequence (the spawn_blocking task still gets the same `state_map` clone).

**However:** The §3 fix for WR-03 (Option A) **incidentally closes INT-W1** by moving the canonical resolution outside spawn_blocking to match siblings. The planner can either (a) note this side-effect in 66/WR-03 and not list INT-W1 as a separate task, or (b) explicitly mark INT-W1 as "swept up by WR-03 fix; no separate task needed." Recommend (a).

**Surface evidence to escalate?** No. The audit's "not a bug" judgment is correct.

### INT-W2 — save_draft_comment race with End-review

**Audit verdict:** "Out-of-band failure path not surfaced in composer." **Verified:** `commands/review.rs:746-820` shows `save_draft_comment` returns the standard `no_session` TrunkError when End-review has dropped the session, but `CommentComposer.svelte:67` shows the error path:

```ts
} catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? "Save draft failed", "error");
}
```

**A toast IS surfaced** — the message will read "No active review session for this repository." This is not silent — it's just a generic error. The audit's gloss "not surfaced in composer" is technically incorrect; the issue is more accurately "the message is generic, not specifically guiding the user to discard or reopen."

**Surface evidence to escalate?** Minor. The composer's UX could be richer (e.g., on `no_session` show "Review ended in another tab — discard draft?"), but that's a feature ask, not a bug. **Recommend: confirm out of scope** for this phase. If the user disagrees during discuss-phase, this becomes a small extra item in `CommentComposer.svelte:63-67`.

---

## Cross-Cutting Risks

### A. CommitGraph listener fixes (§1 + §2) share a file and a function

`reloadSession()` (lines 319-340) is modified by both 66/WR-01 (add `canonicalPath` capture) and 66/WR-02 (branch on error code). The two edits don't conflict but must be planned as ONE task or two sequential tasks in the same wave (not parallel) to avoid merge thrash. Recommend: **one combined task** "fix CommitGraph session-changed listener and error handling," because the WR-02 fix's `catch` branch also needs to reset `canonicalPath = null` per §1. They are coupled.

### B. `commands/review.rs` is touched by §3 (WR-03), §4 (WR-04), and the §5 incidental INT-W1 sweep

These can be planned as separate tasks in the same wave only if they touch disjoint lines:
- WR-03 modifies lines 632-663 (`seed_review_range` body) only.
- WR-04 modifies 10 single-line emit sites scattered across the file (661, 681, 701, 735, 789, 815, 839, 1065, 1120, 1142). **Site 661 lives inside `seed_review_range`** and is in WR-03's modified range. Conflict.

Recommend: order WR-03 and WR-04 sequentially (same wave, WR-04 after WR-03) OR combine into one task. The combined task is less risky for a 1-line × 10-site mechanical refactor.

### C. Phase 73's `canonicalPath` pattern is the template

The fix for §1 directly copies the pattern Phase 73-01 introduced in ReviewPanel. The planner should reference Phase 73-01's PLAN and SUMMARY for the exact shape, including the `let cancelled = false;` listener-leak guard. This is not new design — it's pattern replication. Cite `.planning/phases/73-review-lifecycle-end-review-cold-boot-resume/73-01-PLAN.md` in the Plan as the canonical shape.

### D. Rust unit test fixtures

§3 and §5 both need test fixtures that build temp repos. The existing helpers in `src-tauri/src/git/review.rs:586-650` (`make_repo`, `commit_with_file`, etc.) and the duplicated set in `commands/review.rs:1199-2185` (audit IN-02) can be reused. The audit's IN-02 "extract test helpers" is **out of scope** for Phase 74 — flag it but don't bundle.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework (Rust) | `cargo test` (built-in) |
| Framework (TS) | Vitest 3.2 (per `package.json`) |
| Config files | `Cargo.toml` (workspace tests); `vitest.config.ts`; `biome.json` |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml --lib slice_diff` (per-item Rust); `npx vitest run src/components/CommitGraph` (per-item TS) |
| Full suite command | `just check` |
| Phase gate | `just check` exits 0 |

### Phase Requirements → Test Map

This phase has no formal REQUIREMENTS.md IDs (it's a tech-debt phase). The audit items are the de facto requirements:

| Audit ID | Behavior | Test Type | Automated Command | File Exists? |
|----------|----------|-----------|-------------------|--------------|
| 66/WR-01 | session-changed listener filters by canonical path when sessionStatus is null | unit (TS) | `npx vitest run src/components/CommitGraph -t "session-changed"` | ❌ Wave 0 |
| 66/WR-02 | reloadSession surfaces toast on non-no_session errors | unit (TS) | `npx vitest run src/components/CommitGraph -t "reloadSession error"` | ❌ Wave 0 |
| 66/WR-03 | seed_review_range returns no_session without running git walk when session absent | unit (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml seed_review_range_rejects_when_no_session` | ❌ Wave 0 |
| 66/WR-04 | app.emit warnings appear in log on failure | manual / refactor-only | (none — refactor) | N/A |
| 70/CR-01 | slice_diff multi-hunk excludes unrelated opposing-side lines | unit (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml slice_diff_multi_hunk_excludes_unrelated_hunks` | ❌ Wave 0 |
| 70/WR-01 | (dropped — no code) | none | none | N/A |
| Biome warnings | `bunx biome ci src/components/diff/CommentComposer.svelte` returns 0 warnings | static (Biome) | `bunx biome ci src/components/diff/CommentComposer.svelte` | ✅ (biome already in `just check`) |

### Sampling Rate

- **Per task commit:** Run the targeted test for the edited surface (e.g., `cargo test slice_diff` after the §5 fix).
- **Per wave merge:** Full `just check`.
- **Phase gate:** Full `just check` exits 0 before `/gsd:verify-work 74`.

### Wave 0 Gaps

- [ ] Existing test files cover the surfaces; no new test FILE is needed, but four NEW TESTS must be added:
  - `slice_diff_multi_hunk_excludes_unrelated_hunks` in `src-tauri/src/git/review.rs` (CR-01 RED test, the keystone)
  - `seed_review_range_rejects_when_no_session` in `src-tauri/src/commands/review.rs` (WR-03 fast-fail assertion)
  - `commit_graph_session_changed_filters_when_canonical_known_null` in `src/components/CommitGraph.test.ts` (WR-01 — note: CommitGraph.test.ts existence must be verified by planner — if absent it becomes a Wave 0 task)
  - `commit_graph_reload_session_surfaces_toast_on_unexpected_error` in `src/components/CommitGraph.test.ts` (WR-02)
- [ ] No new framework or fixture infrastructure needed; the existing harnesses suffice.

*(If `CommitGraph.test.ts` does not exist today, the planner must add it as a Wave 0 task before WR-01/WR-02 tests can land. Verify during /gsd:plan-phase.)*

---

## Project Constraints (from CLAUDE.md)

The planner MUST honor these:

1. **git2-only:** No shelling out to `git` for any operation. (CR-01 fix stays in git2; no new shell calls.)
2. **No inline colors:** Always CSS custom properties from the theme. (No color edits in this phase, but if any toast styling is touched, theme tokens only.)
3. **No layout positioning hacks:** Use grid/flexbox. (No layout work in this phase.)
4. **`just check` exits 0 before commit/push:** Phase gate. All 6 tools (fmt, biome, svelte-check, clippy, cargo-test, vitest) must be green.
5. **TDD_MODE is on** (`.planning/config.json` → `workflow.tdd_mode: true`): items 1, 2, 3, 5 should follow RED→GREEN→REFACTOR. Items 4, 6 (dropped), 7 are not TDD-eligible (refactor / no-op / lint).
6. **Nyquist validation is enabled** (`workflow.nyquist_validation: true`): phase needs a VALIDATION.md.
7. **commit_docs is true:** RESEARCH.md should be committed.

---

## Code Examples

### CommitGraph listener fix (§1) — pattern from ReviewPanel.svelte:447-461

```ts
// Source: ReviewPanel.svelte:447-461 (Phase 73-01 PLAN, verified 2026-05-27)
$effect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    listen<string>("session-changed", (event) => {
        if (canonicalPath && event.payload !== canonicalPath) return;
        reloadSession();
    }).then((fn) => {
        if (cancelled) fn();
        else unlisten = fn;
    });
    return () => {
        cancelled = true;
        unlisten?.();
    };
});
```

### CommitGraph reloadSession error branching (§2) — pattern from ReviewPanel.svelte:281-292

```ts
// Source: ReviewPanel.svelte:281-292 (verified 2026-05-27)
} catch (e) {
    if (isTrunkError(e) && (e.code === "no_session" || e.code === "not_open")) {
        sessionStatus = null;
        canonicalPath = null;
        sessionOids = new Set();
        return;
    }
    showToast("Failed to load review session. Try again or reopen the repo.", "error");
}
```

### slice_diff multi-hunk fix (§5) — git2 Patch iteration shape

See per-item §5 above for the full sketch. **`Patch::from_diff` API signature must be verified against the pinned git2 version in `src-tauri/Cargo.toml` before planning.** `[CITED: docs.rs/git2/0.19/git2/struct.Patch.html — verify at plan time]`

---

## State of the Art / Pattern Notes

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `if (sessionStatus && event.payload !== …)` (null-permissive) | `if (canonicalPath && …)` (canonical tracked separately, fail-closed when null) | Phase 73-01 | ReviewPanel fixed; CommitGraph still on old pattern |
| `let _ = app.emit(…)` (silent discard) | `if let Err(e) = app.emit(…) { log::warn!(…) }` | Phase 74 (proposed) | Observable but non-blocking |
| `slice_diff` with `Diff::foreach` closures + opposing-side keep-all | `Patch::from_diff` + per-hunk `overlaps` gate | Phase 74 (proposed) | Correctness fix; same fence shape, narrower content |

---

## Suggested Wave / Dependency Structure

This is a HINT for the planner — final wave assignment belongs to plan-phase.

### Wave 1 (parallelizable — disjoint files)
- **Task A:** CommitGraph fixes 66/WR-01 + 66/WR-02 combined (one file, one function — see cross-cutting risk A)
- **Task B:** Biome warning fix (`CommentComposer.svelte:43` — Option B helper or Option C annotation)
- **Task C:** Drop-from-scope evidence for 70/WR-01 (one-line note in SUMMARY, no code change — could even be in the plan itself)

### Wave 2 (sequential — both in `commands/review.rs`, shared lines)
- **Task D:** 66/WR-03 (precheck before walk; also closes INT-W1)
- **Task E:** 66/WR-04 (emit-failure logging — depends on D because D moves a line that E also touches)

### Wave 3 (isolated — the only correctness fix, kept separate for risk isolation)
- **Task F:** 70/CR-01 (slice_diff multi-hunk fix; TDD RED→GREEN; only Rust git/review.rs touched)

Wave 3 could run parallel to Wave 1, but isolating the only behavior-changing fix makes verification cleaner. The planner may collapse waves if the codebase contention is acceptable.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `git2::Patch::from_diff` is stable and available in the pinned `git2` version (0.19 per CLAUDE.md) | §5 fix sketch | Plan-phase must verify against `src-tauri/Cargo.toml`; if API differs, a `RefCell` shape over `diff.foreach` works as fallback (per the audit's original pseudocode) |
| A2 | `CommitGraph.test.ts` exists with a vitest harness that can mock `listen`/`safeInvoke` | §1 / §2 test strategy | If absent, planner adds the file as a Wave 0 task (still tractable, ReviewPanel.test.ts provides the template) |
| A3 | `isTrunkError` helper is importable from `$lib/invoke.js` | §2 fix | Verified by grep in ReviewPanel.svelte:282 — true |
| A4 | The project uses `log::warn!` (not `tracing::warn!`); the planner must grep to confirm | §4 fix | Trivial swap if wrong |
| A5 | No `biome-ignore` precedent exists; introducing one is acceptable if Option B is rejected | §7 fix | Confirmed by grep (zero matches). Adding one sets precedent — surface this decision in plan/discuss |

---

## Open Questions

1. **Should INT-W2 (save_draft_comment race) be in scope?** Audit says no; the current toast does surface the error generically. Surface to user during discuss-phase: "Want richer composer UX for End-from-other-tab?" Recommend: no.
2. **Wave 3 isolation or collapse?** If user wants minimum total wallclock and accepts the risk of bundling, Wave 3 can merge into Wave 1 (no file overlap with Task A/B/C). Recommend: keep isolated; the only correctness fix deserves its own verification surface.
3. **Biome §7: Option B (runtime guard) vs Option C (annotation)?** Option B aligns with the project's "be explicit" tone (per `~/.agents/rules/coding_style.md`); Option C is one annotation. Surface to user.

---

## Sources

### Primary (HIGH confidence)
- `.planning/v0.13-MILESTONE-AUDIT.md` — scope source of truth
- `.planning/phases/66-commit-selection/66-REVIEW.md` — items 1-4 origin
- `.planning/phases/70-excerpt-resolution-markdown-render/70-REVIEW.md` — items 5-6 origin
- `src/components/CommitGraph.svelte:307-340, 1376-1389` — verified bug locations
- `src/components/ReviewPanel.svelte:224-292, 441-461` — verified existing fix pattern (Phase 73-01)
- `src-tauri/src/commands/review.rs:632-663` — verified seed_review_range structure
- `src-tauri/src/git/review.rs:147-214` — verified slice_diff structure
- `src/components/diff/CommentComposer.svelte:42-44` — verified biome warning locations
- `bunx biome ci src/components/diff/CommentComposer.svelte` (run 2026-05-27) — confirmed 3 warnings, exit 0
- `grep -rn "previewMarkdown\|panelMode\|ReviewDocPreview" src/` (run 2026-05-27) — confirmed 0 matches (70/WR-01 resolved)
- `.planning/config.json` — confirmed `tdd_mode: true`, `nyquist_validation: true`

### Secondary (MEDIUM confidence)
- `.planning/phases/73-review-lifecycle-end-review-cold-boot-resume/73-01-PLAN.md` (referenced for §1 pattern; should be re-read during plan-phase)
- `docs.rs/git2/0.19/git2/struct.Patch.html` — `Patch::from_diff` API for §5 (verify in plan-phase)

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Scope (still-applies verification): **HIGH** — each item grepped against current source
- Fix approaches: **HIGH** for §1, §2, §3 (pattern-replicate from ReviewPanel/sibling commands); **MEDIUM** for §5 (git2 Patch API needs version verification); **HIGH** for §7 (multiple valid options)
- Pitfalls: **HIGH** — cross-cutting risks A and B identified by reading line ranges directly
- Test strategy: **HIGH** for Rust (§3, §5); **MEDIUM** for TS (§1, §2 — depends on CommitGraph.test.ts existence)

**Research date:** 2026-05-27
**Valid until:** 2026-06-27 (low movement expected; tech-debt items don't drift)

## RESEARCH COMPLETE

**Phase:** 74 - Address v0.13 tech debt
**Confidence:** HIGH

### Key Findings
- 6 of 7 audit items still apply; 70/WR-01 is fully resolved by Phase 72's preview-pane deletion (drop from scope)
- 66/WR-01 (null-permissive listener) is already fixed in `ReviewPanel.svelte` during Phase 73-01; only `CommitGraph.svelte` still has the bug — pattern-replicate is sufficient
- 70/CR-01 is the only correctness fix with behavioral risk; TDD with a multi-hunk reproducer is mandatory
- 66/WR-03 fix incidentally closes INT-W1 (canonical-resolution ordering) — bundle as a side-effect note
- Biome warnings are exit-code-0 noise today, not gates; Option B (runtime guard helper) is recommended over `biome-ignore` per project tone

### File Created
`.planning/phases/74-address-v0-13-tech-debt-66-wr-01-04-70-cr-01-70-wr-01-biome-/74-RESEARCH.md`

### Confidence Assessment
| Area | Level | Reason |
|------|-------|--------|
| Scope (still-applies) | HIGH | Each item grepped against current source |
| Fix approaches | HIGH (5 items) / MEDIUM (§5 git2 API) | Pattern-replication from existing fixes; one needs API verification |
| Pitfalls / cross-cutting | HIGH | Cross-file/cross-line conflicts identified |
| Test strategy | HIGH (Rust) / MEDIUM (TS) | Rust harnesses verified; TS depends on CommitGraph.test.ts existence |

### Open Questions
1. INT-W2 in scope? — recommend NO (toast already surfaces error)
2. Wave 3 isolated or collapsed? — recommend ISOLATED
3. Biome Option B vs C? — recommend B; surface to user

### Ready for Planning
Research complete. Planner can now create PLAN.md files in three waves (or two if collapsing Wave 3 into Wave 1).
