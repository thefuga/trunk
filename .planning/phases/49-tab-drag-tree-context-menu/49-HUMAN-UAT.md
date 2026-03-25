---
status: partial
phase: 49-tab-drag-tree-context-menu
source: [49-VERIFICATION.md]
started: 2026-03-25T00:35:00Z
updated: 2026-03-25T00:35:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Drag tab to new position
expected: Tab moves with smooth animation (animation:150ms); the + button cannot be dragged; tab order updates immediately
result: [pending]

### 2. Tab reorder persists across app relaunch
expected: After dragging tab 3 to position 1 and relaunching the app, the tab order matches the dragged order
result: [pending]

### 3. Tab bar auto-scrolls when dragging near edges
expected: When dragging a tab near the left or right edge of the tab bar (overflow scenario), the bar scrolls
result: [pending]

### 4. Directory context menu in unstaged section
expected: Right-clicking a directory node in the unstaged tree section shows a native menu with 'Stage All (N)' and 'Discard All (N)' items
result: [pending]

### 5. Directory context menu in staged section
expected: Right-clicking a directory node in the staged tree section shows a native menu with 'Unstage All (N)'
result: [pending]

### 6. Directory context menu in conflicted section
expected: Right-clicking a conflicted directory shows 'Resolve All (N)' and 'Unresolve All (N)'
result: [pending]

### 7. Discard All shows confirmation dialog
expected: Clicking 'Discard All' triggers a native warning dialog before any files are discarded
result: [pending]

### 8. Context menu appears on directory nodes only, not file nodes
expected: Right-clicking a file row does not trigger the directory context menu
result: [pending]

## Summary

total: 8
passed: 0
issues: 0
pending: 8
skipped: 0
blocked: 0

## Gaps
