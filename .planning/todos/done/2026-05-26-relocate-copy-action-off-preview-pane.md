---
created: 2026-05-26T23:50:00.000Z
title: Relocate Copy action off the preview pane and onto the comments view
area: ui
origin: 71-UAT.md gap G-71-A
files:
  - src/components/ReviewDocPreview.svelte
  - src/components/ReviewPanel.svelte
---

## Problem

Phase 71 placed the Copy button on the `ReviewDocPreview` header, assuming users would navigate to the markdown preview pane to export the document. User feedback during 71-UAT:

> "We don't need to preview the markdown, we can just copy it directly from the comments view."

The preview pane is unnecessary friction on the copy path. The Copy action should live where the user already is — the comments view.

## Decisions needed

1. **Host location** — where on the comments view does Copy live? Candidates:
   - Toolbar action button (top-right of the comments view, mirroring common "export" patterns)
   - Per-comment "copy this excerpt" affordance (alongside or instead of doc-level)
   - Both (per-comment + whole-doc), if the user wants both granularities
2. **Fate of the preview pane** — does it still have a purpose?
   - Keep as a read-only verification view, behind a discoverable toggle
   - Fold into a "view raw markdown" disclosure
   - Remove entirely if Copy was its only justification
3. **What Copy emits** — full doc only, or scoped to the current comment / selection?

## Carry-forward from 71

Preserve these from the existing implementation when relocating:

- Awaited `writeText` with `try/catch` + `showToast(\`Failed to copy: ${msg}\`, "error")` — silent failure unacceptable for artifact copies
- `instanceof Error` narrowing (not `as` casts) in catch blocks
- `clearTimeout` before `setTimeout` for re-clickable affordance
- 1500ms "✓ Copied" affordance window (tests 3 and 4 from 71-UAT not yet manually validated — re-test against new placement)
- `vi.useFakeTimers` test pattern from `ReviewDocPreview.test.ts`

## Scope

Independent UI relocation. Likely a small Phase 72 (or a discrete plan within a broader "review-pane UX" phase that also closes the G-71-B dead-button issue below).

## Resolution (2026-05-28)

Closed by Phase 72. `ReviewDocPreview.svelte` was deleted (Plan 04). The Copy affordance now lives on `ReviewPanel.svelte` (Phase 72 Copy state at line 131; explicit "deleted Phase 71 preview component" note at line 889), carrying forward the awaited writeText + try/catch + 1500ms re-clickable-affordance pattern from 71.
