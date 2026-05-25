# Phase 69: Comment Management UI - Context

**Gathered:** 2026-05-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the throwaway `ReviewPanel.svelte` stub (Phase 65 D-12) with the real
review panel: the accumulated review is fully visible and actionable. The user can
**view** every comment in the active session (CMT-01), **add a commit-level
comment** with no code anchor (ANCH-03), **edit** a comment's text (CMT-02),
**delete** a comment with a confirmation prompt (CMT-03), and **jump** from a
comment to its anchored code location (CMT-04) — with an anchor that no longer
resolves shown as a read-only "orphaned" state with a reason badge instead of
navigating nowhere or erroring.

**In scope:** the real review panel in the right pane (replaces
CommitDetail/DiffPanel content when Review mode is active, driven by a
`review-session.svelte.ts` rune module per ROADMAP §69); a backend command to
read all comments for the panel (none exists today); commit-level comment write
(tied to a commit, no file/lines); `edit_comment` / `delete_comment` commands;
a git2-backed resolvability check; jump-to-anchor navigation that reveals the
diff/full-file view per `Source`; the schema v1→v2 bump that adds a stable
comment `id` and a representation for commit-level comments.

**Out of scope (later phases / deferred):**
- In-diff comment markers — gutter badges/click-to-edit on anchored lines in the
  diff/full-file views (deferred again this phase — see Deferred Ideas).
- Markdown render, render-time excerpt re-resolution, the "unresolvable" render
  section (Phase 70 / DOC-04). The eager resolvability check here is for the
  panel's orphan badges; Phase 70 owns the render-side resolution.
- Clipboard / save-to-file output (Phase 71).
- New comment metadata (severity, author, threading) — explicitly out per
  REQUIREMENTS "Out of Scope".

</domain>

<decisions>
## Implementation Decisions

### Commit-level comments (ANCH-03) — discussed
- **D-01:** A commit-level comment is **tied to a specific commit** (no
  file/line). It renders with the commit SHA heading like line-anchored comments
  (Phase 70 contract `(sha)`), NOT as a free-floating session note. The persisted
  representation MUST store `commit_oid` and MUST be distinguishable from
  line-anchored comments so render/jump can branch. Exact mechanism is the
  planner's call (see Discretion) but it rides the v2 schema bump (D-04).
- **D-02:** Creation surface is a **per-commit "Add note" affordance in the panel's
  commit list** — each commit row gets the affordance, keeping creation next to the
  commit the note is about. (The Phase 65/66 commit list already lives in the panel.)

### Edit/Delete identity (CMT-02/03) — discussed
- **D-03:** Add a **stable `id` field** to each `Comment`, generated on write.
  `edit_comment` / `delete_comment` target by `id` — robust to list reordering and
  to a second same-repo tab mutating the list between the panel's read and the
  action. Index-by-position was rejected: it edits/deletes the wrong comment under
  a stale index, which undercuts the multi-tab correctness Phase 65 invested in
  (canonical-path keying, atomic writes, `session-changed`).
- **D-04:** **Bump `schema_version` 1 → 2.** Both the comment `id` (D-03) and the
  commit-level representation (D-01) land in v2 — one bump, not two. **Backfill ids
  on load** for any existing v1 session (assign ids to anchored comments that lack
  them). Honor the Phase 65 recovery policy: a *newer*-than-supported version is
  still refused untouched (D-16); this build now supports v2.
- **D-05:** **Delete uses the v0.6 confirmation-dialog pattern** (`@tauri-apps/
  plugin-dialog` `ask`), consistent with `DiffPanel.handleDiscardLines`. One-click
  irreversible delete would be inconsistent with the rest of the app (carried from
  ROADMAP §69 Notes).

### Jump-to-anchor & orphan handling (CMT-04) — discussed
- **D-06:** Resolvability is checked **eagerly at panel load** via a **Rust
  git2-backed command** that resolves every comment: does the `commit_oid` exist,
  does `file_path` exist at that commit on the anchor's `side`, is the line range
  within bounds. Orphan badges + reasons therefore show up-front (matches the
  success criterion's "shows a read-only orphaned state with a reason badge"
  without requiring a click). A frontend "is the commit in `session.commits`"
  check is insufficient — it can't catch a renamed/deleted file or rewritten
  history. Cost (git2 per comment on load) is acceptable for review-sized sessions.
- **D-07:** Jump **reveals the diff**: clicking jump selects the commit + file and
  switches the right pane back to the diff/full-file view **per `Source`**
  (`Source::Diff` → the commit's diff of that file; `Source::FullFile` → the
  full-file-at-commit view), scrolls to the line range and highlights it. The
  session stays active; a persistent "Review" toggle returns to the panel. This
  respects the locked right-pane placement (panel replaces diff content) — jump is
  the moment the diff is shown instead.
- **D-08:** An **orphaned comment is read-only** — its jump action is disabled and
  it shows a reason badge (e.g. commit gone / file gone / line out of range). The
  comment text + cached excerpt stay visible (Phase 65 D-04: text is stored
  independently of anchor resolvability).

### Panel presentation — discussed
- **D-09:** The comment list is **grouped by commit** — comments nested under their
  commit (line-anchored carry `commit_oid`; commit-level are tied to one).
  Matches the per-commit "Add note" affordance (D-02) and the review mental model.
  Panel grouping is independent of Phase 70's render-by-line ordering (Phase 65
  D-07).
- **D-10:** Edit is **inline in the panel** — click edit on a comment row → a
  textarea appears in place; save/cancel there. Self-contained, works uniformly for
  line-anchored, commit-level, AND orphaned comments (does not require a working
  anchor or navigating to the diff). Reusing the Phase 67 diff composer was
  rejected because it couples edit to a resolvable anchor.

### Claude's Discretion (delegated to planner)
- **Commit-level storage mechanism (D-01):** the least-disruptive v2 representation
  that stores `commit_oid` and stays distinguishable from line anchors. Candidates:
  a new `Source::Commit` enum variant (additive; reuse `Anchor` with line fields
  unused), or making `Anchor`'s file/line fields optional, or a dedicated
  commit-comment shape. Keep Phase 67/68 line-anchored anchors unchanged.
- **`id` generation:** uuid vs monotonic counter vs content hash — planner's call,
  following existing serde conventions. Must be stable for a comment's lifetime.
- **Command surface & naming:** the new read-comments command (CMT-01 — none exists;
  the stub only lists commits), `edit_comment`, `delete_comment`, and the
  commit-level writer (extend the existing `add_comment` to accept an optional
  anchor / commit-level mode, OR a sibling command). Follow the established
  `_inner(data_dir, …)` + thin `#[tauri::command]` wrapper + `emit("session-changed")`
  pattern in `commands/review.rs`. Whether the resolvability check (D-06) is its own
  command or folded into the read-comments command is the planner's call.
- **`review-session.svelte.ts` rune module shape** (ROADMAP §69 names it; it does
  not exist yet) and how it drives Review-mode right-pane swapping.
- **Attach-success / save feedback:** rely on `session-changed` → reload, mostly
  silent per project convention; toasts optional.
- **Empty-text validation:** zero-text comments disabled; exact validation point
  is the planner's (consistent with Phase 67 discretion).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone & phase spec
- `.planning/ROADMAP.md` §"Phase 69: Comment Management UI" — goal, the four
  success criteria, Requirements (ANCH-03, CMT-01..04), and the locked Notes:
  reuse the v0.6 confirm-dialog for delete, jump must check resolvability first,
  the panel lives in the right pane (replaces CommitDetail/DiffPanel when Review
  mode is active) driven by `review-session.svelte.ts`. Also read §"Phase 70"
  (render contract `path:Lstart-Lend (sha)` / `(sha)` for commit-level; render-time
  re-resolution the eager check here parallels but does NOT replace).
- `.planning/REQUIREMENTS.md` — **ANCH-03, CMT-01, CMT-02, CMT-03, CMT-04** (this
  phase) and the "Out of Scope" table (no threading, no severity tags, no
  re-anchoring on history rewrite, static snapshot).

### Phase 65 keystone (the schema + store this phase MODIFIES via the v2 bump — read first)
- `.planning/phases/65-data-model-persistence-session-lifecycle/65-CONTEXT.md` —
  D-04 (text stored independent of anchor resolvability; underpins D-08 orphan
  display), D-05 (full schema), D-07 (capture-order storage, render re-orders;
  no id/timestamp/author in v1 — this phase ADDS id), D-13 (end hard-deletes),
  D-15/D-16 (corrupt → `.corrupt` sidecar; newer-version refused untouched — the
  v2 bump must keep D-16 working).
- `src-tauri/src/git/types.rs:288-336` — the review schema being extended to v2:
  `Source{Diff,FullFile}`, `Side{Old,New}`, `Anchor{commit_oid, file_path, source,
  side, start_line, end_line}` (all non-optional), `Comment{text, anchor:
  Option<Anchor>, cached_excerpt}` (**no id today**), `DraftComment{text, anchor}`,
  `ReviewSession{schema_version, commits, comments, draft_comment}`.
- `src-tauri/src/commands/review.rs` — Phase 65/66/67 commands. Key facts:
  `add_comment` (line ~511) currently requires a **non-optional** `Anchor`
  (`AddCommentRequest.anchor: Anchor`, `cached_excerpt: String`, lines ~368-374)
  and pushes `anchor: Some(req.anchor)` — commit-level (no line anchor) is not
  writable yet. `list_session_commits` (line ~570) is the ONLY session-read command;
  **there is no read-comments command** (CMT-01 needs one). The
  `_inner(data_dir, …)` + thin wrapper + `mutate_session_rmw` + `emit("session-
  changed", canonical)` pattern (e.g. `add_comment_inner` ~390) is what new
  commands mirror. Recovery / version handling lives here too.
- `src-tauri/src/git/review_store.rs` — `save_session`/`load_session`/
  `delete_session` (atomic tmp+rename). The v2 load path + id-backfill (D-04) hooks
  here or in the load-then-normalize step.

### Phase 67/68 (the anchors this panel lists, jumps to, and resolves)
- `.planning/phases/67-diff-source-anchor-capture/67-CONTEXT.md` — diff-source
  anchor semantics (L-01..L-08): source coords only, `side` discriminator,
  file-status constraints, cached excerpt as canonical body. Its Deferred Ideas
  section explicitly hands the in-diff comment browser + ANCH-03 to THIS phase.
- `.planning/phases/68-full-file-source-anchor-capture/68-CONTEXT.md` — full-file
  anchor semantics (absolute blob line numbers on the `new` side); the
  `Source::FullFile` branch jump (D-07) targets the full-file-at-commit view.

### Codebase patterns to mirror
- `src/components/ReviewPanel.svelte` — the throwaway stub THIS phase replaces.
  Shows the `session-changed` listener + per-repo canonical-path filter and the
  three lifecycle states; the real panel keeps the listener, adds the comment list.
- `src/App.svelte:587-605` — `reviewPanelOpen` gating; ReviewPanel currently renders
  as a thin bar above `RepoView`. ROADMAP §69 relocates the real panel to the right
  pane (replaces CommitDetail/DiffPanel) — reconcile with RepoView's pane layout.
- `src/components/CommitGraph.svelte:1312` `scrollToOid(oid)` and
  `src/components/RepoView.svelte` `selectedCommitOid`/`selectedFile`/
  `selectedCommitFile` + `showDiff` (~line 97-123) — the existing commit/file
  selection + scroll machinery jump-to-anchor (D-07) drives.
- `src/components/DiffPanel.svelte:257` `scrollToHunk`/`scrollIntoView` — the
  scroll-to-line-range pattern jump reuses after selecting the file.
- `src/components/DiffPanel.svelte` `handleDiscardLines` → plugin-dialog `ask` —
  the confirm pattern for delete (D-05).
- `.planning/codebase/CONVENTIONS.md`, `STACK.md`, `ARCHITECTURE.md`,
  `INTEGRATIONS.md` — Rust/Svelte conventions, git2-only for local reads, theme CSS
  custom properties (no inline colors), `safeInvoke<T>` for all IPC.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Phase 65/66 store + event** (`review_store.rs`, `commands/review.rs`): atomic
  save + `session-changed` emit + `mutate_session_rmw` — `edit_comment`,
  `delete_comment`, the read-comments command, and the commit-level writer reuse
  them.
- **Commit list in the panel** (`ReviewPanel.svelte` + `list_session_commits`):
  the per-commit "Add note" affordance (D-02) and the group-by-commit layout (D-09)
  attach to this existing list.
- **Selection + scroll machinery** (`CommitGraph.scrollToOid`,
  `RepoView.selectedCommitOid/selectedFile`, `DiffPanel.scrollToHunk`): jump-to-
  anchor (D-07) composes these — select commit, show file diff/full-file, scroll.
- **Confirm dialog** (`DiffPanel.handleDiscardLines` → plugin-dialog `ask`): D-05.

### Established Patterns
- **`_inner(data_dir, …)` + thin `#[tauri::command]` wrapper + emit** in
  `review.rs` — the shape every new review command follows; testable core takes a
  `data_dir`, the wrapper resolves canonical path + data dir and emits
  `session-changed`.
- **serde conventions:** review-schema enums serialize PascalCase with NO
  `rename_all`; Serialize-default structs snake_case; frontend-facing request
  types use `#[serde(rename_all = "camelCase")]`. The new `id` field + any
  commit-level shape must follow this.
- **Theme CSS custom properties only** — orphan badges, edit affordances, group
  headers use `--color-*` vars (Phase 67 D-04 lineage; `--color-danger*` already
  used for destructive actions in the stub).

### Integration Points
- **Schema v2** (`types.rs:288-336`): add `id` to `Comment`; add the commit-level
  representation; bump `schema_version` default to 2; load path backfills ids and
  handles v1→v2 (D-04) while keeping D-16 (refuse newer) intact.
- **`commands/review.rs` + `lib.rs` `invoke_handler`**: register the read-comments
  command, `edit_comment`, `delete_comment`, the resolvability command (or folded
  read), and the commit-level write path.
- **`App.svelte` / `RepoView.svelte`**: relocate the panel to the right pane with
  Review-mode swap; wire the new `review-session.svelte.ts` rune that owns
  panel-vs-diff right-pane state and the jump action (D-07).
- **`add_comment`**: either extend to accept an optional/commit-level anchor or add
  a sibling commit-level command (planner's call) — it must NOT break the existing
  line-anchored Phase 67/68 callers.

</code_context>

<specifics>
## Specific Ideas

- The rendered doc's recipient is an **AI coding agent** — commit-level comments
  are tied to a commit so the agent gets the `(sha)` framing, not an unanchored
  blob of notes. Keep the data lean (id is for stable edit/delete reference, not
  metadata creep).
- "Review mode" is a right-pane state, not a separate screen: the panel and the
  diff share the right pane and jump is the handoff between them (D-07). The
  persistent "Review" toggle is the way back.

</specifics>

<deferred>
## Deferred Ideas

- **In-diff comment markers / browser** — gutter badges on anchored lines in the
  diff & full-file views, with click-to-open/edit. Deferred from Phase 67, and
  deferred again here to keep Phase 69 focused on the panel-side flow + the 5
  success criteria. Backlog candidate for a future review-UX slice.
- **Render-time excerpt re-resolution + "unresolvable" render section** — Phase 70
  (DOC-04). The eager panel resolvability check (D-06) is panel-only; render owns
  its own resolution against the cached excerpt.
- **Comment metadata** (severity, author, threading) — out of scope per
  REQUIREMENTS; the `id` added here is identity only, not a metadata foothold.
- No scope-creep ideas surfaced — discussion stayed within the phase boundary.

### Reviewed Todos (not folded)
- **`2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md`**
  — matched on keywords "commit, instead, state" (score 0.6), but it concerns
  merge/revert *commit-message editing*, unrelated to review-session comment
  management. Already reviewed and declined in Phases 66/67; not folded.

</deferred>

---

*Phase: 69-comment-management-ui*
*Context gathered: 2026-05-26*
