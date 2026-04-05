<script lang="ts">
import {
	splitInvisibles,
	trailingWhitespaceStart,
} from "../../lib/diff-utils.js";
import type { FileDiff } from "../../lib/types.js";

interface Props {
	fileDiffs: FileDiff[];
	showInvisibles: boolean;
	wordWrap: boolean;
}

let { fileDiffs, showInvisibles, wordWrap }: Props = $props();

function lineBackground(origin: string): string {
	if (origin === "Add") return "var(--color-diff-add-bg)";
	if (origin === "Delete") return "var(--color-diff-delete-bg)";
	return "transparent";
}

function lineColor(origin: string): string {
	if (origin === "Add") return "var(--color-diff-add)";
	if (origin === "Delete") return "var(--color-diff-delete)";
	return "var(--color-text)";
}

function originSymbol(origin: string): string {
	if (origin === "Add") return "+";
	if (origin === "Delete") return "-";
	return " ";
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
  {@const allLines = fd.hunks.flatMap(h => h.lines)}
  <div style="min-width: 100%; width: max-content;">
    {#if fd.is_binary}
      <div style="
        padding: 8px;
        color: var(--color-text-muted);
        font-size: 12px;
      ">
        Binary file — no diff available
      </div>
    {:else}
      {#each allLines as line}
        {@const trailStart = showInvisibles ? trailingWhitespaceStart(line.content) : line.content.length}
        <div
          class="diff-line {line.origin === 'Add' ? 'diff-line-add' : line.origin === 'Delete' ? 'diff-line-delete' : 'diff-line-context'}"
          style="
            font-family: monospace;
            font-size: 12px;
            line-height: 1.5;
            padding: 0 8px;
            white-space: {wordWrap ? 'pre-wrap' : 'pre'};
            background: {lineBackground(line.origin)};
            color: {lineColor(line.origin)};
            display: flex;
            align-items: flex-start;
          "
        ><span style="min-width: {gutterW}; text-align: right; color: var(--color-text-muted); padding-right: 8px; user-select: none; flex-shrink: 0;">{line.old_lineno ?? ''}</span><span style="min-width: {gutterW}; text-align: right; color: var(--color-text-muted); padding-right: 8px; user-select: none; flex-shrink: 0;">{line.new_lineno ?? ''}</span><span class="diff-line-content">{#if line.spans.length > 0}<span>{originSymbol(line.origin)}</span>{#each line.spans as span}{@const sliced = line.content.slice(span.start, span.end)}{@const spanInTrailing = span.start >= trailStart}{#if showInvisibles}{@const segments = splitInvisibles(sliced, spanInTrailing || span.end > trailStart)}{#each segments as seg}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}{seg.isInvisible ? ' invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}<span class="{span.syntax_class}{span.emphasized ? (line.origin === 'Add' ? ' word-add' : ' word-delete') : ''}">{sliced}</span>{/if}{/each}{:else}{#if showInvisibles}{@const segments = splitInvisibles(line.content, false)}{originSymbol(line.origin)}{#each segments as seg}<span class="{seg.isInvisible ? 'invisible-char' : ''}{seg.isTrailing ? ' trailing-ws' : ''}">{seg.text}</span>{/each}{:else}{originSymbol(line.origin)}{line.content}{/if}{/if}</span></div>
      {/each}
    {/if}
  </div>
{/each}

<style>
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

  /* Desaturate syntax colors on add/delete backgrounds */
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
