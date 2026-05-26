<script lang="ts">
import { listen } from "@tauri-apps/api/event";
import { buildTree, collectFilePaths } from "../lib/build-tree.js";
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import type { RemoteState } from "../lib/remote-state.svelte.js";
import { createReviewSession } from "../lib/review-session.svelte.js";
import {
	getDiffContextLines,
	getDiffIgnoreWhitespace,
	getDiffShowFullFile,
	getFetchIntervalMs,
	getTreeViewEnabled,
	setLeftPaneCollapsed,
	setLeftPaneWidth,
	setRightPaneCollapsed,
	setRightPaneWidth,
	setTreeViewEnabled,
} from "../lib/store.js";
import { showToast } from "../lib/toast.svelte.js";
import type {
	Comment,
	CommitDetail as CommitDetailType,
	DiffRequestOptions,
	FileDiff,
	RebaseTodoItem,
	RefsResponse,
	WorkingTreeStatus,
} from "../lib/types.js";
import type { UndoRedoManager } from "../lib/undo-redo.svelte.js";
import BranchSidebar from "./BranchSidebar.svelte";
import CommitDetail from "./CommitDetail.svelte";
import CommitGraph from "./CommitGraph.svelte";
import DiffPanel from "./DiffPanel.svelte";
import MergeEditor from "./MergeEditor.svelte";
import RebaseEditor from "./RebaseEditor.svelte";
import ReviewPanel from "./ReviewPanel.svelte";
import StagingPanel from "./StagingPanel.svelte";

interface DirtyCounts {
	staged: number;
	unstaged: number;
	conflicted: number;
}

interface Props {
	repoPath: string;
	repoName: string;
	remoteState: RemoteState;
	undoRedo: UndoRedoManager;
	leftPaneWidth: number;
	leftPaneCollapsed: boolean;
	rightPaneWidth: number;
	rightPaneCollapsed: boolean;
	windowVisible: boolean;
	// Review mode is toggled by the OS menu (review-toggle) at the App level so the
	// global event only affects the active tab; App passes the flag down per tab.
	reviewActive: boolean;
	onleftpanecollapsedchange: (collapsed: boolean) => void;
	onrightpanecollapsedchange: (collapsed: boolean) => void;
	onleftpanewidthchange: (width: number) => void;
	onrightpanewidthchange: (width: number) => void;
}

let {
	repoPath,
	repoName,
	remoteState,
	undoRedo,
	leftPaneWidth,
	leftPaneCollapsed,
	rightPaneWidth,
	rightPaneCollapsed,
	windowVisible,
	reviewActive,
	onleftpanecollapsedchange,
	onrightpanecollapsedchange,
	onleftpanewidthchange,
	onrightpanewidthchange,
}: Props = $props();

// Center-pane Review-mode state (UI-SPEC:133, LOCKED to the center pane). The
// rune owns rightPaneMode (panel|diff); jumpTo composes the existing
// selection/scroll machinery via injected deps. App's review-toggle flag syncs
// into the rune so only the active tab enters review mode.
const reviewSession = createReviewSession();

$effect(() => {
	reviewSession.setReviewActive(reviewActive);
});

// DiffPanel ref for jump-to-range scroll+highlight (Phase 69 / D-07).
let diffPanelRef = $state<{
	scrollToLine: (startLine: number, endLine: number) => void;
} | null>(null);

// Bind the panel's jump affordance to the rune, wiring the existing RepoView
// machinery as the rune's navigation seams.
function handleReviewJump(comment: Comment) {
	reviewSession.jumpTo(comment, {
		selectCommit: handleCommitSelect,
		selectFile: handleCommitFileSelect,
		scrollToRange: (startLine, endLine) => {
			// The panel→diff swap destroys ReviewPanel and mounts a fresh DiffPanel;
			// diffPanelRef is bound during that render. Poll a few frames until it's
			// available before scrolling, so the jump never silently no-ops.
			const tryScroll = (retries = 3) => {
				if (diffPanelRef) {
					diffPanelRef.scrollToLine(startLine, endLine);
				} else if (retries > 0) {
					requestAnimationFrame(() => tryScroll(retries - 1));
				}
			};
			requestAnimationFrame(() => tryScroll());
		},
	});
}

// Commit-header click in the review panel: select the commit (loads detail)
// and scroll the graph to it. Panel stays open — no view swap.
async function handleReviewJumpToCommit(oid: string) {
	await handleCommitSelect(oid);
	await commitGraphRef?.scrollToOid(oid);
}

// Per-repo state
let refreshSignal = $state(0);
let dirtyCounts = $state<DirtyCounts>({
	staged: 0,
	unstaged: 0,
	conflicted: 0,
});
let headBranch = $state<string | undefined>(undefined);
let wipSubject = $state("");
let treeViewEnabled = $state(false);

// Staging file selection (from StagingPanel)
let selectedFile = $state<{
	path: string;
	kind: "unstaged" | "staged" | "conflicted";
} | null>(null);
let stagingDiffFiles = $state<FileDiff[]>([]);
let stagingDiffLoading = $state(false);
let selectGeneration = 0;
let cachedStatus = $state<WorkingTreeStatus | null>(null);
let stagingPanelRef = $state<StagingPanel | null>(null);

// Commit selection (from CommitGraph)
let selectedCommitOid = $state<string | null>(null);
let commitDetail = $state<CommitDetailType | null>(null);
let commitFileDiffs = $state<FileDiff[]>([]);
let selectedCommitFile = $state<string | null>(null);

// CommitGraph component ref -- used to call scrollToOid for ref navigation (GRAPH-03)
let commitGraphRef = $state<{
	scrollToOid: (oid: string) => Promise<void>;
} | null>(null);

// Rebase editor state
let showRebaseEditor = $state(false);
let rebaseEditorCommits = $state<RebaseTodoItem[]>([]);
let rebaseBaseOid = $state("");
let rebaseBranchName = $state("");
let rebaseBaseName = $state("");
let rebaseFocusedCommitDetail = $state<CommitDetailType | null>(null);
let rebaseFocusedFileDiffs = $state<FileDiff[]>([]);
let rebaseFocusedFileSelected = $state<string | null>(null);
let rebaseDiffFile = $state<string | null>(null);

const wipCount = $derived(
	dirtyCounts.staged + dirtyCounts.unstaged + dirtyCounts.conflicted,
);

// Center pane: show DiffPanel when a file is selected (from either source)
let showDiff = $derived(selectedFile !== null || selectedCommitFile !== null);
let showMergeEditor = $derived(selectedFile?.kind === "conflicted");

// The diffs to display: filtered commit file diff, or staging diff
let currentDiffFiles = $derived(
	selectedCommitFile
		? commitFileDiffs.filter((f) => f.path === selectedCommitFile)
		: stagingDiffFiles,
);

async function loadDirtyCounts() {
	try {
		const result = await safeInvoke<DirtyCounts>("get_dirty_counts", {
			path: repoPath,
		});
		dirtyCounts = result;
	} catch {
		// non-fatal -- keep previous counts
	}
}

async function loadHeadBranch() {
	try {
		const refs = await safeInvoke<RefsResponse>("list_refs", {
			path: repoPath,
		});
		headBranch = refs.local.find((b) => b.is_head)?.name;
	} catch {
		// non-fatal -- keep previous value
	}
}

function handleRefresh() {
	refreshSignal += 1;
}

function clearStagingDiff() {
	selectedFile = null;
	stagingDiffFiles = [];
	stagingDiffLoading = false;
}

function clearCommitFileDiff() {
	selectedCommitFile = null;
}

function clearCommit() {
	selectedCommitOid = null;
	commitDetail = null;
	commitFileDiffs = [];
	selectedCommitFile = null;
}

// Cached diff options — loaded once on mount, updated via ondiffoptionschange callback.
// Avoids 3 LazyStore IPC reads per file click.
let cachedDiffOptions = $state<DiffRequestOptions>({
	contextLines: 3,
	ignoreWhitespace: false,
	showFullFile: false,
});

$effect(() => {
	void repoPath; // re-load when repo changes
	Promise.all([
		getDiffContextLines(),
		getDiffIgnoreWhitespace(),
		getDiffShowFullFile(),
	])
		.then(([contextLines, ignoreWhitespace, showFullFile]) => {
			cachedDiffOptions = { contextLines, ignoreWhitespace, showFullFile };
		})
		.catch(() => {});
});

function buildDiffOptions(): DiffRequestOptions {
	return cachedDiffOptions;
}

/** WIP row clicked -- switch to staging view and auto-open right pane if collapsed. */
function handleWipClick() {
	clearCommit();
	// Auto-open right pane if collapsed (LAYOUT-01)
	if (rightPaneCollapsed) {
		onrightpanecollapsedchange(false);
	}
}

function handleDiffClose() {
	if (selectedFile) clearStagingDiff();
	else clearCommitFileDiff();
}

function advanceToNextFile(
	currentPath: string,
	section: "unstaged" | "staged" | "conflicted",
) {
	if (!cachedStatus) {
		clearStagingDiff();
		return;
	}
	const rawFiles = cachedStatus[section];
	// Use the same ordering the visual list uses
	const orderedPaths = treeViewEnabled
		? collectFilePaths(buildTree(rawFiles))
		: rawFiles.map((f) => f.path);
	const idx = orderedPaths.indexOf(currentPath);
	const nextPath =
		idx >= 0 ? (orderedPaths[idx + 1] ?? orderedPaths[idx - 1]) : undefined;
	if (nextPath) {
		handleFileSelect(nextPath, section);
	} else {
		clearStagingDiff();
	}
}

function handleFileResolved() {
	if (selectedFile) {
		advanceToNextFile(selectedFile.path, "conflicted");
	} else {
		clearStagingDiff();
	}
}

async function handleFileSelect(
	path: string,
	kind: "unstaged" | "staged" | "conflicted",
) {
	if (selectedFile?.path === path && selectedFile?.kind === kind) {
		clearStagingDiff();
		return;
	}
	selectedFile = { path, kind };
	if (!repoPath) return;
	if (kind === "conflicted") {
		// MergeEditor loads its own data via get_merge_sides
		stagingDiffFiles = [];
		return;
	}
	const gen = ++selectGeneration;
	stagingDiffLoading = true;
	try {
		const command = kind === "unstaged" ? "diff_unstaged" : "diff_staged";
		const options = buildDiffOptions();
		const result = await safeInvoke<FileDiff[]>(command, {
			path: repoPath,
			filePath: path,
			options,
		});
		if (gen !== selectGeneration) return;
		stagingDiffFiles = result;
	} catch (e) {
		if (gen !== selectGeneration) return;
		const err = e as TrunkError;
		showToast(err.message ?? "Failed to load diff", "error");
		stagingDiffFiles = [];
	} finally {
		if (gen === selectGeneration) {
			stagingDiffLoading = false;
		}
	}
}

async function handleCommitSelect(oid: string) {
	if (selectedCommitOid === oid) {
		clearCommit();
		return;
	}
	// Switching to commit view -- close any open staging diff
	clearStagingDiff();
	selectedCommitFile = null;

	// Auto-open right pane if collapsed (LAYOUT-01)
	if (rightPaneCollapsed) {
		onrightpanecollapsedchange(false);
	}

	selectedCommitOid = oid;
	if (!repoPath) return;
	try {
		const [files, detail] = await Promise.all([
			safeInvoke<FileDiff[]>("list_commit_files", {
				path: repoPath,
				oid,
			}),
			safeInvoke<CommitDetailType>("get_commit_detail", {
				path: repoPath,
				oid,
			}),
		]);
		commitFileDiffs = files;
		commitDetail = detail;
	} catch {
		commitFileDiffs = [];
		commitDetail = null;
	}
}

/** Resolve a ref name or OID to a commit OID, select it, and scroll the graph to it (GRAPH-03). */
async function handleRefNavigate(refNameOrOid: string) {
	if (!repoPath) return;

	let oid: string;

	// If it looks like a full git OID (40 hex chars), use directly (stash case)
	if (/^[0-9a-f]{40}$/i.test(refNameOrOid)) {
		oid = refNameOrOid;
	} else {
		// Resolve ref name to OID via backend
		try {
			oid = await safeInvoke<string>("resolve_ref", {
				path: repoPath,
				refName: refNameOrOid,
			});
		} catch {
			return; // ref not found -- ignore silently
		}
	}

	// Select commit (loads detail into right pane, also auto-opens pane via handleCommitSelect)
	await handleCommitSelect(oid);

	// Scroll graph to the commit row
	await commitGraphRef?.scrollToOid(oid);
}

async function handleCommitFileSelect(path: string) {
	if (selectedCommitFile === path) {
		clearCommitFileDiff();
		return;
	}
	selectedCommitFile = path;
	if (!repoPath || !selectedCommitOid) return;
	try {
		const options = buildDiffOptions();
		const fileDiffs = await safeInvoke<FileDiff[]>("diff_commit_file", {
			path: repoPath,
			oid: selectedCommitOid,
			filePath: path,
			options,
		});
		// Replace the lightweight entry with the raw diff data
		commitFileDiffs = commitFileDiffs.map((fd) =>
			fd.path === path && fileDiffs.length > 0 ? fileDiffs[0] : fd,
		);
	} catch {
		// Keep the lightweight entry — DiffPanel will show empty diff
	}
}

async function refetchFileDiff(
	path: string,
	kind: "unstaged" | "staged" | "conflicted",
	options?: DiffRequestOptions,
): Promise<boolean> {
	if (!repoPath) return false;
	if (kind === "conflicted") return false; // MergeEditor handles its own data loading
	const gen = selectGeneration;
	try {
		const command = kind === "unstaged" ? "diff_unstaged" : "diff_staged";
		const reloadOptions = options ?? buildDiffOptions();
		const result = await safeInvoke<FileDiff[]>(command, {
			path: repoPath,
			filePath: path,
			options: reloadOptions,
		});
		if (gen !== selectGeneration) return false;
		stagingDiffFiles = result;
		return result.length === 0 || result.every((f) => f.hunks.length === 0);
	} catch {
		if (gen !== selectGeneration) return false;
		stagingDiffFiles = [];
		return false;
	}
}

function handleTreeViewToggle() {
	treeViewEnabled = !treeViewEnabled;
	setTreeViewEnabled(treeViewEnabled);
}

// Load initial data
$effect(() => {
	void repoPath;
	loadDirtyCounts();
	loadHeadBranch();
	getTreeViewEnabled().then((v) => {
		treeViewEnabled = v;
	});
});

// Silent periodic background fetch. Pauses while the window is unfocused;
// backend swallows auth/rebase/busy cases so errors never surface.
$effect(() => {
	const path = repoPath;
	let timer: ReturnType<typeof setInterval> | undefined;
	let cancelled = false;

	(async () => {
		const intervalMs = await getFetchIntervalMs();
		if (cancelled || intervalMs <= 0) return;
		timer = setInterval(() => {
			if (!windowVisible) return;
			safeInvoke("git_fetch_background", { path }).catch(() => {});
		}, intervalMs);
	})();

	return () => {
		cancelled = true;
		if (timer) clearInterval(timer);
	};
});

// Listen for repo-changed events scoped to this repo
$effect(() => {
	let unlisten: (() => void) | undefined;
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	const path = repoPath;

	listen<string>("repo-changed", (event) => {
		if (event.payload === path) {
			if (debounceTimer) clearTimeout(debounceTimer);
			debounceTimer = setTimeout(() => {
				handleRefresh();
				loadDirtyCounts();
				loadHeadBranch();
				if (selectedFile) {
					refetchFileDiff(selectedFile.path, selectedFile.kind);
				}
			}, 200);
		}
	}).then((fn) => {
		unlisten = fn;
	});

	return () => {
		unlisten?.();
		if (debounceTimer) clearTimeout(debounceTimer);
	};
});

// Escape key handler for closing diffs
$effect(() => {
	function handleKeydown(e: KeyboardEvent) {
		if (
			e.key === "Escape" &&
			!showRebaseEditor &&
			(showDiff || showMergeEditor)
		) {
			e.preventDefault();
			handleDiffClose();
		}
	}
	window.addEventListener("keydown", handleKeydown);
	return () => window.removeEventListener("keydown", handleKeydown);
});

async function handleOpenRebaseEditor(baseOid: string, inclusive = false) {
	if (!repoPath) return;
	try {
		const todoItems = await safeInvoke<RebaseTodoItem[]>("get_rebase_todo", {
			path: repoPath,
			baseOid,
			inclusive,
		});
		if (todoItems.length === 0) return;
		rebaseEditorCommits = todoItems;
		rebaseBaseOid = baseOid;
		rebaseBranchName = headBranch ?? "HEAD";
		// Resolve base name: use short ref if possible
		try {
			const refs = await safeInvoke<RefsResponse>("list_refs", {
				path: repoPath,
			});
			const allBranches = [...refs.local, ...refs.remote];
			let foundName: string | null = null;
			for (const b of allBranches) {
				try {
					const branchOid = await safeInvoke<string>("resolve_ref", {
						path: repoPath,
						refName: b.name,
					});
					if (branchOid === baseOid) {
						foundName = b.name;
						break;
					}
				} catch {
					// ref resolution failed -- skip
				}
			}
			rebaseBaseName = foundName ?? baseOid.slice(0, 7);
		} catch {
			rebaseBaseName = baseOid.slice(0, 7);
		}
		// Clear any open diffs/selections before showing editor
		clearStagingDiff();
		clearCommit();
		rebaseFocusedCommitDetail = null;
		rebaseFocusedFileDiffs = [];
		rebaseFocusedFileSelected = null;
		showRebaseEditor = true;
	} catch (e) {
		const err = e as { message?: string };
		showToast(err.message ?? "Failed to load commits for rebase", "error");
	}
}

function handleRebaseEditorClose() {
	showRebaseEditor = false;
	rebaseEditorCommits = [];
	rebaseBaseOid = "";
	rebaseBranchName = "";
	rebaseBaseName = "";
	rebaseFocusedCommitDetail = null;
	rebaseFocusedFileDiffs = [];
	rebaseFocusedFileSelected = null;
	rebaseDiffFile = null;
}

async function handleRebaseFocusChange(oid: string) {
	if (!repoPath) return;
	rebaseFocusedFileSelected = null;
	rebaseDiffFile = null;
	try {
		const [detail, files] = await Promise.all([
			safeInvoke<CommitDetailType>("get_commit_detail", {
				path: repoPath,
				oid,
			}),
			safeInvoke<FileDiff[]>("list_commit_files", {
				path: repoPath,
				oid,
			}),
		]);
		rebaseFocusedCommitDetail = detail;
		rebaseFocusedFileDiffs = files;
	} catch {
		rebaseFocusedCommitDetail = null;
		rebaseFocusedFileDiffs = [];
	}
}

async function handleRebaseStart(
	todoItems: {
		oid: string;
		action: string;
		summary: string;
		newMessage: string | null;
	}[],
) {
	if (!repoPath) return;
	const baseOid = rebaseBaseOid;
	handleRebaseEditorClose();
	try {
		await safeInvoke("start_interactive_rebase", {
			path: repoPath,
			baseOid,
			todoItems,
		});
	} catch (e) {
		const err = e as { message?: string };
		showToast(err.message ?? "Rebase failed", "error");
	}
}

function startLeftResize(e: MouseEvent) {
	e.preventDefault();
	const startX = e.clientX;
	const startWidth = leftPaneCollapsed ? 0 : leftPaneWidth;

	function onMouseMove(ev: MouseEvent) {
		const newWidth = Math.max(0, startWidth + ev.clientX - startX);
		if (newWidth < 50) {
			onleftpanecollapsedchange(true);
		} else {
			onleftpanecollapsedchange(false);
			onleftpanewidthchange(Math.min(600, newWidth));
		}
	}

	function onMouseUp() {
		if (leftPaneCollapsed) {
			setLeftPaneCollapsed(true);
		} else {
			setLeftPaneWidth(leftPaneWidth);
			setLeftPaneCollapsed(false);
		}
		window.removeEventListener("mousemove", onMouseMove);
		window.removeEventListener("mouseup", onMouseUp);
	}

	window.addEventListener("mousemove", onMouseMove);
	window.addEventListener("mouseup", onMouseUp);
}

function startRightResize(e: MouseEvent) {
	e.preventDefault();
	const startX = e.clientX;
	const startWidth = rightPaneCollapsed ? 0 : rightPaneWidth;

	function onMouseMove(ev: MouseEvent) {
		const newWidth = Math.max(0, startWidth - (ev.clientX - startX));
		if (newWidth < 50) {
			onrightpanecollapsedchange(true);
		} else {
			onrightpanecollapsedchange(false);
			onrightpanewidthchange(Math.min(700, newWidth));
		}
	}

	function onMouseUp() {
		if (rightPaneCollapsed) {
			setRightPaneCollapsed(true);
		} else {
			setRightPaneWidth(rightPaneWidth);
			setRightPaneCollapsed(false);
		}
		window.removeEventListener("mousemove", onMouseMove);
		window.removeEventListener("mouseup", onMouseUp);
	}

	window.addEventListener("mousemove", onMouseMove);
	window.addEventListener("mouseup", onMouseUp);
}
</script>

<style>
  .pane-divider {
    width: 4px;
    flex-shrink: 0;
    cursor: col-resize;
    background: linear-gradient(to right, transparent 1.5px, var(--color-border) 1.5px, var(--color-border) 2.5px, transparent 2.5px);
    transition: background 0.15s;
  }
  .pane-divider:hover {
    background: linear-gradient(to right, transparent 1px, var(--color-accent) 1px, var(--color-accent) 3px, transparent 3px);
  }
</style>

<main class="flex-1 overflow-hidden flex">
  {#if showRebaseEditor}
    <!-- Full-window takeover for interactive rebase -->
    <div class="flex-1 overflow-hidden">
      <div style="height: 100%; {rebaseDiffFile ? 'display: none;' : 'display: flex; flex-direction: column;'}">
        <RebaseEditor
          {repoPath}
          commits={rebaseEditorCommits}
          branchName={rebaseBranchName}
          baseName={rebaseBaseName}
          onclose={handleRebaseEditorClose}
          onstart={handleRebaseStart}
          onfocuschange={handleRebaseFocusChange}
        />
      </div>
      {#if rebaseDiffFile}
          <DiffPanel
            fileDiffs={rebaseFocusedFileDiffs.filter((f) => f.path === rebaseDiffFile)}
            commitDetail={rebaseFocusedCommitDetail}
            selectedPath={rebaseDiffFile}
            diffKind="commit"
            {repoPath}
            onclose={() => { rebaseDiffFile = null; }}
          />
      {/if}
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="pane-divider" onmousedown={startRightResize}></div>
    <div style="width: {rightPaneCollapsed ? 0 : rightPaneWidth}px; flex-shrink: 0; overflow: hidden; display: flex; flex-direction: column;">
      {#if rebaseFocusedCommitDetail}
        <CommitDetail
          commitDetail={rebaseFocusedCommitDetail}
          fileDiffs={rebaseFocusedFileDiffs}
          selectedFile={rebaseFocusedFileSelected}
          onfileselect={(path) => {
            if (rebaseFocusedFileSelected === path) {
              rebaseFocusedFileSelected = null;
              rebaseDiffFile = null;
            } else {
              rebaseFocusedFileSelected = path;
              rebaseDiffFile = path;
            }
          }}
          onclose={() => { rebaseFocusedCommitDetail = null; }}
          {repoPath}
          {treeViewEnabled}
          ontreeviewtoggle={handleTreeViewToggle}
        />
      {:else}
        <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--color-text-muted); font-size: 13px;">
          Select a commit to view details
        </div>
      {/if}
    </div>
  {:else}
  <div style="width: {leftPaneCollapsed ? 0 : leftPaneWidth}px; flex-shrink: 0; overflow: hidden; display: flex; flex-direction: column;">
    <BranchSidebar {repoPath} onrefreshed={handleRefresh} onstashselect={handleCommitSelect} onrefnavigate={handleRefNavigate} {refreshSignal} onopenrebaseeditor={handleOpenRebaseEditor} />
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="pane-divider" style="display: {leftPaneCollapsed ? 'none' : 'block'};" onmousedown={startLeftResize}></div>
  <div class="flex-1 overflow-hidden">
    {#if reviewSession.state.reviewActive}
      <!-- Review mode claims the center pane (UI-SPEC:133). A persistent accent
           "Review" toggle returns from a jumped-to diff back to the panel. -->
      <div class="flex flex-col" style="height: 100%; min-height: 0;">
        <div class="flex items-center" style="gap: 8px; padding: 4px 8px; border-bottom: 1px solid var(--color-border); flex-shrink: 0; background: var(--color-surface);">
          <button
            type="button"
            onclick={() => reviewSession.showPanel()}
            style="
              background: {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-accent)' : 'transparent'};
              color: {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-bg)' : 'var(--color-text-muted)'};
              border: 1px solid {reviewSession.state.rightPaneMode === 'panel' ? 'var(--color-accent)' : 'var(--color-border)'};
              border-radius: 4px;
              cursor: pointer;
              padding: 2px 10px;
              font-size: 12px;
            "
          >Review</button>
        </div>
        <div class="flex flex-col" style="flex: 1; min-height: 0; overflow: hidden;">
          {#if reviewSession.state.rightPaneMode === 'diff' && showDiff}
            <DiffPanel
              bind:this={diffPanelRef}
              fileDiffs={currentDiffFiles}
              commitDetail={commitDetail}
              selectedPath={selectedCommitFile ?? selectedFile?.path ?? null}
              diffKind="commit"
              {repoPath}
              loading={stagingDiffLoading}
              onclose={() => { handleDiffClose(); reviewSession.showPanel(); }}
            />
          {:else}
            <ReviewPanel {repoPath} onJump={handleReviewJump} onJumpToCommit={handleReviewJumpToCommit} />
          {/if}
        </div>
      </div>
    {:else if showMergeEditor && selectedFile}
      <MergeEditor
        {repoPath}
        filePath={selectedFile.path}
        onclose={handleDiffClose}
        onresolved={handleFileResolved}
      />
    {:else if showDiff}
      <DiffPanel
        fileDiffs={currentDiffFiles}
        commitDetail={commitDetail}
        selectedPath={selectedCommitFile ?? selectedFile?.path ?? null}
        diffKind={selectedCommitFile ? 'commit' : (selectedFile?.kind === 'conflicted' ? 'commit' : selectedFile?.kind ?? 'commit')}
        {repoPath}
        loading={stagingDiffLoading}
        onhunkaction={async (filePath) => {
          if (selectedFile) {
            const { path, kind } = selectedFile;
            const isEmpty = await refetchFileDiff(filePath, kind);
            if (isEmpty && selectedFile?.path === path && selectedFile?.kind === kind) {
              advanceToNextFile(path, kind);
            }
          }
        }}
        onfileemptied={(filePath, action) => {
          if (selectedFile?.path === filePath) {
            const { kind } = selectedFile;
            advanceToNextFile(filePath, kind);
            stagingPanelRef?.optimisticMove(filePath, kind, action);
          }
        }}
        ondiffoptionschange={async (options) => {
          cachedDiffOptions = options;
          if (selectedFile && selectedFile.kind !== "conflicted") {
            await refetchFileDiff(selectedFile.path, selectedFile.kind, options);
          } else if (selectedCommitFile && selectedCommitOid) {
            try {
              const fileDiffs = await safeInvoke<FileDiff[]>("diff_commit_file", {
                path: repoPath,
                oid: selectedCommitOid,
                filePath: selectedCommitFile,
                options,
              });
              commitFileDiffs = commitFileDiffs.map((fd) =>
                fd.path === selectedCommitFile && fileDiffs.length > 0 ? fileDiffs[0] : fd,
              );
            } catch {
              // non-fatal
            }
          }
        }}
        onclose={handleDiffClose}
      />
    {:else}
      <CommitGraph bind:this={commitGraphRef} {repoPath} oncommitselect={handleCommitSelect} {wipCount} wipMessage={wipSubject.trim() || 'WIP'} onWipClick={handleWipClick} {refreshSignal} {selectedCommitOid} onopenrebaseeditor={handleOpenRebaseEditor} clearRedoStack={undoRedo.clear} />
    {/if}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="pane-divider" style="display: {rightPaneCollapsed ? 'none' : 'block'};" onmousedown={startRightResize}></div>
  <div style="width: {rightPaneCollapsed ? 0 : rightPaneWidth}px; flex-shrink: 0; overflow: hidden; display: flex; flex-direction: column;">
    {#if selectedCommitOid && commitDetail}
      <CommitDetail
        {commitDetail}
        fileDiffs={commitFileDiffs}
        selectedFile={selectedCommitFile}
        onfileselect={handleCommitFileSelect}
        onclose={clearCommit}
        {repoPath}
        {treeViewEnabled}
        ontreeviewtoggle={handleTreeViewToggle}
      />
    {:else}
      <StagingPanel
        bind:this={stagingPanelRef}
        {repoPath}
        currentBranch={headBranch}
        onfileselect={handleFileSelect}
        onsubjectchange={(v) => (wipSubject = v)}
        onfileresolved={handleFileResolved}
        onfileadvance={(path, kind) => {
          if (selectedFile?.path === path && selectedFile?.kind === kind) {
            advanceToNextFile(path, kind);
          }
        }}
        selectedPath={selectedFile?.path ?? null}
        selectedKind={selectedFile?.kind ?? null}
        onstatuschange={(s) => { cachedStatus = s; }}
        clearRedoStack={undoRedo.clear}
        {treeViewEnabled}
        ontreeviewtoggle={handleTreeViewToggle}
      />
    {/if}
  </div>
  {/if}
</main>
