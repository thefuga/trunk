---
phase: 63
slug: full-file-view-display-options
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 63 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest |
| **Config file** | vitest.config.ts |
| **Quick run command** | `bun run test` |
| **Full suite command** | `bun run test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run test`
- **After every plan wave:** Run `bun run test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 63-01-01 | 01 | 1 | VIEW-04 | unit | `bun run test` | ❌ W0 | ⬜ pending |
| 63-01-02 | 01 | 1 | WHSP-02 | unit | `bun run test` | ❌ W0 | ⬜ pending |
| 63-01-03 | 01 | 1 | WHSP-03 | unit | `bun run test` | ❌ W0 | ⬜ pending |
| 63-01-04 | 01 | 1 | DISP-02 | unit | `bun run test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Full file view renders as continuous scrollable document | VIEW-04 | Visual layout verification | Open diff, switch to Full view, confirm no hunk headers, continuous scroll |
| Invisible chars display correctly (dots/arrows) | WHSP-03 | Visual glyph rendering | Toggle show invisibles, confirm spaces=·, tabs=→ |
| Word wrap wraps at container edge | DISP-02 | Visual layout verification | Toggle word wrap, confirm long lines wrap at container boundary |
| Staging buttons disabled with tooltip on whitespace ignore | WHSP-02 | Interactive UI state | Enable whitespace ignore, confirm buttons disabled, hover for tooltip |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
