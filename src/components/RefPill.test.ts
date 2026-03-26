import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import RefPill from "./RefPill.svelte";
import "../__tests__/helpers/tauri-mock";
import { makeRef } from "../__tests__/helpers/factories";

describe("RefPill", () => {
	it("renders primary ref short name", () => {
		render(RefPill, {
			props: {
				refs: [makeRef({ short_name: "main", is_head: true })],
			},
		});
		expect(screen.getByText("main")).toBeInTheDocument();
	});

	it("applies font-bold class to HEAD ref", () => {
		render(RefPill, {
			props: {
				refs: [makeRef({ short_name: "main", is_head: true })],
			},
		});
		const pill = screen.getByText("main");
		expect(pill.className).toContain("font-bold");
	});

	it("renders tag prefix", () => {
		render(RefPill, {
			props: {
				refs: [
					makeRef({
						short_name: "v1.0",
						ref_type: "Tag",
						name: "refs/tags/v1.0",
					}),
				],
			},
		});
		expect(screen.getByText(/\u25C6\s*v1\.0/)).toBeInTheDocument();
	});

	it("renders stash prefix", () => {
		render(RefPill, {
			props: {
				refs: [
					makeRef({
						short_name: "stash@{0}",
						ref_type: "Stash",
						name: "refs/stash",
					}),
				],
			},
		});
		expect(screen.getByText(/\u2691/)).toBeInTheDocument();
	});

	it("renders all refs when showAll=true", () => {
		const refs = [
			makeRef({ short_name: "main", is_head: true }),
			makeRef({ short_name: "develop", color_index: 1 }),
			makeRef({ short_name: "feature", color_index: 2 }),
		];
		render(RefPill, { props: { refs, showAll: true } });
		expect(screen.getByText("main")).toBeInTheDocument();
		expect(screen.getByText("develop")).toBeInTheDocument();
		expect(screen.getByText("feature")).toBeInTheDocument();
	});

	it("renders only first ref when showAll=false", () => {
		const refs = [
			makeRef({ short_name: "main", is_head: true }),
			makeRef({ short_name: "develop", color_index: 1 }),
			makeRef({ short_name: "feature", color_index: 2 }),
		];
		render(RefPill, { props: { refs, showAll: false } });
		expect(screen.getByText("main")).toBeInTheDocument();
		expect(screen.queryByText("develop")).toBeNull();
		expect(screen.queryByText("feature")).toBeNull();
	});
});
