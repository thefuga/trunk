# Phase 30: Graph Polish & Navigation - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Frontend-only polish phase delivering four improvements to the commit graph: visible padding above/below the graph content (GRAPH-01), graph column shrink/compress behavior without horizontal scroll (GRAPH-02), sidebar branch/tag/stash click navigates (scrolls) to the commit row (GRAPH-03), and right pane auto-opens when a commit or ref selection would show detail (LAYOUT-01). One new Rust backend command needed (`resolve_ref`) for GRAPH-03 ref resolution.

</domain>

<decisions>
## Implementation Decisions

### Branch/Tag Click Behavior
- Branch click navigates (scrolls commit graph to that branch's commit) — replaces current checkout-on-click behavior
- Checkout remains in context menu only (already present from Phase 28)
- Tag click navigates (scrolls to the tag's commit) — tags previously had no click handler
- Stash click also navigates (scrolls to the stash's row in the graph) — extending existing stash selection behavior

### Graph Padding & Shrink
- 8px padding top and bottom on the graph content area — subtle breathing room without wasting space
- Implement padding via CSS on the virtual-list-viewport element (scroll area) — avoids SVG overlay coordinate misalignment
- Graph column minimum width when shrinking: 20px — allows arbitrary shrinking while preventing zero-width collapse
- Graph column header also shrinkable — header and row widths should match behavior for consistent column resize experience

### Right Pane Auto-Open
- Auto-open triggers when any commit/ref click would show detail AND the right pane is currently collapsed
- Auto-open persists across restarts: call `setRightPaneCollapsed(false)` to save state
- Always re-opens on detail selection — user can manually re-collapse, next click will open again
- Ref navigation (GRAPH-03 branch/tag click) also auto-opens right pane — clicking a branch selects the commit and shows its detail

### Claude's Discretion
- Exact scroll behavior for center alignment (manual computation vs 'auto' align — either is acceptable)
- Whether scrollToOid is exposed via `bind:this` on CommitGraph or via a callback prop
- Load-more loop implementation details when target commit is beyond loaded batch
- VirtualList scroll smoothness (smooth vs instant) on ref navigate

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CommitGraph.svelte`: Has `listRef` with `scroll({ index, smoothScroll, align })` method (line 57), `displayItems` array, `loadMore()` function, `hasMore` flag — all needed for GRAPH-03 scroll-to
- `BranchSidebar.svelte`: Has `onstashselect` callback prop pattern (line ~36) — extend with `onrefnavigate` callback for branch/tag/stash navigation
- `App.svelte`: Has `handleCommitSelect(oid)` at line ~112, `rightPaneCollapsed` $state at line 27, `setRightPaneCollapsed()` from store.ts
- `VirtualList.svelte` (vendored): Supports align: 'auto'/'top'/'bottom'/'nearest' — no 'center'; compute center manually
- `safeInvoke<T>`: Standard IPC wrapper for new `resolve_ref` command call
- `src-tauri/src/commands/branches.rs`: `open_repo_from_state()` helper already available — use same pattern for `resolve_ref_inner`

### Established Patterns
- Callback prop communication: Sidebar → App → Graph (stash select is the canonical example)
- inner-fn pattern: Tauri command calls `spawn_blocking` → `command_inner` for testability
- `bind:this` on child components to call exported methods (VirtualList line 905 in CommitGraph)
- `$derived.by()` for imperative reactive computations
- `revparse_single()` from git2 for ref → OID resolution

### Integration Points
- `CommitRow.svelte` line 53: `min-width: {Math.max(maxColumns, commit.column + 1) * LANE_WIDTH}px` — remove this to allow GRAPH-02 shrinking
- `CommitGraph.svelte` minWidths (line ~100): graph min-width enforces full content width — change to 20px for GRAPH-02
- `App.svelte` handleCommitSelect: Add right pane auto-open here (LAYOUT-01)
- `lib.rs` command registration: Register new `resolve_ref` command

</code_context>

<specifics>
## Specific Ideas

- Research already contains detailed code examples for all four requirements — use as direct implementation guides
- SVG overlay misalignment is a known pitfall (see RESEARCH.md Pitfall 1) — prefer CSS on viewport over padding content div
- VirtualList 'center' align is not supported — compute: `rowIndex * rowHeight - viewportHeight / 2 + rowHeight / 2`
- Research example shows `resolve_ref_inner` using `repo.revparse_single(ref_name).peel_to_commit()` — direct and accurate

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>
