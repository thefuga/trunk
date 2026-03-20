---
created: 2026-03-19T23:15:46.303Z
title: Add right-click context menu to file panels
area: ui
files:
  - src/components/StagingPanel.svelte
  - src/components/DiffPanel.svelte
  - src/components/CommitDetail.svelte
  - src/components/FileRow.svelte
---

## Problem

The right-side pane (staging panel, diff panel, commit detail) has no right-click context menu on file entries. Users cannot quickly copy file paths or perform common file-level git actions without navigating elsewhere. GitKraken and similar tools provide rich context menus on file rows.

## Solution

Add a context menu component that appears on right-click of file entries in the right pane. Priority actions:

**Must have:**
- Copy relative file path
- Copy absolute file path

**Should have (GitKraken-style actions):**
- Open file in editor
- Open file in Finder/file manager
- Stage / Unstage file
- Discard changes
- View file history / blame
- Copy file content

Implement as a reusable `ContextMenu.svelte` component that can be positioned at cursor and populated with action items. Hook into `contextmenu` events on `FileRow.svelte` and similar file list entries across `StagingPanel`, `DiffPanel`, and `CommitDetail`.
