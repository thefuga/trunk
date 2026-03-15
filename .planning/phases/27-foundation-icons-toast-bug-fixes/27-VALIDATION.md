---
phase: 27
slug: foundation-icons-toast-bug-fixes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 27 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.0 (TypeScript) + Cargo test (Rust) |
| **Config file** | `vite.config.ts` (implicit) |
| **Quick run command** | `npx vitest run` |
| **Full suite command** | `npx vitest run && cd src-tauri && cargo test --lib` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npx vitest run` (TypeScript) + `cd src-tauri && cargo test --lib` (Rust)
- **After every plan wave:** Run `npx vitest run && cd src-tauri && cargo test --lib`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 27-01-01 | 01 | 0 | TOAST-01 | unit | `npx vitest run src/lib/toast.svelte.test.ts` | ❌ W0 | ⬜ pending |
| 27-01-02 | 01 | 0 | FIX-01 | Rust unit | `cd src-tauri && cargo test --lib staging` | ❌ W0 | ⬜ pending |
| 27-02-01 | 02 | 1 | TOAST-01 | unit | `npx vitest run src/lib/toast.svelte.test.ts` | ❌ W0 | ⬜ pending |
| 27-03-01 | 03 | 1 | ICON-01 | visual | manual — verify icons render | ✅ manual only | ⬜ pending |
| 27-04-01 | 04 | 1 | FIX-01 | Rust unit | `cd src-tauri && cargo test --lib staging` | ❌ W0 | ⬜ pending |
| 27-04-02 | 04 | 1 | FIX-02 | visual | manual — verify no trailing divider | ✅ manual only | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/lib/toast.svelte.test.ts` — stubs for TOAST-01 (store push/pop/auto-dismiss)
- [ ] `src-tauri/src/commands/staging.rs` — new test `get_dirty_counts_includes_untracked` for FIX-01 (follow existing `#[cfg(test)]` pattern with `make_test_repo`)

*Wave 0 creates test scaffolds before implementation so executor has failing tests to pass.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Icons render as SVG, no Unicode glyphs | ICON-01 | Visual rendering can't be unit-tested in Vitest/jsdom | Launch app, inspect each toolbar button, file row, sidebar section, tab bar item — confirm SVG icons appear, no `&#8617;` etc. |
| Last visible column header has no trailing divider | FIX-02 | CSS visual layout requires browser rendering | Hide a column via context menu; confirm right edge of last visible column has no resize handle |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
