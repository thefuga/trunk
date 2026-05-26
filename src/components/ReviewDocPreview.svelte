<script lang="ts">
// Phase 70 D-02 — markdown preview of the generated review doc. Lives as the
// panel-internal `preview` face of ReviewPanel (panelMode === "preview"). The
// body is a <pre> with white-space: pre so long lines do NOT wrap — the user
// reviews exactly what the AI agent will see (RESEARCH Item 6).
//
// The .preview-spacer flex cell in the header is load-bearing: Phase 71 docks
// Copy / Save buttons there. Keep it even though it is visually empty.

interface Props {
	markdown: string;
	onBack: () => void;
}

let { markdown, onBack }: Props = $props();
</script>

<div class="preview-wrap">
  <header class="preview-header">
    <button
      type="button"
      class="back-button"
      onclick={onBack}
    >← Back to comments</button>
    <!-- Phase 71 Copy/Save buttons will dock to the right of this spacer. -->
    <span class="preview-spacer"></span>
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
</style>
