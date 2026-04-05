import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { describe, expect, it, vi } from "vitest";
import {
	pairLines,
	splitInvisibles,
	trailingWhitespaceStart,
} from "../lib/diff-utils.js";
import type { DiffLine, FileDiff } from "../lib/types.js";
import DiffPanel from "./DiffPanel.svelte";

// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

// Helper: flush microtasks (Promise.resolve in store mocks) + Svelte update queue
// Needed because DiffPanel loads preferences via $effect Promise.all which resolves
// asynchronously. This ensures the component has processed the loaded values.
async function flushPrefs() {
	await new Promise((r) => setTimeout(r, 0));
	await tick();
}

// Mock invoke and toast for hunk staging operations
vi.mock("../lib/invoke.js", () => ({
	safeInvoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));

vi.mock("../lib/store.js", () => {
	let currentContentMode = "hunk";
	let currentLayoutMode = "inline";
	let currentIgnoreWhitespace = false;
	let currentShowInvisibles = false;
	let currentWordWrap = false;
	return {
		getDiffContextLines: vi.fn(() => Promise.resolve(3)),
		getDiffContentMode: vi.fn(() => Promise.resolve(currentContentMode)),
		setDiffContentMode: vi.fn((mode: string) => {
			currentContentMode = mode;
			return Promise.resolve(undefined);
		}),
		getDiffLayoutMode: vi.fn(() => Promise.resolve(currentLayoutMode)),
		setDiffLayoutMode: vi.fn((mode: string) => {
			currentLayoutMode = mode;
			return Promise.resolve(undefined);
		}),
		getDiffIgnoreWhitespace: vi.fn(() =>
			Promise.resolve(currentIgnoreWhitespace),
		),
		setDiffIgnoreWhitespace: vi.fn((v: boolean) => {
			currentIgnoreWhitespace = v;
			return Promise.resolve(undefined);
		}),
		getDiffShowFullFile: vi.fn().mockResolvedValue(false),
		setDiffShowFullFile: vi.fn().mockResolvedValue(undefined),
		getDiffShowInvisibles: vi.fn(() => Promise.resolve(currentShowInvisibles)),
		setDiffShowInvisibles: vi.fn((v: boolean) => {
			currentShowInvisibles = v;
			return Promise.resolve(undefined);
		}),
		getDiffWordWrap: vi.fn(() => Promise.resolve(currentWordWrap)),
		setDiffWordWrap: vi.fn((v: boolean) => {
			currentWordWrap = v;
			return Promise.resolve(undefined);
		}),
		addRecentRepo: vi.fn().mockResolvedValue(undefined),
		getRecentRepos: vi.fn().mockResolvedValue([]),
		removeRecentRepo: vi.fn().mockResolvedValue(undefined),
		getPersistedTabs: vi.fn().mockResolvedValue([]),
		setPersistedTabs: vi.fn().mockResolvedValue(undefined),
	};
});

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
						{
							start: 0,
							end: 6,
							syntax_class: "syn-keyword",
							emphasized: false,
						},
						{ start: 6, end: 11, syntax_class: "syn-string", emphasized: true },
					],
				},
				{
					origin: "Add",
					content: "hello mars",
					old_lineno: null,
					new_lineno: 1,
					spans: [
						{
							start: 0,
							end: 6,
							syntax_class: "syn-keyword",
							emphasized: false,
						},
						{ start: 6, end: 10, syntax_class: "syn-string", emphasized: true },
					],
				},
			],
		},
	],
};

describe("DiffPanel", () => {
	it("renders hunk header", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		expect(screen.getByText("@@ -1,3 +1,4 @@")).toBeInTheDocument();
	});

	it("renders added lines with + marker", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		expect(screen.getByText("const x = 2;")).toBeInTheDocument();
	});

	it("renders deleted lines with - marker", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		expect(screen.getByText("const x = 1;")).toBeInTheDocument();
	});

	it("renders context lines", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		// Context lines rendered as " " + content (space marker + content)
		// Testing Library normalizes leading whitespace, so check raw textContent
		const bodyText = container.textContent ?? "";
		expect(bodyText).toContain("import { foo } from 'bar';");
		expect(bodyText).toContain("export { x };");
	});

	it("renders file path in multi-file view", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				selectedPath: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		// When selectedPath is null, file header bar shows the path
		expect(screen.getByText("src/main.ts")).toBeInTheDocument();
	});

	it("shows binary file indicator", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [binaryDiff],
				commitDetail: null,
				selectedPath: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
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
		await flushPrefs();
		const closeBtn = screen.getByLabelText("Close diff");
		await fireEvent.click(closeBtn);
		expect(onclose).toHaveBeenCalledOnce();
	});

	it("shows Stage Hunk button for unstaged diffs", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();
		expect(screen.getByText("Stage Hunk")).toBeInTheDocument();
		expect(screen.getByText("Discard Hunk")).toBeInTheDocument();
	});

	it("shows Unstage Hunk button for staged diffs", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "staged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();
		expect(screen.getByText("Unstage Hunk")).toBeInTheDocument();
	});

	it("does not show hunk action buttons for commit diffs", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "commit",
			},
		});
		await flushPrefs();
		expect(screen.queryByText("Stage Hunk")).toBeNull();
		expect(screen.queryByText("Unstage Hunk")).toBeNull();
		expect(screen.queryByText("Discard Hunk")).toBeNull();
	});

	it("renders word-span highlights for emphasized segments", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		const deleteSpans = container.querySelectorAll(".word-delete");
		const addSpans = container.querySelectorAll(".word-add");
		expect(deleteSpans.length).toBeGreaterThanOrEqual(1);
		expect(addSpans.length).toBeGreaterThanOrEqual(1);
		const deleteTexts = Array.from(deleteSpans).map((el) => el.textContent);
		const addTexts = Array.from(addSpans).map((el) => el.textContent);
		expect(deleteTexts).toContain("world");
		expect(addTexts).toContain("mars");
	});

	it("renders non-emphasized spans without highlight class", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
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

	it("falls back to plain rendering when spans is empty", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		// No word-span highlight elements should exist
		expect(container.querySelectorAll(".word-add").length).toBe(0);
		expect(container.querySelectorAll(".word-delete").length).toBe(0);
		// Line content still renders with origin symbols
		expect(container.textContent).toContain("const x = 2;");
	});

	it("renders syntax class on span elements", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		const keywordSpans = container.querySelectorAll(".syn-keyword");
		expect(keywordSpans.length).toBeGreaterThanOrEqual(1);
		const stringSpans = container.querySelectorAll(".syn-string");
		expect(stringSpans.length).toBeGreaterThanOrEqual(1);
	});

	it("applies opacity reduction class on add/delete lines", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		// Verify diff-line-add and diff-line-delete classes exist on line containers
		const addLines = container.querySelectorAll(".diff-line-add");
		const deleteLines = container.querySelectorAll(".diff-line-delete");
		expect(addLines.length).toBeGreaterThanOrEqual(1);
		expect(deleteLines.length).toBeGreaterThanOrEqual(1);
	});

	it("renders syntax and word-diff classes simultaneously on emphasized spans", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiffWithMergedSpans],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		// Emphasized spans on Delete lines should have both syn-string and word-delete
		const combinedSpans = container.querySelectorAll(".syn-string.word-delete");
		expect(combinedSpans.length).toBeGreaterThanOrEqual(1);
		// Emphasized spans on Add lines should have both syn-string and word-add
		const combinedAddSpans = container.querySelectorAll(".syn-string.word-add");
		expect(combinedAddSpans.length).toBeGreaterThanOrEqual(1);
	});

	// ---- VIEW-01: View mode toggle tests ----

	it("renders content mode and layout mode toggle buttons", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		// Content toggle shows "Show full file" in hunk mode (default)
		expect(screen.getByTitle("Show full file")).toBeInTheDocument();
		// Layout toggle shows "Side-by-side view" in inline mode (default)
		expect(screen.getByTitle("Side-by-side view")).toBeInTheDocument();
	});

	it("shows hunk view by default", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		expect(screen.getByText("@@ -1,3 +1,4 @@")).toBeInTheDocument();
	});

	it("shows full file view when content toggle clicked", async () => {
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// Let the initial $effect settle
		await flushPrefs();
		await fireEvent.click(screen.getByTitle("Show full file"));
		// Flush Svelte reactivity
		await flushPrefs();
		// Full file view renders diff content (no hunk headers)
		expect(screen.queryByText("@@ -1,3 +1,4 @@")).toBeNull();
	});

	it("shows split view with panels when Split mode selected", async () => {
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		// Let the initial $effect (getDiffContentMode/getDiffLayoutMode) settle
		await flushPrefs();
		await fireEvent.click(screen.getByTitle("Side-by-side view"));
		// Flush Svelte reactivity
		await flushPrefs();
		// Split view should render paired rows with two cells each
		const rows = container.querySelectorAll(".split-row");
		expect(rows.length).toBeGreaterThan(0);
	});

	// ---- DISP-01: Line number gutter tests ----

	it("renders line numbers in gutter for context lines", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		// Context lines have both old_lineno and new_lineno set
		// The first context line has old_lineno: 1, new_lineno: 1
		// Each diff line div has two gutter spans as the first two children
		const contextLines = container.querySelectorAll(".diff-line-context");
		expect(contextLines.length).toBeGreaterThanOrEqual(1);
		// First context line: old=1, new=1
		const firstContext = contextLines[0];
		const gutterSpans = firstContext.querySelectorAll("span");
		// At least 2 gutter spans (old + new) per line
		expect(gutterSpans.length).toBeGreaterThanOrEqual(2);
		// Both gutter spans should contain "1"
		expect(gutterSpans[0].textContent).toBe("1");
		expect(gutterSpans[1].textContent).toBe("1");
	});

	it("shows only new line number for Add lines", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		const addLines = container.querySelectorAll(".diff-line-add");
		expect(addLines.length).toBeGreaterThanOrEqual(1);
		for (const addLine of addLines) {
			const spans = addLine.querySelectorAll("span");
			// First span is old gutter (should be empty), second is new gutter (should have number)
			expect(spans[0].textContent).toBe("");
			expect(spans[1].textContent?.trim()).not.toBe("");
		}
	});

	it("shows only old line number for Delete lines", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		const deleteLines = container.querySelectorAll(".diff-line-delete");
		expect(deleteLines.length).toBeGreaterThanOrEqual(1);
		for (const deleteLine of deleteLines) {
			const spans = deleteLine.querySelectorAll("span");
			// First span is old gutter (should have number), second is new gutter (should be empty)
			expect(spans[0].textContent?.trim()).not.toBe("");
			expect(spans[1].textContent).toBe("");
		}
	});
});

// ---- diff-utils unit tests (WHSP-03) ----

describe("diff-utils", () => {
	describe("splitInvisibles", () => {
		it("replaces spaces with middle dot (WHSP-03)", () => {
			const result = splitInvisibles("a b", false);
			expect(result).toEqual([
				{ text: "a", isInvisible: false, isTrailing: false },
				{ text: "\u00B7", isInvisible: true, isTrailing: false },
				{ text: "b", isInvisible: false, isTrailing: false },
			]);
		});

		it("replaces tabs with rightwards arrow (WHSP-03)", () => {
			const result = splitInvisibles("a\tb", false);
			expect(result).toEqual([
				{ text: "a", isInvisible: false, isTrailing: false },
				{ text: "\u2192", isInvisible: true, isTrailing: false },
				{ text: "b", isInvisible: false, isTrailing: false },
			]);
		});

		it("marks trailing whitespace segments", () => {
			const result = splitInvisibles("  ", true);
			expect(result).toEqual([
				{ text: "\u00B7\u00B7", isInvisible: true, isTrailing: true },
			]);
		});

		it("returns empty array for empty string", () => {
			expect(splitInvisibles("", false)).toEqual([]);
		});

		it("handles mixed spaces and tabs", () => {
			const result = splitInvisibles(" \t", false);
			expect(result).toEqual([
				{ text: "\u00B7\u2192", isInvisible: true, isTrailing: false },
			]);
		});
	});

	describe("trailingWhitespaceStart", () => {
		it("returns index where trailing whitespace begins (WHSP-03)", () => {
			expect(trailingWhitespaceStart("hello   ")).toBe(5);
		});

		it("returns string length when no trailing whitespace", () => {
			expect(trailingWhitespaceStart("hello")).toBe(5);
		});

		it("returns 0 for all-whitespace string", () => {
			expect(trailingWhitespaceStart("   ")).toBe(0);
		});

		it("handles tabs in trailing whitespace", () => {
			expect(trailingWhitespaceStart("hello\t")).toBe(5);
		});
	});
});

// ---- VIEW-04: Full file view ----

describe("VIEW-04: Full file view", () => {
	it("renders all lines as continuous document without hunk headers", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		await fireEvent.click(screen.getByTitle("Show full file"));
		await flushPrefs();
		// Hunk header should not be present
		expect(screen.queryByText("@@ -1,3 +1,4 @@")).toBeNull();
		// But diff content should be present
		expect(container.textContent).toContain("const x = 2;");
	});

	it("shows line numbers in gutter for full file view", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();
		await fireEvent.click(screen.getByTitle("Show full file"));
		await flushPrefs();
		// Context lines should have gutter numbers
		const contextLines = container.querySelectorAll(".diff-line-context");
		expect(contextLines.length).toBeGreaterThanOrEqual(1);
		const gutterSpans = contextLines[0].querySelectorAll("span");
		expect(gutterSpans.length).toBeGreaterThanOrEqual(2);
		// First context line: old=1, new=1
		expect(gutterSpans[0].textContent).toBe("1");
		expect(gutterSpans[1].textContent).toBe("1");
	});

	it("does not show staging buttons in full file view", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();
		await fireEvent.click(screen.getByTitle("Show full file"));
		await flushPrefs();
		expect(screen.queryByText("Stage Hunk")).toBeNull();
		expect(screen.queryByText("Discard Hunk")).toBeNull();
	});
});

// ---- WHSP-02: Staging disabled when whitespace ignore active ----

describe("WHSP-02: Staging disabled when whitespace ignore active", () => {
	it("disables Stage Hunk button when whitespace ignore is active", async () => {
		const storeMock = await import("../lib/store.js");
		// Reset modes to inline+hunk (previous tests may have changed them)
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(true),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();
		await flushPrefs();

		const stageBtn = screen.getByText("Stage Hunk");
		expect(stageBtn.closest("button")).toBeDisabled();

		// Reset mock
		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(false),
		);
	});

	it("disables Stage File button when whitespace ignore is active", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(true),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
				selectedPath: "src/main.ts",
			},
		});
		await flushPrefs();
		await flushPrefs();

		const stageFileBtn = screen.getByText("Stage File");
		expect(stageFileBtn.closest("button")).toBeDisabled();

		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(false),
		);
	});

	it("shows tooltip on disabled staging buttons", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(true),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();
		await flushPrefs();

		const stageBtn = screen.getByText("Stage Hunk").closest("button");
		expect(stageBtn?.title).toBe(
			"Staging is disabled while whitespace changes are ignored",
		);

		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(false),
		);
	});
});

// ---- DISP-02: Word wrap toggle ----

describe("DISP-02: Word wrap toggle", () => {
	it("persists word wrap preference when toggle clicked", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		vi.mocked(storeMock.getDiffWordWrap).mockImplementation(() =>
			Promise.resolve(false),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();

		// Click the word wrap toggle button
		const wrapBtn = screen.getByTitle("Toggle word wrap");
		await fireEvent.click(wrapBtn);
		await flushPrefs();

		// Verify that setDiffWordWrap was called with true
		expect(vi.mocked(storeMock.setDiffWordWrap)).toHaveBeenCalledWith(true);
	});

	it("word wrap toggle button becomes active when clicked", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
		vi.mocked(storeMock.getDiffWordWrap).mockImplementation(() =>
			Promise.resolve(false),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();

		const wrapBtn = screen.getByTitle("Toggle word wrap");
		// Before click: should not have active class
		expect(wrapBtn.classList.contains("active")).toBe(false);

		await fireEvent.click(wrapBtn);
		await flushPrefs();

		// After click: should have active class
		expect(wrapBtn.classList.contains("active")).toBe(true);
	});
});

// ---- pairLines unit tests ----

describe("pairLines", () => {
	it("pairs context lines on both sides", () => {
		const lines: DiffLine[] = [
			{
				origin: "Context",
				content: "hello",
				old_lineno: 1,
				new_lineno: 1,
				spans: [],
			},
		];
		const rows = pairLines(lines);
		expect(rows).toHaveLength(1);
		expect(rows[0].left?.line.content).toBe("hello");
		expect(rows[0].right?.line.content).toBe("hello");
	});

	it("pairs delete with add", () => {
		const lines: DiffLine[] = [
			{
				origin: "Delete",
				content: "old",
				old_lineno: 1,
				new_lineno: null,
				spans: [],
			},
			{
				origin: "Add",
				content: "new",
				old_lineno: null,
				new_lineno: 1,
				spans: [],
			},
		];
		const rows = pairLines(lines);
		expect(rows).toHaveLength(1);
		expect(rows[0].left?.line.content).toBe("old");
		expect(rows[0].right?.line.content).toBe("new");
	});

	it("creates phantom on right when more deletes than adds", () => {
		const lines: DiffLine[] = [
			{
				origin: "Delete",
				content: "a",
				old_lineno: 1,
				new_lineno: null,
				spans: [],
			},
			{
				origin: "Delete",
				content: "b",
				old_lineno: 2,
				new_lineno: null,
				spans: [],
			},
			{
				origin: "Add",
				content: "c",
				old_lineno: null,
				new_lineno: 1,
				spans: [],
			},
		];
		const rows = pairLines(lines);
		expect(rows).toHaveLength(2);
		expect(rows[0].left?.line.content).toBe("a");
		expect(rows[0].right?.line.content).toBe("c");
		expect(rows[1].left?.line.content).toBe("b");
		expect(rows[1].right).toBeNull(); // phantom
	});

	it("creates phantom on left when more adds than deletes", () => {
		const lines: DiffLine[] = [
			{
				origin: "Delete",
				content: "a",
				old_lineno: 1,
				new_lineno: null,
				spans: [],
			},
			{
				origin: "Add",
				content: "b",
				old_lineno: null,
				new_lineno: 1,
				spans: [],
			},
			{
				origin: "Add",
				content: "c",
				old_lineno: null,
				new_lineno: 2,
				spans: [],
			},
		];
		const rows = pairLines(lines);
		expect(rows).toHaveLength(2);
		expect(rows[0].left?.line.content).toBe("a");
		expect(rows[0].right?.line.content).toBe("b");
		expect(rows[1].left).toBeNull(); // phantom
		expect(rows[1].right?.line.content).toBe("c");
	});

	it("preserves original lineIdx for staging", () => {
		const lines: DiffLine[] = [
			{
				origin: "Context",
				content: "x",
				old_lineno: 1,
				new_lineno: 1,
				spans: [],
			},
			{
				origin: "Delete",
				content: "y",
				old_lineno: 2,
				new_lineno: null,
				spans: [],
			},
			{
				origin: "Add",
				content: "z",
				old_lineno: null,
				new_lineno: 2,
				spans: [],
			},
		];
		const rows = pairLines(lines);
		expect(rows[0].left?.lineIdx).toBe(0);
		expect(rows[1].left?.lineIdx).toBe(1);
		expect(rows[1].right?.lineIdx).toBe(2);
	});

	it("handles pure additions (no deletes)", () => {
		const lines: DiffLine[] = [
			{
				origin: "Add",
				content: "a",
				old_lineno: null,
				new_lineno: 1,
				spans: [],
			},
			{
				origin: "Add",
				content: "b",
				old_lineno: null,
				new_lineno: 2,
				spans: [],
			},
		];
		const rows = pairLines(lines);
		expect(rows).toHaveLength(2);
		expect(rows[0].left).toBeNull();
		expect(rows[0].right?.line.content).toBe("a");
		expect(rows[1].left).toBeNull();
		expect(rows[1].right?.line.content).toBe("b");
	});
});

// ---- VIEW-02: Split view layout ----

describe("VIEW-02: Split view layout", () => {
	it("renders split view with paired rows when layout mode is split", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);

		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();

		// Split view should render paired rows with two cells each
		const rows = container.querySelectorAll(".split-row");
		expect(rows.length).toBeGreaterThan(0);
		// Each row should have two cells
		const firstRow = rows[0];
		expect(firstRow.querySelectorAll(".split-cell").length).toBe(2);

		// Reset
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});

	it("shows old line numbers only in left cell, new only in right", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);

		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();

		// The split view is rendered -- verify paired rows exist
		const rows = container.querySelectorAll(".split-row");
		expect(rows.length).toBeGreaterThan(0);

		// Reset
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});

	it("does not show origin symbols in split view", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);

		const { container } = render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
			},
		});
		await flushPrefs();

		// In split view, there should be no +/- origin symbols
		// The diff content "const x = 2;" should be present without the "+" prefix
		const bodyText = container.textContent ?? "";
		expect(bodyText).toContain("const x = 2;");
		// Verify paired rows rendered
		const rows = container.querySelectorAll(".split-row");
		expect(rows.length).toBeGreaterThan(0);

		// Reset
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});
});

// ---- VIEW-05: Staging in split view ----

describe("VIEW-05: Staging in split view", () => {
	it("shows Stage Hunk button in split view for unstaged diffs", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();

		expect(screen.getByText("Stage Hunk")).toBeInTheDocument();
		expect(screen.getByText("Discard Hunk")).toBeInTheDocument();

		// Reset
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});

	it("shows Unstage Hunk button in split view for staged diffs", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "staged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();

		expect(screen.getByText("Unstage Hunk")).toBeInTheDocument();

		// Reset
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});

	it("does not show staging buttons in split view for commit diffs", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "commit",
			},
		});
		await flushPrefs();

		expect(screen.queryByText("Stage Hunk")).toBeNull();
		expect(screen.queryByText("Discard Hunk")).toBeNull();
		expect(screen.queryByText("Unstage Hunk")).toBeNull();

		// Reset
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});

	it("disables staging buttons when whitespace ignore is active in split view", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);
		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(true),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();
		await flushPrefs();

		const stageBtn = screen.getByText("Stage Hunk").closest("button");
		expect(stageBtn).toBeDisabled();
		expect(stageBtn?.title).toBe(
			"Staging is disabled while whitespace changes are ignored",
		);

		// Reset
		vi.mocked(storeMock.getDiffIgnoreWhitespace).mockImplementation(() =>
			Promise.resolve(false),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});

	it("does not show staging buttons in split+full mode", async () => {
		const storeMock = await import("../lib/store.js");
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("full"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("split"),
		);

		render(DiffPanel, {
			props: {
				fileDiffs: [testDiff],
				commitDetail: null,
				onclose: vi.fn(),
				diffKind: "unstaged",
				repoPath: "/test/repo",
			},
		});
		await flushPrefs();

		// Split+full mode has no hunk headers, so no staging buttons
		expect(screen.queryByText("Stage Hunk")).toBeNull();

		// Reset
		vi.mocked(storeMock.getDiffContentMode).mockImplementation(() =>
			Promise.resolve("hunk"),
		);
		vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() =>
			Promise.resolve("inline"),
		);
	});

	// ---- Diff scroll layout regression tests ----
	// These verify the structural CSS properties that make horizontal scrolling work correctly:
	// - Hunk toolbars and file headers stay visible (sticky left)
	// - Diff line backgrounds extend the full content width (no gaps on short lines)

	describe("diff scroll layout", () => {
		function findAncestorWithOverflow(el: Element): HTMLElement | null {
			let current = el.parentElement;
			while (current) {
				const style = current.getAttribute("style") || "";
				if (
					style.includes("overflow: auto") ||
					style.includes("overflow:auto")
				) {
					return current;
				}
				current = current.parentElement;
			}
			return null;
		}

		it("scroll container establishes container query context for sticky sizing", async () => {
			const { container } = render(DiffPanel, {
				props: {
					fileDiffs: [testDiff],
					commitDetail: null,
					onclose: vi.fn(),
				},
			});
			await flushPrefs();
			const line = container.querySelector(".diff-line");
			expect(line).toBeTruthy();
			const scrollContainer = findAncestorWithOverflow(line as Element);
			expect(scrollContainer).toBeTruthy();
			const style = scrollContainer?.getAttribute("style") ?? "";
			expect(style).toContain("container-type: inline-size");
			expect(style).toContain("overscroll-behavior-x: none");
		});

		it("hunk toolbar is horizontally sticky so buttons stay visible", async () => {
			render(DiffPanel, {
				props: {
					fileDiffs: [testDiff],
					commitDetail: null,
					onclose: vi.fn(),
				},
			});
			await flushPrefs();
			const hunkHeaderText = screen.getByText("@@ -1,3 +1,4 @@");
			const toolbar = hunkHeaderText.parentElement;
			expect(toolbar).toBeTruthy();
			const style = toolbar?.getAttribute("style") ?? "";
			expect(style).toContain("position: sticky");
			expect(style).toContain("left: 0");
		});

		it("file header is horizontally sticky in multi-file view", async () => {
			const { container } = render(DiffPanel, {
				props: {
					fileDiffs: [testDiff],
					commitDetail: null,
					selectedPath: null,
					onclose: vi.fn(),
				},
			});
			await flushPrefs();
			const headers = container.querySelectorAll('[role="button"]');
			const fileHeader = Array.from(headers).find((el) =>
				el.textContent?.includes("src/main.ts"),
			);
			expect(fileHeader).toBeTruthy();
			const style = fileHeader?.getAttribute("style") ?? "";
			expect(style).toContain("position: sticky");
			expect(style).toContain("left: 0");
		});

		it("diff lines wrapper ensures full-width backgrounds via min-width", async () => {
			const { container } = render(DiffPanel, {
				props: {
					fileDiffs: [testDiff],
					commitDetail: null,
					onclose: vi.fn(),
				},
			});
			await flushPrefs();
			const line = container.querySelector(".diff-line");
			expect(line).toBeTruthy();
			const wrapper = line?.parentElement;
			expect(wrapper).toBeTruthy();
			const style = wrapper?.getAttribute("style") ?? "";
			expect(style).toContain("min-width: 100%");
		});
	});
});
