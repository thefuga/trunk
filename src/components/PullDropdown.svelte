<script lang="ts">
import { ChevronDown } from "@lucide/svelte";
import type { TrunkError } from "../lib/invoke.js";
import { safeInvoke } from "../lib/invoke.js";
import type { RemoteState } from "../lib/remote-state.svelte.js";
import { showToast } from "../lib/toast.svelte.js";

interface Props {
	repoPath: string;
	disabled: boolean;
	remoteState: RemoteState;
}

let { repoPath, disabled, remoteState }: Props = $props();
let open = $state(false);

interface PullOption {
	label: string;
	action: () => Promise<void>;
}

const options: PullOption[] = [
	{
		label: "Fetch",
		action: () => runRemote("git_fetch", "Fetched successfully", {}),
	},
	{
		label: "Fast-forward if possible",
		action: () =>
			runRemote("git_pull", "Pulled successfully", { strategy: "ff" }),
	},
	{
		label: "Fast-forward only",
		action: () =>
			runRemote("git_pull", "Pulled successfully", { strategy: "ff-only" }),
	},
	{
		label: "Pull (rebase)",
		action: () =>
			runRemote("git_pull", "Pulled successfully (rebase)", {
				strategy: "rebase",
			}),
	},
];

function errorMessage(error: TrunkError): string {
	switch (error.code) {
		case "auth_failure":
			return "Authentication failed \u2014 check your SSH key or credential helper";
		case "non_fast_forward":
			return "Push rejected (non-fast-forward)";
		default:
			return error.message;
	}
}

async function runRemote(
	cmd: string,
	successMsg: string,
	extra: Record<string, unknown>,
) {
	remoteState.isRunning = true;
	remoteState.error = null;
	remoteState.progressLine = "";
	try {
		await safeInvoke(cmd, { path: repoPath, ...extra });
		remoteState.isRunning = false;
		remoteState.progressLine = "";
		showToast(successMsg, "success");
	} catch (e: unknown) {
		remoteState.isRunning = false;
		const err = e as TrunkError;
		remoteState.error = err;
		showToast(errorMessage(err), "error");
	}
}

function handleOptionClick(opt: PullOption) {
	open = false;
	opt.action();
}

function toggle() {
	if (!disabled) open = !open;
}

// Close on outside click
function handleWindowClick(e: MouseEvent) {
	const target = e.target as HTMLElement;
	if (!target.closest(".pull-dropdown")) {
		open = false;
	}
}

$effect(() => {
	if (open) {
		window.addEventListener("click", handleWindowClick, true);
		return () => window.removeEventListener("click", handleWindowClick, true);
	}
});
</script>

<style>
  .pull-dropdown {
    position: relative;
    display: inline-flex;
  }

  .chevron-btn {
    background: none;
    border: none;
    border-left: 1px solid var(--line);
    border-radius: 0 var(--radius-m) var(--radius-m) 0;
    color: var(--fg-2);
    cursor: pointer;
    font-size: 10px;
    padding: 0 6px;
    height: 100%;
    display: flex;
    align-items: center;
  }
  .chevron-btn:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
  .chevron-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg-1);
  }
  .chevron-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }

  .dropdown-panel {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 100;
    margin-top: 4px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    border-radius: var(--radius-m);
    box-shadow: var(--shadow-md);
    min-width: 180px;
    padding: 4px 0;
  }

  .dropdown-option {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    color: var(--fg-1);
    font-size: 12px;
    padding: 6px 12px;
    cursor: pointer;
  }
  .dropdown-option:hover {
    background: var(--accent);
    color: var(--accent-fg);
  }
</style>

<div class="pull-dropdown">
  <button class="chevron-btn" onclick={toggle} disabled={disabled} title="Pull options">
    <ChevronDown size={12} />
  </button>

  {#if open}
    <div class="dropdown-panel">
      {#each options as opt}
        <button class="dropdown-option" onclick={() => handleOptionClick(opt)}>
          {opt.label}
        </button>
      {/each}
    </div>
  {/if}
</div>
