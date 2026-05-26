import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { safeInvoke } from "../lib/invoke.js";
import { createReviewSession } from "../lib/review-session.svelte.js";
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

// The panel registers a session-changed listener in an $effect; mock listen so
// it doesn't reach the real Tauri IPC core (which is undefined under jsdom).
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn().mockResolvedValue(() => {}),
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
// undefined. Per-test overrides extend `extra`. `generateDoc` routes the Phase
// 70 generate_review_doc IPC to a fixture string (the safeInvoke wrapper
// returns it; the panel's rune-call then drives the preview swap).
function installReads(opts: {
	commits?: SessionCommit[];
	comments?: Comment[];
	resolutions?: CommentResolution[];
	generateDoc?: string;
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
			case "generate_review_doc":
				return Promise.resolve(opts.generateDoc ?? "# stub\n");
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
		render(ReviewPanel, {
			props: {
				repoPath: "/repo",
				session: createReviewSession(),
				onJump: vi.fn(),
				onJumpToCommit: vi.fn(),
			},
		});
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
		render(ReviewPanel, {
			props: {
				repoPath: "/repo",
				session: createReviewSession(),
				onJump: vi.fn(),
				onJumpToCommit: vi.fn(),
			},
		});
		await flush();
		const cmds = calledCommands();
		expect(cmds).toContain("list_session_commits");
		expect(cmds).toContain("list_session_comments");
		expect(cmds).toContain("resolve_session_comments");
	});

	it("shows the no-comments empty state when commits exist but no comments", async () => {
		installReads({ commits, comments: [], resolutions: [] });
		render(ReviewPanel, {
			props: {
				repoPath: "/repo",
				session: createReviewSession(),
				onJump: vi.fn(),
				onJumpToCommit: vi.fn(),
			},
		});
		await flush();
		expect(screen.getByText("No comments yet.")).toBeInTheDocument();
	});

	it("shows the no-commits empty state when the session has no commits", async () => {
		installReads({ commits: [], comments: [], resolutions: [] });
		render(ReviewPanel, {
			props: {
				repoPath: "/repo",
				session: createReviewSession(),
				onJump: vi.fn(),
				onJumpToCommit: vi.fn(),
			},
		});
		await flush();
		expect(
			screen.getByText("No commits in this review yet."),
		).toBeInTheDocument();
	});

	// Regression: a comment whose anchor.commit_oid is not in session.commits
	// (e.g. user commented from a diff without marking the commit "in review"
	// via the graph) must still render in a fallback group — the resolver, not
	// the session list, is the truth about whether the commit is gone.
	it("renders fallback group for comments whose commit isn't in session.commits", async () => {
		installReads({
			commits: [],
			comments: [lineAnchoredComment("c1", COMMIT_A, "i need eyes on this")],
			resolutions: [resolvable("c1")],
		});
		render(ReviewPanel, {
			props: {
				repoPath: "/repo",
				session: createReviewSession(),
				onJump: vi.fn(),
				onJumpToCommit: vi.fn(),
			},
		});
		await flush();
		expect(screen.getByText("i need eyes on this")).toBeInTheDocument();
		// Fallback header uses the short oid; no synthetic "(commit gone)" label
		// when the resolver says the comment is resolvable.
		expect(screen.getByText("aaaaaaa")).toBeInTheDocument();
		expect(screen.queryByText("(commit gone)")).not.toBeInTheDocument();
		// The no-commits empty state must NOT fire when comments exist.
		expect(
			screen.queryByText("No commits in this review yet."),
		).not.toBeInTheDocument();
	});

	describe("add note", () => {
		it("writes a commit-level comment via add_commit_comment on Save", async () => {
			installReads({ commits, comments: [], resolutions: [] });
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump,
					onJumpToCommit: vi.fn(),
				},
			});
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
			await flush();

			// Reason badge with the LOCKED label.
			expect(screen.getByText("file gone")).toBeInTheDocument();
			// Jump affordance is gone (or disabled) for an orphan.
			expect(screen.queryByLabelText("Jump to code")).toBeNull();
			// The comment text + excerpt remain visible.
			expect(screen.getByText("stale note")).toBeInTheDocument();
		});

		it("clicking the commit header short oid calls onJumpToCommit with the full oid", async () => {
			const onJumpToCommit = vi.fn();
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "note")],
				resolutions: [resolvable("c1")],
			});
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit,
				},
			});
			await flush();

			await fireEvent.click(
				screen.getByLabelText(`Jump to commit ${commits[0].short_oid}`),
			);
			await flush();

			expect(onJumpToCommit).toHaveBeenCalledTimes(1);
			expect(onJumpToCommit).toHaveBeenCalledWith(COMMIT_A);
		});

		it("orders commit-level comments before line-anchored within the same commit group", async () => {
			installReads({
				commits: [commits[0]],
				comments: [
					lineAnchoredComment("L1", COMMIT_A, "line note one"),
					commitLevelComment("C1", COMMIT_A, "commit note one"),
					lineAnchoredComment("L2", COMMIT_A, "line note two"),
					commitLevelComment("C2", COMMIT_A, "commit note two"),
				],
				resolutions: [
					resolvable("L1"),
					resolvable("C1"),
					resolvable("L2"),
					resolvable("C2"),
				],
			});
			const { container } = render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
			await flush();

			const order = Array.from(
				container.querySelectorAll(".comment-card-text"),
			).map((el) => el.textContent);
			// Both commit-level first (capture-order stable), then both line-anchored.
			expect(order).toEqual([
				"commit note one",
				"commit note two",
				"line note one",
				"line note two",
			]);
		});

		it("classifies diff-source excerpt lines by their +/-/space prefix", async () => {
			const commentWithDiff: Comment = {
				id: "c1",
				text: "look at this",
				anchor: {
					commit_oid: COMMIT_A,
					file_path: "src/main.ts",
					source: "Diff",
					side: "New",
					start_line: 10,
					end_line: 12,
				},
				cached_excerpt:
					" const ctx = 0;\n+const added = 1;\n-const removed = 2;",
				commit_oid: null,
			};
			installReads({
				commits,
				comments: [commentWithDiff],
				resolutions: [resolvable("c1")],
			});
			const { container } = render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
			await flush();

			const addedRow = screen
				.getByText("const added = 1;")
				.closest(".diff-line");
			const removedRow = screen
				.getByText("const removed = 2;")
				.closest(".diff-line");
			const contextRow = screen
				.getByText("const ctx = 0;")
				.closest(".diff-line");
			expect(addedRow?.className).toContain("diff-line-add");
			expect(removedRow?.className).toContain("diff-line-del");
			expect(contextRow?.className).toContain("diff-line-context");
			// The gutter character is in its own span so copy-paste of the content
			// doesn't include the +/-.
			expect(container.querySelectorAll(".diff-gutter").length).toBe(3);
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
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
			await flush();

			expect(screen.getByText("commit gone")).toBeInTheDocument();
			expect(screen.getByText("line out of range")).toBeInTheDocument();
			expect(screen.getByText("file gone")).toBeInTheDocument();
		});
	});

	// Phase 70 D-01 / D-02 / DOC-01: the Generate button + preview swap.
	// The rune is exercised through the panel — only safeInvoke is mocked, so
	// these tests verify the integration ReviewPanel↔rune↔IPC end-to-end.
	describe("Generate / preview", () => {
		it("generate button is disabled when no comments", async () => {
			installReads({ commits, comments: [], resolutions: [] });
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
			await flush();

			const generateBtn = screen.getByRole("button", { name: /generate/i });
			expect(generateBtn).toBeDisabled();
			// The D-01 LOCKED tooltip drives the affordance when disabled.
			expect(generateBtn.getAttribute("title")).toBe(
				"Add at least one comment to generate",
			);
		});

		it("generate click invokes generate_review_doc and swaps to preview", async () => {
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "look here")],
				resolutions: [resolvable("c1")],
				generateDoc: "# Code review: trunk\n\nfoo",
			});
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
			await flush();

			const generateBtn = screen.getByRole("button", { name: /generate/i });
			expect(generateBtn).not.toBeDisabled();
			await fireEvent.click(generateBtn);
			await flush();

			expect(calledCommands()).toContain("generate_review_doc");
			expect(callArgs("generate_review_doc")?.path).toBe("/repo");
			// The preview view replaces the list — the markdown body is in the DOM.
			expect(
				await screen.findByText(/# Code review: trunk/),
			).toBeInTheDocument();
		});

		it("back to comments returns to list view", async () => {
			installReads({
				commits,
				comments: [lineAnchoredComment("c1", COMMIT_A, "look here")],
				resolutions: [resolvable("c1")],
				generateDoc: "# Code review: trunk\n\nfoo",
			});
			render(ReviewPanel, {
				props: {
					repoPath: "/repo",
					session: createReviewSession(),
					onJump: vi.fn(),
					onJumpToCommit: vi.fn(),
				},
			});
			await flush();

			// Drive into preview first.
			await fireEvent.click(screen.getByRole("button", { name: /generate/i }));
			await flush();
			expect(
				await screen.findByText(/# Code review: trunk/),
			).toBeInTheDocument();

			// Click the back affordance inside the preview header.
			await fireEvent.click(screen.getByRole("button", { name: /back/i }));
			await flush();

			// The list view's file-ref text returns; the preview body is gone.
			expect(screen.getByText("src/main.ts:L10-L12")).toBeInTheDocument();
			expect(screen.queryByText(/# Code review: trunk/)).toBeNull();
		});
	});
});
