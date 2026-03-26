import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import BranchRow from "./BranchRow.svelte";
import "../__tests__/helpers/tauri-mock";

describe("BranchRow", () => {
	it("renders branch name", () => {
		render(BranchRow, { props: { name: "feature/login" } });
		expect(screen.getByText("feature/login")).toBeInTheDocument();
	});

	it("calls onclick when clicked", async () => {
		const onclick = vi.fn();
		render(BranchRow, { props: { name: "main", onclick } });
		await fireEvent.click(screen.getByRole("button"));
		expect(onclick).toHaveBeenCalled();
	});

	it("shows error message when isError=true", () => {
		render(BranchRow, {
			props: {
				name: "main",
				isError: true,
				errorText: "Checkout failed",
			},
		});
		expect(screen.getByText("Checkout failed")).toBeInTheDocument();
	});

	it("shows default error when isError=true but no errorText", () => {
		render(BranchRow, { props: { name: "main", isError: true } });
		expect(
			screen.getByText(/Cannot checkout/),
		).toBeInTheDocument();
	});

	it("shows ahead count", () => {
		render(BranchRow, { props: { name: "main", ahead: 3 } });
		expect(screen.getByText("3")).toBeInTheDocument();
	});

	it("shows behind count", () => {
		render(BranchRow, { props: { name: "main", behind: 2 } });
		expect(screen.getByText("2")).toBeInTheDocument();
	});

	it("does not show ahead/behind when both zero", () => {
		const { container } = render(BranchRow, {
			props: { name: "main", ahead: 0, behind: 0 },
		});
		// The ahead/behind span wrapper should not be present
		// when both are 0 (the {#if behind > 0 || ahead > 0} guard)
		const arrows = container.querySelectorAll("svg");
		// No ArrowUp or ArrowDown icons rendered
		expect(
			Array.from(arrows).filter(
				(svg) =>
					svg.innerHTML.includes("ArrowUp") ||
					svg.innerHTML.includes("ArrowDown"),
			),
		).toHaveLength(0);
	});

	it("renders with isHead=true without error", () => {
		const { container } = render(BranchRow, {
			props: { name: "main", isHead: true },
		});
		// isHead=true sets visual emphasis on the branch name
		expect(screen.getByText("main")).toBeInTheDocument();
		expect(container.querySelector("[role='button']")).toBeInTheDocument();
	});
});
