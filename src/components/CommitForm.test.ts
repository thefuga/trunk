import { render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import CommitForm from "./CommitForm.svelte";
import "../__tests__/helpers/tauri-mock";

describe("CommitForm", () => {
	const defaultProps = {
		repoPath: "/repo",
		stagedCount: 1,
		clearRedoStack: vi.fn(),
	};

	it("renders Commit button in commit mode", () => {
		render(CommitForm, { props: defaultProps });
		// "Commit" appears both as tab label and submit button text
		const buttons = screen.getAllByText("Commit");
		expect(buttons.length).toBeGreaterThanOrEqual(2);
	});

	it("renders Amend tab button", () => {
		render(CommitForm, { props: defaultProps });
		expect(screen.getByText("Amend")).toBeInTheDocument();
	});

	it("renders Stash tab button", () => {
		render(CommitForm, { props: defaultProps });
		expect(screen.getByText("Stash")).toBeInTheDocument();
	});

	it("renders subject input with commit placeholder", () => {
		render(CommitForm, { props: defaultProps });
		expect(
			screen.getByPlaceholderText("Summary (required)"),
		).toBeInTheDocument();
	});

	it("renders body textarea", () => {
		render(CommitForm, { props: defaultProps });
		expect(
			screen.getByPlaceholderText("Description (optional)"),
		).toBeInTheDocument();
	});

	it("shows all three mode tabs", () => {
		render(CommitForm, { props: defaultProps });
		const buttons = screen.getAllByRole("button");
		const tabLabels = buttons.map((b) => b.textContent?.trim());
		expect(tabLabels).toContain("Commit");
		expect(tabLabels).toContain("Amend");
		expect(tabLabels).toContain("Stash");
	});
});
