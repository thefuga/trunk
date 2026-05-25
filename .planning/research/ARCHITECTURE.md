# Architecture Research

**Domain:** Desktop Git GUI — Code Review Mode feature (Tauri 2 + Svelte 5 + Rust + git2)
**Researched:** 2026-05-25
**Confidence:** HIGH

This document maps how the v0.13 Code Review Mode integrates with Trunk's existing
architecture. Every integration point is grounded in a real existing module/pattern.
NEW vs MODIFIED is marked explicitly so the roadmapper can map phases cleanly.

---

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                         Svelte 5 Frontend                              │
├──────────────────────────────────────────────────────────────────────┤
│  RepoView.svelte (layout host — MODIFIED: adds Review mode toggle)     │
│    center pane                          right pane                     │
│  ┌──────────────────────────┐   ┌────────────────────────────────┐   │
│  │ DiffPanel / DiffViewer   │   │ CommitDetail | DiffPanel        │   │
│  │  HunkView   (MODIFIED:   │   │   ──OR── (when Review active) ── │   │
│  │   add-comment action)    │   │ ReviewPanel.svelte    [NEW]     │   │
│  │  FullFileView (MODIFIED: │   │  - included commit list         │   │
│  │   add line-selection)    │   │  - comment list (jump/edit/del) │   │
│  └──────────────────────────┘   │  - "Generate Markdown" button   │   │
│  CommitGraph/CommitRow           └────────────────────────────────┘   │
│   (MODIFIED: context-menu                                              │
│    "Add to review")          review-session.svelte.ts [NEW $state]     │
├──────────────────────────────────────────────────────────────────────┤
│              safeInvoke<T>  (existing IPC wrapper, reused)             │
├──────────────────────────────────────────────────────────────────────┤
│                          Rust Backend                                  │
│  commands/review.rs [NEW]                                              │
│   start_review  resume_review  end_review  save_review                 │
│   add_comment   edit_comment   delete_comment                         │
│   seed_review_range (revwalk)  add_commits_to_review                   │
│   resolve_excerpt   render_review_markdown                            │
│                                                                        │
│  git/review.rs [NEW]   pure logic: revwalk, excerpt extraction,       │
│                        markdown assembly, anchor resolution            │
│  reuses: git/syntax.rs, diff.rs walk_diff, graph::walk_commits        │
├──────────────────────────────────────────────────────────────────────┤
│  Managed state:  RepoState  CommitCache  WatcherState  RunningOp      │
│                  ReviewSessionsState [NEW: Mutex<HashMap<path,Sess>>]  │
├──────────────────────────────────────────────────────────────────────┤
│  Persistence:  app_data_dir/review-sessions/<sha256(path)>.json [NEW] │
│                (Rust-owned serde_json, atomic tmp+rename)              │
└──────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | New / Modified |
|-----------|----------------|----------------|
| `commands/review.rs` | Tauri command wrappers (thin) for session lifecycle, comments, seeding, excerpt, render | NEW |
| `git/review.rs` | Pure logic: revwalk range, excerpt extraction from diff/blob, markdown assembly | NEW |
| `ReviewSessionsState` | `Mutex<HashMap<repo_path, ReviewSession>>` managed state, mirrors `CommitCache` | NEW |
| `review-session.svelte.ts` | Shared `$state` rune module: active session, comments, included commits, mode flag | NEW |
| `ReviewPanel.svelte` | Right-pane review surface: commit list, comment list, jump/edit/delete, render button | NEW |
| `CommentEditor` (InputDialog reuse or small NEW component) | Capture/edit comment body for an anchor | NEW (reuses InputDialog pattern) |
| `RepoView.svelte` | Hosts Review mode toggle; swaps right-pane content | MODIFIED |
| `HunkView.svelte` | Adds "Add comment" action on a line selection (alongside Stage/Unstage Lines) | MODIFIED |
| `FullFileView.svelte` | Gains line-range selection state (none today) | MODIFIED |
| `CommitRow.svelte` / `CommitGraph.svelte` | Context-menu item "Add commit(s) to review" | MODIFIED |
| `diff.rs::walk_diff`, `git/syntax.rs`, `graph::walk_commits` | Reused at render/seed time | REUSED |

---

## Recommended Project Structure

```
src-tauri/src/
├── commands/
│   ├── review.rs            # NEW — thin #[tauri::command] wrappers (inner-fn pattern)
│   └── mod.rs               # MODIFIED — add `pub mod review;`
├── git/
│   ├── review.rs            # NEW — pure: revwalk, excerpt, markdown, anchor resolve
│   └── types.rs             # MODIFIED — add ReviewSession/ReviewComment/Anchor structs
├── state.rs                 # MODIFIED — add ReviewSessionsState
└── lib.rs                   # MODIFIED — .manage(ReviewSessionsState), register commands

src/
├── lib/
│   ├── review-session.svelte.ts  # NEW — shared $state rune module
│   └── types.ts                  # MODIFIED — TS mirrors of the new Rust DTOs
└── components/
    ├── ReviewPanel.svelte        # NEW — right-pane review surface
    ├── ReviewCommentRow.svelte   # NEW — single comment in the list
    └── (RepoView/HunkView/FullFileView/CommitRow modified in place)
```

### Structure Rationale

- **`commands/review.rs` + `git/review.rs` split:** Matches the established inner-fn
  pattern (`commands/diff.rs` thin wrappers, pure logic testable without Tauri runtime).
  All review logic — revwalk, excerpt, markdown — lives in `git/review.rs` as pure
  functions taking `&git2::Repository`, unit-testable via the GOOS harness.
- **`ReviewSessionsState` mirrors `CommitCache`:** Same `Mutex<HashMap<String, _>>`
  shape, same per-repo keying, same `spawn_blocking` + clone-the-map access pattern.
- **`review-session.svelte.ts` mirrors `remote-state.svelte.ts`:** Shared `$state` rune
  module is the established cross-component channel; ReviewPanel (right pane), HunkView
  (center), and CommitRow (graph) all touch the active session without prop drilling.
- **Render in Rust, not TS:** `git/review.rs` already has the repo handle for excerpt
  resolution; assembling the markdown there avoids a second IPC round-trip and keeps the
  excerpt+format logic in one testable place.

---

## Data Model

Concrete enough to implement. Rust structs in `git/types.rs`; TS mirrors in `types.ts`
(string-literal unions for enums per the existing convention).

### Rust (`git/types.rs`)

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct ReviewSession {
    pub schema_version: u32,          // = 1; bump on breaking change
    pub repo_path: String,            // canonical abs path, stored for debuggability
    pub base_ref: Option<String>,     // e.g. "main" or a SHA; None for pure hand-pick
    pub tip_ref: Option<String>,      // e.g. "HEAD" or a SHA
    pub included_commits: Vec<String>,// ordered commit SHAs in the review
    pub comments: Vec<ReviewComment>,
    pub created_at: i64,              // unix seconds
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReviewComment {
    pub id: String,                  // uuid/v4 or monotonic — frontend-generated ok
    pub body: String,                // markdown comment text
    pub anchor: Option<Anchor>,      // None = commit-level comment (commit_sha required)
    pub commit_sha: String,          // always present (commit-level or anchor's commit)
    pub cached_excerpt: Option<String>, // canonical excerpt captured at attach-time
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "source")]            // serde tagged enum -> clean TS union + versioning
pub enum Anchor {
    Diff {
        commit_sha: String,
        file_path: String,
        // diff lines live in TWO line-number spaces (old/new). Be explicit.
        side: DiffSide,             // Old | New | Both
        old_range: Option<(u32, u32)>, // inclusive start..=end in old file space
        new_range: Option<(u32, u32)>, // inclusive start..=end in new file space
    },
    FullFile {
        commit_sha: String,
        file_path: String,
        start_line: u32,            // 1-based line in the blob at commit_sha
        end_line: u32,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DiffSide { Old, New, Both }
```

### TS mirror (`types.ts`)

```typescript
export type DiffSide = "Old" | "New" | "Both";

export type Anchor =
  | { source: "Diff"; commit_sha: string; file_path: string;
      side: DiffSide; old_range: [number, number] | null;
      new_range: [number, number] | null; }
  | { source: "FullFile"; commit_sha: string; file_path: string;
      start_line: number; end_line: number; };

export interface ReviewComment {
  id: string; body: string; anchor: Anchor | null;
  commit_sha: string; cached_excerpt: string | null;
  created_at: number; updated_at: number;
}

export interface ReviewSession {
  schema_version: number; repo_path: string;
  base_ref: string | null; tip_ref: string | null;
  included_commits: string[]; comments: ReviewComment[];
  created_at: number; updated_at: number;
}
```

**Why the Anchor is a tagged enum with explicit old/new ranges (key decision):**
A diff line carries BOTH `old_lineno` and `new_lineno` (see `DiffLine` in `types.ts`).
A single `(start, end)` range is ambiguous when a selection spans deletes and adds —
they live in different line-number spaces. Modeling `side` + `old_range` + `new_range`
makes the renderer's diff-fenced output deterministic without re-deriving intent.

---

## Persistence Layer

**Decision: Rust-owned, one JSON file per repo, NOT LazyStore.** (HIGH confidence)

- **Path:** `app_data_dir()/review-sessions/<sha256(canonical_abs_repo_path)>.json`.
  `app_data_dir` is reachable from the `app` handle (already used in `lib.rs` setup) and
  from commands via `AppHandle`/`tauri::Manager`.
- **Format:** `serde_json` serialization of `ReviewSession`, with `schema_version: 1` at
  the root for forward-compat. The repo path is stored inside the file for debuggability
  (the filename is an opaque hash).
- **Atomic write:** write to `<hash>.json.tmp`, then `fs::rename` over the target — avoids
  torn writes on crash. Trivial from Rust; awkward via LazyStore.
- **In-memory mirror:** `ReviewSessionsState(Mutex<HashMap<String, ReviewSession>>)`,
  managed in `lib.rs` exactly like `CommitCache`. Load-on-start/resume, mutate in memory,
  flush to disk on every mutating command (cheap; sessions are small).

**Why Rust-owned over LazyStore:**

| Concern | Rust serde_json file | LazyStore (frontend KV) |
|---------|----------------------|--------------------------|
| Nested mutating document | Natural (one struct) | Awkward (flat KV, manual nesting) |
| Schema versioning | `#[serde(tag)]` enums + version field | Hand-rolled migrations in TS |
| Atomic write | tmp + rename in one place | Plugin-controlled, less direct |
| Validate anchors on load | Has repo handle already | Would need extra IPC |
| Excerpt resolution co-location | Same module as render | Split across layers |

LazyStore stays correct for **UI preferences** (review mode last-open flag, panel width)
— that is its established role (`store.ts`). Session documents are not UI prefs.

**Lifecycle (matches "persist until render" decision):**
`start_review`/`seed_review_range` create the file → mutations update it →
`render_review_markdown` produces the snapshot (the generated `.md` is a static copy, never
re-synced) → `end_review` deletes the file and clears the map entry. One active session per
repo = one file per repo.

---

## Architectural Patterns

### Pattern 1: Inner-fn command + pure git logic (REUSED)

**What:** Thin `#[tauri::command]` wrapper clones the state map, runs the pure `_inner`
fn inside `spawn_blocking`; the `_inner` fn opens a fresh `git2::Repository`.
**When:** Every review command (`seed_review_range`, `resolve_excerpt`, `render_review_markdown`).
**Trade-offs:** Slight `Repository::open` cost per call (already accepted project-wide);
in exchange every function is unit-testable via the GOOS harness with real git repos.

```rust
pub fn render_review_markdown_inner(
    path: &str,
    session: &ReviewSession,
    state_map: &HashMap<String, PathBuf>,
) -> Result<String, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;   // same helper as diff.rs
    git::review::assemble_markdown(&repo, session)        // pure, testable
}
```

### Pattern 2: Shared `$state` rune module for cross-pane session state (REUSED)

**What:** `review-session.svelte.ts` exposes a `$state` object (active session, mode flag,
selection-in-progress). Mirrors `remote-state.svelte.ts` / per-tab factory pattern.
**When:** ReviewPanel (right), HunkView/FullFileView (center), CommitRow (graph) all need
the active session without prop drilling.
**Trade-offs:** Must be per-tab (factory function, like remote/undo-redo state) so multiple
repo tabs hold independent sessions.

### Pattern 3: Cache-at-attach, re-resolve-at-render with graceful fallback (NEW, key decision)

**What:** When a comment is attached, capture the excerpt immediately into
`cached_excerpt` (the canonical body). At render time, re-resolve the excerpt from git as a
freshness check; if the anchor no longer resolves (history rewrite, deleted file), fall
back to the cached excerpt and mark the anchor `[unresolved at render]` in the markdown.
**Why:** Satisfies both spec invariants — "static snapshot" (cached canonical body) AND
"render-time surfaces unresolvable anchors gracefully" (re-resolve attempt). No crash on
rewrite; no silent loss of the reviewer's context.
**Trade-offs:** Stores excerpt text in the session file (slightly larger), but that is the
whole point — the review must survive a moved HEAD.

### Pattern 4: Selection adapter — hunk-index → lineno-based Anchor (NEW)

**What:** Existing `HunkView` selection is `selectedLineIndices: Set<number>` indexing into
`hunk.lines[]` — NOT line numbers. A NEW converter maps selected indices to
`(side, old_range, new_range)` by reading each line's `old_lineno`/`new_lineno`/`origin`.
**Why explicit:** This is **adapted, not reused as-is**. The staging line-selection
machinery (`stage_lines`) proves the index→lineno transform is feasible, but review needs
the lineno output, not the staging patch output.

---

## Data Flow

### Seeding a session from a range

```
User picks base→tip (ReviewPanel)
   → safeInvoke("seed_review_range", {path, base, tip})
   → seed_review_range_inner: git2 revwalk push(tip) hide(base)
   → ordered Vec<sha> → ReviewSession.included_commits
   → write file + insert into ReviewSessionsState
   → return session → review-session.svelte.ts updates → ReviewPanel renders
```

### Hand-picking commits from the graph

```
Right-click CommitRow → context menu "Add to review"
   → safeInvoke("add_commits_to_review", {path, shas})
   → merge into included_commits (dedupe, preserve graph order)
   → write file → return session → rune module updates
```

### Attaching a comment (diff source)

```
Select lines in HunkView (existing selection) → "Add comment" action
   → adapter builds Anchor::Diff {side, old_range, new_range, file, commit}
   → CommentEditor captures body
   → safeInvoke("add_comment", {path, anchor, body})
   → add_comment_inner: resolve_excerpt(repo, anchor) -> cached_excerpt
   → push ReviewComment → write file → return session
```

### Rendering the markdown

```
"Generate Markdown" (ReviewPanel)
   → safeInvoke("render_review_markdown", {path})
   → for each comment: re-resolve excerpt (fallback to cached on failure)
   → assemble: per-commit refs + code excerpts (diff-fenced for source=Diff,
     ```lang fences for source=FullFile via syntax::extension_from_path) + comment bodies
   → AI-framed preamble → return String
   → ReviewPanel: clipboard plugin (copy) OR dialog plugin save() (write file)
```

### Key Data Flows

1. **Session is purely user-mutated.** It subscribes to NO repo events. `repo-changed`
   (commit/checkout/rebase) does NOT touch the session — anchors are immutable by design.
   Re-anchoring on history rewrite is explicitly out of scope; render-time resolution
   handles staleness.
2. **Excerpt extraction is git-read-only.** `source=Diff` re-runs the relevant
   `diff_tree_to_tree` (parent→commit) and slices the hunk; `source=FullFile` reads the
   blob at `(commit_sha, file_path)` via `repo.find_commit().tree().get_path().to_object()`
   and slices `start_line..=end_line`.

---

## Excerpt Extraction (detail)

Pure functions in `git/review.rs`, command surface `resolve_excerpt` + used inside render.

- **source = FullFile:** `find_commit(sha).tree().get_path(file).to_object().as_blob()`
  → split content by lines → slice `start_line..=end_line` (1-based) → fence with the
  language from `syntax::extension_from_path(file)` (reuses existing detection).
- **source = Diff:** rerun `diff_tree_to_tree(parent_tree, commit_tree)` with `pathspec(file)`
  (same as `diff_commit_file_inner`), walk to the hunk(s) overlapping `old_range`/`new_range`,
  emit the `+`/`-`/context lines as a ```diff fence. Reuse `walk_diff`'s line iteration
  shape; excerpt does not need syntax enrichment (diff fence is plain).
- **Unresolved (rewrite / file gone):** `find_commit` or `get_path` errors → return a
  typed "unresolved" marker; render substitutes `cached_excerpt` and annotates the section.

---

## UI Surface (high-level — UI phase will detail)

**Recommendation (ONE option, opinionated):** A **"Review" mode toggle that replaces the
right-pane content** with `ReviewPanel.svelte`. The center pane (DiffViewer / FullFileView)
stays as-is for selecting anchors; the right pane swaps from CommitDetail/DiffPanel to the
review session panel (included-commit list + comment list + Generate button).

**Why this over alternatives:**
- **Replace right pane (RECOMMENDED):** The right pane already swaps between CommitDetail
  and DiffPanel in `RepoView`. Adding one more swappable surface is the smallest change and
  keeps the center diff/full-file view — exactly where anchors are captured — fully intact.
- **New dedicated pane/column:** Rejected — the 6-column layout is already dense; a fourth
  vertical region hurts the diff reading width that review depends on.
- **Floating overlay:** Rejected — comments need to coexist with the diff for jump-to-anchor;
  an overlay fights the center view it must reference.

Mode flag lives in `review-session.svelte.ts`; `RepoView` reads it to choose right-pane
content. Graph context-menu "Add to review" and HunkView/FullFileView "Add comment" are
gated on the active session existing.

---

## Anti-Patterns (review-mode-specific)

### Anti-Pattern 1: Auto-re-anchoring on history rewrite

**What people do:** Try to keep anchors valid after rebase/amend by re-mapping line numbers.
**Why it's wrong:** Line-number re-mapping across a rewrite is unreliable and explicitly out
of scope; it invites silent corruption of the reviewer's intent.
**Do this instead:** Treat anchors as immutable. Re-resolve at render, fall back to
`cached_excerpt`, annotate unresolved anchors. Session subscribes to no repo events.

### Anti-Pattern 2: Storing excerpts only by reference to live git state

**What people do:** Store just the anchor, fetch the excerpt fresh every time.
**Why it's wrong:** A moved HEAD or deleted file loses the reviewed code — the "snapshot"
guarantee breaks.
**Do this instead:** Cache the excerpt at attach-time as the canonical body (Pattern 3).

### Anti-Pattern 3: Persisting the session in LazyStore as flat keys

**What people do:** Reuse the UI-prefs store for the session document.
**Why it's wrong:** LazyStore is a flat KV for prefs; nested mutating documents, versioning,
and atomic writes don't fit. Mixing review drafts into `trunk-prefs.json` couples unrelated
lifecycles.
**Do this instead:** Rust-owned per-repo JSON file with `schema_version` + atomic write.

### Anti-Pattern 4: Reusing HunkView's index-based selection as the anchor directly

**What people do:** Persist `selectedLineIndices` (hunk-relative indices) as the anchor.
**Why it's wrong:** Indices are meaningless across diff-option changes (context lines) and
carry no line-number space. Re-resolution at render would be impossible.
**Do this instead:** Convert to `(side, old_range, new_range)` at attach-time (Pattern 4).

---

## Integration Points

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| ReviewPanel ↔ session state | `review-session.svelte.ts` ($state rune) | Per-tab factory, like remote-state |
| HunkView/FullFileView ↔ session | rune module + safeInvoke("add_comment") | Selection adapter builds Anchor |
| CommitRow ↔ session | context menu → safeInvoke("add_commits_to_review") | Reuses graph order |
| review.rs ↔ git2 | fresh Repository per command (spawn_blocking) | Same as diff.rs / history.rs |
| review.rs ↔ syntax.rs | `extension_from_path` for full-file fence language | REUSED |
| review.rs ↔ diff walk | `diff_tree_to_tree` + line walk for diff excerpt | Pattern from diff_commit_file_inner |
| state.rs ↔ lib.rs | `.manage(ReviewSessionsState)` + register commands | Mirrors CommitCache |
| Output ↔ OS | plugin-clipboard-manager (copy), plugin-dialog (save) | Both already in Cargo.toml |

### External Services

None. No GitHub/GitLab posting (out of scope). Local markdown only.

---

## Scaling Considerations

Single-user desktop app; N/A. Sessions are tiny (a handful of commits + comments + cached
excerpts). The only practical bound is excerpt size for huge selections — cap selection
range in the UI phase if needed.

---

## Suggested Build Order (→ roadmap phases)

Respects dependencies: model/persistence first, then selection/commenting, then render/output.

1. **Foundation: data model + persistence + session lifecycle.**
   `ReviewSession`/`ReviewComment`/`Anchor` structs + TS mirrors; `ReviewSessionsState`;
   `app_data_dir` JSON file (sha256 path, schema_version, atomic write); commands
   `start_review` / `resume_review` / `end_review` / `save_review`. *(Depends on: nothing.)*

2. **Commit selection.** `seed_review_range` (git2 revwalk push(tip)/hide(base)) +
   `add_commits_to_review` from graph context menu (`CommitRow` MODIFIED). ReviewPanel
   skeleton shows the included-commit list. *(Depends on: 1.)*

3. **Diff-source anchor capture.** Adapt `HunkView` selection → `Anchor::Diff`
   (index→lineno converter, NEW); "Add comment" action; `add_comment` with attach-time
   excerpt caching. *(Depends on: 1; 2 for an active session.)*

4. **Full-file-source anchor capture.** Add line-range selection to `FullFileView`
   (net-new selection state) → `Anchor::FullFile`; same add_comment path.
   *(Depends on: 1. Independent of 3 — could swap or parallelize.)*

5. **Comment management UI.** ReviewPanel comment list: list / edit / delete / jump-to-anchor;
   optional commit-level comment (no anchor). `edit_comment` / `delete_comment`.
   *(Depends on: 3 and/or 4.)*

6. **Excerpt resolution + markdown render.** `resolve_excerpt` (full-file blob slice +
   diff hunk slice) and `render_review_markdown` (AI-framed doc, diff/lang fences,
   re-resolve with cached fallback). *(Depends on: 1, 3, 4.)*

7. **Output.** Copy-to-clipboard (clipboard plugin) + save-to-file (dialog plugin) in
   ReviewPanel; mark snapshot generated. *(Depends on: 6. Can collapse into 6 for a
   6-phase roadmap.)*

Phases 3 and 4 are independent. 6+7 may merge if a tighter roadmap is preferred.

---

## Sources

- Codebase (HIGH): `src-tauri/src/commands/diff.rs`, `history.rs`, `staging.rs`, `mod.rs`;
  `src-tauri/src/state.rs`, `lib.rs`; `src/lib/types.ts`, `store.ts`, `invoke.ts`,
  `remote-state.svelte.ts`; `src/components/RepoView.svelte`, `diff/HunkView.svelte`,
  `diff/FullFileView.svelte`.
- `.planning/PROJECT.md` (HIGH) — v0.13 decisions, established patterns, constraints.
- `Cargo.toml` (HIGH) — confirms `tauri-plugin-clipboard-manager`, `tauri-plugin-dialog`,
  `tauri-plugin-store`, `git2 0.19` available.

---
*Architecture research for: Trunk Code Review Mode (v0.13)*
*Researched: 2026-05-25*
