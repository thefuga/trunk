<script lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { displayPath } from "../lib/path.js";
import {
	addRecentRepo,
	getRecentRepos,
	type RecentRepo,
	removeRecentRepo,
} from "../lib/store.js";

interface Props {
	onopen: (path: string, name: string) => void;
	isFullscreen?: boolean;
}

let { onopen, isFullscreen = false }: Props = $props();

let recentRepos = $state<RecentRepo[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

$effect(() => {
	getRecentRepos().then((repos) => {
		recentRepos = repos;
	});
});

async function openRepository() {
	error = null;
	const selected = await open({ directory: true, multiple: false });
	if (typeof selected !== "string") return;

	await openPath(selected);
}

async function openPath(path: string) {
	error = null;
	loading = true;
	try {
		await safeInvoke("open_repo", { path });
		const name = path.split("/").at(-1) || path;
		await addRecentRepo({ name, path });
		recentRepos = await getRecentRepos();
		onopen(path, name);
	} catch (e: unknown) {
		const trunk = e as TrunkError;
		error = trunk.message ?? "Failed to open repository";
	} finally {
		loading = false;
	}
}

async function handleRemoveRecent(path: string, event: MouseEvent) {
	event.stopPropagation();
	await removeRecentRepo(path);
	recentRepos = await getRecentRepos();
}
</script>

<div class="flex flex-col h-screen" style="background: var(--color-bg);">
  <!-- LAYOUT-02: drag region for window movement on welcome screen -->
  <div data-tauri-drag-region class="flex-shrink-0" style="height: 36px; padding-left: {isFullscreen ? 0 : 78}px;"></div>
  <div class="flex-1 flex flex-col items-center justify-center gap-6">
  <div class="flex flex-col items-center gap-4 w-full max-w-md px-4">
    <h1 class="text-2xl font-semibold tracking-tight" style="color: var(--color-text);">Trunk</h1>
    <p class="text-sm" style="color: var(--color-text-muted);">Git history, beautifully visualized</p>

    {#if error}
      <div
        class="w-full rounded-md px-4 py-2 text-sm"
        style="background: #3d1c1c; border: 1px solid #6b2a2a; color: #f87171;"
      >
        {error}
      </div>
    {/if}

    <button
      onclick={openRepository}
      disabled={loading}
      class="w-full rounded-md px-4 py-2.5 text-sm font-medium transition-opacity disabled:opacity-50"
      style="background: var(--color-accent); color: #fff;"
    >
      {loading ? 'Opening...' : 'Open Repository'}
    </button>
  </div>

  {#if recentRepos.length > 0}
    <div class="w-full max-w-md px-4">
      <p class="text-xs font-medium mb-2 uppercase tracking-widest" style="color: var(--color-text-muted);">Recent</p>
      <ul class="flex flex-col gap-1">
        {#each recentRepos as repo (repo.path)}
          <li>
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div
              class="group flex items-center gap-2 rounded px-3 py-1.5 cursor-pointer hover:bg-white/5"
              onclick={() => openPath(repo.path)}
              role="button"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && openPath(repo.path)}
            >
              <span class="text-sm truncate min-w-0 flex-1">
                <span style="color: var(--color-text-muted);">{displayPath(repo.path).substring(0, displayPath(repo.path).lastIndexOf('/'))}/</span><span class="font-semibold" style="color: var(--color-text);">{displayPath(repo.path).split('/').at(-1)}</span>
              </span>
              <button
                class="ml-2 flex-shrink-0 w-5 h-5 flex items-center justify-center rounded opacity-0 group-hover:opacity-100 transition-opacity text-xs"
                style="color: var(--color-text-muted);"
                onclick={(e) => handleRemoveRecent(repo.path, e)}
                aria-label="Remove from recent"
                title="Remove from recent"
              >
                ×
              </button>
            </div>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
  </div>
</div>
