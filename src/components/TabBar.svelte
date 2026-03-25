<script lang="ts">
  import type { TabInfo } from '../lib/tab-types.js';
  import { X, Plus } from '@lucide/svelte';
  import Sortable from 'sortablejs';

  interface Props {
    tabs: TabInfo[];
    activeTabId: string;
    onactivate: (tabId: string) => void;
    onclose: (tabId: string, force: boolean) => void;
    onnew: () => void;
    oncontextmenu: (tabId: string, event: MouseEvent) => void;
    onauxclose: (tabId: string) => void;
    onreorder: (newTabs: TabInfo[]) => void;
  }

  let { tabs, activeTabId, onactivate, onclose, onnew, oncontextmenu, onauxclose, onreorder }: Props = $props();

  let tabBarEl: HTMLDivElement;

  $effect(() => {
    const activeButton = tabBarEl?.querySelector(`[data-tab-id="${activeTabId}"]`);
    activeButton?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  });

  $effect(() => {
    if (!tabBarEl) return;
    const sortable = Sortable.create(tabBarEl, {
      animation: 150,
      direction: 'horizontal',
      forceFallback: true,
      ghostClass: 'tab-ghost',
      chosenClass: 'tab-chosen',
      dragClass: 'tab-drag',
      filter: '.new-tab-btn',
      preventOnFilter: false,
      scroll: true,
      scrollSensitivity: 50,
      onEnd: (e) => {
        if (e.oldIndex == null || e.newIndex == null || e.oldIndex === e.newIndex) return;
        const updated = [...tabs];
        const [moved] = updated.splice(e.oldIndex, 1);
        updated.splice(e.newIndex, 0, moved);
        onreorder(updated);
      },
    });
    return () => sortable.destroy();
  });
</script>

<div class="tab-bar" bind:this={tabBarEl}>
  {#each tabs as tab (tab.id)}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="tab-item"
      class:active={tab.id === activeTabId}
      data-tab-id={tab.id}
      onclick={() => onactivate(tab.id)}
      onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter' || e.key === ' ') onactivate(tab.id); }}
      oncontextmenu={(e: MouseEvent) => { e.preventDefault(); oncontextmenu(tab.id, e); }}
      onauxclick={(e: MouseEvent) => { if (e.button === 1) { e.preventDefault(); onauxclose(tab.id); } }}
      role="tab"
      tabindex="0"
      aria-selected={tab.id === activeTabId}
    >
      {#if tab.dirty}<span class="dirty-dot"></span>{/if}
      <span class="truncate" style="max-width: 200px;">{tab.repoName || 'New Tab'}</span>
      <button
        class="close-btn"
        onclick={(e: MouseEvent) => { e.stopPropagation(); onclose(tab.id, e.shiftKey); }}
        aria-label="Close tab"
      >
        <X size={12} />
      </button>
    </div>
  {/each}
  <button class="new-tab-btn" onclick={onnew} aria-label="New tab">
    <Plus size={14} />
  </button>
</div>

<style>
  .tab-bar {
    display: flex;
    align-items: center;
    height: 100%;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
  }

  .tab-bar::-webkit-scrollbar {
    display: none;
  }

  .tab-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 12px;
    height: 100%;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-muted);
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    background: none;
    border: none;
    border-right: 1px solid var(--color-border);
  }

  .tab-item:hover {
    background: var(--color-tab-hover);
  }

  .tab-item.active {
    color: var(--color-text);
    background: var(--color-bg);
  }

  .tab-item.active:hover {
    background: var(--color-bg);
  }

  .dirty-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-accent);
    flex-shrink: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    border-radius: 2px;
    border: none;
    background: none;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    transition: background-color 0.15s;
  }

  .close-btn:hover {
    background: var(--color-border);
  }

  .new-tab-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    border: none;
    background: none;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    margin-left: 4px;
    transition: background-color 0.15s;
  }

  .new-tab-btn:hover {
    background: var(--color-border);
  }

  :global(.tab-ghost) {
    opacity: 0.4;
  }

  :global(.tab-chosen) {
    background: var(--color-selected-row) !important;
  }

  :global(.tab-drag) {
    opacity: 0;
  }
</style>
