<script lang="ts">
import {
	type PairedRow,
	pairLines,
	splitInvisibles,
	trailingWhitespaceStart,
} from "../../lib/diff-utils.js";
import type {
	ContentMode,
	DiffLine,
	DiffOrigin,
	FileDiff,
} from "../../lib/types.js";

interface Props {
	contentMode: ContentMode;
	fileDiffs: FileDiff[];
	selectedPath: string | null;
	diffKind: "unstaged" | "staged" | "commit";
	hunkOperationInFlight: boolean;
	ignoreWhitespace: boolean;
	showInvisibles: boolean;
	wordWrap: boolean;
	selectedHunkKey: string | null;
	selectedLineIndices: Set<number>;
	selectedCount: number;
	isMerge: boolean;
	collapsedFiles: Set<string>;
	hunkElements: Record<string, HTMLDivElement>;
	onfilecollapsetoggle: (path: string) => void;
	onlineclick: (
		filePath: string,
		hunkIdx: number,
		lineIndex: number,
		origin: DiffOrigin,
		hunkLines: DiffLine[],
		e: MouseEvent,
	) => void;
	onlinemousedown: (
		filePath: string,
		hunkIdx: number,
		lineIndex: number,
		origin: DiffOrigin,
		hunkLines: DiffLine[],
		e: MouseEvent,
	) => void;
	onlineenter: (
		filePath: string,
		hunkIdx: number,
		lineIndex: number,
		e: MouseEvent,
	) => void;
	onstagehunk: (filePath: string, hunkIndex: number) => void;
	onunstagehunk: (filePath: string, hunkIndex: number) => void;
	ondiscardhunk: (filePath: string, hunkIndex: number) => void;
	onstagelines: (filePath: string, hunkIndex: number) => void;
	onunstagelines: (filePath: string, hunkIndex: number) => void;
	ondiscardlines: (filePath: string, hunkIndex: number) => void;
	oncommentlines: (filePath: string, hunkIndex: number) => void;
	oncommenthunk: (filePath: string, hunkIndex: number) => void;
}

let {
	contentMode,
	fileDiffs,
	selectedPath,
	diffKind,
	hunkOperationInFlight,
	ignoreWhitespace,
	showInvisibles,
	wordWrap,
	selectedHunkKey,
	selectedLineIndices,
	selectedCount,
	isMerge,
	collapsedFiles,
	hunkElements,
	onfilecollapsetoggle,
	onlineclick,
	onlinemousedown,
	onlineenter,
	onstagehunk,
	onunstagehunk,
	ondiscardhunk,
	onstagelines,
	onunstagelines,
	ondiscardlines,
	oncommentlines,
	oncommenthunk,
}: Props = $props();

const syncedCols: Set<HTMLElement> = new Set();
let syncing = false;

function splitColSync(node: HTMLElement) {
	syncedCols.add(node);

	function onScroll() {
		if (syncing) return;
		syncing = true;
		const { scrollLeft } = node;
		for (const col of syncedCols) {
			if (col !== node) col.scrollLeft = scrollLeft;
		}
		syncing = false;
	}

	node.addEventListener("scroll", onScroll);

	return {
		destroy() {
			node.removeEventListener("scroll", onScroll);
			syncedCols.delete(node);
		},
	};
}

const stagingDisabled = $derived(hunkOperationInFlight || ignoreWhitespace);
const stagingDisabledTitle = $derived(
	ignoreWhitespace
		? "Staging is disabled while whitespace changes are ignored"
		: undefined,
);

function lineBackground(origin: string, isSelected: boolean = false): string {
	if (origin === "Add")
		return isSelected
			? "var(--color-diff-add-bg-selected)"
			: "var(--color-diff-add-bg)";
	if (origin === "Delete")
		return isSelected
			? "var(--color-diff-delete-bg-selected)"
			: "var(--color-diff-delete-bg)";
	return "transparent";
}

function lineColor(origin: string): string {
	if (origin === "Add") return "var(--color-diff-add)";
	if (origin === "Delete") return "var(--color-diff-delete)";
	return "var(--color-text)";
}

function maxLineNumber(fd: FileDiff): number {
	let max = 0;
	for (const hunk of fd.hunks) {
		for (const line of hunk.lines) {
			if (line.old_lineno !== null && line.old_lineno > max)
				max = line.old_lineno;
			if (line.new_lineno !== null && line.new_lineno > max)
				max = line.new_lineno;
		}
	}
	return max;
}

function gutterWidth(maxNum: number): string {
	const digits = Math.max(String(maxNum).length, 1);
	return `${digits + 1}ch`;
}

interface Section {
	type: "header" | "lines";
	header?: string;
	hunkIdx: number;
	rows: PairedRow[];
	hunkLines?: DiffLine[];
}

const pairedData = $derived(
	fileDiffs.map((fd) => {
		const maxLn = maxLineNumber(fd);
		const gw = gutterWidth(maxLn);
		if (contentMode === "full") {
			const allLines = fd.hunks.flatMap((h) => h.lines);
			return {
				fd,
				gutterW: gw,
				sections: [
					{
						type: "lines" as const,
						rows: pairLines(allLines),
						hunkIdx: 0,
						hunkLines: allLines,
					},
				] as Section[],
			};
		}
		const sections: Section[] = fd.hunks.flatMap((hunk, hunkIdx) => [
			{
				type: "header" as const,
				header: hunk.header,
				hunkIdx,
				rows: [] as PairedRow[],
				hunkLines: hunk.lines,
			},
			{
				type: "lines" as const,
				rows: pairLines(hunk.lines),
				hunkIdx,
				hunkLines: hunk.lines,
			},
		]);
		return { fd, gutterW: gw, sections };
	}),
);
</script>

{#each pairedData as { fd, gutterW, sections } (fd.path)}
  <div class="split-file">
    <!-- File header bar (hidden for single-file view since top bar shows the path) -->
    {#if !selectedPath}
      <div
        role="button"
        tabindex="0"
        style="
        background: var(--color-surface);
        border-bottom: 1px solid var(--color-border);
        font-size: 12px;
        font-weight: 500;
        padding: 4px 8px;
        color: var(--color-text);
        position: sticky;
        top: 0;
        z-index: 1;
        cursor: pointer;
        user-select: none;
        display: flex;
        align-items: center;
        gap: 4px;
      "
        onclick={() => onfilecollapsetoggle(fd.path)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onfilecollapsetoggle(fd.path); } }}
      >
        <span style="font-size: 10px; color: var(--color-text-muted); width: 10px; display: inline-block;">{collapsedFiles.has(fd.path) ? '▶' : '▼'}</span>
        {fd.path}
      </div>
    {/if}

    {#if !collapsedFiles.has(fd.path)}
    {#if fd.is_binary}
      <div style="
        padding: 8px;
        color: var(--color-text-muted);
        font-size: 12px;
      ">
        Binary file — no diff available
      </div>
    {:else}
      {#each sections as section}
        {#if section.type === "header"}
          <!-- Hunk header spans full width -->
          <div
            bind:this={hunkElements[`${fd.path}-${section.hunkIdx}`]}
            class="split-hunk-header"
          >
            <span style="flex: 1; color: var(--color-text-muted); font-size: 11px; font-family: var(--font-mono, monospace);">
              {section.header}
            </span>
            {#if diffKind === 'unstaged'}
              {@const hunkKey = `${fd.path}-${section.hunkIdx}`}
              {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
              {#if hasSelection}
                <!-- Working-tree Comment affordance (260531-k4j): reuses the
                     commit-mode accent button class verbatim (no new color).
                     New-side scope + Old-side guard live in the host. Leads the
                     action cluster (260531-l02 UX: Comment left of staging). -->
                <button
                  class="staging-btn accent-btn"
                  style="cursor: pointer;"
                  onclick={() => oncommentlines(fd.path, section.hunkIdx)}
                >Comment ({selectedCount})</button>
                <button
                  disabled={stagingDisabled}
                  title={stagingDisabledTitle}
                  class="staging-btn danger-btn"
                  style="cursor: {stagingDisabled ? 'not-allowed' : 'pointer'}; opacity: {stagingDisabled ? 0.4 : 1};"
                  onclick={() => ondiscardlines(fd.path, section.hunkIdx)}
                >Discard Lines ({selectedCount})</button>
                <button
                  disabled={stagingDisabled}
                  title={stagingDisabledTitle}
                  class="staging-btn success-btn"
                  style="cursor: {stagingDisabled ? 'not-allowed' : 'pointer'}; opacity: {stagingDisabled ? 0.4 : 1};"
                  onclick={() => onstagelines(fd.path, section.hunkIdx)}
                >Stage Lines ({selectedCount})</button>
              {:else}
                <!-- Whole-hunk Comment affordance (260531-l02): comment the hunk
                     without selecting lines. Reuses the accent button class
                     verbatim (no new color); host applies the New-side guard.
                     Leads the action cluster. -->
                <button
                  class="staging-btn accent-btn"
                  style="cursor: pointer;"
                  onclick={() => oncommenthunk(fd.path, section.hunkIdx)}
                >Comment</button>
                <button
                  disabled={stagingDisabled}
                  title={stagingDisabledTitle}
                  class="staging-btn danger-btn"
                  style="cursor: {stagingDisabled ? 'not-allowed' : 'pointer'}; opacity: {stagingDisabled ? 0.4 : 1};"
                  onclick={() => ondiscardhunk(fd.path, section.hunkIdx)}
                >Discard Hunk</button>
                <button
                  disabled={stagingDisabled}
                  title={stagingDisabledTitle}
                  class="staging-btn success-btn"
                  style="cursor: {stagingDisabled ? 'not-allowed' : 'pointer'}; opacity: {stagingDisabled ? 0.4 : 1};"
                  onclick={() => onstagehunk(fd.path, section.hunkIdx)}
                >Stage Hunk</button>
              {/if}
            {:else if diffKind === 'staged'}
              {@const hunkKey = `${fd.path}-${section.hunkIdx}`}
              {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
              {#if hasSelection}
                <!-- Staged Comment (260531-l02b): index-snapshot anchored, both sides
                     resolve (no Old-side guard). Leads the cluster. -->
                <button
                  class="staging-btn accent-btn"
                  style="cursor: pointer;"
                  onclick={() => oncommentlines(fd.path, section.hunkIdx)}
                >Comment ({selectedCount})</button>
                <button
                  disabled={stagingDisabled}
                  title={stagingDisabledTitle}
                  class="staging-btn warning-btn"
                  style="cursor: {stagingDisabled ? 'not-allowed' : 'pointer'}; opacity: {stagingDisabled ? 0.4 : 1};"
                  onclick={() => onunstagelines(fd.path, section.hunkIdx)}
                >Unstage Lines ({selectedCount})</button>
              {:else}
                <button
                  class="staging-btn accent-btn"
                  style="cursor: pointer;"
                  onclick={() => oncommenthunk(fd.path, section.hunkIdx)}
                >Comment</button>
                <button
                  disabled={stagingDisabled}
                  title={stagingDisabledTitle}
                  class="staging-btn warning-btn"
                  style="cursor: {stagingDisabled ? 'not-allowed' : 'pointer'}; opacity: {stagingDisabled ? 0.4 : 1};"
                  onclick={() => onunstagehunk(fd.path, section.hunkIdx)}
                >Unstage Hunk</button>
              {/if}
            {:else if diffKind === 'commit'}
              {@const hunkKey = `${fd.path}-${section.hunkIdx}`}
              {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
              {#if hasSelection}
                <button
                  disabled={isMerge}
                  title={isMerge ? "Diff comments aren't available on merge commits" : ""}
                  class="staging-btn accent-btn"
                  style="cursor: {isMerge ? 'not-allowed' : 'pointer'}; opacity: {isMerge ? 0.4 : 1};"
                  onclick={() => oncommentlines(fd.path, section.hunkIdx)}
                >Comment ({selectedCount})</button>
              {:else}
                <!-- Whole-hunk Comment in commit diffs (260531-l02): same accent
                     class + isMerge disable guard as the line-level commit Comment. -->
                <button
                  disabled={isMerge}
                  title={isMerge ? "Diff comments aren't available on merge commits" : ""}
                  class="staging-btn accent-btn"
                  style="cursor: {isMerge ? 'not-allowed' : 'pointer'}; opacity: {isMerge ? 0.4 : 1};"
                  onclick={() => oncommenthunk(fd.path, section.hunkIdx)}
                >Comment</button>
              {/if}
            {/if}
          </div>
        {:else}
          <div class="split-columns">
            <!-- Left column (old content) -->
            <div class="split-column" use:splitColSync>
              <div style="min-width: 100%; width: {wordWrap ? '100%' : 'max-content'};">
              {#each section.rows as row}
                {#if row.left}
                  {@const line = row.left.line}
                  {@const isSelected = selectedHunkKey === `${fd.path}-${section.hunkIdx}` && selectedLineIndices.has(row.left.lineIdx)}
                  {@const trailStart = showInvisibles ? trailingWhitespaceStart(line.content) : line.content.length}
                  <div
                    class="diff-line {line.origin === 'Add' ? 'diff-line-add' : line.origin === 'Delete' ? 'diff-line-delete' : 'diff-line-context'}"
                    style="
                      background: {lineBackground(line.origin, isSelected)};
                      color: {lineColor(line.origin)};
                      white-space: {wordWrap ? 'pre-wrap' : 'pre'};
                    "
                  ><span class="gutter" style="min-width: {gutterW};">{line.old_lineno ?? ''}</span><span class="diff-line-content">{#if line.spans.length > 0}{#each line.spans as span}{@const sliced = line.content.slice(span.start, span.end)}{@const spanInTrailing = span.start >= trailStart}{#if showInvisibles}{@const segments = splitInvisibles(sliced, spanInTrailing || span.end > trailStart)}{#each segments as seg}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}{seg.isInvisible ? ' invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}">{sliced}</span>{/if}{/each}{:else}{#if showInvisibles}{@const segments = splitInvisibles(line.content, false)}{#each segments as seg}<span class="{seg.isInvisible ? 'invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}{line.content}{/if}{/if}</span></div>
                {:else}
                  <div class="split-phantom"></div>
                {/if}
              {/each}
              </div>
            </div>
            <!-- Right column (new content) -->
            <div class="split-column" use:splitColSync>
              <div style="min-width: 100%; width: {wordWrap ? '100%' : 'max-content'};">
              {#each section.rows as row}
                {#if row.right}
                  {@const line = row.right.line}
                  {@const isSelectable = line.origin === 'Add'}
                  {@const isSelected = selectedHunkKey === `${fd.path}-${section.hunkIdx}` && selectedLineIndices.has(row.right.lineIdx)}
                  {@const trailStart = showInvisibles ? trailingWhitespaceStart(line.content) : line.content.length}
                  <div
                    class="diff-line {line.origin === 'Add' ? 'diff-line-add' : line.origin === 'Delete' ? 'diff-line-delete' : 'diff-line-context'}"
                    role={isSelectable ? 'button' : undefined}
                    style="
                      background: {lineBackground(line.origin, isSelected)};
                      color: {lineColor(line.origin)};
                      cursor: {isSelectable ? 'pointer' : 'default'};
                      -webkit-user-select: {isSelectable ? 'none' : 'text'};
                      user-select: {isSelectable ? 'none' : 'text'};
                      white-space: {wordWrap ? 'pre-wrap' : 'pre'};
                    "
                    onmousedown={(e) => { if (isSelectable && section.hunkLines) onlinemousedown(fd.path, section.hunkIdx, row.right!.lineIdx, line.origin, section.hunkLines, e); }}
                    onmouseenter={(e) => onlineenter(fd.path, section.hunkIdx, row.right!.lineIdx, e)}
                    onkeydown={(e) => { if (isSelectable && (e.key === 'Enter' || e.key === ' ') && section.hunkLines) { e.preventDefault(); onlineclick(fd.path, section.hunkIdx, row.right!.lineIdx, line.origin, section.hunkLines, new MouseEvent('click', { shiftKey: e.shiftKey })); } }}
                  ><span class="gutter" style="min-width: {gutterW};">{line.new_lineno ?? ''}</span><span class="diff-line-content">{#if line.spans.length > 0}{#each line.spans as span}{@const sliced = line.content.slice(span.start, span.end)}{@const spanInTrailing = span.start >= trailStart}{#if showInvisibles}{@const segments = splitInvisibles(sliced, spanInTrailing || span.end > trailStart)}{#each segments as seg}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}{seg.isInvisible ? ' invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}">{sliced}</span>{/if}{/each}{:else}{#if showInvisibles}{@const segments = splitInvisibles(line.content, false)}{#each segments as seg}<span class="{seg.isInvisible ? 'invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}{line.content}{/if}{/if}</span></div>
                {:else}
                  <div class="split-phantom"></div>
                {/if}
              {/each}
              </div>
            </div>
          </div>
        {/if}
      {/each}
    {/if}
    {/if}
  </div>
{/each}

<style>
  .split-file {
    display: flex;
    flex-direction: column;
  }

  .split-columns {
    display: flex;
  }

  .split-column {
    flex: 1;
    min-width: 0;
    overflow-x: auto;
    overscroll-behavior-x: none;
    scrollbar-width: none;
  }

  .split-column::-webkit-scrollbar {
    display: none;
  }

  .split-column:first-child {
    border-right: 1px solid var(--color-border);
  }

  .diff-line {
    font-family: monospace;
    font-size: 12px;
    line-height: 1.5;
    padding: 0 8px;
    display: flex;
    align-items: flex-start;
  }

  .gutter {
    text-align: right;
    color: var(--color-text-muted);
    padding-right: 8px;
    user-select: none;
    flex-shrink: 0;
  }

  .split-phantom {
    font-family: monospace;
    font-size: 12px;
    line-height: 1.5;
    padding: 0 8px;
    background: var(--color-diff-phantom-bg);
  }

  .split-hunk-header {
    background: color-mix(in oklch, var(--info) 6%, var(--bg-2));
    color: color-mix(in oklch, var(--info) 70%, var(--fg-3));
    display: flex;
    align-items: center;
    padding: 4px 8px;
    gap: 8px;
  }

  .staging-btn {
    border-radius: 3px;
    font-size: 11px;
    font-family: var(--font-sans, sans-serif);
    padding: 2px 8px;
    white-space: nowrap;
  }

  .danger-btn {
    background: var(--color-danger-bg);
    border: 1px solid var(--color-danger-border);
    color: var(--color-danger);
  }

  .success-btn {
    background: var(--color-success-bg);
    border: 1px solid var(--color-success-border);
    color: var(--color-success);
  }

  .warning-btn {
    background: var(--color-warning-bg);
    border: 1px solid var(--color-warning-border);
    color: var(--color-warning);
  }

  .accent-btn {
    background: var(--color-accent-bg);
    border: 1px solid var(--color-border);
    color: var(--color-accent);
  }

  :global(.hunk-highlight) {
    animation: hunk-flash 0.6s ease-out;
  }
  @keyframes hunk-flash {
    0% { background-color: var(--color-hunk-flash); }
    100% { background-color: transparent; }
  }
  .word-add {
    background-color: var(--color-diff-word-add-bg);
    border-radius: 2px;
  }
  .word-delete {
    background-color: var(--color-diff-word-delete-bg);
    border-radius: 2px;
  }

  /* Syntax highlighting classes */
  .syn-keyword { color: var(--color-syn-keyword); }
  .syn-string { color: var(--color-syn-string); }
  .syn-comment { color: var(--color-syn-comment); }
  .syn-number { color: var(--color-syn-number); }
  .syn-type { color: var(--color-syn-type); }
  .syn-function { color: var(--color-syn-function); }
  .syn-variable { color: var(--color-syn-variable); }
  .syn-constant { color: var(--color-syn-constant); }
  .syn-operator { color: var(--color-syn-operator); }
  .syn-punctuation { color: var(--color-syn-punctuation); }
  .syn-attribute { color: var(--color-syn-attribute); }
  .syn-tag { color: var(--color-syn-tag); }
  .syn-property { color: var(--color-syn-property); }
  .syn-regex { color: var(--color-syn-regex); }
  .syn-escape { color: var(--color-syn-escape); }

  /* Change-indicator accent bar: saturated for add/delete, neutral rail for context.
     Every line carries the 3px border so columns stay aligned regardless of origin. */
  .diff-line {
    border-left: 3px solid var(--color-border);
  }
  .diff-line-add {
    border-left-color: var(--color-diff-add);
  }
  .diff-line-delete {
    border-left-color: var(--color-diff-delete);
  }

  /* Desaturate syntax colors on add/delete backgrounds */
  .diff-line-add [class*="syn-"],
  .diff-line-delete [class*="syn-"] {
    opacity: 0.7;
  }

  /* Invisible character styling */
  .invisible-char {
    color: var(--color-invisible);
  }

  /* Trailing whitespace warning */
  .trailing-ws {
    background-color: var(--color-trailing-ws-bg);
    color: var(--color-invisible);
  }
</style>
