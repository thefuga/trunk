# Phase 67 — Deferred Items

Out-of-scope discoveries logged during execution. Not fixed by the plan that found them.

## From Plan 67-03

- **Worktree `node_modules` is empty (orchestrator-level setup gap).** Running
  `bun run test -- src/components/DiffPanel.test.ts` from inside the worktree fails with
  `Cannot find module '/@fs/.../node_modules/@testing-library/svelte/src/vitest.js'` — the
  worktree has no installed dependencies and the cross-tree `/@fs/` resolution to the main
  repo's `node_modules` does not resolve. `svelte-check` (`bun run check`) runs clean
  in-worktree because it does not need the test deps. Verification for this plan's Vitest
  gate ran against the identical edits in the main repo working tree (58/58 passed) before
  they were re-applied in the worktree. The orchestrator should link/install `node_modules`
  in spawned worktrees so per-agent Vitest runs work in isolation.
