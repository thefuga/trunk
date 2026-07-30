import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadWslOpenTargets, sortDistrosDefaultFirst } from "./wsl.js";

vi.mock("./invoke.js", () => ({
	safeInvoke: vi.fn(),
}));

describe("loadWslOpenTargets", () => {
	beforeEach(async () => {
		const { safeInvoke } = await import("./invoke.js");
		vi.mocked(safeInvoke).mockReset();
	});

	it("returns availability and distros when WSL is available", async () => {
		const { safeInvoke } = await import("./invoke.js");
		vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
			if (cmd === "wsl_availability") {
				return Promise.resolve({
					available: true,
					supported_platform: true,
					message: null,
				});
			}
			if (cmd === "list_wsl_distros") {
				return Promise.resolve([{ name: "Arch", default: true }]);
			}
			return Promise.resolve(undefined);
		});

		const targets = await loadWslOpenTargets();
		expect(targets.availability?.available).toBe(true);
		expect(targets.distros).toEqual([{ name: "Arch", default: true }]);
	});

	it("skips the distro list when WSL is unavailable", async () => {
		const { safeInvoke } = await import("./invoke.js");
		vi.mocked(safeInvoke).mockResolvedValue({
			available: false,
			supported_platform: true,
			message: "WSL is not installed.",
		});

		const targets = await loadWslOpenTargets();
		expect(targets.distros).toEqual([]);
		expect(vi.mocked(safeInvoke)).toHaveBeenCalledTimes(1);
	});

	it("degrades to empty results instead of throwing", async () => {
		const { safeInvoke } = await import("./invoke.js");
		vi.mocked(safeInvoke).mockRejectedValue({
			code: "unknown_error",
			message: "boom",
		});

		const targets = await loadWslOpenTargets();
		expect(targets.availability).toBeNull();
		expect(targets.distros).toEqual([]);
	});
});

describe("sortDistrosDefaultFirst", () => {
	it("moves the default distro to the front, keeping relative order", () => {
		const input = [
			{ name: "Debian", default: false },
			{ name: "docker-desktop", default: false },
			{ name: "Arch", default: true },
		];
		expect(sortDistrosDefaultFirst(input).map((d) => d.name)).toEqual([
			"Arch",
			"Debian",
			"docker-desktop",
		]);
	});

	it("does not mutate its input", () => {
		const input = [
			{ name: "Debian", default: false },
			{ name: "Arch", default: true },
		];
		sortDistrosDefaultFirst(input);
		expect(input.map((d) => d.name)).toEqual(["Debian", "Arch"]);
	});
});
