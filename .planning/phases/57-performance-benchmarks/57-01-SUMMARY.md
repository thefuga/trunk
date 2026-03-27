---
phase: 57-performance-benchmarks
plan: 01
subsystem: testing
tags: [criterion, benchmarks, rust, git2, performance]

# Dependency graph
requires:
  - phase: 53-test-harness
    provides: inner-fn pattern for testable Tauri commands
provides:
  - Criterion benchmark suites for walk_commits and command inner functions
  - Parameterized walk_commits benchmarks at 100/1k/10k commit scales
  - OnceLock-cached git2 fixture generation patterns
affects: [57-02-ci-benchmark-pipeline]

# Tech tracking
tech-stack:
  added: [criterion 0.8 with html_reports]
  patterns: [OnceLock fixture caching, iter_batched for mutating benchmarks, in-memory git2 repo generation]

key-files:
  created:
    - src-tauri/benches/bench_graph.rs
    - src-tauri/benches/bench_commands.rs
  modified:
    - src-tauri/Cargo.toml

key-decisions:
  - "Used OnceLock for read-only fixtures, iter_batched for mutating stage_hunk benchmark"
  - "Added get_status_inner benchmark (D-03 discretion) as it exercises repo.statuses() on every poll cycle"
  - "Linear repos only for walk_commits scaling benchmark to isolate lane algorithm per pitfall 5"

patterns-established:
  - "OnceLock<BenchRepo> pattern for cached benchmark fixtures"
  - "iter_batched with BatchSize::SmallInput for mutating git operations"
  - "In-memory git2 blob+treebuilder+commit for fast fixture generation"

requirements-completed: [BENCH-01, BENCH-02]

# Metrics
duration: 9min
completed: 2026-03-27
---

# Phase 57 Plan 01: Criterion Benchmark Suites Summary

**Criterion benchmarks for walk_commits at 100/1k/10k scales and 4 command inner functions (list_refs, diff_unstaged, get_status, stage_hunk) with OnceLock-cached git2 fixtures**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-27T05:57:55Z
- **Completed:** 2026-03-27T06:07:13Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Criterion 0.8 dev-dependency with html_reports feature and two [[bench]] entries in Cargo.toml
- bench_graph.rs with parameterized walk_commits benchmarks at 100, 1k, and 10k commit scales using OnceLock-cached in-memory git2 fixtures
- bench_commands.rs with benchmarks for list_refs_inner (50-branch fixture), diff_unstaged_inner, get_status_inner (unstaged changes fixture), and stage_hunk_inner (iter_batched with fresh fixture per iteration)
- Both benchmark binaries compile and pass smoke tests (cargo bench -- --test)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Criterion dependency and walk_commits benchmark suite** - `547b176` (feat)
2. **Task 2: Create command inner function benchmark suite** - `8172d11` (feat)

## Files Created/Modified
- `src-tauri/Cargo.toml` - Added criterion dev-dependency and [[bench]] entries
- `src-tauri/benches/bench_graph.rs` - walk_commits parameterized benchmarks at 3 scales with OnceLock fixtures
- `src-tauri/benches/bench_commands.rs` - 4 command inner function benchmarks with state_map construction

## Decisions Made
- Used OnceLock for read-only benchmark fixtures (list_refs, diff_unstaged, get_status) and iter_batched for the mutating stage_hunk benchmark -- per D-09 and pitfall 2
- Added get_status_inner benchmark per D-03 discretion -- it exercises repo.statuses() which is called every poll cycle
- Kept walk_commits fixtures as linear single-branch repos to isolate lane algorithm scaling per pitfall 5
- Set sample_size(20) for the 10k walk_commits case since each iteration is slower

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Benchmark suites are ready for CI integration (Plan 02: benchmarks.yml workflow and ci.yml compile-check)
- Both binaries verified with cargo bench -- --test smoke runs
- Criterion output format ready for benchmark-action parsing

## Self-Check: PASSED

- All 3 files verified on disk
- Both commit hashes (547b176, 8172d11) verified in git log

---
*Phase: 57-performance-benchmarks*
*Completed: 2026-03-27*
