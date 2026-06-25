import type { RepoDescriptor } from "./types.js";

export interface TabInfo {
	id: string;
	repoId?: string | null;
	repoDescriptor?: RepoDescriptor | null;
	repoPath: string | null;
	repoName: string;
	dirty: boolean;
}

export interface PersistedTab {
	id: string;
	repoId?: string | null;
	repoDescriptor?: RepoDescriptor | null;
	repoPath: string | null;
	repoName: string;
}

export function createTabId(): string {
	return crypto.randomUUID();
}
