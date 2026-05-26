# Phase 69: Comment Management UI - Pattern Map

**Mapped:** 2026-05-26
**Files analyzed:** 11 (5 backend, 6 frontend incl. tests)
**Analogs found:** 10 / 11 (1 net-new dependency, no codebase analog)

All analogs verified by reading the cited ranges this session. Line numbers are
exact at mapping time; the planner should re-confirm if upstream files shift.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/git/types.rs` | serde schema | persisted-data | `Comment`/`Anchor` in same file (`types.rs:307-336`) + `SessionCommit`/`RebaseTodoItem` (`types.rs:280-286`) | exact (extending the type in place) |
| `src-tauri/src/git/review_store.rs` | store / migration | file-I/O + transform | `load_session` recovery state machine in same file (`review_store.rs:104-135`) | exact (extending the load path) |
| `src-tauri/src/commands/review.rs` (mutating cmds) | Rust command | CRUD | `add_comment_inner`/`add_comment` (`review.rs:390-404`, `511-535`) | exact |
| `src-tauri/src/commands/review.rs` (`list_session_comments`) | Rust command | request-response (read) | `list_session_commits` (`review.rs:569-619`) | exact |
| `src-tauri/src/commands/review.rs` (`resolve_session_comments`) | Rust command | request-response (git2 read) | `list_session_commits` wrapper (`review.rs:605-616`) + `intersect_graph_order` pure fn (`review.rs:250-287`) | role-match (new git2 classifier, proven wrapper) |
| `src-tauri/src/commands/review.rs` (`OrphanReason`/`CommentResolution`) | serde schema | DTO | `Source`/`Side` enums (`types.rs:295-305`); `SessionCommit` struct (`types.rs:280-286` style) | exact (serde convention) |
| `src-tauri/src/lib.rs` (invoke_handler) | wiring | — | existing `commands::review::*` registrations (`lib.rs:124-133`) | exact |
| `src/lib/types.ts` | TS DTO mirror | DTO | `Comment`/`Anchor`/`SessionCommit` (`types.ts:288-353`) | exact |
| `src/lib/review-session.svelte.ts` | rune module | event-driven / UI-state | `createUndoRedoState` factory (`undo-redo.svelte.ts:17-35`); gating via `showDiff` derived (`RepoView.svelte:122-131`) | role-match (rune factory shape; pane-gating pattern) |
| `src/components/ReviewPanel.svelte` | Svelte component | request-response + event | `ReviewPanel.svelte` stub (whole file) + `DiffPanel.handleDiscardLines` confirm (`DiffPanel.svelte:505-516`) | exact (rewrite-in-place) |
| `src/components/RepoView.svelte` + `src/App.svelte` | wiring | UI-state | `handleCommitSelect`/`handleCommitFileSelect`/`scrollToHunk` (`RepoView.svelte:285-370`, `DiffPanel.svelte:257-265`); panel render site (`App.svelte:587-615`) | exact |
| **TEST** `review_store.rs` `#[cfg(test)]` | test | — | existing `review_store.rs:153-173` test mod | role-match |
| **TEST** `review.rs` `#[cfg(test)]` | test | — | `make_repo:808`, `seeded_sessions:1151`, `distinct_anchor:1167`, `loaded:1178` | exact |
| **TEST** `ReviewPanel.test.ts` | test | — | existing vitest `*.test.ts` (e.g. `DiffPanel.test.ts`) | role-match (new file) |

## Pattern Assignments

### `src-tauri/src/git/types.rs` (serde schema, persisted-data)

**Analog:** `Comment` struct in the same file. Add `id: String` and
`commit_oid: Option<String>` (RESEARCH primary rec — sibling field, NOT a
`Source::Commit` variant, NOT optional `Anchor` line fields; keeps Phase 67/68
`Anchor` invariants intact).

**Serde convention header** (`types.rs:288-294`) — read and obey before editing:
```rust
// Enums serialize as PascalCase strings with NO rename_all (mirrors RefType).
// Struct fields stay snake_case.
```

**Current `Comment` to extend** (`types.rs:317-322`):
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub text: String,
    pub anchor: Option<Anchor>,
    pub cached_excerpt: Option<String>,
}
```
New fields land here. Per RESEARCH Pitfall 1: `id` needs `#[serde(default)]`
(migration shape A) so a v1 file lacking it deserializes to `""`; `commit_oid:
Option<String>` needs NO `#[serde(default)]` (serde maps a missing field to
`None` for `Option` automatically). `ReviewSession.schema_version` default bumps
to 2 (the constant lives in `review_store.rs`, see below).

---

### `src-tauri/src/git/review_store.rs` (store / migration, file-I/O + transform)

**Analog:** the `load_session` recovery state machine in the same file. This is
the highest-risk edit (RESEARCH Pitfall 1 + 2 + 3).

**Version constant to bump** (`review_store.rs:23`):
```rust
const CURRENT_SCHEMA_VERSION: u32 = 1;  // → 2
```

**The order that MUST be preserved** (`review_store.rs:112-135`) — read → quarantine-on-unparseable → **version-gate (D-16) BEFORE migrate** → `from_value` → quarantine-on-shape-error:
```rust
let value = match serde_json::from_str::<serde_json::Value>(&raw) { /* …quarantine… */ };

let version = value.get("schema_version").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
if version > CURRENT_SCHEMA_VERSION {
    return Ok(LoadOutcome::RefusedNewer);   // D-16 — keep this BEFORE any migration
}

match serde_json::from_value::<ReviewSession>(value) {
    Ok(session) => Ok(LoadOutcome::Loaded(session)),
    Err(_) => { quarantine_corrupt(&final_path)?; Ok(LoadOutcome::RecoveredCorrupt) }
}
```
The id-backfill + v1→v2 normalize hooks between line 126 (after the newer-refusal)
and the `from_value` at 128, OR as a normalize pass after a successful
`from_value` (RESEARCH shape A vs B). **Re-save after backfill** (Pitfall 3) so
ids are stable across reloads — reuse the existing atomic writer:

**Atomic write to reuse, do not reimplement** (`review_store.rs:91-99` `save_session` → `atomic_write_json:66-82`): tmp-in-same-dir + `sync_all` + `rename`.

**Regression test target** (Pitfall 1 warning sign): hand-build a v1 JSON (no
`id`), `load_session`, assert `Loaded` (NOT `RecoveredCorrupt`) and every comment
has a non-empty `id`. Extend the existing test mod (`review_store.rs:153-173`).

---

### `src-tauri/src/commands/review.rs` — mutating commands (Rust command, CRUD)

Applies to `add_commit_comment`, `edit_comment`, `delete_comment`.

**Analog:** `add_comment_inner` + thin `add_comment` wrapper.

**Testable `_inner` core** (`review.rs:390-404`):
```rust
fn add_comment_inner(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    req: AddCommentRequest,
) -> Result<(), TrunkError> {
    mutate_session_rmw(data_dir, canonical, sessions, |session| {
        session.comments.push(Comment {
            text: req.text,
            anchor: Some(req.anchor),
            cached_excerpt: Some(req.cached_excerpt),
        });
        session.draft_comment = None;
    })
}
```
- `add_commit_comment`: same shape, pushes `Comment { id: Uuid::new_v4()…, text, anchor: None, commit_oid: Some(oid), cached_excerpt: None }`.
- `edit_comment`: `mutate_session_rmw(…, |s| { if let Some(c) = s.comments.iter_mut().find(|c| c.id == id) { c.text = text } })`. Missing id → `not_found` `TrunkError` (RESEARCH Q3).
- `delete_comment`: `mutate_session_rmw(…, |s| s.comments.retain(|c| c.id != id))`. Missing id = idempotent no-op (parity with `apply_remove`, `review.rs:347-357`).

**The RMW primitive to reuse — do not hand-roll locking** (`review.rs:304-320`): holds the sessions mutex across read→mutate→`save_session`, never across `.await`.

**Thin wrapper + emit** (`review.rs:511-535`) — resolve `data_dir` + canonical path, call `_inner`, then emit:
```rust
add_comment_inner(&data_dir, &canonical, &sessions.0, req)
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
Ok(())
```
Only the three mutating commands emit `session-changed`. (Mirror `save_draft_comment`'s deliberate NO-emit at `review.rs:543-561` — inline-edit keystrokes are local component state, never a per-keystroke emit. RESEARCH Pitfall 5.)

**Request bundle convention** (`review.rs:368-374`): each `_inner` takes a struct bundle; the thin command takes flat named args off the wire (camelCase JS keys map to snake_case Rust). Frontend-facing request structs use `#[serde(rename_all = "camelCase")]` (`types.ts:337-340` notes the mirror).

---

### `src-tauri/src/commands/review.rs` — `list_session_comments` (Rust command, read)

**Analog:** `list_session_commits` (`review.rs:569-619`). Read-only, no `save_session`, no emit.

**Canonical-key read pattern** (`review.rs:580-593`):
```rust
let commits = {
    let map = sessions.0.lock().unwrap();
    map.get(&canonical)
        .ok_or_else(|| serde_json::to_string(&TrunkError::new(
            "no_session", "No active review session for this repository")).unwrap())?
        .commits.clone()
};
```
`list_session_comments` clones `.comments` instead of `.commits`. `no_session`
when the in-memory map has no entry. No git2 needed (it returns stored data).

---

### `src-tauri/src/commands/review.rs` — `resolve_session_comments` (Rust command, git2 read)

Two distinct analogs: the **wrapper** and the **pure classifier**.

**Wrapper analog — spawn_blocking on a fresh repo handle** (`review.rs:605-616`):
```rust
let result = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<SessionCommit>, TrunkError> {
    let repo = git2::Repository::open(&path).map_err(TrunkError::from)?;
    Ok(intersect_graph_order(&commits, &graph, &repo))
})
.await
.map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
.map_err(|e| serde_json::to_string(&e).unwrap())?;
```
Never hold the `RepoState` lock across git2 work — open fresh inside the block.

**Pure-classifier analog — `intersect_graph_order`** (`review.rs:250-287`): a pure
`fn(&[…], &git2::Repository) -> Vec<…>` that never errors-as-result and never
silently drops an input (an unresolvable oid gets an `(unavailable)` summary
rather than being omitted). The new `resolve_all(&comments, &repo) ->
Vec<CommentResolution>` mirrors this exactly: one entry per input comment,
unresolvable → `resolvable: false` + reason, never panic (malformed `commit_oid`
→ classify `CommitGone`, never crash — RESEARCH Security/DoS row).

**git2 blob-by-path + line-count (RESEARCH Pitfall 4 + Code Examples), side-aware:**
```rust
let commit = repo.find_commit(oid)?;                       // Err → CommitGone
let tree = match side { Side::New => commit.tree()?,
                        Side::Old => commit.parent(0)?.tree()? };  // root commit → no parent → FileGone
let entry = tree.get_path(std::path::Path::new(&file_path))?;      // Err → FileGone
let blob = repo.find_blob(entry.id())?;                            // blob-read analog: merge_editor.rs:54-62
let n = String::from_utf8_lossy(blob.content()).lines().count() as u32;
let ok = start_line >= 1 && end_line <= n;                         // else LineOutOfRange
```
`Source::FullFile` is always `Side::New` (`full-file-anchor.ts:60`). Commit-level
comments (`anchor: None`, `commit_oid: Some`) only need `find_commit` to succeed —
no file/line check.

---

### `src-tauri/src/commands/review.rs` — `OrphanReason` / `CommentResolution` (serde DTO)

**Analog (enum):** `Source`/`Side` (`types.rs:295-305`) — PascalCase variants, NO `rename_all`:
```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Source { Diff, FullFile }
```
`OrphanReason { CommitGone, FileGone, LineOutOfRange }` follows this exactly. The
UI-SPEC maps these variants to badge strings ("commit gone" / "file gone" /
"line out of range").

**Analog (struct):** `SessionCommit`-style Serialize struct, snake_case fields
(`types.rs:280-286` `RebaseTodoItem` is the literal serde-default snake_case
shape). `CommentResolution { id: String, resolvable: bool, reason:
Option<OrphanReason> }`.

---

### `src-tauri/src/lib.rs` (wiring)

**Analog:** the existing `commands::review::*` registrations (`lib.rs:124-133`).
Add the five new commands to the `invoke_handler` list alongside them:
```rust
commands::review::list_session_commits,
commands::review::add_comment,
commands::review::save_draft_comment,
// + add_commit_comment, list_session_comments, edit_comment,
//   delete_comment, resolve_session_comments
```
`ReviewSessionsState` is already `.manage`d (`lib.rs:78`) — no new state.

The stub's `review-toggle` menu + `reviewPanelOpen` thin-bar trigger (`lib.rs:27,67`,
`App.svelte:592`) is the persistent "Review" toggle candidate (D-07) — planner
decides whether it stays as the panel toggle or is reworked for the center-pane swap.

---

### `src/lib/types.ts` (TS DTO mirror)

**Analog:** `Comment`/`Anchor`/`SessionCommit` (`types.ts:288-353`). String-for-string
with the Rust wire shape (PascalCase enum strings, snake_case fields, `| null`
for `Option<T>`).

**Current `Comment` to extend** (`types.ts:300-304`):
```typescript
export interface Comment {
    text: string;
    anchor: Anchor | null;
    cached_excerpt: string | null;
}
```
Add `id: string;` and `commit_oid?: string | null;`. Add new types mirroring the
Rust DTOs: `OrphanReason = "CommitGone" | "FileGone" | "LineOutOfRange"` and
`CommentResolution { id: string; resolvable: boolean; reason: OrphanReason | null }`
(follow the existing PascalCase-enum-string convention at `types.ts:288-289`).

---

### `src/lib/review-session.svelte.ts` (rune module — NEW file)

**Analog (factory shape):** `createUndoRedoState` (`undo-redo.svelte.ts:17-35`) —
a factory returning `{ state, ...methods }` with `$state(...)` declared inside:
```typescript
export function createUndoRedoState(): UndoRedoManager {
    const state: UndoRedoState = $state({ redoStack: [] as UndoEntry[] });
    return {
        state,
        push(entry: UndoEntry) { state.redoStack = [...state.redoStack, entry]; },
        // …
    };
}
```
The new module owns Review-mode right-pane state: `reviewActive: boolean`,
`rightPaneMode: "panel" | "diff"`, and a `jumpTo(comment)` action that composes
the existing RepoView selection/scroll machinery (see RepoView assignment below).

**Analog (pane gating):** `RepoView.svelte:122-131` `showDiff = $derived(...)` is the
existing two-state pane gate. `rightPaneMode` adds a third state (panel) to the
same center pane — model it on this derived-gating pattern, not a new pane.

**Note:** the file extension MUST be `.svelte.ts` (Svelte 5 runes only compile in
`.svelte`/`.svelte.ts` modules — the three existing rune modules all use it).

---

### `src/components/ReviewPanel.svelte` (Svelte component — REWRITE in place)

**Analog:** the stub being replaced (whole file). Keep these two load-bearing
pieces, drop the lifecycle-button scaffolding:

**`session-changed` listener + canonical filter** (`ReviewPanel.svelte:81-93`) — keep:
```typescript
$effect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>("session-changed", (event) => {
        if (status && event.payload !== status.canonical_path) return;
        reloadStatus();
        reloadCommits();   // + reloadComments() + reloadResolutions()
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
});
```

**`safeInvoke` + toast-on-error read** (`ReviewPanel.svelte:24-48`) — the panel's
read pattern. CMT-01 adds `list_session_comments` + `resolve_session_comments`
reads; the UI-SPEC error string is "Failed to load review comments. Reload the
panel to retry." Inactive-session reads stay silent (line 45-47).

**Delete confirm — `@tauri-apps/plugin-dialog` `ask`** (analog `DiffPanel.svelte:505-516`):
```typescript
const { ask } = await import("@tauri-apps/plugin-dialog");
const confirmed = await ask("Delete this comment? This cannot be undone.", {
    title: "Delete comment",
    kind: "warning",
});
if (!confirmed) return;
await safeInvoke("delete_comment", { path: repoPath, id });
```
(UI-SPEC locks the body/title strings.)

**Styling:** the stub already uses `--color-*` vars and flex (`ReviewPanel.svelte:96-203`)
— follow it. The destructive affordances reuse `--color-danger`/`--color-danger-bg`/
`--color-danger-border` (`ReviewPanel.svelte:114-122`). Group headers, orphan badges,
"Add note" affordance per the UI-SPEC token table (orphan badge = `--color-warning`
on `--color-warning-bg`; accent reserved for the Review toggle + resolvable jump only).

---

### `src/components/RepoView.svelte` + `src/App.svelte` (wiring)

**Jump composition analog** — `jumpTo(comment)` chains the SAME functions RepoView
already uses for commit/file selection + scroll:

`handleCommitSelect(oid)` (`RepoView.svelte:285-318`) — loads files + detail, auto-opens the pane:
```typescript
async function handleCommitSelect(oid: string) {
    // …clearStagingDiff(); selectedCommitOid = oid;
    const [files, detail] = await Promise.all([
        safeInvoke<FileDiff[]>("list_commit_files", { path: repoPath, oid }),
        safeInvoke<CommitDetailType>("get_commit_detail", { path: repoPath, oid }),
    ]);
    commitFileDiffs = files; commitDetail = detail;
}
```

`handleCommitFileSelect(path)` (`RepoView.svelte:348-370`) — selects the file →
`DiffPanel` renders (`Source::Diff` → diff; `Source::FullFile` → full-file view).

`scrollToHunk` (analog `DiffPanel.svelte:257-265`) — scroll-to + transient highlight:
```typescript
function scrollToHunk(index: number) {
    const el = hunkElements[keys[index]];
    el?.scrollIntoView({ behavior: "smooth", block: "start" });
    el?.classList.add("hunk-highlight");
    setTimeout(() => el?.classList.remove("hunk-highlight"), 600);
}
```
Jump reuses this scroll+highlight after selecting the file, targeting
`[start_line, end_line]`. Graph scroll uses `commitGraphRef.scrollToOid`
(`RepoView.svelte:103-105`, called at `:345`).

**Panel render site to relocate** (`App.svelte:587-615`): the stub renders as a thin
bar above `RepoView` (`App.svelte:592-594`). UI-SPEC LOCKS the panel to the **center
pane** (where `DiffPanel` lives); the `review-session.svelte.ts` rune gates which of
{panel, diff} the center pane shows. Selection state to wire against:
`selectedCommitOid` / `selectedCommitFile` / `showDiff` (`RepoView.svelte:96-131`).

---

### TEST files (Wave 0)

**`review.rs` `#[cfg(test)]` — exact helper analogs to reuse:**
- `seeded_sessions(data_dir)` (`review.rs:1151-1164`): `TempDir` data dir + a sessions map seeded with one empty session keyed by a synthetic canonical path. No git repo needed — drives `add_commit_comment`/`edit_comment`/`delete_comment`/`list_session_comments` inner-core tests.
- `make_repo()` (`review.rs:808-827`): in-process `git2::Repository::init` with a 6-commit DAG (A root → B → C → D, side off B, merge). Drives the `resolve_all` classifier tests (real commits/trees/blobs, no mocks).
- `distinct_anchor()` (`review.rs:1167-1176`) and `loaded(data_dir, canonical)` (`review.rs:1178-1183`): anchor fixture + load-and-unwrap assertion helper.
- Test signature shape (`review.rs:1187-1194`): `let (canonical, sessions) = seeded_sessions(&data_dir); let req = …; <inner>(…); assert on loaded(&data_dir, &canonical)`.

**`review_store.rs` `#[cfg(test)]`** — extend the existing mod (`review_store.rs:153-173`)
with the v1→v2 migration + id-backfill + D-16-preserved cases.

**`ReviewPanel.test.ts`** — NEW vitest file; analog is any existing component test
(`DiffPanel.test.ts` for the mock-dialog + mock-invoke pattern). Covers
group-by-commit render, inline edit, delete-confirm (cancel aborts / confirm
invokes), jump-vs-orphan.

## Shared Patterns

### Testable `_inner(data_dir, …)` + thin `#[tauri::command]` wrapper + emit
**Source:** `review.rs:304-320` (`mutate_session_rmw`), `390-404` (`add_comment_inner`), `511-535` (`add_comment`).
**Apply to:** every new mutating command (`add_commit_comment`, `edit_comment`, `delete_comment`).
The `_inner` is the test wedge (provable with a `TempDir`, no Tauri runtime); the
thin command resolves canonical path + data dir, calls `_inner`, then emits
`session-changed`. Reads (`list_session_comments`, `resolve_session_comments`)
skip the RMW and the emit.

### Serde conventions (review schema)
**Source:** `types.rs:288-294` (the convention comment), `295-305` (enums), `307-336` (structs).
**Apply to:** all new schema/DTO types.
- Persisted/wire enums: PascalCase variants, NO `rename_all` (`OrphanReason`, mirrors `Source`/`Side`).
- Serialize structs: snake_case fields (`CommentResolution`, mirrors `SessionCommit`/`RebaseTodoItem`).
- Frontend-facing request structs: `#[serde(rename_all = "camelCase")]` (camelCase JS keys → snake_case Rust).
- `Option<T>` deserializes a missing field to `None` with no `#[serde(default)]`; a non-Option new field (`id: String`) needs `#[serde(default)]` to survive a v1 file.

### git2 read in `spawn_blocking` on a fresh repo handle
**Source:** `review.rs:605-616`; blob-read at `merge_editor.rs:54-62`; `find_commit`+`tree()` at `diff.rs:406-407`.
**Apply to:** `resolve_session_comments`.
Never hold the `RepoState` lock across git2 work — open the repo fresh inside the
block; the pure classifier (modeled on `intersect_graph_order:250-287`) runs there.

### `safeInvoke<T>` + `session-changed` listener (panel reactivity)
**Source:** `ReviewPanel.svelte:24-93`; toast helper `toast.svelte.ts`.
**Apply to:** the rewritten panel.
All IPC goes through `safeInvoke<T>` (CLAUDE.md); the panel reloads on
`session-changed` filtered by canonical path; errors toast via `showToast(…, "error")`;
inactive-session reads stay silent.

### Native confirm dialog (destructive actions)
**Source:** `DiffPanel.svelte:505-516`.
**Apply to:** delete (D-05). `@tauri-apps/plugin-dialog` `ask` (already a dep);
cancel is a no-op. UI-SPEC locks body "Delete this comment? This cannot be undone."
and title "Delete comment".

### Theme CSS custom properties only (no inline colors)
**Source:** `ReviewPanel.svelte:96-203` (the stub already complies); UI-SPEC token table.
**Apply to:** all panel UI. Every color references a `--color-*` var; layout via
flex/grid (CLAUDE.md non-negotiable). Accent (`--color-accent`) is reserved for the
active "Review" toggle and the resolvable-jump hover emphasis only — NOT for
comment rows, "Add note", or edit controls.

### Rune-module factory shape
**Source:** `undo-redo.svelte.ts:17-35` (and `toast`/`remote-state` siblings).
**Apply to:** `review-session.svelte.ts`. A `.svelte.ts` module exporting a
`createXxxState()` factory returning `{ state, ...methods }` with `$state(...)`
declared inside.

## No Analog Found

| File / Concern | Role | Reason |
|------|------|--------|
| `uuid` crate (v4) for `id` generation | dependency | No id-generator exists in the codebase. RESEARCH approves adding `uuid = { version = "1", features = ["v4"] }` (run `cargo search uuid` to pin). Do NOT hand-roll. Used in `add_commit_comment`, the line-anchored writer's id assignment, and the v1→v2 backfill. |

## Metadata

**Analog search scope:** `src-tauri/src/git/` (types, review_store), `src-tauri/src/commands/` (review, merge_editor, diff, lib.rs invoke_handler), `src/lib/` (types.ts, *.svelte.ts rune modules), `src/components/` (ReviewPanel stub, RepoView, DiffPanel), `src/App.svelte`.
**Files scanned:** 11 read this session; all cited ranges verified.
**Pattern extraction date:** 2026-05-26
