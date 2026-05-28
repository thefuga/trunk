---
created: 2026-05-26T23:50:00.000Z
title: Fix clunky review-pane entry/exit and remove dead blue Review button
area: ui
origin: 71-UAT.md gap G-71-B
files:
  - src/components/ReviewPanel.svelte
  - src/components/ReviewDocPreview.svelte
  - src/App.svelte
---

## Problem

User feedback during 71-UAT surfaced two coupled issues with the review pane (introduced in Phase 70):

> "We should have a better way of going in and out of the review pane. That menu option is too clunky, and for some reason we have a blue review button in the review pane that doesn't do anything. We should think about how to fit this new review pane in the UI, and a good user experience way of going in and out of it."

### Sub-issue B1 — Clunky navigation

Entering and leaving the review pane goes through a menu route that feels heavy for a frequent action. The pane doesn't yet feel integrated into the surrounding UI.

### Sub-issue B2 — Dead blue Review button

A blue "Review" button is visible inside the review pane and does nothing on click. Source location not yet confirmed — start by grepping `src/components/ReviewPanel.svelte` and adjacent files for the button. Likely introduced in Phase 70 alongside the pane host.

Per `ownership.md`: this is ours regardless of which phase introduced it.

## Decisions needed

1. **Entry/exit pattern** — what replaces the current menu route?
   - Persistent toolbar action with icon + tooltip
   - Keyboard shortcut (e.g., Cmd+R) with discoverable hint
   - Contextual button on the comments view (couples naturally with the Copy relocation in the sibling TODO)
2. **Dead button fate** — wire it up to a real action, remove it, or repurpose it?
3. **Pane lifecycle** — is the review pane modal, side-by-side, full-screen, dismissible by Esc?

## Coupling

This work pairs naturally with the Copy-relocation TODO (`2026-05-26-relocate-copy-action-off-preview-pane.md`). Both questions answer "how does the review pane fit into the comments-view-centric UX?" Consider planning them as one phase.

## Investigation first step

```bash
grep -rn -iE 'review.*button|button.*review' src/components/ src/App.svelte
```

Identifies the dead button before deciding wire-vs-remove.

## Resolution (2026-05-28)

Closed by Phases 72 + 73. Review pane entry/exit now goes through the Toolbar Review button and the View → Start/End Code Review menu item (RepoView.svelte:812 references both routes). Phase 73-02 added the in-pane End-review button as a danger-tinted sibling of the Copy button (ReviewPanel.svelte:913). The "dead blue Review button" referenced in 71-UAT no longer appears in the current ReviewPanel.
