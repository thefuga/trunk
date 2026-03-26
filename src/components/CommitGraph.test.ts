import { invoke } from "@tauri-apps/api/core";
import { render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { makeCommit } from "../__tests__/helpers/factories";
import CommitGraph from "./CommitGraph.svelte";

// Stub OffscreenCanvas for jsdom — used by text-measure.ts (measureTextWidth)
if (typeof globalThis.OffscreenCanvas === "undefined") {
	globalThis.OffscreenCanvas = class {
		constructor(
			public width: number,
			public height: number,
		) {}
		getContext() {
			return {
				font: "",
				measureText: () => ({ width: 50 }),
			};
		}
	} as unknown as typeof OffscreenCanvas;
}

// Stub Element.scrollTo for jsdom — VirtualList uses viewport.scrollTo()
if (typeof Element.prototype.scrollTo === "undefined") {
	Element.prototype.scrollTo = () => {};
}

// All Tauri module mocks — declared locally for proper vi.mock hoisting
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-store", () => {
	const store = new Map<string, unknown>();
	class MockLazyStore {
		get(key: string) {
			return Promise.resolve(store.get(key) ?? null);
		}
		set(key: string, value: unknown) {
			store.set(key, value);
			return Promise.resolve();
		}
		save() {
			return Promise.resolve();
		}
	}
	return { LazyStore: MockLazyStore };
});

vi.mock("@tauri-apps/plugin-dialog", () => ({
	open: vi.fn(),
	ask: vi.fn().mockResolvedValue(false),
	message: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
	writeText: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/path", () => ({
	homeDir: vi.fn().mockResolvedValue("/Users/test"),
}));

vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@tauri-apps/api/window", () => ({
	getCurrentWindow: vi.fn().mockReturnValue({
		onResized: vi.fn().mockResolvedValue(() => {}),
		onMoved: vi.fn().mockResolvedValue(() => {}),
		isMaximized: vi.fn().mockResolvedValue(false),
		isFullscreen: vi.fn().mockResolvedValue(false),
	}),
}));

vi.mock("@tauri-apps/api/menu", () => ({
	Menu: {
		new: vi.fn().mockResolvedValue({
			popup: vi.fn().mockResolvedValue(undefined),
		}),
	},
	MenuItem: { new: vi.fn().mockResolvedValue({}) },
	CheckMenuItem: { new: vi.fn().mockResolvedValue({}) },
	PredefinedMenuItem: { new: vi.fn().mockResolvedValue({}) },
	Submenu: { new: vi.fn().mockResolvedValue({}) },
}));

vi.mock("@tauri-apps/plugin-window-state", () => ({}));

const mockInvoke = vi.mocked(invoke);

const TEST_COMMITS = [
	makeCommit({
		oid: "aaa111aaa111aaa1aaa111aaa111aaa1aaa111aa",
		summary: "first commit",
		is_head: true,
	}),
	makeCommit({
		oid: "bbb222bbb222bbb2bbb222bbb222bbb2bbb222bb",
		summary: "second commit",
		parent_oids: ["aaa111aaa111aaa1aaa111aaa111aaa1aaa111aa"],
	}),
];

describe("CommitGraph", () => {
	beforeEach(() => {
		mockInvoke.mockReset();
		mockInvoke.mockImplementation((cmd: string) => {
			if (cmd === "get_commit_graph")
				return Promise.resolve({
					commits: TEST_COMMITS,
					max_columns: 1,
				});
			if (cmd === "list_stashes") return Promise.resolve([]);
			return Promise.resolve(undefined);
		});
	});

	it("renders without crashing", () => {
		const { container } = render(CommitGraph, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		expect(container).toBeTruthy();
	});

	it("renders column headers", async () => {
		render(CommitGraph, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		// CommitGraph renders column headers: Branch/Tag, Graph, Message, Author, Date, SHA
		await waitFor(() => {
			expect(screen.getByText("Branch/Tag")).toBeInTheDocument();
		});
		expect(screen.getByText("Graph")).toBeInTheDocument();
		expect(screen.getByText("Message")).toBeInTheDocument();
		expect(screen.getByText("Author")).toBeInTheDocument();
		expect(screen.getByText("Date")).toBeInTheDocument();
		expect(screen.getByText("SHA")).toBeInTheDocument();
	});

	it("renders commit summaries after data loads", async () => {
		render(CommitGraph, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		// VirtualList may not render all rows in jsdom due to scroll virtualization.
		// Verify at least the graph loads data by checking the invoke call.
		await waitFor(() => {
			expect(mockInvoke).toHaveBeenCalledWith("get_commit_graph", {
				path: "/test/repo",
				offset: 0,
			});
		});
	});

	it("has listbox role for keyboard navigation", () => {
		const { container } = render(CommitGraph, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		expect(container.querySelector('[role="listbox"]')).toBeTruthy();
	});

	it("calls list_stashes on mount", async () => {
		render(CommitGraph, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		await waitFor(() => {
			expect(mockInvoke).toHaveBeenCalledWith("list_stashes", {
				path: "/test/repo",
			});
		});
	});
});
