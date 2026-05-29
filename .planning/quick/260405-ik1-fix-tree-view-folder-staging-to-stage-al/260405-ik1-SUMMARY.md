---
status: complete
phase: quick-260405-ik1
plan: 01
subsystem: staging
tags: [git2, tauri-command, bulk-staging, race-condition]

requires:
  - phase: none
    provides: existing stage_file/unstage_file commands
provides:
  - bulk stage_files command (stages N files in one index write)
  - bulk unstage_files command (unstages N files in one reset_default call)
  - frontend stageDirectory/unstageDirectory using bulk IPC
affects: [staging, tree-view]

tech-stack:
  added: []
  patterns:
    - "Bulk git index operations: single open/write cycle for multiple paths"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/staging.rs
    - src-tauri/src/lib.rs
    - src/components/StagingPanel.svelte
    - src-tauri/tests/common/drivers/staging.rs
    - src-tauri/tests/test_staging.rs

key-decisions:
  - "stage_files_inner uses single index.write() after looping all paths (same pattern as stage_all_inner)"
  - "unstage_files_inner uses reset_default with iterator for all paths in one call"

patterns-established:
  - "Bulk staging pattern: collect all paths, single index open/write cycle"

requirements-completed: [fix-tree-view-folder-staging]

duration: 14min
completed: 2026-04-05
---

# Quick Task 260405-ik1: Fix Tree View Folder Staging Summary

**Bulk stage_files/unstage_files commands that process all directory paths in a single git index write, eliminating the race condition from concurrent individual IPC calls**

## Performance

- **Duration:** 14 min
- **Started:** 2026-04-05T16:28:51Z
- **Completed:** 2026-04-05T16:42:47Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Bulk `stage_files_inner` stages N files atomically in one index read/write cycle (handles adds, modifications, and deletions)
- Bulk `unstage_files_inner` unstages N files atomically in one `reset_default` call (handles unborn HEAD)
- Frontend `stageDirectory()` and `unstageDirectory()` now use single IPC calls instead of `Promise.all` with individual calls
- 7 new tests covering: multiple modified files, mixed new+modified, deletions, empty vec no-op, bulk unstage, unborn HEAD, empty unstage no-op

## Task Commits

Each task was committed atomically:

1. **Task 1 (RED): Add failing tests for bulk stage_files/unstage_files** - `6128799` (test)
2. **Task 1 (GREEN): Implement bulk stage_files and unstage_files commands** - `27960f5` (feat)
3. **Task 2: Update frontend to call bulk commands** - `96013e7` (fix)

## Files Created/Modified
- `src-tauri/src/commands/staging.rs` - Added stage_files_inner, unstage_files_inner, and their Tauri command wrappers
- `src-tauri/src/lib.rs` - Registered stage_files and unstage_files in invoke_handler
- `src/components/StagingPanel.svelte` - stageDirectory/unstageDirectory now call bulk commands
- `src-tauri/tests/common/drivers/staging.rs` - Added stage_files and unstage_files driver methods
- `src-tauri/tests/test_staging.rs` - 7 new tests for bulk staging/unstaging

## Decisions Made
- Used single `index.write()` after looping all paths (same pattern as `stage_all_inner`) rather than one write per file
- Used `reset_default` with iterator accepting all paths for unstage (git2 handles this atomically)
- Tauri serde maps `file_paths: Vec<String>` in Rust to `filePaths` in JS (camelCase convention matching existing commands)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None.

## Self-Check: PASSED

- All 5 modified files exist on disk
- All 3 task commits found in git log (6128799, 27960f5, 96013e7)
- Must-have artifacts verified: stage_files_inner, stage_files command, frontend invoke calls

---
*Quick task: 260405-ik1*
*Completed: 2026-04-05*
