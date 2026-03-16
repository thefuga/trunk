---
created: 2026-03-16T14:33:22.613Z
title: Align commit graph and staging panel header heights
area: ui
files:
  - CommitGraph.svelte:647
  - StagingPanel.svelte:152
---

## Problem

The commit graph column header row and the right pane summary header have mismatched heights. The graph header is 24px (`CommitGraph.svelte:647`) while the staging panel summary header is 32px (`StagingPanel.svelte:152`). This causes the bottom borders of the two headers to not line up horizontally across the full window, creating a visual misalignment.

## Solution

Pick one consistent height (likely 32px to match the staging panel) and apply it to both the commit graph header and the staging panel summary header so their bottom borders align horizontally across the full window width.
