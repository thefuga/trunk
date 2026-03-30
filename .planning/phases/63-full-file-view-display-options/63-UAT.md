---
status: complete
phase: 63-full-file-view-display-options
source: [63-01-SUMMARY.md, 63-02-SUMMARY.md]
started: 2026-03-30T00:00:00Z
updated: 2026-03-30T00:15:00Z
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
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Switching from Hunk to Full view mode updates the diff display live"
  status: failed
  reason: "User reported: only works after closing and reopening diff panel, not when switching modes live"
  severity: major
  test: 4
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""

- truth: "Toggling Ignore Whitespace re-fetches diff and removes whitespace-only changes"
  status: failed
  reason: "User reported: toggling ignore whitespace on/off shows the same exact diff, indentation changes still visible"
  severity: major
  test: 7
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
