<script lang="ts">
  import VirtualList from './VirtualList.svelte';
  import { tick, untrack } from 'svelte';
  import { safeInvoke, type TrunkError } from '../lib/invoke.js';
  import { showToast } from '../lib/toast.svelte.js';
  import { clearRedoStack } from '../lib/undo-redo.svelte.js';
  import type { GraphCommit, GraphResponse, EdgeType, StashEntry } from '../lib/types.js';
  import { getColumnWidths, setColumnWidths, type ColumnWidths, getColumnVisibility, setColumnVisibility, type ColumnVisibility } from '../lib/store.js';
  import { DEFAULT_GRAPH_SETTINGS, PILL_HEIGHT, PILL_PADDING_X, PILL_FONT_SIZE, PILL_GAP, BADGE_HEIGHT, BADGE_FONT_SIZE, ICON_WIDTH, ICON_GAP, COLUMN_PADDING_X } from '../lib/graph-constants.js';
  import { buildGraphData } from '../lib/active-lanes.js';
  import { buildOverlayPaths } from '../lib/overlay-paths.js';
  import { getVisibleOverlayElements } from '../lib/overlay-visible.js';
  import { buildRefPillData } from '../lib/ref-pill-data.js';
  import { measureTextWidth } from '../lib/text-measure.js';
  import type { OverlayRefPill, RefLabel } from '../lib/types.js';

  import { Menu, MenuItem, Submenu, PredefinedMenuItem, CheckMenuItem } from '@tauri-apps/api/menu';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import { ask, message } from '@tauri-apps/plugin-dialog';
  import CommitRow from './CommitRow.svelte';
  import InputDialog from './InputDialog.svelte';
  import SearchBar from './SearchBar.svelte';
  import { Laptop, Globe, Tag, Archive } from '@lucide/svelte';
  import type { SearchResult } from '../lib/types.js';

  interface Props {
    repoPath: string;
    oncommitselect?: (oid: string) => void;
    wipCount?: number;
    wipMessage?: string;
    onWipClick?: () => void;
    refreshSignal?: number;
    selectedCommitOid?: string | null;
  }

  let { repoPath, oncommitselect, wipCount = 0, wipMessage = 'WIP', onWipClick, refreshSignal, selectedCommitOid }: Props = $props();

  const BATCH = 200;
  const SKELETON_COUNT = 10;

  /** Icon map for commit graph pills — matches sidebar BranchRow icon vocabulary */
  const PILL_ICONS: Record<string, typeof Laptop> = {
    LocalBranch: Laptop,
    RemoteBranch: Globe,
    Tag: Tag,
    Stash: Archive,
  };

  // Graph display settings — will be wired to user preferences in a future settings page.
  // Change values here (or load from store) to adjust layout without touching any other file.
  let displaySettings = $state({ ...DEFAULT_GRAPH_SETTINGS });

  let commits = $state<GraphCommit[]>([]);
  let maxColumns = $state(1);
  let hasMore = $state(true);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let offset = $state(0);
  let listRef = $state<{ scroll: (opts: { index: number; smoothScroll?: boolean; align?: string }) => Promise<void> } | null>(null);
  let scrolledToHead = false;

  let columnWidths = $state<ColumnWidths>({ ref: 120, graph: 120, author: 120, date: 100, sha: 80 });
  let columnVisibility = $state<ColumnVisibility>({ ref: true, graph: true, message: true, author: true, date: true, sha: true });

  const ORDERED_COLUMNS = ['ref', 'graph', 'message', 'author', 'date', 'sha'] as const;
  type ColumnKey = typeof ORDERED_COLUMNS[number];

  const visibleColumns = $derived(
    ORDERED_COLUMNS.filter(k => columnVisibility[k])
  );
  const lastVisibleColumn = $derived(
    visibleColumns[visibleColumns.length - 1] as ColumnKey | undefined
  );

  $effect(() => {
    getColumnWidths().then((w) => { columnWidths = w; });
  });

  $effect(() => {
    getColumnVisibility().then((v) => { columnVisibility = v; });
  });

  // GRAPH-02: horizontal panning offset for the graph column.
  // When graphColWidth < naturalGraphWidth, graphScrollX tracks which portion of the
  // graph lanes are visible. Controlled by trackpad/wheel horizontal gestures.
  let graphScrollX = $state(0);

  // Derived: natural graph width based on actual lane count
  const naturalGraphWidth = $derived(Math.max(maxColumns, 1) * displaySettings.laneWidth);
  // Derived: maximum horizontal scroll within the graph column
  const maxGraphScrollX = $derived(
    columnVisibility.graph ? Math.max(0, naturalGraphWidth - columnWidths.graph + 2 * COLUMN_PADDING_X) : 0
  );

  // Clamp graphScrollX when maxGraphScrollX shrinks (e.g. column widened or fewer lanes)
  $effect(() => {
    if (graphScrollX > maxGraphScrollX) graphScrollX = maxGraphScrollX;
  });

  let stashOidToIndex = $state<Map<string, number>>(new Map());

  // Search state
  let searchOpen = $state(false);
  let searchQuery = $state('');
  let searchResults = $state<SearchResult[]>([]);
  let searchCurrentIndex = $state(0);
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Derived: Set of matching OIDs for O(1) lookup in CommitRow
  const searchMatchOids = $derived(new Set(searchResults.map(r => r.oid)));
  // Derived: OID of current match for strong highlight
  const searchCurrentOid = $derived(searchResults.length > 0 ? searchResults[searchCurrentIndex]?.oid ?? null : null);
  // Derived: whether SVG dimming should be active (search open + query + results)
  const searchDimmingActive = $derived(searchOpen && searchQuery.length > 0 && searchResults.length > 0);

  async function loadStashMap() {
    try {
      const stashes = await safeInvoke<StashEntry[]>('list_stashes', { path: repoPath });
      const map = new Map<string, number>();
      for (const stash of stashes) {
        map.set(stash.oid, stash.index);
      }
      stashOidToIndex = map;
    } catch {
      stashOidToIndex = new Map();
    }
  }

  function startColumnResize(column: keyof ColumnWidths, e: MouseEvent, invert = false) {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = columnWidths[column];
    const minWidths: Record<keyof ColumnWidths, number> = {
      ref: 60,
      graph: 20, // GRAPH-02: allow graph column to shrink below lane content width
      author: 60,
      date: 60,
      sha: 50,
    };
    const maxWidths: Record<keyof ColumnWidths, number> = {
      ref: 400,
      graph: naturalGraphWidth + 2 * COLUMN_PADDING_X, // GRAPH-02: cap at natural content width + padding
      author: 400,
      date: 400,
      sha: 400,
    };

    function onMouseMove(ev: MouseEvent) {
      const delta = (ev.clientX - startX) * (invert ? -1 : 1);
      const newWidth = Math.max(minWidths[column], Math.min(maxWidths[column], startWidth + delta));
      columnWidths = { ...columnWidths, [column]: newWidth };
    }

    function onMouseUp() {
      setColumnWidths(columnWidths);
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  const columnLabels: { key: keyof ColumnVisibility; label: string }[] = [
    { key: 'ref', label: 'Branch/Tag' },
    { key: 'graph', label: 'Graph' },
    { key: 'message', label: 'Message' },
    { key: 'author', label: 'Author' },
    { key: 'date', label: 'Date' },
    { key: 'sha', label: 'SHA' },
  ];

  // InputDialog state
  interface DialogConfig {
    title: string;
    fields: { key: string; label: string; placeholder?: string; multiline?: boolean; required?: boolean; defaultValue?: string }[];
    onsubmit: (values: Record<string, string>) => void;
  }
  let dialogConfig = $state<DialogConfig | null>(null);

  function closeDialog() {
    dialogConfig = null;
  }

  // Commit context menu actions

  async function handleCheckoutCommit(commit: GraphCommit) {
    const confirmed = await ask(
      "Checkout this commit in detached HEAD mode? You won't be on any branch. Create a branch afterward to save your work.",
      { title: 'Checkout Commit', kind: 'warning' }
    );
    if (!confirmed) return;
    try {
      await safeInvoke('checkout_commit', { path: repoPath, oid: commit.oid });
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Failed to checkout commit', { title: 'Checkout Error', kind: 'error' });
    }
  }

  function handleCreateBranch(commit: GraphCommit) {
    dialogConfig = {
      title: 'Create Branch',
      fields: [{ key: 'name', label: 'Branch name', required: true }],
      onsubmit: async (values) => {
        closeDialog();
        try {
          await safeInvoke('create_branch', { path: repoPath, name: values.name, fromOid: commit.oid });
          showToast('Checked out ' + values.name, 'success');
        } catch (e) {
          const err = e as TrunkError;
          if (err.code === 'dirty_workdir') {
            showToast('Branch created (checkout skipped — uncommitted changes)', 'success');
          } else {
            await message(err.message ?? 'Failed to create branch', { title: 'Create Branch Error', kind: 'error' });
          }
        }
      },
    };
  }

  function handleCreateTag(commit: GraphCommit) {
    dialogConfig = {
      title: 'Create Tag',
      fields: [
        { key: 'name', label: 'Tag name', required: true },
        { key: 'message', label: 'Message (optional)', multiline: true },
      ],
      onsubmit: async (values) => {
        closeDialog();
        try {
          await safeInvoke('create_tag', { path: repoPath, oid: commit.oid, tagName: values.name, message: values.message || '' });
        } catch (e) {
          const err = e as TrunkError;
          await message(err.message ?? 'Failed to create tag', { title: 'Create Tag Error', kind: 'error' });
        }
      },
    };
  }

  async function handleCherryPick(commit: GraphCommit) {
    clearRedoStack();
    try {
      await safeInvoke('cherry_pick', { path: repoPath, oid: commit.oid });
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Cherry-pick failed. You may need to resolve conflicts manually.', { title: 'Cherry-pick Error', kind: 'error' });
    }
  }

  async function handleRevert(commit: GraphCommit) {
    clearRedoStack();
    try {
      await safeInvoke('revert_commit', { path: repoPath, oid: commit.oid });
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Revert failed. You may need to resolve conflicts manually.', { title: 'Revert Error', kind: 'error' });
    }
  }

  async function handleReset(commit: GraphCommit, mode: 'soft' | 'mixed' | 'hard') {
    const labels: Record<string, string> = {
      soft: 'Soft reset keeps all changes staged.',
      mixed: 'Mixed reset keeps changes but unstages them.',
      hard: 'Hard reset discards ALL changes. This cannot be undone!',
    };
    const confirmed = await ask(
      `Reset current branch to this commit?\n\n${labels[mode]}`,
      { title: `Reset (${mode})`, kind: mode === 'hard' ? 'warning' : 'info' }
    );
    if (!confirmed) return;
    try {
      await safeInvoke('reset_to_commit', { path: repoPath, oid: commit.oid, mode });
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Reset failed.', { title: 'Reset Error', kind: 'error' });
    }
  }

  async function showCommitContextMenu(e: MouseEvent, commit: GraphCommit) {
    e.preventDefault();
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Copy SHA', action: () => { writeText(commit.oid).catch(() => {}); } }),
        await MenuItem.new({ text: 'Copy Message', action: () => { writeText(commit.summary).catch(() => {}); } }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({ text: 'Checkout Commit...', action: () => { handleCheckoutCommit(commit).catch(() => {}); } }),
        await MenuItem.new({ text: 'Create Branch...', action: () => { handleCreateBranch(commit); } }),
        await MenuItem.new({ text: 'Create Tag...', action: () => { handleCreateTag(commit); } }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({ text: 'Cherry-pick', enabled: !commit.is_merge, action: () => { handleCherryPick(commit).catch(() => {}); } }),
        await MenuItem.new({ text: 'Revert', enabled: !commit.is_merge, action: () => { handleRevert(commit).catch(() => {}); } }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await Submenu.new({ text: 'Reset...', items: [
          await MenuItem.new({ text: 'Soft', action: () => { handleReset(commit, 'soft').catch(() => {}); } }),
          await MenuItem.new({ text: 'Mixed', action: () => { handleReset(commit, 'mixed').catch(() => {}); } }),
          await MenuItem.new({ text: 'Hard', action: () => { handleReset(commit, 'hard').catch(() => {}); } }),
        ]}),
      ]
    });
    await menu.popup();
  }

  // Stash context menu actions

  async function handleStashPop(index: number) {
    try {
      await safeInvoke('stash_pop', { path: repoPath, index });
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Failed to pop stash', { title: 'Stash Error', kind: 'error' });
    }
  }

  async function handleStashApply(index: number) {
    try {
      await safeInvoke('stash_apply', { path: repoPath, index });
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Failed to apply stash', { title: 'Stash Error', kind: 'error' });
    }
  }

  async function handleStashDrop(index: number) {
    const confirmed = await ask(`Drop stash@{${index}}? This cannot be undone.`, {
      title: 'Confirm Drop',
      kind: 'warning',
    });
    if (!confirmed) return;
    try {
      await safeInvoke('stash_drop', { path: repoPath, index });
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Failed to drop stash', { title: 'Stash Error', kind: 'error' });
    }
  }

  async function showStashContextMenu(e: MouseEvent, commit: GraphCommit) {
    e.preventDefault();
    const stashIndex = stashOidToIndex.get(commit.oid);
    if (stashIndex === undefined) return;
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Pop', action: () => { handleStashPop(stashIndex).catch(() => {}); } }),
        await MenuItem.new({ text: 'Apply', action: () => { handleStashApply(stashIndex).catch(() => {}); } }),
        await MenuItem.new({ text: 'Drop', action: () => { handleStashDrop(stashIndex).catch(() => {}); } }),
      ]
    });
    await menu.popup();
  }

  function handleRowContextMenu(e: MouseEvent, commit: GraphCommit) {
    if (commit.is_stash) {
      showStashContextMenu(e, commit);
    } else {
      showCommitContextMenu(e, commit);
    }
  }

  // Pill context menu actions (branch delete/rename, tag delete)

  async function handleDeleteBranch(branchName: string) {
    const confirmed = await ask(
      `Delete branch '${branchName}'? This cannot be undone.`,
      { title: 'Delete Branch', kind: 'warning' }
    );
    if (!confirmed) return;
    try {
      await safeInvoke('delete_branch', { path: repoPath, branchName });
      showToast(`Deleted branch ${branchName}`, 'success');
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Failed to delete branch', { title: 'Delete Branch Error', kind: 'error' });
    }
  }

  function handleRenameBranch(branchName: string) {
    dialogConfig = {
      title: 'Rename Branch',
      fields: [{ key: 'name', label: 'New name', required: true, defaultValue: branchName }],
      onsubmit: async (values) => {
        closeDialog();
        const newName = values.name.trim();
        if (!newName || newName === branchName) return;
        try {
          await safeInvoke('rename_branch', { path: repoPath, oldName: branchName, newName });
          showToast(`Renamed branch to ${newName}`, 'success');
        } catch (e) {
          const err = e as TrunkError;
          await message(err.message ?? 'Failed to rename branch', { title: 'Rename Error', kind: 'error' });
        }
      },
    };
  }

  async function handleDeleteTag(tagName: string) {
    const confirmed = await ask(
      `Delete tag '${tagName}'? This cannot be undone.`,
      { title: 'Delete Tag', kind: 'warning' }
    );
    if (!confirmed) return;
    try {
      await safeInvoke('delete_tag', { path: repoPath, tagName });
      showToast(`Deleted tag ${tagName}`, 'success');
    } catch (e) {
      const err = e as TrunkError;
      await message(err.message ?? 'Failed to delete tag', { title: 'Delete Tag Error', kind: 'error' });
    }
  }

  async function showPillContextMenu(e: MouseEvent, pill: OverlayRefPill) {
    e.preventDefault();
    e.stopPropagation();

    if (pill.refType === 'LocalBranch') {
      const menu = await Menu.new({
        items: [
          await MenuItem.new({
            text: 'Rename…',
            action: () => { handleRenameBranch(pill.label); },
          }),
          await PredefinedMenuItem.new({ item: 'Separator' }),
          await MenuItem.new({
            text: 'Delete',
            enabled: !pill.isHead,
            action: () => { handleDeleteBranch(pill.label).catch(() => {}); },
          }),
        ],
      });
      await menu.popup();
    } else if (pill.refType === 'Tag') {
      const menu = await Menu.new({
        items: [
          await MenuItem.new({
            text: 'Delete',
            action: () => { handleDeleteTag(pill.label).catch(() => {}); },
          }),
        ],
      });
      await menu.popup();
    }
  }

  async function showOverflowRefContextMenu(e: MouseEvent, ref: RefLabel) {
    e.preventDefault();
    e.stopPropagation();

    if (ref.ref_type === 'LocalBranch') {
      const menu = await Menu.new({
        items: [
          await MenuItem.new({
            text: 'Rename…',
            action: () => { handleRenameBranch(ref.short_name); },
          }),
          await PredefinedMenuItem.new({ item: 'Separator' }),
          await MenuItem.new({
            text: 'Delete',
            enabled: !ref.is_head,
            action: () => { handleDeleteBranch(ref.short_name).catch(() => {}); },
          }),
        ],
      });
      await menu.popup();
    } else if (ref.ref_type === 'Tag') {
      const menu = await Menu.new({
        items: [
          await MenuItem.new({
            text: 'Delete',
            action: () => { handleDeleteTag(ref.short_name).catch(() => {}); },
          }),
        ],
      });
      await menu.popup();
    }
  }

  async function showHeaderContextMenu(e: MouseEvent) {
    e.preventDefault();
    const items = await Promise.all(
      columnLabels.map((col) =>
        CheckMenuItem.new({
          text: col.label,
          checked: columnVisibility[col.key],
          enabled: col.key !== 'message',
          action: () => {
            if (col.key === 'message') return;
            columnVisibility = { ...columnVisibility, [col.key]: !columnVisibility[col.key] };
            setColumnVisibility(columnVisibility);
          },
        })
      )
    );
    const menu = await Menu.new({ items });
    await menu.popup();
  }

  function makeWipItem(msg: string, col: number, colorIdx: number): GraphCommit {
    return {
      oid: '__wip__',
      short_oid: '',
      summary: msg,
      body: null,
      author_name: '',
      author_email: '',
      author_timestamp: 0,
      parent_oids: [],
      column: col,
      color_index: colorIdx,
      edges: [{ from_column: col, to_column: col, edge_type: 'Straight' as EdgeType, color_index: colorIdx, dashed: false }],
      refs: [],
      is_head: false,
      is_merge: false,
      is_branch_tip: false,
      is_stash: false,
    };
  }

  const displayItems = $derived.by(() => {
    // Stash commits are now included in the backend graph result with proper lane data.
    // We only need to prepend the WIP row if there are uncommitted changes.
    if (wipCount > 0) {
      // Find the actual HEAD commit (the one with is_head flag) to match WIP's column and color.
      const headCommit = commits.find(c => c.is_head);
      const col = headCommit?.column ?? 0;
      const colorIdx = headCommit?.color_index ?? 0;
      return [makeWipItem(wipMessage, col, colorIdx), ...commits];
    }
    return [...commits];
  });

  const laneColor = (idx: number) => `var(--lane-${idx % 8})`;
  const cx = (col: number) => col * displaySettings.laneWidth + displaySettings.laneWidth / 2;
  const cy = (row: number) => row * displaySettings.rowHeight + displaySettings.rowHeight / 2;

  const graphData = $derived.by(() => buildGraphData(displayItems, maxColumns));
  const paths = $derived.by(() => buildOverlayPaths(graphData, displaySettings));
  const pillData = $derived.by(() => buildRefPillData(graphData.nodes, displayItems, columnWidths.ref, measureTextWidth, displaySettings));

  let hoveredPill = $state<OverlayRefPill | null>(null);
  let hoverTimeout: ReturnType<typeof setTimeout> | null = null;

  function pillMouseEnter(pill: OverlayRefPill) {
    if (hoverTimeout) { clearTimeout(hoverTimeout); hoverTimeout = null; }
    if (pill.overflowCount > 0 || pill.truncatedLabel !== pill.label) {
      hoveredPill = pill;
    }
  }

  function pillMouseLeave() {
    hoverTimeout = setTimeout(() => { hoveredPill = null; }, 50);
  }

  function overlayMouseEnter() {
    if (hoverTimeout) { clearTimeout(hoverTimeout); hoverTimeout = null; }
  }

  function overlayMouseLeave() {
    hoveredPill = null;
  }

  async function loadMore() {
    if (loading || !hasMore) return;
    loading = true;
    error = null;
    try {
      const response = await safeInvoke<GraphResponse>('get_commit_graph', {
        path: repoPath,
        offset,
      });
      commits.push(...response.commits);
      maxColumns = response.max_columns;
      offset += response.commits.length;
      if (response.commits.length < BATCH) hasMore = false;
    } catch (e) {
      const err = e as TrunkError;
      error = err.message ?? 'Failed to load commits';
    } finally {
      loading = false;
    }
  }

  /** Scroll the graph to center the row for the given OID.
   * Loads additional history batches if the commit is not yet loaded.
   * Called from App.svelte via bind:this (GRAPH-03). */
  export async function scrollToOid(oid: string): Promise<void> {
    let idx = displayItems.findIndex(c => c.oid === oid);

    // Load more batches until found or all commits exhausted
    while (idx < 0 && hasMore && !loading) {
      await loadMore();
      await tick();
      idx = displayItems.findIndex(c => c.oid === oid);
    }

    if (idx < 0 || !listRef) return;

    // Center the row in the viewport by computing the scroll offset manually.
    // VirtualList doesn't support 'center' alignment, so we calculate:
    //   scrollTop = rowTop - (viewportHeight / 2) + (rowHeight / 2)
    const rowTop = idx * displaySettings.rowHeight;
    const viewport = document.querySelector('.virtual-list-viewport') as HTMLElement | null;
    if (viewport) {
      const viewportHeight = viewport.clientHeight;
      const centerOffset = Math.max(0, rowTop - viewportHeight / 2 + displaySettings.rowHeight / 2);
      viewport.scrollTo({ top: centerOffset, behavior: 'smooth' });
    } else {
      // Fallback: use VirtualList's scroll with 'auto' alignment
      await listRef.scroll({ index: idx, smoothScroll: true, align: 'auto' });
    }
  }

  async function refresh() {
    try {
      const response = await safeInvoke<GraphResponse>('refresh_commit_graph', {
        path: repoPath,
      });
      // Swap data atomically -- old data stays visible until this assignment
      commits = response.commits;
      maxColumns = response.max_columns;
      offset = response.commits.length;
      hasMore = response.commits.length >= BATCH;
      error = null;
      await loadStashMap();
    } catch (e) {
      const err = e as TrunkError;
      error = err.message ?? 'Failed to load commits';
      // Keep old commits visible on error -- do NOT clear
    }
  }

  $effect(() => {
    untrack(async () => {
      await loadMore();
      await loadStashMap();
    });
  });

  $effect(() => {
    // Access refreshSignal to create reactive dependency
    if (refreshSignal !== undefined && refreshSignal > 0) {
      untrack(() => refresh());
    }
  });

  $effect(() => {
    // Only scroll once per mount (scrolledToHead guards against re-firing)
    if (scrolledToHead) return;
    if (!listRef) return;
    if (displayItems.length === 0) return;

    const headIdx = displayItems.findIndex(c => c.is_head);
    if (headIdx >= 0) {
      scrolledToHead = true;
      tick().then(() => listRef?.scroll({ index: headIdx, smoothScroll: false, align: 'top' }));
    } else if (untrack(() => hasMore)) {
      // HEAD not in current batch -- load the next batch so the effect re-fires with more commits.
      // untrack prevents hasMore from creating a reactive dependency here.
      untrack(() => loadMore());
    }
  });

  // Cmd+F keyboard handler — capture phase to intercept before WebView native find (P7)
  $effect(() => {
    function handleSearchKeydown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault();
        e.stopPropagation();
        if (searchOpen) {
          // Already open: focus input and select all text (VS Code behavior)
          const input = document.querySelector('.search-bar-input') as HTMLInputElement | null;
          if (input) { input.focus(); input.select(); }
        } else {
          searchOpen = true;
        }
      }
    }
    window.addEventListener('keydown', handleSearchKeydown, { capture: true });
    return () => {
      window.removeEventListener('keydown', handleSearchKeydown, { capture: true });
      if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
    };
  });

  function handleSearchQueryChange(query: string) {
    searchQuery = query;
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer);

    if (!query.trim()) {
      searchResults = [];
      searchCurrentIndex = 0;
      return;
    }

    searchDebounceTimer = setTimeout(async () => {
      try {
        const results = await safeInvoke<SearchResult[]>('search_commits', {
          path: repoPath,
          query: query.trim(),
        });
        searchResults = results;
        searchCurrentIndex = 0;
        if (results.length > 0) {
          scrollToOid(results[0].oid);
          oncommitselect?.(results[0].oid);
        }
      } catch {
        searchResults = [];
        searchCurrentIndex = 0;
      }
    }, 200);
  }

  function handleSearchNext() {
    if (searchResults.length === 0) return;
    searchCurrentIndex = (searchCurrentIndex + 1) % searchResults.length;
    const oid = searchResults[searchCurrentIndex].oid;
    scrollToOid(oid);
    oncommitselect?.(oid);
  }

  function handleSearchPrev() {
    if (searchResults.length === 0) return;
    searchCurrentIndex = (searchCurrentIndex - 1 + searchResults.length) % searchResults.length;
    const oid = searchResults[searchCurrentIndex].oid;
    scrollToOid(oid);
    oncommitselect?.(oid);
  }

  function handleSearchClose() {
    searchOpen = false;
    searchQuery = '';
    searchResults = [];
    searchCurrentIndex = 0;
    if (searchDebounceTimer) { clearTimeout(searchDebounceTimer); searchDebounceTimer = null; }
  }
</script>

<div class="h-full overflow-hidden flex flex-col" style="background: var(--color-bg);">
  <!-- Header row (always visible) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex items-center flex-shrink-0"
    style="height: 24px; background: var(--color-surface); border-bottom: 1px solid var(--color-border); font-size: 11px; color: var(--color-text-muted); user-select: none; padding: 0 {COLUMN_PADDING_X}px;"
    oncontextmenu={showHeaderContextMenu}
  >
    {#if columnVisibility.ref}
      <div class="flex-shrink-0 relative" style="width: {columnWidths.ref}px; padding: 0 {COLUMN_PADDING_X}px;">
        Branch/Tag
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        {#if 'ref' !== lastVisibleColumn}
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('ref', e)}></div>
        {/if}
      </div>
    {/if}
    {#if columnVisibility.graph}
      <div class="flex-shrink-0 relative" style="width: {columnWidths.graph}px; padding: 0 {COLUMN_PADDING_X}px;">
        Graph
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        {#if 'graph' !== lastVisibleColumn}
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('graph', e)}></div>
        {/if}
      </div>
    {/if}
    {#if columnVisibility.message}
      <div class="flex-1 relative" style="padding: 0 {COLUMN_PADDING_X}px;">
        Message
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        {#if 'message' !== lastVisibleColumn}
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('author', e, true)}></div>
        {/if}
      </div>
    {/if}
    {#if columnVisibility.author}
      <div class="flex-shrink-0 relative" style="width: {columnWidths.author}px; padding: 0 {COLUMN_PADDING_X}px;">
        Author
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        {#if 'author' !== lastVisibleColumn}
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('date', e, true)}></div>
        {/if}
      </div>
    {/if}
    {#if columnVisibility.date}
      <div class="flex-shrink-0 relative" style="width: {columnWidths.date}px; padding: 0 {COLUMN_PADDING_X}px;">
        Date
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        {#if 'date' !== lastVisibleColumn}
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('sha', e, true)}></div>
        {/if}
      </div>
    {/if}
    {#if columnVisibility.sha}
      <div class="flex-shrink-0" style="width: {columnWidths.sha}px; padding: 0 {COLUMN_PADDING_X}px;">
        SHA
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        {#if 'sha' !== lastVisibleColumn}
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('sha', e, true)}></div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Content area (grows to fill remaining space) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="flex-1 overflow-hidden" style="position: relative; padding: 0 {COLUMN_PADDING_X}px;" onwheel={(e) => {
    // GRAPH-02: horizontal pan on trackpad swipe or shift+wheel
    if (maxGraphScrollX > 0 && e.deltaX !== 0) {
      graphScrollX = Math.max(0, Math.min(maxGraphScrollX, graphScrollX + e.deltaX));
    }
  }}>
    {#if searchOpen}
      <SearchBar
        query={searchQuery}
        currentIndex={searchCurrentIndex}
        totalMatches={searchResults.length}
        onquerychange={handleSearchQueryChange}
        onnext={handleSearchNext}
        onprev={handleSearchPrev}
        onclose={handleSearchClose}
      />
    {/if}

    {#if commits.length === 0 && loading}
      <!-- Initial skeleton loading -->
      {#each { length: SKELETON_COUNT } as _}
        <div class="flex items-center gap-2 px-2 animate-pulse" style="height: {displaySettings.rowHeight}px">
          <div
            class="rounded-full flex-shrink-0"
            style="background: var(--color-border); width: 64px; height: 12px;"
          ></div>
          <div
            class="rounded flex-shrink-0"
            style="background: var(--color-border); width: 32px; height: 100%;"
          ></div>
          <div class="rounded flex-1" style="background: var(--color-border); height: 12px;"></div>
        </div>
      {/each}
    {:else if commits.length === 0 && error}
      <!-- Initial load error -->
      <div
        class="m-4 rounded-md px-4 py-3 text-sm"
        style="background: #3d1c1c; border: 1px solid #6b2a2a; color: #f87171;"
      >
        {error}
      </div>
    {:else}
      <!-- SVG overlay snippet - renders inside virtual list scroll container -->
      {#snippet graphOverlay(contentHeight: number, visibleStart: number, visibleEnd: number)}
        {@const refOffset = columnVisibility.ref ? columnWidths.ref : 0}
        {@const visible = getVisibleOverlayElements(paths, graphData.nodes, visibleStart, visibleEnd, pillData)}
        {@const graphColWidth = columnVisibility.graph ? columnWidths.graph : naturalGraphWidth}
        {@const scrollX = Math.min(graphScrollX, Math.max(0, naturalGraphWidth - graphColWidth + 2 * COLUMN_PADDING_X))}
        <svg
          class="absolute top-0"
          width={refOffset + Math.max(graphColWidth, naturalGraphWidth)}
          height={contentHeight}
          style="left: 0; pointer-events: none; z-index: 1; {searchDimmingActive ? 'opacity: 0.2;' : ''}"
        >
          <!-- GRAPH-02: clip graph content to column width -->
          <defs>
            <clipPath id="graph-clip">
              <rect x={refOffset + COLUMN_PADDING_X} y="0" width={graphColWidth - 2 * COLUMN_PADDING_X} height={contentHeight} />
            </clipPath>
          </defs>
          <!-- GRAPH-02: Layer A — rails + connections, scrolled and clipped.
               Translated left by scrollX to pan through lanes. -->
          <g clip-path="url(#graph-clip)">
            <g class="overlay-rails" transform="translate({refOffset + COLUMN_PADDING_X - scrollX}, 0)">
              {#each visible.rails as path}
                <path d={path.d} fill="none"
                  stroke={laneColor(path.colorIndex)}
                  stroke-width={displaySettings.edgeStroke}
                  stroke-linecap="butt"
                  stroke-dasharray={path.dashed ? '3 3' : 'none'} />
              {/each}
            </g>
            <g class="overlay-connections" transform="translate({refOffset + COLUMN_PADDING_X - scrollX}, 0)">
              {#each visible.connections as path}
                <path d={path.d} fill="none"
                  stroke={laneColor(path.colorIndex)}
                  stroke-width={displaySettings.edgeStroke}
                  stroke-linecap="round"
                  stroke-dasharray={path.dashed ? '3 3' : 'none'} />
              {/each}
            </g>
          </g>
          <!-- GRAPH-02: Layer B — dots with "sticky" X clamping.
               Dots slide along their horizontal line to stay visible in the viewport.
               Viewport spans graph coordinates [scrollX, scrollX + graphColWidth].
               Dots clamp to viewport edges (bead-on-a-string effect). -->
          <g class="overlay-dots" transform="translate({refOffset + COLUMN_PADDING_X}, 0)">
            {#each visible.dots as node}
              {@const clampedCx = Math.max(displaySettings.laneWidth / 2, Math.min(graphColWidth - 2 * COLUMN_PADDING_X - displaySettings.dotRadius, cx(node.x) - scrollX))}
              {#if node.isWip}
                <circle cx={clampedCx} cy={cy(node.y)} r={displaySettings.dotRadius}
                  fill="none" stroke={laneColor(node.colorIndex)}
                  stroke-width={displaySettings.edgeStroke} stroke-dasharray="3 3" />
              {:else if node.isStash}
                <rect
                  x={clampedCx - displaySettings.dotRadius}
                  y={cy(node.y) - displaySettings.dotRadius}
                  width={displaySettings.dotRadius * 2}
                  height={displaySettings.dotRadius * 2}
                  fill="none"
                  stroke={laneColor(node.colorIndex)}
                  stroke-width={displaySettings.edgeStroke}
                  stroke-dasharray="3 3" />
              {:else if node.isMerge}
                <circle cx={clampedCx} cy={cy(node.y)} r={displaySettings.dotRadius}
                  fill="var(--color-bg)" stroke={laneColor(node.colorIndex)}
                  stroke-width={displaySettings.mergeStroke} />
              {:else}
                <circle cx={clampedCx} cy={cy(node.y)} r={displaySettings.dotRadius}
                  fill={laneColor(node.colorIndex)} />
              {/if}
            {/each}
          </g>
          {#if columnVisibility.ref}
            <g class="overlay-pills">
              {#each visible.pills as pill}
                <!-- Connector line from pill to commit dot (uses sticky X position, scroll-adjusted) -->
                {#if columnVisibility.graph}
                  {@const stickyDotCx = Math.max(displaySettings.laneWidth / 2, Math.min(graphColWidth - 2 * COLUMN_PADDING_X - displaySettings.dotRadius, pill.dotCx - scrollX))}
                  <line
                    x1={pill.x + pill.width}
                    y1={pill.y}
                    x2={refOffset + COLUMN_PADDING_X + stickyDotCx}
                    y2={pill.dotCy}
                    stroke={laneColor(pill.commitColorIndex)}
                    stroke-width={displaySettings.pillStroke}
                    opacity={pill.isRemoteOnly ? 0.67 : 1}
                    style={pill.isNonHead && !pill.isRemoteOnly ? 'filter: brightness(0.75)' : ''}
                  />
                {/if}

                <!-- Capsule rect -->
                <rect
                  x={pill.x}
                  y={pill.y - PILL_HEIGHT / 2}
                  width={pill.width}
                  height={PILL_HEIGHT}
                  rx={PILL_HEIGHT / 2}
                  ry={PILL_HEIGHT / 2}
                  fill={laneColor(pill.colorIndex)}
                  opacity={pill.isRemoteOnly ? 0.67 : 1}
                  style={pill.isNonHead && !pill.isRemoteOnly ? 'filter: brightness(0.75)' : ''}
                  pointer-events="auto"
                  onmouseenter={() => pillMouseEnter(pill)}
                  onmouseleave={pillMouseLeave}
                  oncontextmenu={(e) => showPillContextMenu(e, pill)}
                />

                <!-- Icon rendered directly in SVG at a fixed position (no CSS layout) -->
                {#if PILL_ICONS[pill.refType]}
                  {@const PillIcon = PILL_ICONS[pill.refType]}
                  <g transform="translate({pill.x + PILL_PADDING_X}, {pill.y - ICON_WIDTH / 2})" opacity="0.9" style="pointer-events: auto; cursor: context-menu;" oncontextmenu={(e) => showPillContextMenu(e, pill)}>
                    <PillIcon size={ICON_WIDTH} />
                  </g>
                {/if}

                <!-- Text in its own foreignObject sized to exactly the canvas-measured text width.
                     No flex layout — icon is positioned separately so the text width is unambiguous. -->
                <foreignObject
                  x={pill.x + PILL_PADDING_X + ICON_WIDTH + ICON_GAP}
                  y={pill.y - PILL_HEIGHT / 2}
                  width={Math.ceil(pill.textWidth)}
                  height={PILL_HEIGHT}
                >
                  <span
                    style="
                      display: block;
                      line-height: {PILL_HEIGHT}px;
                      color: white;
                      font-size: {PILL_FONT_SIZE}px;
                      font-family: var(--font-sans);
                      font-weight: {pill.isHead ? 700 : 500};
                      white-space: nowrap;
                      overflow: hidden;
                      cursor: context-menu;
                    "
                    oncontextmenu={(e) => showPillContextMenu(e, pill)}
                  >{pill.truncatedLabel}</span>
                </foreignObject>

                <!-- Overflow +N badge -->
                {#if pill.overflowCount > 0}
                  {@const badgeText = `+${pill.overflowCount}`}
                  {@const badgeWidth = badgeText.length * BADGE_FONT_SIZE * 0.7 + PILL_PADDING_X * 2}
                  <rect
                    x={pill.x + pill.width + PILL_GAP}
                    y={pill.y - BADGE_HEIGHT / 2}
                    width={badgeWidth}
                    height={BADGE_HEIGHT}
                    rx={BADGE_HEIGHT / 2}
                    ry={BADGE_HEIGHT / 2}
                    fill={laneColor(pill.colorIndex)}
                    style="filter: brightness(0.65)"
                    pointer-events="auto"
                    onmouseenter={() => pillMouseEnter(pill)}
                    onmouseleave={pillMouseLeave}
                  />
                  <foreignObject
                    x={pill.x + pill.width + PILL_GAP}
                    y={pill.y - BADGE_HEIGHT / 2}
                    width={badgeWidth}
                    height={BADGE_HEIGHT}
                  >
                    <span
                      style="
                        color: white;
                        font-size: {BADGE_FONT_SIZE}px;
                        font-family: var(--font-sans);
                        font-weight: 500;
                        line-height: {BADGE_HEIGHT}px;
                        display: block;
                        text-align: center;
                        white-space: nowrap;
                      "
                    >{badgeText}</span>
                  </foreignObject>
                {/if}
              {/each}
            </g>
          {/if}
        </svg>
        {#if hoveredPill && columnVisibility.ref}
          {#if hoveredPill.overflowCount > 0}
            <!-- Multi-ref expansion: shows all refs vertically -->
            <div
              class="absolute rounded-lg shadow-lg"
              style="
                left: {hoveredPill.x}px;
                top: {hoveredPill.y - PILL_HEIGHT / 2}px;
                background: var(--lane-{hoveredPill.colorIndex % 8});
                padding: 4px 8px;
                z-index: 50;
                pointer-events: auto;
                opacity: 1;
                transition: opacity 180ms ease;
              "
              onmouseenter={overlayMouseEnter}
              onmouseleave={overlayMouseLeave}
            >
              {#each hoveredPill.allRefs as ref}
                <div
                  style="display: flex; align-items: center; gap: 3px; cursor: context-menu; border-radius: 4px;"
                  class="text-[11px] leading-5 font-medium text-white whitespace-nowrap hover:bg-white/15 px-1 -mx-1"
                  oncontextmenu={(e) => showOverflowRefContextMenu(e, ref)}
                >
                  {#if PILL_ICONS[ref.ref_type]}
                    {@const RefIcon = PILL_ICONS[ref.ref_type]}
                    <RefIcon size={10} style="flex-shrink: 0; opacity: 0.85;" />
                  {/if}
                  {ref.short_name}
                </div>
              {/each}
            </div>
          {:else}
            <!-- Truncated single-ref: width-only expansion showing full label -->
            <div
              class="absolute rounded-full shadow-lg"
              style="
                left: {hoveredPill.x}px;
                top: {hoveredPill.y - PILL_HEIGHT / 2}px;
                height: {PILL_HEIGHT}px;
                background: var(--lane-{hoveredPill.colorIndex % 8});
                padding: 0 {PILL_PADDING_X}px;
                z-index: 50;
                pointer-events: auto;
                display: flex;
                align-items: center;
                opacity: 1;
                transition: opacity 180ms ease;
              "
              onmouseenter={overlayMouseEnter}
              onmouseleave={overlayMouseLeave}
            >
              <span style="display: flex; align-items: center; gap: 2px; font-weight: {hoveredPill.isHead ? 700 : 500};" class="text-[11px] font-medium text-white whitespace-nowrap">
                {#if PILL_ICONS[hoveredPill.refType]}
                  {@const HoverIcon = PILL_ICONS[hoveredPill.refType]}
                  <HoverIcon size={10} style="flex-shrink: 0; opacity: 0.9;" />
                {/if}
                {hoveredPill.label}
              </span>
            </div>
          {/if}
        {/if}
      {/snippet}

      {#key displaySettings.rowHeight}
      <VirtualList
        bind:this={listRef}
        items={displayItems}
        defaultEstimatedItemHeight={displaySettings.rowHeight}
        onLoadMore={loadMore}
        loadMoreThreshold={50}
        {hasMore}
        overlaySnippet={graphOverlay}
      >
        {#snippet renderItem(commit, index)}
          <CommitRow {commit} rowIndex={index} onselect={commit.oid === '__wip__' ? () => onWipClick?.() : oncommitselect} oncontextmenu={handleRowContextMenu} {maxColumns} {columnWidths} {columnVisibility} selected={commit.oid === selectedCommitOid && commit.oid !== '__wip__'} rowHeight={displaySettings.rowHeight} isSearchMatch={searchMatchOids.has(commit.oid)} isCurrentMatch={commit.oid === searchCurrentOid} isSearchActive={searchOpen && searchQuery.length > 0 && searchResults.length > 0} />
        {/snippet}
      </VirtualList>
      {/key}

      <!-- Mid-scroll skeleton (more commits loading) -->
      {#if loading && commits.length > 0}
        {#each { length: 3 } as _}
        <div class="flex items-center gap-2 animate-pulse" style="height: {displaySettings.rowHeight}px">
            <div
              class="rounded-full flex-shrink-0"
              style="background: var(--color-border); width: 64px; height: 12px;"
            ></div>
            <div
              class="rounded flex-shrink-0"
              style="background: var(--color-border); width: 32px; height: 100%;"
            ></div>
            <div
              class="rounded flex-1"
              style="background: var(--color-border); height: 12px;"
            ></div>
          </div>
        {/each}
      {/if}

      <!-- Mid-scroll error + retry -->
      {#if error && commits.length > 0}
        <div class="flex items-center gap-3 px-4 py-2">
          <span class="text-sm" style="color: #f87171;">{error}</span>
          <button
            onclick={loadMore}
            class="rounded px-3 py-1 text-xs font-medium"
            style="background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text);"
          >
            Retry
          </button>
        </div>
      {/if}
    {/if}
  </div>
</div>

{#if dialogConfig}
  <InputDialog
    title={dialogConfig.title}
    fields={dialogConfig.fields}
    onsubmit={dialogConfig.onsubmit}
    oncancel={closeDialog}
  />
{/if}

<style>
  .col-resize-handle {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 4px;
    cursor: col-resize;
    user-select: none;
    background: linear-gradient(to right, transparent 1.5px, var(--color-border) 1.5px, var(--color-border) 2.5px, transparent 2.5px);
    transition: background 0.15s;
  }
  .col-resize-handle:hover {
    background: linear-gradient(to right, transparent 1px, var(--color-accent) 1px, var(--color-accent) 3px, transparent 3px);
  }
  /* GRAPH-01: visible padding above first and below last commit row */
  :global(.virtual-list-viewport) {
    padding-top: 8px;
    padding-bottom: 8px;
    box-sizing: border-box;
    overflow-x: hidden;
  }
</style>
