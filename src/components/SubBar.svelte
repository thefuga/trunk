<script lang="ts">
interface Props {
	branch?: string;
	ahead: number;
	behind: number;
	filesChanged: number;
	hasUpstream: boolean;
}

let { branch, ahead, behind, filesChanged, hasUpstream }: Props = $props();

let synced = $derived(hasUpstream && ahead === 0 && behind === 0);
</script>

<div class="subbar">
  <span class="chip">
    <span class="dot"></span>
    on <strong>{branch ?? 'HEAD'}</strong>
  </span>
  {#if ahead > 0 || behind > 0}
    <span class="sep">·</span>
    {#if ahead > 0}<span class="ahead">{ahead} ahead</span>{/if}
    {#if ahead > 0 && behind > 0}<span class="sep">·</span>{/if}
    {#if behind > 0}<span class="behind">{behind} behind</span>{/if}
  {/if}
  <span class="sep">·</span>
  <span class="files">{filesChanged} file{filesChanged === 1 ? '' : 's'} changed</span>
  {#if synced}
    <span class="synced">All changes synced to origin</span>
  {/if}
</div>

<style>
  .subbar {
    height: var(--subbar-h);
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 12px;
    background: var(--bg-1);
    border-bottom: 1px solid var(--line);
    color: var(--fg-2);
    font-family: var(--font-sans);
    font-weight: 500;
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    user-select: none;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 22px;
    padding: 0 10px;
    border-radius: 999px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    color: var(--fg-1);
    flex-shrink: 0;
  }
  .chip strong {
    color: var(--fg-0);
    font-weight: 600;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    flex-shrink: 0;
  }
  .sep {
    color: var(--fg-4);
  }
  .ahead {
    color: var(--ok);
  }
  .behind {
    color: var(--warn);
  }
  .files {
    color: var(--fg-2);
  }
  .synced {
    margin-left: auto;
    color: var(--fg-3);
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
