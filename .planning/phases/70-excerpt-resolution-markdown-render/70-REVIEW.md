---
phase: 70-excerpt-resolution-markdown-render
reviewed: 2026-05-26T14:36:19Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - src-tauri/src/git/review.rs
  - src-tauri/src/git/mod.rs
  - src-tauri/src/commands/review.rs
  - src-tauri/src/lib.rs
  - src/lib/review-session.svelte.ts
  - src/lib/review-session.svelte.test.ts
  - src/components/ReviewDocPreview.svelte
  - src/components/ReviewPanel.svelte
  - src/components/ReviewPanel.test.ts
  - src/components/RepoView.svelte
findings:
  critical: 1
  warning: 7
  info: 5
  total: 13
status: issues_found
---

# Phase 70: Code Review Report

**Reviewed:** 2026-05-26T14:36:19Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

Phase 70 ships three artifacts: a pure Rust renderer (`git/review.rs`, ~860 lines + 30 tests), one Tauri IPC command (`generate_review_doc` in `commands/review.rs`), and a Svelte preview swap with a Generate button. The renderer is well-disciplined — `classify_anchor` gate, `slice_lines` bounds-checks, `fence_length` linear scan, no panics, no `tauri::*` or `syntax::*` imports — and the test suite covers the documented happy paths and L-04/L-05/L-09 failure routes.

Adversarial review surfaces one concrete correctness defect (`slice_diff` keeps every opposing-side `±` line in the entire file, not just those near the anchored hunk) plus a TS lifecycle leak (`previewMarkdown` is keyed on the rune, but the rune outlives a `repoPath` swap inside `RepoView`). Several smaller defensiveness, code-quality, and test-isolation gaps round out the warnings. The IPC layer's TOCTOU between `canonical_repo_path` and `Repository::open(&path)` is consistent with the codebase's existing pattern (so flagged as INFO, not a blocker).

The renderer's hexagonal layering is clean (the pure `render(&ReviewSession, &git2::Repository) -> String` has no IPC/UI dependencies, and the spawn_blocking IPC at the adapter boundary is the only invocation path). The deliberate no-escape stance for markdown injection in `comment.text` is documented and consistent with the "AI agent recipient" decision in DOC-01.

## Critical Issues

### CR-01: `slice_diff` keeps opposing-side `±` lines from EVERY hunk in the file, not just near the anchored range

**File:** `src-tauri/src/git/review.rs:179-204`

**Issue:** The line-filter inside `diff.foreach`'s line callback is:

```rust
let in_range = match lineno {
    Some(n) => n >= start_line && n <= end_line,
    None => matches!(
        (side.clone(), line.origin()),
        (Side::New, '-') | (Side::Old, '+')
    ),
};
```

For a `Side::New` anchor spanning lines [10, 12]:
- Same-side lines (`+` and ` ` with `new_lineno`) are correctly filtered to [10, 12].
- Opposing-side lines (`-` with `new_lineno == None`) are kept **whenever `side==New && origin=='-'`** — with no positional check at all.

If the diff produces multiple hunks (e.g. a file with edits at lines 5, 60, and 200, where only line 11 is anchored), every `-` line from every hunk in the file flows into the output. The doc-comment claims "the opposing-side `-`/`+` rows are kept per Phase 67 L-03 so the body matches what the cached_excerpt looked like at capture", but the test fixture (`slice_diff_returns_requested_range`) only exercises a single-hunk file (`b"old\n"` → `b"new\n"`), so the unfiltered-hunk case is untested.

Result: the generated markdown's resolvable Diff sections may contain large unrelated chunks of foreign deletions/additions, defeating the "anchored excerpt" framing the doc promises to the AI consumer.

**Reproducer hint:** add a test that creates a parent→child with `foo.rs` changing at lines 1 and 50, then `slice_diff` for an anchor on `Side::New, 50, 50`. The output will include the `-` line from the line-1 hunk, even though it has nothing to do with the anchored range.

**Fix:** Gate "no-lineno opposing-side" lines by the hunk they belong to. Capture hunk header bounds in the hunk callback (the `_h` parameter is the `DiffHunk` — read `new_start`/`new_lines` for Side::New, `old_start`/`old_lines` for Side::Old), store the "current hunk overlaps the anchored range" flag in the outer closure, and only emit opposing-side rows when that flag is true.

```rust
// Pseudocode for the fix shape (the actual implementation needs to thread
// the hunk-overlap flag through the foreach closures via RefCell or by
// restructuring to a single-pass DiffPrint walk).
let mut hunk_overlaps = false;
diff.foreach(
    &mut |_d, _p| true,
    None,
    Some(&mut |_d, h| {
        let (h_start, h_lines) = match side {
            Side::New => (h.new_start(), h.new_lines()),
            Side::Old => (h.old_start(), h.old_lines()),
        };
        let h_end = h_start + h_lines.saturating_sub(1);
        hunk_overlaps = h_start <= end_line && h_end >= start_line;
        true
    }),
    Some(&mut |_d, _h, line| {
        let in_range = match lineno {
            Some(n) => n >= start_line && n <= end_line,
            None => hunk_overlaps && matches!(
                (side.clone(), line.origin()),
                (Side::New, '-') | (Side::Old, '+')
            ),
        };
        // …rest unchanged
        true
    }),
)
```

Also add a regression test with a multi-hunk file proving unrelated `-`/`+` rows are excluded.

## Warnings

### WR-01: `previewMarkdown` is keyed on the rune's lifetime, not on `repoPath` — preview leaks across repo swaps inside the same `RepoView`

**File:** `src/lib/review-session.svelte.ts:65-103` + `src/components/RepoView.svelte:86`

**Issue:** `RepoView` creates exactly one `reviewSession` rune at the top level: `const reviewSession = createReviewSession();`. The rune's `previewMarkdown` is mutated by `generate(repoPath)` but never cleared on `repoPath` change. The rune's only invalidation path is `setReviewActive(false)`, which is bound to the View-menu "Start/End Code Review" toggle — not to repo swaps.

If `RepoView` survives a `repoPath` prop change (the common pattern in Tauri SPAs where the parent doesn't `{#key repoPath}` the view, and indeed the existing `$effect(() => { void repoPath; reload(); })` in `ReviewPanel` is built around the survives-prop-change model), then opening repo A → Generate → switch to repo B → toggle Review will show repo A's generated markdown for repo B.

The rune's docstring claims "the rune is reused across activate-toggles within the same logical session; only the explicit teardown clears" — but does not define "logical session" in terms of `repoPath`.

**Fix:** Either (a) clear `previewMarkdown` and reset `panelMode` to `"list"` inside the `ReviewPanel`'s repo-change `$effect`, by exposing a `clearPreview()` action on the rune; or (b) recreate the rune per repoPath by moving `createReviewSession()` to an effect keyed on `repoPath` (more invasive, may break the share-with-host pattern); or (c) add a `{#key repoPath}` block around the `RepoView` instance at the parent level. Option (a) is the smallest correct change.

### WR-02: `generate_review_doc` re-opens the repo by raw `path` inside `spawn_blocking` without re-checking RepoState — TOCTOU on `force_close_repo`

**File:** `src-tauri/src/commands/review.rs:1000-1041`

**Issue:** The command checks `canonical_repo_path(&path, &state_map)` outside spawn_blocking, then inside the blocking task does `git2::Repository::open(&path)`. Between those two calls, `force_close_repo` may have removed the repo from `RepoState`. The blocking task will still successfully open the on-disk repo (the path still exists), bypass the "must be open" invariant the canonical check is supposed to enforce, and return a generated doc against a repo the user has explicitly closed.

The same shape exists in `list_session_commits` and `resolve_session_comments`, so this is an inherited pattern, not a regression. It is still a real defect in the contract.

**Fix:** Move the `canonical_repo_path` check INSIDE `spawn_blocking` so the state_map clone is consulted at the same time as `Repository::open`, OR snapshot the canonical path under the lock and pass it (not the raw path) to `Repository::open`. The second option also fixes a smaller bug: symlinked repos currently get opened by the raw path, not the canonical one — `git2::Repository::open` follows symlinks, so this is mostly harmless, but the canonical path is what the rest of the session state is keyed on.

### WR-03: `out.push_str(&String::from_utf8_lossy(line.content()))` in `slice_diff` silently substitutes invalid UTF-8 with U+FFFD in the rendered diff

**File:** `src-tauri/src/git/review.rs:201`

**Issue:** Lines whose content includes non-UTF-8 bytes (legitimately possible: git stores blob bytes verbatim, and diffs of partly-binary text files can yield such lines) are lossy-decoded and emitted with U+FFFD substitution. Unlike the `slice_full_file` Binary short-circuit (which routes to the `[binary file, no excerpt]` placeholder via `blob.is_binary()`), `slice_diff` has no `is_binary` gate — git2's `Diff::foreach` will surface lines from a binary diff unless `DiffOptions::force_text(false)` is set. The diff fence will then contain garbled lossy characters embedded in `+/-/ ` rows, which is exactly the "wrong section" failure L-05 was supposed to prevent.

**Fix:** Either gate `slice_diff` on `blob.is_binary()` for both sides up-front (matching the `slice_full_file` semantics), routing to `ExcerptError::Binary`, OR detect a binary diff via `DiffDelta::is_binary()` in the file callback and return `ExcerptError::Binary` to route into the resolved-section binary placeholder.

### WR-04: Test `_empty_commit_helper_is_used` is a dead-code-warning suppressor masquerading as a test

**File:** `src-tauri/src/git/review.rs:1658-1662`

**Issue:** `empty_commit` (line 615) is declared but never called by any non-marker test. The author added `_empty_commit_helper_is_used` purely to make the compiler stop emitting a warning. This is the smell that `growing-object-oriented-software-guided-by-tests` calls "dead test code": the helper exists to support a future test that was never written, so either the helper is unused or the test it's supposed to support is missing.

**Fix:** Either delete `empty_commit` AND `_empty_commit_helper_is_used` (it's `commit_with_file` that's actually used everywhere — empty trees aren't needed by the tests that exist), or add the real test it was meant for (e.g. a root-empty-tree → child-with-file FileGone path that's distinct from the existing `slice_diff_handles_root_commit`). Leaving the marker test is the wrong middle ground.

### WR-05: `fence_language` has a thin, ad-hoc extension table — common extensions silently fall through to `"text"`

**File:** `src-tauri/src/git/review.rs:57-80`

**Issue:** Files with extensions `.c`, `.cpp`, `.h`, `.hpp`, `.java`, `.kt`, `.scala`, `.dart`, `.rb`, `.swift`, `.php`, `.cs`, `.ex`, `.exs`, `.lua`, `.r`, `.sql`, `.zig` (and many more) all render as `\`\`\`text`. The recipient (an AI coding agent) loses the language hint, hurting downstream syntax-aware reading and grounding. The doc-comment notes this is "hand-rolled per L-10 (no syntect call)" — fine — but the table is short enough to be incomplete, with no obvious selection rationale (`.svelte` is included but `.vue` isn't; `.toml`/`.yaml` are in but `.ini`/`.xml` aren't).

**Fix:** Extend the table to cover the next-most-common 15–20 extensions. Better still, lift the language map to a single `const` slice/HashMap so the contract is one block of data, not buried in a 23-arm match. (This is NOT a syntect import — `L-10` bans calling into `git::syntax`, but a static table is just data.)

### WR-06: `repo_name` returns the workdir directory basename, which leaks the local filesystem layout into the AI-consumer doc

**File:** `src-tauri/src/git/review.rs:264-270` + `:382`

**Issue:** `# Code review: trunk` is the H1 today. For a repo cloned into `~/code/foo-bar-internal-secret`, the H1 becomes `# Code review: foo-bar-internal-secret`. If the user shares the generated markdown (the entire feature exists to be shared with an AI agent), the local directory name flows out unintentionally. The repo's actual name should come from `origin` remote URL or be omitted; the workdir basename is a privacy/info-leak concern in any shared artifact.

**Fix:** Either (a) accept a configured display name (passed from the frontend) rather than infer from disk; (b) parse `git config remote.origin.url` and use the trailing path component; or (c) default to a generic title like `# Code review` and let the user rename in their copy. Option (b) is the most useful but adds git2 surface; (c) is the safest by default.

### WR-07: `commit.summary()` returns `None` for commits with no message subject; the unwrap_or fallback `(subject unavailable)` is correct but the wider `unwrap_or_else` chain swallows real errors

**File:** `src-tauri/src/git/review.rs:394-398` and `:501-505`

**Issue:** The pattern
```rust
git2::Oid::from_str(oid_str).ok()
    .and_then(|oid| repo.find_commit(oid).ok())
    .and_then(|c| c.summary().map(String::from))
    .unwrap_or_else(|| "(subject unavailable)".to_string());
```
collapses three distinct failures — unparseable OID, missing commit, missing/non-UTF-8 summary — into one phrase. Two of these (unparseable OID, missing commit) should never happen for OIDs that came from `session.commits` (they were added through validate-then-walk); seeing `(subject unavailable)` for one of those is a silent data-loss signal that something corrupted the session file. The Phase 65 "never silently destroy" stance is partially defeated here.

**Fix:** Either log/emit a probe event when the find_commit branch fails (the renderer is no-panic, so don't return Err — but a probe call is fine), or distinguish the phrases ("commit gone" vs "no subject") so the AI consumer and the human auditor can both see which failure mode fired. The same collapse exists in the commit-level section at `:501`.

## Info

### IN-01: `AddCommentRequest.path` is set by the caller but never read by `add_comment_inner`

**File:** `src-tauri/src/commands/review.rs:496-501` + `:526-542`

**Issue:** The `path` field on the bundle is dead — `add_comment_inner` operates on `canonical: &Path`, never touching `req.path`. The bundle's reason for existing is the wedge contract, but `path` belongs in the thin-command argument list, not the inner request struct. Same for `SaveDraftCommentRequest.path`.

**Fix:** Drop the `path` field from both bundles; keep `text` and `anchor` only. Trivial cleanup, removes a misleading field.

### IN-02: Test helper duplication between `git/review.rs` and `commands/review.rs`

**File:** `src-tauri/src/git/review.rs:586-650` vs `src-tauri/src/commands/review.rs:1199-2185`

**Issue:** Both test modules re-implement `sig()`, `commit_with_file`, `make_repo`/`make_file_repo`, the `TestRepo` shape, and the `commit_level_comment` helper. The renderer's tests were "lifted from commands/review.rs:1135-2102" per the comment at line 582, then drifted in details (e.g. `commit_with_file` here takes `content: &[u8]` while the sibling takes `content: &str`). Two near-identical fixtures double the maintenance cost when types or signature semantics shift.

**Fix:** Extract the test helpers into a `src-tauri/src/git/test_support.rs` module gated by `#[cfg(test)]` or expose them via `pub(crate)` within a shared test-only module. Not a blocker — the duplication is contained — but worth doing in the next refactor pass.

### IN-03: `RepoView` passes `session={reviewSession}` only to `ReviewPanel`, but `ReviewDocPreview` is rendered conditionally inside `ReviewPanel` based on `session.state.panelMode` — the rune is the only coordination channel for the swap

**File:** `src/components/ReviewPanel.svelte:332-340` + `src/components/RepoView.svelte:842`

**Issue:** The preview-swap logic lives in `ReviewPanel.svelte` (panelMode === "preview" branch), which means the panel both reads AND mutates rune state. The "Back" button calls `session.showList()`, the Generate button calls `session.generate(repoPath)`. The component's responsibility surface is therefore "render comments OR render preview, depending on shared state" — not a clean separation, but acceptable given that both views own the same lifecycle. Worth a docstring noting `ReviewPanel` is the rune's view-controller, not just a list view.

**Fix:** Add a top-of-file comment in `ReviewPanel.svelte` explicitly stating that the component owns BOTH the list view AND the preview-swap dispatcher; the preview is rendered inline rather than at the RepoView level by design (so the rune doesn't need to escape the panel boundary).

### IN-04: `safeInvoke` mock test uses a JSON-stringified TrunkError but the production safeInvoke parses both stringified-JSON AND raw-object rejections — test fixture style is brittle

**File:** `src/lib/review-session.svelte.test.ts:69-72`

**Issue:** The test passes `mockRejectedValueOnce('{"code":"no_comments",…}')` — a STRING containing JSON. This works because the real `safeInvoke` parses TrunkError from a stringified payload. But Tauri's `invoke` rejection is typically a string-or-object union, and the safeInvoke implementation has a code path for each. Mocking only the string path leaves the object path uncovered.

**Fix:** Add a parallel test that rejects with `{code: "no_comments", message: "..."}` as a plain object, so both safeInvoke parsing paths are exercised by the generate-flow tests.

### IN-05: `parseExcerpt` in `ReviewPanel.svelte` re-implements the diff-prefix parsing inline, duplicating logic from `diff-anchor.ts`'s `prefixLine`

**File:** `src/components/ReviewPanel.svelte:151-177`

**Issue:** The panel parses `+/-/space` line prefixes locally. The cited comment says "carry +/-/space prefixes per `prefixLine` in diff-anchor.ts", but the consumer here does the inverse — splits the prefix back off. Two implementations of the same transform invite drift the moment the prefix vocabulary changes (e.g. if a `\` "no newline at end of file" marker ever needs to be handled).

**Fix:** Extract a single `parseDiffExcerpt(text)` helper colocated with `prefixLine` and import it from both producers and the panel.

---

_Reviewed: 2026-05-26T14:36:19Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
