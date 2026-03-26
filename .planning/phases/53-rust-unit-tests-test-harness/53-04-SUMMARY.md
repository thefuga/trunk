---
phase: 53-rust-unit-tests-test-harness
plan: 04
subsystem: testing
tags: [rust, git2, integration-tests, test-harness, goos, migration]

# Dependency graph
requires:
  - phase: 53-01
    provides: TestContext, builder, assertion helpers, module visibility
provides:
  - 4 driver files (operation_state, merge_editor, interactive_rebase, remote)
  - 6 test files migrating 56 tests from inline #[cfg(test)] modules
  - Zero inline test modules remaining in src-tauri/src/
  - make_test_repo() and make_large_test_repo() removed (D-07)
affects: [54, 55]

# Tech tracking
tech-stack:
  added: []
  patterns: [direct walk_commits calls for graph algorithm tests, pure function testing without TestContext]

key-files:
  created:
    - src-tauri/tests/common/drivers/operation_state.rs
    - src-tauri/tests/common/drivers/merge_editor.rs
    - src-tauri/tests/common/drivers/interactive_rebase.rs
    - src-tauri/tests/common/drivers/remote.rs
    - src-tauri/tests/test_operation_state.rs
    - src-tauri/tests/test_merge_editor.rs
    - src-tauri/tests/test_interactive_rebase.rs
    - src-tauri/tests/test_remote.rs
    - src-tauri/tests/test_graph.rs
    - src-tauri/tests/test_repository.rs
  modified:
    - src-tauri/tests/common/drivers/mod.rs
    - src-tauri/src/commands/operation_state.rs
    - src-tauri/src/commands/merge_editor.rs
    - src-tauri/src/commands/interactive_rebase.rs
    - src-tauri/src/commands/remote.rs
    - src-tauri/src/git/graph.rs
    - src-tauri/src/git/repository.rs

key-decisions:
  - "Graph tests call walk_commits() directly (acceptable per research - algorithm tests, not command wrappers)"
  - "Remote tests call classify_git_error() directly (pure function, no state dependency)"
  - "merge_branch_inner conflict behavior documented as-is (CONFLICT on stdout, not stderr)"

patterns-established:
  - "Pure function tests do not need TestContext - call directly via trunk_lib::module::function()"
  - "Complex graph topologies use raw git2 setup with raw_commit helper when builder cannot express the topology"

requirements-completed: [HARN-02, HARN-04, UNIT-01]

# Metrics
duration: 15min
completed: 2026-03-26
---

# Phase 53 Plan 04: Remaining Module Migration Summary

**12 driver methods, 56 tests migrated across 6 modules, zero #[cfg(test)] modules remaining, make_test_repo removed**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-26T18:08:55Z
- **Completed:** 2026-03-26T18:24:00Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments
- Created 4 driver files wrapping 12 _inner functions for TestContext (operation_state: 8, merge_editor: 2, interactive_rebase: 2)
- Migrated 56 tests to integration test crate: 8 operation_state, 3 merge_editor, 5 interactive_rebase, 16 remote, 22 graph, 2 repository
- Removed all #[cfg(test)] modules from src-tauri/src/ (zero remain)
- Removed make_test_repo() and make_large_test_repo() helpers from repository.rs (D-07)
- 156 total integration tests pass across 14 test files

## Task Commits

Each task was committed atomically:

1. **Task 1: Migrate operation_state, merge_editor, interactive_rebase, remote** - `5264b56` (feat)
2. **Task 2: Migrate graph and repository tests, remove make_test_repo** - `12586f4` (feat)

## Files Created/Modified
- `src-tauri/tests/common/drivers/operation_state.rs` - 8 driver methods wrapping operation state _inner functions
- `src-tauri/tests/common/drivers/merge_editor.rs` - 2 driver methods wrapping merge editor _inner functions
- `src-tauri/tests/common/drivers/interactive_rebase.rs` - 2 driver methods wrapping rebase _inner functions
- `src-tauri/tests/common/drivers/remote.rs` - Minimal driver (classify_git_error is pure function)
- `src-tauri/tests/test_operation_state.rs` - 8 tests for merge/rebase state management
- `src-tauri/tests/test_merge_editor.rs` - 3 tests for conflict resolution
- `src-tauri/tests/test_interactive_rebase.rs` - 5 tests for rebase todo and fork point
- `src-tauri/tests/test_remote.rs` - 16 tests for error classification and RunningOp
- `src-tauri/tests/test_graph.rs` - 22 tests for graph walk algorithm (lane allocation, stash placement, etc.)
- `src-tauri/tests/test_repository.rs` - 2 tests for ref_map (HEAD and stash)
- `src-tauri/src/commands/operation_state.rs` - Removed #[cfg(test)] module (195 lines)
- `src-tauri/src/commands/merge_editor.rs` - Removed #[cfg(test)] module (255 lines)
- `src-tauri/src/commands/interactive_rebase.rs` - Removed #[cfg(test)] module (187 lines)
- `src-tauri/src/commands/remote.rs` - Removed #[cfg(test)] module (137 lines)
- `src-tauri/src/git/graph.rs` - Removed #[cfg(test)] module (1570 lines)
- `src-tauri/src/git/repository.rs` - Removed #[cfg(test)] module (162 lines)

## Decisions Made
- Graph tests call `walk_commits()` directly rather than through a driver, since it is an algorithm function (not a command with an `_inner` wrapper). Per research Open Question 2.
- Remote tests call `classify_git_error()` directly since it is a pure function with no state dependency (string in, TrunkError out).
- Documented existing behavior of `merge_branch_inner` where CONFLICT message is on stdout (not stderr), causing the conflict detection to fail. Tests document actual behavior rather than ideal behavior.
- Used raw git2 operations for graph tests requiring complex topologies (octopus merges, mid-chain stashes, criss-cross merges) that the builder cannot express.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Adjusted operation_state driver signatures to match actual code**
- **Found during:** Task 1 (reading source files)
- **Issue:** Plan interface section listed incorrect signatures (e.g., merge_continue_inner missing message parameter, wrong return types)
- **Fix:** Read actual source files and created drivers with correct signatures (GraphResult returns, Option<&str> message parameters)
- **Files modified:** src-tauri/tests/common/drivers/operation_state.rs
- **Verification:** All 8 operation_state tests compile and pass
- **Committed in:** 5264b56

**2. [Rule 1 - Bug] Fixed multiple_stashes_on_same_parent test for builder stash behavior**
- **Found during:** Task 2 (graph test migration)
- **Issue:** Builder's with_stash() creates an additional "Add stash marker" commit, changing the topology from what the original raw test expected
- **Fix:** Rewrote the test to use raw git2 operations (matching original test style) instead of the builder
- **Files modified:** src-tauri/tests/test_graph.rs
- **Verification:** All 22 graph tests pass
- **Committed in:** 12586f4

**3. [Rule 1 - Bug] Fixed make_merge_repo_ctx borrow checker issue**
- **Found during:** Task 2 (graph test compilation)
- **Issue:** Dropping repo while tree/commit references still borrowed caused E0505
- **Fix:** Created raw_commit_in helper that drops borrows promptly, scoped repo in a block
- **Files modified:** src-tauri/tests/test_graph.rs
- **Verification:** Compilation succeeds, all 22 graph tests pass
- **Committed in:** 12586f4

---

**Total deviations:** 3 auto-fixed (3 bugs)
**Impact on plan:** All auto-fixes necessary for compilation correctness and test accuracy. No scope creep.

## Issues Encountered
- merge_branch_inner has a known issue where `git merge` outputs CONFLICT to stdout (not stderr), causing the conflict detection to fail and return an empty error. Tests document this actual behavior. This is pre-existing code, not introduced by this plan.

## Known Stubs
None - all driver methods and test files are fully implemented.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 148+ original inline tests migrated to integration test crate
- Zero #[cfg(test)] modules remain in src-tauri/src/
- make_test_repo() and make_large_test_repo() fully removed
- 156 total integration tests pass
- Phase 53 test migration complete; ready for Phase 54

## Self-Check: PASSED

All 10 created files verified on disk. Both task commits (5264b56, 12586f4) verified in git log.

---
*Phase: 53-rust-unit-tests-test-harness*
*Completed: 2026-03-26*
