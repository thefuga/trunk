import { render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Toast from "./Toast.svelte";
import "../__tests__/helpers/tauri-mock";
import { _resetToasts, showToast } from "../lib/toast.svelte.js";

describe("Toast", () => {
	beforeEach(() => {
		_resetToasts();
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it("renders nothing when no toasts", () => {
		render(Toast);
		expect(screen.queryByRole("status")).toBeNull();
	});

	it("renders toast message with role=status", () => {
		showToast("Hello", "success");
		render(Toast);
		const status = screen.getByRole("status");
		expect(status).toHaveTextContent("Hello");
	});

	it("renders error toast message", () => {
		showToast("Fail", "error");
		render(Toast);
		const status = screen.getByRole("status");
		expect(status).toHaveTextContent("Fail");
	});

	it("renders multiple toasts", () => {
		showToast("First", "success");
		showToast("Second", "error");
		render(Toast);
		const statuses = screen.getAllByRole("status");
		expect(statuses).toHaveLength(2);
		expect(statuses[0]).toHaveTextContent("First");
		expect(statuses[1]).toHaveTextContent("Second");
	});
});
