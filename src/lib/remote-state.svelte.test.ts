import { describe, it, expect } from 'vitest';
import { createRemoteState } from './remote-state.svelte.js';

describe('createRemoteState', () => {
  it('returns object with correct defaults', () => {
    const state = createRemoteState();
    expect(state.isRunning).toBe(false);
    expect(state.progressLine).toBe('');
    expect(state.error).toBe(null);
  });

  it('returns independent instances — mutating one does not affect the other', () => {
    const a = createRemoteState();
    const b = createRemoteState();
    a.isRunning = true;
    expect(b.isRunning).toBe(false);
  });
});
