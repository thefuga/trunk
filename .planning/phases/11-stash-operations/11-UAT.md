---
status: complete
phase: 11-stash-operations
source: [11-01-SUMMARY.md, 11-02-SUMMARY.md, 11-03-SUMMARY.md, 11-04-SUMMARY.md, 11-05-SUMMARY.md, 11-06-SUMMARY.md]
started: 2026-03-15T12:00:00Z
updated: 2026-03-15T12:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Stash Save from Sidebar
expected: Stash section always visible in sidebar with '+' button. Clicking '+' opens inline form with optional name input. Submitting stashes working tree changes. If nothing to stash, inline error message appears.
result: pass

### 2. Stash Pop/Apply/Drop from Sidebar
expected: Right-clicking a stash entry in sidebar shows native context menu with Pop, Apply, Drop. Drop shows native OS confirmation dialog. Operations execute and stash list refreshes immediately without white flash.
result: pass

### 3. Stash Graph Rendering
expected: Stash entries appear as hollow square dots in a dedicated rightmost column in the commit graph, positioned next to their parent commit.
result: pass

### 4. Stash Graph Context Menu
expected: Right-clicking a stash row in the commit graph shows Pop, Apply, Drop context menu. Drop shows confirmation. Operations work and graph updates.
result: pass

### 5. Stash Click-to-Diff
expected: Clicking a stash entry in the sidebar loads its diff in the right pane, showing the stash contents.
result: pass

### 6. Stash Section Auto-Expand
expected: Clicking '+' on a collapsed stash section automatically expands it to reveal the create form.
result: pass

### 7. No White Flash on Stash Operations
expected: Performing any stash operation (save/pop/apply/drop) refreshes the UI smoothly without a white flash or full-page re-render.
result: pass

## Summary

total: 7
passed: 7
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
