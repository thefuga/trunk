---
phase: 59-backend-data-model-diff-options
plan: 01
subsystem: backend
tags: [git2, diff, serde, tauri-command, rust]

# Dependency graph
requires: []
provides:
  - DiffRequestOptions struct with context_lines, ignore_whitespace, show_full_file
  - WordSpan and SyntaxToken enrichment types on DiffLine
  - apply_request_options helper for git2::DiffOptions configuration
  - Test driver _with_options variants for all diff commands
affects: [60-word-level-diff, 61-syntax-highlighting, 62-frontend-diff-controls]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Options struct threading: single DiffRequestOptions flows through command -> _inner -> git2"
    - "Enrichment fields: empty vecs on DiffLine to be populated by downstream phases"
    - "apply_request_options helper: centralizes git2::DiffOptions configuration"

key-files:
  created: []
  modified:
    - src-tauri/src/git/types.rs
    - src-tauri/src/commands/diff.rs
    - src-tauri/tests/common/drivers/diff.rs
    - src-tauri/tests/test_diff.rs
    - src-tauri/tests/test_integ_serde.rs
    - src-tauri/benches/bench_ipc.rs
    - src-tauri/benches/bench_commands.rs

key-decisions:
  - "Byte offset ranges for WordSpan/SyntaxToken (u32 start/end) -- compact over IPC, frontend slices content string"
  - "100_000 context_lines cap for show_full_file instead of u32::MAX -- avoids IPC payload issues on large files"
  - "serde rename_all camelCase on DiffRequestOptions -- matches Tauri IPC convention and existing RebaseTodoAction pattern"

patterns-established:
  - "DiffRequestOptions default pattern: context_lines=3, ignore_whitespace=false, show_full_file=false"
  - "Test driver _with_options variant pattern: default wrapper + explicit options variant"

requirements-completed: [CTXL-01, CTXL-02, WHSP-01]

# Metrics
duration: 5min
completed: 2026-03-28
---

# Phase 59 Plan 01: Backend Data Model & Diff Options Summary

**DiffRequestOptions struct with context_lines/whitespace/full-file threading through all 3 diff commands, plus WordSpan/SyntaxToken enrichment fields on DiffLine**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-28T12:48:12Z
- **Completed:** 2026-03-28T12:53:42Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments
- Added WordSpan, SyntaxToken, and DiffRequestOptions structs to types.rs with proper serde attributes
- Threaded DiffRequestOptions through all three diff _inner functions and Tauri command wrappers
- Added apply_request_options helper centralizing git2::DiffOptions configuration
- 5 new integration tests covering context_lines, whitespace ignore, show_full_file, commit diff options, and enrichment field serialization
- All 161 existing Rust tests continue to pass with zero regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Add new types to types.rs and update DiffLine** - `84a5947` (feat)
2. **Task 2: Thread DiffRequestOptions through diff commands and update walk_diff_into_file_diffs** - `2af26cc` (feat)
3. **Task 3: Add integration tests for context_lines, whitespace ignore, show_full_file, and enrichment serialization** - `543ea7e` (test)

## Files Created/Modified
- `src-tauri/src/git/types.rs` - Added WordSpan, SyntaxToken, DiffRequestOptions structs; extended DiffLine with word_spans and syntax_tokens
- `src-tauri/src/commands/diff.rs` - Added apply_request_options helper; updated all _inner and command signatures to accept DiffRequestOptions
- `src-tauri/tests/common/drivers/diff.rs` - Updated test drivers with DiffRequestOptions::default() and _with_options variants
- `src-tauri/tests/test_diff.rs` - Added 4 new integration tests for diff options
- `src-tauri/tests/test_integ_serde.rs` - Added word_spans/syntax_tokens empty array serialization test
- `src-tauri/benches/bench_ipc.rs` - Updated benchmark caller to pass DiffRequestOptions::default()
- `src-tauri/benches/bench_commands.rs` - Updated benchmark caller to pass DiffRequestOptions::default()

## Decisions Made
- Used byte offset ranges (u32 start/end) for WordSpan and SyntaxToken rather than pre-segmented strings -- more compact over IPC, frontend slices content string directly
- Capped show_full_file context_lines at 100,000 instead of u32::MAX to avoid IPC payload issues on large files
- Applied serde rename_all camelCase on DiffRequestOptions to match Tauri convention (JS sends contextLines, ignoreWhitespace, showFullFile)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated benchmark callers for new _inner signatures**
- **Found during:** Task 2 (threading DiffRequestOptions)
- **Issue:** bench_ipc.rs and bench_commands.rs also call diff_unstaged_inner and would fail to compile without the new parameter
- **Fix:** Added DiffRequestOptions::default() as 4th argument to benchmark callers
- **Files modified:** src-tauri/benches/bench_ipc.rs, src-tauri/benches/bench_commands.rs
- **Verification:** cargo clippy clean, all tests pass
- **Committed in:** 2af26cc (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to maintain compilation of benchmarks. No scope creep.

## Issues Encountered
None

## Known Stubs
None - all enrichment fields (word_spans, syntax_tokens) are intentionally empty vecs in this phase, to be populated by Phases 60-61 as designed.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- DiffRequestOptions struct ready for frontend TypeScript mirroring (Plan 59-02)
- WordSpan and SyntaxToken types ready for Phase 60 (word-level diff) and Phase 61 (syntax highlighting) population
- Test infrastructure has _with_options driver methods ready for future phases
- All existing functionality preserved with DiffRequestOptions::default()

## Self-Check: PASSED

All 7 modified files verified on disk. All 3 task commits (84a5947, 2af26cc, 543ea7e) found in git log.

---
*Phase: 59-backend-data-model-diff-options*
*Completed: 2026-03-28*
