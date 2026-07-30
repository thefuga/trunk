import { describe, expect, it, vi } from "vitest";
import {
	handleRecentOpenFailure,
	isDefinitiveRepoError,
	repoErrorMessage,
	restoreFailureMessage,
} from "./repo-lifecycle.js";
import { localRepoDescriptor, wslRepoDescriptor } from "./types.js";

describe("repository lifecycle failures", () => {
	it("keeps a recent and surfaces a cold-WSL transient error", async () => {
		const remove = vi.fn(async () => {});
		const error = {
			code: "wsl_unavailable",
			message: "WSL is still starting",
		};

		const message = await handleRecentOpenFailure(error, remove);

		expect(message).toBe("WSL is still starting");
		expect(remove).not.toHaveBeenCalled();
	});

	it.each([
		"wsl_repo_invalid",
		"wsl_missing_distro",
	])("removes a recent for definitive %s failures", async (code) => {
		const remove = vi.fn(async () => {});
		await handleRecentOpenFailure({ code, message: "Gone" }, remove);
		expect(remove).toHaveBeenCalledOnce();
	});

	it("distinguishes transient WSL validation failures from absence", () => {
		expect(
			isDefinitiveRepoError({
				code: "wsl_io_error",
				message: "WSL is cold",
			}),
		).toBe(false);
		expect(
			isDefinitiveRepoError({
				code: "wsl_repo_invalid",
				message: "Not a repository",
			}),
		).toBe(true);
	});

	it("provides a visible fallback for malformed errors", () => {
		expect(repoErrorMessage(new Error("opaque IPC error"))).toBe(
			"Failed to open repository",
		);
	});

	it("keeps a cold-WSL restored tab with a visible retry error", () => {
		const message = restoreFailureMessage(
			wslRepoDescriptor("Ubuntu", "/home/me/trunk"),
			{ code: "wsl_unavailable", message: "WSL is still starting" },
		);

		expect(message).toBe("WSL is still starting");
	});

	it("retains the existing local-tab skip behavior", () => {
		expect(
			restoreFailureMessage(localRepoDescriptor("/gone", "gone"), {
				code: "git_error",
				message: "not found",
			}),
		).toBeNull();
	});
});
