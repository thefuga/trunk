---
phase: 42
slug: rebase-skip-inline-ui
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-23
---

# Phase 42 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (via vite.config.ts) |
| **Config file** | `vite.config.ts` test section |
| **Quick run command** | `bun run test` |
| **Full suite command** | `bun run test && bun run check` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run test`
- **After every plan wave:** Run `bun run test && bun run check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 42-01-01 | 01 | 1 | REB-06 | manual-only | Visual: Skip button between Continue and Abort | N/A | ⬜ pending |
| 42-01-02 | 01 | 1 | REB-06 | manual-only | Visual: Skip invokes rebase_skip IPC | N/A | ⬜ pending |
| 42-01-03 | 01 | 1 | REB-06 | regression | `bun run test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test files needed — the new code is UI template + thin IPC handler with no unit-testable logic.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Skip button appears in StagingPanel rebase form between Continue and Abort | REB-06 | Svelte component template — vitest runs in node, no DOM rendering | 1. Start dev mode (`bun run dev`) 2. Init a rebase with conflicts 3. Verify Skip Commit button appears between Continue Rebase and Abort Rebase |
| Skip invokes rebase_skip IPC and refreshes UI | REB-06 | IPC integration requires running Tauri app | 1. During active rebase with conflicts 2. Click Skip Commit 3. Verify commit is skipped, graph refreshes, no toast shown |
| OperationBanner skip no longer shows toast | REB-06 | Component event handler, requires running app | 1. During active rebase 2. Click Skip in OperationBanner 3. Verify no "Commit skipped" toast appears |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
