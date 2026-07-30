// Shared Tauri mock (mocks invoke, dialog, plugin-store, etc.). Must be
// imported before the component so the module mocks register first.
import "../__tests__/helpers/tauri-mock";

import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { _resetToasts, toasts } from "../lib/toast.svelte.js";
import OpenRepoButton from "./OpenRepoButton.svelte";

// Keep the real isTrunkError; only stub the IPC boundary.
vi.mock("../lib/invoke.js", async (importOriginal) => ({
	...(await importOriginal<typeof import("../lib/invoke.js")>()),
	safeInvoke: vi.fn(),
}));

const AVAILABLE = {
	available: true,
	supported_platform: true,
	message: null,
};

async function mockWsl(
	distros: { name: string; default: boolean }[],
	extra?: (cmd: string, args?: Record<string, unknown>) => unknown,
) {
	const { safeInvoke } = await import("../lib/invoke.js");
	vi.mocked(safeInvoke).mockImplementation(
		(cmd: string, args?: Record<string, unknown>) => {
			if (cmd === "wsl_availability") return Promise.resolve(AVAILABLE);
			if (cmd === "list_wsl_distros") return Promise.resolve(distros);
			const handled = extra?.(cmd, args);
			if (handled !== undefined) return Promise.resolve(handled);
			return Promise.resolve(undefined);
		},
	);
	return vi.mocked(safeInvoke);
}

async function openMenu(): Promise<HTMLElement> {
	const button = screen.getByRole("button", { name: /Open Repository/ });
	await vi.waitFor(() => {
		expect(button).toHaveAttribute("aria-haspopup", "menu");
	});
	await fireEvent.click(button);
	return screen.getByRole("menu");
}

describe("OpenRepoButton", () => {
	beforeEach(async () => {
		const { safeInvoke } = await import("../lib/invoke.js");
		vi.mocked(safeInvoke).mockReset();
		vi.mocked(safeInvoke).mockResolvedValue(undefined);
		vi.mocked(openDialog).mockReset();
		vi.mocked(openDialog).mockResolvedValue(null);
		_resetToasts();
	});

	it("opens the local dialog directly when WSL is unavailable", async () => {
		const { safeInvoke } = await import("../lib/invoke.js");
		vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
			if (cmd === "wsl_availability") {
				return Promise.resolve({
					available: false,
					supported_platform: true,
					message: "WSL is not installed.",
				});
			}
			return Promise.resolve(undefined);
		});
		const onpick = vi.fn();
		render(OpenRepoButton, { props: { onpick } });

		await fireEvent.click(
			screen.getByRole("button", { name: "Open Repository" }),
		);

		await vi.waitFor(() => {
			expect(vi.mocked(openDialog)).toHaveBeenCalledWith({
				directory: true,
				multiple: false,
			});
		});
		expect(screen.queryByRole("menu")).not.toBeInTheDocument();
		expect(onpick).not.toHaveBeenCalled();
	});

	it("opens the local dialog directly when no distros are installed", async () => {
		await mockWsl([]);
		render(OpenRepoButton, { props: { onpick: vi.fn() } });

		await fireEvent.click(
			screen.getByRole("button", { name: "Open Repository" }),
		);

		await vi.waitFor(() => {
			expect(vi.mocked(openDialog)).toHaveBeenCalled();
		});
		expect(screen.queryByRole("menu")).not.toBeInTheDocument();
	});

	it("lists Local first, then distros with the default marked", async () => {
		await mockWsl([
			{ name: "Debian", default: false },
			{ name: "Arch", default: true },
		]);
		render(OpenRepoButton, { props: { onpick: vi.fn() } });

		await openMenu();

		const items = screen
			.getAllByRole("menuitem")
			.map((el) => el.textContent?.trim());
		expect(items).toEqual(["Local", "Arch (default)", "Debian"]);
	});

	it("opens the dialog at the distro home directory", async () => {
		await mockWsl([{ name: "Arch", default: true }], (cmd) => {
			if (cmd === "wsl_default_open_path") {
				return "\\\\wsl.localhost\\Arch\\home\\me";
			}
		});
		render(OpenRepoButton, { props: { onpick: vi.fn() } });

		await openMenu();
		await fireEvent.click(screen.getByRole("menuitem", { name: /Arch/ }));

		await vi.waitFor(() => {
			expect(vi.mocked(openDialog)).toHaveBeenCalledWith({
				directory: true,
				multiple: false,
				defaultPath: "\\\\wsl.localhost\\Arch\\home\\me",
			});
		});
	});

	it("falls back to the distro root when the home probe fails", async () => {
		const { safeInvoke } = await import("../lib/invoke.js");
		vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
			if (cmd === "wsl_availability") return Promise.resolve(AVAILABLE);
			if (cmd === "list_wsl_distros") {
				return Promise.resolve([{ name: "Arch", default: true }]);
			}
			if (cmd === "wsl_default_open_path") {
				return Promise.reject({ code: "wsl_home_failed", message: "no home" });
			}
			return Promise.resolve(undefined);
		});
		render(OpenRepoButton, { props: { onpick: vi.fn() } });

		await openMenu();
		await fireEvent.click(screen.getByRole("menuitem", { name: /Arch/ }));

		await vi.waitFor(() => {
			expect(vi.mocked(openDialog)).toHaveBeenCalledWith({
				directory: true,
				multiple: false,
				defaultPath: "\\\\wsl.localhost\\Arch",
			});
		});
	});

	it("emits a local descriptor with the right name for backslash paths", async () => {
		vi.mocked(openDialog).mockResolvedValue("C:\\code\\my-repo");
		const onpick = vi.fn();
		render(OpenRepoButton, { props: { onpick } });

		await fireEvent.click(
			screen.getByRole("button", { name: "Open Repository" }),
		);

		await vi.waitFor(() => {
			expect(onpick).toHaveBeenCalledWith({
				name: "my-repo",
				path: "C:\\code\\my-repo",
				repoId: "local:C:\\code\\my-repo",
				repoDescriptor: expect.objectContaining({
					display_name: "my-repo",
					locator: { backend: "Local", path: "C:\\code\\my-repo" },
				}),
			});
		});
	});

	it("routes WSL UNC picks through validate_wsl_repo even from Local", async () => {
		const descriptor = {
			id: "wsl:Arch:/home/me/trunk",
			display_name: "trunk",
			display_path: "Arch:/home/me/trunk",
			locator: {
				backend: "Wsl" as const,
				distro: "Arch",
				linux_path: "/home/me/trunk",
			},
		};
		const safeInvoke = await mockWsl(
			[{ name: "Arch", default: true }],
			(cmd, args) => {
				if (cmd === "validate_wsl_repo") {
					expect(args).toEqual({ distro: "Arch", linuxPath: "/home/me/trunk" });
					return {
						distro: "Arch",
						linux_path: "/home/me/trunk",
						repo_root: "/home/me/trunk",
						descriptor,
					};
				}
			},
		);
		vi.mocked(openDialog).mockResolvedValue(
			"\\\\wsl.localhost\\Arch\\home\\me\\trunk",
		);
		const onpick = vi.fn();
		render(OpenRepoButton, { props: { onpick } });

		await openMenu();
		await fireEvent.click(screen.getByRole("menuitem", { name: "Local" }));

		await vi.waitFor(() => {
			expect(onpick).toHaveBeenCalledWith({
				name: "trunk",
				path: "Arch:/home/me/trunk",
				repoId: descriptor.id,
				repoDescriptor: descriptor,
			});
		});
		expect(safeInvoke).toHaveBeenCalledWith("validate_wsl_repo", {
			distro: "Arch",
			linuxPath: "/home/me/trunk",
		});
	});

	it("reports validation failures through onerror", async () => {
		await mockWsl([{ name: "Arch", default: true }], (cmd) => {
			if (cmd === "validate_wsl_repo") {
				return Promise.reject({
					code: "wsl_repo_invalid",
					message: "not a repo",
				});
			}
		});
		vi.mocked(openDialog).mockResolvedValue(
			"\\\\wsl.localhost\\Arch\\home\\me\\not-a-repo",
		);
		const onpick = vi.fn();
		const onerror = vi.fn();
		render(OpenRepoButton, { props: { onpick, onerror } });

		await openMenu();
		await fireEvent.click(screen.getByRole("menuitem", { name: /Arch/ }));

		await vi.waitFor(() => {
			expect(onerror).toHaveBeenCalledWith("not a repo");
		});
		expect(onpick).not.toHaveBeenCalled();
	});

	it("falls back to a toast when no onerror is given", async () => {
		await mockWsl([{ name: "Arch", default: true }], (cmd) => {
			if (cmd === "validate_wsl_repo") {
				return Promise.reject({
					code: "wsl_repo_invalid",
					message: "not a repo",
				});
			}
		});
		vi.mocked(openDialog).mockResolvedValue(
			"\\\\wsl.localhost\\Arch\\home\\me\\not-a-repo",
		);
		render(OpenRepoButton, { props: { onpick: vi.fn() } });

		await openMenu();
		await fireEvent.click(screen.getByRole("menuitem", { name: /Arch/ }));

		await vi.waitFor(() => {
			expect(toasts.items).toEqual([
				expect.objectContaining({ message: "not a repo", kind: "error" }),
			]);
		});
	});

	it("does not emit a pick when the dialog is cancelled", async () => {
		await mockWsl([{ name: "Arch", default: true }]);
		vi.mocked(openDialog).mockResolvedValue(null);
		const onpick = vi.fn();
		render(OpenRepoButton, { props: { onpick } });

		await openMenu();
		await fireEvent.click(screen.getByRole("menuitem", { name: "Local" }));

		await vi.waitFor(() => {
			expect(vi.mocked(openDialog)).toHaveBeenCalled();
		});
		expect(onpick).not.toHaveBeenCalled();
	});

	it("closes the menu on Escape", async () => {
		await mockWsl([{ name: "Arch", default: true }]);
		render(OpenRepoButton, { props: { onpick: vi.fn() } });

		await openMenu();
		await fireEvent.keyDown(window, { key: "Escape" });

		expect(screen.queryByRole("menu")).not.toBeInTheDocument();
	});
});
