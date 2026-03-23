# Pitfalls Research: Trunk v0.9 Multi-tab & Tree View

**Domain:** Adding multi-repository tab management and directory tree file views to a single-repo Tauri 2 + Svelte 5 + Rust desktop Git GUI
**Researched:** 2026-03-23
**Confidence:** HIGH -- based on direct codebase analysis of all Rust state structs (state.rs, watcher.rs, remote.rs), all frontend state modules (store.ts, remote-state.svelte.ts, undo-redo.svelte.ts, toast.svelte.ts), App.svelte (631 LOC orchestrator), all 24 Svelte components, all 21 Tauri commands, and the established patterns from v0.1-v0.8

---

## Context: What Is Changing in v0.9

v0.8 shipped conflict resolution and interactive rebase. The codebase is ~9,300 LOC Rust, ~11,400 LOC Svelte/TS with 43 completed phases. Every piece of state -- Rust backend, frontend modules, Tauri events, LazyStore persistence, filesystem watchers -- currently assumes a single active repository.

v0.9 adds:
1. **Multi-tab repos:** Each tab opens a different repository with independent state
2. **Splash screen as new-tab page:** Opening a new tab shows the project picker / recent repos
3. **Tree view toggle:** Switch between flat file list and directory tree view in staging panel, commit diffs, and merge editor

---

## Critical Pitfalls

### Pitfall 1: Global Singleton Frontend State Leaks Between Tabs

**What goes wrong:**
Three frontend state modules use ES module-level `$state()` singletons: `remoteState` (remote-state.svelte.ts), `undoRedoState` (undo-redo.svelte.ts), and `showToast` (toast.svelte.ts). These are imported directly by components. When two tabs exist, both share the same `remoteState.isRunning` flag. Tab A starts a `git pull`, `remoteState.isRunning = true`. Tab B's Toolbar sees `isRunning = true` and disables all its remote buttons -- even though Tab B's repo has no running operation. Worse, when Tab A's pull completes and sets `remoteState.progressLine = ''`, Tab B loses any progress line it may have been displaying for its own operation.

The undo/redo stack (`undoRedoState.redoStack`) is even more dangerous: undoing a commit in Tab A pushes to the shared redo stack. Clicking Redo in Tab B re-commits Tab A's message to Tab B's repository -- silently creating a garbage commit in the wrong repo.

**Why it happens:**
ES module singletons are the standard Svelte 5 pattern for sharing state between sibling components in a single-instance app. The pattern works perfectly when there is exactly one active context. It breaks fundamentally when multiple independent contexts coexist in the same JavaScript runtime. Tauri uses a single webview with a single JS context, so all tabs share the same module scope.

**How to avoid:**
Replace each singleton with a per-repo state factory. Create a `RepoContext` class or object keyed by repo path that holds `remoteState`, `undoRedoState`, and all other per-repo state. Use Svelte 5's `setContext`/`getContext` to scope state to each tab's component subtree. The tab container creates the context, and child components consume it via `getContext`. This ensures Tab A's `remoteState.isRunning` is independent from Tab B's.

Concrete migration path:
- Create `repo-context.svelte.ts` exporting a `createRepoContext(path: string)` function
- Move `remoteState`, `undoRedoState`, per-repo selection state into this context
- Keep truly global state (toast notifications, zoom level, pane widths) as module singletons -- these are legitimately app-wide

**Warning signs:**
- Actions in one tab visually affect another tab's toolbar or status
- Undo in Tab A creates commits in Tab B's repo
- Remote progress lines appear in the wrong tab

**Phase to address:**
Phase 1 (backend state scoping) or Phase 2 (frontend tab architecture) -- this must be resolved BEFORE tabs exist, because the damage from cross-tab state leaks is data corruption (wrong commits in wrong repos)

---

### Pitfall 2: RunningOp is a Global Singleton Mutex -- Only One Remote Op Across All Tabs

**What goes wrong:**
`RunningOp(pub Mutex<Option<u32>>)` stores a single PID. When Tab A starts `git push`, RunningOp holds Tab A's subprocess PID. If Tab B tries `git fetch`, `run_git_remote` checks `running.lock().unwrap()` and finds `Some(pid)` -- it returns `op_in_progress` error. The user cannot fetch in one repo while pushing in another. Worse, `cancel_remote_op` kills whatever PID is stored -- which could be Tab A's push when the user clicked Cancel in Tab B.

**Why it happens:**
RunningOp was designed as mutual exclusion for a single repo -- preventing concurrent push+pull on the same repo (which would corrupt the git state). The single-valued design is correct for that purpose. But multi-tab requires per-repo mutual exclusion, not global.

**How to avoid:**
Change `RunningOp` from `Mutex<Option<u32>>` to `Mutex<HashMap<String, u32>>` keyed by repo path (matching the existing patterns of `RepoState`, `CommitCache`, and `WatcherState`). Update `run_git_remote` to check/set by repo path. Update `cancel_remote_op` to accept a `path` parameter and kill only that repo's subprocess. Mutual exclusion remains per-repo -- you cannot push and pull the same repo simultaneously, but you can push repo A while fetching repo B.

**Warning signs:**
- "Another remote operation is already running" error when operating on different repos
- Cancel button kills the wrong repo's operation

**Phase to address:**
Phase 1 (Rust backend state scoping) -- must be done before multi-tab is functional

---

### Pitfall 3: App.svelte Monolithic State -- 30+ State Variables Assume Single Repo

**What goes wrong:**
App.svelte holds ~30 `$state()` variables that are all implicitly scoped to "the current repo": `repoPath`, `repoName`, `refreshSignal`, `dirtyCounts`, `headBranch`, `wipSubject`, `selectedFile`, `stagingDiffFiles`, `selectedCommitOid`, `commitDetail`, `commitFileDiffs`, `selectedCommitFile`, `showRebaseEditor`, `rebaseEditorCommits`, etc. The `repo-changed` event listener (line 257) checks `event.payload === repoPath` -- it only processes events for the active repo. When switching tabs, ALL of this state must be swapped atomically. If any variable is not saved/restored correctly, the new tab shows stale data from the previous tab, or worse, state from repo A (commit detail, selected file, rebase editor) bleeds into repo B's view.

**Why it happens:**
App.svelte grew organically across 8 milestones as the single orchestrator. Each new feature added state variables to the same scope. This is the natural architecture for a single-repo app and it worked well. Multi-tab fundamentally requires either: (a) lifting this state into per-tab instances, or (b) save/restore the full state snapshot on tab switch.

**How to avoid:**
Extract the repo view into a dedicated `RepoView.svelte` component that encapsulates ALL per-repo state. App.svelte becomes a thin shell that manages the tab bar and renders one `RepoView` per tab. Each `RepoView` instance has its own `repoPath`, `refreshSignal`, `dirtyCounts`, `selectedFile`, etc.

Two architectural options:
- **Option A (mount/unmount):** Only the active tab's `RepoView` is mounted. Tab switching destroys and recreates the component. Simple but loses scroll position, selection state, and requires re-fetching data.
- **Option B (keep-alive):** All tabs' `RepoView` instances stay mounted but hidden (`display: none` for inactive tabs). Preserves all state and scroll positions. Uses more memory (one full component tree per tab) but provides instant tab switching. Memory cost is bounded: a tab is ~40 DOM nodes (virtual list) + cached graph data.

Option B is recommended because users expect instant tab switching (like browser tabs or VS Code tabs), and the memory cost is acceptable for a desktop app with 2-10 tabs.

**Warning signs:**
- Switching tabs shows previous tab's commit detail or diff
- Rebase editor from one repo appears over another repo's view
- Selected file highlight persists when switching to a tab that has no selection

**Phase to address:**
Phase 2 (frontend tab architecture) -- this is the core structural refactor

---

### Pitfall 4: Tauri Event Bus is Global -- All Listeners Receive All Events

**What goes wrong:**
Tauri's event system (`app.emit("repo-changed", path)`) broadcasts to ALL listeners in the webview. Currently, App.svelte's `listen<string>('repo-changed', ...)` filters by checking `event.payload === repoPath`. With multiple `RepoView` instances, each instance registers its own listener. When repo A's watcher fires `repo-changed` with repo A's path, ALL tab listeners receive the event. Each checks `event.payload === myRepoPath` and only repo A's tab processes it. This is correct -- BUT the filtering relies on string equality of the path payload. If the same repo is opened in two tabs (which should probably be prevented), both process the event, leading to duplicate refreshes.

The `remote-progress` event in Toolbar.svelte (line 23) also filters by `event.payload.path === path`. This works correctly per-tab only if each Toolbar receives its own `repoPath` prop.

**Why it happens:**
Tauri v2's event system has no built-in scoping -- it is a flat global bus. This is fine for single-window apps but requires discipline in multi-context setups.

**How to avoid:**
1. Keep the current filtering pattern -- it works correctly as long as each tab knows its own `repoPath` and checks `event.payload === myRepoPath`.
2. Prevent opening the same repo in two tabs simultaneously. On `open_repo`, check if the path is already in the tab list and switch to that tab instead of opening a duplicate. This prevents duplicate event processing and duplicate watchers.
3. Ensure cleanup: when a tab closes, its event listeners must be unregistered. Svelte's `$effect` cleanup functions handle this if the component is properly unmounted. But with the "keep-alive" pattern (Option B above), hidden tabs still have active listeners -- this is intentional (they should still process `repo-changed` to stay fresh).

**Warning signs:**
- Same repo opened in two tabs causes double refreshes
- Closing a tab leaves orphan event listeners that process events for a closed repo
- `remote-progress` lines appear in the wrong tab

**Phase to address:**
Phase 2 (frontend tab architecture) -- event listener lifecycle must be designed with tab mount/unmount

---

### Pitfall 5: LazyStore Persistence is Not Repo-Scoped

**What goes wrong:**
`store.ts` uses a single `LazyStore('trunk-prefs.json')` with flat keys: `open_repo`, `column_widths`, `column_visibility`, `zoom_level`, `left_pane_width`, `right_pane_collapsed`, etc. With multi-tab:

- `open_repo` stores a single `RecentRepo` -- but now there are multiple open repos. Which one is persisted for restore on next launch?
- `column_widths` is shared across all tabs. If the user resizes columns in Tab A, Tab B also gets those widths. This is probably fine (users generally want consistent column sizes). But if we later add per-repo column preferences, the flat key structure prevents it.
- `left_pane_width` and `right_pane_width` are app-wide layout state -- these SHOULD be shared across tabs.

The real danger is the `open_repo` key: on app launch, the restoration logic (App.svelte line 278-289) calls `getOpenRepo()` and opens that single repo. Multi-tab needs to restore ALL open tabs, their order, and which one was active.

**Why it happens:**
LazyStore was designed for a single-repo app. All keys are global because there was only one context.

**How to avoid:**
1. Replace `open_repo` with `open_tabs`: an ordered array of `{ path, name, active }` objects. On launch, restore all tabs and activate the last-active one.
2. Keep truly global keys as-is: `zoom_level`, `left_pane_width`, `right_pane_width`, `left_pane_collapsed`, `right_pane_collapsed`, `column_widths`, `column_visibility`, `recent_repos`. These are app-wide preferences that should be the same across all tabs.
3. If per-repo state is needed later (e.g., per-repo search filters), use namespaced keys: `repo:${path}:key_name`.
4. Save tab state on every tab open/close/switch, not just on app close. Prevents data loss if the app crashes.

**Warning signs:**
- App only restores one tab on relaunch instead of all open tabs
- Tab order not preserved across restarts
- Active tab reverts to first tab after relaunch

**Phase to address:**
Phase 2 or 3 (tab persistence) -- after basic tab switching works, before the feature is "complete"

---

### Pitfall 6: Filesystem Watcher Accumulation -- N Repos = N Recursive Watchers

**What goes wrong:**
Each `open_repo` call creates a new `notify` debouncer watching the repo directory recursively (`RecursiveMode::Recursive`). With 5 tabs open, 5 watchers run concurrently, each monitoring an entire repo directory tree. Large repos (monorepos, repos with `node_modules` not gitignored) can have 100k+ files. 5 such watchers can hit the OS file descriptor limit (`ulimit -n`, typically 256-10240 on macOS). The `notify` crate on macOS uses `kqueue` which requires one file descriptor per watched file in recursive mode (unlike Linux's `inotify` which uses one fd per directory).

Even without hitting fd limits, 5 concurrent watchers generate more debounced events, increasing CPU usage from the 300ms debouncer callbacks and the subsequent `get_dirty_counts` / `refresh_commit_graph` calls.

**Why it happens:**
The current watcher design is correct for single-repo use. It watches recursively because git status changes can happen in any subdirectory. The 300ms debounce keeps event rates manageable for one repo.

**How to avoid:**
1. Use `close_repo` diligently when tabs close -- the existing `stop_watcher` call in `close_repo` (repo.rs line 43) removes the watcher. Verify this cleanup path works when tabs are closed.
2. Consider watching only `.git/` directory changes plus the top-level directory (non-recursive) for most repos, using `RecursiveMode::NonRecursive`. Most git operations modify `.git/` files (HEAD, index, refs), and most working tree changes happen via editors that save files (triggering a workdir-level event). This dramatically reduces fd usage but misses nested directory changes.
3. Better approach: keep recursive watching but increase the debounce interval for background tabs. Active tab: 300ms. Background tabs: 2000ms. This reduces the event processing rate for repos the user isn't actively looking at.
4. Set a max tab limit (e.g., 15-20) with a warning. This is reasonable UX -- even browser tab hoarders rarely work with 20 repos simultaneously.
5. Monitor: log the watcher count and emit a warning toast if > 10 watchers are active.

**Warning signs:**
- "Too many open files" errors when opening multiple repos
- High CPU usage with many tabs open
- Sluggish UI when switching to a tab with a large repo

**Phase to address:**
Phase 1 (backend state scoping) -- watcher lifecycle is part of the `open_repo`/`close_repo` contract

---

### Pitfall 7: Tree View Path Splitting Correctness Across Platforms

**What goes wrong:**
Converting flat file paths (e.g., `src/lib/store.ts`) into a tree structure requires splitting by `/`. Git always uses forward slashes in its internal path representation (even on Windows). But the file paths returned by `git2`'s status/diff APIs use the repository-internal format (forward slashes). If the tree builder naively uses `path.split(os_separator)` instead of `path.split('/')`, it works on macOS/Linux but breaks on Windows where `os_separator` is `\`.

Additionally, paths with spaces, unicode characters, or deeply nested directories (20+ levels) can cause issues with naive tree construction.

**Why it happens:**
Developers test on their primary OS (macOS in this case) where `/` works. The bug only manifests on Windows, which is a target platform for v0.10 (CI/CD & Releases).

**How to avoid:**
Always split on `/` for tree construction, never on `path.separator`. Git normalizes paths internally, and `git2` returns forward-slash paths regardless of OS. Validate this assumption by checking `git2::StatusEntry::path()` documentation -- it returns paths relative to the workdir with forward slashes.

For the tree data structure: use a nested map/object, not an array of path segments. Each node: `{ name: string, children: Map<string, TreeNode>, files: FileStatus[] }`. Insert by walking the path segments and creating intermediate directories as needed.

**Warning signs:**
- Tree shows single deeply-nested chain instead of proper hierarchy
- Files appear at wrong nesting level
- Windows build shows flat list instead of tree

**Phase to address:**
Phase 3 or 4 (tree view implementation) -- when building the path-to-tree transformer

---

### Pitfall 8: Tree View Expand/Collapse State Lost on Refresh

**What goes wrong:**
When the filesystem watcher fires `repo-changed`, the staging panel reloads `get_status`. The file list changes. If the tree view is open with several directories expanded, the entire tree is rebuilt from the new file list. All expand/collapse state is lost -- every directory collapses back to default (typically collapsed). The user was looking at `src/lib/components/deep/file.ts`, the tree refreshes, and now they have to re-expand 4 directory levels to get back to their file.

This is the most common UX complaint about tree views in git GUIs. GitKraken handles it by preserving expanded paths across refreshes. VS Code's Source Control does the same.

**Why it happens:**
The obvious implementation rebuilds the tree from scratch on each status refresh. The tree structure is derived from the file list, so when the file list changes, the tree is recomputed. Without explicit state tracking, the expand/collapse state is ephemeral.

**How to avoid:**
Maintain a `Set<string>` of expanded directory paths, separate from the tree data structure. When the tree is rebuilt from new file data, apply the expanded set to restore the previous state. If a previously-expanded directory no longer exists in the new data (all its files were staged/committed), silently remove it from the expanded set.

Default behavior for new directories: collapsed. Exception: if a directory contains only one subdirectory (common: `src/` > `lib/` > `components/`), auto-expand single-child chains ("compact folders" like VS Code).

Store the expanded set in component state (not LazyStore) -- it should reset when switching repos but persist across refreshes within the same session.

**Warning signs:**
- User expands directories, stages a file, tree collapses completely
- Rapid re-expansion clicks cause UI jank from repeated tree rebuilds

**Phase to address:**
Phase 3 or 4 (tree view implementation) -- must be part of the initial tree view design, not added as a patch

---

### Pitfall 9: Tree View in Multiple Contexts with Different Semantics

**What goes wrong:**
The tree view must work in three distinct contexts:
1. **Staging panel:** unstaged files, staged files, conflicted files -- each in its own section with different actions (stage/unstage/discard)
2. **Commit detail:** files changed in a commit -- read-only list, click opens diff
3. **Merge editor file list:** conflicted files only

Each context has different file lists and different interaction behaviors. A single generic `TreeView` component that handles all three risks becoming a complex prop-driven monster with many conditional branches. Alternatively, three separate tree implementations lead to code duplication and inconsistent behavior (one gets a bug fix, the others don't).

**Why it happens:**
The current flat `FileRow` component is simple enough to reuse across contexts. A tree adds structural complexity (indentation, expand/collapse, directory nodes) that interacts differently with each context's actions.

**How to avoid:**
Build a single `FileTree.svelte` component that handles ONLY the tree structure (expand/collapse, indentation, directory grouping). It accepts a generic `files: { path: string, [key: string]: any }[]` array and an `onfileclick` callback. It does NOT know about staging, committing, or conflict resolution. Each parent context (StagingPanel, CommitDetail, MergeEditor) wraps `FileTree` and provides the appropriate file list and click handler.

The toggle between flat and tree mode should be a simple prop on a wrapper component (`FileListView.svelte`) that renders either `FileRow` items (flat) or `FileTree` (tree), using the same underlying data. The toggle state should be stored in LazyStore as a global preference (`file_list_mode: 'flat' | 'tree'`).

**Warning signs:**
- Tree view works in staging panel but breaks in commit detail
- Different expand/collapse behavior between staging and commit views
- Action buttons (stage/unstage) appear incorrectly in commit detail tree view

**Phase to address:**
Phase 3 or 4 (tree view implementation) -- design the component boundary before implementing

---

### Pitfall 10: Directory-Level Actions in Tree View Create Ambiguity

**What goes wrong:**
When files are shown as a tree, users expect to right-click a directory and "Stage all files in this directory" or "Discard all files in this directory." But what does "stage directory" mean when some files in the directory are new and others are modified? What about nested subdirectories? The current `stage_all` command stages everything -- there is no `stage_directory` command. Adding directory-level actions requires either: (a) calling `stage_file` in a loop for each file in the directory, or (b) adding a new backend command that stages a path prefix.

The loop approach is slow for large directories (N IPC round-trips) and has atomicity issues (what if staging fails halfway through -- some files staged, others not?). The backend command approach is cleaner but requires new Rust code.

**Why it happens:**
Directory-level operations are a natural expectation of tree views. Users see a directory node and want to act on it. But the backend was designed around individual file operations.

**How to avoid:**
For v0.9, keep directory-level actions simple:
1. **Stage directory:** Call `stage_file` in sequence for each file in the directory (including subdirectories). Show a loading indicator on the directory node. If any file fails, show an error but continue with the rest.
2. **Discard directory:** Show a confirmation dialog listing all files that will be discarded. Then call `discard_file` for each. This is destructive, so the explicit list is important.
3. Do NOT add backend bulk commands in v0.9 -- the IPC overhead for <100 files is negligible (<50ms). Optimize later if profiling shows it is a bottleneck.
4. Alternatively, defer directory-level actions entirely for v0.9 and only support them in a later milestone. The tree view itself (visual grouping + expand/collapse) provides value even without directory actions.

**Warning signs:**
- Staging a large directory freezes the UI (sequential IPC without async batching)
- Partial stage failure leaves directory in inconsistent visual state
- Discard directory without confirmation causes data loss

**Phase to address:**
Phase 4 or 5 (tree view polish) -- after basic tree view is working

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Keep-alive all tabs (never unmount) | Instant tab switching, preserved scroll | Memory grows linearly with tab count | Acceptable for v0.9 (desktop app, 2-10 tabs) |
| Global event bus with path filtering | No Tauri API changes, reuses existing events | Every listener checks every event | Acceptable indefinitely (N listeners x M events is small) |
| Flat LazyStore keys for tab state | Simple to implement | Cannot do per-repo preferences | Acceptable for v0.9; migrate to namespaced keys if per-repo prefs are needed |
| Sequential `stage_file` for directory staging | No new Rust commands | Slow for 100+ files, non-atomic | Acceptable for v0.9; add bulk command if profiling shows need |
| Rebuilding tree on every refresh | Simple implementation | Expand state lost without explicit tracking | Never acceptable -- must track expanded paths from day one |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Tab close + watcher cleanup | Forgetting to call `close_repo` when a tab closes, leaving orphan watchers consuming fd's | Always call `close_repo` on tab close; verify watcher map shrinks; add an `$effect` cleanup in `RepoView` that calls `close_repo` |
| Tab switch + stale event listeners | Old tab's `$effect` cleanup runs async, new tab's listeners register first -- brief window where both process events | Use `$effect` cleanup (Svelte guarantees synchronous cleanup before new effect runs); verify with concurrent tab switches |
| Tree view + virtual list | Trying to virtualize the tree view when there are <200 files (staging panel rarely has >100 changed files) | Skip virtualization for tree view initially; the staging panel is not the bottleneck. Only CommitDetail with huge commits (1000+ files) needs virtual tree -- defer this |
| Merge editor + tree view | Merge editor currently takes a single `filePath` prop. Tree view in merge context means navigating between conflicted files via the tree, requiring the merge editor to handle file switching | Keep merge editor as-is for v0.9; tree view in merge context is just a navigation aid that calls the existing `onfileselect` callback |
| Tab restoration + repo validation | Persisted tab paths may reference repos that have been moved or deleted since last session | On restore, validate each path with `open_repo`. If it fails, show the tab with an error state ("Repository not found at /path/to/repo") and offer to close or relocate |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| N watchers x recursive monitoring | High fd usage, "too many open files" error | Increase debounce for background tabs; cap tab count; consider non-recursive watching | >5 tabs with large repos (100k+ files each) on macOS |
| N cached GraphResults in memory | RSS grows by ~50-100MB per large repo (10k commits with edges/refs) | Evict graph cache for background tabs after timeout; re-walk on tab activation | >10 tabs with 50k+ commit repos |
| Tree rebuild on every status refresh | UI jank, lost expand state | Track expanded paths separately; diff old vs new tree and update incrementally | Repos with 500+ changed files (monorepo staging) |
| All tabs listen for all events | CPU cycles wasted checking event payloads | Negligible for <20 tabs; only optimize if profiled | Theoretical concern, not practical |
| Commit detail tree for huge commits | 1000+ file changes in one commit, tree has thousands of nodes | Virtualize only the commit detail tree; staging panel stays non-virtualized | Merge commits in monorepos (10k+ files) |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No visual indicator of background tab activity | User does not know Tab B's fetch completed while viewing Tab A | Show a dot/badge on the tab when background activity completes or status changes |
| Tab strip becomes unreadable with many tabs | Tab names truncate to nothing beyond 5-6 tabs | Set max tab width, add horizontal scroll to tab bar, show full name on hover tooltip |
| Tree view as default for new users | Tree adds visual complexity, new users may be overwhelmed | Default to flat list (matching current behavior); add toggle button; persist preference |
| No keyboard navigation in tree | Power users expect arrow keys to navigate tree nodes | Support Up/Down to move between visible nodes, Left to collapse, Right to expand, Enter to select file |
| Switching tabs resets center pane | User was viewing a diff, switches tabs, comes back -- diff is gone | With keep-alive approach, this is preserved automatically. With mount/unmount, save/restore the viewed file and diff |
| Opening same repo in two tabs | Duplicate watchers, confusing UX (edits in one tab affect the other silently) | Detect duplicate and switch to existing tab with a toast: "Already open in Tab 2" |

## "Looks Done But Isn't" Checklist

- [ ] **Tab close:** Verify `close_repo` is called (watcher stopped, cache cleared, RepoState entry removed) -- inspect WatcherState map size after close
- [ ] **Tab restore:** Open 3 repos in tabs, quit app, relaunch -- verify all 3 tabs restore in correct order with correct active tab
- [ ] **Background tab freshness:** Make a commit in Tab A's repo via terminal while Tab B is active -- switch to Tab A and verify the new commit appears without manual refresh
- [ ] **Tree view refresh:** Expand 3 directories in tree view, stage a file, verify the expanded directories stay expanded after status refresh
- [ ] **Tree view toggle persistence:** Switch to tree view, close app, relaunch -- verify tree view mode is restored
- [ ] **Cross-tab isolation:** Run `git push` in Tab A, switch to Tab B -- verify Tab B's toolbar is not disabled and shows no progress
- [ ] **Undo isolation:** Create commits in Tab A and Tab B. Undo in Tab A. Verify Tab B's redo stack is empty (not contaminated by Tab A's undo)
- [ ] **Duplicate repo prevention:** Try to open the same repo path in two tabs -- verify it switches to existing tab
- [ ] **Tree view in commit detail:** Click a commit with 50+ changed files, toggle to tree view, verify directory grouping is correct and click-to-diff still works
- [ ] **Empty tree view:** Repo with no changes -- tree view shows empty state, not a broken tree

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| State leaks between tabs (Pitfall 1) | MEDIUM | Refactor singleton modules to per-repo context; requires touching all components that import these modules |
| RunningOp global mutex (Pitfall 2) | LOW | Change from `Option<u32>` to `HashMap<String, u32>` and add `path` param to `cancel_remote_op`; localized Rust change |
| App.svelte monolith (Pitfall 3) | HIGH | Extract `RepoView.svelte` from App.svelte; largest single refactor in v0.9; must move ~30 state variables and ~15 handler functions |
| Event listener leaks (Pitfall 4) | LOW | Add missing cleanup in `$effect` return functions; audit each `listen()` call |
| LazyStore not tab-aware (Pitfall 5) | LOW | Replace `open_repo` key with `open_tabs` array; add save/restore logic |
| Watcher accumulation (Pitfall 6) | MEDIUM | Add debounce scaling for background tabs; requires watcher API changes to support dynamic debounce intervals |
| Tree path splitting (Pitfall 7) | LOW | Fix split character; isolated change in tree builder utility function |
| Tree expand state lost (Pitfall 8) | MEDIUM if discovered late | Retrofitting expand state tracking into an existing tree component; easier if designed in from the start |
| Tree in multiple contexts (Pitfall 9) | HIGH if wrong abstraction chosen | May require rewriting tree component with different component boundaries |
| Directory actions (Pitfall 10) | LOW | Sequential `stage_file` loop; can be optimized later without API changes |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| P1: Singleton state leaks | Backend state scoping (Phase 1) | Open 2 tabs, run operations in both, verify no cross-contamination |
| P2: RunningOp global mutex | Backend state scoping (Phase 1) | Fetch in Tab A while pushing in Tab B -- both succeed |
| P3: App.svelte monolith | Frontend tab architecture (Phase 2) | Switch between 3 tabs rapidly -- each preserves its own state |
| P4: Global event bus | Frontend tab architecture (Phase 2) | Watcher fires for repo A -- only Tab A refreshes |
| P5: LazyStore not tab-aware | Tab persistence (Phase 2 or 3) | Quit with 3 tabs, relaunch, all 3 restore correctly |
| P6: Watcher accumulation | Backend state scoping (Phase 1) | Open 10 tabs, check fd count stays reasonable (`lsof -p <pid> \| wc -l`) |
| P7: Tree path splitting | Tree view implementation (Phase 3-4) | Test with paths containing `/`, nested dirs, unicode characters |
| P8: Tree expand state | Tree view implementation (Phase 3-4) | Expand dirs, stage file, verify expanded state preserved |
| P9: Tree in multiple contexts | Tree view implementation (Phase 3-4) | Toggle tree in staging panel, commit detail, verify both work independently |
| P10: Directory actions | Tree view polish (Phase 4-5) | Right-click directory > stage all > verify all files staged |

## Sources

- Direct codebase analysis: `state.rs`, `watcher.rs`, `remote.rs`, `store.ts`, `remote-state.svelte.ts`, `undo-redo.svelte.ts`, `App.svelte`, all 21 Tauri commands, all 24 Svelte components
- [Tauri v2 State Management docs](https://v2.tauri.app/develop/state-management/)
- [Tauri multi-window best practices discussion](https://github.com/tauri-apps/tauri/discussions/9423)
- [Svelte 5 shared state patterns](https://joyofcode.xyz/how-to-share-state-in-svelte-5)
- [notify-rs crate documentation](https://docs.rs/notify)
- [GitHub Desktop multi-window discussion](https://github.com/desktop/desktop/issues/3606)
- [GitButler tree view feature request](https://github.com/gitbutlerapp/gitbutler/issues/7036)
- [GitHub blog: monorepo performance with FSMonitor](https://github.blog/engineering/infrastructure/improve-git-monorepo-performance-with-a-file-system-monitor/)

---
*Pitfalls research for: Trunk v0.9 -- Multi-tab & Tree View*
*Researched: 2026-03-23*
