---
phase: 63-full-file-view-display-options
plan: 02
subsystem: ui
tags: [svelte, diff-viewer, full-file-view, invisible-characters, whitespace, testing]

# Dependency graph
requires:
  - phase: 63-01
    provides: DiffToolbar with display option toggles, DiffViewer mode dispatch, view mode persistence
provides:
  - FullFileView continuous document renderer with flatMap hunk flattening
  - splitInvisibles/trailingWhitespaceStart utilities for invisible character rendering
  - Invisible character rendering in both HunkView and FullFileView
  - Test coverage for VIEW-04, WHSP-02, WHSP-03, DISP-02
affects: [64-split-view]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "splitInvisibles: slice-first-then-split pattern for byte-offset-safe invisible rendering"
    - "trailingWhitespaceStart: reverse scan for trailing whitespace boundary detection"

key-files:
  created:
    - src/lib/diff-utils.ts
  modified:
    - src/components/diff/FullFileView.svelte
    - src/components/diff/HunkView.svelte
    - src/components/diff/DiffViewer.svelte
    - src/components/DiffPanel.test.ts

key-decisions:
  - "Stateful store mock for test isolation: getDiffIgnoreWhitespace/getDiffWordWrap share mutable state to match real store behavior"
  - "DISP-02 tested via toggle click + store call verification instead of inline style assertion (Svelte 5 compiles dynamic styles to property assignments invisible to jsdom getAttribute)"
  - "Existing full-file-view stub test updated to match implemented component behavior"

patterns-established:
  - "InvisibleSegment pattern: split text into visible/invisible sub-segments with isTrailing flag"
  - "Trailing whitespace detection: compute trailStart once per line, check span boundaries against it"

requirements-completed: [VIEW-04, WHSP-03]

# Metrics
duration: 10min
completed: 2026-03-29
---

# Phase 63 Plan 02: FullFileView and Invisible Characters Summary

**FullFileView continuous document renderer with invisible character rendering (middle dot/arrow) in both view modes, plus 17 new tests covering VIEW-04, WHSP-02, WHSP-03, DISP-02**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-29T23:00:58Z
- **Completed:** 2026-03-29T23:11:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- FullFileView renders entire file as one scrollable document with no hunk headers, no staging buttons, and two-column line number gutter
- splitInvisibles utility replaces spaces with middle dot and tabs with rightwards arrow, with trailing whitespace detection
- Invisible character rendering integrated into both HunkView and FullFileView with CSS custom property styling
- 17 new tests across 4 requirement areas (VIEW-04: 3, WHSP-02: 3, WHSP-03: 9, DISP-02: 2), full suite at 402 tests

## Task Commits

Each task was committed atomically:

1. **Task 1: Invisible character utility and FullFileView component** - `ded28ed` (feat)
2. **Task 2: Tests for VIEW-04, WHSP-02, WHSP-03, DISP-02** - `05b1df1` (test)

## Files Created/Modified
- `src/lib/diff-utils.ts` - InvisibleSegment type, splitInvisibles() and trailingWhitespaceStart() utilities
- `src/components/diff/FullFileView.svelte` - Continuous document renderer replacing stub, with invisible character support
- `src/components/diff/HunkView.svelte` - Integrated splitInvisibles into span rendering loop with .invisible-char/.trailing-ws CSS
- `src/components/diff/DiffViewer.svelte` - Pass fileDiffs/showInvisibles/wordWrap props to FullFileView
- `src/components/DiffPanel.test.ts` - 17 new tests + stateful store mock for preference toggles

## Decisions Made
- Stateful store mock: getDiffIgnoreWhitespace/getDiffWordWrap mocks share mutable state via closure to properly test $effect-driven state changes across test cases
- DISP-02 word wrap tested via toggle click + setDiffWordWrap call assertion rather than inline style inspection, because Svelte 5 compiles dynamic template styles to direct property assignments that jsdom getAttribute doesn't capture
- Updated existing "shows full file stub" test to "shows full file view" since stub was replaced with real implementation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated existing test for replaced stub**
- **Found during:** Task 2
- **Issue:** Existing test "shows full file stub when Full mode selected" checked for text "Full file view" which no longer exists after stub replacement
- **Fix:** Updated test to verify hunk header absence instead of stub text
- **Files modified:** src/components/DiffPanel.test.ts
- **Verification:** All 40 DiffPanel tests pass
- **Committed in:** 05b1df1

**2. [Rule 1 - Bug] Reset viewMode in store mock between test blocks**
- **Found during:** Task 2
- **Issue:** Stateful mock retained "Full" viewMode from VIEW-04 tests, causing WHSP-02 tests to render FullFileView instead of HunkView (no Stage Hunk button)
- **Fix:** Each WHSP-02/DISP-02 test explicitly resets getDiffViewMode mock to return "hunk"
- **Files modified:** src/components/DiffPanel.test.ts
- **Verification:** All test blocks pass independently
- **Committed in:** 05b1df1

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes necessary for test correctness. No scope creep.

## Issues Encountered
- Svelte 5 dynamic inline style compilation: template `style="white-space: {expr}"` compiles to runtime style property assignments (`element.style.whiteSpace = value`) that jsdom's `getAttribute('style')` returns null for. Resolved by testing word wrap behavior through toggle click + store call verification instead of DOM style inspection.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 63 fully complete: all 4 requirements (VIEW-04, WHSP-02, WHSP-03, DISP-02) implemented and tested
- FullFileView and HunkView both support invisible characters, word wrap, and whitespace ignore
- SplitView remains a stub for Phase 64
- Ready for phase verification

## Self-Check: PASSED

- src/lib/diff-utils.ts: FOUND
- src/components/diff/FullFileView.svelte: FOUND
- src/components/DiffPanel.test.ts: FOUND
- 63-02-SUMMARY.md: FOUND
- Commit ded28ed: FOUND
- Commit 05b1df1: FOUND

---
*Phase: 63-full-file-view-display-options*
*Completed: 2026-03-29*
