<script lang="ts">
  import Sortable from 'sortablejs';
  import { ROW_HEIGHT, COLUMN_PADDING_X } from '../lib/graph-constants.js';
  import { validateRebasePlan } from '../lib/rebase-validation.js';
  import type { RebaseTodoItem } from '../lib/types.js';
  import {
    getRebaseColumnWidths,
    setRebaseColumnWidths,
    getRebaseColumnVisibility,
    setRebaseColumnVisibility,
  } from '../lib/store.js';
  import type { RebaseColumnWidths, RebaseColumnVisibility } from '../lib/store.js';

  type RebaseAction = 'pick' | 'squash' | 'reword' | 'drop';

  interface RebaseCommit {
    oid: string;
    shortOid: string;
    summary: string;
    authorName: string;
    authorTimestamp: number;
    action: RebaseAction;
    newMessage: string | null;
  }

  interface Props {
    commits: RebaseTodoItem[];
    onclose: () => void;
    onstart: (items: { oid: string; action: string; summary: string; newMessage: string | null }[]) => void;
  }

  let { commits, onclose, onstart }: Props = $props();

  function toRebaseCommits(source: RebaseTodoItem[]): RebaseCommit[] {
    return source.map((c) => ({
      oid: c.oid,
      shortOid: c.short_oid,
      summary: c.summary,
      authorName: c.author_name,
      authorTimestamp: c.author_timestamp,
      action: 'pick' as RebaseAction,
      newMessage: null,
    }));
  }

  let items = $state<RebaseCommit[]>(toRebaseCommits(commits));
  let originalItems = $state<RebaseCommit[]>(structuredClone(toRebaseCommits(commits)));
  let focusedIndex = $state<number>(0);
  let listEl: HTMLDivElement | undefined = $state();
  let columnWidths = $state<RebaseColumnWidths>({ sha: 80, author: 120, date: 100 });
  let columnVisibility = $state<RebaseColumnVisibility>({ sha: true, author: true, date: true });

  let validationErrors = $derived(validateRebasePlan(items));
  let hasChanges = $derived(JSON.stringify(items) !== JSON.stringify(originalItems));
  let canStart = $derived(validationErrors.length === 0);

  // Load persisted column state on mount
  $effect(() => {
    getRebaseColumnWidths().then((w) => (columnWidths = w));
    getRebaseColumnVisibility().then((v) => (columnVisibility = v));
  });

  // --- Helpers ---

  function actionColor(action: string): string {
    switch (action) {
      case 'pick':
        return 'var(--color-rebase-pick)';
      case 'reword':
        return 'var(--color-rebase-reword)';
      case 'squash':
        return 'var(--color-rebase-squash)';
      case 'drop':
        return 'var(--color-rebase-drop)';
      default:
        return 'var(--color-text-muted)';
    }
  }

  function formatRelativeDate(timestamp: number): string {
    const now = Date.now() / 1000;
    const diff = Math.max(0, now - timestamp);
    const minutes = Math.floor(diff / 60);
    const hours = Math.floor(diff / 3600);
    const days = Math.floor(diff / 86400);

    if (days > 0) return `${days}d ago`;
    if (hours > 0) return `${hours}h ago`;
    return `${minutes}m ago`;
  }

  function errorForIndex(idx: number): string | null {
    const err = validationErrors.find((e) => e.index === idx);
    return err ? err.message : null;
  }

  // --- Column resize ---

  function startColumnResize(column: keyof RebaseColumnWidths, e: MouseEvent, invert = false) {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = columnWidths[column];
    const minWidths: Record<keyof RebaseColumnWidths, number> = { sha: 50, author: 60, date: 60 };
    const maxWidths: Record<keyof RebaseColumnWidths, number> = { sha: 120, author: 400, date: 400 };

    function onMouseMove(ev: MouseEvent) {
      const delta = (ev.clientX - startX) * (invert ? -1 : 1);
      const newWidth = Math.max(minWidths[column], Math.min(maxWidths[column], startWidth + delta));
      columnWidths = { ...columnWidths, [column]: newWidth };
    }

    function onMouseUp() {
      setRebaseColumnWidths(columnWidths);
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  // --- Header context menu ---

  async function showHeaderContextMenu(e: MouseEvent) {
    e.preventDefault();
    const { Menu, CheckMenuItem } = await import('@tauri-apps/api/menu');
    const cols: { key: keyof RebaseColumnVisibility; label: string }[] = [
      { key: 'sha', label: 'SHA' },
      { key: 'author', label: 'Author' },
      { key: 'date', label: 'Date' },
    ];
    const menuItems = await Promise.all(
      cols.map((col) =>
        CheckMenuItem.new({
          text: col.label,
          checked: columnVisibility[col.key],
          action: () => {
            columnVisibility = { ...columnVisibility, [col.key]: !columnVisibility[col.key] };
            setRebaseColumnVisibility(columnVisibility);
          },
        })
      )
    );
    const menu = await Menu.new({ items: menuItems });
    await menu.popup();
  }

  // --- Drag-and-drop (SortableJS) ---

  $effect(() => {
    if (!listEl) return;
    const sortable = Sortable.create(listEl, {
      animation: 150,
      forceFallback: true,
      ghostClass: 'rebase-row-ghost',
      chosenClass: 'rebase-row-chosen',
      dragClass: 'rebase-row-drag',
      fallbackClass: 'rebase-row-fallback',
      filter: 'select, option',
      preventOnFilter: false,
      onStart: (e) => {
        if (e.oldIndex != null) focusedIndex = e.oldIndex;
      },
      onEnd: (e) => {
        if (e.oldIndex == null || e.newIndex == null || e.oldIndex === e.newIndex) return;
        // Update state — {#key items} forces full DOM recreation so no conflict
        const updated = [...items];
        const [moved] = updated.splice(e.oldIndex, 1);
        updated.splice(e.newIndex, 0, moved);
        items = updated;
        focusedIndex = e.newIndex;
      },
    });
    return () => sortable.destroy();
  });

  // --- Keyboard shortcuts ---

  function scrollRowIntoView(idx: number) {
    const row = document.querySelector(`[data-rebase-row="${idx}"]`);
    row?.scrollIntoView({ block: 'nearest' });
  }

  function handleEditorKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement)?.tagName;
    if (tag === 'SELECT' || tag === 'INPUT' || tag === 'TEXTAREA') return;

    switch (e.key) {
      case 'p':
      case 'P':
        e.preventDefault();
        items[focusedIndex].action = 'pick';
        items[focusedIndex].newMessage = null;
        break;
      case 's':
      case 'S':
        e.preventDefault();
        items[focusedIndex].action = 'squash';
        if (!items[focusedIndex].newMessage) items[focusedIndex].newMessage = items[focusedIndex].summary;
        break;
      case 'r':
      case 'R':
        e.preventDefault();
        items[focusedIndex].action = 'reword';
        if (!items[focusedIndex].newMessage) items[focusedIndex].newMessage = items[focusedIndex].summary;
        break;
      case 'd':
      case 'D':
        e.preventDefault();
        items[focusedIndex].action = 'drop';
        items[focusedIndex].newMessage = null;
        break;
      case 'ArrowUp':
        e.preventDefault();
        if (e.shiftKey && focusedIndex > 0) {
          const updated = [...items];
          [updated[focusedIndex - 1], updated[focusedIndex]] = [updated[focusedIndex], updated[focusedIndex - 1]];
          items = updated;
          focusedIndex -= 1;
        } else if (!e.shiftKey) {
          focusedIndex = Math.max(0, focusedIndex - 1);
        }
        scrollRowIntoView(focusedIndex);
        break;
      case 'ArrowDown':
        e.preventDefault();
        if (e.shiftKey && focusedIndex < items.length - 1) {
          const updated = [...items];
          [updated[focusedIndex], updated[focusedIndex + 1]] = [updated[focusedIndex + 1], updated[focusedIndex]];
          items = updated;
          focusedIndex += 1;
        } else if (!e.shiftKey) {
          focusedIndex = Math.min(items.length - 1, focusedIndex + 1);
        }
        scrollRowIntoView(focusedIndex);
        break;
      case 'Escape':
        e.preventDefault();
        onclose();
        break;
    }
  }

  function autofocus(node: HTMLElement) {
    node.focus();
  }

  // --- Toolbar handlers ---

  function handleReset() {
    items = structuredClone(originalItems);
    focusedIndex = 0;
  }

  function handleCancel() {
    onclose();
  }

  function handleStartRebase() {
    if (!canStart) return;
    onstart(items.map((i) => ({ oid: i.oid, action: i.action, summary: i.summary, newMessage: i.newMessage })));
  }

  // Determine last visible resizable column for resize handle logic
  let lastVisibleColumn = $derived.by(() => {
    if (columnVisibility.date) return 'date';
    if (columnVisibility.author) return 'author';
    if (columnVisibility.sha) return 'sha';
    return 'message';
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="rebase-editor"
  tabindex="-1"
  onkeydown={handleEditorKeydown}
  use:autofocus
>
  <!-- Toolbar -->
  <div class="rebase-toolbar">
    <div class="rebase-toolbar-left">
      <span class="rebase-toolbar-title">Interactive Rebase</span>
      <span class="rebase-toolbar-badge">{items.length} commits</span>
    </div>
    <div class="rebase-toolbar-right">
      <button
        class="rebase-btn rebase-btn-ghost"
        disabled={!hasChanges}
        onclick={handleReset}
      >Reset</button>
      <button
        class="rebase-btn rebase-btn-ghost"
        onclick={handleCancel}
      >Cancel</button>
      <button
        class="rebase-btn rebase-btn-primary"
        disabled={!canStart}
        onclick={handleStartRebase}
      >Start Rebase</button>
    </div>
  </div>

  <!-- Column header -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="rebase-header"
    oncontextmenu={showHeaderContextMenu}
  >
    <div class="rebase-col-action" style="width: 90px; padding: 0 {COLUMN_PADDING_X}px;">
      Action
    </div>
    {#if columnVisibility.sha}
      <div class="rebase-col flex-shrink-0 relative" style="width: {columnWidths.sha}px; padding: 0 {COLUMN_PADDING_X}px;">
        SHA
        {#if 'sha' !== lastVisibleColumn}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('sha', e)}></div>
        {/if}
      </div>
    {/if}
    <div class="rebase-col flex-1 relative" style="padding: 0 {COLUMN_PADDING_X}px;">
      Message
      {#if 'message' !== lastVisibleColumn}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div class="col-resize-handle" onmousedown={(e) => startColumnResize('author', e, true)}></div>
      {/if}
    </div>
    {#if columnVisibility.author}
      <div class="rebase-col flex-shrink-0 relative" style="width: {columnWidths.author}px; padding: 0 {COLUMN_PADDING_X}px;">
        Author
        {#if 'author' !== lastVisibleColumn}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('date', e, true)}></div>
        {/if}
      </div>
    {/if}
    {#if columnVisibility.date}
      <div class="rebase-col flex-shrink-0 relative" style="width: {columnWidths.date}px; padding: 0 {COLUMN_PADDING_X}px;">
        Date
        {#if 'date' !== lastVisibleColumn}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="col-resize-handle" onmousedown={(e) => startColumnResize('date', e)}></div>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Commit list: {#key} forces DOM recreation after reorder so SortableJS and Svelte don't fight -->
  {#key items}
  <div class="rebase-list" bind:this={listEl}>
    {#each items as item, idx (item.oid)}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="rebase-row"
        class:rebase-row-focused={focusedIndex === idx}
        class:rebase-row-drop={item.action === 'drop'}
        data-rebase-row={idx}
        onclick={() => (focusedIndex = idx)}
        style="height: {ROW_HEIGHT}px;"
      >
        <!-- Action column -->
        <div class="rebase-cell-action" style="width: 90px; padding: 0 {COLUMN_PADDING_X}px;">
          <span
            class="rebase-action-dot"
            style="background: {actionColor(item.action)};"
          ></span>
          <select
            class="rebase-select"
            bind:value={item.action}
            onclick={(e) => e.stopPropagation()}
            onchange={() => {
              if (item.action === 'reword' || item.action === 'squash') {
                if (!item.newMessage) item.newMessage = item.summary;
              } else {
                item.newMessage = null;
              }
            }}
          >
            <option value="pick">Pick</option>
            <option value="reword">Reword</option>
            <option value="squash">Squash</option>
            <option value="drop">Drop</option>
          </select>
        </div>

        <!-- SHA column -->
        {#if columnVisibility.sha}
          <div
            class="rebase-cell flex-shrink-0"
            style="width: {columnWidths.sha}px; padding: 0 {COLUMN_PADDING_X}px; font-family: var(--font-mono);"
          >
            {item.shortOid}
          </div>
        {/if}

        <!-- Message column -->
        <div class="rebase-cell rebase-cell-message flex-1" style="padding: 0 {COLUMN_PADDING_X}px;">
          {#if item.newMessage != null}
            <input
              class="rebase-message-input"
              type="text"
              bind:value={item.newMessage}
              onclick={(e) => e.stopPropagation()}
              onkeydown={(e) => e.stopPropagation()}
            />
          {:else}
            <span class:rebase-text-drop={item.action === 'drop'}>{item.summary}</span>
          {/if}
        </div>

        <!-- Author column -->
        {#if columnVisibility.author}
          <div
            class="rebase-cell flex-shrink-0"
            style="width: {columnWidths.author}px; padding: 0 {COLUMN_PADDING_X}px;"
          >
            <span class:rebase-text-drop={item.action === 'drop'}>{item.authorName}</span>
          </div>
        {/if}

        <!-- Date column -->
        {#if columnVisibility.date}
          <div
            class="rebase-cell flex-shrink-0 rebase-cell-date"
            style="width: {columnWidths.date}px; padding: 0 {COLUMN_PADDING_X}px;"
          >
            <span class:rebase-text-drop={item.action === 'drop'}>{formatRelativeDate(item.authorTimestamp)}</span>
          </div>
        {/if}
      </div>

      <!-- Validation error inline -->
      {#if errorForIndex(idx)}
        <div class="rebase-validation-error">
          {errorForIndex(idx)}
        </div>
      {/if}
    {/each}
  </div>
  {/key}
</div>

<style>
  .rebase-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg);
    outline: none;
  }

  /* --- Toolbar --- */

  .rebase-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    flex-shrink: 0;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
  }

  .rebase-toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rebase-toolbar-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text);
  }

  .rebase-toolbar-badge {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .rebase-toolbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rebase-btn {
    border-radius: 4px;
    padding: 4px 12px;
    font-size: 11px;
    cursor: pointer;
    white-space: nowrap;
    font-family: var(--font-sans);
    font-weight: 600;
    border: none;
  }

  .rebase-btn:disabled {
    opacity: 0.5;
    cursor: default;
  }

  .rebase-btn-ghost {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    color: var(--color-text);
  }

  .rebase-btn-primary {
    background: var(--color-accent);
    color: white;
  }

  /* --- Column header --- */

  .rebase-header {
    display: flex;
    align-items: center;
    height: 24px;
    flex-shrink: 0;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .rebase-col-action {
    flex-shrink: 0;
  }

  .rebase-col {
    /* base column styling for header cells */
  }

  .col-resize-handle {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 4px;
    cursor: col-resize;
    background: linear-gradient(to right, transparent 1.5px, var(--color-border) 1.5px, var(--color-border) 2.5px, transparent 2.5px);
    transition: background 0.15s;
  }

  .col-resize-handle:hover {
    background: linear-gradient(to right, transparent 1px, var(--color-accent) 1px, var(--color-accent) 3px, transparent 3px);
  }

  /* --- Commit list --- */

  .rebase-list {
    flex: 1;
    overflow-y: auto;
  }

  .rebase-row {
    display: flex;
    align-items: center;
    font-size: 13px;
    color: var(--color-text);
    cursor: grab;
  }

  .rebase-row:hover:not(.rebase-row-focused) {
    background: var(--color-surface);
  }

  .rebase-row-focused {
    border-left: 2px solid var(--color-accent);
    background: var(--color-selected-row);
  }

  .rebase-row-drop {
    opacity: var(--color-rebase-drop-opacity);
  }

  :global(.rebase-row-ghost) {
    opacity: 0.4;
  }

  :global(.rebase-row-chosen) {
    background: var(--color-selected-row);
  }

  :global(.rebase-row-drag) {
    opacity: 0;
  }

  :global(.rebase-row-fallback) {
    background: var(--color-surface);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    opacity: 0.9;
  }

  .rebase-text-drop {
    text-decoration: line-through;
  }

  .rebase-message-input {
    width: 100%;
    background: var(--color-bg);
    border: 1px solid var(--color-accent);
    color: var(--color-text);
    font-size: 13px;
    font-family: var(--font-sans);
    padding: 0 4px;
    border-radius: 3px;
    outline: none;
  }

  /* --- Cells --- */

  .rebase-cell-action {
    display: flex;
    align-items: center;
    flex-shrink: 0;
    gap: 4px;
  }

  .rebase-action-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    vertical-align: middle;
    flex-shrink: 0;
  }

  .rebase-select {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    font-size: 11px;
    padding: 4px 4px;
    border-radius: 3px;
    cursor: pointer;
    font-family: var(--font-sans);
  }

  .rebase-cell {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rebase-cell-message {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rebase-cell-date {
    color: var(--color-text-muted);
  }

  /* --- Validation error --- */

  .rebase-validation-error {
    background: var(--color-rebase-error-bg);
    padding: 4px 12px;
    font-size: 11px;
    color: var(--color-rebase-error);
  }
</style>
