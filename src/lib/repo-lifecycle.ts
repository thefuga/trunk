import { isTrunkError } from "./invoke.js";
import type { RepoDescriptor } from "./types.js";

const DEFINITIVE_REPO_ERRORS = new Set([
	"git_error",
	"repo_invalid",
	"repo_not_found",
	"wsl_invalid_path",
	"wsl_missing_distro",
	"wsl_repo_invalid",
]);

export function repoErrorMessage(error: unknown): string {
	return isTrunkError(error) ? error.message : "Failed to open repository";
}

export function isDefinitiveRepoError(error: unknown): boolean {
	return isTrunkError(error) && DEFINITIVE_REPO_ERRORS.has(error.code);
}

export async function handleRecentOpenFailure(
	error: unknown,
	remove: () => Promise<void>,
): Promise<string> {
	if (isDefinitiveRepoError(error)) await remove();
	return repoErrorMessage(error);
}

export function restoreFailureMessage(
	descriptor: RepoDescriptor | null,
	error: unknown,
): string | null {
	return descriptor?.locator.backend === "Wsl" ? repoErrorMessage(error) : null;
}
