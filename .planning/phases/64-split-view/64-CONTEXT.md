# Phase 64: Split View - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement side-by-side diff with old content on the left panel and new content on the right, with phantom/spacer rows for vertical alignment, synchronized scrolling, and hunk/line staging support. The split layout is an independent toggle orthogonal to hunk/full content mode — producing 4 combinations (inline hunk, inline full, split hunk, split full). This replaces the current 3-way ViewMode selector with two independent toggles.

</domain>

<decisions>
## Implementation Decisions

### Architecture: Two Orthogonal Toggles
- **D-01:** Split is a **layout** dimension (inline vs side-by-side), independent of the **content** dimension (hunk vs full). This produces 4 combinations: inline hunk, inline full, split hunk, split full.
- **D-02:** The current `ViewMode = "hunk" | "full" | "split"` 3-way type must be refactored into two separate state values — one for content mode (hunk/full) and one for layout mode (inline/split). Both persist independently via LazyStore.
- **D-03:** DiffViewer dispatches rendering based on both dimensions. SplitView receives `contentMode` to know whether to show hunk headers or flatten lines.

### Toolbar: Two Segmented Controls
- **D-04:** Toolbar layout: `[Hunk|Full] | [Inline|Split] | WS ¶ ↩ | file.ts | Stage ×`. Two segmented controls side by side with a visual separator between them.
- **D-05:** The inline/split segmented control uses icons (inline icon and split icon, Lucide). Both toggles are always visible and independent.

### Row Alignment Strategy
- **D-06:** Paired rows with phantom spacers. Delete lines on the left are matched with corresponding Add lines on the right. When one side has more lines, blank phantom rows are inserted on the shorter side to maintain vertical alignment.
- **D-07:** Phantom rows are empty rows with a slightly muted background (CSS custom property). Clean and minimal, no stripes or patterns.

### Panel Layout
- **D-08:** Resizable divider between the two panels. A draggable handle lets users resize the split.
- **D-09:** Single gutter per panel — left panel shows old line numbers only, right panel shows new line numbers only. One gutter column per side to save horizontal space.
- **D-10:** No origin symbols (+/-/space) in split view. The color backgrounds (green for add on right, red for delete on left) already indicate the change type. Context lines have no background.

### Synchronized Scrolling
- **D-11:** Scrolling either panel scrolls both panels in sync (VIEW-03). Locked sync, no independent scroll option.

### Hunk Presentation
- **D-12:** In split+hunk mode: hunk headers (@@ lines) span the full width of both panels. Stage/Discard/Unstage buttons sit on the right side of the header, consistent with inline hunk view.
- **D-13:** In split+full mode: no hunk headers, continuous document on both sides with changed lines highlighted by background color.

### Staging Interactions (VIEW-05)
- **D-14:** Hunk-level actions (Stage Hunk, Discard Hunk, Unstage Hunk) appear in the hunk header row spanning both panels, same position and behavior as inline hunk view.
- **D-15:** Line selection works on the right (new) panel only. User clicks Add lines on the right to select them. Shift+click for range selection. Stage Lines / Discard Lines buttons appear in the hunk header when lines are selected.
- **D-16:** All staging is disabled when whitespace ignore is active (carrying forward from Phase 59/63).

### Claude's Discretion
- Exact Lucide icons for inline/split toggle buttons
- CSS custom property name and value for phantom row muted background
- Internal data structure for paired row alignment (how to pair delete/add lines and generate phantom entries)
- Drag handle implementation for resizable divider (CSS resize, custom drag handler, or library)
- Scroll sync implementation (scroll event listener approach)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Diff components (modify)
- `src/components/diff/SplitView.svelte` — Stub to implement as side-by-side diff renderer
- `src/components/diff/DiffToolbar.svelte` — Replace 3-way segmented control with two independent controls
- `src/components/diff/DiffViewer.svelte` — Update dispatch logic for 2-dimensional mode (content + layout)
- `src/components/diff/HunkView.svelte` — Reference for line rendering, gutter, spans, staging buttons
- `src/components/diff/FullFileView.svelte` — Reference for continuous document rendering pattern
- `src/components/DiffPanel.svelte` — Owns state; refactor ViewMode to two separate state values, wire props to SplitView

### Types
- `src/lib/types.ts` — ViewMode type to refactor; DiffLine, FileDiff, DiffHunk, MergedSpan, DiffRequestOptions

### State persistence
- `src/lib/store.ts` — LazyStore; refactor diff_view_mode into content mode + layout mode keys

### Theme / CSS
- `src/app.css` — CSS custom properties; add phantom row background variable

### Tests
- `src/components/DiffPanel.test.ts` — Existing tests that must pass after ViewMode refactor

### Requirements
- `.planning/REQUIREMENTS.md` — VIEW-02, VIEW-03, VIEW-05 mapped to this phase

### Prior phase context
- `.planning/phases/62-ui-refactor-component-structure/62-CONTEXT.md` — D-01/D-02 (component decomposition), D-04/D-05 (original 3-way ViewMode, segmented control)
- `.planning/phases/63-full-file-view-display-options/63-CONTEXT.md` — D-01 (continuous document pattern), D-14 (whitespace staging guard), D-16 (word wrap global)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `HunkView.svelte` line rendering: gutter columns, merged span loop, syntax classes, word-diff emphasis — SplitView reuses this rendering pattern for each panel
- `FullFileView.svelte` continuous rendering: flatMap all hunks' lines — SplitView reuses for split+full mode
- `lineBackground()`, `lineColor()`, `originSymbol()`, `maxLineNumber()`, `gutterWidth()` — helper functions duplicated across HunkView and FullFileView, can be reused
- `DiffToolbar.svelte` segmented control: existing flexbox pattern — refactor from 3-way to two 2-way controls
- `splitInvisibles()`, `trailingWhitespaceStart()` from `diff-utils.ts` — reuse for invisible character rendering in split view
- Staging callback props pattern (onstagehunk, onunstagelines, etc.) — same interface for SplitView

### Established Patterns
- Svelte 5 runes (`$state`, `$derived`, `$effect`) for reactive state
- CSS custom properties for all colors — no inline colors
- `$props()` destructuring for component interfaces
- Inline styles on diff elements (not Tailwind classes)
- Monospace font at 12px, line-height 1.5 for diff content
- Dynamic gutter width based on max line number digits

### Integration Points
- `DiffViewer.svelte` currently dispatches with `viewMode === "split"` passing no props — needs full props pass-through
- `DiffPanel.svelte` loads ViewMode from LazyStore — needs to split into two separate stored values
- DiffToolbar emits mode change callbacks — needs two callbacks instead of one
- Existing tests reference `getDiffViewMode` / `setDiffViewMode` — need to handle the type change

</code_context>

<specifics>
## Specific Ideas

- The split view should feel like VS Code's side-by-side diff — clean, paired rows, phantom rows with subtle background
- The two segmented controls in the toolbar ([Hunk|Full] and [Inline|Split]) should be visually distinct but consistent in style
- The resizable divider should have a visible but non-intrusive drag handle
- In split+full mode, both panels show the entire file as a continuous document with aligned rows
- Word wrap, show invisibles, and whitespace ignore toggles apply equally to split view (global settings from Phase 63)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 64-split-view*
*Context gathered: 2026-03-30*
