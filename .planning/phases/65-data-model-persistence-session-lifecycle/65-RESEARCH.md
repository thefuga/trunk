# Phase 65: Data Model + Persistence + Session Lifecycle - Research

**Researched:** 2026-05-25
**Domain:** Tauri 2 + Rust local persistence (per-repo JSON document, serde data model, app_data_dir, atomic writes, path canonicalization, lifecycle commands)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Persist anchors as `(commit_oid, file_path, side, start_line, end_line, source)`. **Never** persist `hunk_index`/`line_index` (positions in the in-memory diff array, not source coordinates) and **never** persist diff options (`context_lines`/`ignore_whitespace`). Getting this wrong is a high-cost migration.
- **D-02:** `source` is a discriminator distinguishing diff-source anchors (Phase 67) from full-file-source anchors (Phase 68). `side` distinguishes old/new for diff-source anchors.
- **D-03:** Include `schema_version: 1` in the persisted document from the first commit.
- **D-04:** Comment **text** is stored independently of anchor resolvability, so render (Phase 70) can always surface a comment even when its anchor no longer resolves. Cache the anchor's excerpt at attach-time too (relevant 67+; the schema must accommodate it now).
- **D-05:** Define the *full* schema now (session + comment + anchor + enums), even though capture lands in 67/68. Later phases write into it, they do not reshape it.
- **D-06:** One active session per repo. Multiple concurrent sessions per repo is **SESS-F1** (future) — do not design for it now, but `schema_version` leaves the door open.
- **D-07:** Comment ordering is implicit capture-order in storage; render re-orders by line. No separate ordering/sequence field. No timestamps or author fields — single-user tool.
- **D-08:** `ReviewSessionsState: Mutex<HashMap<canonical_path, ReviewSession>>`, registered via `.manage()` in `lib.rs`, mirroring the existing `CommitCache` pattern at `src-tauri/src/state.rs:32`.
- **D-09:** One JSON file per repo under `app_data_dir`. **NOT** `tauri-plugin-store` / LazyStore — its read-modify-write is non-atomic and loses updates across v0.9 same-repo tabs. (Plugin stays installed for frontend prefs; just not used for sessions.)
- **D-10:** Write atomically via **tmp file + rename**. Every write goes through this path.
- **D-11:** **Path canonicalization is net-new for sessions.** Store keys by `std::fs::canonicalize(path)` so the same repo via symlink/different path string resumes the *same* session. The existing `RepoState`/`CommitCache`/`RunningOp` keying (raw `path` string) is **unchanged** — do not retrofit canonicalization onto them this phase.
- **D-12:** Add a **temporary** lifecycle trigger — a View-menu item (e.g. "Start/End Code Review") — plus a **bare review-panel stub** rendering three states: session-active (empty), no-session, resume-available. Throwaway scaffolding; **replaced by the real panel in Phase 69**. Do not over-invest.
- **D-13:** **End-and-clear (SESS-03) hard-deletes** the per-repo JSON file. No soft-archive/`status:ended` state. The rendered markdown (Phase 70/71) is the durable artifact.
- **D-14:** **Resume is prompted, not automatic.** On opening a repo with an existing session file, detect it and surface a resume indicator; the user clicks **Resume** to load into `ReviewSessionsState`. Opening a repo does not silently enter review mode.
- **D-15:** **Corrupt/unparseable JSON** → rename bad file to `.corrupt` sidecar, start fresh empty session, warn via toast. Never silently destroy a file we cannot read.
- **D-16:** **Newer `schema_version` than this build supports** → **refuse to load**, leave the file untouched, surface "this review session was created by a newer version of Trunk." Do **not** auto-create a fresh session, so a downgrade can never silently wipe newer data.

### Claude's Discretion
- Exact Rust struct/enum names and field naming (follow existing serde conventions — snake_case fields for Serialize-default structs, PascalCase enum variants).
- JSON filename scheme and on-disk layout under `app_data_dir` (subject to atomic tmp+rename, D-10).
- Whether lifecycle commands live in a new `src-tauri/src/commands/review.rs` (recommended, mirrors per-domain command files).

### Deferred Ideas (OUT OF SCOPE)
- **Soft-archive / past-session history** — rejected (D-13). Rendered markdown is the artifact.
- **Multiple concurrent sessions per repo (SESS-F1)** — Future Requirement; out of scope for v0.13.
- No comment *content capture* this phase — the session starts empty. (Capture lands in 67/68.)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SESS-01 | User can start a code review session for the current repository | `start_review_session` command + `save_session` helper + canonical-path keying (Standard Stack, Architecture Patterns) |
| SESS-02 | User can resume an in-progress session after restart (persists per repo) | atomic JSON persistence in `app_data_dir`, `load_session` with corrupt/newer recovery, `get_review_session_status` for resume detection (D-14/D-15/D-16 sections) |
| SESS-03 | User can end and clear the active session | `end_review_session` command → `delete_session` hard-deletes file + drops in-memory entry (D-13) |
</phase_requirements>

## Summary

This phase is almost entirely **patterns the codebase already establishes**, plus two genuinely net-new primitives: resolving `app_data_dir` from Rust and `std::fs::canonicalize`. There are no new external dependencies — `serde`, `serde_json`, `std::fs`, and the Tauri 2 path API (already linked via the `tauri` crate) cover everything. The persistence layer is a direct analog of the existing `CommitCache` pattern (`Mutex<HashMap>` newtype registered with `.manage()`), the command module mirrors `stash.rs` (thin `#[tauri::command]` wrapper over a testable `_inner` function), and the frontend live-coordination uses the same `app.emit` / `listen` event pattern already used for `repo-changed`.

Three facts were verified against primary sources and shape the design: (1) `AppHandle.path().app_data_dir()` returns `Result<PathBuf>` and **does not create the directory** — first write must `create_dir_all`; (2) `std::fs::canonicalize` resolves symlinks but **errors on non-existent paths** and applies the `\\?\` verbatim prefix on Windows, making the raw canonical path unsafe as a literal filename; (3) the codebase serializes enums as **PascalCase strings with no `rename_all`** (`RefType` → `"LocalBranch"`), which the new `Source`/`Side` enums must follow to stay consistent with the TS string-union mirror convention.

**Primary recommendation:** Build a new `src-tauri/src/commands/review.rs` exposing thin Tauri commands over `_inner(data_dir: &Path, ...)` functions (so the existing `tests/common/context.rs` harness can test persistence without Tauri state); store sessions at `app_data_dir/sessions/<hash-of-canonical-path>.json`; write atomically with std `tmp-in-same-dir + sync_all + rename`; key `ReviewSessionsState` by canonicalized path; broadcast a `session-changed` Tauri event after every successful mutation (DP-01); and put a single `Option<DraftComment>` field on the persisted `ReviewSession` now (DP-02), because ROADMAP Phase 67 explicitly requires "persist the draft comment on change, not only on submit."

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Session schema (anchor/comment/session DTOs) | API / Backend (Rust `git/types.rs` or `git/review.rs`) | Frontend mirror (`src/lib/types.ts`) | Schema is the keystone — Rust owns it; TS mirrors string-for-string like every other DTO |
| Per-repo JSON persistence (read/write/delete) | API / Backend (Rust `std::fs`) | — | D-09: Rust-owned, "std for local writes" project rule; never the frontend |
| Path canonicalization + filename hashing | API / Backend (Rust `std::fs::canonicalize`) | — | D-11: net-new; lives only in the session layer, not retrofitted onto existing keying |
| `app_data_dir` resolution | API / Backend (Tauri `AppHandle.path()`) | — | Only the Rust side can resolve the path resolver |
| Lifecycle commands (start/resume/end/status) | API / Backend (`commands/review.rs`) | Frontend trigger (menu + stub) | Mirrors per-domain command files; frontend only invokes |
| Live multi-tab coordination | API / Backend (`app.emit`) | Frontend (`listen`) | DP-01: reuse the existing `repo-changed` event precedent |
| Lifecycle trigger + panel stub | Frontend Server / Client (Svelte) | API (invoke) | D-12: throwaway UI; the menu item lives in `lib.rs` SubmenuBuilder |
| In-memory active-session cache | Managed State (`ReviewSessionsState`) | — | D-08: mirrors `CommitCache`; `Mutex<HashMap>` keyed by canonical path |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1 (already in Cargo.toml) | Derive `Serialize`/`Deserialize` on session/comment/anchor DTOs | Every DTO in `git/types.rs` already uses it `[VERIFIED: src-tauri/Cargo.toml]` |
| `serde_json` | 1 (already in Cargo.toml) | (De)serialize the per-repo JSON document; `serde_json::Value` peek for `schema_version` | Already the IPC serialization layer `[VERIFIED: src-tauri/Cargo.toml]` |
| `std::fs` | std | Atomic write (tmp + `sync_all` + `rename`), `create_dir_all`, `canonicalize`, hard-delete | Project rule: "git2/std for local writes, plugins for UI" `[CITED: CLAUDE.md, ARCHITECTURE.md]` |
| `tauri` path API | 2 (already linked) | `AppHandle.path().app_data_dir()` → `Result<PathBuf>` | Tauri 2's canonical per-app data dir resolver `[VERIFIED: docs.rs/tauri PathResolver]` |
| `std::hash` or inline digest | std | Stable hash of canonical path → safe filename | Avoids Windows `\\?\` prefix and `..` path-traversal in filenames (see Pitfalls) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | 3 (dev-dep) | TempDir for persistence round-trip tests | In `tests/` only — feed the temp dir as the `data_dir` arg to `_inner` fns |
| `@tauri-apps/api/event` | 2 (frontend) | `listen<string>("session-changed", ...)` in the stub | Frontend live-coordination (DP-01) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| inline std atomic write | `atomic-write-file` / `sapling-atomicfile` crate | Rejected — violates project "std for local writes" policy, adds a package-legitimacy gate, and the std pattern is ~10 lines. `[ASSUMED]` these crates exist; not needed. |
| one JSON file per repo (D-09) | `tauri-plugin-store` LazyStore | Rejected by D-09 — non-atomic read-modify-write loses updates across same-repo tabs |
| hashed filename | raw canonical path as filename | Rejected — Windows `\\?\` prefix and embedded separators/`..` make the raw path unsafe and non-portable as a filename `[VERIFIED: doc.rust-lang.org canonicalize]` |
| stable digest of path | `std::hash::DefaultHasher` | `DefaultHasher` is **not** guaranteed stable across Rust versions/builds — a session would orphan after a toolchain bump. Use a content-stable scheme instead (see Code Examples). |

**Installation:** None. No new packages — all dependencies already present in `src-tauri/Cargo.toml`.

**Version verification:** `serde 1`, `serde_json 1`, `tauri 2`, `tempfile 3` all confirmed present `[VERIFIED: src-tauri/Cargo.toml read 2026-05-25]`. No `npm view`/`cargo search` needed — nothing new is added.

## Package Legitimacy Audit

**No external packages are added by this phase.** All required crates (`serde`, `serde_json`, `std`, `tauri`, `tempfile`) already exist in `src-tauri/Cargo.toml`. No slopcheck/registry verification or `checkpoint:human-verify` task is needed. Disposition: N/A.

## Architecture Patterns

### System Architecture Diagram

```text
  ┌──────────────────────────────────────────────────────────────┐
  │ Svelte Frontend                                               │
  │                                                              │
  │  View menu "Start/End Code Review"  ──invoke──┐               │
  │  Review-panel stub (3 states)       ──invoke──┤               │
  │     ▲                                         │               │
  │     │ listen<string>("session-changed")       │               │
  └─────┼─────────────────────────────────────────┼──────────────┘
        │ event (canonical_path payload)           │ invoke(cmd, {path})
        │                                          ▼
  ┌─────┴──────────────────────────────────────────────────────────┐
  │ Rust Backend — commands/review.rs                              │
  │                                                                │
  │  start_review_session / resume_review_session /                │
  │  end_review_session / get_review_session_status                │
  │     │ (thin #[tauri::command] wrappers; spawn_blocking)        │
  │     ▼                                                          │
  │  *_inner(data_dir: &Path, path: &str, ...)  ◄── testable       │
  │     │                                                          │
  │     ├─ canonicalize(path) ─► canonical PathBuf (HashMap key)   │
  │     ├─ ReviewSessionsState: Mutex<HashMap<PathBuf, Session>>   │
  │     └─ persistence helpers (git/review_store.rs):              │
  │          load_session / save_session / delete_session /        │
  │          session_exists                                        │
  │             │                                                  │
  │             ▼ filename = hash(canonical_path)                  │
  └─────────────┼──────────────────────────────────────────────────┘
                │ std::fs (tmp + sync_all + rename / remove_file)
                ▼
  ┌────────────────────────────────────────────────────────────────┐
  │ Disk: app_data_dir/sessions/<hash>.json                        │
  │   (+ <hash>.json.tmp transient during write)                   │
  │   (+ <hash>.json.corrupt sidecar on D-15 recovery)             │
  └────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure
```text
src-tauri/src/
├── commands/
│   └── review.rs          # NEW: #[tauri::command] lifecycle fns (thin) + _inner
├── git/
│   ├── types.rs           # ADD: ReviewSession, Comment, Anchor, Source, Side, DraftComment
│   └── review_store.rs     # NEW (recommended): load/save/delete/exists + atomic write + filename hash
├── state.rs               # ADD: ReviewSessionsState(Mutex<HashMap<PathBuf, ReviewSession>>)
└── lib.rs                 # ADD: .manage(ReviewSessionsState), 4 commands in invoke_handler,
                           #      View-menu item + on_menu_event branch
src/
├── lib/
│   └── types.ts           # ADD: ReviewSession/Comment/Anchor/Source/Side TS mirrors
└── components/
    └── ReviewPanel.svelte  # NEW (throwaway stub): 3 states + listen("session-changed")
```

### Pattern 1: Managed-state newtype mirroring CommitCache (D-08)
**What:** A `Mutex<HashMap>` newtype registered with `.manage()`, injected into commands via `State<'_, T>`.
**When to use:** The in-memory active-session cache. The map **key is the canonicalized `PathBuf`** (not the raw String key the other maps use — D-11).
```rust
// Source: pattern from src-tauri/src/state.rs:32 (CommitCache)
// In state.rs:
use std::path::PathBuf;
pub struct ReviewSessionsState(pub Mutex<HashMap<PathBuf, crate::git::types::ReviewSession>>);
// In lib.rs (alongside the existing .manage calls ~line 64-67):
.manage(ReviewSessionsState(Default::default()))
```

### Pattern 2: Thin command over testable `_inner` (mirrors stash.rs)
**What:** `#[tauri::command] async fn` clones state, runs `_inner` in `spawn_blocking`, maps errors to JSON `TrunkError`. The `_inner` takes plain args so tests call it directly.
**When to use:** Every lifecycle command. **Critical wedge:** thread `data_dir: &Path` into `_inner` (the command resolves it from `AppHandle`), exactly as `stash.rs` threads `state_map: &HashMap`. This is what makes persistence testable via `tests/common/context.rs` with a `tempfile::TempDir`.
```rust
// Source: pattern from src-tauri/src/commands/stash.rs:147 (list_stashes)
#[tauri::command]
pub async fn start_review_session(
    path: String,
    state: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| serde_json::to_string(&TrunkError::new("app_data_dir", e.to_string())).unwrap())?;
    let result = tauri::async_runtime::spawn_blocking(move || {
        start_review_session_inner(&data_dir, &path)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    // ...insert into state, then broadcast (Pattern 4)...
    let _ = app.emit("session-changed", result.canonical_path_string);
    Ok(())
}
```
Note: `use tauri::Manager;` is required to call `app.path()` on an `AppHandle`.

### Pattern 3: Atomic write — tmp in same dir + sync_all + rename (D-10)
**What:** Write to `<final>.tmp` in the **same directory** (rename is only atomic within a filesystem), `sync_all()` to flush data to disk before the rename, then `std::fs::rename` to replace the target. Serialize with `serde_json::to_string_pretty` (not `to_string`) — session files are tiny (~KB) and a human *will* `cat` one when debugging an anchor that won't resolve.
**Mutation ordering (important):** On any successful mutation, do disk first: **atomic write succeeds → THEN insert/update `ReviewSessionsState` → THEN `app.emit`.** If you update in-memory state before the disk write and the write fails, memory and disk diverge silently.
**When to use:** Every `save_session`. See Code Examples for the full function.

### Pattern 4: Live coordination via Tauri event broadcast (DP-01 — RECOMMENDED)
**What:** After any successful session mutation, `app.emit("session-changed", canonical_path_string)`. The stub `listen<string>("session-changed", ...)` and reloads if the payload matches its repo's canonical path.
**When to use:** This is the simplest correct option and **reuses the exact pattern already in the codebase** (`repo-changed` emitted in `stash.rs:179`, consumed in `App.svelte:521` and `StagingPanel.svelte:724`). Recommended over tab-reload-on-focus (laggy) and last-write-wins-no-sync (stale UI in the other tab).
```rust
// emit (Rust): mirrors stash.rs:179
let _ = app.emit("session-changed", canonical_path_string);
```
```typescript
// listen (frontend): mirrors App.svelte:523
import { listen } from "@tauri-apps/api/event";
const unlisten = await listen<string>("session-changed", (e) => {
  if (e.payload === myCanonicalPath) reloadSession();
});
```

### Anti-Patterns to Avoid
- **Storing `git2::Repository` (or any git2 handle) in `ReviewSessionsState`:** not needed here, but the rule stands — store only owned data (`ReviewSession`, `PathBuf`). `[CITED: state.rs:5 "git2::Repository is not Sync"]`
- **Retrofitting canonicalization onto `RepoState`/`CommitCache`/`RunningOp`:** explicitly forbidden by D-11 — those keep their raw-String keying. Canonicalization lives *only* in the session layer.
- **Using the raw canonical path as a filename:** Windows `\\?\` prefix + embedded separators break it; hash it instead (Pitfall 1).
- **`#[serde(rename_all = "snake_case")]` on the new enums:** breaks the codebase convention — enums serialize as PascalCase strings (`RefType` → `"LocalBranch"`). See Pitfall 4.
- **Deleting the session file in `close_repo`/`force_close_repo`:** those only drop the *in-memory* entry so resume works on reopen. Only `end_review_session` (SESS-03) hard-deletes (D-13).
- **Bare `serde_json::from_str::<ReviewSession>()` for load:** can't distinguish "corrupt" (D-15) from "newer schema" (D-16). Peek `schema_version` from a `serde_json::Value` first (Code Examples).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON (de)serialization | manual string building | `serde` derive + `serde_json` | Round-trips DTOs, handles escaping, matches every other DTO |
| Per-app data directory | hardcoded `~/.config/...` paths | `AppHandle.path().app_data_dir()` | Cross-platform (macOS `~/Library/Application Support/com.joaofnds.trunk`, Windows `%APPDATA%`, Linux `~/.local/share`); derived from the bundle identifier `[VERIFIED: docs.rs/tauri]` |
| Symlink/path resolution | manual `..`/symlink walking | `std::fs::canonicalize` | OS-native realpath/`GetFinalPathNameByHandle` `[VERIFIED: doc.rust-lang.org]` |
| Atomic replace | write directly over the target | tmp + `sync_all` + `std::fs::rename` | A crash mid-write over the real file corrupts the session; rename is atomic within a filesystem |
| Schema-version peek | full deserialize then inspect | `serde_json::Value` partial parse | Lets you refuse a newer-version file (D-16) *before* a full deserialize that might also fail for unrelated reasons |

**Key insight:** Every primitive this phase needs already exists in std or the Tauri 2 API. The only genuinely net-new *code* is the ~10-line atomic-write helper, the filename hash, and the load-with-recovery state machine (D-15/D-16). No external dependency is justified.

## Runtime State Inventory

> This is a greenfield persistence layer (no existing session data on any user's disk yet — Phase 65 is the first to write `app_data_dir/sessions/`). It is **not** a rename/refactor. The inventory is therefore minimal, but recorded for completeness because the phase introduces new on-disk state.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None exists yet — this phase *creates* the first `app_data_dir/sessions/*.json` store. No migration of prior data. | None (greenfield) |
| Live service config | None — no external service involved. | None |
| OS-registered state | None — no OS task/daemon registration. | None |
| Secrets/env vars | None — session files contain no secrets (commit OIDs, file paths, comment text only). | None |
| Build artifacts | None — no generated/installed artifacts carry session identity. The app bundle identifier `com.joaofnds.trunk` is already fixed `[VERIFIED: tauri.conf.json:5]` and determines the `app_data_dir` location. | None |

**Forward note (not this phase):** Because D-03 ships `schema_version: 1` from day one, any *future* schema change becomes a migration concern. D-16 already guards the downgrade direction; an upgrade migration path is out of scope until SESS-F1 or a v2 schema.

## Common Pitfalls

### Pitfall 1: Using the canonical path directly as a filename
**What goes wrong:** `save_session` tries `app_data_dir/sessions/<canonical_path>.json` and either fails (separators/`\\?\` on Windows) or allows `..` traversal out of the sessions dir.
**Why it happens:** `canonicalize` returns a full absolute path with OS-specific prefixes — not a filename-safe token.
**How to avoid:** Hash the canonical path string to a fixed hex token (filename), and store the human-readable canonical path *inside* the JSON for debugging. `[VERIFIED: doc.rust-lang.org canonicalize — Windows `\\?\` prefix]`
**Warning signs:** Session works on macOS/Linux but file-not-found or path errors on Windows.

### Pitfall 2: `app_data_dir` directory does not exist on first write
**What goes wrong:** First `save_session` errors with "No such file or directory" because `app_data_dir()` only *resolves* a path; nothing has created `app_data_dir/sessions/`.
**Why it happens:** `app_data_dir()` returns a suggested `PathBuf`, not a created directory. `[VERIFIED: docs.rs/tauri PathResolver — "does not create the directory"]`
**How to avoid:** `std::fs::create_dir_all(&sessions_dir)?` before the first write. Cheap and idempotent.
**Warning signs:** Works in dev (dir created by a prior run) but fails on a fresh install / clean machine — a classic "works on my machine" bug. Test this explicitly (Validation Architecture).

### Pitfall 3: `canonicalize` errors on a non-existent path
**What goes wrong:** `canonicalize` returns `Err` if the path does not exist. For sessions this is benign (the repo is open, so its path exists), but a defensive caller must handle the `Result`.
**Why it happens:** `canonicalize` requires the path to exist (it resolves real inodes). `[VERIFIED: doc.rust-lang.org — "path does not exist" is an error case]`
**How to avoid:** Always `?`/map the `Result` to `TrunkError`. The repo path passed by the frontend is an open repo, so this should not fail in practice — but never `unwrap`.
**Warning signs:** Panic on a deleted-then-referenced repo path.

### Pitfall 4: Wrong serde enum representation breaks the TS mirror
**What goes wrong:** Adding `#[serde(rename_all = "snake_case")]` to `Source`/`Side` makes them serialize as `"diff"`/`"full_file"` while the rest of the codebase's enums serialize as PascalCase — the TS union mirror and any later code get out of sync, and an on-disk schema change becomes a migration.
**Why it happens:** The Phase 59 STATE.md "snake_case" rule is about **struct fields**, not enum variants. Codebase enums (`RefType`, `EdgeType`, `DiffStatus`, `OperationType`) use **PascalCase variants with no `rename_all`**, serializing as PascalCase strings. `[VERIFIED: src-tauri/src/git/types.rs + src/lib/types.ts — `RefType = "LocalBranch" | "RemoteBranch" | "Tag" | "Stash"`]`
**How to avoid:** Define `Source`/`Side` as plain PascalCase unit enums; the TS mirror is a PascalCase string union (e.g. `type Source = "Diff" | "FullFile"`, `type Side = "Old" | "New"`). Struct **fields** stay snake_case (Serialize default). Derive `Deserialize` too (the file is read back — unlike most existing DTOs which are write-only).
**Warning signs:** A session file written by one build won't deserialize in another; TS type errors against the on-wire shape.

### Pitfall 5: rename across filesystems / tmp in the wrong dir
**What goes wrong:** Writing the tmp file in a system temp dir then `rename` into `app_data_dir` can fail (`EXDEV`, cross-device link) if they're on different filesystems.
**Why it happens:** `std::fs::rename` is only atomic within a single filesystem.
**How to avoid:** Write the tmp file in the **same directory** as the target (`<final>.tmp`), then rename. `[CITED: users.rust-lang.org atomic-write thread]`
**Warning signs:** Intermittent "cross-device link" errors, or non-atomic partial writes.

## Code Examples

### Atomic write (the D-10 helper)
```rust
// Source: synthesized from std docs + users.rust-lang.org atomic-write thread.
// Project policy: inline std, no external crate.
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn atomic_write_json(final_path: &Path, json: &str) -> Result<(), TrunkError> {
    let dir = final_path.parent().ok_or_else(|| {
        TrunkError::new("bad_path", "session path has no parent dir")
    })?;
    fs::create_dir_all(dir).map_err(|e| TrunkError::new("io", e.to_string()))?;

    let tmp_path = final_path.with_extension("json.tmp"); // same dir → rename is atomic
    {
        let mut f = File::create(&tmp_path).map_err(|e| TrunkError::new("io", e.to_string()))?;
        f.write_all(json.as_bytes()).map_err(|e| TrunkError::new("io", e.to_string()))?;
        f.sync_all().map_err(|e| TrunkError::new("io", e.to_string()))?; // flush before rename
    }
    fs::rename(&tmp_path, final_path).map_err(|e| TrunkError::new("io", e.to_string()))?;
    Ok(())
}
```
*Note: macOS `F_FULLFSYNC` (true platter flush) is database-grade durability and unnecessary for a desktop review tool — `sync_all` is sufficient. `std::fs::rename` on Windows 10+ atomically replaces an existing target via `MoveFileExW`. `[ASSUMED: Windows rename-replace semantics — verify on a Windows build during Validation]`*

### Filename hash (stable across builds)
```rust
// DO NOT use std::hash::DefaultHasher — not stable across Rust versions.
// Use a content-stable digest. Hex-encode for a filename-safe token.
fn session_filename(canonical: &Path) -> String {
    // Option A (no new dep): FNV-1a or a small inline hash over canonical.to_string_lossy() bytes.
    // Option B (if a hasher is later justified): sha2. NOT added this phase — keep it dependency-free.
    let s = canonical.to_string_lossy();
    let mut hash: u64 = 0xcbf29ce484222325;       // FNV-1a 64-bit offset basis
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);  // FNV prime
    }
    format!("{:016x}.json", hash)
}
```
*FNV-1a is deterministic and dependency-free; collision risk for the handful of repos a user opens is negligible. The canonical path is also stored inside the JSON so a collision is detectable.*

### Load with corrupt (D-15) + newer-version (D-16) recovery
```rust
// Source: serde_json Value-peek pattern.
const CURRENT_SCHEMA_VERSION: u32 = 1;

enum LoadOutcome {
    Loaded(ReviewSession),
    None,                     // no file
    RecoveredCorrupt,         // D-15: renamed to .corrupt, caller starts fresh + toasts
    RefusedNewer,             // D-16: left untouched, caller warns, does NOT create fresh
}

fn load_session(final_path: &Path) -> Result<LoadOutcome, TrunkError> {
    let raw = match fs::read_to_string(final_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(LoadOutcome::None),
        Err(e) => return Err(TrunkError::new("io", e.to_string())),
    };

    // Step 1: peek schema_version without committing to the full shape.
    match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(v) => {
            let version = v.get("schema_version").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
            if version > CURRENT_SCHEMA_VERSION {
                return Ok(LoadOutcome::RefusedNewer); // D-16: leave file untouched
            }
            // Step 2: full deserialize. If THIS fails on a same-or-older version, treat as corrupt.
            match serde_json::from_value::<ReviewSession>(v) {
                Ok(session) => Ok(LoadOutcome::Loaded(session)),
                Err(_) => { quarantine_corrupt(final_path)?; Ok(LoadOutcome::RecoveredCorrupt) }
            }
        }
        Err(_) => { quarantine_corrupt(final_path)?; Ok(LoadOutcome::RecoveredCorrupt) } // D-15
    }
}

fn quarantine_corrupt(final_path: &Path) -> Result<(), TrunkError> {
    let corrupt = final_path.with_extension("json.corrupt");
    fs::rename(final_path, corrupt).map_err(|e| TrunkError::new("io", e.to_string()))
}
```

### Recommended Rust DTO shapes (D-01..D-07, DP-02)
```rust
// In git/types.rs — fields snake_case (Serialize default), enums PascalCase (no rename_all).
// All derive Deserialize too, because sessions are read back from disk.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewSession {
    pub schema_version: u32,           // D-03: always 1 for now
    pub commits: Vec<String>,          // commit OIDs in selection (Phase 66 fills this)
    pub comments: Vec<Comment>,        // capture-order (D-07); render re-orders by line
    pub draft_comment: Option<DraftComment>, // DP-02 — one in-progress draft (see DP-02 section)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub text: String,                  // D-04: stored independent of anchor resolvability
    pub anchor: Option<Anchor>,        // None = commit-level comment (ANCH-03, Phase 69)
    pub cached_excerpt: Option<String>,// D-04: excerpt cached at attach-time (filled 67+)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Anchor {
    pub commit_oid: String,            // D-01
    pub file_path: String,             // D-01 — path as it exists at the anchored commit/side
    pub source: Source,                // D-02
    pub side: Side,                    // D-02 (meaningful for Diff source; FullFile is always New)
    pub start_line: u32,               // D-01 (1-based source line)
    pub end_line: u32,                 // D-01
    // NEVER hunk_index / line_index / context_lines / ignore_whitespace (D-01)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Source { Diff, FullFile }    // serializes as "Diff" / "FullFile"

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Side { Old, New }            // serializes as "Old" / "New"

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DraftComment {             // DP-02
    pub text: String,
    pub anchor: Option<Anchor>,
}
```
```typescript
// src/lib/types.ts mirror — snake_case fields, PascalCase string unions.
export type Source = "Diff" | "FullFile";
export type Side = "Old" | "New";
export interface Anchor {
  commit_oid: string; file_path: string; source: Source; side: Side;
  start_line: number; end_line: number;
}
export interface Comment { text: string; anchor: Anchor | null; cached_excerpt: string | null; }
export interface DraftComment { text: string; anchor: Anchor | null; }
export interface ReviewSession {
  schema_version: number; commits: string[]; comments: Comment[];
  draft_comment: DraftComment | null;
}
```

## DP-01 — Same-repo multi-tab live coordination (DECIDE)

**Recommendation: (a) a `session-changed` Tauri event broadcast.** Strongest option because it *reuses an existing codebase pattern* rather than inventing one.

| Option | Verdict | Reason |
|--------|---------|--------|
| (a) `session-changed` event broadcast | **RECOMMENDED** | Mirrors `repo-changed` exactly (`stash.rs:179` emit, `App.svelte:521`/`StagingPanel.svelte:724` listen). Other same-repo tab reloads on the matching canonical-path payload. Lowest novelty, correct, already tested in mocks. |
| (b) tab-reload-on-focus | Acceptable fallback | Simpler still, but stale until the user refocuses — visible lag when editing in two tabs of the same repo. |
| (c) last-write-wins-no-sync | Reject | The other tab shows stale data and can clobber on its next write. Given v0.9 same-repo tabs share one canonical key (D-11), this is a correctness footgun. |

**Concrete API:** Backend `app.emit("session-changed", canonical_path_string)` after every successful mutation (start/resume/end/any future capture). Frontend `listen<string>("session-changed", e => { if (e.payload === myCanonicalPath) reload(); })` in `ReviewPanel.svelte`. Payload is the **canonical path string** (so a tab can match its own repo). `[VERIFIED: codebase event pattern]`

## DP-02 — Where the in-progress draft comment lives (DECIDE)

**Recommendation: a `draft_comment` field on the persisted `ReviewSession` (not component-level).**

The inputs contain a constraint that forces this: **ROADMAP Phase 67 notes state "persist the draft comment on change, not only on submit."** `[CITED: ROADMAP.md Phase 67 notes]` Component-level (Svelte) persistence cannot satisfy "survives an app restart mid-draft" — only the durable session file can. Per **D-05** ("define the *full* schema now... later phases write into it, they do not reshape it"), the schema shape decision belongs to this phase even though *capture* is Phase 67+.

- **Shape:** `draft_comment: Option<DraftComment>` — a single in-progress draft. One-at-a-time matches the single-anchor capture UX (the user is composing one comment against one selection). If multi-draft is ever needed it becomes a `Vec`, but YAGNI now (D-06 spirit).
- **This phase writes nothing into it** — the field exists and serializes as `null` until Phase 67 wires capture. Defining it now is the entire point of D-05; it avoids a schema reshape (and on-disk migration) later.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 `tauri::api::path::app_data_dir(&config)` (free function taking Config) | Tauri 2 `app.path().app_data_dir()` on the `PathResolver` via `Manager` trait | Tauri 2.0 | Use the v2 API only — v1 examples will not compile `[VERIFIED: docs.rs/tauri PathResolver]` |
| `tauri-plugin-store` for all persistence | Per-file std write for atomic/concurrent-safe documents | This project's D-09 | LazyStore's RMW loses concurrent updates; sessions use std |

**Deprecated/outdated:**
- Any v1 path-API snippet (`tauri::api::path::*`) — superseded by `AppHandle.path()`.
- `std::hash::DefaultHasher` for stable on-disk identifiers — not version-stable; use a fixed digest (FNV-1a inline).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `std::fs::rename` atomically replaces an existing file on Windows 10+ (`MoveFileExW`) | Code Examples / Pitfall 5 | If non-atomic on the target Windows version, a crash mid-rename could leave no session file. Verify on a Windows build (Validation). Mitigated by tmp-in-same-dir. |
| A2 | `atomic-write-file` / `sapling-atomicfile` crates exist on crates.io | Alternatives Considered | None — explicitly NOT used (std-only policy). Listed only to document the rejected path. |
| A3 | FNV-1a collision risk is negligible for the number of repos a single user opens | Code Examples (filename hash) | A collision would alias two repos' sessions. Canonical path stored inside JSON makes it detectable; could upgrade to sha2 if ever a concern. |

**Note:** Items A2/A3 are design-internal assumptions, not user-facing decisions. A1 is the one to verify empirically on Windows.

## Open Questions (RESOLVED)

1. **Exact `app_data_dir` subfolder name (`sessions/`) and filename hash algorithm**
   - What we know: layout `app_data_dir/sessions/<hash>.json` is sound; FNV-1a is stable and dependency-free.
   - What's unclear: nothing blocking — this is Claude's-discretion (CONTEXT.md). Planner just needs to pick and lock it.
   - RESOLVED: Planner locked `app_data_dir/sessions/<FNV-1a hex>.json` with the canonical path stored inside the JSON for collision detection (Plan 65-02).

2. **Should `resume_review_session` and `start_review_session` be one command or two?**
   - What we know: D-14 makes resume an explicit user action distinct from start; D-15/D-16 recovery applies on the resume/load path.
   - What's unclear: whether "start" on a repo that already has a file should be blocked (forcing the user to Resume or End-and-clear first) — this is a UX call.
   - RESOLVED: Planner locked four separate commands — `start_review_session` (creates fresh; rejects with a `session_exists` TrunkError when a file already exists, forcing the user to Resume or End-and-clear first rather than silently overwriting), `resume_review_session` (loads via the D-15/D-16 recovery state machine), `end_review_session` (deletes), `get_review_session_status` (drives the stub's 3 states). `start_review_session` requires the repo be open (`not_open` TrunkError otherwise). (Plan 65-03.)
   - Precondition: `start_review_session` must require the repo be open (present in `RepoState`) — SESS-01 says "for the currently open repository." Return a `not_open` `TrunkError` otherwise, matching the existing precondition pattern (`stash.rs:17` `open_repo` helper). Do not let a lifecycle command operate on a closed repo.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All backend code | ✓ | 1.93.1 (mise-pinned) | — |
| `serde` / `serde_json` | DTO (de)serialization | ✓ | 1 / 1 | — |
| `tauri` path API | `app_data_dir` | ✓ | 2 | — |
| `tempfile` (dev) | persistence tests | ✓ | 3 | — |
| Bun / Vitest | frontend stub tests | ✓ | 1.3.8 / 4.1 | — |

**Missing dependencies with no fallback:** None.
**Missing dependencies with fallback:** None. Everything required is already present `[VERIFIED: src-tauri/Cargo.toml, STACK.md]`.

## Validation Architecture

> `workflow.nyquist_validation: true` and `tdd_mode: true` in config.json — this section is required and tests come first.

### Test Framework
| Property | Value |
|----------|-------|
| Framework (Rust) | Rust built-in `#[test]` + integration tests in `src-tauri/tests/*.rs`; `tempfile 3` for temp dirs |
| Framework (Frontend) | Vitest 4.1 + @testing-library/svelte 5.3.1 (jsdom) |
| Config file | `vite.config.ts` (vitest block); Cargo test config implicit |
| Quick run command (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml <name>` |
| Quick run command (Frontend) | `bun run test -- <pattern>` (vitest) |
| Full suite command | `just check` (fmt, biome, svelte-check, clippy, cargo-test, vitest) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SESS-01 | Start creates an empty session, persists it, loads back equal | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml start_creates_session` | ❌ Wave 0 (`tests/test_review.rs`) |
| SESS-02 | save → load round-trip is byte-equal-shaped (serde) | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml session_round_trips` | ❌ Wave 0 |
| SESS-02 | resume after "restart": new `ReviewSessionsState`, load from disk yields same session | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml resume_after_restart` | ❌ Wave 0 |
| SESS-02 (crit #3) | symlink / alternate path string canonicalizes to the SAME file | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml symlink_resumes_same_session` | ❌ Wave 0 |
| SESS-03 | end hard-deletes the file; status then reports "none" | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml end_clears_session` | ❌ Wave 0 |
| D-15 | corrupt JSON → `.corrupt` sidecar + fresh session outcome | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml corrupt_quarantined` | ❌ Wave 0 |
| D-16 | `schema_version: 2` file → refused, left untouched | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml newer_version_refused` | ❌ Wave 0 |
| D-10 | atomic write leaves no `.tmp` and a valid file after success | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml atomic_write_clean` | ❌ Wave 0 |
| Pitfall 2 | first write on a non-existent `app_data_dir/sessions/` succeeds (create_dir_all) | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml first_write_creates_dir` | ❌ Wave 0 |
| serde shape (D-01..D-07) | `Source`/`Side` serialize PascalCase; anchor has no `hunk_index` | integration (Rust) | `cargo test --manifest-path src-tauri/Cargo.toml session_serde_shape` | ❌ Wave 0 (extend `tests/test_integ_serde.rs`) |
| D-12 stub | 3 states render; "session-changed" listener reloads | component (Vitest) | `bun run test -- ReviewPanel` | ❌ Wave 0 (`src/components/ReviewPanel.test.ts`) |

### Sampling Rate
- **Per task commit:** the specific `cargo test ... <name>` for the task's behavior.
- **Per wave merge:** `cargo test --manifest-path src-tauri/Cargo.toml` (all Rust) + `bun run test` (all frontend).
- **Phase gate:** `just check` green before `/gsd:verify-work`.

### Wave 0 Gaps
- [ ] `src-tauri/tests/test_review.rs` — covers SESS-01/02/03, D-10, D-15, D-16, canonicalization, dir-creation. Reuse `tests/common/context.rs` `TestContext` (TempDir-backed) and thread the temp dir in as `data_dir`.
- [ ] Extend `src-tauri/tests/common/context.rs` (or builder) with a session `data_dir` helper if needed (a `tempfile::TempDir` for `app_data_dir`-equivalent).
- [ ] Extend `src-tauri/tests/test_integ_serde.rs` — `session_serde_shape` (assert PascalCase enums, snake_case fields, absence of forbidden fields).
- [ ] `src/components/ReviewPanel.test.ts` — stub 3-state rendering + `session-changed` listener (mock `@tauri-apps/api/event` per `src/__tests__/helpers/tauri-mock.ts`).
- [ ] **Testability wedge:** `_inner` fns MUST take `data_dir: &Path` (Tauri command resolves it from `AppHandle`). Without this, persistence is untestable via the existing harness. This is the single most important structural requirement for the plan.

## Security Domain

> `security_enforcement` is not set (= enabled). This is a **local single-user desktop tool** writing JSON to its own `app_data_dir`; most ASVS categories do not apply. Applied categories below; others marked no with reason.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No accounts/auth — single local user (consistent with rest of app, ARCHITECTURE.md "No app-level auth") |
| V3 Session Management | no | "Session" here is a review document, not an auth session |
| V4 Access Control | no | Local files owned by the OS user; no multi-tenant boundary |
| V5 Input Validation | **yes** | Validate `schema_version` before trusting file (D-16); treat any unparseable file as untrusted (D-15 quarantine); never `unwrap` a deserialize |
| V6 Cryptography | no | No secrets stored; FNV-1a filename hash is for uniqueness, not security — do not present it as a security control |
| V12 File / Resource | **yes (light)** | Path-traversal mitigation: hashed filename means a malicious/odd repo path can't write outside `app_data_dir/sessions/`; atomic write avoids partial-file corruption |

### Known Threat Patterns for this stack
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via repo path used as filename | Tampering | Hash canonical path → filename can't contain `..`/separators (Pitfall 1) |
| Corrupt/hostile JSON crashes the app | Denial of Service | `Result`-based load, quarantine corrupt to `.corrupt`, never `unwrap` (D-15) |
| Downgrade silently wipes newer-schema data | Tampering / data loss | Refuse newer `schema_version`, leave file untouched (D-16) |
| Crash mid-write corrupts the session | (availability) | Atomic tmp + sync_all + rename (D-10) |

## Sources

### Primary (HIGH confidence)
- `docs.rs/tauri` — `PathResolver::app_data_dir()` returns `Result<PathBuf>`, does not create the directory; accessed via `AppHandle.path()` (needs `Manager` trait).
- `doc.rust-lang.org/std/fs/fn.canonicalize.html` — resolves symlinks, errors on non-existent paths, applies `\\?\` prefix on Windows.
- Codebase primary sources (read 2026-05-25): `src-tauri/src/state.rs` (CommitCache pattern), `src-tauri/src/commands/stash.rs` (thin-command/`_inner` pattern, emit), `src-tauri/src/commands/repo.rs` (open/close integration points), `src-tauri/src/git/types.rs` + `src/lib/types.ts` (enum serialize as PascalCase strings), `src-tauri/src/lib.rs` (`.manage`/invoke/menu), `src-tauri/tests/common/context.rs` + `test_integ_serde.rs` (test harness), `src-tauri/Cargo.toml` (deps), `src/App.svelte`/`StagingPanel.svelte` (`listen` pattern), `src-tauri/capabilities/default.json` (no new capability needed this phase).

### Secondary (MEDIUM confidence)
- users.rust-lang.org "How to write/replace files atomically?" — tmp-in-same-dir + fsync + rename; cross-device caveat.
- serde.rs enum-representations / container-attrs — confirms PascalCase default for unit enum variants; `rename_all` is opt-in.

### Tertiary (LOW confidence — flagged)
- Windows `std::fs::rename` atomic-replace semantics (A1) — verify empirically on a Windows build.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new deps; everything verified present in Cargo.toml.
- Architecture: HIGH — direct analog of existing `CommitCache`/`stash.rs` patterns, read from source.
- Pitfalls: HIGH — each verified against std/Tauri docs or codebase primary source.
- Windows atomic-rename (A1): MEDIUM — verify on a Windows build during validation.

**Research date:** 2026-05-25
**Valid until:** 2026-06-24 (stable domain — std + Tauri 2 APIs; 30 days)
