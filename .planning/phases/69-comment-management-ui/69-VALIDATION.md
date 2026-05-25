---
phase: 69
slug: comment-management-ui
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-26
---

# Phase 69 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Source validation architecture: `69-RESEARCH.md` § Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` (in-process `tempfile::TempDir` + `git2::Repository::init`, no mocks); Frontend Vitest + Testing Library (`*.test.ts`) |
| **Config file** | `src-tauri/Cargo.toml` (`[dev-dependencies] tempfile`, `tauri test`); Vitest via repo `package.json` |
| **Quick run command** | `cargo test --manifest-path src-tauri/Cargo.toml review` |
| **Full suite command** | `just check` |
| **Estimated runtime** | quick ~15s · full `just check` ~2 min (estimate) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path src-tauri/Cargo.toml review` (+ relevant Vitest file)
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** `just check` must be green
- **Max feedback latency:** ~15 seconds (quick run)

---

## Per-Task Verification Map

> Populated by the planner from PLAN.md task IDs. Requirement→behavior map is locked in `69-RESEARCH.md` § Phase Requirements → Test Map.

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 69-XX-XX | XX | X | REQ-XX | — | N/A | unit | `cargo test … review` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/git/review_store.rs` tests — v1→v2 migration + id backfill + D-16 (newer-version refusal) preserved (extend existing `#[cfg(test)]` mod)
- [ ] `src-tauri/src/commands/review.rs` tests — `add_commit_comment`, `list_session_comments`, `edit_comment`, `delete_comment`, resolvability classifier (reuse `make_repo`, `seeded_sessions` helpers)
- [ ] `src/components/ReviewPanel.test.ts` — panel render, group-by-commit, inline edit, delete-confirm, jump vs orphan (new file)

*No framework install needed — `tempfile`, `tauri test`, Vitest all present.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Jump reveals the diff/full-file view, scrolls to the line range, and highlights it | CMT-04 | Visual scroll/highlight fidelity in the live right pane is not fully assertable in unit/component tests | In a session with a resolvable line-anchored comment, click jump; confirm the correct commit+file opens (diff for `Source::Diff`, full-file for `Source::FullFile`), scrolls to and highlights the range |
| Orphaned comment shows read-only state with the correct reason badge | CMT-04 | Visual badge state across commit-gone / file-gone / line-out-of-range | Force each orphan condition; confirm jump disabled and the badge reason matches |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
