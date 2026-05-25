import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SessionState, SessionStatus } from "../lib/types";
import ReviewPanel from "./ReviewPanel.svelte";

vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
}));

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

// Captured session-changed handlers, so a test can fire the event by hand.
let sessionChangedHandlers: Array<(event: { payload: string }) => void> = [];

function makeStatus(state: SessionState): SessionStatus {
	return {
		state,
		file_exists: state !== "none",
		canonical_path: "/canonical/repo",
	};
}

function setStatus(state: SessionState) {
	mockInvoke.mockImplementation((cmd: string) => {
		if (cmd === "get_review_session_status") {
			return Promise.resolve(makeStatus(state));
		}
		return Promise.resolve(undefined);
	});
}

describe("ReviewPanel", () => {
	beforeEach(() => {
		mockInvoke.mockReset();
		mockListen.mockReset();
		sessionChangedHandlers = [];
		// biome-ignore lint/suspicious/noExplicitAny: test-only handler capture
		mockListen.mockImplementation((event: string, handler: any) => {
			if (event === "session-changed") sessionChangedHandlers.push(handler);
			return Promise.resolve(() => {});
		});
		setStatus("none");
	});

	it("renders a Start button when there is no session", async () => {
		setStatus("none");
		render(ReviewPanel, { props: { repoPath: "/test/repo" } });
		expect(await screen.findByText("Start Code Review")).toBeInTheDocument();
	});

	it("renders Resume and Discard when a session is resume-available", async () => {
		setStatus("resume-available");
		render(ReviewPanel, { props: { repoPath: "/test/repo" } });
		expect(await screen.findByText("Resume")).toBeInTheDocument();
		expect(screen.getByText("Discard")).toBeInTheDocument();
	});

	it("renders an empty session view with an End button when active", async () => {
		setStatus("active");
		render(ReviewPanel, { props: { repoPath: "/test/repo" } });
		expect(await screen.findByText("End Review")).toBeInTheDocument();
		expect(screen.getByText("No comments yet")).toBeInTheDocument();
	});

	it("invokes start_review_session with the repo path when Start is clicked", async () => {
		setStatus("none");
		render(ReviewPanel, { props: { repoPath: "/test/repo" } });
		const startButton = await screen.findByText("Start Code Review");
		startButton.click();
		await waitFor(() => {
			expect(mockInvoke).toHaveBeenCalledWith("start_review_session", {
				path: "/test/repo",
			});
		});
	});

	it("re-fetches status when a matching session-changed event arrives", async () => {
		setStatus("none");
		render(ReviewPanel, { props: { repoPath: "/test/repo" } });
		await screen.findByText("Start Code Review");

		const statusCallsBefore = mockInvoke.mock.calls.filter(
			(c) => c[0] === "get_review_session_status",
		).length;

		setStatus("active");
		for (const handler of sessionChangedHandlers) {
			handler({ payload: "/canonical/repo" });
		}

		await waitFor(() => {
			const statusCallsAfter = mockInvoke.mock.calls.filter(
				(c) => c[0] === "get_review_session_status",
			).length;
			expect(statusCallsAfter).toBeGreaterThan(statusCallsBefore);
		});
		expect(await screen.findByText("End Review")).toBeInTheDocument();
	});
});
