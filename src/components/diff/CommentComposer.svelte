<script lang="ts">
import { buildDiffAnchor } from "../../lib/diff-anchor.js";
import { safeInvoke, type TrunkError } from "../../lib/invoke.js";
import { showToast } from "../../lib/toast.svelte.js";
import type { Anchor, FileDiff } from "../../lib/types.js";

interface Props {
	// Diff path passes file/hunkIdx/selectedLineIndices and the composer derives
	// the captured result via buildDiffAnchor. The full-file host instead injects
	// a pre-built `captured` result (from buildFullFileAnchor) and omits the three
	// diff-path props. Exactly one of the two contracts is satisfied by the caller.
	captured?: { anchor: Anchor; cachedExcerpt: string };
	file?: FileDiff;
	hunkIdx?: number;
	selectedLineIndices?: Set<number>;
	commitOid: string;
	repoPath: string;
	onclose: () => void;
}

let {
	captured,
	file,
	hunkIdx,
	selectedLineIndices,
	commitOid,
	repoPath,
	onclose,
}: Props = $props();

let text = $state("");
let submitting = $state(false);

const DRAFT_DEBOUNCE_MS = 300;
let draftTimer: ReturnType<typeof setTimeout> | null = null;

// The capture-time adapter is the single source of truth for both the persisted
// range (start_line..end_line) and the excerpt. When the host injects a captured
// result (full-file path) use it directly; otherwise derive it from the diff-path
// props. The diff-path caller always supplies file/hunkIdx/selectedLineIndices,
// so the non-null assertions document that contract rather than guard it.
const capturedResult = $derived(
	captured ?? buildDiffAnchor(commitOid, file!, hunkIdx!, selectedLineIndices!),
);

const submitDisabled = $derived(text.trim() === "" || submitting);

function scheduleDraftSave() {
	if (draftTimer !== null) clearTimeout(draftTimer);
	draftTimer = setTimeout(() => {
		draftTimer = null;
		void persistDraft();
	}, DRAFT_DEBOUNCE_MS);
}

async function persistDraft() {
	try {
		await safeInvoke("save_draft_comment", {
			path: repoPath,
			text,
			anchor: capturedResult.anchor,
		});
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Save draft failed", "error");
	}
}

function handleInput() {
	scheduleDraftSave();
}

async function handleSubmit() {
	if (submitDisabled) return;
	submitting = true;
	if (draftTimer !== null) {
		clearTimeout(draftTimer);
		draftTimer = null;
	}
	try {
		await safeInvoke("add_comment", {
			path: repoPath,
			text,
			anchor: capturedResult.anchor,
			cachedExcerpt: capturedResult.cachedExcerpt,
		});
	} catch (e) {
		const err = e as TrunkError;
		showToast(err.message ?? "Add comment failed", "error");
		return;
	} finally {
		submitting = false;
	}
	text = "";
	onclose();
}

// Instance method the host (DiffPanel) calls before switching the selection to a
// new range. Confirms only when the draft is dirty (non-empty); an empty draft
// switches silently. Mirrors DiffPanel.handleDiscardLines' confirm pattern.
export async function confirmDiscardIfDirty(): Promise<boolean> {
	if (text.trim() === "") return true;
	const { ask } = await import("@tauri-apps/plugin-dialog");
	return ask("Discard your unsaved comment?", {
		title: "Discard Comment",
		kind: "warning",
	});
}
</script>

<div class="comment-composer">
	<div class="composer-preview">
		Comments on lines {capturedResult.anchor.start_line}-{capturedResult.anchor.end_line}
	</div>
	<textarea
		class="composer-textarea"
		placeholder="Leave a comment on these lines…"
		bind:value={text}
		oninput={handleInput}
	></textarea>
	<div class="composer-actions">
		<button class="composer-btn cancel-btn" onclick={onclose}>Cancel</button>
		<button
			class="composer-btn submit-btn"
			disabled={submitDisabled}
			style="cursor: {submitDisabled ? 'not-allowed' : 'pointer'}; opacity: {submitDisabled ? 0.4 : 1};"
			onclick={handleSubmit}
		>Submit</button>
	</div>
</div>

<style>
	.comment-composer {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border);
	}

	.composer-preview {
		color: var(--color-text-muted);
		font-size: 11px;
		font-family: var(--font-mono, monospace);
	}

	.composer-textarea {
		min-height: 60px;
		resize: vertical;
		padding: 6px 8px;
		font-size: 12px;
		font-family: var(--font-sans, sans-serif);
		color: var(--color-text);
		background: var(--color-bg);
		border: 1px solid var(--color-border);
		border-radius: 3px;
		box-sizing: border-box;
	}

	.composer-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.composer-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.composer-btn {
		border-radius: 3px;
		font-size: 11px;
		font-family: var(--font-sans, sans-serif);
		padding: 4px 12px;
		white-space: nowrap;
		cursor: pointer;
	}

	.cancel-btn {
		background: transparent;
		border: 1px solid var(--color-border);
		color: var(--color-text-muted);
	}

	.submit-btn {
		background: var(--color-success-bg);
		border: 1px solid var(--color-success-border);
		color: var(--color-success);
	}
</style>
