# Quick Task 260325-lkj: Fix graph column width — Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Task Boundary

Fix the graph column being too wide for repos with linear commit histories (single lane, no branches). The default 120px is ~5x wider than the natural 24px needed. When users try to resize, the column jumps left because the max constraint (24px) is far less than the rendered width.

</domain>

<decisions>
## Implementation Decisions

### Initial Width Strategy
- Auto-fit to content: set initial width to `naturalGraphWidth + padding`
- Linear repos get ~24px, branchy repos get what they need
- Stored user preference only applies if the user has manually resized

### Max Width Cap
- Cap at natural width: max = `naturalGraphWidth + 2 * COLUMN_PADDING_X`
- Resize handle stops at the content edge — no wasted space allowed
- Prevents the "too wide" problem entirely

### Resize Jump Behavior
- Clamp on load: immediately set `width = min(storedWidth, maxWidth)` when graph data arrives
- No visible jump or snap — column renders at correct size from the start

### Claude's Discretion
- Whether to persist the clamped width back to the store or keep the stored value for repos that might need more space later

</decisions>

<specifics>
## Specific Ideas

- `naturalGraphWidth = max(maxColumns, 1) * LANE_WIDTH` (already computed)
- `maxWidth = naturalGraphWidth + 2 * COLUMN_PADDING_X`
- Default width should be derived from content, not a fixed 120px constant
- Key files: `CommitGraph.svelte` (resize logic, width state), `store.ts` (column width persistence), `graph-constants.ts` (LANE_WIDTH, COLUMN_PADDING_X)

</specifics>
