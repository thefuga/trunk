import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import { makeFile } from "../__tests__/helpers/factories";
import TreeFileList from "./TreeFileList.svelte";

// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

describe("TreeFileList", () => {
	it("renders file paths in flat mode", () => {
		const files = [makeFile("src/a.ts"), makeFile("b.ts")];
		render(TreeFileList, {
			props: {
				files,
				treeMode: false,
				actionLabel: "Stage",
				onfileaction: vi.fn(),
			},
		});
		expect(screen.getByText("src/a.ts")).toBeInTheDocument();
		expect(screen.getByText("b.ts")).toBeInTheDocument();
	});

	it("renders tree structure in tree mode", () => {
		const files = [
			makeFile("src/lib/utils.ts"),
			makeFile("src/lib/types.ts"),
			makeFile("README.md"),
		];
		render(TreeFileList, {
			props: {
				files,
				treeMode: true,
				actionLabel: "Stage",
				onfileaction: vi.fn(),
			},
		});
		// In tree mode, "src/lib" should be a compressed directory name
		expect(screen.getByText("src/lib")).toBeInTheDocument();
		expect(screen.getByText("README.md")).toBeInTheDocument();
	});

	it("calls onfileaction when file action triggered", async () => {
		const onfileaction = vi.fn();
		const files = [makeFile("src/a.ts")];
		render(TreeFileList, {
			props: {
				files,
				treeMode: false,
				actionLabel: "+",
				onfileaction,
			},
		});
		// FileRow shows action button on hover only — trigger mouseenter first
		const fileRow = screen.getByRole("listitem");
		await fireEvent.mouseEnter(fileRow);
		// Action button aria-label is "Stage file" when actionLabel="+"
		const stageBtn = screen.getByLabelText("Stage file");
		await fireEvent.click(stageBtn);
		expect(onfileaction).toHaveBeenCalledWith("src/a.ts");
	});

	it("calls onfileclick when file clicked", async () => {
		const onfileclick = vi.fn();
		const files = [makeFile("src/a.ts")];
		render(TreeFileList, {
			props: {
				files,
				treeMode: false,
				actionLabel: "Stage",
				onfileaction: vi.fn(),
				onfileclick,
			},
		});
		// Click on the file name text
		const fileText = screen.getByText("src/a.ts");
		await fireEvent.click(fileText);
		expect(onfileclick).toHaveBeenCalledWith("src/a.ts");
	});

	it("renders list role in flat mode and tree role in tree mode", () => {
		const files = [makeFile("a.ts")];
		const { rerender } = render(TreeFileList, {
			props: {
				files,
				treeMode: false,
				actionLabel: "Stage",
				onfileaction: vi.fn(),
			},
		});
		expect(screen.getByRole("list")).toBeInTheDocument();

		rerender({
			files,
			treeMode: true,
			actionLabel: "Stage",
			onfileaction: vi.fn(),
		});
		expect(screen.getByRole("tree")).toBeInTheDocument();
	});
});
