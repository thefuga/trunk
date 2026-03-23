---
phase: 38
slug: merge-editor
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-20
---

# Phase 38 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust: `cargo test` (built-in), TypeScript: vitest 4.1.0 |
| **Config file** | vite.config.ts (test section), Cargo.toml |
| **Quick run command** | `cd src-tauri && cargo test merge_editor -- --nocapture` |
| **Full suite command** | `cd src-tauri && cargo test && cd .. && npx vitest run` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd src-tauri && cargo test merge_editor -- --nocapture`
- **After every plan wave:** Run `cd src-tauri && cargo test && cd .. && npx vitest run`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 38-01-01 | 01 | 0 | CONF-02 | unit (Rust) | `cd src-tauri && cargo test merge_editor -x` | ❌ W0 | ⬜ pending |
| 38-01-02 | 01 | 0 | CONF-02 | unit (Rust) | `cd src-tauri && cargo test merge_editor -x` | ❌ W0 | ⬜ pending |
| 38-02-01 | 02 | 0 | CONF-04, CONF-05 | unit (TS) | `npx vitest run src/lib/merge-parser.test.ts` | ❌ W0 | ⬜ pending |
| 38-02-02 | 02 | 0 | CONF-07 | unit (TS) | `npx vitest run src/lib/merge-parser.test.ts` | ❌ W0 | ⬜ pending |
| 38-02-03 | 02 | 0 | CONF-08 | unit (TS) | `npx vitest run src/lib/merge-parser.test.ts` | ❌ W0 | ⬜ pending |
| 38-03-01 | 03 | 1 | CONF-09 | unit (Rust) | `cd src-tauri && cargo test merge_editor -x` | ❌ W0 | ⬜ pending |
| 38-XX-XX | XX | X | CONF-03 | manual-only | N/A -- requires browser DOM | N/A | ⬜ pending |
| 38-XX-XX | XX | X | CONF-06 | manual-only | N/A -- requires browser DOM | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/commands/merge_editor.rs` — new module with tests for get_merge_sides and save_merge_result
- [ ] `src/lib/merge-parser.ts` — conflict region parsing + output computation logic
- [ ] `src/lib/merge-parser.test.ts` — unit tests for parsing and selection logic

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Synchronized scroll across all three panels | CONF-03 | Requires browser DOM and scroll event interaction | Open merge editor with conflicted file, scroll in any panel, verify all three panels scroll together |
| Output textarea is directly editable | CONF-06 | Requires browser DOM interaction with textarea | Open merge editor, click in output panel, type text, verify edits are preserved |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
