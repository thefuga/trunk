/**
 * Pure key/index logic for the recent-repos picker.
 *
 * Extracted so the keyboard contract (which Backspace closes, which keys
 * preventDefault, what the arrow keys do at list edges) is unit-testable
 * without rendering the component.
 */

export type PickerAction =
	| { kind: "highlight-down" }
	| { kind: "highlight-up" }
	| { kind: "pick" }
	| { kind: "close" }
	| { kind: "ignore" };

export interface PickerKeyContext {
	key: string;
	queryEmpty: boolean;
}

export interface PickerKeyResult {
	action: PickerAction;
	preventDefault: boolean;
}

export function pickerKeyAction(ctx: PickerKeyContext): PickerKeyResult {
	switch (ctx.key) {
		case "ArrowDown":
			return { action: { kind: "highlight-down" }, preventDefault: true };
		case "ArrowUp":
			return { action: { kind: "highlight-up" }, preventDefault: true };
		case "Enter":
			return { action: { kind: "pick" }, preventDefault: true };
		case "Escape":
			return { action: { kind: "close" }, preventDefault: true };
		case "Backspace":
			// Close only when there's nothing to delete; otherwise let the
			// browser keep removing characters.
			return ctx.queryEmpty
				? { action: { kind: "close" }, preventDefault: false }
				: { action: { kind: "ignore" }, preventDefault: false };
		default:
			return { action: { kind: "ignore" }, preventDefault: false };
	}
}

export function nextHighlightedIdx(
	direction: "up" | "down",
	current: number,
	length: number,
): number {
	if (length === 0) return 0;
	if (direction === "down") return Math.min(length - 1, current + 1);
	return Math.max(0, current - 1);
}

export function clampHighlightedIdx(idx: number, length: number): number {
	if (length === 0) return 0;
	if (idx > length - 1) return length - 1;
	if (idx < 0) return 0;
	return idx;
}
