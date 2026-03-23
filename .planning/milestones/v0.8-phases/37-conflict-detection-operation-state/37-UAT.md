---
status: resolved
phase: 37-conflict-detection-operation-state
source: 37-01-SUMMARY.md, 37-02-SUMMARY.md
started: 2026-03-20T17:10:00Z
updated: 2026-03-20T18:30:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Operation Banner During Merge Conflict
expected: Create a merge conflict, open Trunk. A yellow banner appears at the top of the staging panel showing merge in progress with Continue and Abort buttons. No Skip button for merge.
result: pass

### 2. Operation Banner During Rebase Conflict
expected: Create a rebase conflict (`git rebase` on divergent branches). Open Trunk. A blue banner appears showing rebase in progress with Continue, Skip, and Abort buttons. Rebase step progress is shown (e.g., "Step 1/3").
result: pass

### 3. Conflicted Files Section
expected: During a merge or rebase conflict, conflicted files appear in a **dedicated section above** the unstaged/staged file lists. The section has a yellow warning triangle icon and a count badge showing the number of conflicted files. The section is collapsible.
result: pass

### 4. Read-Only Diff for Conflicted Files
expected: Click a conflicted file in the conflicted section. The diff panel shows the file contents (with conflict markers) but **no** stage/discard hunk action buttons — it's read-only.
result: issue
reported: "reject. No diff appears. The left panel stays completely empty when clicking a conflicted file."
severity: major

### 5. Abort Requires Confirmation
expected: Click the Abort button on the operation banner. A confirmation dialog appears asking to confirm the abort before it executes. Cancelling the dialog does not abort.
result: pass

### 6. Continue Fires Without Confirmation
expected: After resolving conflicts and staging the fixed files, click Continue on the operation banner. The action fires immediately — no confirmation dialog. The merge/rebase proceeds.
result: pass

### 7. Conflicted File Context Menu
expected: Right-click a conflicted file. The context menu shows only "Copy Relative Path" and "Copy Absolute Path" — no stage, discard, or other file actions.
result: pass

## Summary

total: 7
passed: 6
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Clicking a conflicted file shows a read-only diff of the file contents with conflict markers"
  status: resolved
  reason: "User reported: No diff appears. The left panel stays completely empty when clicking a conflicted file."
  severity: major
  test: 4
  root_cause: "diff_unstaged uses repo.diff_index_to_workdir() which skips conflicted files (no stage-0 index entry), returning empty Vec<FileDiff>"
  artifacts:
    - path: "src-tauri/src/commands/diff.rs"
      issue: "diff_unstaged_inner uses diff_index_to_workdir which cannot produce output for conflicted files"
    - path: "src/App.svelte"
      issue: "handleFileSelect calls diff_unstaged for conflicted files — wrong diff strategy for conflicts"
  missing:
    - "New diff approach for conflicted files: repo.diff_tree_to_workdir() (HEAD tree vs workdir) to show conflict markers"
    - "Either new Tauri command diff_conflicted or conditional branch in diff_unstaged_inner"
    - "Frontend to call the correct diff command for conflicted kind"
  debug_session: ".planning/debug/conflicted-diff-empty.md"
