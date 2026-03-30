# Phase 64: Split View - Research

**Researched:** 2026-03-30
**Domain:** Side-by-side diff rendering, scroll synchronization, resizable panels, ViewMode refactoring
**Confidence:** HIGH

## Summary

Phase 64 implements the split (side-by-side) diff view. The core work is (1) refactoring the 3-way `ViewMode` type into two orthogonal toggles (content: hunk/full, layout: inline/split), (2) building SplitView.svelte with paired-row alignment and phantom spacers, (3) synchronized scrolling between the two panels, and (4) hunk/line staging support in the new layout.

The codebase already contains all the rendering patterns needed -- HunkView and FullFileView have line rendering, gutter columns, merged spans, syntax classes, and word-diff emphasis that SplitView will replicate per-panel. The RespoView already has a drag-to-resize pane divider pattern that can be adapted. No new libraries are needed; this is purely a frontend Svelte component and type refactoring phase.

**Primary recommendation:** Split into 3 plans: (P01) ViewMode refactor + toolbar, (P02) SplitView component with row alignment + scroll sync, (P03) staging interactions in split view. This keeps each plan focused and testable.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Split is a layout dimension (inline vs side-by-side), independent of the content dimension (hunk vs full). Produces 4 combinations.
- D-02: Refactor ViewMode from 3-way ("hunk" | "full" | "split") into two separate state values -- content mode (hunk/full) and layout mode (inline/split). Both persist independently via LazyStore.
- D-03: DiffViewer dispatches rendering based on both dimensions. SplitView receives contentMode.
- D-04: Toolbar layout: [Hunk|Full] | [Inline|Split] | WS P Return | file.ts | Stage x. Two segmented controls side by side with visual separator.
- D-05: Inline/split segmented control uses Lucide icons. Both toggles are always visible and independent.
- D-06: Paired rows with phantom spacers. Delete on left matched with Add on right. Blank phantom rows on shorter side.
- D-07: Phantom rows are empty rows with slightly muted background (CSS custom property). Clean and minimal.
- D-08: Resizable divider between two panels. Draggable handle lets users resize the split.
- D-09: Single gutter per panel -- left panel shows old line numbers only, right panel shows new line numbers only.
- D-10: No origin symbols (+/-/space) in split view. Color backgrounds indicate change type.
- D-11: Scrolling either panel scrolls both in sync. Locked sync, no independent scroll option.
- D-12: In split+hunk mode: hunk headers span full width of both panels. Stage/Discard/Unstage buttons on right.
- D-13: In split+full mode: no hunk headers, continuous document on both sides.
- D-14: Hunk-level actions appear in hunk header row spanning both panels.
- D-15: Line selection works on right (new) panel only. Shift+click range selection. Stage Lines/Discard Lines buttons in hunk header.
- D-16: All staging disabled when whitespace ignore is active.

### Claude's Discretion
- Exact Lucide icons for inline/split toggle buttons
- CSS custom property name and value for phantom row muted background
- Internal data structure for paired row alignment
- Drag handle implementation for resizable divider
- Scroll sync implementation approach

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VIEW-02 | Split view shows old content on left, new on right, with phantom/spacer rows for alignment | Row pairing algorithm, phantom row CSS, single-gutter-per-panel pattern |
| VIEW-03 | Split view panels scroll in sync (locked) | Scroll event listener pattern with recursion guard |
| VIEW-05 | User can stage/unstage/discard hunks and lines in all view modes (disabled when whitespace ignore is active) | Existing staging callback interface, line selection on right panel only, hunk header spanning both panels |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Never inline colors -- always use CSS custom properties from the theme
- Never fight layout with positioning hacks -- use grid/flexbox so elements flow naturally
- All git operations go through git2 crate, no shelling out
- Run `just check` before every commit and push
- Svelte 5 runes ($state, $derived, $effect) for reactive state
- TypeScript 5.6 strict mode
- Tailwind CSS 4 (but diff components use inline styles, not Tailwind classes)

## Standard Stack

No new libraries needed. Everything is built with existing dependencies.

### Core (existing, no changes)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte 5 | 5.x | Component framework | Already used; runes for reactive state |
| TypeScript | 5.6 | Type safety | Already used; strict mode |
| @lucide/svelte | installed | Icons | Already used in DiffToolbar |
| @tauri-apps/plugin-store | installed | LazyStore persistence | Already used for diff preferences |

### Supporting (no additions)
No new libraries. The row alignment algorithm, scroll sync, and resizable divider are all implemented with vanilla DOM APIs.

## Architecture Patterns

### Current Code Structure (before refactoring)
```
src/
  components/
    DiffPanel.svelte          # State owner: ViewMode, preferences, staging handlers
    diff/
      DiffToolbar.svelte      # 3-way segmented control (Hunk|Full|Split)
      DiffViewer.svelte       # Dispatches to HunkView, FullFileView, or SplitView
      HunkView.svelte         # Inline hunk rendering with staging
      FullFileView.svelte     # Inline continuous rendering (no staging)
      SplitView.svelte        # Stub ("coming soon")
  lib/
    types.ts                  # ViewMode = "hunk" | "full" | "split"
    store.ts                  # getDiffViewMode/setDiffViewMode (single key)
    diff-utils.ts             # splitInvisibles, trailingWhitespaceStart
```

### After Phase 64
```
src/
  components/
    DiffPanel.svelte          # Refactored: contentMode + layoutMode as separate state values
    diff/
      DiffToolbar.svelte      # Two segmented controls: [Hunk|Full] [Inline|Split]
      DiffViewer.svelte       # 2D dispatch: 4 combinations
      HunkView.svelte         # Unchanged (inline hunk)
      FullFileView.svelte     # Unchanged (inline full)
      SplitView.svelte        # NEW: side-by-side rendering with paired rows
  lib/
    types.ts                  # ContentMode = "hunk" | "full"; LayoutMode = "inline" | "split"
    store.ts                  # Separate getDiffContentMode/getDiffLayoutMode
    diff-utils.ts             # Unchanged + possibly new pairLines() helper
```

### Pattern 1: Two Orthogonal Mode Types
**What:** Replace `ViewMode = "hunk" | "full" | "split"` with two types:
```typescript
export type ContentMode = "hunk" | "full";
export type LayoutMode = "inline" | "split";
```
**When to use:** Everywhere that currently uses ViewMode.
**Implications:**
- DiffPanel.svelte: `let viewMode` becomes `let contentMode` + `let layoutMode`
- DiffToolbar: Two separate `oncontentmodechange` and `onlayoutmodechange` callbacks
- DiffViewer: Dispatch on `layoutMode` first (inline vs split), then pass `contentMode` to child
- Store: Two new keys `diff_content_mode` and `diff_layout_mode` replace `diff_view_mode`
- RepoView.svelte: `showFullFile` derived from `contentMode === "full"` (unchanged logic)
- DiffPanel.test.ts: Mock functions change names, test toolbar has 4 buttons instead of 3

### Pattern 2: Row Pairing Algorithm for Split View
**What:** Transform a flat array of DiffLines (from a hunk) into paired rows for side-by-side display.
**Data structure:**
```typescript
interface PairedRow {
  left: DiffLine | null;   // old side (Context or Delete)
  right: DiffLine | null;  // new side (Context or Add)
}
```
**Algorithm:**
1. Walk hunk lines sequentially
2. Context lines: `{ left: line, right: line }` (same line on both sides)
3. Collect consecutive Delete lines into a buffer, then consecutive Add lines into a buffer
4. Pair Delete[0] with Add[0], Delete[1] with Add[1], etc.
5. If deletes > adds: remaining deletes get `right: null` (phantom on right)
6. If adds > deletes: remaining adds get `left: null` (phantom on left)
7. For split+full mode: flatMap all hunks, then run the same pairing

**Why this structure:** Each PairedRow renders as a single visual row. Both panels are guaranteed to have the same number of rows, which makes scroll sync trivial (same scrollTop works).

### Pattern 3: Scroll Synchronization
**What:** When user scrolls either panel, the other panel scrolls to match.
**Implementation:**
```typescript
let syncing = false;

function handleScroll(source: HTMLElement, target: HTMLElement) {
  if (syncing) return;
  syncing = true;
  target.scrollTop = source.scrollTop;
  target.scrollLeft = source.scrollLeft;
  syncing = false;
}
```
**Why the guard:** Without `syncing` flag, setting `target.scrollTop` fires a scroll event on target, which would try to set source.scrollTop, creating an infinite loop. The boolean guard breaks the recursion.

**Important detail:** Because paired rows guarantee equal height on both sides, `scrollTop` synchronization is exact. No row-height mapping needed.

### Pattern 4: Resizable Divider
**What:** Draggable handle between left and right diff panels.
**Implementation:** Follow the existing `pane-divider` pattern from RepoView.svelte:
- A 4px div with `cursor: col-resize`
- `onmousedown` starts tracking: capture `startX` and initial split ratio
- `mousemove` on window: compute new ratio, clamp between 20-80%
- `mouseup` on window: remove listeners
- Store the split ratio as a CSS variable or flex-basis on the two panels

The existing codebase uses this exact pattern for left/right pane resizing in RepoView. No need for a library.

### Pattern 5: Toolbar Two-Control Layout
**What:** Replace the single 3-way segmented control with two 2-way segmented controls.
**Current:** `[Hunk | Full | Split]` -- one segmented-control div with 3 buttons
**After:** `[Hunk | Full] | divider | [icon:Inline | icon:Split] | divider | WS P Return | file.ts | Stage x`

Recommended Lucide icons (verified available in installed package):
- Inline mode: `Rows2` (two horizontal rows -- represents inline/stacked view)
- Split mode: `Columns2` (two vertical columns -- represents side-by-side view)

Alternative: `Split` icon exists but is more abstract. `Columns2` / `Rows2` pair is more intuitive.

### Anti-Patterns to Avoid
- **Separate scroll containers with different row counts:** Never let left and right panels have different numbers of rows. Phantom rows must pad the shorter side to maintain 1:1 row correspondence.
- **requestAnimationFrame for scroll sync:** Not needed here since both panels have identical row counts/heights. Simple `scrollTop` assignment is sufficient.
- **CSS `position: sticky` for hunk headers in split view:** Hunk headers span both panels, so they live outside the individual panel scroll containers. They are part of the row flow, not sticky-positioned.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Resizable panels | CSS `resize` property | mousedown/mousemove drag pattern (already in RepoView) | CSS resize is limited (no min/max ratio, no callbacks); the codebase already has the pattern |
| Row alignment | Complex DOM measurement | PairedRow data structure pre-computed before render | Measuring DOM heights would be fragile and cause layout thrashing |
| Scroll sync library | Third-party scroll sync lib | Simple scrollTop mirroring with boolean guard | The paired-row guarantee makes this trivial; a library would be overkill |

## Common Pitfalls

### Pitfall 1: Scroll Event Infinite Loop
**What goes wrong:** Setting scrollTop on panel B triggers its scroll event, which sets scrollTop on panel A, creating an infinite loop.
**Why it happens:** The scroll event fires synchronously when scrollTop is assigned.
**How to avoid:** Use a boolean `syncing` flag. Set it true before assigning scrollTop, check it at the top of the handler, reset after assignment.
**Warning signs:** Browser freezing, max call stack exceeded, or visual jittering.

### Pitfall 2: ViewMode Migration Breaking Tests
**What goes wrong:** Existing tests reference `getDiffViewMode`/`setDiffViewMode` and the 3-button segmented control. Changing the type breaks all existing tests.
**Why it happens:** 23 tests in DiffPanel.test.ts use the current mock structure.
**How to avoid:** Update all test mocks in the same commit as the type change. Replace `getDiffViewMode` mock with `getDiffContentMode` + `getDiffLayoutMode` mocks. Update toolbar button assertions from "Hunk"/"Full"/"Split" to the new two-control layout.
**Warning signs:** Test failures after type refactor.

### Pitfall 3: showFullFile Coupling in RepoView
**What goes wrong:** RepoView.svelte reads `getDiffShowFullFile()` on mount and uses `cachedDiffOptions` to decide whether to request full-file diff. If the content/layout refactor doesn't update this path, split+full mode won't fetch full-file content.
**Why it happens:** The `showFullFile` boolean was previously coupled to `viewMode === "full"`. After the refactor, it should be `contentMode === "full"` regardless of layout mode.
**How to avoid:** In DiffPanel's `handleContentModeChange`, persist both `contentMode` and `showFullFile = (mode === "full")` -- same pattern as the existing `handleViewModeChange`. The RepoView cachedDiffOptions path via `getDiffShowFullFile()` continues to work unchanged.
**Warning signs:** Split+full mode showing only hunks (context lines missing).

### Pitfall 4: Phantom Row Height Mismatch with Word Wrap
**What goes wrong:** When word wrap is enabled, long lines may wrap to multiple visual lines, but their paired phantom row stays at single-line height, breaking vertical alignment.
**Why it happens:** Phantom rows are empty, so they don't wrap.
**How to avoid:** Render all paired rows in a single table-like structure (CSS grid or actual rows) where each row's height is determined by the tallest cell. Using `display: grid; grid-template-columns: 1fr 4px 1fr` with each row as a grid row ensures both sides auto-size to the tallest content. Alternatively, wrap both panels' content in a shared row container.
**Warning signs:** Lines drifting out of alignment when word wrap is on and lines are long.

### Pitfall 5: Line Selection Index Mismatch in Split View
**What goes wrong:** Line selection in HunkView uses `lineIdx` (index within `hunk.lines`). In split view, the right panel only shows Add and Context lines. The selection index must still reference the original `hunk.lines` index for the staging backend to work.
**Why it happens:** The pairing algorithm transforms the line array.
**How to avoid:** Each PairedRow must carry the original line index from `hunk.lines`. The `onlineclick` callback should pass this original index, not the paired-row index.
**Warning signs:** Wrong lines being staged/discarded.

### Pitfall 6: Store Key Migration
**What goes wrong:** Users with existing `diff_view_mode` store key get unexpected behavior after the refactor.
**Why it happens:** The old key stores "hunk", "full", or "split". New code reads different keys.
**How to avoid:** On first load, check if old `diff_view_mode` key exists. If value is "split", set layoutMode to "split" and contentMode to "hunk". If value is "full", set contentMode to "full" and layoutMode to "inline". If "hunk", set contentMode to "hunk" and layoutMode to "inline". Then delete the old key (or just ignore it -- the new keys take priority and old key becomes dead).
**Warning signs:** User preferences lost after update.

## Code Examples

### Row Pairing Algorithm
```typescript
// New helper in diff-utils.ts or inline in SplitView.svelte
interface PairedRow {
  left: { line: DiffLine; lineIdx: number } | null;
  right: { line: DiffLine; lineIdx: number } | null;
}

function pairLines(lines: DiffLine[]): PairedRow[] {
  const rows: PairedRow[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (line.origin === "Context") {
      rows.push({
        left: { line, lineIdx: i },
        right: { line, lineIdx: i },
      });
      i++;
      continue;
    }

    // Collect consecutive deletes
    const deletes: { line: DiffLine; lineIdx: number }[] = [];
    while (i < lines.length && lines[i].origin === "Delete") {
      deletes.push({ line: lines[i], lineIdx: i });
      i++;
    }

    // Collect consecutive adds
    const adds: { line: DiffLine; lineIdx: number }[] = [];
    while (i < lines.length && lines[i].origin === "Add") {
      adds.push({ line: lines[i], lineIdx: i });
      i++;
    }

    // Pair them up
    const maxLen = Math.max(deletes.length, adds.length);
    for (let j = 0; j < maxLen; j++) {
      rows.push({
        left: j < deletes.length ? deletes[j] : null,
        right: j < adds.length ? adds[j] : null,
      });
    }
  }

  return rows;
}
```

### Scroll Synchronization
```typescript
// In SplitView.svelte
let leftPanel: HTMLDivElement;
let rightPanel: HTMLDivElement;
let syncing = false;

function syncScroll(source: HTMLDivElement, target: HTMLDivElement) {
  if (syncing) return;
  syncing = true;
  target.scrollTop = source.scrollTop;
  target.scrollLeft = source.scrollLeft;
  syncing = false;
}

// Bind in template:
// <div bind:this={leftPanel} onscroll={() => syncScroll(leftPanel, rightPanel)}>
// <div bind:this={rightPanel} onscroll={() => syncScroll(rightPanel, leftPanel)}>
```

### Resizable Split Divider
```typescript
// In SplitView.svelte
let splitRatio = $state(0.5); // 50/50 default

function startResize(e: MouseEvent) {
  e.preventDefault();
  const container = (e.target as HTMLElement).parentElement!;
  const containerRect = container.getBoundingClientRect();

  function onMouseMove(ev: MouseEvent) {
    const ratio = (ev.clientX - containerRect.left) / containerRect.width;
    splitRatio = Math.max(0.2, Math.min(0.8, ratio));
  }

  function onMouseUp() {
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }

  window.addEventListener("mousemove", onMouseMove);
  window.addEventListener("mouseup", onMouseUp);
}
```

### Type Definitions (refactored)
```typescript
// In types.ts -- replace ViewMode
export type ContentMode = "hunk" | "full";
export type LayoutMode = "inline" | "split";

// Keep ViewMode as deprecated alias if needed, or remove entirely
```

### Store Functions (refactored)
```typescript
// In store.ts -- replace getDiffViewMode/setDiffViewMode
const DIFF_CONTENT_MODE_KEY = "diff_content_mode";
const DIFF_LAYOUT_MODE_KEY = "diff_layout_mode";

export async function getDiffContentMode(): Promise<ContentMode> {
  const stored = await store.get<string>(DIFF_CONTENT_MODE_KEY);
  if (stored === "hunk" || stored === "full") return stored;
  return "hunk";
}

export async function setDiffContentMode(mode: ContentMode): Promise<void> {
  await store.set(DIFF_CONTENT_MODE_KEY, mode);
  await store.save();
}

export async function getDiffLayoutMode(): Promise<LayoutMode> {
  const stored = await store.get<string>(DIFF_LAYOUT_MODE_KEY);
  if (stored === "inline" || stored === "split") return stored;
  return "inline";
}

export async function setDiffLayoutMode(mode: LayoutMode): Promise<void> {
  await store.set(DIFF_LAYOUT_MODE_KEY, mode);
  await store.save();
}
```

### Phantom Row CSS Custom Property
```css
/* In app.css */
--color-diff-phantom-bg: rgba(139, 148, 158, 0.04);
```
This is subtler than the muted background color (--color-muted-bg at 0.1 opacity). The phantom row should be barely visible -- just enough to indicate it's a spacer, not content.

### Toolbar Icons Import
```typescript
// In DiffToolbar.svelte
import { Columns2, Pilcrow, Rows2, Space, TextWrap } from "@lucide/svelte";
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 3-way ViewMode (hunk/full/split) | Two orthogonal modes (content + layout) | Phase 64 | Cleaner mental model; 4 combinations instead of 3 exclusive modes |
| Single segmented control | Two segmented controls with icons | Phase 64 | Independent toggling; split+full becomes possible |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 3.x + @testing-library/svelte |
| Config file | vitest.config.ts |
| Quick run command | `npx vitest --run src/components/DiffPanel.test.ts` |
| Full suite command | `npx vitest --run` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VIEW-02 | Split view shows old/new with phantom rows | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "VIEW-02"` | Partial (stub test exists at line 467-481) |
| VIEW-03 | Scroll sync between panels | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "VIEW-03"` | No -- Wave 0 |
| VIEW-05 | Stage/unstage/discard in split view | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "VIEW-05"` | No -- Wave 0 |
| - | ViewMode refactor: toolbar renders two controls | unit | `npx vitest --run src/components/DiffPanel.test.ts -t "segmented control"` | Existing (line 424-436, needs update) |
| - | Store migration: old key compat | unit | `npx vitest --run src/lib/store.test.ts` | Existing (needs new cases) |

### Sampling Rate
- **Per task commit:** `npx vitest --run src/components/DiffPanel.test.ts`
- **Per wave merge:** `just check`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Update existing DiffPanel.test.ts mocks: replace `getDiffViewMode`/`setDiffViewMode` with `getDiffContentMode`/`setDiffContentMode`/`getDiffLayoutMode`/`setDiffLayoutMode`
- [ ] Update toolbar test assertions from 3-button to 2+2 button layout
- [ ] Add VIEW-02 tests: split view renders left/right panels, phantom rows present, correct gutter (old-only left, new-only right)
- [ ] Add VIEW-03 tests: scroll sync (mock scrollTop assignment, verify propagation)
- [ ] Add VIEW-05 tests: staging buttons in split hunk headers, line selection on right panel, disabled when whitespace ignore active
- [ ] Add store.test.ts cases for `getDiffContentMode`/`getDiffLayoutMode`

### Existing Test Baseline
There is 1 pre-existing test failure in TabBar.test.ts (unrelated to this phase -- "calls onactivate when tab clicked"). All 40 other test files pass. Phase 64 changes should not introduce additional failures.

## Files to Modify

### Must Change
| File | Change | Reason |
|------|--------|--------|
| `src/lib/types.ts` | Replace `ViewMode` with `ContentMode` + `LayoutMode` | D-01, D-02 |
| `src/lib/store.ts` | Replace `getDiffViewMode`/`setDiffViewMode` with content/layout getters/setters | D-02 |
| `src/components/DiffPanel.svelte` | Refactor state to `contentMode` + `layoutMode`; update handlers and $effect | D-02, D-03 |
| `src/components/diff/DiffToolbar.svelte` | Two segmented controls with icons | D-04, D-05 |
| `src/components/diff/DiffViewer.svelte` | 2D dispatch based on contentMode + layoutMode | D-03 |
| `src/components/diff/SplitView.svelte` | Full implementation: row pairing, two panels, scroll sync, staging | D-06 through D-16 |
| `src/app.css` | Add `--color-diff-phantom-bg` | D-07 |
| `src/components/DiffPanel.test.ts` | Update mocks and assertions for new type system; add VIEW-02/03/05 tests | Testing |

### Should Check
| File | Why |
|------|-----|
| `src/components/RepoView.svelte` | Uses `getDiffShowFullFile` -- verify no breakage from store refactor |
| `src/lib/store.test.ts` | Add tests for new store functions |

## Open Questions

1. **Word wrap + phantom row height alignment**
   - What we know: When word wrap is on, lines can be taller than 1.5em (line-height). Phantom rows are empty.
   - What's unclear: Whether CSS grid row auto-sizing will naturally handle this, or if explicit height synchronization is needed.
   - Recommendation: Use a CSS grid layout where each "row" spans both panels. The grid row height is determined by the tallest cell, automatically keeping phantoms aligned. If that creates complexity, fall back to measuring and setting explicit heights via $effect. Test word wrap alignment specifically during development.

2. **Hunk header spanning both panels**
   - What we know: In split+hunk mode, hunk headers span full width (D-12). They contain staging buttons.
   - What's unclear: Whether the hunk header should be outside the two scrollable panels (always visible at top) or inline in the row flow.
   - Recommendation: Hunk headers are inline in the row flow (they scroll with content, not sticky). They span the full width of the grid, interrupting the two-column layout. This is simpler and consistent with how HunkView works today.

## Sources

### Primary (HIGH confidence)
- Codebase inspection: All files listed in Canonical References section of CONTEXT.md were read and analyzed
- Lucide icon availability: Verified `Columns2`, `Rows2`, `Split` exist in `node_modules/@lucide/svelte/dist/icons/`
- Existing pane-divider pattern: Verified in RepoView.svelte lines 536-594, 598-607

### Secondary (MEDIUM confidence)
- Scroll sync recursion guard pattern: Standard DOM technique, widely documented

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, everything exists in codebase
- Architecture: HIGH -- all patterns derived from existing codebase code and locked CONTEXT.md decisions
- Pitfalls: HIGH -- identified from direct codebase analysis (test mocks, RepoView coupling, scroll events)

**Research date:** 2026-03-30
**Valid until:** 2026-04-30 (stable -- no external dependency changes expected)
