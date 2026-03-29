---
phase: 62
slug: ui-refactor-component-structure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 62 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest |
| **Config file** | vite.config.ts |
| **Quick run command** | `bun run test` |
| **Full suite command** | `bun run test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run test`
- **After every plan wave:** Run `bun run test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 62-01-01 | 01 | 1 | VIEW-01 | unit | `bun run test` | ✅ | ⬜ pending |
| 62-01-02 | 01 | 1 | DISP-01 | unit | `bun run test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| View mode segmented control renders with active state | VIEW-01 | Visual verification | Toggle between Hunk/Full/Split modes; active state should have highlighted background |
| Line numbers display correctly in gutter | DISP-01 | Visual verification | Open a diff with multi-digit line numbers; verify old/new columns align |
| Hunk stage/unstage/discard still works after refactor | VIEW-01 | Integration with git ops | Stage a hunk, verify file moves to staged; unstage, verify it moves back |
| Keyboard navigation [/] between hunks | VIEW-01 | Interactive behavior | Press ] to advance, [ to go back; verify smooth scroll |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
