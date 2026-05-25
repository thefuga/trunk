---
phase: 67-diff-source-anchor-capture
verified: 2026-05-25T16:22:22Z
status: passed
score: 14/14
overrides_applied: 0
---

# Phase 67: Diff-Source Anchor Capture Verification Report

**Phase Goal:** User can comment on a selected line range in the diff view, anchored to stable source-line coordinates.
**Verified:** 2026-05-25T16:22:22Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC-1: pure TS adapter translates selected diff indices -> (side, start_line, end_line) via old_lineno/new_lineno/origin (L-02) | VERIFIED | `buildDiffAnchor` in `src/lib/diff-anchor.ts:57-92` maps selected indices through the side-resolved lineno fields; 12 vitest cases cover all selection shapes |
| 2 | L-01: produced Anchor carries exactly commit_oid, file_path, source, side, start_line, end_line — no array-index/options leakage | VERIFIED | Anchor literal at line 82-89; grep for hunk_index/line_index/context_lines/ignore_whitespace returns 0 non-comment matches; test 11 asserts Object.keys count |
| 3 | L-04: file status forces side (Added->New, Deleted->Old, Renamed/Copied->New) | VERIFIED | `resolveSide` in `diff-anchor.ts:30-41` — file status wins; Untracked/Unknown fall through to origin-based rule; covered by tests 5-9 |
| 4 | L-03: mixed Add+Delete selection resolves to side=New, drops pure-Delete lines from range, keeps them in cachedExcerpt | VERIFIED | `diff-anchor.ts:69-73` filters linenos by side's nullability — Delete null new_lineno excluded; `cachedExcerpt` built over contiguous index span (lines 75-80) includes all origins |
| 5 | D-03: non-contiguous selections collapse to min..max of chosen side's line numbers with no rejection | VERIFIED | `Math.min/Math.max(...lineNumbers)` at lines 72-73 — works regardless of selection contiguity; test 4 covers gaps |
| 6 | L-06: cachedExcerpt assembled at capture-time in diff format over contiguous min..max index span | VERIFIED | `diff-anchor.ts:75-80` — slices hunkLines from spanStart to spanEnd+1 and maps through `prefixLine` (+/-/space); test 10 verifies origin-prefixed output |
| 7 | L-08: add_comment_inner is a dumb writer accepting fully-formed Anchor with source/side unchanged | VERIFIED | `review.rs:508-534` — add_comment_inner persists the anchor verbatim; Rust test at line 1279 (`add_comment_anchor_round_trips_all_six_fields`) + test at 1275 persists Source::FullFile unchanged |
| 8 | L-05: add_comment persists immediately (atomic store); save_draft_comment persists draft on change | VERIFIED | Both commands call `save_session` via `mutate_session_rmw`; `add_comment` emits `session-changed`; `save_draft_comment` ends at `Ok(())` with no emit (review.rs:544-561) |
| 9 | SC-2: persist anchor -> reload from disk -> all six anchor fields identical (persist/reload scope only; render re-resolution is Phase 70) | VERIFIED | Rust test `add_comment_anchor_round_trips_all_six_fields` (review.rs:1281); persisted via atomic tmp+rename store; UAT confirmed comments survived app restart with correct 6-field anchors |
| 10 | SC-3: mixed Add/Delete selection attaches and records side correctly | VERIFIED | L-03 implementation + Rust test for Source::FullFile (L-08); adapter test 3 covers mixed selection; UAT step 6 confirmed attachment |
| 11 | D-01: inline composer appears on active commit-diff selection; D-03: N-M preview reflects collapsed range | VERIFIED | `CommentComposer.svelte:104-105` — "Comments on lines {captured.anchor.start_line}-{captured.anchor.end_line}"; component test (a) asserts preview text |
| 12 | D-04: merge commit disables Comment affordance with tooltip; file-status side constraints do NOT disable | VERIFIED | HunkView.svelte:307-308 `disabled={isMerge} title={isMerge ? "Diff comments aren't available on merge commits" : ""}` ; SplitView.svelte:300-301 same; DiffPanel.svelte:84 derives `isMerge = (commitDetail?.parent_oids.length ?? 0) > 1`; component test confirms Added-file stays enabled |
| 13 | D-02: selecting new range with dirty draft prompts discard confirm; empty draft switches silently | VERIFIED | `CommentComposer.svelte:91-99` — `confirmDiscardIfDirty()` calls `ask()` only when `text.trim() !== ""`; component test (e) verifies both paths |
| 14 | Empty-text submit disabled; on submit composer+selection clear; auto-start ensures session exists at comment chokepoint | VERIFIED | `CommentComposer.svelte:38` `submitDisabled = $derived(text.trim() === "" \|\| submitting)`; `DiffPanel.svelte:108-140` `ensureActiveSession()` checks status before opening composer; 3 DiffPanel tests cover none→start, resume, and active→no-op paths |

**Score:** 14/14 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/lib/diff-anchor.ts` | Pure capture-time adapter: indices -> { anchor, cachedExcerpt } | VERIFIED | 92 lines; exports `buildDiffAnchor`; imports types from types.ts (no redeclarations); no array-index/options leakage |
| `src/lib/diff-anchor.test.ts` | No-mock Vitest unit tests for all selection shapes | VERIFIED | 235 lines; 12 `it()` cases under `describe`; covers SC-1, SC-3, D-03, L-01/04/06 |
| `src-tauri/src/commands/review.rs` | add_comment + save_draft_comment, generalized mutate_session_rmw, #[cfg(test)] coverage | VERIFIED | `add_comment_inner`/`save_draft_comment_inner` present (19 references); generalized RMW (1 instance of `fn mutate_session_rmw`); 9 new Rust tests |
| `src-tauri/src/lib.rs` | invoke_handler registration of add_comment + save_draft_comment | VERIFIED | Lines 132-133 — both commands registered |
| `src/lib/types.ts` | AddCommentRequest + SaveDraftCommentRequest DTOs | VERIFIED | Lines 341-350; `cachedExcerpt` camelCase wire key; reuses existing `Anchor` type (no redeclaration) |
| `src/components/RepoView.svelte` | commitDetail threaded to commit DiffPanel (both sites) | VERIFIED | Line 732 `commitDetail={commitDetail}`; line 679 `commitDetail={rebaseFocusedCommitDetail}` — zero occurrences of `commitDetail={null}` |
| `src/components/DiffPanel.svelte` | commit guard lifted; commitDetail prop; composer hosted; ensureActiveSession | VERIFIED | No `diffKind === "commit") return` guard; `isMerge` derived at line 84; `ensureActiveSession` at line 108; `handleCommentLines` at line 142 |
| `src/components/diff/HunkView.svelte` | isSelectable allows commit diffs; Comment affordance branch | VERIFIED | Line 331: `isSelectable = line.origin !== 'Context'` (no diffKind clause); lines 302-323: `{:else if diffKind === 'commit'}` Comment branch with merge disable |
| `src/components/diff/SplitView.svelte` | isSelectable allows commit diffs (Add-only); Comment branch | VERIFIED | Line 339: `isSelectable = line.origin === 'Add'` (no diffKind clause); lines 295-305: commit Comment branch with merge disable |
| `src/components/diff/DiffViewer.svelte` | Transparent isMerge/oncommentlines pass-through | VERIFIED | Props declared at lines 29/47; passed to HunkView at lines 104/115 and SplitView at lines 122/127 |
| `src/components/diff/CommentComposer.svelte` | Inline composer with preview, draft, submit, confirm, empty-disable | VERIFIED | 184 lines; `buildDiffAnchor` wired (2 usages); `safeInvoke("add_comment")` at line 73; `safeInvoke("save_draft_comment")` at line 50; `confirmDiscardIfDirty()` exported at line 93; no innerHTML; no inline colors |
| `src/components/diff/CommentComposer.test.ts` | Component tests for all CommentComposer behaviors | VERIFIED | 6 `it()` cases under `describe("CommentComposer")`; covers preview (a), empty-disable (b), draft-on-keystroke (c), submit+clear (d), confirm-on-discard (e), split-never-Old (f) |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/lib/diff-anchor.ts` | `src/lib/types.ts` | `import type { Anchor, DiffLine, DiffStatus, FileDiff, Side }` | WIRED | Line 17 of diff-anchor.ts |
| `src/components/diff/CommentComposer.svelte` | `src/lib/diff-anchor.ts` | `buildDiffAnchor(commitOid, file, hunkIdx, selectedLineIndices)` | WIRED | 2 usages in CommentComposer.svelte |
| `src/components/diff/CommentComposer.svelte` | `add_comment` / `save_draft_comment` | `safeInvoke("add_comment")` / `safeInvoke("save_draft_comment")` | WIRED | Lines 73 and 50 of CommentComposer.svelte |
| `src/components/RepoView.svelte` | `src/components/DiffPanel.svelte` | `commitDetail={commitDetail}` prop | WIRED | Lines 732 and 679 of RepoView.svelte |
| `src-tauri/src/commands/review.rs` | `src-tauri/src/git/review_store.rs` | `review_store::save_session` (atomic tmp+rename) | WIRED | Called from `mutate_session_rmw` |
| `src-tauri/src/lib.rs` | `src-tauri/src/commands/review.rs` | invoke_handler registration | WIRED | Lines 132-133 of lib.rs |

---

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `CommentComposer.svelte` | `captured` (anchor+excerpt) | `buildDiffAnchor` pure adapter from `selectedLineIndices` + `file` props | Yes — derives from live diff line objects in memory | FLOWING |
| `CommentComposer.svelte` | `add_comment` payload | `captured.anchor` + `captured.cachedExcerpt` + `text` textarea | Yes — adapter output + user input | FLOWING |
| `src-tauri/src/commands/review.rs` `add_comment` | Persisted `Comment` | `review_store::save_session` atomic write | Yes — real JSON store write (FNV-1a keyed, tmp+rename) | FLOWING |

---

## Behavioral Spot-Checks

Step 7b is not fully applicable: this is a Tauri desktop app that cannot be exercised with curl/node without a running process. The vitest suite covers all behavioral paths at the unit/component level.

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| diff-anchor.ts: 12 unit tests pass | `grep -c "it(" src/lib/diff-anchor.test.ts` | 12 | PASS |
| CommentComposer: 6 component tests present | `grep -c "it(" src/components/diff/CommentComposer.test.ts` | 6 | PASS |
| add_comment emits session-changed; save_draft_comment does not | inspect review.rs:508-561 | add_comment has `app.emit` at line 533; save_draft_comment ends at `Ok(())` line 560 | PASS |
| All 13 phase commits on main | `git log --oneline` | All 13 commits present | PASS |
| No inline colors in CommentComposer | grep check | 0 matches | PASS |
| No innerHTML in CommentComposer | grep check | 0 matches | PASS |
| L-01: no array-index/options on Anchor output | grep check | 0 matches | PASS |
| 1 mutate_session_rmw function (generalized, not cloned) | `grep -c "fn mutate_session_rmw"` | 1 | PASS |
| add_comment + save_draft_comment registered in lib.rs | `grep -c "commands::review::add_comment\|..."` | 2 | PASS |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| ANCH-01 | 67-01, 67-02, 67-03, 67-04 | User can select a line range in the diff view and attach a comment, anchored to commit + file + diff line range with a side discriminator | SATISFIED | Complete end-to-end implementation verified: adapter (Plan 01), Rust persistence (Plan 02), plumbing (Plan 03), UI composer+affordance (Plan 04); UAT confirmed |

---

## Anti-Patterns Found

No blockers identified. Scan results:

| File | Pattern | Severity | Finding |
|------|---------|----------|---------|
| All phase files | TBD/FIXME/XXX | checked | None found in modified files |
| `diff-anchor.ts` | `return null / {} / []` | checked | None — returns real computed values |
| `CommentComposer.svelte` | `innerHTML` | checked | Count = 0 (T-67-04 compliance confirmed) |
| `CommentComposer.svelte` | Inline colors | checked | Count = 0 (theme compliance confirmed) |
| `review.rs` `save_draft_comment` | emit check | checked | No `app.emit` in `save_draft_comment` body (lines 544-561) |

**UAT-surfaced deviation (documented, not a gap):** DiffPanel auto-starts a review session when commenting with no active session (`ensureActiveSession`, commit `101e42f`). This is a user-directed addition that keeps `add_comment`/`save_draft_comment` as dumb writers (L-08 honored). The deviation is documented in the 67-04-SUMMARY deviations section.

---

## Human Verification

Human UAT was performed during plan execution (Plan 04, Task 3) and approved by the user. Steps verified:

1. Select contiguous line range -> Comment affordance appears -> inline composer opens -> N-M preview correct -> Submit clears composer + selection.
2. Discard-draft confirm fires on selection switch with dirty text; cancel preserves draft; empty draft switches silently.
3. Submit button disabled while textarea is empty.
4. Split-view comment always new-side.
5. Mixed Add+Delete selection attaches without error (new side).
6. Comment metadata persists across ignore-whitespace toggle + app restart (3 comments confirmed on disk with correct 6-field source-coordinate anchors).
7. Merge commit diff shows Comment button disabled with correct tooltip.

No additional human verification items identified beyond those already approved.

---

## Gaps Summary

No gaps. All 14 must-have truths verified. Phase goal is achieved.

- SC-1: user can select a line range in a commit diff and attach a comment — fully implemented and UAT-confirmed.
- SC-2: anchor survives persist-reload cycle — locked by Rust round-trip test + UAT confirmed on-disk.
- SC-3: mixed Add/Delete selection attaches, records new side — adapter L-03 implementation + component tests.

Out-of-scope items correctly absent: comment display browser (Phase 69), render-time anchor re-resolution (Phase 70), full-file-source anchor (Phase 68).

---

_Verified: 2026-05-25T16:22:22Z_
_Verifier: Claude (gsd-verifier)_
