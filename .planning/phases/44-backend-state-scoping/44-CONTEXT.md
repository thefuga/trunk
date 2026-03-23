# Phase 44: Backend State Scoping - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Make RunningOp per-repo so each open repository can run remote operations (fetch, push, pull) independently without blocking other repositories. This is pure backend infrastructure — no frontend/UI changes.

</domain>

<decisions>
## Implementation Decisions

### Same-repo operation policy
- **D-01:** One operation per repo at a time (mutex keyed by repo path). Second op on same repo gets "op_in_progress" error. This is naturally enforced since TAB-10 (Phase 48) prevents duplicate repo tabs — only one tab per repo path will exist.

### Close-repo cleanup behavior
- **D-02:** Normal close (`close_repo`) lets any running remote operation finish. Don't kill the subprocess — clean up watcher and cache only. The PID entry in RunningOp stays until the process completes naturally.
- **D-03:** Force close (Shift+click tab X, implemented in Phase 45 UI) cancels the running remote op via SIGTERM before cleanup. Backend must expose both paths: graceful close (existing `close_repo`) and force close (new command or flag that also cancels the running op for that repo).

### Progress event scoping
- **D-04:** Add repo path to the `remote-progress` event payload in this phase. The frontend can ignore the extra field until Phase 45 tabs exist, but the contract is ready.

### Claude's Discretion
- Internal structure of the per-repo RunningOp (HashMap vs other keying strategy)
- Whether `cancel_remote_op` takes a path param or a separate `force_close_repo` command handles cancellation
- Error message wording for per-repo "op_in_progress"

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend state
- `src-tauri/src/state.rs` — Defines RunningOp (the one that needs changing), RepoState, CommitCache
- `src-tauri/src/watcher.rs` — WatcherState, start_watcher(), stop_watcher()

### Remote operations
- `src-tauri/src/commands/remote.rs` — git_fetch, git_pull, git_push, cancel_remote_op, run_git_remote()

### Repo lifecycle
- `src-tauri/src/commands/repo.rs` — open_repo, close_repo (state lifecycle management)

### App setup
- `src-tauri/src/lib.rs` — Tauri builder with .manage() for all state types

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **RepoState, CommitCache, WatcherState** — Already `HashMap<String, _>` keyed by repo path. Proven pattern to follow for RunningOp.
- **run_git_remote()** — Core async function handling subprocess spawn, PID tracking, stderr streaming, and cleanup. Needs path-scoped PID management.
- **classify_git_error()** — Error classification for remote ops. No changes needed.

### Established Patterns
- **Mutex<HashMap<String, T>>** — All per-repo state uses this pattern. RunningOp should follow the same.
- **Path-based keying** — All state lookups use `String` repo path as key, validated against RepoState.
- **spawn_blocking for git2** — Repository objects opened fresh per command, never stored.
- **Tauri event emission** — `app.emit("event-name", payload)` for backend→frontend communication.

### Integration Points
- **close_repo()** in `commands/repo.rs` — Currently cleans up RepoState, CommitCache, WatcherState. Needs to handle RunningOp cleanup (graceful: leave running; force: cancel then clean).
- **run_git_remote()** in `commands/remote.rs` — PID store/check/clear logic needs per-repo scoping.
- **cancel_remote_op()** in `commands/remote.rs` — Needs repo path parameter to target correct PID.
- **"remote-progress" event** — Payload needs repo path field added.

</code_context>

<specifics>
## Specific Ideas

- Normal tab close (X button) = graceful — let running ops finish
- Shift+click tab close = force — cancel running op then close
- Backend must support both paths so Phase 45 can wire up the UI distinction

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 44-backend-state-scoping*
*Context gathered: 2026-03-23*
