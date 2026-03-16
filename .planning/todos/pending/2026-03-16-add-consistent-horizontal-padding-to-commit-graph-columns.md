---
created: 2026-03-16T14:33:22.613Z
title: Add consistent horizontal padding to commit graph columns
area: ui
files:
  - graph-constants.ts
  - CommitGraph.svelte:651-696
  - CommitRow.svelte:48-87
---

## Problem

Horizontal padding in the commit graph columns is inconsistent. Header cells use Tailwind `px-2` (8px) while data cells use `px-1` (4px), with no shared constant governing the value. This leads to visual misalignment between headers and their data columns, and makes it harder to adjust padding globally.

## Solution

1. Add a `COLUMN_PADDING_X` constant to `graph-constants.ts` (e.g. `2` for 2px).
2. Remove the hardcoded Tailwind `px-*` classes from `CommitGraph.svelte` header cells (~lines 651-696) and `CommitRow.svelte` column cells (~lines 48-87).
3. Apply `padding: 0 ${COLUMN_PADDING_X}px` via inline style instead.
4. This gives ~4px between adjacent columns (2px right from one column + 2px left from the next).
