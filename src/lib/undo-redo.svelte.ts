export interface UndoEntry {
  subject: string;
  body: string | null;
}

export interface UndoRedoState {
  redoStack: UndoEntry[];
}

export interface UndoRedoManager {
  state: UndoRedoState;
  push(entry: UndoEntry): void;
  pop(): UndoEntry | undefined;
  clear(): void;
}

export function createUndoRedoState(): UndoRedoManager {
  const state: UndoRedoState = $state({ redoStack: [] as UndoEntry[] });

  return {
    state,
    push(entry: UndoEntry) {
      state.redoStack = [...state.redoStack, entry];
    },
    pop(): UndoEntry | undefined {
      if (state.redoStack.length === 0) return undefined;
      const entry = state.redoStack[state.redoStack.length - 1];
      state.redoStack = state.redoStack.slice(0, -1);
      return entry;
    },
    clear() {
      state.redoStack = [];
    },
  };
}

// DEPRECATED: singleton for backward compat until Plan 02 updates consumers
const _compat = createUndoRedoState();
export const undoRedoState = _compat.state;
export const pushToRedoStack = _compat.push;
export const popFromRedoStack = _compat.pop;
export const clearRedoStack = _compat.clear;
