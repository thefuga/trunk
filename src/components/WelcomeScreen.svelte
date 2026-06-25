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
import {
	localRepoDescriptor,
	type WslAvailability,
	type WslDistro,
	type WslRepoValidation,
} from "../lib/types.js";

interface Props {
	onopen: (repo: RecentRepo) => void;
	isFullscreen?: boolean;
}

let { onopen, isFullscreen = false }: Props = $props();

let recentRepos = $state<RecentRepo[]>([]);
let resolvedPaths: Record<string, string> = $state({});
let loading = $state(false);
let error = $state<string | null>(null);
let wslAvailability = $state<WslAvailability | null>(null);
let wslDistros = $state<WslDistro[]>([]);
let selectedWslDistro = $state("");
let wslLinuxPath = $state("");
let wslLoading = $state(false);

// Storage is uncapped (the picker shows full history); the dashboard intentionally
// shows only the most recent few to keep the welcome screen compact.
const DASHBOARD_RECENT_LIMIT = 10;
const displayedRepos = $derived(recentRepos.slice(0, DASHBOARD_RECENT_LIMIT));

function repoKey(repo: RecentRepo): string {
	return repo.repoId ?? repo.repoDescriptor?.id ?? repo.path;
}

function repoDisplayPath(repo: RecentRepo): string {
	return repo.repoDescriptor?.display_path ?? repo.path;
}

$effect(() => {
	getRecentRepos().then((repos) => {
		recentRepos = repos;
	});
});

$effect(() => {
	(async () => {
		const availability = await safeInvoke<WslAvailability>(
			"wsl_availability",
		).catch(() => null);
		wslAvailability = availability;
		if (!availability?.available) return;

		const distros = await safeInvoke<WslDistro[]>("list_wsl_distros").catch(
			() => [],
		);
		wslDistros = distros;
		selectedWslDistro =
			distros.find((d) => d.default)?.name ?? distros[0]?.name ?? "";
	})();
});

$effect(() => {
	for (const repo of recentRepos) {
		const key = repoKey(repo);
		const path = repoDisplayPath(repo);
		if (!(key in resolvedPaths)) {
			displayPath(path).then((p) => {
				resolvedPaths[key] = p;
			});
		}
	}
});

async function openRepository() {
	error = null;
	const selected = await open({ directory: true, multiple: false });
	if (typeof selected !== "string") return;

	const name = selected.split("/").at(-1) || selected;
	const descriptor = localRepoDescriptor(selected, name);
	await openRepo({
		name,
		path: selected,
		repoId: descriptor.id,
		repoDescriptor: descriptor,
	});
}

async function openRepo(recent: RecentRepo): Promise<boolean> {
	error = null;
	loading = true;
	try {
		const descriptor =
			recent.repoDescriptor ?? localRepoDescriptor(recent.path, recent.name);
		await safeInvoke("open_repo", { path: descriptor.id, repo: descriptor });
		await addRecentRepo({
			name: recent.name,
			path: recent.path,
			repoId: descriptor.id,
			repoDescriptor: descriptor,
		});
		recentRepos = await getRecentRepos();
		onopen({
			name: recent.name,
			path: recent.path,
			repoId: descriptor.id,
			repoDescriptor: descriptor,
		});
		return true;
	} catch (e: unknown) {
		const trunk = e as TrunkError;
		error = trunk.message ?? "Failed to open repository";
		return false;
	} finally {
		loading = false;
	}
}

async function openWslRepository() {
	error = null;
	wslLoading = true;
	try {
		const validation = await safeInvoke<WslRepoValidation>(
			"validate_wsl_repo",
			{
				distro: selectedWslDistro,
				linuxPath: wslLinuxPath,
			},
		);
		const descriptor = validation.descriptor;
		const opened = await openRepo({
			name: descriptor.display_name,
			path: descriptor.display_path,
			repoId: descriptor.id,
			repoDescriptor: descriptor,
		});
		if (opened) wslLinuxPath = validation.repo_root;
	} catch (e: unknown) {
		const trunk = e as TrunkError;
		error = trunk.message ?? "Failed to open WSL repository";
	} finally {
		wslLoading = false;
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
      <div class="error-banner w-full rounded-md px-4 py-2 text-sm">
        {error}
      </div>
    {/if}

    <button
      onclick={openRepository}
      disabled={loading}
      class="w-full rounded-md px-4 py-2.5 text-sm font-medium transition-opacity cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
      style="background: var(--color-accent); color: var(--color-on-accent);"
    >
      {loading ? 'Opening...' : 'Open Repository'}
    </button>

    {#if wslAvailability?.supported_platform}
      <div class="w-full flex flex-col gap-2 rounded-md p-3" style="border: 1px solid var(--color-border); background: var(--color-surface);">
        <div class="flex items-center justify-between gap-3">
          <span class="text-sm font-medium" style="color: var(--color-text);">Open from WSL</span>
          {#if !wslAvailability.available}
            <span class="text-xs" style="color: var(--color-text-muted);">Unavailable</span>
          {/if}
        </div>

        {#if wslAvailability.available}
          <select
            bind:value={selectedWslDistro}
            disabled={wslLoading || wslDistros.length === 0}
            class="w-full rounded px-2 py-1.5 text-sm outline-none"
            style="background: var(--color-bg); color: var(--color-text); border: 1px solid var(--color-border);"
            aria-label="WSL distro"
          >
            {#each wslDistros as distro (distro.name)}
              <option value={distro.name}>{distro.name}{distro.default ? ' (default)' : ''}</option>
            {/each}
          </select>
          <div class="flex gap-2">
            <input
              bind:value={wslLinuxPath}
              disabled={wslLoading || !selectedWslDistro}
              placeholder="/home/me/project"
              class="min-w-0 flex-1 rounded px-2 py-1.5 text-sm outline-none"
              style="background: var(--color-bg); color: var(--color-text); border: 1px solid var(--color-border);"
              onkeydown={(e) => e.key === 'Enter' && openWslRepository()}
            />
            <button
              onclick={openWslRepository}
              disabled={wslLoading || !selectedWslDistro || !wslLinuxPath.trim()}
              class="rounded-md px-3 py-1.5 text-sm font-medium cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
              style="background: var(--color-accent); color: var(--color-on-accent);"
            >
              {wslLoading ? 'Opening...' : 'Open'}
            </button>
          </div>
        {:else if wslAvailability.message}
          <p class="text-xs leading-relaxed" style="color: var(--color-text-muted);">{wslAvailability.message}</p>
        {/if}
      </div>
    {/if}
  </div>

  {#if displayedRepos.length > 0}
    <div class="w-full max-w-md px-4">
      <p class="text-xs font-medium mb-2 uppercase tracking-widest" style="color: var(--color-text-muted);">Recent</p>
      <ul class="flex flex-col gap-1">
        {#each displayedRepos as repo (repoKey(repo))}
          {@const key = repoKey(repo)}
          {@const displayPathValue = repoDisplayPath(repo)}
          {@const dp = resolvedPaths[key] ?? displayPathValue}
          <li>
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <div
              class="group flex items-center gap-2 rounded px-3 py-1.5 cursor-pointer hover:bg-white/5"
              onclick={() => openRepo(repo)}
              role="button"
              tabindex="0"
              onkeydown={(e) => e.key === 'Enter' && openRepo(repo)}
            >
              <span class="text-sm truncate min-w-0 flex-1">
                <span style="color: var(--color-text-muted);">{dp.substring(0, dp.lastIndexOf('/'))}/</span><span class="font-semibold" style="color: var(--color-text);">{dp.split('/').at(-1)}</span>
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

<style>
  .error-banner {
    background: var(--color-danger-bg);
    border: 1px solid var(--color-danger-border);
    color: var(--color-danger);
  }
</style>
