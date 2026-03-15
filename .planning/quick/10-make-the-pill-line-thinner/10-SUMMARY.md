---
phase: quick
plan: 10
subsystem: ui
tags: [commit-graph, styling, visual-polish]
dependency_graph:
  requires: []
  provides: [thinner-pill-connector-line]
  affects: [commit-graph-rendering]
tech_stack:
  patterns: [hardcoded-stroke-width-override]
key_files:
  modified:
    - src/components/CommitGraph.svelte
decisions:
  - Hardcoded stroke-width to 1 rather than introducing a new display setting constant
metrics:
  duration_seconds: 27
  completed: "2026-03-15T03:43:54Z"
---

# Quick Task 10: Make the Pill Line Thinner — Summary

Reduced pill connector line stroke-width from 1.5px (displaySettings.edgeStroke) to 1px for subtler visual weight.

## What Was Done

### Task 1: Reduce pill connector line stroke width
- **Commit:** a2b7b95
- **Files:** `src/components/CommitGraph.svelte`
- Changed the connector line (from ref pill to commit dot) `stroke-width` from `{displaySettings.edgeStroke}` (1.5) to `{1}`
- Pill connector lines are now visually thinner than graph edges, reducing visual clutter

## Deviations from Plan

None — plan executed exactly as written.

## Verification

- `grep 'stroke-width={1}' src/components/CommitGraph.svelte` confirms the connector line uses the new value at line 592
- Graph edges remain at 1.5px via `displaySettings.edgeStroke`, only pill connectors changed

## Self-Check: PASSED

- [x] `src/components/CommitGraph.svelte` — FOUND
- [x] Commit `a2b7b95` — FOUND
