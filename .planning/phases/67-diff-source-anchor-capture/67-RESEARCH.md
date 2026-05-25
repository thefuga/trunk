# Phase 67: Diff-Source Anchor Capture - Research

**Researched:** 2026-05-25
**Domain:** Diff-selection → stable source-coordinate anchor capture (Tauri 2 + Svelte 5 + git2/libgit2), writing into the frozen Phase 65 review schema/store
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Attach Flow & Composer**
- **D-01:** Inline composer in the diff. A floating "Comment" button appears on the active diff selection (the same on-selection action affordance that surfaces Stage/Unstage/Discard today); clicking it opens an expanding textarea directly under the selected lines (GitHub-PR style). Chosen over routing to the review panel because it keeps the comment visually tied to the code and does **not** depend on the Phase 69 panel shape. NOTE: commit diffs currently render no Stage/Unstage/Discard buttons, so the Comment affordance is net-new in that context.

**Draft Collision**
- **D-02:** Confirm before discard. The schema has exactly one `draft_comment` slot and drafts persist on every keystroke (L-05), so a half-typed draft is durable state. Selecting a *new* range while the current draft has non-empty text prompts "Discard your unsaved comment?" before switching. An **empty** draft switches silently. Mirrors the existing confirm-on-discard pattern (`DiffPanel.handleDiscardLines` → `@tauri-apps/plugin-dialog` `ask`).

**Selection → Range Collapse**
- **D-03:** Collapse to min–max on the chosen side, with a preview. The anchor stores a single `start_line..end_line` range, but a v0.7 selection can be a non-contiguous `Set` within one hunk. Capture collapses to `min..max` of the source line numbers on the chosen `side`, and the composer shows a "Comments on lines N–M" preview. No rejection of non-contiguous selections.

**Blocked-Anchor Feedback**
- **D-04:** Disabled affordance + tooltip on merge commits. Diff-source anchoring is disabled on merge commits (carried from Phase 66 D-08 / L-07). On a merge commit's diff the Comment affordance renders **disabled** with a tooltip "Diff comments aren't available on merge commits." **File-status side constraints are NOT a "blocked" case** — Added/Deleted/Renamed simply *force* the side (L-04); the affordance stays enabled.

**Locked Carry-Forwards (do NOT re-litigate)**
- **L-01:** Persist the anchor as source coordinates only. **Never** persist `hunk_index` / `line_index` or the diff options (`context_lines` / `ignore_whitespace`). Schema frozen: `Anchor { commit_oid, file_path, source, side, start_line, end_line }`.
- **L-02:** A capture-time adapter translates selected diff indices → `(side, start_line, end_line)` via each line's `old_lineno` / `new_lineno` / `origin`. Add lines carry only `new_lineno`, Delete lines only `old_lineno`.
- **L-03:** For a selection mixing Added and Deleted lines, default to the `new` side and **drop pure-Delete lines from the persisted line range**, but keep them as `-` lines in the cached excerpt. Requires a `side` discriminator on every diff-source anchor.
- **L-04:** File-status constraints (`DiffStatus`): Added → `new`-side only, Deleted → `old`-side only, Renamed → store the **new** path with `new` side. Store the path as it exists at the anchored commit on the anchor's side.
- **L-05:** Persist the anchor **immediately on attach** (survives the watcher's `repo-changed` re-fetch) and persist the draft **on change**, not only on submit (`draft_comment` field already on the session).
- **L-06:** Cache the excerpt at attach-time as the canonical comment body (`Comment.cached_excerpt`), in diff format (with `-` context per L-03). Render (Phase 70) re-resolves from source coords with the cached excerpt as fallback — re-resolution and the "unresolvable" section are **Phase 70**, not here.
- **L-07:** Merge commits are disabled for diff-source anchoring. Full-file-source review of a merge remains valid (Phase 68).
- **L-08:** The `add_comment` command is **shared with Phase 68** — design it so `source`/`side` are parameters, not hard-coded to `Diff`.

### Claude's Discretion
- **Attach-success feedback:** on submit, clear the composer + selection; rely on `session-changed` → panel reload. A success toast is optional. Planner's call.
- **Empty-text submit** is disabled (no zero-text comments); exact validation point is the planner's.
- **Command surface & naming:** whether `add_comment` (and any draft-persist command) lives in `src-tauri/src/commands/review.rs` (recommended) or a new file; exact struct/command names following serde conventions.
- **Composer rendering across layout modes:** hunk/split layouts are both diff-source. The full-file content mode is Phase 68 territory; Phase 67's affordance lives in the hunk/split diff rendering.

### Deferred Ideas (OUT OF SCOPE)
- **In-diff comment browser** (gutter markers / badges / click-to-edit) — Phase 69.
- **Commit-level comments with no anchor** — Phase 69 (ANCH-03).
- **Render-time excerpt re-resolution + "unresolvable" section** — Phase 70 (DOC-04).
- Full-file-source capture — Phase 68. Markdown render — Phase 70. Clipboard/save — Phase 71.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ANCH-01 | User can select a line range in the diff view and attach a comment, anchored to commit + file + diff line range with a side discriminator | The TS capture adapter (Architecture Pattern 1) translates `selectedLineIndices` → `Anchor { side, start_line, end_line }` via `hunk.lines[idx].{origin,old_lineno,new_lineno}`; the shared `add_comment` Rust command (Pattern 2) persists it into the frozen Phase 65 store; the inline composer (Pattern 4) provides the attach UX; guard-lift across three files (Pattern 3) enables commit-diff selection. |
</phase_requirements>

## Summary

This phase has **no new external dependencies and no new persistence machinery** — the Phase 65 schema (`Anchor`/`Comment`/`DraftComment`/`ReviewSession`), the atomic-write store (`review_store.rs`), the `session-changed` event, the `safeInvoke` IPC wrapper, and the `@tauri-apps/plugin-dialog` `ask` confirm pattern all already exist and are verified in-tree. Phase 67 is overwhelmingly **wiring + one pure adapter + one thin persistence command**.

The single most important architectural finding: **the index→source adapter should live in TypeScript at capture time, not in Rust.** The frontend already holds the full `DiffLine[]` for the selected hunk, and `selectedLineIndices` are direct indices into that array (`PairedRow.lineIdx` preserves the original `hunk.lines` index in split view too). Every input the adapter needs — `origin`, `old_lineno`, `new_lineno` per line, plus `FileDiff.status` and `commitDetail.oid`/`parent_oids` — is already in memory on the frontend. Keeping the adapter in TS makes `add_comment` a *dumb persistence command*: it receives a fully-formed `Anchor` + `cached_excerpt` and just writes them. No diff re-derivation server-side, no extra IPC round-trip to re-send hunk lines, and the Rust `_inner` stays tiny and trivially testable.

The hard parts are **plumbing gaps**, not algorithms: (1) the commit diff is rendered by `DiffPanel` with `commitDetail={null}` hardcoded (RepoView.svelte:732) — the adapter needs `commitDetail` (for `oid` and merge detection) threaded through; (2) commit-diff line selection is disabled at **three** sites (DiffPanel, HunkView, SplitView), all of which must be lifted as one logical change; (3) split view can only select `Add` lines on the right column, so split-view commit diffs can only ever produce a `new`-side anchor.

**Primary recommendation:** Build a pure TS adapter (`src/lib/diff-anchor.ts`) that takes `(commitOid, FileDiff, hunkIdx, Set<lineIdx>)` → `{ anchor: Anchor, cachedExcerpt: string }`; add one thin shared `add_comment` Tauri command and one `save_draft_comment` command in `review.rs` following the existing `_inner` + emit pattern; thread `commitDetail` into the commit `DiffPanel`; lift the three commit-diff selection guards; render the Comment affordance + inline composer mirroring the existing hunk-header-toolbar action pattern.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Index→source coordinate translation (L-02) | Frontend (TS, capture-time) | — | All inputs (`DiffLine[]`, status, commit oid) already in frontend memory; keeping it client-side makes `add_comment` a pure writer. |
| Cached-excerpt assembly (L-06) | Frontend (TS, capture-time) | — | Same `DiffLine[]` source; excerpt is a string the backend persists verbatim. |
| Anchor persistence (write to store) | Backend (Rust command + store) | — | Disk I/O, atomic write, in-memory map mutation, `session-changed` emit — all owned by the Phase 65 store + command layer. |
| Draft persistence on keystroke (L-05) | Backend (Rust command) | Frontend (debounce optional) | The `draft_comment` slot lives on the persisted session; only the backend can write it. Frontend decides cadence. |
| Merge-commit disabling (L-07/D-04) | Frontend (affordance gate) | — | Pure UI gate on `commitDetail.parent_oids.length > 1`; no backend enforcement needed for capture. |
| File-status side constraint (L-04) | Frontend (adapter) | — | Derived from `FileDiff.status`, already in frontend memory. |
| Inline composer + affordance (D-01) | Frontend (Svelte component) | — | Pure UI, mounts in the diff rendering. |

## Standard Stack

No new packages. Every dependency below is already installed and in use.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte | 5 (runes) | Composer component, selection state | Project standard; `$state`/`$derived` for selection + draft text |
| Tauri | 2 | `add_comment` / draft command, `app.emit` | Project IPC layer |
| git2 | 0.19 | (read-only, already produces `DiffLine.old_lineno`/`new_lineno`) | git2-only rule; diff already built in `diff.rs` |
| serde / serde_json | (in tree) | Anchor/Comment (de)serialization | Frozen Phase 65 schema already derives Serialize+Deserialize |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@tauri-apps/plugin-dialog` | (in tree) | `ask()` confirm for D-02 discard-draft | Exact pattern at `DiffPanel.handleDiscardLines` |
| `@tauri-apps/api/event` | (in tree) | `listen("session-changed")` reload | Already consumed by `ReviewPanel.svelte` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| TS capture-time adapter | Rust command-side adapter | Rust would require re-sending the full hunk `DiffLine[]` over IPC (it does not retain the in-memory diff), re-deriving coordinates, and a fatter command. The frontend already holds every input. **Rejected** — strictly more work and more IPC for no correctness benefit. |
| Pass-through existing `commitDetail` to DiffPanel | Add two new props (`commitOid`, `isMerge`) | `commitDetail` is already loaded by the same `handleCommitSelect` that drives the commit diff (RepoView.svelte:313) and is RepoView `$state`. Pass-through is the smaller change. **Recommended:** pass `commitDetail` through (currently hardcoded `null` at line 732). |

**Installation:** None required.

## Package Legitimacy Audit

No new external dependencies. Phase 67 reuses the Phase 65 store, `@tauri-apps/plugin-dialog` `ask`, `@tauri-apps/api/event` `listen`, the existing `safeInvoke` IPC wrapper, and the frozen schema/serde derives. The audit table is omitted because no package is installed.

## Architecture Patterns

### System Architecture Diagram

```
                         COMMIT DIFF VIEW (diffKind="commit")
                                      │
  user clicks/​shift-clicks diff lines │  (guard lifted at 3 sites: DiffPanel/HunkView/SplitView)
                                      ▼
            DiffPanel.handleLineClick → selectedHunkKey + selectedLineIndices: Set<lineIdx>
                                      │
              clicks "Comment" affordance (disabled if commitDetail.parent_oids>1)
                                      ▼
        ┌────────────────────────────────────────────────────────────────┐
        │  TS CAPTURE ADAPTER  (src/lib/diff-anchor.ts — pure, no IPC)     │
        │  inputs: commitDetail.oid, FileDiff(.path,.status),              │
        │          hunk.lines: DiffLine[], selectedLineIndices: Set        │
        │  1. resolve side:                                                │
        │       status Added→New | Deleted→Old | Renamed→New(+new path)   │
        │       Modified → mixed-rule: any Delete+Add ⇒ New (drop deletes  │
        │                  from range); pure-Delete ⇒ Old; pure-Add ⇒ New  │
        │  2. start/end = min..max of (old|new)_lineno on chosen side      │
        │  3. cached_excerpt = `±/space`-prefixed lines over min..max idx  │
        │       range incl. dropped Delete lines as `-` (L-03/L-06)        │
        │  output: { anchor: Anchor, cachedExcerpt: string }              │
        └────────────────────────────────────────────────────────────────┘
                                      │
              composer textarea (every keystroke ─┐    │ on submit
                                                  ▼    ▼
                   safeInvoke("save_draft_comment")  safeInvoke("add_comment",
                   { path, draft:{text,anchor} })     { path, anchor, side, source, text, cachedExcerpt })
                                      │                       │
                                      ▼                       ▼
        ┌────────────────────────────────────────────────────────────────┐
        │  RUST  review.rs  (thin #[tauri::command] over _inner)          │
        │  add_comment_inner(data_dir, canonical, comment) :              │
        │    lock sessions → push Comment → clear draft_comment slot →    │
        │    review_store::save_session (atomic tmp+rename) → return      │
        │  save_draft_comment_inner: write/replace draft_comment slot     │
        └────────────────────────────────────────────────────────────────┘
                                      │  app.emit("session-changed", canonical)
                                      ▼
              ReviewPanel.svelte listen("session-changed") → reload (confirmation)
```

File-to-implementation mapping is in Component Responsibilities below — the diagram shows data flow only.

### Component Responsibilities

| Component / File | Responsibility | Change Type |
|------------------|----------------|-------------|
| `src/lib/diff-anchor.ts` (NEW) | Pure adapter: indices → `{anchor, cachedExcerpt}`; side resolution; excerpt assembly | new |
| `src/lib/diff-anchor.test.ts` (NEW) | Unit tests for the adapter (all selection shapes) | new |
| `src/components/diff/CommentComposer.svelte` (NEW, name planner's call) | Inline textarea + "Comments on lines N–M" preview + submit; keystroke→save_draft; submit→add_comment | new |
| `src/components/DiffPanel.svelte` | Lift `diffKind==="commit"` guard (line 307); host composer state; D-02 confirm-on-switch; accept `commitDetail` prop | edit |
| `src/components/diff/HunkView.svelte` | Lift `isSelectable` commit exclusion (line 303); add Comment affordance branch for `diffKind==="commit"` (disabled on merge) | edit |
| `src/components/diff/SplitView.svelte` | Lift commit exclusion (line 323, right col only); add Comment affordance branch | edit |
| `src/components/RepoView.svelte` | Pass existing `commitDetail` through to commit `DiffPanel` (line 732, currently `null`) | edit |
| `src-tauri/src/commands/review.rs` | `add_comment` + `add_comment_inner` (shared, source/side parameterized); `save_draft_comment` + `_inner` | edit |
| `src-tauri/src/lib.rs` | Register the two new commands in `invoke_handler` (after line 131) | edit |
| `src/lib/types.ts` | Add `Anchor`/`Comment`/`DraftComment`/`Source`/`Side` TS types if not present; request types for the two commands | edit |

### Pattern 1: TS capture-time adapter (the heart of L-02/L-03/L-04/L-06)

**What:** A pure function converting an in-memory diff selection to source coordinates + a cached diff excerpt.
**When to use:** At "Comment" submit and for the draft anchor.

```typescript
// Source: derived from src/lib/types.ts (DiffLine, FileDiff, DiffStatus) +
//         src-tauri/src/git/types.rs:288-336 (frozen Anchor/Source/Side schema)
// Shapes (VERIFIED in tree):
//   DiffLine { origin: "Context"|"Add"|"Delete"; old_lineno: number|null; new_lineno: number|null; content; spans }
//   FileDiff.status: "Added"|"Deleted"|"Modified"|"Renamed"|"Copied"|"Untracked"|"Unknown"
//   Anchor { commit_oid; file_path; source: "Diff"|"FullFile"; side: "Old"|"New"; start_line; end_line }

type Side = "Old" | "New";

// Side precedence (CRITICAL — file-status ALWAYS wins over mixed-selection default):
//   Added    -> "New"  (only new-side lines exist)
//   Deleted  -> "Old"  (only old-side lines exist)
//   Renamed  -> "New"  (+ store the new path)
//   Modified -> mixed rule (L-03): any Delete present alongside Add -> "New" and
//               drop the Delete lines from the persisted range; pure-Delete -> "Old";
//               pure-Add -> "New".
// Copied/Untracked/Unknown: treat as Modified for side derivation (verify in tests).

function resolveSide(status: string, selected: DiffLine[]): Side {
  if (status === "Added") return "New";
  if (status === "Deleted") return "Old";
  if (status === "Renamed") return "New";
  const hasAdd = selected.some((l) => l.origin === "Add");
  const hasDel = selected.some((l) => l.origin === "Delete");
  if (hasAdd) return "New";   // mixed or pure-Add -> New, deletes dropped from range
  if (hasDel) return "Old";   // pure-Delete -> Old
  return "New";               // defensive default
}

// start/end = min..max of the lineno field for the chosen side, over selected lines
// that actually have that side's lineno (Add has only new_lineno; Delete only old_lineno).
```

### Pattern 2: Shared `add_comment` Rust command (L-08, mirrors existing `_inner` + emit)

**What:** Thin `#[tauri::command]` over a testable `_inner`, source/side parameterized so Phase 68 reuses it.
**When to use:** On composer submit.

```rust
// Source: pattern mirrors src-tauri/src/commands/review.rs add_review_commit (399-416)
//         + mutate_session_rmw (304-320) for lock-across-read-mutate-save.
// Request type: camelCase via serde rename_all (frontend-facing); the persisted
// Anchor/Comment use snake_case + PascalCase enums (frozen Phase 65 schema).

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCommentRequest {
    pub path: String,
    pub text: String,
    pub anchor: Anchor,            // already source coords from the TS adapter
    pub cached_excerpt: String,    // -> Comment.cached_excerpt
}

// add_comment_inner(data_dir, canonical, sessions, req) under the sessions mutex:
//   1. get_mut session (no_session error if absent)
//   2. session.comments.push(Comment { text, anchor: Some(anchor), cached_excerpt: Some(excerpt) })
//   3. session.draft_comment = None;   // submit clears the single draft slot
//   4. review_store::save_session(...)  // atomic tmp+rename
// thin command: resolve_data_dir -> canonical_repo_path -> _inner -> app.emit("session-changed", canonical)
```

Because the command takes a fully-formed `Anchor` (which carries `source` and `side`), it is **already** Phase-68-shareable — Phase 68 passes `source: "FullFile"` with no command change. Add a Rust unit test that calls it with `Source::FullFile` to lock that contract.

### Pattern 3: Lifting the commit-diff selection guard (ONE logical change, THREE files)

**What:** Commit-diff line selection is currently disabled in three places. All must be lifted together, while keeping staging buttons absent in commit diffs.

```
1. src/components/DiffPanel.svelte:307   if (diffKind === "commit") return;   // early return in handleLineClick
2. src/components/diff/HunkView.svelte:303  isSelectable = diffKind !== 'commit' && line.origin !== 'Context'
3. src/components/diff/SplitView.svelte:323 isSelectable = diffKind !== 'commit' && line.origin === 'Add'  (right col only)
```

After lifting, selection works in commit diffs; staging buttons stay absent because they are gated on `diffKind === 'unstaged' | 'staged'` (HunkView:172/254, SplitView:237/271) with no `'commit'` branch — leave those untouched and add a separate Comment-affordance branch.

### Pattern 4: Inline composer + Comment affordance (D-01)

**What:** A "Comment" button on the active selection that expands a textarea.
**Where it mounts:** Mirror the existing on-selection action pattern, which lives **inside the hunk-header toolbar row** (HunkView:172-298 / SplitView:237-291) — not literally floating. D-01's word "floating" is loose; the locked decision cites this exact affordance, so match the hunk-header-toolbar placement and render the textarea below the selected lines. In **split view** the composer mounts in the right (`new`) panel context, consistent with right-column-only selection.

**Anti-Patterns to Avoid**
- **Persisting array indices.** Never put `hunk_index`/`line_index`/`context_lines`/`ignore_whitespace` on the anchor (L-01). The whole point is source coordinates.
- **Re-deriving the adapter in Rust.** The backend does not retain the in-memory diff; doing it server-side means re-sending the hunk over IPC for zero benefit.
- **Forgetting to clear the draft slot on submit.** `add_comment` must set `draft_comment = None` or the composer reopens with stale text.
- **Letting file-status lose to the mixed rule.** `Added`/`Deleted`/`Renamed` force the side unconditionally; the mixed-selection default only applies to `Modified`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic session persistence | Custom file write | `review_store::save_session` (tmp+sync_all+rename, FNV-1a keyed) | Already handles crash-safety, path-traversal, canonical keying (D-10/D-11) |
| Concurrent write safety | Ad-hoc locking | `mutate_session_rmw` pattern (review.rs:304) — lock across read→mutate→save | Two rapid clicks racing RMW lose writes otherwise (verified by `selection_rmw_serialized` test) |
| Cross-tab/window reload | Manual polling | `app.emit("session-changed", canonical)` + existing `listen` in ReviewPanel | Round-trip already wired and tested |
| Confirm-before-destroy dialog | Custom modal | `@tauri-apps/plugin-dialog` `ask(...)` | Exact pattern at `DiffPanel.handleDiscardLines:386` |
| IPC error parsing | Manual JSON parse | `safeInvoke<T>` | Parses `TrunkError`; project-wide convention |
| `old_lineno`/`new_lineno` side detection | Parsing diff text | git2 already populated them per line | `line.old_lineno()`/`new_lineno()` return `Option<u32>` (None on the absent side) — verified in diff.rs:235 |

**Key insight:** git2's `DiffLine` bindings already map libgit2's `-1` "not present on this side" sentinel to `None`/`null`. An **Add** line has `old_lineno: null, new_lineno: Some`; a **Delete** line has `old_lineno: Some, new_lineno: null`; **Context** has both `Some`. The adapter just reads the field for the chosen side — no sentinel handling, no diff-text parsing.

## Common Pitfalls

### Pitfall 1: DiffPanel has no commit OID or merge info for commit diffs
**What goes wrong:** The adapter needs `commit_oid` (for the anchor) and merge detection (for D-04 disabling), but the commit `DiffPanel` is instantiated with `commitDetail={null}` (RepoView.svelte:732).
**Why it happens:** Commit detail lives in the right-pane `CommitDetail` component; the center DiffPanel was never given it because staging/commit diffs didn't need it.
**How to avoid:** Pass the existing RepoView `commitDetail` (`$state`, loaded at line 313 in the same `handleCommitSelect`) through to the line-730 DiffPanel. Derive `commit_oid = commitDetail.oid`, `isMerge = commitDetail.parent_oids.length > 1`.
**Warning signs:** Anchor `commit_oid` is empty/undefined; merge commits don't disable the affordance.

### Pitfall 2: Split view cannot produce an Old-side anchor
**What goes wrong:** In split view the right column only allows selecting `Add` lines (SplitView:323) and the left column has no selection at all. A user expecting to comment on a deleted line in split view can't.
**Why it happens:** Inherited from Phase 64's staging model (you only stage Add lines on the new side).
**How to avoid:** Document as a real UX constraint, not a bug. It is consistent with L-03's `new`-side default. Pure `Old`-side comments require the inline (hunk) view, where both Add and Delete are selectable (HunkView:303 excludes only `Context`). Do **not** widen split-view selection in this phase.
**Warning signs:** A test asserting an `Old`-side anchor from split view — it can't happen by construction.

### Pitfall 3: Mixed selection range must exclude dropped Delete lines but excerpt must include them
**What goes wrong:** For a `Modified` file with a selection spanning Add + Delete, the persisted range is `min..max` of **new_lineno over the Add lines only** (deletes dropped from the range), but the `cached_excerpt` must still render the dropped `-` lines (L-03/L-06).
**Why it happens:** Two different outputs from one selection — the range is the resolvable coordinate, the excerpt is the human/agent-readable body.
**How to avoid:** Compute the range from side-filtered linenos; compute the excerpt from the contiguous `min..max` **index** span over `hunk.lines` (so in-between unselected and Delete lines appear). Test both outputs independently.
**Warning signs:** Excerpt missing `-` lines, or range including a deleted line's number.

### Pitfall 4: Draft slot is single and shared (D-02)
**What goes wrong:** Selecting a new range while a non-empty draft exists silently overwrites it.
**Why it happens:** The schema has exactly one `draft_comment` slot (frozen Phase 65).
**How to avoid:** Before switching selection when the current draft text is non-empty, `ask("Discard your unsaved comment?")`; empty draft switches silently. Mirror `handleDiscardLines`.
**Warning signs:** Lost in-progress comments on selection change.

### Pitfall 5: Forgetting `cached_excerpt` is None on the draft
**What goes wrong:** `DraftComment` has **no** `cached_excerpt` field (types.rs:324) — the excerpt is computed at submit, not while drafting.
**Why it happens:** Schema asymmetry between `Comment` (has excerpt) and `DraftComment` (no excerpt).
**How to avoid:** `save_draft_comment` writes only `{ text, anchor }`. The excerpt is assembled by the adapter only on `add_comment`.
**Warning signs:** serde error trying to put `cached_excerpt` on a draft.

## Code Examples

### Reading per-line side coordinates (frontend, verified shapes)
```typescript
// Source: src/lib/types.ts:18,146-149 (DiffOrigin, DiffLine fields)
// Add line:    { origin: "Add",    old_lineno: null, new_lineno: 42 }
// Delete line: { origin: "Delete", old_lineno: 17,   new_lineno: null }
// Context:     { origin: "Context",old_lineno: 16,   new_lineno: 41 }
const selected = Array.from(selectedLineIndices).map((i) => hunk.lines[i]);
const newNums = selected.filter((l) => l.new_lineno !== null).map((l) => l.new_lineno!);
const start = Math.min(...newNums), end = Math.max(...newNums);
```

### Existing emit + reload round-trip (to mirror)
```rust
// Source: src-tauri/src/commands/review.rs:412-415 (add_review_commit)
add_review_commit_rmw(&data_dir, &canonical, &sessions.0, &oid)
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
```
```typescript
// Source: src/components/ReviewPanel.svelte:83-89 (the consumer)
listen<string>("session-changed", (event) => {
  if (status && event.payload !== status.canonical_path) return;
  reloadStatus(); reloadCommits();
});
```

### Confirm-before-discard (to mirror for D-02)
```typescript
// Source: src/components/DiffPanel.svelte:386-394
const { ask } = await import("@tauri-apps/plugin-dialog");
const confirmed = await ask("Discard your unsaved comment?", {
  title: "Discard Comment", kind: "warning",
});
if (!confirmed) return;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Line selection only in staging diffs (`diffKind` unstaged/staged) | Selection enabled in commit diffs (guard lifted) for comment capture | Phase 67 | Commit diffs become interactive; staging buttons still absent |
| `tauri-plugin-store` for review data | Own JSON store, atomic tmp+rename, FNV-1a keyed | Phase 65 (D-09/D-10) | Use `review_store`, never the plugin store |

**Deprecated/outdated:**
- Do not use `@tauri-apps/plugin-store` for review-session data — Phase 65 deliberately rejected it (D-09).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Copied`/`Untracked`/`Unknown` file statuses should be treated like `Modified` for side derivation | Pattern 1 | Low — commit diffs rarely surface these; adapter test should assert behavior. Planner may prefer to disable the affordance for non-A/D/M/R statuses. |
| A2 | The composer should mount inside/near the hunk-header toolbar row (the cited affordance), not as a true floating overlay | Pattern 4 | Low — D-01 is locked and cites this exact affordance; "floating" is loose wording. Confirm visual placement at discuss/plan time if UX-critical. |

**Note:** No `[ASSUMED]` factual claims about library behavior remain — all schema shapes, line numbers, and round-trips were verified against live code this session.

## Open Questions (RESOLVED)

1. **Non-A/D/M/R file statuses (Copied/Untracked/Unknown) in a commit diff**
   - What we know: commit diffs are produced by `diff_commit_file` (diff.rs); status maps from `git2::Delta`. Renamed/Copied detection depends on diff find-options.
   - What's unclear: whether the affordance should be enabled for `Copied` (treat as Renamed?) or disabled for `Unknown`.
   - Recommendation: Treat `Copied` like `Renamed` (new side, new path) and `Untracked`/`Unknown` like `Modified`; cover with an adapter test. Low risk; planner can tighten.
   - **RESOLVED:** Plan 01 adopts this — truth #3 and adapter Tests 8/9 lock `Copied`→Renamed (new side + new path) and `Untracked`/`Unknown`→Modified.

2. **Draft persist cadence (every keystroke vs. debounced)**
   - What we know: L-05 mandates persist-on-change, not only on submit. Each persist is an atomic disk write + emit.
   - What's unclear: literal every-keystroke writes could be chatty (one fsync per keypress).
   - Recommendation: Persist on change but allow a short debounce (e.g., on input idle ~300ms) — still satisfies "survives re-fetch/restart" while avoiding per-keypress fsync. Planner's call; note the `session-changed` emit on every draft write will also reload the panel, so consider whether draft writes should emit at all (likely **not** — drafts are not panel-visible until Phase 69).
   - **RESOLVED:** Plan 04 Task 1 chooses ~300ms input-idle debounce for the draft persist.

3. **Should `save_draft_comment` emit `session-changed`?**
   - What we know: every other mutation emits; the ReviewPanel reloads on it.
   - Recommendation: Probably **omit** the emit for draft writes — the draft is not rendered anywhere in Phase 67/69-stub, and emitting per keystroke causes needless reloads. `add_comment` (submit) does emit. Confirm at plan time.
   - **RESOLVED:** Plan 02 omits the emit for `save_draft_comment` (truth #5, action step 4); only `add_comment` emits `session-changed`.

## Validation Architecture

Nyquist validation is enabled (`workflow.nyquist_validation` not `false` in config). This section is required.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.0 (frontend, jsdom) + Cargo test (backend) |
| Config file | `vite.config.ts` (inline `test` block — no separate vitest.config.ts) |
| Quick run command | `just vitest` (frontend) / `just cargo-test` (backend) |
| Full suite command | `just check` (fmt + biome + svelte-check + clippy + cargo-test + vitest) |

### Success Criteria → Test Map

**SC-1: User can select a line range and attach a comment.**
| Layer | Test | Command |
|-------|------|---------|
| TS unit (adapter) | pure-Add selection → `{ side:"New", start:min, end:max }` | `just vitest` (`diff-anchor.test.ts`) |
| Rust unit | `add_comment_inner` pushes a `Comment` with the anchor + excerpt, clears `draft_comment`, persists | `just cargo-test` |
| Vitest component | guard-lift: clicking a line in a commit diff updates selection state | `just vitest` |
| Human UAT | the attach-then-submit flow feels right; "Comments on lines N–M" preview correct | manual |

**SC-2: A comment still points at the correct code after diff re-fetch (context/whitespace toggle) and after restart.**
> **SCOPE GUARD:** Phase 70 owns render-time re-resolution (L-06). For Phase 67, SC-2 collapses to **persist anchor → reload session from disk → assert anchor fields unchanged**. The anchor IS source coordinates (no `hunk_index`/`context_lines` field exists that *could* shift), so the re-fetch invariance is guaranteed at the schema level. Do **not** pull Phase 70 re-resolution into this phase.
| Layer | Test | Command |
|-------|------|---------|
| Rust unit | store round-trip: `save_session` with a non-trivial `Anchor` → `load_session` → `Loaded(s)` with identical anchor fields | `just cargo-test` |
| Type-level | the anchor schema has no array-index/option fields (verified: types.rs:307-315 has only `commit_oid,file_path,source,side,start_line,end_line`) | static / review |
| Human UAT | attach a comment, toggle ignore-whitespace, restart app — comment metadata intact (full re-resolution display is Phase 70) | manual |

**SC-3: A selection spanning added and deleted lines attaches without error and records which side it targets.**
| Layer | Test | Command |
|-------|------|---------|
| TS unit (adapter) | mixed Add+Delete → `{ side:"New", start:min(new), end:max(new) }`, Delete lines dropped from range; `cachedExcerpt` still contains the `-` lines | `just vitest` |
| TS unit (adapter) | pure-Delete → `{ side:"Old", ... }`; non-contiguous selection → still `min..max`; file-status `Added` forces `New`; `Renamed` stores the new path | `just vitest` |
| Rust unit | `add_comment_inner` accepts `Source::FullFile` too (proves L-08 sharing) | `just cargo-test` |

### Tier Map (what is testable where)
- **Rust unit (cargo test):** `add_comment_inner` (push + clear-draft + persist + returns session), `save_draft_comment_inner` (write/replace draft slot), source-param-not-hardcoded test, store round-trip with non-trivial anchor. Use the existing `tempfile::TempDir` + in-process git fixture pattern (review.rs tests / `TestContext`).
- **TS unit (Vitest, no mocks — pure adapter):** all selection-shape fixtures above. No Tauri mock needed; the adapter is a pure function.
- **Vitest component:** guard-lift (line click works in commit diff), confirm-on-discard for non-empty draft (mock `plugin-dialog` `ask`), draft-on-keystroke `save_draft_comment` invoke (mock `invoke`), merge-commit disabled affordance + tooltip, empty-text submit disabled.
- **Human UAT:** attach UX feel, preview text, success feedback via `session-changed` reload.

### Sampling Rate
- **Per task commit:** `just vitest` (or `just cargo-test` for Rust-only tasks)
- **Per wave merge:** `just check`
- **Phase gate:** `just check` green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/lib/diff-anchor.test.ts` — covers SC-1 / SC-3 (adapter)
- [ ] Rust tests for `add_comment_inner` / `save_draft_comment_inner` — covers SC-1 / SC-2 / SC-3 (add to `review.rs` `#[cfg(test)]` module or `src-tauri/tests/test_review*.rs`)
- [ ] Component test additions for guard-lift + composer in `DiffPanel.test.ts` (or a new `CommentComposer.test.ts`)
- [ ] No framework install needed — Vitest + Cargo test already configured.

## Security Domain

`security_enforcement` is not set to `false` in config (absent = enabled). Phase 67 is a local-only, single-user desktop feature writing to the app data dir; the relevant controls are inherited from Phase 65 and verified.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local single-user desktop app; no auth surface |
| V3 Session Management | no | No web session; "session" here is a review document, not an auth session |
| V4 Access Control | no | No multi-user access model |
| V5 Input Validation | yes | Comment `text` is user free-text persisted to JSON via serde — no injection sink (rendered as markdown only in Phase 70, which owns its own escaping). Path is the canonical repo path, already validated by `canonical_repo_path`. |
| V6 Cryptography | no | No secrets/crypto in this phase |
| V12 File/Resource (path traversal) | yes | Session filename is an FNV-1a hash of the canonical path (review_store.rs:49) — no user-controlled path component reaches the filesystem. Verified in Phase 65 (D-11). |

### Known Threat Patterns
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via repo path | Tampering | FNV-1a hashed filename; canonical-path keying (Phase 65, verified) |
| Corrupt/oversized session file | Denial of Service | `load_session` quarantines unparseable files to `.corrupt` sidecar (D-15) — never crashes, never deletes |
| Comment text injection into rendered markdown | Tampering | Out of scope — Phase 70 owns render-time fence-length computation (DOC-02) and escaping |

No new security controls are required in Phase 67 beyond reusing the verified Phase 65 store behavior.

## Environment Availability

SKIPPED — Phase 67 is a code-only change (TS adapter + Svelte composer + Rust command). No external tools, services, or runtimes beyond the already-present toolchain (`just`, cargo, bun/vitest), which is required to build the project at all.

## Sources

### Primary (HIGH confidence — verified against live code this session)
- `src-tauri/src/git/types.rs:130-336` — `DiffOrigin`, `DiffLine`/`DiffHunk`/`DiffStatus`/`FileDiff`, frozen `Source`/`Side`/`Anchor`/`Comment`/`DraftComment`/`ReviewSession`
- `src-tauri/src/commands/diff.rs:222-242, 332-333` — `DiffLine` construction from git2 `line.old_lineno()`/`new_lineno()`/`origin()`
- `src-tauri/src/commands/review.rs` (full) — `_inner` + thin wrapper + emit pattern; `mutate_session_rmw` lock pattern; test fixtures
- `src-tauri/src/git/review_store.rs` (full) — atomic `save_session`/`load_session`/`delete_session`, FNV-1a keying
- `src-tauri/src/lib.rs:124-131` — command registration site
- `src/components/DiffPanel.svelte:298-339, 383-414` — `handleLineClick`, commit guard (307), confirm-on-discard
- `src/components/diff/HunkView.svelte:172-327` — action affordance pattern, `isSelectable` (303), line click emit
- `src/components/diff/SplitView.svelte:237-340` — split affordance, right-col-only `Add` selection (323)
- `src/components/RepoView.svelte:285-318, 730-774` — `commitDetail` load, `commitDetail={null}` at the commit DiffPanel
- `src/components/ReviewPanel.svelte:79-93` — `session-changed` consumer
- `src/lib/diff-utils.ts:9-55` — `PairedRow.lineIdx` preserves original `hunk.lines` index
- `src/lib/types.ts:18,146-149` — `DiffOrigin`, `DiffLine` TS shape
- `.planning/codebase/CONVENTIONS.md`, `TESTING.md` — serde/naming/test conventions
- `.planning/phases/67-diff-source-anchor-capture/67-CONTEXT.md` — locked decisions D-01..D-04, L-01..L-08

### Secondary (MEDIUM confidence)
- None — all findings verified against primary sources.

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new packages; all infra verified in tree
- Architecture (adapter-in-TS, plumbing gaps, guard-lift sites): HIGH — traced through live code, data flow confirmed end-to-end
- Pitfalls: HIGH — each derived from a specific verified code location

**Research date:** 2026-05-25
**Valid until:** 2026-06-24 (stable — internal codebase, frozen schema; revisit only if Phase 65/66 schema or DiffPanel selection model changes)
