---
status: diagnosed
trigger: "Overflow pill context menus missing - right-click on expanded overflow pill branch names shows no context menu"
created: 2026-03-15T00:00:00Z
updated: 2026-03-15T00:00:00Z
---

## Current Focus

hypothesis: The expanded overflow pill div renders each ref as a plain div with no oncontextmenu handler
test: Read the overflow expansion template (CommitGraph.svelte lines 807-833)
expecting: Missing oncontextmenu handlers on individual ref items
next_action: Return diagnosis

## Symptoms

expected: Each branch/tag name in expanded overflow pill should show right-click context menu (Rename/Delete)
actual: Right-clicking on branch names in the expanded overflow pill does nothing
errors: None (silent missing feature)
reproduction: Have a commit with 2+ refs, hover to expand overflow pill, right-click any branch name
started: Always — context menu was only wired to the single-pill SVG elements, never to the overflow expansion

## Eliminated

(none needed — root cause identified on first inspection)

## Evidence

- timestamp: 2026-03-15T00:01:00Z
  checked: CommitGraph.svelte lines 717-731 (single pill rendering)
  found: The capsule rect has oncontextmenu={(e) => showPillContextMenu(e, pill)} at line 730
  implication: Single pills have context menus properly wired

- timestamp: 2026-03-15T00:02:00Z
  checked: CommitGraph.svelte lines 807-833 (overflow expansion rendering)
  found: The {#each hoveredPill.allRefs as ref} loop (line 824) renders each ref as a plain <div> with NO oncontextmenu handler
  implication: This is the root cause — overflow expansion items lack context menu handlers

- timestamp: 2026-03-15T00:03:00Z
  checked: showPillContextMenu signature (line 366)
  found: Takes (e: MouseEvent, pill: OverlayRefPill) — it expects an OverlayRefPill, NOT a RefLabel
  implication: Can't directly pass a RefLabel from allRefs to showPillContextMenu — need to construct a synthetic OverlayRefPill or refactor showPillContextMenu to accept RefLabel

- timestamp: 2026-03-15T00:04:00Z
  checked: RefLabel interface (types.ts line 17-23)
  found: RefLabel has {name, short_name, ref_type, is_head, color_index} — it has ref_type and short_name (label) and is_head which are the fields showPillContextMenu actually uses
  implication: showPillContextMenu only uses pill.refType, pill.label, and pill.isHead — all available on RefLabel (as ref_type, short_name, is_head)

## Resolution

root_cause: |
  The expanded overflow pill (CommitGraph.svelte lines 807-833) renders each ref from
  hoveredPill.allRefs as a plain <div> with NO oncontextmenu handler. The context menu
  is only wired to the single-pill SVG elements (capsule rect at line 730, icon at line 736,
  text span at line 761), which are the non-expanded pill representations.

  When a user hovers over an overflow pill and it expands to show all refs, the expansion
  is a separate HTML overlay div (lines 809-833) that replaces the SVG pill visually.
  Each ref inside is rendered with {#each hoveredPill.allRefs as ref} (line 824) as a
  simple div containing only an icon and text — no oncontextmenu binding.

fix: (not applied — diagnosis only)
verification: (not applied — diagnosis only)
files_changed: []
