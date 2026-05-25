<script lang="ts">
// THROWAWAY STUB (Phase 65, D-12): replaced by the real review panel in Phase 69.
// Smallest thing that makes SESS-01/02/03 hand-verifiable: three session states
// (no-session / resume-available / session-active) with lifecycle buttons that
// invoke the Plan 65-03 commands and toast on error, plus a session-changed
// listener so other tabs/windows reload. Do not over-invest.
import { listen } from "@tauri-apps/api/event";
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type { SessionStatus } from "../lib/types.js";

interface Props {
	repoPath: string;
}

let { repoPath }: Props = $props();

let status = $state<SessionStatus | null>(null);
let loading = $state(false);

let sessionState = $derived(status?.state ?? "none");

async function reloadStatus() {
	try {
		status = await safeInvoke<SessionStatus>("get_review_session_status", {
			path: repoPath,
		});
	} catch (e) {
		showToast(
			(e as TrunkError).message ?? "Failed to load review session",
			"error",
		);
	}
}

async function runLifecycle(cmd: string) {
	loading = true;
	try {
		await safeInvoke(cmd, { path: repoPath });
		await reloadStatus();
	} catch (e) {
		showToast((e as TrunkError).message ?? "Review action failed", "error");
	} finally {
		loading = false;
	}
}

// Initial load when the panel mounts / its repo changes.
$effect(() => {
	reloadStatus();
});

// Live coordination (DP-01): reload when a session-changed event arrives for
// this repo's canonical path. Mirrors App.svelte's repo-changed listener.
$effect(() => {
	let unlisten: (() => void) | undefined;
	listen<string>("session-changed", (event) => {
		if (status && event.payload !== status.canonical_path) return;
		reloadStatus();
	}).then((fn) => {
		unlisten = fn;
	});
	return () => {
		unlisten?.();
	};
});
</script>

<div
  class="flex flex-col"
  style="
    flex-shrink: 0;
    gap: 8px;
    padding: 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 12px;
  "
>
  {#if sessionState === "active"}
    <div class="flex items-center" style="gap: 8px; justify-content: space-between;">
      <span>Code review in progress</span>
      <button
        onclick={() => runLifecycle("end_review_session")}
        disabled={loading}
        style="
          background: var(--color-danger-bg);
          color: var(--color-danger);
          border: 1px solid var(--color-danger-border);
          border-radius: 4px;
          cursor: pointer;
          padding: 2px 8px;
          white-space: nowrap;
        "
      >End Review</button>
    </div>
    <span style="color: var(--color-text-muted);">No comments yet</span>
  {:else if sessionState === "resume-available"}
    <div class="flex items-center" style="gap: 8px; justify-content: space-between;">
      <span>A saved review session is available</span>
      <div class="flex" style="gap: 4px;">
        <button
          onclick={() => runLifecycle("resume_review_session")}
          disabled={loading}
          style="
            background: var(--color-success-bg);
            color: var(--color-success);
            border: 1px solid var(--color-success-border);
            border-radius: 4px;
            cursor: pointer;
            padding: 2px 8px;
            white-space: nowrap;
          "
        >Resume</button>
        <button
          onclick={() => runLifecycle("end_review_session")}
          disabled={loading}
          style="
            background: var(--color-danger-bg);
            color: var(--color-danger);
            border: 1px solid var(--color-danger-border);
            border-radius: 4px;
            cursor: pointer;
            padding: 2px 8px;
            white-space: nowrap;
          "
        >Discard</button>
      </div>
    </div>
  {:else}
    <div class="flex items-center" style="gap: 8px; justify-content: space-between;">
      <span>No code review session</span>
      <button
        onclick={() => runLifecycle("start_review_session")}
        disabled={loading}
        style="
          background: var(--color-banner-info-bg);
          color: var(--color-banner-info-border);
          border: 1px solid var(--color-banner-info-border);
          border-radius: 4px;
          cursor: pointer;
          padding: 2px 8px;
          white-space: nowrap;
        "
      >Start Code Review</button>
    </div>
  {/if}
</div>
