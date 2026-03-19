# Phase 36: Search UI - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Build a floating search overlay bar in CommitGraph with Cmd+F activation, match highlighting, prev/next navigation with auto-scroll, and Escape to close. Requirements: SRCH-01, SRCH-06, SRCH-07, SRCH-08, SRCH-09, SRCH-10. Backend `search_commits` command already implemented in Phase 35. This phase is frontend-only.

</domain>

<decisions>
## Implementation Decisions

### Match highlighting
- Two-tier highlighting: current match gets a strong highlight (amber/orange), other matches get a softer one (dimmed yellow)
- Background tint on the entire row — consistent with existing `selected` row pattern in CommitRow
- Non-matching rows are dimmed/faded when search is active (reduced opacity)
- Graph SVG elements (dots, rails, edges) are also dimmed for non-matching commits — full visual focus on matches

### Navigation & selection behavior
- Enter navigates to next match AND selects the commit (opening detail panel)
- Shift+Enter navigates to previous match AND selects
- Navigation wraps around: last match → Enter → first match, first match → Shift+Enter → last match
- Cmd+F while search is open: focus input + select all text (VS Code / Chrome behavior)
- Escape closes search bar, clears all highlights, restores full opacity — clean slate

### Empty & edge states
- Zero matches: counter displays "0 matches" — no special indicator or toast
- Empty input: graph looks normal — no dimming, no highlights until user starts typing
- Search only queries loaded commits in CommitCache — does not trigger loading all commits
- Navigation via Enter uses existing `scrollToOid` which handles lazy-loading (loops `loadMore` until OID found)

### Search bar appearance
- VS Code-style slim bar: compact, unobtrusive
- Positioned top-right corner of CommitGraph content area (absolutely positioned, below 24px column header)
- Does not push content down — floats over the graph
- Contains: input field + match counter ("3 of 17") + prev/next arrow buttons + close (×) button
- Prev/next navigation available via both clickable buttons AND Enter/Shift+Enter keyboard shortcuts
- Slides down from top edge when opened, slides up when closed (smooth animation)

### Claude's Discretion
- Exact CSS colors for highlight tints (amber/orange for current, yellow for others) and dimming opacity
- Search bar width (~300px suggested by PITFALLS.md P14)
- z-index value (10 suggested — above SVG overlay z-1, below modals z-9999)
- Slide animation implementation (CSS transition vs Svelte transition)
- Debounce implementation for live search (200ms per SRCH-11)
- Whether SearchBar is a separate component or inline in CommitGraph
- Cmd+F suppression strategy (JS preventDefault first, Rust fallback if needed per PITFALLS.md P7)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Search backend (Phase 35 output)
- `src-tauri/src/commands/history.rs` §66-133 — `search_commits_inner` and `search_commits` command. Returns `Vec<SearchResult>` with OID + match_types.
- `src-tauri/src/git/types.rs` §77-89 — `MatchType` enum and `SearchResult` struct (Rust side)
- `src/lib/types.ts` §49-54 — `MatchType` and `SearchResult` TypeScript types (already defined)

### CommitGraph (primary integration point)
- `src/components/CommitGraph.svelte` — Full component (1070 lines). SearchBar overlays inside this component. Key areas:
  - §569-594: `scrollToOid()` — export function, handles lazy-loading + centering. Use for match navigation.
  - §629-644: Initial scroll-to-HEAD effect
  - §647-1036: Template structure (header row + content area)
  - §712-718: Content area div (SearchBar positions absolutely within this)
  - §744-983: SVG overlay snippet (needs dimming support for non-matching rows)
  - §986-1000: VirtualList binding

### CommitRow (highlight integration)
- `src/components/CommitRow.svelte` — Row component with existing `selected` prop pattern. Extend with search match state for background tint highlighting + dimming.

### Keyboard shortcut pattern
- `src/App.svelte` §267-294 — Global `$effect` keydown handler. Cmd+F handler may go here or in CommitGraph.

### Cmd+F suppression risk
- `.planning/research/PITFALLS.md` §61-65 (P7) — WebView native find bar suppression. JS `preventDefault` may not work on macOS WKWebView. Documents Rust-level fallback.
- `.planning/research/PITFALLS.md` §107-111 (P14) — SearchBar placement recommendations (absolute, top-right, ~300px, z-index 10)

### Existing UI patterns
- `src/components/InputDialog.svelte` — Overlay pattern reference (fixed positioning, z-index, Escape handling)
- `src/lib/invoke.ts` — `safeInvoke<T>` for calling `search_commits`
- `src/lib/toast.svelte.ts` — Shared `$state` module pattern for cross-component state

### Requirements
- `.planning/REQUIREMENTS.md` §24-34 — SRCH-01, SRCH-06 through SRCH-10 requirements
- `.planning/REQUIREMENTS.md` §48-50 — SRCH-E01/E02/E03 (scoped prefixes, regex, persist state) explicitly deferred

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scrollToOid()`: Already handles lazy-loading + viewport centering — use directly for match navigation
- `safeInvoke<T>`: IPC wrapper — call `search_commits` with `{ path, query }`
- `SearchResult` / `MatchType` TS types: Already defined in `types.ts` from Phase 35
- `CommitRow.selected` prop pattern: Extend for search highlight states (match, current-match, dimmed)
- `$effect` keydown pattern: Reuse for Cmd+F binding
- `displayItems` derived array: Available in CommitGraph for matching OIDs to visible rows

### Established Patterns
- `$state` rune modules for cross-component state (e.g., `toast.svelte.ts`) — could use for search state if needed
- `dialogConfig` pattern: `$state` variable controls overlay visibility
- CSS custom properties for theme colors — search highlight colors should follow this convention
- `hunkOperationInFlight` boolean guard pattern — similar debounce/guard for search-in-flight

### Integration Points
- SearchBar renders inside CommitGraph content area (absolutely positioned, top-right)
- CommitRow needs new props: `isSearchMatch`, `isCurrentMatch`, `isSearchActive` (or similar)
- SVG overlay elements need dimming when search is active (opacity via CSS class or inline style)
- `oncommitselect` callback fires when Enter navigates to a match (selects the commit)
- Cmd+F keydown handler with `e.preventDefault()` to suppress native WebView find

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. VS Code search bar is the primary reference for look and feel.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 36-search-ui*
*Context gathered: 2026-03-18*
