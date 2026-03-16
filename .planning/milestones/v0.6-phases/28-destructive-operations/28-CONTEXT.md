# Phase 28: Destructive Operations - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can perform common destructive git operations (discard, delete, rename, reset) with clear confirmation safeguards. Specifically: discard individual unstaged file changes, discard all unstaged changes, delete local branches, delete tags, rename local branches, and reset current branch to any commit (soft/mixed/hard). All destructive actions require confirmation dialogs. Remote branch delete is out of scope.

**Note:** Reset (GITOP-06) is already fully implemented — backend `reset_to_commit` command + frontend context menu submenu with soft/mixed/hard modes and confirmation dialogs. No work needed.

</domain>

<decisions>
## Implementation Decisions

### Discard single file
- Triggered via right-click context menu on unstaged file rows only — no hover trash button
- Confirmation dialog: "Discard changes to {filename}? This cannot be undone." — simple, file name only, no status info
- For tracked files: reverts to last committed state (git checkout -- file)
- For untracked files: deletes the file from disk entirely
- Untracked files get a stronger warning: "Delete {filename}? This file is untracked and will be permanently removed. This cannot be undone."
- Uses `ask()` from `@tauri-apps/plugin-dialog` with `kind: 'warning'`

### Discard all unstaged changes
- "Discard All" button in the unstaged files section header bar, alongside the existing "Stage All Changes" button
- Includes both tracked modifications AND untracked files (git checkout -- . + git clean -fd equivalent)
- Confirmation shows file count: "Discard all changes to {N} files? This cannot be undone." — single count, no modified/untracked breakdown
- Uses `ask()` with `kind: 'warning'`

### Branch delete
- Accessible from both sidebar right-click context menu AND graph pill right-click context menu
- HEAD branch: "Delete" menu item appears greyed out (disabled) — matches merge-commit disabled cherry-pick pattern from Phase 12
- Confirmation: "Delete branch '{name}'? This cannot be undone." — branch name only, no SHA
- Uses `ask()` with `kind: 'warning'`
- After success: toast notification

### Tag delete
- Accessible from both sidebar right-click context menu AND graph pill right-click context menu
- Confirmation: "Delete tag '{name}'? This cannot be undone."
- Uses `ask()` with `kind: 'warning'`
- After success: toast notification

### Branch rename
- Accessible from both sidebar right-click context menu AND graph pill right-click context menu
- Uses `InputDialog` component (same as branch/tag create from Phase 12)
- Pre-fills the current branch name — user edits in place
- Renaming HEAD (currently checked-out) branch is allowed — git branch -m supports this
- No extra confirmation step after submitting the dialog — rename is reversible (just rename again)
- After success: toast notification

### Reset (soft/mixed/hard)
- Already fully implemented in Phase 12 — backend `reset_to_commit_inner` in `commit_actions.rs`, frontend `handleReset` in `CommitGraph.svelte` with submenu
- No additional work needed for GITOP-06

### Claude's Discretion
- Exact Rust implementation for `discard_file_inner`, `discard_all_inner`, `delete_branch_inner`, `delete_tag_inner`, `rename_branch_inner`
- Whether discard uses git2 API or git CLI subprocess
- Context menu item ordering and separator placement in sidebar menus
- How graph pill right-click menus are implemented (new context menu surface on SVG pills)
- Toast message copy for success/error cases
- Error handling for edge cases (e.g., branch has unmerged changes, rename conflicts with existing branch name)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above and REQUIREMENTS.md.

### Requirements
- `.planning/REQUIREMENTS.md` — GITOP-01 through GITOP-06 define acceptance criteria for this phase

### Prior phase context
- `.planning/phases/12-commit-context-menu/12-CONTEXT.md` — Context menu patterns, InputDialog usage, branch/tag creation UX decisions
- `.planning/phases/11-stash-operations/11-CONTEXT.md` — Stash context menu in sidebar pattern, drop confirmation dialog

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `@tauri-apps/api/menu` (Menu, MenuItem, PredefinedMenuItem, Submenu): Used in CommitGraph.svelte and BranchSidebar.svelte for all context menus
- `ask()` / `message()` from `@tauri-apps/plugin-dialog`: Confirmation pattern used for checkout, reset, stash drop
- `InputDialog.svelte`: Modal with fields array, onsubmit/oncancel — used for branch/tag create, reuse for rename
- `showToast()` from `src/lib/toast.svelte.ts`: Success/error feedback after operations
- `safeInvoke<T>`: All IPC wrapper — new commands follow same pattern
- `open_repo_from_state()` helper in `branches.rs`: Opens git2 repo from state map
- `is_dirty()` helper in `branches.rs`: Checks working tree dirtiness

### Established Patterns
- **inner-fn pattern**: All commands have `foo_inner()` (pure logic) + `#[tauri::command] foo()` (Tauri glue with spawn_blocking)
- **cache-repopulate-before-emit**: Mutation commands rebuild CommitCache before emitting `repo-changed`
- **Native Tauri Menu API**: `Menu.new({ items }) → menu.popup()` — no custom Svelte context menus
- **Dialog config pattern**: `dialogConfig` $state object toggles InputDialog visibility and content
- **git CLI for complex ops**: cherry-pick/revert shell out to git CLI. Branch rename/delete could use git2 directly since they're simple local operations
- **`oncontextmenu` prop pattern**: CommitRow.svelte receives `oncontextmenu` prop, parent dispatches to different menus based on row type

### Integration Points
- `FileRow.svelte`: Add `oncontextmenu` prop (currently only has onclick/onaction)
- `StagingPanel.svelte`: Wire file context menu for discard, add "Discard All" button to unstaged header
- `BranchRow.svelte`: Add `oncontextmenu` prop (currently only has onclick)
- `BranchSidebar.svelte`: Wire branch/tag context menus for delete/rename on local branch rows and tag rows
- `CommitGraph.svelte`: Add right-click handling on graph pills (SVG foreignObject elements) for branch delete/rename and tag delete
- `src-tauri/src/commands/branches.rs`: Add `delete_branch_inner`, `rename_branch_inner`
- `src-tauri/src/commands/staging.rs`: Add `discard_file_inner`, `discard_all_inner`
- `src-tauri/src/commands/commit_actions.rs`: Add `delete_tag_inner`
- `src-tauri/src/lib.rs`: Register new commands in `generate_handler![]`

</code_context>

<specifics>
## Specific Ideas

- Graph pills for branches AND tags should both support right-click with delete (and rename for branches) — consistent surface, not just sidebar
- HEAD branch gets disabled delete menu item (greyed out), not hidden — user should see the option exists but understand why it's blocked
- Untracked file discard gets an explicit "delete" warning to differentiate from tracked file revert — user should know the file disappears from disk
- Rename pre-fills current name for quick typo fixes — doesn't follow Phase 12 branch create pattern (which starts empty), because rename is a different use case
- Reset is already done — GITOP-06 requires no new work

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 28-destructive-operations*
*Context gathered: 2026-03-15*
