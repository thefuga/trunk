<script lang="ts">
  import DiffPanel from './DiffPanel.svelte';
  import { safeInvoke } from '../lib/invoke.js';
  import type { FileDiff } from '../lib/types.js';

  interface Props {
    repoPath: string;
    commitOid: string;
    commitShortOid: string;
    message: string;
    actionType: 'reword' | 'squash';
    remaining: number;
    onconfirm: (newMessage: string) => void;
    oncancel: () => void;
  }

  let { repoPath, commitOid, commitShortOid, message, actionType, remaining, onconfirm, oncancel }: Props = $props();

  let editedMessage = $state(message);
  let fileDiffs = $state<FileDiff[]>([]);

  $effect(() => {
    safeInvoke<FileDiff[]>('diff_commit', { path: repoPath, oid: commitOid })
      .then((diffs) => { fileDiffs = diffs; })
      .catch(() => { fileDiffs = []; });
  });

  function handleConfirm() {
    onconfirm(editedMessage.trim());
  }
</script>

<div class="rme-container">
  <!-- Header bar -->
  <div class="rme-header">
    <div class="rme-header-left">
      <span class="rme-header-action" style="color: var(--color-rebase-{actionType});">
        {actionType === 'squash' ? 'Squash' : 'Reword'}
      </span>
      <span class="rme-header-oid">{commitShortOid}</span>
      {#if remaining > 1}
        <span class="rme-header-remaining">{remaining} remaining</span>
      {/if}
    </div>
    <div class="rme-header-right">
      <button class="rme-btn rme-btn-ghost" onclick={oncancel}>Cancel</button>
      <button class="rme-btn rme-btn-primary" onclick={handleConfirm}>
        {remaining > 1 ? 'Confirm & Next' : 'Confirm & Rebase'}
      </button>
    </div>
  </div>

  <!-- Commit message textarea -->
  <div class="rme-message-section">
    <textarea
      class="rme-textarea"
      bind:value={editedMessage}
      rows="4"
      placeholder="Commit message..."
    ></textarea>
  </div>

  <!-- File diffs -->
  <div class="rme-diffs">
    <DiffPanel
      {fileDiffs}
      commitDetail={null}
      selectedPath={null}
      diffKind="commit"
      repoPath={repoPath}
      onclose={() => {}}
    />
  </div>
</div>

<style>
  .rme-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg);
  }

  .rme-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    flex-shrink: 0;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    padding: 0 12px;
  }

  .rme-header-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rme-header-action {
    font-size: 13px;
    font-weight: 600;
  }

  .rme-header-oid {
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
  }

  .rme-header-remaining {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .rme-header-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rme-btn {
    border-radius: 4px;
    padding: 4px 12px;
    font-size: 11px;
    cursor: pointer;
    white-space: nowrap;
    font-family: var(--font-sans);
    font-weight: 600;
    border: none;
  }

  .rme-btn-ghost {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    color: var(--color-text);
  }

  .rme-btn-primary {
    background: var(--color-accent);
    color: white;
  }

  .rme-message-section {
    flex-shrink: 0;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
  }

  .rme-textarea {
    width: 100%;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text);
    font-size: 13px;
    font-family: var(--font-mono);
    padding: 8px;
    resize: vertical;
    outline: none;
    box-sizing: border-box;
  }

  .rme-textarea:focus {
    border-color: var(--color-accent);
  }

  .rme-diffs {
    flex: 1;
    overflow: hidden;
  }
</style>
