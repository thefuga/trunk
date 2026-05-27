import { render, screen, waitFor } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { makeCommit } from "../__tests__/helpers/factories";
import { safeInvoke } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type { SessionStatus } from "../lib/types.js";
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

// Mock safeInvoke at the wrapper layer so tests can dispatch by command name and
// reject with TrunkError shapes for the WR-02 error branching tests.
vi.mock("../lib/invoke.js", async () => {
	const actual = await vi.importActual<typeof import("../lib/invoke.js")>(
		"../lib/invoke.js",
	);
	return {
		...actual,
		safeInvoke: vi.fn(),
	};
});

vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));

// Capture the session-changed handler so tests can simulate cross-tab emits.
// CommitGraph also registers a search-toggle listener; filter by event name.
let sessionChangedHandler: ((event: { payload: string }) => void) | null = null;
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn(
		(event: string, cb: (event: { payload: string }) => void) => {
			if (event === "session-changed") sessionChangedHandler = cb;
			return Promise.resolve(() => {
				if (event === "session-changed") sessionChangedHandler = null;
			});
		},
	),
}));

function fireSessionChanged(payload: string): void {
	sessionChangedHandler?.({ payload });
}

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

vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn().mockResolvedValue(undefined),
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

// Install the dispatcher. Reads route by command name; tests override individual
// commands via `extra` (called BEFORE this installer to layer rejections).
type DispatchOverride = (cmd: string) => unknown | undefined;
function installReads(opts: {
	commits?: typeof TEST_COMMITS;
	status?: SessionStatus | null;
	sessionCommits?: { oid: string }[];
	override?: DispatchOverride;
} = {}) {
	const status = opts.status;
	const commits = opts.commits ?? TEST_COMMITS;
	const sessionCommits = opts.sessionCommits ?? [];
	vi.mocked(safeInvoke).mockReset();
	vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
		const overridden = opts.override?.(cmd);
		if (overridden !== undefined) return overridden as Promise<unknown>;
		switch (cmd) {
			case "get_commit_graph":
				return Promise.resolve({ commits, max_columns: 1 });
			case "refresh_commit_graph":
				return Promise.resolve({ commits, max_columns: 1 });
			case "list_stashes":
				return Promise.resolve([]);
			case "get_review_session_status":
				return status === null
					? Promise.reject({ code: "no_session", message: "no session" })
					: Promise.resolve(
							status ?? {
								state: "none",
								file_exists: false,
								canonical_path: "/repo",
							},
						);
			case "list_session_commits":
				return Promise.resolve(sessionCommits);
			default:
				return Promise.resolve(undefined);
		}
	});
}

async function flush() {
	await new Promise((r) => setTimeout(r, 0));
	await tick();
}

beforeEach(() => {
	sessionChangedHandler = null;
	vi.clearAllMocks();
	installReads();
});

describe("CommitGraph", () => {
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
		await waitFor(() => {
			expect(vi.mocked(safeInvoke)).toHaveBeenCalledWith("get_commit_graph", {
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
			expect(vi.mocked(safeInvoke)).toHaveBeenCalledWith("list_stashes", {
				path: "/test/repo",
			});
		});
	});

	describe("session-changed listener (66/WR-01)", () => {
		it("fails closed when canonicalPath is null — cross-repo event does not trigger reload", async () => {
			// status reject => sessionStatus stays null AND canonicalPath stays null.
			// Today the bug: `if (sessionStatus && …) return` short-circuits to falsy,
			// so the guard never triggers, and every cross-repo event triggers a reload.
			// After the fix: `canonicalPath && …` is still null/falsy, but the listener
			// uses canonicalPath (fail-closed because canonicalPath null means "we don't
			// know yet — drop everything"). Plan §1 reverses the polarity by requiring
			// the listener to gate on canonicalPath being known AND matching.
			installReads({ status: null });
			render(CommitGraph, {
				props: { repoPath: "/this/repo", clearRedoStack: vi.fn() },
			});
			await flush();

			const callsBefore = vi
				.mocked(safeInvoke)
				.mock.calls.filter(
					(c) =>
						c[0] === "get_review_session_status" ||
						c[0] === "list_session_commits",
				).length;

			fireSessionChanged("/some/other/repo");
			await flush();

			const callsAfter = vi
				.mocked(safeInvoke)
				.mock.calls.filter(
					(c) =>
						c[0] === "get_review_session_status" ||
						c[0] === "list_session_commits",
				).length;

			// After fix: cross-repo event with null canonicalPath must NOT reload.
			expect(callsAfter).toBe(callsBefore);
		});

		it("filters events by canonical_path once known — own-repo reloads, other-repo does not", async () => {
			installReads({
				status: {
					state: "active",
					file_exists: true,
					canonical_path: "/this/repo",
				},
			});
			render(CommitGraph, {
				props: { repoPath: "/this/repo", clearRedoStack: vi.fn() },
			});
			await flush();

			const callsBefore = vi
				.mocked(safeInvoke)
				.mock.calls.filter((c) => c[0] === "get_review_session_status").length;

			fireSessionChanged("/other/repo");
			await flush();
			expect(
				vi
					.mocked(safeInvoke)
					.mock.calls.filter((c) => c[0] === "get_review_session_status")
					.length,
			).toBe(callsBefore);

			fireSessionChanged("/this/repo");
			await flush();
			expect(
				vi
					.mocked(safeInvoke)
					.mock.calls.filter((c) => c[0] === "get_review_session_status")
					.length,
			).toBeGreaterThan(callsBefore);
		});
	});

	describe("reloadSession error branching (66/WR-02)", () => {
		it("silently empties state on no_session — no toast", async () => {
			installReads({
				override: (cmd) =>
					cmd === "get_review_session_status"
						? Promise.reject({ code: "no_session", message: "no session" })
						: undefined,
			});
			render(CommitGraph, {
				props: { repoPath: "/this/repo", clearRedoStack: vi.fn() },
			});
			await flush();
			expect(vi.mocked(showToast)).not.toHaveBeenCalled();
		});

		it("silently empties state on not_open — no toast", async () => {
			installReads({
				override: (cmd) =>
					cmd === "get_review_session_status"
						? Promise.reject({ code: "not_open", message: "not open" })
						: undefined,
			});
			render(CommitGraph, {
				props: { repoPath: "/this/repo", clearRedoStack: vi.fn() },
			});
			await flush();
			expect(vi.mocked(showToast)).not.toHaveBeenCalled();
		});

		it("surfaces an error toast on unexpected backend failure", async () => {
			installReads({
				override: (cmd) =>
					cmd === "get_review_session_status"
						? Promise.reject({ code: "internal", message: "boom" })
						: undefined,
			});
			render(CommitGraph, {
				props: { repoPath: "/this/repo", clearRedoStack: vi.fn() },
			});
			await flush();
			expect(vi.mocked(showToast)).toHaveBeenCalledWith(
				expect.stringMatching(/review/i),
				"error",
			);
		});
	});
});
