---
phase: 72-review-pane-ux-integration
plan: 04
subsystem: ui
tags: [svelte, tauri-menu, rust, cleanup, deletion, accelerator]

# Dependency graph
requires:
  - phase: 72-review-pane-ux-integration
    provides: ReviewSession surface (72-01), Toolbar Review button (72-02), ReviewPanel Copy refactor (72-03)
provides:
  - Dead-code removal of ReviewDocPreview.svelte and ReviewDocPreview.test.ts
  - RepoView.svelte stripped of the obsolete blue Review button header strip
  - Cmd+Shift+R keyboard accelerator bound to the View -> Start/End Code Review menu item
affects: [73, 74, 75, future-review-ux, future-keyboard-shortcuts]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Tauri MenuItemBuilder accelerator chaining (mirrors find/CmdOrCtrl+F precedent at lib.rs:21-23)"

key-files:
  created: []
  modified:
    - src/components/RepoView.svelte
    - src-tauri/src/lib.rs
    - src/components/ReviewPanel.test.ts (stale comment scrub)
  deleted:
    - src/components/ReviewDocPreview.svelte
    - src/components/ReviewDocPreview.test.ts

key-decisions:
  - "Updated the stale `Temporary trigger for the review-session stub (D-12); replaced by the real panel in Phase 69` comment above `review_item` to describe the now-shipped behavior — Phase 72 is the real panel that pre-Phase-72 plans referenced as future work."

patterns-established:
  - "Tauri menu accelerator: chain `.accelerator(\"CmdOrCtrl+Mod+Key\")` between `MenuItemBuilder::with_id(...)` and `.build(app)?`. No additional handler wiring needed — the existing `on_menu_event` matcher fires for both menu-click and accelerator paths."

requirements-completed: [REQ-72-1b, REQ-72-1c, REQ-72-4a, REQ-72-5a, REQ-72-5b, REQ-72-6, G-71-A, G-71-B]

# Metrics
duration: 8min
completed: 2026-05-27
---

# Phase 72 Plan 04: Cleanup + Cmd+Shift+R Accelerator Summary

**Deleted ReviewDocPreview component and its test, removed the obsolete blue Review button strip from RepoView.svelte, and bound Cmd+Shift+R to the View -> Start/End Code Review Tauri menu item.**

## Performance

- **Duration:** ~8 min (excluding `bun install` bootstrap of empty worktree node_modules)
- **Started:** 2026-05-27T02:55Z
- **Completed:** 2026-05-27T03:05Z
- **Tasks:** 2 of 3 (Task 3 is a blocking manual-UAT checkpoint surfaced to the user — see below)
- **Files modified:** 3 (RepoView.svelte, src-tauri/src/lib.rs, ReviewPanel.test.ts comment)
- **Files deleted:** 2 (ReviewDocPreview.svelte, ReviewDocPreview.test.ts)

## Accomplishments

- `src/components/ReviewDocPreview.svelte` and `src/components/ReviewDocPreview.test.ts` removed via `git rm` — both unreferenced after Plan 03 collapsed the Generate/Preview surface into a single Copy action.
- `src/components/RepoView.svelte` lines 813-828 (pre-72 numbering) deleted: the inline-style ternary `<button>Review</button>` that violated the "never inline colors" CLAUDE.md rule is gone. The inner conditional that picks between DiffPanel and ReviewPanel survives unchanged; the surviving DiffPanel `onclose={() => { handleDiffClose(); reviewSession.showPanel(); }}` is the back-affordance after a comment jump (CONTEXT.md success criterion 5).
- `src-tauri/src/lib.rs:28` now chains `.accelerator("CmdOrCtrl+Shift+R")` onto the existing `review_item` MenuItemBuilder. The `on_menu_event` matcher at lib.rs:66-69 already routes `"review-toggle"` -> `app.emit("review-toggle", ())`, so no handler change was needed.
- `just check` exits 0: fmt, biome, svelte-check, clippy, cargo test (109+ Rust tests across multiple test binaries), and vitest (529 frontend tests across 49 files) all green.

## Task Commits

1. **Task 1: Delete ReviewDocPreview component and its test** — `c944d84` (chore)
   - `git rm src/components/ReviewDocPreview.svelte src/components/ReviewDocPreview.test.ts`
   - Also scrubbed a stale `// ReviewDocPreview.test.ts` comment reference in `src/components/ReviewPanel.test.ts:641` so the verification `rg -n "ReviewDocPreview" src/` returns no matches.
2. **Task 2: Delete blue-button header strip + add Cmd+Shift+R accelerator** — `f3a4218` (feat)
   - RepoView.svelte: 38 lines removed, 23 simpler lines inserted (net -15).
   - lib.rs one-line diff: `MenuItemBuilder::with_id(...).build(app)?` -> `MenuItemBuilder::with_id(...).accelerator("CmdOrCtrl+Shift+R").build(app)?` (formatted across three lines).

**Plan metadata commit:** appended after this SUMMARY.md is written.

### Exact pre-deletion blue-strip block (PATTERNS.md lines 418-435, removed at old `src/components/RepoView.svelte:813-828`)

```svelte
<div class="flex flex-col" style="height: 100%; min-height: 0;">
  <div class="flex items-center" style="gap: 8px; padding: 4px 8px; border-bottom: 1px solid var(--color-border); flex-shrink: 0; background: var(--color-surface);">
    <button
      type="button"
      onclick={() => reviewSession.showPanel()}
      style="
        background: {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-accent)' : 'transparent'};
        color: {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-bg)' : 'var(--color-text-muted)'};
        border: 1px solid {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-accent)' : 'var(--color-border)'};
        border-radius: 4px;
        cursor: pointer;
        padding: 2px 10px;
        font-size: 12px;
      "
    >Review</button>
  </div>
  ...inner conditional moves up one level...
</div>
```

### One-line Rust diff (`src-tauri/src/lib.rs:27-29` after edit)

```rust
let review_item = MenuItemBuilder::with_id("review-toggle", "Start/End Code Review")
    .accelerator("CmdOrCtrl+Shift+R")
    .build(app)?;
```

## Files Created/Modified

- `src/components/RepoView.svelte` — removed the outer flex-column wrapper, the header strip with the inline-style ternary `<button>Review</button>`, and the borderless `<div>` wrapper around the inner conditional. The DiffPanel/ReviewPanel conditional now lives directly inside `{#if reviewSession.state.reviewActive}` under a single `<div class="flex flex-col" style="flex: 1; min-height: 0; overflow: hidden;">` wrapper.
- `src-tauri/src/lib.rs` — chained `.accelerator("CmdOrCtrl+Shift+R")` onto the `review_item` `MenuItemBuilder`; replaced the now-stale "D-12 stub; Phase 69 placeholder" comment above it with a current description.
- `src/components/ReviewPanel.test.ts` — removed the trailing sentence of the Phase-72 Copy describe-block comment that still referenced the now-deleted `ReviewDocPreview.test.ts`.

## Decisions Made

- **Updated the stale `review_item` comment in-flight.** Per `coding_style.md` section 1 ("Reserve comments for *why*, never *what*; out-of-date comments are noise"), the "Temporary trigger for the review-session stub (D-12); replaced by the real panel in Phase 69" comment was outdated — Phase 72 *is* the panel it referenced as future work. New comment: "View -> Start/End Code Review menu item; emits `review-toggle` so the frontend can flip review mode. Accelerator mirrors find -> search-toggle." Trivial in-scope fix per the ownership rule.
- **Scrubbed the stale `ReviewDocPreview.test.ts` reference** in the Phase-72 Copy describe-block comment of `ReviewPanel.test.ts` so the plan's `rg -n "ReviewDocPreview" src/` verification gate returns zero matches without leaving misleading historical pointers.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Bootstrap empty worktree `node_modules` before running vitest**

- **Found during:** Task 1 verification (`bun run vitest run`).
- **Issue:** This worktree was spawned with empty `node_modules` — vitest exited 1 with `Cannot find module '/@fs/.../node_modules/@testing-library/svelte/src/vitest.js'` on every test file. Not a code defect; the per-worktree filesystem isolation simply hadn't installed deps yet.
- **Fix:** Ran `bun install` (176 packages, 289ms). Re-ran vitest -> 529 tests pass.
- **Files modified:** none committed (`node_modules` is gitignored).
- **Verification:** Subsequent vitest runs and `just check` exit 0.
- **Committed in:** N/A — environment fix, not a code change.

**2. [Rule 1 - Bug] Scrubbed stale `// ReviewDocPreview.test.ts` comment**

- **Found during:** Task 1 (deletion verification).
- **Issue:** `src/components/ReviewPanel.test.ts:641` still carried a comment ending "Pattern carry-forward from the now-deleted ReviewDocPreview.test.ts." After Task 1's `git rm`, that comment would (a) fail the plan's `rg -n "ReviewDocPreview" src/` zero-match gate and (b) point readers at a file that no longer exists — a small but real correctness defect for future code archaeology.
- **Fix:** Truncated the comment after `instanceof Error narrowing.`; the explanatory lead-in still documents the carry-forward without the stale file pointer.
- **Files modified:** `src/components/ReviewPanel.test.ts`.
- **Verification:** `rg -n "ReviewDocPreview" src/` returns no matches; vitest still passes 529/529.
- **Committed in:** `c944d84` (Task 1 commit).

**3. [Rule 1 - Bug] Replaced stale `review_item` comment in `src-tauri/src/lib.rs`**

- **Found during:** Task 2.
- **Issue:** The comment above `let review_item = ...` read "Temporary trigger for the review-session stub (D-12); replaced by the real panel in Phase 69." Phase 72 *is* the panel; the comment misrepresents current state.
- **Fix:** Rewrote to describe current behavior: "View -> Start/End Code Review menu item; emits `review-toggle` so the frontend can flip review mode. Accelerator mirrors find -> search-toggle."
- **Files modified:** `src-tauri/src/lib.rs`.
- **Verification:** `just check` exits 0 (clippy/fmt did not object).
- **Committed in:** `f3a4218` (Task 2 commit).

**Total deviations:** 3 auto-fixed (1 blocking environment bootstrap, 2 bug-class stale-comment scrubs).
**Impact on plan:** All three are necessary for correctness / verification gates / freshness; no scope creep. None of the three changed observable behavior beyond what the plan ordered.

## Issues Encountered

- The Edit and Write tools reported success multiple times on substantive edits/writes (RepoView.svelte blue-strip removal, ReviewPanel.test.ts comment scrub, lib.rs accelerator, and the first attempt to create this SUMMARY.md) but did not flush the writes to disk. Each time the disk content remained unchanged. Worked around by switching to `python3` heredocs that do an in-process `s.replace(old, new, 1)` or `open(..., 'w').write(content)` against the disk file. All substantive edits/writes ultimately landed via Python, verified via post-edit `grep`/`ls` against disk. No code defect; tooling friction only. **Note for future reflection (continuous_improvement section 1):** repeated friction signal — Edit and Write tools appear to silently no-op on certain inputs in this worktree session; worth a recurring follow-up to investigate the trigger.

## Manual UAT Required (Task 3 — blocking checkpoint surfaced to user)

This plan is `autonomous: false` because three behaviors have no automated test coverage and require a running Tauri app. **The orchestrator must surface this to the user.** Run `just dev` and verify all five steps:

1. **REQ-72-1c — View menu regression:** Open macOS View menu. Confirm "Start Code Review" (or "End Code Review" if active) is present. Confirm the keyboard hint `Cmd+Shift+R` displays next to it. Click -> review mode toggles -> Toolbar Review button changes active state.

2. **REQ-72-1b — Cmd+Shift+R accelerator:** With the app focused (any pane), press `Cmd+Shift+R`. Expected: review mode toggles — Toolbar Review button flips active state, ReviewPanel appears (or disappears). Press again -> reverses. Confirm from at least two focus contexts (e.g., commit graph focused, then diff panel focused).

3. **REQ-72-5b — DiffPanel close returns to ReviewPanel:** Enter review mode. Add a line-anchored comment in a commit. From ReviewPanel, click the comment to jump to the diff. Confirm DiffPanel renders. Click DiffPanel close. Expected: right pane returns to ReviewPanel (comments list), NOT a blank state. Verifies the surviving `onclose={() => { handleDiffClose(); reviewSession.showPanel(); }}` wiring still works after the blue-strip deletion.

4. **Visual sanity (Plan 02 spot-check):** Toolbar Review button on right edge. Default state: flat. Active state: accent-blue background, white text. Inspect element: `aria-pressed` toggles `"true"`/`"false"`.

5. **NOT TESTED (out of scope per RESEARCH Pitfall 2):** Do NOT verify light-mode rendering. The app is forced dark (`src/app.css:4`).

Report failure with step number and observed vs. expected behavior. Type "approved" once all five pass.

## Phase 72 Closeout

**Phase 72 success criteria 1-6 are now all met (pending manual UAT sign-off above).** The milestone is ready for `/gsd:verify-work 72`.

- REQ-72-1a (Toolbar entry surface): delivered in Plan 02.
- REQ-72-1b (Cmd+Shift+R accelerator): code shipped here in `c944d84` + `f3a4218`; manual UAT pending.
- REQ-72-1c (View menu retained): code intact; manual UAT confirms keyboard hint renders.
- REQ-72-4a (Copy action replaces Generate/Preview): delivered in Plan 03.
- REQ-72-5a (no inline button strip): delivered here.
- REQ-72-5b (DiffPanel close returns to ReviewPanel): wiring preserved; manual UAT pending.
- REQ-72-6 (`just check` is the phase gate): exits 0.
- G-71-A / G-71-B (no inline colors, no dead UI): cleared here.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 72 closes the Review UX integration thread. The Toolbar button, Cmd+Shift+R accelerator, View menu item, and DiffPanel close-back all route through `reviewSession` cleanly.
- The only outstanding gate is the manual UAT above. If any UAT step fails, surface the failure with the step number — fixes will be small and localized.
- After UAT pass: run `/gsd:verify-work 72` to formalize milestone closure.

## Continuous Improvement Reflection (rules section 1)

1. **What was harder than expected?** The Edit and Write tools silently no-op'd on multiple substantive edits/writes to existing files. The first attempts presented as "succeeded" but disk read confirmed no change. Recovered via Python `replace()` / `open().write()`. Cost: ~5 extra minutes.
2. **Was anything done twice?** Yes — several edits and the SUMMARY write were attempted via Edit/Write, then re-attempted via Python. Eliminate the repetition by reaching for Python (or `sed -i` / heredoc cat) directly whenever an Edit/Write fails verification, instead of retrying the same tool call.
3. **Did I make any incorrect assumptions?** Assumed the worktree would have a populated `node_modules` (bun's worktree isolation surprised the first vitest run). Worth capturing: parallel-executor worktrees may need `bun install` as a default first step.
4. **Is there a follow-up improvement?** Friction: Edit/Write silently no-op; root cause unclear (possibly tab/space or large-content handling); fix: short-circuit to Python on first tool-verification failure; benefit: ~5 min saved per affected task across the team; cost: zero (just a habit shift).
5. **Should any memory files be updated?** "Worktree GSD parallel-execute may need `bun install` first" + "Edit/Write tools can silently no-op in worktree sessions — verify on disk before believing the tool" are both worth a note in `project_gsd_workflow_gotchas.md`; logging from this session, not by this agent.

## Self-Check

- File `src/components/ReviewDocPreview.svelte`: `test ! -f` exits 0 (MISSING from disk: confirmed).
- File `src/components/ReviewDocPreview.test.ts`: `test ! -f` exits 0 (MISSING from disk: confirmed).
- Commit `c944d84`: found in `git log --oneline --all`.
- Commit `f3a4218`: found in `git log --oneline --all`.
- `src/components/RepoView.svelte` no longer contains `background: {reviewSession.state.rightPaneMode === 'panel' ?` (grep returns 0).
- `src/components/RepoView.svelte` no longer contains `onclick={() => reviewSession.showPanel()}` (grep returns 0).
- `src/components/RepoView.svelte` still contains `onclose={() => { handleDiffClose(); reviewSession.showPanel(); }}` (line 824).
- `src/components/RepoView.svelte` still contains `{#if reviewSession.state.reviewActive}` (line 810).
- `src-tauri/src/lib.rs` contains `.accelerator("CmdOrCtrl+Shift+R")` (line 28).
- `just check` exit code: 0.

## Self-Check: PASSED
