<script lang="ts">
import {
	COLUMN_PADDING_X,
	LANE_WIDTH,
	ROW_HEIGHT,
} from "../lib/graph-constants.js";
import type { ColumnVisibility, ColumnWidths } from "../lib/store.js";
import type { GraphCommit } from "../lib/types.js";

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
	/** True when this row's OID is in the search results */
	isSearchMatch?: boolean;
	/** True when this row is the current navigated match */
	isCurrentMatch?: boolean;
	/** True when any search is active (for dimming non-matches) */
	isSearchActive?: boolean;
	/** True when this commit is in the active review session (D-04 membership marker) */
	inSession?: boolean;
	/** True when this commit is the transient range-base highlight (D-01 support) */
	isPendingBase?: boolean;
}

let {
	commit,
	rowIndex,
	onselect,
	oncontextmenu,
	maxColumns = 1,
	columnWidths,
	columnVisibility,
	selected = false,
	rowHeight = ROW_HEIGHT,
	isSearchMatch = false,
	isCurrentMatch = false,
	isSearchActive = false,
	inSession = false,
	isPendingBase = false,
}: Props = $props();

function relativeDate(ts: number): string {
	if (ts === 0) return "";
	const now = Date.now() / 1000;
	const diff = Math.max(0, now - ts);
	if (diff < 60) return "just now";
	if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
	if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
	if (diff < 2592000) return `${Math.floor(diff / 86400)}d ago`;
	if (diff < 31536000) return `${Math.floor(diff / 2592000)}mo ago`;
	return `${Math.floor(diff / 31536000)}y ago`;
}

const isWip = $derived(commit.oid === "__wip__");
const isStash = $derived(commit.is_stash);

// D-04 in-session + D-01 pending-base markers: theme-variable inset accents on
// distinct edges so they compose with the background ternaries (and each other)
// without fighting them. Never an inline literal color, never the SVG pipeline.
const reviewMarker = $derived(
	[
		inSession ? "inset 3px 0 0 var(--color-review-row)" : "",
		isPendingBase ? "inset 0 -3px 0 var(--color-review-pending-base)" : "",
	]
		.filter(Boolean)
		.join(", "),
);
</script>

<div
  data-testid="commit-row"
  role="row"
  tabindex="0"
  class="relative flex items-center cursor-pointer text-[13px]"
  class:hover:bg-[var(--color-surface)]={!selected && !isCurrentMatch && !isSearchMatch}
  style:height="{rowHeight}px"
  style="color: var(--color-text); {isCurrentMatch ? 'background: rgba(245, 158, 11, 0.2);' : isSearchMatch ? 'background: rgba(234, 179, 8, 0.1);' : selected ? 'background: var(--color-selected-row);' : ''} {isSearchActive && !isSearchMatch && !isCurrentMatch ? 'opacity: 0.35;' : ''} {reviewMarker ? `box-shadow: ${reviewMarker};` : ''}"
  onclick={() => onselect?.(commit.oid)}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onselect?.(commit.oid); } }}
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
    <div data-testid="commit-row-summary" class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap italic" style="color: var(--color-text-muted); padding: 0 {COLUMN_PADDING_X}px;">
      {commit.summary}
    </div>
  {:else}
    <div data-testid="commit-row-summary" class="flex-1 overflow-hidden text-ellipsis whitespace-nowrap" style="padding: 0 {COLUMN_PADDING_X}px;">
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
