---
status: complete
phase: 40-rebase-workflow
source: 40-01-SUMMARY.md
started: 2026-03-22T00:00:00Z
updated: 2026-03-23T00:05:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Rebase From Sidebar Local Branch
expected: Right-click a local branch in the sidebar (not HEAD). Context menu shows "Rebase [HEAD] onto [branch]". Click it. Rebase executes, graph refreshes. No success toast.
result: pass

### 2. Rebase From Sidebar Remote Branch
expected: Right-click a remote branch in the sidebar. Context menu shows rebase option. Click it. Rebase executes and graph refreshes.
result: pass

### 3. Rebase From Graph Pill
expected: Right-click a branch pill on the commit graph. Context menu includes rebase option adjacent to merge option. Click it. Rebase executes.
result: pass

### 4. Rebase Items Adjacent to Merge Items
expected: Right-click any branch surface. The Rebase option appears immediately adjacent to the Merge option (grouped together, separator after the group).
result: pass

### 5. Rebase Conflict Triggers Operation Banner
expected: Rebase onto a branch that causes conflicts. The operation banner appears showing "Rebase in progress" with Continue, Skip, and Abort buttons. Step progress shown (e.g., "Step 1/3").
result: pass

## Summary

total: 5
passed: 5
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
