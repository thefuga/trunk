---
status: complete
phase: 12-commit-context-menu
source: [12-01-SUMMARY.md, 12-02-SUMMARY.md]
started: 2026-03-15T12:00:00Z
updated: 2026-03-15T12:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Context Menu on Commit Row
expected: Right-clicking a commit row shows a native context menu with 7 actions: Copy SHA, Copy Message, Checkout, Create Branch, Create Tag, Cherry-pick, Revert.
result: pass

### 2. Copy SHA and Copy Message
expected: Selecting Copy SHA or Copy Message copies the respective value to clipboard.
result: pass

### 3. Checkout Commit (Detached HEAD)
expected: Selecting Checkout shows a confirmation dialog warning about detached HEAD. Confirming checks out the commit.
result: pass

### 4. Create Branch from Commit
expected: Selecting Create Branch opens an input dialog for branch name. Submitting creates a branch at that commit.
result: pass

### 5. Create Tag on Commit
expected: Selecting Create Tag opens an input dialog for tag name. Submitting creates an annotated tag on the commit.
result: pass

### 6. Cherry-pick and Revert
expected: Cherry-pick and Revert execute the respective git operations. Both are disabled (greyed out) for merge commits.
result: pass

### 7. Synthetic Rows Excluded
expected: WIP and stash rows do not show the commit context menu when right-clicked.
result: pass

## Summary

total: 7
passed: 7
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
