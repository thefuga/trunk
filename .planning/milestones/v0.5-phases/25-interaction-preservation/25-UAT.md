---
status: complete
phase: 25-interaction-preservation
source: [25-01-SUMMARY.md]
started: 2026-03-15T12:00:00Z
updated: 2026-03-15T12:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Selected Row Highlight
expected: Clicking a commit row shows a persistent subtle blue-tinted background (10% opacity accent). Hover state is suppressed on the selected row.
result: pass

### 2. Stash Context Menu in Graph
expected: Right-clicking a stash row in the commit graph shows Pop/Apply/Drop context menu (not the full commit menu). Drop shows confirmation dialog.
result: pass

### 3. WIP Row Exclusion
expected: The WIP row is excluded from both selection highlight and context menu.
result: pass

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
