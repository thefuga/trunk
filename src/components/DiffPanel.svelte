<script lang="ts">
import { hunkSelectableIndices, resolveSide } from "../lib/diff-anchor.js";
import { buildFullFileAnchor } from "../lib/full-file-anchor.js";
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import {
	getDiffContentMode,
	getDiffContextLines,
	getDiffIgnoreWhitespace,
	getDiffLayoutMode,
	getDiffShowFullFile,
	getDiffShowInvisibles,
	getDiffWordWrap,
	setDiffContentMode,
	setDiffIgnoreWhitespace,
	setDiffLayoutMode,
	setDiffShowFullFile,
	setDiffShowInvisibles,
	setDiffWordWrap,
} from "../lib/store.js";
import { showToast } from "../lib/toast.svelte.js";
import type {
	CommitDetail,
	ContentMode,
	DiffLine,
	DiffOrigin,
	DiffRequestOptions,
	FileDiff,
	LayoutMode,
	SessionStatus,
	Side,
} from "../lib/types.js";
import CommentComposer from "./diff/CommentComposer.svelte";
import DiffToolbar from "./diff/DiffToolbar.svelte";
import DiffViewer from "./diff/DiffViewer.svelte";
import type FullFileView from "./diff/FullFileView.svelte";

interface Props {
	fileDiffs: FileDiff[];
	commitDetail: CommitDetail | null;
	selectedPath?: string | null;
	onclose: () => void;
	diffKind?: "unstaged" | "staged" | "commit";
	repoPath?: string;
	onhunkaction?: (filePath: string) => Promise<void>;
	onfileemptied?: (
		filePath: string,
		action: "stage" | "unstage" | "discard",
	) => void;
	ondiffoptionschange?: (options: DiffRequestOptions) => void;
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
	onfileemptied,
	ondiffoptionschange,
	loading = false,
}: Props = $props();

let contentMode = $state<ContentMode>("hunk");
let layoutMode = $state<LayoutMode>("inline");
let contextLines = $state(3);
let ignoreWhitespace = $state(false);
let showInvisibles = $state(false);
let wordWrap = $state(false);
let hunkOperationInFlight = $state(false);

let focusedHunkIndex = $state(0);
let hunkElements = $state<Record<string, HTMLDivElement>>({});

let selectedHunkKey = $state<string | null>(null);
let selectedLineIndices = $state<Set<number>>(new Set());
let lastClickedIndex = $state<number | null>(null);
let selectedCount = $derived(selectedLineIndices.size);

let collapsedFiles = $state<Set<string>>(new Set());

// Commit-diff comment composer host state (Phase 67). `commitOid` and `isMerge`
// derive from the threaded commitDetail; the composer opens for a specific
// (file, hunk) when the user clicks the Comment affordance.
//
// Working-tree (unstaged) commenting (260531-k4j): the anchor's commit_oid is the
// get-or-create snapshot OID returned by ensure_working_tree_snapshot, not a
// commitDetail oid (commitDetail is null for the working tree). Scope commitOid
// to the mode so a stale snapshot oid can never leak into staged/commit views.
let workingTreeSnapshotOid = $state<string | null>(null);
const commitOid = $derived(
	diffKind === "unstaged"
		? (workingTreeSnapshotOid ?? "")
		: (commitDetail?.oid ?? ""),
);
const isMerge = $derived((commitDetail?.parent_oids.length ?? 0) > 1);
let composerOpen = $state(false);
let composerFilePath = $state<string | null>(null);
let composerHunkIdx = $state<number | null>(null);
let composer = $state<CommentComposer | null>(null);

const composerFile = $derived(
	composerFilePath
		? (fileDiffs.find((f) => f.path === composerFilePath) ?? null)
		: null,
);

// Full-file capture host state (Phase 68). The selection lives in FullFileView;
// the host receives the chosen flat indices on the Comment affordance click and
// builds the captured FullFile anchor + plain excerpt via the 68-01 adapter,
// then reuses CommentComposer with that injected result. Merge commits ARE valid
// here (L-05) — no isMerge guard, unlike the diff path.
let fullFileView = $state<FullFileView | null>(null);
let fullFileComposerOpen = $state(false);
let fullFileComposerPath = $state<string | null>(null);
let fullFileSelectedIndices = $state<Set<number>>(new Set());

const fullFileCaptured = $derived(
	fullFileComposerPath
		? (() => {
				const fd = fileDiffs.find((f) => f.path === fullFileComposerPath);
				return fd
					? buildFullFileAnchor(commitOid, fd, fullFileSelectedIndices)
					: null;
			})()
		: null,
);

function closeComposer() {
	composerOpen = false;
	composerFilePath = null;
	composerHunkIdx = null;
	fullFileComposerOpen = false;
	fullFileComposerPath = null;
	fullFileSelectedIndices = new Set();
	workingTreeSnapshotOid = null;
	clearSelection();
}

// Auto-start a review session when the user goes to comment with none active.
// The add_comment / save_draft_comment commands stay dumb writers (L-08); the
// session is established here at the capture chokepoint so the composer never
// opens onto a backend that returns no_session. start/resume emit
// session-changed, so the review panel flips to "active" as visible feedback.
async function ensureActiveSession(): Promise<boolean> {
	let state: SessionStatus["state"];
	try {
		const status = await safeInvoke<SessionStatus>(
			"get_review_session_status",
			{ path: repoPath },
		);
		state = status.state;
	} catch (e) {
		showToast(
			(e as TrunkError).message ?? "Failed to load review session",
			"error",
		);
		return false;
	}

	if (state === "active") return true;

	const command =
		state === "resume-available"
			? "resume_review_session"
			: "start_review_session";
	try {
		await safeInvoke(command, { path: repoPath });
		return true;
	} catch (e) {
		showToast(
			(e as TrunkError).message ?? "Failed to start review session",
			"error",
		);
		return false;
	}
}

async function handleCommentLines(filePath: string, hunkIndex: number) {
	if (isMerge) return;

	// Working-tree (unstaged) scope guard (260531-k4j): New-side only this pass.
	// Compute the side with the SAME exported resolveSide buildDiffAnchor uses so
	// the guard and the anchor agree on the mixed Add+Delete → New edge case. An
	// Old-side (pure-delete / Deleted file) unstaged selection is a no-op + toast,
	// NOT an Old-side comment. Commit-mode is NOT guarded (byte-for-byte intact).
	if (diffKind === "unstaged") {
		const fd = fileDiffs.find((f) => f.path === filePath);
		const hunkLines = fd?.hunks[hunkIndex]?.lines ?? [];
		const selected = Array.from(selectedLineIndices)
			.sort((a, b) => a - b)
			.map((i) => hunkLines[i]);
		if (fd && resolveSide(fd.status, selected) === "Old") {
			showToast("Commenting on removed lines isn't supported yet", "error");
			return;
		}
	}

	const ready = await ensureActiveSession();
	if (!ready) return;

	// For the working tree, anchor to a get-or-create snapshot OID. Set it BEFORE
	// composerOpen so the composer reads the snapshot oid reactively as commitOid.
	if (diffKind === "unstaged") {
		try {
			workingTreeSnapshotOid = await safeInvoke<string>(
				"ensure_working_tree_snapshot",
				{ path: repoPath },
			);
		} catch (e) {
			showToast(
				(e as TrunkError).message ?? "Failed to snapshot working tree",
				"error",
			);
			return;
		}
	}

	composerFilePath = filePath;
	composerHunkIdx = hunkIndex;
	composerOpen = true;
}

// Whole-hunk Comment affordance (260531-l02): comment a hunk without first
// selecting lines. Synthesize a full-hunk selection (every non-context line,
// the same lines the user could click) then DELEGATE to handleCommentLines, so
// the isMerge guard, the unstaged Old-side scope guard (pure-delete → toast),
// ensureActiveSession, the snapshot fetch, and the composer are all reused
// verbatim — zero duplicated logic. A degenerate hunk with no selectable lines
// opens nothing.
async function handleCommentHunk(filePath: string, hunkIndex: number) {
	const fd = fileDiffs.find((f) => f.path === filePath);
	const hunk = fd?.hunks[hunkIndex];
	if (!hunk) return;

	const indices = hunkSelectableIndices(hunk);
	if (indices.size === 0) return;

	selectedHunkKey = `${filePath}-${hunkIndex}`;
	selectedLineIndices = indices;
	await handleCommentLines(filePath, hunkIndex);
}

// Full-file analog of handleCommentLines. NO isMerge guard — merge commits are
// valid for full-file capture (L-05). NO Old-side guard — full-file is always
// New-side by construction (buildFullFileAnchor, Phase 68). Reuses the same
// ensureActiveSession().
async function handleCommentFullFile(filePath: string, indices: Set<number>) {
	const ready = await ensureActiveSession();
	if (!ready) return;

	if (diffKind === "unstaged") {
		try {
			workingTreeSnapshotOid = await safeInvoke<string>(
				"ensure_working_tree_snapshot",
				{ path: repoPath },
			);
		} catch (e) {
			showToast(
				(e as TrunkError).message ?? "Failed to snapshot working tree",
				"error",
			);
			return;
		}
	}

	fullFileComposerPath = filePath;
	fullFileSelectedIndices = indices;
	fullFileComposerOpen = true;
}

$effect(() => {
	Promise.all([
		getDiffContentMode(),
		getDiffLayoutMode(),
		getDiffContextLines(),
		getDiffIgnoreWhitespace(),
		getDiffShowInvisibles(),
		getDiffWordWrap(),
	])
		.then(([cm, lm, cl, iw, si, ww]) => {
			contentMode = cm;
			layoutMode = lm;
			contextLines = cl;
			ignoreWhitespace = iw;
			showInvisibles = si;
			wordWrap = ww;
		})
		.catch(() => {});
});

function currentDiffOptions(
	overrides?: Partial<DiffRequestOptions>,
): DiffRequestOptions {
	return {
		contextLines,
		ignoreWhitespace,
		showFullFile: contentMode === "full",
		...overrides,
	};
}

async function handleContentModeChange(mode: ContentMode) {
	contentMode = mode;
	const shouldShowFull = mode === "full";
	clearSelection();
	ondiffoptionschange?.(currentDiffOptions({ showFullFile: shouldShowFull }));
	Promise.all([setDiffContentMode(mode), setDiffShowFullFile(shouldShowFull)]);
}

async function handleLayoutModeChange(mode: LayoutMode) {
	layoutMode = mode;
	clearSelection();
	setDiffLayoutMode(mode);
}

async function handleIgnoreWhitespaceChange(value: boolean) {
	ignoreWhitespace = value;
	ondiffoptionschange?.(currentDiffOptions({ ignoreWhitespace: value }));
	setDiffIgnoreWhitespace(value);
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
	// Reset the full-file view's own selection state so it never goes stale across
	// mode/layout toggles or Escape (null when not mounted in full-file mode).
	fullFileView?.clearSelection();
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

// Jump-to-range (Phase 69 / D-07): scroll to and transiently highlight the hunk
// that contains a comment's anchored line range. The review panel jump resolves
// to a single rendered file; we locate the hunk whose old/new-line span covers
// `startLine`, branching on `side` because an Old-side anchor's line number is
// a parent-tree coordinate and a New-side anchor's is a commit-tree coordinate.
// In hunk view the hunk wrapper is the scroll target. This is best-effort —
// if the line isn't currently rendered (e.g. full-file mode or no matching
// hunk), it falls back to the first hunk so the file is at least brought into
// view rather than leaving the user stranded.
export function scrollToLine(
	startLine: number,
	_endLine: number,
	side: Side = "New",
) {
	const keys = Object.keys(hunkElements);
	if (keys.length === 0) return;
	let targetKey: string | null = null;
	for (const fd of fileDiffs) {
		for (let hunkIdx = 0; hunkIdx < fd.hunks.length; hunkIdx++) {
			const hunk = fd.hunks[hunkIdx];
			const start = side === "Old" ? hunk.old_start : hunk.new_start;
			const lines = side === "Old" ? hunk.old_lines : hunk.new_lines;
			const end = start + lines - 1;
			if (startLine >= start && startLine <= end) {
				targetKey = `${fd.path}-${hunkIdx}`;
				break;
			}
		}
		if (targetKey !== null) break;
	}
	const index = targetKey !== null ? keys.indexOf(targetKey) : 0;
	scrollToHunk(index >= 0 ? index : 0);
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

function isLastHunk(filePath: string): boolean {
	const fd = fileDiffs.find((f) => f.path === filePath);
	return !fd || fd.hunks.length <= 1;
}

async function handleStageFile() {
	if (!selectedPath || hunkOperationInFlight) return;
	const path = selectedPath;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("stage_file", { path: repoPath, filePath: path });
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Stage file failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
	}
	onfileemptied?.(path, "stage");
}

async function handleUnstageFile() {
	if (!selectedPath || hunkOperationInFlight) return;
	const path = selectedPath;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("unstage_file", {
			path: repoPath,
			filePath: path,
		});
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Unstage file failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
	}
	onfileemptied?.(path, "unstage");
}

async function handleStageHunk(filePath: string, hunkIndex: number) {
	if (hunkOperationInFlight) return;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("stage_hunk", { path: repoPath, filePath, hunkIndex });
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Stage hunk failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
	}
	if (isLastHunk(filePath)) {
		onfileemptied?.(filePath, "stage");
	} else {
		await onhunkaction?.(filePath);
	}
}

async function handleUnstageHunk(filePath: string, hunkIndex: number) {
	if (hunkOperationInFlight) return;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("unstage_hunk", { path: repoPath, filePath, hunkIndex });
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Unstage hunk failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
	}
	if (isLastHunk(filePath)) {
		onfileemptied?.(filePath, "unstage");
	} else {
		await onhunkaction?.(filePath);
	}
}

async function handleDiscardHunk(filePath: string, hunkIndex: number) {
	if (hunkOperationInFlight) return;
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask("Discard this hunk? This cannot be undone.", {
		title: "Discard Hunk",
		kind: "warning",
	});
	if (!confirmed) return;

	hunkOperationInFlight = true;
	try {
		await safeInvoke("discard_hunk", { path: repoPath, filePath, hunkIndex });
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Discard hunk failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
	}
	showToast("Discarded hunk", "success");
	if (isLastHunk(filePath)) {
		onfileemptied?.(filePath, "discard");
	} else {
		await onhunkaction?.(filePath);
	}
}

async function handleLineClick(
	filePath: string,
	hunkIdx: number,
	lineIndex: number,
	origin: DiffOrigin,
	hunkLines: DiffLine[],
	e: MouseEvent,
) {
	if (origin === "Context") return;

	// D-02: switching to a new range while an open composer holds a dirty draft
	// prompts a discard confirmation. On cancel, keep the current selection and
	// composer; on confirm (or empty draft), close the composer and proceed.
	if (composerOpen && composer) {
		const proceed = await composer.confirmDiscardIfDirty();
		if (!proceed) return;
		composerOpen = false;
		composerFilePath = null;
		composerHunkIdx = null;
	}

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
	if (hunkOperationInFlight) return;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("stage_lines", {
			path: repoPath,
			filePath,
			hunkIndex,
			lineIndices: Array.from(selectedLineIndices),
		});
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Stage lines failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
		clearSelection();
	}
	await onhunkaction?.(filePath);
}

async function handleUnstageLines(filePath: string, hunkIndex: number) {
	if (hunkOperationInFlight) return;
	hunkOperationInFlight = true;
	try {
		await safeInvoke("unstage_lines", {
			path: repoPath,
			filePath,
			hunkIndex,
			lineIndices: Array.from(selectedLineIndices),
		});
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Unstage lines failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
		clearSelection();
	}
	await onhunkaction?.(filePath);
}

async function handleDiscardLines(filePath: string, hunkIndex: number) {
	if (hunkOperationInFlight) return;
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
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Discard lines failed", "error");
		return;
	} finally {
		hunkOperationInFlight = false;
		clearSelection();
	}
	showToast(`Discarded ${count} lines`, "success");
	await onhunkaction?.(filePath);
}
</script>

<div style="height: 100%; display: flex; flex-direction: column; overflow: hidden; background: var(--color-bg);">
	<DiffToolbar
		{contentMode}
		{layoutMode}
		oncontentmodechange={handleContentModeChange}
		onlayoutmodechange={handleLayoutModeChange}
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
		{contentMode}
		{layoutMode}
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
		{isMerge}
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
		oncommentlines={handleCommentLines}
		oncommenthunk={handleCommentHunk}
		{commitOid}
		{repoPath}
		oncommentfullfile={handleCommentFullFile}
		bind:fullFileView
	/>
	{#if composerOpen && composerFile && composerHunkIdx !== null}
		<CommentComposer
			bind:this={composer}
			file={composerFile}
			hunkIdx={composerHunkIdx}
			{selectedLineIndices}
			{commitOid}
			{repoPath}
			onclose={closeComposer}
		/>
	{:else if fullFileComposerOpen && fullFileCaptured}
		<CommentComposer
			bind:this={composer}
			captured={fullFileCaptured}
			{commitOid}
			{repoPath}
			onclose={closeComposer}
		/>
	{/if}
</div>
