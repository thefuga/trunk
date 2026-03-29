import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import type { FileDiff } from "../lib/types.js";
import DiffPanel from "./DiffPanel.svelte";

// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

// Mock invoke and toast for hunk staging operations
vi.mock("../lib/invoke.js", () => ({
	safeInvoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));

const testDiff: FileDiff = {
	path: "src/main.ts",
	status: "Modified",
	is_binary: false,
	hunks: [
		{
			header: "@@ -1,3 +1,4 @@",
			old_start: 1,
			old_lines: 3,
			new_start: 1,
			new_lines: 4,
			lines: [
				{
					origin: "Context",
					content: "import { foo } from 'bar';",
					old_lineno: 1,
					new_lineno: 1,
					spans: [],
				},
				{
					origin: "Delete",
					content: "const x = 1;",
					old_lineno: 2,
					new_lineno: null,
					spans: [],
				},
				{
					origin: "Add",
					content: "const x = 2;",
					old_lineno: null,
					new_lineno: 2,
					spans: [],
				},
				{
					origin: "Add",
					content: "const y = 3;",
					old_lineno: null,
					new_lineno: 3,
					spans: [],
				},
				{
					origin: "Context",
					content: "export { x };",
					old_lineno: 3,
					new_lineno: 4,
					spans: [],
				},
			],
		},
	],
};

const binaryDiff: FileDiff = {
	path: "image.png",
	status: "Modified",
	is_binary: true,
	hunks: [],
};

const testDiffWithMergedSpans: FileDiff = {
	path: "src/main.rs",
	status: "Modified",
	is_binary: false,
	hunks: [
		{
			header: "@@ -1,1 +1,1 @@",
			old_start: 1,
			old_lines: 1,
			new_start: 1,
			new_lines: 1,
			lines: [
				{
					origin: "Delete",
					content: "hello world",
					old_lineno: 1,
					new_lineno: null,
					spans: [
						{ start: 0, end: 6, syntax_class: "syn-keyword", emphasized: false },
						{ start: 6, end: 11, syntax_class: "syn-string", emphasized: true },
					],
				},
				{
					origin: "Add",
					content: "hello mars",
					old_lineno: null,
					new_lineno: 1,
					spans: [
						{ start: 0, end: 6, syntax_class: "syn-keyword", emphasized: false },
						{ start: 6, end: 10, syntax_class: "syn-string", emphasized: true },
					],
				},
			],
		},
	],
};

describe("DiffPanel", () => {
	it("renders hunk header", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		expect(screen.getByText("@@ -1,3 +1,4 @@")).toBeInTheDocument();
	});

	it("renders added lines with + marker", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// originSymbol("Add") = "+", content follows
		expect(screen.getByText("+const x = 2;")).toBeInTheDocument();
	});

	it("renders deleted lines with - marker", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		expect(screen.getByText("-const x = 1;")).toBeInTheDocument();
	});

	it("renders context lines", () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// Context lines rendered as " " + content (space marker + content)
		// Testing Library normalizes leading whitespace, so check raw textContent
		const bodyText = container.textContent ?? "";
		expect(bodyText).toContain("import { foo } from 'bar';");
		expect(bodyText).toContain("export { x };");
	});

	it("renders file path in multi-file view", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				selectedPath: null,
				onclose: vi.fn(),
			},
		});
		// When selectedPath is null, file header bar shows the path
		expect(screen.getByText("src/main.ts")).toBeInTheDocument();
	});

	it("shows binary file indicator", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [binaryDiff],
				commitDetail: null,
				selectedPath: null,
				onclose: vi.fn(),
			},
		});
		expect(
			screen.getByText(/Binary file.*no diff available/),
		).toBeInTheDocument();
	});

	it("calls onclose when close button clicked", async () => {
		const onclose = vi.fn();
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose,
			},
		});
		const closeBtn = screen.getByLabelText("Close diff");
		await fireEvent.click(closeBtn);
		expect(onclose).toHaveBeenCalledOnce();
	});

	it("shows Stage Hunk button for unstaged diffs", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		expect(screen.getByText("Stage Hunk")).toBeInTheDocument();
		expect(screen.getByText("Discard Hunk")).toBeInTheDocument();
	});

	it("shows Unstage Hunk button for staged diffs", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "staged",
				repoPath: "/test/repo",
			},
		});
		expect(screen.getByText("Unstage Hunk")).toBeInTheDocument();
	});

	it("does not show hunk action buttons for commit diffs", () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "commit",
			},
		});
		expect(screen.queryByText("Stage Hunk")).toBeNull();
		expect(screen.queryByText("Unstage Hunk")).toBeNull();
		expect(screen.queryByText("Discard Hunk")).toBeNull();
	});

	it("renders word-span highlights for emphasized segments", () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		const deleteSpans = container.querySelectorAll(".word-delete");
		const addSpans = container.querySelectorAll(".word-add");
		expect(deleteSpans.length).toBeGreaterThanOrEqual(1);
		expect(addSpans.length).toBeGreaterThanOrEqual(1);
		const deleteTexts = Array.from(deleteSpans).map((el) => el.textContent);
		const addTexts = Array.from(addSpans).map((el) => el.textContent);
		expect(deleteTexts).toContain("world");
		expect(addTexts).toContain("mars");
	});

	it("renders non-emphasized spans without highlight class", () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// "hello " text should not be inside a .word-add or .word-delete element
		const highlightedEls = container.querySelectorAll(
			".word-add, .word-delete",
		);
		const highlightedTexts = Array.from(highlightedEls).map(
			(el) => el.textContent,
		);
		// None of the highlighted spans should contain "hello "
		for (const text of highlightedTexts) {
			expect(text).not.toContain("hello ");
		}
		// But the container should still have "hello " in the rendered text
		expect(container.textContent).toContain("hello ");
	});

	it("falls back to plain rendering when spans is empty", () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// No word-span highlight elements should exist
		expect(container.querySelectorAll(".word-add").length).toBe(0);
		expect(container.querySelectorAll(".word-delete").length).toBe(0);
		// Line content still renders with origin symbols
		expect(container.textContent).toContain("+const x = 2;");
	});

	it("renders syntax class on span elements", () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		const keywordSpans = container.querySelectorAll(".syn-keyword");
		expect(keywordSpans.length).toBeGreaterThanOrEqual(1);
		const stringSpans = container.querySelectorAll(".syn-string");
		expect(stringSpans.length).toBeGreaterThanOrEqual(1);
	});

	it("applies opacity reduction class on add/delete lines", () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// Verify diff-line-add and diff-line-delete classes exist on line containers
		const addLines = container.querySelectorAll(".diff-line-add");
		const deleteLines = container.querySelectorAll(".diff-line-delete");
		expect(addLines.length).toBeGreaterThanOrEqual(1);
		expect(deleteLines.length).toBeGreaterThanOrEqual(1);
	});

	it("renders syntax and word-diff classes simultaneously on emphasized spans", () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// Emphasized spans on Delete lines should have both syn-string and word-delete
		const combinedSpans = container.querySelectorAll(".syn-string.word-delete");
		expect(combinedSpans.length).toBeGreaterThanOrEqual(1);
		// Emphasized spans on Add lines should have both syn-string and word-add
		const combinedAddSpans = container.querySelectorAll(".syn-string.word-add");
		expect(combinedAddSpans.length).toBeGreaterThanOrEqual(1);
	});
});
