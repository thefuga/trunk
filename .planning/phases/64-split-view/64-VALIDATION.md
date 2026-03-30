---
phase: 64
slug: split-view
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-30
---

# Phase 64 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 3.x + @testing-library/svelte |
| **Config file** | vitest.config.ts |
| **Quick run command** | `npx vitest --run src/components/DiffPanel.test.ts` |
| **Full suite command** | `npx vitest --run` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest --run src/components/DiffPanel.test.ts`
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 64-01-01 | 01 | 1 | VIEW-02 | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "VIEW-02"` | ❌ W0 | ⬜ pending |
| 64-01-02 | 01 | 1 | - | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "segmented control"` | ✅ (needs update) | ⬜ pending |
| 64-02-01 | 02 | 2 | VIEW-02 | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "VIEW-02"` | ❌ W0 | ⬜ pending |
| 64-02-02 | 02 | 2 | VIEW-03 | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "VIEW-03"` | ❌ W0 | ⬜ pending |
| 64-03-01 | 03 | 2 | VIEW-05 | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "VIEW-05"` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Update existing DiffPanel.test.ts mocks: replace `getDiffViewMode`/`setDiffViewMode` with `getDiffContentMode`/`setDiffContentMode`/`getDiffLayoutMode`/`setDiffLayoutMode`
- [ ] Update toolbar test assertions from 3-button to 2+2 button layout
- [ ] Add VIEW-02 tests: split view renders left/right panels, phantom rows present, correct gutter (old-only left, new-only right)
- [ ] Add VIEW-03 tests: scroll sync (mock scrollTop assignment, verify propagation)
- [ ] Add VIEW-05 tests: staging buttons in split hunk headers, line selection on right panel, disabled when whitespace ignore active
- [ ] Add store.test.ts cases for `getDiffContentMode`/`getDiffLayoutMode`

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Resizable divider drag | D-08 | Mouse interaction not testable in jsdom | Drag divider left/right, verify panels resize, check 20-80% clamp |
| Scroll sync visual alignment | VIEW-03 | jsdom doesn't have real scroll layout | Scroll one panel, verify other follows; check alignment with word wrap on |
| Phantom row height with word wrap | D-07 | CSS grid height behavior not in jsdom | Enable word wrap, view split diff with long lines, verify phantom rows match height |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
