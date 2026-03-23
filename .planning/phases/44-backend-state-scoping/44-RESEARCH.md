# Phase 44: Backend State Scoping - Research

**Researched:** 2026-03-23
**Domain:** Rust/Tauri backend state management -- per-repo scoping of RunningOp
**Confidence:** HIGH

## Summary

Phase 44 converts the backend's `RunningOp` from a single global `Mutex<Option<u32>>` to a per-repo `Mutex<HashMap<String, u32>>`, matching the pattern already established by `RepoState`, `CommitCache`, and `WatcherState`. This is a straightforward refactor with no new dependencies -- the existing codebase already demonstrates the exact `Mutex<HashMap<String, T>>` pattern three times. The scope is small (4 files, ~50 lines changed) and purely backend.

The main subtlety is the `close_repo` command needing two behaviors: graceful close (leave running op, clean up watcher/cache) and force close (SIGTERM the running op first, then clean up). The `cancel_remote_op` command also needs a `path` parameter to target the correct repo's PID. The `remote-progress` event already includes `repo_path` in the payload (added in a prior milestone), so the event contract is already scoped.

**Primary recommendation:** Follow the existing `Mutex<HashMap<String, T>>` pattern exactly. The refactor is mechanical: change the type, update 4 call sites, add `path` parameter to `cancel_remote_op`, and add RunningOp cleanup to `close_repo`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** One operation per repo at a time (mutex keyed by repo path). Second op on same repo gets "op_in_progress" error. This is naturally enforced since TAB-10 (Phase 48) prevents duplicate repo tabs -- only one tab per repo path will exist.
- **D-02:** Normal close (`close_repo`) lets any running remote operation finish. Don't kill the subprocess -- clean up watcher and cache only. The PID entry in RunningOp stays until the process completes naturally.
- **D-03:** Force close (Shift+click tab X, implemented in Phase 45 UI) cancels the running remote op via SIGTERM before cleanup. Backend must expose both paths: graceful close (existing `close_repo`) and force close (new command or flag that also cancels the running op for that repo).
- **D-04:** Add repo path to the `remote-progress` event payload in this phase. The frontend can ignore the extra field until Phase 45 tabs exist, but the contract is ready.

### Claude's Discretion
- Internal structure of the per-repo RunningOp (HashMap vs other keying strategy)
- Whether `cancel_remote_op` takes a path param or a separate `force_close_repo` command handles cancellation
- Error message wording for per-repo "op_in_progress"

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BACK-01 | Concurrent remote operations across tabs (each tab can fetch/push independently) | Changing `RunningOp` from `Mutex<Option<u32>>` to `Mutex<HashMap<String, u32>>` directly enables per-repo independence. The existing pattern in RepoState/CommitCache/WatcherState proves this works. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- All git operations go through git2 crate, no shelling out (except GIT_EDITOR for rebase/merge message editing). NOTE: Remote ops (fetch/push/pull) already shell out via `tokio::process::Command` -- this is the established exception for operations needing SSH/credential agent access.
- Backend: Tauri 2, git2 0.19, notify 7, tokio 1
- Frontend-to-Backend: `invoke("command_name", args)` calls Rust `#[tauri::command]` fns
- Never inline colors, use CSS custom properties (not relevant for this backend-only phase)

## Standard Stack

No new dependencies. This phase modifies existing code only.

### Core (existing, unchanged)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2 | App framework, state management, IPC | Project foundation |
| tokio | 1 | Async runtime, subprocess management | Already used for remote ops |
| libc | 0.2 | SIGTERM for process cancellation | Already used in cancel_remote_op |
| serde_json | 1 | Event payload serialization | Already used for remote-progress |

### No New Dependencies Needed
The refactor uses only `std::collections::HashMap`, `std::sync::Mutex`, and existing Tauri state management (`State<'_, T>`, `.manage()`). No crate additions.

## Architecture Patterns

### Current State Architecture
```
lib.rs
  .manage(RepoState(Mutex<HashMap<String, PathBuf>>))      // per-repo: path -> pathbuf
  .manage(CommitCache(Mutex<HashMap<String, GraphResult>>)) // per-repo: path -> graph
  .manage(WatcherState(Mutex<HashMap<String, Debouncer>>))  // per-repo: path -> watcher
  .manage(RunningOp(Mutex<Option<u32>>))                    // GLOBAL: single PID <-- THE PROBLEM
```

### Target State Architecture
```
lib.rs
  .manage(RepoState(Mutex<HashMap<String, PathBuf>>))      // per-repo: unchanged
  .manage(CommitCache(Mutex<HashMap<String, GraphResult>>)) // per-repo: unchanged
  .manage(WatcherState(Mutex<HashMap<String, Debouncer>>))  // per-repo: unchanged
  .manage(RunningOp(Mutex<HashMap<String, u32>>))           // per-repo: path -> PID <-- THE FIX
```

### Pattern 1: Per-Repo State with Mutex<HashMap<String, T>>
**What:** Every piece of per-repo state wraps a `Mutex<HashMap<String, T>>` newtype struct. The key is the repo path (String), the value is the repo-specific data.
**When to use:** Any state that differs between open repositories.
**Example (existing, from state.rs):**
```rust
pub struct RepoState(pub Mutex<HashMap<String, PathBuf>>);
pub struct CommitCache(pub Mutex<HashMap<String, GraphResult>>);
```
**For RunningOp:**
```rust
// Before (global):
pub struct RunningOp(pub Mutex<Option<u32>>);

// After (per-repo):
pub struct RunningOp(pub Mutex<HashMap<String, u32>>);
```

### Pattern 2: Lock-Check-Operate for Mutual Exclusion
**What:** Lock the mutex, check if an entry exists (op in progress), and either return error or insert the new entry. All in a short-lived lock scope.
**When to use:** When enforcing "one operation at a time per repo."
**Example (adapted from existing run_git_remote):**
```rust
// Check mutual exclusion for THIS repo
{
    let guard = running.lock().unwrap();
    if guard.contains_key(repo_path) {
        return Err(TrunkError::new(
            "op_in_progress",
            format!("A remote operation is already running for {}", repo_path),
        ));
    }
}
// ... spawn subprocess ...
// Store PID for THIS repo
{
    let mut guard = running.lock().unwrap();
    guard.insert(repo_path.to_owned(), pid);
}
```

### Pattern 3: Cleanup on Close
**What:** When closing a repo, remove its entries from all state maps.
**When to use:** In `close_repo` and `force_close_repo`.
**Existing pattern (from close_repo):**
```rust
state.0.lock().unwrap().remove(&path);
cache.0.lock().unwrap().remove(&path);
watcher::stop_watcher(&path, &watcher_state);
```

### Anti-Patterns to Avoid
- **Holding the mutex across await points:** Never `.lock()` then `.await` -- this blocks other repos. Lock briefly, extract data, drop lock, then do async work.
- **Storing the HashMap key differently than other state types:** Always use the same `String` repo path key used by RepoState/CommitCache/WatcherState for consistency.
- **Leaving orphan PIDs:** If a repo is force-closed, the PID entry must be removed from the HashMap after SIGTERM, otherwise the map entry blocks future ops on the same path if the repo is reopened.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Per-repo mutex | Custom lock-per-repo system | Single `Mutex<HashMap<String, u32>>` | Proven pattern in 3 existing state types; single lock is sufficient for the low contention (human-speed UI actions) |
| Process cancellation | Custom signal handling | `libc::kill(pid, SIGTERM)` | Already used in `cancel_remote_op`, battle-tested |
| Async subprocess tracking | Custom task/future tracking | PID storage + tokio process | Existing pattern in `run_git_remote` works well |

**Key insight:** This entire phase is about changing a type signature and threading a `path` parameter through 4 call sites. No new abstractions needed.

## Common Pitfalls

### Pitfall 1: Forgetting to Remove the PID Entry on Process Completion
**What goes wrong:** `run_git_remote` spawns a subprocess, stores PID, then the process completes (success or failure). If the cleanup code (clearing the PID from HashMap) doesn't run on ALL exit paths, the map entry stays, permanently blocking that repo.
**Why it happens:** Error paths in async code can skip cleanup.
**How to avoid:** The existing code already clears `RunningOp` in a finally-equivalent block after `child.wait().await`. Keep this pattern -- clear the HashMap entry at the same point, regardless of success/failure.
**Warning signs:** After a failed remote op, the same repo's next remote op gets "op_in_progress" error.

### Pitfall 2: Graceful Close Leaving Orphan Map Entry (D-02)
**What goes wrong:** User gracefully closes a repo while a remote op is running. The watcher and cache are cleaned up, but the PID entry in RunningOp persists (by design per D-02). The subprocess finishes and tries to clear its PID, but the repo is already "closed" in RepoState.
**Why it happens:** The subprocess's cleanup code in `run_git_remote` executes after `close_repo` has already removed the repo from RepoState.
**How to avoid:** The PID cleanup in `run_git_remote` should unconditionally remove the entry from RunningOp's HashMap regardless of whether the repo is still in RepoState. This is safe -- removing a key from a HashMap that doesn't have it is a no-op.
**Warning signs:** HashMap grows indefinitely with stale entries.

### Pitfall 3: Changing cancel_remote_op Signature Without Updating Frontend
**What goes wrong:** Adding a `path` parameter to `cancel_remote_op` changes the IPC contract. If the frontend isn't updated to pass the path, calls fail silently.
**Why it happens:** Backend and frontend IPC contracts are loosely coupled.
**How to avoid:** Update the frontend `invoke('cancel_remote_op', { path })` call at the same time. However, currently `cancel_remote_op` is NOT called from the frontend (grep shows no usage in `src/`). It only exists as an exposed Tauri command. The frontend manages remote state via `remoteState` in Svelte. So changing the signature is safe for now -- Phase 45 will wire up the UI.
**Warning signs:** TypeScript invoke call without matching Rust parameter.

### Pitfall 4: Holding Mutex Lock Across the Subprocess Spawn
**What goes wrong:** If the mutex is held while spawning the subprocess or awaiting its completion, ALL repos are blocked from starting remote ops.
**Why it happens:** Natural coding instinct to "check and set" in one locked section.
**How to avoid:** Use two separate lock scopes: (1) check if an op is running for this repo, (2) after spawn, store the PID. There's a tiny TOCTOU window between check and store, but it's harmless because only one tab per repo exists (TAB-10/D-01).
**Warning signs:** Repos block each other during remote ops.

### Pitfall 5: run_git_remote Signature Change Ripple
**What goes wrong:** Currently `run_git_remote` takes `&Mutex<Option<u32>>`. Changing it to `&Mutex<HashMap<String, u32>>` requires also passing the repo path (already available as `repo_path: &str` parameter). Three callers (git_fetch, git_pull, git_push) pass `&running.0` -- all need updating.
**Why it happens:** Type change propagates through all callers.
**How to avoid:** Change signature and all 3 callers together. The compiler will catch any missed call sites.
**Warning signs:** Compilation errors (the Rust compiler makes this impossible to miss).

## Code Examples

### 1. New RunningOp Definition (state.rs)
```rust
// Source: derived from existing RepoState pattern in state.rs line 8
/// Stores the PID of the currently running remote operation per repo.
/// Key: repo path (String), Value: PID (u32).
/// Used for: (a) cancel button kills the subprocess, (b) mutual exclusion prevents
/// concurrent ops on the SAME repo.
pub struct RunningOp(pub Mutex<HashMap<String, u32>>);
```

### 2. Updated run_git_remote Mutual Exclusion (remote.rs)
```rust
// Source: adapted from existing run_git_remote in remote.rs lines 41-120
async fn run_git_remote(
    args: &[&str],
    cwd: &std::path::Path,
    app: &AppHandle,
    repo_path: &str,
    running: &Mutex<HashMap<String, u32>>,
) -> Result<(), TrunkError> {
    // Check mutual exclusion for THIS repo
    {
        let guard = running.lock().unwrap();
        if guard.contains_key(repo_path) {
            return Err(TrunkError::new(
                "op_in_progress",
                format!("A remote operation is already running for this repository"),
            ));
        }
    }

    // ... spawn subprocess (unchanged) ...

    // Store PID for THIS repo
    if let Some(pid) = child.id() {
        let mut guard = running.lock().unwrap();
        guard.insert(repo_path.to_owned(), pid);
    }

    // ... read stderr, emit progress (unchanged) ...

    // Clear RunningOp for THIS repo regardless of outcome
    {
        let mut guard = running.lock().unwrap();
        guard.remove(repo_path);
    }

    // ... check status, return result (unchanged) ...
}
```

### 3. Updated cancel_remote_op (remote.rs)
```rust
// Source: adapted from existing cancel_remote_op in remote.rs lines 251-261
#[tauri::command]
pub async fn cancel_remote_op(
    path: String,
    running: State<'_, RunningOp>,
) -> Result<(), String> {
    let mut guard = running.0.lock().unwrap();
    if let Some(pid) = guard.remove(&path) {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }
    Ok(())
}
```

### 4. Updated close_repo -- Graceful (repo.rs)
```rust
// Source: adapted from existing close_repo in repo.rs lines 36-46
#[tauri::command]
pub async fn close_repo(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
    // NOTE: RunningOp NOT cleaned up here -- D-02 says let running ops finish
) -> Result<(), String> {
    state.0.lock().unwrap().remove(&path);
    cache.0.lock().unwrap().remove(&path);
    watcher::stop_watcher(&path, &watcher_state);
    Ok(())
}
```

### 5. New force_close_repo Command (repo.rs)
```rust
// Source: new command per D-03
#[tauri::command]
pub async fn force_close_repo(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
    running: State<'_, RunningOp>,
) -> Result<(), String> {
    // Cancel running op first (D-03)
    {
        let mut guard = running.0.lock().unwrap();
        if let Some(pid) = guard.remove(&path) {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
    }
    // Then clean up all other state
    state.0.lock().unwrap().remove(&path);
    cache.0.lock().unwrap().remove(&path);
    watcher::stop_watcher(&path, &watcher_state);
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Global `RunningOp(Mutex<Option<u32>>)` | Per-repo `RunningOp(Mutex<HashMap<String, u32>>)` | This phase | Enables concurrent remote ops across repos |
| Single `cancel_remote_op()` (no params) | `cancel_remote_op(path)` | This phase | Can target specific repo's operation |
| `close_repo` only path | `close_repo` + `force_close_repo` | This phase | Graceful vs force close distinction |

## Open Questions

1. **Force close: separate command vs flag?**
   - What we know: D-03 says "new command or flag." Both approaches work.
   - Recommendation: Use a separate `force_close_repo` command. Cleaner than a boolean flag on `close_repo` -- the Tauri command handler gets the right state types injected automatically, and the intent is explicit. The planner should decide, but research recommends the separate command pattern.

2. **Remote-progress event: path field already present?**
   - What we know: The existing `run_git_remote` code (line 98) already emits `{"path": repo_path, "line": display}`. The frontend Toolbar already filters by `event.payload.path === path`. So D-04 is **already satisfied** in the current codebase. No code change needed for the event payload.
   - Recommendation: Verify this during implementation and skip if confirmed. Mark D-04 as "already done" in the plan.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + cargo test |
| Config file | `src-tauri/Cargo.toml` (dev-dependencies: tempfile 3) |
| Quick run command | `cargo test --lib -p trunk` |
| Full suite command | `cargo test -p trunk` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| BACK-01a | RunningOp allows concurrent ops on different repos | unit | `cargo test -p trunk -- running_op` | Wave 0 |
| BACK-01b | RunningOp blocks concurrent ops on same repo | unit | `cargo test -p trunk -- running_op` | Wave 0 |
| BACK-01c | cancel_remote_op targets specific repo | unit | `cargo test -p trunk -- cancel_remote` | Wave 0 |
| BACK-01d | close_repo does NOT cancel running op (graceful) | unit | `cargo test -p trunk -- close_repo` | Existing (extend) |
| BACK-01e | force_close_repo cancels running op then cleans up | unit | `cargo test -p trunk -- force_close` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib -p trunk`
- **Per wave merge:** `cargo test -p trunk`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Unit tests for new `RunningOp` HashMap behavior (insert/check/remove by path)
- [ ] Unit tests for `cancel_remote_op` with path parameter
- [ ] Unit tests for `force_close_repo` cleanup sequence

Note: Testing `run_git_remote` with actual subprocess spawning requires integration tests (Tauri AppHandle). The unit tests can verify the mutex/HashMap logic in isolation using the same pattern as existing `close_removes_state` test in `repo.rs`.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | Backend compilation + tests | Yes (via mise) | 1.94.0 | -- |
| bun | Frontend build/test | Yes | 1.3.11 | -- |
| node | Build tooling | Yes | 25.8.1 | -- |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

Note: Rust toolchain is installed via mise. The agent environment requires mise shell initialization to access `cargo` directly. The user's normal terminal has this configured.

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/state.rs` -- Existing state type definitions (RepoState, CommitCache, RunningOp)
- `src-tauri/src/commands/remote.rs` -- run_git_remote, git_fetch, git_pull, git_push, cancel_remote_op
- `src-tauri/src/commands/repo.rs` -- open_repo, close_repo
- `src-tauri/src/watcher.rs` -- WatcherState, start_watcher, stop_watcher
- `src-tauri/src/lib.rs` -- Tauri builder with .manage() registrations
- `src/lib/remote-state.svelte.ts` -- Frontend remote operation state
- `src/components/Toolbar.svelte` -- Frontend remote-progress listener and remote op invocation

### Secondary (MEDIUM confidence)
- None needed -- all findings derive from direct code inspection

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, using existing patterns
- Architecture: HIGH -- direct code inspection of all 4 files that need changes
- Pitfalls: HIGH -- derived from reading the actual async flow and understanding Mutex semantics

**Research date:** 2026-03-23
**Valid until:** Indefinite (internal refactor, no external dependency drift)
