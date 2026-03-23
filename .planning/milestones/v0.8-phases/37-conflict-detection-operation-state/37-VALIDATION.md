---
phase: 37
slug: conflict-detection-operation-state
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-20
---

# Phase 37 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.0 (frontend), cargo test (backend) |
| **Config file** | `vite.config.ts` (test section), `Cargo.toml` |
| **Quick run command** | `cargo test -p trunk` |
| **Full suite command** | `npm run test && cargo test -p trunk` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p trunk`
- **After every plan wave:** Run `npm run test && cargo test -p trunk`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 37-01-01 | 01 | 1 | OPS-01 | unit | `cargo test -p trunk operation_state` | ❌ W0 | ⬜ pending |
| 37-01-02 | 01 | 1 | OPS-02 | unit | `cargo test -p trunk operation_state` | ❌ W0 | ⬜ pending |
| 37-01-03 | 01 | 1 | OPS-03 | unit | `cargo test -p trunk operation_state` | ❌ W0 | ⬜ pending |
| 37-02-01 | 02 | 2 | CONF-01 | manual | Visual inspection in Tauri dev | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/commands/operation_state.rs` — unit tests for `get_operation_state_inner` with merge, rebase, clean states (OPS-01, OPS-02)
- [ ] `src-tauri/src/commands/operation_state.rs` — unit tests for `merge_continue_inner`, `merge_abort_inner`, `rebase_continue_inner`, `rebase_skip_inner`, `rebase_abort_inner` (OPS-03)
- [ ] `src-tauri/src/commands/operation_state.rs` — tests for MERGE_MSG branch name parsing edge cases

*Existing infrastructure covers frontend test framework (Vitest already configured).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Conflicted files section styling (yellow warning icon, count badge, collapsible) | CONF-01 | Visual/CSS verification | 1. Create merge conflict in test repo 2. Open Trunk 3. Verify yellow warning icon in section header 4. Verify yellow count badge shows correct count 5. Verify section collapses/expands |
| Operation banner color-coding (yellow for merge, blue for rebase) | OPS-01/OPS-02 | Visual/CSS verification | 1. Create merge conflict → verify yellow-tinted banner 2. Create rebase conflict → verify blue-tinted banner |
| DiffPanel shows raw conflict markers for conflicted files | CONF-01 | Visual/interaction verification | 1. Click conflicted file 2. Verify <<<<<<< / ======= / >>>>>>> markers visible 3. Verify no hunk action buttons |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
