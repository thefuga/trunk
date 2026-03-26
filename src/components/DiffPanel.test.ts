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
				},
				{
					origin: "Delete",
					content: "const x = 1;",
					old_lineno: 2,
					new_lineno: null,
				},
				{
					origin: "Add",
					content: "const x = 2;",
					old_lineno: null,
					new_lineno: 2,
				},
				{
					origin: "Add",
					content: "const y = 3;",
					old_lineno: null,
					new_lineno: 3,
				},
				{
					origin: "Context",
					content: "export { x };",
					old_lineno: 3,
					new_lineno: 4,
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
});
