<script lang="ts">
import {
	FileMinus,
	FilePen,
	FilePlus,
	FileSymlink,
	FileType2,
	FileWarning,
	Minus,
	Plus,
} from "@lucide/svelte";
import type { Component } from "svelte";
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

type StatusIconConfig = {
	component: Component<Record<string, unknown>>;
	color: string;
};

const STATUS_ICON_COMPONENTS: Record<FileStatusType, StatusIconConfig> = {
	New: { component: FilePlus, color: "var(--color-status-new)" },
	Modified: { component: FilePen, color: "var(--color-status-modified)" },
	Deleted: { component: FileMinus, color: "var(--color-status-deleted)" },
	Renamed: { component: FileSymlink, color: "var(--color-status-renamed)" },
	Typechange: { component: FileType2, color: "var(--color-status-typechange)" },
	Conflicted: {
		component: FileWarning,
		color: "var(--color-status-conflicted)",
	},
};

let iconConfig = $derived(
	STATUS_ICON_COMPONENTS[file.status] ?? {
		component: FilePen,
		color: "var(--color-text-muted)",
	},
);

let Icon = $derived(iconConfig.component);
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
  <!-- Status icon -->
  <span style="
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    min-width: 14px;
    color: {isLoading ? 'var(--color-text-muted)' : iconConfig.color};
  ">
    <Icon size={12} color={isLoading ? 'var(--color-text-muted)' : iconConfig.color} />
  </span>

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
