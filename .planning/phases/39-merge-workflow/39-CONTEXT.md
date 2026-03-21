# Phase 39: Merge Workflow - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can initiate and complete merges through the GUI without touching the terminal. This phase adds "Merge" context menu items to branch right-click menus (sidebar and graph pills), handles fast-forward and non-conflicting merges, and wires into the existing conflict resolution infrastructure (Phases 37-38). Drag-and-drop merge initiation is explicitly out of scope (dropped from requirements). Interactive rebase editor is Phase 41.

</domain>

<decisions>
## Implementation Decisions

### Merge confirmation
- **No confirmation dialog** — clicking "Merge" executes immediately
- Non-destructive action: conflicts pause it, success auto-commits
- Consistent with GitKraken behavior
- Abort already requires confirmation (Phase 37 decision)

### Merge feedback
- **No toast on success** — neither fast-forward nor merge commit
- The graph refreshes and shows the new state (FF advances pointer, merge shows merge commit) — sufficient visual feedback
- **Error toast for failures** — git error message shown as red error toast, consistent with existing checkout/remote error patterns

### Dirty workdir handling
- **Let git decide** — pass through to git CLI
- Git rejects if changes conflict with merge, allows if they don't
- Show git's error as error toast if rejected

### Conflict flow
- **Banner + conflicts appear** — reuses Phase 37 infrastructure entirely
- Merge operation banner appears, conflicted files show in staging panel
- User resolves via merge editor (Phase 38), then clicks Continue
- No auto-opening of first conflicted file

### Merge commit message
- **Editable in staging panel** — pre-filled with standard "Merge branch 'X' into Y" message
- User can edit before clicking Continue
- Already partially built in StagingPanel (lines 196-203 pre-fill the message)

### Menu wording
- **"Merge [branch] into [current]"** — fully explicit with both branch names
- Example: "Merge feature-x into main"
- No ambiguity about merge direction

### Menu placement
- Claude's discretion on exact position relative to Checkout/Rename/Delete
- Group with primary actions (near Checkout), keep destructive ops (rename/delete) separated

### Menu visibility
- **Both local and remote branches** get the merge context menu option
- "Merge origin/feature-x into main" is valid — git merge works with remote refs
- **Hidden on HEAD branch** — can't merge a branch into itself, same pattern as Checkout being disabled for HEAD

### Drag-and-drop
- **No drag-and-drop** — MERGE-02 dropped entirely from requirements, not deferred
- Merge is initiated exclusively via right-click context menu

### Claude's Discretion
- Exact menu item position in context menu (after Checkout recommended)
- Whether to add a separator before/after the Merge item
- Fast-forward detection approach (git output parsing vs rev-list check)
- Loading state during merge execution (disable menu item, spinner, etc.)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — MERGE-01 (context menu merge), MERGE-03 (fast-forward handling), MERGE-04 (non-conflicting auto-commit). MERGE-02 (drag-and-drop) is dropped.

### Prior phase context
- `.planning/phases/37-conflict-detection-operation-state/37-CONTEXT.md` — Operation banner design, conflict section layout, abort confirmation pattern
- `.planning/phases/38-merge-editor/38-CONTEXT.md` — Merge editor placement, resolution flow, Save and Mark Resolved behavior

### Existing merge backend
- `src-tauri/src/commands/operation_state.rs` — `merge_branch_inner()` (lines 240-265), `merge_continue_inner()` (lines 122-152), `merge_abort_inner()` (lines 154-173), Tauri command wrappers (lines 435-453). Already registered in `lib.rs`.

### Context menu infrastructure
- `src/components/BranchSidebar.svelte` — `showBranchContextMenu()` (lines 303-325) with Checkout/Rename/Delete items for local branches, tag context menu (lines 327-338)
- `src/components/CommitGraph.svelte` — `showPillContextMenu()` (lines 452-483) for graph ref pill right-click, SVG pill rendering (lines 958-1022)

### Staging panel merge state
- `src/components/StagingPanel.svelte` — `isMerge` derived state (line 34-40), merge message pre-fill (lines 196-203), `commitMerge()` (lines 205-222), `abortMerge()` (lines 224-242)
- `src/components/OperationBanner.svelte` — Merge/rebase state display with Continue/Abort buttons

### Supporting code
- `src/lib/toast.svelte.ts` — `showToast()` (lines 22-26) for error feedback
- `src/lib/invoke.ts` — `safeInvoke<T>` for all IPC calls
- `src-tauri/src/commands/commit_actions.rs` — Cherry-pick/revert pattern (git CLI subprocess with error handling)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `merge_branch` Tauri command: Already implemented and registered — handles `git merge <branch> --no-edit`, detects conflicts, returns refreshed graph
- `merge_continue` / `merge_abort` commands: Already implemented for post-conflict flow
- `safeInvoke<T>`: Type-safe IPC wrapper for all frontend-to-backend calls
- `showToast()`: For error feedback on merge failures
- Native Tauri `Menu` / `MenuItem` API: Used in both BranchSidebar and CommitGraph for context menus
- StagingPanel merge message pre-fill: Already populates "Merge branch 'X' into Y" when merge state detected

### Established Patterns
- Context menu pattern: `import('@tauri-apps/api/menu')` → `Menu.new({ items: [...] })` → `menu.popup()`
- Git CLI subprocess: `std::process::Command::new("git")` with `GIT_TERMINAL_PROMPT=0` and `GIT_EDITOR=true`
- Cache-repopulate-before-emit: After merge, refresh CommitCache then emit `repo-changed`
- `spawn_blocking` for git operations: Keeps async Tauri command responsive

### Integration Points
- `BranchSidebar.svelte` `showBranchContextMenu()`: Add "Merge [branch] into [current]" item — needs current HEAD branch name
- `CommitGraph.svelte` `showPillContextMenu()`: Add same merge item for LocalBranch ref pills — needs HEAD branch from commits data
- `merge_branch` command already returns `GraphResult` on success — frontend just needs to handle the response
- Conflict path: `merge_branch_inner()` returns error with `conflict_state` code — frontend shows operation banner + conflicts

</code_context>

<specifics>
## Specific Ideas

- Much of the backend work is already done (`merge_branch`, `merge_continue`, `merge_abort` commands exist)
- Primary work is wiring context menu items to invoke the existing backend
- StagingPanel already handles merge-in-progress state with editable message

</specifics>

<deferred>
## Deferred Ideas

- MERGE-02 (drag-and-drop merge initiation) — **dropped entirely**, not deferred

</deferred>

---

*Phase: 39-merge-workflow*
*Context gathered: 2026-03-21*
