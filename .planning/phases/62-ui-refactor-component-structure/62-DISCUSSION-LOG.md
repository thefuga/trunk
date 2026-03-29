# Phase 62: UI Refactor & Component Structure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 62-ui-refactor-component-structure
**Mode:** auto (discuss)
**Areas discussed:** Component decomposition, View mode switching, Line number gutter, Toolbar design

---

## Component Decomposition

| Option | Description | Selected |
|--------|-------------|----------|
| Three-level split (DiffToolbar + DiffViewer + DiffLineRenderer) | Clean separation: toolbar owns controls, viewer dispatches by mode, line renderer handles per-line rendering | ✓ |
| Two-level split (DiffToolbar + DiffContent) | Simpler but viewer and line rendering stay coupled | |
| Keep monolith, add view mode only | Minimal change but grows the 667-line file further | |

**User's choice:** [auto] Three-level split (recommended default)
**Notes:** DiffPanel stays as thin shell owning state and staging handlers. Maintains existing RepoView props interface.

---

## View Mode Switching

| Option | Description | Selected |
|--------|-------------|----------|
| Segmented control in toolbar (Hunk/Full/Split) | Standard pattern, persisted to LazyStore, clear visual state | ✓ |
| Dropdown select | Less discoverable, takes fewer pixels | |
| Tab-style buttons | More visual weight than needed for 3 options | |

**User's choice:** [auto] Segmented control in toolbar (recommended default)
**Notes:** Only Hunk mode functional in this phase. Full and Split are stubs.

---

## Line Number Gutter

| Option | Description | Selected |
|--------|-------------|----------|
| Two-column gutter (old + new) before origin symbol | GitHub/GitKraken standard, shows both line numbers | ✓ |
| Single column (new lineno only) | Simpler but loses old-file context | |
| Inline at end of line | Unconventional, hard to scan | |

**User's choice:** [auto] Two-column gutter (recommended default)
**Notes:** old_lineno and new_lineno already exist on DiffLine from Rust backend.

---

## Toolbar Design

| Option | Description | Selected |
|--------|-------------|----------|
| View mode (left) + filename (center) + actions (right) | Clean layout, existing actions stay in place | ✓ |
| All controls left-aligned | Simple but wastes space | |
| Floating toolbar above diff | Unusual pattern, creates layout complexity | |

**User's choice:** [auto] View mode left, filename center, actions right (recommended default)
**Notes:** Phase 63 controls (context lines, whitespace, etc.) not added here.

---

## Claude's Discretion

- File organization (subdirectory vs flat)
- DiffLineRenderer as component vs snippet
- CSS class naming for gutter
- View mode switch animation

## Deferred Ideas

None — analysis stayed within phase scope
