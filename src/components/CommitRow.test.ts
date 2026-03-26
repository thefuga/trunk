import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import CommitRow from "./CommitRow.svelte";
import "../__tests__/helpers/tauri-mock";
import { makeCommit } from "../__tests__/helpers/factories";
import type { ColumnVisibility, ColumnWidths } from "../lib/store";

const defaultWidths: ColumnWidths = {
	ref: 120,
	graph: 24,
	author: 60,
	date: 40,
	sha: 50,
};

const allVisible: ColumnVisibility = {
	ref: true,
	graph: true,
	message: true,
	author: true,
	date: true,
	sha: true,
};

describe("CommitRow", () => {
	it("renders commit summary", () => {
		const commit = makeCommit({ oid: "abc1234567", summary: "fix: bug" });
		render(CommitRow, {
			props: {
				commit,
				rowIndex: 0,
				columnWidths: defaultWidths,
				columnVisibility: allVisible,
			},
		});
		expect(screen.getByText("fix: bug")).toBeInTheDocument();
	});

	it("renders author name when column visible", () => {
		const commit = makeCommit({
			oid: "abc1234567",
			author_name: "Alice",
		});
		render(CommitRow, {
			props: {
				commit,
				rowIndex: 0,
				columnWidths: defaultWidths,
				columnVisibility: allVisible,
			},
		});
		expect(screen.getByText("Alice")).toBeInTheDocument();
	});

	it("hides author column when not visible", () => {
		const commit = makeCommit({
			oid: "abc1234567",
			author_name: "Alice",
		});
		render(CommitRow, {
			props: {
				commit,
				rowIndex: 0,
				columnWidths: defaultWidths,
				columnVisibility: { ...allVisible, author: false },
			},
		});
		expect(screen.queryByText("Alice")).toBeNull();
	});

	it("renders short OID when column visible", () => {
		const commit = makeCommit({ oid: "def5678901" });
		render(CommitRow, {
			props: {
				commit,
				rowIndex: 0,
				columnWidths: defaultWidths,
				columnVisibility: allVisible,
			},
		});
		expect(screen.getByText("def5678")).toBeInTheDocument();
	});

	it("renders WIP row with italic class", () => {
		const commit = makeCommit({
			oid: "__wip__",
			summary: "Working changes",
		});
		const { container } = render(CommitRow, {
			props: {
				commit,
				rowIndex: 0,
				columnWidths: defaultWidths,
				columnVisibility: allVisible,
			},
		});
		const italicEl = container.querySelector(".italic");
		expect(italicEl).not.toBeNull();
		expect(italicEl?.textContent).toContain("Working changes");
	});

	it("calls onselect with oid when clicked", async () => {
		const onselect = vi.fn();
		const commit = makeCommit({ oid: "abc1234567" });
		const { container } = render(CommitRow, {
			props: {
				commit,
				rowIndex: 0,
				columnWidths: defaultWidths,
				columnVisibility: allVisible,
				onselect,
			},
		});
		await fireEvent.click(container.firstElementChild!);
		expect(onselect).toHaveBeenCalledWith("abc1234567");
	});

	it("hides SHA column when not visible", () => {
		const commit = makeCommit({ oid: "xyz9876543" });
		render(CommitRow, {
			props: {
				commit,
				rowIndex: 0,
				columnWidths: defaultWidths,
				columnVisibility: { ...allVisible, sha: false },
			},
		});
		expect(screen.queryByText("xyz9876")).toBeNull();
	});
});
