<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { isTrunkError, safeInvoke } from "../lib/invoke.js";
import { repoNameFromPath } from "../lib/path.js";
import type { RecentRepo } from "../lib/store.js";
import { showToast } from "../lib/toast.svelte.js";
import {
	localRepoDescriptor,
	type WslDistro,
	type WslRepoValidation,
} from "../lib/types.js";
import { loadWslOpenTargets, sortDistrosDefaultFirst } from "../lib/wsl.js";
import { parseWslUncPath, wslRootUncPath } from "../lib/wsl-paths.js";

interface Props {
	onpick: (repo: RecentRepo) => void;
	onerror?: (message: string) => void;
	disabled?: boolean;
	label?: string;
	// "overlay" floats the menu over content; "inline" renders it in normal
	// flow for overflow-hidden containers like the recents picker modal.
	menuPlacement?: "overlay" | "inline";
	fullWidth?: boolean;
}

let {
	onpick,
	onerror,
	disabled = false,
	label = "Open Repository",
	menuPlacement = "overlay",
	fullWidth = false,
}: Props = $props();

let menuOpen = $state(false);
let busy = $state(false);
let distros = $state<WslDistro[]>([]);

// Empty unless we're on Windows with working WSL and >=1 installed distro;
// when empty the button opens the local dialog directly.
const wslReady = $derived(distros.length > 0);
const sortedDistros = $derived(sortDistrosDefaultFirst(distros));

$effect(() => {
	loadWslOpenTargets().then((targets) => {
		distros = targets.distros;
	});
});

function fail(message: string) {
	if (onerror) onerror(message);
	else showToast(message, "error");
}

function handleMainClick() {
	if (disabled || busy) return;
	if (!wslReady) {
		void pickLocal();
		return;
	}
	menuOpen = !menuOpen;
}

async function pickLocal() {
	menuOpen = false;
	await runPick(() => openDialog({ directory: true, multiple: false }));
}

async function pickDistro(distro: string) {
	menuOpen = false;
	await runPick(async () => {
		// Probing the home dir also boots the distro, so the UNC browse in the
		// dialog that follows is responsive.
		const defaultPath = await safeInvoke<string>("wsl_default_open_path", {
			distro,
		}).catch(() => wslRootUncPath(distro));
		return openDialog({ directory: true, multiple: false, defaultPath });
	});
}

async function runPick(dialog: () => Promise<string | string[] | null>) {
	busy = true;
	try {
		const selected = await dialog();
		if (typeof selected !== "string") return;
		onpick(await resolvePick(selected));
	} catch (e: unknown) {
		fail(isTrunkError(e) ? e.message : "Failed to open repository");
	} finally {
		busy = false;
	}
}

// A pick is a WSL repo whenever the dialog returned a WSL UNC path — even when
// the user navigated there through "Local" — so it always gets the WSL backend
// instead of git2 over the 9p share.
async function resolvePick(selected: string): Promise<RecentRepo> {
	const wsl = parseWslUncPath(selected);
	if (wsl) {
		const validation = await safeInvoke<WslRepoValidation>(
			"validate_wsl_repo",
			{ distro: wsl.distro, linuxPath: wsl.linuxPath },
		);
		const descriptor = validation.descriptor;
		return {
			name: descriptor.display_name,
			path: descriptor.display_path,
			repoId: descriptor.id,
			repoDescriptor: descriptor,
		};
	}

	const name = repoNameFromPath(selected);
	const descriptor = localRepoDescriptor(selected, name);
	return {
		name,
		path: selected,
		repoId: descriptor.id,
		repoDescriptor: descriptor,
	};
}

// Close on outside click / Escape
function handleWindowClick(e: MouseEvent) {
	const target = e.target as HTMLElement;
	if (!target.closest(".open-repo-dropdown")) {
		menuOpen = false;
	}
}

function handleWindowKeydown(e: KeyboardEvent) {
	if (e.key === "Escape") menuOpen = false;
}

$effect(() => {
	if (menuOpen) {
		window.addEventListener("click", handleWindowClick, true);
		window.addEventListener("keydown", handleWindowKeydown, true);
		return () => {
			window.removeEventListener("click", handleWindowClick, true);
			window.removeEventListener("keydown", handleWindowKeydown, true);
		};
	}
});
</script>

<style>
  .open-repo-dropdown {
    position: relative;
    display: inline-flex;
    flex-direction: column;
  }
  .open-repo-dropdown.full-width {
    display: flex;
    width: 100%;
  }

  .menu-panel {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    min-width: 180px;
    padding: 4px 0;
  }
  .menu-panel.overlay {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 100;
    margin-top: 4px;
    box-shadow: 0 4px 12px var(--color-backdrop);
  }
  .menu-panel.inline {
    margin-top: 8px;
  }

  .menu-option {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: var(--color-text);
    font-size: 13px;
    padding: 6px 12px;
    cursor: pointer;
  }
  .menu-option:hover {
    background: var(--color-hover);
  }
</style>

<div class="open-repo-dropdown" class:full-width={fullWidth}>
	<button
		onclick={handleMainClick}
		disabled={disabled || busy}
		aria-haspopup={wslReady ? "menu" : undefined}
		aria-expanded={wslReady ? menuOpen : undefined}
		class="flex items-center justify-center gap-1.5 rounded-md px-4 py-2.5 text-sm font-medium transition-opacity cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed {fullWidth ? 'w-full' : ''}"
		style="background: var(--color-accent); color: var(--color-on-accent);"
	>
		{busy ? "Opening..." : label}
		{#if wslReady}
			<ChevronDown size={14} />
		{/if}
	</button>

	{#if menuOpen}
		<div class="menu-panel {menuPlacement}" role="menu">
			<button
				type="button"
				role="menuitem"
				class="menu-option"
				onclick={() => void pickLocal()}
			>
				Local
			</button>
			{#each sortedDistros as distro (distro.name)}
				<button
					type="button"
					role="menuitem"
					class="menu-option"
					onclick={() => void pickDistro(distro.name)}
				>
					{distro.name}{distro.default ? " (default)" : ""}
				</button>
			{/each}
		</div>
	{/if}
</div>
