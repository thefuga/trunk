---
status: resolved
trigger: "Text selection on shift+click in DiffPanel"
created: 2026-03-18T23:00:00Z
updated: 2026-05-28T00:00:00Z
verified_at: 2026-05-28
---

## Current Focus

hypothesis: e.preventDefault() on click is too late - browser text selection on shift+click is triggered during mousedown, not click
test: Examine the event handler binding - is it on click or mousedown?
expecting: Handler is bound to onclick, which fires after mousedown (when selection already happened)
next_action: Verify by reading the event binding in the template

## Symptoms

expected: Shift+click should only trigger custom line selection without browser text selection
actual: Browser native text selection is also triggered alongside custom line highlights
errors: N/A (UX issue, not error)
reproduction: Shift+click to range-select diff lines in DiffPanel
started: After line selection feature was added (phase 34-02)

## Eliminated

## Evidence

- timestamp: 2026-03-18T23:00:00Z
  checked: Previous fix commit 69beeef
  found: Added e.preventDefault() in shift+click branch of handleLineClick, and user-select:none on selectable lines
  implication: The fix attempted two things - preventDefault on click event, and CSS user-select:none. If the bug persists, these are insufficient.

- timestamp: 2026-03-18T23:01:00Z
  checked: DiffPanel.svelte line 557 - event binding
  found: Handler is bound via `onclick` attribute, not `onmousedown`
  implication: The click event fires AFTER mousedown. Browser text selection on shift+click is initiated during the mousedown phase, before click fires. So e.preventDefault() on click is too late to prevent the selection.

- timestamp: 2026-03-18T23:01:30Z
  checked: DiffPanel.svelte line 555 - user-select CSS
  found: `user-select: none` is set on selectable lines via inline style
  implication: This SHOULD prevent text selection on individual lines. However, shift+click text selection can span across multiple elements, and the browser may still create a selection range between the anchor and the shift-clicked element regardless of user-select on individual children if the parent container allows selection. Need to check the parent.

## Resolution

root_cause: The `e.preventDefault()` is called on the `click` event (line 162), but browser text selection on shift+click is initiated during the `mousedown` event, which fires before `click`. By the time the click handler runs, the browser has already extended the native text selection. The `user-select: none` CSS on individual selectable lines partially helps, but context lines (origin === 'Context') have `user-select: auto`, and shift+click range selection can still select text across those context lines and parent containers.
fix: Each selectable diff-line div now binds `onmousedown={(e) => { if (isSelectable && e.shiftKey) e.preventDefault(); }}` — intercepting browser text selection during the mousedown phase before it starts, while leaving non-selectable context lines text-selectable for copy.
verification: Verified 2026-05-28 across all three diff-line renderers: src/components/diff/HunkView.svelte:352, src/components/diff/SplitView.svelte:353, src/components/diff/FullFileView.svelte:173.
files_changed:
- src/components/diff/HunkView.svelte
- src/components/diff/SplitView.svelte
- src/components/diff/FullFileView.svelte
