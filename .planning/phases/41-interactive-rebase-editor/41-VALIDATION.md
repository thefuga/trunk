---
phase: 41
slug: interactive-rebase-editor
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 41 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend), cargo test (backend) |
| **Config file** | `vite.config.ts` (vitest), `Cargo.toml` (cargo test) |
| **Quick run command** | `npm run test -- --run` |
| **Full suite command** | `npm run test -- --run && cd src-tauri && cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm run test -- --run`
- **After every plan wave:** Run `npm run test -- --run && cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | REB-03 | manual | Context menu check | N/A | ⬜ pending |
| TBD | TBD | TBD | IREB-01 | manual | Editor opens with commit list | N/A | ⬜ pending |
| TBD | TBD | TBD | IREB-02 | unit | `npm run test -- --run` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | IREB-03 | unit | `npm run test -- --run` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | IREB-04 | unit | `npm run test -- --run` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | IREB-05 | manual | Reword dialog check | N/A | ⬜ pending |
| TBD | TBD | TBD | IREB-06 | manual | Squash dialog check | N/A | ⬜ pending |
| TBD | TBD | TBD | IREB-07 | manual | Mid-rebase conflict resolution | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/lib/__tests__/rebase-validation.test.ts` — stubs for IREB-02, IREB-03, IREB-04
- [ ] Validation logic tests (squash-first-commit, drop-all-commits)

*Existing infrastructure covers backend requirements via cargo test.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Context menu "Interactive Rebase" opens editor | REB-03 | GUI interaction | Right-click commit → select Interactive Rebase → verify editor opens |
| Commit list display with action selectors | IREB-01 | GUI rendering | Verify commits listed with Pick/Squash/Reword/Drop dropdowns |
| Drag-and-drop reordering | IREB-02 | GUI interaction | Drag row up/down → verify reorder |
| Reword message dialog | IREB-05 | GUI interaction + git execution | Set Reword → Start → verify dialog appears |
| Squash message dialog | IREB-06 | GUI interaction + git execution | Set Squash → Start → verify concatenated messages |
| Mid-rebase conflict resolution | IREB-07 | Full git workflow | Create conflicting commits → rebase → verify merge editor opens |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
