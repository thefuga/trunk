// Phase 71-01 Wave 0 scaffold for the ReviewDocPreview Copy button TDD cycle.
// Wave 1 (this same plan, Task 2) populates this describe with eight `it`
// blocks — one per row of 71-VALIDATION.md's Per-Task Verification Map.
// The mocks below are verbatim from CommitGraph.test.ts (clipboard) and
// ReviewPanel.test.ts (toast); both are documented boundary-mock carve-outs.
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
	// Helper: render with the standard Props shape. Tests reach for this in W1.
	// CRITICAL: do NOT introduce a setTimeout(r, 0) flush helper — that pattern
	// deadlocks under vi.useFakeTimers. Flush microtasks with
	// `await Promise.resolve()` and/or `await tick()` directly inside tests.
	function renderPreview(markdown: string, onBack: () => void = vi.fn()) {
		return render(ReviewDocPreview, { props: { markdown, onBack } });
	}

	// W1 RED→GREEN cycles add the eight `it` blocks here, in this order:
	//   71-01-01 writes markdown prop
	//   71-01-02 shows Copied affordance on success
	//   71-01-03 reverts after timeout
	//   71-01-04 remains clickable during window
	//   71-01-05 shows error toast on failure
	//   71-01-06 does not flip copied on failure
	//   71-01-07 coerces non-Error rejection
	//   71-01-08 back button still invokes onBack
	// The names above MUST match the `-t` selectors in 71-VALIDATION.md.
	void renderPreview;
	void fireEvent;
	void screen;
	void tick;
	void writeText;
	void showToast;
});
