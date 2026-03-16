<script lang="ts">
  import { safeInvoke } from '../lib/invoke.js';
  import type { TrunkError } from '../lib/invoke.js';
  import { remoteState } from '../lib/remote-state.svelte.js';
  import { undoRedoState, pushToRedoStack, popFromRedoStack } from '../lib/undo-redo.svelte.js';
  import { showToast } from '../lib/toast.svelte.js';
  import { listen } from '@tauri-apps/api/event';
  import PullDropdown from './PullDropdown.svelte';
  import InputDialog from './InputDialog.svelte';
  import { Undo2, Redo2, ArrowDown, ArrowUp, GitBranch, Archive, PackageOpen } from '@lucide/svelte';

  interface Props {
    repoPath: string;
  }

  let { repoPath }: Props = $props();

  // Listen to remote-progress events from backend (relocated from StatusBar)
  $effect(() => {
    let unlisten: (() => void) | undefined;
    const path = repoPath;

    listen<{ path: string; line: string }>('remote-progress', (event) => {
      if (event.payload.path === path) {
        remoteState.progressLine = event.payload.line;
      }
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  });

  // Branch creation dialog state
  let branchDialogOpen = $state(false);

  // Undo/redo state
  let canUndo = $state(false);

  async function checkUndoAvailable() {
    try {
      canUndo = await safeInvoke<boolean>('check_undo_available', { path: repoPath });
    } catch {
      canUndo = false;
    }
  }

  // Check undo availability on mount and repo changes
  $effect(() => {
    // Re-run when repoPath changes
    void repoPath;
    checkUndoAvailable();

    const unlistenPromise = listen<string>('repo-changed', (event) => {
      if (event.payload === repoPath) {
        checkUndoAvailable();
      }
    });

    return () => {
      unlistenPromise.then((fn) => fn());
    };
  });

  async function handleUndo() {
    try {
      const result = await safeInvoke<{ subject: string; body: string | null }>('undo_commit', { path: repoPath });
      pushToRedoStack({ subject: result.subject, body: result.body });
    } catch (e) {
      console.error('undo failed:', e);
    }
  }

  async function handleRedo() {
    const entry = popFromRedoStack();
    if (!entry) return;
    try {
      await safeInvoke('redo_commit', {
        path: repoPath,
        subject: entry.subject,
        body: entry.body,
      });
    } catch (e) {
      console.error('redo failed:', e);
      // Push back on failure
      pushToRedoStack(entry);
    }
  }

  function errorMessage(error: TrunkError): string {
    switch (error.code) {
      case 'auth_failure':
        return 'Authentication failed \u2014 check your SSH key or credential helper';
      case 'non_fast_forward':
        return 'Push rejected (non-fast-forward)';
      case 'no_upstream':
      case 'remote_error':
      default:
        return error.message;
    }
  }

  async function runRemote(
    cmd: string,
    successMsg: string,
    extra: Record<string, unknown> = {}
  ) {
    remoteState.isRunning = true;
    remoteState.error = null;
    remoteState.progressLine = '';
    try {
      await safeInvoke(cmd, { path: repoPath, ...extra });
      remoteState.isRunning = false;
      remoteState.progressLine = '';
      showToast(successMsg, 'success');
    } catch (e: unknown) {
      remoteState.isRunning = false;
      const err = e as TrunkError;
      remoteState.error = err;
      showToast(errorMessage(err), 'error');
    }
  }

  function handlePull() {
    runRemote('git_pull', 'Pulled successfully');
  }

  function handlePush() {
    runRemote('git_push', 'Pushed successfully');
  }

  async function handleStash() {
    try {
      await safeInvoke('stash_save', { path: repoPath, message: '' });
      showToast('Stash created', 'success');
    } catch (e) {
      console.error('stash_save failed:', e);
      showToast('Failed to create stash', 'error');
    }
  }

  async function handlePop() {
    try {
      await safeInvoke('stash_pop', { path: repoPath, index: 0 });
      showToast('Stash applied', 'success');
    } catch (e) {
      console.error('stash_pop failed:', e);
      showToast('Failed to apply stash', 'error');
    }
  }

  function handleBranch() {
    branchDialogOpen = true;
  }

  async function handleBranchCreate(values: Record<string, string>) {
    branchDialogOpen = false;
    const name = values.name?.trim();
    if (!name) return;
    try {
      await safeInvoke('create_branch', { path: repoPath, name });
      showToast('Checked out ' + name, 'success');
    } catch (e) {
      const err = e as TrunkError;
      if (err.code === 'dirty_workdir') {
        showToast('Branch created (checkout skipped — uncommitted changes)', 'success');
      } else {
        showToast('Failed to create branch', 'error');
      }
    }
  }
</script>

<style>
  .toolbar {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 12px;
    user-select: none;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .toolbar-btn {
    background: none;
    border: none;
    border-radius: 4px;
    color: var(--color-text);
    font-size: 12px;
    padding: 4px 10px;
    cursor: pointer;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 4px;
    height: 26px;
  }
  .toolbar-btn:hover:not(:disabled) {
    background: var(--color-border);
  }
  .toolbar-btn:disabled {
    opacity: 0.5;
    cursor: default;
    pointer-events: none;
  }

  .btn-group {
    display: inline-flex;
    align-items: stretch;
  }
  .btn-group .toolbar-btn {
    border-radius: 4px 0 0 4px;
  }

</style>

<div data-tauri-drag-region class="toolbar">
  <div class="toolbar-group">
    <button class="toolbar-btn" disabled={!canUndo} onclick={handleUndo}>
      <Undo2 size={14} /> Undo
    </button>
    <button class="toolbar-btn" disabled={undoRedoState.redoStack.length === 0} onclick={handleRedo}>
      <Redo2 size={14} /> Redo
    </button>
  </div>

  <div class="toolbar-group">
    <div class="btn-group">
      <button class="toolbar-btn" disabled={remoteState.isRunning} onclick={handlePull}>
        <ArrowDown size={14} /> Pull
      </button>
      <PullDropdown {repoPath} disabled={remoteState.isRunning} />
    </div>
    <button class="toolbar-btn" disabled={remoteState.isRunning} onclick={handlePush}>
      <ArrowUp size={14} /> Push
    </button>
  </div>

  <div class="toolbar-group">
    <button class="toolbar-btn" onclick={handleBranch}>
      <GitBranch size={14} /> Branch
    </button>
    <button class="toolbar-btn" onclick={handleStash}>
      <Archive size={14} /> Stash
    </button>
    <button class="toolbar-btn" onclick={handlePop}>
      <PackageOpen size={14} /> Pop
    </button>
  </div>
</div>

{#if branchDialogOpen}
  <InputDialog
    title="Create Branch"
    fields={[{ key: 'name', label: 'Branch name', placeholder: 'feature/my-branch', required: true }]}
    onsubmit={handleBranchCreate}
    oncancel={() => (branchDialogOpen = false)}
  />
{/if}
