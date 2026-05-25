# Phase 68: Full-File-Source Anchor Capture - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

The user selects a line range in the **full-file-at-commit view** (content-mode =
`full`, `FullFileView.svelte`) and attaches a comment. The comment is persisted as
a `FullFile`-source anchor — `(commit_oid, file_path, source=FullFile, side=New,
start_line, end_line)` storing **absolute 1-based blob line numbers on the `new`
side** — and survives an app restart. This is the second of two capture paths
(Phase 67 delivered diff-source) and reuses the **already-shipped, shared**
`add_comment` command (Phase 67 L-08).

**In scope:** net-new line-selection state in `FullFileView` (it has none today),
a full-file capture-time adapter (analog of `buildDiffAnchor`) that derives
`(side=New, start_line, end_line)` from the selected lines' `new_lineno`, mounting
the existing Comment affordance + `CommentComposer` inline in the full-file view,
caching the excerpt at attach-time, persisting the in-progress draft via the
existing `save_draft_comment`, and writing through the shared `add_comment`.

**Out of scope (later phases):** diff-source capture (done — Phase 67), the real
comment-management panel — list / edit / delete / jump-to-anchor (69, replaces
today's stub), commit-level comments with no anchor (69 / ANCH-03), markdown
render incl. the fresh-blob excerpt re-resolution and the "unresolvable" section
(70), clipboard / save output (71). The cached excerpt here is an **offline
fallback only** — Phase 70 re-resolves full-file excerpts from a fresh git2
tree→blob read at the commit, never by re-running the diff. Showing accumulated
comments as gutter markers in the full-file view is **Phase 69**, not here.

</domain>

<decisions>
## Implementation Decisions

### Selection Model (discussed)
- **D-01:** **Contiguous range via click + shift-click.** `FullFileView` is a flat
  continuous document (`fd.hunks.flatMap(h => h.lines)`) with no hunk boundaries,
  so the diff view's single-hunk `selectedHunkKey` + `selectedLineIndices:
  Set<number>` model does **not** map. A plain click sets a single-line selection;
  shift-click extends to a contiguous span (mirrors GitHub's full-file
  `#Lstart-Lend` UX). This collapses 1:1 onto the anchor's single `start..end` —
  no non-contiguous gaps the user can't see in the result. Net-new selection state
  lives in `FullFileView` (or a thin parent-owned store), NOT the diff-view
  selection state.

### Delete-Line Selectability (discussed)
- **D-02:** **Delete lines are non-selectable in the full-file view.** The full-file
  view is still a diff and shows interleaved Delete lines (red, `new_lineno = null`),
  but the mental frame is "the file **at** this commit" = new side only. Delete
  lines render but cannot be selection endpoints; a contiguous span simply **passes
  over** them and they are excluded from the persisted range AND the excerpt (they
  are not new-side content). This intentionally **diverges from Phase 67 L-03**
  (diff capture keeps `-` lines in the excerpt) because full-file semantics are
  new-side blob coordinates.

### Gap-Crossing Selections (discussed)
- **D-03:** **Allow gap-crossing; cache visible lines with a marker.** The full-file
  view is a 100k-context diff, so very large unchanged regions can be dropped and
  `new_lineno` skips at those boundaries. A contiguous selection straddling a gap
  keeps a **correct** `start..end` (they are blob coords), and the cached excerpt
  holds the visible lines with a `… N lines unchanged …` marker inserted at the
  gap. No rejection of gap-crossing selections — render (Phase 70) re-resolves from
  a fresh blob read anyway, so the cache is purely an offline fallback and low
  friction wins.

### Cached Excerpt Format (discussed)
- **D-04:** **Plain new-side content (no `+`/`-` prefixes).** Per DOC-02, full-file
  excerpts render **language-fenced**, not diff-fenced. The attach-time cached
  excerpt is therefore the selected lines as plain code (their content verbatim),
  with the D-03 gap marker where applicable. With D-02 (deletes excluded), the
  excerpt is naturally just the file's lines across the selected span. This
  diverges from Phase 67 L-06's diff-format excerpt — on purpose, to match the
  full-file render contract.

### Locked Carry-Forwards (from ROADMAP §"Phase 68" Notes + Phase 65/67 — do NOT re-litigate)
- **L-01:** Persist the anchor as **source coordinates only**, never diff-array
  positions or diff options. Schema is frozen (Phase 65 D-01):
  `Anchor { commit_oid, file_path, source, side, start_line, end_line }`. For this
  phase `source = FullFile`, `side = New`, line numbers are **absolute 1-based blob
  line numbers** read from each selected line's `new_lineno`.
  (`src-tauri/src/git/types.rs:295-315`.)
- **L-02:** `add_comment` is **already implemented and shared** (Phase 67 L-08) — a
  dumb writer that persists whatever `source`/`side` the TS adapter supplies, and
  already has a passing `Source::FullFile` round-trip test
  (`add_comment_persists_full_file_source_unchanged`,
  `src-tauri/src/commands/review.rs:1257`). **No backend command work expected**
  beyond possibly a full-file excerpt helper if planning decides one is needed;
  this phase is essentially frontend.
- **L-03:** Persist the comment **immediately on attach** (survives the watcher's
  `repo-changed` re-fetch) and persist the draft **on change** via the existing
  `save_draft_comment` (Phase 65 DP-02 → `draft_comment` field; 300ms debounce as
  in `CommentComposer`).
- **L-04:** Cache the excerpt at attach-time as the canonical comment body
  (`Comment.cached_excerpt`) — in the D-04 plain-content format. Phase 70 re-resolves
  from a fresh tree→blob read with this cached excerpt as fallback.
- **L-05:** **Merge commits ARE valid for full-file-source capture.** Phase 67 L-07
  disabled diff-source anchoring on merge commits; Phase 68 explicitly does **not**
  mirror that disable (ROADMAP §66 D-08 / §67 L-07: "Full-file-source review of a
  merge remains valid"). The full-file blob at a merge commit is unambiguous (the
  file as it exists at that commit). Do not reflexively copy Phase 67's
  `isMerge`-disable into the full-file affordance.

### Claude's Discretion (delegated to planner)
- **Full-file adapter API shape & location:** a `buildFullFileAnchor` analog of
  `buildDiffAnchor` operating on the flat line list (not `file.hunks[hunkIdx]`);
  whether it lives in a new `src/lib/full-file-anchor.ts` or extends
  `src/lib/diff-anchor.ts`. Keep it pure and tested like the existing adapter.
- **`CommentComposer` adaptation:** the existing composer is coupled to
  `buildDiffAnchor` + `hunkIdx` + `selectedLineIndices`. Whether to parameterize it
  by an injected "captured anchor/excerpt" result vs. add a full-file mode is the
  planner's call — but reuse it (D-01 affordance/composer carries Phase 67 D-01).
- **Selection-state ownership & wiring:** new state in `FullFileView` vs. a
  parent-owned `$state` passed down (mirror however the diff path threads
  `selectedLineIndices`/`oncommentlines` through `DiffViewer`).
- **Empty/zero-hunk file:** an unchanged file produces zero hunks → nothing to
  select; the Comment affordance simply doesn't appear (no special UI).
- **Attach-success feedback:** clear composer + selection on submit; rely on the
  existing `session-changed` event → panel reload (project's mostly-silent-success
  convention). Optional toast is the planner's call.
- **Empty-text submit** is disabled (no zero-text comments), matching Phase 67.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone & phase spec
- `.planning/ROADMAP.md` §"Phase 68: Full-File-Source Anchor Capture" — goal, the
  two success criteria, and the locked Notes: the full-file view is a 100k-context
  diff NOT a blob read (zero hunks for unchanged files, line-number skips at
  dropped-hunk boundaries, `from_utf8_lossy` content), use it only to LET THE USER
  PICK lines, persist absolute blob line numbers on the `new` side, render re-reads
  from git2 tree→blob (Phase 70). Also read §"Phase 67" Notes (shared `add_comment`,
  the L-01..L-08 capture rules this phase inherits/diverges from) and §"Phase 70"
  (DOC-02 language-fenced full-file render that the cached excerpt format feeds).
- `.planning/REQUIREMENTS.md` — **ANCH-02** (this phase), the downstream
  CMT → DOC-02/03/04 → OUT chain the anchor feeds, and the "Out of Scope" table
  (no threading, no severity tags, no re-anchoring on history rewrite, static
  snapshot).

### Phase 67 (the diff-source sibling — shared command, divergent excerpt rules)
- `.planning/phases/67-diff-source-anchor-capture/67-CONTEXT.md` — D-01 (inline
  composer affordance, reused), D-02 (confirm-before-discard draft), L-08 (shared
  `add_comment`), and L-03/L-06 (diff-format excerpt + keep `-` lines) which Phase 68
  **intentionally diverges from** (D-02/D-04 above).
- `src/lib/diff-anchor.ts` — `buildDiffAnchor` / `resolveSide` / `prefixLine`: the
  pure capture-time adapter pattern (and tests in `diff-anchor.test.ts`) the
  full-file adapter mirrors. Note it operates on `file.hunks[hunkIdx]` — the
  full-file analog must operate on the flat line list.
- `src/components/diff/CommentComposer.svelte` — the existing inline composer
  (props: `file`, `hunkIdx`, `selectedLineIndices`, `commitOid`, `repoPath`,
  `onclose`); draft debounce + `save_draft_comment` + `add_comment` submit wiring to
  reuse/adapt.

### Phase 65 keystone (the schema + store this phase writes into — FROZEN)
- `.planning/phases/65-data-model-persistence-session-lifecycle/65-CONTEXT.md` —
  D-01 (anchor = source coords, never index/options), D-04 (text stored independent
  of resolvability; cache excerpt at attach), DP-02 (draft on the persisted
  session).
- `src-tauri/src/git/types.rs:288-336` — frozen review schema: `Source{Diff,
  FullFile}`, `Side{Old,New}`, `Anchor`, `Comment{text, anchor, cached_excerpt}`,
  `DraftComment{text, anchor}`, `ReviewSession`.
- `src-tauri/src/git/types.rs:187-223` — `DiffLine{origin, content, old_lineno,
  new_lineno, spans}`, `DiffHunk`, `DiffStatus`, `FileDiff{path, status, is_binary,
  hunks}` — the inputs the full-file adapter reads (`new_lineno` per line).
- `src-tauri/src/commands/review.rs` — `add_comment` / `save_draft_comment`
  (already shipped, L-02); `_inner(data_dir, …)` + thin `#[tauri::command]` wrapper
  + `app.emit("session-changed", canonical)` pattern.

### Codebase patterns to mirror
- `src/components/diff/FullFileView.svelte` — the capture surface; **has no
  selection today**, renders `fd.hunks.flatMap(h => h.lines)` as a flat continuous
  list with per-line `old_lineno`/`new_lineno` gutters. Net-new selection state +
  Comment affordance + composer mount go here.
- `src/components/diff/DiffViewer.svelte` — how `selectedLineIndices`,
  `oncommentlines`, etc. are threaded to `HunkView`/`SplitView`; `FullFileView` is
  currently passed only `{fileDiffs} {showInvisibles} {wordWrap}` (line ~118) and
  must be extended.
- `src/components/diff/HunkView.svelte` — the on-selection floating "Comment
  ({selectedCount})" button (~lines 305-327) the full-file affordance parallels
  (but WITHOUT the `isMerge`-disable, per L-05).
- `.planning/codebase/CONVENTIONS.md`, `STACK.md`, `ARCHITECTURE.md` — Rust/Svelte
  conventions, git2-only, theme CSS custom properties (no inline colors),
  `safeInvoke<T>` for all IPC.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`CommentComposer.svelte`** (Phase 67): inline expanding textarea, 300ms draft
  debounce → `save_draft_comment`, submit → `add_comment`, live anchor/excerpt
  preview via the adapter. Reused with a full-file adapter swapped in.
- **`buildDiffAnchor`** (`src/lib/diff-anchor.ts`): the pure adapter to mirror; the
  full-file version is simpler (always `side = New`, plain content, no `prefixLine`).
- **Shared backend** (`add_comment` / `save_draft_comment` in `review.rs`,
  `save_session` atomic tmp+rename in `review_store.rs`, `session-changed` emit):
  done in Phase 65/67 — full-file capture just calls them.
- **v0.12 full-file render** (`FullFileView.svelte`): the line/gutter layout to add
  selection highlighting onto (theme CSS vars for the selection background).

### Established Patterns
- **Pure, tested capture adapter** (`diff-anchor.ts` + `diff-anchor.test.ts`): the
  full-file adapter follows the same shape (inputs → `{ anchor, cachedExcerpt }`).
- **`_inner` + thin command wrapper + emit** in `review.rs` (no new command
  expected, but any helper follows this).
- **serde conventions:** review-schema enums serialize PascalCase with NO
  `rename_all`; frontend-facing request types use camelCase via `rename_all`.
- **Theme CSS custom properties only** — selection highlight uses a `--color-*`
  var, never an inline color.

### Integration Points
- `FullFileView.svelte` — add selection state (D-01), the Comment affordance, and
  the composer mount; exclude Delete lines from selectable endpoints (D-02).
- `DiffViewer.svelte` — extend the `FullFileView` invocation to thread
  selection/comment props (commit oid, repo path, `oncommentlines`-equivalent).
- The commit full-file data path (`contentMode === "full"`, `diffKind === "commit"`)
  provides the `FileDiff` whose lines' `new_lineno` the adapter reads.

</code_context>

<specifics>
## Specific Ideas

- The recipient of the eventual rendered doc is an **AI coding agent** — the cached
  excerpt + comment text IS the instruction. Keep capture lean (no severity,
  author, threading).
- "Full-file source" specifically means the v0.12 full-file-at-commit content view,
  a distinct source from the hunk/split diff (Phase 67). The `source = FullFile`
  enum value is set at capture, which is exactly why `add_comment` is parameterized.
- The full-file frame is deliberately "the file **as it exists at the commit**" —
  this is why deletes are non-selectable (D-02) and the excerpt is plain new-side
  content (D-04), unlike the diff-source path.

</specifics>

<deferred>
## Deferred Ideas

- **Full-file gutter markers / in-place comment browser** — Phase 69 (Comment
  Management UI / CMT-04 jump-to-anchor). Phase 68 only confirms a successful attach.
- **Fresh-blob excerpt re-resolution + "unresolvable" section** — Phase 70
  (DOC-04). The cached plain-content excerpt (D-04) exists as the fallback.
- No scope-creep ideas surfaced — discussion stayed within the phase boundary.

### Reviewed Todos (not folded)
- **`2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md`**
  — matched on keywords "commit"/"state" (score 0.6) but is about merge/revert
  *commit-message editing*, unrelated to review-session anchor capture. Already
  reviewed and declined in Phases 66 and 67; not folded.

</deferred>

---

*Phase: 68-full-file-source-anchor-capture*
*Context gathered: 2026-05-25*
