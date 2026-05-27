import { invoke } from "@tauri-apps/api/core";

// Tauri IPC errors arrive as raw strings, not Error objects.
// catch(e) { e.message } returns undefined — this wrapper fixes that.
export interface TrunkError {
	code: string;
	message: string;
}

// Type guard for the TrunkError shape thrown by safeInvoke (a plain object with
// string `code` + `message`, NOT an Error subclass). Used in catch blocks to
// surface `.message` and branch on `.code` without an unchecked `as` cast.
export function isTrunkError(e: unknown): e is TrunkError {
	return (
		typeof e === "object" &&
		e !== null &&
		"code" in e &&
		"message" in e &&
		typeof (e as { message: unknown }).message === "string"
	);
}

export async function safeInvoke<T>(
	cmd: string,
	args?: Record<string, unknown>,
): Promise<T> {
	try {
		return await invoke<T>(cmd, args);
	} catch (e: unknown) {
		let parsed: TrunkError;
		try {
			parsed = JSON.parse(e as string) as TrunkError;
		} catch {
			parsed = {
				code: "unknown_error",
				message: typeof e === "string" ? e : "An unexpected error occurred",
			};
		}
		throw parsed;
	}
}
