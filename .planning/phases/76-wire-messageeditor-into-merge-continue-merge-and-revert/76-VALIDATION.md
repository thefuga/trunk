---
phase: 76
slug: wire-messageeditor-into-merge-continue-merge-and-revert
status: approved
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-29
---

# Phase 76 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from 76-RESEARCH.md §Validation Architecture (Phase Requirements → Test Map).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[cfg(test)]` + `cargo test --lib` · frontend vitest + @testing-library/svelte + tauri `mockIPC` |
| **Config file** | none — existing infrastructure (temp-repo helper analog at `src-tauri/src/git/review.rs:662`) |
| **Quick run (Rust)** | `cd src-tauri && cargo test --lib operation_state commit_actions` |
| **Quick run (frontend)** | `npx vitest run src/components/StagingPanel.test.ts src/components/OperationBanner.test.ts` |
| **Full suite command** | `just check` (fmt + biome + svelte-check + clippy + cargo-test + vitest) |
| **Estimated runtime** | ~60-120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib operation_state commit_actions` (Rust tasks) or targeted `npx vitest run <file>` (frontend tasks)
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** `just check` must be green (success criterion 6)
- **Max feedback latency:** 120 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 76-01-T0 | 01 | 1 | MSG-01, MSG-02 | — | N/A | tdd (RED) | `cargo test --lib operation_state` | ❌ W0 | ⬜ pending |
| 76-01-T1 | 01 | 1 | MSG-01, MSG-02 | T-cmd-inject | edited msg + branch passed as argv elements (no shell); `--cleanup=strip` drops `# Conflicts:`; `MERGE_HEAD` cleared; ff probe leaves repo untouched | tdd (GREEN) | `cargo test --lib operation_state` | ❌ W0 | ⬜ pending |
| 76-02-T0 | 02 | 2 | MSG-03, MSG-06 | — | N/A | tdd (RED) | `cargo test --lib commit_actions` | ❌ W0 | ⬜ pending |
| 76-02-T1 | 02 | 2 | MSG-03, MSG-06 | T-cmd-inject | edited msg argv-only; full 40-char OID in default; `--cleanup=strip`; `revert_continue` clears `REVERT_HEAD`; `revert_abort` recovers clean tree | tdd (GREEN) | `cargo test --lib commit_actions` | ❌ W0 | ⬜ pending |
| 76-03-T0 | 03 | 3 | MSG-02, MSG-03 | — | N/A | auto | `npx vitest run src/components/RepoView.test.ts` | ✅ | ⬜ pending |
| 76-03-T1 | 03 | 3 | MSG-02, MSG-03, MSG-06 | — | begin emits `repo-changed` → cancel state stays visible | auto | `npx vitest run src/components/CommitGraph.test.ts src/components/BranchSidebar.test.ts` | ✅ | ⬜ pending |
| 76-04-T0 | 04 | 4 | MSG-01 | — | live StagingPanel form routes through host modal; dead OperationBanner:33 path NOT wired | auto | `npx vitest run src/components/StagingPanel.test.ts` | ✅ | ⬜ pending |
| 76-04-T1 | 04 | 4 | MSG-06 | — | OperationBanner renders Revert Continue/Abort; Abort → `revert_abort` | auto | `npx vitest run src/components/OperationBanner.test.ts` | ✅ | ⬜ pending |
| 76-04-T2 | 04 | 4 | MSG-01, MSG-06 | — | full GUI merge/revert + cancel-then-recover loop | manual UAT | — (human-verify checkpoint) | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*
*`❌ W0` = test infrastructure created in that task's own Wave 0 (RED) step; not a pre-existing gap.*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/commands/operation_state.rs` — add `#[cfg(test)]` module + temp-repo helper (mirror `git/review.rs:662` `make_repo`); covers MSG-01/02 git-state. Created in 76-01 Task 0.
- [ ] `src-tauri/src/commands/commit_actions.rs` — add `#[cfg(test)]` module; covers MSG-03/06 revert git-state + `revert_abort`. Created in 76-02 Task 0.
- [ ] Extend `RepoView.test.ts`, `StagingPanel.test.ts`, `OperationBanner.test.ts`, `CommitGraph.test.ts`, `BranchSidebar.test.ts` for editor-routing, null-abort, and begin-emit assertions (tasks 76-03/76-04).
- [ ] No new framework install needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Full end-to-end UX: real GUI merge/revert, edit message, commit; then cancel and recover via OperationBanner | MSG-01, MSG-06 | Integration + vitest prove state mutation + emit, but the visible banner-after-cancel recovery loop is best confirmed in the live app | 76-04 Task 2 checkpoint: trigger each of the 3 ops in the running app, confirm editor pre-fill matches git's default, commit lands edited message, and Esc/empty leaves a visible recoverable banner |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or are explicit human-verify checkpoints
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (created in 76-01/76-02 Task 0)
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter (every auto/tdd task carries an automated command; Wave 0 tests are authored in-task)

**Approval:** approved 2026-05-29
