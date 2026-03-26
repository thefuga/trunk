import "@testing-library/jest-dom/vitest";

// jsdom does not implement ResizeObserver — stub it globally for VirtualList etc.
if (typeof globalThis.ResizeObserver === "undefined") {
	globalThis.ResizeObserver = class {
		observe() {}
		unobserve() {}
		disconnect() {}
	} as unknown as typeof ResizeObserver;
}

// jsdom does not implement Element.prototype.animate — stub for Svelte transitions (slide, fly, etc.)
if (typeof Element.prototype.animate === "undefined") {
	Element.prototype.animate = () =>
		({
			finished: Promise.resolve(),
			cancel() {},
			play() {},
			pause() {},
			reverse() {},
			addEventListener() {},
			removeEventListener() {},
			onfinish: null,
			oncancel: null,
		}) as unknown as Animation;
}
