import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import BranchSidebar from "./BranchSidebar.svelte";

// Shared Tauri mocks (event, store, dialog, path, menu, etc.)
import "../__tests__/helpers/tauri-mock";

// Re-declare invoke mock locally so vi.mocked() works with hoisting
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn().mockResolvedValue(undefined),
}));

const mockInvoke = vi.mocked(invoke);

function mockListRefs(overrides?: {
	local?: Array<{
		name: string;
		is_head: boolean;
		upstream: string | null;
		ahead: number;
		behind: number;
		last_commit_timestamp: number;
	}>;
	remote?: Array<{
		name: string;
		is_head: boolean;
		upstream: string | null;
		ahead: number;
		behind: number;
		last_commit_timestamp: number;
	}>;
	tags?: Array<{
		name: string;
		short_name: string;
		ref_type: string;
		is_head: boolean;
		color_index: number;
	}>;
	stashes?: Array<{
		index: number;
		name: string;
		short_name: string;
		oid: string;
		parent_oid: string | null;
	}>;
}) {
	return {
		local: overrides?.local ?? [
			{
				name: "main",
				is_head: true,
				upstream: null,
				ahead: 0,
				behind: 0,
				last_commit_timestamp: 1700000000,
			},
		],
		remote: overrides?.remote ?? [],
		tags: overrides?.tags ?? [],
		stashes: overrides?.stashes ?? [],
	};
}

describe("BranchSidebar", () => {
	beforeEach(() => {
		mockInvoke.mockReset();
		mockInvoke.mockImplementation((cmd: string) => {
			if (cmd === "list_refs") return Promise.resolve(mockListRefs());
			return Promise.resolve(undefined);
		});
	});

	it("renders without crashing", () => {
		const { container } = render(BranchSidebar, {
			props: { repoPath: "/test/repo" },
		});
		expect(container).toBeTruthy();
	});

	it("renders local branch section header", async () => {
		render(BranchSidebar, {
			props: { repoPath: "/test/repo" },
		});
		// BranchSection renders "{label} ({count})" — e.g. "Local (1)"
		await waitFor(() => {
			expect(screen.getByText("Local (1)")).toBeInTheDocument();
		});
	});

	it("renders branch name from refs response", async () => {
		render(BranchSidebar, {
			props: { repoPath: "/test/repo" },
		});
		await waitFor(() => {
			expect(screen.getByText("main")).toBeInTheDocument();
		});
	});

	it("renders multiple local branches", async () => {
		mockInvoke.mockImplementation((cmd: string) => {
			if (cmd === "list_refs")
				return Promise.resolve(
					mockListRefs({
						local: [
							{
								name: "main",
								is_head: true,
								upstream: null,
								ahead: 0,
								behind: 0,
								last_commit_timestamp: 1700000000,
							},
							{
								name: "feature/login",
								is_head: false,
								upstream: null,
								ahead: 0,
								behind: 0,
								last_commit_timestamp: 1700000100,
							},
						],
					}),
				);
			return Promise.resolve(undefined);
		});

		render(BranchSidebar, {
			props: { repoPath: "/test/repo" },
		});

		await waitFor(() => {
			expect(screen.getByText("main")).toBeInTheDocument();
			expect(screen.getByText("feature/login")).toBeInTheDocument();
		});
	});

	it("calls list_refs on mount with correct repo path", async () => {
		render(BranchSidebar, {
			props: { repoPath: "/my/project" },
		});
		await waitFor(() => {
			expect(mockInvoke).toHaveBeenCalledWith("list_refs", {
				path: "/my/project",
			});
		});
	});

	describe("remote branch double-click checkout", () => {
		const refsWithRemote = {
			local: [
				{
					name: "main",
					is_head: true,
					upstream: null,
					ahead: 0,
					behind: 0,
					last_commit_timestamp: 1700000000,
				},
			],
			remote: [
				{
					name: "origin/feature",
					is_head: false,
					upstream: null,
					ahead: 0,
					behind: 0,
					last_commit_timestamp: 1700000000,
				},
			],
			tags: [],
			stashes: [],
		};

		it("calls create_branch with correct args on remote branch double-click", async () => {
			mockInvoke.mockImplementation((cmd: string) => {
				if (cmd === "list_refs") return Promise.resolve(refsWithRemote);
				return Promise.resolve(undefined);
			});

			render(BranchSidebar, {
				props: { repoPath: "/test/repo" },
			});

			// Wait for Remote section to appear, then expand it
			await waitFor(() => {
				expect(screen.getByText("Remote (1)")).toBeInTheDocument();
			});
			await fireEvent.click(screen.getByText("Remote (1)"));

			// Wait for the remote branch row to appear
			await waitFor(() => {
				expect(screen.getByText("feature")).toBeInTheDocument();
			});

			// Double-click the remote branch
			const branchButtons = screen
				.getByTestId("branch-section-remote")
				.querySelectorAll('[role="button"]');
			// First button is the section header, subsequent ones are branch rows
			const remoteBranchButton = Array.from(branchButtons).find(
				(btn) => btn.textContent?.includes("feature"),
			);
			expect(remoteBranchButton).toBeTruthy();
			await fireEvent.dblClick(remoteBranchButton!);

			await waitFor(() => {
				expect(mockInvoke).toHaveBeenCalledWith("create_branch", {
					path: "/test/repo",
					name: "feature",
					fromOid: "origin/feature",
				});
			});
		});

		it("shows error toast when create_branch fails", async () => {
			mockInvoke.mockImplementation((cmd: string) => {
				if (cmd === "list_refs") return Promise.resolve(refsWithRemote);
				if (cmd === "create_branch")
					return Promise.reject(
						JSON.stringify({
							code: "branch_exists",
							message: "branch 'feature' already exists",
						}),
					);
				return Promise.resolve(undefined);
			});

			render(BranchSidebar, {
				props: { repoPath: "/test/repo" },
			});

			// Expand Remote section
			await waitFor(() => {
				expect(screen.getByText("Remote (1)")).toBeInTheDocument();
			});
			await fireEvent.click(screen.getByText("Remote (1)"));

			await waitFor(() => {
				expect(screen.getByText("feature")).toBeInTheDocument();
			});

			const branchButtons = screen
				.getByTestId("branch-section-remote")
				.querySelectorAll('[role="button"]');
			const remoteBranchButton = Array.from(branchButtons).find(
				(btn) => btn.textContent?.includes("feature"),
			);
			await fireEvent.dblClick(remoteBranchButton!);

			// Verify create_branch was called (and it rejected)
			await waitFor(() => {
				expect(mockInvoke).toHaveBeenCalledWith("create_branch", {
					path: "/test/repo",
					name: "feature",
					fromOid: "origin/feature",
				});
			});
		});
	});
});
