---
phase: 69
slug: comment-management-ui
status: approved
nyquist_compliant: true
wave_0_complete: true
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
| 69-01-T1 | 01 | 1 | CMT-02, CMT-03 | T-69-01, T-69-SC | v1-shaped JSON deserializes (id=="" sentinel), uuid pinned | unit | `cargo test --manifest-path src-tauri/Cargo.toml types` | ❌ W0 | ⬜ pending |
| 69-01-T2 | 01 | 1 | CMT-02, CMT-03 | T-69-01, T-69-02, T-69-03 | v1→v2 migrate-on-load (no corrupt); v3 RefusedNewer; garbage quarantined | unit | `cargo test --manifest-path src-tauri/Cargo.toml review_store` | ❌ W0 | ⬜ pending |
| 69-02-T1 | 02 | 2 | ANCH-03, CMT-02 | T-69-05, T-69-06 | commit-level write (anchor None) + edit-by-id; missing id → not_found | unit | `cargo test --manifest-path src-tauri/Cargo.toml review` | ❌ W0 | ⬜ pending |
| 69-02-T2 | 02 | 2 | CMT-03 | T-69-05, T-69-08 | delete-by-id idempotent no-op; commands registered + emit | unit | `cargo test --manifest-path src-tauri/Cargo.toml review && cargo build --manifest-path src-tauri/Cargo.toml` | ❌ W0 | ⬜ pending |
| 69-03-T1 | 03 | 3 | CMT-01, CMT-04 | T-69-10, T-69-12 | resolve_all classifies CommitGone/FileGone/LineOutOfRange side-aware, never drops/panics | unit (in-process repo) | `cargo test --manifest-path src-tauri/Cargo.toml review` | ❌ W0 | ⬜ pending |
| 69-03-T2 | 03 | 3 | CMT-01, CMT-04 | T-69-09, T-69-11 | resolver runs git2 off-lock in spawn_blocking; reads only | unit | `cargo test --manifest-path src-tauri/Cargo.toml review && cargo build --manifest-path src-tauri/Cargo.toml` | ❌ W0 | ⬜ pending |
| 69-04-T1 | 04 | 2 | CMT-01, CMT-04 | T-69-13 | TS DTOs mirror v2 wire shape (id/commit_oid/OrphanReason/CommentResolution) | type-check | `npx svelte-check --threshold error` | ❌ W0 | ⬜ pending |
| 69-04-T2 | 04 | 2 | CMT-04 | T-69-14 | rune owns center-pane state; jumpTo returns early on null anchor | type-check | `npx svelte-check --threshold error` | ❌ W0 | ⬜ pending |
| 69-05-T1 | 05 | 4 | CMT-01, CMT-02, CMT-03, CMT-04 | T-69-15, T-69-16, T-69-17 | group-by-commit render; inline edit; delete-confirm cancel/confirm; jump vs orphan | component (vitest) | `npx vitest run src/components/ReviewPanel.test.ts` | ❌ W0 | ⬜ pending |
| 69-05-T2 | 05 | 4 | ANCH-03, CMT-04 | T-69-18 | center-pane swap wiring the rune; full suite green | integration | `just check` | ❌ W0 | ⬜ pending |
| 69-05-T3 | 05 | 4 | CMT-04 | T-69-15, T-69-17 | live jump scroll/highlight + read-only orphan badge fidelity | manual (human-check) | manual — see Manual-Only Verifications | n/a | ⬜ pending |

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

- [x] All tasks have `<automated>` verify or Wave 0 dependencies (69-05-T3 is the single manual checkpoint, backed by the Manual-Only Verifications table)
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved
