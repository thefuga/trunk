---
phase: 27-foundation-icons-toast-bug-fixes
verified: 2026-03-15T01:37:00Z
status: passed
score: 12/12 must-haves verified
gaps: []
human_verification:
  - test: "Visual icon rendering across all 7 components"
    expected: "All SVG icons render crisply at all DPIs; no Unicode glyphs visible in Toolbar, FileRow, StagingPanel, BranchSection, BranchRow, TabBar, PullDropdown"
    why_human: "Cannot verify visual DPI rendering or icon appearance programmatically"
  - test: "Toast auto-dismiss in live app"
    expected: "Toast appears on stash/pop/pull/push/branch-create/checkout and disappears after ~3 seconds"
    why_human: "setTimeout auto-dismiss requires running app; Vitest fake timers confirm logic but not real-time UX"
---

# Phase 27: Foundation — Icons, Toast & Bug Fixes — Verification Report

**Phase Goal:** Deliver ICON-01 (replace Unicode symbols with Lucide SVG icons across all 7 UI components), TOAST-01 (reactive toast notification system), FIX-01 (untracked files counted in dirty counts), and FIX-02 (suppress trailing resize divider on last commit graph column).
**Verified:** 2026-03-15T01:37:00Z
**Status:** ✅ PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `@lucide/svelte` installed and node_modules present | ✓ VERIFIED | `package.json` line 18: `"@lucide/svelte": "^0.577.0"`; `node_modules/@lucide/svelte/dist` exists |
| 2 | No Unicode HTML entities (&#...;) remain in all 7 components | ✓ VERIFIED | grep for `&#[0-9]+;` across all 7 files: zero matches |
| 3 | No Unicode symbol literals (▼▶✎×↓↑) remain in all 7 components | ✓ VERIFIED | grep for symbol chars and `\u219x` escapes: zero matches |
| 4 | All 7 components import from `@lucide/svelte` | ✓ VERIFIED | All 7 files contain `from '@lucide/svelte'` import |
| 5 | `toast.svelte.ts` exports `Toast`, `ToastKind`, `toasts`, `showToast` | ✓ VERIFIED | File exists, 30 lines, all 4 exports confirmed; `$state` reactive store with auto-dismiss |
| 6 | `Toast.svelte` renders overlay using `toasts.items` | ✓ VERIFIED | Uses `{#each toasts.items as toast}`, fixed `bottom-4 right-4 z-50`, kind-based styling |
| 7 | `App.svelte` mounts `<Toast />` unconditionally | ✓ VERIFIED | Line 11: `import Toast`; line 369: `<Toast />` outside `{#if repoPath}` block |
| 8 | Toolbar calls `showToast` for 5 operations (stash, pop, pull, push, branch) | ✓ VERIFIED | Lines 87/91 (runRemote), 106/109 (stash), 116/119 (pop), 133/135 (branch) — all success+error pairs |
| 9 | BranchSidebar calls `showToast` on checkout success and error | ✓ VERIFIED | Lines 117/126: `showToast('Checked out ' + branchName, 'success')` and `showToast('Checkout failed', 'error')` |
| 10 | All 5 toast unit tests pass | ✓ VERIFIED | `npx vitest run src/lib/toast.svelte.test.ts` → 5/5 GREEN |
| 11 | `get_dirty_counts_inner` uses `include_untracked(true)` + `WT_NEW` flag | ✓ VERIFIED | `staging.rs` lines 186-190: `StatusOptions::new()` with `include_untracked(true).recurse_untracked_dirs(true)`; line 206: `Status::WT_NEW` in unstaged accumulator |
| 12 | Rust test `get_dirty_counts_includes_untracked` passes (GREEN) | ✓ VERIFIED | `cargo test --lib staging` → 9/9 pass including `get_dirty_counts_includes_untracked` |
| 13 | `CommitGraph.svelte` has `lastVisibleColumn` derived state | ✓ VERIFIED | Lines 53-61: `ORDERED_COLUMNS`, `visibleColumns`, `lastVisibleColumn` all present |
| 14 | All trailing resize handles guarded except `message` column's inverted handle | ✓ VERIFIED | Lines 473/482/499/508/517: `{#if 'col' !== lastVisibleColumn}` guards; line 492: message handle always rendered |

**Score:** 14/14 truths verified (12 automated + 2 human-visual)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/lib/toast.svelte.ts` | Reactive store with showToast() and toasts.items | ✓ VERIFIED | 30 lines; `$state` reactive; exports `Toast`, `ToastKind`, `toasts`, `showToast`, `_resetToasts`; `setTimeout` auto-dismiss |
| `src/components/Toast.svelte` | Fixed overlay rendering active toasts | ✓ VERIFIED | 19 lines; `fixed bottom-4 right-4 z-50`; `{#each toasts.items}`; error/success kind styling; `fly` transition |
| `src/App.svelte` | Mounts `<Toast />` at top level | ✓ VERIFIED | Contains `<Toast />` at line 369, outside all conditionals |
| `package.json` | @lucide/svelte dependency | ✓ VERIFIED | Line 18: `"@lucide/svelte": "^0.577.0"` in `dependencies` |
| `src/components/Toolbar.svelte` | SVG icons for undo, redo, pull, push, branch, stash, pop | ✓ VERIFIED | Imports `Undo2, Redo2, ArrowDown, ArrowUp, GitBranch, Archive, PackageOpen`; all used at `size={14}` |
| `src/components/PullDropdown.svelte` | ChevronDown icon | ✓ VERIFIED | Imports `ChevronDown`; renders `<ChevronDown size={12} />` |
| `src/components/FileRow.svelte` | STATUS_ICON_COMPONENTS with Lucide components | ✓ VERIFIED | `StatusIconConfig` type; `STATUS_ICON_COMPONENTS` record; `svelte:component`; `Plus`/`Minus` for actions |
| `src/components/StagingPanel.svelte` | ChevronDown/ChevronRight for expand/collapse | ✓ VERIFIED | Imports `ChevronDown, ChevronRight`; used in section headers |
| `src/components/BranchSection.svelte` | ChevronDown, ChevronRight, Plus icons | ✓ VERIFIED | All three imported and used at `size={12}` |
| `src/components/BranchRow.svelte` | ArrowDown/ArrowUp for tracking counts | ✓ VERIFIED | Imports `ArrowDown, ArrowUp`; used at `size={11}` in tracking count display |
| `src/components/TabBar.svelte` | X icon for tab close | ✓ VERIFIED | Imports `X`; renders `<X size={12} />` |
| `src-tauri/src/commands/staging.rs` | `get_dirty_counts_inner` with WT_NEW + StatusOptions | ✓ VERIFIED | Lines 181-219: sync fn with `StatusOptions::new().include_untracked(true).recurse_untracked_dirs(true)` and `Status::WT_NEW` in unstaged accumulator |
| `src/components/CommitGraph.svelte` | `lastVisibleColumn` derived state + handle guards | ✓ VERIFIED | Lines 53-61: `ORDERED_COLUMNS`, `visibleColumns`, `lastVisibleColumn`; handle guards on ref/graph/author/date/sha; message handle unguarded |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `Toolbar.svelte` | `showToast` | `import { showToast } from '../lib/toast.svelte.js'` | ✓ WIRED | Line 6 import; called 5+ times with success/error pairs |
| `BranchSidebar.svelte` | `showToast` | `import { showToast } from '../lib/toast.svelte.js'` | ✓ WIRED | Line 4 import; called in handleCheckout success + error |
| `Toast.svelte` | `toasts.items` | `import { toasts } from '../lib/toast.svelte.js'` | ✓ WIRED | Line 2 import; line 7 `{#each toasts.items as toast}` |
| `App.svelte` | `<Toast>` | component mount | ✓ WIRED | Line 11 import; line 369 `<Toast />` unconditional render |
| `Toolbar.svelte` | `@lucide/svelte` | named import | ✓ WIRED | Line 10: 7 icons imported and used |
| `FileRow.svelte` | `@lucide/svelte` | named import | ✓ WIRED | Line 3: 8 icons imported; `svelte:component` uses them dynamically |
| `get_dirty_counts_inner` | `git2::StatusOptions` | `opts.include_untracked(true)` | ✓ WIRED | Lines 186-190 in staging.rs |
| `CommitGraph.svelte` | `col-resize-handle` | guarded by `col !== lastVisibleColumn` | ✓ WIRED | All 5 non-message column handles wrapped in `{#if}` guards |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| ICON-01 | 27-03 | App uses Lucide SVG icons replacing Unicode symbols across all components | ✓ SATISFIED | All 7 plan-targeted components import from `@lucide/svelte`; zero Unicode entities/symbols remain; REQUIREMENTS.md also lists CommitForm but it had no Unicode symbols to replace (clean text form — no gap) |
| TOAST-01 | 27-01, 27-02 | App displays toast notifications for operation success and error feedback | ✓ SATISFIED | `toast.svelte.ts` + `Toast.svelte` + `App.svelte` mount + Toolbar 5 ops + BranchSidebar checkout — all wired; 5 unit tests GREEN |
| FIX-01 | 27-01, 27-04 | New untracked files included in dirty counts and trigger WIP row in graph | ✓ SATISFIED | `get_dirty_counts_inner` with `WT_NEW` + `include_untracked(true)`; Rust test `get_dirty_counts_includes_untracked` GREEN (9/9) |
| FIX-02 | 27-04 | Last visible column header does not render a trailing resize divider | ✓ SATISFIED | `lastVisibleColumn` derived state in CommitGraph.svelte; all 5 non-message column handles guarded; message column handle preserved |

**Note on ICON-01 component list discrepancy:** REQUIREMENTS.md lists "CommitForm" and "BranchSidebar" as targets. CommitForm (`CommitForm.svelte`) contains no Unicode symbols requiring replacement — it is a plain text input form. BranchSidebar's icons are delivered through BranchSection.svelte and BranchRow.svelte (both updated). The PLAN 03 accurately targeted the 7 components that actually contained Unicode symbols. ICON-01 is fully satisfied.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | — | — | — | No anti-patterns detected |

Scanned: toast.svelte.ts, Toast.svelte, Toolbar.svelte, BranchSidebar.svelte, staging.rs, CommitGraph.svelte for TODO/FIXME, empty implementations, placeholder returns. All clean.

---

### Human Verification Required

#### 1. Visual Icon Rendering
**Test:** Run `npm run dev` or `cargo tauri dev`; inspect all 7 components visually
**Expected:**
- Toolbar: SVG icons for Undo2, Redo2, ArrowDown (Pull), ArrowUp (Push), GitBranch, Archive (Stash), PackageOpen (Pop) — no Unicode glyphs
- File list: FilePlus (green), FilePen (orange), FileMinus (red), FileSymlink (blue), FileType2 (purple), FileWarning (yellow) for each status
- File row hover: Plus/Minus SVG icons for stage/unstage buttons
- StagingPanel headers: ChevronDown/ChevronRight (not ▼/▶)
- BranchSection: ChevronDown/ChevronRight expand/collapse + Plus create button
- BranchRow tracking: ArrowDown/ArrowUp (not ↓/↑) for behind/ahead counts
- TabBar: X icon for close button (not ×)
- PullDropdown: ChevronDown SVG
**Why human:** Visual DPI rendering and icon appearance cannot be verified programmatically

#### 2. Toast Auto-Dismiss in Live App
**Test:** Open a repo, click Stash; observe toast notification; wait ~3 seconds
**Expected:** "Stash created" toast appears at bottom-right, then disappears after 3 seconds without interaction. Error toast (failed operation) should appear with red styling.
**Why human:** `setTimeout` auto-dismiss tested with fake timers in Vitest; real-time behavior requires live app verification

---

### Test Suite Results

| Suite | Count | Result |
|-------|-------|--------|
| TypeScript (Vitest) | 126/126 | ✅ ALL PASS |
| Rust (cargo test --lib staging) | 9/9 | ✅ ALL PASS |

---

### Gaps Summary

No gaps found. All four requirements (ICON-01, TOAST-01, FIX-01, FIX-02) are fully implemented, substantive (not stubs), and properly wired:

- **ICON-01**: All 7 targeted components migrated to `@lucide/svelte`; package installed in production dependencies; zero Unicode remnants; library present in node_modules
- **TOAST-01**: Reactive `$state` store with `showToast()`/`dismiss()`, fixed overlay component, mounted unconditionally in App, wired into 5 Toolbar operations + BranchSidebar checkout; 5/5 unit tests green
- **FIX-01**: `get_dirty_counts_inner` uses `StatusOptions.include_untracked(true)` and `Status::WT_NEW`; Rust test confirms fix; 9/9 staging tests green
- **FIX-02**: `lastVisibleColumn` derived state guards all trailing column resize handles in CommitGraph header; `message` column's inverted handle correctly preserved

The only items requiring human attention are visual rendering quality (icons at various DPIs) and real-time toast dismiss timing — neither of which can be verified statically.

---

_Verified: 2026-03-15T01:37:00Z_
_Verifier: Claude (gsd-verifier)_
