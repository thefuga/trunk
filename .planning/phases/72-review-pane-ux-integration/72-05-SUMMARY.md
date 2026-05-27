---
phase: 72-review-pane-ux-integration
plan: 05
subsystem: ui
tags: [gap-closure, ux-bugfix, css-layout, tauri-menu, requirement-retraction]

# Dependency graph
requires:
  - phase: 72-review-pane-ux-integration
    provides: 72-04 RepoView/menu cleanup (the change set this plan refines)
provides:
  - View menu "Start/End Code Review" entry without a keyboard accelerator
  - Restored ReviewPanel comments-list scroll (wrapper sized with height:100%)
  - Phase 72 docs reflect REQ-72-1b retraction (VERIFICATION, VALIDATION)
  - Phase 73 carry-forward bundle for cold-boot resume + End-review redesign
affects: [73-review-lifecycle-management]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wrapper sizing for nested flex-children: prefer height:100% when the parent is a flex *child* (no display:flex on it), since flex:1 then resolves to intrinsic height and any inner scroll body loses its constrained height."
    - "Requirement retraction: when UAT contradicts a planned binding (here, OS-shortcut clash), retract rather than substitute — record RETRACTED in VERIFICATION.md/VALIDATION.md and defer any redesign to the next phase, paired with the affordance that motivated the original requirement."

key-files:
  created:
    - .planning/phases/72-review-pane-ux-integration/72-VERIFICATION.md (was untracked in main checkout; first-tracked copy reflects retraction)
    - .planning/todos/pending/phase-73-review-lifecycle.md
  modified:
    - src-tauri/src/lib.rs
    - src/components/RepoView.svelte
    - .planning/phases/72-review-pane-ux-integration/72-VALIDATION.md

key-decisions:
  - "Retract REQ-72-1b (Cmd+Shift+R) rather than rebind: user's UAT surfaced a clash with launcher tools (Recents) and the user prefers no shortcut at all for review toggle."
  - "Restore the pre-72 wrapper sizing (height:100%) on RepoView.svelte's review-mode wrapper rather than restructuring the parent. The 72-04 cleanup over-collapsed the flex chain; a single attribute reverts the regression without re-introducing the deleted wrappers."
  - "Defer Bug 3 (cold-boot resume) to Phase 73, bundled with the End-review redesign. Auto-resume in isolation would worsen the lifecycle permanence problem (advisor finding)."

patterns-established:
  - "Pattern: gap-closure plans for UAT regressions. Three surgical commits (one Rust line, one CSS attribute, three docs), no new tests when the bugs are OS-config or visual layout. just check is the only automated gate."
  - "Pattern: stale-comment cleanup as part of behavior change. When removing a feature (here, the accelerator), update the in-file comment that explains it in the same commit so the next reader doesn't see a lie."

requirements-completed: [REQ-72-1c, REQ-72-3]

# Metrics
duration: 12min
completed: 2026-05-27
---

# Phase 72 Plan 05: Gap closure — drop accelerator, restore scroll, defer lifecycle Summary

**Removed the Cmd+Shift+R accelerator (UAT clash), restored ReviewPanel scroll by sizing the review-mode wrapper with height:100%, and recorded the cold-boot resume + End-review redesign as a Phase 73 carry-forward.**

## Performance

- **Duration:** ~12 min (executor wall-clock; includes `bun install` in the fresh worktree)
- **Started:** 2026-05-27T07:22:00Z (approx)
- **Completed:** 2026-05-27T07:34:30Z
- **Tasks:** 3
- **Files modified:** 5 (2 source, 3 docs)

## Accomplishments

- `src-tauri/src/lib.rs`: dropped the `.accelerator("CmdOrCtrl+Shift+R")` chain from the `review_item` MenuItemBuilder; rewrote the comment at lines 25-27 to tell the truth (no accelerator — UAT clash).
- `src/components/RepoView.svelte:818`: review-mode wrapper restored to `style="height: 100%; min-height: 0; overflow: hidden;"` so the inner ReviewPanel scroll body has a constrained height; expanded the explanatory comment so the next reader doesn't re-collapse the wrapper.
- `72-VERIFICATION.md`: marked REQ-72-1b RETRACTED in the observable-truths table and the per-requirement section; updated G-71-B closure and gaps summary; revised manual UAT items (dropped Cmd+Shift+R, added scroll item for T2).
- `72-VALIDATION.md`: struck REQ-72-1b in all four spots (72-04-T3 row, Nyquist manual-UAT note, per-requirement test map, Manual-Only Verifications table).
- `.planning/todos/pending/phase-73-review-lifecycle.md`: new carry-forward bundling Bug 3 (cold-boot resume) with the End-review redesign, with the advisor concern recorded verbatim and open design questions enumerated for `/gsd:discuss-phase`.

## Task Commits

Each task was committed atomically:

1. **Task 1 (T1): Drop Cmd+Shift+R menu accelerator** — `ba347d0` (fix)
2. **Task 2 (T2): Restore ReviewPanel scroll by sizing wrapper with height:100%** — `9ce5a54` (fix)
3. **Task 3 (T3): Retract REQ-72-1b; record Phase 73 lifecycle carry-forward** — `0a8f0bc` (docs)

_Plan metadata commit (orchestrator-owned, after this SUMMARY)._

## Files Created/Modified

- `src-tauri/src/lib.rs` — removed `.accelerator("CmdOrCtrl+Shift+R")` from the `review_item` builder; rewrote the explanatory comment at lines 25-27 to reflect the UAT-driven retraction
- `src/components/RepoView.svelte` — review-mode wrapper attribute reverted from `flex: 1` to `height: 100%` at line 818; comment expanded with the layout rationale and the entry-point list updated (no more Cmd+Shift+R mention)
- `.planning/phases/72-review-pane-ux-integration/72-VERIFICATION.md` — created in the worktree as a tracked first copy (was untracked in main checkout); marks REQ-72-1b RETRACTED, updates truths/UAT/gaps tables
- `.planning/phases/72-review-pane-ux-integration/72-VALIDATION.md` — four spots struck/annotated as RETRACTED in 72-05
- `.planning/todos/pending/phase-73-review-lifecycle.md` — new file bundling Bug 3 + End-review redesign + advisor concern + open design questions

## Decisions Made

- **Retract REQ-72-1b rather than re-bind.** The user explicitly said they want no keyboard shortcut for code review. Picking an alternative shortcut (`Cmd+Alt+R`, `Cmd+Shift+Y`, etc.) would re-litigate UAT and risk another clash; retraction is the honest closure.
- **Single-attribute fix for the scroll regression.** The 72-04 cleanup collapsed two layout wrappers and changed `height:100%` → `flex:1`. The parent `.flex-1.overflow-hidden` is a flex *child* (no `display:flex` on it), so `flex:1` resolved to intrinsic height. Reverting only the attribute (not re-introducing the deleted wrappers) preserves the cleanup's main benefit and restores the scroll body's constrained height.
- **Defer Bug 3 (cold-boot resume) to Phase 73, paired with End-review.** The advisor's wave-1 finding was decisive: auto-resuming the session at panel mount without an End-review affordance compounds the lifecycle permanence problem rather than fixing it. The Phase 73 todo records both pieces together with open design questions for `/gsd:discuss-phase`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Stale comment] Updated RepoView.svelte review-mode wrapper comment**

- **Found during:** Task 2 (Restore ReviewPanel scroll)
- **Issue:** The comment immediately above the modified wrapper still read "Entry/exit is via Toolbar Review button or Cmd+Shift+R". After T1 retracted the accelerator, that comment was a lie.
- **Fix:** Rewrote the comment to (a) cite the View menu item as the secondary entry point and (b) document the layout rationale (parent is a flex child, so `flex:1` collapses to intrinsic height — height:100% is mandatory).
- **Files modified:** src/components/RepoView.svelte (lines 811-816)
- **Verification:** `grep -n "Cmd+Shift+R" src/components/RepoView.svelte` returns 0 matches; comment expanded as expected.
- **Committed in:** 9ce5a54 (Task 2 commit)

**2. [Rule 3 — Blocking] `bun install` in the fresh worktree**

- **Found during:** Task 2 verification step (`bun run vitest run`)
- **Issue:** The worktree's `node_modules/` was empty; vitest failed with `Cannot find module '@testing-library/svelte/src/vitest.js'` across 49 test files.
- **Fix:** `bun install` in the worktree root (176 packages installed in ~390 ms).
- **Verification:** `bun run vitest run` returned 529/529 passing after install.
- **Note:** This matches the friction recorded in 72-04-SUMMARY ("Worktree GSD parallel-execute may need `bun install` first"). The recurring signal supports a long-running follow-up to ensure the worktree-spawn step seeds `node_modules/` automatically.

**3. [Rule 3 — Blocking] `mise trust` in the fresh worktree**

- **Found during:** First `just check` invocation
- **Issue:** `mise` refused to read the worktree's `mise.toml` ("Config files are not trusted").
- **Fix:** `mise trust .` in the worktree root.
- **Verification:** Subsequent `just check` ran to completion and exited 0.

**Note on a near-deviation that was deliberately *not* taken:** The plan's must-have truth says "RepoView.svelte:814 review-mode wrapper uses height: 100%". After my comment expansion (deviation #1), the wrapper sits at line 818, not 814. The substring contract in `must_haves.artifacts.contains` is still satisfied (the wrapper line contains the exact required style string), and the comment cleanup matters more than the literal line number — but the line drift is worth flagging for verifier sanity.

---

**Total deviations:** 3 auto-fixed (1 stale-comment cleanup, 2 worktree-bootstrap blocking)
**Impact on plan:** None of the deviations changed scope. The stale-comment cleanup paired naturally with T2; the bootstrap blockers were pure tooling friction in a fresh worktree.

## Issues Encountered

- 72-VERIFICATION.md did not exist as a tracked file in the worktree — it lived only as an untracked file in the main checkout. I copied its content into the worktree and committed the updated version as a brand-new tracked file. The main-checkout untracked copy is now stale; the orchestrator may want to delete it after merging this worktree.
- The original verifier scored Truth #3 ("Cmd+Shift+R registered in Rust") as VERIFIED; the updated VERIFICATION.md now marks it RETRACTED with a pointer to the 72-05 commit. The truth-count footer in the doc has been adjusted accordingly.

## User Setup Required

None — no external service configuration introduced or changed by this plan.

## Next Phase Readiness

- **Phase 72 closeable.** All in-scope UAT regressions addressed; the only open Phase 72 item is the user's manual confirmation that the comments list now scrolls (orchestrator handles, not gated by executor).
- **Phase 73 has a structured kickoff.** `.planning/todos/pending/phase-73-review-lifecycle.md` enumerates Bug 3, the End-review ask, the advisor concern, the available backend primitives (`start/resume/end_review_session_inner`), and five open design questions ready for `/gsd:discuss-phase`.
- **Worktree-bootstrap friction is now twice-observed** (72-04, 72-05). Worth lifting into a setup script or a hook on worktree spawn rather than discovered per-executor.

## Continuous Improvement Reflection (per CLAUDE.md ownership rule)

1. **What was harder than expected?** Nothing structural — the three commits were as surgical as the plan promised. The only mild friction was the empty `node_modules/` and untrusted `mise.toml` in the fresh worktree.
2. **Was anything done twice?** Two `just check` runs (one to surface the `mise trust` block, one after trust). Eliminate by making the worktree-spawn hook either pre-trust mise or run `bun install` so the executor starts on a green baseline.
3. **Did I make any incorrect assumptions?** I assumed `72-VERIFICATION.md` would be tracked in the worktree (the plan's `files_modified` lists it). It was only an untracked file in the main checkout — recovered by copying the content via Read and writing a new tracked copy. Worth noting in `project_gsd_workflow_gotchas.md`: "When a gap-closure plan modifies a previously-written artifact, verify the artifact is actually in HEAD before assuming Edit will work."
4. **Is there a follow-up improvement?**
   - **Friction:** Two-step worktree bootstrap (`bun install` + `mise trust .`) every executor session.
   - **Root cause:** Worktree spawn doesn't propagate `node_modules/` (build artifact, correctly out of git) or trust state (`~/.local/state/mise/trusted-configs`, user-scoped).
   - **Proposed fix:** Add a `worktree_post_spawn` hook in `.claude/get-shit-done/...` that runs `bun install --frozen-lockfile` + `mise trust .` automatically.
   - **Expected benefit:** ~2 minutes saved per executor, eliminates the spurious "tests failing" panic moment.
   - **Cost:** Small (one shell script, one hook registration). Bun install on a warm cache is ~400 ms.
5. **Should any memory files be updated?** Yes — `project_gsd_workflow_gotchas.md` already mentions worktree `bun install`; add the `mise trust .` requirement and the "untracked-in-main-checkout artifact" gotcha. Not done by this executor (memory writes outside this plan's scope).

---

*Phase: 72-review-pane-ux-integration*
*Plan: 05 (gap closure)*
*Completed: 2026-05-27*

## Self-Check: PASSED

- All claimed files exist on disk (src-tauri/src/lib.rs, src/components/RepoView.svelte, 72-VERIFICATION.md, 72-VALIDATION.md, phase-73-review-lifecycle.md, this SUMMARY).
- All three task commits exist (ba347d0, 9ce5a54, 0a8f0bc).
- `grep -rn 'CmdOrCtrl+Shift+R' src/ src-tauri/` returns 0 matches.
- `grep -n 'style="height: 100%; min-height: 0; overflow: hidden;"' src/components/RepoView.svelte` finds line 818.
- `just check` exits 0 (verified before T3 commit).
