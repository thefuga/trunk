---
phase: 29
slug: staging-commit-ux
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 29 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.0 |
| **Config file** | `vite.config.ts` (test section) |
| **Quick run command** | `npm test` |
| **Full suite command** | `npm test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm test`
- **After every plan wave:** Run `npm test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 29-01-01 | 01 | 1 | STAGE-03, STAGE-04 | visual | N/A — inline CSS changes | N/A | ⬜ pending |
| 29-01-02 | 01 | 1 | STAGE-05 | visual | N/A — layout CSS changes | N/A | ⬜ pending |
| 29-02-01 | 02 | 1 | STAGE-01 | unit | `npx vitest run src/lib/commit-mode.test.ts` | ❌ W0 | ⬜ pending |
| 29-02-02 | 02 | 1 | STAGE-01, STAGE-02 | integration | `npm test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/lib/commit-mode.test.ts` — unit tests for mode state logic (button labels per mode, validation rules per mode, message preservation across switches)

*This phase is heavily UI/CSS — most requirements (STAGE-03, STAGE-04, STAGE-05) are visual and cannot be automatically tested with the current Vitest+Node setup (no DOM, no Svelte component testing configured). Unit tests cover mode state logic only (STAGE-01).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Stage All button green | STAGE-03 | Visual CSS assertion | Inspect StagingPanel — button has `background: #22c55e` and `color: white` |
| Unstage All button red | STAGE-04 | Visual CSS assertion | Inspect StagingPanel — button has `background: #f87171` and `color: white` |
| File lists equal height | STAGE-05 | Layout assertion | With files in both sections, verify each section takes 50% of available space |
| Stash name auto-populates | STAGE-02 | Requires Tauri IPC | Switch to stash mode, type in subject, submit — verify subject passed as stash message |
| Tab selector UI | STAGE-01 | Visual component test | Verify tab row renders above subject input, underline indicator on active tab |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
