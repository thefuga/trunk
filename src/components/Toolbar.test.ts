import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import Toolbar from "./Toolbar.svelte";

// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

// Explicitly mock @tauri-apps/api/event to prevent real listen calls
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
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
			},
		});

		const redoBtn = screen.getByText("Redo").closest("button");
		expect(redoBtn).toBeDisabled();
	});
});
