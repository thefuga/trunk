---
status: resolved
trigger: "Manual editing in Output textarea does NOT disable auto-recompute. Toggling hunk/line overrides manual edits."
created: 2026-03-23T00:00:00Z
updated: 2026-05-28T00:00:00Z
verified_at: 2026-05-28
---

## Current Focus

hypothesis: Every handleToggleHunk and handleToggleLine call explicitly sets manualEdit = false, which clears the manual edit flag even when the user has already manually edited the output
test: Read the toggle handler functions and trace the manualEdit flag lifecycle
expecting: Confirm that toggle handlers unconditionally reset manualEdit to false
next_action: Trace the full lifecycle of manualEdit flag through all handlers

## Symptoms

expected: After manually editing the Output textarea, toggling hunks/lines in Current/Incoming panels should NOT auto-update the Output textarea (manual edits should be preserved)
actual: Toggling a hunk/line after manual editing overrides the Output with auto-computed result, losing manual edits
errors: No error messages - silent data loss
reproduction: 1) Open merge editor 2) Manually type in Output textarea 3) Toggle any hunk or line in Current/Incoming panels 4) Output textarea is overridden with computed result
started: Present in current code

## Eliminated

## Evidence

- timestamp: 2026-03-23T00:01:00Z
  checked: manualEdit flag lifecycle in MergeEditor.svelte
  found: manualEdit is correctly set to true by handleOutputEdit (line 249), and outputText derived (line 137-140) correctly returns manualText when manualEdit is true
  implication: The flag mechanism itself is correct

- timestamp: 2026-03-23T00:02:00Z
  checked: All handlers that modify takenLines
  found: handleTakeAllCurrent (L206), handleTakeAllIncoming (L212), handleToggleHunk (L217), handleToggleLine (L238, L244) all unconditionally set manualEdit = false
  implication: This is the root cause -- any toggle action after manual editing resets the flag, causing outputText to recompute from takenLines and overwrite manual edits

## Resolution

root_cause: Every hunk/line toggle handler (handleToggleHunk, handleToggleLine, handleTakeAllCurrent, handleTakeAllIncoming) unconditionally sets manualEdit = false. When the user has manually edited the Output textarea (manualEdit = true, manualText holds their edits), clicking any toggle in the Current/Incoming panels resets manualEdit to false, causing the outputText derived value to fall through to computeOutput(regions, takenLines), which overwrites the manual edits.
fix: Phase 38-07 (commit 6c76bbf "fix(38-07): preserve manualEdit flag in toggle/take handlers") removed the `manualEdit = false` line from handleTakeAllCurrent, handleTakeAllIncoming, handleToggleHunk, and handleToggleLine. The flag is now only reset in handleReset and the initial-load `.then()` block in MergeEditor.svelte. Once the user manually edits the Output textarea, outputText stays bound to manualText regardless of subsequent hunk/line toggles.
verification: Verified 2026-05-28 against src/components/MergeEditor.svelte (toggle handlers at lines 303, 307, 311, 317 — none reset manualEdit).
files_changed:
- src/components/MergeEditor.svelte
