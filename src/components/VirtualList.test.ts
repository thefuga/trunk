import { render } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import VirtualList from "./VirtualList.svelte";

// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

// Mock esm-env — BROWSER must be true for VirtualList to initialize
vi.mock("esm-env", () => ({
	BROWSER: true,
	DEV: false,
}));

describe("VirtualList", () => {
	// jsdom limitations:
	// - scrollTop, offsetHeight, scrollHeight are all 0 in jsdom
	// - ResizeObserver is stubbed in vitest-setup.ts (observe/unobserve/disconnect are no-ops)
	// - getBoundingClientRect returns zero-sized rects
	// These limitations mean we cannot test scroll behavior or viewport-based rendering.
	// Tests verify the component mounts without errors and renders basic DOM structure.

	it("renders without crashing with empty items", () => {
		const { container } = render(VirtualList, {
			props: {
				items: [],
				renderItem: (() => {}) as any,
			},
		});
		expect(
			container.querySelector(".virtual-list-container"),
		).toBeInTheDocument();
	});

	it("renders container and viewport structure", () => {
		const { container } = render(VirtualList, {
			props: {
				items: ["a", "b", "c"],
				renderItem: (() => {}) as any,
			},
		});
		expect(
			container.querySelector(".virtual-list-container"),
		).toBeInTheDocument();
		expect(
			container.querySelector(".virtual-list-viewport"),
		).toBeInTheDocument();
		expect(
			container.querySelector(".virtual-list-content"),
		).toBeInTheDocument();
		expect(container.querySelector(".virtual-list-items")).toBeInTheDocument();
	});

	it("renders items div with transform style", () => {
		const { container } = render(VirtualList, {
			props: {
				items: Array.from({ length: 10 }, (_, i) => `item-${i}`),
				renderItem: (() => {}) as any,
				defaultEstimatedItemHeight: 40,
			},
		});
		const itemsDiv = container.querySelector(".virtual-list-items");
		expect(itemsDiv).toBeInTheDocument();
		// The transform should be set (translateY)
		const style = itemsDiv?.getAttribute("style") ?? "";
		expect(style).toContain("transform");
	});
});
