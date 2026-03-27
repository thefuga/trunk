---
phase: 57
slug: performance-benchmarks
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-27
---

# Phase 57 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Criterion 0.8.2 |
| **Config file** | `src-tauri/Cargo.toml` (dev-dependency + [[bench]] entries) |
| **Quick run command** | `cd src-tauri && cargo test --benches --no-run` |
| **Full suite command** | `cd src-tauri && cargo bench` |
| **Estimated runtime** | ~60 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test --benches --no-run`
- **After every plan wave:** Run `cd src-tauri && cargo bench --bench bench_graph -- --test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 57-01-01 | 01 | 1 | BENCH-01 | benchmark | `cd src-tauri && cargo bench --bench bench_graph` | ❌ W0 | ⬜ pending |
| 57-01-02 | 01 | 1 | BENCH-02 | benchmark | `cd src-tauri && cargo bench --bench bench_commands` | ❌ W0 | ⬜ pending |
| 57-02-01 | 02 | 2 | BENCH-05 | integration | `cargo test --benches --no-run --manifest-path src-tauri/Cargo.toml` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/benches/bench_graph.rs` — stubs for BENCH-01
- [ ] `src-tauri/benches/bench_commands.rs` — stubs for BENCH-02
- [ ] `src-tauri/Cargo.toml` — criterion dev-dependency + [[bench]] entries
- [ ] `.github/workflows/benchmarks.yml` — CI regression detection workflow for BENCH-05
- [ ] `.github/workflows/ci.yml` — add `cargo test --benches --no-run` compile-check

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Benchmark regression detection alerts | BENCH-05 | Requires CI run with cached baseline to compare against | Push to main, verify benchmark-action step passes and would fail on >130% regression |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
