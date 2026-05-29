---
status: complete
phase: quick
plan: 260403-1yi
subsystem: remote-branches
tags: [context-menu, remote, delete, sidebar, graph]
dependency_graph:
  requires: []
  provides: [delete_remote_branch]
  affects: [BranchSidebar, CommitGraph, remote.rs, lib.rs]
tech_stack:
  added: []
  patterns: [git-push-delete, confirmation-dialog]
key_files:
  created: []
  modified:
    - src-tauri/src/commands/remote.rs
    - src-tauri/src/lib.rs
    - src-tauri/tests/test_branches.rs
    - src/components/BranchSidebar.svelte
    - src/components/CommitGraph.svelte
decisions:
  - Separator before Delete item in remote branch context menus for visual grouping
  - BranchSidebar remote menu no longer early-returns on detached HEAD; shows Delete even without merge/rebase items
  - CommitGraph error shown via message() dialog (matching handleDeleteBranch pattern)
metrics:
  duration: 11min
  completed: 2026-04-03
---

# Quick 260403-1yi: Delete Remote Branches from Sidebar and Graph Summary

Tauri command `delete_remote_branch` that runs `git push --delete` via the existing `run_git_remote` helper, with Delete menu items added to remote branch context menus in both BranchSidebar and CommitGraph.

## What Was Done

### Task 1: Add delete_remote_branch Rust command (673efc6)

Added a new `delete_remote_branch` Tauri command in `src-tauri/src/commands/remote.rs` following the exact same pattern as `git_push`:
- Parses `branch_name` (e.g. "origin/feature") into remote and branch parts
- Runs `git push --delete --progress {remote} {branch}` via `run_git_remote`
- Calls `refresh_graph` on success to rebuild cache and emit `repo-changed`
- Registered in `lib.rs` invoke_handler

Added integration test in `test_branches.rs` that:
- Creates a bare repo as remote, clones it, creates and pushes a feature branch
- Verifies `git push --delete` removes the remote branch
- Confirms branch is gone after fetch --prune

### Task 2: Add Delete to remote branch context menus (5c48d8d)

**BranchSidebar.svelte:**
- Added `handleDeleteRemoteBranch` function with confirmation dialog and error handling
- Updated `showRemoteContextMenu` to include separator + "Delete" item
- Restructured menu to work in detached HEAD state (Delete is always available; merge/rebase are conditional)

**CommitGraph.svelte:**
- Added `handleDeleteRemoteBranch` function with confirmation dialog and error dialog on failure
- Updated `showRefContextMenu` RemoteBranch section with separator + "Delete" item at the end

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing functionality] BranchSidebar detached HEAD support**
- **Found during:** Task 2
- **Issue:** `showRemoteContextMenu` early-returned on detached HEAD (`if (!headBranchName) return`), meaning no context menu at all -- including no Delete option
- **Fix:** Restructured to always show Delete, conditionally show merge/rebase items only when HEAD branch exists
- **Files modified:** src/components/BranchSidebar.svelte
- **Commit:** 5c48d8d

## Verification

- `cargo clippy --lib -- -D warnings`: PASSED (0 errors)
- `cargo test --test test_branches`: PASSED (16 tests, including new `delete_remote_branch_removes_ref`)
- `npx vitest run`: PASSED (430 tests across 41 files)
- `npx svelte-check`: PASSED (0 errors, 0 warnings)
- `npx biome check`: PASSED

## Known Stubs

None.

## Self-Check: PASSED
