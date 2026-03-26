import { render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import FileRow from "./FileRow.svelte";
import "../__tests__/helpers/tauri-mock";
import { makeFile } from "../__tests__/helpers/factories";

describe("FileRow", () => {
	it("renders file path", () => {
		render(FileRow, {
			props: {
				file: makeFile("README.md", "Modified"),
				actionLabel: "+",
				onaction: vi.fn(),
			},
		});
		expect(screen.getByText("README.md")).toBeInTheDocument();
	});

	it("renders displayName when provided", () => {
		render(FileRow, {
			props: {
				file: makeFile("src/lib/utils/short.ts", "Modified"),
				actionLabel: "+",
				onaction: vi.fn(),
				displayName: "short.ts",
			},
		});
		expect(screen.getByText("short.ts")).toBeInTheDocument();
		expect(screen.queryByText("src/lib/utils/short.ts")).toBeNull();
	});

	it("has listitem role when depth=0", () => {
		render(FileRow, {
			props: {
				file: makeFile("README.md"),
				actionLabel: "+",
				onaction: vi.fn(),
				depth: 0,
			},
		});
		expect(screen.getByRole("listitem")).toBeInTheDocument();
	});

	it("has treeitem role when depth>0", () => {
		render(FileRow, {
			props: {
				file: makeFile("README.md"),
				actionLabel: "+",
				onaction: vi.fn(),
				depth: 1,
			},
		});
		expect(screen.getByRole("treeitem")).toBeInTheDocument();
	});

	it("renders New file with file path", () => {
		render(FileRow, {
			props: {
				file: makeFile("new-file.ts", "New"),
				actionLabel: "+",
				onaction: vi.fn(),
			},
		});
		expect(screen.getByText("new-file.ts")).toBeInTheDocument();
	});
});
