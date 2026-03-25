---
phase: 49
slug: tab-drag-tree-context-menu
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 49 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (via Vite config) |
| **Config file** | vite.config.ts (test section) |
| **Quick run command** | `bun run test` |
| **Full suite command** | `bun run test && bun run check` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `bun run test`
- **After every plan wave:** Run `bun run test && bun run check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 49-01-01 | 01 | 1 | TAB-11 | manual | N/A (SortableJS DOM interaction) | N/A | ⬜ pending |
| 49-01-02 | 01 | 1 | TAB-11 | regression | `bun run test` | ✅ | ⬜ pending |
| 49-02-01 | 02 | 1 | TREE-11 | manual | N/A (Tauri native menus) | N/A | ⬜ pending |
| 49-02-02 | 02 | 1 | TREE-11 | regression | `bun run test` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No new test files needed.

Both TAB-11 and TREE-11 are primarily UI interaction features (drag-and-drop, native context menus) requiring the full Tauri runtime and DOM environment. The underlying logic (array reordering, directory prefix matching) is already tested via existing build-tree and flatten-tree tests.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Tab drag reorder persists order | TAB-11 | SortableJS manipulates DOM directly; requires real browser + Tauri runtime | 1. Open 3+ tabs. 2. Drag tab 3 to position 1. 3. Verify tab order updates. 4. Quit and relaunch. 5. Verify order persisted. |
| Directory context menu — Stage All | TREE-11 | Tauri native menus not available in vitest | 1. Create unstaged files in a directory. 2. Switch to tree view. 3. Right-click directory. 4. Click "Stage All". 5. Verify all files in directory are staged. |
| Directory context menu — Unstage All | TREE-11 | Tauri native menus not available in vitest | 1. Stage files in a directory. 2. Right-click staged directory. 3. Click "Unstage All". 4. Verify all files in directory are unstaged. |
| Directory context menu — Discard All | TREE-11 | Tauri native menus + confirmation dialog | 1. Create unstaged changes in a directory. 2. Right-click directory. 3. Click "Discard All". 4. Verify confirmation dialog appears. 5. Confirm. 6. Verify changes discarded. |
| Directory context menu — Resolve/Unresolve All | TREE-11 | Tauri native menus + merge conflict state | 1. Create merge conflict with conflicted files in a directory. 2. Right-click conflicted directory. 3. Click "Resolve All". 4. Verify files marked resolved. 5. Right-click again. 6. Click "Unresolve All". 7. Verify files returned to conflicted. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
