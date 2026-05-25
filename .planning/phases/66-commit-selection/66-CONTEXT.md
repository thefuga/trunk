# Phase 66: Commit Selection - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

The user defines which commits a review session covers. Two entry paths feed one
flat set of commits: **range seeding** (pick a base and a tip on the graph) and
**hand-picking** (add individual commits from the graph context menu). The user
can remove commits and always see the resulting list — in graph order, without
duplicates.

**In scope:** range-seed command (git2 revwalk), add/remove-commit commands,
populating the existing `ReviewSession.commits: Vec<String>`, persisting through
the Phase 65 store, a graph marker for in-session commits, and a minimal commit
list in the review panel (SEL-01..04).

**Out of scope (later phases):** anchor/comment capture from diff or full-file
(67/68), the real comment-management panel (69 — replaces today's stub), markdown
render (70), clipboard/save (71). No comment content is captured here; selection
only populates the commit set.

</domain>

<decisions>
## Implementation Decisions

### Range Seeding (SEL-01)
- **D-01:** Range is seeded by **two right-clicks on the graph** — right-click a
  commit → "Set as review base", then right-click another → "Add range to
  review". Reuses the existing native context menu (`showCommitContextMenu`,
  `CommitGraph.svelte:567`), same pattern as Checkout/Reset/Cherry-pick. No new
  modal. A **transient base highlight** marks the pending base between the two
  clicks (cleared after the range is added or the action is cancelled).
- **D-02:** Range is **inclusive of both base and tip** — `[base..tip]`, not
  `base..tip`. The base commit is itself part of the review set. Implement via
  revwalk `push(tip)` + `hide(base.parent(0))`, with the root-commit fallback
  (don't hide anything) already demonstrated in `interactive_rebase.rs:53`.
- **D-03:** Seeding a range **unions** its commits into the existing selection —
  it never replaces. The session is a set; hand-picked commits survive a
  subsequent range seed, and the only way out is explicit remove. Dedup keeps the
  set clean (success criterion #4).

### Selection Visibility (SEL-04)
- **D-04:** In-session commits are **visually marked in the graph** (e.g. a
  left-gutter accent / dot or tinted row). The user hand-picks *from* the graph,
  so the marker shows what's already in, prevents accidental duplicate-adds, and
  shows what "Remove" will act on. Use a **theme CSS custom property — no inline
  color** (project rule).
- **D-05:** The panel commit list is **minimal**: short SHA + commit summary per
  row, rendered in **graph order**, dedup'd. This satisfies SEL-04 with the least
  throwaway markup — the review panel is Phase 65's throwaway stub (D-12),
  replaced by the real panel in Phase 69. Do not over-invest.

### Add / Remove Affordances (SEL-02, SEL-03)
- **D-06:** The graph context menu carries a **single toggle item**: "Add to
  review" when the commit is not in the session, "Remove from review" when it is.
  Covers both SEL-02 and SEL-03 from the graph.
- **D-07:** Each row in the panel list also gets an **× / remove button** —
  remove works both from the graph and from the list the user is looking at.

### Merge-Commit Handling
- **D-08:** Merge commits are **selectable like any other commit** (add,
  range-seed). The "diff-source anchors only see the first parent" restriction
  from ROADMAP §"Phase 66" Notes is **NOT enforced at selection time** — it is
  deferred to **anchor-capture time (Phase 67)**. Rationale: source (diff vs
  full-file) is chosen at capture, not selection; full-file-source review of a
  merge is valid; and blocking merges would make range seeding diverge from
  `git log base..tip`. (Contrast: Cherry-pick/Revert *are* disabled for merges at
  `CommitGraph.svelte:682` — that constraint does not apply here.)

### Claude's Discretion (delegated to planner)
- **Invalid-range handling** — when base is not an ancestor of tip, base == tip,
  or the range is empty. Lean: validate with `merge_base` / `graph_descendant_of`
  and reject with a toast rather than seeding a surprising set. Planner decides
  exact validation and messaging.
- **Selected-but-not-loaded commits** — graph history is paginated
  (`history.rs`); decide how a selected commit that's outside the currently
  loaded graph slice is ordered/displayed in the list (e.g. resolve its summary
  on demand, or order by stored order with a fallback).
- **Command surface & naming** — whether selection commands live in the existing
  `commands/review.rs` (recommended) or a new file; exact command/struct names
  (follow established serde conventions: snake_case structs, camelCase via
  `rename_all` for frontend-facing request types).
- **Persistence/event wiring** — reuse the Phase 65 store
  (`review_store::save_session`, atomic tmp+rename) and emit `session-changed`
  after each mutation, mirroring the existing lifecycle commands. Whether each
  add/remove writes immediately (recommended, matches D-10 disk-first) is the
  planner's call.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone & phase spec
- `.planning/ROADMAP.md` §"Phase 66: Commit Selection" — goal, the four success
  criteria, and the Notes: revwalk push/hide approach, hand-pick reuses the
  v0.3/v0.5 commit-row context-menu pattern, and the merge-commit policy flagged
  as decide-in-planning (resolved here as D-08).
- `.planning/REQUIREMENTS.md` — SEL-01..04 (this phase) and the downstream
  anchor/render chain the selection feeds, plus the "Out of Scope" table.

### Phase 65 keystone (the schema and store this phase writes into)
- `.planning/phases/65-data-model-persistence-session-lifecycle/65-CONTEXT.md` —
  locked schema/persistence decisions: D-01 (anchor coordinates), D-06 (one
  session per repo), D-08 (`ReviewSessionsState`), D-09/D-10 (own JSON store,
  atomic tmp+rename — NOT tauri-plugin-store), D-11 (canonical-path keying),
  DP-01 (`session-changed` event for multi-tab sync).
- `src-tauri/src/git/types.rs:330` — `ReviewSession { schema_version, commits:
  Vec<String>, comments, draft_comment }`. Phase 66 **populates `commits`**; the
  shape does not change.
- `src-tauri/src/git/review_store.rs` — `save_session` / `load_session` /
  `delete_session` (atomic tmp+rename); the persistence path every mutation uses.
- `src-tauri/src/commands/review.rs` — Phase 65 lifecycle commands; the
  `_inner(data_dir, …)` testable core + thin command wrapper + `app.emit(
  "session-changed", canonical)` pattern (lines 165-248) to mirror for selection
  commands.

### Codebase patterns to mirror
- `src-tauri/src/commands/interactive_rebase.rs:40-59` — revwalk `push` + `hide`
  range walk, including the root-commit "don't hide anything" branch (line 53)
  needed for D-02.
- `src/components/CommitGraph.svelte:567-721` — `showCommitContextMenu`: native
  Tauri `Menu`/`MenuItem` build + `menu.popup()`; where the Add/Remove toggle
  (D-06) is added, and the precedent for merge-gated items (`enabled:
  !commit.is_merge`, line 682) that D-08 deliberately does NOT follow.
- `src/components/ReviewPanel.svelte` — the throwaway stub; where the minimal
  commit list (D-05) and per-row × (D-07) attach; already listens for
  `session-changed` and reloads.

### Project conventions
- `.planning/codebase/CONVENTIONS.md`, `STACK.md`, `ARCHITECTURE.md` — Rust/Svelte
  conventions, git2-only (no shelling out), theme CSS custom properties (no inline
  colors), command structure.
- `.claude/rules/commit-graph.md` + `.planning/COMMIT-GRAPH-ARCHITECTURE.md` —
  required reading before touching the graph pipeline (the D-04 marker touches
  graph rendering).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`ReviewSession.commits: Vec<String>`** (`types.rs:333`): already exists and is
  currently empty — Phase 66 just fills it. No schema change.
- **Phase 65 store + event pattern** (`review_store.rs`, `review.rs`): atomic
  save/load/delete and the `session-changed` emit are done; selection commands
  reuse them rather than inventing persistence.
- **Native commit context menu** (`CommitGraph.svelte:567`): the insertion point
  for the Add/Remove toggle.
- **Revwalk range pattern** (`interactive_rebase.rs:40`): push tip / hide base
  (and base.parent for inclusivity), root-commit fallback included.
- **`graph::walk_commits`** (used across `history.rs`, `commit.rs`, etc.):
  produces commits in graph order — the ordering authority for SEL-04's list.

### Established Patterns
- **`_inner(data_dir, …)` + thin `#[tauri::command]` wrapper + emit** in
  `review.rs` — the testable command shape selection commands should follow.
- **serde conventions** (Phase 59-64): snake_case Serialize-default structs;
  camelCase via `rename_all` for frontend-facing request/option types.
- **Theme CSS custom properties only** — the D-04 graph marker must use a
  `--color-*` var, never an inline color.

### Integration Points
- `CommitGraph.svelte` `showCommitContextMenu` — Add/Remove toggle item; needs to
  know current session membership and whether a session is active.
- `ReviewPanel.svelte` — minimal commit list + per-row remove; already wired to
  `session-changed`.
- `commands/review.rs` + `lib.rs` `invoke_handler` — register the new selection
  commands.
- Graph render layer — the in-session marker (read `commit-graph.md` first).

</code_context>

<specifics>
## Specific Ideas

- The session is a **set of commits**, not an ordered range record: range seeding
  and hand-picking both just add to the set; remove subtracts; render is always
  graph-order + dedup. "Range" is an input gesture, not stored state.
- "Base" means **the first commit the user wants in the review** (hence inclusive,
  D-02), not "the point I'm diffing against."
- The add affordance is the graph (where you browse); the remove affordance is
  both the graph toggle and the list (where you see what's in).

</specifics>

<deferred>
## Deferred Ideas

- **Enforcing diff-source-only-on-non-merges** — surfaced via the ROADMAP merge
  note; deliberately pushed to Phase 67 (anchor capture), not selection (D-08).
  Phase 67 must disable diff-source anchors on merge commits.
- **Richer commit list** (author, date, etc.) — rejected for this phase (D-05);
  belongs to the real review panel in Phase 69.
- No new scope-creep ideas surfaced — discussion stayed within the phase boundary.

### Reviewed Todos (not folded)
- **`2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md`**
  — matched on keywords "commit, merge" (score 0.6) but is about merge/revert
  *commit-message editing*, unrelated to review-session commit *selection*. Not
  folded; left for its own track.

</deferred>

---

*Phase: 66-commit-selection*
*Context gathered: 2026-05-25*
