# Phase 40: Rebase Workflow - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can rebase branches through the GUI with full conflict resolution support during the rebase. This phase adds "Rebase" context menu items to all branch surfaces (sidebar, graph pills, commit menu), handles successful rebases and mid-rebase conflicts using existing infrastructure (Phases 37-38). Drag-and-drop rebase initiation is explicitly dropped (REB-02). Interactive rebase editor is Phase 41.

</domain>

<decisions>
## Implementation Decisions

### Rebase feedback
- **No toast on success** — silent like merge (Phase 39 pattern). Graph refresh shows rewritten history, which is sufficient visual feedback
- **Error toast for failures** — git error message shown as red error toast, consistent with merge error handling
- **Banner updates silently on re-conflict** — when Continue hits another conflicting commit, the staging panel refreshes with new conflicts. No toast or progress indicator
- **User clicks conflicted file** — no auto-open of first conflicted file when rebase pauses (consistent with merge flow, Phase 39 decision)

### Dirty workdir handling
- **Let git decide** — pass through to git CLI, show git's error as error toast if rejected. Same approach as merge (Phase 39)

### Rebase confirmation
- **No confirmation dialog** — clicking "Rebase" executes immediately, consistent with merge behavior
- Abort already requires confirmation (Phase 37 decision), which provides safety for the destructive reverse action
- GitKraken doesn't confirm rebase either

### Drag-and-drop
- **REB-02 dropped entirely** — no drag-and-drop to initiate rebase, just like MERGE-02 was dropped for merge
- Rebase initiation is exclusively via right-click context menu
- Drag-and-drop is reserved for Phase 41's interactive rebase editor (commit reordering only)

### Menu surface coverage
- **Same surfaces as merge** — "Rebase [current] onto [branch]" appears everywhere "Merge [branch] into [current]" appears:
  - BranchSidebar local branches
  - BranchSidebar remote branches
  - CommitGraph commit context menu (already exists)
  - CommitGraph local branch pill context menu
  - CommitGraph remote branch pill context menu
  - CommitGraph overflow ref context menu
- **Hidden on HEAD branch** — can't rebase onto yourself, same pattern as merge

### Menu wording
- **"Rebase [current] onto [branch]"** — already used in CommitGraph commit menu, apply consistently everywhere
- Example: "Rebase main onto feature-x"

### Menu grouping
- **Merge and Rebase grouped together** — adjacent in context menus, then separator. CommitGraph commit menu already does this

### Post-rebase divergence
- **Existing ahead/behind is enough** — sidebar already shows ahead/behind counts. After rebase, ahead count naturally reflects divergence. No special warning needed

### Claude's Discretion
- Exact menu item position relative to merge item (after merge recommended)
- Loading state during rebase execution
- Whether to remove the existing success toast in CommitGraph's `handleRebaseBranch()` or leave it (should be removed for consistency)
- How to handle the existing `handleRebaseBranch()` in CommitGraph — may need refactoring to share with other surfaces

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — REB-01 (context menu rebase), REB-04 (mid-rebase conflicts), REB-05 (abort rebase), REB-06 (skip commit). REB-02 (drag-and-drop) is dropped. REB-03 (interactive rebase) is Phase 41.

### Prior phase context
- `.planning/phases/37-conflict-detection-operation-state/37-CONTEXT.md` — Operation banner design, abort confirmation pattern, conflict section layout
- `.planning/phases/38-merge-editor/38-CONTEXT.md` — Merge editor placement, resolution flow
- `.planning/phases/39-merge-workflow/39-CONTEXT.md` — Merge context menu wiring pattern (reference for adding rebase to same surfaces), silent success pattern

### Existing rebase backend
- `src-tauri/src/commands/operation_state.rs` — `rebase_branch_inner()` (lines 267-291), `rebase_continue_inner()` (lines 175-195), `rebase_skip_inner()` (lines 196-216), `rebase_abort_inner()` (lines 217-237), Tauri command wrappers (lines 375-424, 456-466). All already registered in `lib.rs`.

### Context menu infrastructure (merge pattern to replicate)
- `src/components/BranchSidebar.svelte` — `handleMergeBranch()` (line 303), merge items in local branch menu (line 327), remote branch menu (lines 360-368)
- `src/components/CommitGraph.svelte` — `handleRebaseBranch()` (line 290, already exists), `handleMergeBranch()` (line 280), commit menu with merge+rebase items (lines 309-315), local pill menu (lines 458-465), remote pill menu (lines 481-488), overflow ref menu (lines 514-544)

### Operation state UI
- `src/components/OperationBanner.svelte` — Already handles rebase state with Continue/Skip/Abort buttons
- `src/components/StagingPanel.svelte` — Already handles rebase-in-progress state detection

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `rebase_branch` Tauri command: Already implemented and registered — handles `git rebase <onto_branch>`, detects conflicts, returns refreshed graph
- `rebase_continue` / `rebase_skip` / `rebase_abort` commands: Already implemented for mid-rebase operations
- `handleRebaseBranch()` in CommitGraph: Already wired to `rebase_branch` IPC call with toast — needs toast removal
- `OperationBanner.svelte`: Already handles rebase banner with Continue/Skip/Abort buttons
- `safeInvoke<T>`: Type-safe IPC wrapper for all frontend-to-backend calls
- `showToast()`: For error feedback on rebase failures
- Native Tauri `Menu` / `MenuItem` API: Used throughout for context menus

### Established Patterns
- Phase 39 merge wiring: Exact pattern to replicate for rebase — same surfaces, same menu structure
- Context menu pattern: `import('@tauri-apps/api/menu')` → `Menu.new({ items: [...] })` → `menu.popup()`
- Git CLI subprocess: `std::process::Command::new("git")` with `GIT_TERMINAL_PROMPT=0` and `GIT_EDITOR=true`
- Cache-repopulate-before-emit: After rebase, refresh CommitCache then emit `repo-changed`

### Integration Points
- `BranchSidebar.svelte`: Add `handleRebaseBranch()` and "Rebase" menu items alongside existing merge items
- `CommitGraph.svelte` pill menus: Add rebase items to local pill, remote pill, and overflow ref menus (commit menu already done)
- Existing `handleRebaseBranch()` shows success toast — remove for consistency with silent merge pattern

</code_context>

<specifics>
## Specific Ideas

- Backend work is entirely done — all rebase commands exist and are registered
- CommitGraph commit menu already has rebase wired — primary work is adding to remaining surfaces (BranchSidebar + pill menus)
- Very similar scope to Phase 39 (merge wiring) — mostly frontend context menu additions
- Remove existing success toast from `handleRebaseBranch()` to match silent merge pattern

</specifics>

<deferred>
## Deferred Ideas

- REB-02 (drag-and-drop rebase initiation) — **dropped entirely**, not deferred
- REB-03 (interactive rebase via commit right-click) — Phase 41

</deferred>

---

*Phase: 40-rebase-workflow*
*Context gathered: 2026-03-21*
