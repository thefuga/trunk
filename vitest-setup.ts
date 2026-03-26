import "@testing-library/jest-dom/vitest";

// jsdom does not implement ResizeObserver — stub it globally for VirtualList etc.
if (typeof globalThis.ResizeObserver === "undefined") {
	globalThis.ResizeObserver = class {
		observe() {}
		unobserve() {}
		disconnect() {}
	} as unknown as typeof ResizeObserver;
}
