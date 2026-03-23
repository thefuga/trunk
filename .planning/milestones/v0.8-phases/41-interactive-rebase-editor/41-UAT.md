---
status: resolved
phase: 41-interactive-rebase-editor
source: 41-01-SUMMARY.md, 41-02-SUMMARY.md, 41-03-SUMMARY.md, 41-04-SUMMARY.md
started: 2026-03-22T00:00:00Z
updated: 2026-03-23T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Interactive Rebase From Commit Context Menu
expected: Right-click a non-HEAD, non-stash commit. "Interactive Rebase..." appears. Click it. RebaseEditor opens showing commits from that point to HEAD.
result: pass

### 2. Interactive Rebase From Branch Context Menu
expected: Right-click a branch (local or remote) in sidebar or graph pill. "Interactive Rebase [branch]..." appears. Click it. RebaseEditor opens with commits between fork point and HEAD.
result: pass

### 3. Editor Column Layout
expected: RebaseEditor shows columns: Action, SHA, Message, Author, Date. Column headers are visible. Commit rows display data in each column.
result: pass

### 4. Action Dropdown with Color Indicators
expected: Click the Action dropdown on any commit row. Options: pick (green dot), reword (amber), squash (purple), drop (red). Selecting an action updates the row's color indicator.
result: pass

### 5. Drag-and-Drop Reordering
expected: Drag a commit row to a different position. The row moves to the new position. The commit order in the list updates accordingly.
result: pass

### 6. Keyboard Shortcuts
expected: Select a row. Press P → action changes to pick. S → squash. R → reword. D → drop. Arrow Up/Down moves selection. Escape closes editor.
result: pass

### 7. Validation Errors
expected: Set all commits to "drop". An inline validation error appears. Set the first commit to "squash" (no predecessor to squash into). A validation error appears below that row. "Start Rebase" button should be disabled when validation errors exist.
result: pass

### 8. Start Rebase Executes Plan
expected: Reorder or change actions on some commits. Click "Start Rebase". The editor closes, rebase executes, and the graph refreshes showing the new commit history.
result: pass

### 9. Reword Pauses for Message Editing
expected: Set a commit to "reword" and start rebase. A dialog appears with the commit message pre-filled, allowing you to edit it. Submit the new message. Rebase continues.
result: pass

### 10. Squash Message Editing
expected: Set a commit to "squash" and start rebase. A dialog appears with the combined commit messages. Edit the message and submit. The squashed commit uses your edited message.
result: issue
reported: "reject, no dialog, I have to manually select the commit below and reword."
severity: major

### 11. Column Resize and Persistence
expected: Drag a column header border to resize. The column width changes. Close and reopen the editor — column widths are preserved.
result: issue
reported: "pass, but SHA is currently to the left of the message, it should be to the right."
severity: minor

### 12. Cancel Button
expected: Click "Cancel" in the toolbar. The editor closes without executing any rebase. The graph remains unchanged.
result: pass

## Summary

total: 12
passed: 9
issues: 3
pending: 0
skipped: 0

## Gaps

- truth: "Squash action shows dialog with combined commit messages for editing"
  status: resolved
  reason: "User reported: reject, no dialog, I have to manually select the commit below and reword."
  severity: major
  test: 10
  root_cause: "Signal/wait IPC mechanism was removed in commit de9ea19 due to rebase hanging. start_interactive_rebase_blocking now uses blocking .output() with no poll loop, no event emission. GIT_EDITOR script exits 0 when no queued message exists. Frontend blocks squash from openMessageEditor (line 285). No rebase-message-needed listener in App.svelte."
  artifacts:
    - path: "src-tauri/src/commands/interactive_rebase.rs"
      issue: "GIT_EDITOR script exits 0 with no modifications when no queued file exists (line 163)"
    - path: "src/components/RebaseEditor.svelte"
      issue: "openMessageEditor blocks squash actions (line 285)"
    - path: "src/App.svelte"
      issue: "No rebase-message-needed listener, no InputDialog for rebase messages"
  missing:
    - "Re-implement interactive message editing during rebase (fix the hang bug), OR allow pre-editing squash messages before rebase starts by combining messages of squash target + squash commits"
  debug_session: ".planning/debug/squash-message-no-dialog.md"

- truth: "SHA column is positioned to the right of the Message column"
  status: resolved
  reason: "User reported: SHA is currently to the left of the message, it should be to the right."
  severity: minor
  test: 11
  root_cause: "In RebaseEditor.svelte, both header (lines 383-397) and data row (lines 457-470) render SHA block before Message block. Order is Action→SHA→Message but should be Action→Message→SHA."
  artifacts:
    - path: "src/components/RebaseEditor.svelte"
      issue: "SHA column block rendered before Message column block in header and data row templates"
  missing:
    - "Swap SHA and Message block positions in both header and data row templates"
  debug_session: ".planning/debug/rebase-editor-column-order-squash-arrow.md"

- truth: "Squash arrow indicator renders next to commit dot, not in validation error row"
  status: resolved
  reason: "User reported: squash arrow ended up in the error message area instead of next to the commit dot"
  severity: minor
  test: 7
  root_cause: "Squash arrow uses position: absolute with bottom: -4px relative to rebase-row-wrapper. When validation error div is inside the same wrapper, the wrapper grows taller and the arrow shifts down into the error area."
  artifacts:
    - path: "src/components/RebaseEditor.svelte"
      issue: "Arrow span (line 425) positioned absolute bottom:-4px inside wrapper that also contains validation error div (lines 494-498)"
  missing:
    - "Either move validation error div outside rebase-row-wrapper, or make rebase-row the positioning context instead of the wrapper"
  debug_session: ".planning/debug/rebase-editor-column-order-squash-arrow.md"
