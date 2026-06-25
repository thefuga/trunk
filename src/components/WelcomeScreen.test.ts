import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import WelcomeScreen from "./WelcomeScreen.svelte";

// Shared Tauri mock (mocks invoke, dialog, plugin-store, etc.)
import "../__tests__/helpers/tauri-mock";

// Explicitly mock @tauri-apps/api/path to prevent real homeDir call
vi.mock("@tauri-apps/api/path", () => ({
	homeDir: vi.fn().mockResolvedValue("/Users/test"),
}));

// Mock store module for getRecentRepos / addRecentRepo / removeRecentRepo
vi.mock("../lib/store.js", () => ({
	getRecentRepos: vi.fn().mockResolvedValue([]),
	addRecentRepo: vi.fn().mockResolvedValue(undefined),
	removeRecentRepo: vi.fn().mockResolvedValue(undefined),
}));

// Mock invoke module — safeInvoke resolves so openPath succeeds
vi.mock("../lib/invoke.js", () => ({
	safeInvoke: vi.fn().mockResolvedValue(undefined),
}));

describe("WelcomeScreen", () => {
	beforeEach(async () => {
		const invokeModule = await import("../lib/invoke.js");
		vi.mocked(invokeModule.safeInvoke).mockReset();
		vi.mocked(invokeModule.safeInvoke).mockResolvedValue(undefined);
	});

	it("renders 'Open Repository' button", () => {
		render(WelcomeScreen, {
			props: { onopen: vi.fn() },
		});
		expect(screen.getByText("Open Repository")).toBeInTheDocument();
	});

	it("renders app title 'Trunk'", () => {
		render(WelcomeScreen, {
			props: { onopen: vi.fn() },
		});
		expect(screen.getByText("Trunk")).toBeInTheDocument();
	});

	it("renders tagline", () => {
		render(WelcomeScreen, {
			props: { onopen: vi.fn() },
		});
		expect(
			screen.getByText("Git history, beautifully visualized"),
		).toBeInTheDocument();
	});

	it("renders recent repos when available", async () => {
		const { getRecentRepos } = await import("../lib/store.js");
		vi.mocked(getRecentRepos).mockResolvedValue([
			{ name: "trunk", path: "/Users/test/code/trunk" },
			{ name: "other", path: "/Users/test/code/other" },
		]);

		render(WelcomeScreen, {
			props: { onopen: vi.fn() },
		});

		// Wait for $effect to run and populate recentRepos
		await vi.waitFor(() => {
			expect(screen.getByText("Recent")).toBeInTheDocument();
		});

		expect(screen.getByText("trunk")).toBeInTheDocument();
	});

	it("calls onopen when recent repo clicked", async () => {
		const storeModule = await import("../lib/store.js");
		vi.mocked(storeModule.getRecentRepos).mockResolvedValue([
			{ name: "trunk", path: "/Users/test/code/trunk" },
		]);
		vi.mocked(storeModule.addRecentRepo).mockResolvedValue(undefined);

		const onopen = vi.fn();
		render(WelcomeScreen, {
			props: { onopen },
		});

		// Wait for recent repos to load
		await vi.waitFor(() => {
			expect(screen.getByText("trunk")).toBeInTheDocument();
		});

		// Click the repo entry (the parent div with role="button")
		const repoButton = screen.getByText("trunk").closest('[role="button"]');
		expect(repoButton).toBeTruthy();
		await fireEvent.click(repoButton as Element);

		// openPath is async (calls safeInvoke then onopen)
		await vi.waitFor(() => {
			expect(onopen).toHaveBeenCalledWith(
				expect.objectContaining({
					name: "trunk",
					path: "/Users/test/code/trunk",
					repoId: "local:/Users/test/code/trunk",
					repoDescriptor: expect.objectContaining({
						id: "local:/Users/test/code/trunk",
						display_name: "trunk",
						display_path: "/Users/test/code/trunk",
						locator: { backend: "Local", path: "/Users/test/code/trunk" },
					}),
				}),
			);
		});
	});

	it("shows WSL unavailable message on supported Windows hosts", async () => {
		const { safeInvoke } = await import("../lib/invoke.js");
		vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
			if (cmd === "wsl_availability") {
				return Promise.resolve({
					available: false,
					supported_platform: true,
					message: "WSL is not installed or `wsl.exe` is not on PATH.",
				});
			}
			return Promise.resolve(undefined);
		});

		render(WelcomeScreen, {
			props: { onopen: vi.fn() },
		});

		await vi.waitFor(() => {
			expect(screen.getByText("Open from WSL")).toBeInTheDocument();
		});
		expect(screen.getByText("Unavailable")).toBeInTheDocument();
		expect(
			screen.getByText("WSL is not installed or `wsl.exe` is not on PATH."),
		).toBeInTheDocument();
	});

	it("opens a validated WSL repository from distro and Linux path", async () => {
		const { safeInvoke } = await import("../lib/invoke.js");
		const storeModule = await import("../lib/store.js");
		const descriptor = {
			id: "wsl:Ubuntu:/home/me/trunk",
			display_name: "trunk",
			display_path: "Ubuntu:/home/me/trunk",
			locator: {
				backend: "Wsl" as const,
				distro: "Ubuntu",
				linux_path: "/home/me/trunk",
			},
		};
		vi.mocked(safeInvoke).mockImplementation(
			(cmd: string, args?: Record<string, unknown>) => {
				if (cmd === "wsl_availability") {
					return Promise.resolve({
						available: true,
						supported_platform: true,
						message: null,
					});
				}
				if (cmd === "list_wsl_distros") {
					return Promise.resolve([{ name: "Ubuntu", default: true }]);
				}
				if (cmd === "validate_wsl_repo") {
					expect(args).toEqual({
						distro: "Ubuntu",
						linuxPath: "/home/me/trunk",
					});
					return Promise.resolve({
						distro: "Ubuntu",
						linux_path: "/home/me/trunk",
						repo_root: "/home/me/trunk",
						descriptor,
					});
				}
				if (cmd === "open_repo") {
					expect(args).toEqual({
						path: descriptor.id,
						repo: descriptor,
					});
					return Promise.resolve(undefined);
				}
				return Promise.resolve(undefined);
			},
		);
		vi.mocked(storeModule.getRecentRepos).mockResolvedValue([]);
		const onopen = vi.fn();

		render(WelcomeScreen, {
			props: { onopen },
		});

		await vi.waitFor(() => {
			expect(screen.getByLabelText("WSL distro")).toBeInTheDocument();
		});
		await fireEvent.input(screen.getByPlaceholderText("/home/me/project"), {
			target: { value: "/home/me/trunk" },
		});
		await fireEvent.click(screen.getByRole("button", { name: "Open" }));

		await vi.waitFor(() => {
			expect(storeModule.addRecentRepo).toHaveBeenCalledWith({
				name: "trunk",
				path: "Ubuntu:/home/me/trunk",
				repoId: descriptor.id,
				repoDescriptor: descriptor,
			});
		});
		await vi.waitFor(() => {
			expect(onopen).toHaveBeenCalledWith({
				name: "trunk",
				path: "Ubuntu:/home/me/trunk",
				repoId: descriptor.id,
				repoDescriptor: descriptor,
			});
		});
	});
});
