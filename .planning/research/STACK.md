# Stack Research: Trunk v0.9

**Domain:** Multi-tab repository management + directory tree file views for Tauri 2 + Svelte 5 Git GUI
**Researched:** 2026-03-23
**Confidence:** HIGH

## Existing Stack (DO NOT change)

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Framework | Tauri 2 | 2.x | Desktop shell, IPC via `invoke`/`listen` |
| Frontend | Svelte 5 | 5.x | Vite SPA, runes (`$state`, `$derived`, `$effect`) |
| Styling | Tailwind CSS v4 | 4.x | Forced dark theme via CSS custom properties |
| Git backend | `git2` (libgit2) | 0.19 | `vendored-libgit2` feature, all local ops |
| Git CLI | subprocess | -- | Remote ops (fetch/pull/push), cherry-pick/revert |
| Icons | `@lucide/svelte` | ^0.577 | SVG icon components |
| FS watching | `notify` + `notify-debouncer-mini` | 7 / 0.5 | 300ms debounce |
| State persistence | `tauri-plugin-store` | 2.4.2 | LazyStore for UI prefs |
| Async runtime | `tokio` | 1 | `process`, `io-util` features |
| Virtual scroll | `@humanspeak/svelte-virtual-list` | ^0.4.2 | Commit graph virtualization |
| Drag & drop | `sortablejs` | ^1.15.7 | Interactive rebase reordering |
| Clipboard | `tauri-plugin-clipboard-manager` | 2.x | Copy SHA/path |
| Window state | `tauri-plugin-window-state` | 2.x | Restore window size/position |

## New Dependencies

| Dependency | Version | Purpose | Why Needed |
|-----------|---------|---------|------------|
| **None** | -- | -- | Both features are implementable with the existing stack. No new crates or npm packages required. |

## Core Finding: Zero New Dependencies

Both v0.9 features (multi-tab and tree view) require **zero new dependencies**. The reasoning:

1. **Multi-tab**: Tabs are HTML/JS-only UI within a single Tauri window. The backend already supports multiple open repos via `RepoState(HashMap<String, PathBuf>)`, `CommitCache(HashMap<String, GraphResult>)`, and `WatcherState(HashMap<String, Debouncer>)`. The `repo-changed` event already carries the repo path as payload, and all existing listeners already filter by `event.payload === repoPath`. The infrastructure is already multi-repo capable -- only the frontend assumes one active repo.

2. **Tree view**: A directory tree is a ~100 LOC recursive Svelte component. File lists in this app are small (typically 1-100 files in staging, rarely 1000+). No need for a virtual-scroll tree or third-party tree library. Svelte 5 supports recursive self-import natively -- no `<svelte:self>` needed.

---

## Stack Decisions

### 1. Multi-Tab: Single Window, HTML/JS Tabs (NOT Tauri multi-window)

**Decision:** Implement tabs as pure frontend Svelte components within a single Tauri window.

**Why NOT Tauri multi-window or multi-webview:**
- Tauri 2's multi-webview support is experimental and has known bugs (e.g., closing a webview can panic, issue #8888).
- Each Tauri webview is a separate process/context with no shared JavaScript state -- every piece of UI state would need to pass through Rust IPC.
- The UX goal is browser-like tabs (drag to reorder, Cmd+T to open, Cmd+W to close), not separate OS windows.
- Every professional Git GUI (GitKraken, Fork, SourceTree) uses single-window HTML/JS tabs, not native multi-window.
- Spacedrive (a production Tauri app) implements tabs the same way -- pure frontend.

**Architecture:**

```
App.svelte (tab container)
  |-- TabBar.svelte (tab strip with + button)
  |-- {#each tabs as tab}
  |     RepoView.svelte (all current App.svelte repo content, extracted)
  |   OR
  |     WelcomeScreen.svelte (new-tab page / splash)
  |-- {/each} (only active tab rendered, or all with display:none)
```

**Key pattern -- extract `RepoView`:** The current `App.svelte` contains ~500 lines of repo-specific state (repoPath, refreshSignal, dirtyCounts, selectedFile, commitDetail, etc.). This gets extracted into a new `RepoView.svelte` component. `App.svelte` becomes a thin tab orchestrator that mounts one `RepoView` per tab.

**State isolation strategy:**
- Each tab is a `RepoView` component instance with its own `$state` variables. Svelte component instances naturally isolate state -- no extra work needed.
- The Rust backend already isolates per-repo: `RepoState`, `CommitCache`, and `WatcherState` are all `HashMap<String, ...>` keyed by repo path.
- The `repo-changed` event already carries the path as payload. Each `RepoView` instance already filters `event.payload === repoPath`. No backend changes needed.
- Shared global state (zoom level, pane widths) stays in `store.ts` and applies to all tabs.

**Tab state to persist (via LazyStore):**
- `open_tabs: Array<{ path: string; name: string }>` -- ordered list of open tabs
- `active_tab_index: number` -- which tab is focused
- Replaces the current `open_repo: { path, name }` single-repo persistence

**Rendering strategy -- mount active tab only (with keepalive via state):**
- Only mount the active tab's `RepoView` in the DOM. Unmounted tabs lose their component state.
- For cheap "tab restore": When switching back to a tab, `RepoView` mounts fresh and re-fetches from the Rust cache (which is already populated and warm). The `CommitCache` holds the full graph per repo path, so re-mount is just an IPC call + render -- fast enough (<100ms).
- Alternative: keep all tabs mounted with `display: none` on inactive ones. This preserves scroll position and selection state but increases memory. Start with unmount + re-fetch; optimize to keep-alive later if UX demands it.

**Confidence:** HIGH -- the backend is already multi-repo. Tabs are pure UI. This is the same approach as Spacedrive, GitKraken, and Fork.

---

### 2. Tree View: Custom Recursive Svelte Component (NOT a library)

**Decision:** Build a custom `FileTree.svelte` + `TreeNode.svelte` recursive component. No third-party tree library.

**Why NOT `@keenmate/svelte-treeview`:**
- It pulls in FlexSearch as a dependency (we do not need text search in file trees).
- It has 743 weekly npm downloads -- low adoption, risk of abandonment.
- It ships its own SCSS styles that would conflict with our Tailwind + CSS custom properties theme system.
- Its feature set (drag & drop, search, editing, virtual scroll for 50k+ nodes) is massively overengineered for our use case of rendering 1-500 file paths as a tree.
- The `nodeTemplate` snippet customization would still require fighting the library's CSS and structure.

**Why custom is better here:**
- A recursive tree view in Svelte 5 is ~80-120 lines of code total.
- Our file trees are small: staging panel shows working tree status (rarely >100 files), commit diffs show changed files (rarely >500), merge editor shows conflicted files (rarely >20).
- We already have `FileRow.svelte` with status icons, hover actions, and context menus. A tree view just wraps it with expand/collapse at directory nodes.
- Full control over CSS custom properties, hover states, action buttons, and integration with existing staging/diff workflows.
- No risk of breaking changes from third-party library updates.

**Data structure -- flat path to tree conversion:**

The backend already returns `FileStatus[]` with flat paths (e.g., `"src/lib/store.ts"`). Transform on the frontend:

```typescript
interface TreeNode {
  name: string;          // "store.ts" or "lib"
  path: string;          // "src/lib/store.ts" (full relative path)
  isDir: boolean;
  children: TreeNode[];  // sorted: dirs first, then files, both alphabetical
  file?: FileStatus;     // present only for leaf files
}

function buildTree(files: FileStatus[]): TreeNode[] {
  const root: TreeNode = { name: '', path: '', isDir: true, children: [] };
  for (const file of files) {
    const parts = file.path.split('/');
    let current = root;
    for (let i = 0; i < parts.length; i++) {
      const isLast = i === parts.length - 1;
      const partPath = parts.slice(0, i + 1).join('/');
      let child = current.children.find(c => c.name === parts[i]);
      if (!child) {
        child = {
          name: parts[i],
          path: partPath,
          isDir: !isLast,
          children: [],
          file: isLast ? file : undefined,
        };
        current.children.push(child);
      }
      current = child;
    }
  }
  sortTree(root);
  return root.children;
}
```

**Component structure:**

```svelte
<!-- FileTree.svelte -->
<script lang="ts">
  import FileTreeNode from './FileTreeNode.svelte';
  // props: files, onaction, onclick, etc.
  // $derived: tree = buildTree(files)
</script>
{#each tree as node}
  <FileTreeNode {node} depth={0} ... />
{/each}

<!-- FileTreeNode.svelte (recursive) -->
<script lang="ts">
  import FileTreeNode from './FileTreeNode.svelte'; // self-import for recursion
  // if node.isDir: render expand/collapse toggle + directory name
  // if node.file: render existing FileRow component
</script>
{#if node.isDir}
  <div class="dir-row" onclick={toggle}>
    <ChevronRight/ChevronDown /> {node.name}
  </div>
  {#if expanded}
    {#each node.children as child}
      <FileTreeNode node={child} depth={depth + 1} ... />
    {/each}
  {/if}
{:else}
  <FileRow file={node.file} ... />
{/if}
```

**Expand/collapse state:** Store expanded directory paths in a `Set<string>` as `$state`. Default: expand all directories (file counts are small). Persist toggle preference (flat vs tree) in LazyStore, but not individual expansion state (it changes too often with working tree edits).

**Toggle between flat and tree view:** A toggle button in the staging panel header and commit detail header. Store the preference per-component-type (staging vs commit detail) in LazyStore. Default to flat (preserving current behavior) so the upgrade is non-breaking.

**Performance:** The `buildTree` transform is O(n * d) where n = file count, d = average path depth. For 500 files at depth 5, that is 2500 string comparisons -- sub-millisecond. No virtual scrolling needed for tree view; the existing scroll container handles it.

**Confidence:** HIGH -- recursive Svelte 5 components are well-documented, the data transform is trivial, and file counts in our use cases are small.

---

### 3. Tab Bar: Custom Component (NOT sortablejs for tab drag)

**Decision:** Build a custom `TabBar.svelte` with basic Cmd+T/Cmd+W keyboard shortcuts. Defer tab drag-to-reorder to a later milestone.

**Why:** Tab reordering is a nice-to-have, not a table-stakes feature for v0.9. The existing `sortablejs` dependency could be reused for tab drag later, but adding drag behavior to tabs introduces edge cases (drag a tab out to create a new window, drag between windows) that are out of scope.

**Tab bar features for v0.9:**
- Horizontal tab strip showing repo name per tab
- Active tab indicator (accent color underline)
- Close button (X) on each tab
- "+" button to open a new tab (shows WelcomeScreen)
- Middle-click to close a tab (standard browser behavior)
- Cmd+T to open a new tab, Cmd+W to close current tab
- Cmd+1-9 to switch to tab by index
- Scroll overflow when many tabs open (horizontal scroll or min-width shrink)

**Confidence:** HIGH -- purely UI work with established patterns.

---

## Integration Notes

### Backend Changes Required

**Minimal.** The Rust backend is already multi-repo capable:

1. **`open_repo`**: Already inserts into `RepoState(HashMap)`, `CommitCache(HashMap)`, and starts a watcher per path. Opening a second repo while another is open already works.

2. **`close_repo`**: Already removes a single path from all state maps and stops its watcher. Does not affect other open repos.

3. **`repo-changed` event**: Already carries path as payload. Multiple watchers already coexist.

4. **All commands accept `path: String`**: Every Tauri command already takes the repo path as an argument and opens a fresh `Repository::open(path)` per call. No command assumes a single global repo.

**One concern: memory.** Each open tab holds a full `GraphResult` in `CommitCache`. For a repo with 50k commits, that is ~20-30MB. With 10 tabs open, that is 200-300MB. This is acceptable for a desktop app, but worth monitoring. If needed, an LRU eviction on `CommitCache` can be added later (evict oldest unused repo when cache exceeds a threshold).

### Frontend Changes Required

1. **Extract `RepoView.svelte`** from current `App.svelte` repo content (~lines 30-500 of state + template).
2. **Refactor `App.svelte`** into tab orchestrator: `tabs: Tab[]` state, `activeTabIndex`, tab lifecycle methods.
3. **Evolve `TabBar.svelte`** from current single-tab display to multi-tab strip.
4. **Evolve `store.ts`**: Replace `open_repo` with `open_tabs` + `active_tab_index`.
5. **Build `FileTree.svelte` + `FileTreeNode.svelte`** (~120 LOC total).
6. **Build `buildTree()` utility** in a new `src/lib/tree.ts` (~50 LOC).
7. **Add tree/flat toggle** to `StagingPanel.svelte`, `CommitDetail.svelte`, and `MergeEditor.svelte` file list sections.
8. **Persist view preference** (flat vs tree) in LazyStore.

### Keyboard Shortcuts

| Shortcut | Action | Scope |
|----------|--------|-------|
| Cmd+T | New tab (WelcomeScreen) | Global |
| Cmd+W | Close current tab | Global |
| Cmd+1-9 | Switch to tab N | Global |
| Cmd+Shift+] | Next tab | Global |
| Cmd+Shift+[ | Previous tab | Global |

These must be registered at the `App.svelte` level, before the existing Cmd+F (search) handler which lives in `CommitGraph.svelte`.

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| Tauri multi-window / multi-webview | Experimental, no shared JS state, separate process per webview, known panics on close | HTML/JS tabs in single window |
| `@keenmate/svelte-treeview` | Pulls FlexSearch, overengineered for <500 file trees, own SCSS breaks our theme, 743 weekly downloads | Custom ~120 LOC recursive component |
| `svelte-file-tree-explorer` | Svelte 4 only, last updated 2021, dead project | Custom component |
| `@svar/svelte-filemanager` | Full file manager UI with its own layout, far too heavy for a file list sub-component | Custom component |
| Skeleton / shadcn-svelte tree views | Would require adopting those UI frameworks; conflicts with existing Tailwind + CSS custom properties | Custom component |
| `directory-tree` npm package | Reads from filesystem via Node.js -- we are in a browser context, files come from git2 backend | Frontend `buildTree()` utility |
| Svelte router (svelte-spa-router, etc.) | Tabs are not URL routes. No navigation/history needed. Component show/hide is sufficient | Conditional rendering `{#if activeTab === i}` |
| State management library (zustand-svelte, etc.) | Svelte 5 runes (`$state`, `$derived`) are the state management. Component instance isolation gives us per-tab state for free | `$state` runes in each `RepoView` instance |
| Tab drag-to-reorder | Nice-to-have, not table stakes for v0.9. Existing `sortablejs` can be reused later | Defer to future milestone |

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Single window HTML tabs | Tauri multi-window | If each repo needs true OS-level window management (e.g., dual-monitor drag-out). Revisit when Tauri multi-webview stabilizes. |
| Custom tree component | `@keenmate/svelte-treeview` | If file trees grow to 10k+ items and need virtual scrolling. Unlikely in a Git GUI staging panel. |
| Unmount inactive tabs | Keep all tabs mounted (display:none) | If users report slow tab switching or loss of scroll position becomes a UX pain point. Easy to switch later. |
| Flat-first default | Tree-first default | After user testing. Tree may become preferred once available; let usage data decide. |

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| Svelte 5 self-import recursion | Svelte 5.x | Replaces deprecated `<svelte:self>`. Import component from itself. |
| `tauri-plugin-store` 2.4.2 | Tauri 2.x | Already used. `open_tabs` array replaces `open_repo` single object. |
| `@humanspeak/svelte-virtual-list` ^0.4.2 | Svelte 5.x | Not needed for tree view (files are few). Already used for commit graph only. |

---

## Sources

- [Tauri 2 State Management docs](https://v2.tauri.app/develop/state-management/) -- State wraps in Arc automatically, Mutex for interior mutability (HIGH confidence)
- [Tauri multi-webview discussion #2975](https://github.com/tauri-apps/tauri/issues/2975) -- Multiple webviews now implemented but experimental (HIGH confidence)
- [Tauri splittable tab discussion #6464](https://github.com/tauri-apps/tauri/discussions/6464) -- Spacedrive example, FabianLars confirms tabs should be HTML/JS (HIGH confidence)
- [Tauri multi-window discussion #9423](https://github.com/tauri-apps/tauri/discussions/9423) -- No shared JS context across windows, state must go through Rust (HIGH confidence)
- [Recursive Svelte 5 components](https://scriptraccoon.dev/blog/recursive-svelte-components) -- Self-import pattern replaces svelte:self (HIGH confidence)
- [@keenmate/svelte-treeview GitHub](https://github.com/KeenMate/svelte-treeview) -- Svelte 5 compatible but overengineered, FlexSearch dependency (MEDIUM confidence)
- [GitKraken/Fork/SourceTree comparison](https://www.kaels-kabbage.com/posts/gitkraken-vs-fork-facts-vs-feelings/) -- All use single-window tab-based UI (HIGH confidence)
- Existing codebase: `state.rs` (RepoState/CommitCache/WatcherState HashMaps), `watcher.rs` (per-path watchers), `repo.rs` (open_repo/close_repo), `App.svelte` (repo-changed listener filters by path), `store.ts` (LazyStore patterns) (HIGH confidence)

---
*Stack research for: Trunk v0.9 Multi-tab & Tree View*
*Researched: 2026-03-23*
