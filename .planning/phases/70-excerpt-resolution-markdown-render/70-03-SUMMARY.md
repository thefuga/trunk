---
phase: 70-excerpt-resolution-markdown-render
plan: 03
subsystem: ui
tags: [svelte5, frontend, review-panel, ipc, tdd]

# Dependency graph
requires:
  - phase: 70-01
    provides: pub fn render(session, repo) -> String (pure markdown renderer)
  - phase: 70-02
    provides: Tauri command generate_review_doc(path) -> Result<String, String>
  - phase: 69-comment-management-ui
    provides: ReviewPanel host, review-session.svelte.ts rune, RepoView wiring
provides:
  - Generate button in the ReviewPanel header (D-01) — disabled until comments.length >= 1
  - Panel-internal swap between comment-list and markdown-preview faces (D-02)
  - ReviewDocPreview.svelte — small standalone preview component with a Phase-71 docking slot
  - review-session rune extended with panelMode + previewMarkdown + showList / showPreview / generate
affects:
  - 71-copy-save (Copy and Save buttons will dock to the .preview-spacer slot in ReviewDocPreview)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Rune-as-prop: the review-session manager is held by RepoView and threaded into ReviewPanel as a typed `session: ReviewSessionManager` prop. Avoids duplicating $state, keeps panel-internal swaps coordinated with the rune's existing rightPaneMode dimension."
    - "Panel-internal view-swap: ReviewPanel branches on session.state.panelMode at the top level. The preview replaces the entire list block (not coexists); the cached previewMarkdown survives swaps so re-Generate is the only invalidation path."
    - "IPC via rune action: ReviewPanel.onGenerateClick wraps `session.generate(repoPath)` in try/catch. The rune owns the safeInvoke call and state mutation; the panel owns the toast surfacing — single-responsibility on each side."
    - "Forward-compatible header slot: ReviewDocPreview ships a load-bearing `.preview-spacer` flex cell between Back and the right edge. Phase 71's Copy/Save buttons dock there without layout churn."

key-files:
  created:
    - src/components/ReviewDocPreview.svelte
    - src/lib/review-session.svelte.test.ts
    - .planning/phases/70-excerpt-resolution-markdown-render/70-03-SUMMARY.md
  modified:
    - src/lib/review-session.svelte.ts (panelMode + previewMarkdown + showList/showPreview/generate)
    - src/components/ReviewPanel.svelte (Generate header button + preview-swap branch + onGenerateClick + session prop)
    - src/components/RepoView.svelte (pass reviewSession into ReviewPanel)
    - src/components/ReviewPanel.test.ts (extend installReads with generateDoc; add three Phase-70 tests; thread session prop into existing render calls)

key-decisions:
  - "Pass the entire ReviewSessionManager into ReviewPanel as a single `session` prop (not destructured pieces). Keeps the rune's state-mutation surface coupled to the methods that drive it, mirrors how RepoView already holds the manager. Forced an incidental contract change: every existing render() call site in ReviewPanel.test.ts now passes `session: createReviewSession()`."
  - "Treat the no_comments TrunkError catch branch as a defensive backstop, not the primary D-11 gate. The button's `disabled={!hasAnyComment}` is the UX gate; the Rust command's `no_comments` error reaches the catch only via race (a session emptied between render and click) and is surfaced via showToast. Pattern lifted from the existing saveAddNote err-cast handler verbatim."
  - "Rune test file is `review-session.svelte.test.ts` (matches the project's `*.svelte.ts ↔ *.svelte.test.ts` convention — see remote-state.svelte.test.ts). Vitest's `include: ['src/**/*.test.ts']` glob matches it correctly."
  - "Mock `@tauri-apps/api/core::invoke` (the underlying primitive), not `safeInvoke` directly, in the rune test. Keeps the TrunkError-parsing path live in the test — same shape as src/lib/invoke.test.ts:5-9."
  - "Generate button title attribute is the ONLY tooltip surface. The plan locked the disabled tooltip text 'Add at least one comment to generate'; HTML's native `title=` is the standard widget and matches the existing project pattern (no custom tooltip widget anywhere in src/components/)."

patterns-established:
  - "Rune-as-prop: when a panel needs both to read rune state and to drive rune actions, pass the entire manager as a typed prop (ReviewSessionManager). Future panels mounted under the same rune (e.g. a hypothetical ReviewToolbar) follow the same shape."
  - "Test the rune through the panel: ReviewPanel.test.ts's three new Phase-70 cases construct a real createReviewSession() and exercise it through the panel. Only safeInvoke is mocked. The rune's own contract is covered separately in review-session.svelte.test.ts."

requirements-completed: [DOC-01]

# Metrics
duration: ~35min
completed: 2026-05-26
---

# Phase 70 Plan 03: ReviewPanel Generate + Preview Summary

**Wires the Phase 70-02 IPC command into the Phase 69 ReviewPanel: a Generate button in the panel header (disabled until comments.length >= 1) clicks through `safeInvoke<string>('generate_review_doc', { path })` and swaps the panel to a `ReviewDocPreview.svelte` view rendering the returned markdown inside a `<pre>` with `white-space: pre` (no soft wrap — the user reviews exactly what the AI agent will see). Back-to-comments returns to the list view; the cached previewMarkdown is preserved across swaps. After this plan, a user can: open a repo, start a review session, add at least one comment, click Generate, and read the rendered markdown end-to-end — DOC-01 reachable.**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-05-26
- **Completed:** 2026-05-26
- **Tasks:** 3 (one TDD: RED + GREEN; two non-TDD)
- **Files modified:** 4 source + 1 new test file + 1 SUMMARY
- **Lines added:** ~280 (rune extension ~50, preview component ~80, ReviewPanel changes ~80, tests ~95)
- **Test count delta:** 517 → 527 (+7 rune tests, +3 ReviewPanel tests)

## Accomplishments

- `review-session.svelte.ts` rune extended with `panelMode: 'list' | 'preview'`, `previewMarkdown: string | null`, and three actions: `showList()`, `showPreview(md)`, async `generate(repoPath)`. Rejection from `generate` propagates with no partial state update.
- `setReviewActive(false)` clears `previewMarkdown` and resets `panelMode` to `'list'` (the cached doc belongs to the just-ended session). `setReviewActive(true)` does NOT touch preview fields.
- New `src/components/ReviewDocPreview.svelte`: flex column with a header (Back affordance + `.preview-spacer` slot for Phase 71) and a `<pre>` body. `white-space: pre` so long lines do not wrap; horizontal scroll instead. All colors flow through `var(--color-*)`; zero inline hex/rgb literals.
- `ReviewPanel.svelte` gains a panel-level header with a Generate button (FileText icon). Disabled until `comments.length >= 1`. The `title="Add at least one comment to generate"` carries the LOCKED tooltip text from D-01. Click handler `onGenerateClick` wraps `session.generate(repoPath)` in try/catch; the no_comments race is surfaced via showToast.
- ReviewPanel branches at top-level on `session.state.panelMode === 'preview' && session.state.previewMarkdown !== null` to render `<ReviewDocPreview />` with `onBack=() => session.showList()`. The preview replaces (not coexists with) the list block; the cached previewMarkdown is preserved across swaps.
- RepoView wires its existing `reviewSession` instance into ReviewPanel via the new `session` prop.
- 6 vitest cases for the rune's preview state (default values, transitions, cache preservation, IPC integration, reset on deactivate, rejection propagation) — `src/lib/review-session.svelte.test.ts`.
- 3 vitest cases for the ReviewPanel UI (Generate disabled when no comments + LOCKED tooltip; click invokes IPC and swaps to preview; Back returns to list view) added to `src/components/ReviewPanel.test.ts`. The 18 existing tests stay green after threading the new required `session` prop.

## Task Commits

1. **Task 1 (TDD RED):** `bc5d547` — `test(70-03): add failing tests for review-session preview state`
2. **Task 1 (TDD GREEN):** `66a0810` — `feat(70-03): add panelMode + previewMarkdown + generate action to review-session rune`
3. **Task 2:** `f306309` — `feat(70-03): add Generate button + ReviewDocPreview panel-internal swap`
4. **Task 3:** `8a022fe` — `test(70-03): cover Generate button disabled state, click->preview swap, back-to-list`

No REFACTOR commit was warranted on Task 1 — the GREEN implementation was already shaped per the plan's `<interfaces>` and existing patterns. No further cleanup pass would have changed anything observable.

## Files Created/Modified

- `src/lib/review-session.svelte.ts` — Added PanelMode type, extended ReviewSessionState (panelMode + previewMarkdown), extended ReviewSessionManager (showList / showPreview / generate), added `setReviewActive(false)` reset behavior. Imported safeInvoke.
- `src/lib/review-session.svelte.test.ts` — NEW. 7 cases covering defaults, swap transitions, cache preservation, generate IPC integration, deactivation reset, no-touch-on-reactivate, rejection propagation.
- `src/components/ReviewDocPreview.svelte` — NEW. Standalone preview component (markdown + onBack props). Phase 71 docking slot kept as `.preview-spacer`.
- `src/components/ReviewPanel.svelte` — Added FileText + ReviewDocPreview + ReviewSessionManager imports; added `session` prop (required, ReviewSessionManager); added onGenerateClick handler; added panel-level header with Generate button (disabled gate + LOCKED title tooltip); wrapped existing list rendering in a top-level branch on session.state.panelMode === 'preview'; added .generate-button styles.
- `src/components/RepoView.svelte` — One-line edit: thread `session={reviewSession}` into the `<ReviewPanel />` call site.
- `src/components/ReviewPanel.test.ts` — Extended `installReads` with `generateDoc?: string` + the generate_review_doc switch case (default fallback `'# stub\n'`); added 3 Phase-70 cases under a new `describe('Generate / preview')`; threaded `session: createReviewSession()` into all existing `render()` call sites.

## Decisions Made

- **Rune-as-prop over destructured pieces.** Passing the full `ReviewSessionManager` keeps the read-state + drive-actions coupling intact and matches the way RepoView already holds the manager. Verified by advisor before commit.
- **Mock `invoke`, not `safeInvoke`, in the rune test.** Preserves the TrunkError-parsing path live in the test (same pattern as `src/lib/invoke.test.ts:5-9`).
- **`title` attribute is the LOCKED-tooltip widget.** No custom tooltip widget exists in `src/components/`; HTML's native `title=` is the project pattern.
- **No REFACTOR commit on the rune.** The GREEN code matched the plan's interface verbatim; no observable change would have come from a refactor pass. TDD cycle: RED → GREEN, stop.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Worktree `node_modules` empty on spawn; `npm install` required**

- **Found during:** Initial environment bootstrap (before Task 1)
- **Issue:** Same condition Phase 70-01 and 70-02 SUMMARYs documented. Claude Code worktrees do not inherit the parent's `node_modules`; `npx vitest` fails on first run with "Cannot find module '@testing-library/svelte/...'".
- **Fix:** Ran `npm install` once; the install touched `package-lock.json` as a registry side-effect; restored via `git checkout -- package-lock.json` (no actual lockfile drift, only the install's transient metadata).
- **Files modified:** None permanently. `node_modules/` is gitignored; `package-lock.json` restored.
- **Verification:** `just check` exits 0 after the install.
- **Committed in:** N/A — environment-bootstrap step.

**2. [Rule 3 — Blocking] First `Write` call landed in the main repo, not the worktree (worktree-path-safety #3099)**

- **Found during:** Task 1 RED — initial Write of `src/lib/review-session.svelte.test.ts` reported success but `ls` against the worktree path returned "No such file or directory."
- **Issue:** The Write tool's absolute-path resolution went through the orchestrator's cwd rather than the worktree's cwd, so the file landed at `/Users/joaofnds/code/trunk/src/lib/review-session.svelte.test.ts` (main repo) instead of the worktree path. Documented landmine in `references/worktree-path-safety.md`.
- **Fix:** `rm` the errant file from the main repo, then Write the file again using the fully-qualified worktree-prefixed absolute path (`/Users/joaofnds/code/trunk/.claude/worktrees/agent-aa0e341d26a70916f/...`).
- **Files modified:** None permanently. The errant file was never committed.
- **Verification:** Subsequent `ls` against the worktree path succeeded; `npx vitest run review-session` discovered the file.
- **Committed in:** N/A — pre-commit cleanup.

**3. [Rule 3 — Blocking] `biome` reformatted multi-line `props: { ... }` objects after the test edits**

- **Found during:** `just check` after Tasks 2 and 3
- **Issue:** Hand-written multi-line `render(ReviewPanel, { props: { repoPath: "/repo", session: createReviewSession(), onJump: vi.fn(), onJumpToCommit: vi.fn() } })` blocks did not match biome's preferred indentation (extra tab inside object literals nested in args). `just check` exited 1 with a "biome" recipe failure showing the diff.
- **Fix:** Ran `npx biome check --write src/components/ReviewPanel.test.ts` after each set of edits. Formatting-only change.
- **Files modified:** `src/components/ReviewPanel.test.ts` (formatting only).
- **Verification:** Subsequent `just check` exits 0; the 21 ReviewPanel tests stay green.
- **Committed in:** `f306309` (Task 2) and `8a022fe` (Task 3) — formatting subsumed into the same commits.

**4. [Rule 3 — Blocking] mise.toml in the worktree was not trusted**

- **Found during:** First `just check` invocation
- **Issue:** `just check` aborts with "Config files in ~/.../mise.toml are not trusted." This is a per-worktree trust ledger; the parent's trust does not propagate.
- **Fix:** Ran `mise trust` once at the worktree root.
- **Files modified:** None (mise stores trust in `~/.local/state/mise/`).
- **Verification:** Subsequent `just check` runs proceed past the mise gate.
- **Committed in:** N/A — environment-bootstrap step.

---

**Total deviations:** 4 auto-fixed (all Rule 3 blocking environmental / tooling gaps). No scope creep. The plan was followed task-by-task with no architectural deviations.

**Impact on plan:** None on observable behavior. Three of the four (1, 2, 4) are recurring worktree-bootstrap conditions that 70-01 and 70-02 also hit — worth a one-time bootstrap hook in the Claude Code worktree spawn path if this pattern persists. The fourth (biome formatting) is a routine post-edit auto-fix.

## Issues Encountered

- **Test file path quirk (#3099 landmine).** Documented above as Deviation 2. The fix is mechanical (use the fully-qualified worktree-prefixed path the first time), but worth surfacing for future executors: when Write seems to silently succeed but the file is missing, check `find /Users/joaofnds/code/trunk -name 'filename' 2>/dev/null` — the file likely landed in the main repo. The harness's "file state is current in your context — no need to Read it back" message can mask the redirection.
- **Plan / current code drift.** Plan 70-03's `<interfaces>` claims `review-session.svelte.ts` is "presumably already imported and used in [ReviewPanel.svelte]"; in reality, the rune is held by `RepoView.svelte` and ReviewPanel did not reference it at all. Surfaced the gap via an advisor call before writing Task 2; resolved by adding a required `session: ReviewSessionManager` prop and threading it from RepoView. Resolved cleanly; surfacing here as planner feedback for the next phase.

## Known Stubs

None. The Generate flow is fully wired: button → onGenerateClick → session.generate(repoPath) → safeInvoke<string>("generate_review_doc", { path }) → state.previewMarkdown = result → ReviewDocPreview renders the markdown. Phase 71's Copy/Save buttons have a docking slot (`.preview-spacer`) ready in ReviewDocPreview's header.

## TDD Gate Compliance

Plan frontmatter declares `tdd="true"` on Task 1 only. RED commit (`bc5d547` — `test(70-03):`) precedes GREEN commit (`66a0810` — `feat(70-03):`). Verified via `git log --oneline a9111221..HEAD`:

```
8a022fe test(70-03): cover Generate button disabled state, click->preview swap, back-to-list
f306309 feat(70-03): add Generate button + ReviewDocPreview panel-internal swap
66a0810 feat(70-03): add panelMode + previewMarkdown + generate action to review-session rune
bc5d547 test(70-03): add failing tests for review-session preview state
```

Task 1 follows the RED-then-GREEN sequence (commits 1 and 2 in the listing above, reading bottom-up). Tasks 2 and 3 are non-TDD per the plan; Task 3 itself adds tests that exercise the Task 2 surface.

## Verification Results

- `ls src/components/ReviewDocPreview.svelte` → exists
- `grep -c 'preview-spacer' src/components/ReviewDocPreview.svelte` → **3** (>= 1)
- `grep -cE '#[0-9a-fA-F]{3,6}|rgb\(' src/components/ReviewDocPreview.svelte` → **0**
- `grep -c 'panelMode' src/lib/review-session.svelte.ts` → **6** (>= 1)
- `grep -c 'previewMarkdown' src/lib/review-session.svelte.ts` → **6** (>= 1)
- `grep -c 'generate_review_doc' src/lib/review-session.svelte.ts` → **2** (>= 1)
- `grep -c 'generate_review_doc' src/components/ReviewPanel.test.ts` → **5** (>= 1)
- `grep -c 'ReviewDocPreview' src/components/ReviewPanel.svelte` → **2** (>= 1)
- `npx vitest run ReviewPanel.test` → **21 tests passed** (18 existing + 3 Phase-70)
- `npx vitest run review-session` → **7 tests passed** (6 Phase-70 + 1 baseline — actually all 7 are Phase-70 cases since the rune had no prior tests)
- `just check` → **exits 0** (fmt + biome + svelte-check + clippy + cargo-test + 527 vitest tests)

## User Setup Required

None — no external service configuration, no new dependencies.

## Next Phase Readiness

- Phase 71 (Copy / Save) can dock new buttons inside `ReviewDocPreview.svelte`'s header `.preview-spacer` flex cell — no layout restructuring required.
- `session.state.previewMarkdown` is the canonical string source; Phase 71's Copy reads it directly. Save will hand it off to a file-dialog IPC + write.
- The cached previewMarkdown survives swap-to-list-and-back, so a user can Generate, click Back to compare to comments, click forward to preview again, then Copy — all without re-invoking the IPC.
- `setReviewActive(false)` already clears the cached markdown on session end (D-02 contract); no Phase-71 lifecycle work is needed.

## Self-Check: PASSED

- `src/lib/review-session.svelte.ts` — FOUND, contains panelMode/previewMarkdown/showList/showPreview/generate
- `src/lib/review-session.svelte.test.ts` — FOUND
- `src/components/ReviewDocPreview.svelte` — FOUND
- `src/components/ReviewPanel.svelte` — FOUND, contains Generate button + preview-swap branch
- `src/components/RepoView.svelte` — FOUND, threads `session={reviewSession}` into ReviewPanel
- `src/components/ReviewPanel.test.ts` — FOUND, contains `generate_review_doc` switch case + 3 Phase-70 tests
- Commit hashes verified in git log:
  - `bc5d547 test(70-03): add failing tests for review-session preview state` — FOUND
  - `66a0810 feat(70-03): add panelMode + previewMarkdown + generate action to review-session rune` — FOUND
  - `f306309 feat(70-03): add Generate button + ReviewDocPreview panel-internal swap` — FOUND
  - `8a022fe test(70-03): cover Generate button disabled state, click->preview swap, back-to-list` — FOUND
- No STATE.md / ROADMAP.md edits in this worktree (orchestrator owns those)
- `just check` exits 0 — verified

---
*Phase: 70-excerpt-resolution-markdown-render*
*Completed: 2026-05-26*
