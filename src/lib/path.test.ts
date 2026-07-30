import { describe, expect, it } from "vitest";
import { repoNameFromPath } from "./path.js";

// displayPath is exercised through component tests; it depends on the Tauri
// homeDir API. Only the pure helper is unit-tested here.
describe("repoNameFromPath", () => {
	it("returns the last segment of a unix path", () => {
		expect(repoNameFromPath("/home/me/code/trunk")).toBe("trunk");
	});

	it("returns the last segment of a Windows backslash path", () => {
		expect(repoNameFromPath("C:\\code\\my-repo")).toBe("my-repo");
	});

	it("handles UNC paths", () => {
		expect(repoNameFromPath("\\\\wsl.localhost\\Arch\\home\\me\\trunk")).toBe(
			"trunk",
		);
	});

	it("ignores trailing separators", () => {
		expect(repoNameFromPath("/home/me/trunk/")).toBe("trunk");
		expect(repoNameFromPath("C:\\code\\my-repo\\")).toBe("my-repo");
	});

	it("falls back to the input for separator-only paths", () => {
		expect(repoNameFromPath("/")).toBe("/");
	});
});
