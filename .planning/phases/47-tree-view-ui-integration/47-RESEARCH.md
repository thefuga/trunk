# Phase 47: Tree View UI Integration - Research

**Researched:** 2026-03-24
**Domain:** Svelte 5 UI components, tree rendering, keyboard navigation, persisted preferences
**Confidence:** HIGH

## Summary

This phase wires the existing `buildTree()` utility (Phase 46) into three file list contexts: StagingPanel (unstaged, staged, conflicted sections), CommitDetail (commit diff file list), and MergeEditor (conflict file selector). The core challenges are: (1) creating a reusable tree rendering system that works with both `FileStatus[]` and `FileDiff[]` data types, (2) managing per-section expand/collapse state that survives reactive data refreshes, (3) implementing VS Code-style keyboard navigation over a flattened visible-row model, and (4) persisting the flat/tree toggle preference via LazyStore.

All required building blocks exist in the codebase: `buildTree()` for data transformation, `FileRow.svelte` for leaf node rendering, `ChevronDown`/`ChevronRight` from `@lucide/svelte` for directory indicators, the LazyStore pattern in `src/lib/store.ts` for persistence, and established Svelte 5 rune patterns (`$state`, `$derived`, `$effect`) for reactive state management.

**Primary recommendation:** Build a `flattenTree()` utility that converts `TreeNode[]` + expanded `Set<string>` into a flat array of renderable rows (with depth info), then render that flat array with conditional indentation. This avoids recursive Svelte `{#each}` blocks and makes keyboard navigation trivial (index into flat array).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Single global toggle in the staging panel header ("N files changed on main" bar). One icon button (list/tree) controls the view mode for all file list contexts -- staging panel sections, commit diff file lists, and merge editor file lists.
- **D-02:** View mode is a single persisted boolean (`tree_view_enabled`) in LazyStore. Toggle once, see tree view everywhere. Restored on app relaunch.
- **D-03:** The toggle applies uniformly -- unstaged, staged, conflicted, commit diff file lists, and merge editor file lists all respect the same setting.
- **D-04:** Indent + chevron only. Each nesting level adds ~16px left padding. Directories show a chevron (ChevronRight collapsed / ChevronDown expanded) before the directory name. Files show their existing status icon at the same indentation depth. No folder icons, no connector lines.
- **D-05:** In tree mode, files show filename only (e.g. "store.ts"), not the full relative path. The directory structure provides path context. Flat mode continues showing the full relative path as today.
- **D-06:** Directory rows have the same 26px height as FileRow. Clicking the row or chevron toggles expand/collapse.
- **D-07:** Expand/collapse state is per-section (unstaged, staged, conflicted each track their own `Set<string>` of expanded directory paths), keyed by the directory's full relative path. This allows independent tree navigation in each section.
- **D-08:** On status refresh (repo-changed event, stage/unstage operations), the expanded set is preserved -- only the tree data re-derives from the new `FileStatus[]` array. This satisfies TREE-05 (state survives refreshes).
- **D-09:** When toggling from flat to tree mode, all directories start **collapsed**. User expands to drill in.
- **D-10:** Expand/collapse state is ephemeral (not persisted to disk). Switching tabs and back preserves it via keep-alive (Phase 45 D-08), but closing and reopening the app resets it.
- **D-11:** VS Code-style arrow key navigation. Up/Down moves focus between visible rows. Right on collapsed dir: expand. Right on expanded dir: move to first child. Left on file: jump to parent dir. Left on expanded dir: collapse. Left on collapsed dir: jump to parent. Enter on file: select (show diff). Enter on dir: no-op.
- **D-12:** Keyboard focus shows a visible highlight on the focused row (subtle background, similar to hover state). Active in both flat and tree modes.
- **D-13:** Focus is tracked per-section as an index into the visible (flattened) row list. Focus resets to 0 when the file list changes (status refresh), unless the previously focused path still exists.

### Claude's Discretion
- How to structure the TreeView component (single component wrapping FileRow, or new DirectoryRow + TreeList)
- Whether to create a shared `useTreeState` hook/factory or inline the state in each consumer
- How to render tree in DiffPanel and MergeEditor (these show `FileDiff[]` not `FileStatus[]` -- may need a lightweight adapter to map FileDiff paths into buildTree-compatible input)
- Animation on expand/collapse (subtle slide or instant)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TREE-01 | User can toggle between flat file list and directory tree view in staging panel | Toggle button in panel header (D-01), LazyStore persistence (D-02), `buildTree()` for data transformation, `flattenTree()` for rendering |
| TREE-02 | User can toggle between flat file list and directory tree view in commit diffs | CommitDetail file list uses same toggle signal, adapter maps `FileDiff[]` to `FileStatus[]` for `buildTree()` |
| TREE-03 | User can toggle between flat file list and directory tree view in merge editor | MergeEditor conflict file selector (StagingPanel conflicted section) already covered by TREE-01; standalone MergeEditor has no separate file list to tree-ify |
| TREE-04 | Directory nodes expand/collapse with chevron indicators | `ChevronRight`/`ChevronDown` icons, 26px row height, click handler toggles `Set<string>` |
| TREE-05 | Expand/collapse state is preserved across status refreshes | Expanded `Set<string>` is separate from tree data; tree re-derives on status change but Set persists (D-08) |
| TREE-06 | User can navigate the tree with arrow keys | `flattenTree()` produces indexed flat array; keyboard handler uses index for Up/Down/Left/Right/Enter (D-11) |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Never inline colors** -- always use CSS custom properties from the theme
- **Never fight layout with positioning hacks** -- use grid/flexbox so elements flow naturally
- **All git operations go through git2 crate** -- no shelling out
- **Svelte 5 runes** -- all state uses `$state()`, `$derived()`, `$effect()`
- **CSS custom properties must use semantic names** (e.g. `--color-focus-ring`, not `--color-blue`)
- **`@lucide/svelte`** not `lucide-svelte` for Svelte 5 compatibility
- **Immutable state patterns** -- mutating collections in place does not trigger Svelte 5 updates; use `new Set(...)`, spread, etc.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte 5 | current | Component framework with runes ($state, $derived, $effect) | Project standard |
| @lucide/svelte | 0.577.0 | Icon library -- ChevronRight, ChevronDown, List, FolderTree | Already used across all components |
| @tauri-apps/plugin-store | current | LazyStore for tree_view_enabled persistence | Established persistence pattern |
| TypeScript 5.6 strict | current | Type safety for TreeNode, FlatRow types | Project standard |

### Supporting
No new dependencies needed. Everything required is already in the project.

## Architecture Patterns

### Recommended Component Structure

```
src/
├── lib/
│   ├── build-tree.ts          # EXISTING: buildTree() → TreeNode[]
│   ├── flatten-tree.ts        # NEW: flattenTree() → FlatRow[], keyboard nav helpers
│   ├── flatten-tree.test.ts   # NEW: unit tests for flattenTree
│   ├── store.ts               # MODIFY: add tree_view_enabled persistence
│   └── types.ts               # EXISTING: FileStatus, FileDiff types
├── components/
│   ├── TreeFileList.svelte     # NEW: renders flat/tree file list with keyboard nav
│   ├── DirectoryRow.svelte     # NEW: 26px directory row with chevron + name
│   ├── FileRow.svelte          # MODIFY: accept optional indentation depth prop
│   ├── StagingPanel.svelte     # MODIFY: replace {#each} blocks with TreeFileList
│   ├── CommitDetail.svelte     # MODIFY: replace file list with TreeFileList
│   └── DiffPanel.svelte        # NO CHANGE: DiffPanel renders diffs, not file lists
```

### Pattern 1: Flatten-then-render (FlatRow approach)

**What:** Convert the nested `TreeNode[]` tree into a flat `FlatRow[]` array that includes depth, type (file/directory), expanded state, and the original data reference. Render with a single `{#each flatRows as row}` block.

**When to use:** Always for tree views that need keyboard navigation. Recursive `{#each}` blocks in Svelte make focus tracking extremely difficult.

**Why:** Keyboard navigation (TREE-06) requires knowing "what's the next visible row?" and "what's the parent of this row?". A flat array with indices makes this O(1). A nested tree requires traversal.

```typescript
// src/lib/flatten-tree.ts

import type { TreeNode, DirectoryNode, FileNode } from './build-tree.js';
import type { FileStatus } from './types.js';

export interface FlatFileRow {
  type: 'file';
  depth: number;
  node: FileNode;
  parentPath: string | null;  // for Left-arrow → jump to parent
}

export interface FlatDirRow {
  type: 'directory';
  depth: number;
  node: DirectoryNode;
  expanded: boolean;
  parentPath: string | null;
}

export type FlatRow = FlatFileRow | FlatDirRow;

/**
 * Flatten a TreeNode[] into a renderable row array,
 * respecting expand/collapse state.
 *
 * Only expanded directories have their children included.
 * Depth is used for indentation (depth * 16px).
 */
export function flattenTree(
  nodes: TreeNode[],
  expanded: Set<string>,
  depth: number = 0,
  parentPath: string | null = null,
): FlatRow[] {
  const result: FlatRow[] = [];
  for (const node of nodes) {
    if (node.type === 'directory') {
      const isExpanded = expanded.has(node.path);
      result.push({
        type: 'directory',
        depth,
        node,
        expanded: isExpanded,
        parentPath,
      });
      if (isExpanded) {
        result.push(
          ...flattenTree(node.children, expanded, depth + 1, node.path)
        );
      }
    } else {
      result.push({
        type: 'file',
        depth,
        node,
        parentPath,
      });
    }
  }
  return result;
}
```

### Pattern 2: Shared tree state factory

**What:** A factory function `createTreeSectionState()` that encapsulates the expand/collapse `Set<string>` and focus index for one section.

**When to use:** Each section (unstaged, staged, conflicted, commit files) needs its own independent expand/collapse and focus state.

```typescript
// Could live in flatten-tree.ts or a separate file

export interface TreeSectionState {
  expanded: Set<string>;
  focusIndex: number;
}

// In components, use $state() directly:
let unstagedTree = $state<TreeSectionState>({
  expanded: new Set(),
  focusIndex: 0,
});
```

### Pattern 3: FileDiff-to-FileStatus adapter for CommitDetail

**What:** CommitDetail receives `FileDiff[]` but `buildTree()` expects `FileStatus[]`. A lightweight mapping function converts between them.

```typescript
// In CommitDetail.svelte or flatten-tree.ts
import type { FileDiff, FileStatus, FileStatusType } from './types.js';

const DIFF_STATUS_MAP: Record<string, FileStatusType> = {
  Added: 'New',
  Deleted: 'Deleted',
  Modified: 'Modified',
  Renamed: 'Renamed',
  Copied: 'Modified',
  Untracked: 'New',
  Unknown: 'Modified',
};

export function fileDiffToFileStatus(fd: FileDiff): FileStatus {
  return {
    path: fd.path,
    status: DIFF_STATUS_MAP[fd.status] ?? 'Modified',
    is_binary: fd.is_binary,
  };
}
```

### Pattern 4: Global toggle via LazyStore + reactive signal

**What:** The `tree_view_enabled` boolean is loaded from LazyStore on mount and exposed as a reactive `$state` in a parent component. Passed as a prop to all consumers.

**How it flows:** `App.svelte` loads preference on init, passes to `RepoView`, which passes to `StagingPanel`, `CommitDetail`. The toggle button in StagingPanel header updates the store and the reactive signal.

Alternative (simpler): StagingPanel owns the toggle state since the toggle button lives in its header. It loads from/saves to LazyStore. CommitDetail reads from the same store independently. Since LazyStore is backed by a file, both read the same value. On toggle, StagingPanel updates the store and its local state; CommitDetail can poll on mount or the toggle can be prop-drilled from RepoView.

**Recommended approach:** Pass `treeViewEnabled` as a prop from RepoView (which loads it from store on mount). This ensures all consumers see the same value synchronously without duplicate store reads.

### Anti-Patterns to Avoid
- **Recursive `{#each}` for tree rendering:** Makes keyboard navigation index tracking extremely difficult. Always flatten first.
- **Mutating the expanded Set in place:** Svelte 5 requires `expanded = new Set([...expanded, path])` not `expanded.add(path)`. The retrospective explicitly calls this out.
- **Inlining colors for focus highlight:** Must use CSS custom properties (`var(--color-tree-focus-bg)` or reuse `var(--color-surface)`).
- **Coupling tree state to tree data:** The expanded `Set<string>` must be independent of the `TreeNode[]` data. When status refreshes, only the tree data rebuilds; the set persists. This is key to TREE-05.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Flat-to-tree conversion | Custom recursive parser | `buildTree()` from Phase 46 | Already handles path compression, sorting, edge cases |
| Icons (chevrons, list, folder-tree) | SVG strings | `@lucide/svelte` ChevronRight, ChevronDown, List, FolderTree | Consistent with rest of codebase, tree-shakeable |
| Preference persistence | localStorage / custom file | LazyStore from `@tauri-apps/plugin-store` | Established pattern in store.ts, works with Tauri sandboxing |
| Status icons for files | New icon set | Existing `FileRow.svelte` | Already renders file status icons correctly |

**Key insight:** The tree view is primarily a rendering/interaction layer. The data layer (buildTree) already exists. The challenge is making the UI responsive to expand/collapse state while preserving that state across reactive updates.

## Common Pitfalls

### Pitfall 1: Expanded state lost on status refresh
**What goes wrong:** Status refresh triggers `status = result` which re-derives the tree, and if the expanded Set is derived from or coupled to the tree data, it resets.
**Why it happens:** Natural instinct is to store expanded state inside tree nodes, which means new tree = new state.
**How to avoid:** Keep expanded `Set<string>` completely separate from tree data. Tree nodes don't have expanded state -- the flatten function takes the external Set as a parameter.
**Warning signs:** Staging a file causes the tree to collapse back.

### Pitfall 2: Keyboard focus index stale after data change
**What goes wrong:** User focuses row 5, then a file gets staged (removed from unstaged list). Row 5 now points to a different file or doesn't exist.
**Why it happens:** Focus is tracked as an index into a flat array that can change length.
**How to avoid:** After data change, try to find the previously focused path in the new flat array. If found, update index. If not found, clamp to valid range (D-13).
**Warning signs:** Focus jumps to wrong file after stage/unstage.

### Pitfall 3: Svelte 5 immutable update pattern for Set
**What goes wrong:** `expanded.add(path)` doesn't trigger reactivity. UI doesn't update.
**Why it happens:** Svelte 5 runes track assignments, not mutations.
**How to avoid:** Always create new Set: `expanded = new Set([...expanded, path])` for add, or `const next = new Set(expanded); next.delete(path); expanded = next;` for delete.
**Warning signs:** Click on chevron does nothing visually.

### Pitfall 4: DiffPanel tree confusion
**What goes wrong:** Trying to add tree view to DiffPanel's file header bars (the `{#each fileDiffs as fd}` that renders sticky file headers with collapse triangles).
**Why it happens:** DiffPanel already has its own file-level collapse for showing/hiding hunks. Adding tree view here conflicts.
**How to avoid:** TREE-02 is about the commit diff **file list** (in CommitDetail.svelte's sidebar), NOT about DiffPanel's inline file headers. DiffPanel itself does not need tree view.
**Warning signs:** Attempting to add tree rendering inside DiffPanel.

### Pitfall 5: MergeEditor scope confusion
**What goes wrong:** Trying to add tree view to MergeEditor's internal panels (ours/theirs/output).
**Why it happens:** TREE-03 says "merge editor file list" but MergeEditor works on a single file at a time.
**How to avoid:** TREE-03 is satisfied by the **StagingPanel's conflicted files section** (which already renders `status?.conflicted` via `FileRow`). That's the "merge editor file list" -- the file selector for which file to open in the merge editor. The MergeEditor component itself has no file list to tree-ify.
**Warning signs:** Trying to modify MergeEditor.svelte template.

### Pitfall 6: Toggle button placement conflict
**What goes wrong:** Adding toggle button to the staging panel header might conflict with the "N files changed on main" text and branch badge when space is tight.
**Why it happens:** The header is already packed with content (file count, "on", branch pill).
**How to avoid:** The toggle icon button should be small (16px icon, no text) placed at the right edge of the header bar. It's a visual indicator, not a labeled button.
**Warning signs:** Header text truncated or layout breaks at narrow widths.

## Code Examples

### LazyStore persistence pattern (verified from existing store.ts)

```typescript
// Add to src/lib/store.ts
const TREE_VIEW_KEY = 'tree_view_enabled';

export async function getTreeViewEnabled(): Promise<boolean> {
  return (await store.get<boolean>(TREE_VIEW_KEY)) ?? false;
}

export async function setTreeViewEnabled(enabled: boolean): Promise<void> {
  await store.set(TREE_VIEW_KEY, enabled);
  await store.save();
}
```

### DirectoryRow component pattern

```svelte
<!-- DirectoryRow.svelte -->
<script lang="ts">
  import { ChevronDown, ChevronRight } from '@lucide/svelte';
  import type { DirectoryNode } from '../lib/build-tree.js';

  interface Props {
    node: DirectoryNode;
    depth: number;
    expanded: boolean;
    focused: boolean;
    ontoggle: () => void;
  }

  let { node, depth, expanded, focused, ontoggle }: Props = $props();
</script>

<div
  role="treeitem"
  aria-expanded={expanded}
  onclick={ontoggle}
  style="
    height: 26px;
    padding: 0 8px;
    padding-left: {8 + depth * 16}px;
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    background: {focused ? 'var(--color-tree-focus)' : 'transparent'};
    color: var(--color-text);
    font-size: 12px;
  "
>
  <span style="display: inline-flex; align-items: center; color: var(--color-text-muted); width: 12px; min-width: 12px;">
    {#if expanded}
      <ChevronDown size={12} />
    {:else}
      <ChevronRight size={12} />
    {/if}
  </span>
  <span style="
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  ">{node.name}</span>
</div>
```

### Keyboard navigation handler pattern

```typescript
// In a component managing a tree section
function handleTreeKeydown(e: KeyboardEvent, flatRows: FlatRow[], focusIndex: number): number {
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      return Math.min(focusIndex + 1, flatRows.length - 1);

    case 'ArrowUp':
      e.preventDefault();
      return Math.max(focusIndex - 1, 0);

    case 'ArrowRight': {
      e.preventDefault();
      const row = flatRows[focusIndex];
      if (row?.type === 'directory') {
        if (!row.expanded) {
          // Expand this directory
          toggleExpanded(row.node.path);
          return focusIndex;
        } else {
          // Move to first child
          return Math.min(focusIndex + 1, flatRows.length - 1);
        }
      }
      return focusIndex;
    }

    case 'ArrowLeft': {
      e.preventDefault();
      const row = flatRows[focusIndex];
      if (row?.type === 'directory' && row.expanded) {
        // Collapse this directory
        toggleExpanded(row.node.path);
        return focusIndex;
      }
      // Jump to parent directory
      if (row?.parentPath) {
        const parentIdx = flatRows.findIndex(
          r => r.type === 'directory' && r.node.path === row.parentPath
        );
        if (parentIdx >= 0) return parentIdx;
      }
      return focusIndex;
    }

    case 'Enter': {
      const row = flatRows[focusIndex];
      if (row?.type === 'file') {
        // Trigger file selection
        onfileselect(row.node.file.path);
      }
      return focusIndex;
    }
  }
  return focusIndex;
}
```

### Immutable Set update pattern (Svelte 5)

```typescript
// Toggle a directory path in the expanded set
function toggleExpanded(path: string) {
  const next = new Set(expanded);
  if (next.has(path)) {
    next.delete(path);
  } else {
    next.add(path);
  }
  expanded = next;  // Assignment triggers reactivity
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Recursive Svelte {#each} for trees | Flatten then render flat array | Svelte 5 (runes era) | Keyboard nav possible, better perf |
| `lucide-svelte` | `@lucide/svelte` | ~2024 | Must use new package for Svelte 5 compat |
| Svelte stores (writable) | Svelte 5 runes ($state) | Svelte 5 | Project uses runes exclusively |
| `expanded.add()` mutation | `expanded = new Set([...expanded, path])` | Svelte 5 | Required for reactivity |

## Open Questions

1. **Where does the toggle state originate?**
   - What we know: D-01 says toggle in StagingPanel header. D-02 says persisted in LazyStore. CommitDetail also needs to read it.
   - What's unclear: Does RepoView load and prop-drill, or does each consumer read from store independently?
   - Recommendation: RepoView loads on mount, passes as prop. StagingPanel header has the toggle button that updates both the prop (via callback) and the store. This keeps consumers in sync without multiple store reads.

2. **TREE-03 scope for MergeEditor**
   - What we know: MergeEditor works on a single file at a time. The "file list" for merge is actually the conflicted files section in StagingPanel.
   - What's unclear: Does TREE-03 require any changes to MergeEditor.svelte itself?
   - Recommendation: No. TREE-03 is fully satisfied by applying tree view to StagingPanel's conflicted files section. MergeEditor.svelte requires no modifications.

3. **Animation on expand/collapse (Claude's Discretion)**
   - What we know: D-04 specifies chevron toggle. No explicit decision on animation.
   - Recommendation: **Instant** (no animation). Matches the existing StagingPanel section expand/collapse behavior (unstaged_expanded, staged_expanded chevrons toggle instantly). Consistency is more important than polish here. Animation adds complexity with the flatten-then-render approach.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest (latest, via vite config) |
| Config file | vite.config.ts (test section) |
| Quick run command | `bun run test` |
| Full suite command | `bun run test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TREE-01 | Toggle between flat/tree in staging panel | manual | Visual verification in dev mode | N/A |
| TREE-02 | Toggle between flat/tree in commit diffs | manual | Visual verification in dev mode | N/A |
| TREE-03 | Toggle between flat/tree in merge editor | manual | Visual verification in dev mode | N/A |
| TREE-04 | Directory expand/collapse with chevrons | unit + manual | `bun run test -- flatten-tree` | Wave 0 |
| TREE-05 | State preserved across refreshes | unit | `bun run test -- flatten-tree` | Wave 0 |
| TREE-06 | Arrow key navigation | unit | `bun run test -- flatten-tree` | Wave 0 |

### Sampling Rate
- **Per task commit:** `bun run test`
- **Per wave merge:** `bun run test && bun run check`
- **Phase gate:** Full suite green + visual verification of all 6 requirements

### Wave 0 Gaps
- [ ] `src/lib/flatten-tree.test.ts` -- covers TREE-04, TREE-05, TREE-06 (flattenTree logic, keyboard nav helpers)
- [ ] Framework install: None needed, vitest already configured

## Sources

### Primary (HIGH confidence)
- **Codebase inspection** -- Direct reading of all integration point files (StagingPanel.svelte, FileRow.svelte, DiffPanel.svelte, MergeEditor.svelte, CommitDetail.svelte, RepoView.svelte, store.ts, build-tree.ts, types.ts)
- **Phase 46 output** -- build-tree.ts and build-tree.test.ts verified as complete and working
- **CONTEXT.md** -- 13 locked decisions from user discussion
- **RETROSPECTIVE.md** -- Svelte 5 immutable patterns lesson, lucide-svelte package lesson

### Secondary (MEDIUM confidence)
- **Lucide icon availability** -- Verified `List`, `FolderTree`, `ChevronRight`, `ChevronDown` all exist in @lucide/svelte 0.577.0 dist

### Tertiary (LOW confidence)
- None -- all findings verified against codebase

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, all verified in codebase
- Architecture: HIGH -- flatten-then-render is well-established pattern, all integration points read and understood
- Pitfalls: HIGH -- Svelte 5 reactivity pitfalls documented in project retrospective, scope confusion risks identified from reading actual component code

**Research date:** 2026-03-24
**Valid until:** 2026-04-24 (stable codebase, no framework migrations planned)
