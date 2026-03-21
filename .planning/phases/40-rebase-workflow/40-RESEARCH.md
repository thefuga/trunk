# Phase 40: Rebase Workflow - Research

**Researched:** 2026-03-21
**Domain:** Frontend context menu wiring for git rebase, following Phase 39 merge pattern
**Confidence:** HIGH

## Summary

Phase 40 is a frontend-only wiring phase that adds "Rebase [current] onto [branch]" context menu items to all branch surfaces where merge already exists. The backend is entirely complete -- `rebase_branch`, `rebase_continue`, `rebase_skip`, and `rebase_abort` Tauri commands are implemented and registered. The OperationBanner already handles rebase state with Continue/Skip/Abort buttons. The StagingPanel already shows conflicted files in a dedicated section during rebase (the `!isMerge` path).

The primary work is: (1) add `handleRebaseBranch()` to BranchSidebar with rebase menu items in local and remote branch context menus, (2) add rebase items to CommitGraph pill and overflow ref menus (commit menu already has rebase wired), and (3) remove the existing success toast from CommitGraph's `handleRebaseBranch()`. This is structurally identical to Phase 39 -- same surfaces, same menu infrastructure, same error handling pattern.

**Primary recommendation:** Replicate the Phase 39 merge wiring pattern exactly, replacing merge-specific text/commands with rebase equivalents. A single plan with 2 tasks (CommitGraph fixes + BranchSidebar additions) plus a human verification checkpoint is sufficient.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **No toast on success** -- silent like merge (Phase 39 pattern). Graph refresh shows rewritten history, which is sufficient visual feedback
- **Error toast for failures** -- git error message shown as red error toast, consistent with merge error handling
- **Banner updates silently on re-conflict** -- when Continue hits another conflicting commit, the staging panel refreshes with new conflicts. No toast or progress indicator
- **User clicks conflicted file** -- no auto-open of first conflicted file when rebase pauses (consistent with merge flow, Phase 39 decision)
- **Let git decide** on dirty workdir -- pass through to git CLI, show git's error as error toast if rejected. Same approach as merge (Phase 39)
- **No confirmation dialog** -- clicking "Rebase" executes immediately, consistent with merge behavior
- **REB-02 dropped entirely** -- no drag-and-drop to initiate rebase
- **Same surfaces as merge** -- "Rebase [current] onto [branch]" appears everywhere "Merge [branch] into [current]" appears
- **Hidden on HEAD branch** -- can't rebase onto yourself, same pattern as merge
- **Menu wording** -- "Rebase [current] onto [branch]" consistently everywhere
- **Merge and Rebase grouped together** -- adjacent in context menus, then separator
- **Existing ahead/behind is enough** -- no special post-rebase divergence warning

### Claude's Discretion
- Exact menu item position relative to merge item (after merge recommended)
- Loading state during rebase execution
- Whether to remove the existing success toast in CommitGraph's `handleRebaseBranch()` or leave it (should be removed for consistency)
- How to handle the existing `handleRebaseBranch()` in CommitGraph -- may need refactoring to share with other surfaces

### Deferred Ideas (OUT OF SCOPE)
- REB-02 (drag-and-drop rebase initiation) -- **dropped entirely**, not deferred
- REB-03 (interactive rebase via commit right-click) -- Phase 41
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| REB-01 | User can rebase current branch onto another branch via right-click context menu on a branch | Add rebase menu items to BranchSidebar (local + remote) and CommitGraph (pill + overflow ref) context menus. Backend `rebase_branch` command already exists and is registered. CommitGraph commit menu already has this wired. |
| REB-02 | User can initiate rebase by dragging a branch onto another branch in the graph | **DROPPED** per user decision. No implementation needed. |
| REB-04 | Mid-rebase conflicts pause the rebase and show conflicted files in the staging panel for resolution via the merge editor | Already working. Backend `rebase_branch_inner()` detects conflicts and returns graph result. StagingPanel shows conflicted files in dedicated section when `!isMerge`. OperationBanner shows rebase banner with Continue/Skip/Abort. No new code needed. |
| REB-05 | User can abort an in-progress rebase to restore the repository to its pre-rebase state | Already working. `rebase_abort` command exists. OperationBanner Abort button already wired with confirmation dialog. No new code needed. |
| REB-06 | User can skip a conflicting commit during rebase and continue with the next commit | Already working. `rebase_skip` command exists. OperationBanner Skip button already wired. No new code needed. |
</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Purpose | Relevance |
|---------|---------|-----------|
| Svelte 5 | Frontend framework | All menu wiring is in Svelte components |
| @tauri-apps/api/menu | Native context menus | `Menu`, `MenuItem`, `PredefinedMenuItem` used for all right-click menus |
| Tauri 2 | Backend framework | `safeInvoke` calls to registered commands |

No new libraries needed. This phase uses exclusively existing infrastructure.

## Architecture Patterns

### Pattern 1: Context Menu Wiring (from Phase 39)

**What:** Adding menu items to existing context menus by following the established merge pattern.
**When to use:** Every surface that has a "Merge X into Y" item gets a corresponding "Rebase [current] onto [branch]" item.

**Surfaces requiring rebase items:**

| Surface | Component | Merge exists at | Rebase status |
|---------|-----------|----------------|---------------|
| Commit context menu | CommitGraph.svelte L309-315 | `showCommitContextMenu()` | Already wired (L313) |
| Local branch pill | CommitGraph.svelte L456-465 | `showPillContextMenu()` LocalBranch case | Needs rebase item |
| Remote branch pill | CommitGraph.svelte L479-490 | `showPillContextMenu()` RemoteBranch case | Needs rebase item |
| Local overflow ref | CommitGraph.svelte L512-534 | `showOverflowRefContextMenu()` LocalBranch case | Needs rebase item |
| Remote overflow ref | CommitGraph.svelte L535-546 | `showOverflowRefContextMenu()` RemoteBranch case | Needs rebase item |
| Sidebar local branch | BranchSidebar.svelte L315-344 | `showBranchContextMenu()` | Needs handler + item |
| Sidebar remote branch | BranchSidebar.svelte L359-372 | `showRemoteContextMenu()` | Needs rebase item |

### Pattern 2: Rebase Handler (mirror of merge handler)

**What:** An async function that calls `safeInvoke('rebase_branch', ...)` with error-only toast.

**CommitGraph pattern** (existing, needs toast removal):
```typescript
// Current (has success toast -- needs removal):
async function handleRebaseBranch(ontoBranch: string) {
  try {
    await safeInvoke('rebase_branch', { path: repoPath, ontoBranch });
    showToast(`Rebased onto ${ontoBranch}`, 'success'); // REMOVE THIS
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Rebase failed', 'error');
  }
}

// Target (no success toast, matches merge):
async function handleRebaseBranch(ontoBranch: string) {
  try {
    await safeInvoke('rebase_branch', { path: repoPath, ontoBranch });
    // No toast on success -- graph refresh via repo-changed event is sufficient
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Rebase failed', 'error');
  }
}
```

**BranchSidebar pattern** (new, mirrors `handleMergeBranch`):
```typescript
async function handleRebaseBranch(ontoBranch: string) {
  try {
    await safeInvoke('rebase_branch', { path: repoPath, ontoBranch });
    // No toast on success -- graph refresh via repo-changed event is sufficient
    await loadRefs(repoPath);
    onrefreshed?.();
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Rebase failed', 'error');
  }
}
```

Note: BranchSidebar handler includes `loadRefs` + `onrefreshed` because the sidebar needs to refresh its own ref list (same pattern as `handleMergeBranch` at line 303).

### Pattern 3: Menu Item Grouping

**What:** Merge and Rebase items appear adjacent, followed by a separator.

**Example for local branch pill menu:**
```typescript
// Merge + Rebase grouped, then separator
...(!pill.isHead && headBranchName ? [
  await MenuItem.new({
    text: `Merge ${pill.label} into ${headBranchName}`,
    action: () => { handleMergeBranch(pill.label).catch(() => {}); },
  }),
  await MenuItem.new({
    text: `Rebase ${headBranchName} onto ${pill.label}`,
    action: () => { handleRebaseBranch(pill.label).catch(() => {}); },
  }),
  await PredefinedMenuItem.new({ item: 'Separator' }),
] : []),
```

### Pattern 4: Menu Wording Distinction

**Merge:** "Merge [clicked-branch] into [HEAD]" -- clicked branch is the source
**Rebase:** "Rebase [HEAD] onto [clicked-branch]" -- clicked branch is the target (onto)

The subjects and prepositions are swapped. The merge handler receives the source branch, the rebase handler receives the onto branch. The existing `rebase_branch` Tauri command parameter is named `onto_branch`.

### Anti-Patterns to Avoid
- **Don't add new backend commands** -- all rebase backend code is complete and registered
- **Don't modify OperationBanner** -- it already handles rebase state correctly with Continue/Skip/Abort
- **Don't modify StagingPanel** -- it already handles rebase conflict display (the `!isMerge` path shows conflicts in dedicated section)
- **Don't show success toast** -- the Phase 39 decision for merge silence applies to rebase too
- **Don't add confirmation dialog** -- rebase executes immediately on click (abort has confirmation via Phase 37)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rebase execution | Custom git2 rebase API | Existing `rebase_branch` Tauri command | Already implemented, conflict-aware, cache-refreshing |
| Conflict display | Custom conflict UI | Existing StagingPanel conflicted section + MergeEditor | Phase 37-38 infrastructure handles this |
| Operation banner | New rebase banner | Existing OperationBanner | Already renders rebase state with progress, Continue/Skip/Abort |
| Context menu API | Custom menu rendering | `@tauri-apps/api/menu` Menu/MenuItem | Native menus, established pattern |

**Key insight:** This phase has zero backend work and zero new UI components. It is purely context menu wiring -- connecting existing UI surfaces to existing backend commands.

## Common Pitfalls

### Pitfall 1: Forgetting a Menu Surface
**What goes wrong:** A branch surface shows "Merge" but not "Rebase", creating inconsistency.
**Why it happens:** There are 7 surfaces (commit menu, 2 pill types, 2 overflow types, sidebar local, sidebar remote). Easy to miss one.
**How to avoid:** Systematically go through the table in Architecture Patterns. Commit menu is already done -- verify the remaining 6.
**Warning signs:** Any context menu that shows "Merge" without a corresponding "Rebase" item.

### Pitfall 2: Wrong Wording Direction
**What goes wrong:** Menu says "Rebase [branch] onto [HEAD]" instead of "Rebase [HEAD] onto [branch]".
**Why it happens:** Confusing which branch name goes where -- merge and rebase have opposite subject/object.
**How to avoid:** Always: "Rebase {headBranchName} onto {clicked_branch}". The existing commit menu (L313) already does this correctly.
**Warning signs:** The user sees their current branch as the "onto" target instead of the source being rebased.

### Pitfall 3: Leaving the Success Toast in CommitGraph
**What goes wrong:** CommitGraph's existing `handleRebaseBranch()` shows `showToast('Rebased onto ${ontoBranch}', 'success')`. This is inconsistent with the silent merge pattern.
**Why it happens:** The function was written before Phase 39 established the "no success toast" convention.
**How to avoid:** Remove line 293 (`showToast(...)`) from the existing handler.
**Warning signs:** Success toast appears when rebasing from the commit context menu.

### Pitfall 4: Missing HEAD Branch Guards
**What goes wrong:** Rebase item shown when HEAD is detached or when right-clicking the HEAD branch itself.
**Why it happens:** Not replicating the `!isHead && headBranchName` guard from merge items.
**How to avoid:** Use the exact same guard pattern. For sidebar: `!isHead && headBranchName`. For pills: `!pill.isHead && headBranchName`. For remote: `headBranchName` (no isHead check needed since you can't be "on" a remote branch).
**Warning signs:** Rebase item visible when HEAD is detached or on the HEAD branch.

### Pitfall 5: BranchSidebar Handler Missing loadRefs
**What goes wrong:** After rebase from sidebar, the sidebar ref list doesn't update.
**Why it happens:** CommitGraph handler doesn't need `loadRefs` (it gets `repo-changed` event), but BranchSidebar does.
**How to avoid:** Include `await loadRefs(repoPath)` and `onrefreshed?.()` in BranchSidebar's handler, matching `handleMergeBranch`.
**Warning signs:** Sidebar shows stale branch state after rebase completes.

## Code Examples

### Existing Commit Menu Rebase Item (CommitGraph.svelte L313)
```typescript
// Source: CommitGraph.svelte line 313 -- already wired
await MenuItem.new({
  text: `Rebase ${headBranchName} onto ${clickedBranch.short_name}`,
  action: () => { handleRebaseBranch(clickedBranch.short_name).catch(() => {}); }
}),
```

### Existing Merge Item in Local Pill (CommitGraph.svelte L459-464)
```typescript
// Source: CommitGraph.svelte line 459-464 -- pattern to replicate
...(!pill.isHead && headBranchName ? [
  await MenuItem.new({
    text: `Merge ${pill.label} into ${headBranchName}`,
    action: () => { handleMergeBranch(pill.label).catch(() => {}); },
  }),
  await PredefinedMenuItem.new({ item: 'Separator' }),
] : []),
```

### Existing Merge Handler in BranchSidebar (L303-313)
```typescript
// Source: BranchSidebar.svelte line 303 -- pattern to replicate for rebase
async function handleMergeBranch(branch: string) {
  try {
    await safeInvoke('merge_branch', { path: repoPath, branch });
    // No toast on success -- graph refresh via repo-changed event is sufficient
    await loadRefs(repoPath);
    onrefreshed?.();
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Merge failed', 'error');
  }
}
```

### Existing Merge Item in BranchSidebar Menu (L325-329)
```typescript
// Source: BranchSidebar.svelte line 325-329 -- pattern to replicate
...(!isHead && headBranchName ? [
  await MenuItem.new({
    text: `Merge ${branchName} into ${headBranchName}`,
    action: () => { handleMergeBranch(branchName).catch(() => {}); },
  }),
] : []),
```

### Backend Command Signature (already registered)
```rust
// Source: operation_state.rs line 456-474
#[tauri::command]
pub async fn rebase_branch(
    path: String,
    onto_branch: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String>
```

### IPC Call Pattern
```typescript
// Frontend invocation -- parameter name must match: ontoBranch (camelCase of onto_branch)
await safeInvoke('rebase_branch', { path: repoPath, ontoBranch });
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Success toast on rebase | Silent (graph refresh sufficient) | Phase 39 established pattern | Remove existing toast from CommitGraph handler |
| Rebase only in commit menu | All branch surfaces | Phase 40 | Context menu parity with merge |

**Already complete (no changes needed):**
- Backend: `rebase_branch`, `rebase_continue`, `rebase_skip`, `rebase_abort` all registered in lib.rs
- OperationBanner: Renders rebase state with progress (X/Y), Continue/Skip/Abort buttons
- StagingPanel: Shows conflicted files during rebase in dedicated section with merge editor access

## Open Questions

1. **Separator placement around merge+rebase group**
   - What we know: Commit menu groups merge+rebase then adds separator before Copy SHA. Pill menus currently have merge then separator before Rename.
   - What's unclear: Exact separator arrangement when rebase is added to pills/overflow.
   - Recommendation: Replace the post-merge separator with a post-rebase separator (merge, rebase, separator, rename...). This keeps the group tight.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest |
| Config file | `vitest.config.ts` |
| Quick run command | `npm test` |
| Full suite command | `npm test && cd src-tauri && cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REB-01 | Rebase menu items in context menus | manual-only | N/A -- context menus are native Tauri menus, not testable in Vitest | N/A |
| REB-04 | Mid-rebase conflicts show in staging panel | manual-only | N/A -- requires full Tauri app context with git repo | N/A |
| REB-05 | Abort rebase restores pre-rebase state | unit (backend) | `cd src-tauri && cargo test rebase` | Partial -- operation_state.rs has tests but not rebase abort |
| REB-06 | Skip commit during rebase | unit (backend) | `cd src-tauri && cargo test rebase` | Partial -- command exists, no dedicated test |

### Sampling Rate
- **Per task commit:** `npm test`
- **Per wave merge:** `npm test && cd src-tauri && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None -- existing test infrastructure covers all automated testable parts. The primary work (context menu wiring) is inherently manual-only verification since it uses native Tauri menu APIs that cannot be unit tested.

## Sources

### Primary (HIGH confidence)
- Direct codebase inspection of all 5 source files: `operation_state.rs`, `CommitGraph.svelte`, `BranchSidebar.svelte`, `OperationBanner.svelte`, `StagingPanel.svelte`
- Phase 39 PLAN (39-01-PLAN.md) -- exact structural template for this phase
- Phase 40 CONTEXT.md -- user decisions and canonical references

### Secondary (MEDIUM confidence)
- Phase 39 CONTEXT.md -- established patterns for merge wiring that rebase replicates

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new libraries, all existing infrastructure
- Architecture: HIGH -- direct replication of Phase 39 merge pattern, verified by reading all source files
- Pitfalls: HIGH -- identified from actual code inspection (toast at L293, guard patterns, 7 surfaces enumerated)

**Research date:** 2026-03-21
**Valid until:** 2026-04-21 (stable -- no external dependencies)
