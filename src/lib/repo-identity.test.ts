import { describe, expect, it } from "vitest";
import { localRepoDescriptor, repoIdForLocator } from "./types.js";

describe("repo identity", () => {
	it("builds stable ids for local and WSL locators", () => {
		expect(repoIdForLocator({ backend: "Local", path: "/repo" })).toBe(
			"local:/repo",
		);
		expect(
			repoIdForLocator({
				backend: "Wsl",
				distro: "Ubuntu",
				linux_path: "/repo",
			}),
		).toBe("wsl:Ubuntu:/repo");
	});

	it("normalizes trailing slashes in ids without changing display paths", () => {
		expect(repoIdForLocator({ backend: "Local", path: "/repo/" })).toBe(
			"local:/repo",
		);
		expect(
			repoIdForLocator({
				backend: "Wsl",
				distro: "Ubuntu",
				linux_path: "/repo/",
			}),
		).toBe("wsl:Ubuntu:/repo");

		expect(localRepoDescriptor("/repo/", "repo")).toEqual(
			expect.objectContaining({
				id: "local:/repo",
				display_path: "/repo/",
				locator: { backend: "Local", path: "/repo/" },
			}),
		);
	});

	it("normalizes Windows trailing separators in ids without changing display paths", () => {
		expect(repoIdForLocator({ backend: "Local", path: "C:\\repo\\" })).toBe(
			"local:C:\\repo",
		);

		expect(localRepoDescriptor("C:\\repo\\", "repo")).toEqual(
			expect.objectContaining({
				id: "local:C:\\repo",
				display_path: "C:\\repo\\",
				locator: { backend: "Local", path: "C:\\repo\\" },
			}),
		);
	});

	it("keeps local display metadata separate from locator identity", () => {
		expect(localRepoDescriptor("/repo", "Display Name")).toEqual({
			id: "local:/repo",
			display_name: "Display Name",
			display_path: "/repo",
			locator: { backend: "Local", path: "/repo" },
		});
	});
});
