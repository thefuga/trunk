---
phase: 63-full-file-view-display-options
plan: 03
subsystem: ui
tags: [rust, git2, svelte, async, lazystore, whitespace, diff-options, regression-test]

# Dependency graph
requires:
  - phase: 63-01
    provides: DiffToolbar toggles, ondiffoptionschange callback, LazyStore preference persistence
  - phase: 63-02
    provides: FullFileView renderer, invisible character utilities, display option tests
provides:
  - Correct ignore_whitespace() (git -w) API call replacing ignore_whitespace_change() (git -b)
  - Race-condition-free async handlers for view mode and whitespace toggle changes
  - Flicker-free DiffPanel mount via prefsLoaded gate
  - Indentation-only whitespace ignore regression test
affects: [64-split-view]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "prefsLoaded gate: defer DiffPanel content rendering until async LazyStore preferences resolve"
    - "flushPrefs test helper: setTimeout(0) + tick() to flush microtasks before Svelte update queue"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/diff.rs
    - src-tauri/tests/test_diff.rs
    - src/components/DiffPanel.svelte
    - src/components/DiffPanel.test.ts

key-decisions:
  - "prefsLoaded gate over loading spinner: LazyStore loads in <50ms, so empty container (bg fill) preferred over visible spinner"
  - "flushPrefs() test helper encapsulates setTimeout(0) + tick() pattern to handle async $effect preference loading across all DiffPanel tests"

patterns-established:
  - "Async handler pattern: await all store writes before calling ondiffoptionschange callback"
  - "prefsLoaded gate: hide component content until async preferences resolve to prevent default-value flicker"
  - "flushPrefs test helper: flush microtask queue then Svelte updates for components with async $effect initialization"

requirements-completed: [WHSP-03, VIEW-04, DISP-02]

# Metrics
duration: 11min
completed: 2026-03-30
---

# Phase 63 Plan 03: UAT Gap Closure Summary

**Fixed git2 whitespace API (git -w not -b), resolved async race conditions in view mode/whitespace toggle handlers, and eliminated toggle button flicker on mount**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-30T03:44:19Z
- **Completed:** 2026-03-30T03:55:51Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Fixed Rust backend to use `ignore_whitespace()` (git `-w`, ignores ALL whitespace) instead of `ignore_whitespace_change()` (git `-b`, only ignores amount changes) -- indentation-only changes now correctly disappear when toggle is active
- Made `handleViewModeChange` and `handleIgnoreWhitespaceChange` async, awaiting store writes before calling `ondiffoptionschange` -- eliminates race condition where `buildDiffOptions` read stale values
- Added `prefsLoaded` gate that hides DiffPanel content until async LazyStore preferences load -- prevents one-frame flash of default (false) toggle states
- Added regression test for indentation-only whitespace ignore covering the exact user scenario (adding 4-space indent)

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix Rust ignore_whitespace API and add indentation test** - `d9cf0fa` (fix)
2. **Task 2: Fix async race conditions and toggle flicker in DiffPanel** - `43ddaab` (fix)

## Files Created/Modified
- `src-tauri/src/commands/diff.rs` - Changed `ignore_whitespace_change()` to `ignore_whitespace()` in `apply_request_options`
- `src-tauri/tests/test_diff.rs` - Added `diff_unstaged_ignores_indentation_whitespace` test (25 total diff tests)
- `src/components/DiffPanel.svelte` - Added `prefsLoaded` gate, made handlers async with awaited store writes
- `src/components/DiffPanel.test.ts` - Added `flushPrefs()` helper, updated all tests to handle async preference loading, fixed gutter test isolation

## Decisions Made
- prefsLoaded gate over loading spinner: LazyStore loads in <50ms so the user sees the panel appear fully formed rather than a brief spinner flash
- flushPrefs() test helper: encapsulates `setTimeout(0)` + `tick()` to properly flush both the microtask queue (Promise.all in $effect) and the Svelte update queue, replacing bare `tick()` which only flushed Svelte updates

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated all DiffPanel tests with flushPrefs() helper**
- **Found during:** Task 2 (DiffPanel async fixes)
- **Issue:** Adding `prefsLoaded` gate caused 27 existing DiffPanel tests to fail because `tick()` alone doesn't flush microtask promises from the $effect's `Promise.all` -- content was hidden behind the `{#if prefsLoaded}` block
- **Fix:** Added `flushPrefs()` helper (setTimeout(0) + tick()) and replaced all `await tick()` calls after render with `await flushPrefs()`
- **Files modified:** src/components/DiffPanel.test.ts
- **Verification:** All 402 tests pass
- **Committed in:** 43ddaab (Task 2 commit)

**2. [Rule 1 - Bug] Fixed gutter test isolation (viewMode leak)**
- **Found during:** Task 2 (DiffPanel async fixes)
- **Issue:** Gutter tests (DISP-01) failed because stateful store mock retained "full" viewMode from preceding VIEW-01 tests. Previously hidden by tick() not flushing prefs (so default "hunk" was always used); now exposed by correct flushPrefs() behavior.
- **Fix:** Added explicit `getDiffViewMode` mock reset to "hunk" in each gutter test
- **Files modified:** src/components/DiffPanel.test.ts
- **Verification:** All gutter tests pass in isolation and in sequence
- **Committed in:** 43ddaab (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes necessary for test correctness after the prefsLoaded gate change. No scope creep.

## Issues Encountered
None beyond the auto-fixed test issues above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 63 fully complete: all 3 UAT-diagnosed issues fixed with regression tests
- All display options work correctly: whitespace ignore (git -w), live view mode switching, flicker-free toggle persistence
- SplitView remains a stub for Phase 64
- All 402 frontend tests + 25 diff tests pass

## Self-Check: PASSED

- src-tauri/src/commands/diff.rs: FOUND
- src-tauri/tests/test_diff.rs: FOUND
- src/components/DiffPanel.svelte: FOUND
- src/components/DiffPanel.test.ts: FOUND
- 63-03-SUMMARY.md: FOUND
- Commit d9cf0fa: FOUND
- Commit 43ddaab: FOUND
- opts.ignore_whitespace() API: VERIFIED
- prefsLoaded gate: VERIFIED
- Indentation test: VERIFIED

---
*Phase: 63-full-file-view-display-options*
*Completed: 2026-03-30*
