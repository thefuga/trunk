import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import PullDropdown from "./PullDropdown.svelte";
import "../__tests__/helpers/tauri-mock";
import { createRemoteState } from "../lib/remote-state.svelte";

describe("PullDropdown", () => {
	function renderDropdown(disabled = false) {
		return render(PullDropdown, {
			props: {
				repoPath: "/repo",
				disabled,
				remoteState: createRemoteState(),
			},
		});
	}

	it("renders pull/fetch button", () => {
		renderDropdown();
		const button = screen.getByTitle("Pull options");
		expect(button).toBeInTheDocument();
	});

	it("shows dropdown options when clicked", async () => {
		renderDropdown();
		const button = screen.getByTitle("Pull options");
		await fireEvent.click(button);
		expect(screen.getByText("Fetch")).toBeInTheDocument();
		expect(
			screen.getByText("Fast-forward if possible"),
		).toBeInTheDocument();
		expect(screen.getByText("Fast-forward only")).toBeInTheDocument();
		expect(screen.getByText("Pull (rebase)")).toBeInTheDocument();
	});

	it("closes dropdown on second click", async () => {
		renderDropdown();
		const button = screen.getByTitle("Pull options");
		await fireEvent.click(button);
		expect(screen.getByText("Fetch")).toBeInTheDocument();
		await fireEvent.click(button);
		expect(screen.queryByText("Fetch")).toBeNull();
	});

	it("does not open when disabled", async () => {
		renderDropdown(true);
		const button = screen.getByTitle("Pull options");
		await fireEvent.click(button);
		expect(screen.queryByText("Fetch")).toBeNull();
	});
});
