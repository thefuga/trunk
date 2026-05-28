<script lang="ts">
interface Props {
	title: string;
}

let { title }: Props = $props();

let isOpen = $state(false);
let text = $state("");
let resolveFn: ((value: string | null) => void) | null = null;

export function open(defaultValue: string): Promise<string | null> {
	text = defaultValue;
	isOpen = true;
	return new Promise((resolve) => {
		resolveFn = resolve;
	});
}

function close(result: string | null) {
	isOpen = false;
	const resolver = resolveFn;
	resolveFn = null;
	resolver?.(result);
}

function handleSubmit() {
	close(text.trim().length === 0 ? null : text);
}

function handleCancel() {
	close(null);
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === "Escape") {
		e.preventDefault();
		handleCancel();
	} else if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
		e.preventDefault();
		handleSubmit();
	}
}

function handleBackdropClick(e: MouseEvent) {
	if (e.target === e.currentTarget) {
		handleCancel();
	}
}

function autofocus(node: HTMLElement) {
	node.focus();
}
</script>

{#if isOpen}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 flex items-center justify-center"
		style="z-index: 9999; background: var(--color-backdrop);"
		data-testid="message-editor-backdrop"
		onkeydown={handleKeydown}
		onclick={handleBackdropClick}
	>
		<div
			class="rounded-lg shadow-xl"
			style="background: var(--color-surface); border: 1px solid var(--color-border); min-width: 420px; max-width: 640px; padding: 16px;"
		>
			<h3 class="text-sm font-semibold mb-3" style="color: var(--color-text);">
				{title}
			</h3>

			<textarea
				class="w-full rounded text-sm"
				style="background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-text); padding: 6px 8px; resize: vertical; min-height: 200px;"
				bind:value={text}
				use:autofocus
			></textarea>

			<div class="flex justify-end gap-2 mt-4">
				<button
					class="rounded px-3 py-1.5 text-xs font-medium"
					style="background: var(--color-bg); border: 1px solid var(--color-border); color: var(--color-text);"
					onclick={handleCancel}
				>
					Cancel
				</button>
				<button
					class="rounded px-3 py-1.5 text-xs font-medium"
					style="background: var(--color-accent); color: var(--color-on-accent);"
					onclick={handleSubmit}
				>
					Save
				</button>
			</div>
		</div>
	</div>
{/if}
