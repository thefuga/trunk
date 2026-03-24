import { describe, it, expect } from 'vitest';
import { createTabId } from './tab-types.js';
import type { TabInfo, PersistedTab } from './tab-types.js';

describe('tab types and helpers', () => {
  it('tab persistence round-trip — PersistedTab is valid subset of TabInfo', () => {
    const tab: TabInfo = { id: '123', repoPath: '/path', repoName: 'repo', dirty: false };
    const persisted: PersistedTab = { id: tab.id, repoPath: tab.repoPath, repoName: tab.repoName };
    expect(persisted.id).toBe(tab.id);
    expect(persisted.repoPath).toBe(tab.repoPath);
    expect(persisted.repoName).toBe(tab.repoName);
  });

  it('createTabId returns UUID v4 format', () => {
    const id = createTabId();
    expect(id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
  });
});
