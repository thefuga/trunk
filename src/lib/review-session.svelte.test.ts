import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createReviewSession } from "./review-session.svelte.js";

// safeInvoke is a thin wrapper around @tauri-apps/api/core::invoke (src/lib/invoke.ts).
// Mocking the underlying invoke (not safeInvoke itself) keeps the TrunkError-parsing
// path live in the test, matching the project's pattern (src/lib/invoke.test.ts:5-9).
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
	mockInvoke.mockReset();
});

describe("createReviewSession — preview state", () => {
	it("starts with panelMode 'list' and previewMarkdown null", () => {
		const m = createReviewSession();
		expect(m.state.panelMode).toBe("list");
		expect(m.state.previewMarkdown).toBeNull();
	});

	it("showPreview sets previewMarkdown and switches panelMode to 'preview'", () => {
		const m = createReviewSession();
		m.showPreview("# hello\nworld");
		expect(m.state.previewMarkdown).toBe("# hello\nworld");
		expect(m.state.panelMode).toBe("preview");
	});

	it("showList returns panelMode to 'list' and preserves previewMarkdown", () => {
		const m = createReviewSession();
		m.showPreview("# cached");
		m.showList();
		expect(m.state.panelMode).toBe("list");
		// Regenerate is the only invalidation path; swap-back keeps the cached doc.
		expect(m.state.previewMarkdown).toBe("# cached");
	});

	it("generate awaits safeInvoke for generate_review_doc and stores the result", async () => {
		mockInvoke.mockResolvedValueOnce("# generated markdown");
		const m = createReviewSession();
		await m.generate("/some/path");
		expect(mockInvoke).toHaveBeenCalledWith("generate_review_doc", {
			path: "/some/path",
		});
		expect(m.state.previewMarkdown).toBe("# generated markdown");
		expect(m.state.panelMode).toBe("preview");
	});

	it("setReviewActive(false) clears previewMarkdown and resets panelMode to 'list'", () => {
		const m = createReviewSession();
		m.showPreview("# stale");
		m.setReviewActive(false);
		expect(m.state.previewMarkdown).toBeNull();
		expect(m.state.panelMode).toBe("list");
	});

	it("setReviewActive(true) does NOT touch preview fields", () => {
		const m = createReviewSession();
		m.showPreview("# kept");
		m.setReviewActive(true);
		expect(m.state.previewMarkdown).toBe("# kept");
		expect(m.state.panelMode).toBe("preview");
	});

	it("generate propagates rejection and leaves state untouched", async () => {
		mockInvoke.mockRejectedValueOnce(
			'{"code":"no_comments","message":"Generate requires at least one comment in the session"}',
		);
		const m = createReviewSession();
		// Seed a non-default state so we can prove no partial update happened.
		m.showPreview("# previous");
		m.showList();
		await expect(m.generate("/repo")).rejects.toMatchObject({
			code: "no_comments",
		});
		expect(m.state.panelMode).toBe("list");
		expect(m.state.previewMarkdown).toBe("# previous");
	});
});
