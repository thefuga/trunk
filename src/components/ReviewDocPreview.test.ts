// Phase 71-01 Wave 1 — Copy button TDD suite. Each `it` block maps 1:1 to a
// row in 71-VALIDATION.md's Per-Task Verification Map (test IDs 71-01-01..08).
// Names MUST match the `-t` selectors verbatim.
//
// Mocks: @tauri-apps/plugin-clipboard-manager (writeText) — pattern from
// CommitGraph.test.ts:56-58; ../lib/toast.svelte.js (showToast) — pattern from
// ReviewPanel.test.ts:24-26. Both are boundary mocks (not domain logic) and
// fall under the established carve-out documented in 71-PATTERNS.md.
//
// Timer discipline: vi.useFakeTimers() is active per-test. Microtask flush
// uses `await Promise.resolve()` and/or `await tick()` — NEVER
// `setTimeout(r, 0)`, which deadlocks under fake timers.
import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { showToast } from "../lib/toast.svelte.js";
import ReviewDocPreview from "./ReviewDocPreview.svelte";

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
	writeText: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));

beforeEach(() => {
	vi.clearAllMocks();
	vi.useFakeTimers();
});

afterEach(() => {
	vi.useRealTimers();
});

describe("ReviewDocPreview", () => {
	function renderPreview(markdown: string, onBack: () => void = vi.fn()) {
		return render(ReviewDocPreview, { props: { markdown, onBack } });
	}

	// Microtask flush helper — safe under fake timers (no setTimeout(0)).
	async function flush() {
		await Promise.resolve();
		await tick();
	}

	// `Copy` vs `Copied` share only the "Cop" prefix (no "y" in "Copied"!), so a
	// substring match on `/Copy/` would NOT match the success-state button. Use
	// `/Cop(y|ied)/` to cover both states via a single accessor.
	function getCopyButton() {
		return screen.getByRole("button", { name: /Cop(y|ied)/ });
	}

	function getBackButton() {
		return screen.getByRole("button", { name: /Back to comments/ });
	}

	it("writes markdown prop", async () => {
		renderPreview("hello world");
		await fireEvent.click(getCopyButton());
		await flush();
		expect(vi.mocked(writeText)).toHaveBeenCalledTimes(1);
		expect(vi.mocked(writeText)).toHaveBeenCalledWith("hello world");
	});

	it("shows Copied affordance on success", async () => {
		renderPreview("the doc");
		// Before the click the button reads "Copy".
		expect(screen.getByRole("button", { name: /Copy/ })).toHaveTextContent(
			/^Copy$/,
		);
		await fireEvent.click(getCopyButton());
		await flush();
		expect(
			screen.getByRole("button", { name: /Copied/ }),
		).toHaveTextContent(/Copied/);
	});

	it("reverts after timeout", async () => {
		renderPreview("the doc");
		await fireEvent.click(getCopyButton());
		await flush();
		expect(
			screen.getByRole("button", { name: /Copied/ }),
		).toHaveTextContent(/Copied/);
		vi.advanceTimersByTime(1500);
		await tick();
		expect(screen.getByRole("button", { name: /Copy/ })).toHaveTextContent(
			/^Copy$/,
		);
	});

	it("remains clickable during window", async () => {
		renderPreview("the doc");
		// First click at virtual t=0.
		await fireEvent.click(getCopyButton());
		await flush();
		expect(
			screen.getByRole("button", { name: /Copied/ }),
		).toHaveTextContent(/Copied/);

		// Mid-window second click at virtual t=500.
		vi.advanceTimersByTime(500);
		await fireEvent.click(getCopyButton());
		await flush();

		// If the FIRST timer were still alive it would fire at t=1500
		// (we're at t=500 + 1499 = t=1999). Advance 1499 and assert still Copied.
		vi.advanceTimersByTime(1499);
		await tick();
		expect(
			screen.getByRole("button", { name: /Copied/ }),
		).toHaveTextContent(/Copied/);

		// Second timer fires at t=500 + 1500 = t=2000.
		vi.advanceTimersByTime(1);
		await tick();
		expect(screen.getByRole("button", { name: /Copy/ })).toHaveTextContent(
			/^Copy$/,
		);
	});

	it("shows error toast on failure", async () => {
		vi.mocked(writeText).mockRejectedValueOnce(new Error("plugin disabled"));
		renderPreview("the doc");
		await fireEvent.click(getCopyButton());
		await flush();
		expect(vi.mocked(showToast)).toHaveBeenCalledWith(
			"Failed to copy: plugin disabled",
			"error",
		);
	});

	it("does not flip copied on failure", async () => {
		vi.mocked(writeText).mockRejectedValueOnce(new Error("plugin disabled"));
		renderPreview("the doc");
		await fireEvent.click(getCopyButton());
		await flush();
		// Button text must still be Copy — never Copied — on the failure path.
		expect(screen.getByRole("button", { name: /Copy/ })).toHaveTextContent(
			/^Copy$/,
		);
		expect(
			screen.queryByRole("button", { name: /Copied/ }),
		).not.toBeInTheDocument();
	});

	it("coerces non-Error rejection", async () => {
		vi.mocked(writeText).mockRejectedValueOnce("raw string");
		renderPreview("the doc");
		await fireEvent.click(getCopyButton());
		await flush();
		expect(vi.mocked(showToast)).toHaveBeenCalledWith(
			"Failed to copy: raw string",
			"error",
		);
	});

	it("back button still invokes onBack", async () => {
		const onBack = vi.fn();
		renderPreview("the doc", onBack);
		await fireEvent.click(getBackButton());
		await flush();
		expect(onBack).toHaveBeenCalledTimes(1);
	});
});
