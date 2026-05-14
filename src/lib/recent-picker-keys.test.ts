import { describe, expect, it } from "vitest";
import {
	clampHighlightedIdx,
	nextHighlightedIdx,
	pickerKeyAction,
} from "./recent-picker-keys.js";

describe("pickerKeyAction", () => {
	it("maps ArrowDown to highlight-down and prevents default", () => {
		expect(pickerKeyAction({ key: "ArrowDown", queryEmpty: false })).toEqual({
			action: { kind: "highlight-down" },
			preventDefault: true,
		});
	});

	it("maps ArrowUp to highlight-up and prevents default", () => {
		expect(pickerKeyAction({ key: "ArrowUp", queryEmpty: false })).toEqual({
			action: { kind: "highlight-up" },
			preventDefault: true,
		});
	});

	it("maps Enter to pick and prevents default", () => {
		expect(pickerKeyAction({ key: "Enter", queryEmpty: false })).toEqual({
			action: { kind: "pick" },
			preventDefault: true,
		});
	});

	it("maps Escape to close and prevents default", () => {
		expect(pickerKeyAction({ key: "Escape", queryEmpty: false })).toEqual({
			action: { kind: "close" },
			preventDefault: true,
		});
	});

	it("maps Backspace on an empty query to close without preventing default", () => {
		// The browser's default for Backspace on an empty input is a no-op, so
		// we close the picker but let the event proceed normally.
		expect(pickerKeyAction({ key: "Backspace", queryEmpty: true })).toEqual({
			action: { kind: "close" },
			preventDefault: false,
		});
	});

	it("ignores Backspace when the query is not empty (browser deletes the char)", () => {
		expect(pickerKeyAction({ key: "Backspace", queryEmpty: false })).toEqual({
			action: { kind: "ignore" },
			preventDefault: false,
		});
	});

	it("ignores any other key", () => {
		expect(pickerKeyAction({ key: "a", queryEmpty: false })).toEqual({
			action: { kind: "ignore" },
			preventDefault: false,
		});
		expect(pickerKeyAction({ key: "Tab", queryEmpty: true })).toEqual({
			action: { kind: "ignore" },
			preventDefault: false,
		});
	});
});

describe("nextHighlightedIdx", () => {
	it("moves down one step within bounds", () => {
		expect(nextHighlightedIdx("down", 0, 5)).toBe(1);
		expect(nextHighlightedIdx("down", 2, 5)).toBe(3);
	});

	it("clamps down at the last index", () => {
		expect(nextHighlightedIdx("down", 4, 5)).toBe(4);
	});

	it("moves up one step within bounds", () => {
		expect(nextHighlightedIdx("up", 3, 5)).toBe(2);
		expect(nextHighlightedIdx("up", 1, 5)).toBe(0);
	});

	it("clamps up at zero", () => {
		expect(nextHighlightedIdx("up", 0, 5)).toBe(0);
	});

	it("returns zero when the list is empty (no rows to land on)", () => {
		expect(nextHighlightedIdx("down", 0, 0)).toBe(0);
		expect(nextHighlightedIdx("up", 0, 0)).toBe(0);
	});
});

describe("clampHighlightedIdx", () => {
	it("returns the index unchanged when in bounds", () => {
		expect(clampHighlightedIdx(2, 5)).toBe(2);
		expect(clampHighlightedIdx(0, 5)).toBe(0);
		expect(clampHighlightedIdx(4, 5)).toBe(4);
	});

	it("clamps to the last index when the list shrank below the current highlight", () => {
		expect(clampHighlightedIdx(5, 3)).toBe(2);
		expect(clampHighlightedIdx(100, 1)).toBe(0);
	});

	it("returns zero when the list is empty", () => {
		expect(clampHighlightedIdx(3, 0)).toBe(0);
		expect(clampHighlightedIdx(0, 0)).toBe(0);
	});
});
