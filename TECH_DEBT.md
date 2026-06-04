# Tech Debt

Living register of known debt in Trunk. Each item is verified against the
codebase (counts and `file:line` were grepped at the date below, not estimated).
Ranked within each section by payoff (impact ÷ effort).

**Conventions**
- **Severity** — high / med / low (impact if left unpaid).
- **Effort** — trivial (<10m) / small (<1h) / medium (1–3h) / large (>3h).
- When you pay an item, move it to **Paid** at the bottom with the commit SHA.

_Last audited: 2026-06-04 (`main` @ 8869754)_

---

## A. Tracked & ready

### A1 — Clean-tree snapshot adds a changeless review commit
- **Severity:** low · **Effort:** small · **Source:** `.planning/todos/pending/2026-05-31-snapshot-empty-working-tree-no-op-guard.md`
- **Problem:** `snapshot_working_tree` (`src-tauri/src/git/workdir_snapshot.rs`) has no
  empty-diff guard. On a clean tree the snapshot tree equals HEAD's tree, so
  `add_working_tree_review` creates a dangling commit whose diff against its
  parent is empty and adds it to the session. It renders as an
  "Uncommitted changes — …" entry with no diff. Not a crash — just confusing.
- **Fix:** In `snapshot_working_tree` (or `add_working_tree_review`), compare the
  snapshot tree OID to HEAD's tree OID. If equal, return a distinct
  `nothing_to_review` `TrunkError` code; surface it in `commands/review.rs` and
  show a "nothing to review" toast instead of adding the commit. ~3 files:
  `workdir_snapshot.rs`, `commands/review.rs`, `review-session.svelte.ts` /
  `ReviewPanel.svelte`.
- **Bonus:** add a `ReviewPanel` test locking the snapshot-button gate
  (`sessionState === "active"`, fixed in 8dc2e7b) — hidden in `resume-available`,
  shown in `active`.

---

## B. Structural — oversized files

These are not bugs; they are leaky abstractions that slow every future change.
Pay by *mechanical extraction behind passing tests* (refactor on green), never
by rewriting behavior.

### B1 — `commands/review.rs` is 2730 lines, four concerns interleaved
- **Severity:** med · **Effort:** medium
- **Problem:** Session I/O + comment management + range/snapshot logic + rendering
  hints + 350+ lines of tests in one file.
- **Fix:** Split into submodules: `review_session.rs` (start/resume/end),
  `review_comments.rs` (add/edit/delete/resolve), `review_rendering.rs`
  (render/snapshot). Pure extraction — no behavior change. Run `just check`
  after (the integration binary in `src-tauri/tests/` compiles independently;
  a filtered `cargo test --lib` will lie — see MEMORY).

### B2 — `git/review.rs` is 1746 lines, monolithic `render()`
- **Severity:** med · **Effort:** medium
- **Problem:** One `render()` (`git/review.rs:357`) partitions comments, builds
  markdown, groups by file/commit, handles orphans, emits fences — plus
  `fence_length()`/`fence_language()` helpers.
- **Fix:** Extract a `MarkdownBuilder { session, repo, resolved_comments }` with
  `add_header()` / `add_commits_section()` / `add_comments_section()`. Makes the
  pipeline unit-testable incrementally (and would help pay D-test gaps below).

### B3 — `CommitGraph.svelte` is 2069 lines
- **Severity:** med · **Effort:** large
- **Problem:** Largest component by far; mixes layout geometry, column auto-fit,
  rendering, and event handling. Four separate `$effect` blocks
  (`CommitGraph.svelte:250,263,272,281`) each `untrack`-read `columnWidths` and
  write back, causing multiple reactive passes for one logical concern.
- **Fix:** Two independent slices: (a) collapse the four width-fit effects into a
  single `$derived.by(...)` that reads all inputs once and writes once;
  (b) extract pure geometry/path helpers into `$lib` (the v0.5 pure-pipeline
  pattern already established for graph rendering).

### B4 — Other large components (lower priority)
- `StagingPanel.svelte` (1349), `RebaseEditor.svelte` (1041), `RepoView.svelte`
  (983), `ReviewPanel.svelte` (954). Address opportunistically when touching them.

---

## C. Backend duplication & error handling

### C1 — ✅ PAID (working tree, uncommitted) — `open_repo` duplicated across 10 command modules
- **Severity:** high · **Effort:** small · **Verified:** 10 definitions
  (`diff.rs:15`, `operation_state.rs:26`, `repo.rs:18`, `commit_actions.rs:23`,
  `staging.rs:9`, `branches.rs:15`, `merge_editor.rs:9`, `commit.rs:19`,
  `stash.rs:11`, `interactive_rebase.rs:19`)
- **Problem:** Same logic (state-map lookup → `not_open` error → `Repository::open`)
  copied 10×, under **two different names** — a hesitation smell. Any change to
  the open/error contract is shotgun surgery across 10 files.
- **Fix:** One `pub(crate) fn open_repo_from_state(...)` in `git/` (or
  `commands/mod.rs`); delete the 9 copies; settle on one name.

### C2 — ✅ PAID (working tree, uncommitted) — Error→JSON serialization boilerplate
- **Severity:** high · **Effort:** small · **Verified:** 94 occurrences of
  `serde_json::to_string(&TrunkError…)` across `commands/` (review.rs 20,
  staging.rs 16, commit_actions.rs 11, operation_state.rs 9, diff.rs 6, …)
- **Problem:** Every command wrapper hand-serializes its error and `.unwrap()`s
  the serialization. Brittle (a serialize failure panics) and noisy.
- **Fix:** Extract `fn to_json_err(code: &str, msg: impl ToString) -> String` in
  `error.rs` and use everywhere; drop the `.unwrap()` for an infallible fallback.
  Optional follow-up: a `wrap_command!` macro or a Tauri responder type so
  wrappers stop hand-serializing entirely.

### C3 — ✅ PAID (working tree, uncommitted) — `is_dirty` duplicated verbatim
- **Severity:** med · **Effort:** trivial · **Verified:** `commit_actions.rs:33`
  and `branches.rs:27` (identical signature + body)
- **Problem:** The definition of "dirty repo" lives in two places.
- **Fix:** Move to `git/` as `pub(crate) fn is_repo_dirty(repo) -> Result<bool, …>`;
  import in both.

### C4 — ~~`not_open` error code overloaded~~ — DISMISSED on inspection (2026-06-04)
- An audit pass flagged `not_open` as overloaded for both repo and session
  lookups. **Not true:** `not_open` appears only at `review.rs:73` and `:1094`,
  both "Repository not open" (repo-map misses); session misses already use a
  distinct `no_session` code (8 sites: `review.rs:465,714,757,846,1074,1140,1175,1232`).
  The taxonomy is already clean. Kept as a record that this was checked.

### C5 — Internal helpers marked `pub` for testability
- **Severity:** low · **Effort:** trivial
- **Problem:** `validate_range` (`review.rs:193`), `compute_range_oids` (`:223`),
  `apply_add` (`:248`), `apply_remove` (`:255`), `union_dedup` (`:262`),
  `intersect_graph_order` (`:273`) are `pub` but only used in-crate — they read
  as public API.
- **Fix:** Narrow to `pub(crate)`. (Recall the MEMORY footgun: after changing
  visibility/signatures, verify with `just check`, not filtered `--lib`.)

---

## D. Frontend — project-rule violations & type safety

### D1 — ✅ PAID (working tree, uncommitted) — Inline error-color hex violated "never inline colors"
- **Severity:** high · **Effort:** small · **Verified:** 26 inline `#rrggbb` in
  `src/components/*.svelte`
- **Problem:** The error box `background:#3d1c1c; border:1px solid #6b2a2a;
  color:#f87171` is pasted verbatim in `BranchRow.svelte:81`,
  `CommitGraph.svelte:1737`, `WelcomeScreen.svelte:88`; bare `color:#f87171`
  error text in `CommitForm.svelte:171,194`, `BranchSidebar.svelte:653`,
  `CommitGraph.svelte:2025`. **These are out-of-sync near-duplicates of tokens
  that already exist** — `--color-danger` (`#f87171`), `--color-danger-bg`,
  `--color-danger-border` (`src/app.css:55-57`).
- **Fix:** Add an `.error-box` / `.error-text` class (or a small `<ErrorBox>` /
  `<ErrorText>` component) backed by the existing `--color-danger*` tokens;
  replace all 26 inline literals. Direct CLAUDE.md-rule cleanup.

### D2 — File-status color map duplicated
- **Severity:** med · **Effort:** small · **Verified:** `CommitDetail.svelte:14-19`
  and `FileRow.svelte:47-52` (same status→hex map); plus a one-off
  `#22c55e/#f87171` in `FileRow.svelte:117`
- **Problem:** Status-to-color knowledge lives in two component files; changing a
  status color means editing both.
- **Fix:** Extract `src/lib/status-colors.ts` (and add the colors to `app.css` as
  tokens to satisfy D1's rule); import in both components and the action button.

### D3 — `VirtualList` piggybacks state via `as unknown as` casts (6 sites)
- **Severity:** high · **Effort:** small · **Verified:**
  `VirtualList.svelte:165,167,169,190,192,197`
- **Problem:** Pending height deltas are stashed on `heightManager.viewport`
  under a string key via `"__svl_pendingHeightAdj__" as unknown as keyof
  HTMLElement` and `viewport as unknown as Record<string, number>`. Defeats
  TS strict mode on a hot scroll path; a refactor of `viewport` breaks silently.
- **Fix:** Give the height manager an explicit `pendingDelta` field with
  `getPendingDelta()` / `addDelta(n)` / `clearPending()`; delete the cast chain.

### D4 — `rgba()`/shadow literals not tokenized
- **Severity:** low · **Effort:** small · **Verified:** 5 `rgba(` literals in
  components (drop-shadows in `PullDropdown.svelte:139`, `SearchBar.svelte:67`,
  `RebaseEditor.svelte`; search highlight in `CommitRow.svelte`)
- **Fix:** Add `--shadow-sm/-md/-lg` and `--color-search-current/-other` tokens to
  `app.css`; reference via `var()`. Folds into D1.

### D5 — `RepoView` layout prop-drilling
- **Severity:** med · **Effort:** medium
- **Problem:** `RepoView.svelte` threads ~9 pane-geometry props + their
  `on*change` callbacks from `App.svelte`. Adding a pane/gesture adds more
  callbacks through the chain.
- **Fix:** A `layout-state.svelte.ts` rune owning pane geometry (mirrors the
  existing `review-session.svelte.ts` pattern); `RepoView` imports it directly.

---

## E. Test & tooling debt

### E1 — `diff_commit` command has no production caller
- **Severity:** med · **Effort:** trivial · **Verified:** defined
  `commands/diff.rs:552`, registered `lib.rs:116`, exercised by
  `tests/test_diff.rs` + `tests/common/drivers/diff.rs`, **never `invoke`d from
  `src/`** (frontend uses `diff_commit_file`).
- **Problem:** A registered IPC command with tests but no caller — maintenance
  surface with no user. (Not pure dead code: tests/driver keep it alive.)
- **Fix:** Decide deliberately — either delete it (command + lib.rs registration +
  test + driver) or document why it's retained (e.g. future/external use). Don't
  leave it ambiguous.

### E2 — Review system is the most complex backend feature and the least tested
- **Severity:** high · **Effort:** large · **Verified:** `commands/review.rs`
  (2730) + `git/review.rs` (1746) ≈ 4.4k LOC vs `tests/test_review.rs` (263 LOC)
- **Problem:** Tests cover session persistence/lifecycle only. No coverage for
  add/remove review commit, add/edit/delete/resolve comment, snapshot, or doc
  generation.
- **Fix:** Add integration tests for the mutation + snapshot + doc-gen paths.
  The B1/B2 splits make this far easier — pair them.

### E3 — `operation_state` / `commit_actions` thinly tested
- **Severity:** med · **Effort:** medium · **Verified:** `operation_state.rs`
  (903 LOC vs 213 test LOC); `commit_actions.rs` (823 vs 273)
- **Problem:** Merge/rebase state machines and undo/redo/cherry-pick/reset cover
  happy paths; missing corrupt-state recovery, abort cleanup, dirty-tree resets,
  conflict handling.
- **Fix:** Add edge-case tests per the gaps above.

### E4 — `--lib`-passes-while-integration-fails footgun unguarded in CI
- **Severity:** med · **Effort:** small · **Verified:** `justfile` cargo-test;
  `.github/workflows/ci.yml`; documented in MEMORY (hit on 76-01)
- **Problem:** `cargo test --lib <module>` can pass green while the independent
  `src-tauri/tests/` integration binary fails to compile against a renamed
  symbol. Nothing enforces the full build.
- **Fix:** Ensure CI runs the full `cargo test` (lib + integration) and treats
  integration-compile failure as a gate. Already noted in CLAUDE.md/MEMORY —
  make it a CI assertion, not tribal knowledge.

### E5 — Component test files dwarf their components
- **Severity:** low · **Effort:** medium · **Verified:** `DiffPanel.test.ts`
  (1861) vs `DiffPanel.svelte` (877); `ReviewPanel.test.ts` (1539) vs
  `ReviewPanel.svelte` (954)
- **Problem:** Heavy `invoke` mocking + manual promise sequencing suggests tests
  assert implementation (mock call counts, state transitions) over behavior —
  fragile under refactor.
- **Fix:** Shift toward role/text queries; extract shared mock factories. Low
  priority — only when these tests start blocking refactors.

### E6 — Doc drift: CLAUDE.md doesn't list active Tauri plugins
- **Severity:** low · **Effort:** trivial
- **Fix:** `src-tauri/Cargo.toml` enables specific plugins (dialog, store,
  window-state, clipboard); CLAUDE.md says only "Tauri 2". List them.

---

## Suggested order

1. **Quick wins (one sitting):** C3, C1, C2, D1 — high-payoff dedup + the
   CLAUDE-rule color cleanup. Low risk, big readability gain.
2. **Scoped fix:** A1 (the tracked todo) + its `ReviewPanel` gate test.
3. **Type safety:** D3.
4. **Structural, behind green:** B1 → B2, then add E2 tests in the same pass.
5. **Backstop:** E4 (CI gate) so the integration footgun can't recur.

---

## Paid

_Append `- [ID] paid in <sha> — note` as items are closed._

- **C1** paid in `d470cba` — extracted `commands::open_repo_from_state`; deleted 9
  duplicate local fns across diff/operation_state/staging/branches/commit_actions/
  commit/merge_editor/stash/interactive_rebase. `repo.rs::open_repo` (the Tauri
  command) and the new shared helper are the only `open_repo*` left.
- **C2** paid in `d470cba` — added `TrunkError::to_json()` (infallible, no panic);
  converted **197** sites (`&e` + `&TrunkError::new(...)`) across 14 command files.
  Removes ~197 `.unwrap()` panic points on the error path.
- **C3** paid in `d470cba` — extracted `git::repository::is_repo_dirty`; deleted
  both local `is_dirty` copies (branches, commit_actions).
- **D1** paid in `59f6353` — moved 16 error-context hex literals into scoped
  `.error-banner`/`.error-text` classes backed by `--color-danger*` tokens across
  6 components (incl. Toast.svelte). **Visual note:** error-box backgrounds shift
  opaque→translucent (intentional convergence to the design system). Status-color
  maps (D2) left untouched.
- Verified by `just check` (fmt, biome, svelte-check, clippy `-D warnings`,
  cargo-test, 603 vitest) — all green.
