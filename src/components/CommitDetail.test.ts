import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import type { CommitDetail, FileDiff } from "../lib/types.js";
import CommitDetailComponent from "./CommitDetail.svelte";

// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

const detail: CommitDetail = {
	oid: "abc123def456",
	short_oid: "abc123d",
	summary: "fix: null check",
	body: null,
	author_name: "Test User",
	author_email: "test@test.com",
	author_timestamp: 1700000000,
	committer_name: "Test User",
	committer_email: "test@test.com",
	committer_timestamp: 1700000000,
	parent_oids: ["parent1abc"],
};

const fileDiffs: FileDiff[] = [
	{
		path: "src/main.ts",
		status: "Modified",
		is_binary: false,
		hunks: [],
	},
	{
		path: "src/lib/utils.ts",
		status: "Added",
		is_binary: false,
		hunks: [],
	},
];

describe("CommitDetail", () => {
	it("renders commit summary", () => {
		render(CommitDetailComponent, {
			props: {
				commitDetail: detail,
				fileDiffs,
				selectedFile: null,
				onfileselect: vi.fn(),
				onclose: vi.fn(),
			},
		});
		expect(screen.getByText("fix: null check")).toBeInTheDocument();
	});

	it("renders author name and email", () => {
		render(CommitDetailComponent, {
			props: {
				commitDetail: detail,
				fileDiffs,
				selectedFile: null,
				onfileselect: vi.fn(),
				onclose: vi.fn(),
			},
		});
		expect(screen.getByText("Test User")).toBeInTheDocument();
		expect(screen.getByText("<test@test.com>")).toBeInTheDocument();
	});

	it("renders parent OIDs", () => {
		render(CommitDetailComponent, {
			props: {
				commitDetail: detail,
				fileDiffs,
				selectedFile: null,
				onfileselect: vi.fn(),
				onclose: vi.fn(),
			},
		});
		// parent_oids[0].slice(0,7) = "parent1"
		expect(screen.getByText("parent1")).toBeInTheDocument();
	});

	it("renders short oid in toolbar", () => {
		render(CommitDetailComponent, {
			props: {
				commitDetail: detail,
				fileDiffs,
				selectedFile: null,
				onfileselect: vi.fn(),
				onclose: vi.fn(),
			},
		});
		expect(
			screen.getByText((_, el) => el?.textContent === "commit: abc123d"),
		).toBeInTheDocument();
	});

	it("renders file count", () => {
		render(CommitDetailComponent, {
			props: {
				commitDetail: detail,
				fileDiffs,
				selectedFile: null,
				onfileselect: vi.fn(),
				onclose: vi.fn(),
			},
		});
		expect(screen.getByText("2 files changed")).toBeInTheDocument();
	});

	it("calls onclose when close button clicked", async () => {
		const onclose = vi.fn();
		render(CommitDetailComponent, {
			props: {
				commitDetail: detail,
				fileDiffs,
				selectedFile: null,
				onfileselect: vi.fn(),
				onclose,
			},
		});
		const closeBtn = screen.getByLabelText("Close commit detail");
		await fireEvent.click(closeBtn);
		expect(onclose).toHaveBeenCalledOnce();
	});

	it("renders commit body when present", () => {
		const detailWithBody: CommitDetail = {
			...detail,
			body: "This fixes a null pointer issue in the parser.",
		};
		render(CommitDetailComponent, {
			props: {
				commitDetail: detailWithBody,
				fileDiffs,
				selectedFile: null,
				onfileselect: vi.fn(),
				onclose: vi.fn(),
			},
		});
		expect(
			screen.getByText("This fixes a null pointer issue in the parser."),
		).toBeInTheDocument();
	});
});
