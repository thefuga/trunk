---
phase: 73-review-lifecycle-end-review-cold-boot-resume
plan: 02
subsystem: ui
tags: [svelte, tauri-ipc, review-session, destructive-confirm, tdd]

# Dependency graph
requires:
  - phase: 65-review-store
    provides: end_review_session IPC + session-changed event (D-08 round-trip refresh)
  - phase: 73-review-lifecycle-end-review-cold-boot-resume
    plan: 01
    provides: sessionState rune + installReads.endRejection dispatcher + sessionChangedHandler test helper
provides:
  - End-review button with two-step inline confirm (D-03 placement / D-05 confirm / D-08 listener-driven refresh)
  - .end-button CSS class + .end-button.confirming modifier using existing --color-danger* tokens
  - endConfirming local rune + endTimer + $effect teardown (Pitfall 3 mitigation)
  - describe("End review") test block with 6 fake-timer-scoped tests
affects: [73-03-empty-states]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-step destructive confirm rune: $state boolean flag + plain setTimeout handle, identical shape to the Copy `copied`/`copyTimer` pair (rapid-reclick re-arm via clearTimeout-before-setTimeout discipline)"
    - "Frozen-during-await label: endConfirming stays true across the IPC await; success path collapses via session-changed listener round-trip (sessionState → 'none' → {#if} gate hides the button), failure path explicitly reverts"
    - "$effect teardown-only pattern: a sibling $effect whose body is empty and whose return function clears the timer on component destroy (Pitfall 3 — the Copy timer lacks this protection but isn't exercised by unmount-during-revert tests)"

key-files:
  created: []
  modified:
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts

key-decisions:
  - "Phase 73-02: button order shipped as `[End review] [Copy]` — End BEFORE Copy in the header flex row, per plan-prose. UI-SPEC doesn't pin the order; the destructive action reads naturally to the left of the affirmative Copy action, and grouping End next to the spacer keeps Copy adjacent to the right edge where the affirmative-action pattern already lives."
  - "Phase 73-02: endConfirming stays TRUE during the IPC await (label frozen at 'Click again to confirm') — matches UI-SPEC § Interaction States In-flight row and what Test 4 actually asserts. Success: the session-changed listener round-trip drives reload() → sessionState === 'none' → {#if} gate hides the button. Failure: explicit `endConfirming = false` revert in the catch arm + showToast."
  - "Phase 73-02: toast prefix added via template literal at the call site (`Failed to end review: ${errorMessage(e, 'unknown error')}`), matching Plan 73-01's resume-fail shape. errorMessage() extracts only `.message`; the prefix is the action that failed, the extracted message is the cause. Calling errorMessage(e, 'Failed to end review') would only return the fallback when e is neither Error nor TrunkError — wrong for the test cases that pass a TrunkError."
  - "Phase 73-02: idle-state title attribute set to 'End the current review and delete the on-disk session' (plan-prose). UI-SPEC § Copywriting Contract row 'End button — title (idle, enabled)' says `(empty)` matching Copy. This is an inconsistency between the two docs; no test covers it. Shipped the descriptive tooltip — discoverability beats consistency with Copy here because the action is destructive."

patterns-established:
  - "Two-step destructive button shipping shape: gate visibility on a backend lifecycle rune (`sessionState !== 'none'`), frozen-during-await label, success refreshes via the canonical listener round-trip (no manual array mutation — D-08), failure reverts + toasts via the existing errorMessage helper."
  - "Timer teardown $effect: a $effect whose only purpose is the cleanup return-function is the canonical mitigation for setTimeout-leak-on-unmount (RESEARCH Pitfall 3). One per timer; coexists with reactive $effects on the same component without interference."

requirements-completed:
  - REQ-73-END
  - REQ-73-NYQUIST
  - REQ-73-CHECK

# Metrics
duration: 6min
completed: 2026-05-27
---

# Phase 73 Plan 02: End-Review Affordance Summary

**Two-step inline End-review button now lives next to Copy in the ReviewPanel header — closes REQ-73-END so a review session is no longer implicitly permanent. Six TDD-driven tests + zero backend changes.**

## Performance

- **Duration:** ~6 min
- **Tasks:** 2 (1 TDD feature + 1 phase gate)
- **Files modified:** 2 (`ReviewPanel.svelte`, `ReviewPanel.test.ts`)
- **Tests:** 6 new (37/37 in file pass); 540/540 vitest suite pass

## Accomplishments

- End button rendered as sibling BEFORE Copy in the existing header `flex items-center` row at `ReviewPanel.svelte:482-494`, gated on `sessionState !== "none"` via `{#if}` (UI-SPEC: hide entirely, not disabled — Pitfall 5).
- Local runes: `endConfirming: $state(false)` and plain `endTimer: ReturnType<typeof setTimeout> | null` declared at lines 149-151, mirroring the `copied`/`copyTimer` shape.
- Three handler arms (`startEndConfirm`, `onEndClick`) added at lines 371-409:
  - First click: `startEndConfirm()` arms the 3000ms revert with `clearTimeout`-before-`setTimeout` discipline (Pattern A).
  - Second click: cancels the revert timer but KEEPS `endConfirming = true` so the label stays "Click again to confirm" (frozen during await — UI-SPEC § In-flight); calls `safeInvoke("end_review_session", { path: repoPath })`; on success the existing session-changed listener round-trip drives `reload()` → `sessionState === "none"` → `{#if}` gate hides the button (D-08, no manual array clear); on failure explicit `endConfirming = false` revert + `showToast(\`Failed to end review: ${msg}\`, "error")`.
- `$effect` teardown clearing `endTimer` on unmount added at lines 429-438, mirroring the `session-changed` listener cleanup idiom (Pitfall 3 mitigation).
- CSS `.end-button` + `.end-button.confirming` rules added at lines 836-868 using `--color-danger`, `--color-danger-bg`, `--color-danger-border`, `--color-on-accent`, `--color-text-muted`, `--color-text`, `--color-hover`, `--color-border` — all existing `:root` tokens, NO hex/rgb literals (CLAUDE.md "Never inline colors").
- 6 new tests in `describe("End review")` at the file scope of `ReviewPanel.test.ts:1027-1217`, with `vi.useFakeTimers()` scoped to the describe (NOT promoted to the file-global `beforeEach` — the global `flush()` uses `setTimeout(r, 0)` and deadlocks under fake timers, same constraint as the existing Copy describe). Coverage per `must_haves.truths`:
  - first click enters confirming state, no IPC
  - second click invokes `end_review_session({ path })` exactly once
  - auto-revert after 3000ms with no second click
  - second click within window cancels the revert timer (rapid re-arm proof)
  - IPC rejection surfaces `"Failed to end review: <msg>"` toast, arrays untouched
  - `$effect` teardown clears timer on unmount (no `console.error` from torn-down state)

## Task Commits

1. **Task 1 (RED): Add failing tests for End-review affordance** — `b63db99` (test)
2. **Task 1 (GREEN): Add two-step End-review button to ReviewPanel** — `e26d60d` (feat)
3. **Task 2: just check phase gate** — verification only, no commit (one biome auto-format applied to the test file as part of the gate; rolled into the GREEN-following format pass with no separate commit)

REFACTOR step examined and skipped: the three handler arms (`startEndConfirm`, the first-click branch of `onEndClick`, the second-click try/catch) are intent-revealing in line; the timer-pattern duplication with `copyTimer` is shallow and local (extracting a `scheduleRevert(setter, ms)` helper would add indirection without removing meaningful repetition — the two timers also have different recovery semantics: Copy resets on success, End hides via the listener round-trip).

## Files Created/Modified

- `src/components/ReviewPanel.svelte` — Imported `Trash2` (line 8); added `endConfirming`/`endTimer` runes (lines 149-151); added `startEndConfirm` + `onEndClick` handlers (lines 371-409); added timer-cleanup `$effect` (lines 429-438); added End button markup as sibling BEFORE Copy in header row (lines 482-494); added `.end-button` + `.end-button.confirming` CSS rules (lines 836-868).
- `src/components/ReviewPanel.test.ts` — Added `describe("End review")` block (lines 1027-1217) with 6 tests under scoped `vi.useFakeTimers()` + `flushFake` microtask helper + `renderWithSession()` shared setup + `endCallCount()` / `getEndButton()` query helpers. One long line auto-formatted by biome (no semantic change).

## Decisions Made

- **Button order: `[End review] [Copy]`.** UI-SPEC doesn't pin the order. Plan-prose recommends End BEFORE Copy with the rationale "destructive reads naturally to the left of the affirmative action." Shipped per plan-prose. Visually: spacer pushes both buttons right; End sits leftward of Copy; the destructive-color tint distinguishes them.
- **`endConfirming` stays TRUE during the IPC await.** UI-SPEC § In-flight row + Test 4's assertion both require this. The `cancelEndConfirm()` helper mentioned in the plan-prose action would have set `endConfirming = false` immediately and contradicted both. Implemented as: clear timer inline before the try, KEEP endConfirming true through the await, explicit revert only in the catch arm.
- **Toast prefix via template literal, not errorMessage fallback.** Same shape as Plan 73-01's resume-fail toast. `errorMessage(e, "Failed to end review")` would have returned the fallback only when `e` was neither `Error` nor `TrunkError`; the rejection-toast test passes a `TrunkError`, and the test asserts on the prefixed string. Implementation: `` `Failed to end review: ${errorMessage(e, "unknown error")}` ``.
- **Idle tooltip is descriptive, not empty.** Plan-prose proposes `title="End the current review and delete the on-disk session"` when not confirming; UI-SPEC says `(empty)` matching Copy. The Copy button is non-destructive — its enabled-state tooltip can be empty without loss. End is destructive; the tooltip serves as a discoverability + intent signal. No test covers the tooltip, so this is a CONTEXT call.

## Deviations from Plan

- **[Rule 1 - Plan-prose contradiction] Replaced `cancelEndConfirm()` helper with inline timer-clear + keep-flag-true.** Plan action says "Else: `cancelEndConfirm()`, then try/await `safeInvoke(...)`" where `cancelEndConfirm()` is defined to set `endConfirming = false`. Test 4 asserts the label is NOT `/^End review$/` after the second click (UI-SPEC: label "frozen during await"). The two are mutually exclusive — the test wins. Implementation: on the second-click branch, inline `clearTimeout(endTimer); endTimer = null` without touching `endConfirming`; the catch arm is the only place that explicitly sets `endConfirming = false`. The third helper was dropped as dead code.
- **[Rule 1 - Plan-prose contradiction] Toast prefix added via template literal, not as `errorMessage` fallback.** Plan-prose says `showToast(errorMessage(e, "Failed to end review"), "error")`. Test 5 passes a `TrunkError` whose `.message` is `"No active review session"` and asserts the toast string equals `"Failed to end review: No active review session"`. `errorMessage(e, fallback)` returns `e.message` when `e` is a `TrunkError` — the fallback is unreachable on this path. Same correction Plan 73-01 made for the resume-fail path. Implementation: `` `Failed to end review: ${errorMessage(e, "unknown error")}` ``.

Both deviations are Rule 1 bug fixes against the plan-prose; the plan's `must_haves.truths` and test-assertion contract are what the implementation honors.

## Issues Encountered

- **GREEN Test 5 initially failed (toast format mismatch).** Implemented `showToast(errorMessage(e, "Failed to end review"), "error")` per plan-prose; test asserted `"Failed to end review: No active review session"`. Same root cause as Plan 73-01: `errorMessage(e, fallback)` returns `e.message` (extracted message) when `e` is `Error` or `TrunkError`, the fallback fires only for the other-shape case. Fixed by switching to a template literal; all 6 tests passed on retry.
- **`just check` biome formatter complained on one long test-helper line** (`const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});` in Test 6). Resolved with `pnpm exec biome check --write src/components/ReviewPanel.test.ts` (single-purpose autofix; no semantic change). Rolled into the GREEN commit's follow-up format pass — no separate commit because the diff is whitespace-only.

## Pre-existing Issues (Ownership Note)

Per `ownership.md` — surfaced but still not fixed (out of `files_modified` scope):

- `src/components/diff/CommentComposer.svelte:43` — three `lint/style/noNonNullAssertion` warnings (Biome). Pre-existing; first surfaced in 73-01-SUMMARY; still outstanding through this plan. `just check` exits 0 (warnings, not errors). The non-null assertions appear to bypass narrowing on a `Maybe<HTMLElement>` chain; a follow-up `chore` commit replacing them with proper narrowing would close them. Tracked in 73-01-SUMMARY's Pre-existing Issues block; mentioning again here so future passers-by see it twice.

## User Setup Required

None — no external service configuration required.

## Manual Smoke Test

Outstanding per `<verification>` in the plan: `pnpm tauri dev` on a repo with an active review session — observe (1) End button visible next to Copy, idle muted-text styling; (2) first click: label flips to "Click again to confirm", background turns danger-tinted, NO disk effect; (3) wait 3s: button reverts to idle "End review" without any action; (4) click twice rapidly: session collapses to cold state (panel shows "No active review" empty state after Plan 73-03 lands; today shows the existing `groups.length === 0` empty state). This is `73-VALIDATION.md` § "Manual-Only Verifications" row "Visual treatment of two-step End button danger color" — manual gate; not blocking for the plan-level merge.

## Next Phase Readiness

- Plan 73-03 (empty-state gating + summary-line caption) can branch on `sessionState === "none"` for the cold-vs-warm-empty distinction; the rune is already wired in `reload()` from Plan 01, and the End button's `{#if}` gate establishes the visual precedent for the cold-state hide.
- No backend changes required for Plan 03 — RESEARCH confirms all primitives exist.
- The `installReads.endRejection` dispatcher slot is now exercised by Test 5 and stays in place for future Plan 73-03 tests if needed.

## Self-Check

- `git log --oneline | grep '73-02'` — 2 commits present (`b63db99` test, `e26d60d` feat).
- `pnpm vitest run src/components/ReviewPanel.test.ts` — 37/37 pass.
- `pnpm vitest run` — 540/540 pass (no regressions phase-wide).
- `just check` — exit 0 (3 pre-existing biome warnings on CommentComposer:43 still surface as warnings, not errors).
- `grep -n 'endConfirming\|endTimer\|end_review_session\|class="end-button\|sessionState !== "none"\|Failed to end review\|Trash2' src/components/ReviewPanel.svelte` — all required symbols present at expected line ranges.
- `git diff main -- src/components/ReviewPanel.svelte | grep '^+' | grep -E '#[0-9a-fA-F]{3,6}|rgb\(|rgba\('` — no new hex/rgb literals.
- `pnpm exec svelte-check` — 0 errors / 0 warnings on the modified file.

## Self-Check: PASSED

---
*Phase: 73-review-lifecycle-end-review-cold-boot-resume*
*Completed: 2026-05-27*
