<script lang="ts">
  import WelcomeScreen from './components/WelcomeScreen.svelte';
  import TabBar from './components/TabBar.svelte';
  import Toolbar from './components/Toolbar.svelte';
  import RepoView from './components/RepoView.svelte';
  import Toast from './components/Toast.svelte';
  import { safeInvoke } from './lib/invoke.js';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { getZoomLevel, setZoomLevel, getLeftPaneWidth, setLeftPaneWidth, getRightPaneWidth, setRightPaneWidth, getLeftPaneCollapsed, setLeftPaneCollapsed, getRightPaneCollapsed, setRightPaneCollapsed, getOpenRepo, setOpenRepo, getOpenTabs, setOpenTabs, getActiveTabId, setActiveTabId } from './lib/store.js';
  import type { TabInfo } from './lib/tab-types.js';
  import { createTabId } from './lib/tab-types.js';
  import { createRemoteState, type RemoteState } from './lib/remote-state.svelte.js';
  import { createUndoRedoState, type UndoRedoManager } from './lib/undo-redo.svelte.js';

  // Global layout state (D-01, D-02, D-03)
  let zoomLevel = $state(1);
  let leftPaneWidth = $state(220);
  let leftPaneCollapsed = $state(false);
  let rightPaneWidth = $state(240);
  let rightPaneCollapsed = $state(false);
  let isFullscreen = $state(false);

  // Tab state
  let tabs = $state<TabInfo[]>([]);
  let activeTabId = $state<string>('');

  // Per-tab state management (App.svelte owns creation, passes as props)
  interface TabState {
    remoteState: RemoteState;
    undoRedo: UndoRedoManager;
  }

  let tabStates = $state<Map<string, TabState>>(new Map());

  function getOrCreateTabState(tabId: string): TabState {
    let state = tabStates.get(tabId);
    if (!state) {
      state = { remoteState: createRemoteState(), undoRedo: createUndoRedoState() };
      tabStates.set(tabId, state);
    }
    return state;
  }

  // Active tab derived
  let activeTab = $derived(tabs.find(t => t.id === activeTabId));

  // Tab CRUD functions
  function addNewTab() {
    const existing = tabs.find(t => t.repoPath === null);
    if (existing) {
      activeTabId = existing.id;
      persistTabs();
      return;
    }
    const tab: TabInfo = {
      id: createTabId(),
      repoPath: null,
      repoName: 'New Tab',
      dirty: false,
    };
    tabs = [...tabs, tab];
    activeTabId = tab.id;
    persistTabs();
  }

  function closeOtherTabs(keepTabId: string) {
    const toClose = tabs.filter(t => t.id !== keepTabId);
    for (const t of toClose) {
      if (t.repoPath) {
        safeInvoke('close_repo', { path: t.repoPath }).catch(() => {});
      }
      tabStates.delete(t.id);
    }
    tabs = tabs.filter(t => t.id === keepTabId);
    activeTabId = keepTabId;
    persistTabs();
  }

  function closeAllTabs() {
    for (const t of tabs) {
      if (t.repoPath) {
        safeInvoke('close_repo', { path: t.repoPath }).catch(() => {});
      }
      tabStates.delete(t.id);
    }
    tabs = [];
    addNewTab();
  }

  async function showTabContextMenu(tabId: string, _event: MouseEvent) {
    const { Menu, MenuItem, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const tab = tabs.find(t => t.id === tabId);

    const menu = await Menu.new({
      items: [
        await MenuItem.new({
          text: 'Close Others',
          enabled: tabs.length > 1,
          action: () => { closeOtherTabs(tabId); },
        }),
        await MenuItem.new({
          text: 'Close All',
          action: () => { closeAllTabs(); },
        }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({
          text: 'Copy Path',
          enabled: !!tab?.repoPath,
          action: () => { if (tab?.repoPath) writeText(tab.repoPath); },
        }),
      ],
    });
    await menu.popup();
  }

  function openRepoInTab(tabId: string, path: string, name: string) {
    // TAB-10: Duplicate detection — switch to existing tab instead of creating duplicate
    const normalizedPath = path.replace(/\/+$/, '');
    const existing = tabs.find(t => t.repoPath && t.repoPath.replace(/\/+$/, '') === normalizedPath);
    if (existing) {
      activeTabId = existing.id;
      // Close the transient empty tab that triggered the open
      const triggerTab = tabs.find(t => t.id === tabId);
      if (triggerTab && !triggerTab.repoPath) {
        tabs = tabs.filter(t => t.id !== tabId);
        tabStates.delete(tabId);
      }
      persistTabs();
      return;
    }

    const tab = tabs.find(t => t.id === tabId);
    if (tab) {
      tab.repoPath = path;
      tab.repoName = name;
      persistTabs();
    }
  }

  function closeTab(tabId: string) {
    const idx = tabs.findIndex(t => t.id === tabId);
    if (idx === -1) return;
    const tab = tabs[idx];
    const repoPath = tab.repoPath;

    // Remove tab from array
    tabs = tabs.filter(t => t.id !== tabId);

    // Clean up per-tab state
    tabStates.delete(tabId);

    // If last tab closed, add empty tab
    if (tabs.length === 0) {
      addNewTab();
      return;
    }

    // If active tab was closed, switch to adjacent
    if (activeTabId === tabId) {
      const newIdx = Math.min(idx, tabs.length - 1);
      activeTabId = tabs[newIdx].id;
    }

    // Close repo on backend (after removing from DOM to prevent stale effects)
    if (repoPath) {
      safeInvoke('close_repo', { path: repoPath }).catch(() => {});
    }

    persistTabs();
  }

  function forceCloseTab(tabId: string) {
    const tab = tabs.find(t => t.id === tabId);
    if (!tab) return;
    const repoPath = tab.repoPath;

    // Remove tab from array first
    const idx = tabs.findIndex(t => t.id === tabId);
    tabs = tabs.filter(t => t.id !== tabId);

    // Clean up per-tab state
    tabStates.delete(tabId);

    if (tabs.length === 0) {
      addNewTab();
    } else if (activeTabId === tabId) {
      const newIdx = Math.min(idx, tabs.length - 1);
      activeTabId = tabs[newIdx].id;
    }

    // Force close on backend (D-11)
    if (repoPath) {
      safeInvoke('force_close_repo', { path: repoPath }).catch(() => {});
    }

    persistTabs();
  }

  // Tab persistence (debounced 500ms)
  let persistTimer: ReturnType<typeof setTimeout> | undefined;

  function persistTabs() {
    // Active tab ID saved immediately (no debounce) so it survives Cmd+Q
    setActiveTabId(activeTabId);
    if (persistTimer) clearTimeout(persistTimer);
    persistTimer = setTimeout(async () => {
      await setOpenTabs(tabs.map(t => ({ id: t.id, repoPath: t.repoPath, repoName: t.repoName })));
    }, 500);
  }

  // Restore on mount
  $effect(() => {
    (async () => {
      let restored = await getOpenTabs();

      // Legacy migration: if no tabs but old open_repo exists
      if (restored.length === 0) {
        const legacy = await getOpenRepo();
        if (legacy) {
          const id = createTabId();
          restored = [{ id, repoPath: legacy.path, repoName: legacy.name }];
          await setOpenTabs(restored);
          await setOpenRepo(null); // clear legacy key
        }
      }

      if (restored.length === 0) {
        addNewTab();
        return;
      }

      // Restore tabs
      const restoredTabs: TabInfo[] = [];
      for (const pt of restored) {
        const tab: TabInfo = { id: pt.id, repoPath: pt.repoPath, repoName: pt.repoName, dirty: false };
        if (pt.repoPath) {
          try {
            await safeInvoke('open_repo', { path: pt.repoPath });
          } catch {
            // Repo no longer exists -- skip this tab
            continue;
          }
          // Initial dirty check for restored tab
          try {
            const counts = await safeInvoke<{ staged: number; unstaged: number; conflicted: number }>('get_dirty_counts', { path: pt.repoPath });
            tab.dirty = counts.staged + counts.unstaged > 0;
          } catch {
            // non-fatal
          }
        }
        restoredTabs.push(tab);
      }

      if (restoredTabs.length === 0) {
        addNewTab();
        return;
      }

      tabs = restoredTabs;
      const savedActive = await getActiveTabId();
      activeTabId = restoredTabs.find(t => t.id === savedActive)?.id ?? restoredTabs[0].id;
    })();
  });

  // Zoom persistence
  $effect(() => {
    getZoomLevel().then((level) => { zoomLevel = level; });
  });

  // Pane persistence
  $effect(() => {
    getLeftPaneWidth().then((w) => { leftPaneWidth = w; });
    getRightPaneWidth().then((w) => { rightPaneWidth = w; });
    getLeftPaneCollapsed().then((c) => { leftPaneCollapsed = c; });
    getRightPaneCollapsed().then((c) => { rightPaneCollapsed = c; });
  });

  // Track fullscreen state (hide traffic-light padding when fullscreen)
  $effect(() => {
    const appWindow = getCurrentWindow();
    appWindow.isFullscreen().then((fs) => { isFullscreen = fs; });

    let unlisten: (() => void) | undefined;
    appWindow.onResized(async () => {
      isFullscreen = await appWindow.isFullscreen();
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  });

  // Apply zoom to document
  $effect(() => {
    document.documentElement.style.zoom = String(zoomLevel);
  });

  // Keyboard shortcuts
  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      // Tab shortcuts (Cmd/Ctrl key combinations)
      if (e.metaKey || e.ctrlKey) {
        // Cmd+T: New tab (TAB-02)
        if (e.key === 't' || e.key === 'T') {
          if (e.metaKey) {
            e.preventDefault();
            addNewTab();
            return;
          }
        }

        // Cmd+W: Close tab (TAB-03, D-10 graceful)
        // Cmd+Shift+W: Force close tab (D-11)
        if (e.key === 'w' || e.key === 'W') {
          if (e.metaKey) {
            e.preventDefault();
            if (e.shiftKey) {
              forceCloseTab(activeTabId);
            } else {
              closeTab(activeTabId);
            }
            return;
          }
        }

        // Cmd+1-9: Switch to tab by index (TAB-04)
        const num = parseInt(e.key);
        if (e.metaKey && num >= 1 && num <= 9) {
          e.preventDefault();
          const idx = num === 9 ? tabs.length - 1 : Math.min(num - 1, tabs.length - 1);
          if (idx >= 0 && idx < tabs.length) {
            activeTabId = tabs[idx].id;
          }
          return;
        }

        // Ctrl+Tab / Ctrl+Shift+Tab: Next/Prev tab (TAB-04)
        if (e.ctrlKey && e.key === 'Tab') {
          e.preventDefault();
          const cur = tabs.findIndex(t => t.id === activeTabId);
          if (e.shiftKey) {
            activeTabId = tabs[(cur - 1 + tabs.length) % tabs.length].id;
          } else {
            activeTabId = tabs[(cur + 1) % tabs.length].id;
          }
          return;
        }

        // Zoom: Cmd+/Cmd-/Cmd+0
        if (e.key === '=' || e.key === '+') {
          e.preventDefault();
          zoomLevel = +(Math.min(3, zoomLevel + 0.1).toFixed(1));
          setZoomLevel(zoomLevel);
        } else if (e.key === '-') {
          e.preventDefault();
          zoomLevel = +(Math.max(0.5, zoomLevel - 0.1).toFixed(1));
          setZoomLevel(zoomLevel);
        } else if (e.key === '0') {
          e.preventDefault();
          zoomLevel = 1;
          setZoomLevel(zoomLevel);
        }

        // Pane toggles: Cmd+J/Cmd+K
        if (e.key === 'j' || e.key === 'J') {
          e.preventDefault();
          leftPaneCollapsed = !leftPaneCollapsed;
          setLeftPaneCollapsed(leftPaneCollapsed);
        } else if (e.key === 'k' || e.key === 'K') {
          e.preventDefault();
          rightPaneCollapsed = !rightPaneCollapsed;
          setRightPaneCollapsed(rightPaneCollapsed);
        }
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  // Trigger resize recalculation when switching tabs so virtual lists
  // that were mounted under display:none get correct viewport dimensions.
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    activeTabId; // track
    requestAnimationFrame(() => {
      window.dispatchEvent(new Event('resize'));
    });
  });

  // Dirty detection: listen for repo-changed events and update tab.dirty for ALL tabs (TAB-07, D-04, D-05)
  $effect(() => {
    let unlisten: (() => void) | undefined;

    listen<string>('repo-changed', async (event) => {
      const repoPath = event.payload;
      const tab = tabs.find(t => t.repoPath === repoPath);
      if (!tab) return;
      try {
        const counts = await safeInvoke<{ staged: number; unstaged: number; conflicted: number }>('get_dirty_counts', { path: repoPath });
        tab.dirty = counts.staged + counts.unstaged > 0;
      } catch {
        // non-fatal -- keep previous dirty state
      }
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  });
</script>

<div class="flex flex-col h-screen" style="background: var(--color-bg);">
  <!-- LAYOUT-02: unified title bar + toolbar -->
  <div data-tauri-drag-region class="flex items-center flex-shrink-0" style="height: 32px; background: var(--color-surface); border-bottom: 1px solid var(--color-border); padding-left: {isFullscreen ? 0 : 78 / zoomLevel}px;">
    <TabBar
      {tabs}
      {activeTabId}
      onactivate={(id) => { activeTabId = id; persistTabs(); }}
      onclose={(id, force) => { if (force) forceCloseTab(id); else closeTab(id); }}
      onnew={addNewTab}
      oncontextmenu={showTabContextMenu}
      onauxclose={(id) => closeTab(id)}
      onreorder={(newTabs) => { tabs = newTabs; persistTabs(); }}
    />
    <div data-tauri-drag-region class="flex-1 h-full"></div>
    {#if activeTab?.repoPath}
      {@const activeState = getOrCreateTabState(activeTabId)}
      <Toolbar repoPath={activeTab.repoPath} remoteState={activeState.remoteState} undoRedo={activeState.undoRedo} />
    {/if}
  </div>

  {#each tabs as tab (tab.id)}
    <div style="display: {tab.id === activeTabId ? 'contents' : 'none'};">
      {#if tab.repoPath}
        {@const tabState = getOrCreateTabState(tab.id)}
        <RepoView
          repoPath={tab.repoPath}
          repoName={tab.repoName}
          remoteState={tabState.remoteState}
          undoRedo={tabState.undoRedo}
          {leftPaneWidth}
          {leftPaneCollapsed}
          {rightPaneWidth}
          {rightPaneCollapsed}
          onleftpanecollapsedchange={(c) => { leftPaneCollapsed = c; setLeftPaneCollapsed(c); }}
          onrightpanecollapsedchange={(c) => { rightPaneCollapsed = c; setRightPaneCollapsed(c); }}
          onleftpanewidthchange={(w) => { leftPaneWidth = w; setLeftPaneWidth(w); }}
          onrightpanewidthchange={(w) => { rightPaneWidth = w; setRightPaneWidth(w); }}
        />
      {:else}
        <WelcomeScreen {isFullscreen} onopen={(path, name) => openRepoInTab(tab.id, path, name)} />
      {/if}
    </div>
  {/each}

  <Toast />
</div>
