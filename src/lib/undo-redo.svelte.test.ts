import { describe, it, expect } from 'vitest';
import { createUndoRedoState } from './undo-redo.svelte.js';

describe('createUndoRedoState', () => {
  it('returns independent managers — push on one, pop on other returns undefined', () => {
    const a = createUndoRedoState();
    const b = createUndoRedoState();
    a.push({ subject: 'test', body: null });
    expect(b.pop()).toBeUndefined();
  });

  it('push/pop/clear work correctly for a single instance', () => {
    const mgr = createUndoRedoState();
    mgr.push({ subject: 'first', body: null });
    mgr.push({ subject: 'second', body: 'desc' });
    expect(mgr.state.redoStack).toHaveLength(2);

    const popped = mgr.pop();
    expect(popped).toEqual({ subject: 'second', body: 'desc' });
    expect(mgr.state.redoStack).toHaveLength(1);

    mgr.clear();
    expect(mgr.state.redoStack).toHaveLength(0);
  });
});
