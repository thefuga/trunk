# Architecture Patterns: Multi-Tab & Tree View Integration

**Domain:** Desktop Git GUI -- multi-repository tab management and directory tree file views
**Researched:** 2026-03-23
**Confidence:** HIGH (based on full codebase audit of state.rs, App.svelte, all commands, watcher, store, and component hierarchy)

## Current Architecture Overview

```
Single-Repo Architecture (v0.8)
================================

 App.svelte (God Component)
   |-- repoPath: string | null           <-- single active repo
   |-- refreshSignal, dirtyCounts, etc.  <-- all repo-scoped state
   |
   |-- WelcomeScreen  (when repoPath === null)
   |-- TabBar          (shows single repo name + close button)
   |-- Toolbar         (receives repoPath)
   |-- BranchSidebar   (receives repoPath)
   |-- CommitGraph     (receives repoPath)
   |-- StagingPanel    (receives repoPath)
   |-- DiffPanel       (receives selectedFile data)
   |-- MergeEditor     (receives repoPath + filePath)
   |-- RebaseEditor    (receives repoPath + commits)
   |-- CommitDetail    (receives commitDetail + fileDiffs)

 Rust Backend (state.rs)
   RepoState:    Mutex<HashMap<String, PathBuf>>   <-- keyed by path, already multi-repo
   CommitCache:  Mutex<HashMap<String, GraphResult>> <-- keyed by path, already multi-repo
   WatcherState: Mutex<HashMap<String, Debouncer>>   <-- keyed by path, already multi-repo
   RunningOp:    Mutex<Option<u32>>                  <-- SINGLE global PID

 Events
   "repo-changed" -> payload: path string
   All Tauri commands take `path: String` as first arg
```

### Critical Observation: Rust Is Already Multi-Repo Ready

Every Rust state structure (RepoState, CommitCache, WatcherState) uses `HashMap<String, _>` keyed by repo path. The `open_repo` command inserts into these maps, `close_repo` removes. All commands accept a `path` argument and look up state by that key. The watcher emits `repo-changed` with the path as payload.

**The only Rust change needed is RunningOp** -- it currently stores a single global PID, preventing concurrent remote operations across repos.

### Where Multi-Repo Is NOT Ready: The Frontend

App.svelte is a monolithic god component with ~500 lines of script. It holds a single `repoPath` state variable and ALL repo-scoped state (refreshSignal, dirtyCounts, headBranch, selectedFile, commitDetail, rebase state, etc.) as top-level `$state` variables. The WelcomeScreen/repo-view toggle is a simple `{#if repoPath === null}` conditional. There is no concept of "tabs" or "multiple active repo contexts."

## Recommended Architecture: Multi-Tab

### Component Structure

```
New Architecture
=================

 App.svelte (Shell)
   |-- tabState (shared $state rune module)
   |   |-- tabs: TabInfo[]
   |   |-- activeTabId: string
   |
   |-- TabBar (multi-tab: shows all tabs, + button, close buttons)
   |-- {#if activeTab is "new-tab"}
   |     WelcomeScreen (splash / repo picker)
   |-- {:else}
   |     RepoView (new component -- extracts ALL current repo logic from App.svelte)
   |       |-- repoPath from activeTab
   |       |-- ALL current App.svelte repo state moves here
   |       |-- Toolbar, BranchSidebar, CommitGraph, StagingPanel, etc.
   |-- Toast (global, stays in App.svelte)
```

### New Types

```typescript
// src/lib/types.ts additions
interface TabInfo {
  id: string;           // crypto.randomUUID()
  kind: 'welcome' | 'repo';
  repoPath?: string;    // set when kind === 'repo'
  repoName?: string;
}
```

### New Shared State Module: tab-state.svelte.ts

```typescript
// src/lib/tab-state.svelte.ts
// Follows established pattern from remote-state.svelte.ts and undo-redo.svelte.ts

export const tabState = $state({
  tabs: [] as TabInfo[],
  activeTabId: '' as string,
});

export function addTab(tab: TabInfo) { ... }
export function removeTab(id: string) { ... }
export function setActiveTab(id: string) { ... }
export function updateTab(id: string, patch: Partial<TabInfo>) { ... }

// Derived helper
export function getActiveTab(): TabInfo | undefined {
  return tabState.tabs.find(t => t.id === tabState.activeTabId);
}
```

### Data Flow: Tab Switching

```
User clicks tab B (was on tab A)
  |
  1. tabState.activeTabId = tabB.id
  |
  2. App.svelte's {#key activeTabId} block re-renders
  |   Option A: {#key} destroys tab A's RepoView, mounts tab B's RepoView
  |   Option B: render all RepoViews, hide inactive with CSS display:none
  |
  3. RepoView for tab B mounts with repoPath = tabB.repoPath
  |   - If first mount: calls open_repo, starts watcher, populates state
  |   - If re-mount (option A): calls open_repo again (idempotent -- just re-inserts HashMap)
  |   - If unhide (option B): already mounted, just becomes visible
  |
  4. repo-changed events fire for ALL open repos continuously
  |   - Each RepoView's $effect filters: event.payload === repoPath
  |   - Already works this way! No change needed.
```

### Mount Strategy Decision: Destroy/Recreate vs Keep-Alive

**Recommendation: Destroy/Recreate with `{#key}`** because:

1. **Memory**: Each RepoView + CommitGraph + SVG overlay consumes significant memory. Keeping 10+ repo views mounted simultaneously is wasteful.
2. **Simplicity**: No need to manage hidden component lifecycle, pausing/resuming effects, or handling visibility changes.
3. **Existing pattern**: The Rust backend already caches graph data in CommitCache (keyed by path). Re-mounting a RepoView that calls `open_repo` is fast because the cache is populated from the existing HashMap entry.
4. **Perceived speed**: Tab switch latency is dominated by Rust `walk_commits` on first open. Subsequent switches hit the cache and feel instant.
5. **The tradeoff**: Some per-tab UI state (scroll position, selected commit, expanded sections) is lost on switch. This is acceptable for v0.9 and can be enhanced later with a lightweight state snapshot in tabState.

```svelte
<!-- App.svelte -->
{#if activeTab?.kind === 'welcome' || !activeTab}
  <WelcomeScreen onopen={handleTabOpen} />
{:else}
  {#key activeTab.id}
    <RepoView repoPath={activeTab.repoPath!} repoName={activeTab.repoName!} />
  {/key}
{/if}
```

### Rust Changes

#### 1. RunningOp: Per-Repo Remote Operations

```rust
// Before (v0.8)
pub struct RunningOp(pub Mutex<Option<u32>>);

// After (v0.9)
pub struct RunningOp(pub Mutex<HashMap<String, u32>>);
// Key: repo path, Value: PID
```

This allows concurrent fetch/push across different repos. The `cancel_remote_op` command already takes `path` -- just change it to look up by path key instead of reading a single Option.

#### 2. open_repo: Make Idempotent

Currently `open_repo` always walks the full commit graph. When switching tabs to an already-open repo, skip the walk if cache already has an entry:

```rust
// In open_repo, before walk_commits:
{
    let cache_lock = cache.0.lock().unwrap();
    if cache_lock.contains_key(&path) {
        // Already open -- just ensure watcher is running
        return Ok(());
    }
}
// ... proceed with walk_commits only if not cached
```

#### 3. No Other Rust Changes Needed

All commands already take `path` and look up state by key. The watcher already emits per-path events. The Mutex-guarded HashMaps already support concurrent repos.

### Store Persistence Changes

```typescript
// store.ts changes

// Before: single repo
const OPEN_REPO_KEY = 'open_repo';

// After: tab list
const OPEN_TABS_KEY = 'open_tabs';
const ACTIVE_TAB_KEY = 'active_tab_id';

export async function getOpenTabs(): Promise<TabInfo[]> { ... }
export async function setOpenTabs(tabs: TabInfo[]): Promise<void> { ... }
export async function getActiveTabId(): Promise<string | null> { ... }
export async function setActiveTabId(id: string | null): Promise<void> { ... }
```

On app launch: restore tabs from store, re-open each repo in Rust (sequential, not parallel, to avoid hammering IO), set active tab.

### Keyboard Shortcuts

| Shortcut | Action | Reference |
|----------|--------|-----------|
| Cmd+T | New tab (welcome screen) | GitKraken/Fork pattern |
| Cmd+W | Close active tab | Standard |
| Cmd+1..9 | Switch to tab N | GitKraken pattern |
| Cmd+Shift+[ / ] | Previous/Next tab | Standard macOS |

## Recommended Architecture: Tree View

### Problem

Currently, all file lists (StagingPanel unstaged/staged/conflicted, CommitDetail file diffs) render flat paths like `src/components/CommitGraph.svelte`. Users want to toggle between flat list and directory tree view.

### File List Locations (Components That Need Tree View)

| Component | File Source | Data Type |
|-----------|------------|-----------|
| StagingPanel | `get_status` | `FileStatus[]` (unstaged, staged, conflicted) |
| CommitDetail | `diff_commit` | `FileDiff[]` |
| MergeEditor | conflicted file list | Single file (no tree needed) |

### Tree Data Model

```typescript
// src/lib/file-tree.ts (new pure utility, no Svelte runes)

export interface TreeNode {
  name: string;           // "CommitGraph.svelte" or "components"
  path: string;           // full relative path: "src/components/CommitGraph.svelte"
  isDirectory: boolean;
  children: TreeNode[];   // sorted: directories first, then files, both alphabetical
  // For leaf nodes only:
  file?: FileStatus | FileDiff;  // original data for actions/clicks
}

export function buildTree<T extends { path: string }>(
  files: T[],
  getFile: (item: T) => T
): TreeNode[] {
  // 1. Split each path by '/'
  // 2. Build nested map: Map<string, TreeNode>
  // 3. Sort: dirs first, then alpha within each group
  // 4. Return root children array
}

export function flattenTree(nodes: TreeNode[]): TreeNode[] {
  // For keyboard navigation: DFS traversal respecting expanded state
}
```

### Conversion Algorithm

```
Input:  ["src/lib/types.ts", "src/lib/invoke.ts", "src/App.svelte", "README.md"]

Step 1: Split paths into segments
  ["src", "lib", "types.ts"]
  ["src", "lib", "invoke.ts"]
  ["src", "App.svelte"]
  ["README.md"]

Step 2: Insert into trie (Map-based)
  root/
    src/               (dir)
      lib/             (dir)
        types.ts       (file, leaf)
        invoke.ts      (file, leaf)
      App.svelte       (file, leaf)
    README.md          (file, leaf)

Step 3: Sort each level (dirs first, alpha)
  root/
    src/
      lib/
        invoke.ts
        types.ts
      App.svelte
    README.md
```

This is a pure frontend transformation. No Rust changes needed -- git2 already returns flat paths.

### Component Structure for Tree View

```
FileList (new wrapper component)
  |-- viewMode: 'flat' | 'tree' (persisted in LazyStore)
  |-- toggle button in header
  |
  |-- {#if viewMode === 'flat'}
  |     {#each files as f}
  |       <FileRow file={f} ... />   (existing component, unchanged)
  |
  |-- {:else}
  |     {#each treeNodes as node}
  |       <TreeRow {node} depth={0} ... />   (new component)
  |         |-- if directory: chevron + folder icon + name + file count badge
  |         |-- if file: indent + status icon + name + hover action
  |         |-- onclick directory: toggle expanded
  |         |-- onclick file: same as current FileRow click
```

### New Components

**FileList.svelte** -- wraps the flat/tree toggle logic:
- Accepts `files: FileStatus[] | FileDiff[]`, `viewMode`, callbacks
- Converts to tree when needed using `buildTree()`
- Renders either flat FileRow list or recursive TreeRow list

**TreeRow.svelte** -- renders a single tree node:
- Indentation: `padding-left: {depth * 16}px`
- Directory: ChevronDown/ChevronRight + Folder icon + name + `(N files)` count
- File: StatusIcon + name + hover action button
- Uses existing FileRow action pattern (stage/unstage/discard)
- Expanded state tracked per-directory in a `Set<string>` of expanded paths

### Where Tree View Integrates

**StagingPanel.svelte** -- Replace the three `{#each}` file lists:
```svelte
<!-- Before -->
{#each status?.unstaged ?? [] as f (f.path)}
  <FileRow file={f} ... />
{/each}

<!-- After -->
<FileList
  files={status?.unstaged ?? []}
  viewMode={fileViewMode}
  actionLabel="+"
  onaction={(path) => stageFile(path)}
  onclick={(path) => onfileselect?.(path, 'unstaged')}
/>
```

**CommitDetail.svelte** -- Replace the file list section:
```svelte
<!-- Before -->
{#each fileDiffs as fd (fd.path)}
  <button onclick={() => onfileselect(fd.path)}>...</button>
{/each}

<!-- After -->
<FileList
  files={fileDiffs}
  viewMode={fileViewMode}
  onaction={(path) => onfileselect(path)}
/>
```

### View Mode Persistence

```typescript
// store.ts addition
const FILE_VIEW_MODE_KEY = 'file_view_mode';

export async function getFileViewMode(): Promise<'flat' | 'tree'> {
  return (await store.get<'flat' | 'tree'>(FILE_VIEW_MODE_KEY)) ?? 'flat';
}
export async function setFileViewMode(mode: 'flat' | 'tree'): Promise<void> {
  await store.set(FILE_VIEW_MODE_KEY, mode);
  await store.save();
}
```

The view mode is global (not per-tab) -- when you toggle tree view, all file lists switch.

## Integration Points Summary

### New Files to Create

| File | Type | Purpose |
|------|------|---------|
| `src/lib/tab-state.svelte.ts` | Shared state | Tab list, active tab, mutations |
| `src/lib/file-tree.ts` | Pure utility | `buildTree()`, `flattenTree()` |
| `src/lib/file-tree.test.ts` | Tests | Tree building edge cases |
| `src/components/RepoView.svelte` | Component | Extracted repo workspace (from App.svelte) |
| `src/components/FileList.svelte` | Component | Flat/tree toggle wrapper |
| `src/components/TreeRow.svelte` | Component | Single tree node renderer |

### Existing Files to Modify

| File | Change | Scope |
|------|--------|-------|
| `src/App.svelte` | Extract repo logic to RepoView, add tab management shell | **Major refactor** |
| `src/components/TabBar.svelte` | Multi-tab rendering, + button, close per tab, drag-reorder | **Major rewrite** |
| `src/components/StagingPanel.svelte` | Replace FileRow loops with FileList | Moderate |
| `src/components/CommitDetail.svelte` | Replace file list with FileList | Moderate |
| `src/lib/store.ts` | Add tab persistence, file view mode | Small additions |
| `src/lib/types.ts` | Add TabInfo type | Small addition |
| `src-tauri/src/state.rs` | RunningOp -> HashMap | Small |
| `src-tauri/src/commands/repo.rs` | Make open_repo idempotent (skip if cached) | Small |
| `src-tauri/src/commands/remote.rs` | Per-repo PID lookup | Small |

### Files NOT Modified

- All other Rust command files (staging, history, branches, diff, etc.) -- already multi-repo via path key
- watcher.rs -- already per-repo
- All SVG/graph rendering code -- unaffected
- invoke.ts, types for graph/diff/branch -- unaffected

## Patterns to Follow

### Pattern 1: Shared $state Rune Module for Tabs

**What:** Use the established `$state` rune module pattern (like `remote-state.svelte.ts`) for tab state.
**Why:** Consistent with codebase conventions. Avoids prop drilling tab info through every component.
**When:** Any component needs to read tab list or active tab (TabBar, App.svelte).

### Pattern 2: Extract-Then-Compose Refactor

**What:** Extract the RepoView by literally moving App.svelte's repo-scoped script block into a new component. App.svelte becomes a thin shell.
**Why:** The current App.svelte has ~40 state variables and ~20 functions all scoped to a single repo. Moving them intact preserves all existing behavior.
**When:** Phase 1 of implementation -- do this before adding tab logic.

### Pattern 3: Pure Function Tree Builder

**What:** `buildTree()` is a pure function with no Svelte dependencies. Input: flat file array. Output: TreeNode array.
**Why:** Easily unit-testable. Can be used in StagingPanel, CommitDetail, and future components without duplication.
**When:** Tree view implementation phase.

### Pattern 4: `{#key}` for Tab Switching

**What:** Use Svelte's `{#key activeTab.id}` to destroy/recreate RepoView on tab switch.
**Why:** Simpler than keep-alive. Memory-efficient. Rust cache makes re-mount fast.
**When:** After RepoView extraction is complete.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Keep All Tabs Mounted with display:none

**What:** Render all RepoViews and toggle visibility.
**Why bad:** Each RepoView has effects listening to `repo-changed`, SVG overlays consuming memory, virtual list scroll state. With 10+ tabs, memory and event handler accumulation becomes problematic.
**Instead:** Use `{#key}` destroy/recreate. Accept the tradeoff of losing scroll position on tab switch.

### Anti-Pattern 2: Pass repoPath Through Every Component Prop

**What:** Thread `repoPath` from App -> RepoView -> every child.
**Why bad:** Already the pattern, but once tabs exist, components may need to know "am I the active tab?" for pausing updates. Prop drilling gets worse.
**Instead:** `repoPath` stays as a prop on RepoView (which gets it from tab state). Children receive it from RepoView props. Tab state module is only used by App.svelte and TabBar.

### Anti-Pattern 3: Building Tree in Rust

**What:** Add a Rust command to return a tree-structured file status.
**Why bad:** The flat list is the natural output of `git2::Repository::statuses()`. Converting to tree in Rust adds serialization complexity, new types, and gains nothing -- the JS transformation is O(n) and trivial.
**Instead:** Pure frontend transformation in `file-tree.ts`.

### Anti-Pattern 4: Global Event Bus for Tab Changes

**What:** Use Tauri events to communicate tab changes between components.
**Why bad:** Tab switching is purely a frontend concern. Adding Rust->frontend events for something that's just UI state adds unnecessary complexity.
**Instead:** Svelte 5 reactive state via `tab-state.svelte.ts`.

## Suggested Build Order

The build order is driven by dependencies -- each phase can be tested independently.

```
Phase 1: Extract RepoView (prerequisite for tabs)
  - Create RepoView.svelte from App.svelte repo code
  - App.svelte becomes thin shell
  - Zero behavior change -- pure refactor
  - Test: app works exactly as before

Phase 2: Tab State + TabBar (multi-tab infrastructure)
  - Create tab-state.svelte.ts
  - Rewrite TabBar.svelte for multi-tab
  - Wire App.svelte to use tab state
  - Add tab persistence in store.ts
  - Cmd+T/W/1-9 keyboard shortcuts
  - New tab shows WelcomeScreen
  - Test: open multiple repos in tabs, switch between them

Phase 3: RunningOp Per-Repo (unblock concurrent remote ops)
  - Change RunningOp to HashMap<String, u32>
  - Update remote commands and cancel_remote_op
  - Make open_repo idempotent (skip if cached)
  - Test: fetch in repo A while pushing in repo B

Phase 4: File Tree Utility (pure logic, no UI)
  - Create file-tree.ts with buildTree()
  - Create file-tree.test.ts with edge cases
  - Test: unit tests pass for nested dirs, single files, empty input

Phase 5: Tree View Components (UI integration)
  - Create FileList.svelte (flat/tree wrapper)
  - Create TreeRow.svelte (tree node renderer)
  - Integrate into StagingPanel (unstaged, staged, conflicted)
  - Integrate into CommitDetail (commit file diffs)
  - Add view mode toggle + persistence
  - Test: toggle between flat/tree in both panels
```

## Scalability Considerations

| Concern | At 3 tabs | At 10 tabs | At 30+ tabs |
|---------|-----------|------------|-------------|
| Memory (Rust) | ~50MB (3 cached GraphResults) | ~150MB | Consider LRU eviction for CommitCache |
| FS watchers | 3 notify watchers | 10 watchers | May need watcher pooling |
| Tab bar width | Fits easily | Tabs need to shrink/scroll | Horizontal scroll + overflow menu |
| Open repo startup | Sequential, ~1s total | ~3s sequential | Lazy-load: only open active tab's repo, open others on first switch |

## Sources

- Full codebase audit: state.rs, lib.rs, App.svelte, TabBar.svelte, WelcomeScreen.svelte, store.ts, all command files
- [Tauri 2 State Management](https://v2.tauri.app/develop/state-management/) -- confirms Mutex<HashMap> pattern for multi-key state
- [GitKraken Interface Documentation](https://help.gitkraken.com/gitkraken-desktop/interface/) -- tab UX patterns (Cmd+T, Cmd+1-9, drag-reorder)
- [GitKraken tab performance feedback](https://feedback.gitkraken.com/suggestions/196509/improve-performance-when-switching-tabs) -- validates destroy/recreate concern
- [Flat paths to tree algorithm](https://gist.github.com/Aracki/477fe5246e8c29b6440e49627e30eb0c) -- trie-based approach reference
