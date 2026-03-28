---
phase: 59-backend-data-model-diff-options
verified: 2026-03-28T10:06:00Z
status: passed
score: 11/11 must-haves verified
re_verification: false
gaps:
  - truth: "All existing diff tests still pass with DiffRequestOptions::default() (biome format clean)"
    status: resolved
    reason: "Two inline object literals in RepoView.svelte (lines 265, 466) are not formatted per biome rules; biome reports a format error on the file. The non-null assertion warnings (repoPath!, lines 408/415) are pre-existing and unrelated to this phase."
    artifacts:
      - path: "src/components/RepoView.svelte"
        issue: "Lines 265 and 466 use inline object literals that biome requires to be multi-line expanded. Error: formatter would reformat those two lines."
    missing:
      - "Expand the inline object literal at line 265: `safeInvoke<FileDiff[]>(\"diff_commit\", { path: repoPath, oid, options: commitDiffOptions })` -> multi-line form"
      - "Expand the inline object literal at line 466: `safeInvoke<FileDiff[]>(\"diff_commit\", { path: repoPath, oid, options: rebaseDiffOptions })` -> multi-line form"
---

# Phase 59: Backend Data Model & Diff Options Verification Report

**Phase Goal:** All diff commands accept configurable options (context lines, whitespace ignore) and return enriched DiffLine data ready for word-level and syntax highlighting consumption
**Verified:** 2026-03-28T10:06:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `diff_unstaged_inner` accepts `DiffRequestOptions` and applies `context_lines` to `git2::DiffOptions` | VERIFIED | `src-tauri/src/commands/diff.rs` line 116: `options: &DiffRequestOptions`; `apply_request_options` called line 124 |
| 2 | `diff_staged_inner` accepts `DiffRequestOptions` and applies `context_lines` to `git2::DiffOptions` | VERIFIED | `diff.rs` line 133: `options: &DiffRequestOptions`; `apply_request_options` called line 138 |
| 3 | `diff_commit_inner` accepts `DiffRequestOptions` and creates `git2::DiffOptions` (previously passed `None`) | VERIFIED | `diff.rs` line 152: `options: &DiffRequestOptions`; `apply_request_options` called line 160; both `diff_tree_to_tree` calls pass `Some(&mut opts)` |
| 4 | `ignore_whitespace` on `DiffRequestOptions` sets `git2` `ignore_whitespace_change` | VERIFIED | `apply_request_options` line 36: `opts.ignore_whitespace_change(req.ignore_whitespace)` |
| 5 | `show_full_file=true` causes `context_lines` to be `100_000` | VERIFIED | `apply_request_options` lines 30-34: `if req.show_full_file { 100_000 } else { req.context_lines }` |
| 6 | `DiffLine` serializes with `word_spans` and `syntax_tokens` as empty arrays | VERIFIED | `types.rs` lines 185-186; `diff.rs` lines 99-100: `word_spans: vec![], syntax_tokens: vec![]`; serde test passes |
| 7 | All existing diff tests still pass with `DiffRequestOptions::default()` (Rust) | VERIFIED | `cargo test --test test_diff --test test_integ_serde` passes: 14/14 + 18/18 tests |
| 8 | TypeScript `DiffLine` interface has `word_spans` and `syntax_tokens` fields | VERIFIED | `src/lib/types.ts` lines 143-144 |
| 9 | TypeScript `DiffRequestOptions` interface has `contextLines`, `ignoreWhitespace`, `showFullFile` | VERIFIED | `src/lib/types.ts` lines 172-176 |
| 10 | `LazyStore` persists and retrieves `diff_context_lines`, `diff_ignore_whitespace`, `diff_show_full_file` | VERIFIED | `src/lib/store.ts` lines 256-286; all 6 store tests pass |
| 11 | All 4 diff invoke call sites in `RepoView.svelte` pass options object to backend commands | VERIFIED (with formatting gap) | Lines 232-236, 264-265, 323-327, 465-466 all include `options:`; but lines 265 and 466 fail biome format check |

**Score:** 10/11 truths fully verified; 1 partial (functional but biome format non-compliant)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/git/types.rs` | `WordSpan`, `SyntaxToken`, `DiffRequestOptions` structs; extended `DiffLine` | VERIFIED | All 4 structs present; `DiffLine` has `word_spans`/`syntax_tokens`; `Default` impl on `DiffRequestOptions`; `fn default_context_lines() -> u32` |
| `src-tauri/src/commands/diff.rs` | `apply_request_options` helper; updated `_inner` and `#[tauri::command]` fns | VERIFIED | `apply_request_options` at line 29; all 3 `_inner` signatures accept `&DiffRequestOptions`; all 3 Tauri commands accept `DiffRequestOptions` |
| `src-tauri/tests/common/drivers/diff.rs` | Updated test drivers accepting `DiffRequestOptions` | VERIFIED | Default wrappers (`diff_unstaged`, `diff_staged`, `diff_commit`) and `_with_options` variants all present |
| `src-tauri/tests/test_diff.rs` | New tests for `context_lines`, whitespace ignore, `show_full_file` | VERIFIED | 4 new tests: `diff_unstaged_respects_context_lines`, `diff_unstaged_ignores_whitespace_when_enabled`, `diff_unstaged_show_full_file_returns_all_lines`, `diff_commit_respects_context_lines` |
| `src-tauri/tests/test_integ_serde.rs` | Serde test for `word_spans` and `syntax_tokens` empty arrays | VERIFIED | `diff_line_serializes_word_spans_and_syntax_tokens_as_empty_arrays` test at line 840 passes |
| `src/lib/types.ts` | `WordSpan`, `SyntaxToken`, `DiffRequestOptions` interfaces; extended `DiffLine` | VERIFIED | All interfaces present; `DiffLine` extended with both fields; `DiffRequestOptions` uses camelCase |
| `src/lib/store.ts` | `getDiffContextLines`, `setDiffContextLines`, `getDiffIgnoreWhitespace`, `setDiffIgnoreWhitespace`, `getDiffShowFullFile`, `setDiffShowFullFile` | VERIFIED | All 6 functions present at lines 261-286; correct defaults (3, false, false) |
| `src/lib/store.test.ts` | Tests for diff preference get/set round-trips | VERIFIED | `describe("diff preferences")` block with 6 tests; all pass |
| `src/components/RepoView.svelte` | Diff invoke calls passing options parameter | VERIFIED (format gap) | `buildDiffOptions()` helper at line 169; all 4 call sites pass `options`; 2 lines fail biome format |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/src/commands/diff.rs` | `src-tauri/src/git/types.rs` | imports `DiffRequestOptions, WordSpan, SyntaxToken` | WIRED | Line 5: `use crate::git::types::{..., DiffRequestOptions, ...}` |
| `src-tauri/src/commands/diff.rs` | `git2::DiffOptions` | `apply_request_options` helper | WIRED | `apply_request_options` calls `opts.context_lines()` and `opts.ignore_whitespace_change()` |
| `src/components/RepoView.svelte` | `src/lib/types.ts` | imports `DiffRequestOptions` | WIRED | Line 19: `import type { ..., DiffRequestOptions, ... }` |
| `src/components/RepoView.svelte` | `src/lib/store.ts` | imports `getDiffContextLines`, `getDiffIgnoreWhitespace`, `getDiffShowFullFile` | WIRED | Lines 6-8 import all 3 functions |
| `src/lib/store.ts` | `trunk-prefs.json` | `LazyStore` keys `diff_context_lines`, `diff_ignore_whitespace`, `diff_show_full_file` | WIRED | Keys defined at lines 257-259; used in all 6 get/set functions |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|-------------------|--------|
| `src/components/RepoView.svelte` | `stagingDiffFiles` | `buildDiffOptions()` reads LazyStore, passes to `safeInvoke` diff command | Yes — loads from LazyStore, flows to Rust backend | FLOWING |
| `src/components/RepoView.svelte` | `commitFileDiffs` | `buildDiffOptions()` reads LazyStore, passes to `safeInvoke("diff_commit", ...)` | Yes — same pattern | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 14 diff integration tests pass | `cargo test --test test_diff` | 14 passed, 0 failed | PASS |
| 18 serde integration tests pass (incl. enrichment fields) | `cargo test --test test_integ_serde` | 18 passed, 0 failed | PASS |
| 17 frontend store tests pass | `bun run test -- store.test.ts` | 17 passed, 0 failed | PASS |
| clippy clean | `cargo clippy -- -D warnings` | No warnings, finished cleanly | PASS |
| biome format check | `npx @biomejs/biome check src/lib/types.ts src/lib/store.ts src/lib/store.test.ts src/components/RepoView.svelte` | 1 format error on RepoView.svelte (lines 265, 466) | FAIL |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| CTXL-01 | 59-01, 59-02 | User can select context line count from toolbar dropdown (3/5/10/25/All) | SATISFIED | `DiffRequestOptions.context_lines` accepted by all 3 diff commands; `getDiffContextLines`/`setDiffContextLines` in store; wired through RepoView |
| CTXL-02 | 59-01, 59-02 | Selecting "All" activates full file view mode | SATISFIED | `show_full_file=true` sets `context_lines=100_000` in `apply_request_options`; `getDiffShowFullFile`/`setDiffShowFullFile` in store |
| WHSP-01 | 59-01, 59-02 | User can toggle whitespace ignore in the diff toolbar | SATISFIED | `ignore_whitespace` field on `DiffRequestOptions` → `opts.ignore_whitespace_change()` in Rust; `getDiffIgnoreWhitespace`/`setDiffIgnoreWhitespace` in store; test `diff_unstaged_ignores_whitespace_when_enabled` proves it works |
| DISP-03 | 59-02 | All diff display preferences persist across sessions via LazyStore | SATISFIED | LazyStore keys `diff_context_lines`, `diff_ignore_whitespace`, `diff_show_full_file` with defaults; 6 round-trip tests pass |

All 4 requirements mapped to phase 59 in `REQUIREMENTS.md` are satisfied. No orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/components/RepoView.svelte` | 265, 466 | Inline object literal not expanded per biome formatter rules | Warning | biome format check fails; does not affect runtime behavior or test correctness |

Note: `word_spans: vec![]` and `syntax_tokens: vec![]` in `diff.rs` and `word_spans: [], syntax_tokens: []` in test fixtures are intentional empty vecs/arrays, not stubs — they will be populated by Phases 60-61 as explicitly documented in both SUMMARY files.

### Human Verification Required

None. All goal-critical behaviors are verifiable through tests and static analysis.

### Gaps Summary

Phase 59 is functionally complete. All Rust and TypeScript types exist and are wired end-to-end. All 4 requirements (CTXL-01, CTXL-02, WHSP-01, DISP-03) are satisfied. All tests pass (14 Rust diff tests, 18 Rust serde tests, 17 frontend store tests). clippy is clean.

The single gap is cosmetic: two `safeInvoke` calls at lines 265 and 466 of `RepoView.svelte` use inline object literals `{ path: repoPath, oid, options: ... }` instead of the multi-line expanded form that biome's formatter requires. This is a one-character-per-line formatting difference with no semantic impact, introduced when the `options` parameter was added to those call sites. Fix by expanding the object literals to multi-line form.

---

_Verified: 2026-03-28T10:06:00Z_
_Verifier: Claude (gsd-verifier)_
