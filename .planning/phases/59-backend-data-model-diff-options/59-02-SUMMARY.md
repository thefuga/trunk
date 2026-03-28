---
phase: 59-backend-data-model-diff-options
plan: 02
subsystem: frontend
tags: [typescript, svelte, tauri-store, diff, ipc]

# Dependency graph
requires:
  - phase: 59-01
    provides: DiffRequestOptions, WordSpan, SyntaxToken Rust structs with serde attributes
provides:
  - TypeScript WordSpan, SyntaxToken, DiffRequestOptions interfaces mirroring Rust types
  - LazyStore get/set functions for diff_context_lines, diff_ignore_whitespace, diff_show_full_file
  - All RepoView diff invoke calls wired to pass DiffRequestOptions from persisted preferences
affects: [60-word-level-diff, 61-syntax-highlighting, 62-frontend-diff-controls]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "buildDiffOptions() helper: loads all 3 preferences in parallel via Promise.all, returns typed DiffRequestOptions"
    - "Diff preference persistence: LazyStore keys diff_context_lines, diff_ignore_whitespace, diff_show_full_file with sensible defaults"

key-files:
  created: []
  modified:
    - src/lib/types.ts
    - src/lib/store.ts
    - src/lib/store.test.ts
    - src/components/RepoView.svelte
    - src/components/DiffPanel.test.ts

key-decisions:
  - "DiffLine fields use snake_case (word_spans, syntax_tokens) matching Rust Serialize default; DiffRequestOptions uses camelCase matching serde rename_all"
  - "Default values: contextLines=3, ignoreWhitespace=false, showFullFile=false -- matches Rust DiffRequestOptions::default()"

patterns-established:
  - "buildDiffOptions() pattern: single async helper consolidating all diff preference reads for reuse across call sites"

requirements-completed: [DISP-03, CTXL-01, CTXL-02, WHSP-01]

# Metrics
duration: 5min
completed: 2026-03-28
---

# Phase 59 Plan 02: Frontend Type Mirrors & Diff Preference Wiring Summary

**TypeScript DiffRequestOptions/WordSpan/SyntaxToken type mirrors, LazyStore diff preference persistence, and all 4 RepoView diff invoke calls wired to pass options**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-28T12:55:49Z
- **Completed:** 2026-03-28T13:01:20Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added WordSpan, SyntaxToken, DiffRequestOptions TypeScript interfaces mirroring Rust types 1:1
- Extended DiffLine with word_spans and syntax_tokens fields for future enrichment phases
- Added 6 LazyStore get/set functions for diff preferences with correct defaults
- Wired all 4 diff invoke call sites in RepoView.svelte to pass DiffRequestOptions from stored preferences
- 6 new store tests verifying preference round-trips and defaults

## Task Commits

Each task was committed atomically:

1. **Task 1: Add TypeScript type mirrors and LazyStore persistence functions** - `08393ec` (feat)
2. **Task 2: Add store tests for diff preference persistence** - `aba85a0` (test)
3. **Task 3: Wire RepoView.svelte diff invoke calls to pass DiffRequestOptions** - `a98affd` (feat)

## Files Created/Modified
- `src/lib/types.ts` - Added WordSpan, SyntaxToken, DiffRequestOptions interfaces; extended DiffLine with word_spans/syntax_tokens
- `src/lib/store.ts` - Added getDiffContextLines/setDiffContextLines, getDiffIgnoreWhitespace/setDiffIgnoreWhitespace, getDiffShowFullFile/setDiffShowFullFile
- `src/lib/store.test.ts` - 6 new tests for diff preference get/set round-trips with default verification
- `src/components/RepoView.svelte` - buildDiffOptions() helper; all 4 diff invoke calls pass options
- `src/components/DiffPanel.test.ts` - Updated DiffLine test fixtures with word_spans/syntax_tokens fields

## Decisions Made
- DiffLine enrichment fields use snake_case (word_spans, syntax_tokens) matching Rust Serialize default output; DiffRequestOptions uses camelCase matching serde rename_all attribute
- Default preference values (contextLines=3, ignoreWhitespace=false, showFullFile=false) match Rust DiffRequestOptions::default()

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated DiffPanel.test.ts fixtures for new DiffLine fields**
- **Found during:** Task 1 (adding word_spans/syntax_tokens to DiffLine interface)
- **Issue:** DiffPanel.test.ts had DiffLine object literals missing the new required word_spans and syntax_tokens fields, causing svelte-check type errors
- **Fix:** Added `word_spans: []` and `syntax_tokens: []` to all 5 DiffLine test fixtures
- **Files modified:** src/components/DiffPanel.test.ts
- **Verification:** svelte-check passes with 0 errors
- **Committed in:** 08393ec (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to maintain type correctness after DiffLine interface extension. No scope creep.

## Issues Encountered
None

## Known Stubs
None - word_spans and syntax_tokens are intentionally empty arrays at this stage, to be populated by Phases 60-61 as designed. DiffRequestOptions defaults are fully functional.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- TypeScript types fully mirror Rust structs, ready for Phase 60 (word-level diff) and Phase 61 (syntax highlighting)
- LazyStore persistence ready for Phase 62 (frontend diff controls UI)
- buildDiffOptions() helper centralizes preference loading, ready for UI controls to call setDiff* functions
- All diff commands accept options end-to-end: LazyStore -> TypeScript -> Tauri invoke -> Rust _inner functions

## Self-Check: PASSED

All 5 modified files verified on disk. All 3 task commits (08393ec, aba85a0, a98affd) found in git log.

---
*Phase: 59-backend-data-model-diff-options*
*Completed: 2026-03-28*
