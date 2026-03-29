<script lang="ts">
import type { ViewMode } from "../../lib/types.js";

interface Props {
  viewMode: ViewMode;
  onviewmodechange: (mode: ViewMode) => void;
  selectedPath: string | null;
  diffKind: "unstaged" | "staged" | "commit";
  hunkOperationInFlight: boolean;
  onstagefile: () => void;
  onunstagefile: () => void;
  onclose: () => void;
}

let {
  viewMode,
  onviewmodechange,
  selectedPath,
  diffKind,
  hunkOperationInFlight,
  onstagefile,
  onunstagefile,
  onclose,
}: Props = $props();

const modes: { label: string; value: ViewMode }[] = [
  { label: "Hunk", value: "hunk" },
  { label: "Full", value: "full" },
  { label: "Split", value: "split" },
];
</script>

<div class="toolbar">
  <div class="segmented-control">
    {#each modes as mode}
      <button
        class="segment"
        class:active={viewMode === mode.value}
        onclick={() => onviewmodechange(mode.value)}
      >
        {mode.label}
      </button>
    {/each}
  </div>

  <span class="filename">
    {#if selectedPath}{selectedPath}{/if}
  </span>

  {#if diffKind === 'unstaged'}
    <button
      class="action-btn stage-btn"
      disabled={hunkOperationInFlight}
      style="
        cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
        opacity: {hunkOperationInFlight ? 0.4 : 1};
      "
      onclick={onstagefile}
    >
      Stage File
    </button>
  {:else if diffKind === 'staged'}
    <button
      class="action-btn unstage-btn"
      disabled={hunkOperationInFlight}
      style="
        cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
        opacity: {hunkOperationInFlight ? 0.4 : 1};
      "
      onclick={onunstagefile}
    >
      Unstage File
    </button>
  {/if}

  <button
    onclick={onclose}
    aria-label="Close diff"
    class="close-btn"
  >&#x2715;</button>
</div>

<style>
  .toolbar {
    height: 32px;
    border-bottom: 1px solid var(--color-border);
    padding: 0 8px;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    gap: 8px;
  }

  .segmented-control {
    display: inline-flex;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    overflow: hidden;
  }

  .segment {
    background: none;
    border: none;
    border-right: 1px solid var(--color-border);
    color: var(--color-text-muted);
    font-size: 11px;
    padding: 2px 8px;
    cursor: pointer;
  }

  .segment:last-child {
    border-right: none;
  }

  .segment.active {
    background: var(--color-accent-bg);
    color: var(--color-accent);
  }

  .filename {
    flex: 1;
    font-size: 11px;
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: center;
  }

  .action-btn {
    border-radius: 3px;
    font-size: 11px;
    font-family: var(--font-sans, sans-serif);
    padding: 2px 8px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .stage-btn {
    background: var(--color-success-bg);
    border: 1px solid var(--color-success-border);
    color: var(--color-success);
  }

  .unstage-btn {
    background: var(--color-warning-bg);
    border: 1px solid var(--color-warning-border);
    color: var(--color-warning);
  }

  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--color-text-muted);
    font-size: 16px;
    line-height: 1;
    padding: 2px 4px;
    border-radius: 3px;
    flex-shrink: 0;
  }
</style>
