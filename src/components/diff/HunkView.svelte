<script lang="ts">
import {
	splitInvisibles,
	trailingWhitespaceStart,
} from "../../lib/diff-utils.js";
import type { DiffLine, DiffOrigin, FileDiff } from "../../lib/types.js";

interface Props {
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
	onstagehunk,
	onunstagehunk,
	ondiscardhunk,
	onstagelines,
	onunstagelines,
	ondiscardlines,
	oncommentlines,
	oncommenthunk,
}: Props = $props();

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
</script>

{#each fileDiffs as fd (fd.path)}
  {@const gutterW = gutterWidth(maxLineNumber(fd))}
  <div style="min-width: 100%; width: {wordWrap ? '100%' : 'max-content'};">
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
        left: 0;
        z-index: 2;
        width: 100cqi;
        box-sizing: border-box;
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
      <!-- Binary file fallback -->
      <div style="
        padding: 8px;
        color: var(--color-text-muted);
        font-size: 12px;
      ">
        Binary file — no diff available
      </div>
    {:else}
      <!-- Hunks -->
      {#each fd.hunks as hunk, hunkIdx}
        <!-- Hunk toolbar row -->
        <div
          bind:this={hunkElements[`${fd.path}-${hunkIdx}`]}
          style="
            background: var(--color-bg);
            display: flex;
            align-items: center;
            padding: 4px 8px;
            gap: 8px;
            position: sticky;
            left: 0;
            z-index: 1;
            width: 100cqi;
            box-sizing: border-box;
          "
        >
          <span style="flex: 1; color: var(--color-text-muted); font-size: 11px; font-family: var(--font-mono, monospace);">
            {hunk.header}
          </span>
          {#if diffKind === 'unstaged'}
            {@const hunkKey = `${fd.path}-${hunkIdx}`}
            {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
            {#if hasSelection}
              <button
                disabled={stagingDisabled}
              title={stagingDisabledTitle}
                style="
                  background: var(--color-danger-bg);
                  border: 1px solid var(--color-danger-border);
                  border-radius: 3px;
                  color: var(--color-danger);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {stagingDisabled ? 'not-allowed' : 'pointer'};
                  opacity: {stagingDisabled ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => ondiscardlines(fd.path, hunkIdx)}
              >
                Discard Lines ({selectedCount})
              </button>
              <button
                disabled={stagingDisabled}
              title={stagingDisabledTitle}
                style="
                  background: var(--color-success-bg);
                  border: 1px solid var(--color-success-border);
                  border-radius: 3px;
                  color: var(--color-success);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {stagingDisabled ? 'not-allowed' : 'pointer'};
                  opacity: {stagingDisabled ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => onstagelines(fd.path, hunkIdx)}
              >
                Stage Lines ({selectedCount})
              </button>
              <!-- Working-tree Comment affordance (260531-k4j): reuses the
                   commit-mode Comment button markup/styles verbatim (no new color).
                   New-side scope + Old-side guard live in the host. -->
              <button
                style="
                  background: var(--color-accent-bg, var(--color-surface));
                  border: 1px solid var(--color-border);
                  border-radius: 3px;
                  color: var(--color-accent);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: pointer;
                  white-space: nowrap;
                "
                onclick={() => oncommentlines(fd.path, hunkIdx)}
              >
                Comment ({selectedCount})
              </button>
            {:else}
              <button
                disabled={stagingDisabled}
              title={stagingDisabledTitle}
                style="
                  background: var(--color-danger-bg);
                  border: 1px solid var(--color-danger-border);
                  border-radius: 3px;
                  color: var(--color-danger);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {stagingDisabled ? 'not-allowed' : 'pointer'};
                  opacity: {stagingDisabled ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => ondiscardhunk(fd.path, hunkIdx)}
              >
                Discard Hunk
              </button>
              <button
                disabled={stagingDisabled}
              title={stagingDisabledTitle}
                style="
                  background: var(--color-success-bg);
                  border: 1px solid var(--color-success-border);
                  border-radius: 3px;
                  color: var(--color-success);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {stagingDisabled ? 'not-allowed' : 'pointer'};
                  opacity: {stagingDisabled ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => onstagehunk(fd.path, hunkIdx)}
              >
                Stage Hunk
              </button>
              <!-- Whole-hunk Comment affordance (260531-l02): comment the hunk
                   without selecting lines. Reuses the line-level accent button
                   markup verbatim (no new color); host synthesizes the full-hunk
                   selection + applies the New-side guard. -->
              <button
                style="
                  background: var(--color-accent-bg, var(--color-surface));
                  border: 1px solid var(--color-border);
                  border-radius: 3px;
                  color: var(--color-accent);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: pointer;
                  white-space: nowrap;
                "
                onclick={() => oncommenthunk(fd.path, hunkIdx)}
              >
                Comment
              </button>
            {/if}
          {:else if diffKind === 'staged'}
            {@const hunkKey = `${fd.path}-${hunkIdx}`}
            {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
            {#if hasSelection}
              <button
                disabled={stagingDisabled}
              title={stagingDisabledTitle}
                style="
                  background: var(--color-warning-bg);
                  border: 1px solid var(--color-warning-border);
                  border-radius: 3px;
                  color: var(--color-warning);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {stagingDisabled ? 'not-allowed' : 'pointer'};
                  opacity: {stagingDisabled ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => onunstagelines(fd.path, hunkIdx)}
              >
                Unstage Lines ({selectedCount})
              </button>
            {:else}
              <button
                disabled={stagingDisabled}
              title={stagingDisabledTitle}
                style="
                  background: var(--color-warning-bg);
                  border: 1px solid var(--color-warning-border);
                  border-radius: 3px;
                  color: var(--color-warning);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {stagingDisabled ? 'not-allowed' : 'pointer'};
                  opacity: {stagingDisabled ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => onunstagehunk(fd.path, hunkIdx)}
              >
                Unstage Hunk
              </button>
            {/if}
          {:else if diffKind === 'commit'}
            {@const hunkKey = `${fd.path}-${hunkIdx}`}
            {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
            {#if hasSelection}
              <button
                disabled={isMerge}
                title={isMerge ? "Diff comments aren't available on merge commits" : ""}
                style="
                  background: var(--color-accent-bg, var(--color-surface));
                  border: 1px solid var(--color-border);
                  border-radius: 3px;
                  color: var(--color-accent);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {isMerge ? 'not-allowed' : 'pointer'};
                  opacity: {isMerge ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => oncommentlines(fd.path, hunkIdx)}
              >
                Comment ({selectedCount})
              </button>
            {:else}
              <!-- Whole-hunk Comment in commit diffs (260531-l02): same accent
                   button + isMerge disable guard as the line-level commit Comment. -->
              <button
                disabled={isMerge}
                title={isMerge ? "Diff comments aren't available on merge commits" : ""}
                style="
                  background: var(--color-accent-bg, var(--color-surface));
                  border: 1px solid var(--color-border);
                  border-radius: 3px;
                  color: var(--color-accent);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 2px 8px;
                  cursor: {isMerge ? 'not-allowed' : 'pointer'};
                  opacity: {isMerge ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => oncommenthunk(fd.path, hunkIdx)}
              >
                Comment
              </button>
            {/if}
          {/if}
        </div>

        <!-- Diff lines -->
        {#each hunk.lines as line, lineIdx}
          {@const isSelectable = line.origin !== 'Context'}
          {@const hunkKey = `${fd.path}-${hunkIdx}`}
          {@const isSelected = selectedHunkKey === hunkKey && selectedLineIndices.has(lineIdx)}
          {@const trailStart = showInvisibles ? trailingWhitespaceStart(line.content) : line.content.length}
          <div
            class="diff-line {line.origin === 'Add' ? 'diff-line-add' : line.origin === 'Delete' ? 'diff-line-delete' : 'diff-line-context'}"
            role={isSelectable ? 'button' : undefined}
            style="
              font-family: monospace;
              font-size: 12px;
              line-height: 1.5;
              padding: 0 8px;
              white-space: {wordWrap ? 'pre-wrap' : 'pre'};
              background: {lineBackground(line.origin, isSelected)};
              color: {lineColor(line.origin)};
              cursor: {isSelectable ? 'pointer' : 'default'};
              -webkit-user-select: {isSelectable ? 'none' : 'text'};
              user-select: {isSelectable ? 'none' : 'text'};
              display: flex;
              align-items: flex-start;
            "
            onmousedown={(e) => { if (isSelectable && e.shiftKey) e.preventDefault(); }}
            onclick={(e) => isSelectable && onlineclick(fd.path, hunkIdx, lineIdx, line.origin, hunk.lines, e)}
            onkeydown={(e) => { if (isSelectable && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); onlineclick(fd.path, hunkIdx, lineIdx, line.origin, hunk.lines, new MouseEvent('click', { shiftKey: e.shiftKey })); } }}
          ><span style="min-width: {gutterW}; text-align: right; color: var(--color-text-muted); padding-right: 8px; user-select: none; flex-shrink: 0;">{line.old_lineno ?? ''}</span><span style="min-width: {gutterW}; text-align: right; color: var(--color-text-muted); padding-right: 8px; user-select: none; flex-shrink: 0;">{line.new_lineno ?? ''}</span><span class="diff-line-content">{#if line.spans.length > 0}{#each line.spans as span}{@const sliced = line.content.slice(span.start, span.end)}{@const spanInTrailing = span.start >= trailStart}{#if showInvisibles}{@const segments = splitInvisibles(sliced, spanInTrailing || span.end > trailStart)}{#each segments as seg}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}{seg.isInvisible ? ' invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}">{sliced}</span>{/if}{/each}{:else}{#if showInvisibles}{@const segments = splitInvisibles(line.content, false)}{#each segments as seg}<span class="{seg.isInvisible ? 'invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}{line.content}{/if}{/if}</span></div>
        {/each}
      {/each}
    {/if}
    {/if}
  </div>
{/each}

<style>
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

  /* Syntax highlighting classes -- text color from CSS custom properties (per D-03) */
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

  /* Desaturate syntax colors on add/delete backgrounds (per D-04, SYNT-03) */
  .diff-line-add [class*="syn-"],
  .diff-line-delete [class*="syn-"] {
    opacity: 0.7;
  }

  /* Invisible character styling (Phase 63 -- WHSP-03, D-11) */
  .invisible-char {
    color: var(--color-invisible);
  }

  /* Trailing whitespace warning (Phase 63 -- D-12) */
  .trailing-ws {
    background-color: var(--color-trailing-ws-bg);
    color: var(--color-invisible);
  }
</style>
