# Phase 66: Commit Selection - Pattern Map

**Mapped:** 2026-05-25
**Files analyzed:** 6 (5 modified + 1 new interface)
**Analogs found:** 6 / 6 (all exact or strong role+data-flow matches)

All analogs below were read and verified against live code this session. Line numbers are current as of the read.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/commands/review.rs` (MODIFY) | command (backend) | CRUD / read-modify-write | self — Phase 65 lifecycle cmds `review.rs:63-274`; range walk from `interactive_rebase.rs:29-56` | exact (shape) + role-match (RMW differs — see Pitfall) |
| `src-tauri/src/lib.rs` (MODIFY) | config / command registration | — | `lib.rs:124-127` (existing `commands::review::*` registration block) | exact |
| `src/components/CommitGraph.svelte` (MODIFY) | component (context menu + membership Set) | event-driven | self — `showCommitContextMenu` (567-721); derived-Set `searchMatchOids` (293); prop wiring (1806) | exact |
| `src/components/CommitRow.svelte` (MODIFY) | component (row render / tint) | request-response (prop-driven) | self — `selected`/`isSearchMatch` tint precedent (61-115) | exact |
| `src/components/ReviewPanel.svelte` (MODIFY) | component (list + remove + listener) | event-driven | self — `session-changed` listener (55-66) + status reload (23-51) | exact |
| `src/lib/types.ts` (MODIFY) | type / DTO | — | `SessionStatus` interface (323), `GraphCommit` (55) | exact |

---

## Pattern Assignments

### `src-tauri/src/commands/review.rs` (command, read-modify-write)

**Analog:** itself (Phase 65 lifecycle commands) + `interactive_rebase.rs` (range walk).

Four new commands: `seed_review_range`, `add_review_commit`, `remove_review_commit`, `list_session_commits`. New `SessionCommit` Serialize struct.

**Imports / module conventions** — already present at `review.rs:14-21`. Reuse as-is; range walk additionally needs `git2`. The file already imports `TrunkError`, `review_store`, `ReviewSession`, `RepoState`, `ReviewSessionsState`, serde, `tauri::{AppHandle, Emitter, Manager, State}`.

**Canonical-path keying (every command uses this)** — `review.rs:50-58`:
```rust
fn canonical_repo_path(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<PathBuf, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    std::fs::canonicalize(path_buf).map_err(|e| TrunkError::new("io", e.to_string()))
}
```
NOTE Pitfall 3: this canonical key is for `review_store` (session file). `CommitCache`/`RepoState` are keyed by the RAW `path` string (`state.rs:8,32`). `list_session_commits` reads the session by canonical key and the graph by raw `path` — both from the same incoming `path` arg.

**Thin command wrapper shape (CREATE-only analog — copy the shape, NOT the read-no-prior-state assumption)** — `review.rs:149-173`:
```rust
#[tauri::command]
pub async fn start_review_session(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let (canonical, session) = tauri::async_runtime::spawn_blocking(move || {
        start_review_session_inner(&data_dir, &path, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    // Disk-first ordering (D-10): _inner already wrote the file → in-memory → emit.
    sessions.0.lock().unwrap().insert(canonical.clone(), session);
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}
```
The `_inner` for create is pure (`review.rs:63-83`): it reads NO prior session, only checks existence then writes a fresh one. **This is the trap (Pitfall 2):** add/remove/seed are read-modify-write on the persisted session. They must NOT independently read the in-memory map, compute, then write back — that races. Serialize the whole RMW under the `ReviewSessionsState` mutex: lock → read current session for canonical key → mutate → `save_session` while holding the lock → write back into the map → release → emit. Extract pure set logic (`apply_add`/`apply_remove`/`compute_range`) into testable helpers; keep mutex+IO orchestration in the thin command.

**`resolve_data_dir` helper (reuse verbatim)** — `review.rs:143-147`:
```rust
fn resolve_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| {
        serde_json::to_string(&TrunkError::new("app_data_dir", e.to_string())).unwrap()
    })
}
```

**Range walk — inclusive `[base..tip]` (D-02)** — analog `interactive_rebase.rs:40-56` (push HEAD there; here push `tip`):
```rust
// interactive_rebase.rs:40-56 — the verified push/hide + root-commit fallback.
let mut revwalk = repo.revwalk().map_err(TrunkError::from)?;
revwalk
    .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
    .map_err(TrunkError::from)?;
revwalk.push_head().map_err(TrunkError::from)?;   // SELECTION: push(tip) instead

if inclusive {
    let commit = repo.find_commit(base).map_err(TrunkError::from)?;
    if commit.parent_count() > 0 {
        revwalk
            .hide(commit.parent_id(0).map_err(TrunkError::from)?)
            .map_err(TrunkError::from)?;
    }
    // Root commit: don't hide anything — all commits included
} else {
    revwalk.hide(base).map_err(TrunkError::from)?;
}
```
For SEL-01: `revwalk.push(tip)` (not `push_head`), always inclusive of base via `hide(base.parent_id(0))`, root-commit fallback hides nothing. `base == tip` is valid (D-02 → set `{base}`); validate ancestry BEFORE the walk via `merge_base` (unrelated → Err) + `graph_descendant_of(tip, base)` (note `graph_descendant_of(x,x)==false`, so handle `base==tip` first). RESEARCH §"Code Examples / Detecting an invalid range" has the full `validate_range` helper.

**Set union + dedup (D-03)** — store order is irrelevant; `list_session_commits` re-imposes graph order on read:
```rust
let mut set: std::collections::HashSet<String> = session.commits.iter().cloned().collect();
for oid in range_oids { set.insert(oid); }
session.commits = set.into_iter().collect();
```

**`list_session_commits` — graph-ordered read (SEL-04)** — intersect session set with the FULL cached graph order. `CommitCache` is `HashMap<String, GraphResult>` (`state.rs:32`), built once at open via `walk_commits(.., 0, usize::MAX)` (`repo.rs:32,45`), keyed by RAW path. `GraphResult.commits: Vec<GraphCommit>` and `GraphCommit { oid, short_oid, summary }` (`types.rs:52-55,72-73`) map 1:1 to `SessionCommit`:
```rust
#[derive(Debug, Serialize, Clone)]
pub struct SessionCommit {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
}
// intersect with cache.get(raw_path).commits (graph order, dedup);
// append find_commit fallback for OIDs not in cache (never silently drop). See RESEARCH Pattern 3.
```

**serde naming** — Serialize-default structs snake_case (`SessionCommit`, matching `GraphCommit`/`SessionStatus`). Scalar command params received camelCase automatically by Tauri (`baseOid` ⇄ `base_oid`), as `get_rebase_todo_inner(base_oid: ...)` shows — no request wrapper struct needed. For request structs that DO need it, the precedent is `#[serde(rename_all = "camelCase")]` (`interactive_rebase.rs:10-11`).

**Testing** — `#[cfg(test)] mod tests` already at `review.rs:276-295` (pure-fn tests, no Tauri state). Add unit tests for the pure set helpers + revwalk against an in-process test repo (check for an existing `tempfile`/`Repository::init` helper first per RESEARCH Wave 0).

---

### `src-tauri/src/lib.rs` (config, command registration)

**Analog:** existing registration block `lib.rs:124-127`:
```rust
commands::review::start_review_session,
commands::review::resume_review_session,
commands::review::end_review_session,
commands::review::get_review_session_status,
```
Add the four new commands here (inside `tauri::generate_handler![` at `lib.rs:79`), after line 127. One line per command; no other change.

---

### `src/components/CommitGraph.svelte` (component, event-driven)

**Analog:** itself — context menu (567-721), derived membership Set (293), CommitRow prop wiring (1806).

**Context menu Add/Remove toggle (D-06)** — insertion point is the `items` array in `showCommitContextMenu` (`CommitGraph.svelte:643-718`). Item-build style to mirror (`648-659`):
```ts
await MenuItem.new({
    text: "Copy SHA",
    action: () => {
        writeText(commit.oid).catch(() => {});
    },
}),
```
Toggle: a single `MenuItem` whose `text`/`action` depend on `sessionOids.has(commit.oid)`, injected ONLY when a session is active. See RESEARCH Pattern 5 for the exact item.

**DO NOT COPY — merge gating (anti-pattern for D-08)** — `CommitGraph.svelte:680-693`:
```ts
await MenuItem.new({
    text: "Cherry-pick",
    enabled: !commit.is_merge,   // <-- THIS guard is for cherry-pick/revert ONLY
    action: () => {
        handleCherryPick(commit).catch(() => {});
    },
}),
await MenuItem.new({
    text: "Revert",
    enabled: !commit.is_merge,
    ...
```
D-08: merges ARE selectable. The Add/Remove toggle must NOT carry `enabled: !commit.is_merge`. The diff-source-on-merge restriction is deferred to Phase 67.

**Membership Set — analog is the derived search Set** — `CommitGraph.svelte:293`:
```ts
// Derived: Set of matching OIDs for O(1) lookup in CommitRow
const searchMatchOids = $derived(new Set(searchResults.map((r) => r.oid)));
```
VARIATION (do not blindly copy `$derived`): `sessionOids` is **event-driven**, not derived from reactive local state. It must be `let sessionOids = $state<Set<string>>(new Set())`, reloaded from `list_session_commits` on mount and on each `session-changed` for this repo (reassign `sessionOids = new Set(...)` so Svelte 5 reactivity fires — Pitfall 5). The `$derived` line above shows the Set-construction idiom only; the reload uses the ReviewPanel listener pattern below.

**Prop wiring to CommitRow** — `CommitGraph.svelte:1806` (existing call, shows the prop-passing style; add `inSession={sessionOids.has(commit.oid)}`):
```svelte
<CommitRow {commit} rowIndex={index} ... selected={commit.oid === selectedCommitOid && commit.oid !== '__wip__'} ... isSearchMatch={searchMatchOids.has(commit.oid)} isCurrentMatch={commit.oid === searchCurrentOid} ... />
```

**Required reading before touching graph render:** `.claude/rules/commit-graph.md` — but note the D-04 marker lives on the HTML `CommitRow`, NOT the SVG pipeline, so it stays clear of that boundary (see CommitRow below).

---

### `src/components/CommitRow.svelte` (component, prop-driven render)

**Analog:** itself — `selected`/`isSearchMatch`/`isCurrentMatch` tint precedent.

**Props block** — `CommitRow.svelte:18-43`. Add `inSession?: boolean` alongside `selected`, `isSearchMatch`, etc. (and optionally `isPendingBase` for the D-01 transient base highlight — see RESEARCH Open Question 3):
```ts
selected?: boolean;
/** True when this row's OID is in the search results */
isSearchMatch?: boolean;
/** True when this row is the current navigated match */
isCurrentMatch?: boolean;
```

**Row-tint precedent (D-04 home — extend this exact style expression)** — `CommitRow.svelte:61-72`:
```svelte
<div
  data-testid="commit-row"
  role="row"
  ...
  class:hover:bg-[var(--color-surface)]={!selected && !isCurrentMatch && !isSearchMatch}
  style:height="{rowHeight}px"
  style="color: var(--color-text); {isCurrentMatch ? 'background: rgba(245, 158, 11, 0.2);' : isSearchMatch ? 'background: rgba(234, 179, 8, 0.1);' : selected ? 'background: var(--color-selected-row);' : ''} {isSearchActive && !isSearchMatch && !isCurrentMatch ? 'opacity: 0.35;' : ''}"
  ...
>
```
Add an `inSession` branch driving a **theme CSS custom property** (project rule: never inline a literal color — note the existing `rgba(...)` literals are a pre-existing exception; the D-04 marker must use a `--color-*` var, e.g. `box-shadow: inset 3px 0 0 var(--color-accent)` or reuse/add `--color-selected-row`/`--color-review-row`).

**Anti-pattern (D-04):** the marker stays on this plain HTML flex row. Do NOT add membership styling to the SVG pipeline (`overlay-paths.ts`, `active-lanes.ts`, the `<g class="overlay-dots">` block) — violates `.claude/rules/commit-graph.md`. The SVG lanes are an absolutely-positioned overlay; this row never touches them.

**Testing** — no `CommitRow.test.ts` exists today (RESEARCH Wave 0). Add one for the `inSession` tint.

---

### `src/components/ReviewPanel.svelte` (component, event-driven)

**Analog:** itself — `session-changed` listener + status reload.

**`session-changed` listener (reuse verbatim, swap reload fn)** — `ReviewPanel.svelte:55-66`:
```ts
$effect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>("session-changed", (event) => {
        if (status && event.payload !== status.canonical_path) return;
        reloadStatus();
    }).then((fn) => {
        unlisten = fn;
    });
    return () => {
        unlisten?.();
    };
});
```
Panel already listens; extend `reloadStatus` (or add a sibling effect) to ALSO call `list_session_commits` and render the D-05 list.

**Reload-on-mount + invoke + toast-on-error pattern** — `ReviewPanel.svelte:23-51`:
```ts
async function reloadStatus() {
    try {
        status = await safeInvoke<SessionStatus>("get_review_session_status", { path: repoPath });
    } catch (e) {
        showToast((e as TrunkError).message ?? "Failed to load review session", "error");
    }
}
// $effect(() => { reloadStatus(); });  // initial load on mount / repo change
```
Mirror this for `list_session_commits` → `SessionCommit[]`. Render minimal rows (short SHA + summary, D-05), graph order already imposed server-side.

**Per-row × remove (D-07)** — button style precedent in the same file (`ReviewPanel.svelte:84-96`, the End-Review button); each list row calls `safeInvoke("remove_review_commit", { path: repoPath, oid })`.

---

### `src/lib/types.ts` (type / DTO)

**Analog:** `SessionStatus` interface (`types.ts:323`) and `GraphCommit` (`types.ts:55`). Add:
```ts
export interface SessionCommit {
    oid: string;
    short_oid: string;
    summary: string;
}
```
Mirrors the Rust `SessionCommit` (snake_case fields match the Serialize-default struct, like every other interface in this file).

---

## Shared Patterns

### Canonical-path keying
**Source:** `review.rs:50-58` (`canonical_repo_path`)
**Apply to:** every new selection command (session-file side).
Canonical key for `review_store`; RAW `path` for `CommitCache`/`RepoState` (Pitfall 3).

### Async command shape (spawn_blocking + JSON-stringified errors + data dir)
**Source:** `review.rs:143-147` (`resolve_data_dir`), `156-163` (spawn_blocking + error map)
**Apply to:** all four new commands.
Wrap disk work in `tauri::async_runtime::spawn_blocking`; JSON-stringify `TrunkError` into the `Err(String)`.

### Read-modify-write serialization (NEW — diverges from Phase 65)
**Source:** the GAP — create-only `_inner` at `review.rs:63-83` reads no prior state.
**Apply to:** `seed_review_range`, `add_review_commit`, `remove_review_commit`.
Hold the `ReviewSessionsState` mutex across read→mutate→`save_session`→map-write (Pitfall 2). Never hold it across an `.await`; the small disk write inside the lock is fine.

### Single emit per gesture
**Source:** `review.rs:171,226,248` — `app.emit("session-changed", canonical.to_string_lossy().into_owned())`
**Apply to:** every mutation (one emit per user gesture; do NOT decompose `seed_review_range` into N adds → N emits).

### Frontend session-changed listener + Set reload
**Source:** `ReviewPanel.svelte:55-66`
**Apply to:** ReviewPanel (list reload) AND CommitGraph (`sessionOids` reload). Reassign the Set so Svelte 5 reactivity fires.

### Theme CSS custom property only (no inline colors)
**Source:** project rule (`CLAUDE.md`) + existing `var(--color-*)` usage throughout `CommitRow.svelte` / `ReviewPanel.svelte`
**Apply to:** the D-04 graph marker and any new panel styling. Use a `--color-*` var, never a literal.

---

## No Analog Found

None. Every file has a strong in-repo analog (most are self-modifications extending an established pattern). The only genuinely NEW logic — revwalk range validation + set union/dedup + RMW mutex serialization — is built from verified primitives (`interactive_rebase.rs` walk, git2 `merge_base`/`graph_descendant_of`, the `ReviewSessionsState` mutex), documented inline above and in RESEARCH §"Code Examples" / §"Common Pitfalls".

## Metadata

**Analog search scope:** `src-tauri/src/commands/` (review.rs, interactive_rebase.rs, repo.rs, history.rs), `src-tauri/src/{state.rs, git/types.rs}`, `src/components/` (CommitGraph, CommitRow, ReviewPanel), `src/lib/types.ts`, `.claude/rules/commit-graph.md`.
**Files scanned:** 9 read in full or targeted ranges; all line-number citations verified live.
**Pattern extraction date:** 2026-05-25
