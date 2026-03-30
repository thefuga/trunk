---
status: diagnosed
trigger: "Switching diff view mode from Hunk to Full does NOT update the diff display live. It only works after closing the diff panel and reopening it."
created: 2026-03-30T00:00:00Z
updated: 2026-03-30T00:00:00Z
---

## Current Focus

hypothesis: The view mode switch in DiffPanel correctly updates the `viewMode` state (so DiffViewer reactively switches from HunkView to FullFileView), BUT FullFileView renders with stale hunk-only diff data because `ondiffoptionschange` triggers a re-fetch with `showFullFile: true` via the store -- and this async re-fetch completes AFTER the component has already rendered with the old hunk-limited data.
test: Trace the data flow from viewMode change to re-render
expecting: FullFileView renders with hunk-limited data (missing inter-hunk context lines)
next_action: DIAGNOSED -- return root cause

## Symptoms

expected: Switching from "Hunk" to "Full" view mode should instantly update the diff display to show full file content with all context lines
actual: Display does not visually update on mode switch. Only works after closing and reopening the diff panel.
errors: No error messages -- silent incorrect behavior
reproduction: Open a diff, click "Full" in the toolbar segmented control
started: Phase 63 implementation (FullFileView is new)

## Eliminated

- hypothesis: DiffToolbar does not call the mode change handler
  evidence: DiffToolbar.svelte line 52 correctly calls `onviewmodechange(mode.value)` on click
  timestamp: 2026-03-30

- hypothesis: DiffPanel does not update viewMode state
  evidence: `handleViewModeChange` on line 74-81 of DiffPanel.svelte correctly sets `viewMode = mode` and passes it to DiffViewer
  timestamp: 2026-03-30

- hypothesis: DiffViewer does not react to viewMode prop changes
  evidence: DiffViewer.svelte uses `{#if viewMode === "hunk"}` / `{:else if viewMode === "full"}` at lines 78-105. Svelte 5 reactive `{#if}` blocks DO react to prop changes. The conditional rendering IS reactive.
  timestamp: 2026-03-30

## Evidence

- timestamp: 2026-03-30
  checked: DiffPanel.svelte handleViewModeChange (lines 74-81)
  found: When mode changes to "full", it (1) sets viewMode, (2) calls setDiffShowFullFile(true), (3) calls ondiffoptionschange()
  implication: The ondiffoptionschange callback triggers a data re-fetch from the backend

- timestamp: 2026-03-30
  checked: RepoView.svelte ondiffoptionschange handler (lines 662-666)
  found: It calls refetchFileDiff which calls buildDiffOptions() then re-invokes the diff command
  implication: The re-fetch reads showFullFile from the async store and sends it to the backend

- timestamp: 2026-03-30
  checked: RepoView.svelte buildDiffOptions (lines 169-176)
  found: buildDiffOptions reads getDiffShowFullFile() from the Tauri persistent store (async)
  implication: The re-fetch is async and the data eventually arrives, but the key issue is what FullFileView receives

- timestamp: 2026-03-30
  checked: RepoView.svelte refetchFileDiff (lines 319-336)
  found: It updates `stagingDiffFiles` with the new data. But `currentDiffFiles` (passed to DiffPanel as `fileDiffs`) is computed from `stagingDiffFiles`.
  implication: When the re-fetch completes, `fileDiffs` prop SHOULD update and FullFileView SHOULD re-render with full file data

- timestamp: 2026-03-30
  checked: DiffViewer.svelte conditional rendering (lines 78-105)
  found: viewMode === "full" renders FullFileView with {fileDiffs}. The component receives the same fileDiffs array that was fetched for hunk mode.
  implication: There is a timing issue -- FullFileView renders immediately with the OLD hunk data, then re-fetch completes and updates fileDiffs

- timestamp: 2026-03-30
  checked: Full data flow for what user sees
  found: The REAL issue is that FullFileView flattens all hunk lines (`fd.hunks.flatMap(h => h.lines)` at line 50 of FullFileView.svelte). When `showFullFile` is false in the backend, hunks only contain changed lines + context lines (typically 3 lines). When `showFullFile` is true, the backend returns ALL lines of the file. So if the data hasn't been re-fetched yet, FullFileView shows the hunk data which IS the same as what HunkView showed -- it just looks the same, no visual change.
  implication: The actual update DOES happen but only after the async re-fetch completes. The perceived "no update" is because the re-fetch takes time and the initial render uses old data.

- timestamp: 2026-03-30
  checked: Whether the re-fetch actually arrives and causes re-render
  found: The ondiffoptionschange IS async and DOES update stagingDiffFiles. The issue is likely that `setDiffShowFullFile(shouldShowFull)` is also async (writes to Tauri store). There is a RACE CONDITION: `handleViewModeChange` calls `setDiffShowFullFile(true)` and then `ondiffoptionschange()`. Both are fire-and-forget (no await). `ondiffoptionschange` calls `refetchFileDiff` which calls `buildDiffOptions()` which reads `getDiffShowFullFile()` from the store. If the store write hasn't completed yet when the read happens, `getDiffShowFullFile()` returns false (old value), and the re-fetch uses `showFullFile: false` -- fetching hunk-only data AGAIN.
  implication: THIS IS THE ROOT CAUSE. Race condition between store write and store read.

## Resolution

root_cause: Race condition in `handleViewModeChange` (DiffPanel.svelte lines 74-81). The function calls `setDiffShowFullFile(shouldShowFull)` (async store write, not awaited) followed by `ondiffoptionschange?.()` (which triggers `refetchFileDiff` -> `buildDiffOptions` -> `getDiffShowFullFile()`). Because `setDiffShowFullFile` is not awaited, the store read in `buildDiffOptions` races with the write and reads the OLD value (`showFullFile: false`). The re-fetch therefore requests hunk-only data, and FullFileView renders with the same hunk-limited lines -- appearing unchanged. When the panel is closed and reopened, the store write has long since completed, so the next open reads the correct `showFullFile: true` and fetches full file data.
fix:
verification:
files_changed: []
