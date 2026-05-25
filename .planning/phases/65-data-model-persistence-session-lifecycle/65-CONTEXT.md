# Phase 65: Data Model + Persistence + Session Lifecycle - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

A code review session exists as a durable, per-repo document the user can **start**, **resume** across app restarts, and **end/clear**. This phase delivers the keystone data model (anchor schema), the Rust-owned persistence layer, and the session lifecycle commands — plus a *temporary* UI trigger and panel stub so the lifecycle is verifiable end-to-end before the real review panel arrives in Phase 69.

**In scope:** anchor/comment/session schema (the keystone), per-repo JSON persistence in `app_data_dir`, start/resume/end commands + Rust state, path canonicalization, corrupt-file recovery, a throwaway lifecycle trigger + panel stub.

**Out of scope (later phases):** commit selection (66), anchor capture from diff/full-file (67/68), the real comment-management panel (69), markdown render (70), clipboard/save output (71). No comment *content capture* happens in this phase — the session starts empty.

</domain>

<decisions>
## Implementation Decisions

### Anchor Schema (keystone — locked by ROADMAP.md, gates every later phase)
- **D-01:** Persist anchors as `(commit_oid, file_path, side, start_line, end_line, source)`. **Never** persist `hunk_index`/`line_index` (those are positions in the in-memory diff array, not source coordinates) and **never** persist diff options (`context_lines`/`ignore_whitespace`). Getting this wrong is a high-cost migration.
- **D-02:** `source` is a discriminator distinguishing diff-source anchors (Phase 67) from full-file-source anchors (Phase 68). `side` distinguishes old/new for diff-source anchors.
- **D-03:** Include `schema_version: 1` in the persisted document from the first commit.
- **D-04:** Comment **text** is stored independently of anchor resolvability, so render (Phase 70) can always surface a comment even when its anchor no longer resolves. Cache the anchor's excerpt at attach-time too (relevant 67+; the schema must accommodate it now).
- **D-05:** Define the *full* schema now (session + comment + anchor + enums), even though capture lands in 67/68. The schema is the keystone; later phases write into it, they do not reshape it.
- **D-06:** One active session per repo. Multiple concurrent sessions per repo is **SESS-F1** (future) — do not design for it now, but `schema_version` leaves the door open.
- **D-07:** Comment ordering is implicit capture-order in storage; render (Phase 70 / DOC-03) re-orders by line. No separate ordering/sequence field needed. No timestamps or author fields — single-user tool, and nothing in the render contract (`path:Lstart-Lend (sha)` headings) consumes them.

### Persistence (Rust-owned — locked by ROADMAP.md)
- **D-08:** `ReviewSessionsState: Mutex<HashMap<canonical_path, ReviewSession>>`, registered via `.manage()` in `lib.rs`, mirroring the existing `CommitCache` pattern at `src-tauri/src/state.rs:32`.
- **D-09:** One JSON file per repo under `app_data_dir`. **NOT** `tauri-plugin-store` / LazyStore — its read-modify-write is non-atomic and loses updates across v0.9 same-repo tabs. (Plugin stays installed for frontend prefs; just not used for sessions.)
- **D-10:** Write atomically via **tmp file + rename**. Every write goes through this path.
- **D-11:** **Path canonicalization is net-new for sessions.** The store keys by `std::fs::canonicalize(path)` so the same repo opened via a symlink or a different path string resumes the *same* session (success criterion #3). The existing `RepoState`/`CommitCache`/`RunningOp` keying (raw `path` string, see `src-tauri/src/commands/repo.rs:34`) is **unchanged** — do not retrofit canonicalization onto them in this phase.

### Lifecycle Surface (scope of UI in this phase)
- **D-12:** Add a **temporary** lifecycle trigger — a View-menu item (e.g. "Start/End Code Review") — plus a **bare review-panel stub**. The stub renders three states: session-active (empty), no-session, and **resume-available**. This is throwaway scaffolding so SESS-01/02/03 are verifiable by hand this phase; it is **replaced by the real panel in Phase 69**. Do not over-invest in the stub.

### Session Lifecycle Semantics
- **D-13:** **End-and-clear (SESS-03) hard-deletes** the per-repo JSON file. Criterion #4 ("restarting shows no session") is then trivially satisfied; no soft-archive/`status:ended` state to manage. The rendered markdown (Phase 70/71) is the durable artifact — there is nothing to archive.
- **D-14:** **Resume is prompted, not automatic.** On opening a repo that has an existing session file, detect it and surface a resume indicator in the trigger/stub; the user clicks **Resume** to load the session into `ReviewSessionsState`. Opening a repo does not silently enter review mode. (The stub's resume-available state from D-12 backs this.)

### Corrupt / Incompatible File Recovery (split policy)
- **D-15:** **Corrupt/unparseable JSON** → rename the bad file to a `.corrupt` sidecar, start a fresh empty session, and warn via toast. Never silently destroy a file we cannot read.
- **D-16:** **Newer `schema_version` than this build supports** → **refuse to load**, leave the file untouched, surface "this review session was created by a newer version of Trunk." Do **not** auto-create a fresh session in this case, so a downgrade can never silently wipe data written by a newer app version.

### Deferred to the Planner (ROADMAP.md says "DECIDE in this phase's planning")
- **DP-01:** Same-repo multi-tab live coordination strategy — a `session-changed` Tauri event broadcast vs. tab-reload-on-focus vs. last-write-wins-no-sync. Has UX implications (two tabs of the same repo, edit in one). The planner picks; lean toward the simplest correct option given v0.9 same-repo tabs share one canonical key (D-11).
- **DP-02:** Where the in-progress draft comment lives — a `draft_comment` field on the persisted session vs. component-level persistence. (Capture itself is Phase 67+, but the schema decision belongs to this phase's plan since D-05 locks the full schema now.)

### Claude's Discretion
- Exact Rust struct/enum names and field naming convention (follow existing serde conventions in the codebase — see Phase 59 decisions: snake_case for Serialize-default structs, camelCase via `rename_all` for request options).
- JSON filename scheme and on-disk layout under `app_data_dir` (subject to atomic tmp+rename, D-10).
- Whether the lifecycle commands live in a new `src-tauri/src/commands/review.rs` (recommended, mirrors per-domain command files).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone & phase spec
- `.planning/ROADMAP.md` §"Phase 65" — goal, success criteria, and the locked notes (anchor schema, Rust-owned persistence, the two DECIDE-in-planning items). Also read the §"Phase 66/67/68/70" notes: they constrain what the schema must support (merge-commit policy, diff-index→source translation, full-file blob line numbers, render-time excerpt resolution).
- `.planning/REQUIREMENTS.md` — SESS-01/02/03 (this phase) plus the full anchor/comment/render requirement chain the schema must serve, and the "Out of Scope" table (no threading, no severity tags, no re-anchoring on history rewrite, static snapshot).

### Codebase patterns to mirror
- `src-tauri/src/state.rs` — `CommitCache(Mutex<HashMap<String, GraphResult>>)` at line 32 is the exact pattern `ReviewSessionsState` mirrors; note the "store PathBuf only, git2::Repository is not Sync" constraint at the top.
- `src-tauri/src/lib.rs` §`.manage(...)` (lines ~64-67) — where new managed state is registered; also lists installed plugins (dialog, store, window-state, clipboard-manager).
- `src-tauri/src/commands/repo.rs` `open_repo`/`close_repo` (lines 8-52) — current raw-`path`-string keying; the integration point where session load/unload likely hooks in.

### Project conventions
- `.planning/codebase/CONVENTIONS.md`, `STACK.md`, `ARCHITECTURE.md` — Rust/Svelte conventions, git2/std-for-local-writes-vs-plugins-for-UI pattern, command structure.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`CommitCache` pattern** (`src-tauri/src/state.rs:32`): the template for `ReviewSessionsState` — a `Mutex<HashMap<_, _>>` newtype registered via `.manage()`.
- **Per-domain command files** (`src-tauri/src/commands/*.rs`, e.g. `stash.rs`, `commit.rs`): pattern for a new `review.rs` holding `#[tauri::command]` lifecycle fns, registered in `lib.rs` `invoke_handler`.
- **Atomic local writes**: the project's stated rule is "git2/std for local writes, plugins for UI" — `std::fs` tmp+rename for the session file fits this exactly.

### Established Patterns
- **State keys by repo path** but currently **raw, non-canonicalized** (`repo.rs:34`). The session store must canonicalize independently (D-11) — this is the one place this phase diverges from existing keying.
- **No `app_data_dir` usage exists yet** — file persistence to `app_data_dir` is net-new; resolve it via the Tauri path API (`AppHandle.path()`).
- **serde conventions** (from Phase 59-64 decisions in STATE.md): snake_case for Serialize-default structs; camelCase via `rename_all` for frontend-facing request/option types. IPC payloads kept compact.

### Integration Points
- `open_repo` / `close_repo` in `repo.rs` — natural hooks to detect-and-offer-resume on open and to drop in-memory session state on close.
- `lib.rs` `.manage()` + `invoke_handler` — register `ReviewSessionsState` and the new review commands.
- View menu in `lib.rs` (SubmenuBuilder) — where the temporary "Start/End Code Review" trigger (D-12) is added.

</code_context>

<specifics>
## Specific Ideas

- The recipient of the eventual rendered doc is an **AI coding agent**, not a human reviewer — this is why the schema carries no severity tags / author / threading (the comment phrasing IS the instruction). Keep the data model lean accordingly.
- The lifecycle stub (D-12) is explicitly throwaway — favor the smallest thing that makes SESS-01/02/03 hand-verifiable, since Phase 69 replaces it.

</specifics>

<deferred>
## Deferred Ideas

- **Soft-archive / past-session history** — considered for SESS-03 and rejected (D-13). Belongs to a future milestone if ever; the rendered markdown is the artifact.
- **Multiple concurrent sessions per repo (SESS-F1)** — already tracked as a Future Requirement in REQUIREMENTS.md; out of scope for v0.13.
- No new scope-creep ideas surfaced — discussion stayed within phase boundary.

</deferred>

---

*Phase: 65-data-model-persistence-session-lifecycle*
*Context gathered: 2026-05-25*
