---
status: diagnosed
phase: 38-merge-editor
source: 38-01-SUMMARY.md, 38-02-SUMMARY.md, 38-03-SUMMARY.md, 38-04-SUMMARY.md, 38-05-SUMMARY.md, 38-06-SUMMARY.md
started: 2026-03-22T00:00:00Z
updated: 2026-03-23T00:05:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Merge Editor Opens for Conflicted File
expected: Create a merge conflict. Click a conflicted file in the Conflicted section. A three-panel merge editor opens: Current (ours) and Incoming (theirs) side-by-side on top, Output textarea spanning the bottom.
result: pass

### 2. Conflict Regions Highlighted
expected: In the merge editor, conflict regions (lines that differ between Current and Incoming) are visually distinguished from context regions (identical lines). Each conflict region has a hunk header row.
result: pass

### 3. Per-Hunk Toggle Selection
expected: Click a hunk header row in the Current or Incoming panel. The entire hunk gets selected (green check icon appears). The Output textarea updates to include those lines. Click again to deselect.
result: pass

### 4. Per-Line Toggle Selection
expected: Click an individual line within a conflict region. That single line toggles selected/deselected (icon changes). The Output updates accordingly with just that line included or removed.
result: pass

### 5. Synchronized Scroll
expected: Scroll in any of the three panels (Current, Incoming, Output). The other two panels scroll in sync to the same position.
result: pass

### 6. Prev/Next Conflict Navigation
expected: Click Prev/Next buttons in the editor toolbar. The view scrolls to the previous/next conflict region. Buttons disable at boundaries (no previous at first conflict, no next at last).
result: pass

### 7. Manual Edit Disables Auto-Recompute
expected: Type directly in the Output textarea. After manual editing, toggling hunks/lines in Current/Incoming no longer auto-updates the Output (manual edit takes precedence).
result: issue
reported: "reject, it overrides"
severity: major

### 8. Save and Mark Resolved
expected: After selecting lines to resolve the conflict, click "Save and Mark Resolved". The file is written to disk, staged in git index, and the editor closes. The file moves from Conflicted to Staged section.
result: pass

### 9. Auto-Open Next Conflicted File
expected: After resolving a file (Save and Mark Resolved), if other conflicted files remain, the merge editor automatically opens the next conflicted file. If no conflicts remain, returns to the commit graph view.
result: pass

### 10. Take All Current/Incoming Context Menu
expected: Right-click a conflicted file in the Conflicted section. Context menu shows "Take All Current" and "Take All Incoming". Clicking one resolves the file using that side entirely, stages it, and refreshes the staging panel — without opening the editor.
result: issue
reported: "It just selects all of that file, it doesn't continue the action, I still need to click Save and Mark Resolved on the bottom pane in the output."
severity: major

## Summary

total: 10
passed: 8
issues: 2
pending: 0
skipped: 0

## Gaps

- truth: "Manual editing in Output textarea disables auto-recompute from hunk/line toggles"
  status: failed
  reason: "User reported: reject, it overrides"
  severity: major
  test: 7
  root_cause: "All toggle/take handlers unconditionally set manualEdit = false, clearing the manual-edit guard. Lines 206, 212, 217, 238, 244 in MergeEditor.svelte."
  artifacts:
    - path: "src/components/MergeEditor.svelte"
      issue: "handleTakeAllCurrent, handleTakeAllIncoming, handleToggleHunk, handleToggleLine all reset manualEdit = false"
  missing:
    - "Remove manualEdit = false from all toggle/take handlers — flag should only reset on file reload"
  debug_session: ".planning/debug/merge-editor-manual-edit-overridden.md"

- truth: "Take All Current/Incoming resolves the file entirely without opening the editor"
  status: failed
  reason: "User reported: It just selects all of that file, it doesn't continue the action, I still need to click Save and Mark Resolved on the bottom pane in the output."
  severity: major
  test: 10
  root_cause: "resolveConflictedFile in StagingPanel.svelte correctly saves and stages via save_merge_result, but does not notify App.svelte. selectedFile remains { kind: 'conflicted' }, so MergeEditor stays mounted with stale state."
  artifacts:
    - path: "src/components/StagingPanel.svelte"
      issue: "resolveConflictedFile (line 155) has no parent callback after resolving"
    - path: "src/App.svelte"
      issue: "selectedFile/showMergeEditor not updated by StagingPanel resolution path"
  missing:
    - "After resolveConflictedFile succeeds, call a parent callback (like onresolved) to clear selectedFile or advance to next conflicted file"
  debug_session: ".planning/debug/context-menu-take-all-no-resolve.md"
