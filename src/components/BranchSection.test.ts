import { fireEvent, render, screen } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { describe, expect, it, vi } from "vitest";
import BranchSection from "./BranchSection.svelte";
import "../__tests__/helpers/tauri-mock";

const emptySnippet = createRawSnippet(() => ({
	render: () => "",
}));

describe("BranchSection", () => {
	it("renders label with count", () => {
		render(BranchSection, {
			props: {
				label: "Branches",
				count: 5,
				expanded: false,
				ontoggle: vi.fn(),
				children: emptySnippet,
			},
		});
		expect(screen.getByText("Branches (5)")).toBeInTheDocument();
	});

	it("calls ontoggle when header clicked", async () => {
		const ontoggle = vi.fn();
		render(BranchSection, {
			props: {
				label: "Branches",
				count: 3,
				expanded: false,
				ontoggle,
				children: emptySnippet,
			},
		});
		await fireEvent.click(screen.getByRole("button"));
		expect(ontoggle).toHaveBeenCalled();
	});

	it("shows create button when showCreateButton=true", () => {
		render(BranchSection, {
			props: {
				label: "Branches",
				count: 3,
				expanded: false,
				ontoggle: vi.fn(),
				showCreateButton: true,
				oncreate: vi.fn(),
				children: emptySnippet,
			},
		});
		expect(screen.getByLabelText("Create new branch")).toBeInTheDocument();
	});

	it("hides create button by default", () => {
		render(BranchSection, {
			props: {
				label: "Branches",
				count: 3,
				expanded: false,
				ontoggle: vi.fn(),
				children: emptySnippet,
			},
		});
		expect(screen.queryByLabelText("Create new branch")).toBeNull();
	});

	it("calls oncreate when create button clicked", async () => {
		const oncreate = vi.fn();
		render(BranchSection, {
			props: {
				label: "Branches",
				count: 3,
				expanded: false,
				ontoggle: vi.fn(),
				showCreateButton: true,
				oncreate,
				children: emptySnippet,
			},
		});
		await fireEvent.click(screen.getByLabelText("Create new branch"));
		expect(oncreate).toHaveBeenCalled();
	});
});
