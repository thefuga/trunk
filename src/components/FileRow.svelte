<script lang="ts">
import { Minus, Plus } from "@lucide/svelte";
import type { FileStatus, FileStatusType } from "../lib/types.js";

interface Props {
	file: FileStatus;
	isLoading?: boolean;
	actionLabel: string;
	onaction: () => void;
	onclick?: () => void;
	oncontextmenu?: (e: MouseEvent) => void;
	depth?: number;
	displayName?: string;
	focused?: boolean;
}

let {
	file,
	isLoading = false,
	actionLabel,
	onaction,
	onclick,
	oncontextmenu,
	depth = 0,
	displayName,
	focused = false,
}: Props = $props();

let hovered = $state(false);

type StatusBadge = { letter: string; color: string };

const STATUS_BADGES: Record<FileStatusType, StatusBadge> = {
	New: { letter: "A", color: "var(--color-status-new)" },
	Modified: { letter: "M", color: "var(--color-status-modified)" },
	Deleted: { letter: "D", color: "var(--color-status-deleted)" },
	Renamed: { letter: "R", color: "var(--color-status-renamed)" },
	Typechange: { letter: "T", color: "var(--color-status-typechange)" },
	Conflicted: { letter: "C", color: "var(--color-status-conflicted)" },
};

let badge = $derived(
	STATUS_BADGES[file.status] ?? {
		letter: "?",
		color: "var(--color-text)",
	},
);

let badgeBg = $derived(
	isLoading
		? "transparent"
		: `color-mix(in oklch, ${badge.color} 6%, transparent)`,
);
</script>

<div
  data-testid="staging-file"
  role={depth > 0 ? 'treeitem' : 'listitem'}
  aria-level={depth > 0 ? depth + 1 : undefined}
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onclick={() => onclick?.()}
  oncontextmenu={(e) => { if (oncontextmenu) { e.preventDefault(); oncontextmenu(e); } }}
  style="
    height: 26px;
    padding: 0 8px;
    padding-left: {8 + depth * 16}px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: {onclick ? 'pointer' : 'default'};
    background: {focused ? 'var(--color-tree-focus)' : hovered ? 'var(--bg-hover)' : 'transparent'};
    color: {isLoading ? 'var(--color-text-muted)' : 'var(--color-text)'};
  "
>
  <!-- Status badge -->
  <span style="
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: var(--radius-s);
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 10px;
    line-height: 1;
    color: {isLoading ? 'var(--color-text-muted)' : badge.color};
    background: {badgeBg};
  ">{badge.letter}</span>

  <!-- Filename -->
  <span style="
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 12px;
  ">
    {displayName ?? file.path}
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
        color: {actionLabel === '+' ? 'var(--ok)' : 'var(--err)'};
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
