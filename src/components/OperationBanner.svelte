<script lang="ts">
  import type { OperationInfo } from '../lib/types.js';
  import { safeInvoke, type TrunkError } from '../lib/invoke.js';
  import { showToast } from '../lib/toast.svelte.js';
  import { GitMerge, GitBranch } from '@lucide/svelte';

  interface Props {
    info: OperationInfo;
    repoPath: string;
    onaction?: () => void;
  }

  let { info, repoPath, onaction }: Props = $props();
  let loading = $state(false);

  let isMerge = $derived(info.op_type === 'Merge');
  let isRebase = $derived(info.op_type === 'Rebase');

  let sourceBranch = $derived(info.source_branch ?? '???');
  let targetBranch = $derived(info.target_branch ?? '???');
  let sourceColor = $derived(`var(--lane-${(info.source_color_index ?? 1) % 8})`);
  let targetColor = $derived(`var(--lane-${(info.target_color_index ?? 0) % 8})`);

  let label = $derived.by(() => {
    if (info.op_type === 'CherryPick') return 'Cherry-pick in progress';
    if (info.op_type === 'Revert') return 'Revert in progress';
    return '';
  });

  async function handleContinue() {
    loading = true;
    try {
      const cmd = isMerge ? 'merge_continue' : 'rebase_continue';
      await safeInvoke(cmd, { path: repoPath });
      showToast(isMerge ? 'Merge completed' : 'Rebase continued', 'success');
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Continue failed', 'error');
    } finally {
      loading = false;
      onaction?.();
    }
  }

  async function handleSkip() {
    loading = true;
    try {
      await safeInvoke('rebase_skip', { path: repoPath });
      showToast('Commit skipped', 'success');
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Skip failed', 'error');
    } finally {
      loading = false;
      onaction?.();
    }
  }

  async function handleAbort() {
    const { ask } = await import('@tauri-apps/plugin-dialog');
    const opName = isMerge ? 'merge' : 'rebase';
    const confirmed = await ask(
      `Abort ${opName}? This will discard all ${opName} progress and return to the previous state.`,
      { title: `Abort ${opName.charAt(0).toUpperCase() + opName.slice(1)}`, kind: 'warning' }
    );
    if (!confirmed) return;
    loading = true;
    try {
      const cmd = isMerge ? 'merge_abort' : 'rebase_abort';
      await safeInvoke(cmd, { path: repoPath });
      showToast(`${opName.charAt(0).toUpperCase() + opName.slice(1)} aborted`, 'success');
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Abort failed', 'error');
    } finally {
      loading = false;
      onaction?.();
    }
  }
</script>

<div style="
  flex-shrink: 0;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid var(--color-border);
  background: {isMerge ? 'var(--color-banner-merge-bg)' : 'var(--color-banner-rebase-bg)'};
  border-left: 3px solid {isMerge ? 'var(--color-banner-merge-border)' : 'var(--color-banner-rebase-border)'};
">
  <span style="color: {isMerge ? 'var(--color-banner-merge-border)' : 'var(--color-banner-rebase-border)'}; display: inline-flex; align-items: center; flex-shrink: 0;">
    {#if isMerge}<GitMerge size={14} />{:else}<GitBranch size={14} />{/if}
  </span>
  <div style="font-size: 12px; color: var(--color-text); flex: 1; overflow: hidden; display: flex; align-items: center; gap: 4px; white-space: nowrap;">
    {#if isMerge || isRebase}
      <span style="flex-shrink: 0;">{isMerge ? 'Merging' : 'Rebasing'}</span>
      <span style="
        background: {sourceColor};
        border-radius: 9999px;
        padding: 0 6px;
        font-size: 11px;
        line-height: 16px;
        color: white;
        font-weight: 700;
        overflow: hidden;
        text-overflow: ellipsis;
        min-width: 0;
      ">{sourceBranch}</span>
      <span style="flex-shrink: 0;">{isMerge ? 'into' : 'onto'}</span>
      <span style="
        background: {targetColor};
        border-radius: 9999px;
        padding: 0 6px;
        font-size: 11px;
        line-height: 16px;
        color: white;
        font-weight: 700;
        overflow: hidden;
        text-overflow: ellipsis;
        min-width: 0;
      ">{targetBranch}</span>
      {#if isRebase && info.progress}
        <span style="color: var(--color-text-muted);">({info.progress})</span>
      {/if}
    {:else}
      <span>{label}</span>
    {/if}
  </div>
  {#if isRebase}
    <div style="display: flex; gap: 4px; flex-shrink: 0;">
      <button
        onclick={handleContinue}
        disabled={loading}
        style="
          background: var(--color-btn-continue-bg);
          color: var(--color-btn-continue);
          font-size: 11px;
          border: 1px solid var(--color-btn-continue-border);
          border-radius: 4px;
          cursor: pointer;
          padding: 2px 8px;
          white-space: nowrap;
        "
      >Continue</button>
      <button
        onclick={handleSkip}
        disabled={loading}
        style="
          background: var(--color-btn-skip-bg);
          color: var(--color-btn-skip);
          font-size: 11px;
          border: 1px solid var(--color-btn-skip-border);
          border-radius: 4px;
          cursor: pointer;
          padding: 2px 8px;
          white-space: nowrap;
        "
      >Skip</button>
      <button
        onclick={handleAbort}
        disabled={loading}
        style="
          background: var(--color-btn-abort-bg);
          color: var(--color-btn-abort);
          font-size: 11px;
          border: 1px solid var(--color-btn-abort-border);
          border-radius: 4px;
          cursor: pointer;
          padding: 2px 8px;
          white-space: nowrap;
        "
      >Abort</button>
    </div>
  {/if}
</div>
