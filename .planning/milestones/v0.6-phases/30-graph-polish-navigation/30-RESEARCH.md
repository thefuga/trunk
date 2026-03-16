# Phase 30: Graph Polish & Navigation - Research

**Researched:** 2026-03-15
**Domain:** Svelte 5 frontend — CSS layout, virtual list scrolling, component communication
**Confidence:** HIGH

## Summary

Phase 30 addresses four requirements: graph container padding (GRAPH-01), graph column shrink/compress behavior (GRAPH-02), sidebar ref navigation with scroll-to-commit (GRAPH-03), and right pane auto-open on detail click (LAYOUT-01). All are frontend-only changes — no new Rust backend commands needed for GRAPH-01, GRAPH-02, or LAYOUT-01.

**GRAPH-03 requires a new Rust backend command** (`resolve_ref`) because the sidebar's `BranchInfo` and `RefLabel` types do not include commit OIDs. When the user clicks a branch/tag in the sidebar, the frontend needs to resolve the ref name to a commit OID, then find (or load) that commit's row in the virtual list and scroll to it.

**Primary recommendation:** Implement GRAPH-01 and GRAPH-02 as pure CSS changes, add a `resolve_ref` Tauri command for GRAPH-03, extend `BranchSidebar` with an `onrefnavigate` callback, and modify `App.svelte` to auto-open the right pane (LAYOUT-01) when handling commit/ref selection while the pane is collapsed.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GRAPH-01 | Commit graph has visible padding above first and below last commit row | VirtualList renders items into a content div; padding can be added to the virtual-list-content element or via a CSS padding-block on the viewport. The SVG overlay coordinates must remain aligned. |
| GRAPH-02 | Graph column can shrink below graph content width, compressing lanes | CommitRow.svelte line 53 sets `min-width: {Math.max(maxColumns, commit.column + 1) * LANE_WIDTH}px` on the graph column — this enforces minimum width. Removing or reducing this min-width + adding `overflow: hidden` allows shrinking. The graph column header (CommitGraph.svelte line 606) also uses `flex-shrink-0` and a fixed width. |
| GRAPH-03 | Clicking a branch or tag in sidebar scrolls graph to that commit's row | Sidebar `BranchInfo` has no OID; `RefLabel` (tags) has no OID. Need `resolve_ref` backend command. Frontend needs: resolve ref → OID, find OID index in `displayItems`, load more if not found, then call `listRef.scroll()` with 'center' align. VirtualList scroll supports 'top'/'bottom'/'auto'/'nearest' — no 'center' align exists, must be added or computed manually. |
| LAYOUT-01 | Right pane auto-opens when user clicks a commit, branch, or tag while pane is closed | `rightPaneCollapsed` is a `$state` in App.svelte (line 27). `handleCommitSelect()` at line 112 handles commit clicks. Simply set `rightPaneCollapsed = false` when a commit is selected and the pane is collapsed. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte | ^5.0.0 | UI framework | Project standard — uses runes ($state, $derived, $effect) |
| Tauri | 2.x | Desktop shell + Rust backend | Project standard |
| @humanspeak/svelte-virtual-list | ^0.4.2 | Vendored virtual list for commit graph | Already vendored in `src/components/virtual-list/` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| git2 | (Rust) | Git operations | For `resolve_ref` backend command |
| @tauri-apps/plugin-store | ^2.4.2 | Persist pane state | Already used for `rightPaneCollapsed` |

## Architecture Patterns

### Current Project Structure (relevant files)
```
src/
├── App.svelte                          # Layout orchestrator — pane state, commit selection
├── components/
│   ├── CommitGraph.svelte              # Graph container — VirtualList, SVG overlay, column layout
│   ├── CommitRow.svelte                # Individual row — graph column min-width lives here
│   ├── VirtualList.svelte              # Vendored virtual scroll — scroll() method
│   ├── BranchSidebar.svelte            # Left sidebar — branch/tag/stash lists
│   ├── BranchRow.svelte                # Individual branch/tag row
│   ├── BranchSection.svelte            # Collapsible section header
│   └── CommitDetail.svelte             # Right pane commit detail
├── lib/
│   ├── store.ts                        # Persistent preferences (pane widths, collapsed state)
│   ├── types.ts                        # TypeScript DTOs mirroring Rust
│   └── graph-constants.ts              # LANE_WIDTH, ROW_HEIGHT, pill constants
src-tauri/src/
├── commands/
│   ├── branches.rs                     # list_refs — BranchInfo (no OID), RefLabel (no OID)
│   └── history.rs                      # get_commit_graph, refresh_commit_graph
├── git/types.rs                        # Rust DTOs
└── lib.rs                              # Command registration
```

### Pattern 1: Callback Prop Communication (Sidebar → App → Graph)
**What:** Parent passes callback props down to children; children call them to trigger parent-level actions.
**When to use:** Cross-component actions like "sidebar click scrolls graph."
**Current example (stash select):**
```typescript
// App.svelte passes onstashselect to BranchSidebar
<BranchSidebar onstashselect={handleCommitSelect} />

// BranchSidebar calls it
onclick={() => onstashselect?.(stash.oid)}

// App.svelte's handleCommitSelect selects commit in graph
async function handleCommitSelect(oid: string) { ... }
```

**For GRAPH-03:** Add a new callback (e.g., `onrefnavigate`) that BranchSidebar calls when a branch/tag is clicked for navigation. App.svelte resolves ref → OID, finds the commit index, and tells CommitGraph to scroll.

### Pattern 2: VirtualList scroll() API
**What:** The virtual list exposes a `scroll()` method that scrolls to a specific item index.
**Current usage:**
```typescript
// CommitGraph.svelte line 57 — listRef type
let listRef = $state<{ scroll: (opts: { index: number; smoothScroll?: boolean; align?: string }) => Promise<void> } | null>(null);

// Scroll to HEAD on mount (line 579)
tick().then(() => listRef?.scroll({ index: headIdx, smoothScroll: false, align: 'top' }));
```
**Supported align values:** `'auto'`, `'top'`, `'bottom'`, `'nearest'` (no `'center'`).

**For GRAPH-03 centering:** Either:
1. Add `'center'` align to VirtualList/scrollCalculation.js, OR
2. Compute `scrollTop = itemTop - viewportHeight/2 + itemHeight/2` and call `viewport.scrollTo()` directly

**Recommendation:** Compute center scroll position manually — avoids modifying vendored virtual list code. CommitGraph already has access to `listRef` and display settings.

### Pattern 3: Pane State in App.svelte
**What:** `rightPaneCollapsed` is a reactive `$state` variable managed in App.svelte.
**Current code:**
```typescript
// App.svelte line 27
let rightPaneCollapsed = $state(false);

// Persisted to store
setRightPaneCollapsed(rightPaneCollapsed);

// Right pane visibility (line 353)
<div style="width: {rightPaneCollapsed ? 0 : rightPaneWidth}px; ...">
```

### Anti-Patterns to Avoid
- **Don't modify vendored VirtualList unless necessary:** It's complex, heavily optimized code. Prefer working around it (computing scroll position externally) over adding alignment modes.
- **Don't add OID to BranchInfo/RefLabel for all calls:** That would change the data shape for all consumers. A dedicated `resolve_ref` command is cleaner and only called when needed.
- **Don't make CommitGraph search for commits by ref name:** The graph has commits loaded by index/offset. It doesn't know which branch points to which commit. Resolution must happen via the backend.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Scroll-to-index in virtual list | Manual scroll position math | VirtualList's existing `scroll()` method | It already handles height caching, buffer sizing, scroll correction |
| Ref name → commit OID resolution | Frontend traversal of commit list | git2 `repo.revparse_single(ref_name)` in Rust | Accurate, handles all ref formats (branches, tags, abbreviated) |
| Pane state persistence | Custom localStorage wrapper | Existing `setRightPaneCollapsed()` in store.ts | Already wired, saves to trunk-prefs.json |

## Common Pitfalls

### Pitfall 1: SVG Overlay Misalignment After Adding Padding
**What goes wrong:** Adding padding to the virtual list content div shifts the item positions but the SVG overlay coordinates stay the same, causing dots/rails to not align with rows.
**Why it happens:** The SVG overlay is rendered inside the virtual-list-content div (via `overlaySnippet`). Its coordinate system starts at (0,0) of that div.
**How to avoid:** Add padding via the SVG overlay's `transform` attribute (translate Y) and extend the SVG height, rather than adding CSS padding that shifts the content container. Alternatively, add padding rows (empty spacer items) at the start/end of `displayItems`.
**Warning signs:** Graph dots don't line up with commit row text after the change.

### Pitfall 2: Graph Column min-width Prevents Shrinking
**What goes wrong:** CommitRow.svelte line 53 sets `min-width` on the graph column div to `Math.max(maxColumns, commit.column + 1) * LANE_WIDTH`. This means the column can never be narrower than the content.
**Why it happens:** The min-width was added to prevent lane content from being clipped. But GRAPH-02 explicitly wants clipping/compression.
**How to avoid:** Remove or reduce the `min-width` and add `overflow: hidden` to the graph column div. The SVG overlay already handles rendering — it clips naturally.
**Warning signs:** Graph column still won't shrink after changing the CSS width.

### Pitfall 3: Commit Not Loaded Yet When Navigating
**What goes wrong:** User clicks a branch in the sidebar whose commit is not in the loaded `displayItems` (it's beyond the current `offset`). The `findIndex()` returns -1.
**Why it happens:** The virtual list lazy-loads commits in batches of 200. A branch tip might be at commit #500+.
**How to avoid:** After resolving ref → OID, search `displayItems` for the OID. If not found, keep calling `loadMore()` until either found or `hasMore` is false. Then scroll to the found index.
**Warning signs:** Nothing happens when clicking a branch that points to an old commit.

### Pitfall 4: VirtualList scroll() Doesn't Support 'center' Alignment
**What goes wrong:** Calling `listRef.scroll({ index, align: 'center' })` doesn't center the item — `calculateScrollTarget` doesn't handle `'center'` and returns `null`.
**Why it happens:** The vendored virtual list only implements `'auto'`, `'top'`, `'bottom'`, `'nearest'`.
**How to avoid:** Compute the center scroll offset manually: `scrollTop = rowIndex * rowHeight - viewportHeight / 2 + rowHeight / 2`. Use the VirtualList's viewport element directly to set scrollTop, or extend the scroll calculation.
**Warning signs:** scrolling "works" but item appears at top/bottom of viewport instead of center.

### Pitfall 5: Right Pane Auto-Open Without Persisting State
**What goes wrong:** Right pane opens when a commit is clicked, but after app restart it opens collapsed (because the auto-open didn't persist).
**Why it happens:** `setRightPaneCollapsed(false)` isn't called during auto-open.
**How to avoid:** Call `setRightPaneCollapsed(false)` when auto-opening to persist the state change.
**Warning signs:** Pane opens on click but reverts to collapsed after restart.

## Code Examples

### GRAPH-01: Adding Padding to Graph Container
```typescript
// Option A: CSS padding on virtual-list-viewport
// In CommitGraph.svelte, the VirtualList renders into a container
// Add padding to the content area:

// In CommitGraph.svelte, wrap or style the VirtualList container
// The most reliable approach is adding padding via CSS on .virtual-list-content:
// :global(.virtual-list-content) { padding-block: 8px; }
//
// BUT this may misalign the SVG overlay. A safer approach:

// Option B: Adjust the SVG overlay's vertical offset
// The overlay already receives contentHeight and renders in position relative 
// to the virtual-list-content div. Add a CSS padding-top/padding-bottom 
// to the virtual-list-viewport, which shifts the scroll area without 
// affecting the overlay coordinate system.
```

### GRAPH-02: Removing Graph Column min-width
```svelte
<!-- CommitRow.svelte line 53 — CURRENT -->
<div class="relative z-[1] flex items-center flex-shrink-0" 
     style="width: {columnWidths.graph}px; min-width: {Math.max(maxColumns, commit.column + 1) * LANE_WIDTH}px;">
</div>

<!-- PROPOSED: Remove min-width, add overflow: hidden -->
<div class="relative z-[1] flex items-center flex-shrink-0 overflow-hidden" 
     style="width: {columnWidths.graph}px;">
</div>
```

Also update the resize handler in CommitGraph.svelte to allow smaller minimum:
```typescript
// CommitGraph.svelte line 100-106 — CURRENT minWidths
const minWidths: Record<keyof ColumnWidths, number> = {
  ref: 60,
  graph: Math.max(maxColumns, 1) * displaySettings.laneWidth, // enforces full width
  // ...
};

// PROPOSED: Allow graph to shrink to a small minimum
const minWidths: Record<keyof ColumnWidths, number> = {
  ref: 60,
  graph: 20, // allow arbitrary shrinking
  // ...
};
```

### GRAPH-03: Resolve Ref Backend Command
```rust
// src-tauri/src/commands/branches.rs — new command

#[tauri::command]
pub async fn resolve_ref(
    path: String,
    ref_name: String,
    state: State<'_, RepoState>,
) -> Result<String, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        resolve_ref_inner(&path, &ref_name, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())
}

fn resolve_ref_inner(
    path: &str,
    ref_name: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<String, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let obj = repo.revparse_single(ref_name).map_err(TrunkError::from)?;
    let commit = obj.peel_to_commit().map_err(TrunkError::from)?;
    Ok(commit.id().to_string())
}
```

### GRAPH-03: Sidebar Navigation Flow
```typescript
// BranchSidebar.svelte — add onrefnavigate callback
interface Props {
  repoPath: string;
  onrefreshed?: () => void;
  onstashselect?: (oid: string) => void;
  onrefnavigate?: (refName: string) => void;  // NEW
  refreshSignal?: number;
}

// BranchRow click for navigation (not checkout):
// Option: Double-click navigates, single-click still checks out
// OR: Add a navigation icon button on each row
// OR: Single click navigates, context menu for checkout
// Recommendation: Single click navigates (calls onrefnavigate),
// keeping checkout via context menu (already exists)

// App.svelte — handle navigation
async function handleRefNavigate(refName: string) {
  if (!repoPath) return;
  try {
    const oid = await safeInvoke<string>('resolve_ref', { 
      path: repoPath, refName 
    });
    // Auto-open right pane (LAYOUT-01)
    if (rightPaneCollapsed) {
      rightPaneCollapsed = false;
      setRightPaneCollapsed(false);
    }
    await handleCommitSelect(oid);
    // Tell CommitGraph to scroll to this OID
    // Need to expose scrollToOid on CommitGraph
  } catch (e) {
    // Ref might not resolve (deleted branch, etc.)
  }
}
```

### GRAPH-03: Scroll to OID in CommitGraph
```typescript
// CommitGraph.svelte — new public method or prop callback
export async function scrollToOid(oid: string) {
  let idx = displayItems.findIndex(c => c.oid === oid);
  
  // Load more commits until found or exhausted
  while (idx < 0 && hasMore) {
    await loadMore();
    // After loadMore, displayItems updates reactively
    idx = displayItems.findIndex(c => c.oid === oid);
  }
  
  if (idx < 0) return; // Commit not in history
  
  // Scroll to center
  await tick();
  if (listRef) {
    // Manual center calculation since VirtualList doesn't support 'center'
    const rowHeight = displaySettings.rowHeight;
    // Use scroll with 'auto' as approximate — or compute manually
    await listRef.scroll({ index: idx, smoothScroll: true, align: 'auto' });
  }
}
```

### LAYOUT-01: Right Pane Auto-Open
```typescript
// App.svelte — modify handleCommitSelect
async function handleCommitSelect(oid: string) {
  if (selectedCommitOid === oid) {
    clearCommit();
    return;
  }
  // Auto-open right pane if collapsed (LAYOUT-01)
  if (rightPaneCollapsed) {
    rightPaneCollapsed = false;
    setRightPaneCollapsed(false);
  }
  // ... existing logic
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| VirtualList from npm | Vendored VirtualList | v0.5 (Phase 20) | Full control over scroll behavior, overlay snippet |
| Unicode symbols | @lucide/svelte icons | v0.6 (Phase 27) | All UI elements use Lucide icons |
| Old graph column rendering | SVG overlay system | v0.5 (Phases 20-26) | Graph rendered as SVG overlay on virtual list |

## Open Questions

1. **Should branch/tag click navigate or checkout?**
   - What we know: Currently, clicking a local branch in sidebar performs checkout. Tags have no click handler (only context menu).
   - What's unclear: Should GRAPH-03 change the click behavior (navigate instead of checkout), or add a separate navigation affordance?
   - Recommendation: **Change branch click to navigate** (scroll to commit), keep checkout in context menu (already there). This matches the success criterion "clicking a branch name in the sidebar scrolls the commit graph." The existing checkout via context menu provides the action pathway.

2. **VirtualList center align — modify or work around?**
   - What we know: VirtualList scroll supports 'auto', 'top', 'bottom', 'nearest'. No 'center'.
   - What's unclear: Adding 'center' to vendored code vs computing scroll offset externally.
   - Recommendation: **Work around** by computing center offset: `rowIndex * rowHeight - viewportHeight / 2 + rowHeight / 2` and using the viewport's `scrollTo()`. This avoids touching vendored code.

3. **How to expose scrollToOid from CommitGraph?**
   - What we know: CommitGraph is rendered by App.svelte. Svelte 5 supports `export function` in components or `bind:this` to get component reference.
   - What's unclear: Best pattern for calling a method on a child component in Svelte 5.
   - Recommendation: Use `bind:this` on CommitGraph and call the exported `scrollToOid` method directly. This is already the pattern used with VirtualList (line 905: `bind:this={listRef}`).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest ^4.1.0 |
| Config file | Inferred from vite.config — no explicit vitest.config |
| Quick run command | `npm test` |
| Full suite command | `npm test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GRAPH-01 | Graph container has vertical padding | manual-only | N/A — CSS visual verification | N/A |
| GRAPH-02 | Graph column shrinks below content width | manual-only | N/A — CSS visual verification | N/A |
| GRAPH-03 | Ref click scrolls to commit row | unit (resolve_ref) + manual (scroll) | `cargo test resolve_ref` | ❌ Wave 0 |
| LAYOUT-01 | Right pane auto-opens on detail click | manual-only | N/A — UI state verification | N/A |

### Sampling Rate
- **Per task commit:** `npm test` (frontend), `cargo test` (Rust for resolve_ref)
- **Per wave merge:** `npm test && cargo test`
- **Phase gate:** Full suite green before verification

### Wave 0 Gaps
- [ ] `src-tauri/src/commands/branches.rs` — unit test for `resolve_ref_inner` (covers GRAPH-03 backend)
- GRAPH-01, GRAPH-02, LAYOUT-01 are CSS/state changes best verified manually — no automated tests needed

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `src/components/CommitGraph.svelte` — full file read (980 lines)
- Codebase analysis: `src/components/VirtualList.svelte` — full file read (702 lines)
- Codebase analysis: `src/components/BranchSidebar.svelte` — full file read (571 lines)
- Codebase analysis: `src/App.svelte` — full file read (370 lines)
- Codebase analysis: `src/components/CommitRow.svelte` — full file read (88 lines)
- Codebase analysis: `src/lib/store.ts` — full file read (146 lines)
- Codebase analysis: `src/lib/types.ts` — full file read (198 lines)
- Codebase analysis: `src-tauri/src/git/types.rs` — Rust DTOs (BranchInfo, RefLabel lack OID)
- Codebase analysis: `src-tauri/src/commands/branches.rs` — list_refs implementation
- Codebase analysis: `src-tauri/src/commands/history.rs` — graph loading implementation
- Codebase analysis: `src/components/virtual-list/utils/scrollCalculation.js` — scroll alignment logic

### Secondary (MEDIUM confidence)
- git2 crate: `revparse_single()` for ref resolution — standard API well-documented

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in use, no new dependencies needed
- Architecture: HIGH — patterns established by existing sidebar→app→graph communication (stash select)
- Pitfalls: HIGH — identified from direct codebase analysis (min-width enforcement, scroll alignment gaps, missing OIDs)
- GRAPH-03 complexity: MEDIUM — the load-more-until-found loop needs careful async handling

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable codebase, no external dependency changes)
