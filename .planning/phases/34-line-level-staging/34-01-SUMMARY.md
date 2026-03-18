---
phase: 34-line-level-staging
plan: 01
subsystem: api
tags: [git2, partial-patch, unified-diff, line-staging, tauri]

# Dependency graph
requires:
  - phase: 32-hunk-staging-backend
    provides: stage_hunk_inner/unstage_hunk_inner/discard_hunk_inner patterns, Patch::from_diff, ApplyOptions
provides:
  - build_partial_patch_text helper for constructing partial unified diffs
  - stage_lines_inner, unstage_lines_inner, discard_lines_inner backend functions
  - stage_lines, unstage_lines, discard_lines Tauri IPC commands
affects: [34-line-level-staging-frontend, DiffPanel.svelte]

# Tech tracking
tech-stack:
  added: []
  patterns: [reverse partial patch construction for undo operations, forward/reverse mode in build_partial_patch_text]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/staging.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Used forward diff + reverse patch construction instead of git2 .reverse(true) for unstage/discard -- ensures line indices from frontend match the diff the user sees"
  - "Single build_partial_patch_text function with reverse flag instead of separate forward/reverse builders -- less code duplication"

patterns-established:
  - "Reverse partial patch: for undo operations (unstage/discard), generate the forward diff matching user view, then construct a reversed patch from it. This avoids index mismatch between forward and reversed diffs."

requirements-completed: [HUNK-07, HUNK-08]

# Metrics
duration: 11min
completed: 2026-03-18
---

# Phase 34 Plan 01: Line-Level Staging Backend Summary

**Partial patch construction with forward/reverse modes for line-level stage, unstage, and discard via git2 Diff::from_buffer + repo.apply**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-18T06:39:29Z
- **Completed:** 2026-03-18T06:51:07Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- build_partial_patch_text constructs valid unified diff patches from selected line indices with correct old_lines/new_lines recalculation
- Forward mode (staging) and reverse mode (unstaging/discarding) in single function, ensuring line indices always match the diff the user sees
- All 6 unit tests pass covering selected adds, selected deletes, mixed selection, stale hunk index, unstage, and discard
- Three Tauri IPC commands registered and callable from frontend

## Task Commits

Each task was committed atomically:

1. **Task 1: RED -- Write failing tests** - `231ff58` (test)
2. **Task 2: GREEN -- Implement build_partial_patch_text and inner functions** - `7194092` (feat)
3. **Task 3: Wire Tauri async command wrappers and register in lib.rs** - `cb17bd0` (feat)

## Files Created/Modified
- `src-tauri/src/commands/staging.rs` - Added build_partial_patch_text, stage_lines_inner, unstage_lines_inner, discard_lines_inner, 3 async Tauri command wrappers, create_add_delete_hunk_file test helper, 6 unit tests
- `src-tauri/src/lib.rs` - Registered stage_lines, unstage_lines, discard_lines in invoke_handler

## Decisions Made
- Used forward diff + reverse patch construction instead of git2 `.reverse(true)` for unstage/discard. Rationale: `.reverse(true)` produces a diff with swapped add/delete origins AND different line ordering, which means line indices from the frontend (based on the forward diff) don't map correctly. By using the forward diff and building a reversed patch manually, indices always match.
- Single `build_partial_patch_text` function with a `reverse` boolean parameter handles both forward (staging) and reverse (unstage/discard) patch construction, reducing code duplication.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed line index mismatch in reversed diffs for unstage/discard**
- **Found during:** Task 2 (GREEN implementation)
- **Issue:** Plan specified using git2's `.reverse(true)` diff option for unstage/discard, but reversed diffs have different line ordering than forward diffs (deletes and adds swap positions), causing frontend line indices to not map correctly
- **Fix:** Changed unstage_lines_inner and discard_lines_inner to use the forward (non-reversed) diff matching the user's view, then construct a reversed partial patch via a `reverse` parameter on build_partial_patch_text
- **Files modified:** src-tauri/src/commands/staging.rs
- **Verification:** All 6 tests pass including discard_lines_discards_selected
- **Committed in:** 7194092

**2. [Rule 1 - Bug] Fixed substring match in discard_lines test assertion**
- **Found during:** Task 2 (GREEN implementation)
- **Issue:** Test used `file_content.contains("MODIFIED line 2")` which matched substring within "MODIFIED line 29", causing false assertion failure
- **Fix:** Changed to line-by-line comparison using `file_lines.contains(&trimmed)` for exact line matching
- **Files modified:** src-tauri/src/commands/staging.rs (test)
- **Verification:** discard_lines_discards_selected test passes
- **Committed in:** 7194092

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes were necessary for correctness. The reverse-patch approach is architecturally equivalent to the plan's intent but avoids a subtle index-mapping bug. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Backend commands are ready for frontend integration
- Frontend can call `invoke('stage_lines', {path, filePath, hunkIndex, lineIndices})` and equivalents for unstage/discard
- Next plan should add line selection UI in DiffPanel.svelte with toolbar mode switching

---
*Phase: 34-line-level-staging*
*Completed: 2026-03-18*
