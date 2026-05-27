---
phase: 72-review-pane-ux-integration
plan: 01
subsystem: review-session-rune
tags: [svelte5, rune, tdd, refactor]
status: complete
requires: []
provides:
  - "createReviewSession().generate(repoPath: string): Promise<string>"
  - "Simplified ReviewSessionState { reviewActive, rightPaneMode }"
affects:
  - "Downstream Plan 03 (ReviewPanel Copy refactor) — can now await session.generate(repoPath) for the markdown"
  - "Downstream Plan 04 (deletion of ReviewDocPreview + preview rendering in ReviewPanel) — preview surface in the rune is gone"
tech_stack:
  added: []
  patterns:
    - "Pure async getter on a rune (no state caching) — caller composes the result"
key_files:
  created: []
  modified:
    - src/lib/review-session.svelte.ts
    - src/lib/review-session.svelte.test.ts
  deleted: []
commits:
  - hash: c26f38e
    type: test
    message: "test(72-01): RED — rewrite rune tests for simplified review-session shape"
  - hash: 032a36e
    type: feat
    message: "feat(72-01): GREEN — simplify review-session rune to two state axes"
decisions:
  - "generate() returns Promise<string> instead of mutating state — composition is the caller's job (CONTEXT.md design lock)"
  - "Each Copy click re-invokes the IPC; no markdown cache in the rune (YAGNI per CONTEXT.md)"
metrics:
  duration_minutes: ~3
  tests_before: 7
  tests_after: 2
  files_changed: 2
  lines_added: 11
  lines_removed: 93
  completed: 2026-05-27
---

# Phase 72 Plan 01: Simplify review-session rune Summary

Collapsed the review-session rune from three state axes to two, deleted the
preview-pane plumbing, and turned `generate(repoPath)` into a pure async
function returning `Promise<string>`. Plan 03 (Copy refactor) can now
compose `const md = await session.generate(repoPath); await writeText(md);`.

## What changed

### `src/lib/review-session.svelte.ts` (–44 lines)

Deleted surface:
- `PanelMode` type export
- `panelMode` and `previewMarkdown` fields on `ReviewSessionState`
- `showList()` and `showPreview(md)` methods (declarations + bodies)
- Initial values for the removed fields in the `$state(...)` factory
- The dead-cleanup branch in `setReviewActive(false)` that was zeroing
  `previewMarkdown` and resetting `panelMode`
- Phase-70-era docstring fragments that named the deleted surface

Reshaped surface:
- `generate(repoPath: string): Promise<void>` → `Promise<string>`
- Body collapses to a single line: `return await safeInvoke<string>("generate_review_doc", { path: repoPath })`
- Docstring rewritten to: "calls `generate_review_doc` IPC and returns the
  markdown. State is untouched; the caller composes the result (e.g.
  writeText for clipboard)."

Surviving surface (untouched verbatim):
- `reviewActive`, `rightPaneMode` fields
- `setReviewActive`, `showPanel`, `showDiff`, `jumpTo` methods
- `JumpDeps` interface, `RightPaneMode` type

### `src/lib/review-session.svelte.test.ts` (–49 lines, +5 lines)

Test count: 7 → 2.

Kept (rewritten):
- `generate returns the markdown string` — RED-gate test; asserts return
  value + IPC argument shape, no state assertions.
- `generate propagates rejection` — `mockRejectedValueOnce` with the
  TrunkError JSON string; asserts `.rejects.toMatchObject({ code: "no_comments" })`.

Deleted (all referenced removed surface):
- `starts with panelMode 'list' and previewMarkdown null`
- `showPreview sets previewMarkdown and switches panelMode to 'preview'`
- `showList returns panelMode to 'list' and preserves previewMarkdown`
- `setReviewActive(false) clears previewMarkdown and resets panelMode to 'list'`
- `setReviewActive(true) does NOT touch preview fields`
- The original `generate awaits safeInvoke for generate_review_doc and stores the result` (rewritten)

Describe block rename: `createReviewSession — preview state` → `createReviewSession — generate`.

## TDD gate compliance

| Gate     | Commit  | Evidence |
|----------|---------|----------|
| RED      | c26f38e | `bun run vitest run src/lib/review-session.svelte.test.ts` → 1 failed (`generate returns the markdown string` got `undefined`, expected `"# generated markdown"`) |
| GREEN    | 032a36e | `bun run vitest run src/lib/review-session.svelte.test.ts` → 2 passed |
| REFACTOR | n/a     | Implementation was already simplest form — single-line `return await safeInvoke<string>(...)` |

## Verification

Plan-level `<verification>` block (PLAN.md lines 187-191) is scoped to the two
touched files:

- `bun run vitest run src/lib/review-session.svelte.test.ts` → 2/2 pass.
- `bunx svelte-check --tsconfig ./tsconfig.json` for the touched files → 0 errors.
- `grep -nE 'panelMode|previewMarkdown|showList|showPreview|PanelMode' src/lib/review-session.svelte.ts` → 0 matches.

Project-wide `svelte-check` reports 4 errors in `src/components/ReviewPanel.svelte`
(references to the now-deleted `state.panelMode`, `state.previewMarkdown`,
`manager.showList`). These are the **planned cross-plan handoff** — Plan 03
rewrites ReviewPanel's Copy handler and Plan 04 deletes the preview rendering
branch (CONTEXT.md "Component changes" table). Project-wide `just check` will
not pass until the wave-2 plans land; that is by design, not a regression.

## Deviations from Plan

None — plan executed exactly as written.

## Downstream handoff note

Plan 03's Copy handler can now consume the rune as:

```ts
const md = await session.generate(repoPath);
await writeText(md);
```

No state-read indirection, no `previewMarkdown` cache to worry about.
Rejection still propagates as a `TrunkError`-shaped object (via `safeInvoke`'s
catch-translate), so the existing try/catch + `instanceof Error` narrowing
pattern from Phase 71 carries forward verbatim.

## Self-Check: PASSED

- src/lib/review-session.svelte.ts — present, contains `Promise<string>`, no forbidden strings.
- src/lib/review-session.svelte.test.ts — present, contains `generate returns the markdown string`, no forbidden strings.
- Commit c26f38e — exists on branch `worktree-agent-a81aceec1ef2af90d`.
- Commit 032a36e — exists on branch `worktree-agent-a81aceec1ef2af90d`.

## Post-task reflection

1. **What was harder than expected?** Worktree base correction — HEAD pointed
   at `bf4446a` (pre-phase-72) instead of the expected `19384c8`. The startup
   `<worktree_branch_check>` block recovered automatically via `git reset --hard`.
   None of this was about the work itself.
2. **Was anything done twice?** No. `bun install` was a one-time setup
   cost because the worktree had no `node_modules`.
3. **Did I make any incorrect assumptions?** Initially assumed `pre-commit`
   hooks would gate the GREEN commit on full-project `svelte-check` (which
   would have failed on the cross-plan handoff). Checked — no hooks installed.
   No action needed.
4. **Is there a follow-up improvement?**
   - Friction: an executor agent landing on a worktree with no `node_modules`
     has to either install or fail every test command. Cost is ~6s here; could
     compound across many parallel agents.
   - Root cause: worktree spawn doesn't share `node_modules` with the parent.
   - Proposed fix: orchestrator could `bun install` once before fanning out
     waves; or document in the worktree-spawn step that `bun install` is
     expected on first command.
   - Benefit: removes a 6–60s wait from every parallel agent's startup.
   - Cost: small change to the orchestrator workflow, no risk if `bun install`
     is idempotent (it is).
5. **Should any memory files be updated?** none.
