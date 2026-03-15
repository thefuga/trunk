<script lang="ts">
  import { toasts } from '../lib/toast.svelte.js';
  import { fly } from 'svelte/transition';
</script>

<div class="fixed bottom-4 right-4 flex flex-col gap-2 z-50 pointer-events-none">
  {#each toasts.items as toast (toast.id)}
    <div
      role="status"
      transition:fly={{ y: 8, duration: 150 }}
      class="px-4 py-2 rounded-lg text-sm font-medium shadow-lg pointer-events-auto"
      style="background: {toast.kind === 'error' ? '#3d1c1c' : 'var(--color-surface)'};
             border: 1px solid {toast.kind === 'error' ? '#6b2a2a' : 'var(--color-border)'};
             color: {toast.kind === 'error' ? '#f87171' : 'var(--color-text)'};"
    >
      {toast.message}
    </div>
  {/each}
</div>
