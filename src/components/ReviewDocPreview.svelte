<script lang="ts">
// Phase 70 D-02 — markdown preview of the generated review doc. Lives as the
// panel-internal `preview` face of ReviewPanel (panelMode === "preview"). The
// body is a <pre> with white-space: pre so long lines do NOT wrap — the user
// reviews exactly what the AI agent will see (RESEARCH Item 6).
//
// Phase 71 (this addition) docks a Copy button into the .preview-spacer flex
// cell. The handler awaits @tauri-apps/plugin-clipboard-manager.writeText with
// the markdown prop verbatim; on success the button label/icon swap to
// "✓ Copied" for 1500ms (re-click extends the window via clearTimeout); on
// failure showToast surfaces the underlying reason. No new IPC, no new
// capability — clipboard-manager:allow-write-text was granted in Phase 65.
import { Clipboard } from "@lucide/svelte";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { showToast } from "../lib/toast.svelte.js";

interface Props {
	markdown: string;
	onBack: () => void;
}

let { markdown, onBack }: Props = $props();

let copied = $state(false);
// Plain handle, not $state — only used to clear; reactivity is on `copied`.
let copyTimer: ReturnType<typeof setTimeout> | null = null;

async function onCopy() {
	try {
		await writeText(markdown);
		// Pitfall 2: clear any in-flight revert timer before scheduling a new
		// one. Rapid re-clicks must extend the affordance, not race against it.
		if (copyTimer !== null) clearTimeout(copyTimer);
		copied = true;
		copyTimer = setTimeout(() => {
			copied = false;
			copyTimer = null;
		}, 1500);
	} catch (e) {
		// `unknown` in TS strict; narrow rather than cast. Pitfall 1 + 3.
		const msg = e instanceof Error ? e.message : String(e);
		showToast(`Failed to copy: ${msg}`, "error");
	}
}
</script>

<div class="preview-wrap">
  <header class="preview-header">
    <button
      type="button"
      class="back-button"
      onclick={onBack}
    >← Back to comments</button>
    <span class="preview-spacer"></span>
    <button type="button" class="copy-button" onclick={onCopy}>
      {#if copied}
        <span aria-hidden="true">✓</span>
        <span>Copied</span>
      {:else}
        <Clipboard size={14} />
        <span>Copy</span>
      {/if}
    </button>
  </header>
  <pre class="preview-body">{markdown}</pre>
</div>

<style>
  .preview-wrap {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    background: var(--color-bg);
    color: var(--color-text);
  }
  .preview-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    font-size: 12px;
  }
  .preview-spacer { flex: 1; }
  .back-button {
    background: transparent;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 8px;
    font-size: 12px;
    font-family: inherit;
  }
  .back-button:hover,
  .back-button:focus-visible {
    color: var(--color-text);
    background: var(--color-hover);
  }
  .preview-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    margin: 0;
    padding: 12px;
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    line-height: 1.5;
    /* `pre`, not `pre-wrap` — long lines must overflow horizontally so the
       user sees exactly what the AI agent will see (no soft wrapping). */
    white-space: pre;
    background: var(--color-bg);
    color: var(--color-text);
  }
  .copy-button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 8px;
    font-size: 12px;
    font-family: inherit;
  }
  .copy-button:hover,
  .copy-button:focus-visible {
    color: var(--color-text);
    background: var(--color-hover);
  }
</style>
