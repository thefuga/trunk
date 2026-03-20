---
created: 2026-03-19T23:08:43.023Z
title: Fix merge commit line bend direction
area: ui
files:
  - src/lib/overlay-paths.ts:117-181
---

## Problem

The merge commit connection lines in the git graph currently bend upwards when they should bend downwards. The `isMergePattern` function in `overlay-paths.ts` determines curve direction — merge patterns should curve down (toward higher Y) and fork patterns should curve up (toward lower Y). The current logic or its application appears to produce the wrong visual result for merge commits, causing the connecting line to arc in the wrong direction.

## Solution

Investigate the `isMergePattern` function (line 117) and its usage in the connection path builder (line 172) in `src/lib/overlay-paths.ts`. The vertical sign (`vSign`) at line 181 controls the bend direction — verify this is being applied correctly and that the merge detection logic matches the expected visual output. May need to invert the curve direction or fix the merge detection heuristic.
