---
status: diagnosed
phase: 28-destructive-operations
source: [28-01-SUMMARY.md, 28-02-SUMMARY.md, 28-03-SUMMARY.md]
started: 2026-03-15T20:30:00Z
updated: 2026-03-15T20:40:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Discard Single File (Tracked)
expected: Right-click a modified (tracked) file in the unstaged section. A native context menu appears with a "Discard Changes" option. Clicking it shows a confirmation dialog warning you'll lose changes. Confirming reverts the file to its last committed state.
result: pass

### 2. Discard Single File (Untracked)
expected: Right-click an untracked file in the unstaged section. A native context menu appears with a "Delete File" option (different wording from tracked files). Clicking it shows a confirmation dialog. Confirming deletes the file from disk.
result: pass

### 3. Discard All
expected: The unstaged section header has a "Discard All" button. Clicking it shows a confirmation dialog that mentions the number of files to be discarded. Confirming reverts all tracked changes and deletes all untracked files.
result: pass

### 4. Discard Toast Feedback
expected: After any successful discard operation (single file or all), a toast notification appears confirming the operation succeeded. If it fails, the toast shows an error message.
result: pass

### 5. Branch Delete from Sidebar
expected: Right-click a local branch (not the currently checked-out one) in the branch sidebar. A context menu shows a "Delete" option. Clicking it shows a confirmation dialog. Confirming removes the branch and updates the sidebar and graph.
result: pass

### 6. Branch Delete Blocked for HEAD
expected: Right-click the currently checked-out (HEAD) branch in the sidebar. The Delete option is either disabled or absent — you cannot delete the branch you're on.
result: pass

### 7. Branch Rename from Sidebar
expected: Right-click a local branch in the sidebar. A context menu shows a "Rename" option. Clicking it opens an InputDialog pre-filled with the current branch name. Changing the name and submitting renames the branch. Sidebar and graph update.
result: pass

### 8. Tag Delete from Sidebar
expected: Right-click a tag in the sidebar. A context menu shows a "Delete" option. Clicking it shows a confirmation dialog. Confirming removes the tag and updates the sidebar and graph.
result: pass

### 9. Graph Pill Context Menus
expected: Right-click a branch pill on a commit row in the graph. A context menu appears with Rename and Delete options for local branches (Delete only for tags). These work the same as the sidebar context menus — same dialogs, same toast feedback.
result: issue
reported: "pass, But when the pill has multiple branches with the overflow and it expands, I can't click on the branch and have that context menu pop up. I think the context menu is wired only to the single pill. With Overflow pills, we should be able to hover over the branch name on the bigger pill that has the multiple branches, and have the same options available."
severity: major

## Summary

total: 9
passed: 8
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Right-click context menus work on overflow/expanded pills showing multiple branches — each branch name in the expanded pill should have its own context menu with Rename/Delete options"
  status: failed
  reason: "User reported: when the pill has multiple branches with the overflow and it expands, I can't click on the branch and have that context menu pop up. Context menu is wired only to the single pill. With overflow pills, we should be able to hover over the branch name on the bigger pill that has the multiple branches, and have the same options available."
  severity: major
  test: 9
  root_cause: "Overflow pill expansion overlay (CommitGraph.svelte ~line 824) renders each ref as a plain <div> with no oncontextmenu handler. showPillContextMenu is only wired to single-pill SVG elements (rect, icon g, text span). The expanded {#each hoveredPill.allRefs as ref} loop has zero event handlers."
  artifacts:
    - path: "src/components/CommitGraph.svelte"
      issue: "Overflow expansion div elements (lines 824-831) have no oncontextmenu handler"
    - path: "src/components/CommitGraph.svelte"
      issue: "showPillContextMenu (line 366) accepts OverlayRefPill but needs to also accept RefLabel for overflow items"
  missing:
    - "Add oncontextmenu handler to each ref div in the overflow expansion {#each} block"
    - "Adapt showPillContextMenu to accept RefLabel (ref_type, short_name, is_head) or create a thin wrapper"
    - "Add cursor: context-menu styling to overflow ref divs"
  debug_session: ".planning/debug/overflow-pill-context-menu.md"
