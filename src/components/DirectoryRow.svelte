<script lang="ts">
import { ChevronDown, ChevronRight, Minus, Plus } from "@lucide/svelte";
import type { DirectoryNode } from "../lib/build-tree.js";
import { countFiles } from "../lib/build-tree.js";

interface Props {
	node: DirectoryNode;
	depth: number;
	expanded: boolean;
	focused: boolean;
	ontoggle: () => void;
	actionLabel?: string;
	onaction?: () => void;
	oncontextmenu?: (e: MouseEvent) => void;
}

let {
	node,
	depth,
	expanded,
	focused,
	ontoggle,
	actionLabel = "",
	onaction,
	oncontextmenu,
}: Props = $props();

let hovered = $state(false);

let fileCount = $derived(countFiles(node.children));
</script>

<div
  role="treeitem"
  aria-expanded={expanded}
  aria-selected={focused}
  aria-level={depth + 1}
  tabindex="0"
  onmouseenter={() => (hovered = true)}
  onmouseleave={() => (hovered = false)}
  onclick={ontoggle}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); ontoggle(); } }}
  oncontextmenu={(e) => { if (oncontextmenu) { e.preventDefault(); oncontextmenu(e); } }}
  style="
    height: 26px;
    padding: 0 8px;
    padding-left: {8 + depth * 16}px;
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    background: {focused ? 'var(--color-tree-focus)' : hovered ? 'var(--color-surface)' : 'transparent'};
    color: var(--color-text);
    font-size: 12px;
  "
>
  <span style="display: inline-flex; align-items: center; color: var(--color-text-muted); width: 12px; min-width: 12px;">
    {#if expanded}
      <ChevronDown size={12} />
    {:else}
      <ChevronRight size={12} />
    {/if}
  </span>
  <span style="
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
  ">{node.name}</span>
  <span style="
    color: var(--color-text-muted);
    font-size: 11px;
    font-weight: 400;
    flex-shrink: 0;
  ">({fileCount})</span>
  <span style="flex: 1;"></span>
  {#if hovered && actionLabel && onaction}
    <button
      onclick={(e) => { e.stopPropagation(); onaction(); }}
      aria-label={actionLabel === '+' ? 'Stage directory' : 'Unstage directory'}
      style="
        background: none;
        border: none;
        cursor: pointer;
        color: {actionLabel === '+' ? 'var(--color-success)' : 'var(--color-danger)'};
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
