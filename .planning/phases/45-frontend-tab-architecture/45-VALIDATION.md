---
phase: 45
slug: frontend-tab-architecture
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 45 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest 4.1.0 |
| **Config file** | `vite.config.ts` (test section) |
| **Quick run command** | `bun run test` |
| **Full suite command** | `bun run test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run test`
- **After every plan wave:** Run `bun run test && bun run check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 45-01-01 | 01 | 1 | TAB-05 | unit | `bun run test -- src/lib/remote-state.svelte.test.ts` | ❌ W0 | ⬜ pending |
| 45-01-02 | 01 | 1 | TAB-05 | unit | `bun run test -- src/lib/undo-redo.svelte.test.ts` | ❌ W0 | ⬜ pending |
| 45-01-03 | 01 | 1 | TAB-06 | unit | `bun run test -- src/lib/store.test.ts` | ❌ W0 | ⬜ pending |
| 45-XX-XX | XX | X | TAB-01 | manual-only | N/A (UI rendering) | N/A | ⬜ pending |
| 45-XX-XX | XX | X | TAB-02 | manual-only | N/A (keyboard + UI) | N/A | ⬜ pending |
| 45-XX-XX | XX | X | TAB-03 | manual-only | N/A (keyboard + UI) | N/A | ⬜ pending |
| 45-XX-XX | XX | X | TAB-04 | manual-only | N/A (keyboard + UI) | N/A | ⬜ pending |

*Removed 45-01-04 (src/lib/tab-dirty.test.ts): dirty detection is an inline boolean expression (`staged + unstaged > 0`) in App.svelte's repo-changed listener, not an extracted pure function. A dedicated test file would be testing framework wiring, not logic. TAB-07 is verified via manual testing in Plan 03's checkpoint.*

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/lib/remote-state.svelte.test.ts` — test that createRemoteState returns independent instances
- [ ] `src/lib/undo-redo.svelte.test.ts` — test that createUndoRedoState instances are independent (push/pop/clear)
- [ ] Tab persistence helpers unit tests (serialize/deserialize tab list)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Multiple tabs open simultaneously | TAB-01 | UI rendering — requires visual confirmation | Open 2+ repos via Cmd+T, verify separate tabs render with correct content |
| Cmd+T creates new tab | TAB-02 | Keyboard + UI interaction | Press Cmd+T, verify splash/project picker appears in new tab |
| Cmd+W / X closes tab | TAB-03 | Keyboard + UI interaction | Close tab via Cmd+W and X button, verify tab removed and adjacent tab activates |
| Cmd+1-9, Ctrl+Tab switching | TAB-04 | Keyboard shortcuts | Open 3+ tabs, verify Cmd+1/2/3 switches correctly, Ctrl+Tab cycles forward, Ctrl+Shift+Tab cycles backward |
| Dirty indicator on background tab | TAB-07 | Visual badge rendering | Modify file in background tab's repo, verify dot badge appears on that tab |
| Tab persistence across relaunch | TAB-06 | App lifecycle | Open 3 tabs, quit app, relaunch, verify same 3 tabs restored with correct active tab |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
