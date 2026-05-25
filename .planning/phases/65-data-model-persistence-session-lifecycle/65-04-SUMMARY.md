---
phase: 65-data-model-persistence-session-lifecycle
plan: 04
subsystem: review-panel-stub
tags: [svelte5, runes, tauri-event, throwaway-stub, component-test, session-lifecycle]

# Dependency graph
requires:
  - "Plan 65-03: start/resume/end/get_review_session_status Tauri commands + session-changed + review-toggle events"
  - "Plan 65-01/65-03: SessionStatus + SessionState TS mirror in src/lib/types.ts"
  - "Existing safeInvoke + TrunkError (src/lib/invoke.ts), showToast (src/lib/toast.svelte.ts)"
provides:
  - "ReviewPanel.svelte (D-12 throwaway): 3-state lifecycle stub (no-session / resume-available / session-active)"
  - "App.svelte review-toggle listener that toggles the panel for the active repo"
  - "Hand-verifiable SESS-01/02/03 end-to-end before the real review panel (Phase 69)"
affects: [69-review-panel]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Svelte 5 runes component invoking lifecycle commands via safeInvoke + toast-on-error (OperationBanner idiom)"
    - "session-changed $effect listener with cleanup, payload matched against the loaded status canonical_path (DP-01, mirrors App.svelte repo-changed)"
    - "review-toggle listen<void> $effect in App.svelte (mirrors CommitGraph search-toggle) flipping a $state boolean"
    - "Component test captures the mocked listen handler by event name and fires it by hand to assert a status re-fetch"

key-files:
  created:
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts
  modified:
    - src/App.svelte

key-decisions:
  - "Renamed the derived state var to sessionState — let state = $derived(...) shadows the $state rune and breaks svelte-check."
  - "session-changed handler reloads when payload matches status.canonical_path, and also reloads when status is not yet loaded (bootstrap) so the first cross-tab event is never dropped."
  - "Panel renders inside the active tab's container, gated by tab.id === activeTabId, so the global toggle does not show a panel on every stacked tab at once."
  - "Stub renders text-only state bodies (No comments yet) — no comment list / editing (Phase 69 owns the real panel)."

patterns-established:
  - "Frontend session lifecycle button = safeInvoke(cmd, { path }) then await reloadStatus(); catch -> showToast((e as TrunkError).message, error)."
  - "Throwaway-stub component test asserts button presence per state + invoke args + a fired event re-fetch; does not over-specify layout."

requirements-completed: [SESS-01, SESS-02, SESS-03]

# Metrics
duration: ~10min
completed: 2026-05-25
---

# Phase 65 Plan 04: Throwaway Review Panel Stub Summary

**A minimal Svelte 5 `ReviewPanel.svelte` stub (D-12) renders the three session states — no-session (Start Code Review), resume-available (Resume + Discard), session-active (empty view + End Review) — driving the Plan 65-03 lifecycle commands through `safeInvoke` with toast-on-error, reloading on the `session-changed` live event for its repo's canonical path, and toggled by the View-menu `review-toggle` event wired in `App.svelte`; makes SESS-01/02/03 hand-verifiable end-to-end before the real panel lands in Phase 69.**

## Performance
- **Duration:** ~10 min
- **Started:** 2026-05-25T~09:29Z
- **Completed:** 2026-05-25
- **Tasks:** 2 (both auto)
- **Files created:** 2 (`ReviewPanel.svelte`, `ReviewPanel.test.ts`)
- **Files modified:** 1 (`App.svelte`)

## Accomplishments
- Built `ReviewPanel.svelte`: a `repoPath`-prop runes component holding `SessionStatus` in `$state`, deriving the three D-12 states, with Start / Resume / Discard / End buttons each calling `safeInvoke(cmd, { path })` then `reloadStatus()`, toasting `TrunkError.message` on failure (surfaces `session_exists` / `not_open` / `newer_version` recovery messages).
- Added the `session-changed` `$effect` listener with cleanup, mirroring App.svelte's `repo-changed` listener: it reloads when the event payload matches the loaded status's `canonical_path` (T-65-04-EVT mitigation — foreign payloads ignored), and during bootstrap (status not yet loaded) so the first cross-tab event is not dropped.
- Wired `App.svelte`: a `listen<void>("review-toggle")` `$effect` (mirroring CommitGraph's `search-toggle`) flips a `reviewPanelOpen` `$state` boolean; `<ReviewPanel repoPath={tab.repoPath} />` renders inside the active tab's container, gated by `tab.id === activeTabId`. No Rust files touched.
- Wrote `ReviewPanel.test.ts`: five behavioral tests covering all three states, the `start_review_session` invocation with `{ path }`, and a fired `session-changed` event triggering a `get_review_session_status` re-fetch (handler captured from the mocked `listen` by event name).

## Task Commits
1. **Task 1: ReviewPanel.svelte stub + App.svelte review-toggle wiring (auto)** — `f194a22` (feat)
2. **Task 2: ReviewPanel.test.ts 3-state + Start invoke + session-changed reload (auto)** — `3e16298` (test)

**Plan metadata:** (this docs commit)

## Files Created/Modified
- `src/components/ReviewPanel.svelte` (NEW) — `repoPath` prop; `status: SessionStatus | null` in `$state`; `sessionState` `$derived`; `reloadStatus()` + `runLifecycle(cmd)`; initial-load `$effect`; `session-changed` `$effect` listener with cleanup; three state bodies with theme-CSS-custom-property styling only (no inline hex).
- `src/components/ReviewPanel.test.ts` (NEW) — mocks `invoke` (stateful per state) and `@tauri-apps/api/event` `listen` (captures `session-changed` handlers); five tests.
- `src/App.svelte` — `ReviewPanel` import; `reviewPanelOpen` `$state`; `review-toggle` `$effect` listener; conditional render of `<ReviewPanel>` for the active tab.

## Decisions Made
- **`sessionState` not `state`:** `let state = $derived(...)` shadows the `$state` rune (svelte-check: "Block-scoped variable '$state' used before its declaration"). Renamed the derived to `sessionState`.
- **Bootstrap reload on session-changed:** the listener reloads when `status` is null (not yet loaded) OR the payload matches `status.canonical_path`. This keeps the canonical-path match (T-65-04-EVT) while never dropping the very first cross-tab event that arrives before the initial fetch resolves.
- **Active-tab gating:** the toggle is global but the panel is rendered inside each tab's stacked container, so it is gated by `tab.id === activeTabId` to avoid showing on every tab simultaneously.
- **Text-only state bodies:** the active state shows "No comments yet" — no comment list, editing, or real layout (Phase 69 deliverable). Smallest hand-verifiable stub per D-12.

## Deviations from Plan
None — plan executed exactly as written. No deviation rules triggered. (Renaming `state` to `sessionState` is a mechanical fix for a Svelte rune name collision, not a behavioral change.)

## Threat Model Compliance
- **T-65-04-XSS (accept):** toast renders `TrunkError.message` as interpolated text; Svelte escapes by default, no `{@html}`. No change needed — risk accepted per register.
- **T-65-04-EVT (mitigate):** the `session-changed` listener acts only when the payload matches this repo's `canonical_path` (foreign payloads return early), mirroring the `repo-changed` precedent. Verified by the fired-event test using the matching `/canonical/repo` payload.
- **T-65-04-SC (mitigate):** no new packages added — uses existing `@tauri-apps/api`, testing-library, vitest. No install task.

## Known Stubs
- `ReviewPanel.svelte` is itself the intentional D-12 throwaway stub (documented in a file header comment and the plan), replaced by the real review panel in Phase 69. The "No comments yet" body is a placeholder for the active state; this is by design — Phase 69 wires the real comment data. Not a blocking stub: the lifecycle (Start/Resume/Discard/End + live reload) is fully wired and hand-verifiable, which is the plan's goal.

## Issues Encountered
- First svelte-check failed because the derived `state` variable name collided with the `$state` rune. Resolved by renaming to `sessionState`; no logic change.

## User Setup Required
None.

## Next Phase Readiness
- SESS-01/02/03 are now hand-verifiable: View-menu "Start/End Code Review" toggles the panel; Start → empty session; force-quit + reopen → resume-available; Resume → same session; End → no session.
- Phase 69 replaces `ReviewPanel.svelte` with the real panel; it can reuse the lifecycle-command + `session-changed`-listener wiring established here.
- No blockers.

## Self-Check: PASSED
- Files verified present: `src/components/ReviewPanel.svelte`, `src/components/ReviewPanel.test.ts`, `src/App.svelte`, `65-04-SUMMARY.md`.
- Commits verified in git log: `f194a22` (feat), `3e16298` (test).
- `just check` green: rustfmt, biome, svelte-check (0 errors), clippy, cargo-test, 461 vitest (incl. 5 new ReviewPanel tests).
- Grep confirmed: 0 inline hex colors in ReviewPanel.svelte; `safeInvoke` for all four commands; `listen<string>("session-changed"` in ReviewPanel.svelte; `listen<void>("review-toggle"` in App.svelte; no Rust files modified.

---
*Phase: 65-data-model-persistence-session-lifecycle*
*Completed: 2026-05-25*
