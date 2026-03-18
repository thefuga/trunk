---
phase: 32
slug: hunk-staging-backend
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 32 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `src-tauri/Cargo.toml` |
| **Quick run command** | `cargo test -p trunk-lib --test-threads=4 2>&1 | tail -20` |
| **Full suite command** | `cargo test -p trunk-lib 2>&1 | tail -30` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p trunk-lib --test-threads=4 2>&1 | tail -20`
- **After every plan wave:** Run `cargo test -p trunk-lib 2>&1 | tail -30`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 32-01-01 | 01 | 1 | HUNK-01 | unit | `cargo test stage_hunk` | ✅ | ⬜ pending |
| 32-01-02 | 01 | 1 | HUNK-02 | unit | `cargo test unstage_hunk` | ✅ | ⬜ pending |
| 32-01-03 | 01 | 1 | HUNK-03 | unit | `cargo test discard_hunk` | ✅ | ⬜ pending |
| 32-01-04 | 01 | 2 | HUNK-05 | unit | `cargo test hunk` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The codebase already has:
- `src-tauri/src/commands/staging.rs` with test helpers (`make_test_repo`, `make_state_map`)
- `cargo test` infrastructure in place
- No new test framework installation needed

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Untracked (new) file hunk apply via `ApplyLocation::Index` | HUNK-01 | Edge case noted as LOW confidence in research | Create new file, stage one hunk, verify index contains partial file |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
