---
plan: 31-01
phase: 31
status: complete
completed: 2026-03-15
---

# Summary: Plan 31-01 — Unified Title Bar + Toolbar

## What was done

- **Task 31-01-01:** Set `"decorations": false` in `src-tauri/tauri.conf.json` window config to remove the native OS title bar.
- **Task 31-01-02:** Added `data-tauri-drag-region` attribute and `padding-left: 78px` (macOS traffic light clearance) to the bar in `App.svelte`. Added a 36px drag region bar to `WelcomeScreen.svelte` for window dragging on the welcome screen.

## Files changed

- `src-tauri/tauri.conf.json` — added `"decorations": false`
- `src/App.svelte` — added drag region and traffic light padding to bar
- `src/components/WelcomeScreen.svelte` — added drag region bar, restructured layout

## Test results

- `npm test`: 126/126 passed
