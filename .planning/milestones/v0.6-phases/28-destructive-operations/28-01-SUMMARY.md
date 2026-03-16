---
phase: 28-destructive-operations
plan: 01
subsystem: api
tags: [git2, tauri, rust, ipc, discard, branch, tag]

# Dependency graph
requires: []
provides:
  - discard_file IPC command (reverts tracked, deletes untracked)
  - discard_all IPC command (reverts all tracked + removes all untracked)
  - delete_branch IPC command (with HEAD protection)
  - rename_branch IPC command
  - delete_tag IPC command
affects: [28-destructive-operations]

# Tech tracking
tech-stack:
  added: []
  patterns: [inner-fn + Tauri wrapper for destructive ops, HEAD branch guard pattern]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/staging.rs
    - src-tauri/src/commands/branches.rs
    - src-tauri/src/commands/commit_actions.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "discard_file uses git2 checkout for tracked files and std::fs::remove_file for untracked — no git CLI subprocess needed"
  - "discard commands skip cache rebuild + repo-changed emit — FS watcher handles workdir change detection"
  - "delete_branch/rename_branch/delete_tag rebuild graph cache before emitting repo-changed — matches existing create_branch/create_tag pattern"

patterns-established:
  - "HEAD branch guard: check repo.head().shorthand() before allowing destructive branch ops"
  - "Untracked file cleanup in discard_all: collect paths before checkout, delete after, ignore remove_dir errors for non-empty parents"

requirements-completed: [GITOP-01, GITOP-02, GITOP-03, GITOP-04, GITOP-05]

# Metrics
duration: 4min
completed: 2026-03-15
---

# Phase 28 Plan 01: Backend Destructive Operations Summary

**5 new Tauri IPC commands for discard file/all, delete/rename branch, and delete tag — all with inner-fn pattern, unit tests, and cache rebuild**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-15T20:09:27Z
- **Completed:** 2026-03-15T20:14:22Z
- **Tasks:** 2 (TDD: 4 commits total — 2 RED + 2 GREEN)
- **Files modified:** 4

## Accomplishments
- discard_file_inner reverts tracked files via checkout and deletes untracked files from disk
- discard_all_inner force-checkouts HEAD then removes all untracked files
- delete_branch_inner rejects HEAD branch deletion with clear error code, removes non-HEAD local branches
- rename_branch_inner renames local branches (fails on duplicate target)
- delete_tag_inner removes tag refs by full refs/tags/ name
- All 5 new commands registered in lib.rs generate_handler
- 96 total tests pass (7 new + 89 existing)

## Task Commits

Each task was committed atomically (TDD RED → GREEN):

1. **Task 1: Discard commands — staging.rs**
   - `b2d223f` (test) — failing tests for discard_file and discard_all
   - `3d6ba75` (feat) — implement discard_file and discard_all commands
2. **Task 2: Branch/Tag commands — branches.rs + commit_actions.rs + lib.rs**
   - `05fd6c8` (test) — failing tests for delete_branch, rename_branch, delete_tag
   - `843519d` (feat) — implement delete_branch, rename_branch, delete_tag commands

## Files Created/Modified
- `src-tauri/src/commands/staging.rs` — Added discard_file_inner, discard_all_inner + Tauri wrappers + 3 tests
- `src-tauri/src/commands/branches.rs` — Added delete_branch_inner, rename_branch_inner + Tauri wrappers + 3 tests
- `src-tauri/src/commands/commit_actions.rs` — Added delete_tag_inner + Tauri wrapper + 1 test
- `src-tauri/src/lib.rs` — Registered all 5 new commands in generate_handler

## Decisions Made
- discard_file uses git2 checkout for tracked files and std::fs::remove_file for untracked — avoids git CLI subprocess
- discard commands skip cache rebuild + repo-changed emit — FS watcher already detects workdir changes
- Branch/tag mutation commands rebuild graph cache before emitting repo-changed — matches existing pattern

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed borrow checker error in rename_branch_inner**
- **Found during:** Task 2 (GREEN phase)
- **Issue:** `branch.rename()` requires `&mut Branch`, and `drop(repo)` before `drop(branch)` violates borrow rules
- **Fix:** Added `mut` to branch binding and explicit `drop(branch)` before `drop(repo)`
- **Files modified:** src-tauri/src/commands/branches.rs
- **Verification:** cargo build succeeds, all tests pass
- **Committed in:** 843519d (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial Rust borrow checker fix. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 5 backend IPC commands ready for frontend consumption
- Ready for 28-02 (Discard UI) and 28-03 (Branch/Tag UI) plans

## Self-Check: PASSED

All 4 modified files exist on disk. All 4 commit hashes verified in git log.

---
*Phase: 28-destructive-operations*
*Completed: 2026-03-15*
