import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import SubBar from "./SubBar.svelte";

describe("SubBar", () => {
	const base = {
		branch: "main",
		ahead: 0,
		behind: 0,
		filesChanged: 0,
		hasUpstream: false,
	};

	it("shows the head branch name", () => {
		render(SubBar, { props: base });
		expect(screen.getByText("main")).toBeInTheDocument();
	});

	it("shows the files-changed count", () => {
		render(SubBar, { props: { ...base, filesChanged: 8 } });
		expect(screen.getByText("8 files changed")).toBeInTheDocument();
	});

	it("singularizes a single changed file", () => {
		render(SubBar, { props: { ...base, filesChanged: 1 } });
		expect(screen.getByText("1 file changed")).toBeInTheDocument();
	});

	it("shows ahead and behind when nonzero", () => {
		render(SubBar, {
			props: { ...base, ahead: 3, behind: 2, hasUpstream: true },
		});
		expect(screen.getByText("3 ahead")).toBeInTheDocument();
		expect(screen.getByText("2 behind")).toBeInTheDocument();
	});

	it("reports a synced upstream when level", () => {
		render(SubBar, { props: { ...base, hasUpstream: true } });
		expect(
			screen.getByText("All changes synced to origin"),
		).toBeInTheDocument();
	});

	it("omits the synced label without an upstream", () => {
		render(SubBar, { props: base });
		expect(screen.queryByText("All changes synced to origin")).toBeNull();
	});
});
