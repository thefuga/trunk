---
status: complete
phase: 26-svg-ref-pills
source: [26-01-SUMMARY.md, 26-02-SUMMARY.md]
started: 2026-03-15T12:00:00Z
updated: 2026-03-15T12:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. SVG Ref Pills Render
expected: Branch, tag, and stash refs appear as capsule-shaped SVG pills with lane-colored backgrounds next to their commit rows in the graph overlay.
result: pass

### 2. Connector Lines
expected: Horizontal connector lines run from pill right edge to commit dot, using the commit's lane color.
result: pass

### 3. Text Truncation
expected: Long ref names are truncated with ellipsis to fit within available space. Text measurement uses Canvas API with caching.
result: pass

### 4. Remote Dimming and HEAD Styling
expected: Remote-only pills are dimmed at 67% opacity. Non-HEAD pills at brightness(0.75). HEAD pill at full brightness with bold text.
result: pass

### 5. Overflow Badge and Hover Expansion
expected: When multiple refs exceed available space, an overflow +N badge appears. Hovering expands to show all refs with animation.
result: pass

### 6. Ref Type Icons
expected: Tags show a diamond icon. Stashes show a flag icon in the pill.
result: pass

### 7. HTML Pills Removed
expected: CommitRow no longer renders HTML-based ref pills. All ref pill rendering is handled by the SVG overlay.
result: pass

### 8. All 115 Tests Pass
expected: Full test suite passes — 26 new tests (text-measure, ref-pill-data, overlay-visible pills) plus all existing tests.
result: pass

## Summary

total: 8
passed: 8
issues: 0
pending: 0
skipped: 0

## Gaps

[none]
