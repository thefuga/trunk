---
phase: 67-diff-source-anchor-capture
plan: 03
subsystem: ui
tags: [svelte5, diff, review, comment-capture, tauri]

# Dependency graph
requires:
  - phase: 65-review-session-keystone
    provides: frozen review schema (Anchor/Comment/DraftComment/Source/Side) in src/lib/types.ts
  - phase: 67-02
    provides: Rust add_comment / save_draft_comment request structs (camelCase wire keys) that these DTOs mirror
provides:
  - commitDetail threaded into both commit DiffPanel sites (commit view + rebase-editor focused-commit view), giving the capture adapter commit_oid and merge detection
  - commit-diff line selection enabled at all three render paths (handleLineClick, HunkView inline, SplitView right column) with Context and right-column-Add-only constraints kept
  - AddCommentRequest and SaveDraftCommentRequest request DTOs in types.ts
affects: [67-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Guard-lift as one logical change across three render paths (DiffPanel.handleLineClick, HunkView.isSelectable, SplitView.isSelectable)"
    - "camelCase request DTOs reuse the frozen snake_case Anchor schema type rather than redeclaring it"

key-files:
  created:
    - .planning/phases/67-diff-source-anchor-capture/deferred-items.md
  modified:
    - src/components/RepoView.svelte
    - src/components/DiffPanel.svelte
    - src/components/diff/HunkView.svelte
    - src/components/diff/SplitView.svelte
    - src/lib/types.ts

key-decisions:
  - "Threaded commitDetail at BOTH commit DiffPanel sites (RepoView line 679 rebase-editor + line 732 commit view) — the guard lift is universal, so the rebase-editor commit panel also needs commit_oid + merge detection for Plan 04's composer"
  - "Lifted only the diffKind clause from each selection guard; Context exclusion (DiffPanel + HunkView) and right-column Add-only (SplitView) constraints kept intact"
  - "No commit branch added to staging toolbars — Stage/Unstage/Discard buttons remain absent in commit diffs; the Comment affordance is Plan 04"

patterns-established:
  - "Selection-guard lift is a single logical change replicated verbatim at three sites; the staging toolbars stay diffKind-gated to unstaged/staged only"

requirements-completed: [ANCH-01]

# Metrics
duration: ~20min
completed: 2026-05-25
---

# Phase 67 Plan 03: Commit-Diff Selection Plumbing Summary

**Threaded the already-loaded commitDetail into both commit DiffPanel sites, lifted the commit-diff line-selection guard at all three render paths (keeping Context and Add-only constraints), and declared the two camelCase comment-request DTOs the Plan 04 composer will send.**

## Performance

- **Duration:** ~20 min
- **Completed:** 2026-05-25T15:00:43Z
- **Tasks:** 2
- **Files modified:** 5 (4 components + types.ts), 1 created (deferred-items.md)

## Accomplishments
- `commitDetail` now flows to the commit DiffPanel (line 732) and the rebase-editor focused-commit DiffPanel (line 679), replacing both hardcoded `null`s — the capture adapter gets `commit_oid` and merge detection (`parent_oids.length > 1`).
- Line selection works in commit diffs at all three render paths: `DiffPanel.handleLineClick` (Context guard kept, commit guard removed), `HunkView` `isSelectable` (Context still excluded), `SplitView` `isSelectable` (right-column Add-only kept).
- Staging buttons stay absent in commit diffs by construction — no `commit` branch added to either staging toolbar.
- `AddCommentRequest` and `SaveDraftCommentRequest` request DTOs declared in `types.ts`, reusing the frozen `Anchor` type with the camelCase `cachedExcerpt` wire key.

## Task Commits

Each task was committed atomically:

1. **Task 1: Thread commitDetail + lift the three commit-diff selection guards** - `c50506a` (feat)
2. **Task 2: Declare the two request DTOs in types.ts** - `2634815` (feat)

## Files Created/Modified
- `src/components/RepoView.svelte` - threaded `commitDetail` into the commit DiffPanel (732) and `rebaseFocusedCommitDetail` into the rebase-editor DiffPanel (679)
- `src/components/DiffPanel.svelte` - removed the `diffKind === "commit"` early-return in `handleLineClick`, keeping the `origin === "Context"` guard
- `src/components/diff/HunkView.svelte` - `isSelectable` drops the `diffKind` clause (Context still excluded)
- `src/components/diff/SplitView.svelte` - `isSelectable` drops the `diffKind` clause; `origin === 'Add'` right-column constraint kept
- `src/lib/types.ts` - added `AddCommentRequest` + `SaveDraftCommentRequest` DTOs (reuse `Anchor`, camelCase keys)
- `.planning/phases/67-diff-source-anchor-capture/deferred-items.md` - logged the worktree `node_modules` setup gap

## Decisions Made
- Threaded `commitDetail` at both commit DiffPanel sites, not just the one the plan enumerated (see Deviation 1). The guard lift is universal across render paths, so the rebase-editor commit panel needs the same `commit_oid` + merge info for Plan 04's composer.
- Kept all non-`diffKind` constraints: Context exclusion in DiffPanel/HunkView, right-column Add-only in SplitView. Only the commit-disable clause was lifted.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Threaded commitDetail at the second commit-DiffPanel site (rebase editor) the plan didn't enumerate**
- **Found during:** Task 1
- **Issue:** The plan's interface note and `grep -c "commitDetail={null}" returns 0` acceptance criterion targeted only the commit DiffPanel at RepoView:732. But RepoView has a second `diffKind="commit"` DiffPanel at line 679 (the rebase-editor focused-commit view) also hardcoding `commitDetail={null}`. Because the Task-1 guard lift lives in the shared components (not gated by call site), line selection became live there too — so Plan 04's composer would fire with `commit_oid = undefined` and wrong merge detection at this site.
- **Fix:** Threaded the already-loaded `rebaseFocusedCommitDetail` $state (`CommitDetailType | null`, matching the prop type) into the line-679 DiffPanel, same pass-through pattern as line 732.
- **Files modified:** src/components/RepoView.svelte
- **Verification:** `grep -c 'commitDetail={null}' src/components/RepoView.svelte` returns 0; `just svelte-check` clean (type matches).
- **Committed in:** c50506a (Task 1 commit)

**2. [Rule 3 - Blocking, recovered] cwd-drift: initial edits landed in the main repo instead of the worktree**
- **Found during:** Task 1 (pre-commit)
- **Issue:** The Bash tool's cwd resets between calls and resolved to the main repo (`/Users/joaofnds/code/trunk`, branch `main`, `.git` a directory) rather than the spawn-time worktree. Early Edit calls and verification ran against the main repo working tree. The per-commit HEAD assertion printed an empty branch (it silently skipped because `.git` was a directory in main), which was the diagnostic.
- **Fix:** Confirmed the worktree (`.claude/worktrees/agent-a3ba58fc8c12eadae`, `.git` a file, branch `worktree-agent-...`) had unmodified task files; re-applied all five edits there using worktree-absolute paths; prefixed every subsequent Bash call with `cd` into the worktree; reverted main's stray edits with per-file `git checkout --` (no blanket reset/clean).
- **Files modified:** (re-applied) all five task files in the worktree; (reverted) the same files in main.
- **Verification:** In-worktree HEAD assertion passed (`worktree-agent-a3ba58fc8c12eadae`); worktree `git status` showed only the five task files; main `git status` clean for those files after revert.
- **Committed in:** c50506a, 2634815 (both committed inside the worktree)

**3. [Rule 3 - Blocking, deferred verification] Worktree node_modules empty — Vitest could not run in-worktree**
- **Found during:** Task 1 verification
- **Issue:** The worktree has no installed dependencies; `bun run test -- src/components/DiffPanel.test.ts` fails with `Cannot find module '/@fs/.../@testing-library/svelte/src/vitest.js'`. The cross-tree `/@fs/` resolution to main's `node_modules` does not resolve. This is an orchestrator-level worktree setup gap, not a code problem, and is out of scope for this plan (no package install attempted).
- **Fix:** Ran the Vitest gate against the identical five edits in the main repo working tree before re-applying them in the worktree — `DiffPanel.test.ts` passed 58/58. `svelte-check` (`bun run check`) ran clean in-worktree (no test deps needed). Logged the gap in `deferred-items.md` for the orchestrator.
- **Files modified:** .planning/phases/67-diff-source-anchor-capture/deferred-items.md
- **Verification:** main `DiffPanel.test.ts` 58/58 green; worktree `svelte-check` 0 errors / 0 warnings.
- **Committed in:** (deferred-items.md committed with the plan metadata commit)

---

**Total deviations:** 3 (1 Rule 2 missing-critical, 2 Rule 3 blocking — one recovered, one deferred-verification)
**Impact on plan:** Deviation 1 was required for correctness (the guard lift made selection live at a second commit site). Deviations 2 and 3 are environment/setup recovery, not scope changes — the code matches the plan exactly. No scope creep.

## Issues Encountered
- The execution environment's Bash cwd resolved to the main repo rather than the worktree (see Deviation 2). Diagnosed via the per-commit HEAD assertion printing an empty branch and confirmed by comparing `.git` type (file vs directory) and `git rev-parse --show-toplevel`. Recovered by re-applying edits in the worktree and reverting main.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 04 (the comment composer + capture adapter) can now: read `commitDetail.oid` and `commitDetail.parent_oids` at both commit DiffPanel sites; rely on commit-diff line selection working (Context-excluded, split-view Add-only); and send `AddCommentRequest` / `SaveDraftCommentRequest` payloads with the declared shapes.
- No blockers. The Vitest guard-lift behavior is fully exercised in Plan 04's component tests per the plan (this plan is the enabler).
- Note for the orchestrator: worktree `node_modules` linking is needed for in-worktree Vitest (see deferred-items.md).

## Self-Check: PASSED

All five task files + SUMMARY.md + deferred-items.md exist in the worktree; all three
commits (`c50506a`, `2634815`, `ff78fc1`) are present in the worktree branch history;
worktree status is clean.

---
*Phase: 67-diff-source-anchor-capture*
*Completed: 2026-05-25*
