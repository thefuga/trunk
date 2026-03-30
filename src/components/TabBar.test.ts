import { fireEvent, render, screen } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TabInfo } from "../lib/tab-types.js";
import TabBar from "./TabBar.svelte";

// Shared Tauri mock
import "../__tests__/helpers/tauri-mock";

// Mock sortablejs — SortableJS manipulates DOM directly, not testable in jsdom
// TabBar imports `Sortable` as default and calls `Sortable.create()`
vi.mock("sortablejs", () => {
	const sortableInstance = { destroy: vi.fn() };
	const SortableMock = Object.assign(
		vi.fn().mockImplementation(() => sortableInstance),
		{ create: vi.fn().mockReturnValue(sortableInstance) },
	);
	return { default: SortableMock };
});

// jsdom does not implement scrollIntoView — stub it globally
beforeEach(() => {
	Element.prototype.scrollIntoView = vi.fn();
});

const tabs: TabInfo[] = [
	{ id: "1", repoPath: "/path/to/trunk", repoName: "trunk", dirty: false },
	{ id: "2", repoPath: "/path/to/other", repoName: "other", dirty: true },
];

describe("TabBar", () => {
	const defaultProps = {
		tabs,
		activeTabId: "1",
		onactivate: vi.fn(),
		onclose: vi.fn(),
		onnew: vi.fn(),
		oncontextmenu: vi.fn(),
		onauxclose: vi.fn(),
		onreorder: vi.fn(),
	};

	it("renders tab names", () => {
		render(TabBar, { props: defaultProps });
		expect(screen.getByText("trunk")).toBeInTheDocument();
		expect(screen.getByText("other")).toBeInTheDocument();
	});

	it("highlights active tab", () => {
		render(TabBar, { props: defaultProps });
		const activeTab = screen.getByText("trunk").closest(".tab-item");
		expect(activeTab?.classList.contains("active")).toBe(true);

		const inactiveTab = screen.getByText("other").closest(".tab-item");
		expect(inactiveTab?.classList.contains("active")).toBe(false);
	});

	it("renders new tab button", () => {
		render(TabBar, { props: defaultProps });
		expect(screen.getByLabelText("New tab")).toBeInTheDocument();
	});

	it("calls onactivate when tab clicked", async () => {
		const onactivate = vi.fn();
		render(TabBar, {
			props: { ...defaultProps, onactivate },
		});
		// Click the inactive tab (role="tab")
		const otherTab = screen.getByText("other").closest('[role="tab"]');
		expect(otherTab).toBeTruthy();
		await fireEvent.click(otherTab as Element);
		expect(onactivate).toHaveBeenCalledWith("2");
	});

	it("calls onclose when tab close button clicked", async () => {
		const onclose = vi.fn();
		render(TabBar, {
			props: { ...defaultProps, onclose },
		});
		// Close buttons have aria-label "Close tab"
		const closeBtns = screen.getAllByLabelText("Close tab");
		await fireEvent.click(closeBtns[0]);
		expect(onclose).toHaveBeenCalledWith("1", false);
	});

	it("calls onnew when new tab button clicked", async () => {
		const onnew = vi.fn();
		render(TabBar, {
			props: { ...defaultProps, onnew },
		});
		await fireEvent.click(screen.getByLabelText("New tab"));
		expect(onnew).toHaveBeenCalledOnce();
	});

	it("renders dirty dot for dirty tabs", () => {
		const { container } = render(TabBar, { props: defaultProps });
		// "other" tab is dirty, should have a dirty-dot element
		const dirtyDots = container.querySelectorAll(".dirty-dot");
		expect(dirtyDots.length).toBe(1);
	});

	it("marks active tab with aria-selected", () => {
		render(TabBar, { props: defaultProps });
		const activeTabs = screen.getAllByRole("tab");
		const activeTab = activeTabs.find(
			(t) => t.getAttribute("aria-selected") === "true",
		);
		expect(activeTab).toBeTruthy();
		expect(activeTab?.textContent).toContain("trunk");
	});
});
