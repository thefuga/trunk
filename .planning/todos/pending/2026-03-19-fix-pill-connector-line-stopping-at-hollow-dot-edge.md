---
created: 2026-03-19T23:11:13.516Z
title: Fix pill connector line stopping at hollow dot edge
area: ui
files:
  - src/components/CommitGraph.svelte:923-932
  - src/lib/ref-pill-data.ts
  - src/lib/overlay-paths.ts:43-44
---

## Problem

The connector line drawn from a ref pill to the commit dot always terminates at the dot center (`pill.dotCy`). For filled dots this looks correct, but for hollow dots (merge commits, stash, WIP) the line is visible through the unfilled interior, creating a visual artifact that resembles a sideways power button symbol. The rail rendering in `overlay-paths.ts` already accounts for hollow dots by stopping at `dotRadius + DASH_GAP` from the center, but the pill connector line does not apply the same adjustment.

## Solution

In `CommitGraph.svelte` (line 923-932), adjust the `x2`/`y2` endpoint of the pill connector `<line>` to stop at the dot edge (center minus dotRadius) when the commit has a hollow dot, instead of going all the way to the center. The `isHollow` check from `overlay-paths.ts` (or the node type) can be used to determine whether to apply the offset. May also need to pass a `isHollow` flag through the pill data in `ref-pill-data.ts`.
