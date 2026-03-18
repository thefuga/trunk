<script lang="ts">
  import type { FileDiff, CommitDetail } from '../lib/types.js';
  import { safeInvoke, type TrunkError } from '../lib/invoke.js';
  import { showToast } from '../lib/toast.svelte.js';

  interface Props {
    fileDiffs: FileDiff[];
    commitDetail: CommitDetail | null;
    selectedPath?: string | null;
    onclose: () => void;
    diffKind?: 'unstaged' | 'staged' | 'commit';
    repoPath?: string;
    onhunkaction?: (filePath: string) => Promise<void>;
  }

  let { fileDiffs, commitDetail, selectedPath = null, onclose, diffKind = 'commit', repoPath = '', onhunkaction }: Props = $props();

  let hunkOperationInFlight = $state(false);
  let focusedHunkIndex = $state(0);
  let hunkElements: Record<string, HTMLDivElement> = {};

  function scrollToHunk(index: number) {
    const keys = Object.keys(hunkElements);
    if (index < 0 || index >= keys.length) return;
    focusedHunkIndex = index;
    const el = hunkElements[keys[index]];
    el?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    el?.classList.add('hunk-highlight');
    setTimeout(() => el?.classList.remove('hunk-highlight'), 600);
  }

  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

      if (e.key === ']') {
        e.preventDefault();
        scrollToHunk(focusedHunkIndex + 1);
      } else if (e.key === '[') {
        e.preventDefault();
        scrollToHunk(focusedHunkIndex - 1);
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  $effect(() => {
    fileDiffs;
    focusedHunkIndex = 0;
    hunkElements = {};
  });

  async function handleStageHunk(filePath: string, hunkIndex: number) {
    hunkOperationInFlight = true;
    try {
      await safeInvoke('stage_hunk', { path: repoPath, filePath, hunkIndex });
      showToast('Staged hunk', 'success');
      await onhunkaction?.(filePath);
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Stage hunk failed', 'error');
    } finally {
      hunkOperationInFlight = false;
    }
  }

  async function handleUnstageHunk(filePath: string, hunkIndex: number) {
    hunkOperationInFlight = true;
    try {
      await safeInvoke('unstage_hunk', { path: repoPath, filePath, hunkIndex });
      showToast('Unstaged hunk', 'success');
      await onhunkaction?.(filePath);
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Unstage hunk failed', 'error');
    } finally {
      hunkOperationInFlight = false;
    }
  }

  async function handleDiscardHunk(filePath: string, hunkIndex: number) {
    const { ask } = await import('@tauri-apps/plugin-dialog');
    const confirmed = await ask('Discard this hunk? This cannot be undone.', {
      title: 'Discard Hunk',
      kind: 'warning',
    });
    if (!confirmed) return;

    hunkOperationInFlight = true;
    try {
      await safeInvoke('discard_hunk', { path: repoPath, filePath, hunkIndex });
      showToast('Discarded hunk', 'success');
      await onhunkaction?.(filePath);
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Discard hunk failed', 'error');
    } finally {
      hunkOperationInFlight = false;
    }
  }

  function lineBackground(origin: string): string {
    if (origin === 'Add') return 'rgba(74, 222, 128, 0.1)';
    if (origin === 'Delete') return 'rgba(248, 113, 113, 0.1)';
    return 'transparent';
  }

  function lineColor(origin: string): string {
    if (origin === 'Add') return '#4ade80';
    if (origin === 'Delete') return '#f87171';
    return 'var(--color-text)';
  }

  function originSymbol(origin: string): string {
    if (origin === 'Add') return '+';
    if (origin === 'Delete') return '-';
    return ' ';
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
  {#if fileDiffs.length === 0 && commitDetail === null}
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
        ">
          {fd.path}
        </div>
      {/if}

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
              <button
                disabled={hunkOperationInFlight}
                style="
                  background: none;
                  border: 1px solid var(--color-border);
                  border-radius: 3px;
                  color: var(--color-text);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 4px 8px;
                  cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                  opacity: {hunkOperationInFlight ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => handleStageHunk(fd.path, hunkIdx)}
              >
                Stage Hunk
              </button>
              <button
                disabled={hunkOperationInFlight}
                style="
                  background: none;
                  border: 1px solid var(--color-border);
                  border-radius: 3px;
                  color: #f87171;
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 4px 8px;
                  cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                  opacity: {hunkOperationInFlight ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => handleDiscardHunk(fd.path, hunkIdx)}
              >
                Discard Hunk
              </button>
            {:else if diffKind === 'staged'}
              <button
                disabled={hunkOperationInFlight}
                style="
                  background: none;
                  border: 1px solid var(--color-border);
                  border-radius: 3px;
                  color: var(--color-text);
                  font-size: 11px;
                  font-family: var(--font-sans, sans-serif);
                  padding: 4px 8px;
                  cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
                  opacity: {hunkOperationInFlight ? 0.4 : 1};
                  white-space: nowrap;
                "
                onclick={() => handleUnstageHunk(fd.path, hunkIdx)}
              >
                Unstage Hunk
              </button>
            {/if}
          </div>

          <!-- Diff lines -->
          {#each hunk.lines as line}
            <div style="
              font-family: monospace;
              font-size: 12px;
              line-height: 1.5;
              padding: 0 8px;
              white-space: pre;
              overflow-x: auto;
              background: {lineBackground(line.origin)};
              color: {lineColor(line.origin)};
            ">{originSymbol(line.origin)}{line.content}</div>
          {/each}
        {/each}
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
    0% { background-color: rgba(96, 165, 250, 0.3); }
    100% { background-color: transparent; }
  }
</style>
