---
status: complete
phase: 39-merge-workflow
source: 39-01-SUMMARY.md
started: 2026-03-22T00:00:00Z
updated: 2026-03-23T00:05:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Merge From Sidebar Local Branch
expected: Right-click a local branch in the sidebar (not HEAD). Context menu shows "Merge [branch] into [HEAD]". Click it. Merge executes, graph refreshes. No success toast.
result: pass

### 2. Merge From Sidebar Remote Branch
expected: Right-click a remote branch in the sidebar. Context menu shows "Merge [remote/branch] into [HEAD]". Click it. Merge executes and graph refreshes.
result: pass

### 3. Merge From Graph Pill
expected: Right-click a branch pill on the commit graph. Context menu includes merge option. Click it. Merge executes.
result: pass

### 4. Merge Hidden on HEAD Branch
expected: Right-click the HEAD branch (the currently checked-out branch). The merge option does NOT appear in the context menu.
result: pass

### 5. Merge Hidden on Detached HEAD
expected: Check out a detached HEAD (git checkout <commit>). Right-click any branch. Merge option does NOT appear.
result: pass

### 6. Merge Conflict Triggers Operation Banner
expected: Merge a branch that causes conflicts. The operation banner appears showing "Merge in progress" with Continue and Abort buttons. Conflicted files appear in the Conflicted section.
result: pass

## Summary

total: 6
passed: 6
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
