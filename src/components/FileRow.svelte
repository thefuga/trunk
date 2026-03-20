<script lang="ts">
  import type { FileStatus, FileStatusType } from '../lib/types.js';
  import { FilePlus, FilePen, FileMinus, FileSymlink, FileType2, FileWarning, Plus, Minus } from '@lucide/svelte';
  import type { Component } from 'svelte';

  interface Props {
    file: FileStatus;
    isLoading?: boolean;
    actionLabel: string;
    onaction: () => void;
    onclick?: () => void;
    oncontextmenu?: (e: MouseEvent) => void;
  }

  let {
    file,
    isLoading = false,
    actionLabel,
    onaction,
    onclick,
    oncontextmenu,
  }: Props = $props();

  let hovered = $state(false);

  type StatusIconConfig = { component: Component<any>; color: string };

  const STATUS_ICON_COMPONENTS: Record<FileStatusType, StatusIconConfig> = {
    New:        { component: FilePlus,     color: '#22c55e' },
    Modified:   { component: FilePen,      color: '#fb923c' },
    Deleted:    { component: FileMinus,    color: '#f87171' },
    Renamed:    { component: FileSymlink,  color: '#60a5fa' },
    Typechange: { component: FileType2,    color: '#a78bfa' },
    Conflicted: { component: FileWarning,  color: '#facc15' },
  };

  let iconConfig = $derived(STATUS_ICON_COMPONENTS[file.status] ?? { component: FilePen, color: 'var(--color-text-muted)' });
</script>

<div
  role="listitem"
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onclick={() => onclick?.()}
  oncontextmenu={(e) => { if (oncontextmenu) { e.preventDefault(); oncontextmenu(e); } }}
  style="
    height: 26px;
    padding: 0 8px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: {onclick ? 'pointer' : 'default'};
    background: {hovered ? 'var(--color-surface)' : 'transparent'};
    color: {isLoading ? 'var(--color-text-muted)' : 'var(--color-text)'};
  "
>
  <!-- Status icon -->
  <span style="
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    min-width: 14px;
    color: {isLoading ? 'var(--color-text-muted)' : iconConfig.color};
  ">
    <svelte:component this={iconConfig.component} size={12} color={isLoading ? 'var(--color-text-muted)' : iconConfig.color} />
  </span>

  <!-- Filename -->
  <span style="
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
  ">
    {file.path}
  </span>

  <!-- Hover action button (hidden during loading or when no actionLabel) -->
  {#if hovered && !isLoading && actionLabel}
    <button
      onclick={(e) => { e.stopPropagation(); onaction(); }}
      aria-label={actionLabel === '+' ? 'Stage file' : 'Unstage file'}
      style="
        background: none;
        border: none;
        cursor: pointer;
        color: {actionLabel === '+' ? '#22c55e' : '#f87171'};
        display: flex;
        align-items: center;
        padding: 0 4px;
        line-height: 1;
      "
    >
      {#if actionLabel === '+'}
        <Plus size={11} />
      {:else}
        <Minus size={11} />
      {/if}
    </button>
  {/if}
</div>
