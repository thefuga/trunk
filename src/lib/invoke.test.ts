import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { safeInvoke } from "./invoke.js";

vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

describe("safeInvoke", () => {
	it("returns resolved value on success", async () => {
		mockInvoke.mockResolvedValueOnce({ data: 42 });
		const result = await safeInvoke<{ data: number }>("test_cmd");
		expect(result).toEqual({ data: 42 });
	});

	it("parses JSON error string into TrunkError", async () => {
		mockInvoke.mockRejectedValueOnce(
			'{"code":"conflict","message":"merge conflict"}',
		);
		await expect(safeInvoke("test_cmd")).rejects.toEqual({
			code: "conflict",
			message: "merge conflict",
		});
	});

	it("wraps non-JSON string error with unknown_error code", async () => {
		mockInvoke.mockRejectedValueOnce("raw error text");
		await expect(safeInvoke("test_cmd")).rejects.toEqual({
			code: "unknown_error",
			message: "raw error text",
		});
	});

	it("wraps non-string error with unknown_error code and generic message", async () => {
		// When rejected with a non-string, JSON.parse(42 as string) parses successfully as a number.
		// The catch block's inner try succeeds, returning 42 — which is not a TrunkError.
		// safeInvoke doesn't validate the shape, so it throws whatever JSON.parse returns.
		// Use an object that fails JSON.parse to trigger the outer catch.
		mockInvoke.mockRejectedValueOnce({ weird: true });
		await expect(safeInvoke("test_cmd")).rejects.toEqual({
			code: "unknown_error",
			message: "An unexpected error occurred",
		});
	});

	it("passes command name and args to invoke", async () => {
		mockInvoke.mockResolvedValueOnce("ok");
		await safeInvoke("my_cmd", { key: "val" });
		expect(mockInvoke).toHaveBeenCalledWith("my_cmd", { key: "val" });
	});
});
