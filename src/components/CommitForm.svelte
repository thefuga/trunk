<script lang="ts">
import { safeInvoke } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type { HeadCommitMessage } from "../lib/types.js";

interface Props {
	repoPath: string;
	stagedCount: number;
	onsubjectchange?: (value: string) => void;
	clearRedoStack: () => void;
}

let { repoPath, stagedCount, onsubjectchange, clearRedoStack }: Props =
	$props();

let subject = $state("");
let body = $state("");
let mode = $state<"commit" | "amend" | "stash">("commit");
let committing = $state(false);
let subjectError = $state("");
let stagedError = $state("");

let buttonLabel = $derived.by(() => {
	if (committing) {
		return mode === "commit"
			? "Committing..."
			: mode === "amend"
				? "Amending..."
				: "Stashing...";
	}
	return mode === "commit" ? "Commit" : mode === "amend" ? "Amend" : "Stash";
});

// Clear stagedError when stagedCount changes or mode changes
$effect(() => {
	// access reactive values to track them
	const _staged = stagedCount;
	const _mode = mode;
	stagedError = "";
});

async function handleModeSwitch(newMode: "commit" | "amend" | "stash") {
	if (newMode === mode) return;
	mode = newMode;
	if (newMode === "amend") {
		try {
			const msg = await safeInvoke<HeadCommitMessage>(
				"get_head_commit_message",
				{
					path: repoPath,
				},
			);
			subject = msg.subject;
			body = msg.body ?? "";
		} catch (e) {
			console.error("Failed to get HEAD commit message:", e);
		}
	}
	// Switching away from amend or between commit/stash: keep current values (don't clear)
}

async function handleSubmit() {
	subjectError = "";
	stagedError = "";

	// Stash mode: subject is optional (stash name). Commit/amend: subject required.
	if (mode !== "stash" && !subject.trim()) {
		subjectError = "Subject is required";
		return;
	}

	// All modes require staged files (except amend which can amend message-only)
	if (mode !== "amend" && stagedCount === 0) {
		stagedError = "No files staged";
		return;
	}

	// clearRedoStack only for commit/amend (modifies history), not stash
	if (mode !== "stash") {
		clearRedoStack();
	}

	committing = true;
	try {
		if (mode === "amend") {
			await safeInvoke("amend_commit", {
				path: repoPath,
				subject: subject.trim(),
				body: body.trim() || null,
			});
		} else if (mode === "stash") {
			await safeInvoke("stash_save", {
				path: repoPath,
				message: subject.trim(),
			});
			showToast("Stash created", "success");
		} else {
			await safeInvoke("create_commit", {
				path: repoPath,
				subject: subject.trim(),
				body: body.trim() || null,
			});
		}
		subject = "";
		onsubjectchange?.("");
		body = "";
		mode = "commit"; // Always reset to commit mode after any successful operation
	} catch (e) {
		const err = e as { message?: string };
		const action =
			mode === "commit" ? "Commit" : mode === "amend" ? "Amend" : "Stash";
		console.error(`${action} failed:`, e);
		if (mode === "stash") {
			showToast(err.message ?? "Stash failed", "error");
		}
	} finally {
		committing = false;
	}
}
</script>

<div style="
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
">
  <!-- Mode tab selector -->
  <div style="display: flex; gap: 0; border-bottom: 1px solid var(--color-border); margin-bottom: 2px;">
    {#each [['commit', 'Commit'], ['amend', 'Amend'], ['stash', 'Stash']] as [tab, label]}
      <button
        onclick={() => handleModeSwitch(tab as 'commit' | 'amend' | 'stash')}
        disabled={committing}
        style="
          flex: 1;
          padding: 6px 0 4px;
          font-size: 12px;
          background: none;
          border: none;
          border-bottom: 2px solid {mode === tab ? 'var(--color-accent)' : 'transparent'};
          color: {mode === tab ? 'var(--fg-0)' : 'var(--fg-3)'};
          cursor: {committing ? 'default' : 'pointer'};
          text-transform: none;
        "
      >
        {label}
      </button>
    {/each}
  </div>

  <!-- Subject field -->
  <input
    data-testid="commit-form-subject"
    type="text"
    bind:value={subject}
    placeholder={mode === 'stash' ? 'Stash name (optional)' : 'Summary (required)'}
    oninput={(e) => { if (subjectError) subjectError = ''; onsubjectchange?.((e.target as HTMLInputElement).value); }}
    style="
      width: 100%;
      box-sizing: border-box;
      border: 1px solid var(--line);
      background: var(--bg-0);
      color: var(--fg-1);
      border-radius: var(--radius-m);
      padding: 8px 10px;
      font-size: 12px;
    "
  />
  {#if subjectError}
    <span class="error-text" style="font-size: 11px;">{subjectError}</span>
  {/if}

  <!-- Body field -->
  <textarea
    bind:value={body}
    rows={3}
    placeholder="Description (optional)"
    style="
      width: 100%;
      box-sizing: border-box;
      border: 1px solid var(--line);
      background: var(--bg-0);
      color: var(--fg-1);
      border-radius: var(--radius-m);
      padding: 8px 10px;
      font-size: 12px;
      resize: vertical;
    "
  ></textarea>

  <!-- Staged error -->
  {#if stagedError}
    <span class="error-text" style="font-size: 11px;">{stagedError}</span>
  {/if}

  <!-- Commit button -->
  <button
    data-testid="commit-form-submit"
    onclick={handleSubmit}
    disabled={committing}
    style="
      width: 100%;
      height: 32px;
      background: var(--accent);
      color: var(--accent-fg);
      border: 0;
      border-radius: var(--radius-m);
      font-size: 12.5px;
      font-weight: 600;
      cursor: pointer;
      opacity: {committing ? 0.6 : 1};
    "
  >
    {buttonLabel}
  </button>
</div>

<style>
  .error-text {
    color: var(--color-danger);
  }
</style>
