---
created: 2026-03-16T14:33:22.613Z
title: Fix staged files header sinking when collapsed
area: ui
files:
  - StagingPanel.svelte:~188
  - StagingPanel.svelte:152
---

## Problem

In the staging panel, when the staged files section is collapsed, its 28px header sinks to the bottom of the panel instead of staying hugged to the content above it. The root cause is that the unstaged section gets `flex: 1` when staged is collapsed (around line ~188 in `StagingPanel.svelte`), which pushes the staged header to the bottom of the flex container.

## Solution

Either:
1. Don't give the unstaged section `flex: 1` when the staged section is collapsed, or
2. Add a spacer `<div style="flex: 1">` after the staged section so the gap appears below both headers instead of between them.

This ensures the staged header always stays visually adjacent to the unstaged content above it, regardless of collapse state.
