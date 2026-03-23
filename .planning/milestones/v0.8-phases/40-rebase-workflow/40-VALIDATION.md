---
phase: 40
slug: rebase-workflow
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 40 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest (frontend) + cargo test (backend) |
| **Config file** | `vitest.config.ts` / `src-tauri/Cargo.toml` |
| **Quick run command** | `npm test` |
| **Full suite command** | `npm test && cd src-tauri && cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm test`
- **After every plan wave:** Run `npm test && cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 40-01-01 | 01 | 1 | REB-01 | manual-only | N/A -- native Tauri menus | N/A | ⬜ pending |
| 40-01-02 | 01 | 1 | REB-01, REB-04, REB-05, REB-06 | manual-only | N/A -- native Tauri menus | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements.

The primary work (context menu wiring) uses native Tauri menu APIs that cannot be unit tested. Backend rebase commands are already implemented and have existing test coverage.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Rebase menu items appear in all branch context menus | REB-01 | Native Tauri menus, not DOM-testable | Right-click branch in sidebar/pill/overflow; verify "Rebase [current] onto [branch]" item present |
| Rebase item hidden on HEAD branch | REB-01 | Native menu guard logic | Right-click HEAD branch; verify no rebase item |
| Rebase item hidden when HEAD detached | REB-01 | Native menu guard logic | Detach HEAD; right-click any branch; verify no rebase item |
| Mid-rebase conflicts show in staging panel | REB-04 | Requires full Tauri app with git repo | Create conflicting branches; rebase; verify conflicts appear in staging panel |
| Abort restores pre-rebase state | REB-05 | Requires full Tauri app context | Start rebase with conflicts; click Abort; verify branch state restored |
| Skip commit during rebase | REB-06 | Requires full Tauri app context | Start rebase with conflicts; click Skip; verify commit skipped and rebase continues |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
