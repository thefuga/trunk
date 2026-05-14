# Codebase Concerns

**Analysis Date:** 2026-05-14

---

## Tech Debt

**Duplicated private helper functions across command modules:**
- Issue: `open_repo`/`open_repo_from_state`, `is_dirty`, and `is_head_unborn` are each copy-pasted into multiple command files instead of living in a shared utility.
- Files:
  - `src-tauri/src/commands/interactive_rebase.rs:19` — `fn open_repo`
  - `src-tauri/src/commands/stash.rs:11` — `fn open_repo`
  - `src-tauri/src/commands/commit.rs:19` — `fn open_repo_from_state`
  - `src-tauri/src/commands/merge_editor.rs:9` — `fn open_repo_from_state`
  - `src-tauri/src/commands/branches.rs:15` — `fn open_repo_from_state`
  - `src-tauri/src/commands/staging.rs:9` — `fn open_repo_from_state`
  - `src-tauri/src/commands/commit_actions.rs:12` — `fn open_repo`
  - `src-tauri/src/commands/operation_state.rs:12` — `fn open_repo`
  - `src-tauri/src/commands/diff.rs:15` — `fn open_repo_from_state`
  - `src-tauri/src/commands/branches.rs:27` — `fn is_dirty`
  - `src-tauri/src/commands/commit_actions.rs:22` — `fn is_dirty`
  - `src-tauri/src/commands/staging.rs:202` — `fn is_head_unborn`
  - `src-tauri/src/commands/diff.rs:25` — `fn is_head_unborn`
- Impact: Fixes to one copy don't propagate. The RETROSPECTIVE explicitly accepted this trade-off ("Duplicating small helpers is OK"), but the surface is now 9 copies, not the 2 originally justified.
- Fix approach: Extract into `src-tauri/src/commands/common.rs` or promote to `src-tauri/src/git/repository.rs` alongside `validate_and_open`.

**Boilerplate Tauri command wrappers:**
- Issue: Every Tauri command follows an identical pattern: lock state, clone, `spawn_blocking`, double `map_err` with `serde_json::to_string(...).unwrap()`. This pattern repeats 35+ times.
- Files: All files under `src-tauri/src/commands/` — every `pub async fn` wrapper.
- Impact: High noise-to-signal ratio. Any change to error serialization format requires touching every command.
- Fix approach: A macro or a helper such as `fn tauri_command<T, F>(state, f: F)` that inlines the spawn_blocking + error conversion.

**Merge/revert operations bypass the user's editor with `--no-edit` / `GIT_EDITOR=true`:**
- Issue: Three git operations silently use the default commit message without giving the user a chance to edit:
  - `merge_branch_inner` (`src-tauri/src/commands/operation_state.rs:300-306`) — `--no-edit` and `GIT_EDITOR=true`
  - `merge_continue_inner` else branch (`src-tauri/src/commands/operation_state.rs:167-173`) — `GIT_EDITOR=true`
  - `revert_commit_inner` (`src-tauri/src/commands/commit_actions.rs:153`) — `--no-edit`
- Impact: Violates the project's terminal-parity principle. Merge/revert commits always use the git-generated default message; users cannot customize them.
- Fix approach: Collect a message in the UI before invoking the command, then pass it as a parameter and use the file-based GIT_EDITOR script pattern already established in `src-tauri/src/commands/interactive_rebase.rs:158-179`. Tracked in `.planning/todos/pending/2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md`.

---

## Known Bugs / Design Gaps

**`discard_all` silently ignores untracked file/directory deletion failures:**
- Files: `src-tauri/src/commands/staging.rs:330-335`
- What happens: `let _ = std::fs::remove_file(file_path)` and `let _ = std::fs::remove_dir(parent)` swallow IO errors. If a file is locked or permissions deny deletion, `discard_all` returns `Ok(())` but leaves files on disk.
- Risk: User believes the discard succeeded but the file persists. No feedback to the frontend.
- Fix approach: Collect errors and return a `TrunkError` listing failed paths, or at minimum emit a warning toast.

**`start_interactive_rebase` silently drops `create_dir_all` failures:**
- Files: `src-tauri/src/commands/interactive_rebase.rs:145,251`
- What happens: `let _ = std::fs::create_dir_all(&msg_queue_dir)` and `let _ = std::fs::create_dir_all(&session_dir)`. If `/tmp` is full or permissions are denied the session directory is missing; subsequent file writes will then fail with a confusing `io_error`, not an obvious "couldn't create temp dir" message.
- Fix approach: Propagate errors from `create_dir_all` via `map_err(|e| TrunkError::new("io_error", ...))`.

**`notify` watcher uses `.expect()` — panics on watcher creation failure:**
- Files: `src-tauri/src/watcher.rs:28,33,38`
- What happens: `new_debouncer(...).expect("failed to create debouncer")` and `.watch(...).expect("failed to watch path")`. On Linux with inotify limits exhausted, or Windows with path-length issues, this panics the Tauri process rather than returning an error to the frontend.
- Fix approach: Propagate errors via `Result` and surface them as a non-fatal toast in the frontend.

**Interactive rebase shell scripts are Unix-only; Windows is silently broken:**
- Files: `src-tauri/src/commands/interactive_rebase.rs:132-178`
- What happens: The rebase session writes `.sh` shell scripts, sets `+x` permissions via `#[cfg(unix)]`, and invokes them as `GIT_SEQUENCE_EDITOR`/`GIT_EDITOR`. On Windows, git does not invoke `.sh` files directly without a POSIX shell.
- Impact: `start_interactive_rebase` will silently fail or hang on Windows. The `#[cfg(unix)]` block that sets permissions is guarded, but the `.sh` file writing and the `git rebase -i` invocation are not.
- Fix approach: On Windows, use batch scripts (`.cmd`) or point `GIT_SEQUENCE_EDITOR` at a compiled helper binary.

**`get_fork_point` shells out to `git merge-base` instead of using git2:**
- Files: `src-tauri/src/commands/interactive_rebase.rs:92-106`
- What happens: `std::process::Command::new("git").args(["merge-base", branch, "HEAD"])` — this is the only non-remote git subprocess that should be a pure git2 call.
- Impact: Violates the "all git operations go through git2" rule from CLAUDE.md. Also inherits PATH resolution issues on systems where git is not in `system_path()`.
- Fix approach: Use `repo.merge_base(local_oid, remote_oid)` from git2 0.19.

---

## Performance Bottlenecks

**`walk_commits` is called with `usize::MAX` (full graph) on every mutation:**
- Files: 35 call sites across 10 command files — every mutating command calls `graph::walk_commits(&mut repo, 0, usize::MAX)` after the mutation.
  - Examples: `src-tauri/src/commands/branches.rs:195,220,289,347,413,438`, `src-tauri/src/commands/stash.rs:77,103,134,144`, all `commit_actions.rs` commands.
- Problem: `walk_commits` traverses all commits reachable from all refs, computes lane assignments, and builds a `HashMap` of per-OID data — all in `O(n)` where `n` = total commits across all branches. For a repo with 100K commits this can take 2–5 seconds on every stage/unstage/commit action.
- Scaling limit: Repositories with >10K commits will show noticeable latency on every graph-mutating action.
- Improvement path: Incremental graph updates (invalidate only affected branches) or a background refresh that doesn't block the command response.

**HEAD chain pre-computation walks the entire first-parent chain on every graph build:**
- Files: `src-tauri/src/git/graph.rs:152-166`
- Problem: `while let Some(c_oid) = current { head_chain.insert(c_oid); ... }` walks every ancestor of HEAD on every call to `walk_commits`. On main branches with 50K+ commits this is a full `O(n)` traversal before the main loop even starts.
- Improvement path: Cache the head_chain set alongside the `GraphResult` in `CommitCache`, invalidating only when HEAD moves.

**`search_commits` clones the entire `CommitCache` before searching:**
- Files: `src-tauri/src/commands/history.rs:133`
- Problem: `let cache_map = cache.0.lock().unwrap().clone()` clones the full `HashMap<String, GraphResult>`. For a repo with 100K commits, `GraphResult` can be tens of MB; cloning it per search keystroke is wasteful.
- Improvement path: Hold the lock for the duration of the search (the search is already O(n) but synchronous), or switch to `Arc<GraphResult>` to make the clone cheap.

**`CommitCache` stores the full graph as `Vec<GraphCommit>` — all commits in memory:**
- Files: `src-tauri/src/state.rs:32`, `src-tauri/src/commands/history.rs:23-34`
- Problem: `GraphCommit` includes edges, refs, body text, and lane data per commit. With a large monorepo this can be hundreds of MB.
- Scaling limit: No eviction or size cap. The cache grows unbounded as repos are opened.
- Improvement path: Store only lane/column data and essential metadata in the cache; fetch body/diff on demand.

---

## Fragile Areas

**`CommitGraph.svelte` is the highest-complexity file (1,878 lines) with shallow test coverage:**
- Files: `src/components/CommitGraph.svelte`, `src/components/CommitGraph.test.ts` (188 lines, 3 basic render tests)
- Why fragile: The file owns all context menu construction, all graph action handlers (merge, rebase, cherry-pick, revert, reset, stash, branch operations, interactive rebase), virtual list scrolling, search integration, and SVG overlay coordination.
- Test coverage gap: Tests only assert that the component renders and shows column headers. All 20+ context menu actions are untested at the component level.
- Safe modification: Any change to action handlers (e.g., adding a parameter, changing toast behavior) has no test protection. Run Rust integration tests and do manual E2E verification after changes.

**Native Tauri context menu actions fire errors into `.catch(() => {})` — errors silently discarded:**
- Files: `src/components/CommitGraph.svelte:589,595,651,657,664,684,691,701,707,713,775,781,787,971,977,983,1000,1021,1027,1033,1042,1054` and `src/components/BranchSidebar.svelte:235,241,247,476,482,488,504,519,539,545,551,560`
- What happens: Native `MenuItem.action` callbacks cannot be async, so operations are called with `.catch(() => {})`. The async handlers (`handleMergeBranch`, `handleRebaseBranch`, `handleCherryPick`, `handleRevert`, `handleReset`, `handleCheckoutCommit`, stash ops) DO have try/catch with toasts internally — but if they throw synchronously before the catch, or if a secondary `.catch` fires, errors vanish.
- Risk: Hard to diagnose failures in the field because there is no logging fallback; the only observable symptom is that the UI does not update.
- Fix approach: Add a `.catch((e) => { console.error(...); showToast(..., "error"); })` fallback on all menu item action callbacks, or restructure to avoid the async-in-sync callback problem.

**`StagingPanel.svelte` (1,386 lines) owns staging, diff, merge editor, conflict resolution, and rebase banners:**
- Files: `src/components/StagingPanel.svelte`
- Why fragile: The component has 5 distinct operating modes (normal staging, merge in progress, rebase in progress, merge editor open, interactive rebase editor open). State transitions between these modes are managed with `$derived` and `$effect` chains that are difficult to reason about in isolation.
- Test coverage: `src/components/StagingPanel.test.ts` exists but is component-level; complex multi-mode transitions are not covered.
- Safe modification: Always test all operation modes manually after changes. Pay special attention to `isUndoing`/`isRedoing` guards and the `repo-changed` listener, which caused a race condition in v0.3 and needed a targeted fix.

**`WatcherState` holds live `Debouncer` objects inside a `Mutex<HashMap>`; poisoned mutex panics:**
- Files: `src-tauri/src/watcher.rs:10,38,43`
- What happens: `state.0.lock().unwrap()` — if any thread panics while holding this lock, all subsequent `unwrap()` calls panic the process. The debouncer callback thread is not guarded against panics.
- Fix approach: Use `lock().unwrap_or_else(|e| e.into_inner())` (poison recovery) or restructure to avoid a mutex over objects that hold OS resources.

---

## Security Considerations

**Interactive rebase writes shell scripts to `$TMPDIR` with predictable naming:**
- Files: `src-tauri/src/commands/interactive_rebase.rs:250`
- What happens: `std::env::temp_dir().join(format!("trunk-rebase-{}", std::process::id()))`. An attacker with local access could pre-create or replace the session directory before the process writes to it, injecting arbitrary rebase instructions or editor scripts.
- Current mitigation: Uses PID in the path, reducing predictability. Session dir is removed after completion (`interactive_rebase.rs:267`).
- Recommendation: Use `tempfile::tempdir()` (already a dep or easily added) to get an O_EXCL-created directory with a random suffix; or verify the directory did not exist before writing.

**`GIT_TERMINAL_PROMPT=0` is not set for remote operations:**
- Files: `src-tauri/src/commands/remote.rs:60-67` — `run_git_remote` constructs the git command with only `PATH` set.
- Risk: On systems where git falls back to an interactive terminal prompt (e.g., credential helper falls through to terminal), the subprocess could hang indefinitely waiting for stdin that never arrives. The RETROSPECTIVE notes that `GIT_TERMINAL_PROMPT=0` was established as a pattern in v0.3 but it is absent from `run_git_remote`.
- Fix: Add `.env("GIT_TERMINAL_PROMPT", "0")` to the `Command` builder in `run_git_remote` (`remote.rs:60`).

---

## Test Coverage Gaps

**`start_interactive_rebase_blocking` — the core rebase execution — has no integration test:**
- Files: `src-tauri/src/commands/interactive_rebase.rs:108-204`, `src-tauri/tests/test_interactive_rebase.rs`
- What's not tested: The actual execution of `git rebase -i` via the file-based IPC engine (todo file, seq-editor script, msg-queue, editor script). Only `get_rebase_todo` and `get_fork_point` are tested.
- Risk: Regressions in todo file construction, message queue ordering, or conflict handling go undetected.
- Priority: High — interactive rebase is the most complex command and has historically required the most debugging.

**Merge/rebase operation state transitions have no integration tests:**
- Files: `src-tauri/tests/test_operation_state.rs`, `src-tauri/src/commands/operation_state.rs`
- What's not tested: `merge_branch_inner`, `rebase_branch_inner`, `merge_continue_inner`, `rebase_continue_inner` — all the mutating state transitions.
- Risk: Conflict detection logic and error classification via string matching (`"conflict"`, `"could not apply"`) can break silently across git versions.
- Priority: High.

**Nyquist validation consistently skipped across all shipped phases:**
- Files: `.planning/phases/59-backend-data-model-diff-options/59-VALIDATION.md`, `60-VALIDATION.md`, `61-VALIDATION.md`, `62-VALIDATION.md`, `63-VALIDATION.md`, `64-VALIDATION.md` — all have `nyquist_compliant: false`.
- What's not tested: Each phase's VALIDATION.md is a stub. Cross-phase integration paths are not formally verified at the phase level.
- Risk: Integration gaps that phase-level execution misses (as happened with REB-06 in v0.8) will recur.
- Priority: Medium — the retrospective has noted this 8 consecutive milestones. Either enforce the validation step in the execute workflow or formally retire the requirement.

**Frontend mock tests are fragile — many components depend on custom mocks:**
- Files: All `src/components/*.test.ts` files using `vi.mock('@tauri-apps/api/core')`, `vi.mock('../lib/store')`, LazyStore mocks, OffscreenCanvas stubs.
- Risk: Mock drift — when the real `invoke` signature or store contract changes, mocks remain outdated and tests pass while the runtime breaks. This is identified in the RETROSPECTIVE as a known fragility.
- Priority: Medium.

---

## Scaling Limits

**`CommitCache` has no eviction policy — all open-repo graphs stay in memory:**
- Files: `src-tauri/src/state.rs:32`
- Current capacity: Fine for typical development repos (<10K commits, <10 open tabs).
- Limit: Multi-tab usage with large repos (Linux kernel, chromium) can exhaust memory. No size limit, no LRU eviction.
- Scaling path: Add a per-repo commit count limit (e.g., 50K) and evict on tab close.

**`walk_commits` builds a `HashMap<Oid, (col, edges, color, tip, stash)>` for ALL commits even when only a page is needed:**
- Files: `src-tauri/src/git/graph.rs:141-408`
- Current capacity: Works up to ~50K commits before perceptible latency.
- Limit: The full per-OID data map (`per_oid_data`) is built for every OID in the repo even when the requested page is only 200 commits. Lane continuity requires this, but the memory and CPU cost scales with total commit count, not page size.
- Scaling path: Separating lane-assignment state from commit data would enable streaming; alternatively, caching `per_oid_data` between pages so only the first page triggers a full walk.

---

*Concerns audit: 2026-05-14*
