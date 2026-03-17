<script lang="ts">
  import type { GraphCommit } from '../lib/types.js';
  import type { ColumnWidths, ColumnVisibility } from '../lib/store.js';
  import { LANE_WIDTH, ROW_HEIGHT, COLUMN_PADDING_X } from '../lib/graph-constants.js';

  interface Props {
    commit: GraphCommit;
    rowIndex: number;
    onselect?: (oid: string) => void;
    oncontextmenu?: (e: MouseEvent, commit: GraphCommit) => void;
    maxColumns?: number;
    columnWidths: ColumnWidths;
    columnVisibility: ColumnVisibility;
    selected?: boolean;
    /** Row height in px. Defaults to ROW_HEIGHT constant.
     *  Accepts displaySettings.rowHeight from CommitGraph for future settings-page wiring. */
    rowHeight?: number;
  }

  let { commit, rowIndex, onselect, oncontextmenu, maxColumns = 1, columnWidths, columnVisibility, selected = false, rowHeight = ROW_HEIGHT }: Props = $props();

  function relativeDate(ts: number): string {
    if (ts === 0) return '';
    const now = Date.now() / 1000;
    const diff = Math.max(0, now - ts);
    if (diff < 60) return 'just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 2592000) return `${Math.floor(diff / 86400)}d ago`;
    if (diff < 31536000) return `${Math.floor(diff / 2592000)}mo ago`;
    return `${Math.floor(diff / 31536000)}y ago`;
  }

  const isWip = $derived(commit.oid === '__wip__');
  const isStash = $derived(commit.is_stash);
</script>

<div
  class="relative flex items-center cursor-pointer text-[13px]"
  class:hover:bg-[var(--color-surface)]={!selected}
  style:height="{rowHeight}px"
  style="color: var(--color-text); {selected ? 'background: var(--color-selected-row);' : ''}"
  onclick={() => onselect?.(commit.oid)}
  oncontextmenu={(e: MouseEvent) => { if (oncontextmenu && !isWip) { e.preventDefault(); oncontextmenu(e, commit); } }}
>
  <!-- Column 1: Branch/Tag refs spacer (SVG overlay handles rendering) -->
  {#if columnVisibility.ref}
    <div class="flex-shrink-0" style="width: {columnWidths.ref}px; padding: 0 {COLUMN_PADDING_X}px;"></div>
  {/if}

  <!-- Column 2: Graph -->
  {#if columnVisibility.graph}
    <div class="relative z-[1] flex items-center flex-shrink-0 overflow-hidden" style="width: {columnWidths.graph}px; padding: 0 {COLUMN_PADDING_X}px;">
    </div>
  {/if}

  <!-- Column 3: Message (flex-1, always visible) -->
  {#if isWip || isStash}
    <div class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap italic" style="color: var(--color-text-muted); padding: 0 {COLUMN_PADDING_X}px;">
      {commit.summary}
    </div>
  {:else}
    <div class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap" style="padding: 0 {COLUMN_PADDING_X}px;">
      {commit.summary}
    </div>
  {/if}

  <!-- Column 4: Author -->
  {#if columnVisibility.author}
    <div class="flex-shrink-0 overflow-hidden text-ellipsis whitespace-nowrap text-[12px]" style="width: {columnWidths.author}px; color: var(--color-text-muted); padding: 0 {COLUMN_PADDING_X}px;">
      {#if !isWip && !isStash}{commit.author_name}{/if}
    </div>
  {/if}

  <!-- Column 5: Date -->
  {#if columnVisibility.date}
    <div class="flex-shrink-0 overflow-hidden whitespace-nowrap text-[11px]" style="width: {columnWidths.date}px; color: var(--color-text-muted); padding: 0 {COLUMN_PADDING_X}px;">
      {#if !isWip && !isStash}{relativeDate(commit.author_timestamp)}{/if}
    </div>
  {/if}

  <!-- Column 6: SHA -->
  {#if columnVisibility.sha}
    <div class="flex-shrink-0 font-mono text-[11px]" style="width: {columnWidths.sha}px; color: var(--color-text-muted); padding: 0 {COLUMN_PADDING_X}px;">
      {#if !isWip && !isStash}{commit.short_oid}{/if}
    </div>
  {/if}
</div>
