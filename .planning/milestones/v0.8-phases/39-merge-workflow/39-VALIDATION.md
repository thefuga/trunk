---
phase: 39
slug: merge-workflow
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 39 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.0 (frontend), cargo test (backend) |
| **Config file** | vite.config.ts (vitest inherits) |
| **Quick run command** | `npm test` |
| **Full suite command** | `npm test && cd src-tauri && cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm test`
- **After every plan wave:** Run `npm test && cd src-tauri && cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 39-01-01 | 01 | 1 | MERGE-01 | manual-only | N/A — requires Tauri runtime for native menus | N/A | ⬜ pending |
| 39-01-02 | 01 | 1 | MERGE-03 | unit (Rust) | `cd src-tauri && cargo test operation_state::tests` | ✅ | ⬜ pending |
| 39-01-03 | 01 | 1 | MERGE-04 | unit (Rust) | `cd src-tauri && cargo test operation_state::tests` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Right-click branch in sidebar shows "Merge into current branch" | MERGE-01 | Native Tauri context menus cannot be unit-tested | Right-click local branch in sidebar → verify menu item appears → click → verify merge executes |
| Right-click branch pill in graph shows "Merge into current branch" | MERGE-01 | Native Tauri context menus cannot be unit-tested | Right-click branch pill in commit graph → verify menu item → click → verify merge |
| Fast-forward merge advances pointer without merge commit | MERGE-03 | Requires real git repo state with FF-eligible branches | Create branch at HEAD, add commit to main, merge branch → verify no merge commit created |
| Non-conflicting merge creates merge commit and graph refreshes | MERGE-04 | Requires real git repo with diverged branches | Create diverged branches, merge → verify merge commit appears in graph |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
