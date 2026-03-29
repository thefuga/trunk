# Phase 62: UI Refactor & Component Structure - Context

**Gathered:** 2026-03-29 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Decompose the 667-line DiffPanel monolith into focused components (toolbar, viewer dispatcher, line renderer) that support multiple view modes (Hunk, Full File, Split) and display line numbers. Existing hunk view behavior (stage/unstage/discard hunks and lines, keyboard navigation) must work identically after refactor. Word-level diff highlights and syntax coloring render correctly through the new component structure.

</domain>

<decisions>
## Implementation Decisions

### Component Decomposition
- **D-01:** Three-level split: `DiffToolbar` (view mode toggle, file-level actions, close button), `DiffViewer` (view mode dispatcher — renders HunkView, FullFileView, or SplitView based on current mode), and a `DiffLineRenderer` snippet/component for individual line rendering (origin symbol, merged spans, line backgrounds).
- **D-02:** `DiffPanel.svelte` becomes a thin shell: owns state (selectedHunkKey, line selection, collapsedFiles), passes props down. All staging operation handlers stay in DiffPanel — they need access to `repoPath`, `diffKind`, `onhunkaction` from RepoView.
- **D-03:** For this phase, only HunkView is functional (existing behavior). FullFileView and SplitView are stub components that show a placeholder message — they get implemented in Phases 63 and 64.

### View Mode Switching
- **D-04:** Three view modes: `"hunk"` (default, current behavior), `"full"` (Phase 63), `"split"` (Phase 64). Represented as a TypeScript union type `ViewMode = "hunk" | "full" | "split"`.
- **D-05:** Segmented control in DiffToolbar with three buttons. Active mode has a highlighted background. Persisted to LazyStore via `diff_view_mode` key.
- **D-06:** View mode state lives in DiffPanel (loaded from LazyStore on mount, saved on change). Passed to DiffViewer as prop.

### Line Number Gutter
- **D-07:** Two-column gutter showing old line number (left) and new line number (right), positioned before the origin symbol (+/-/space). Context lines show both numbers, Add lines show only new, Delete lines show only old. GitHub/GitKraken style.
- **D-08:** Gutter columns use fixed-width `min-width` based on max line number digits. Color matches `--color-text-muted` to avoid visual competition with diff content.
- **D-09:** Line numbers come from existing `DiffLine.old_lineno` and `DiffLine.new_lineno` fields (already populated by Rust backend — these are `Option<u32>` / `number | null` in TS).

### Toolbar Design
- **D-10:** Toolbar contains: view mode segmented control (left), filename (center, overflow ellipsis), file-level actions + close button (right). Replaces the current 24px header bar.
- **D-11:** Stage File / Unstage File buttons remain in the toolbar (same position as now — right side). Hunk-level and line-level action buttons remain inline in the hunk toolbar row (unchanged).
- **D-12:** Context lines dropdown, whitespace toggle, word wrap, show invisibles are NOT added in this phase — they belong to Phase 63.

### Claude's Discretion
- Exact file organization (separate files vs `DiffPanel/` subdirectory)
- Whether DiffLineRenderer is a separate `.svelte` component or a `{#snippet}` within DiffViewer
- CSS class naming for gutter elements
- Transition/animation on view mode switch (if any)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Current diff implementation (to refactor)
- `src/components/DiffPanel.svelte` — 667-line monolith being decomposed: toolbar, file headers, hunk toolbars, line rendering, staging handlers, keyboard navigation
- `src/components/DiffPanel.test.ts` — Existing tests that must pass after refactor

### Consumer of DiffPanel
- `src/components/RepoView.svelte` — Uses DiffPanel in 2 places (lines 597-604 for rebase, lines 650-663 for main diff). Props interface must remain compatible.

### Types
- `src/lib/types.ts` — DiffLine (old_lineno, new_lineno, spans), FileDiff, DiffHunk, MergedSpan, DiffRequestOptions, ViewMode (to be added)

### Persistence
- `src/lib/store.ts` — LazyStore pattern for diff preferences. Add diff_view_mode key.

### Theme / CSS
- `src/app.css` — CSS custom properties for diff colors, syntax colors, word-diff colors

### Requirements
- `.planning/REQUIREMENTS.md` — DISP-01 (line numbers), VIEW-01 (view mode toggle)

### Prior phase context
- `.planning/phases/59-backend-data-model-diff-options/59-CONTEXT.md` — D-06/D-07 (global preferences, persisted view_mode)
- `.planning/phases/61-syntax-highlighting/61-CONTEXT.md` — D-07/D-08 (merged span rendering, single loop)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `DiffPanel.svelte` staging handlers (handleStageHunk, handleUnstageHunk, handleDiscardHunk, handleStageLines, handleUnstageLines, handleDiscardLines): 160 lines of staging logic to keep in parent shell
- `lineBackground()`, `lineColor()`, `originSymbol()`: Helper functions to extract into line renderer
- `scrollToHunk()`, keyboard navigation: Hunk-specific logic that stays with HunkView
- `LazyStore` pattern (store.ts): Proven get/set/save per key — extend with `diff_view_mode`
- `DiffLine.old_lineno` / `DiffLine.new_lineno`: Already populated by Rust backend, ready for gutter rendering

### Established Patterns
- Svelte 5 runes (`$state`, `$derived`, `$effect`) used throughout
- CSS custom properties for all colors — no inline colors
- Props interface pattern with `$props()` destructuring
- Inline styles used extensively in DiffPanel (no Tailwind classes in this component)

### Integration Points
- `RepoView.svelte` passes `fileDiffs`, `commitDetail`, `selectedPath`, `onclose`, `diffKind`, `repoPath`, `onhunkaction`, `loading` to DiffPanel — this props interface must not change
- `DiffPanel.test.ts` renders DiffPanel directly — tests must continue to work against the same public interface

</code_context>

<specifics>
## Specific Ideas

- The refactor should be purely structural — no visual changes to the current hunk view behavior. A user should not notice any difference when using hunk mode after this phase.
- View mode stubs for "full" and "split" should render a centered message like "Full file view — coming soon" so the toggle is functional but clearly unfinished.
- Line number gutter should handle files with 10,000+ lines gracefully (4-5 digit numbers) without layout shifts.

</specifics>

<deferred>
## Deferred Ideas

None — analysis stayed within phase scope

</deferred>

---

*Phase: 62-ui-refactor-component-structure*
*Context gathered: 2026-03-29*
