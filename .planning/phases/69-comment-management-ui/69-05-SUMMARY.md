---
phase: 69-comment-management-ui
plan: 05
subsystem: ui
tags: [svelte5, runes, review-panel, comment-management, center-pane, jump, orphan, tdd]

# Dependency graph
requires:
  - phase: 69-comment-management-ui
    plan: 02
    provides: "write commands add_commit_comment / edit_comment / delete_comment"
  - phase: 69-comment-management-ui
    plan: 03
    provides: "read commands list_session_comments / resolve_session_comments (CommentResolution + OrphanReason)"
  - phase: 69-comment-management-ui
    plan: 04
    provides: "TS v2 Comment DTO, OrphanReason/CommentResolution types, review-session.svelte.ts rune (rightPaneMode panel|diff, jumpTo + JumpDeps)"
provides:
  - "the real ReviewPanel.svelte: group-by-commit render (D-09), per-commit Add note (D-02), inline edit (D-10), delete-with-confirm (D-05), jump on resolvable comments (D-07), read-only orphan rows with reason badge (D-08)"
  - "DiffPanel.scrollToLine(start, end): hunk-targeted scroll + transient highlight reused for jump"
  - "RepoView center-pane Review-mode swap (panel <-> jumped-to diff) driven by the review-session rune, with a persistent accent Review toggle"
affects: [phase 69 verification, phase 70 render-by-line]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Center-pane single-pane swap gated on a rune's rightPaneMode (panel|diff), LOCKED to the center pane (UI-SPEC:133) — no new pane"
    - "Window-global menu event (review-toggle) kept at the App level and passed to only the active tab as a prop, so the global event never fans out across tabs"
    - "bind:this component method seam (DiffPanel.scrollToLine) wired into a rune's injected JumpDeps.scrollToRange, polled across frames to survive the mount race after the panel->diff swap"

key-files:
  created: []
  modified:
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts
    - src/components/DiffPanel.svelte
    - src/components/RepoView.svelte
    - src/components/RepoView.test.ts
    - src/App.svelte

key-decisions:
  - "Kept the OS-menu review-toggle flag in App.svelte and passed `reviewActive={reviewPanelOpen && tab.id === activeTabId}` to RepoView, instead of moving the listener into RepoView — a window-global event in every tab's RepoView would toggle all tabs at once."
  - "Comments on a commit no longer in the session (CommitGone) still render in a fallback group keyed by the oid with a '(commit gone)' header rather than being dropped — nothing the user wrote disappears (D-08)."
  - "scrollToLine targets the hunk whose new-side range covers the start line (reusing scrollToHunk's scrollIntoView + 600ms hunk-highlight) rather than a per-line DOM scroll, because line markup lives three components deep (HunkView/FullFileView/SplitView) and adding queryable line attributes across all three is out of plan scope."
  - "Single shared textarea primitive (draftText + draftValid = trim().length > 0) for both Add note and inline edit; the disabled Save IS the empty-text feedback (no toast), per the LOCKED copywriting contract."

requirements-completed: [ANCH-03, CMT-01, CMT-02, CMT-03, CMT-04]

# Metrics
duration: ~30min
completed: 2026-05-26
---

# Phase 69 Plan 05: Comment Management Panel + Center-Pane Wiring Summary

**Replaced the Phase 65 throwaway ReviewPanel stub with the real review panel and relocated it to the center pane: comments grouped by commit (D-09), per-commit "Add note" (D-02), inline edit (D-10), delete-with-confirm via the plugin-dialog `ask` (D-05), jump-to-anchor on resolvable comments (D-07), and read-only orphan rows with an OrphanReason reason badge (D-08). The review-session rune drives a single-pane swap (panel ↔ jumped-to diff) in the center pane, with a persistent accent "Review" toggle to return. Implementation tasks (1-2) are complete and `just check` is green; Task 3 is a human-verify checkpoint that is PENDING.**

## Performance

- **Duration:** ~30 min
- **Tasks:** 3 (Task 1 TDD RED→GREEN; Task 2 wiring; Task 3 human-verify checkpoint — PENDING)
- **Files modified:** 6

## Accomplishments

- **ReviewPanel.svelte (real panel):** reads `list_session_commits` / `list_session_comments` / `resolve_session_comments` on mount via `safeInvoke`; groups comments by commit (line-anchored under `anchor.commit_oid`, commit-level under `commit_oid`) in session order, with a fallback group for CommitGone comments. Per-commit "Add note" (MessageSquarePlus + 12px label) opens an inline composer → `add_commit_comment`. Inline "Edit" swaps text for a textarea with Save/Cancel → `edit_comment` by id. "Delete" → `ask("Delete this comment? This cannot be undone.", { title: "Delete comment", kind: "warning" })` → `delete_comment` by id on confirm, no-op on cancel. Orphans (resolvable:false) render read-only: jump removed, `--color-warning`-on-`--color-warning-bg` reason badge ("commit gone"/"file gone"/"line out of range"), anchor metadata dimmed via `--opacity-dimmed`, comment text + cached excerpt at full `--color-text`. Resolvable line-anchored comments get an accent-on-hover `CornerDownRight` jump (`aria-label="Jump to code"`).
- **DiffPanel.scrollToLine(start, end):** locates the hunk whose new-side line range covers `start` and reuses `scrollToHunk` (scrollIntoView + transient `hunk-highlight`); falls back to the first hunk so the file is at least brought into view.
- **RepoView wiring:** instantiates the `createReviewSession()` rune, syncs `reviewActive` into it, and gates the center pane on `rightPaneMode` — panel → ReviewPanel, diff → the jumped-to DiffPanel (`bind:this={diffPanelRef}`). `onJump` binds `rune.jumpTo` to `handleCommitSelect` / `handleCommitFileSelect` / a frame-polled `scrollToLine`. A persistent accent "Review" toggle returns to the panel.
- **App.svelte:** removed the thin-bar `<ReviewPanel>` render; keeps the global `review-toggle` flag and passes `reviewActive` only to the active tab.

## Task Commits

1. **Task 1 (RED): failing ReviewPanel tests** — `0e27f57` (test)
2. **Task 1 (GREEN): rewrite ReviewPanel** — `be7272a` (feat)
3. **Task 2 (part): DiffPanel.scrollToLine** — `0d7388a` (feat)
4. **Task 2: center-pane wiring (RepoView/App)** — `84acc54` (feat)
5. **Task 2 refinement: poll diffPanelRef before scroll** — `a26d514` (fix)
6. **Task 3: human-verify checkpoint** — APPROVED (`just dev` hands-on verification passed on the panel, add-note, edit, delete-confirm, jump, and orphan flows). Five post-checkpoint polish commits applied during verification:
   - `e18c3cf`: fix — render panel comments whose commit isn't in session.commits (template gated on `commits.length` instead of `groups.length`, hiding the fallback group; also dropped the misleading `(commit gone)` synthetic header — the per-comment orphan badge carries the truth).
   - `8630308`: fix — jump affordance moved from the curved-arrow icon to the file:line ref itself.
   - `56746a2`: feat — commit short-hash header is clickable; selects the commit and scrolls the graph to it (panel stays open).
   - `cf41f7e`: feat — GitHub-style comment cards with red/green diff backgrounds on `+`/`-` lines and a non-selectable gutter; edit mode now replaces only the body so the diff stays in view.
   - `4773374`: feat — commit-level comments sort before line-anchored within a group (stable sort).

## Files Created/Modified

- `src/components/ReviewPanel.svelte` — rewritten as the real panel (group-by-commit, add-note, inline edit, delete-confirm, jump vs orphan).
- `src/components/ReviewPanel.test.ts` — replaced the stub's lifecycle tests with 14 contract tests (grouping, empty states, add-note + empty-disable, inline edit + cancel + empty-disable, delete cancel/confirm by id, jump-vs-orphan + OrphanReason badge map).
- `src/components/DiffPanel.svelte` — added `scrollToLine`.
- `src/components/RepoView.svelte` — rune instantiation, center-pane gate, jump deps, Review toggle.
- `src/components/RepoView.test.ts` — added the new required `reviewActive` prop to the four render calls.
- `src/App.svelte` — removed the thin-bar panel render; pass `reviewActive` to the active tab.

## Decisions Made

- **App owns the review-toggle flag** (not RepoView): the menu event is window-global; per-tab listeners would toggle every tab. App gates it to the active tab via the prop.
- **CommitGone comments are kept** in a fallback group rather than dropped (D-08 — text is stored independent of resolvability).
- **Hunk-targeted scroll** for jump instead of per-line DOM scroll, because line markup is three components deep and broad data-attribute changes are out of scope.
- **Shared textarea primitive** for add-note and edit; disabled Save is the empty-text feedback.

## Deviations from Plan

None of Rules 1–4 triggered for the automated tasks; one refinement (`a26d514`) hardened the jump-scroll against a mount race the plan didn't anticipate (Rule 1 — preventing a silent no-op).

## Deferred Issues

- **`Source::FullFile` jump renders in whatever contentMode is active, not forced to full-file view.** The plan says jump swaps "diff/full-file per `Source`", but the Plan-04 `JumpDeps` interface (`selectCommit` / `selectFile` / `scrollToRange`) has **no `setContentMode` seam**, so RepoView cannot force DiffPanel's content mode from the rune without reopening the Plan-04 rune signature. For a `Source::FullFile` anchor the jump still selects the right commit + file and scrolls to the line range, but the view mode is whatever the user last set (the `Source::Diff` path is fully correct). Acknowledged at the human-verify checkpoint. Follow-up: extend `JumpDeps` with a content-mode seam.
- **No syntax highlighting in the comment card's diff hunk.** Lines render red/green by `+`/`-` prefix using the existing `--color-diff-{add,delete}-bg` tokens, but the code text itself is plain monospace. The project's live diff view uses syntect (Rust) to produce structured token spans; the panel doesn't have access to those tokens because cached_excerpt is a plain string. Adding a JS highlighter (Shiki/Prism/highlight.js) would create a second color source that drifts from the diff view's `--color-syn-*` tokens, so it was deliberately not bolted on. Resolution: a future phase that either re-tokenizes cached_excerpt via syntect through IPC, or stores structured spans alongside cached_excerpt in the schema.
- **Comment-only-from-diff sessions always land in a fallback group.** Phase 67's `add_comment` path auto-starts a session but never adds the commenting commit to `session.commits`. The new fallback rendering (e18c3cf) makes this navigable, but the natural "I'll just comment on this diff" flow doesn't pick up the real commit summary header. This is a Phase 66/67 UX question, not a Phase 69 bug — leaving for the user to decide whether to open a follow-up.

## Known Stubs

None. The panel is fully wired to the live backend reads/writes; no hardcoded/placeholder data paths remain.

## Threat Model Compliance

- **T-69-15 (bypassable/accidental delete):** delete is the only delete path and is gated by the plugin-dialog `ask` confirm; cancel writes nothing; vitest asserts cancel → no `delete_comment`, confirm → `delete_comment` by id.
- **T-69-16 (edit/delete hitting the wrong comment):** the panel targets by stable `id` (never row position); the session-changed listener reloads after external mutations.
- **T-69-17 (jump on an unresolvable anchor):** orphan rows disable jump and render a read-only badge; the rune's `jumpTo` early-returns on a null anchor — never navigates, never errors.
- **T-69-18 (inline-edit emit storms — accept):** editing is local component state until Save; only Save-time `edit_comment` emits.

---
*Phase: 69-comment-management-ui*
*Status: All 3 tasks complete (human-verify checkpoint approved); `just check` green at 517 tests*

## Self-Check: PASSED

- All modified files present on disk (ReviewPanel.svelte, ReviewPanel.test.ts, DiffPanel.svelte, RepoView.svelte, App.svelte, 69-05-SUMMARY.md).
- All task commits found in git history including the post-checkpoint polish: 0e27f57 (RED), be7272a (GREEN), 0d7388a, 84acc54, a26d514, e18c3cf, 8630308, 56746a2, cf41f7e, 4773374.
- `export function scrollToLine` present in DiffPanel.svelte; `onJumpToCommit` prop wired through RepoView's `handleReviewJumpToCommit` → `handleCommitSelect` + `commitGraphRef.scrollToOid`.
- `just check` exits 0 (fmt, biome, svelte-check 0 errors, clippy, cargo-test, vitest 517 passing). No inline color literals in ReviewPanel.svelte — diff backgrounds use `--color-diff-{add,delete}-bg`.
- TDD gate: `test(69-05)` (0e27f57) precedes `feat(69-05)` GREEN (be7272a).
- Human-verify (Task 3) approved after the five polish commits.
