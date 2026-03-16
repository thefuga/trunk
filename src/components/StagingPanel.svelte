<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { safeInvoke, type TrunkError } from '../lib/invoke.js';
  import type { WorkingTreeStatus, FileStatusType } from '../lib/types.js';
  import { showToast } from '../lib/toast.svelte.js';
  import FileRow from './FileRow.svelte';
  import CommitForm from './CommitForm.svelte';
  import { ChevronDown, ChevronRight } from '@lucide/svelte';

  interface Props {
    repoPath: string;
    currentBranch?: string;
    onfileselect?: (path: string, kind: 'unstaged' | 'staged') => void;
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

  let totalCount = $derived(
    (status?.unstaged.length ?? 0) +
    (status?.staged.length ?? 0) +
    (status?.conflicted.length ?? 0)
  );

  async function loadStatus() {
    const seq = ++loadSeq;
    const result = await safeInvoke<WorkingTreeStatus>('get_status', { path: repoPath });
    if (seq === loadSeq) {
      status = result;
    }
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

  async function showFileContextMenu(e: MouseEvent, filePath: string, fileStatus: FileStatusType) {
    const { Menu, MenuItem } = await import('@tauri-apps/api/menu');
    const isUntracked = fileStatus === 'New';
    const menu = await Menu.new({
      items: [
        await MenuItem.new({
          text: isUntracked ? 'Delete File' : 'Discard Changes',
          action: () => { handleDiscardFile(filePath, fileStatus).catch(() => {}); },
        }),
      ],
    });
    await menu.popup();
  }

  async function handleDiscardAll() {
    const count = (status?.unstaged.length ?? 0) + (status?.conflicted.length ?? 0);
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

  <!-- File sections flex container (50/50 split when both expanded) -->
  <div style="flex: 1; display: flex; flex-direction: column; overflow: hidden; min-height: 0;">
    <!-- Unstaged Files section -->
    <div style="
      {unstaged_expanded && staged_expanded ? 'flex: 1;' : unstaged_expanded ? 'flex: 1;' : ''}
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
          user-select: none;
          flex-shrink: 0;
        "
      >
        <span style="color: var(--color-text-muted); display: inline-flex; align-items: center; margin-right: 4px;">
          {#if unstaged_expanded}<ChevronDown size={12} />{:else}<ChevronRight size={12} />{/if}
        </span>
        <span style="color: var(--color-text); font-size: 12px; font-weight: 500; flex: 1;">
          Unstaged Files ({(status?.unstaged.length ?? 0) + (status?.conflicted.length ?? 0)})
        </span>
        {#if (status?.unstaged.length ?? 0) > 0 || (status?.conflicted.length ?? 0) > 0}
          <button
            onclick={(e) => { e.stopPropagation(); handleDiscardAll(); }}
            style="
              background: #f87171;
              color: white;
              font-size: 11px;
              border: none;
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
              background: #22c55e;
              color: white;
              font-size: 11px;
              border: none;
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
              oncontextmenu={(e) => showFileContextMenu(e, f.path, f.status)}
            />
          {/each}
          {#each status?.conflicted ?? [] as f (f.path)}
            <FileRow
              file={f}
              actionLabel="+"
              isLoading={loadingFiles.has(f.path)}
              onaction={() => stageFile(f.path)}
              onclick={() => onfileselect?.(f.path, 'unstaged')}
              oncontextmenu={(e) => showFileContextMenu(e, f.path, f.status)}
            />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Staged Files section -->
    <div style="
      {staged_expanded && unstaged_expanded ? 'flex: 1;' : staged_expanded ? 'flex: 1;' : ''}
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
          user-select: none;
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
              background: #f87171;
              color: white;
              font-size: 11px;
              border: none;
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
            />
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <!-- Permanent divider above commit form -->
  <div style="flex-shrink: 0; border-top: 1px solid var(--color-border);"></div>

  <!-- CommitForm — always visible at bottom -->
  <CommitForm {repoPath} stagedCount={status?.staged.length ?? 0} {onsubjectchange} />
</div>
