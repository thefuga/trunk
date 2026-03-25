---
phase: quick-260325-lkj
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/lib/store.ts
  - src/components/CommitGraph.svelte
autonomous: false
requirements: [fix-graph-column-width]
must_haves:
  truths:
    - "Graph column width matches content on first load for linear repos (~24px, not 120px)"
    - "Graph column width matches content on first load for branchy repos (wider, proportional to lane count)"
    - "Stored user preference is clamped to max on load — no jump when resizing"
    - "User can still manually resize the graph column within valid bounds"
  artifacts:
    - path: "src/lib/store.ts"
      provides: "Reduced default graph width and hasUserResized detection"
    - path: "src/components/CommitGraph.svelte"
      provides: "Auto-fit and clamp logic for graph column width"
  key_links:
    - from: "src/components/CommitGraph.svelte"
      to: "src/lib/store.ts"
      via: "getColumnWidths / setColumnWidths"
      pattern: "getColumnWidths|setColumnWidths"
---

<objective>
Fix graph column being too wide for linear histories and jumping on resize.

Purpose: The default 120px graph column is ~5x wider than needed for single-lane repos (16px natural width). When the user tries to resize, the column snaps because max is capped at naturalGraphWidth + padding (~24px), far below the rendered 120px. This creates a jarring UX.

Output: Graph column auto-fits to content width on load; stored width is clamped to the natural max; no visual jumps.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/lib/store.ts
@src/lib/graph-constants.ts
@src/components/CommitGraph.svelte
@src/components/CommitRow.svelte

<interfaces>
From src/lib/store.ts:
```typescript
export interface ColumnWidths {
  ref: number;
  graph: number;
  author: number;
  date: number;
  sha: number;
}
const DEFAULT_WIDTHS: ColumnWidths = { ref: 120, graph: 120, author: 120, date: 100, sha: 80 };
export async function getColumnWidths(): Promise<ColumnWidths>;
export async function setColumnWidths(widths: ColumnWidths): Promise<void>;
```

From src/lib/graph-constants.ts:
```typescript
export const LANE_WIDTH = 16;
export const COLUMN_PADDING_X = 4;
```

From CommitGraph.svelte (current state, lines 56-94):
```typescript
let maxColumns = $state(1);
let columnWidths = $state<ColumnWidths>({ ref: 120, graph: 120, author: 120, date: 100, sha: 80 });
const naturalGraphWidth = $derived(Math.max(maxColumns, 1) * displaySettings.laneWidth);
// maxColumns set by API response in loadMore() (line 693) and refresh() (line 741)
```
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Auto-fit graph column width to content and clamp stored width</name>
  <files>src/lib/store.ts, src/components/CommitGraph.svelte</files>
  <action>
**In `src/lib/store.ts`:**
1. Change `DEFAULT_WIDTHS.graph` from `120` to `24` (= 1 lane * 16 + 2 * 4 padding). This is the minimum sensible default for when no graph data is available yet. The actual width will be computed from content once data loads.

**In `src/components/CommitGraph.svelte`:**

1. Change the initial `columnWidths` state (line 64) to match the new store default: `graph: 24` instead of `graph: 120`.

2. Track whether the user has explicitly resized the graph column during this session. Add a boolean flag:
   ```typescript
   let userResizedGraph = false;
   ```

3. In `startColumnResize`, when column === 'graph', set `userResizedGraph = true` before the mousemove handler runs. This marks that the user intentionally chose a width.

4. Add an `$effect` that auto-fits the graph column width when `maxColumns` changes (i.e., after data loads). The logic:
   ```
   $effect(() => {
     // Read maxColumns to subscribe to changes
     const cols = maxColumns;
     const fitWidth = Math.max(cols, 1) * displaySettings.laneWidth + 2 * COLUMN_PADDING_X;
     // Only auto-fit if user hasn't manually resized this session
     if (!userResizedGraph) {
       columnWidths = { ...columnWidths, graph: fitWidth };
     } else {
       // Clamp stored width to max (prevent stored 120px exceeding natural max for 1-lane repo)
       const maxW = fitWidth;
       if (columnWidths.graph > maxW) {
         columnWidths = { ...columnWidths, graph: maxW };
       }
     }
   });
   ```
   This effect fires:
   - On initial load when `maxColumns` is set from the API response (loadMore line 693) -- auto-fits to content
   - On refresh when `maxColumns` may change (refresh line 741) -- re-fits or clamps
   - On loadMore when additional commits reveal more lanes -- grows to fit

5. In the existing `$effect` that loads stored column widths (line 77-79), after loading from store, also clamp the graph width. Since `naturalGraphWidth` depends on `maxColumns` which starts at 1, the initial clamp uses the minimal value. The auto-fit effect above handles the real clamping once data arrives. No change needed here -- the auto-fit effect will override.

6. The `startColumnResize` already has `maxWidths.graph = naturalGraphWidth + 2 * COLUMN_PADDING_X` which correctly caps resize dragging. No change needed there.

**Important:** The auto-fit `$effect` must access `maxColumns` directly (not through `naturalGraphWidth`) to ensure the effect tracks `maxColumns` reactivity. Use `displaySettings.laneWidth` inside the effect body for the calculation.
  </action>
  <verify>
    <automated>cd /Users/joaofnds/code/trunk && bun run check</automated>
  </verify>
  <done>
- Graph column starts at ~24px for linear repos (1 lane), not 120px
- Graph column grows proportionally for branchy repos (e.g., 3 lanes = 56px)
- Stored user preference is clamped to max width on load -- no snap/jump
- Manual resize still works within valid bounds (20px min to naturalWidth+padding max)
- svelte-check passes with no type errors
  </done>
</task>

<task type="checkpoint:human-verify" gate="blocking">
  <name>Task 2: Verify graph column width behavior visually</name>
  <files>n/a</files>
  <action>
Human verifies the graph column auto-fit and resize behavior across different repository types.
  </action>
  <verify>User confirms visual behavior matches expectations</verify>
  <done>User approves the graph column width behavior</done>
</task>

</tasks>

<verification>
- `bun run check` passes (TypeScript/Svelte type checking)
- Visual: linear repo graph column is narrow, not 120px wide
- Visual: resize handle moves smoothly without jumping
</verification>

<success_criteria>
Graph column width matches actual content (lane count * LANE_WIDTH + padding) on load. No visible jump when resizing. Stored preferences respected when within valid bounds.
</success_criteria>

<output>
After completion, create `.planning/quick/260325-lkj-fix-graph-column-width-too-wide-for-line/260325-lkj-SUMMARY.md`
</output>
