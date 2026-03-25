# Phase 48: Polish & Differentiators - Context

**Gathered:** 2026-03-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Competitive-parity tab interactions and tree view power features that elevate UX beyond basic functionality. Covers requirements TAB-08 (tab context menu), TAB-09 (middle-click close), TAB-10 (duplicate detection), TREE-08 (directory staging), TREE-09 (count badges), TREE-10 (Expand All / Collapse All).

</domain>

<decisions>
## Implementation Decisions

### Tab context menu (TAB-08)
- **D-01:** Right-click a tab opens a native Tauri context menu with three actions: Close Others, Close All, Copy Path.
- **D-02:** "Close Others" closes all tabs except the right-clicked tab. The right-clicked tab becomes active.
- **D-03:** "Close All" closes every tab and opens a new empty tab (same behavior as closing the last tab — Phase 45).
- **D-04:** "Copy Path" copies the repo's absolute filesystem path to the clipboard. Disabled (grayed out) for tabs without an open repo.
- **D-05:** No additional context menu items — keep it minimal for v0.9.

### Middle-click close (TAB-09)
- **D-06:** Middle-click on a tab = graceful close (same behavior as clicking the X button — Phase 45 D-10). No force close via middle-click.

### Duplicate tab detection (TAB-10)
- **D-07:** When the user opens a repo (from WelcomeScreen or any entry point), check all existing tabs for a matching normalized absolute path. If found, switch to the existing tab instead of opening a duplicate.
- **D-08:** Silent switch — no toast or notification. The tab bar visually confirms the switch.
- **D-09:** The "New Tab" page that triggered the open should be closed if it has no repo (it was just a transient empty tab).

### Directory staging (TREE-08)
- **D-10:** Clicking the action button on a directory row stages (or unstages) all files within that directory recursively.
- **D-11:** Frontend implementation — loop over all `FileStatus` entries whose paths start with the directory's path prefix and call `stage_file` / `unstage_file` for each. No new Rust backend command needed.
- **D-12:** The action button appears on hover, matching the existing FileRow hover pattern. Icon: a stage (plus) or unstage (minus) icon consistent with FileRow's action button.
- **D-13:** The action label matches the section context: "Stage" button in unstaged section, "Unstage" button in staged section. Same as individual file actions.

### Count badges (TREE-09)
- **D-14:** Directory nodes display the recursive count of files within that directory (not just direct children).
- **D-15:** Badge rendered as inline muted text after the directory name, e.g. `src/lib (3)`. Uses `var(--color-text-muted)` for the count.
- **D-16:** Count is visible in both collapsed and expanded states — always informative.

### Expand All / Collapse All (TREE-10)
- **D-17:** Two icon buttons (Expand All, Collapse All) placed in the staging panel header bar, next to the existing flat/tree toggle.
- **D-18:** Buttons only visible when tree mode is active — irrelevant in flat mode.
- **D-19:** Expand All opens every directory node in all sections (unstaged, staged, conflicted). Collapse All closes them all.
- **D-20:** Uses Lucide icons: `ChevronsDownUp` for Collapse All, `ChevronsUpDown` for Expand All (or similar chevron-based icons).

### Claude's Discretion
- Native Tauri menu API vs custom Svelte context menu for tab right-click (native recommended for consistency with existing graph context menus)
- Whether directory staging calls are sequential or batched (Promise.all for performance)
- Exact Lucide icon choices for Expand All / Collapse All
- How to propagate Expand All / Collapse All to all TreeFileList instances (callback props vs shared signal)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Tab system
- `src/components/TabBar.svelte` — Current tab bar with click/close handlers; needs context menu and middle-click
- `src/App.svelte` — Tab CRUD (addNewTab, closeTab, forceCloseTab, openRepoInTab); needs duplicate detection and Close Others/All
- `src/lib/tab-types.ts` — TabInfo type definition
- `src/lib/store.ts` — Tab persistence (getOpenTabs, setOpenTabs, getActiveTabId)

### Tree view
- `src/components/TreeFileList.svelte` — Tree rendering with expanded state, keyboard nav; needs Expand All/Collapse All support
- `src/components/DirectoryRow.svelte` — Directory node rendering (chevron + name); needs count badge and stage/unstage action button
- `src/components/FileRow.svelte` — File node with hover action button pattern to replicate for DirectoryRow
- `src/components/StagingPanel.svelte` — Staging panel header bar; needs Expand All/Collapse All buttons next to tree toggle
- `src/lib/build-tree.ts` — TreeNode, DirectoryNode, FileNode types; DirectoryNode.children for recursive count

### Backend staging
- `src-tauri/src/commands/staging.rs` — `stage_file`, `unstage_file` commands (no directory-level command exists; frontend loops)

### Context menus (existing pattern)
- `src/App.svelte` — Existing Tauri native context menu usage for graph commit rows (pattern to follow)

### Requirements
- `.planning/REQUIREMENTS.md` — TAB-08, TAB-09, TAB-10, TREE-08, TREE-09, TREE-10 acceptance criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **TabBar.svelte**: Already has click/close handlers, scroll overflow, dirty dot — add contextmenu and auxclick events
- **DirectoryRow.svelte**: 26px height, chevron + name — extend with count badge text and hover action button
- **FileRow.svelte**: Hover action button pattern (icon button appears on hover, triggers onaction callback) — replicate for DirectoryRow
- **TreeFileList.svelte**: Owns `expanded` Set per instance — Expand All/Collapse All can set/clear this Set via callback
- **`buildTree()`**: DirectoryNode.children array enables recursive file counting
- **`@tauri-apps/plugin-clipboard-manager`**: Already imported in StagingPanel for Copy Path action
- **Lucide icons**: Full library available (`ChevronsDownUp`, `ChevronsUpDown`, `FolderPlus`, `FolderMinus`, etc.)

### Established Patterns
- **Native Tauri context menus**: Used for graph commit rows — `@tauri-apps/api/menu` with `Menu.new()` / `MenuItem.new()` / `.popup()`
- **`safeInvoke` for all IPC**: All backend calls go through `src/lib/invoke.ts`
- **CSS custom properties**: All colors via `var(--color-*)` — count badge must use `--color-text-muted`
- **Hover action pattern**: FileRow shows action button on `:hover` — DirectoryRow follows same CSS pattern

### Integration Points
- **TabBar `oncontextmenu`**: New event handler on `.tab-item` div to show native context menu
- **TabBar `onauxclick`**: Middle-click (button === 1) on `.tab-item` triggers close
- **App.svelte `openRepoInTab`**: Add duplicate check before opening — scan `tabs` for matching `repoPath`
- **StagingPanel header**: Add Expand All / Collapse All buttons beside the existing List/FolderTree toggle
- **TreeFileList**: Expose `expandAll()` / `collapseAll()` via callback props from StagingPanel

</code_context>

<specifics>
## Specific Ideas

- Tab context menu follows the same native Tauri menu pattern already used for commit graph context menus — consistent across the app
- Directory staging loops on frontend (no backend changes) — keeps the Rust API surface minimal
- Count badges use the VS Code style: muted inline text, not a separate pill or circle badge
- Expand All / Collapse All buttons should be small icon-only buttons that blend into the header bar

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 48-polish-differentiators*
*Context gathered: 2026-03-24*
