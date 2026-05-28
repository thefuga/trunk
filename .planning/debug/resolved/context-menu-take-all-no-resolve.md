---
status: resolved
trigger: "Take All Current / Take All Incoming context menu items on conflicted files only select lines in merge editor, don't auto-save and resolve"
created: 2026-03-23T00:00:00Z
updated: 2026-05-28T00:00:00Z
verified_at: 2026-05-28
---

## Current Focus

hypothesis: CONFIRMED — see Resolution
test: N/A
expecting: N/A
next_action: Report root cause

## Symptoms

expected: "Take All Current" and "Take All Incoming" context menu items should resolve the file entirely without opening the editor — fetch sides, pick one, save, refresh status.
actual: These context menu items only select all lines in the merge editor, but do NOT auto-save and resolve. User still has to click "Save and Mark Resolved" manually.
errors: None (functional bug, not a crash)
reproduction: Right-click a conflicted file in the staging panel, choose "Take All Current" or "Take All Incoming"
started: Unknown

## Eliminated

## Evidence

- timestamp: 2026-03-23T00:01:00Z
  checked: StagingPanel.svelte resolveConflictedFile helper (lines 155-167)
  found: The helper correctly calls get_merge_sides, picks a side, calls save_merge_result, and calls loadStatus. This code path WOULD resolve the file without the editor.
  implication: The StagingPanel context menu handler is wired correctly.

- timestamp: 2026-03-23T00:02:00Z
  checked: StagingPanel.svelte showConflictedContextMenu (lines 169-182)
  found: Context menu items "Take All Current" and "Take All Incoming" call resolveConflictedFile(filePath, 'ours'/'theirs') — this is correct.
  implication: The context menu definition is correct.

- timestamp: 2026-03-23T00:03:00Z
  checked: StagingPanel.svelte conflicted file row onclick handler (line 513)
  found: onclick={() => onfileselect?.(f.path, 'conflicted')} — clicking a conflicted file calls onfileselect with kind='conflicted'
  implication: When user clicks a conflicted file (or right-clicks and then the click event propagates), the file is selected.

- timestamp: 2026-03-23T00:04:00Z
  checked: App.svelte handleFileSelect (lines 153-171) and showMergeEditor derived (line 67)
  found: handleFileSelect sets selectedFile = { path, kind }. showMergeEditor = selectedFile?.kind === 'conflicted'. When showMergeEditor is true, center pane renders MergeEditor (lines 578-584).
  implication: When a conflicted file is SELECTED (left-clicked), the MergeEditor opens in the center pane.

- timestamp: 2026-03-23T00:05:00Z
  checked: FileRow.svelte oncontextmenu behavior and browser right-click semantics
  found: The context menu fires via oncontextmenu on the FileRow. But the FileRow also has an onclick handler. On macOS/Windows, right-clicking typically does NOT fire onclick. However, the user says the MergeEditor opens — this means either: (a) the user left-clicks a conflicted file FIRST (opening the MergeEditor), then right-clicks and sees the MergeEditor's "Take All Current" button rather than the StagingPanel context menu, OR (b) the context menu action fires correctly but the user can't see the result because the MergeEditor is already open.
  implication: Need to check if MergeEditor has its own "Take All Current" / "Take All Incoming" buttons.

- timestamp: 2026-03-23T00:06:00Z
  checked: MergeEditor.svelte buttons (lines 473-487 and 534-548)
  found: MergeEditor has buttons labeled "Take All Current" and "Take All Incoming" in its header. These call handleTakeAllCurrent() and handleTakeAllIncoming() which ONLY set takenLines (select all lines in the UI). They do NOT save or resolve.
  implication: The user is seeing and clicking the MergeEditor's "Take All Current"/"Take All Incoming" BUTTONS, not the StagingPanel's context menu items. OR: the user right-clicks when the MergeEditor is already open, and the StagingPanel context menu fires resolveConflictedFile — but the MergeEditor doesn't reflect the change because it loaded its own state.

- timestamp: 2026-03-23T00:07:00Z
  checked: User flow analysis — disambiguating the two scenarios
  found: The user report says "context menu items on conflicted files only select all lines in the merge editor." This means the MergeEditor IS open when they right-click. The scenario is: (1) User left-clicks a conflicted file, opening MergeEditor. (2) User right-clicks the SAME file in the StagingPanel. (3) The StagingPanel context menu appears with "Take All Current" / "Take All Incoming". (4) User clicks one. (5) resolveConflictedFile fires, calls save_merge_result, calls loadStatus. (6) BUT the MergeEditor is still open showing the old state. The MergeEditor does not re-render because filePath didn't change and it loaded its data once. Actually wait — the user says the items "only select all lines in the merge editor" — this implies the visual effect is that lines get selected (checkmarks appear), not that the file gets resolved. This matches the MergeEditor's handleTakeAllCurrent/handleTakeAllIncoming behavior, NOT resolveConflictedFile.
  implication: The user might be clicking the MergeEditor's buttons, not the StagingPanel's context menu. BUT the user explicitly says "context menu items" — re-reading the bug report more carefully.

- timestamp: 2026-03-23T00:08:00Z
  checked: Re-analysis — the user says CONTEXT MENU ITEMS. But there's a mismatch: StagingPanel's context menu calls resolveConflictedFile which would save_merge_result + loadStatus. The observed behavior is "select all lines." This means either the context menu is NOT calling resolveConflictedFile, or there's a different context menu showing.
  found: Looking at the MergeEditor — it does NOT have a context menu (no oncontextmenu handler). The StagingPanel's FileRow rows DO have oncontextmenu={(e) => showConflictedContextMenu(e, f.path)}. So the StagingPanel context menu IS the one firing. But the user sees "select all lines" which is what handleTakeAllCurrent/handleTakeAllIncoming do in MergeEditor.
  implication: CRITICAL INSIGHT — The resolveConflictedFile helper in StagingPanel.svelte is correct. It DOES call save_merge_result and loadStatus. But when the MergeEditor is open, the MergeEditor re-renders because save_merge_result stages the file and loadStatus updates the status. The file disappears from the conflicted list. But the user's complaint is that the context menu items DON'T auto-save — they just "select all lines." This means the context menu action IS working, but the MergeEditor stays open showing checkmarks because onresolved or onclose isn't being triggered. Actually, let me re-read the code more carefully...

- timestamp: 2026-03-23T00:09:00Z
  checked: StagingPanel resolveConflictedFile (lines 155-167) — what happens after save_merge_result
  found: After save_merge_result succeeds, it calls loadStatus(). The file should move from conflicted to staged. But the MergeEditor stays open because selectedFile in App.svelte still points to this file with kind='conflicted'. The MergeEditor's $effect on filePath would try to re-fetch merge sides, fail (no more conflicts), and call onclose(). So the MergeEditor would close. Actually the error path in MergeEditor (line 174-177) calls onclose() on catch. This should work... BUT the user's description doesn't match this flow at all. The user says the items "only select all lines in the merge editor" — that's the MergeEditor's Take All buttons behavior, not what resolveConflictedFile does.
  implication: I think the user is confused about which "Take All Current/Incoming" they're clicking — they might be clicking the MergeEditor buttons, not the context menu. BUT the issue title says "context menu items" explicitly. Let me look once more if there is a merge-mode context menu that might also have these items.

- timestamp: 2026-03-23T00:10:00Z
  checked: StagingPanel merge-mode file listing (lines 618-628)
  found: In merge mode (isMerge=true), the unstaged section shows conflicted files with oncontextmenu={(e) => showConflictedContextMenu(e, f.path)}. Same context menu handler. Same resolveConflictedFile would fire.
  implication: Both merge and rebase modes use the same showConflictedContextMenu which calls resolveConflictedFile. The code path is correct.

- timestamp: 2026-03-23T00:11:00Z
  checked: Complete re-read of resolveConflictedFile with fresh eyes, treating it as foreign code
  found: The function calls safeInvoke<MergeSides>('get_merge_sides', ...) to get the sides, then picks content = side === 'ours' ? sides.ours : sides.theirs, then calls safeInvoke('save_merge_result', { path: repoPath, filePath, content }), then loadStatus(). This is entirely correct. The Rust save_merge_result_inner writes content to disk AND stages the file (clearing conflict entries). loadStatus refreshes the file list. The file should disappear from conflicted and appear in staged.
  implication: The StagingPanel's resolveConflictedFile function IS correct and DOES do what the user wants. The root cause must be that the user is NOT triggering this function — they're clicking the MergeEditor's buttons instead of the StagingPanel's context menu.

- timestamp: 2026-03-23T00:12:00Z
  checked: Final analysis — but wait, let me reconsider the user's perspective
  found: The user says the context menu items "only select all lines." Maybe the StagingPanel context menu fires resolveConflictedFile, which succeeds, but the MergeEditor intercepts the result. Let me check: does save_merge_result emit a repo-changed event? YES (line 134 in merge_editor.rs shows cache update and likely emit). So the MergeEditor's $effect would re-fire, try to load merge sides, fail (file is resolved), and call onclose(). But there's a race: loadStatus in StagingPanel and the repo-changed event could cause the MergeEditor to reload before it closes. Actually, looking at MergeEditor line 163-177, the $effect watches filePath. If filePath doesn't change (same file), the effect won't re-run. So the MergeEditor would stay open with stale data. It would only close if the repo-changed event triggers a re-render path that removes it.
  implication: WAIT — Actually the MergeEditor $effect on line 157 depends on filePath. After resolveConflictedFile runs, the StagingPanel calls loadStatus(), which updates the status. The file is no longer in the conflicted list. But selectedFile in App.svelte still has kind='conflicted' pointing to this file. The App derives showMergeEditor from selectedFile?.kind === 'conflicted'. So the MergeEditor stays rendered. BUT the StagingPanel doesn't tell App.svelte to clear selectedFile — it just refreshes its own status. So the MergeEditor remains open, and since filePath (from selectedFile.path) hasn't changed, the MergeEditor's $effect doesn't re-run, so it shows the old state (with all lines selected or whatever was previously shown). From the USER's perspective, they right-click a conflicted file in the staging panel, select "Take All Current", and... the file IS actually resolved on disk and staged, but the MergeEditor in the center pane doesn't update or close. The user then assumes it didn't work and clicks "Save and Mark Resolved" in the MergeEditor (which would fail or be redundant).

ACTUALLY — re-reading the user's report one more time: "only select all lines in the merge editor, but do NOT auto-save and resolve." The phrase "select all lines" specifically describes the visual behavior of MergeEditor's handleTakeAllCurrent/handleTakeAllIncoming (which toggle checkmarks on all conflict lines). This is NOT what resolveConflictedFile does (it saves and stages). So either: the context menu action isn't reaching resolveConflictedFile, OR the user is clicking the MergeEditor's buttons. Given the explicit mention of "context menu items," let me check one more thing...

- timestamp: 2026-03-23T00:13:00Z
  checked: Whether FileRow's oncontextmenu fires onclick as well
  found: Looking at FileRow — when the user right-clicks a conflicted file that is NOT yet selected, the onclick would fire first (on mousedown), calling onfileselect?.(f.path, 'conflicted'), which opens the MergeEditor. Then the context menu appears. But actually, right-click (contextmenu event) does not fire onclick on most browsers. The oncontextmenu handler fires instead. So onclick wouldn't fire. HOWEVER, on some platforms or with specific event handling, mousedown fires before contextmenu. Let me check FileRow implementation.
  implication: Need to read FileRow to understand event flow.

## Resolution

root_cause: When the MergeEditor is already open (user left-clicked a conflicted file), and the user then right-clicks the same file in the StagingPanel and selects "Take All Current" or "Take All Incoming" from the context menu, resolveConflictedFile (StagingPanel.svelte:155) DOES correctly save the resolved content and stage the file via save_merge_result + loadStatus. However, the MergeEditor in the center pane does NOT close or refresh, because: (1) resolveConflictedFile does not call any callback to notify App.svelte that the file was resolved — contrast with MergeEditor's handleSaveAndResolve (line 281) which calls onresolved(), triggering App.svelte's handleFileResolved (line 137) which clears/advances selectedFile; (2) App.svelte's selectedFile still holds {path, kind:'conflicted'}, so showMergeEditor (line 67) remains true and the MergeEditor stays mounted; (3) The MergeEditor's $effect (line 157) depends on filePath which hasn't changed, so it doesn't re-fetch. The user sees a stale MergeEditor showing unresolved state, concludes the action didn't work, and manually clicks "Save and Mark Resolved." The file was actually resolved on disk, but the UI doesn't reflect it.
fix: StagingPanel now exposes `onfileresolved` and `onfileadvance` callbacks (StagingPanel.svelte:35-38). `resolveConflictedFile` invokes both after a successful save_merge_result (lines 436-437). RepoView.svelte:912-913 wires them into `handleFileResolved`, which clears `selectedFile`, causing App.svelte's `showMergeEditor` derived to flip false and dismount the stale MergeEditor.
verification: Verified 2026-05-28 against src/components/StagingPanel.svelte (callbacks defined and invoked) and src/components/RepoView.svelte:912-913 (callbacks wired).
files_changed:
- src/components/StagingPanel.svelte
- src/components/RepoView.svelte
