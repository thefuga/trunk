---
phase: 67-diff-source-anchor-capture
plan: 04
subsystem: ui
tags: [svelte5, diff, review, comment-capture, tauri, vitest]

# Dependency graph
requires:
  - phase: 67-01
    provides: "buildDiffAnchor pure adapter (selection -> { anchor, cachedExcerpt })"
  - phase: 67-02
    provides: "add_comment / save_draft_comment dumb-writer Tauri commands (flat camelCase wire args)"
  - phase: 67-03
    provides: "commitDetail threaded to commit DiffPanel sites; commit-diff line selection enabled; request DTOs"
  - phase: 65-review-session-keystone
    provides: "review session lifecycle commands (get_review_session_status / start / resume) + frozen Anchor schema"
provides:
  - "CommentComposer.svelte: inline composer with N-M range preview, debounced draft persistence, add_comment submit, confirmDiscardIfDirty() instance method, empty-text submit disable"
  - "Comment affordance in HunkView + SplitView (diffKind==='commit' branch), disabled-with-tooltip only on merge commits (D-04)"
  - "DiffPanel host: commitOid/isMerge derivation, composer open/draft state, D-02 confirm-on-switch, auto-start of a review session at the comment chokepoint"
affects: [68-full-file-anchor, 69-review-panel-comments, 70-comment-render]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Svelte 5 instance method export (export function confirmDiscardIfDirty) consumed by both the host (bind:this) and component tests, mirroring VirtualList.scroll"
    - "Capture-chokepoint session bootstrap: ensure an active review session at composer-open, keeping add_comment/save_draft_comment dumb writers (L-08)"
    - "Handler-up / composer-down split: the Comment affordance signals oncommentlines upward; DiffPanel hosts the composer and selection state"

key-files:
  created:
    - src/components/diff/CommentComposer.svelte
    - src/components/diff/CommentComposer.test.ts
  modified:
    - src/components/diff/HunkView.svelte
    - src/components/diff/SplitView.svelte
    - src/components/diff/DiffViewer.svelte
    - src/components/DiffPanel.svelte
    - src/components/DiffPanel.test.ts

key-decisions:
  - "Composer hosted in DiffPanel (flex panel below DiffViewer), not injected into the hunk viewport — avoids positioning hacks (CLAUDE.md)"
  - "confirmDiscardIfDirty exposed as a Svelte 5 instance method (export function) so both DiffPanel and tests call it via bind:this"
  - "Auto-start a review session at the comment chokepoint when none is active (UAT decision); the Tauri write commands stay dumb writers"
  - "Comment affordance stays enabled with no session (auto-start handles it); the only disabled case is merge commits (D-04)"
  - "Success feedback kept silent (no toast) per user + project convention — confirmation relies on the existing session-changed -> panel reload"

patterns-established:
  - "Pure adapter -> reactive $derived anchor: the composer derives both the N-M preview and the submit payload from buildDiffAnchor, single source of truth"
  - "Debounced (~300ms idle) draft persistence via save_draft_comment; submit cancels the pending timer and persists via add_comment"

requirements-completed: [ANCH-01]

# Metrics
duration: ~55min (incl. human-verify wait); ~25min active execution
completed: 2026-05-25
---

# Phase 67 Plan 04: Inline Comment Composer + Comment Affordance Summary

**Inline comment composer for commit-diff line selections — N-M range preview from buildDiffAnchor, debounced draft persistence, add_comment submit, D-02 confirm-on-switch, merge-commit-only disable (D-04), and auto-start of a review session at the comment chokepoint so the dumb-writer commands never hit no_session.**

## Performance

- **Duration:** ~55 min wall (includes the human-verify checkpoint wait); ~25 min active execution
- **Started:** 2026-05-25T15:21:43Z
- **Completed:** 2026-05-25T16:14:35Z
- **Tasks:** 2 code tasks + 1 human-verify checkpoint (UAT)
- **Files modified:** 7 (2 created, 5 modified)

## Accomplishments
- `CommentComposer.svelte` — inline composer: reactive "Comments on lines N-M" preview (D-03), debounced (~300ms) `save_draft_comment` on input (L-05), `add_comment` submit with `{ anchor, cachedExcerpt }` and clear-on-success, empty-text submit disable, and a `confirmDiscardIfDirty()` instance method that prompts only when the draft is dirty (D-02). User text rendered via textarea only — no innerHTML (T-67-04). All colors via `--color-*` vars.
- Comment affordance mounted in both layouts: a `diffKind==='commit'` branch in HunkView and SplitView (right/new-side context), `hasSelection`-gated, disabled-with-tooltip ONLY on merge commits (D-04). File-status side constraints (L-04) never disable it.
- DiffPanel hosts the composer: derives `commitOid`/`isMerge` from `commitDetail`, opens the composer for a specific (file, hunk) on Comment click, gates selection-switch on `confirmDiscardIfDirty()`, and clears selection + closes on submit.
- Auto-start of a review session at the comment chokepoint (UAT fix) so commenting with no active session no longer dead-ends on `no_session`.
- 9 new component tests (6 CommentComposer + 3 DiffPanel auto-start) plus 4 DiffPanel affordance tests; full `just check` green (494 frontend tests).

## Task Commits

1. **Task 1 (TDD): CommentComposer.svelte + tests**
   - RED — failing component tests: `f57d940` (test)
   - GREEN — implement the composer: `feb81ee` (feat)
   _No REFACTOR commit: the GREEN implementation was already clean._
2. **Task 2: mount the Comment affordance + host the composer**: `56c2c1f` (feat)
3. **Task 3 (human-verify checkpoint): pause for UAT**: `1739e4d` (docs — STATE pause)
   - **UAT fix (Rule 1): auto-start review session when commenting with no active session**: `101e42f` (feat)

**Plan metadata:** committed with this SUMMARY + STATE/ROADMAP/REQUIREMENTS (docs: complete plan).

## Files Created/Modified
- `src/components/diff/CommentComposer.svelte` — inline composer (preview, debounced draft, submit, confirmDiscardIfDirty, empty-disable).
- `src/components/diff/CommentComposer.test.ts` — 6 component tests (preview, empty-disable, draft-on-keystroke, submit+clear, confirm-on-discard gating, split-never-Old).
- `src/components/diff/HunkView.svelte` — `diffKind==='commit'` Comment-affordance branch + `isMerge`/`oncommentlines` props; staging branches untouched.
- `src/components/diff/SplitView.svelte` — equivalent commit branch (right/new column) + `.accent-btn` class; staging branches untouched.
- `src/components/diff/DiffViewer.svelte` — transparent pass-through of `isMerge`/`oncommentlines` to HunkView + SplitView.
- `src/components/DiffPanel.svelte` — composer host state, `commitOid`/`isMerge` derivation, `handleCommentLines` + `ensureActiveSession`, D-02 confirm in `handleLineClick`, composer render.
- `src/components/DiffPanel.test.ts` — 4 affordance tests (merge-disable/tooltip, added-enabled, non-merge-enabled, confirm-on-switch) + 3 auto-start tests (none→start, resume-available→resume, active→no-op); command-aware safeInvoke mock.

## Decisions Made
- Composer hosted in DiffPanel as a flex panel below DiffViewer (no viewport injection / positioning hacks).
- `confirmDiscardIfDirty` as a Svelte 5 `export function` instance method, reached via `bind:this` from both DiffPanel and tests.
- Auto-start a session at the comment chokepoint rather than disabling the affordance when no session exists (UAT product-owner call).
- Success feedback kept silent per user + project convention; comment display deferred to Phase 69.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug, user-directed] Added frontend auto-start of a review session at the comment chokepoint**
- **Found during:** Task 3 (human-verify UAT)
- **Issue:** With no active review session, clicking Comment opened the composer, but typing (`save_draft_comment`) and Submit (`add_comment`) both failed with the backend `no_session` error (review.rs:315) because both go through `mutate_session_rmw`. The affordance was gated only on `selectedCount`, not on session presence — a dead-end.
- **Fix:** `DiffPanel.ensureActiveSession()` runs at the Comment-click chokepoint before the composer mounts: `get_review_session_status { path }` → `active` no-op / `resume-available` → `resume_review_session` (no clobber) / `none` → `start_review_session`; on any error, `showToast` and abort opening the composer. The Comment affordance stays ENABLED with no session (auto-start handles it); the only disabled case remains merge commits (D-04). `add_comment` / `save_draft_comment` remain dumb writers (L-08) — session creation is frontend-only. Success feedback kept silent per user + project convention; comment display deferred to Phase 69 as scoped.
- **Files modified:** src/components/DiffPanel.svelte, src/components/DiffPanel.test.ts, src/components/diff/CommentComposer.svelte (biome format only)
- **Verification:** 3 new DiffPanel tests (none→start then add_comment succeeds; resume-available→resume not start; active→neither); full `just check` green (494 frontend tests). Human UAT confirmed comments persist on disk with correct 6-field anchors.
- **Committed in:** `101e42f`

---

**Total deviations:** 1 (Rule 1 — user-directed scope addition surfaced during UAT).
**Impact on plan:** The auto-start is required for the capture flow to function at all; without it the composer wrote into a backend that returns `no_session`. The Tauri write commands and the anchor adapter are unchanged. No scope creep beyond the product-owner-approved session bootstrap; comment display correctly stays in Phase 69.

## Issues Encountered
- Two test-timing issues during test authoring (not code bugs): `handleLineClick`'s async dynamic plugin-dialog import and `handleCommentLines`'s async session-ensure both needed a microtask flush (`setTimeout(0)`) before assertions. Resolved with explicit flushes.
- Mock-state leakage in the auto-start tests: `safeInvoke.mock.calls` accumulates across tests in a file (no global `clearMocks`); fixed by `mockClear()` at the start of each auto-start test.
- Biome formats `.svelte` script blocks via `biome ci .` — CommentComposer.svelte (committed in the prior session before a full `just check`) needed a `$props()` destructure reformat; applied via `biome check --write` and folded into the UAT-fix commit.

## User Setup Required
None - no external service configuration required.

## Known Stubs
None. The composer is fully wired (draft + submit). Comment display/list (badges, viewer) is intentionally Phase 69 — not a stub of this plan's goal. `Anchor.source` is `"Diff"` for this phase (FullFile is Phase 68).

## Next Phase Readiness
- Phase 68 (full-file anchor) reuses `add_comment` verbatim (L-08 dumb-writer contract locked in Plan 02).
- Phase 69 (review panel comments) renders the persisted comments captured here; the on-disk anchors carry the frozen 6 fields.
- No blockers.

## Self-Check: PASSED
- FOUND: src/components/diff/CommentComposer.svelte
- FOUND: src/components/diff/CommentComposer.test.ts
- FOUND: .planning/phases/67-diff-source-anchor-capture/67-04-SUMMARY.md
- FOUND commit: f57d940 (RED)
- FOUND commit: feb81ee (GREEN)
- FOUND commit: 56c2c1f (Task 2)
- FOUND commit: 101e42f (UAT fix)

---
*Phase: 67-diff-source-anchor-capture*
*Completed: 2026-05-25*
