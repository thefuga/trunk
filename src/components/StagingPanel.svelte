<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { safeInvoke, type TrunkError } from '../lib/invoke.js';
  import type { WorkingTreeStatus, FileStatusType, OperationInfo, MergeSides } from '../lib/types.js';
  import { showToast } from '../lib/toast.svelte.js';
  import { writeText } from '@tauri-apps/plugin-clipboard-manager';
  import FileRow from './FileRow.svelte';
  import CommitForm from './CommitForm.svelte';
  import OperationBanner from './OperationBanner.svelte';
  import { ChevronDown, ChevronRight, AlertTriangle } from '@lucide/svelte';

  interface Props {
    repoPath: string;
    currentBranch?: string;
    onfileselect?: (path: string, kind: 'unstaged' | 'staged' | 'conflicted') => void;
    onsubjectchange?: (value: string) => void;
  }

  let {
    repoPath,
    currentBranch,
    onfileselect,
    onsubjectchange,
  }: Props = $props();

  let status = $state<WorkingTreeStatus | null>(null);
  let unstaged_expanded = $state(true);
  let staged_expanded = $state(true);
  let loadingFiles = $state<Set<string>>(new Set());
  let loadSeq = 0;
  let conflicted_expanded = $state(true);
  let operationInfo = $state<OperationInfo | null>(null);

  let totalCount = $derived(
    (status?.unstaged.length ?? 0) +
    (status?.staged.length ?? 0) +
    (status?.conflicted.length ?? 0)
  );

  async function loadOperationState() {
    const result = await safeInvoke<OperationInfo>('get_operation_state', { path: repoPath });
    operationInfo = result;
  }

  async function loadStatus() {
    const seq = ++loadSeq;
    const result = await safeInvoke<WorkingTreeStatus>('get_status', { path: repoPath });
    if (seq === loadSeq) {
      status = result;
    }
    await loadOperationState();
  }

  async function stageFile(filePath: string) {
    loadingFiles = new Set([...loadingFiles, filePath]);
    await safeInvoke('stage_file', { path: repoPath, filePath });
    await loadStatus();
    const next = new Set(loadingFiles);
    next.delete(filePath);
    loadingFiles = next;
  }

  async function unstageFile(filePath: string) {
    loadingFiles = new Set([...loadingFiles, filePath]);
    await safeInvoke('unstage_file', { path: repoPath, filePath });
    await loadStatus();
    const next = new Set(loadingFiles);
    next.delete(filePath);
    loadingFiles = next;
  }

  async function stageAll() {
    await safeInvoke('stage_all', { path: repoPath });
    await loadStatus();
  }

  async function unstageAll() {
    await safeInvoke('unstage_all', { path: repoPath });
    await loadStatus();
  }

  async function handleDiscardFile(filePath: string, fileStatus: FileStatusType) {
    const { ask } = await import('@tauri-apps/plugin-dialog');
    const isUntracked = fileStatus === 'New';
    const msg = isUntracked
      ? `Delete ${filePath}? This file is untracked and will be permanently removed. This cannot be undone.`
      : `Discard changes to ${filePath}? This cannot be undone.`;
    const confirmed = await ask(msg, { title: isUntracked ? 'Delete File' : 'Discard Changes', kind: 'warning' });
    if (!confirmed) return;
    try {
      await safeInvoke('discard_file', { path: repoPath, filePath });
      await loadStatus();
      showToast(`Discarded ${filePath}`, 'success');
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Discard failed', 'error');
    }
  }

  async function showUnstagedContextMenu(e: MouseEvent, filePath: string, fileStatus: FileStatusType) {
    const { Menu, MenuItem, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const isUntracked = fileStatus === 'New';
    const absPath = repoPath + '/' + filePath;
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Copy Relative Path', action: () => { writeText(filePath).catch(() => {}); } }),
        await MenuItem.new({ text: 'Copy Absolute Path', action: () => { writeText(absPath).catch(() => {}); } }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({ text: 'Stage File', action: () => { stageFile(filePath); } }),
        await MenuItem.new({
          text: isUntracked ? 'Delete File' : 'Discard Changes',
          action: () => { handleDiscardFile(filePath, fileStatus).catch(() => {}); },
        }),
      ],
    });
    await menu.popup();
  }

  async function showStagedContextMenu(e: MouseEvent, filePath: string) {
    const { Menu, MenuItem, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const absPath = repoPath + '/' + filePath;
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Copy Relative Path', action: () => { writeText(filePath).catch(() => {}); } }),
        await MenuItem.new({ text: 'Copy Absolute Path', action: () => { writeText(absPath).catch(() => {}); } }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({ text: 'Unstage File', action: () => { unstageFile(filePath); } }),
      ],
    });
    await menu.popup();
  }

  async function resolveConflictedFile(filePath: string, side: 'ours' | 'theirs') {
    try {
      const sides = await safeInvoke<MergeSides>('get_merge_sides', { path: repoPath, filePath });
      const content = side === 'ours' ? sides.ours : sides.theirs;
      await safeInvoke('save_merge_result', { path: repoPath, filePath, content });
      await loadStatus();
      const label = side === 'ours' ? 'current' : 'incoming';
      showToast(`Resolved ${filePath} (took all ${label})`, 'success');
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Resolution failed', 'error');
    }
  }

  async function showConflictedContextMenu(e: MouseEvent, filePath: string) {
    const { Menu, MenuItem, PredefinedMenuItem } = await import('@tauri-apps/api/menu');
    const absPath = repoPath + '/' + filePath;
    const menu = await Menu.new({
      items: [
        await MenuItem.new({ text: 'Take All Current', action: () => { resolveConflictedFile(filePath, 'ours').catch(() => {}); } }),
        await MenuItem.new({ text: 'Take All Incoming', action: () => { resolveConflictedFile(filePath, 'theirs').catch(() => {}); } }),
        await PredefinedMenuItem.new({ item: 'Separator' }),
        await MenuItem.new({ text: 'Copy Relative Path', action: () => { writeText(filePath).catch(() => {}); } }),
        await MenuItem.new({ text: 'Copy Absolute Path', action: () => { writeText(absPath).catch(() => {}); } }),
      ],
    });
    await menu.popup();
  }

  async function handleDiscardAll() {
    const count = status?.unstaged.length ?? 0;
    if (count === 0) return;
    const { ask } = await import('@tauri-apps/plugin-dialog');
    const confirmed = await ask(
      `Discard all changes to ${count} file${count === 1 ? '' : 's'}? This cannot be undone.`,
      { title: 'Discard All Changes', kind: 'warning' }
    );
    if (!confirmed) return;
    try {
      await safeInvoke('discard_all', { path: repoPath });
      await loadStatus();
      showToast(`Discarded all changes (${count} files)`, 'success');
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Discard all failed', 'error');
    }
  }

  // Initial load on mount
  $effect(() => {
    if (repoPath) loadStatus();
  });

  // Auto-refresh on repo-changed event
  $effect(() => {
    let unlisten: (() => void) | undefined;
    listen<string>('repo-changed', (event) => {
      if (event.payload === repoPath) loadStatus();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  });
</script>

<div style="
  width: 100%;
  min-width: 0;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
">
  <!-- Panel header -->
  <div style="
    height: 24px;
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    flex-shrink: 0;
  ">
    <span style="font-size: 12px; color: var(--color-text);">
      {totalCount} file{totalCount === 1 ? '' : 's'} changed
    </span>
    {#if currentBranch}
      <span style="font-size: 11px; color: var(--color-text-muted);">on</span>
      <span style="
        background: var(--lane-0);
        border-radius: 9999px;
        padding: 0 6px;
        font-size: 11px;
        line-height: 16px;
        color: white;
        font-weight: 700;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        max-width: 120px;
      ">
        {currentBranch}
      </span>
    {/if}
  </div>

  <!-- Operation banner (merge/rebase in progress) -->
  {#if operationInfo && operationInfo.op_type !== 'None'}
    <OperationBanner
      info={operationInfo}
      {repoPath}
      onaction={() => { loadStatus(); }}
    />
  {/if}

  <!-- File sections flex container (50/50 split when both expanded) -->
  <div style="flex: 1; display: flex; flex-direction: column; overflow: hidden; min-height: 0;">
    <!-- Conflicted Files section (top, above unstaged/staged) -->
    {#if (status?.conflicted.length ?? 0) > 0}
      <div style="
        display: flex;
        flex-direction: column;
        overflow: hidden;
        min-height: 0;
        flex-shrink: 0;
        max-height: 40%;
      ">
        <div
          role="button"
          tabindex="0"
          onclick={() => (conflicted_expanded = !conflicted_expanded)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') conflicted_expanded = !conflicted_expanded; }}
          style="
            height: 28px;
            border-bottom: 1px solid var(--color-border);
            padding: 0 8px;
            display: flex;
            align-items: center;
            cursor: pointer;
            flex-shrink: 0;
          "
        >
          <span style="color: var(--color-badge-warning); display: inline-flex; align-items: center; margin-right: 4px;">
            <AlertTriangle size={12} />
          </span>
          <span style="color: var(--color-text); font-size: 12px; font-weight: 500; flex: 1;">
            Conflicted Files
          </span>
          <span style="
            background: var(--color-badge-warning-bg);
            color: var(--color-badge-warning);
            font-size: 10px;
            font-weight: 700;
            border-radius: 9999px;
            padding: 0 6px;
            line-height: 16px;
          ">
            {status?.conflicted.length ?? 0}
          </span>
        </div>

        {#if conflicted_expanded}
          <div style="flex: 1; overflow-y: auto; min-height: 0;" role="list">
            {#each status?.conflicted ?? [] as f (f.path)}
              <FileRow
                file={f}
                actionLabel=""
                onaction={() => {}}
                onclick={() => onfileselect?.(f.path, 'conflicted')}
                oncontextmenu={(e) => showConflictedContextMenu(e, f.path)}
              />
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Unstaged Files section -->
    <div style="
      {unstaged_expanded && staged_expanded ? 'flex: 1;' : unstaged_expanded ? 'max-height: calc(100% - 28px);' : ''}
      display: flex;
      flex-direction: column;
      overflow: hidden;
      min-height: 0;
    ">
      <div
        role="button"
        tabindex="0"
        onclick={() => (unstaged_expanded = !unstaged_expanded)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') unstaged_expanded = !unstaged_expanded; }}
        style="
          height: 28px;
          border-bottom: 1px solid var(--color-border);
          padding: 0 8px;
          display: flex;
          align-items: center;
          cursor: pointer;
          flex-shrink: 0;
        "
      >
        <span style="color: var(--color-text-muted); display: inline-flex; align-items: center; margin-right: 4px;">
          {#if unstaged_expanded}<ChevronDown size={12} />{:else}<ChevronRight size={12} />{/if}
        </span>
        <span style="color: var(--color-text); font-size: 12px; font-weight: 500; flex: 1;">
          Unstaged Files ({status?.unstaged.length ?? 0})
        </span>
        {#if (status?.unstaged.length ?? 0) > 0}
          <button
            onclick={(e) => { e.stopPropagation(); handleDiscardAll(); }}
            style="
              background: var(--color-btn-discard-bg);
              color: var(--color-btn-discard);
              font-size: 11px;
              border: 1px solid var(--color-btn-discard-border);
              border-radius: 4px;
              cursor: pointer;
              padding: 2px 8px;
              white-space: nowrap;
            "
            aria-label="Discard all changes"
          >
            Discard All
          </button>
          <button
            onclick={(e) => { e.stopPropagation(); stageAll(); }}
            style="
              background: var(--color-btn-stage-bg);
              color: var(--color-btn-stage);
              font-size: 11px;
              border: 1px solid var(--color-btn-stage-border);
              border-radius: 4px;
              cursor: pointer;
              padding: 2px 8px;
              white-space: nowrap;
              margin-left: 4px;
            "
            aria-label="Stage all changes"
          >
            Stage All Changes
          </button>
        {/if}
      </div>

      {#if unstaged_expanded}
        <div style="flex: 1; overflow-y: auto; min-height: 0;" role="list">
          {#each status?.unstaged ?? [] as f (f.path)}
            <FileRow
              file={f}
              actionLabel="+"
              isLoading={loadingFiles.has(f.path)}
              onaction={() => stageFile(f.path)}
              onclick={() => onfileselect?.(f.path, 'unstaged')}
              oncontextmenu={(e) => showUnstagedContextMenu(e, f.path, f.status)}
            />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Staged Files section -->
    <div style="
      {staged_expanded && unstaged_expanded ? 'flex: 1;' : staged_expanded ? 'max-height: calc(100% - 28px);' : ''}
      display: flex;
      flex-direction: column;
      overflow: hidden;
      min-height: 0;
    ">
      <div
        role="button"
        tabindex="0"
        onclick={() => (staged_expanded = !staged_expanded)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') staged_expanded = !staged_expanded; }}
        style="
          height: 28px;
          border-bottom: 1px solid var(--color-border);
          padding: 0 8px;
          display: flex;
          align-items: center;
          cursor: pointer;
          flex-shrink: 0;
        "
      >
        <span style="color: var(--color-text-muted); display: inline-flex; align-items: center; margin-right: 4px;">
          {#if staged_expanded}<ChevronDown size={12} />{:else}<ChevronRight size={12} />{/if}
        </span>
        <span style="color: var(--color-text); font-size: 12px; font-weight: 500; flex: 1;">
          Staged Files ({status?.staged.length ?? 0})
        </span>
        {#if (status?.staged.length ?? 0) > 0}
          <button
            onclick={(e) => { e.stopPropagation(); unstageAll(); }}
            style="
              background: var(--color-btn-unstage-bg);
              color: var(--color-btn-unstage);
              font-size: 11px;
              border: 1px solid var(--color-btn-unstage-border);
              border-radius: 4px;
              cursor: pointer;
              padding: 2px 8px;
              white-space: nowrap;
            "
            aria-label="Unstage all"
          >
            Unstage All
          </button>
        {/if}
      </div>

      {#if staged_expanded}
        <div style="flex: 1; overflow-y: auto; min-height: 0;" role="list">
          {#each status?.staged ?? [] as f (f.path)}
            <FileRow
              file={f}
              actionLabel="−"
              isLoading={loadingFiles.has(f.path)}
              onaction={() => unstageFile(f.path)}
              onclick={() => onfileselect?.(f.path, 'staged')}
              oncontextmenu={(e) => showStagedContextMenu(e, f.path)}
            />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Spacer: absorbs remaining space when a section is collapsed -->
    {#if !(unstaged_expanded && staged_expanded)}
      <div style="flex: 1;"></div>
    {/if}
  </div>

  <!-- Permanent divider above commit form -->
  <div style="flex-shrink: 0; border-top: 1px solid var(--color-border);"></div>

  <!-- CommitForm — always visible at bottom -->
  <CommitForm {repoPath} stagedCount={status?.staged.length ?? 0} {onsubjectchange} />
</div>
