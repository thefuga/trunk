---
phase: 65-data-model-persistence-session-lifecycle
plan: 03
subsystem: session-lifecycle
tags: [rust, tauri-command, managed-state, canonical-path, tauri-event, menu, three-state-merge]

# Dependency graph
requires:
  - "ReviewSession DTO (Serialize + Deserialize) from 65-01"
  - "review_store::{save_session, load_session, delete_session, session_exists} + LoadOutcome from 65-02"
  - "RepoState raw-String map + stash.rs thin-command/_inner/spawn_blocking pattern (existing)"
provides:
  - "ReviewSessionsState(Mutex<HashMap<PathBuf, ReviewSession>>) — canonical-path-keyed in-memory cache (D-11)"
  - "commands::review four lifecycle commands: start/resume/end/get_status + matching _inner functions"
  - "SessionState (kebab-case IPC enum) + SessionStatus struct, mirrored to src/lib/types.ts"
  - "session-changed Tauri event (canonical-path String payload) on every successful mutation (DP-01)"
  - "review-toggle View-menu trigger (D-12 throwaway) for the frontend stub (Plan 65-04)"
  - "close_repo/force_close_repo drop the in-memory session entry (file preserved for resume)"
affects: [65-04-frontend-stub, review-panel]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Testability wedge: _inner(data_dir: &Path, ...) takes plain args (no Tauri state) so disk behavior is provable with a tempfile::TempDir"
    - "Disk-first mutation ordering (D-10): _inner writes the file → command updates ReviewSessionsState → command emits session-changed"
    - "Three-state merge in the thin command only: _inner returns disk half (ResumeAvailable/None); the command promotes to Active after locking the in-memory map (the only place Active is born)"
    - "Canonical-path keying isolated to the session layer (D-11) — RepoState/CommitCache/RunningOp keep raw-String keys unchanged"
    - "kebab-case transient IPC enum (SessionState) distinct from the PascalCase persisted on-disk enums (Source/Side)"

key-files:
  created:
    - src-tauri/src/commands/review.rs
  modified:
    - src-tauri/src/state.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/commands/repo.rs
    - src-tauri/src/lib.rs
    - src-tauri/tests/test_review.rs
    - src/lib/types.ts

key-decisions:
  - "Canonical-path keying lives only in the session layer (D-11); RepoState/CommitCache keep raw-String keys."
  - "Three-state status merge happens in the thin command, never in _inner (_inner is disk-only and can never report Active)."
  - "start_review_session rejects with session_exists when a file already exists; client must Resume or End first (no silent overwrite, RESEARCH Open Question 2)."
  - "merge_status extracted as a tiny pure helper so the Active-vs-ResumeAvailable distinction is unit-testable without Tauri state."
  - "close-repo canonicalizes path best-effort; if the repo dir is gone there is nothing to remove (no panic)."

patterns-established:
  - "Lifecycle command branches on LoadOutcome: Loaded → insert+emit; None → noop; RecoveredCorrupt → fresh session saved+cached (D-15); RefusedNewer → newer_version TrunkError, no fresh session (D-16)."
  - "Every successful session mutation emits session-changed with the canonical path String so other tabs reload (DP-01); get_review_session_status is read-only and does NOT emit."

requirements-completed: [SESS-01, SESS-02, SESS-03]

# Metrics
duration: ~12min
completed: 2026-05-25
---

# Phase 65 Plan 03: Session Lifecycle Commands Summary

**Four registered Tauri commands (start/resume/end/get-status) over testable `_inner(data_dir: &Path, ...)` functions wire the review-session lifecycle: a canonical-path-keyed `ReviewSessionsState`, disk-first mutations that broadcast `session-changed`, a `get_review_session_status` that merges disk + in-memory presence into the three D-12 states (active / resume-available / none), a close-repo hook that drops only the in-memory entry, and the throwaway `review-toggle` View-menu trigger — all Rust wiring in one plan.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-05-25T~11:20Z
- **Completed:** 2026-05-25
- **Tasks:** 3 (1 auto + 1 TDD + 1 auto)
- **Files created:** 1 (`commands/review.rs`)
- **Files modified:** 6 (`state.rs`, `commands/mod.rs`, `commands/repo.rs`, `lib.rs`, `tests/test_review.rs`, `src/lib/types.ts`)

## Accomplishments
- Added `ReviewSessionsState(Mutex<HashMap<PathBuf, ReviewSession>>)` — the one place keying diverges to canonical `PathBuf` (D-11); the existing raw-String maps are untouched.
- Built four `_inner` functions (`start`/`resume`/`end`/`get_status`) on the testability wedge: each takes `data_dir: &Path` plus a plain `state_map`, with no Tauri state, so the whole lifecycle (including symlink resume, crit #3) is proven by `tempfile::TempDir`-injecting integration tests.
- Implemented the four thin `#[tauri::command]`s mirroring `stash.rs`: clone state → resolve `app_data_dir` → `spawn_blocking(_inner)` → JSON-stringify errors → disk-first in-memory update → `app.emit("session-changed", canonical_string)`.
- Locked the three-state merge in `get_review_session_status`: `_inner` returns the disk half (`ResumeAvailable`/`None`), and only the command — after locking `ReviewSessionsState` and checking the canonical key — can promote to `Active`.
- Wired the close-repo hook (`close_repo` + `force_close_repo`) to drop only the in-memory entry (file preserved so resume works on reopen); neither calls `delete_session`.
- Completed all `lib.rs` wiring: `.manage(ReviewSessionsState)`, four `invoke_handler` entries, and the `review-toggle` View-menu item + `on_menu_event` emit (D-12), so Plan 65-04 (frontend) never touches `lib.rs`.

## Task Commits

Each task was committed atomically:

1. **Task 1: ReviewSessionsState + lib.rs registration (auto)** — `1f29a8b` (feat)
2. **Task 2 RED: failing lifecycle _inner tests** — `fa32170` (test)
3. **Task 2 GREEN: lifecycle _inner functions + SessionStatus + TS mirror** — `26c73c1` (feat)
4. **Task 3: thin commands + 3-state merge + close hook + lib.rs wiring** — `33152af` (feat)

**Plan metadata:** (this docs commit)

_TDD gate sequence (Task 2): test (RED) → feat (GREEN). No REFACTOR commit — the `_inner` functions read cleanly from the start._

## Files Created/Modified
- `src-tauri/src/commands/review.rs` (NEW) — `SessionState` (kebab-case) + `SessionStatus`; `canonical_repo_path` helper (not_open + canonicalize); four `_inner` functions; `merge_status` helper + 3 inline unit tests; four thin `#[tauri::command]`s with disk-first ordering and `session-changed` emits; `resume` branches on all four `LoadOutcome` variants.
- `src-tauri/src/state.rs` — added `ReviewSessionsState` keyed by canonical `PathBuf`.
- `src-tauri/src/commands/mod.rs` — registered `pub mod review;`.
- `src-tauri/src/commands/repo.rs` — `drop_in_memory_session` helper; `ReviewSessionsState` param added to `close_repo` + `force_close_repo`, removing only the in-memory entry.
- `src-tauri/src/lib.rs` — `.manage(ReviewSessionsState)`, `use state::ReviewSessionsState`, four `commands::review::*` in `invoke_handler`, `review-toggle` `MenuItemBuilder` in the View submenu, and the `on_menu_event` branch emitting `review-toggle`.
- `src-tauri/tests/test_review.rs` — seven lifecycle tests: `start_creates_session`, `start_rejects_closed_repo`, `start_rejects_when_session_exists`, `resume_after_restart`, `symlink_resumes_same_session` (`#[cfg(unix)]`), `end_clears_session`, `status_inner_never_reports_active`.
- `src/lib/types.ts` — mirrored `SessionState` ("active" | "resume-available" | "none") and `SessionStatus` for Plan 65-04.

## Decisions Made
- **Canonical-path keying isolated to the session layer (D-11):** `ReviewSessionsState` is `PathBuf`-keyed; the raw-String `RepoState`/`CommitCache`/`RunningOp` maps are not retrofitted, exactly as the plan and RESEARCH forbid.
- **Three-state merge in the command, never in `_inner`:** the testability wedge means `_inner` has no Tauri state, so it returns the disk half (`ResumeAvailable`/`None`) and the thin `get_review_session_status` promotes to `Active` after a `contains_key` against the canonical key. This is the only place `Active` is produced.
- **`session_exists` rejection on start:** starting when a file already exists returns a `session_exists` `TrunkError` rather than silently overwriting — locking RESEARCH Open Question 2; the client must Resume or End-and-clear first.
- **`merge_status` extracted as a pure helper:** keeps the Active-vs-ResumeAvailable distinction unit-testable (3 inline `#[cfg(test)]` tests) without standing up Tauri state, satisfying the acceptance criterion that at least one test distinguishes the two.
- **Best-effort canonicalize on close:** `drop_in_memory_session` only removes when `std::fs::canonicalize(path)` succeeds; a deleted repo dir simply has nothing to remove, so close never panics.

## Deviations from Plan
None — plan executed exactly as written. No deviation rules triggered. (The `resolve_data_dir` helper and `canonical_repo_path` helper are small internal DRY extractions of the patterns the plan prescribed, not behavioral changes.)

## Threat Model Compliance
All five STRIDE register entries are mitigated and verified:
- **T-65-03-AC (access on closed repo):** every `_inner` calls `canonical_repo_path`, which returns `not_open` if the path is absent from `RepoState`. Proven by `start_rejects_closed_repo`.
- **T-65-03-DIV (in-memory vs disk divergence):** disk-first ordering — `_inner` writes/deletes the file, then the command updates `ReviewSessionsState`, then emits; a failed write returns `Err` before any in-memory change. Status reads merge `file_exists` with `contains_key`. Proven by `start_creates_session` + `merge_status` tests.
- **T-65-03-DL (close deletes the file):** `close_repo`/`force_close_repo` call `drop_in_memory_session` only — `grep -c delete_session src-tauri/src/commands/repo.rs` == 0. The file survives for resume (`resume_after_restart`).
- **T-65-03-DG (newer-schema downgrade):** the `resume` command branches `LoadOutcome::RefusedNewer` → returns a `newer_version` `TrunkError` and does NOT create a fresh session (D-16). The byte-untouched guarantee is enforced in 65-02's `load_session` (`newer_version_refused`).
- **T-65-03-SC (supply chain):** no new packages — `serde`, `serde_json`, `std`, `tauri`, `tempfile` already present. No install task.

## Known Stubs
None in the Rust layer. The `review-toggle` View-menu item is an intentional D-12 throwaway trigger (not a stub) — it emits an event the Plan 65-04 frontend stub listens for, and both are replaced by the real review panel in Phase 69. The frontend `ReviewPanel.svelte` stub itself is Plan 65-04's deliverable, not this plan's.

## Issues Encountered
- `just check` runs `cargo fmt --check` (CI mode), which failed on first run because rustfmt wanted to reflow a few lines. Resolved by running `cargo fmt` + `biome format --write`; no logic changes. Final `just check` is fully green.

## User Setup Required
None — no external service configuration required.

## Next Phase Readiness
- Plan 65-04 (frontend stub) can now invoke `start_review_session` / `resume_review_session` / `end_review_session` / `get_review_session_status`, type the response with `SessionStatus`/`SessionState` from `src/lib/types.ts`, listen for `session-changed` (canonical-path String) and `review-toggle` (void), and render the three D-12 states. All Rust wiring is complete; 65-04 must NOT touch `lib.rs`.
- No blockers.

## TDD Gate Compliance
- RED gate: `fa32170` (test) — seven lifecycle tests fail to compile (`could not find review in commands`).
- GREEN gate: `26c73c1` (feat) — all 12 `test_review` tests pass.
- Sequence verified: test commit precedes feat commit. (Task 3's command-layer `merge_status` unit tests were added with their implementation in `33152af` — they test a pure helper introduced in the same commit, which is consistent with the obvious-implementation tactic for a 3-arm match.)

## Self-Check: PASSED
- Files verified present: `src-tauri/src/commands/review.rs`, `src-tauri/src/state.rs`, `src-tauri/src/commands/repo.rs`, `src-tauri/src/lib.rs`, `src-tauri/tests/test_review.rs`, `src/lib/types.ts`, `65-03-SUMMARY.md`.
- Commits verified in git log: `1f29a8b`, `fa32170`, `26c73c1`, `33152af`.
- `just check` green: rustfmt, biome, svelte-check, clippy, cargo-test (incl. 12 test_review + 3 merge_status), 456 vitest.
- Grep confirmed: `app.emit("session-changed"` x3, `contains_key` in `get_review_session_status`, `"newer_version"` in resume, 0 `delete_session` in repo.rs, `review-toggle` menu item + emit in lib.rs, 2 `fs::canonicalize` in review.rs.

---
*Phase: 65-data-model-persistence-session-lifecycle*
*Completed: 2026-05-25*
