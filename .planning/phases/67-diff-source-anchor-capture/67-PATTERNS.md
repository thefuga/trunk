# Phase 67: Diff-Source Anchor Capture - Pattern Map

**Mapped:** 2026-05-25
**Files analyzed:** 11 (4 new, 7 modified)
**Analogs found:** 11 / 11

> RESEARCH.md already derived the architecture (TS capture-time adapter, shared dumb `add_comment`, three guard-lift sites, `commitDetail` threading). This document maps each net-new/modified file to its closest existing analog with copyable excerpts. It does NOT re-derive the design.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/lib/diff-anchor.ts` (NEW) | utility (pure adapter) | transform | `src/lib/merge-parser.ts` | exact (pure transform lib) |
| `src/lib/diff-anchor.test.ts` (NEW) | test | transform | `src/lib/merge-parser.test.ts` | exact (pure-fn Vitest, no mocks) |
| `src/components/diff/CommentComposer.svelte` (NEW, name planner's call) | component | request-response | on-selection action row in `HunkView.svelte:172-298` / `SplitView.svelte:237-289` + `DiffPanel.handleDiscardLines` | role+flow match |
| `src/components/DiffPanel.svelte` (EDIT) | component | request-response | self — `handleStageLines`/`handleUnstageLines`/`handleDiscardLines` (341-414); guard at 307 | self |
| `src/components/diff/HunkView.svelte` (EDIT) | component | request-response | self — `isSelectable` guard (303), `diffKind` toolbar branches (172/254) | self |
| `src/components/diff/SplitView.svelte` (EDIT) | component | request-response | self — `isSelectable` guard (323), `diffKind` toolbar branches (237/271) | self |
| `src/components/RepoView.svelte` (EDIT) | component | request-response | self — `commitDetail` load (313), `commitDetail={null}` at 732 | self |
| `src-tauri/src/commands/review.rs` `add_comment` + `_inner` (EDIT) | controller (Tauri command) | CRUD (write) | `add_review_commit` (399-416) + `mutate_session_rmw` (304-320) | exact |
| `src-tauri/src/commands/review.rs` `save_draft_comment` + `_inner` (EDIT) | controller (Tauri command) | CRUD (write) | `add_review_commit` + `mutate_session_rmw` | exact |
| `src-tauri/src/lib.rs` registration (EDIT) | config | — | `invoke_handler` block (124-131) | exact |
| Rust unit tests in `review.rs` `#[cfg(test)]` (EDIT) | test | CRUD | `selection_rmw_serialized` (936-1006) + `TestRepo`/`TempDir` fixtures (645-711) | exact |
| `src/lib/types.ts` request DTOs (EDIT) | utility (types) | — | existing `DiffRequestOptions` camelCase request convention; review schema already at 285-316 | partial |
| Vitest component test (`DiffPanel.test.ts` or new) (EDIT/NEW) | test | request-response | `DiffPanel.test.ts:12-75` (tauri-mock + invoke/store mocks) + `StagingPanel.test.ts:28-32` (plugin-dialog `ask` mock) | exact |

---

## Pattern Assignments

### `src/lib/diff-anchor.ts` (NEW — utility, transform)

**Analog:** `src/lib/merge-parser.ts` — the established shape for a pure, fully-unit-tested transform lib that a Svelte component imports to drive reactive state. Mirror its structure: a file-level doc comment stating purity, exported `interface` for the return shape, small private helpers, exported pure functions taking primitives/Sets and returning new values (no mutation, no side effects, no IPC).

**Doc-comment + export-shape pattern** (`merge-parser.ts:1-16`):
```typescript
/**
 * Pure TypeScript merge parser: ...
 * All functions are pure (no side effects) and fully tested.
 * The MergeEditor component (Plan 03) imports these functions
 * to drive its reactive state.
 */
export interface ConflictRegion { type: "context" | "conflict"; ... }
```
For Phase 67 the exported return interface is `{ anchor: Anchor; cachedExcerpt: string }` (reuse the existing `Anchor` type from `src/lib/types.ts:291-298` — do NOT redeclare it).

**Set-based, immutable transform pattern** (`merge-parser.ts:276-302`, `toggleHunk`): takes a `Set<...>`, never mutates inputs, returns a fresh value — copy this discipline for reading `selectedLineIndices: Set<number>`.

**Inputs the adapter reads (verified shapes):**
- `DiffLine` — `src/lib/types.ts:145-151`: `{ origin: DiffOrigin; content: string; old_lineno: number | null; new_lineno: number | null; spans }`.
- `DiffOrigin` — `types.ts:18`: `"Context" | "Add" | "Delete"`.
- `DiffStatus` — `types.ts:162-169`: `"Added" | "Deleted" | "Modified" | "Renamed" | "Copied" | "Untracked" | "Unknown"`.
- `FileDiff` — `types.ts:171-176`: `{ path; status: DiffStatus; is_binary; hunks }`.
- `Anchor`/`Source`/`Side` — `types.ts:288-298` (output target).

**Side-resolution core (from RESEARCH Pattern 1, the algorithm — file-status ALWAYS wins over the mixed default):**
```typescript
type Side = "Old" | "New";
function resolveSide(status: DiffStatus, selected: DiffLine[]): Side {
  if (status === "Added") return "New";
  if (status === "Deleted") return "Old";
  if (status === "Renamed") return "New";
  const hasAdd = selected.some((l) => l.origin === "Add");
  const hasDel = selected.some((l) => l.origin === "Delete");
  if (hasAdd) return "New";   // mixed or pure-Add -> New, deletes dropped from range
  if (hasDel) return "Old";   // pure-Delete -> Old
  return "New";               // defensive default
}
```
**Range from side-filtered linenos** (Add carries only `new_lineno`, Delete only `old_lineno` — git2 maps libgit2's `-1` sentinel to `null`, so just read the chosen side's field):
```typescript
const selected = Array.from(selectedLineIndices).map((i) => hunk.lines[i]);
const nums = selected
  .map((l) => (side === "New" ? l.new_lineno : l.old_lineno))
  .filter((n): n is number => n !== null);
const start_line = Math.min(...nums), end_line = Math.max(...nums);
```
**Excerpt vs range divergence (L-03/L-06, RESEARCH Pitfall 3):** the persisted `start_line..end_line` excludes dropped Delete lines, but `cachedExcerpt` is built from the contiguous `min..max` **index** span over `hunk.lines` so dropped `-` lines and in-between context appear. Compute them as two independent outputs and test each.

---

### `src/lib/diff-anchor.test.ts` (NEW — test, transform)

**Analog:** `src/lib/merge-parser.test.ts` — pure-function Vitest with **no mocks** (the adapter is pure). Copy the import + `describe`/`it`/`expect` structure exactly.

**Test-file structure** (`merge-parser.test.ts:1-37`):
```typescript
import { describe, expect, it } from "vitest";
import type { ConflictRegion } from "./merge-parser.js";
import { parseConflictRegions, ... } from "./merge-parser.js";

describe("parseConflictRegions", () => {
  it("identifies context and conflict regions ...", () => {
    const regions = parseConflictRegions(base, ours, theirs);
    expect(regions).toHaveLength(3);
    expect(regions[0].type).toBe("context");
  });
});
```
Note `.js` extension in imports (project convention for TS sources). Required fixtures from RESEARCH Test Map: pure-Add → `{side:"New"}`; pure-Delete → `{side:"Old"}`; mixed Add+Delete → `{side:"New"}` with deletes dropped from range but present in `cachedExcerpt`; non-contiguous → still `min..max`; `Added` forces `New`; `Renamed` stores new path; `Copied`/`Untracked`/`Unknown` behavior (RESEARCH A1/Q1 — planner decides treat-as-Modified vs disable).

---

### `src/components/diff/CommentComposer.svelte` (NEW — component, request-response)

**Analog (affordance placement):** the on-selection action-button row that renders **inside the hunk-header toolbar**, gated on `diffKind` + `hasSelection`. RESEARCH Pattern 4 / A2: D-01's "floating" is loose — match this toolbar placement, add a separate `diffKind === 'commit'` branch (staging branches stay untouched).

**HunkView toolbar action-row pattern** (`HunkView.svelte:172-213`):
```svelte
{#if diffKind === 'unstaged'}
  {@const hunkKey = `${fd.path}-${hunkIdx}`}
  {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
  {#if hasSelection}
    <button
      disabled={stagingDisabled}
      title={stagingDisabledTitle}
      style="... color: var(--color-success); cursor: {stagingDisabled ? 'not-allowed' : 'pointer'}; opacity: {stagingDisabled ? 0.4 : 1}; ..."
      onclick={() => onstagelines(fd.path, hunkIdx)}
    >Stage Lines ({selectedCount})</button>
  {/if}
{/if}
```
For the Comment branch: add `{:else if diffKind === 'commit'}` with a `hasSelection`-gated **Comment** button. The `disabled` + `title` pattern (D-04 merge-commit disable + tooltip) is exactly the `disabled={stagingDisabled} title={stagingDisabledTitle}` pattern above — substitute a merge gate (`commitDetail.parent_oids.length > 1`) and tooltip "Diff comments aren't available on merge commits."

**SplitView toolbar mount site** (`SplitView.svelte:230-289`): same `hasSelection`-gated action row inside `class="split-hunk-header"`, using the shared `.staging-btn` CSS classes (e.g. `class="staging-btn success-btn"`). The composer mounts in the **right (`new`) column context** consistent with right-column-only selection (RESEARCH Pitfall 2: split view can only ever produce a `new`-side anchor).

**Theme rule:** all colors via `--color-*` vars (e.g. `var(--color-success)`, `var(--color-success-bg)`, `var(--color-success-border)`). Never inline a literal color (CLAUDE.md rule).

**Confirm-before-discard (D-02)** — mirror `DiffPanel.handleDiscardLines` (`DiffPanel.svelte:386-394`):
```typescript
const { ask } = await import("@tauri-apps/plugin-dialog");
const confirmed = await ask("Discard your unsaved comment?", {
  title: "Discard Comment",
  kind: "warning",
});
if (!confirmed) return;
```
Gate the prompt on non-empty draft text only; empty draft switches silently (D-02). Dynamic `import("@tauri-apps/plugin-dialog")` is the project convention (matches `handleDiscardLines`).

**IPC on submit / keystroke** — use `safeInvoke<T>` (`src/lib/invoke.ts:10`), the project-wide IPC wrapper that parses `TrunkError`. Submit calls `add_comment`; keystroke calls `save_draft_comment` (cadence/debounce is RESEARCH Q2, planner's call). Error handling mirrors the `handleStageLines` try/catch+`showToast` shape (`DiffPanel.svelte:344-359`):
```typescript
try {
  await safeInvoke("add_comment", { path: repoPath, text, anchor, cachedExcerpt });
} catch (e) {
  const err = e as TrunkError;
  showToast(err.message ?? "Add comment failed", "error");
  return;
}
```

---

### `src/components/DiffPanel.svelte` (EDIT — component, request-response)

**Self-analog.** Three changes:

1. **Lift the commit guard** (`DiffPanel.svelte:307`) — `handleLineClick` currently early-returns for commit diffs:
```typescript
function handleLineClick(filePath, hunkIdx, lineIndex, origin, hunkLines, e) {
  if (origin === "Context") return;
  if (diffKind === "commit") return;   // ← LIFT for comment capture (keep Context guard)
  ...
```
The existing selection model below it (`selectedHunkKey` + `selectedLineIndices: Set<number>` + shift-click contiguous-skip-Context at 318-328) is reused as-is — capture lets the user pick lines, then `diff-anchor.ts` collapses.

2. **Host composer state + D-02 confirm-on-switch.** Add composer open/draft state alongside the existing selection `$state` (71-74). When a *new* range is selected while draft text is non-empty, run the `ask` confirm (see CommentComposer). `commitDetail` is already a declared prop (`DiffPanel.svelte:33`, `Props.commitDetail: CommitDetail | null`) and destructured (51) — derive `commitOid = commitDetail?.oid` and `isMerge = (commitDetail?.parent_oids.length ?? 0) > 1`.

3. **Action-handler analog for the new submit/draft handlers** — copy the `handleStageLines` shape (`341-360`): `hunkOperationInFlight`-style in-flight guard, `safeInvoke`, try/catch + `showToast`, `clearSelection()` in `finally`.

---

### `src/components/diff/HunkView.svelte` (EDIT — component, request-response)

**Self-analog.** Lift the inline-view selectability guard (`HunkView.svelte:303`):
```svelte
{@const isSelectable = diffKind !== 'commit' && line.origin !== 'Context'}
```
→ allow commit diffs while still excluding `Context`. Add a `{:else if diffKind === 'commit'}` Comment-affordance branch in the hunk-header toolbar (after the `staged` branch at 254-298), mirroring the `hasSelection`-gated button rows. **Do NOT** touch the `unstaged`/`staged` staging branches (172/254) — staging buttons stay absent in commit diffs because they have no `commit` branch.

---

### `src/components/diff/SplitView.svelte` (EDIT — component, request-response)

**Self-analog.** Lift the right-column selectability guard (`SplitView.svelte:323`):
```svelte
{@const isSelectable = diffKind !== 'commit' && line.origin === 'Add'}
```
→ allow commit diffs (right column still `Add`-only by construction — RESEARCH Pitfall 2, do NOT widen). Add the Comment-affordance branch in `split-hunk-header` (after the `staged` branch at 271-289), reusing `.staging-btn` classes. Same constraint: leave staging branches (237/271) untouched.

---

### `src/components/RepoView.svelte` (EDIT — component, request-response)

**Self-analog.** Thread the already-loaded `commitDetail` into the commit `DiffPanel`. `commitDetail` is RepoView `$state`, loaded in the same `handleCommitSelect` that drives the commit diff:

**Load site** (`RepoView.svelte:302-313`):
```typescript
const [files, detail] = await Promise.all([
  safeInvoke<FileDiff[]>("list_commit_files", { path: repoPath, oid }),
  safeInvoke<CommitDetailType>("get_commit_detail", { path: repoPath, oid }),
]);
commitFileDiffs = files;
commitDetail = detail;
```

**The hardcoded-null to fix** (`RepoView.svelte:732`):
```svelte
<DiffPanel
  fileDiffs={currentDiffFiles}
  commitDetail={null}   <!-- ← pass the existing `commitDetail` $state instead -->
```
This is the smallest change (RESEARCH "Alternatives Considered" — pass-through beats adding new props). It gives the adapter `commit_oid` (anchor) and merge detection (D-04 disable).

---

### `src-tauri/src/commands/review.rs` — `add_comment` + `add_comment_inner` (EDIT — Tauri command, CRUD write)

**Analog:** `add_review_commit` thin command (`review.rs:399-416`) over `mutate_session_rmw` (`304-320`). The command is a **dumb writer** — it receives a fully-formed `Anchor` (which already carries `source` + `side` from the TS adapter), so it is Phase-68-shareable with no change (L-08).

**Thin-command + emit pattern** (`add_review_commit`, `399-416`):
```rust
#[tauri::command]
pub async fn add_review_commit(
    path: String, oid: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let canonical = canonical_repo_path(&path, &state_map)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    add_review_commit_rmw(&data_dir, &canonical, &sessions.0, &oid)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}
```

**RMW core to copy** (`mutate_session_rmw`, `304-320`) — lock held across read → mutate → `save_session` → return, so concurrent submits never lose a write:
```rust
fn mutate_session_rmw<F>(data_dir, canonical, sessions: &Mutex<HashMap<PathBuf, ReviewSession>>, mutate: F)
  -> Result<(), TrunkError>
where F: FnOnce(&mut Vec<String>) {
    let mut map = sessions.lock().unwrap();
    let session = map.get_mut(canonical)
        .ok_or_else(|| TrunkError::new("no_session", "No active review session for this repository"))?;
    mutate(&mut session.commits);
    review_store::save_session(data_dir, canonical, session)?;
    Ok(())
}
```
**Adaptation for `add_comment_inner`:** the existing `mutate` closure type is `FnOnce(&mut Vec<String>)` (commits only). `add_comment` mutates `session.comments` AND `session.draft_comment`, so the closure must receive `&mut ReviewSession` (generalize the RMW or write a sibling `mutate_session_full_rmw`). The closure:
```rust
// inside the locked session:
session.comments.push(Comment {
    text: req.text,
    anchor: Some(req.anchor),            // already source coords from the TS adapter
    cached_excerpt: Some(req.cached_excerpt),
});
session.draft_comment = None;            // submit clears the single draft slot (RESEARCH anti-pattern)
```
**Request type — camelCase via `rename_all`** (frontend-facing; the persisted `Anchor`/`Comment` keep snake_case + PascalCase enums, frozen `types.rs:288-336`):
```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddCommentRequest {
    pub path: String,
    pub text: String,
    pub anchor: Anchor,
    pub cached_excerpt: String,   // wire key: cachedExcerpt
}
```
**Emit:** `add_comment` DOES emit `session-changed` (submit changes panel-visible state).

---

### `src-tauri/src/commands/review.rs` — `save_draft_comment` + `_inner` (EDIT — Tauri command, CRUD write)

**Analog:** same `add_review_commit` + `mutate_session_rmw` pattern. The mutate closure writes/replaces the single `draft_comment` slot:
```rust
session.draft_comment = Some(DraftComment { text: req.text, anchor: req.anchor });
```
**Schema asymmetry (RESEARCH Pitfall 5):** `DraftComment` has NO `cached_excerpt` (`types.rs:324-328`) — write only `{ text, anchor }`. The excerpt is computed at submit only.
**Emit decision:** RESEARCH Q3 recommends `save_draft_comment` likely **omits** the `session-changed` emit (draft isn't panel-visible until Phase 69, and per-keystroke emits cause needless reloads). This is a planner decision — surface, don't decide. If omitted, the command body ends after the RMW with no `app.emit`.

---

### `src-tauri/src/lib.rs` — command registration (EDIT — config)

**Analog:** the `invoke_handler` review block (`lib.rs:124-131`). Append the two new commands after line 131:
```rust
commands::review::list_session_commits,
commands::review::add_comment,           // ← new
commands::review::save_draft_comment,    // ← new
```

---

### Rust unit tests in `review.rs` `#[cfg(test)]` (EDIT — test, CRUD)

**Analogs (all in `review.rs` `#[cfg(test)]`, 623-1018):**
- **Store round-trip + concurrency:** `selection_rmw_serialized` (`936-1006`) — spins N threads, asserts every write survives in memory AND on disk via `review_store::load_session(...) => LoadOutcome::Loaded(s)`. Adapt for `add_comment_inner`: push N comments concurrently, assert `s.comments.len() == n` and `draft_comment` cleared.
- **No-session error:** `rmw_missing_session_is_no_session_error` (`1008-1017`) — asserts `err.code == "no_session"` when no in-memory session.
- **Fixtures:** `TempDir::new()` + the `TestRepo` / `make_repo` helpers (`645-711`) with deterministic `sig()` (`652-654`). For a comment test you only need a `TempDir` data dir + a seeded `ReviewSession` in the sessions map (see `selection_rmw_serialized:940-952`), not necessarily a git repo.
- **L-08 contract test:** call `add_comment_inner` with an `Anchor { source: Source::FullFile, .. }` and assert it persists unchanged — locks the Phase-68-shareable contract (RESEARCH SC-3 Rust row).
- **SC-2 invariance:** `save_session` a non-trivial `Anchor` → `load_session` → assert all six anchor fields identical (the anchor has no array-index field that *could* shift — `types.rs:307-315`).

**Round-trip assertion shape** (`selection_rmw_serialized:990-995`):
```rust
match review_store::load_session(data_dir.path(), &canonical).unwrap() {
    LoadOutcome::Loaded(s) => { assert_eq!(s.comments.len(), n); }
    _ => panic!("expected a loadable session on disk"),
}
```

---

### `src/lib/types.ts` — request DTOs (EDIT — types)

**Important correction to RESEARCH Component Responsibilities:** the review schema TS types (`Anchor`, `Comment`, `DraftComment`, `Source`, `Side`, `ReviewSession`) **already exist** at `types.ts:285-316` and are string-for-string with the Rust on-wire shape. Do NOT redeclare them. This edit adds only the two **request DTOs** for the new commands, following the existing camelCase request convention (e.g. `DiffRequestOptions` at `types.ts:178-182` uses camelCase keys mirroring a Rust `#[serde(rename_all = "camelCase")]` struct).

```typescript
export interface AddCommentRequest {
  path: string;
  text: string;
  anchor: Anchor;          // reuse existing type (types.ts:291-298)
  cachedExcerpt: string;   // camelCase wire key -> Rust cached_excerpt
}
export interface SaveDraftCommentRequest {
  path: string;
  text: string;
  anchor: Anchor | null;
}
```
(DTOs optional — `safeInvoke` takes `Record<string, unknown>`; declaring them documents the contract.)

---

### Vitest component test (`DiffPanel.test.ts` or new `CommentComposer.test.ts`) (EDIT/NEW — test)

**Analog:** `DiffPanel.test.ts:1-75` for render + invoke/store/toast mocks; `StagingPanel.test.ts:28-32` for the `plugin-dialog` `ask` mock (D-02 confirm test).

**Mock setup** (`DiffPanel.test.ts:12-30`):
```typescript
import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import "../__tests__/helpers/tauri-mock";

vi.mock("../lib/invoke.js", () => ({ safeInvoke: vi.fn().mockResolvedValue(undefined) }));
vi.mock("../lib/toast.svelte.js", () => ({ showToast: vi.fn() }));
// vi.mock("../lib/store.js", () => ({ ... }));  // full store mock at DiffPanel.test.ts:32-75
```

**plugin-dialog mock** (`StagingPanel.test.ts:28-32`):
```typescript
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  ask: vi.fn().mockResolvedValue(false),   // flip per-test for confirm/cancel branches
  message: vi.fn().mockResolvedValue(undefined),
}));
```
Coverage (RESEARCH Tier Map): guard-lift (line click updates selection in a commit diff), confirm-on-discard for non-empty draft (assert `ask` called; cancel → no switch), draft-on-keystroke `save_draft_comment` invoke, merge-commit disabled affordance + tooltip, empty-text submit disabled.

---

## Shared Patterns

### `_inner` + thin wrapper + `mutate_session_rmw` (Rust)
**Source:** `src-tauri/src/commands/review.rs:304-320` (RMW core), `399-416` (thin command + emit), `154-158` (`resolve_data_dir`), `61-69` (`canonical_repo_path`).
**Apply to:** both `add_comment` and `save_draft_comment`. ONE shared RMW pattern, two callers — `add_comment`'s closure pushes a `Comment` AND clears `draft_comment`; `save_draft_comment`'s closure writes the `draft_comment` slot. Both need the lock-across-read-mutate-save discipline so rapid submits/keystrokes don't lose writes. Note the existing `mutate_session_rmw` closure is `FnOnce(&mut Vec<String>)` (commits only) — generalize to `&mut ReviewSession` for the comment/draft mutations.

### `session-changed` emit divergence
**Source:** `review.rs:414` emit + `src/components/ReviewPanel.svelte:83-89` listener.
```rust
let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
```
**Apply to:** `add_comment` emits (submit changes panel state). `save_draft_comment` likely does NOT emit (RESEARCH Q3 — drafts aren't panel-visible in Phase 67/69-stub; per-keystroke emits cause needless reloads). **Planner decides; this divergence is intentional, not an oversight.**

### serde / IPC conventions (honor exactly)
**Source:** `types.rs:288-315` (frozen schema), `types.rs:162-171` (`DiffRequestOptions` request convention), `src/lib/invoke.ts:10`.
- **Persisted review enums** (`Source`, `Side`): PascalCase strings, **NO** `rename_all` (`types.rs:295-305` — `"Diff"`/`"New"` on the wire).
- **Persisted structs** (`Anchor`, `Comment`, `DraftComment`): snake_case fields (`commit_oid`, `cached_excerpt`).
- **Frontend-facing request types** (`AddCommentRequest`): `#[serde(rename_all = "camelCase")]` → TS sends `cachedExcerpt`.
- **All IPC** goes through `safeInvoke<T>` (parses `TrunkError`); never raw `invoke`.

### Confirm-before-destroy dialog
**Source:** `src/components/DiffPanel.svelte:386-394`.
**Apply to:** D-02 discard-draft (CommentComposer). Dynamic `import("@tauri-apps/plugin-dialog")`, `ask(msg, { title, kind: "warning" })`, `if (!confirmed) return;`.

### Theme CSS custom properties only
**Source:** every styled affordance in `HunkView.svelte:172-298` / `SplitView.svelte` uses `var(--color-*)` (e.g. `--color-success`, `--color-danger`, `--color-text-muted`).
**Apply to:** CommentComposer + any selection highlight. Never inline a literal color (CLAUDE.md hard rule). Reuse `.staging-btn`-style classes in SplitView; inline `var(--color-*)` styles in HunkView.

### Pure transform lib + no-mock Vitest
**Source:** `src/lib/merge-parser.ts` + `merge-parser.test.ts`.
**Apply to:** `diff-anchor.ts` + its test. Adapter is a pure function — no Tauri mock needed, test inputs are plain `DiffLine[]` fixtures.

---

## No Analog Found

None. Every new/modified surface has a close in-tree analog (the project already has a pure transform lib, the exact command/RMW/emit pattern, the selection model, the confirm dialog, and the test scaffolding).

---

## Metadata

**Analog search scope:** `src/lib/`, `src/components/`, `src/components/diff/`, `src/__tests__/`, `src-tauri/src/commands/`, `src-tauri/src/git/`, `src-tauri/src/lib.rs`.
**Files scanned (read):** `review.rs`, `merge-parser.ts`, `merge-parser.test.ts`, `invoke.ts`, `types.ts`, `git/types.rs`, `DiffPanel.svelte`, `DiffPanel.test.ts`, `HunkView.svelte`, `SplitView.svelte`, `RepoView.svelte`, `StagingPanel.test.ts`, `lib.rs`.
**Pattern extraction date:** 2026-05-25
