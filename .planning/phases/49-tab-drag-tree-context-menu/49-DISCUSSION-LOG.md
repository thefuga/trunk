# Phase 49: Tab Drag Reorder & Tree Context Menu - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 49-tab-drag-tree-context-menu
**Areas discussed:** Tab drag behavior, Tree context menu trigger, Tree context menu actions

---

## Tab drag behavior

### Drag visual feedback

| Option | Description | Selected |
|--------|-------------|----------|
| SortableJS swap | Reuse SortableJS (already in project). Tabs animate/swap in place. Minimal ghost. | ✓ |
| Native HTML drag | Browser-native drag with ghost image. Drop indicator line. No extra dependency. | |

**User's choice:** SortableJS swap
**Notes:** Already a project dependency used in RebaseEditor. Consistent DnD behavior.

### Edge auto-scroll

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, auto-scroll | When dragging near left/right edge, tab bar scrolls to reveal more tabs. | ✓ |
| No auto-scroll | User must scroll manually before/after dragging. | |

**User's choice:** Yes, auto-scroll
**Notes:** Essential for overflow scenarios with many tabs open.

### New tab button position

| Option | Description | Selected |
|--------|-------------|----------|
| Pinned at end | + button excluded from SortableJS, always stays at far right. | ✓ |

**User's choice:** Pinned at end
**Notes:** Standard browser behavior.

### Drag styling

| Option | Description | Selected |
|--------|-------------|----------|
| Subtle opacity | Dragged tab gets opacity reduction. Drop slot shows accent-colored line. | |
| Background highlight | Drop slot gets subtle background highlight instead of line. | |
| You decide | Claude picks appropriate styling. | ✓ |

**User's choice:** You decide
**Notes:** Claude has discretion on drag styling using existing CSS custom properties.

---

## Tree context menu trigger

### Menu target

| Option | Description | Selected |
|--------|-------------|----------|
| Directories only | Right-click on directory nodes shows bulk actions. Files keep existing per-file context menu. | ✓ |
| Both dirs and files | Unified context menu on all tree nodes. | |

**User's choice:** Directories only
**Notes:** Clean separation — directories get bulk actions, files keep existing StagingPanel menus.

### Menu type

| Option | Description | Selected |
|--------|-------------|----------|
| Native Tauri | Consistent with all other context menus. Uses @tauri-apps/api/menu. | ✓ |

**User's choice:** Native Tauri
**Notes:** Consistent with graph rows, branches, tabs, file rows.

---

## Tree context menu actions

### Unstaged section actions

| Option | Description | Selected |
|--------|-------------|----------|
| Stage All + Discard All | Stage All stages all files. Discard All discards all changes (with confirmation). | ✓ |
| Stage All only | Only staging. Discard per-file only. | |

**User's choice:** Stage All + Discard All
**Notes:** Matches the two main operations for unstaged files.

### Discard confirmation

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, always confirm | Destructive operation — show confirmation dialog. Consistent with single-file discard. | ✓ |

**User's choice:** Yes, always confirm
**Notes:** Consistent with existing discard confirmation pattern.

### Conflicted section actions

| Option | Description | Selected |
|--------|-------------|----------|
| Resolve All + Unresolve All | Resolve marks all as resolved (stages). Unresolve marks back as conflicted. Per TREE-11 requirement. | ✓ |

**User's choice:** Resolve All + Unresolve All
**Notes:** Per TREE-11 requirement spec.

### Staged section actions

| Option | Description | Selected |
|--------|-------------|----------|
| Unstage All | Single action: unstages all files in the directory recursively. | ✓ |

**User's choice:** Unstage All
**Notes:** No discard from staged section — files are already tracked.

---

## Claude's Discretion

- Drag styling details (opacity, cursor, drop indicator visuals)
- SortableJS configuration (animation duration, ghost class, handle)
- Directory context menu handler wiring through TreeFileList/DirectoryRow
- Sequential vs parallel IPC calls for bulk directory operations

## Deferred Ideas

None — discussion stayed within phase scope.
