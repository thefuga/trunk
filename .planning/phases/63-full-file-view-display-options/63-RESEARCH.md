# Phase 63: Full File View & Display Options - Research

**Researched:** 2026-03-29
**Domain:** Svelte 5 diff viewer -- full file rendering, toolbar toggles, invisible character display, word wrap, preference persistence
**Confidence:** HIGH

## Summary

Phase 63 replaces the FullFileView stub with a real continuous-document renderer, adds three display toggle buttons to DiffToolbar, implements invisible character visualization, word wrap, and the whitespace-ignore staging guard. All new preferences persist via the existing LazyStore pattern in `trunk-prefs.json`.

The codebase is well-prepared for this phase. HunkView already contains the complete line-rendering logic (gutter, origin symbols, merged span loop, syntax classes, word-diff emphasis) that FullFileView needs to replicate minus hunk headers and staging buttons. The LazyStore pattern for diff preferences (contextLines, ignoreWhitespace, showFullFile) was established in Phase 59. The `show_full_file` backend parameter already generates full-file diffs via `context_lines=100000`. DiffViewer already dispatches to FullFileView based on viewMode. The toolbar segmented control pattern in DiffToolbar provides the visual template for toggle buttons.

**Primary recommendation:** Implement FullFileView by flattening all hunks' lines into a single list (reusing HunkView's line-rendering pattern), add three Lucide icon toggle buttons to DiffToolbar with the same active/inactive styling as the segmented control, render invisible characters via frontend string replacement in a rendering utility function, and wire all new preferences through DiffPanel to LazyStore.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Continuous document -- remove hunk headers entirely. Show the whole file as one scrollable block with changed lines highlighted (green/red backgrounds). No section dividers.
- **D-02:** No staging in full file view. It's read-only for reviewing. User switches to hunk view to stage. VIEW-05 (staging in all view modes) is Phase 64's scope.
- **D-03:** Two-column gutter (old + new line numbers), same rules as hunk view. Delete lines show old number with blank new column. Add lines show blank old column with new number. Context lines show both. Consistent across all view modes.
- **D-04:** New files (all Add lines) show complete file with add backgrounds and sequential line numbers in full file view.
- **D-05:** Three toggle buttons inline after the view mode segmented control, separated by a visual divider: whitespace ignore (WS), show invisibles, word wrap. Layout: `[Hunk|Full|Split] | WS p ↩ | file.ts | Stage x`
- **D-06:** Active toggle state indicated by highlighted/filled background -- same pattern as the view mode segmented control. Inactive = transparent/ghost.
- **D-07:** No context lines dropdown in toolbar. Context lines is a config-file-only setting (LazyStore `trunk-prefs.json`).
- **D-08:** Context lines preset values (for config file and future settings page): 0 / 3 / 5 / 10 / 25. Default: 3.
- **D-09:** All user-configurable preferences stored in config file (`trunk-prefs.json`). Toolbar toggles read/write from this file.
- **D-10:** Context lines dropdown hidden when view mode is "full" (N/A since no dropdown -- but the setting itself is irrelevant in full file mode since backend uses show_full_file=true).
- **D-11:** Inline substitution: spaces rendered as middle dot (U+00B7), tabs rendered as rightwards arrow (U+2192). Symbols shown in a muted color so they don't compete with code content.
- **D-12:** Trailing whitespace at end of lines gets a subtle warning background highlight (faint red/amber).
- **D-13:** No line ending markers (CR/LF). Only spaces and tabs are shown as invisible characters.
- **D-14:** When whitespace ignore is active, all staging/unstaging/discard buttons (hunk-level and line-level) are disabled with a tooltip explaining why.
- **D-15:** Lines wrap at the diff viewer container edge. No hanging indent -- continuation starts at column 0 (past the gutter). CSS `white-space: pre-wrap`.
- **D-16:** Word wrap is a global toggle -- applies to hunk view, full file view, and split view equally. Single config value in LazyStore.

### Claude's Discretion
- CSS implementation technique for invisible character rendering (span replacement vs pseudo-elements)
- Exact muted color for invisible character markers (CSS custom property)
- Warning background color for trailing whitespace (CSS custom property)
- Lucide icon choices for the three toolbar toggle buttons
- Whether invisibles rendering happens in frontend only or also in Rust backend

### Deferred Ideas (OUT OF SCOPE)
- **Settings/preferences page** -- Full UI for all configurables (themes, fonts, tab size, context lines, etc.). Future milestone.
- **Per-hunk context expand buttons** -- "Show N more lines" within hunk view. Listed in REQUIREMENTS.md as ADVD-02 (future).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| VIEW-04 | Full file view shows entire file with changed lines highlighted (via context_lines=MAX) | Backend `show_full_file` already implemented (100k context_lines). FullFileView component flattens all hunks into continuous list. HunkView line-rendering pattern reusable. |
| WHSP-02 | Hunk/line staging is disabled with tooltip when whitespace ignore is active | `ignoreWhitespace` state already persisted in LazyStore. DiffPanel owns staging handlers -- propagate ignore state to HunkView buttons. HTML `title` attribute for tooltip. |
| WHSP-03 | User can toggle display of invisible characters (spaces as dots, tabs as arrows) | Frontend-only string replacement in rendering utility. New `showInvisibles` LazyStore preference pair. Muted CSS color for markers. |
| DISP-02 | User can toggle word wrap in the diff viewer | CSS `white-space` toggle between `pre` and `pre-wrap`. New `wordWrap` LazyStore preference pair. Applied globally via class/style on diff container. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte 5 | 5.x (runes) | Component framework | Project standard -- `$state`, `$derived`, `$effect`, `$props()` |
| @lucide/svelte | ^0.577.0 | Icon components | Already used across 12+ components for consistent iconography |
| @tauri-apps/plugin-store | 2.x | LazyStore persistence | Established pattern for trunk-prefs.json -- 15+ preference pairs already |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @testing-library/svelte | installed | Component testing | DiffPanel.test.ts test patterns |
| vitest | installed | Test runner | `bun run test` |

**No new dependencies required.** Everything needed is already installed.

## Architecture Patterns

### Data Flow for Full File View

```
RepoView
  buildDiffOptions() reads LazyStore { contextLines, ignoreWhitespace, showFullFile }
  invoke("diff_unstaged/staged/commit", { options })
    --> Rust apply_request_options: show_full_file=true --> context_lines=100000
    --> git2 returns full file as one large hunk (or few hunks)
  fileDiffs passed to DiffPanel
    DiffPanel passes to DiffViewer (viewMode="full")
      DiffViewer renders FullFileView
        FullFileView flattens all hunks' lines into single continuous list
```

### Preference Flow

```
DiffPanel (state owner)
  $effect on mount: loads showInvisibles, wordWrap, ignoreWhitespace from LazyStore
  passes as props to DiffToolbar (toggle display + callbacks)
  passes as props to DiffViewer --> HunkView/FullFileView (rendering behavior)
  on toggle change: updates local state + persists via set* LazyStore functions
```

### FullFileView Component Structure

```
FullFileView.svelte
  Props: fileDiffs, showInvisibles, wordWrap (read-only display, no staging)
  For each fileDiff:
    Compute maxLineNumber, gutterWidth (same helpers as HunkView)
    Flatten all hunks' lines into single array
    Render each line with:
      - Two-column gutter (old_lineno, new_lineno)
      - Origin symbol (+, -, space)
      - Merged span loop (syntax + word-diff, same as HunkView)
      - Invisible character substitution (if showInvisibles)
      - CSS white-space: pre-wrap or pre (if wordWrap)
```

### DiffToolbar Toggle Button Pattern

```
DiffToolbar.svelte
  New props: ignoreWhitespace, showInvisibles, wordWrap
  New callbacks: onignorewhitespacechange, onshowinvisibleschange, onwordwrapchange
  Layout: [segmented-control] [divider] [WS] [invisibles] [wrap] [filename] [actions] [close]

  Each toggle button:
    - Lucide icon (14px, consistent with toolbar height)
    - class:active binding for highlighted background
    - onclick calls parent callback
    - Same CSS pattern as .segment.active
```

### Invisible Character Rendering

```typescript
// Frontend-only utility function
function renderWithInvisibles(text: string): { text: string; trailingStart: number | null } {
  // Replace spaces with middle dot (U+00B7)
  // Replace tabs with rightwards arrow (U+2192)
  // Detect trailing whitespace position for warning highlight
}
```

**Recommendation: Frontend span replacement, not pseudo-elements.**

Rationale:
- The existing line rendering uses `line.content.slice(span.start, span.end)` with byte offsets from merged spans
- Pseudo-elements (::before/::after) cannot target individual characters within a text node
- Span replacement fits naturally into the existing merged span rendering loop
- The invisible characters need to be styled with a muted color (separate from content text), requiring a wrapping element
- Performance: replacement happens once per render, not continuously via CSS

Implementation approach:
1. When `showInvisibles` is true, after slicing content for each span, replace space/tab characters with styled `<span>` elements containing the Unicode substitution
2. Use a Svelte `{#each}` over segments within each span, or simpler: apply the replacement to the content string and use a CSS class to color the substitution characters
3. For trailing whitespace detection: scan from end of `line.content` backwards to find the boundary between content and trailing spaces/tabs

### Anti-Patterns to Avoid
- **Do not modify Rust backend for invisible character rendering** -- this is purely a display concern, not a data concern. The backend returns raw content; the frontend decides how to display it.
- **Do not duplicate HunkView line rendering** -- extract shared rendering logic or copy the exact same pattern. FullFileView's line rendering must be identical to HunkView's (gutter, spans, syntax classes) minus hunk headers.
- **Do not use CSS `::before`/`::after` for invisible chars** -- cannot target individual characters within text content spans. Need inline replacement.
- **Do not couple word wrap to view mode** -- D-16 says word wrap is global across all view modes.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Icon rendering | Custom SVG for toggle buttons | `@lucide/svelte` icons (Space, Pilcrow, WrapText) | Consistent with 12+ existing icon usages across codebase |
| Preference persistence | Custom file I/O or localStorage | LazyStore `trunk-prefs.json` | Established pattern with 15+ preference pairs already working |
| Tooltip rendering | Custom tooltip component | HTML `title` attribute | D-14 says "disabled with a tooltip" -- native tooltip is sufficient for this use case; matches browser standard |
| Full file diff generation | Frontend hunk stitching | Rust `show_full_file=true` (context_lines=100000) | Already implemented in Phase 59; git2 does the heavy lifting |

## Common Pitfalls

### Pitfall 1: Byte Offset Misalignment with Unicode Substitution
**What goes wrong:** Merged spans use byte offsets (`start`, `end`) into `line.content`. If you replace spaces/tabs with Unicode characters of different byte length before slicing, all span offsets break.
**Why it happens:** U+00B7 (middle dot) and U+2192 (arrow) are multi-byte in UTF-8, but JavaScript strings are UTF-16 where these are single code units. The risk is if you do string replacement before span slicing.
**How to avoid:** Always slice first using original offsets, then replace within the sliced segment. The rendering order must be: (1) slice by span offsets, (2) then substitute invisible characters within each slice.
**Warning signs:** Syntax highlighting or word-diff emphasis appearing on wrong characters after enabling "show invisibles."

### Pitfall 2: View Mode vs showFullFile Desync
**What goes wrong:** User switches to "full" view mode but the diff data wasn't fetched with `showFullFile=true`, so only context lines around changes are shown (not the full file).
**Why it happens:** View mode is a frontend UI state; `showFullFile` is a backend parameter that must be set when requesting the diff. If the view mode changes but the diff isn't re-fetched, the data won't match the view.
**How to avoid:** When view mode switches to "full", set `showFullFile=true` in LazyStore and trigger a diff re-fetch. When switching away from "full", set `showFullFile=false` and re-fetch. The `buildDiffOptions()` function in RepoView already reads from LazyStore, so the re-fetch will pick up the new value.
**Warning signs:** Full file view showing "..." or collapsed sections instead of the complete file.

### Pitfall 3: Word Wrap Breaking Gutter Alignment
**What goes wrong:** When `white-space: pre-wrap` is active, long lines wrap to the next visual row, but the gutter columns (line numbers) only occupy the first visual row, causing misalignment.
**Why it happens:** The gutter uses `flex-shrink: 0` and fixed width, but the content area wraps, making it taller than the gutter.
**How to avoid:** The diff line is already `display: flex`. With `align-items: flex-start` (not `center` or `stretch`), the gutter stays pinned to the top of the line, and wrapped content flows below naturally. The current HunkView doesn't set `align-items`, so the default `stretch` works for single-line. For wrapped lines, ensure each diff line row uses `align-items: flex-start`.
**Warning signs:** Line numbers appearing centered vertically within a multi-row wrapped line.

### Pitfall 4: Staging Guard Not Covering All Buttons
**What goes wrong:** Whitespace ignore is active but some staging buttons are still clickable because the guard doesn't reach all locations.
**Why it happens:** Staging buttons exist in three places: (1) toolbar file-level (Stage File / Unstage File in DiffToolbar), (2) hunk-level (Stage Hunk / Discard Hunk in HunkView), (3) line-level (Stage Lines in HunkView). Missing any location means partial protection.
**How to avoid:** Pass `ignoreWhitespace` as a prop through the entire component tree: DiffPanel -> DiffToolbar (file buttons), DiffPanel -> DiffViewer -> HunkView (hunk + line buttons). Each button location checks the flag independently.
**Warning signs:** Being able to click "Stage Hunk" while "WS" toggle is active.

### Pitfall 5: Store Mock Not Updated in Tests
**What goes wrong:** DiffPanel tests fail because the store mock doesn't include the new `getShowInvisibles`, `setShowInvisibles`, `getWordWrap`, `setWordWrap` functions.
**Why it happens:** DiffPanel.test.ts mocks `../lib/store.js` with explicit function exports. Adding new store functions without updating the mock causes import errors.
**How to avoid:** Update the store mock in DiffPanel.test.ts to include all new preference functions.
**Warning signs:** `TypeError: getShowInvisibles is not a function` in test output.

### Pitfall 6: Full File View Re-fetch Not Triggered
**What goes wrong:** Switching to "full" view shows a partial diff (only hunks around changes) because the diff data was fetched with `showFullFile=false`.
**Why it happens:** The current code in DiffPanel calls `setDiffViewMode(mode)` on view mode change, but doesn't trigger a diff re-fetch. The diff is only fetched when a file is selected or a refresh event fires.
**How to avoid:** When view mode changes to/from "full", also update `showFullFile` in LazyStore and signal RepoView to re-fetch. The simplest approach: add an `ondiffoptionschange` callback from DiffPanel to RepoView so that when display options change that affect the backend query (ignoreWhitespace, showFullFile), a re-fetch is triggered.
**Warning signs:** Full file view showing the same abbreviated diff as hunk view.

## Code Examples

### Example 1: FullFileView Line Rendering (based on HunkView pattern)

```svelte
<!-- Source: Derived from existing HunkView.svelte lines 263-289 -->
{#each fileDiffs as fd (fd.path)}
  {@const gutterW = gutterWidth(maxLineNumber(fd))}
  {@const allLines = fd.hunks.flatMap(h => h.lines)}
  {#each allLines as line}
    <div
      class="diff-line {line.origin === 'Add' ? 'diff-line-add' : line.origin === 'Delete' ? 'diff-line-delete' : 'diff-line-context'}"
      style="
        font-family: monospace;
        font-size: 12px;
        line-height: 1.5;
        padding: 0 8px;
        white-space: {wordWrap ? 'pre-wrap' : 'pre'};
        overflow-x: {wordWrap ? 'hidden' : 'auto'};
        background: {lineBackground(line.origin)};
        color: {lineColor(line.origin)};
        display: flex;
        align-items: flex-start;
      "
    >
      <span style="min-width: {gutterW}; ...">{line.old_lineno ?? ''}</span>
      <span style="min-width: {gutterW}; ...">{line.new_lineno ?? ''}</span>
      <span class="diff-line-content">
        <!-- Same merged span loop as HunkView -->
      </span>
    </div>
  {/each}
{/each}
```

### Example 2: LazyStore Preference Pair (established pattern from store.ts)

```typescript
// Source: Follows exact pattern from store.ts lines 258-287
const DIFF_SHOW_INVISIBLES_KEY = "diff_show_invisibles";

export async function getDiffShowInvisibles(): Promise<boolean> {
  return (await store.get<boolean>(DIFF_SHOW_INVISIBLES_KEY)) ?? false;
}

export async function setDiffShowInvisibles(show: boolean): Promise<void> {
  await store.set(DIFF_SHOW_INVISIBLES_KEY, show);
  await store.save();
}

const DIFF_WORD_WRAP_KEY = "diff_word_wrap";

export async function getDiffWordWrap(): Promise<boolean> {
  return (await store.get<boolean>(DIFF_WORD_WRAP_KEY)) ?? false;
}

export async function setDiffWordWrap(wrap: boolean): Promise<void> {
  await store.set(DIFF_WORD_WRAP_KEY, wrap);
  await store.save();
}
```

### Example 3: Invisible Character Rendering Utility

```typescript
// Frontend-only utility -- slice first, then substitute
interface InvisibleSegment {
  text: string;
  isInvisible: boolean;
  isTrailing: boolean;
}

function splitInvisibles(text: string, isTrailingRegion: boolean): InvisibleSegment[] {
  const segments: InvisibleSegment[] = [];
  let current = "";
  let currentIsInvisible = false;

  for (const ch of text) {
    const invisible = ch === " " || ch === "\t";
    if (invisible !== currentIsInvisible && current) {
      segments.push({ text: currentIsInvisible ? current.replace(/ /g, "\u00B7").replace(/\t/g, "\u2192") : current, isInvisible: currentIsInvisible, isTrailing: currentIsInvisible && isTrailingRegion });
      current = "";
    }
    current += ch;
    currentIsInvisible = invisible;
  }
  if (current) {
    segments.push({ text: currentIsInvisible ? current.replace(/ /g, "\u00B7").replace(/\t/g, "\u2192") : current, isInvisible: currentIsInvisible, isTrailing: currentIsInvisible && isTrailingRegion });
  }
  return segments;
}
```

### Example 4: DiffToolbar Toggle Button (following segmented control pattern)

```svelte
<!-- Source: Extends existing DiffToolbar.svelte segmented control pattern -->
<div class="toolbar-divider"></div>
<button
  class="toggle-btn"
  class:active={ignoreWhitespace}
  title="Ignore whitespace changes"
  onclick={() => onignorewhitespacechange(!ignoreWhitespace)}
>
  <Space size={14} />
</button>
<button
  class="toggle-btn"
  class:active={showInvisibles}
  title="Show invisible characters"
  onclick={() => onshowinvisibleschange(!showInvisibles)}
>
  <Pilcrow size={14} />
</button>
<button
  class="toggle-btn"
  class:active={wordWrap}
  title="Toggle word wrap"
  onclick={() => onwordwrapchange(!wordWrap)}
>
  <WrapText size={14} />
</button>

<style>
  .toolbar-divider {
    width: 1px;
    height: 16px;
    background: var(--color-border);
    flex-shrink: 0;
  }
  .toggle-btn {
    background: none;
    border: 1px solid transparent;
    border-radius: 4px;
    color: var(--color-text-muted);
    padding: 2px 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
  }
  .toggle-btn.active {
    background: var(--color-accent-bg);
    color: var(--color-accent);
    border-color: var(--color-border);
  }
</style>
```

### Example 5: CSS Custom Properties for Invisibles

```css
/* Source: Extends existing app.css theme pattern */
:root {
  /* Invisible character markers (Phase 63 -- WHSP-03) */
  --color-invisible: rgba(139, 148, 158, 0.5);

  /* Trailing whitespace warning (Phase 63 -- D-12) */
  --color-trailing-ws-bg: rgba(248, 113, 113, 0.12);
}
```

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest (via `bun run test`) |
| Config file | vite.config.ts (test section) |
| Quick run command | `bun run test -- --reporter=verbose src/components/DiffPanel.test.ts` |
| Full suite command | `bun run test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VIEW-04 | Full file view renders all lines as continuous document (no hunk headers) | unit | `bun run test -- src/components/DiffPanel.test.ts` | Existing file, needs new tests |
| WHSP-02 | Staging buttons disabled when whitespace ignore active | unit | `bun run test -- src/components/DiffPanel.test.ts` | Existing file, needs new tests |
| WHSP-03 | Invisible characters rendered (spaces as dots, tabs as arrows) | unit | `bun run test -- src/components/DiffPanel.test.ts` | Existing file, needs new tests |
| DISP-02 | Word wrap toggles white-space CSS property | unit | `bun run test -- src/components/DiffPanel.test.ts` | Existing file, needs new tests |
| DISP-03 | All display preferences persist across sessions | unit | `bun run test -- src/lib/store.test.ts` | Existing file, needs new tests |

### Sampling Rate
- **Per task commit:** `bun run test -- src/components/DiffPanel.test.ts src/lib/store.test.ts`
- **Per wave merge:** `bun run test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- None -- existing test infrastructure covers all phase requirements. DiffPanel.test.ts and store.test.ts provide the test harness. New test cases need to be added for the new behaviors.

## Specific Technical Findings

### Lucide Icon Recommendations
All three icons confirmed available in `@lucide/svelte@^0.577.0` (verified via `node_modules/@lucide/svelte/dist/icons/`):

| Toggle | Icon | Import Name | Rationale |
|--------|------|-------------|-----------|
| Whitespace ignore (WS) | `space` | `Space` | Represents whitespace/spacing concept. Tags: "text, selection, letters, characters, font, typography" |
| Show invisibles | `pilcrow` | `Pilcrow` | Standard typographic symbol for invisible formatting marks. Tags: "paragraph, mark, paraph, blind, typography" |
| Word wrap | `wrap-text` | `WrapText` | Directly represents text wrapping. Tags: "words, lines, break, paragraph" |

### CSS Variable Recommendations for Invisible Characters

Based on existing theme pattern (muted colors at 50% opacity matching `--color-text-muted: #8b949e`):

- `--color-invisible: rgba(139, 148, 158, 0.5)` -- muted enough to not compete with code, visible enough to be useful
- `--color-trailing-ws-bg: rgba(248, 113, 113, 0.12)` -- uses the same red as `--color-diff-delete` but at very low alpha, subtle warning that doesn't overwhelm diff backgrounds

### Invisibles Rendering: Frontend-Only Recommendation

**Decision: Frontend-only.** Rationale:
1. The Rust backend returns raw file content as `line.content: String`. This is the canonical text.
2. Invisible character visualization is a display concern, not a data concern.
3. Modifying the backend would mean all consumers (future split view, future export) would need to handle substituted characters.
4. The frontend already does content slicing via `line.content.slice(span.start, span.end)` -- adding character substitution after slicing fits naturally.
5. No IPC overhead change -- same data, different rendering.

### View Mode / showFullFile Synchronization

Critical integration point: When user switches view mode to "full", the system must:
1. Set `viewMode = "full"` (UI state in DiffPanel)
2. Set `showFullFile = true` in LazyStore (backend parameter)
3. Trigger diff re-fetch (RepoView must re-call diff command)

Current code in `DiffPanel.handleViewModeChange()` only does step 1 and persists the view mode. Steps 2 and 3 require:
- DiffPanel updating `showFullFile` in LazyStore when viewMode changes to/from "full"
- A callback from DiffPanel to RepoView (e.g., `ondiffoptionschange`) to trigger re-fetch
- RepoView's `buildDiffOptions()` already reads `showFullFile` from LazyStore, so a re-fetch will pick up the change

Same pattern needed for `ignoreWhitespace` toggle -- changing it in the toolbar must trigger a re-fetch.

### Whitespace Ignore Staging Guard Implementation

The guard must disable buttons in three locations:
1. **DiffToolbar** -- "Stage File" / "Unstage File" buttons
2. **HunkView hunk toolbar** -- "Stage Hunk" / "Discard Hunk" / "Unstage Hunk" buttons
3. **HunkView line actions** -- "Stage Lines (N)" / "Discard Lines (N)" / "Unstage Lines (N)" buttons

All three locations already check `hunkOperationInFlight` for disabled state. The `ignoreWhitespace` flag follows the same pattern: pass as prop, check in disabled condition, add `title` attribute for tooltip text.

Tooltip text: "Staging is disabled while whitespace changes are ignored"

## Open Questions

1. **Diff re-fetch trigger mechanism**
   - What we know: DiffPanel owns view mode state, but RepoView owns the diff fetch logic. When view mode changes to/from "full", a re-fetch with updated `showFullFile` is required.
   - What's unclear: Whether to add a new callback prop (`ondiffoptionschange`) to DiffPanel, or whether RepoView should listen for LazyStore changes.
   - Recommendation: Add `ondiffoptionschange` callback prop to DiffPanel. RepoView already passes callbacks like `onhunkaction`. This is simpler than store observation and follows established patterns.

2. **Trailing whitespace detection scope**
   - What we know: D-12 says trailing whitespace gets a warning background. This means spaces/tabs at the end of `line.content` before any trailing newline.
   - What's unclear: Whether trailing whitespace should only be highlighted when "show invisibles" is active, or always.
   - Recommendation: Only show trailing whitespace warning when "show invisibles" is active. This keeps the behavior coupled to the toggle and avoids unexpected visual noise.

## Sources

### Primary (HIGH confidence)
- `src/components/diff/HunkView.svelte` -- Line rendering pattern, gutter implementation, merged span loop
- `src/components/diff/DiffToolbar.svelte` -- Segmented control pattern, toolbar layout
- `src/components/DiffPanel.svelte` -- State ownership, preference loading, view mode switching
- `src/lib/store.ts` -- LazyStore preference persistence pattern (15+ pairs)
- `src/lib/types.ts` -- DiffLine, FileDiff, DiffRequestOptions, ViewMode types
- `src/app.css` -- CSS custom property theme system
- `src-tauri/src/commands/diff.rs` -- `apply_request_options()`, `show_full_file` implementation
- `node_modules/@lucide/svelte/dist/icons/` -- Verified Space, Pilcrow, WrapText icon availability
- `.planning/phases/59-backend-data-model-diff-options/59-CONTEXT.md` -- Backend parameter decisions
- `.planning/phases/62-ui-refactor-component-structure/62-CONTEXT.md` -- Component structure decisions

### Secondary (MEDIUM confidence)
- [Lucide Icons](https://lucide.dev/icons/) -- Icon tags and availability
- [MDN white-space](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/white-space) -- CSS `pre-wrap` behavior
- [W3C CSSWG invisible characters discussion](https://github.com/w3c/csswg-drafts/issues/8874) -- Confirms no CSS-native invisible char rendering; span replacement is standard approach

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, all patterns established in prior phases
- Architecture: HIGH -- FullFileView is a simplified HunkView, toolbar toggles follow segmented control pattern, LazyStore pattern proven
- Pitfalls: HIGH -- identified from direct codebase analysis; byte offset issue is the most critical
- Integration: HIGH -- all connection points (DiffPanel -> DiffToolbar, DiffPanel -> DiffViewer -> FullFileView, RepoView -> DiffPanel) are well-understood

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (stable -- no external dependency changes expected)
