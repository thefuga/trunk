# Feature Landscape: Multi-tab Repository Management & Tree View File Lists

**Domain:** Desktop Git GUI multi-repo tabs and directory tree file display
**Researched:** 2026-03-23
**Target milestone:** v0.9

## Table Stakes

Features users expect from a Git GUI that supports multiple repositories and file tree views. Missing any of these makes the milestone feel incomplete.

### Multi-tab Repository Management

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Tab bar with repo name per tab | Every tabbed Git GUI (GitKraken, Fork, Sourcetree on Windows) shows repo name in tab. Users need to identify which repo they're looking at. | Low | Current `TabBar.svelte` shows a single tab with close button. Extend to render N tabs. Each tab shows `repoName` text + close (X) button. Active tab visually distinct (brighter text, bottom border or background). |
| New tab button (+) | GitKraken, Fork, and browser convention. The + button at the end of the tab bar opens a new tab showing the WelcomeScreen (splash/project picker). | Low | Plus icon button after the last tab. Clicking it pushes a new tab entry with `repoPath: null` state, which renders the existing `WelcomeScreen` component. |
| Close tab (X button) | Standard across all tabbed interfaces. Each tab needs a close button that tears down that repo's state. | Low | Already exists as single close button in `TabBar.svelte`. Extend to per-tab X buttons. Closing calls `close_repo` for that tab's path, removes tab from array. If last tab closes, show WelcomeScreen in a new empty tab. |
| Cmd+T / Ctrl+T for new tab | GitKraken uses this exact shortcut. Browser-standard convention users already know. | Low | Add to existing `handleKeydown` in `App.svelte`. Must not conflict with existing Cmd+J/K (pane toggles). |
| Cmd+W / Ctrl+W for close tab | GitKraken uses this exact shortcut. Universal tab close convention. | Low | Close active tab. If it's the last tab and has no repo open, close the window instead (or keep one empty tab). |
| Cmd+1-9 / Ctrl+1-9 for tab switching | GitKraken supports this. Browser convention. Essential for keyboard-driven workflows. | Low | Map Cmd+1 to first tab, Cmd+2 to second, etc. Cmd+9 always selects the last tab (browser convention). |
| Ctrl+Tab / Ctrl+Shift+Tab for next/prev tab | GitKraken and Fork both support this. Standard OS-level tab navigation. | Low | Cycle through tabs in order. Wrap around at ends. |
| Independent state per tab | Each tab must have its own repo path, commit graph, staging panel state, diff state, selection state. No cross-tab pollution. | High | This is the core architectural challenge. Currently `App.svelte` holds all repo state as top-level `$state` variables. Must refactor to per-tab state objects or a tab-indexed state map. See Architecture research for approaches. |
| Tab order persistence across restart | Users expect their tabs to be where they left them when reopening the app. | Med | Persist tab array (order + repo paths) to LazyStore. On startup, restore tabs. Replace current `getOpenRepo`/`setOpenRepo` (single repo) with `getOpenTabs`/`setOpenTabs` (array). |
| Dirty indicator on tab | Fork shows a star/dot when a repo has uncommitted changes. Prevents users from forgetting unsaved work in a background tab. | Low | Show a small dot/circle indicator on tabs where `dirtyCounts.staged + dirtyCounts.unstaged + dirtyCounts.conflicted > 0`. Color: use accent or a subtle indicator. Poll dirty counts for background tabs on `repo-changed` events. |
| WelcomeScreen as new-tab page | Opening a new tab should show the same project picker / recent repos screen that appears on first launch. Consistent mental model. | Low | Already implemented as `WelcomeScreen.svelte`. When a tab has `repoPath === null`, render WelcomeScreen. When user picks a repo from WelcomeScreen, that tab transitions to the repo view. |
| Middle-click tab to close | GitKraken and browser convention. Power users expect this. | Low | `onmousedown` handler on tab element, check `event.button === 1` (middle click), call close. |

### Tree View File Lists

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Toggle between flat list and tree view | GitKraken, Fork, VS Code, Lazygit, SmartGit all offer this toggle. It is the universal pattern. Users working in deeply nested repos need tree view; users in flat repos prefer the list. | Med | A toggle button (list/tree icon) in the header of each file list section (unstaged, staged, conflicted, commit detail files). Persisted to LazyStore so the preference survives restart. |
| Directory nodes with expand/collapse | Standard tree interaction. Click chevron or directory name to expand/collapse. Chevron rotates (right = collapsed, down = expanded). | Med | Tree node: `{ name: string, path: string, children: TreeNode[], files: FileStatus[] }`. Expand/collapse state tracked per directory path. Indentation via `padding-left: depth * 16px` or similar. |
| File status icon + name at leaf level | Existing `FileRow.svelte` already shows status icon + filename. In tree mode, the filename portion shows just the basename (not full path). | Low | Reuse existing `FileRow` component but pass `file.path.split('/').pop()` as display name in tree mode. Full path continues to be used for actions (stage, unstage, diff). |
| Directory path segments with slash separators | When a directory has only one child directory, compress into single line: `src/lib/` instead of separate `src` > `lib` nodes. Lazygit and VS Code both do this. | Med | Path compression: if a directory node has exactly one child and that child is a directory (not a file), merge them into a single display node with combined path (`parent/child`). Reduces visual noise significantly. |
| Stage/unstage entire directory | GitKraken allows staging entire folders in tree view. Lazygit stages directories with space key. Users expect to batch-stage related files. | Med | Hover action button on directory nodes (same +/- pattern as `FileRow`). Calls stage/unstage for all files recursively under that directory. Must handle mixed states (some files already staged). |
| Directory status color aggregation | Lazygit convention: red = all unstaged, green = all staged, yellow = mixed. Helps users scan tree for outstanding work. | Low | Only relevant in contexts showing mixed staged/unstaged (not typical in Trunk since unstaged and staged are separate sections). For commit diffs, show the dominant diff status color. For staging panel, each section is homogeneous so directories inherit the section's color. |
| Consistent tree view across all file lists | Tree view should work in: (1) unstaged files, (2) staged files, (3) conflicted files, (4) commit detail file list, (5) merge editor file picker if any. | Med | Build a shared `FileTree.svelte` component that accepts `FileStatus[]` or `FileDiff[]` and renders either flat or tree mode. Use in StagingPanel, CommitDetail, and any future file list. |
| Keyboard navigation in tree view | Arrow keys to move between items, Left/Right to collapse/expand directories, Enter to select file. Standard tree keyboard nav (WAI-ARIA treeview pattern). | Med | Focus management with `aria-role="treeitem"` and `aria-expanded`. Up/Down moves between visible items (skip collapsed children). Left collapses current directory or moves to parent. Right expands or moves into first child. |

## Differentiators

Features that set Trunk apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Drag tab to reorder | Fork supports this. Gives users control over tab organization. Not blocking but polished. | Med | HTML5 drag-and-drop on tab elements. Update tab array order. Persist new order. Visual feedback: drag ghost, insertion line. |
| Drag tab to new window | Fork 1.0.69 added this. Power user feature for multi-monitor setups. | Very High | Requires Tauri multi-window support, state serialization, cross-window IPC. Defer to a future milestone. |
| Tab context menu | Right-click on tab for: Close, Close Others, Close All, Close to the Right, Copy Path. Standard in editors/browsers. | Low | Use Tauri native `Menu` API (already established pattern in codebase). Straightforward to implement. |
| Pin tab | Prevent accidental closure of frequently-used repos. Common in browsers and IDEs. | Low | Pinned tabs render as icon-only (smaller), cannot be closed without unpin. Persist pin state. |
| File count badge on directory nodes | Show "(5 files)" or a small count next to directory names in tree view. Helps users gauge change density without expanding. | Low | Count descendant files recursively. Display as muted text after directory name. |
| Expand All / Collapse All buttons | Fork users have requested this (issue #2072). Useful for large changesets to quickly survey or focus. | Low | Two small buttons in the file list header (next to the tree/list toggle). "Expand All" sets all directories to expanded. "Collapse All" sets all to collapsed. |
| Remember expand/collapse state | Zed feature request mentions this. Tree state persists within a session (not across restarts). Prevents annoying re-expansion after staging a file. | Med | Store expanded paths in a `Set<string>` per file list section. Preserve across re-renders triggered by `repo-changed` events. Not persisted to disk (too volatile). |
| Discard all files in directory | Right-click directory in unstaged tree view > "Discard All in Directory". Batch discard for focused cleanup. | Med | Iterate all files under directory, call existing discard logic for each. Show single confirmation dialog listing all files. |

## Anti-Features

Features to explicitly NOT build for v0.9.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Multi-window support | Very high complexity: Tauri multi-window requires separate webview instances, state synchronization, IPC between windows. Massive scope for v0.9. | Keep single-window with tabs. Revisit multi-window in a future milestone after tabs are solid. |
| Workspace grouping (GitKraken-style) | GitKraken Workspaces are a team/cloud feature for organizing repos by project. Overkill for a personal-use desktop client. | Tabs + recent repos is sufficient. If grouping is ever needed, a simple folder/tag system on the WelcomeScreen would be lighter. |
| Tab search / fuzzy finder | Tower has Ctrl+O quick-open dialog for repos. Useful at scale (50+ repos) but premature for v0.9. | Cmd+1-9 and Ctrl+Tab cover navigation. WelcomeScreen's recent repos list handles discovery. |
| Inline rename in tree view | Double-click to rename files directly in the tree. Git GUIs don't do this -- it's a file manager feature, not a VCS feature. | File operations happen through the terminal or OS file manager. |
| File filtering / search within tree | Text input to filter visible files in tree view. Useful but a separate feature that adds scope. | Defer to a later milestone. Flat list + tree view covers the core need. |
| Virtual scrolling for file lists | File lists rarely exceed a few hundred items. The commit graph needed virtualization because repos have 10k+ commits. File lists don't have this problem. | Simple DOM rendering is fine. If a repo has 1000+ changed files, the user has bigger problems. Revisit only if performance is observed to be an issue. |
| Automatic tab for submodules | Some Git GUIs auto-open submodules as child tabs. Complex to implement and niche use case. | Users can manually open submodule paths as separate tabs. |

## Feature Dependencies

```
Multi-tab: Independent state per tab --> All other tab features
  (Everything depends on the state isolation refactor)

Tab bar rendering --> New tab button, Close tab, Tab indicators
  (Visual tab bar must exist before its features work)

WelcomeScreen as new-tab page --> New tab button
  (New tab creates empty tab that renders WelcomeScreen)

Tree view toggle --> Tree data structure
  (Toggle button controls which rendering mode is active)

Tree data structure --> Directory expand/collapse
  (Must build path -> tree transformation before expand/collapse works)

Tree data structure --> Directory staging
  (Must know which files are under a directory to batch-stage them)

Directory expand/collapse --> Keyboard navigation in tree
  (Arrow key nav depends on knowing which nodes are visible)

FileTree shared component --> Consistent tree view across all file lists
  (Build once, use in StagingPanel, CommitDetail, etc.)
```

## MVP Recommendation

### Phase 1: Multi-tab Infrastructure (do first)

Prioritize:
1. **Independent state per tab** -- the architectural foundation everything else builds on
2. **Tab bar rendering with active/inactive states** -- visual container for tabs
3. **New tab (+) / close tab (X) / WelcomeScreen as new-tab page** -- core tab lifecycle
4. **Cmd+T, Cmd+W, Cmd+1-9, Ctrl+Tab shortcuts** -- keyboard-driven tab management
5. **Tab order persistence** -- survive restart
6. **Dirty indicator on tab** -- essential feedback for background repos

Rationale: Tab infrastructure requires the biggest refactor (state isolation in App.svelte). Get this working and stable before adding tree view, which is a contained UI addition.

### Phase 2: Tree View (do second)

Prioritize:
1. **Shared FileTree component with flat/tree toggle** -- the rendering foundation
2. **Path-to-tree transformation with path compression** -- data layer
3. **Directory expand/collapse with chevrons** -- core interaction
4. **Integrate into StagingPanel (unstaged, staged, conflicted sections)** -- primary use case
5. **Integrate into CommitDetail file list** -- secondary use case
6. **Stage/unstage entire directory** -- batch operations
7. **Keyboard navigation** -- accessibility and power users

Defer to post-MVP: Drag to reorder tabs, tab context menu, pin tabs, expand/collapse all, directory discard.

### Phase 3: Polish (do last)

1. **Tab context menu** (Close Others, Close All, Copy Path)
2. **Middle-click to close tab**
3. **Expand All / Collapse All buttons**
4. **File count badges on directories**
5. **Remember expand/collapse state within session**

## Complexity Assessment

| Feature Area | Estimated Complexity | Risk | Notes |
|-------------|---------------------|------|-------|
| Tab state isolation refactor | **High** | **High** | Currently App.svelte has ~30 `$state` variables for a single repo. Must refactor to per-tab state objects. Risk: subtle bugs from state leaking between tabs, event listener cleanup on tab switch. |
| Tab bar UI | **Low** | Low | Straightforward Svelte component. Existing TabBar.svelte provides starting point. |
| Tab keyboard shortcuts | **Low** | Low | Add to existing keydown handler in App.svelte. Well-defined behavior. |
| Tab persistence | **Med** | Low | Replace `getOpenRepo`/`setOpenRepo` with tab array. Migration from old format needed. |
| Path-to-tree transformation | **Med** | Low | Pure function, easily unit testable. Split paths by '/', build tree nodes, apply path compression. |
| FileTree component | **Med** | Med | Must handle expand/collapse state, indentation, action buttons per node type (file vs directory), and both `FileStatus` and `FileDiff` item types. |
| Directory staging | **Med** | Med | Must iterate all files under directory, call stage/unstage for each. Need to handle partial failures gracefully. |
| Keyboard navigation | **Med** | Med | WAI-ARIA treeview pattern has specific requirements. Focus management across nested nodes is fiddly. |
| Fs watcher per tab | **Med** | Med | Current watcher state assumes single repo. Must support multiple watchers, one per open tab. Cleanup on tab close. |

## Sources

- [GitKraken Keyboard Shortcuts](https://help.gitkraken.com/gitkraken-desktop/keyboard-shortcuts/) -- Tab shortcuts (Cmd+T, Cmd+W, Cmd+1-9, Ctrl+Tab), staging shortcuts, panel toggles (HIGH confidence)
- [GitKraken Staging](https://help.gitkraken.com/gitkraken-desktop/staging/) -- Tree view toggle for staging, folder-level staging, file list sections (HIGH confidence)
- [GitKraken Workspaces](https://www.gitkraken.com/features/workspaces) -- Multi-repo workspace concept (MEDIUM confidence)
- [Fork Tab Indicator for Uncommitted Changes](https://github.com/fork-dev/Tracker/issues/515) -- Star/dot badge on tabs with dirty state (HIGH confidence)
- [Fork 1.0.69 Multi-window Support](https://fork.dev/blog/posts/fork-1.0.69/) -- Tab drag to new window feature (MEDIUM confidence)
- [Fork Expand/Collapse All Request](https://github.com/fork-dev/TrackerWin/issues/2072) -- Community request for expand/collapse all in tree views (MEDIUM confidence)
- [GitHub Desktop Tab Request](https://github.com/desktop/desktop/issues/20026) -- Community requesting tabs instead of dropdown repo switcher (MEDIUM confidence)
- [Sourcetree Tab Navigation](https://support.atlassian.com/sourcetree/kb/viewing-and-maneuvering-around-repository-tabs-windows/) -- Tab scrolling and management on Windows (MEDIUM confidence)
- [Lazygit File Tree View PR #1197](https://github.com/jesseduffield/lazygit/pull/1197) -- Tree implementation: path compression, status color aggregation (red/green/yellow), directory staging, backtick toggle (HIGH confidence)
- [Zed Tree View Feature Request](https://github.com/zed-industries/zed/discussions/40052) -- Toggle button, directory actions, state persistence, file count (MEDIUM confidence)
- [VS Code Tree View Toggle Regression](https://github.com/microsoft/vscode/issues/295575) -- Users upset when toggle was removed; reinforces it as table stakes (MEDIUM confidence)
- [PatternFly Tree View Design Guidelines](https://www.patternfly.org/components/tree-view/design-guidelines/) -- Caret expand/collapse, click separation from selection, icon guidance (HIGH confidence)
- [Carbon Design System Tree View](https://carbondesignsystem.com/components/tree-view/usage/) -- Standard tree interaction patterns, accessibility requirements (HIGH confidence)
