import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import OperationBanner from "./OperationBanner.svelte";
import "../__tests__/helpers/tauri-mock";
import type { OperationInfo } from "../lib/types";

function makeInfo(overrides: Partial<OperationInfo> = {}): OperationInfo {
	return {
		op_type: overrides.op_type ?? "Merge",
		source_branch: overrides.source_branch ?? "feature",
		target_branch: overrides.target_branch ?? "main",
		progress: overrides.progress ?? null,
		source_color_index: overrides.source_color_index ?? 1,
		target_color_index: overrides.target_color_index ?? 0,
		rebase_message: overrides.rebase_message ?? null,
	};
}

describe("OperationBanner", () => {
	it("shows 'Merging' for merge operations", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "Merge" }),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("Merging")).toBeInTheDocument();
	});

	it("shows source and target branch names", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({
					op_type: "Merge",
					source_branch: "feature",
					target_branch: "main",
				}),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("feature")).toBeInTheDocument();
		expect(screen.getByText("main")).toBeInTheDocument();
	});

	it("shows 'Rebasing' for rebase operations", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "Rebase" }),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("Rebasing")).toBeInTheDocument();
	});

	it("shows 'onto' for rebase instead of 'into'", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "Rebase" }),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("onto")).toBeInTheDocument();
		expect(screen.queryByText("into")).toBeNull();
	});

	it("shows 'into' for merge instead of 'onto'", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "Merge" }),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("into")).toBeInTheDocument();
		expect(screen.queryByText("onto")).toBeNull();
	});

	it("shows Continue/Skip/Abort buttons for rebase", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "Rebase" }),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("Continue")).toBeInTheDocument();
		expect(screen.getByText("Skip")).toBeInTheDocument();
		expect(screen.getByText("Abort")).toBeInTheDocument();
	});

	it("does not show Continue/Skip/Abort for merge", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "Merge" }),
				repoPath: "/repo",
			},
		});
		expect(screen.queryByText("Continue")).toBeNull();
		expect(screen.queryByText("Skip")).toBeNull();
		expect(screen.queryByText("Abort")).toBeNull();
	});

	it("shows progress for rebase", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "Rebase", progress: "2/5" }),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("(2/5)")).toBeInTheDocument();
	});

	it("shows cherry-pick label", () => {
		render(OperationBanner, {
			props: {
				info: makeInfo({ op_type: "CherryPick" }),
				repoPath: "/repo",
			},
		});
		expect(screen.getByText("Cherry-pick in progress")).toBeInTheDocument();
	});
});
