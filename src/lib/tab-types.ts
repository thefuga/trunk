export interface TabInfo {
  id: string;
  repoPath: string | null;
  repoName: string;
  dirty: boolean;
}

export interface PersistedTab {
  id: string;
  repoPath: string | null;
  repoName: string;
}

export function createTabId(): string {
  return crypto.randomUUID();
}
