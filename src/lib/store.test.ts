import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PersistedTab, TabInfo } from "./tab-types.js";
import { createTabId } from "./tab-types.js";

// The backing store for the mock — shared across all LazyStore instances
const backingStore = new Map<string, unknown>();

vi.mock("@tauri-apps/plugin-store", () => {
	class MockLazyStore {
		get(key: string) {
			return Promise.resolve(backingStore.get(key) ?? null);
		}
		set(key: string, value: unknown) {
			backingStore.set(key, value);
			return Promise.resolve();
		}
		save() {
			return Promise.resolve();
		}
	}
	return { LazyStore: MockLazyStore };
});

// Import store functions after mocking so module-level `new LazyStore(...)` gets the mock
const {
	addRecentRepo,
	getRecentRepos,
	removeRecentRepo,
	getZoomLevel,
	setZoomLevel,
} = await import("./store.js");

describe("tab types and helpers", () => {
	it("createTabId returns UUID v4 format", () => {
		const id = createTabId();
		expect(id).toMatch(
			/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
		);
	});

	it("createTabId returns unique values", () => {
		const ids = new Set(Array.from({ length: 100 }, () => createTabId()));
		expect(ids.size).toBe(100);
	});

	it("TabInfo accepts valid object", () => {
		const tab: TabInfo = {
			id: "abc",
			repoPath: null,
			repoName: "New Tab",
			dirty: false,
		};
		expect(tab.id).toBe("abc");
		expect(tab.repoPath).toBeNull();
		expect(tab.repoName).toBe("New Tab");
		expect(tab.dirty).toBe(false);
	});

	it("PersistedTab is a subset of TabInfo", () => {
		const tab: TabInfo = {
			id: "123",
			repoPath: "/path/to/repo",
			repoName: "repo",
			dirty: true,
		};
		const persisted: PersistedTab = {
			id: tab.id,
			repoPath: tab.repoPath,
			repoName: tab.repoName,
		};
		expect(persisted.id).toBe(tab.id);
		expect(persisted.repoPath).toBe(tab.repoPath);
		expect(persisted.repoName).toBe(tab.repoName);
		// PersistedTab should not have 'dirty' — compile-time check, but runtime verify absence
		expect("dirty" in persisted).toBe(false);
	});
});

describe("store", () => {
	beforeEach(() => {
		backingStore.clear();
	});

	describe("recent repos", () => {
		it("getRecentRepos returns empty array when store is empty", async () => {
			const repos = await getRecentRepos();
			expect(repos).toEqual([]);
		});

		it("addRecentRepo adds a repo and getRecentRepos returns it", async () => {
			await addRecentRepo({ name: "myrepo", path: "/path/to/repo" });
			const repos = await getRecentRepos();
			expect(repos).toEqual([{ name: "myrepo", path: "/path/to/repo" }]);
		});

		it("addRecentRepo moves duplicate repo to front", async () => {
			await addRecentRepo({ name: "A", path: "/a" });
			await addRecentRepo({ name: "B", path: "/b" });
			await addRecentRepo({ name: "A", path: "/a" });
			const repos = await getRecentRepos();
			expect(repos).toEqual([
				{ name: "A", path: "/a" },
				{ name: "B", path: "/b" },
			]);
		});

		it("addRecentRepo caps at MAX_RECENT (10)", async () => {
			for (let i = 0; i < 11; i++) {
				await addRecentRepo({ name: `repo${i}`, path: `/path/${i}` });
			}
			const repos = await getRecentRepos();
			expect(repos).toHaveLength(10);
			// The first-added (repo0) should be dropped, most recent (repo10) should be first
			expect(repos[0]).toEqual({ name: "repo10", path: "/path/10" });
			expect(repos.find((r) => r.path === "/path/0")).toBeUndefined();
		});

		it("removeRecentRepo removes matching path", async () => {
			await addRecentRepo({ name: "A", path: "/a" });
			await addRecentRepo({ name: "B", path: "/b" });
			await removeRecentRepo("/a");
			const repos = await getRecentRepos();
			expect(repos).toEqual([{ name: "B", path: "/b" }]);
		});
	});

	describe("zoom level", () => {
		it("getZoomLevel returns 1 when store is empty (default)", async () => {
			const level = await getZoomLevel();
			expect(level).toBe(1);
		});

		it("setZoomLevel persists and getZoomLevel retrieves it", async () => {
			await setZoomLevel(1.5);
			const level = await getZoomLevel();
			expect(level).toBe(1.5);
		});
	});
});
