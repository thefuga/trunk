# Phase 39: Merge Workflow - Research

**Researched:** 2026-03-21
**Domain:** Svelte frontend context menu wiring, Tauri IPC, git merge semantics
**Confidence:** HIGH

## Summary

This phase is primarily a **frontend wiring task**. The backend Tauri commands (`merge_branch`, `merge_continue`, `merge_abort`) are already fully implemented, registered in `lib.rs`, and tested. The StagingPanel already handles merge-in-progress state with an editable commit message, operation banner, and Continue/Abort buttons. The CommitGraph already has a working `handleMergeBranch()` function and displays merge/rebase items in the commit context menu.

The work is: (1) Add "Merge [branch] into [current]" to the branch pill context menu in CommitGraph for both LocalBranch and RemoteBranch pills, (2) Add the same item to the BranchSidebar context menu for local branches, (3) Add context menu support for remote branches in BranchSidebar (currently none exists), (4) Remove the success toast from `handleMergeBranch` (per user decision: no toast on success), and (5) Ensure the overflow ref context menu also gets the merge item. The `showPillContextMenu` function currently only has Rename/Delete for LocalBranch and Delete for Tag -- no merge, and no RemoteBranch handling at all.

**Primary recommendation:** Wire existing `merge_branch` backend call to all branch context menus (sidebar local, sidebar remote, graph pill, graph overflow pill), using the same pattern as the commit context menu's existing merge item, and adjust toast behavior to match user decisions.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **No confirmation dialog** -- clicking "Merge" executes immediately
- **No toast on success** -- neither fast-forward nor merge commit shows a toast; graph refresh is sufficient visual feedback
- **Error toast for failures** -- git error message shown as red error toast
- **Let git decide** dirty workdir handling -- pass through to git CLI; show git's error as error toast if rejected
- **Banner + conflicts appear** for conflict flow -- reuses Phase 37 infrastructure entirely
- **Merge commit message editable in staging panel** -- pre-filled with standard message, user can edit before clicking Continue (already built)
- **"Merge [branch] into [current]"** -- fully explicit menu wording with both branch names
- **Both local and remote branches** get the merge context menu option
- **Hidden on HEAD branch** -- can't merge a branch into itself
- **No drag-and-drop** -- MERGE-02 dropped entirely from requirements
- **No auto-opening of first conflicted file** during conflict flow

### Claude's Discretion
- Exact menu item position in context menu (after Checkout recommended)
- Whether to add a separator before/after the Merge item
- Fast-forward detection approach (git output parsing vs rev-list check)
- Loading state during merge execution (disable menu item, spinner, etc.)

### Deferred Ideas (OUT OF SCOPE)
- MERGE-02 (drag-and-drop merge initiation) -- dropped entirely, not deferred
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| MERGE-01 | User can merge a branch into the current branch via right-click context menu on a branch (sidebar or graph pill) | Add merge item to `showBranchContextMenu` (BranchSidebar), `showPillContextMenu` (CommitGraph), and `showOverflowRefContextMenu` (CommitGraph). Backend `merge_branch` command already exists. |
| MERGE-02 | ~~Drag-and-drop merge~~ | **DROPPED** per user decision. No work needed. |
| MERGE-03 | Fast-forward merges advance the branch pointer without creating a merge commit | Already handled by `git merge branch --no-edit` (default git behavior). No special code needed -- git auto-detects FF-eligible merges. Per user decision: no toast on success. |
| MERGE-04 | Non-conflicting merges auto-create a merge commit with standard message and refresh the graph | Already handled by `git merge branch --no-edit`. Backend returns refreshed `GraphResult`, frontend receives `repo-changed` event. Per user decision: no toast on success. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @tauri-apps/api/menu | 2.x | Native context menus | Already used in all context menus across project |
| @tauri-apps/api/core | 2.x | IPC invoke | All backend calls go through `safeInvoke` wrapper |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/plugin-dialog | 2.x | Confirmation dialogs | Only for abort confirmation (already handled by Phase 37) |

No new dependencies needed. All required libraries are already installed and in use.

## Architecture Patterns

### Existing Context Menu Pattern
The project has a well-established pattern for native context menus:

```typescript
// Source: BranchSidebar.svelte lines 303-325
async function showBranchContextMenu(e: MouseEvent, branchName: string, isHead: boolean) {
  const { Menu, MenuItem, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
  const menu = await Menu.new({
    items: [
      await MenuItem.new({
        text: 'Checkout',
        enabled: !isHead,
        action: () => { handleCheckout(branchName); },
      }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      // ... more items
    ],
  });
  await menu.popup();
}
```

### Existing Merge Handler Pattern
CommitGraph already has a working merge handler:

```typescript
// Source: CommitGraph.svelte lines 280-292
async function handleMergeBranch(branch: string) {
  try {
    await safeInvoke('merge_branch', { path: repoPath, branch });
    showToast(`Merged ${branch}`, 'success');  // NOTE: Must be REMOVED per user decision
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Merge failed', 'error');
  }
}
```

### HEAD Branch Name Discovery

**In CommitGraph**: Derived from commits data (already done for commit context menu):
```typescript
// Source: CommitGraph.svelte lines 309-311
const headCommit = commits.find(c => c.is_head);
const headRef = headCommit?.refs.find(r => r.ref_type === 'LocalBranch' && r.is_head);
const headBranchName = headRef?.short_name;
```

**In BranchSidebar**: Available from the `refs` data:
```typescript
// refs.local contains BranchInfo[] where is_head marks the current branch
const headBranch = refs?.local.find(b => b.is_head);
const headBranchName = headBranch?.name;
```

### Integration Points Summary

| Location | Current State | What to Add |
|----------|---------------|-------------|
| `BranchSidebar.showBranchContextMenu()` | Checkout, Separator, Rename, Delete | Add Merge item (after Checkout, before separator) |
| `BranchSidebar` Remote branches | No context menu at all | Add `oncontextmenu` to RemoteGroup, create `showRemoteContextMenu` with Merge item |
| `CommitGraph.showPillContextMenu()` LocalBranch | Rename, Separator, Delete | Add Merge item at top (hidden when pill.isHead) |
| `CommitGraph.showPillContextMenu()` RemoteBranch | No handling (falls through) | Add RemoteBranch case with Merge item |
| `CommitGraph.showOverflowRefContextMenu()` LocalBranch | Rename, Separator, Delete | Add Merge item at top |
| `CommitGraph.showOverflowRefContextMenu()` RemoteBranch | No handling (falls through) | Add RemoteBranch case with Merge item |
| `CommitGraph.handleMergeBranch()` | Shows success toast | Remove success toast (keep error toast) |
| `CommitGraph.showCommitContextMenu()` | Already has merge/rebase items | Remove success toast (shared handler) |

### Remote Branch Name Formatting

For remote branches, `git merge` accepts the full remote ref name (e.g., `origin/feature-x`). The menu text should show the full name:
- In graph pills: `pill.label` is already the short_name like `origin/feature-x`
- In sidebar: RemoteGroup passes `remoteName + '/' + branch` as the full name

### Anti-Patterns to Avoid
- **Don't create a new merge handler in BranchSidebar** -- the merge logic (safeInvoke + error handling) should be consistent. Either extract to a shared util or duplicate the small handler.
- **Don't add fast-forward detection logic** -- git handles this automatically; `merge_branch_inner` uses `git merge branch --no-edit` which does FF by default.
- **Don't add confirmation dialogs** -- user explicitly decided against them.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Fast-forward detection | Pre-merge check via `git rev-list` or `git merge-base` | Let `git merge --no-edit` handle it | Git automatically does FF when possible; adding detection adds complexity for no benefit |
| Merge commit message for non-conflicting merges | Custom message dialog before merge | `--no-edit` flag on `git merge` | Non-conflicting merges auto-commit; message editing is only for conflict resolution flow |
| Operation state detection after merge | Manual file system checks | Existing `get_operation_state` + `repo-changed` event | Phase 37 infrastructure handles this automatically |

## Common Pitfalls

### Pitfall 1: Success Toast Inconsistency
**What goes wrong:** The existing `handleMergeBranch` in CommitGraph shows `showToast('Merged ${branch}', 'success')` on success. The user explicitly decided NO toast on success.
**Why it happens:** The handler was written before this decision was made.
**How to avoid:** Remove the success toast line from `handleMergeBranch`. Keep only the error toast in the catch block.
**Warning signs:** Toast appearing after successful merge.

### Pitfall 2: RemoteBranch Name for git merge
**What goes wrong:** Passing wrong ref name to `merge_branch` backend command for remote branches.
**Why it happens:** Remote branches in sidebar are displayed as just the branch name (e.g., `main`) within a remote group, but git needs the full ref `origin/main`.
**How to avoid:** In BranchSidebar, use `remoteName + '/' + branch` as the merge target. In CommitGraph pills, `pill.label` already contains the full short_name like `origin/main`.
**Warning signs:** Git error "merge: origin/main - not something we can merge" or similar.

### Pitfall 3: Merge During Existing Operation
**What goes wrong:** User right-clicks merge while already in a merge/rebase state.
**Why it happens:** No guard against starting merge during existing operation.
**How to avoid:** Git itself will reject `git merge` during an existing merge/rebase state and return an error. This will be caught by the error handling and shown as an error toast. No special guard needed in the frontend -- git's error message is clear.
**Warning signs:** Confusing error messages if git's stderr is not shown directly.

### Pitfall 4: HEAD Branch Not Available
**What goes wrong:** The menu item can't display "Merge X into [current]" because HEAD branch name is unknown (detached HEAD state).
**Why it happens:** When in detached HEAD state, there's no branch name to display.
**How to avoid:** In BranchSidebar, check if `refs?.local.find(b => b.is_head)` exists. In CommitGraph, the existing pattern (lines 309-311) returns undefined for `headBranchName` in detached HEAD. If no HEAD branch name, either hide the merge item or show "Merge X into HEAD".
**Warning signs:** Menu showing "Merge X into undefined".

### Pitfall 5: Duplicate Context Menu Handler Code
**What goes wrong:** BranchSidebar and CommitGraph each implement their own merge handler, leading to divergent behavior.
**Why it happens:** Both components need to call `merge_branch` but they're separate Svelte components.
**How to avoid:** The handler is small (5-6 lines). Duplicating is acceptable and matches existing patterns (both components already duplicate branch delete/rename logic). Keep them consistent.
**Warning signs:** Different error handling behavior between sidebar and graph merge actions.

## Code Examples

### Adding Merge to BranchSidebar Local Branch Menu
```typescript
// Source: Pattern from BranchSidebar.svelte showBranchContextMenu
async function showBranchContextMenu(e: MouseEvent, branchName: string, isHead: boolean) {
  const { Menu, MenuItem, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
  const headBranchName = refs?.local.find(b => b.is_head)?.name;
  const menu = await Menu.new({
    items: [
      await MenuItem.new({
        text: 'Checkout',
        enabled: !isHead,
        action: () => { handleCheckout(branchName); },
      }),
      // Merge item: hidden for HEAD branch, shown for all others
      ...((!isHead && headBranchName) ? [
        await MenuItem.new({
          text: `Merge ${branchName} into ${headBranchName}`,
          action: () => { handleMergeBranch(branchName).catch(() => {}); },
        }),
      ] : []),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({
        text: 'Rename...',
        action: () => { handleRenameBranch(branchName); },
      }),
      await MenuItem.new({
        text: 'Delete',
        enabled: !isHead,
        action: () => { handleDeleteBranch(branchName).catch(() => {}); },
      }),
    ],
  });
  await menu.popup();
}
```

### Adding Merge to Graph Pill Context Menu
```typescript
// Source: Pattern from CommitGraph.svelte showPillContextMenu
async function showPillContextMenu(e: MouseEvent, pill: OverlayRefPill) {
  e.preventDefault();
  e.stopPropagation();

  const headCommit = commits.find(c => c.is_head);
  const headRef = headCommit?.refs.find(r => r.ref_type === 'LocalBranch' && r.is_head);
  const headBranchName = headRef?.short_name;

  if (pill.refType === 'LocalBranch') {
    const mergeItems = (!pill.isHead && headBranchName) ? [
      await MenuItem.new({
        text: `Merge ${pill.label} into ${headBranchName}`,
        action: () => { handleMergeBranch(pill.label).catch(() => {}); },
      }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
    ] : [];
    const menu = await Menu.new({
      items: [
        ...mergeItems,
        await MenuItem.new({ text: 'Rename...', action: () => { handleRenameBranch(pill.label); } }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({ text: 'Delete', enabled: !pill.isHead, action: () => { handleDeleteBranch(pill.label).catch(() => {}); } }),
      ],
    });
    await menu.popup();
  } else if (pill.refType === 'RemoteBranch') {
    // Remote branches get merge only (can't rename/delete remote refs locally)
    if (headBranchName) {
      const menu = await Menu.new({
        items: [
          await MenuItem.new({
            text: `Merge ${pill.label} into ${headBranchName}`,
            action: () => { handleMergeBranch(pill.label).catch(() => {}); },
          }),
        ],
      });
      await menu.popup();
    }
  } else if (pill.refType === 'Tag') {
    // ... existing tag menu
  }
}
```

### Merge Handler (No Success Toast)
```typescript
// Source: Modified from CommitGraph.svelte handleMergeBranch
async function handleMergeBranch(branch: string) {
  try {
    await safeInvoke('merge_branch', { path: repoPath, branch });
    // No toast on success -- graph refresh via repo-changed event is sufficient
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Merge failed', 'error');
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No merge from UI | Merge via commit context menu (partial) | Phase 38 era | handleMergeBranch exists but only in commit context menu |
| No operation banner | Operation banner + conflict resolution | Phase 37-38 | Full merge conflict flow already works |

**Already built (no work needed):**
- `merge_branch` Tauri command (backend)
- `merge_continue` / `merge_abort` Tauri commands (backend)
- StagingPanel merge state detection (`isMerge` derived)
- StagingPanel merge message pre-fill and editing
- StagingPanel `commitMerge()` and `abortMerge()` functions
- OperationBanner for merge-in-progress display
- `repo-changed` event emission after merge operations
- Conflict detection and display in staging panel

## Open Questions

1. **Detached HEAD state behavior**
   - What we know: When HEAD is detached, there's no branch name. `headBranchName` will be undefined.
   - What's unclear: Should merge be available in detached HEAD? Git allows merging into detached HEAD.
   - Recommendation: Hide the merge item when no HEAD branch name is available. This matches the "Merge X into [current]" wording requirement -- if there's no branch name, the wording doesn't work. Users in detached HEAD can use the terminal.

2. **Loading state during merge**
   - What we know: Merge operations are typically fast but can take seconds for large repos.
   - What's unclear: How to indicate loading (the menu closes immediately after click).
   - Recommendation: No loading state needed. The menu closes on click, merge runs, and the graph refreshes via `repo-changed`. If it conflicts, the operation banner appears. If it fails, an error toast shows. This is consistent with the existing checkout behavior.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.0 (frontend), cargo test (backend) |
| Config file | vite.config.ts (vitest inherits) |
| Quick run command | `npm test` |
| Full suite command | `npm test && cd src-tauri && cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| MERGE-01 | Context menu calls merge_branch IPC | manual-only | N/A -- requires Tauri runtime for native menus | N/A |
| MERGE-03 | Fast-forward handled by git merge | unit (Rust) | `cd src-tauri && cargo test operation_state::tests -- --nocapture` | Existing tests cover merge_branch_inner |
| MERGE-04 | Non-conflicting merge auto-commits | unit (Rust) | `cd src-tauri && cargo test operation_state::tests -- --nocapture` | Existing tests cover merge conflict detection |

### Sampling Rate
- **Per task commit:** `npm test`
- **Per wave merge:** `npm test && cd src-tauri && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
None -- existing test infrastructure covers backend behavior. Frontend changes are context menu wiring (native Tauri menus) that cannot be unit-tested without the Tauri runtime. The existing Rust tests for `merge_branch_inner` and `operation_state` already validate the merge logic. Manual verification covers MERGE-01 (right-click menus), MERGE-03 (FF merges), and MERGE-04 (non-conflicting merges).

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/commands/operation_state.rs` -- Full backend implementation of merge_branch, merge_continue, merge_abort (lines 240-453)
- `src/components/BranchSidebar.svelte` -- Context menu pattern (lines 303-325)
- `src/components/CommitGraph.svelte` -- Existing handleMergeBranch (lines 280-292), showPillContextMenu (lines 452-483), showCommitContextMenu (lines 304-343)
- `src/components/StagingPanel.svelte` -- Merge state handling (lines 34-40, 196-222)
- `src/components/RemoteGroup.svelte` -- Remote branch rendering (no context menu)
- `src/lib/types.ts` -- OverlayRefPill, RefLabel, BranchInfo types

### Secondary (MEDIUM confidence)
- Git merge behavior with `--no-edit` flag -- well-documented default behavior (FF when possible, merge commit otherwise)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all libraries already in use, no new dependencies
- Architecture: HIGH - all patterns copied from existing code, integration points clearly identified
- Pitfalls: HIGH - based on direct code reading of all affected files

**Research date:** 2026-03-21
**Valid until:** 2026-04-21 (stable -- no external dependencies or version concerns)
