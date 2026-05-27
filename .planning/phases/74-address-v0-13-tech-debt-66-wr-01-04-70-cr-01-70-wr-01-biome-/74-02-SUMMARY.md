---
phase: 74-address-v0-13-tech-debt-66-wr-01-04-70-cr-01-70-wr-01-biome
plan: 02
subsystem: diff-comments
tags: [biome, lint, refactor, tech-debt, svelte]
dependency_graph:
  requires: []
  provides:
    - "deriveDiffCapture runtime-guard helper in CommentComposer.svelte"
  affects:
    - "src/components/diff/CommentComposer.svelte"
tech_stack:
  added: []
  patterns:
    - "explicit runtime guard helper as documentation-by-throw (replaces non-null assertions for caller contracts that biome cannot statically infer)"
key_files:
  created: []
  modified:
    - "src/components/diff/CommentComposer.svelte (+17/-5)"
decisions:
  - "Adopted RESEARCH §7 Option B (runtime guard helper) over Option C (biome-ignore) to preserve the project's zero-suppression precedent; Option A (discriminated-union Props split) deferred as not worth the DiffPanel call-site churn for three warnings"
metrics:
  duration_minutes: 2
  completed_date: "2026-05-27"
  warnings_removed: 3
  tests_added: 0
  tests_passing: 9
---

# Phase 74 Plan 02: Replace CommentComposer non-null assertions with deriveDiffCapture guard — Summary

Extracted `CommentComposer.svelte:43` `buildDiffAnchor(commitOid, file!, hunkIdx!, selectedLineIndices!)` into a `deriveDiffCapture()` helper that throws on contract violation, eliminating 3× `noNonNullAssertion` biome warnings without introducing any `biome-ignore` directives (project still has zero, verified by grep).

## What changed

Replaced lines 36–44 of `src/components/diff/CommentComposer.svelte`:

- Removed: three `!` non-null assertions on `file`, `hunkIdx`, `selectedLineIndices` inside the `$derived(...)` block.
- Added: `function deriveDiffCapture(): { anchor: Anchor; cachedExcerpt: string }` with a single `if (file === undefined || hunkIdx === undefined || selectedLineIndices === undefined) throw new Error(...)` guard, then returns `buildDiffAnchor(commitOid, file, hunkIdx, selectedLineIndices)`.
- Updated `capturedResult` to `$derived(captured ?? deriveDiffCapture())`.
- Updated the explanatory comment to drop the now-misleading "non-null assertions document that contract" phrasing and reference the DiffPanel.svelte guard explicitly.

The throw branch is unreachable in practice — `DiffPanel.svelte:625` guards `composerOpen && composerFile && composerHunkIdx !== null` before mounting — but it makes the contract explicit at the type level rather than smuggling it through `!`.

## Verification

| Check                                                                  | Result |
| ---------------------------------------------------------------------- | ------ |
| `bunx biome ci src/components/diff/CommentComposer.svelte` warnings    | 3 → 0  |
| `grep -c biome-ignore src/components/diff/CommentComposer.svelte`      | 0      |
| `grep -rn biome-ignore src src-tauri/src` (project-wide)               | 0      |
| `grep -c "function deriveDiffCapture" CommentComposer.svelte`          | 1      |
| `bunx vitest run src/components/diff/CommentComposer.test.ts`          | 9/9 pass (unchanged) |
| `just check`                                                            | exit 0 (49 vitest files / 552 tests; full Rust suite green) |

All `must_haves.truths` from the plan frontmatter satisfied.

## TDD posture (deviation note)

Plan frontmatter marked the task `tdd="true"`, but no new test was added. Reasoning:

- This is a pure lint refactor — the observable behavior of both contracts is unchanged.
- The throw branch is dead in practice (caller contract enforced by `DiffPanel.svelte:625`); RESEARCH §7 explicitly noted "the assertion-throw branch is unreachable in practice but harmless if hit. No new test needed unless Option B's runtime guard is treated as a real invariant."
- Both contracts (diff-path + full-file-path) already have coverage in `CommentComposer.test.ts` (9 tests, all green pre- and post-change).
- Writing a synthetic test that constructs the dead branch (calling the component with `captured: undefined` and all three diff-path props undefined) would be coverage theater per GOOS classical bias: it tests the helper's internal guard, not observable behavior.

Tracked as a single deviation: TDD-default behavior was deliberately skipped per the testing rules' "behavior, not implementation" principle and RESEARCH guidance. Existing tests' continued green status is the contract preservation signal.

## Deviations from Plan

### Skipped step

**1. [Deviation — TDD] No new test added for the throw branch**
- **Found during:** Task 1
- **Issue:** Plan was tagged `tdd="true"` but the change is a refactor with no behavior delta and a deliberately dead defensive throw.
- **Resolution:** Relied on existing `CommentComposer.test.ts` (9 tests, both contracts) to confirm behavior preservation. RESEARCH §7 anticipated this.
- **Files modified:** none beyond the planned `src/components/diff/CommentComposer.svelte`
- **Commit:** 397a82e

## Known Stubs

None.

## Threat Flags

None — pure lint refactor; no new attack surface, no IPC/parsing/auth changes.

## Commits

- `397a82e` refactor(74-02): replace non-null assertions with explicit deriveDiffCapture guard

## Self-Check: PASSED

- File `src/components/diff/CommentComposer.svelte` exists with `function deriveDiffCapture` (grep -c = 1).
- Commit `397a82e` present in `git log` (verified).
- Zero `biome-ignore` directives in the file or project-wide.
- `just check` exit code 0.

## Post-task reflection

1. **What was harder than expected?** Nothing — the change was as scoped. RESEARCH §7 gave the exact Option B sketch.
2. **Was anything done twice?** No.
3. **Did I make any incorrect assumptions?** None — verified baseline warning count (3), confirmed zero project-wide biome-ignore precedent, ran the full gate before committing.
4. **Is there a follow-up improvement?** None small enough to inline. The remaining tech-debt warnings (66/WR-01..04, 70/CR-01) are tracked in Plans 74-03..74-06.
5. **Should any memory files be updated?** No — the "explicit runtime guard over non-null assertion" pattern is already encoded in `coding_style.md` ("Don't defend against your own code… when you DO need to guard, make it an explicit branch").
