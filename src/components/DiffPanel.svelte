<script lang="ts">
  import type { FileDiff, CommitDetail } from '../lib/types.js';

  interface Props {
    fileDiffs: FileDiff[];
    commitDetail: CommitDetail | null;
    selectedPath?: string | null;
    onclose: () => void;
  }

  let { fileDiffs, commitDetail, selectedPath = null, onclose }: Props = $props();

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
        {#each fd.hunks as hunk}
          <!-- Hunk header -->
          <div style="
            background: var(--color-bg);
            color: var(--color-text-muted);
            font-size: 11px;
            font-family: monospace;
            padding: 2px 8px;
          ">
            {hunk.header}
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
