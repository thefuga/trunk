---
created: 2026-03-19T23:13:29.789Z
title: Show diff content for untracked new files
area: api
files:
  - src-tauri/src/commands/diff.rs:93-103
  - src/components/DiffPanel.svelte
  - src/components/StagingPanel.svelte
---

## Problem

When viewing an unstaged new (untracked) file in the staging panel, clicking it to see the diff shows nothing. The `diff_unstaged_inner` function (diff.rs:93-103) uses `repo.diff_index_to_workdir()` which compares the index to the working directory. For untracked files there is no index entry, so git2 either skips the file entirely or produces a delta with status `Untracked` but no content hunks. The result is an empty diff display even though the file has content.

## Solution

In `diff_unstaged_inner`, enable `include_untracked_content` on the `DiffOptions` so git2 generates hunks for untracked files (treating them as fully new). Alternatively, when detecting an untracked file, read its contents directly and synthesize an "all added" diff. The `DiffOptions::include_untracked_content(true)` combined with `include_untracked(true)` should make git2 produce proper content hunks for new files.
