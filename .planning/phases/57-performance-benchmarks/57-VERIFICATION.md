---
phase: 57-performance-benchmarks
verified: 2026-03-27T06:15:06Z
status: passed
score: 9/9 must-haves verified
re_verification: false
---

# Phase 57: Performance Benchmarks Verification Report

**Phase Goal:** Critical Rust operations have reproducible benchmarks with regression detection in CI
**Verified:** 2026-03-27T06:15:06Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `cargo bench --bench bench_graph` runs walk_commits at 100, 1k, 10k commit scales | VERIFIED | bench_graph.rs lines 65-90: configs array with ("100", 100), ("1k", 1_000), ("10k", 10_000); parameterized via BenchmarkId::from_parameter |
| 2 | `cargo bench --bench bench_commands` runs list_refs_inner, diff_unstaged_inner, stage_hunk_inner, and get_status_inner benchmarks | VERIFIED | bench_commands.rs lines 150-215: four benchmark functions wired to trunk_lib functions; criterion_group includes all four |
| 3 | Benchmark fixtures are created once via OnceLock, not per iteration | VERIFIED | bench_graph.rs: REPO_100, REPO_1K, REPO_10K as OnceLock statics; bench_commands.rs: REPO_BRANCHES, REPO_UNSTAGED as OnceLock statics; stage_hunk correctly uses iter_batched (mutating) |
| 4 | `cargo test --benches --no-run --manifest-path src-tauri/Cargo.toml` compiles successfully | VERIFIED | Compile run output: both bench_commands and bench_graph executables produced with exit 0 |
| 5 | `benchmarks.yml` triggers on push to main and runs `cargo bench` with regression detection | VERIFIED | benchmarks.yml line 5: branches: [main]; line 42: cargo bench -- --output-format bencher |
| 6 | Benchmark regressions exceeding 130% cause the workflow to fail | VERIFIED | benchmarks.yml line 56: alert-threshold: '130%'; line 57: fail-on-alert: true |
| 7 | Benchmark baselines are stored in CI cache, not gh-pages | VERIFIED | benchmarks.yml line 45: actions/cache@v4 with external-data-json-path: ./cache/benchmark-data.json; no gh-pages reference found |
| 8 | `ci.yml` cargo-test job includes a compile-check step for benchmark code | VERIFIED | ci.yml lines 113-114: step "Compile-check benchmarks" with cargo test --benches --no-run inside cargo-test job |
| 9 | BENCH-03 and BENCH-04 are explicitly deferred to Phase 58 (not implemented here) | VERIFIED | No frontend IPC or startup-time benchmark code found in bench files; REQUIREMENTS.md marks BENCH-03 and BENCH-04 as Pending; Phase 57 correctly implements only BENCH-01, BENCH-02, BENCH-05 |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/Cargo.toml` | Criterion dev-dependency and [[bench]] entries | VERIFIED | Line 37: criterion = { version = "0.8", features = ["html_reports"] }; lines 39-45: both [[bench]] entries with harness = false |
| `src-tauri/benches/bench_graph.rs` | walk_commits parameterized benchmarks at 3 scales | VERIFIED | 94 lines (min: 60); OnceLock statics, make_linear_repo, BenchmarkId::from_parameter, all required criterion imports |
| `src-tauri/benches/bench_commands.rs` | Command inner function benchmarks | VERIFIED | 216 lines (min: 80); all four command functions, iter_batched for stage_hunk, OnceLock for read-only fixtures, state_map construction |
| `.github/workflows/benchmarks.yml` | Criterion benchmark CI workflow with regression detection | VERIFIED | 57 lines (min: 40); valid YAML, 130% threshold, fail-on-alert, CI cache baselines, no gh-pages |
| `.github/workflows/ci.yml` | Benchmark compile-check in existing CI pipeline | VERIFIED | Contains "Compile-check benchmarks" step with cargo test --benches --no-run --manifest-path src-tauri/Cargo.toml inside cargo-test job |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `bench_graph.rs` | `trunk_lib::git::graph::walk_commits` | direct function call in bench closure | WIRED | Line 85: trunk_lib::git::graph::walk_commits(&mut repo, 0, usize::MAX).unwrap() |
| `bench_commands.rs` | `trunk_lib::commands::branches::list_refs_inner` | direct function call in bench closure | WIRED | Line 158: list_refs_inner called with path and state_map |
| `bench_commands.rs` | `trunk_lib::commands::staging::stage_hunk_inner` | direct function call in iter_batched closure | WIRED | Lines 192-204: iter_batched with stage_hunk_inner call at line 195 |
| `benchmarks.yml` | `src-tauri/benches/` | cargo bench command | WIRED | Line 42: cd src-tauri && cargo bench -- --output-format bencher |
| `benchmarks.yml` | `benchmark-action/github-action-benchmark@v1` | uses directive | WIRED | Line 51: uses: benchmark-action/github-action-benchmark@v1 |
| `benchmarks.yml` | `actions/cache@v4` | benchmark data persistence | WIRED | Line 45: uses: actions/cache@v4 |

### Data-Flow Trace (Level 4)

Not applicable. Benchmark files do not render dynamic data to UI — they call Rust functions directly as the system under test. CI workflow files are configuration, not data-rendering components.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Both benchmark binaries compile | `cargo test --benches --no-run --manifest-path src-tauri/Cargo.toml` | bench_commands and bench_graph executables produced, Finished test profile | PASS |
| benchmarks.yml is valid YAML | `python3 -c "import yaml; yaml.safe_load(...)"` | "YAML valid" | PASS |
| ci.yml is valid YAML | `python3 -c "import yaml; yaml.safe_load(...)"` | "YAML valid" | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| BENCH-01 | 57-01-PLAN.md | Criterion benchmarks for graph lane algorithm (walk_commits) with varying repo sizes | SATISFIED | bench_graph.rs: parameterized benchmarks at 100/1k/10k scales using OnceLock-cached git2 fixtures |
| BENCH-02 | 57-01-PLAN.md | Criterion benchmarks for ref listing, diff computation, and hunk staging | SATISFIED | bench_commands.rs: list_refs_inner, diff_unstaged_inner, stage_hunk_inner (+ get_status_inner) benchmarks all present and compiling |
| BENCH-03 | 57-02-PLAN.md | Frontend IPC round-trip benchmarks for key commands | DEFERRED | Plan 02 explicitly states deferred to Phase 58 per user decision D-01; REQUIREMENTS.md correctly marks as Pending; no implementation present |
| BENCH-04 | 57-02-PLAN.md | Application startup time measurement | DEFERRED | Plan 02 explicitly states deferred to Phase 58 per user decision D-01; REQUIREMENTS.md correctly marks as Pending; no implementation present |
| BENCH-05 | 57-02-PLAN.md | CI pipeline detects performance regressions with threshold-based gates | SATISFIED | benchmarks.yml: 130% alert threshold with fail-on-alert: true, triggered on push to main; REQUIREMENTS.md marks as Complete |

**Note on BENCH-03 and BENCH-04:** Plan 02's `requirements` frontmatter claims these IDs, but a plan truth explicitly documents them as deferred. The implementations are absent and REQUIREMENTS.md correctly reflects their Pending status. This is an intentional design decision (D-01), not a gap.

### Anti-Patterns Found

No anti-patterns found across all five modified files. No TODO/FIXME/PLACEHOLDER comments, no stub implementations, no empty return values in any benchmark or workflow code.

### Human Verification Required

#### 1. Benchmark Execution End-to-End

**Test:** On a Linux machine or CI environment, run `cd src-tauri && cargo bench --bench bench_graph -- --test` and `cargo bench --bench bench_commands -- --test`
**Expected:** Both suites complete without panic, showing timing output for all benchmarks (walk_commits/100, walk_commits/1k, walk_commits/10k, list_refs_inner, diff_unstaged_inner, get_status_inner, stage_hunk_inner)
**Why human:** Cannot run cargo bench in this environment (would require full Tauri system dependencies, significant CPU time)

#### 2. GitHub Actions Workflow Trigger

**Test:** Push a commit to main and observe the Benchmarks workflow run in GitHub Actions
**Expected:** Benchmarks workflow triggers, runs successfully, and stores baseline data in CI cache; first run will not fail (no prior baseline to compare against)
**Why human:** Requires a live GitHub Actions environment

#### 3. Regression Detection Behavior

**Test:** After baseline is established, introduce an artificial 2x slowdown in walk_commits and push to main
**Expected:** Benchmarks workflow fails with a regression alert showing >130% threshold exceeded
**Why human:** Requires multiple pushes to main, baseline establishment, and code modification

### Gaps Summary

No gaps. All 9 observable truths are verified. All 5 artifacts exist, are substantive, and are correctly wired. All 6 key links are confirmed present in the code. BENCH-01, BENCH-02, and BENCH-05 are fully satisfied. BENCH-03 and BENCH-04 are correctly deferred to Phase 58 by documented user decision. The compile check confirms both benchmark binaries build without errors.

---

_Verified: 2026-03-27T06:15:06Z_
_Verifier: Claude (gsd-verifier)_
