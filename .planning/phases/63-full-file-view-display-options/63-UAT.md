---
status: diagnosed
phase: 63-full-file-view-display-options
source: [63-01-SUMMARY.md, 63-02-SUMMARY.md]
started: 2026-03-30T00:00:00Z
updated: 2026-03-30T00:20:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Display Option Toggle Buttons
expected: The diff toolbar shows three new toggle buttons with icons (Space, Pilcrow, TextWrap). Clicking each button toggles between active/inactive states. State persists across navigation.
result: issue
reported: "It worked, but it flickered. When I open the diff back again it had nothing selected initially. Then I could see it read the values and apply them because it took a while for the icons to get selected."
severity: minor

### 2. Staging Disabled When Whitespace Ignored
expected: With a diff open in Hunk mode, toggle "Ignore Whitespace" on. All Stage Hunk and Stage Line buttons become disabled and show a tooltip explaining why staging is unavailable.
result: pass

### 3. Word Wrap Toggle
expected: Open a diff with long lines that overflow horizontally. Toggle the TextWrap button on. Long lines wrap within the viewport instead of requiring horizontal scroll. Toggle off restores horizontal overflow.
result: pass

### 4. Full File View
expected: Switch diff view mode to "Full". The diff displays the entire file as a continuous scrollable document — no hunk headers, no staging buttons, with a two-column line number gutter (old/new).
result: issue
reported: "It works only when I select full and then close the diff panel and open it back again. It does not work when switching from hunk to full view live."
severity: major

### 5. Invisible Characters
expected: Toggle the "Show Invisibles" button on. Spaces in diff lines render as middle dots (·) and tabs render as rightwards arrows (→). Toggle off returns to normal whitespace rendering.
result: pass

### 6. Trailing Whitespace Highlight
expected: With "Show Invisibles" on, trailing whitespace at the end of lines is highlighted with a distinct warning background color, visually distinguishing it from mid-line whitespace.
result: skipped
reason: User sees trailing space as middle dot which is visually acceptable; no need for separate highlight treatment

### 7. Diff Re-fetch on Whitespace Toggle
expected: Toggle "Ignore Whitespace" on. The diff content re-fetches from the backend — whitespace-only changes disappear from the diff. Toggle off restores original diff with whitespace changes visible.
result: issue
reported: "It's not working. I tried indenting a file and toggling whitespace on and off and I still see the same exact diff."
severity: major

## Summary

total: 7
passed: 3
issues: 3
pending: 0
skipped: 1
blocked: 0

## Gaps

- truth: "Toggle buttons show persisted active state immediately when diff opens"
  status: failed
  reason: "User reported: buttons flicker — show inactive briefly then update to persisted state"
  severity: minor
  test: 1
  root_cause: "DiffPanel.svelte initializes toggle state to false (lines 45-48), then $effect loads persisted values async via Promise.all (lines 60-71). First render shows defaults, second render shows persisted values."
  artifacts:
    - path: "src/components/DiffPanel.svelte"
      issue: "Sync defaults + async load = guaranteed flicker on mount"
  missing:
    - "Delay rendering toggle buttons until preferences load, or lift preferences to parent so they're resolved before DiffPanel mounts"
  debug_session: ".planning/debug/toggle-buttons-flicker.md"

- truth: "Switching from Hunk to Full view mode updates the diff display live"
  status: failed
  reason: "User reported: only works after closing and reopening diff panel, not when switching modes live"
  severity: major
  test: 4
  root_cause: "Race condition: handleViewModeChange calls setDiffShowFullFile() (async) but fires ondiffoptionschange() immediately without awaiting. buildDiffOptions in RepoView reads the stale store value."
  artifacts:
    - path: "src/components/DiffPanel.svelte"
      issue: "setDiffShowFullFile() not awaited before ondiffoptionschange() on line 78-80"
    - path: "src/components/RepoView.svelte"
      issue: "buildDiffOptions reads showFullFile from store which still has old value"
  missing:
    - "Await setDiffShowFullFile() before calling ondiffoptionschange()"
  debug_session: ".planning/debug/viewmode-switch-no-live-update.md"

- truth: "Toggling Ignore Whitespace re-fetches diff and removes whitespace-only changes"
  status: failed
  reason: "User reported: toggling ignore whitespace on/off shows the same exact diff, indentation changes still visible"
  severity: major
  test: 7
  root_cause: "Two issues: (1) diff.rs line 39 uses opts.ignore_whitespace_change() (git -b flag, only ignores amount changes) instead of opts.ignore_whitespace() (git -w flag, ignores all whitespace). (2) Same async race as test 4: setDiffIgnoreWhitespace() not awaited before ondiffoptionschange()."
  artifacts:
    - path: "src-tauri/src/commands/diff.rs"
      issue: "Line 39: ignore_whitespace_change() should be ignore_whitespace()"
    - path: "src/components/DiffPanel.svelte"
      issue: "setDiffIgnoreWhitespace() not awaited before ondiffoptionschange()"
  missing:
    - "Change ignore_whitespace_change to ignore_whitespace in diff.rs"
    - "Await setDiffIgnoreWhitespace() before calling ondiffoptionschange()"
    - "Update test_diff.rs to cover indentation (not just amount changes)"
  debug_session: ".planning/debug/ignore-whitespace-toggle.md"
