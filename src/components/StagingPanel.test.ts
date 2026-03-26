import { render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import StagingPanel from "./StagingPanel.svelte";

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

describe("StagingPanel", () => {
	beforeEach(() => {
		mockInvoke.mockReset();
		mockInvoke.mockImplementation((cmd: string) => {
			if (cmd === "get_status")
				return Promise.resolve({
					unstaged: [
						{ path: "README.md", status: "Modified", is_binary: false },
					],
					staged: [
						{ path: "src/main.ts", status: "New", is_binary: false },
					],
					conflicted: [],
				});
			if (cmd === "get_operation_state")
				return Promise.resolve({
					op_type: "None",
					source_branch: null,
					target_branch: null,
					progress: null,
					source_color_index: null,
					target_color_index: null,
					rebase_message: null,
				});
			return Promise.resolve(undefined);
		});
	});

	it("renders without crashing", () => {
		const { container } = render(StagingPanel, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		expect(container).toBeTruthy();
	});

	it("renders file count header", async () => {
		render(StagingPanel, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		// Header shows "{totalCount} file(s) changed" — 1 unstaged + 1 staged = 2
		await waitFor(() => {
			expect(screen.getByText("2 files changed")).toBeInTheDocument();
		});
	});

	it("renders unstaged files section header", async () => {
		render(StagingPanel, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		// Section header: "Unstaged Files (1)"
		await waitFor(() => {
			expect(screen.getByText("Unstaged Files (1)")).toBeInTheDocument();
		});
	});

	it("renders staged files section header", async () => {
		render(StagingPanel, {
			props: {
				repoPath: "/test/repo",
				clearRedoStack: vi.fn(),
			},
		});
		// Section header: "Staged Files (1)"
		await waitFor(() => {
			expect(screen.getByText("Staged Files (1)")).toBeInTheDocument();
		});
	});

	it("renders current branch name when provided", async () => {
		render(StagingPanel, {
			props: {
				repoPath: "/test/repo",
				currentBranch: "feature/test",
				clearRedoStack: vi.fn(),
			},
		});
		await waitFor(() => {
			expect(screen.getByText("feature/test")).toBeInTheDocument();
		});
	});

	it("calls get_status on mount with repo path", async () => {
		render(StagingPanel, {
			props: {
				repoPath: "/my/repo",
				clearRedoStack: vi.fn(),
			},
		});
		await waitFor(() => {
			expect(mockInvoke).toHaveBeenCalledWith("get_status", {
				path: "/my/repo",
			});
		});
	});
});
