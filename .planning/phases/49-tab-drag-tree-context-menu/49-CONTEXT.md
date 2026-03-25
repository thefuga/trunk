# Phase 49: Tab Drag Reorder & Tree Context Menu - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Drag-and-drop tab reordering with persisted order, and right-click context menu on directory nodes in the tree view for bulk stage/unstage/discard/resolve operations. Covers requirements TAB-11 (tab drag reorder with persistence) and TREE-11 (directory context menu with bulk actions).

</domain>

<decisions>
## Implementation Decisions

### Tab drag reorder (TAB-11)
- **D-01:** Use SortableJS for tab drag-and-drop (already a project dependency, used in RebaseEditor). Tabs animate/swap in place as you drag.
- **D-02:** Auto-scroll when dragging near the left/right edge of the tab bar, for overflow scenarios with many open tabs.
- **D-03:** The + (new tab) button is excluded from SortableJS — always pinned at the far right of the tab bar.
- **D-04:** After reorder, call existing `persistTabs()` (which calls `setOpenTabs()`) to save the new tab order. Order persists across app relaunch.

### Tree directory context menu (TREE-11)
- **D-05:** Context menu appears on directory nodes only. Individual files keep their existing per-file context menus from StagingPanel.
- **D-06:** Uses native Tauri menus (`@tauri-apps/api/menu`), consistent with all other context menus in the app (graph rows, branches, tabs, file rows).
- **D-07:** Unstaged section menu: "Stage All" + "Discard All". Stage All stages all files in the directory recursively. Discard All discards all changes in the directory.
- **D-08:** "Discard All" always shows a confirmation dialog before executing. Destructive operation — consistent with existing single-file discard confirmation.
- **D-09:** Staged section menu: "Unstage All" only. Unstages all files in the directory recursively.
- **D-10:** Conflicted section menu: "Resolve All" + "Unresolve All". Resolve All marks all conflicted files in the directory as resolved (stages them). Unresolve All marks resolved files back as conflicted.

### Claude's Discretion
- Drag styling (opacity, cursor, drop indicator) — use existing CSS custom properties, keep it subtle and consistent with the app aesthetic
- SortableJS configuration details (animation duration, ghost class, handle vs full-tab drag)
- How to wire the `oncontextmenu` handler through TreeFileList to DirectoryRow (callback prop pattern)
- Whether directory bulk operations use sequential or parallel IPC calls (Promise.all vs sequential)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Tab system
- `src/components/TabBar.svelte` — Current tab bar (149 lines); needs SortableJS integration for drag reorder
- `src/App.svelte` — Tab CRUD, `persistTabs()` function (line ~193), `showTabContextMenu` (line ~83)
- `src/lib/tab-types.ts` — TabInfo type definition
- `src/lib/store.ts` — `setOpenTabs()` / `getOpenTabs()` for tab persistence

### SortableJS pattern (existing usage)
- `src/components/RebaseEditor.svelte` — Existing SortableJS usage (line ~170): `Sortable.create()` with onEnd handler, dragClass, animation

### Tree view
- `src/components/TreeFileList.svelte` — Tree rendering; needs `ondirectorycontextmenu` prop wired to DirectoryRow
- `src/components/DirectoryRow.svelte` — Directory node; needs `oncontextmenu` event handler
- `src/components/StagingPanel.svelte` — Staging panel with existing file context menus (`showUnstagedContextMenu`, `showStagedContextMenu`, `showConflictedContextMenu`); needs new directory context menu handlers

### Backend staging
- `src-tauri/src/commands/staging.rs` — `stage_file`, `unstage_file` commands (directory ops loop on frontend)

### Requirements
- `.planning/REQUIREMENTS.md` — TAB-11, TREE-11 acceptance criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **SortableJS**: Already installed (`sortablejs@^1.15.7`), used in RebaseEditor with `Sortable.create()`, `onEnd` callback, and `dragClass` for styling
- **TabBar.svelte**: Simple `{#each tabs}` loop with `flex-shrink: 0` items — SortableJS can wrap the tab container
- **`persistTabs()` in App.svelte**: Debounced 500ms persistence — call after SortableJS reorder updates the `tabs` array
- **Native Tauri menu API**: `@tauri-apps/api/menu` with `Menu.new()` / `MenuItem.new()` / `.popup()` — used extensively across the app
- **StagingPanel context menu handlers**: `showUnstagedContextMenu`, `showStagedContextMenu`, `showConflictedContextMenu` — pattern to follow for directory equivalents

### Established Patterns
- **SortableJS integration**: RebaseEditor uses `$effect` with `Sortable.create()` in onMount, returns cleanup function
- **Context menus**: All use dynamic `@tauri-apps/api/menu` imports, build menu items, call `.popup()`
- **Directory actions**: Frontend loops over `FileStatus[]` entries matching directory path prefix (Phase 48 D-11)
- **Confirmation dialogs**: Discard operations use `confirm()` or Tauri dialog before executing

### Integration Points
- **TabBar container**: SortableJS wraps the `.tab-bar` div, excluding the `.new-tab-btn`
- **App.svelte `tabs` array**: SortableJS `onEnd` reorders the array, triggers `persistTabs()`
- **DirectoryRow**: Needs new `oncontextmenu` prop; TreeFileList passes it through
- **StagingPanel**: New `showUnstagedDirContextMenu`, `showStagedDirContextMenu`, `showConflictedDirContextMenu` handlers

</code_context>

<specifics>
## Specific Ideas

- SortableJS swap animation for tabs matches the existing RebaseEditor drag-and-drop feel — consistent DnD behavior across the app
- Directory context menus are section-aware: unstaged shows Stage All + Discard All, staged shows Unstage All, conflicted shows Resolve All + Unresolve All
- Discard All on a directory is the only destructive context menu action — always requires confirmation

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 49-tab-drag-tree-context-menu*
*Context gathered: 2026-03-24*
