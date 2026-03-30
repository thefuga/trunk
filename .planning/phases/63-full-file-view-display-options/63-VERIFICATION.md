---
phase: 63-full-file-view-display-options
verified: 2026-03-30T01:10:00Z
status: passed
score: 14/14 must-haves verified
re_verification:
  previous_status: passed
  previous_score: 11/11
  gaps_closed: []
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "Toggle Pilcrow button in toolbar while viewing a file diff with whitespace"
    expected: "Spaces appear as middle dots (·) and tabs appear as rightwards arrows (→); trailing spaces get a subtle red background"
    why_human: "CSS styling of .invisible-char and .trailing-ws requires a running Tauri app with real diff data to visually confirm"
  - test: "View a diff with very long lines, toggle the TextWrap button"
    expected: "Long lines wrap within the panel width with no horizontal scrollbar; gutter line numbers stay aligned at top of wrapped lines"
    why_human: "white-space: pre-wrap layout behavior requires visual inspection in a running app"
  - test: "Toggle showInvisibles and wordWrap on, close the app, reopen it"
    expected: "Both toggles remain active after restart"
    why_human: "LazyStore file write/read requires running Tauri desktop app; cannot simulate in unit tests"
  - test: "Select a file with whitespace-only changes, toggle the WS (Space) button"
    expected: "Diff content updates immediately; indentation-only changes disappear from the diff"
    why_human: "Requires IPC call to Tauri backend to verify ignore_whitespace re-fetch fires and updates diff content"
---

# Phase 63: Full File View & Display Options Verification Report

**Phase Goal:** Full-file view + display options — toggles for ignore whitespace, show invisibles, word wrap, hunk/full view switching with persistent preferences
**Verified:** 2026-03-30T01:10:00Z
**Status:** passed
**Re-verification:** Yes — re-verification of previous pass (score 11/11 expanded to 14/14 covering Plan 03 must-haves)

## Goal Achievement

### Observable Truths

All truths span Plans 01, 02, and 03 (gap-closure plan).

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | Hunk and line staging buttons are disabled with tooltip when whitespace ignore is active | VERIFIED | `stagingDisabled = $derived(hunkOperationInFlight \|\| ignoreWhitespace)` at HunkView.svelte line 51; 26 references to `stagingDisabled`; tooltip text "Staging is disabled while whitespace changes are ignored"; Stage/Unstage File also guarded in DiffToolbar.svelte lines 92-113 |
| 2  | User can toggle word wrap on/off and it applies to all diff view modes | VERIFIED | `white-space: {wordWrap ? 'pre-wrap' : 'pre'}` in HunkView.svelte line 295 and FullFileView.svelte line 70; `wordWrap` prop threaded DiffPanel → DiffViewer → HunkView/FullFileView |
| 3  | DiffToolbar shows three toggle buttons (WS, invisibles, word wrap) after segmented control | VERIFIED | Space, Pilcrow, TextWrap icons imported and rendered (DiffToolbar.svelte lines 3 and 60-83); `class="toggle-btn"` appears 3 times; separated from segmented control by `.toolbar-divider` |
| 4  | Toggling WS or view mode to/from full triggers a diff re-fetch | VERIFIED | `handleIgnoreWhitespaceChange` and `handleViewModeChange` both call `ondiffoptionschange?.()` in DiffPanel.svelte lines 82 and 88; RepoView.svelte line 662 wires callback to `refetchFileDiff` |
| 5  | All new display preferences (showInvisibles, wordWrap) persist across sessions | VERIFIED | `getDiffShowInvisibles`/`setDiffShowInvisibles` and `getDiffWordWrap`/`setDiffWordWrap` in store.ts lines 302-322; loaded via `Promise.all` in DiffPanel.svelte `$effect` on mount |
| 6  | Full file view shows entire file as one scrollable document with changed lines highlighted | VERIFIED | FullFileView.svelte: `{@const allLines = fd.hunks.flatMap(h => h.lines)}` at line 50; add/delete background colors applied per line origin; 127 lines, substantive |
| 7  | Full file view has no hunk headers, no section dividers, no staging buttons | VERIFIED | FullFileView.svelte contains no hunk header rendering, no Stage/Discard/Unstage buttons; confirmed by VIEW-04 test block |
| 8  | Full file view has two-column gutter with old + new line numbers | VERIFIED | FullFileView.svelte renders `{line.old_lineno ?? ''}` and `{line.new_lineno ?? ''}` in two separate `<span>` gutter elements using `gutterWidth()` at lines 74-75 |
| 9  | Invisible characters are rendered as middle dot and arrow when show invisibles is active | VERIFIED | `splitInvisibles` in diff-utils.ts: spaces → U+00B7 (·), tabs → U+2192 (→); integrated in both HunkView.svelte and FullFileView.svelte via `{#if showInvisibles}` guards |
| 10 | Trailing whitespace gets a subtle warning background when show invisibles is active | VERIFIED | `trailingWhitespaceStart` utility detects boundary; `.trailing-ws { background-color: var(--color-trailing-ws-bg) }` in both views; `--color-trailing-ws-bg: rgba(248, 113, 113, 0.12)` in app.css line 77 |
| 11 | Tests verify all four phase requirements (VIEW-04, WHSP-02, WHSP-03, DISP-02) | VERIFIED | DiffPanel.test.ts: VIEW-04 describe block (line 616), WHSP-02 block (line 675), WHSP-03 unit tests (line 554), DISP-02 block (line 771); 402 tests pass |
| 12 | Toggling Ignore Whitespace re-fetches diff and indentation-only changes disappear | VERIFIED | diff.rs line 39: `opts.ignore_whitespace(req.ignore_whitespace)` (correct git -w API, not -b); test `diff_unstaged_ignores_indentation_whitespace` at test_diff.rs line 342 passes; `ignore_whitespace_change` absent from codebase |
| 13 | Switching view mode updates the diff display live without closing/reopening | VERIFIED | `handleViewModeChange` in DiffPanel.svelte is `async`, awaits `setDiffViewMode` and `setDiffShowFullFile` before calling `ondiffoptionschange?.()`, ensuring store write completes before re-fetch |
| 14 | Toggle buttons show their persisted active state immediately on open, no flicker | VERIFIED | `prefsLoaded = $state(false)` at line 49; set to `true` after `Promise.all` resolves at line 72; `{#if prefsLoaded}` guard at line 347 hides toolbar until preferences load |

**Score:** 14/14 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/lib/store.ts` | getDiffShowInvisibles/setDiffShowInvisibles and getDiffWordWrap/setDiffWordWrap | VERIFIED | Lines 302-322; `DIFF_SHOW_INVISIBLES_KEY` and `DIFF_WORD_WRAP_KEY` present; all 4 functions exported |
| `src/app.css` | CSS custom properties for invisible character markers and trailing whitespace | VERIFIED | `--color-invisible: rgba(139, 148, 158, 0.5)` at line 74; `--color-trailing-ws-bg: rgba(248, 113, 113, 0.12)` at line 77 |
| `src/components/diff/DiffToolbar.svelte` | Three toggle buttons (Space, Pilcrow, TextWrap) with active/inactive states | VERIFIED | 3 `toggle-btn` buttons; Space, Pilcrow, TextWrap imported; `class:active` bindings; `.toggle-btn.active` CSS at line 222 |
| `src/components/DiffPanel.svelte` | State owner for ignoreWhitespace, showInvisibles, wordWrap with LazyStore loading and prefsLoaded guard | VERIFIED | `$state(false)` for all three at lines 46-48; `prefsLoaded` guard at line 347; `$effect` loads all four prefs via `Promise.all`; store functions imported |
| `src/components/diff/HunkView.svelte` | Staging buttons disabled when ignoreWhitespace; word wrap CSS toggle | VERIFIED | `stagingDisabled` derived at line 51; 26 references; `"Staging is disabled..."` tooltip present; `wordWrap ? 'pre-wrap' : 'pre'` at line 295 |
| `src/lib/diff-utils.ts` | splitInvisibles() utility for invisible character rendering | VERIFIED | 73 lines; `InvisibleSegment` type; `splitInvisibles` and `trailingWhitespaceStart` exported |
| `src/components/diff/FullFileView.svelte` | Continuous document renderer flattening all hunks into single list | VERIFIED | 127 lines (min_lines: 50 exceeded); `flatMap` at line 50; no hunk headers; two-column gutter |
| `src/components/DiffPanel.test.ts` | Test cases for VIEW-04, WHSP-02, WHSP-03, DISP-02 | VERIFIED | All 4 requirement describe blocks present; 402 tests pass |
| `src-tauri/src/commands/diff.rs` | `opts.ignore_whitespace()` (git -w) instead of `ignore_whitespace_change()` (git -b) | VERIFIED | Line 39: `opts.ignore_whitespace(req.ignore_whitespace)`; `ignore_whitespace_change` does not appear anywhere in the file |
| `src-tauri/tests/test_diff.rs` | Test covering indentation-only whitespace ignore | VERIFIED | `diff_unstaged_ignores_indentation_whitespace` at line 342; verifies indentation-only changes are invisible with `ignore_whitespace: true` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/components/DiffPanel.svelte` | `src/lib/store.ts` | `$effect` loading preferences on mount | WIRED | `getDiffShowInvisibles` and `getDiffWordWrap` in `Promise.all` at lines 62-74 |
| `src/components/DiffPanel.svelte` | `src/components/diff/DiffToolbar.svelte` | props: ignoreWhitespace, showInvisibles, wordWrap + callbacks | WIRED | `onshowinvisibleschange={handleShowInvisiblesChange}` and `onwordwrapchange={handleWordWrapChange}` passed at lines 357-359 |
| `src/components/DiffPanel.svelte` | `src/components/RepoView.svelte` | ondiffoptionschange callback for re-fetching | WIRED | RepoView.svelte line 662: `ondiffoptionschange={async () => { ... refetchFileDiff ... }}` |
| `src/components/diff/HunkView.svelte` | DiffPanel ignoreWhitespace prop | disabled + title on staging buttons | WIRED | `stagingDisabled = $derived(hunkOperationInFlight \|\| ignoreWhitespace)` at line 51; 26 references |
| `src/components/diff/FullFileView.svelte` | `src/lib/diff-utils.ts` | import splitInvisibles | WIRED | Line 3: `import { splitInvisibles, trailingWhitespaceStart } from "../../lib/diff-utils.js"` |
| `src/components/diff/HunkView.svelte` | `src/lib/diff-utils.ts` | import splitInvisibles | WIRED | Line 3: `import { splitInvisibles, trailingWhitespaceStart } from "../../lib/diff-utils.js"` |
| `src/components/diff/FullFileView.svelte` | fileDiffs prop | fd.hunks.flatMap(h => h.lines) for continuous rendering | WIRED | Line 50: `{@const allLines = fd.hunks.flatMap(h => h.lines)}`; rendered in `{#each allLines as line}` |
| `src/components/DiffPanel.svelte` | `ondiffoptionschange?.()` | async handlers await store writes first | WIRED | `handleViewModeChange` awaits `setDiffViewMode` and `setDiffShowFullFile` before callback; `handleIgnoreWhitespaceChange` awaits `setDiffIgnoreWhitespace` before callback |
| `src-tauri/src/commands/diff.rs` | `git2::DiffOptions` | `apply_request_options` | WIRED | Line 39: `opts.ignore_whitespace(req.ignore_whitespace)` inside `apply_request_options` function |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `src/components/diff/FullFileView.svelte` | `allLines` | `fd.hunks.flatMap(h => h.lines)` from `fileDiffs` prop | Yes (fileDiffs come from Tauri invoke upstream) | FLOWING |
| `src/components/DiffPanel.svelte` | `ignoreWhitespace`, `showInvisibles`, `wordWrap` | LazyStore via `getDiffShowInvisibles`, `getDiffWordWrap` in `Promise.all` | Yes (LazyStore reads persisted values from trunk-prefs.json) | FLOWING |
| `src/components/diff/DiffToolbar.svelte` | `ignoreWhitespace`, `showInvisibles`, `wordWrap` | Props from DiffPanel (gated by `prefsLoaded`) | Yes (DiffPanel state loaded from store) | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 402 tests pass (including all four phase requirement blocks) | `bun run test` | 402 passed (41 files) | PASS |
| svelte-check: 0 errors | `bun run check` | 0 errors, 28 warnings (all pre-existing in RebaseEditor.svelte) | PASS |
| Rust: ignore_whitespace() (not ignore_whitespace_change()) is used | `grep "ignore_whitespace" src-tauri/src/commands/diff.rs` | `opts.ignore_whitespace(req.ignore_whitespace)` at line 39; `ignore_whitespace_change` absent | PASS |
| store writes awaited before ondiffoptionschange fires | `grep "await setDiff" src/components/DiffPanel.svelte` | `await setDiffViewMode(mode)` line 78; `await setDiffIgnoreWhitespace(value)` line 87 | PASS |
| prefsLoaded guard prevents toolbar flicker | `grep "prefsLoaded" src/components/DiffPanel.svelte` | `{#if prefsLoaded}` wraps both DiffToolbar and DiffViewer at line 347 | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| WHSP-02 | 63-01-PLAN | Hunk/line staging is disabled with tooltip when whitespace ignore is active | SATISFIED | `stagingDisabled` derived in HunkView.svelte; "Staging is disabled while whitespace changes are ignored" tooltip; Stage/Unstage File guarded in DiffToolbar.svelte; WHSP-02 describe block passes |
| DISP-02 | 63-01-PLAN | User can toggle word wrap in the diff viewer | SATISFIED | `wordWrap` state in DiffPanel; `handleWordWrapChange` persists via `setDiffWordWrap`; `white-space: {wordWrap ? 'pre-wrap' : 'pre'}` in both HunkView and FullFileView; DISP-02 describe block passes |
| VIEW-04 | 63-02-PLAN | Full file view shows entire file with changed lines highlighted (via context_lines=MAX) | SATISFIED | FullFileView.svelte renders `fd.hunks.flatMap(h => h.lines)` as continuous document; add/delete backgrounds applied per line; no hunk headers; two-column gutter; VIEW-04 describe block passes |
| WHSP-03 | 63-02-PLAN | User can toggle display of invisible characters (spaces as dots, tabs as arrows) | SATISFIED | `splitInvisibles` in diff-utils.ts converts spaces to U+00B7 and tabs to U+2192; integrated in both HunkView and FullFileView when `showInvisibles` active; `.invisible-char` and `.trailing-ws` CSS applied; WHSP-03 unit tests pass |

**Orphaned requirements check:** REQUIREMENTS.md maps VIEW-04, WHSP-02, WHSP-03, DISP-02 to Phase 63. All four are claimed across plan frontmatter (63-01 claims WHSP-02, DISP-02; 63-02 claims VIEW-04, WHSP-03; 63-03 claims WHSP-03, VIEW-04, DISP-02 as gap-closure). No orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/lib/diff-utils.ts` | 39 | `return []` | Info | Guard clause for empty input in `splitInvisibles()` — correct defensive programming, not a stub |

No blockers or warnings found. All empty returns are legitimate guard clauses or unreachable fallbacks with real implementations below.

### Human Verification Required

#### 1. Invisible Character Visual Rendering

**Test:** Open any repository, view a file diff that contains spaces and tabs. Toggle the Pilcrow button in the diff toolbar.
**Expected:** Spaces appear as middle dots (·) and tabs appear as rightwards arrows (→) in the diff lines. Trailing spaces should have a subtle red background tint.
**Why human:** CSS styling of `.invisible-char` and `.trailing-ws` requires a running Tauri app with actual diff data to visually confirm rendering.

#### 2. Word Wrap Layout Correctness

**Test:** View a diff that contains very long lines. Toggle the TextWrap button.
**Expected:** Long lines wrap within the diff panel width without a horizontal scrollbar appearing. Gutter line numbers stay aligned to the top of wrapped lines.
**Why human:** Dynamic CSS `white-space: pre-wrap` and `align-items: flex-start` behavior requires visual inspection in a running app.

#### 3. Persistence Across Sessions

**Test:** Toggle showInvisibles and wordWrap on. Close the app completely and reopen it.
**Expected:** Both toggles remain active after restart (LazyStore persists to `trunk-prefs.json`).
**Why human:** Requires a running Tauri desktop app; LazyStore file write cannot be simulated in unit tests.

#### 4. Re-fetch on WS Toggle and View Mode Change

**Test:** Select a file with whitespace-only indentation changes. Toggle the WS (Space) button.
**Expected:** Diff content updates immediately; the indentation-only changes disappear from the diff (since `opts.ignore_whitespace` covers full whitespace, including indentation).
**Why human:** Requires live IPC call to Tauri backend to verify the re-fetch fires, picks up the updated `ignore_whitespace` flag, and returns the updated diff content.

### Gaps Summary

No gaps. All 14 must-have truths across Plans 01, 02, and 03 are verified. All 4 requirements (VIEW-04, WHSP-02, WHSP-03, DISP-02) are satisfied by substantive, wired, and data-connected implementations. The three UAT issues diagnosed and fixed in Plan 03 are all confirmed closed: the Rust API bug (`ignore_whitespace` not `ignore_whitespace_change`), the async race conditions in view mode and WS toggle handlers, and the toggle flicker on mount. 402 tests pass, svelte-check reports 0 errors.

---

_Verified: 2026-03-30T01:10:00Z_
_Verifier: Claude (gsd-verifier)_
