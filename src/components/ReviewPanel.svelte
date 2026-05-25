<script lang="ts">
// THROWAWAY STUB (Phase 65, D-12): replaced by the real review panel in Phase 69.
// Smallest thing that makes SESS-01/02/03 hand-verifiable: three session states
// (no-session / resume-available / session-active) with lifecycle buttons that
// invoke the Plan 65-03 commands and toast on error, plus a session-changed
// listener so other tabs/windows reload. Do not over-invest.
import { listen } from "@tauri-apps/api/event";
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type { SessionCommit, SessionStatus } from "../lib/types.js";

interface Props {
	repoPath: string;
}

let { repoPath }: Props = $props();

let status = $state<SessionStatus | null>(null);
let sessionCommits = $state<SessionCommit[]>([]);
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

// D-05/SEL-04: the in-session commit list (short SHA + summary, graph order +
// dedup imposed server-side). No-session is a normal state, so swallow the error
// silently — never toast on an inactive session.
async function reloadCommits() {
	try {
		sessionCommits = await safeInvoke<SessionCommit[]>("list_session_commits", {
			path: repoPath,
		});
	} catch {
		sessionCommits = [];
	}
}

// D-07/SEL-03: remove a commit from the list the user is looking at. Silent
// success — remove_review_commit emits session-changed, which the listener below
// turns into a reload.
async function removeCommit(oid: string) {
	try {
		await safeInvoke("remove_review_commit", { path: repoPath, oid });
	} catch (e) {
		showToast((e as TrunkError).message ?? "Failed to remove commit", "error");
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
	reloadCommits();
});

// Live coordination (DP-01): reload when a session-changed event arrives for
// this repo's canonical path. Mirrors App.svelte's repo-changed listener.
$effect(() => {
	let unlisten: (() => void) | undefined;
	listen<string>("session-changed", (event) => {
		if (status && event.payload !== status.canonical_path) return;
		reloadStatus();
		reloadCommits();
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
    {#if sessionCommits.length === 0}
      <span style="color: var(--color-text-muted);">No commits selected yet</span>
    {:else}
      <ul class="flex flex-col" style="gap: 2px; list-style: none; margin: 0; padding: 0;">
        {#each sessionCommits as commit (commit.oid)}
          <li
            data-testid="session-commit-row"
            class="flex items-center"
            style="gap: 8px; padding: 2px 0;"
          >
            <span class="font-mono" style="color: var(--color-text-muted); flex-shrink: 0;">{commit.short_oid}</span>
            <span class="overflow-hidden text-ellipsis whitespace-nowrap" style="flex: 1;">{commit.summary}</span>
            <button
              aria-label="Remove commit {commit.short_oid}"
              onclick={() => removeCommit(commit.oid)}
              style="
                background: var(--color-danger-bg);
                color: var(--color-danger);
                border: 1px solid var(--color-danger-border);
                border-radius: 4px;
                cursor: pointer;
                padding: 0 6px;
                flex-shrink: 0;
              "
            >×</button>
          </li>
        {/each}
      </ul>
    {/if}
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
