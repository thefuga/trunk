import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { safeInvoke } from "../lib/invoke.js";
import type {
	Comment,
	CommentResolution,
	SessionCommit,
} from "../lib/types.js";
import ReviewPanel from "./ReviewPanel.svelte";

// Shared Tauri mock (provides @tauri-apps/plugin-dialog `ask` defaulting to false,
// @tauri-apps/api/event `listen`, etc.).
import "../__tests__/helpers/tauri-mock";

// Command-aware safeInvoke dispatcher: the panel issues three reads on mount
// (list_session_commits / list_session_comments / resolve_session_comments) via
// Promise.all, so a sequential mock would be fragile — route by command name.
vi.mock("../lib/invoke.js", () => ({
	safeInvoke: vi.fn(),
}));

vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));

const COMMIT_A = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const COMMIT_B = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

const commits: SessionCommit[] = [
	{ oid: COMMIT_A, short_oid: "aaaaaaa", summary: "first commit" },
	{ oid: COMMIT_B, short_oid: "bbbbbbb", summary: "second commit" },
];

function lineAnchoredComment(
	id: string,
	commitOid: string,
	text: string,
): Comment {
	return {
		id,
		text,
		anchor: {
			commit_oid: commitOid,
			file_path: "src/main.ts",
			source: "Diff",
			side: "New",
			start_line: 10,
			end_line: 12,
		},
		cached_excerpt: "const x = 1;",
		commit_oid: null,
	};
}

function commitLevelComment(
	id: string,
	commitOid: string,
	text: string,
): Comment {
	return {
		id,
		text,
		anchor: null,
		cached_excerpt: null,
		commit_oid: commitOid,
	};
}

function resolvable(id: string): CommentResolution {
	return { id, resolvable: true, reason: null };
}

function orphan(
	id: string,
	reason: CommentResolution["reason"],
): CommentResolution {
	return { id, resolvable: false, reason };
}

// Install the dispatcher: reads return the supplied fixtures; writes resolve
// undefined. Per-test overrides extend `extra`.
function installReads(opts: {
	commits?: SessionCommit[];
	comments?: Comment[];
	resolutions?: CommentResolution[];
}) {
	vi.mocked(safeInvoke).mockReset();
	vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
		switch (cmd) {
			case "list_session_commits":
				return Promise.resolve(opts.commits ?? []);
			case "list_session_comments":
				return Promise.resolve(opts.comments ?? []);
			case "resolve_session_comments":
				return Promise.resolve(opts.resolutions ?? []);
			default:
				return Promise.resolve(undefined);
		}
	});
}

async function flush() {
	await new Promise((r) => setTimeout(r, 0));
	await tick();
}

function calledCommands(): string[] {
	return vi.mocked(safeInvoke).mock.calls.map((c) => c[0] as string);
}

function callArgs(cmd: string): Record<string, unknown> | undefined {
	const call = vi.mocked(safeInvoke).mock.calls.find((c) => c[0] === cmd);
	return call?.[1] as Record<string, unknown> | undefined;
}

beforeEach(() => {
	vi.clearAllMocks();
});

describe("ReviewPanel", () => {
	it("groups comments under their commit headers", async () => {
		installReads({
			commits,
			comments: [
				lineAnchoredComment("c1", COMMIT_A, "note on A"),
				commitLevelComment("c2", COMMIT_B, "note on B"),
			],
			resolutions: [resolvable("c1"), resolvable("c2")],
		});
		render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
		await flush();

		// Group headers: short SHA of each commit present.
		expect(screen.getByText("aaaaaaa")).toBeInTheDocument();
		expect(screen.getByText("bbbbbbb")).toBeInTheDocument();
		// Comments nested under their commit.
		expect(screen.getByText("note on A")).toBeInTheDocument();
		expect(screen.getByText("note on B")).toBeInTheDocument();
	});

	it("reads the three session sources on mount", async () => {
		installReads({ commits, comments: [], resolutions: [] });
		render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
		await flush();
		const cmds = calledCommands();
		expect(cmds).toContain("list_session_commits");
		expect(cmds).toContain("list_session_comments");
		expect(cmds).toContain("resolve_session_comments");
	});

	it("shows the no-comments empty state when commits exist but no comments", async () => {
		installReads({ commits, comments: [], resolutions: [] });
		render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
		await flush();
		expect(screen.getByText("No comments yet.")).toBeInTheDocument();
	});

	it("shows the no-commits empty state when the session has no commits", async () => {
		installReads({ commits: [], comments: [], resolutions: [] });
		render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
		await flush();
		expect(
			screen.getByText("No commits in this review yet."),
		).toBeInTheDocument();
	});

	describe("add note", () => {
		it("writes a commit-level comment via add_commit_comment on Save", async () => {
			installReads({ commits, comments: [], resolutions: [] });
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			// Open the inline composer for commit A.
			const addBtns = screen.getAllByText("Add note");
			await fireEvent.click(addBtns[0]);
			await tick();

			const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
			await fireEvent.input(textarea, { target: { value: "a fresh note" } });
			await tick();

			await fireEvent.click(screen.getByText("Save"));
			await flush();

			expect(calledCommands()).toContain("add_commit_comment");
			const args = callArgs("add_commit_comment");
			expect(args?.commitOid).toBe(COMMIT_A);
			expect(args?.text).toBe("a fresh note");
		});

		it("disables Save while the add-note textarea is empty/whitespace", async () => {
			installReads({ commits, comments: [], resolutions: [] });
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			const addBtns = screen.getAllByText("Add note");
			await fireEvent.click(addBtns[0]);
			await tick();

			const saveBtn = screen.getByText("Save").closest("button");
			expect(saveBtn).toBeDisabled();

			const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
			await fireEvent.input(textarea, { target: { value: "   " } });
			await tick();
			expect(saveBtn).toBeDisabled();

			await fireEvent.input(textarea, { target: { value: "real" } });
			await tick();
			expect(saveBtn).not.toBeDisabled();
		});
	});

	describe("inline edit", () => {
		it("invokes edit_comment with the id and new text on Save", async () => {
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "original")],
				resolutions: [resolvable("c1")],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			await fireEvent.click(screen.getByText("Edit"));
			await tick();

			const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
			expect(textarea.value).toBe("original");
			await fireEvent.input(textarea, { target: { value: "edited text" } });
			await tick();

			await fireEvent.click(screen.getByText("Save"));
			await flush();

			expect(calledCommands()).toContain("edit_comment");
			const args = callArgs("edit_comment");
			expect(args?.id).toBe("c1");
			expect(args?.text).toBe("edited text");
		});

		it("disables Save when the edit textarea is empty/whitespace", async () => {
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "original")],
				resolutions: [resolvable("c1")],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			await fireEvent.click(screen.getByText("Edit"));
			await tick();

			const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
			await fireEvent.input(textarea, { target: { value: "  " } });
			await tick();

			expect(screen.getByText("Save").closest("button")).toBeDisabled();
		});

		it("Cancel closes the editor without invoking edit_comment", async () => {
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "original")],
				resolutions: [resolvable("c1")],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			await fireEvent.click(screen.getByText("Edit"));
			await tick();
			await fireEvent.click(screen.getByText("Cancel"));
			await flush();

			expect(calledCommands()).not.toContain("edit_comment");
			expect(screen.getByText("original")).toBeInTheDocument();
		});
	});

	describe("delete", () => {
		it("does not invoke delete_comment when the confirm is cancelled", async () => {
			const { ask } = await import("@tauri-apps/plugin-dialog");
			vi.mocked(ask).mockResolvedValue(false);
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "doomed")],
				resolutions: [resolvable("c1")],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			await fireEvent.click(screen.getByText("Delete"));
			await flush();

			expect(vi.mocked(ask)).toHaveBeenCalledTimes(1);
			expect(calledCommands()).not.toContain("delete_comment");
		});

		it("invokes delete_comment by id when the confirm is accepted", async () => {
			const { ask } = await import("@tauri-apps/plugin-dialog");
			vi.mocked(ask).mockResolvedValue(true);
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "doomed")],
				resolutions: [resolvable("c1")],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			await fireEvent.click(screen.getByText("Delete"));
			await flush();

			expect(calledCommands()).toContain("delete_comment");
			expect(callArgs("delete_comment")?.id).toBe("c1");
		});
	});

	describe("jump vs orphan", () => {
		it("calls onJump for a resolvable line-anchored comment", async () => {
			const onJump = vi.fn();
			const comment = lineAnchoredComment("c1", COMMIT_A, "jump me");
			installReads({
				commits,
				comments: [comment],
				resolutions: [resolvable("c1")],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump } });
			await flush();

			await fireEvent.click(screen.getByLabelText("Jump to code"));
			await flush();

			expect(onJump).toHaveBeenCalledTimes(1);
			expect(onJump.mock.calls[0][0].id).toBe("c1");
		});

		it("renders an orphaned comment read-only with a reason badge and no jump", async () => {
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "stale note")],
				resolutions: [orphan("c1", "FileGone")],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			// Reason badge with the LOCKED label.
			expect(screen.getByText("file gone")).toBeInTheDocument();
			// Jump affordance is gone (or disabled) for an orphan.
			expect(screen.queryByLabelText("Jump to code")).toBeNull();
			// The comment text + excerpt remain visible.
			expect(screen.getByText("stale note")).toBeInTheDocument();
		});

		it("maps each OrphanReason to its locked badge label", async () => {
			installReads({
				commits,
				comments: [
					lineAnchoredComment("c1", COMMIT_A, "a"),
					lineAnchoredComment("c2", COMMIT_A, "b"),
					commitLevelComment("c3", COMMIT_B, "c"),
				],
				resolutions: [
					orphan("c1", "CommitGone"),
					orphan("c2", "LineOutOfRange"),
					orphan("c3", "FileGone"),
				],
			});
			render(ReviewPanel, { props: { repoPath: "/repo", onJump: vi.fn() } });
			await flush();

			expect(screen.getByText("commit gone")).toBeInTheDocument();
			expect(screen.getByText("line out of range")).toBeInTheDocument();
			expect(screen.getByText("file gone")).toBeInTheDocument();
		});
	});
});
