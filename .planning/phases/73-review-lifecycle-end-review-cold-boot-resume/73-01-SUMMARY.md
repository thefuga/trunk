---
phase: 73-review-lifecycle-end-review-cold-boot-resume
plan: 01
subsystem: ui
tags: [svelte, tauri-ipc, review-session, lifecycle, tdd]

# Dependency graph
requires:
  - phase: 65-review-store
    provides: get_review_session_status / resume_review_session / end_review_session IPC + session-changed event
  - phase: 69-comment-management-ui
    provides: ReviewPanel.svelte (real panel, post-stub) with reload() chokepoint and WR-02/WR-03 listener invariants
provides:
  - Cold-boot resume wired in ReviewPanel.reload() (D-01/D-07)
  - sessionState local rune ($state<SessionState>) exposed for Plans 02/03
  - installReads test dispatcher extended with lifecycle IPC cases + statusAfterResume flip
  - sessionChangedHandler capture + fireSessionChanged() helper for cross-tab simulation
affects: [73-02-end-button, 73-03-empty-states]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Local rune mirrors backend lifecycle state (sessionState) — single source of truth for resume gating + future empty-state gating"
    - "Test dispatcher closure flag (resumed) models backend state transition for recursion-safety assertions"

key-files:
  created: []
  modified:
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts

key-decisions:
  - "Phase 73-01: cold-boot resume gated on sessionState === 'resume-available'; recursion-safe because the post-resume status is 'active' and the listener-triggered reload skips the resume branch"
  - "Phase 73-01: errorMessage() returns extracted .message only; the 'Failed to resume review: ' prefix is added via template literal at the call site (matching RESEARCH §Pattern 2 canonical shape)"
  - "Phase 73-01: installReads default status = 'active' so the 26 pre-existing ReviewPanel tests stay on the warm path without modification"

patterns-established:
  - "Lifecycle rune mirroring: $state<SessionState> mirrored from get_review_session_status drives both the resume branch (this plan) and future End-button visibility / empty-state gating (Plans 02/03)"
  - "Recursion-safety test pattern: statusAfterResume + fireSessionChanged + exact call-count assertion (not toHaveBeenCalled) proves resume_review_session fires exactly once across the initial reload + the listener-triggered second reload"

requirements-completed:
  - REQ-73-RESUME
  - REQ-73-NYQUIST
  - REQ-73-CHECK

# Metrics
duration: 7min
completed: 2026-05-27
---

# Phase 73 Plan 01: Cold-boot Resume Wiring Summary

**Cold-boot resume now fires resume_review_session exactly once when the panel opens on a repo with an on-disk session — closes Bug 3 from 72-VERIFICATION.md with five TDD-driven unit tests + zero backend changes.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-05-27T15:21:49Z
- **Completed:** 2026-05-27T15:28:56Z
- **Tasks:** 3 (1 infra + 1 TDD feature + 1 phase gate)
- **Files modified:** 2 (`ReviewPanel.svelte`, `ReviewPanel.test.ts`)

## Accomplishments

- Cold-boot resume branch wired into `ReviewPanel.reload()` (lines 244-258) — `sessionState === "resume-available"` triggers `resume_review_session` before the parallel list reads; rejection surfaces a `showToast("Failed to resume review: <msg>", "error")` via `errorMessage()` and falls through to the reads so the existing `no_session` arm renders the cold empty state.
- `sessionState` local `$state<SessionState>` rune introduced at `ReviewPanel.svelte:141`, assigned from `status.state` inside `reload()` — ready for Plan 73-02 (End-button visibility) and Plan 73-03 (empty-state gating) to consume with zero additional scaffolding.
- Test dispatcher `installReads` extended with `status`/`statusAfterResume`/`resumeRejection`/`endRejection` opts + three new switch cases (`get_review_session_status`, `resume_review_session`, `end_review_session`); a closure `resumed` flag flips the status read to `statusAfterResume` after a successful resume, modeling the backend transition `"resume-available" → "active"`.
- `@tauri-apps/api/event` mock upgraded to capture the listener callback into a module-scoped `sessionChangedHandler`; `fireSessionChanged(payload)` helper exposed for cross-tab simulation.
- 5 new tests in `describe("cold-boot resume", ...)` — 31/31 in the file pass; full `just check` exits 0.

## Task Commits

1. **Task 1: Extend installReads + capture listener** — `b7d2d52` (test)
2. **Task 2 (RED): Add failing tests** — `8e8b2c4` (test)
3. **Task 2 (GREEN): Wire cold-boot resume** — `c40ec28` (feat)
4. **Task 3: just check phase gate** — verification only, no commit

REFACTOR step examined and skipped: the resume branch is ~12 lines, intent-revealing, no duplication; `sessionState = status.state` co-located with `canonicalPath = status.canonical_path` (Tell-Don't-Ask — both derived from one source).

## Files Created/Modified

- `src/components/ReviewPanel.svelte` — Imported `SessionState`; added `sessionState` rune (line 141); assigned it in `reload()` (line 232); added cold-boot resume branch (lines 244-258).
- `src/components/ReviewPanel.test.ts` — Imported `SessionStatus`; extended `installReads` dispatcher with lifecycle IPC cases + `resumed` flag; upgraded `listen` mock to capture handler; added `fireSessionChanged` helper; added `describe("cold-boot resume", ...)` at file scope with 5 tests.

## Decisions Made

- **Toast prefix via template literal, not errorMessage fallback.** `errorMessage(e, "Failed to resume review")` would have returned `"Failed to resume review"` only when `e` was neither `Error` nor `TrunkError`. Tests 4 and 5 (which DO pass an Error / TrunkError) need the extracted `.message` PLUS the prefix. Implementation uses `` `Failed to resume review: ${errorMessage(e, "unknown error")}` `` to match the canonical pattern from RESEARCH §Pattern 2 — the prefix is the action that failed, the extracted message is the cause. This matches the planner's intent (acceptance criterion `grep '"Failed to resume review"'` matches the substring inside the template literal at line 256).
- **Top-level `describe("cold-boot resume", ...)`, not nested.** The plan said "peer to `describe("Copy")`" but `describe("Copy")` is itself nested inside `describe("ReviewPanel")`. The clarifying clause — "NOT nested inside `describe("ReviewPanel")`" — won; the new describe is at file scope, consistent with the testing-style preference that each top-level describe names the symbol or behavior under test.

## Deviations from Plan

None — plan executed exactly as written. The two minor clarifications above (toast format, describe placement) were ambiguities resolved by reading the cross-referenced RESEARCH section, not deviations from the plan's intent.

## Issues Encountered

- **Initial GREEN failed Tests 4 & 5.** First-pass GREEN implementation called `showToast(errorMessage(e, "Failed to resume review"), "error")` directly. `errorMessage` returned only the extracted message (`"boom"`, `"Session file is newer..."`), missing the `"Failed to resume review: "` prefix the tests asserted. Fixed by wrapping in a template literal per RESEARCH §Pattern 2; all 5 tests passed on the second attempt.

## Pre-existing Issues (Ownership Note)

Per `ownership.md` — surfaced but not fixed in this plan (out of `files_modified` scope):

- `src/components/diff/CommentComposer.svelte:43` — three `lint/style/noNonNullAssertion` warnings (Biome). Pre-existing; `just check` still exits 0 (warnings, not errors). Worth a follow-up `chore` commit replacing the non-null assertions with proper narrowing — root cause is likely that `target.parentElement` / property access on a `Maybe<HTMLElement>` was bypassed via `!`. Filed as deferred; tracked here so the next plan/quick-task can pick it up.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 73-02 (End-review affordance) can consume `sessionState` directly for the End-button visibility gate (`{#if sessionState !== "none"}`); no new scaffolding required.
- Plan 73-03 (empty-state gating) can branch on `sessionState === "none"` for the cold-vs-warm-empty distinction; the rune assignment is already in `reload()`.
- `fireSessionChanged` test helper is available file-scope for both plans (multi-tab End coordination test + listener filter tests).
- No backend changes required for Plans 02 or 03 either — RESEARCH confirms all primitives exist.

**Manual smoke test outstanding** (per `<verification>` in the plan): `pnpm tauri dev`, open a repo with an existing on-disk session, observe comments appear in the panel on first open without clicking Add Note. This is the Bug 3 closure proof; it is a manual step recorded in 73-VALIDATION.md and is not blocking for the plan-level merge gate.

## Self-Check

Verified via `git log --oneline | grep '73-01'` (3 commits present), `pnpm vitest run src/components/ReviewPanel.test.ts` (31/31 pass), `just check` (exit 0), `grep -n 'sessionState\|resume_review_session\|Failed to resume review' src/components/ReviewPanel.svelte` (all present at expected line numbers).

---
*Phase: 73-review-lifecycle-end-review-cold-boot-resume*
*Completed: 2026-05-27*
