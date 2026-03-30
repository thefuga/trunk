# Phase 64: Split View - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-30
**Phase:** 64-split-view
**Areas discussed:** Row alignment strategy, Staging interactions, Hunk presentation, Panel layout

---

## Row Alignment Strategy

### How should split view align rows when a hunk has unequal adds/deletes?

| Option | Description | Selected |
|--------|-------------|----------|
| Paired + phantom rows | Match Delete/Add lines 1:1, insert blank phantom rows on shorter side. GitHub/VS Code approach. | ✓ |
| Interleaved blocks | Show all Delete lines on left, Add lines on right, without 1:1 pairing. | |
| You decide | Claude picks best approach. | |

**User's choice:** Paired + phantom rows
**Notes:** None

### What should phantom (spacer) rows look like visually?

| Option | Description | Selected |
|--------|-------------|----------|
| Subtle striped background | Diagonal stripe or crosshatch pattern via CSS. VS Code/GitKraken style. | |
| Empty row, muted background | Empty rows with slightly darker/lighter background. Clean and minimal. | ✓ |
| You decide | Claude picks style fitting existing theme. | |

**User's choice:** Empty row, muted background
**Notes:** None

---

## Staging Interactions

### Where should hunk action buttons appear in split view?

| Option | Description | Selected |
|--------|-------------|----------|
| In the hunk header row, spanning both panels | Hunk header spans full width. Buttons on right side. Consistent with hunk view. | ✓ |
| Floating per-panel buttons | Each panel gets own action buttons. More granular but complex. | |
| You decide | Claude picks based on HunkView consistency. | |

**User's choice:** In the hunk header row, spanning both panels
**Notes:** None

### How should line selection for staging work in split view?

| Option | Description | Selected |
|--------|-------------|----------|
| Click lines on either side | Click Add lines on right or Delete lines on left. Same as hunk view. | |
| Right panel only | Line selection only on right (new) panel. Simpler mental model. | ✓ |
| You decide | Claude picks matching HunkView behavior. | |

**User's choice:** Right panel only
**Notes:** None

---

## Hunk Presentation

### Should split view show hunk headers between sections?

| Option | Description | Selected |
|--------|-------------|----------|
| Hunk headers | Show @@ headers spanning both panels. Where staging buttons live. | ✓ |
| Continuous (no headers) | Flatten all hunks like FullFileView. | |
| You decide | Claude picks based on staging decisions. | |

**User's choice:** Hunk headers
**Notes:** None

---

## Panel Layout

### How should the two panels be visually divided?

| Option | Description | Selected |
|--------|-------------|----------|
| 1px border line | Thin vertical border. Clean, minimal. 50/50 split. | |
| Resizable divider | Draggable 4px handle for resizing. More flexible. | ✓ |
| You decide | Claude picks simplest approach. | |

**User's choice:** Resizable divider
**Notes:** None

### Where should line numbers appear in each panel?

| Option | Description | Selected |
|--------|-------------|----------|
| Single gutter per panel | Left panel: old line numbers. Right panel: new line numbers. Space-efficient. | ✓ |
| Dual gutters per panel | Both old and new numbers per panel (like hunk view). | |
| You decide | Claude picks best use of horizontal space. | |

**User's choice:** Single gutter per panel
**Notes:** None

### Should origin symbols (+/-/space) be shown in split view?

| Option | Description | Selected |
|--------|-------------|----------|
| No origin symbols | Color backgrounds already indicate add/delete. Saves horizontal space. | ✓ |
| Keep origin symbols | Show +/-/space like hunk view. Consistent but redundant. | |
| You decide | Claude decides based on other Git GUIs. | |

**User's choice:** No origin symbols
**Notes:** None

---

## Architecture Clarification (User-Initiated)

### Is split a third view mode or an independent layout toggle?

| Option | Description | Selected |
|--------|-------------|----------|
| Two independent toggles | Layout (inline/split) + Content (hunk/full). 4 combinations. | ✓ |
| Keep 3-way selector | Hunk/Full/Split as single toggle. No split+full combo. | |

**User's choice:** Two independent toggles
**Notes:** User clarified: "switching from inline to split, we should still be able to toggle between hunk view and full view." This changes the architecture from a 3-way ViewMode to two orthogonal dimensions.

### How should the two independent toggles appear in the toolbar?

| Option | Description | Selected |
|--------|-------------|----------|
| Two segmented controls | [Hunk\|Full] \| [Inline\|Split]. Two small controls side by side. | ✓ |
| Segmented + icon toggle | Keep [Hunk\|Full] segmented. Add split as icon toggle button. | |
| You decide | Claude picks best toolbar layout. | |

**User's choice:** Two segmented controls
**Notes:** None

---

## Claude's Discretion

- Exact Lucide icons for inline/split toggle buttons
- CSS custom property for phantom row muted background
- Internal data structure for row pairing and phantom generation
- Drag handle implementation approach for resizable divider
- Scroll sync implementation technique

## Deferred Ideas

None — discussion stayed within phase scope
