import type { TrunkError } from './invoke.js';

export interface RemoteState {
  isRunning: boolean;
  progressLine: string;
  error: TrunkError | null;
}

export function createRemoteState(): RemoteState {
  const state: RemoteState = $state({
    isRunning: false,
    progressLine: '',
    error: null as TrunkError | null,
  });
  return state;
}

// DEPRECATED: singleton for backward compat until Plan 02 updates consumers
export const remoteState: RemoteState = createRemoteState();
