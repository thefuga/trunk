---
phase: 30
slug: graph-polish-navigation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 30 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest ^4.1.0 (frontend), cargo test (Rust backend) |
| **Config file** | vite.config (Vitest), Cargo.toml (Rust) |
| **Quick run command** | `npm test` |
| **Full suite command** | `npm test && cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm test && cargo test`
- **After every plan wave:** Run `npm test && cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 30-01-01 | 01 | 1 | GRAPH-01 | manual-only | N/A — CSS visual | N/A | ⬜ pending |
| 30-01-02 | 01 | 1 | GRAPH-02 | manual-only | N/A — CSS visual | N/A | ⬜ pending |
| 30-02-01 | 02 | 1 | GRAPH-03 | unit (Rust) | `cargo test resolve_ref` | ❌ W0 | ⬜ pending |
| 30-02-02 | 02 | 1 | GRAPH-03 | manual (scroll) | N/A — UI interaction | N/A | ⬜ pending |
| 30-03-01 | 03 | 2 | LAYOUT-01 | manual-only | N/A — UI state | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/commands/branches.rs` — unit test for `resolve_ref_inner` (GRAPH-03 backend)

*GRAPH-01, GRAPH-02, LAYOUT-01 are CSS/state changes — no automated test infrastructure needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visible padding above first and below last commit row | GRAPH-01 | CSS visual spacing | Open app, check graph has visible gap above first row and below last row |
| Graph column shrinks narrower than lane content | GRAPH-02 | CSS resize behavior | Drag graph column resize handle leftward past lane content width — lanes should compress, no horizontal scroll |
| Clicking branch/tag in sidebar scrolls graph to that commit | GRAPH-03 | UI interaction flow | Click a branch name in sidebar, verify graph scrolls to center that commit's row |
| Right pane auto-opens when clicking commit while pane collapsed | LAYOUT-01 | UI state transition | Collapse right pane, click a commit in graph, verify pane opens and shows commit detail |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
