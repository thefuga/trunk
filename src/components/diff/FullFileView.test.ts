import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { describe, expect, it, vi } from "vitest";
import type { FileDiff } from "../../lib/types.js";
import FullFileView from "./FullFileView.svelte";

// FullFileView renders the flat full-file line list and owns net-new contiguous
// click + shift-click selection state. It never calls IPC — it only bubbles the
// selected flat indices up via oncommentfullfile. No safeInvoke mock needed.

// A Modified file at a commit: context + add lines on the new side, plus one
// Delete line (new_lineno=null) that must NOT be a valid selection endpoint.
const modifiedFile: FileDiff = {
	path: "src/main.ts",
	status: "Modified",
	is_binary: false,
	hunks: [
		{
			header: "@@ -10,3 +10,4 @@",
			old_start: 10,
			old_lines: 3,
			new_start: 10,
			new_lines: 4,
			lines: [
				{
					origin: "Context",
					content: "context before",
					old_lineno: 10,
					new_lineno: 10,
					spans: [],
				},
				{
					origin: "Add",
					content: "added one",
					old_lineno: null,
					new_lineno: 11,
					spans: [],
				},
				{
					origin: "Add",
					content: "added two",
					old_lineno: null,
					new_lineno: 12,
					spans: [],
				},
				{
					origin: "Delete",
					content: "removed one",
					old_lineno: 11,
					new_lineno: null,
					spans: [],
				},
				{
					origin: "Add",
					content: "added three",
					old_lineno: null,
					new_lineno: 13,
					spans: [],
				},
			],
		},
	],
};

const emptyFile: FileDiff = {
	path: "src/empty.ts",
	status: "Unknown",
	is_binary: false,
	hunks: [],
};

function defaultProps(overrides: Record<string, unknown> = {}) {
	return {
		fileDiffs: [modifiedFile],
		showInvisibles: false,
		wordWrap: false,
		commitOid: "abc123",
		repoPath: "/repo",
		diffKind: "commit" as const,
		isMerge: false,
		oncommentfullfile: vi.fn(),
		...overrides,
	};
}

// Selectable line rows expose role="button"; the gutters/affordance do not. The
// affordance is a real <button>, so query lines by their content text instead.
function lineRow(text: string): HTMLElement {
	const content = screen.getByText(text);
	const row = content.closest('[role="button"]') as HTMLElement | null;
	if (!row) throw new Error(`no selectable row for "${text}"`);
	return row;
}

describe("FullFileView", () => {
	it("V5: an empty/zero-hunk file renders no Comment affordance and never throws", () => {
		expect(() =>
			render(FullFileView, {
				props: defaultProps({ fileDiffs: [emptyFile] }),
			}),
		).not.toThrow();

		expect(screen.queryByRole("button", { name: /comment/i })).toBeNull();
	});

	it("V6: a click sets a single-line selection and the affordance reports count 1", async () => {
		render(FullFileView, { props: defaultProps() });

		// No selection yet -> no affordance.
		expect(screen.queryByRole("button", { name: /comment/i })).toBeNull();

		await fireEvent.click(lineRow("added one"));
		await tick();

		expect(screen.getByRole("button", { name: /comment \(1\)/i })).toBeTruthy();
	});

	it("V6: shift-click extends a contiguous span and bubbles the flat indices", async () => {
		const oncommentfullfile = vi.fn();
		render(FullFileView, { props: defaultProps({ oncommentfullfile }) });

		await fireEvent.click(lineRow("added one")); // flat index 1
		await tick();
		// Shift-click "added three" (flat index 4); the contiguous span is 1..4.
		await fireEvent.click(lineRow("added three"), { shiftKey: true });
		await tick();

		const affordance = screen.getByRole("button", { name: /comment/i });
		await fireEvent.click(affordance);

		expect(oncommentfullfile).toHaveBeenCalledTimes(1);
		const [filePath, indices] = oncommentfullfile.mock.calls[0];
		expect(filePath).toBe("src/main.ts");
		// Indices are the flat line-list positions of the contiguous span (1..4).
		const sorted = Array.from(indices as Set<number>).sort((a, b) => a - b);
		expect(sorted).toEqual([1, 2, 3, 4]);
	});

	it("V6/D-02: a Delete line (new_lineno=null) is not selectable and not an endpoint", async () => {
		render(FullFileView, { props: defaultProps() });

		// The Delete line renders, but is not a selectable row (no role="button").
		const deleteContent = screen.getByText("removed one");
		expect(deleteContent.closest('[role="button"]')).toBeNull();

		// Clicking its row directly must not open a selection / affordance.
		const deleteRow = deleteContent.closest(".diff-line") as HTMLElement;
		await fireEvent.click(deleteRow);
		await tick();

		expect(screen.queryByRole("button", { name: /comment/i })).toBeNull();
	});

	it("V10/L-05: with isMerge=true the Comment affordance is present and NOT disabled", async () => {
		render(FullFileView, { props: defaultProps({ isMerge: true }) });

		await fireEvent.click(lineRow("added one"));
		await tick();

		const affordance = screen.getByRole("button", {
			name: /comment/i,
		}) as HTMLButtonElement;
		expect(affordance).toBeTruthy();
		expect(affordance.disabled).toBe(false);
	});
});
