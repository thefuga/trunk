<script lang="ts">
  import WelcomeScreen from './components/WelcomeScreen.svelte';
  import TabBar from './components/TabBar.svelte';
  import Toolbar from './components/Toolbar.svelte';
  import CommitGraph from './components/CommitGraph.svelte';
  import BranchSidebar from './components/BranchSidebar.svelte';
  import StagingPanel from './components/StagingPanel.svelte';
  import DiffPanel from './components/DiffPanel.svelte';
  import MergeEditor from './components/MergeEditor.svelte';
  import RebaseEditor from './components/RebaseEditor.svelte';
  import RebaseMessageEditor from './components/RebaseMessageEditor.svelte';
  import InputDialog from './components/InputDialog.svelte';
  import CommitDetail from './components/CommitDetail.svelte';
  import Toast from './components/Toast.svelte';
  import { safeInvoke } from './lib/invoke.js';
  import { showToast } from './lib/toast.svelte.js';
  import { getZoomLevel, setZoomLevel, getLeftPaneWidth, setLeftPaneWidth, getRightPaneWidth, setRightPaneWidth, getLeftPaneCollapsed, setLeftPaneCollapsed, getRightPaneCollapsed, setRightPaneCollapsed, getOpenRepo, setOpenRepo } from './lib/store.js';
  import type { FileDiff, CommitDetail as CommitDetailType, RefsResponse, WorkingTreeStatus, RebaseTodoItem } from './lib/types.js';

  interface DirtyCounts {
    staged: number;
    unstaged: number;
    conflicted: number;
  }
  import { listen } from '@tauri-apps/api/event';

  let zoomLevel = $state(1);
  let leftPaneWidth = $state(220);
  let leftPaneCollapsed = $state(false);
  let rightPaneWidth = $state(240);
  let rightPaneCollapsed = $state(false);
  let repoPath = $state<string | null>(null);
  let repoName = $state<string>('');
  let refreshSignal = $state(0);
  let dirtyCounts = $state<DirtyCounts>({ staged: 0, unstaged: 0, conflicted: 0 });
  let headBranch = $state<string | undefined>(undefined);
  let wipSubject = $state('');

  // Staging file selection (from StagingPanel)
  let selectedFile = $state<{ path: string; kind: 'unstaged' | 'staged' | 'conflicted' } | null>(null);
  let stagingDiffFiles = $state<FileDiff[]>([]);

  // Commit selection (from CommitGraph)
  let selectedCommitOid = $state<string | null>(null);
  let commitDetail = $state<CommitDetailType | null>(null);
  let commitFileDiffs = $state<FileDiff[]>([]);
  let selectedCommitFile = $state<string | null>(null);

  // CommitGraph component ref — used to call scrollToOid for ref navigation (GRAPH-03)
  let commitGraphRef = $state<{ scrollToOid: (oid: string) => Promise<void> } | null>(null);

  // Rebase editor state
  let showRebaseEditor = $state(false);
  let rebaseEditorCommits = $state<RebaseTodoItem[]>([]);
  let rebaseBaseOid = $state('');
  let rebaseBranchName = $state('');
  let rebaseBaseName = $state('');
  let rebaseFocusedCommitDetail = $state<CommitDetailType | null>(null);
  let rebaseFocusedFileDiffs = $state<FileDiff[]>([]);
  let rebaseFocusedFileSelected = $state<string | null>(null);
  let rebaseDiffFile = $state<string | null>(null); // when set, center pane shows this file's diff

  // Rebase message editor state (sequential center pane flow)
  interface SquashGroup {
    targetOid: string;
    allOids: string[];
    shortOids: string[];
    combinedMessage: string;
    actionType: 'reword' | 'squash';
  }
  let showRebaseMessageEditor = $state(false);
  let rebaseMessageQueue = $state<SquashGroup[]>([]);
  let rebaseMessageIdx = $state(0);
  let pendingRebaseTodoItems = $state<{ oid: string; action: string; summary: string; newMessage: string | null }[]>([]);
  let pendingRebaseBaseOid = $state('');

  const wipCount = $derived(dirtyCounts.staged + dirtyCounts.unstaged + dirtyCounts.conflicted);

  // Center pane: show DiffPanel when a file is selected (from either source)
  let showDiff = $derived(selectedFile !== null || selectedCommitFile !== null);
  let showMergeEditor = $derived(selectedFile?.kind === 'conflicted');

  // The diffs to display: filtered commit file diff, or staging diff
  let currentDiffFiles = $derived(
    selectedCommitFile
      ? commitFileDiffs.filter((f) => f.path === selectedCommitFile)
      : stagingDiffFiles
  );

  async function loadDirtyCounts() {
    if (!repoPath) return;
    try {
      const result = await safeInvoke<DirtyCounts>('get_dirty_counts', { path: repoPath });
      dirtyCounts = result;
    } catch {
      // non-fatal — keep previous counts
    }
  }

  async function loadHeadBranch() {
    if (!repoPath) return;
    try {
      const refs = await safeInvoke<RefsResponse>('list_refs', { path: repoPath });
      headBranch = refs.local.find(b => b.is_head)?.name;
    } catch {
      // non-fatal — keep previous value
    }
  }

  function handleOpen(path: string, name: string) {
    repoPath = path;
    repoName = name;
    setOpenRepo({ path, name });
  }

  function handleRefresh() {
    refreshSignal += 1;
  }

  function clearStagingDiff() {
    selectedFile = null;
    stagingDiffFiles = [];
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

  /** WIP row clicked — switch to staging view and auto-open right pane if collapsed. */
  function handleWipClick() {
    clearCommit();
    // Auto-open right pane if collapsed (LAYOUT-01)
    if (rightPaneCollapsed) {
      rightPaneCollapsed = false;
      setRightPaneCollapsed(false);
    }
  }

  function handleDiffClose() {
    if (selectedFile) clearStagingDiff();
    else clearCommitFileDiff();
  }

  async function handleFileResolved() {
    const resolvedPath = selectedFile?.path;
    if (!repoPath) { clearStagingDiff(); return; }
    try {
      const status = await safeInvoke<WorkingTreeStatus>('get_status', { path: repoPath });
      const next = status.conflicted.find(f => f.path !== resolvedPath);
      if (next) {
        handleFileSelect(next.path, 'conflicted');
      } else {
        clearStagingDiff();
      }
    } catch {
      clearStagingDiff();
    }
  }

  async function handleFileSelect(path: string, kind: 'unstaged' | 'staged' | 'conflicted') {
    if (selectedFile?.path === path && selectedFile?.kind === kind) {
      clearStagingDiff();
      return;
    }
    selectedFile = { path, kind };
    if (!repoPath) return;
    if (kind === 'conflicted') {
      // MergeEditor loads its own data via get_merge_sides
      stagingDiffFiles = [];
      return;
    }
    try {
      const command = kind === 'unstaged' ? 'diff_unstaged' : 'diff_staged';
      stagingDiffFiles = await safeInvoke<FileDiff[]>(command, { path: repoPath, filePath: path });
    } catch {
      stagingDiffFiles = [];
    }
  }

  async function handleCommitSelect(oid: string) {
    if (selectedCommitOid === oid) {
      clearCommit();
      return;
    }
    // Switching to commit view — close any open staging diff
    clearStagingDiff();
    selectedCommitFile = null;

    // Auto-open right pane if collapsed (LAYOUT-01)
    if (rightPaneCollapsed) {
      rightPaneCollapsed = false;
      setRightPaneCollapsed(false);
    }

    selectedCommitOid = oid;
    if (!repoPath) return;
    try {
      const [files, detail] = await Promise.all([
        safeInvoke<FileDiff[]>('diff_commit', { path: repoPath, oid }),
        safeInvoke<CommitDetailType>('get_commit_detail', { path: repoPath, oid }),
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
        oid = await safeInvoke<string>('resolve_ref', { path: repoPath, refName: refNameOrOid });
      } catch {
        return; // ref not found — ignore silently
      }
    }

    // Select commit (loads detail into right pane, also auto-opens pane via handleCommitSelect)
    await handleCommitSelect(oid);

    // Scroll graph to the commit row
    await commitGraphRef?.scrollToOid(oid);
  }

  function handleCommitFileSelect(path: string) {
    if (selectedCommitFile === path) {
      clearCommitFileDiff();
      return;
    }
    selectedCommitFile = path;
  }

  async function refetchFileDiff(path: string, kind: 'unstaged' | 'staged' | 'conflicted') {
    if (!repoPath) return;
    if (kind === 'conflicted') return; // MergeEditor handles its own data loading
    try {
      const command = kind === 'unstaged' ? 'diff_unstaged' : 'diff_staged';
      stagingDiffFiles = await safeInvoke<FileDiff[]>(command, { path: repoPath, filePath: path });
    } catch {
      stagingDiffFiles = [];
    }
  }

  $effect(() => {
    if (repoPath) {
      loadDirtyCounts();
      loadHeadBranch();
    }
  });

  $effect(() => {
    let unlisten: (() => void) | undefined;
    let debounceTimer: ReturnType<typeof setTimeout> | undefined;

    listen<string>('repo-changed', (event) => {
      if (event.payload === repoPath) {
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
    }).then((fn) => { unlisten = fn; });

    return () => {
      unlisten?.();
      if (debounceTimer) clearTimeout(debounceTimer);
    };
  });


  $effect(() => {
    getOpenRepo().then(async (repo) => {
      if (!repo) return;
      try {
        await safeInvoke('open_repo', { path: repo.path });
        repoPath = repo.path;
        repoName = repo.name;
      } catch {
        setOpenRepo(null);
      }
    });
  });

  $effect(() => {
    getZoomLevel().then((level) => { zoomLevel = level; });
  });

  $effect(() => {
    getLeftPaneWidth().then((w) => { leftPaneWidth = w; });
    getRightPaneWidth().then((w) => { rightPaneWidth = w; });
    getLeftPaneCollapsed().then((c) => { leftPaneCollapsed = c; });
    getRightPaneCollapsed().then((c) => { rightPaneCollapsed = c; });
  });

  $effect(() => {
    document.documentElement.style.zoom = String(zoomLevel);
  });

  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      if (e.key === 'Escape' && !showRebaseEditor && (showDiff || showMergeEditor)) {
        e.preventDefault();
        handleDiffClose();
        return;
      }
      if (!e.metaKey && !e.ctrlKey) return;
      if (e.key === '=' || e.key === '+') {
        e.preventDefault();
        zoomLevel = +(Math.min(3, zoomLevel + 0.1).toFixed(1));
        setZoomLevel(zoomLevel);
      } else if (e.key === '-') {
        e.preventDefault();
        zoomLevel = +(Math.max(0.5, zoomLevel - 0.1).toFixed(1));
        setZoomLevel(zoomLevel);
      } else if (e.key === '0') {
        e.preventDefault();
        zoomLevel = 1;
        setZoomLevel(zoomLevel);
      } else if (e.key === 'j' || e.key === 'J') {
        e.preventDefault();
        leftPaneCollapsed = !leftPaneCollapsed;
        setLeftPaneCollapsed(leftPaneCollapsed);
      } else if (e.key === 'k' || e.key === 'K') {
        e.preventDefault();
        rightPaneCollapsed = !rightPaneCollapsed;
        setRightPaneCollapsed(rightPaneCollapsed);
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  function startLeftResize(e: MouseEvent) {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = leftPaneCollapsed ? 0 : leftPaneWidth;

    function onMouseMove(ev: MouseEvent) {
      const newWidth = Math.max(0, startWidth + ev.clientX - startX);
      if (newWidth < 50) {
        leftPaneCollapsed = true;
      } else {
        leftPaneCollapsed = false;
        leftPaneWidth = Math.min(600, newWidth);
      }
    }

    function onMouseUp() {
      if (leftPaneCollapsed) {
        setLeftPaneCollapsed(true);
      } else {
        setLeftPaneWidth(leftPaneWidth);
        setLeftPaneCollapsed(false);
      }
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  function startRightResize(e: MouseEvent) {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = rightPaneCollapsed ? 0 : rightPaneWidth;

    function onMouseMove(ev: MouseEvent) {
      const newWidth = Math.max(0, startWidth - (ev.clientX - startX));
      if (newWidth < 50) {
        rightPaneCollapsed = true;
      } else {
        rightPaneCollapsed = false;
        rightPaneWidth = Math.min(700, newWidth);
      }
    }

    function onMouseUp() {
      if (rightPaneCollapsed) {
        setRightPaneCollapsed(true);
      } else {
        setRightPaneWidth(rightPaneWidth);
        setRightPaneCollapsed(false);
      }
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  async function handleOpenRebaseEditor(baseOid: string, inclusive = false) {
    if (!repoPath) return;
    try {
      const todoItems = await safeInvoke<RebaseTodoItem[]>('get_rebase_todo', { path: repoPath, baseOid, inclusive });
      if (todoItems.length === 0) return;
      rebaseEditorCommits = todoItems;
      rebaseBaseOid = baseOid;
      rebaseBranchName = headBranch ?? 'HEAD';
      // Resolve base name: use short ref if possible
      try {
        const refs = await safeInvoke<RefsResponse>('list_refs', { path: repoPath! });
        const allBranches = [...refs.local, ...refs.remote];
        const baseRef = allBranches.find(b => {
          // Try to match OID - need to resolve ref to OID
          return false; // fallback below
        });
        rebaseBaseName = baseOid.slice(0, 7);
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
      showToast(err.message ?? 'Failed to load commits for rebase', 'error');
    }
  }

  function handleRebaseEditorClose() {
    showRebaseEditor = false;
    rebaseEditorCommits = [];
    rebaseBaseOid = '';
    rebaseBranchName = '';
    rebaseBaseName = '';
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
        safeInvoke<CommitDetailType>('get_commit_detail', { path: repoPath, oid }),
        safeInvoke<FileDiff[]>('diff_commit', { path: repoPath, oid }),
      ]);
      rebaseFocusedCommitDetail = detail;
      rebaseFocusedFileDiffs = files;
    } catch {
      rebaseFocusedCommitDetail = null;
      rebaseFocusedFileDiffs = [];
    }
  }

  async function handleRebaseStart(todoItems: { oid: string; action: string; summary: string; newMessage: string | null }[]) {
    if (!repoPath) return;
    const baseOid = rebaseBaseOid;

    // Build squash groups: consecutive squash commits attach to the pick/reword above them
    type Group = { target: typeof todoItems[0]; squashes: typeof todoItems[0][] };
    const groups: Group[] = [];
    let currentGroup: Group | null = null;

    for (const item of todoItems) {
      if (item.action === 'squash' && currentGroup && currentGroup.target.action !== 'drop') {
        currentGroup.squashes.push(item);
      } else {
        currentGroup = { target: item, squashes: [] };
        groups.push(currentGroup);
      }
    }

    // Build message queue for groups needing editing
    const messageQueue: SquashGroup[] = [];
    for (const group of groups) {
      const hasSquashes = group.squashes.length > 0;
      const needsEdit = group.target.action === 'reword' || hasSquashes;
      if (!needsEdit || group.target.action === 'drop') continue;

      // Skip standalone rewords already edited inline
      if (!hasSquashes && group.target.action === 'reword' && group.target.newMessage != null) continue;

      const allItems = [group.target, ...group.squashes];
      const allOids = allItems.map((i) => i.oid);

      // Fetch full commit messages (summary + body)
      const details = await Promise.all(
        allOids.map((oid) => safeInvoke<CommitDetailType>('get_commit_detail', { path: repoPath!, oid }))
      );
      const fullMessages = details.map((d) => {
        let msg = d.summary;
        if (d.body) msg += '\n\n' + d.body;
        return msg;
      });

      messageQueue.push({
        targetOid: group.target.oid,
        allOids,
        shortOids: allItems.map((i) => {
          const c = rebaseEditorCommits.find((c) => c.oid === i.oid);
          return c?.short_oid ?? i.oid.slice(0, 7);
        }),
        combinedMessage: fullMessages.join('\n\n'),
        actionType: hasSquashes ? 'squash' : 'reword',
      });
    }

    pendingRebaseTodoItems = todoItems;
    pendingRebaseBaseOid = baseOid;

    if (messageQueue.length > 0) {
      rebaseMessageQueue = messageQueue;
      rebaseMessageIdx = 0;
      showRebaseMessageEditor = true;
      showRebaseEditor = false;
      return;
    }

    handleRebaseEditorClose();
    await executeRebase(baseOid, todoItems);
  }

  async function handleRebaseMessageConfirm(newMessage: string) {
    const group = rebaseMessageQueue[rebaseMessageIdx];

    // Write newMessage to all reword/squash items in this group so the backend
    // msg-queue has a file for every editor invocation git makes.
    // The last squash step's message is the final combined result.
    for (const oid of group.allOids) {
      const item = pendingRebaseTodoItems.find((i) => i.oid === oid);
      if (!item) continue;
      if (item.action === 'reword' || item.action === 'squash') {
        item.newMessage = newMessage;
      }
    }

    if (rebaseMessageIdx < rebaseMessageQueue.length - 1) {
      rebaseMessageIdx += 1;
    } else {
      showRebaseMessageEditor = false;
      handleRebaseEditorClose();
      await executeRebase(pendingRebaseBaseOid, pendingRebaseTodoItems);
    }
  }

  function handleRebaseMessageCancel() {
    showRebaseMessageEditor = false;
    rebaseMessageQueue = [];
    pendingRebaseTodoItems = [];
    handleRebaseEditorClose();
  }

  async function executeRebase(baseOid: string, todoItems: { oid: string; action: string; summary: string; newMessage: string | null }[]) {
    if (!repoPath) return;
    try {
      await safeInvoke('start_interactive_rebase', {
        path: repoPath,
        baseOid,
        todoItems,
      });
    } catch (e) {
      const err = e as { message?: string };
      showToast(err.message ?? 'Rebase failed', 'error');
    }
  }

  async function handleClose() {
    if (repoPath) {
      try {
        await safeInvoke('close_repo', { path: repoPath });
      } catch {
        // State is cleaned up regardless
      }
    }
    repoPath = null;
    repoName = '';
    refreshSignal = 0;
    clearStagingDiff();
    clearCommit();
    setOpenRepo(null);
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

<div class="flex flex-col h-screen" style="background: var(--color-bg);">
  {#if repoPath === null}
    <WelcomeScreen onopen={handleOpen} />
  {:else}
    <!-- LAYOUT-02: unified title bar + toolbar — drag from any non-button background area -->
    <div data-tauri-drag-region class="flex items-center flex-shrink-0" style="height: 32px; background: var(--color-surface); border-bottom: 1px solid var(--color-border); padding-left: {78 / zoomLevel}px;">
      <TabBar {repoName} onclose={handleClose} />
      <div data-tauri-drag-region class="flex-1 h-full"></div>
      <Toolbar repoPath={repoPath!} />
    </div>
    <main class="flex-1 overflow-hidden flex">
      {#if showRebaseEditor || showRebaseMessageEditor}
        <!-- Full-window takeover for interactive rebase -->
        <div class="flex-1 overflow-hidden">
          {#if showRebaseMessageEditor && rebaseMessageQueue[rebaseMessageIdx]}
            {@const group = rebaseMessageQueue[rebaseMessageIdx]}
            <RebaseMessageEditor
              repoPath={repoPath!}
              allOids={group.allOids}
              shortOids={group.shortOids}
              message={group.combinedMessage}
              actionType={group.actionType}
              remaining={rebaseMessageQueue.length - rebaseMessageIdx}
              onconfirm={handleRebaseMessageConfirm}
              oncancel={handleRebaseMessageCancel}
            />
          {:else}
            <div class="flex-1 overflow-hidden" style="position: relative;">
              <div style="position: absolute; inset: 0; {rebaseDiffFile ? 'visibility: hidden;' : ''}">
                <RebaseEditor
                  repoPath={repoPath!}
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
                  commitDetail={null}
                  selectedPath={rebaseDiffFile}
                  diffKind="commit"
                  repoPath={repoPath!}
                  onclose={() => { rebaseDiffFile = null; }}
                />
              {/if}
            </div>
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
              repoPath={repoPath!}
            />
          {:else}
            <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--color-text-muted); font-size: 13px;">
              Select a commit to view details
            </div>
          {/if}
        </div>
      {:else}
      <div style="width: {leftPaneCollapsed ? 0 : leftPaneWidth}px; flex-shrink: 0; overflow: hidden; display: flex; flex-direction: column;">
        <BranchSidebar repoPath={repoPath!} onrefreshed={handleRefresh} onstashselect={handleCommitSelect} onrefnavigate={handleRefNavigate} {refreshSignal} onopenrebaseeditor={handleOpenRebaseEditor} />
      </div>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="pane-divider" style="display: {leftPaneCollapsed ? 'none' : 'block'};" onmousedown={startLeftResize}></div>
      <div class="flex-1 overflow-hidden">
        {#if showMergeEditor && selectedFile}
          <MergeEditor
            repoPath={repoPath!}
            filePath={selectedFile.path}
            onclose={handleDiffClose}
            onresolved={handleFileResolved}
          />
        {:else if showDiff}
          <DiffPanel
            fileDiffs={currentDiffFiles}
            commitDetail={null}
            selectedPath={selectedCommitFile ?? selectedFile?.path ?? null}
            diffKind={selectedCommitFile ? 'commit' : (selectedFile?.kind ?? 'commit')}
            repoPath={repoPath!}
            onhunkaction={async (filePath) => {
              if (selectedFile) {
                await refetchFileDiff(filePath, selectedFile.kind);
              }
            }}
            onclose={handleDiffClose}
          />
        {:else}
          <CommitGraph bind:this={commitGraphRef} {repoPath} oncommitselect={handleCommitSelect} {wipCount} wipMessage={wipSubject.trim() || 'WIP'} onWipClick={handleWipClick} {refreshSignal} {selectedCommitOid} onopenrebaseeditor={handleOpenRebaseEditor} />
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
            repoPath={repoPath!}
          />
        {:else}
          <StagingPanel repoPath={repoPath!} currentBranch={headBranch} onfileselect={handleFileSelect} onsubjectchange={(v) => (wipSubject = v)} />
        {/if}
      </div>
      {/if}
    </main>
  {/if}
  <Toast />
</div>
