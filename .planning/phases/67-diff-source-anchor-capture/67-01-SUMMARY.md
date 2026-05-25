---
phase: 67-diff-source-anchor-capture
plan: 01
subsystem: ui
tags: [diff, anchor, review-session, typescript, vitest, pure-transform]

# Dependency graph
requires:
  - phase: 65-review-session-keystone
    provides: Anchor/Source/Side/Comment TS+Rust schema (frozen 6-field Anchor)
provides:
  - "buildDiffAnchor: pure capture-time adapter mapping selected diff indices -> { anchor, cachedExcerpt }"
  - "Source-coordinate Anchor (commit_oid, file_path, source, side, start_line, end_line) with no array-index leakage"
  - "Diff-format cachedExcerpt assembled over the contiguous selection index span"
affects: [67-02-comment-composer, 68-fullfile-anchor-capture, review-rendering]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure transform lib + no-mock Vitest (mirrors merge-parser.ts)"
    - "Range-vs-excerpt divergence: range from side-filtered linenos, excerpt from contiguous index span"

key-files:
  created:
    - src/lib/diff-anchor.ts
    - src/lib/diff-anchor.test.ts
  modified: []

key-decisions:
  - "File status wins over selected origins for side resolution (Added/Renamed/Copied->New, Deleted->Old); Modified/Untracked/Unknown fall through to origin-based rule"
  - "Excerpt is built over the contiguous min..max INDEX span so dropped Delete lines and in-between context appear, even on a New-side range that excludes them"
  - "Anchor.source is hardcoded 'Diff' for this phase; FullFile source is Plan 68's concern"

patterns-established:
  - "Pure index->source adapter: no IPC, no mutation of inputs, two independent outputs computed and tested separately"

requirements-completed: [ANCH-01]

# Metrics
duration: 18min
completed: 2026-05-25
---

# Phase 67 Plan 01: Diff-Source Anchor Capture Adapter Summary

**Pure `buildDiffAnchor` TS adapter that turns a diff line selection into a 6-field source-coordinate Anchor plus a diff-format cachedExcerpt, with Delete lines dropped from the range but preserved in the excerpt.**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-25T16:53:00Z
- **Completed:** 2026-05-25T16:58:00Z
- **Tasks:** 1 (TDD feature: RED + GREEN)
- **Files modified:** 2 (both created)

## Accomplishments
- `buildDiffAnchor(commitOid, file, hunkIdx, selectedLineIndices)` — pure, side-effect-free adapter returning `{ anchor, cachedExcerpt }`.
- Side resolution where file status wins over selected origins (L-04), with origin-based fallback for Modified/Untracked/Unknown.
- Range computed from the chosen side's line numbers only, naturally dropping Delete linenos from a New-side range (L-03).
- `cachedExcerpt` assembled over the contiguous min..max index span with origin prefixes (`+`/`-`/space), so dropped `-` lines and in-between context survive (L-06).
- Anchor carries exactly the 6 frozen schema fields — no `hunk_index`/`line_index`/`context_lines`/`ignore_whitespace` leakage (L-01).
- 12 no-mock Vitest cases covering SC-1, SC-3, D-03, and L-01/04/06.

## Task Commits

Each step was committed atomically (TDD):

1. **RED: failing test for the adapter** - `0b7159b` (test)
2. **GREEN: implement buildDiffAnchor** - `4aeb0f3` (feat)
3. **Strengthen L-03 test discrimination + split origin cases** - `985c4fd` (test)

**Plan metadata:** `37a20e1` (docs: complete plan)

_No REFACTOR commit: the GREEN implementation was already clean (small private `resolveSide`/`prefixLine` helpers); a no-op refactor would have been kabuki. Commit 3 is a post-review test-strength improvement (see Issues Encountered), not a behavior change._

## Files Created/Modified
- `src/lib/diff-anchor.ts` - Pure capture-time adapter (`buildDiffAnchor` + private `resolveSide`/`prefixLine`); reuses `Anchor`/`Side`/`DiffLine`/`DiffStatus`/`FileDiff` from `types.ts`.
- `src/lib/diff-anchor.test.ts` - 11 no-mock Vitest cases with inline `DiffLine`/`FileDiff` fixtures.

## Decisions Made
- File status takes precedence over selected origins in `resolveSide` (per L-04); only Modified/Untracked/Unknown consult origins.
- Two independent outputs (range vs excerpt) computed and asserted separately — the central correctness property of the adapter (RESEARCH Pitfall 3).
- `anchor.source` hardcoded to `"Diff"`; deferred FullFile handling to Plan 68.

## Deviations from Plan

### Process deviations (non-code)

**1. [Rule 3 - Blocking] Installed worktree dependencies**
- **Found during:** RED verification run.
- **Issue:** The worktree's `node_modules` was empty, so vitest failed on a module-resolution error (`@testing-library/svelte/.../vitest.js`) instead of the intended "function missing" RED failure.
- **Fix:** Ran `bun install` in the worktree (project's package manager). `node_modules` is gitignored; no tracked files changed.
- **Verification:** After install, RED failed for the correct reason ("Failed to resolve import ./diff-anchor.js"); GREEN passed 11/11.
- **Committed in:** N/A (environment setup, no tracked-file change).

**2. [Rule 3 - Blocking] Adjusted the single-file test command**
- **Found during:** RED/GREEN verification.
- **Issue:** The plan's verify command `just vitest src/lib/diff-anchor.test.ts` does not pass the path through — the `vitest` recipe is `bun run test` (`vitest run`) with no arg passthrough, so it would run the whole suite.
- **Fix:** Used `bunx vitest run src/lib/diff-anchor.test.ts` for the per-step RED/GREEN runs; ran biome + svelte-check + vitest (frontend portion of `just check`) before the GREEN commit.
- **Verification:** Single-file run reported 11 passed.
- **Committed in:** N/A (tooling invocation only).

---

**Total deviations:** 2 process/tooling (both blocking, both non-code).
**Impact on plan:** No code-behavior deviations. The plan's algorithm was implemented exactly as specified. No scope creep.

## Issues Encountered
- Biome `ci` flagged import ordering in the test file (`import` value before `import type`); resolved with `biome check --write` per the TS coding-style rule. Folded into the GREEN commit since it was a formatting fix on the new code.
- Post-GREEN review found Test 3's mixed-selection fixture did not discriminate L-03: the Delete's `old_lineno` (16) was already inside the Add range (16..17), so a buggy `new_lineno ?? old_lineno` impl would still produce 16..17. Tightened by moving the Delete to `old_lineno=99` so the test would fail (16..99) under a buggy impl. Also split the combined Untracked/Unknown case into one behavior per test. Committed as `985c4fd`.

## Verification Results
- `bunx vitest run src/lib/diff-anchor.test.ts` → 12 passed.
- `bunx biome ci src/lib/diff-anchor.ts src/lib/diff-anchor.test.ts` → clean.
- `bun run check` (svelte-check) → 0 errors, 0 warnings.
- L-01 source assertion (`grep -E "hunk_index|line_index|context_lines|ignore_whitespace" ... | grep -v '//' | wc -l`) → 0.
- Type-redeclare assertion (`grep -c "interface Anchor\|type Anchor\|type Side =\|type Source =" src/lib/diff-anchor.ts`) → 0.

## Known Stubs
None. The adapter is fully implemented; `source: "Diff"` is intentional for this phase (FullFile is Plan 68), not a stub.

## Continuous Improvement — Post-Task Reflection
1. **What was harder than expected?** The empty worktree `node_modules` masked the intended RED failure as a setup error; needed `bun install` before TDD could proceed cleanly.
2. **Was anything done twice?** The RED vitest run was repeated (once pre-install failing on setup, once post-install failing for the right reason). Eliminated next time by installing deps as the first worktree action.
3. **Did I make any incorrect assumptions?** Assumed `just vitest <path>` would scope to one file — it does not pass args. Documented in deviations.
4. **Is there a follow-up improvement?** Friction: parallel-executor worktrees start with empty `node_modules`. Root cause: worktree creation does not seed/symlink deps. Fix: the execute-phase orchestrator could `bun install` (or symlink `node_modules`) at worktree spawn. Benefit: every TDD/frontend plan avoids a false-failure RED run. Cost: low; one install step at spawn, ~0.5s with bun cache.
5. **Should any memory files be updated?** Candidate: note that GSD worktree executors should `bun install` first when the plan touches frontend tests, and that `just vitest` takes no path arg (use `bunx vitest run <path>`). Surfacing here rather than silently editing memory.

## Next Phase Readiness
- `buildDiffAnchor` is ready to be imported by Plan 02's comment composer to build the `add_comment` payload.
- The anchor already carries `source`/`side`, keeping the Rust `add_comment` writer dumb and Phase-68-shareable.
- No blockers.

## Self-Check: PASSED
- FOUND: src/lib/diff-anchor.ts
- FOUND: src/lib/diff-anchor.test.ts
- FOUND: .planning/phases/67-diff-source-anchor-capture/67-01-SUMMARY.md
- FOUND commit: 0b7159b (RED)
- FOUND commit: 4aeb0f3 (GREEN)

---
*Phase: 67-diff-source-anchor-capture*
*Completed: 2026-05-25*
