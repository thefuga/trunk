---
phase: 66-commit-selection
plan: 04
subsystem: ui
tags: [svelte5, tauri-menu, commit-graph, code-review, context-menu]

# Dependency graph
requires:
  - phase: 66-02
    provides: backend commands seed_review_range / add_review_commit / remove_review_commit / list_session_commits
  - phase: 66-03
    provides: CommitRow inSession / isPendingBase props (the D-04 row marker + transient base highlight)
  - phase: 65
    provides: get_review_session_status (session-active source) + session-changed event
provides:
  - "CommitGraph two-right-click range gesture (D-01) seeding seed_review_range"
  - "CommitGraph single Add/Remove context-menu toggle (D-06)"
  - "session-active gating sourced from get_review_session_status (not panel-open, A1)"
  - "event-driven sessionOids membership Set + transient pendingBase, wired to CommitRow marker props"
affects: [phase-67-diff-source, phase-69-review-panel]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Event-driven membership Set in CommitGraph (reassign new Set(...) on session-changed; Pitfall 5)"
    - "Conditional context-menu item array (reviewItems) spread into Menu.new only when sessionActive"
    - "Two-step transient gesture via $state pendingBase cleared in finally + cancel item"

key-files:
  created:
    - .planning/phases/66-commit-selection/66-04-SUMMARY.md
  modified:
    - src/components/CommitGraph.svelte

key-decisions:
  - "Store full SessionStatus (not just a boolean) so the session-changed listener filters by canonical_path; sessionActive is a $derived off status.state === 'active'."
  - "Clear pendingBase in the seed_review_range finally block (success AND error) so an invalid range leaves the set unchanged and the gesture always completes."
  - "Added an explicit 'Clear review base' cancel item so the user can abandon a half-started range gesture."

patterns-established:
  - "Pattern: review context-menu items live in a conditional array gated on session-active, mirroring mergeRebaseItems; NO is_merge gate (D-08)."
  - "Pattern: every review safeInvoke is wrapped in explicit try/catch -> showToast (safeInvoke rethrows TrunkError; bare .catch would swallow errors)."

requirements-completed: [SEL-01, SEL-02, SEL-03]

# Metrics
duration: 20min
completed: 2026-05-25
---

# Phase 66 Plan 04: Graph-Side Commit Selection Summary

**Two-right-click range seeding (D-01) and a single Add/Remove context-menu toggle (D-06) in CommitGraph, gated on the active review session (not the panel-open flag), with an event-driven membership Set driving the in-row marker.**

## Performance

- **Duration:** ~20 min
- **Completed:** 2026-05-25
- **Tasks:** 2 auto tasks executed + committed; 1 human-verify checkpoint (automated portion run, human portion recorded below)
- **Files modified:** 1 (`src/components/CommitGraph.svelte`)

## Accomplishments
- D-01 two-right-click range gesture: "Set as review base" sets a transient `pendingBase` (shown via the CommitRow `isPendingBase` marker from Plan 03); a second right-click offers "Add range to review" which calls `seed_review_range({path, baseOid, tipOid})` and clears the base. No modal.
- D-06 single Add/Remove toggle: one MenuItem whose label/action flips on `sessionOids.has(commit.oid)`, calling `add_review_commit` / `remove_review_commit`.
- "Clear review base" cancel affordance so a half-started range gesture can be abandoned.
- D-08 honored: NO `enabled: !commit.is_merge` gate on any review item — merges are selectable (contrast the pre-existing Cherry-pick/Revert items which keep that gate).
- session-active gating sourced from `get_review_session_status` (state === "active"), NOT `reviewPanelOpen` (A1: that flag is panel visibility only).
- Event-driven `sessionOids` Set reloaded on mount and on every `session-changed` for this repo (filtered by `canonical_path`), reassigned as `new Set(...)` so Svelte 5 reactivity fires (Pitfall 5).
- `inSession` / `isPendingBase` passed to CommitRow at the existing call site (CommitRow itself untouched — props owned by Plan 03).

## Task Commits

1. **Task 1: sessionOids/sessionActive state + pendingBase + CommitRow prop wiring** - `4e468ae` (feat)
2. **Task 2: Range gesture (D-01) + Add/Remove toggle (D-06) in the context menu** - `6ca3d38` (feat)
3. **Task 3: Human-verify checkpoint** - automated portion (`just check`) run green; human gesture-test steps recorded below (auto-approved under --auto)

## Files Created/Modified
- `src/components/CommitGraph.svelte` - Added `sessionStatus`/`sessionActive` (derived)/`sessionOids`/`pendingBase` state, `reloadSession()`, a mount effect, a `session-changed` listener (canonical-path filtered), a pendingBase-clear effect, the `reviewItems` context-menu block (toggle + range gesture + cancel), and the two CommitRow marker props.

## Decisions Made
- Stored the full `SessionStatus` rather than a bare boolean so the `session-changed` listener can filter by `canonical_path` exactly like ReviewPanel; `sessionActive` is a `$derived` off `status.state === "active"`.
- Cleared `pendingBase` in the `seed_review_range` `finally` (both success and error) — the gesture is done either way, and an invalid range (bad_range / unrelated_history from Plan 01) leaves the set unchanged with a toast.
- Added an explicit "Clear review base" item to let the user cancel a started range gesture.

## Deviations from Plan

None - plan executed exactly as written. (All three deviation-rule categories: no bugs, missing-critical, or blocking issues encountered.)

## Issues Encountered
- `grep -c "inSession={\|isPendingBase={"` returned 1 instead of 2 because both props sit on the same CommitRow line; `grep -c` counts matching lines. Verified with `grep -o ... | wc -l` (2 occurrences) — acceptance intent met. No code change needed.

## TDD Gate Compliance
N/A — this plan is `type: execute` (UI wiring of native Tauri Menu gestures that vitest cannot drive). The phase's testable backend logic was covered in Plans 01/02; this plan's verification is the human-verify checkpoint per 66-VALIDATION.md §Manual-Only Verifications.

## Verification
- `npx svelte-check` — 0 errors, 0 warnings, 0 files with problems (3961 files).
- `just check` — full suite GREEN (fmt, biome ci, svelte-check, clippy `-D warnings`, cargo-test, vitest 469 passed / 44 files). Exit 0.

## Human Verification Needed

The native Tauri `Menu.popup()` gestures and the transient pending-base highlight cannot be driven by vitest (66-VALIDATION.md §Manual-Only Verifications). Under `--auto` the checkpoint was auto-approved; a human must still walk these steps via `just dev` to confirm the UX. Surface these as HUMAN-UAT items:

1. `just dev`, open a repo with a linear stretch AND at least one merge commit, and start a code review session (View menu → Start Code Review; the panel opens).
2. Right-click a commit → "Set as review base". Confirm the base row shows the transient pending-base highlight (bottom inset, `--color-review-pending-base`).
3. Right-click a LATER (descendant) commit → "Add range to review". Confirm: the panel list now shows the inclusive [base..tip] commits (both endpoints included) in graph order with no duplicates, and the pending-base highlight clears.
4. Right-click a commit NOT in the session → confirm the toggle reads "Add to review"; click it; confirm the row gets the in-session marker (left inset, `--color-review-row`) and it appears in the panel list.
5. Right-click a commit ALREADY in the session → confirm the toggle reads "Remove from review"; click it; confirm the marker and the panel row both disappear.
6. Right-click a MERGE commit → confirm "Add to review" is ENABLED (not greyed), add it, confirm it lands in the set (D-08). Also confirm a merge can be a range base/tip.
7. Seed a range where base is NOT an ancestor of tip (pick two unrelated/sibling commits) → confirm a toast appears (e.g. "Base is not an ancestor of tip" / "share no history") and the session set is unchanged; the pending base clears either way.
8. With a base set, right-click any commit → "Clear review base" → confirm the pending-base highlight clears and no range is seeded.
9. In the panel, click a row's × → confirm the commit is removed from both the list and the graph marker.
10. (Optional) Open the same repo in a second tab and confirm add/remove in one reflects in the other (session-changed sync).

Resume-signal expectation: human types "approved" or describes any mismatch (wrong range, merge greyed out, missing marker, toast not shown, lost write on rapid clicks).

## Next Phase Readiness
- SEL-01/02/03 are now actionable from the graph; SEL-04 marker is fed by the live `sessionOids` Set.
- Diff-source-on-merge restriction is deliberately deferred to Phase 67 (D-08).
- No blockers. The human-verify checkpoint is the only outstanding confirmation; under --auto it was auto-approved and the gesture steps above are queued for the phase verifier as HUMAN-UAT.

---
*Phase: 66-commit-selection*
*Completed: 2026-05-25*
