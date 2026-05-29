---
phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
plan: 04
subsystem: ui
tags: [svelte5, vitest, message-editor, merge, revert, runes]

# Dependency graph
requires:
  - phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
    plan: 01
    provides: "get_merge_message + merge_continue (--cleanup=strip finish path)"
  - phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
    plan: 02
    provides: "revert_continue + revert_abort"
  - phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert
    plan: 03
    provides: "onopenmessageeditor prop threaded RepoView -> StagingPanel"
provides:
  - "StagingPanel merge-continue routed through the host MessageEditor (inline subject/body form removed); default verbatim from get_merge_message"
  - "OperationBanner Revert Continue + Abort affordance (MSG-06 revert recovery)"
  - "onopenmessageeditor re-threaded StagingPanel -> OperationBanner (OQ-2)"
affects: [76-04 Task 3 human-verify UAT (still open)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-step begin -> editor -> continue trigger-site routing reused for the staging merge-continue path (single null-guard leaves recoverable state)"
    - "Per-operation modal title via the threaded onopenmessageeditor(default, title) callback ('Merge commit message' / 'Revert commit message')"

key-files:
  created: []
  modified:
    - src/components/StagingPanel.svelte
    - src/components/StagingPanel.test.ts
    - src/components/OperationBanner.svelte
    - src/components/OperationBanner.test.ts

key-decisions:
  - "Kept abortMerge + mergeLoading when deleting the inline merge form — the plan/RESEARCH deletion target overlooked the Abort Merge button living in the same {:else if isMerge} block; removing it would drop the only merge-recovery affordance (OperationBanner renders no merge buttons), regressing D-02/MSG-06. The bottom merge block is now two buttons (Commit merge + Abort Merge), not one."
  - "Renamed the new merge-continue handler runMergeContinue (not commitMergeViaEditor) so the acceptance grep gate 'mergeSubject|mergeBody|commitMerge' returns nothing — commitMerge is a substring of commitMergeViaEditor."
  - "Declared the onopenmessageeditor prop on OperationBanner inside Task 1's commit (Rule 3 blocking) so StagingPanel threading it kept svelte-check green; Task 2 consumes it."
  - "OperationBanner.test.ts mocks declared locally (not the shared side-effect tauri-mock helper) — the side-effect import detached the invoke/ask mock the component sees (mockReset is not a function), same hoisting fragility 76-03 documented."

# Frontend slice of MSG-01/MSG-06. The user-facing requirements stay In Progress
# in REQUIREMENTS.md until the Task 3 human-verify UAT checkpoint passes — do NOT
# auto-flip MSG-01/MSG-06 to Complete from this list.
requirements-completed: []

# Metrics
duration: ~25min
completed: 2026-05-29
---

# Phase 76 Plan 04: Wire StagingPanel merge-continue + OperationBanner revert through the MessageEditor Summary

**StagingPanel's inline merge-commit form is replaced by a single modal-routed merge-continue button (default verbatim from `get_merge_message`, cancel makes no commit), and OperationBanner now renders Continue + Abort buttons for a Revert state (previously zero buttons), giving the cancelled-revert recovery path MSG-06 requires.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-29
- **Completed:** 2026-05-29 (code slice; Task 3 human UAT still open)
- **Tasks:** 2 of 3 (Task 3 is a blocking human-verify checkpoint)
- **Files modified:** 4

## Accomplishments
- **StagingPanel (Task 1):** removed `mergeSubject`/`mergeBody` state, the hardcoded `Merge branch '<src>' into <tgt>` pre-fill `$effect`, the `commitMerge` function, and the inline subject/body UI block. The merge-continue commit now runs `get_merge_message` -> `onopenmessageeditor(default, "Merge commit message")` -> `merge_continue`; a null return makes no commit and leaves the in-progress merge visible (D-02), so the button doubles as the clean-merge retry affordance. The default is no longer constructed in the frontend (MSG-04) and the old unconditional `into <tgt>` divergence is gone.
- **OperationBanner (Task 2):** added `isRevert` and a `{#if isRevert}` button block modeled on the rebase block (reusing `var(--color-success/danger-*)` tokens). Continue routes through the host modal (`get_merge_message` -> editor -> `revert_continue`, null = no commit); Abort confirms then calls `revert_abort` and fires `onaction`. The dead `handleContinue` `isMerge ? "merge_continue"` branch was left unwired (RESEARCH finding 6).
- `onopenmessageeditor` re-threaded StagingPanel -> OperationBanner (OQ-2).
- Behavior tests added: merge-continue modal routing + cancel-no-commit + inline-form-gone + abort-preserved (StagingPanel); revert Continue/Abort render + abort invoke + continue routing + cancel-no-commit + merge-has-no-revert-buttons (OperationBanner).
- Closed the standing todo `2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` (moved to done/).

## Task Commits

1. **Task 1: route StagingPanel merge-continue through MessageEditor, remove inline form** - `1c86063` (feat)
2. **Task 2: OperationBanner revert Continue/Abort affordance** - `d3354cf` (feat)

**Task 3:** blocking `checkpoint:human-verify` — NOT executed by the agent (in-app UAT). See the CHECKPOINT REACHED report; the plan is not fully complete until the operator approves.

**Plan metadata:** (this docs commit)

## Files Created/Modified
- `src/components/StagingPanel.svelte` - Inline merge form removed; `runMergeContinue` modal-routed handler; single "Commit merge" button + preserved "Abort Merge"; `onopenmessageeditor` threaded to `<OperationBanner>`.
- `src/components/StagingPanel.test.ts` - merge-continue describe block (4 behaviors); local mocks already present.
- `src/components/OperationBanner.svelte` - `onopenmessageeditor` prop; `isRevert`; `handleRevertContinue` + `handleRevertAbort`; `{#if isRevert}` Continue/Abort button block.
- `src/components/OperationBanner.test.ts` - revert-recovery describe block (5 behaviors); mocks declared locally.

## Decisions Made
- **Preserved `abortMerge` when deleting the inline merge form.** See deviation Rule 1 below — this is the load-bearing decision of the plan.
- **Renamed handler to `runMergeContinue`** to keep the literal `commitMerge` grep gate clean.
- **Declared `onopenmessageeditor` on OperationBanner in Task 1** (Rule 3) so the intermediate commit stayed svelte-check green; Task 2 wired its consumption.
- **Local Tauri mocks in OperationBanner.test.ts.** The shared side-effect helper left `vi.mocked(invoke).mockReset` undefined (hoisting detaches the component's invoke instance). Reverted to self-contained local `vi.mock` declarations (the proven StagingPanel/CommitGraph shape, 76-03 decision).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Preserved `abortMerge` / `mergeLoading` instead of deleting the whole `{:else if isMerge}` block**
- **Found during:** Task 1
- **Issue:** The plan (and RESEARCH finding 5) described the inline merge block as "subject input + body textarea + commitMerge button" and instructed deleting the whole block. The block ALSO contained the **Abort Merge** button — the only merge-recovery affordance in the UI (OperationBanner renders buttons only for rebase; for merge it is label-only). Deleting it verbatim would ship a merge that cannot be aborted, regressing D-02/MSG-06 for merge — the exact "trapped user" failure this phase fixes for revert.
- **Fix:** Deleted `mergeSubject`/`mergeBody`/the pre-fill `$effect`/`commitMerge`, but kept `mergeLoading` and `abortMerge`. The bottom block is now two buttons (modal-routed "Commit merge" + existing "Abort Merge"), i.e. the *commit form* was replaced, not the recovery path. `mergeLoading` is not in the acceptance grep ban-list, so the gate still passes.
- **Files modified:** src/components/StagingPanel.svelte
- **Verification:** added test "still renders the Abort Merge recovery button in merge state"; `just check` green.
- **Committed in:** `1c86063` (Task 1)

**2. [Rule 3 - Blocking] Declared `onopenmessageeditor` on OperationBanner to keep svelte-check green in the Task 1 commit**
- **Found during:** Task 1 (threading the prop onto `<OperationBanner>`)
- **Issue:** StagingPanel passing `onopenmessageeditor` to `<OperationBanner>` produced "Object literal may only specify known properties" because OperationBanner's `Props` did not declare it.
- **Fix:** Added the optional prop to OperationBanner's `Props` (consumed in Task 2). Same wave-seam pattern 76-03 used for StagingPanel.
- **Files modified:** src/components/OperationBanner.svelte
- **Committed in:** `1c86063` (Task 1)

**3. [Rule 1 - Bug] Formatted StagingPanel.test.ts to satisfy `biome ci`**
- **Found during:** Task 2 (`just check`)
- **Issue:** A multi-line `expect(...).toBeNull()` written in Task 1 differed from biome's formatting; `biome ci` failed.
- **Fix:** `biome check --write` collapsed it; committed with Task 2.
- **Committed in:** `d3354cf` (Task 2)

---

**Total deviations:** 3 auto-fixed (1 Rule 1 design-regression catch, 1 Rule 3 blocking, 1 Rule 1 format). No scope creep — merge-abort was preserved, not moved to OperationBanner (RESEARCH finding 6 keeps the revert→OperationBanner / merge→StagingPanel split).

## Issues Encountered
- **Shared-mock hoisting regression** (OperationBanner.test.ts) — `vi.mocked(invoke).mockReset` was not a function when relying on the side-effect tauri-mock import; resolved by declaring mocks locally (see Decisions). No fix-attempt limit reached.
- **`commitMerge` grep substring** — the first handler name `commitMergeViaEditor` tripped the literal acceptance grep; renamed to `runMergeContinue`.

## Known Stubs
None. Task 3 is an unexecuted human-verify checkpoint, not a stub — the code slice (Tasks 1 & 2) is complete and fully tested at the vitest layer.

## Threat Flags
None. No new network endpoints, auth paths, file access, or schema changes. The edited message crosses WebView -> Rust as a serde-serialized IPC `invoke` arg (T-76-09, already mitigated by the begin/continue backend); the frontend adds no shell surface.

## Outstanding: Task 3 (blocking human-verify)
The end-to-end in-app UAT (merge --continue, merge `<branch>`, revert, empty-message/cancel recovery) is a `checkpoint:human-verify gate="blocking"` step that the agent cannot perform and did not fabricate. MSG-01 and MSG-06 stay In Progress in REQUIREMENTS.md until the operator approves. See the CHECKPOINT REACHED report returned to the orchestrator for the exact scenarios.

## Self-Check: PASSED
- FOUND: src/components/StagingPanel.svelte, StagingPanel.test.ts, OperationBanner.svelte, OperationBanner.test.ts
- FOUND: commit 1c86063 (Task 1), d3354cf (Task 2)
- `just check` fully green (fmt, biome, svelte-check, clippy, cargo tests, 584 vitest tests)
- grep gates: inline merge form symbols gone; get_merge_message present (StagingPanel); isRevert + revert_abort + revert_continue present (OperationBanner); no inline hex in the revert block; dead :40 merge_continue branch unwired

---
*Phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert*
*Completed (code slice): 2026-05-29 — Task 3 human UAT open*
