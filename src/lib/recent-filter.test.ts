import { describe, expect, it } from "vitest";
import { filterRecents } from "./recent-filter.js";
import type { RecentRepo } from "./store.js";

const sample: RecentRepo[] = [
	{ name: "trunk", path: "/Users/joao/code/trunk" },
	{ name: "alpha", path: "/Users/joao/work/alpha" },
	{ name: "Beta", path: "/opt/projects/beta" },
];

describe("filterRecents", () => {
	it("returns the input unchanged for an empty query", () => {
		const result = filterRecents(sample, "");
		expect(result).toEqual(sample);
		expect(result).toHaveLength(sample.length);
	});

	it("matches name case-insensitively", () => {
		const result = filterRecents(sample, "TRU");
		expect(result).toEqual([{ name: "trunk", path: "/Users/joao/code/trunk" }]);
	});

	it("matches path case-insensitively", () => {
		const result = filterRecents(sample, "WORK");
		expect(result).toEqual([{ name: "alpha", path: "/Users/joao/work/alpha" }]);
	});

	it("treats whitespace-only query as empty", () => {
		const result = filterRecents(sample, "   ");
		expect(result).toEqual(sample);
	});

	it("returns an empty array when nothing matches", () => {
		const result = filterRecents(sample, "zzz-no-match");
		expect(result).toEqual([]);
	});

	it("preserves original ordering of input (filter, not sort)", () => {
		// "joao" matches the two paths under /Users/joao/* — trunk first, alpha second.
		const result = filterRecents(sample, "joao");
		expect(result.map((r) => r.name)).toEqual(["trunk", "alpha"]);
	});

	it("matches when either name or path contains the query", () => {
		// "beta" matches both the name "Beta" (case-insensitive) and the path
		// "/opt/projects/beta" — proving union, not intersection.
		const result = filterRecents(sample, "beta");
		expect(result).toEqual([{ name: "Beta", path: "/opt/projects/beta" }]);
	});
});
