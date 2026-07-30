import { describe, expect, it } from "vitest";
import { parseWslUncPath, wslRootUncPath } from "./wsl-paths.js";

describe("parseWslUncPath", () => {
	it("parses a wsl.localhost repo path", () => {
		expect(parseWslUncPath("\\\\wsl.localhost\\Arch\\home\\me\\trunk")).toEqual(
			{
				distro: "Arch",
				linuxPath: "/home/me/trunk",
			},
		);
	});

	it("parses the legacy wsl$ prefix", () => {
		expect(parseWslUncPath("\\\\wsl$\\Ubuntu\\home\\me")).toEqual({
			distro: "Ubuntu",
			linuxPath: "/home/me",
		});
	});

	it("accepts forward slashes and mixed-case prefixes", () => {
		expect(parseWslUncPath("//WSL.LOCALHOST/Ubuntu-22.04/srv/repo")).toEqual({
			distro: "Ubuntu-22.04",
			linuxPath: "/srv/repo",
		});
	});

	it("maps a bare distro root to /", () => {
		expect(parseWslUncPath("\\\\wsl.localhost\\Arch")).toEqual({
			distro: "Arch",
			linuxPath: "/",
		});
		expect(parseWslUncPath("\\\\wsl.localhost\\Arch\\")).toEqual({
			distro: "Arch",
			linuxPath: "/",
		});
	});

	it("trims trailing separators", () => {
		expect(parseWslUncPath("\\\\wsl.localhost\\Arch\\home\\me\\")).toEqual({
			distro: "Arch",
			linuxPath: "/home/me",
		});
	});

	it("preserves distro name case", () => {
		expect(parseWslUncPath("\\\\wsl.localhost\\ArchLinux\\opt")?.distro).toBe(
			"ArchLinux",
		);
	});

	it("returns null for non-WSL paths", () => {
		expect(parseWslUncPath("C:\\code\\repo")).toBeNull();
		expect(parseWslUncPath("/home/me/repo")).toBeNull();
		expect(parseWslUncPath("\\\\server\\share\\repo")).toBeNull();
		expect(parseWslUncPath("")).toBeNull();
	});
});

describe("wslRootUncPath", () => {
	it("builds the distro root UNC path", () => {
		expect(wslRootUncPath("Arch")).toBe("\\\\wsl.localhost\\Arch");
	});
});
