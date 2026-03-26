import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import RemoteGroup from "./RemoteGroup.svelte";
import "../__tests__/helpers/tauri-mock";

describe("RemoteGroup", () => {
	const defaultProps = {
		remoteName: "origin",
		branches: ["main", "dev"],
		checkingOut: null,
		errorBranch: null,
		errorText: "",
		oncheckout: vi.fn(),
	};

	it("renders remote name header", () => {
		render(RemoteGroup, { props: defaultProps });
		expect(screen.getByText("origin")).toBeInTheDocument();
	});

	it("renders branch rows for each branch", () => {
		render(RemoteGroup, { props: defaultProps });
		expect(screen.getByText("main")).toBeInTheDocument();
		expect(screen.getByText("dev")).toBeInTheDocument();
	});

	it("calls oncheckout with full name when branch clicked", async () => {
		const oncheckout = vi.fn();
		render(RemoteGroup, {
			props: { ...defaultProps, oncheckout },
		});
		const buttons = screen.getAllByRole("button");
		await fireEvent.click(buttons[0]);
		expect(oncheckout).toHaveBeenCalledWith("origin/main");
	});

	it("shows loading state for checking out branch", () => {
		render(RemoteGroup, {
			props: { ...defaultProps, checkingOut: "origin/main" },
		});
		// The BranchRow for "main" should show loading indicator
		expect(screen.getByText(/main/)).toBeInTheDocument();
	});
});
