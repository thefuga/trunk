import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import Toolbar from "./Toolbar.svelte";

// All Tauri module mocks — declared locally (NOT via ../__tests__/helpers/tauri-mock)
// for proper vi.mock hoisting before Toolbar.svelte's static imports resolve.
// The new Review-button tests assert on `emit` identity, which requires the
// mocked event module to be the SAME instance Toolbar.svelte sees at import time
// — a guarantee the shared helper cannot provide (its vi.mock runs at the helper
// file's import time, AFTER Toolbar.svelte has already resolved its imports).
//
// Includes:
//   listen: vi.fn().mockResolvedValue(() => {})
//   emit: vi.fn().mockResolvedValue(undefined)
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
	emit: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
	open: vi.fn(),
	ask: vi.fn().mockResolvedValue(false),
	message: vi.fn().mockResolvedValue(undefined),
}));

// Mock invoke module — safeInvoke for check_undo_available etc.
vi.mock("../lib/invoke.js", () => ({
	safeInvoke: vi.fn().mockResolvedValue(false),
}));

// Mock toast module
vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));

function makeRemoteState() {
	return {
		isRunning: false,
		progressLine: "",
		error: null,
	};
}

function makeUndoRedo() {
	return {
		state: { redoStack: [] as Array<{ subject: string; body: string | null }> },
		push: vi.fn(),
		pop: vi.fn(),
		clear: vi.fn(),
	};
}

describe("Toolbar", () => {
	it("renders Pull button", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: false,
			},
		});
		expect(screen.getByText("Pull")).toBeInTheDocument();
	});

	it("renders Push button", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: false,
			},
		});
		expect(screen.getByText("Push")).toBeInTheDocument();
	});

	it("renders Branch button", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: false,
			},
		});
		expect(screen.getByText("Branch")).toBeInTheDocument();
	});

	it("renders Stash and Pop buttons", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: false,
			},
		});
		expect(screen.getByText("Stash")).toBeInTheDocument();
		expect(screen.getByText("Pop")).toBeInTheDocument();
	});

	it("renders Undo and Redo buttons", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: false,
			},
		});
		expect(screen.getByText("Undo")).toBeInTheDocument();
		expect(screen.getByText("Redo")).toBeInTheDocument();
	});

	it("disables Pull and Push when remote operation is running", () => {
		const remoteState = makeRemoteState();
		remoteState.isRunning = true;

		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState,
				undoRedo: makeUndoRedo(),
				reviewActive: false,
			},
		});

		const pullBtn = screen.getByText("Pull").closest("button");
		const pushBtn = screen.getByText("Push").closest("button");
		expect(pullBtn).toBeDisabled();
		expect(pushBtn).toBeDisabled();
	});

	it("disables Redo when redo stack is empty", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(), // empty redoStack
				reviewActive: false,
			},
		});

		const redoBtn = screen.getByText("Redo").closest("button");
		expect(redoBtn).toBeDisabled();
	});

	it("emits review-toggle on click", async () => {
		const { emit } = await import("@tauri-apps/api/event");
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: false,
			},
		});
		const reviewBtn = screen.getByRole("button", { name: /Review/ });
		await fireEvent.click(reviewBtn);
		expect(vi.mocked(emit)).toHaveBeenCalledWith("review-toggle");
	});

	it("shows active state when reviewActive is true", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: true,
			},
		});
		const btn = screen.getByRole("button", { name: /Review/ });
		expect(btn).toHaveClass("toolbar-btn-active");
		expect(btn).toHaveAttribute("aria-pressed", "true");
	});

	it("shows inactive state when a diff is showing inside an active review", () => {
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: true,
				reviewPanelShowing: false,
			},
		});
		const btn = screen.getByRole("button", { name: /Review/ });
		expect(btn).not.toHaveClass("toolbar-btn-active");
		expect(btn).toHaveAttribute("aria-pressed", "false");
	});

	it("emits review-show-panel when clicked while a diff is showing in review", async () => {
		const { emit } = await import("@tauri-apps/api/event");
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: true,
				reviewPanelShowing: false,
			},
		});
		const reviewBtn = screen.getByRole("button", { name: /Review/ });
		await fireEvent.click(reviewBtn);
		expect(vi.mocked(emit)).toHaveBeenCalledWith("review-show-panel");
	});

	it("emits review-toggle when clicked while the review panel is showing", async () => {
		const { emit } = await import("@tauri-apps/api/event");
		render(Toolbar, {
			props: {
				repoPath: "/test/repo",
				remoteState: makeRemoteState(),
				undoRedo: makeUndoRedo(),
				reviewActive: true,
				reviewPanelShowing: true,
			},
		});
		const reviewBtn = screen.getByRole("button", { name: /Review/ });
		await fireEvent.click(reviewBtn);
		expect(vi.mocked(emit)).toHaveBeenCalledWith("review-toggle");
	});
});
