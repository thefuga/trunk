<script lang="ts">
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import {
	getDiffViewMode, setDiffViewMode,
	getDiffIgnoreWhitespace, setDiffIgnoreWhitespace,
	getDiffShowFullFile, setDiffShowFullFile,
	getDiffShowInvisibles, setDiffShowInvisibles,
	getDiffWordWrap, setDiffWordWrap,
} from "../lib/store.js";
import type {
	CommitDetail,
	DiffLine,
	DiffOrigin,
	FileDiff,
	ViewMode,
} from "../lib/types.js";
import DiffToolbar from "./diff/DiffToolbar.svelte";
import DiffViewer from "./diff/DiffViewer.svelte";

interface Props {
	fileDiffs: FileDiff[];
	commitDetail: CommitDetail | null;
	selectedPath?: string | null;
	onclose: () => void;
	diffKind?: "unstaged" | "staged" | "commit";
	repoPath?: string;
	onhunkaction?: (filePath: string) => Promise<void>;
	ondiffoptionschange?: () => void;
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
	ondiffoptionschange,
	loading = false,
}: Props = $props();

let viewMode = $state<ViewMode>("hunk");
let ignoreWhitespace = $state(false);
let showInvisibles = $state(false);
let wordWrap = $state(false);
let prefsLoaded = $state(false);
let hunkOperationInFlight = $state(false);
let focusedHunkIndex = $state(0);
let hunkElements = $state<Record<string, HTMLDivElement>>({});

let selectedHunkKey = $state<string | null>(null);
let selectedLineIndices = $state<Set<number>>(new Set());
let lastClickedIndex = $state<number | null>(null);
let selectedCount = $derived(selectedLineIndices.size);

let collapsedFiles = $state<Set<string>>(new Set());

$effect(() => {
	Promise.all([
		getDiffViewMode(),
		getDiffIgnoreWhitespace(),
		getDiffShowInvisibles(),
		getDiffWordWrap(),
	]).then(([m, iw, si, ww]) => {
		viewMode = m;
		ignoreWhitespace = iw;
		showInvisibles = si;
		wordWrap = ww;
		prefsLoaded = true;
	}).catch(() => {});
});

async function handleViewModeChange(mode: ViewMode) {
	viewMode = mode;
	await setDiffViewMode(mode);
	const shouldShowFull = mode === "full";
	await setDiffShowFullFile(shouldShowFull);
	clearSelection();
	ondiffoptionschange?.();
}

async function handleIgnoreWhitespaceChange(value: boolean) {
	ignoreWhitespace = value;
	await setDiffIgnoreWhitespace(value);
	ondiffoptionschange?.();
}

function handleShowInvisiblesChange(value: boolean) {
	showInvisibles = value;
	setDiffShowInvisibles(value);
}

function handleWordWrapChange(value: boolean) {
	wordWrap = value;
	setDiffWordWrap(value);
}

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

function toggleFileCollapsed(path: string) {
	const next = new Set(collapsedFiles);
	if (next.has(path)) next.delete(path);
	else next.add(path);
	collapsedFiles = next;
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
</script>

<div style="height: 100%; display: flex; flex-direction: column; overflow: hidden; background: var(--color-bg);">
	{#if prefsLoaded}
	<DiffToolbar
		{viewMode}
		onviewmodechange={handleViewModeChange}
		selectedPath={selectedPath}
		{diffKind}
		{hunkOperationInFlight}
		{ignoreWhitespace}
		{showInvisibles}
		{wordWrap}
		onignorewhitespacechange={handleIgnoreWhitespaceChange}
		onshowinvisibleschange={handleShowInvisiblesChange}
		onwordwrapchange={handleWordWrapChange}
		onstagefile={handleStageFile}
		onunstagefile={handleUnstageFile}
		onclose={onclose}
	/>
	<DiffViewer
		{viewMode}
		{fileDiffs}
		{commitDetail}
		{selectedPath}
		{diffKind}
		{loading}
		{hunkOperationInFlight}
		{ignoreWhitespace}
		{showInvisibles}
		{wordWrap}
		{selectedHunkKey}
		{selectedLineIndices}
		{selectedCount}
		{collapsedFiles}
		{hunkElements}
		onfilecollapsetoggle={toggleFileCollapsed}
		onlineclick={handleLineClick}
		onstagehunk={handleStageHunk}
		onunstagehunk={handleUnstageHunk}
		ondiscardhunk={handleDiscardHunk}
		onstagelines={handleStageLines}
		onunstagelines={handleUnstageLines}
		ondiscardlines={handleDiscardLines}
	/>
	{/if}
</div>
