---
status: resolved
trigger: "graph-dot-zoom-drift: Commit graph dots drift out of alignment with commit row text when scrolling down thousands of commits at non-100% zoom levels"
created: 2026-04-05T00:00:00Z
updated: 2026-05-29T00:00:00Z
resolved: 2026-05-29
resolution: code-verified
---

## Resolution (2026-05-29)

Confirmed CONFIRMED hypothesis fix is present in `src/components/CommitGraph.svelte`:
- `let svgRowHeight = $state(displaySettings.rowHeight)` (line ~107) — tracks the real measured height instead of the fixed `ROW_HEIGHT=26`.
- `const cy = (row) => row * svgRowHeight + svgRowHeight / 2` (line ~1288) — SVG dot/path Y-math now uses the measured height.
- `bind:measuredItemHeight={svgRowHeight}` (line ~1990) — VirtualList feeds its measured fractional row height back to the overlay.

This directly removes the fixed-vs-measured mismatch identified as the confirmed root cause, so cumulative drift at non-100% zoom cannot accumulate. Closed on code evidence per operator decision (2026-05-29). **Low-risk follow-up:** a one-time visual smoke (zoom ~125%, scroll a large repo) would be the ideal final confirmation; not blocking.

## Current Focus

hypothesis: CONFIRMED - SVG overlay uses fixed ROW_HEIGHT (26) for Y-coordinate math while VirtualList measures actual fractional heights at non-100% zoom, causing cumulative drift
test: Fix applied - svgRowHeight bound to VirtualList's measured average height
expecting: Dots stay aligned at all zoom levels and scroll positions
next_action: Awaiting human verification in real app

## Symptoms

expected: Commit dots in the graph column should remain vertically centered with their corresponding commit row text at all zoom levels and scroll positions
actual: As user scrolls down thousands of commits, the dot progressively drifts toward the bottom of the commit row. The misalignment accumulates — it starts fine at the top and gets worse the further you scroll. This ONLY happens when the browser/window is zoomed in (e.g. 110%, 125%). Zooming back to 100% realigns everything.
errors: None
reproduction: 1. Open a repo with thousands of commits. 2. Zoom in the window (Cmd+Plus). 3. Scroll down through thousands of commits. 4. Observe dots drifting relative to row text.
started: Unknown -- likely present since commit graph was implemented.

## Eliminated

## Evidence

- timestamp: 2026-04-05
  checked: graph-constants.ts - ROW_HEIGHT constant
  found: ROW_HEIGHT = 26 (fixed integer constant)
  implication: All SVG coordinate math is based on this fixed value

- timestamp: 2026-04-05
  checked: overlay-paths.ts - makePathContext cy function
  found: cy = (row) => row * rowHeight + rowHeight / 2 -- uses fixed rowHeight from settings
  implication: SVG path endpoints, dot positions, and pill positions all use fixed arithmetic

- timestamp: 2026-04-05
  checked: CommitGraph.svelte - cy function (line 1135)
  found: Same formula: row * displaySettings.rowHeight + displaySettings.rowHeight / 2
  implication: SVG dots positioned independently from DOM rows

- timestamp: 2026-04-05
  checked: VirtualList.svelte - calculateTransformY and height measurement
  found: VirtualList measures actual row heights via getBoundingClientRect().height and uses those for transformY positioning. At non-100% zoom, measured heights differ from the CSS-declared 26px due to sub-pixel snapping.
  implication: DOM rows accumulate fractional heights while SVG uses integer multiples -- this is the root cause of drift

- timestamp: 2026-04-05
  checked: CommitRow.svelte - row height CSS
  found: style:height="{rowHeight}px" sets CSS height to ROW_HEIGHT (26px)
  implication: At non-100% zoom, browser snaps 26px to device pixel grid, getBoundingClientRect reports fractional CSS-pixel height (e.g. 25.8 or 26.18)

## Resolution

root_cause: Sub-pixel rounding accumulation at non-100% browser zoom. The SVG overlay computed dot/path/pill Y-coordinates using `row * ROW_HEIGHT` (fixed 26px constant), while the VirtualList measured actual row heights via getBoundingClientRect() which returns fractional values at non-100% zoom due to device-pixel snapping. Over thousands of rows, the cumulative difference between `N * 26` and `sum(measured_heights[0..N])` grew progressively, causing dots to drift from their corresponding text rows.

fix: Added a `measuredItemHeight` bindable prop to VirtualList that exposes the actual measured average row height. CommitGraph binds `svgRowHeight` to this value and uses it (via `svgSettings`) for all SVG coordinate computations (overlay paths, dots, ref pills, scroll-to-center). The CSS row height stays at the declared ROW_HEIGHT constant, but the SVG coordinate math now uses the browser's actual measured height, keeping SVG positions synchronized with DOM positions at any zoom level.

verification: svelte-check passes (0 errors), all 434 vitest tests pass, full `just check` suite (fmt, biome, svelte-check, clippy, cargo-test, vitest) passes.

files_changed:
- src/components/VirtualList.svelte
- src/components/CommitGraph.svelte
