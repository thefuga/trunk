<script lang="ts">
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type {
	CommitDetail,
	DiffLine,
	DiffOrigin,
	FileDiff,
} from "../lib/types.js";

interface Props {
	fileDiffs: FileDiff[];
	commitDetail: CommitDetail | null;
	selectedPath?: string | null;
	onclose: () => void;
	diffKind?: "unstaged" | "staged" | "commit";
	repoPath?: string;
	onhunkaction?: (filePath: string) => Promise<void>;
	loading?: boolean;
}

let {
	fileDiffs,
	commitDetail,
	selectedPath = null,
	onclose,
	diffKind = "commit",
	repoPath = "",
	onhunkaction,
	loading = false,
}: Props = $props();

let hunkOperationInFlight = $state(false);
let focusedHunkIndex = $state(0);
let hunkElements: Record<string, HTMLDivElement> = {};

let selectedHunkKey = $state<string | null>(null);
let selectedLineIndices = $state<Set<number>>(new Set());
let lastClickedIndex = $state<number | null>(null);
let selectedCount = $derived(selectedLineIndices.size);

function clearSelection() {
	selectedHunkKey = null;
	selectedLineIndices = new Set();
	lastClickedIndex = null;
}

function scrollToHunk(index: number) {
	const keys = Object.keys(hunkElements);
	if (index < 0 || index >= keys.length) return;
	focusedHunkIndex = index;
	const el = hunkElements[keys[index]];
	el?.scrollIntoView({ behavior: "smooth", block: "start" });
	el?.classList.add("hunk-highlight");
	setTimeout(() => el?.classList.remove("hunk-highlight"), 600);
}

$effect(() => {
	function handleKeydown(e: KeyboardEvent) {
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

		if (e.key === "Escape" && selectedCount > 0) {
			e.preventDefault();
			clearSelection();
			return;
		}

		if (e.key === "]") {
			e.preventDefault();
			scrollToHunk(focusedHunkIndex + 1);
		} else if (e.key === "[") {
			e.preventDefault();
			scrollToHunk(focusedHunkIndex - 1);
		}
	}
	window.addEventListener("keydown", handleKeydown);
	return () => window.removeEventListener("keydown", handleKeydown);
});

let collapsedFiles = $state<Set<string>>(new Set());

function toggleFileCollapsed(path: string) {
	const next = new Set(collapsedFiles);
	if (next.has(path)) next.delete(path);
	else next.add(path);
	collapsedFiles = next;
}

$effect(() => {
	fileDiffs;
	focusedHunkIndex = 0;
	hunkElements = {};
	clearSelection();
	collapsedFiles = new Set();
});

async function handleStageFile() {
	if (!selectedPath) return;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("stage_file", { path: repoPath, filePath: selectedPath });
		await onhunkaction?.(selectedPath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Stage file failed", "error");
	} finally {
		hunkOperationInFlight = false;
	}
}

async function handleUnstageFile() {
	if (!selectedPath) return;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("unstage_file", {
			path: repoPath,
			filePath: selectedPath,
		});
		await onhunkaction?.(selectedPath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Unstage file failed", "error");
	} finally {
		hunkOperationInFlight = false;
	}
}

async function handleStageHunk(filePath: string, hunkIndex: number) {
	hunkOperationInFlight = true;
	try {
		await safeInvoke("stage_hunk", { path: repoPath, filePath, hunkIndex });
		await onhunkaction?.(filePath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Stage hunk failed", "error");
	} finally {
		hunkOperationInFlight = false;
	}
}

async function handleUnstageHunk(filePath: string, hunkIndex: number) {
	hunkOperationInFlight = true;
	try {
		await safeInvoke("unstage_hunk", { path: repoPath, filePath, hunkIndex });
		await onhunkaction?.(filePath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Unstage hunk failed", "error");
	} finally {
		hunkOperationInFlight = false;
	}
}

async function handleDiscardHunk(filePath: string, hunkIndex: number) {
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask("Discard this hunk? This cannot be undone.", {
		title: "Discard Hunk",
		kind: "warning",
	});
	if (!confirmed) return;

	hunkOperationInFlight = true;
	try {
		await safeInvoke("discard_hunk", { path: repoPath, filePath, hunkIndex });
		showToast("Discarded hunk", "success");
		await onhunkaction?.(filePath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Discard hunk failed", "error");
	} finally {
		hunkOperationInFlight = false;
	}
}

function handleLineClick(
	filePath: string,
	hunkIdx: number,
	lineIndex: number,
	origin: DiffOrigin,
	hunkLines: DiffLine[],
	e: MouseEvent,
) {
	if (origin === "Context") return;
	if (diffKind === "commit") return;

	const hunkKey = `${filePath}-${hunkIdx}`;

	if (hunkKey !== selectedHunkKey) {
		selectedHunkKey = hunkKey;
		selectedLineIndices = new Set([lineIndex]);
		lastClickedIndex = lineIndex;
		return;
	}

	if (e.shiftKey && lastClickedIndex !== null) {
		e.preventDefault();
		const start = Math.min(lastClickedIndex, lineIndex);
		const end = Math.max(lastClickedIndex, lineIndex);
		const newSet = new Set(selectedLineIndices);
		for (let i = start; i <= end; i++) {
			if (i < hunkLines.length && hunkLines[i].origin !== "Context") {
				newSet.add(i);
			}
		}
		selectedLineIndices = newSet;
	} else {
		const newSet = new Set(selectedLineIndices);
		if (newSet.has(lineIndex)) {
			newSet.delete(lineIndex);
		} else {
			newSet.add(lineIndex);
		}
		selectedLineIndices = newSet;
		lastClickedIndex = lineIndex;
	}
}

async function handleStageLines(filePath: string, hunkIndex: number) {
	hunkOperationInFlight = true;
	try {
		await safeInvoke("stage_lines", {
			path: repoPath,
			filePath,
			hunkIndex,
			lineIndices: Array.from(selectedLineIndices),
		});
		await onhunkaction?.(filePath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Stage lines failed", "error");
	} finally {
		hunkOperationInFlight = false;
		clearSelection();
	}
}

async function handleUnstageLines(filePath: string, hunkIndex: number) {
	hunkOperationInFlight = true;
	try {
		await safeInvoke("unstage_lines", {
			path: repoPath,
			filePath,
			hunkIndex,
			lineIndices: Array.from(selectedLineIndices),
		});
		await onhunkaction?.(filePath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Unstage lines failed", "error");
	} finally {
		hunkOperationInFlight = false;
		clearSelection();
	}
}

async function handleDiscardLines(filePath: string, hunkIndex: number) {
	const count = selectedCount;
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask(
		`Discard ${count} selected lines? This cannot be undone.`,
		{
			title: "Discard Lines",
			kind: "warning",
		},
	);
	if (!confirmed) return;

	hunkOperationInFlight = true;
	try {
		await safeInvoke("discard_lines", {
			path: repoPath,
			filePath,
			hunkIndex,
			lineIndices: Array.from(selectedLineIndices),
		});
		showToast(`Discarded ${count} lines`, "success");
		await onhunkaction?.(filePath);
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Discard lines failed", "error");
	} finally {
		hunkOperationInFlight = false;
		clearSelection();
	}
}

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

function originSymbol(origin: string): string {
	if (origin === "Add") return "+";
	if (origin === "Delete") return "-";
	return " ";
}
</script>

<div style="
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-bg);
">

  <!-- Panel toolbar: filename + close button -->
  <div style="
    height: 24px;
    border-bottom: 1px solid var(--color-border);
    padding: 0 8px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    gap: 4px;
  ">
    <span style="
      flex: 1;
      font-size: 11px;
      color: var(--color-text-muted);
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    ">{#if selectedPath}{selectedPath}{/if}</span>
    {#if diffKind === 'unstaged'}
      <button
        disabled={hunkOperationInFlight}
        style="
          background: var(--color-success-bg);
          border: 1px solid var(--color-success-border);
          border-radius: 3px;
          color: var(--color-success);
          font-size: 11px;
          font-family: var(--font-sans, sans-serif);
          padding: 2px 8px;
          cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
          opacity: {hunkOperationInFlight ? 0.4 : 1};
          white-space: nowrap;
          flex-shrink: 0;
        "
        onclick={handleStageFile}
      >
        Stage File
      </button>
    {:else if diffKind === 'staged'}
      <button
        disabled={hunkOperationInFlight}
        style="
          background: var(--color-warning-bg);
          border: 1px solid var(--color-warning-border);
          border-radius: 3px;
          color: var(--color-warning);
          font-size: 11px;
          font-family: var(--font-sans, sans-serif);
          padding: 2px 8px;
          cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
          opacity: {hunkOperationInFlight ? 0.4 : 1};
          white-space: nowrap;
          flex-shrink: 0;
        "
        onclick={handleUnstageFile}
      >
        Unstage File
      </button>
    {/if}
    <button
      onclick={onclose}
      aria-label="Close diff"
      style="
        background: none;
        border: none;
        cursor: pointer;
        color: var(--color-text-muted);
        font-size: 16px;
        line-height: 1;
        padding: 2px 4px;
        border-radius: 3px;
        flex-shrink: 0;
      "
    >✕</button>
  </div>

  <!-- Scrollable content area -->
  <div style="flex: 1; overflow-y: auto; min-height: 0;">

  <!-- Empty state -->
  {#if fileDiffs.length === 0 && commitDetail === null && !loading}
    <div style="
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      color: var(--color-text-muted);
      font-size: 13px;
    ">
      Select a file or commit to view its diff
    </div>
  {/if}

  <!-- File diff list -->
  {#each fileDiffs as fd (fd.path)}
    <div>
      <!-- File header bar (hidden for single-file view since top bar shows the path) -->
      {#if !selectedPath}
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div style="
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
        " onclick={() => toggleFileCollapsed(fd.path)}>
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
                  disabled={hunkOperationInFlight}
                  style="
                    background: var(--color-danger-bg);
                    border: 1px solid var(--color-danger-border);
                    border-radius: 3px;
                    color: var(--color-danger);
                    font-size: 11px;
                    font-family: var(--font-sans, sans-serif);
                    padding: 2px 8px;
                    cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                    opacity: {hunkOperationInFlight ? 0.4 : 1};
                    white-space: nowrap;
                  "
                  onclick={() => handleDiscardLines(fd.path, hunkIdx)}
                >
                  Discard Lines ({selectedCount})
                </button>
                <button
                  disabled={hunkOperationInFlight}
                  style="
                    background: var(--color-success-bg);
                    border: 1px solid var(--color-success-border);
                    border-radius: 3px;
                    color: var(--color-success);
                    font-size: 11px;
                    font-family: var(--font-sans, sans-serif);
                    padding: 2px 8px;
                    cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                    opacity: {hunkOperationInFlight ? 0.4 : 1};
                    white-space: nowrap;
                  "
                  onclick={() => handleStageLines(fd.path, hunkIdx)}
                >
                  Stage Lines ({selectedCount})
                </button>
              {:else}
                <button
                  disabled={hunkOperationInFlight}
                  style="
                    background: var(--color-danger-bg);
                    border: 1px solid var(--color-danger-border);
                    border-radius: 3px;
                    color: var(--color-danger);
                    font-size: 11px;
                    font-family: var(--font-sans, sans-serif);
                    padding: 2px 8px;
                    cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                    opacity: {hunkOperationInFlight ? 0.4 : 1};
                    white-space: nowrap;
                  "
                  onclick={() => handleDiscardHunk(fd.path, hunkIdx)}
                >
                  Discard Hunk
                </button>
                <button
                  disabled={hunkOperationInFlight}
                  style="
                    background: var(--color-success-bg);
                    border: 1px solid var(--color-success-border);
                    border-radius: 3px;
                    color: var(--color-success);
                    font-size: 11px;
                    font-family: var(--font-sans, sans-serif);
                    padding: 2px 8px;
                    cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                    opacity: {hunkOperationInFlight ? 0.4 : 1};
                    white-space: nowrap;
                  "
                  onclick={() => handleStageHunk(fd.path, hunkIdx)}
                >
                  Stage Hunk
                </button>
              {/if}
            {:else if diffKind === 'staged'}
              {@const hunkKey = `${fd.path}-${hunkIdx}`}
              {@const hasSelection = selectedHunkKey === hunkKey && selectedCount > 0}
              {#if hasSelection}
                <button
                  disabled={hunkOperationInFlight}
                  style="
                    background: var(--color-warning-bg);
                    border: 1px solid var(--color-warning-border);
                    border-radius: 3px;
                    color: var(--color-warning);
                    font-size: 11px;
                    font-family: var(--font-sans, sans-serif);
                    padding: 2px 8px;
                    cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                    opacity: {hunkOperationInFlight ? 0.4 : 1};
                    white-space: nowrap;
                  "
                  onclick={() => handleUnstageLines(fd.path, hunkIdx)}
                >
                  Unstage Lines ({selectedCount})
                </button>
              {:else}
                <button
                  disabled={hunkOperationInFlight}
                  style="
                    background: var(--color-warning-bg);
                    border: 1px solid var(--color-warning-border);
                    border-radius: 3px;
                    color: var(--color-warning);
                    font-size: 11px;
                    font-family: var(--font-sans, sans-serif);
                    padding: 2px 8px;
                    cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                    opacity: {hunkOperationInFlight ? 0.4 : 1};
                    white-space: nowrap;
                  "
                  onclick={() => handleUnstageHunk(fd.path, hunkIdx)}
                >
                  Unstage Hunk
                </button>
              {/if}
            {/if}
          </div>

          <!-- Diff lines -->
          {#each hunk.lines as line, lineIdx}
            {@const isSelectable = diffKind !== 'commit' && line.origin !== 'Context'}
            {@const hunkKey = `${fd.path}-${hunkIdx}`}
            {@const isSelected = selectedHunkKey === hunkKey && selectedLineIndices.has(lineIdx)}
            <div
              class="diff-line {line.origin === 'Add' ? 'diff-line-add' : line.origin === 'Delete' ? 'diff-line-delete' : 'diff-line-context'}"
              style="
                font-family: monospace;
                font-size: 12px;
                line-height: 1.5;
                padding: 0 8px;
                white-space: pre;
                overflow-x: auto;
                background: {lineBackground(line.origin, isSelected)};
                color: {lineColor(line.origin)};
                cursor: {isSelectable ? 'pointer' : 'default'};
                -webkit-user-select: {isSelectable ? 'none' : 'text'};
                user-select: {isSelectable ? 'none' : 'text'};
              "
              onmousedown={(e) => { if (isSelectable && e.shiftKey) e.preventDefault(); }}
              onclick={(e) => isSelectable && handleLineClick(fd.path, hunkIdx, lineIdx, line.origin, hunk.lines, e)}
            >{#if line.spans.length > 0}<span>{originSymbol(line.origin)}</span>{#each line.spans as span}<span
                class="{span.syntax_class}{span.emphasized
                  ? (line.origin === 'Add' ? ' word-add' : ' word-delete')
                  : ''}"
              >{line.content.slice(span.start, span.end)}</span>{/each}{:else}{originSymbol(line.origin)}{line.content}{/if}</div>
          {/each}
        {/each}
      {/if}
      {/if}
    </div>
  {/each}

  </div><!-- end scrollable content -->
</div>

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
</style>
