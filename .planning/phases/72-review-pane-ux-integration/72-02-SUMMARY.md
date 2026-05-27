---
phase: 72-review-pane-ux-integration
plan: 02
subsystem: in-window-toolbar / review-mode-entry
tags: [svelte5, toolbar, tauri-event, tdd, a11y, active-state-styling]
dependency_graph:
  requires: []
  provides:
    - "Toolbar Review button (in-window review-toggle emitter)"
    - "App.svelte → Toolbar reviewActive prop pass-through"
    - ".toolbar-btn-active CSS class (referenced by Plan 04 visual sanity)"
  affects:
    - src/components/Toolbar.svelte
    - src/components/Toolbar.test.ts
    - src/App.svelte
tech_stack:
  added: []   # zero new packages (CONTEXT.md Net effect; UI-SPEC §Registry Safety)
  patterns:
    - "Frontend emit() onto Tauri event bus (first frontend emitter in repo)"
    - "Svelte 5 class:directive toggle for active-state styling (replaces inline-style ternary anti-pattern)"
    - "aria-pressed ARIA toggle-button pattern"
    - "Locally-declared vi.mock for mock-identity guarantees across static + dynamic imports"
key_files:
  created: []
  modified:
    - src/components/Toolbar.svelte
    - src/components/Toolbar.test.ts
    - src/App.svelte
decisions:
  - "Use MessagesSquare lucide icon for the Toolbar Review button (CONTEXT.md design choice 1; UI-SPEC §Iconography)"
  - "Active-state styling via class toggle (.toolbar-btn-active) — not inline style — to avoid recreating the deleted blue-button anti-pattern (UI-SPEC §Color Wiring discipline; CLAUDE.md never inline colors)"
  - "Drop shared tauri-mock helper import from Toolbar.test.ts in favor of fully-local vi.mock declarations — the only reliable way to guarantee that Toolbar.svelte's static emit import resolves to the SAME vi.fn instance the test asserts on (Rule 1/3 fix discovered at GREEN time; documented in the test file)"
metrics:
  duration_minutes: 22
  completed_date: 2026-05-27
requirements: [REQ-72-1a, REQ-72-2, G-71-B]
---

# Phase 72 Plan 02: Toolbar Review Button + App Prop Wiring Summary

One-liner: Added a `MessagesSquare`-icon Review toggle button to the in-window Toolbar that emits the existing `review-toggle` Tauri event and shows accent-fill + `aria-pressed` active styling when `reviewActive===true`, with App.svelte piping the App-owned `reviewPanelOpen` rune through as the prop.

## Why

Per CONTEXT.md root-cause read, review mode's "is review on?" axis had no in-window indicator — it was only reachable via a hidden macOS menu item. This plan externalizes that axis to a Toolbar button with proper active-state styling, closing gap G-71-B for the in-window toolbar surface. The Cmd+Shift+R accelerator lands in Plan 04; the Cmd+Shift+R path also routes through the same `review-toggle` event so this plan ships the visible toggle indicator that Plan 04's shortcut will exercise.

## What

### `src/components/Toolbar.svelte` (modified, 25 lines added)

- **Line 7:** add `MessagesSquare` to the existing `@lucide/svelte` import block.
- **Line 12:** change `import { listen } from "@tauri-apps/api/event"` → `import { emit, listen } from "@tauri-apps/api/event"`.
- **Line 25:** add `reviewActive: boolean;` to `Props` interface.
- **Line 28:** add `reviewActive` to destructuring: `let { repoPath, remoteState, undoRedo, reviewActive }: Props = $props();`.
- **Lines 30-32:** add `function handleReviewToggle() { void emit("review-toggle"); }` — fire-and-forget on the in-process event bus (RESEARCH Pattern 1 + Assumption A5; emit on the local event bus never user-fails).
- **Lines 238-244:** new CSS rule pair appended after `.toolbar-btn:disabled` (which sat at lines 226-230) — the new active class:
  ```css
  .toolbar-btn.toolbar-btn-active {
    background: var(--color-accent);
    color: var(--color-on-accent);
  }
  .toolbar-btn.toolbar-btn-active:hover:not(:disabled) {
    background: var(--color-accent);
  }
  ```
  Both rules use only existing CSS custom-property tokens (UI-SPEC §Color; CLAUDE.md "never inline colors").
- **Lines 289-298:** new rightmost `<div class="toolbar-group">` containing one `<button class="toolbar-btn" class:toolbar-btn-active={reviewActive} aria-pressed={reviewActive} onclick={handleReviewToggle}><MessagesSquare size={14} /> Review</button>`. Clones the existing Branch/Stash/Pop group structure (Toolbar.svelte:264-274) — no new spacing values introduced (UI-SPEC §Spacing empty-set contract).

### `src/App.svelte` (modified, 1 line)

- **Line 584:** `<Toolbar>` instantiation extended with `reviewActive={reviewPanelOpen}` prop pass-through. `reviewPanelOpen` is the existing App-owned rune declared at App.svelte:57 and flipped by the `review-toggle` listener at App.svelte:557 (per RESEARCH Finding 2 + Pitfall 4 — the Toolbar is App-mounted, NOT per-tab, so the App-owned `reviewPanelOpen` is the correct source of truth; the per-tab `reviewSession` rune is unreachable here).

### `src/components/Toolbar.test.ts` (modified)

- Added `fireEvent` to the existing `@testing-library/svelte` import block.
- Replaced the `import "../__tests__/helpers/tauri-mock"` line with fully-local `vi.mock` declarations for `@tauri-apps/api/event` (listen + **emit**), `@tauri-apps/api/core` (invoke), and `@tauri-apps/plugin-dialog` (the other mocks the helper provided that Toolbar transitively touches). Reason: the shared helper's `vi.mock` calls fire when the helper module evaluates — which is AFTER Toolbar.svelte's static `import { emit }` has already resolved to the real module. Local `vi.mock` at the top of this file is hoisted BEFORE any imports, so Toolbar.svelte sees the mocked module at static-import time. This guarantees mock identity between the component's `emit` call and the test's `vi.mocked(emit)` assertion. Documented in-file.
- Added `reviewActive: false` to every existing `render(Toolbar, { props: {...} })` call site (TypeScript otherwise rejects the missing prop now that the Props interface requires it).
- Added two new `it` blocks at the bottom of the existing `describe("Toolbar", …)` block:
  - `"emits review-toggle on click"` — renders Toolbar with `reviewActive: false`, looks up the Review button by `getByRole("button", { name: /Review/ })`, fires a click, asserts `vi.mocked(emit).toHaveBeenCalledWith("review-toggle")`.
  - `"shows active state when reviewActive is true"` — renders with `reviewActive: true`, asserts the Review button has class `toolbar-btn-active` AND `aria-pressed="true"`.

## Commits

| Phase | Hash      | Message                                                              |
| ----- | --------- | -------------------------------------------------------------------- |
| RED   | `aae6864` | `test(72-02): add failing tests for Toolbar Review button`           |
| GREEN | `a12565d` | `feat(72-02): add Toolbar Review button + active-state styling`      |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Mock identity collision between shared helper and Toolbar's `emit` import**
- **Found during:** Task 2 GREEN verification — vitest reported `Number of calls: 0` on `vi.mocked(emit)`, then `Cannot read properties of undefined (reading 'transformCallback')` indicating the real `@tauri-apps/api/core` was being invoked by `emit`, proving Toolbar.svelte had resolved `emit` to the REAL module while the test's dynamic `await import("@tauri-apps/api/event")` had resolved to the MOCKED module.
- **Root cause:** The shared helper at `src/__tests__/helpers/tauri-mock.ts` declares `vi.mock("@tauri-apps/api/event", …)` only inside the helper file. `vi.mock` is hoisted only within the file that contains it — when the test file does `import "../__tests__/helpers/tauri-mock"` AFTER `import Toolbar from "./Toolbar.svelte"`, the static import of Toolbar resolves first, BEFORE the helper's `vi.mock` registers. Trying to patch `emit` into the helper produced a second collision: the helper's later registration overwrote the test file's hoisted local registration, and re-cached the module with the helper's `vi.fn` instances — but the Toolbar component had already cached the prior (real) instance.
- **Fix:** Switched Toolbar.test.ts to follow the locally-declared-mocks pattern that other tests asserting on Tauri behavior already use (CommitGraph.test.ts, RebaseEditor.test.ts, RepoView.test.ts, StagingPanel.test.ts). All Tauri module mocks now live at the top of Toolbar.test.ts where they are hoisted before any imports. Helper import removed. Helper file itself unchanged.
- **Files modified:** `src/components/Toolbar.test.ts` (additional changes beyond Task 1 spec).
- **Commit:** `a12565d` (GREEN).

**2. [Rule 1 - Bug] biome `noNonNullAssertion` violation introduced by `reviewBtn!`**
- **Found during:** `just check` final gate.
- **Issue:** The originally-spec'd test pattern (`screen.getByText("Review").closest("button")!`) returns `HTMLButtonElement | null` and required `!` to satisfy the click signature, but biome forbids non-null assertions in this codebase.
- **Fix:** Replaced with `screen.getByRole("button", { name: /Review/ })` which returns `HTMLElement` directly (throws if not found, eliminating the null union without an assertion). Equivalent or stronger assertion behavior.
- **Files modified:** `src/components/Toolbar.test.ts`.
- **Commit:** `a12565d` (GREEN).

### No-inline-style discipline preserved

`grep -n 'style="background:' src/components/Toolbar.svelte` returns no matches near the new Review button. The active-state styling is applied via the `class:toolbar-btn-active` directive only — the deleted blue-button anti-pattern at `RepoView.svelte:815-827` (eight lines of inline `style="background: {…ternary…}; color: {…}; border: 1px solid {…};"`) is explicitly NOT recreated.

### Authentication gates

None.

## Verification

| Gate                                                                  | Result   |
| --------------------------------------------------------------------- | -------- |
| `bun run vitest run src/components/Toolbar.test.ts`                   | 9 / 9 ✓  |
| `bun run vitest run` (full suite, no regressions in other 49 files)   | 537 / 537 ✓ |
| `bunx svelte-check --tsconfig ./tsconfig.json`                        | 0 errors / 0 warnings |
| `just check` (fmt + biome + svelte-check + clippy + cargo-test + vitest) | exit 0 |
| `grep -n 'style="background:' src/components/Toolbar.svelte` (anti-pattern guard) | no matches ✓ |
| Biome warnings before vs after                                        | 4 → 3 (introduced none; one of the original 4 was the new `reviewBtn!`, now fixed) |

## TDD Gate Compliance

- RED gate: `aae6864` `test(72-02): add failing tests for Toolbar Review button` — confirmed 2 new tests failing before any production code.
- GREEN gate: `a12565d` `feat(72-02): add Toolbar Review button + active-state styling` — all 9 tests passing after production code shipped.
- REFACTOR gate: not needed; GREEN code is already at simplest form (single-button group, single class toggle, single emit call).

## Known Stubs

None.

## Threat Flags

None — this plan adds zero new threat surface beyond what was pre-registered in the plan's `<threat_model>` (all rows N/A: Tauri event bus is webview↔core in-process only; no new IPC handler; no new capability; no new logging surface; no new packages).

## Self-Check: PASSED

- File `src/components/Toolbar.svelte`: FOUND
- File `src/components/Toolbar.test.ts`: FOUND
- File `src/App.svelte`: FOUND
- Commit `aae6864` (RED): FOUND
- Commit `a12565d` (GREEN): FOUND
- Substring `reviewActive: boolean` in Toolbar.svelte: FOUND (line 25)
- Substring `MessagesSquare` in Toolbar.svelte: FOUND (line 7, line 297)
- Substring `import { emit, listen } from "@tauri-apps/api/event"` in Toolbar.svelte: FOUND (line 12)
- Substring `void emit("review-toggle")` in Toolbar.svelte: FOUND (line 31)
- Substring `class:toolbar-btn-active={reviewActive}` in Toolbar.svelte: FOUND (line 293)
- Substring `aria-pressed={reviewActive}` in Toolbar.svelte: FOUND (line 294)
- Substring `.toolbar-btn-active` in Toolbar.svelte: FOUND (lines 238, 242)
- Substring `style="background:` near Review button in Toolbar.svelte: NOT FOUND (anti-pattern absent ✓)
- Substring `reviewActive={reviewPanelOpen}` in App.svelte: FOUND (line 584)
- Substring `emits review-toggle on click` in Toolbar.test.ts: FOUND
- Substring `shows active state when reviewActive is true` in Toolbar.test.ts: FOUND
- Substring `emit: vi.fn().mockResolvedValue(undefined)` in Toolbar.test.ts: FOUND
- Substring `vi.mocked(emit)).toHaveBeenCalledWith("review-toggle")` in Toolbar.test.ts: FOUND
- Substring `toHaveClass("toolbar-btn-active")` in Toolbar.test.ts: FOUND
- Substring `toHaveAttribute("aria-pressed", "true")` in Toolbar.test.ts: FOUND
