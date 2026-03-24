# Phase 45: Frontend Tab Architecture - Research

**Researched:** 2026-03-23
**Domain:** Svelte 5 multi-tab UI architecture, per-component state isolation, keep-alive DOM pattern
**Confidence:** HIGH

## Summary

This phase transforms the current single-repo App.svelte into a multi-tab application where each tab hosts an independent repository view. The primary architectural challenge is extracting the ~30 `$state` variables and ~15 handler functions from App.svelte into a new RepoView component, then rendering multiple instances simultaneously using a keep-alive (display:none) pattern.

The user has locked the keep-alive strategy (D-08), meaning all tabs stay mounted in the DOM with CSS `display: none` when inactive. This is the simplest approach for preserving scroll position, open diffs, and transient UI state -- but requires converting the two global singletons (`remoteState` and `undoRedoState`) into per-tab scoped state. Svelte 5's `$state` proxy objects combined with prop-passing (not context API) are the right tool since each RepoView instance naturally creates its own closure scope.

**Primary recommendation:** Extract RepoView from App.svelte, render one per tab with keep-alive, use factory functions for per-tab `remoteState`/`undoRedoState`, persist tab list to LazyStore, and add keyboard shortcuts at the App level.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Zoom level is global -- Cmd+/- changes zoom for the entire app, not per-tab.
- **D-02:** Pane widths (sidebar, right panel) are global -- resizing applies across all tabs.
- **D-03:** Pane collapsed state is global -- collapsing the sidebar collapses it for all tabs.
- **D-04:** "Dirty" means staged + unstaged -- any modified/added/deleted file (staged or unstaged) shows the dot badge on the tab.
- **D-05:** Detection via watcher events -- the fs watcher already emits `repo-changed` events per repo path. Listen to those and update a dirty flag per tab. No polling needed.
- **D-06:** Scroll only -- tabs keep their natural width, and the tab bar becomes horizontally scrollable when tabs don't fit. No shrinking.
- **D-07:** No tab limit -- users can open as many tabs as they want.
- **D-08:** Keep-alive (hidden) -- all mounted tabs stay in the DOM with `display: none` when inactive. Preserves scroll position, open diffs, and all transient UI state across tab switches.
- **D-09:** Restore selected commit -- when switching back to a tab, the previously selected commit remains selected (natural consequence of keep-alive).
- **D-10:** Normal tab close (X button) = graceful -- let running remote ops finish naturally (Phase 44 D-02).
- **D-11:** Force close (Shift+click X) = cancel running op via SIGTERM before cleanup (Phase 44 D-03).
- **D-12:** Remote-progress events already carry repo path in payload (Phase 44 D-04) -- frontend filters by active tab's repo path.

### Claude's Discretion
- How to structure per-tab state (component extraction pattern, context API vs props)
- How to isolate `remoteState` and `undoRedoState` per tab (currently global singletons)
- Tab bar component implementation details (scrolling mechanism, active tab indicator styling)
- Keyboard shortcut handler architecture for Cmd+T/W/1-9/Ctrl+Tab

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TAB-01 | User can open multiple repositories as separate tabs in a single window | RepoView extraction + tab manager state in App.svelte; keep-alive DOM rendering |
| TAB-02 | User can create a new tab via Cmd+T, which shows the splash/project picker | Keyboard handler in App.svelte; new tab = push `{ id, repoPath: null }` to tabs array; WelcomeScreen reuse |
| TAB-03 | User can close a tab via Cmd+W or the X button on the tab | TabBar X button calls close handler; Cmd+W in keydown handler; close_repo/force_close_repo integration |
| TAB-04 | User can switch tabs via Cmd+1-9 and Ctrl+Tab/Ctrl+Shift+Tab | Keyboard handler in App.svelte; index-based (Cmd+N) and relative (Ctrl+Tab) switching |
| TAB-05 | Each tab has fully independent state (graph, staging, diffs, selection, rebase/merge) | RepoView component owns all per-repo $state; factory functions for remoteState/undoRedoState |
| TAB-06 | Open tabs and active tab are persisted and restored on app relaunch | LazyStore schema change: `open_tabs[]` + `active_tab_id`; restore on mount |
| TAB-07 | Background tabs with uncommitted changes show a dirty indicator (dot badge) | Per-tab dirty flag driven by `repo-changed` watcher events + `get_dirty_counts` backend command |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **CSS colors:** Never inline colors -- always use CSS custom properties from the theme (`var(--color-*)`)
- **CSS layout:** Never fight layout with positioning hacks -- use grid/flexbox for natural flow
- **Git operations:** All git operations through git2 crate, no shelling out
- **Frontend stack:** Svelte 5 runes (`$state`, `$derived`, `$effect`), TypeScript strict, Tailwind CSS 4
- **Backend calls:** All via `safeInvoke` from `src/lib/invoke.ts`
- **Icons:** Lucide icons via `@lucide/svelte`
- **Testing:** `bun run test` runs vitest (`src/**/*.test.ts`)

## Architecture Patterns

### Recommended Component Structure

```
src/
  App.svelte                    # Tab manager: tabs[], activeTabId, keyboard shortcuts, global layout state
  components/
    TabBar.svelte               # Full rewrite: renders tab list, scrollable overflow, dirty badges, close buttons
    RepoView.svelte             # NEW: extracted from App.svelte -- all per-repo state and UI
    WelcomeScreen.svelte        # Reused as-is for new/empty tabs
    Toolbar.svelte              # Receives per-tab remoteState + undoRedoState as props (no more global import)
    PullDropdown.svelte         # Receives per-tab remoteState as prop (no more global import)
    CommitForm.svelte           # Receives per-tab clearRedoStack as prop
    CommitGraph.svelte          # Receives per-tab clearRedoStack as prop
    ... (other components unchanged)
  lib/
    remote-state.svelte.ts      # CHANGED: export factory function instead of singleton
    undo-redo.svelte.ts         # CHANGED: export factory function instead of singleton
    store.ts                    # CHANGED: add tab persistence (open_tabs, active_tab)
    tab-types.ts                # NEW: TabInfo interface, tab ID generation
```

### Pattern 1: Per-Tab State via Component Instance Closure

**What:** Each RepoView instance creates its own `remoteState` and `undoRedoState` via factory functions. The state lives in the component's closure scope, not in a module-level singleton.

**When to use:** When state must be isolated per component instance but is currently a global singleton.

**Why not context API:** Context would work but adds indirection. Since RepoView already receives `repoPath` as a prop, passing the per-tab state objects down as props is simpler, more explicit, and avoids the "invisible dependency" pitfall. The component tree is shallow (App -> RepoView -> Toolbar/StagingPanel/etc.), so prop drilling is minimal.

**Example:**

```typescript
// src/lib/remote-state.svelte.ts — CHANGED from singleton to factory
import type { TrunkError } from './invoke.js';

export interface RemoteState {
  isRunning: boolean;
  progressLine: string;
  error: TrunkError | null;
}

export function createRemoteState(): RemoteState {
  return $state({
    isRunning: false,
    progressLine: '',
    error: null,
  });
}
```

```typescript
// src/lib/undo-redo.svelte.ts — CHANGED from singleton to factory
interface UndoEntry {
  subject: string;
  body: string | null;
}

export interface UndoRedoState {
  redoStack: UndoEntry[];
}

export function createUndoRedoState() {
  const state: UndoRedoState = $state({ redoStack: [] });

  return {
    state,
    push(entry: UndoEntry) {
      state.redoStack = [...state.redoStack, entry];
    },
    pop(): UndoEntry | undefined {
      if (state.redoStack.length === 0) return undefined;
      const entry = state.redoStack[state.redoStack.length - 1];
      state.redoStack = state.redoStack.slice(0, -1);
      return entry;
    },
    clear() {
      state.redoStack = [];
    },
  };
}
```

```svelte
<!-- RepoView.svelte -->
<script lang="ts">
  import { createRemoteState } from '../lib/remote-state.svelte.js';
  import { createUndoRedoState } from '../lib/undo-redo.svelte.js';

  interface Props {
    repoPath: string | null;
    onopen: (path: string, name: string) => void;
    onclose: () => void;
    // ... other tab-level callbacks
  }

  let { repoPath, onopen, onclose }: Props = $props();

  // Per-tab state — each RepoView instance gets its own
  const remoteState = createRemoteState();
  const undoRedo = createUndoRedoState();

  // ... all the $state variables currently in App.svelte
</script>
```

### Pattern 2: Keep-Alive Tab Rendering

**What:** All tabs are always rendered in the DOM. The active tab is visible; others have `display: none`. This preserves scroll position, open diffs, selected commits, and all transient UI state.

**When to use:** Always, per D-08 locked decision.

**Implementation:**

```svelte
<!-- App.svelte — simplified sketch -->
{#each tabs as tab (tab.id)}
  <div style="display: {tab.id === activeTabId ? 'contents' : 'none'};">
    {#if tab.repoPath}
      <RepoView
        repoPath={tab.repoPath}
        onopen={(path, name) => openRepoInTab(tab.id, path, name)}
        onclose={() => closeTab(tab.id)}
      />
    {:else}
      <WelcomeScreen onopen={(path, name) => openRepoInTab(tab.id, path, name)} />
    {/if}
  </div>
{/each}
```

**Key detail:** Use `display: contents` (not `display: block`) for the active tab wrapper so it doesn't add an extra DOM layer that could break flex/grid layout.

**Caveat:** Hidden tabs still have active `$effect` hooks (event listeners, timers). The `repo-changed` listener in each RepoView already filters by `repoPath`, so this is safe. The `remote-progress` listener in Toolbar also filters by path. No additional guard needed.

### Pattern 3: Tab Manager State in App.svelte

**What:** App.svelte becomes a thin orchestrator: it manages the tab list, active tab, keyboard shortcuts, and global layout state (zoom, pane widths). All repo-specific logic moves to RepoView.

**Tab data model:**

```typescript
// src/lib/tab-types.ts
export interface TabInfo {
  id: string;          // unique ID (crypto.randomUUID())
  repoPath: string | null;   // null = empty/welcome tab
  repoName: string;
  dirty: boolean;      // has staged+unstaged changes
}
```

**App.svelte state (sketch):**

```typescript
let tabs = $state<TabInfo[]>([]);
let activeTabId = $state<string>('');

// Global layout (D-01, D-02, D-03)
let zoomLevel = $state(1);
let leftPaneWidth = $state(220);
let leftPaneCollapsed = $state(false);
let rightPaneWidth = $state(240);
let rightPaneCollapsed = $state(false);
```

### Pattern 4: Tab Persistence Schema

**What:** Replace `open_repo` single-value persistence with array-based tab persistence.

**Store keys (in `trunk-prefs.json`):**

```typescript
const TABS_KEY = 'open_tabs';
const ACTIVE_TAB_KEY = 'active_tab_id';

interface PersistedTab {
  id: string;
  repoPath: string | null;
  repoName: string;
}
```

**Restore flow on app launch:**
1. Load `open_tabs` from store. If empty/null, show single empty tab (WelcomeScreen).
2. Load `active_tab_id`. If not found in tabs, default to first tab.
3. For each tab with a `repoPath`, call `open_repo` on the backend to initialize watcher + cache.
4. The old `open_repo` key should be migrated: if `open_tabs` is empty but `open_repo` exists, create a single tab from it, then delete `open_repo`.

### Pattern 5: Dirty Detection Per Tab

**What:** Each tab tracks whether its repo has uncommitted changes. The dirty flag drives the dot badge on the tab.

**How:** A single global `repo-changed` listener in App.svelte (not per-RepoView) updates the dirty flag for the matching tab:

```typescript
listen<string>('repo-changed', async (event) => {
  const tab = tabs.find(t => t.repoPath === event.payload);
  if (!tab) return;
  try {
    const counts = await safeInvoke<DirtyCounts>('get_dirty_counts', { path: event.payload });
    tab.dirty = counts.staged + counts.unstaged > 0;
  } catch {
    // non-fatal
  }
});
```

This is efficient: one listener handles all tabs, and it reuses the existing `get_dirty_counts` backend command. Per D-04, dirty means staged + unstaged > 0.

### Anti-Patterns to Avoid

- **Svelte 5 context API for per-tab state:** While `createContext`/`setContext`/`getContext` exists in Svelte 5.53+, it adds invisible coupling. Props are more explicit for this shallow component tree.
- **Destroying/recreating tabs on switch:** Would lose scroll position, open diffs, and transient state. D-08 explicitly requires keep-alive.
- **Module-level singletons for per-tab state:** The current `remoteState` and `undoRedoState` are module-level `$state` objects shared across all importers. With multiple tabs, they would cross-contaminate.
- **Using `{#if active}` for tab visibility:** Svelte's `{#if}` destroys components when the condition becomes false. Must use CSS `display: none` instead.
- **Adding tab ID to every backend call:** The backend already uses repo path as its key. Tab ID is a frontend-only concept.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tab ID generation | Custom incrementing counter | `crypto.randomUUID()` | Universally unique, survives app restarts, no collision risk |
| Tab persistence | Custom file I/O | `@tauri-apps/plugin-store` LazyStore (already used) | Already in the project, handles atomic writes, JSON serialization |
| Horizontal scroll overflow | Custom scroll math | Native CSS `overflow-x: auto` on tab bar container | Browser handles scroll, touch, and momentum natively |
| Keyboard shortcut detection | Raw keyCode parsing | `e.metaKey`/`e.ctrlKey` + `e.key` (existing pattern) | Already used in App.svelte for zoom/pane shortcuts |

## Common Pitfalls

### Pitfall 1: Effects Running in Hidden Tabs
**What goes wrong:** Hidden (display:none) tabs still have active `$effect` hooks. Event listeners fire, timers tick, and state updates happen for invisible tabs.
**Why it happens:** Svelte does not pause effects when a component's DOM parent is hidden via CSS.
**How to avoid:** Already handled -- the existing `repo-changed` listener in App.svelte filters by `repoPath`. The `remote-progress` listener in Toolbar filters by `event.payload.path === repoPath`. No new guards needed for existing code. But any NEW effects added must also filter by repo path.
**Warning signs:** Console logs firing from components in inactive tabs; unexpected state changes after switching tabs.

### Pitfall 2: Memory Growth with Many Open Tabs
**What goes wrong:** Each keep-alive tab holds its full commit graph, virtual list DOM, staging state, and diff data in memory.
**Why it happens:** The keep-alive pattern means nothing is freed until the tab is closed.
**How to avoid:** This is acceptable for the D-07 "no tab limit" decision. The backend commit cache (`CommitCache`) is already per-repo keyed and cleaned up on `close_repo`. The frontend virtual list only renders visible rows. In practice, users rarely open more than 10-15 tabs.
**Warning signs:** Memory usage growing linearly with tab count. If this becomes an issue in the future, consider an LRU eviction strategy for background tab commit data.

### Pitfall 3: Race Condition on Tab Close + Backend Cleanup
**What goes wrong:** Closing a tab calls `close_repo` (or `force_close_repo`), but the RepoView might still have in-flight `safeInvoke` calls that reference the now-closed repo.
**Why it happens:** `close_repo` removes watcher state and commit cache. Subsequent `get_status` or `diff_*` calls for that repo path will fail.
**How to avoid:** Remove the tab from the `tabs` array (which unmounts the RepoView via keep-alive) BEFORE calling `close_repo`. Once unmounted, no more effects or handlers fire.
**Warning signs:** "Repo not found" errors in console after closing a tab.

### Pitfall 4: Tab Bar Competing with Drag Region
**What goes wrong:** The title bar is 32px with `data-tauri-drag-region`. Tab buttons inside it must NOT have the drag region attribute, or clicks on tabs will be swallowed by the window drag handler.
**Why it happens:** Tauri's drag region intercepts mouse events. Any interactive element inside it needs to be excluded.
**How to avoid:** The current TabBar.svelte already has `data-tauri-drag-region` on its container div. The close button works because buttons naturally exclude drag. For the new multi-tab bar, keep `data-tauri-drag-region` on empty space only, not on tab buttons.
**Warning signs:** Clicking a tab doesn't switch tabs but instead drags the window.

### Pitfall 5: Stale Persistence on Crash
**What goes wrong:** If the app crashes, the persisted tab state may reference repos that are no longer valid.
**Why it happens:** Tab state is saved to LazyStore, but the backend state (watchers, cache) is in-memory only.
**How to avoid:** On app launch, when restoring tabs, wrap each `open_repo` call in try/catch. If it fails (repo moved/deleted), mark the tab as broken or remove it. The existing WelcomeScreen `openPath` already handles `open_repo` failures gracefully.
**Warning signs:** Blank/errored tabs after relaunch.

### Pitfall 6: Traffic Light Overlap with Tab Bar
**What goes wrong:** macOS traffic lights (close/minimize/maximize) overlay at left edge. The tab bar must not place tabs underneath them.
**Why it happens:** Tauri config `titleBarStyle: "Overlay"` with `hiddenTitle: true` places traffic lights at ~(12, 16) occupying roughly 68px width.
**How to avoid:** The current App.svelte already uses `padding-left: {78 / zoomLevel}px` to reserve space. The tab bar must respect this same padding. Place the "new tab" (+) button or first tab after the 78px offset.
**Warning signs:** Tabs appearing under the traffic lights; traffic light clicks activating a tab instead.

## Code Examples

### Tab Manager Keyboard Shortcuts

```typescript
// In App.svelte $effect — extends existing keyboard handler
function handleKeydown(e: KeyboardEvent) {
  // Existing shortcuts (zoom, pane toggle) first...

  if (e.metaKey || e.ctrlKey) {
    // Cmd+T: New tab
    if (e.key === 't' || e.key === 'T') {
      e.preventDefault();
      addNewTab();
      return;
    }

    // Cmd+W: Close active tab
    if (e.key === 'w' || e.key === 'W') {
      e.preventDefault();
      closeActiveTab(e.shiftKey); // shiftKey -> force close (D-11)
      return;
    }

    // Cmd+1-9: Switch to tab by index
    const num = parseInt(e.key);
    if (num >= 1 && num <= 9) {
      e.preventDefault();
      const idx = Math.min(num - 1, tabs.length - 1);
      activeTabId = tabs[idx].id;
      return;
    }
  }

  // Ctrl+Tab / Ctrl+Shift+Tab: Cycle tabs
  if (e.ctrlKey && e.key === 'Tab') {
    e.preventDefault();
    const currentIdx = tabs.findIndex(t => t.id === activeTabId);
    if (e.shiftKey) {
      // Previous tab
      activeTabId = tabs[(currentIdx - 1 + tabs.length) % tabs.length].id;
    } else {
      // Next tab
      activeTabId = tabs[(currentIdx + 1) % tabs.length].id;
    }
    return;
  }
}
```

### Tab Persistence (store.ts additions)

```typescript
// src/lib/store.ts — new functions
const TABS_KEY = 'open_tabs';
const ACTIVE_TAB_KEY = 'active_tab_id';

export interface PersistedTab {
  id: string;
  repoPath: string | null;
  repoName: string;
}

export async function getOpenTabs(): Promise<PersistedTab[]> {
  return (await store.get<PersistedTab[]>(TABS_KEY)) ?? [];
}

export async function setOpenTabs(tabs: PersistedTab[]): Promise<void> {
  await store.set(TABS_KEY, tabs);
  await store.save();
}

export async function getActiveTabId(): Promise<string | null> {
  return (await store.get<string>(ACTIVE_TAB_KEY)) ?? null;
}

export async function setActiveTabId(id: string): Promise<void> {
  await store.set(ACTIVE_TAB_KEY, id);
  await store.save();
}
```

### Tab Bar Scrollable Overflow (CSS)

```svelte
<!-- TabBar.svelte — scroll container -->
<div
  class="flex items-center h-full"
  style="overflow-x: auto; overflow-y: hidden; scrollbar-width: none;"
>
  {#each tabs as tab (tab.id)}
    <button
      class="tab-item"
      class:active={tab.id === activeTabId}
      onclick={() => onactivate(tab.id)}
    >
      <span class="truncate max-w-[200px]">{tab.repoName || 'New Tab'}</span>
      {#if tab.dirty}
        <span class="dirty-dot"></span>
      {/if}
      <button
        class="close-btn"
        onclick|stopPropagation={(e) => onclose(tab.id, e.shiftKey)}
        aria-label="Close tab"
      >
        <X size={12} />
      </button>
    </button>
  {/each}
  <button class="new-tab-btn" onclick={onnew} aria-label="New tab">
    <Plus size={14} />
  </button>
</div>
```

```css
/* Hide scrollbar but allow scroll */
.tab-bar::-webkit-scrollbar { display: none; }

.dirty-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-accent);
  flex-shrink: 0;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 10px;
  height: 100%;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
  border-right: 1px solid var(--color-border);
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
  background: none;
  border: none;
}

.tab-item.active {
  color: var(--color-text);
  background: var(--color-bg);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Svelte stores (`writable()`) | Svelte 5 runes (`$state()`) | Svelte 5.0 (2024) | All state in this project already uses runes |
| `setContext`/`getContext` with string keys | `createContext<T>()` typed pair | Svelte 5.40 (2025) | Available but not recommended here -- props are simpler for this tree depth |
| `svelte:keep-alive` proposal | CSS `display: none` | Still no native keep-alive in Svelte 5 | Must use CSS approach for keep-alive tabs |

## Open Questions

1. **Display property for active tab wrapper**
   - What we know: `display: none` hides inactive tabs. Active tab needs a wrapper that doesn't affect layout.
   - What's unclear: Whether `display: contents` is universally safe in this context or if it causes issues with flex layout in specific browsers.
   - Recommendation: Test with `display: contents` first. Fallback to `display: flex; flex-direction: column; flex: 1; overflow: hidden;` with explicit height if needed.

2. **Tab persistence debounce**
   - What we know: Every tab switch, open, and close should persist to LazyStore.
   - What's unclear: Whether rapid tab switches cause excessive disk I/O.
   - Recommendation: Debounce `setOpenTabs`/`setActiveTabId` calls by 500ms. LazyStore already batches writes, but debouncing at the caller reduces churn.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest 4.1.0 |
| Config file | `vite.config.ts` (test section) |
| Quick run command | `bun run test` |
| Full suite command | `bun run test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TAB-01 | Multiple tabs open simultaneously | manual-only | N/A (UI rendering) | N/A |
| TAB-02 | Cmd+T creates new tab with WelcomeScreen | manual-only | N/A (keyboard + UI) | N/A |
| TAB-03 | Cmd+W / X closes tab | manual-only | N/A (keyboard + UI) | N/A |
| TAB-04 | Cmd+1-9, Ctrl+Tab switching | manual-only | N/A (keyboard + UI) | N/A |
| TAB-05 | Per-tab state isolation (remoteState, undoRedo) | unit | `bun run test -- src/lib/remote-state.svelte.test.ts` | No -- Wave 0 |
| TAB-06 | Tab persistence/restore | unit | `bun run test -- src/lib/store.test.ts` | No -- Wave 0 |
| TAB-07 | Dirty indicator logic | unit | `bun run test -- src/lib/tab-dirty.test.ts` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `bun run test`
- **Per wave merge:** `bun run test && bun run check`
- **Phase gate:** Full suite green + manual verification of all 7 TAB requirements

### Wave 0 Gaps
- [ ] `src/lib/remote-state.svelte.test.ts` -- test that createRemoteState returns independent instances
- [ ] `src/lib/undo-redo.svelte.test.ts` -- test that createUndoRedoState instances are independent (push/pop/clear)
- [ ] Tab persistence helpers can be unit tested if extracted as pure functions (serialize/deserialize tab list)

## Sources

### Primary (HIGH confidence)
- Direct code reading: `src/App.svelte`, `src/components/TabBar.svelte`, `src/lib/store.ts`, `src/lib/remote-state.svelte.ts`, `src/lib/undo-redo.svelte.ts`, `src/lib/invoke.ts`
- Direct code reading: `src-tauri/src/commands/repo.rs`, `src-tauri/src/state.rs`, `src-tauri/tauri.conf.json`
- [Svelte 5 Context docs](https://svelte.dev/docs/svelte/context) -- setContext/getContext/createContext API
- [Svelte keep-alive discussion](https://github.com/sveltejs/svelte/issues/6040) -- confirms no native keep-alive; CSS display:none is the standard pattern

### Secondary (MEDIUM confidence)
- [Runes and Global state: do's and don'ts](https://mainmatter.com/blog/2025/03/11/global-state-in-svelte-5/) -- factory pattern for per-instance $state
- [Svelte maintaining state of hidden components](https://umaranis.com/2025/09/09/svelte-maintaining-state-of-hidden-components/) -- display:none preserves component state

### Tertiary (LOW confidence)
- None -- all findings verified against source code and official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in the project; no new dependencies needed
- Architecture: HIGH -- patterns verified against Svelte 5 docs and existing codebase patterns
- Pitfalls: HIGH -- derived from direct code analysis of event listeners, backend state, and DOM layout

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable -- no external dependency changes expected)
