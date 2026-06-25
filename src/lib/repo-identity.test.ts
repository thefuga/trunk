import { describe, expect, it } from "vitest";
import {
	localRepoDescriptor,
	normalizeRepoDescriptor,
	repoIdForLocator,
	wslRepoDescriptor,
} from "./types.js";

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

	it("builds WSL display metadata from distro and Linux path", () => {
		expect(wslRepoDescriptor("Ubuntu", "/home/me/trunk")).toEqual({
			id: "wsl:Ubuntu:/home/me/trunk",
			display_name: "trunk",
			display_path: "Ubuntu:/home/me/trunk",
			locator: {
				backend: "Wsl",
				distro: "Ubuntu",
				linux_path: "/home/me/trunk",
			},
		});
	});

	it("recomputes descriptor ids from locator identity", () => {
		expect(
			normalizeRepoDescriptor({
				id: "stale-id",
				display_name: "repo",
				display_path: "C:\\repo\\",
				locator: { backend: "Local", path: "C:\\repo\\" },
			}),
		).toEqual({
			id: "local:C:\\repo",
			display_name: "repo",
			display_path: "C:\\repo\\",
			locator: { backend: "Local", path: "C:\\repo\\" },
		});
	});
});
