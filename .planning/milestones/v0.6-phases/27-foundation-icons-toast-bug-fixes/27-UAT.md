---
status: resolved
phase: 27-foundation-icons-toast-bug-fixes
source: 27-01-SUMMARY.md, 27-02-SUMMARY.md, 27-03-SUMMARY.md, 27-04-SUMMARY.md
started: 2026-03-15T14:00:00Z
updated: 2026-03-15T14:20:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Toast on Git Operation
expected: Perform a git operation from the Toolbar (e.g., stash, pull, push, or pop). A toast notification appears briefly in the UI showing a success or error message. It disappears on its own without any interaction.
result: pass

### 2. Toast on Branch Checkout
expected: In the branch sidebar, checkout a different branch. A toast notification appears confirming the checkout succeeded (or showing an error if it failed). It auto-dismisses without any click needed.
result: pass

### 3. Toolbar Icons — SVG not Unicode
expected: Open the app and look at the Toolbar buttons (undo, redo, pull, push, stash, pop, branch). All buttons show clean SVG icons instead of Unicode box symbols or raw characters. Icons look crisp at any display scale.
result: pass

### 4. File Status Icons — Colored SVG
expected: Open a repo with staged and unstaged file changes. In the file list, each file shows a colored SVG icon reflecting its status (e.g., green for new/added, orange for modified, red for deleted). Stage/unstage action buttons show + and – icons.
result: pass

### 5. Branch & UI Control Icons
expected: In the branch sidebar: expand/collapse arrows are SVG chevrons; the "add branch" button shows a + icon. In the tab bar: the close button shows an X icon. In the staging panel: expand/collapse uses SVG chevrons. All look clean, no Unicode boxes.
result: pass

### 6. Untracked Files Show WIP Row
expected: Add a new untracked file to a repo (e.g., create a new file and don't stage it). The commit graph's WIP row appears, indicating the working tree is dirty. Previously, untracked-only changes would not trigger this row.
result: pass

### 7. No Trailing Resize Divider on Last Column
expected: Look at the commit graph header. The rightmost visible column has no extra resize divider/handle on its right edge. Only columns that have another column to their right show a resize handle.
result: issue
reported: "still there"
severity: major

## Summary

total: 7
passed: 6
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Last visible column in commit graph header has no trailing resize divider/handle on its right edge"
  status: resolved
  reason: "User reported: still there"
  severity: major
  test: 7
  root_cause: "The message column's col-resize-handle div in CommitGraph.svelte has no {#if} guard. Every other column checks `col !== lastVisibleColumn` before rendering its handle; message does not. A misleading comment rationalized skipping the guard by calling it 'the left edge of the author column', but the element sits on message's right edge and must be suppressed when message is the last visible column."
  artifacts:
    - path: "src/components/CommitGraph.svelte"
      issue: "message column resize handle missing lastVisibleColumn guard (line ~492)"
  missing:
    - "Wrap the message col-resize-handle div in {#if 'message' !== lastVisibleColumn}...{/if}"
  debug_session: ""
