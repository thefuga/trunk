# Phase 71: Output (Clipboard) — Research

**Researched:** 2026-05-26
**Domain:** Svelte 5 + Tauri 2 frontend — single Copy button on `ReviewDocPreview` writing markdown to clipboard with explicit success/failure feedback.
**Confidence:** HIGH

## Summary

Phase 71 is intentionally tiny: one new button inside an existing component, one `await writeText(markdown)` call against a plugin that is already installed and already permitted, and two well-known UX outcomes (in-button "✓ Copied" affordance for success, `showToast(..., "error")` for failure). Every locked decision in `71-CONTEXT.md` (D-01..D-08) is implementable with code patterns that already exist elsewhere in the repo. No new dependencies. No backend changes. No new Tauri capabilities. No new Tauri command.

The only thing this phase *intentionally* introduces is divergence from the three existing `writeText(...).catch(() => {})` fire-and-forget callsites (`App.svelte:133`, `CommitDetail.svelte:72`, `CommitGraph.svelte:757`). Those copy a single short string where silent failure is acceptable; here, the rendered review *is* the user's artifact, so the failure mode must surface.

**Primary recommendation:** Implement the Copy button inline in `ReviewDocPreview.svelte` (the host file already exists, the dock point — `.preview-spacer` — already exists, the visual template — the `← Back to comments` button — already exists in the same file). Hold the "copied" affordance state in a single `$state<boolean>` rune flipped by a `setTimeout`. Wrap `writeText` in `try`/`catch`; on success, flip the rune; on failure, call `showToast(\`Failed to copy: ${err.message ?? String(err)}\`, "error")`. No extraction to a helper module — the logic is ~15 lines, single-callsite, and Beck's "fewest elements" wins here over a hypothetical-reuse helper.

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Affordance**
- **D-01:** Single `Copy` button. Lives in the existing `.preview-spacer` flex cell in `ReviewDocPreview.svelte` header (Phase 70 left it intentionally empty for this).
- **D-02:** Icon + label, Lucide icon (`Clipboard` or `ClipboardCopy`). Match the existing `← Back to comments` button styling exactly (border + transparent bg + hover transition) — no primary-filled accent.

**Click feedback**
- **D-03:** On successful copy, button swaps its label+icon to `✓ Copied` for ~1.5s, then reverts. Local component state, no global toast for the success case.
- **D-04:** Button remains clickable during the "Copied" window — user can re-copy without waiting for the revert.

**Failure handling**
- **D-05:** On clipboard write failure, show an error toast with the underlying reason: `showToast("Failed to copy: <error message>", "error")` using the existing `src/lib/toast.svelte.ts` facade (same pattern as `ReviewPanel`).
- **D-06:** No fallback modal / no retry — surface the error and let the user act.

**Keyboard**
- **D-07:** No app-level keyboard binding. `Cmd/Ctrl+C` is left entirely to native text selection inside the preview `<pre>` (the user can select a region and copy it natively).

**Empty-doc edge case**
- **D-08:** Not reachable. Phase 70 D-11 gates `Generate` on `comment_count > 0`, so the preview view is never shown with an empty markdown string. No defensive handling.

### Claude's Discretion
- Exact Lucide icon name (`Clipboard` vs `ClipboardCopy`) — pick whichever reads cleaner in context.
- The exact "Copied" revert duration (target ~1.5s; tune if it feels off in dev).
- Whether to await the `writeText` Promise in a `try/catch` vs a `.then/.catch` chain — both fine; pick the one that matches the local component style.

### Deferred Ideas (OUT OF SCOPE)
- **Save-to-file (OUT-02)** dropped entirely from REQUIREMENTS.md. Full design ground (default dir = `dirname(repoPath)`, filename `trunk-review-<YYYYMMDD-HHMM>.md`, atomic tmp+rename, `dialog:allow-save` capability) preserved in CONTEXT.md for future revival but NOT in this phase.
- ROADMAP/REQUIREMENTS follow-up edits (descope Phase 71 title, drop success criteria #2/#3, REQUIREMENTS-OUT-02 already removed).

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| OUT-01 | User can copy the generated markdown to the clipboard | `writeText` from `@tauri-apps/plugin-clipboard-manager@2.3.2` (already installed); `clipboard-manager:allow-write-text` already granted in `src-tauri/capabilities/default.json:14`; ReviewDocPreview already accepts `markdown: string` prop; dock cell `.preview-spacer` already reserved at `ReviewDocPreview.svelte:26`; visual template `← Back to comments` button styling at lines 50-64 of the same file. |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Copy button render | Browser / Svelte component | — | Pure UI affordance, lives in `ReviewDocPreview.svelte`. |
| "Copied" affordance state | Browser / Svelte component | — | Transient local UI state; `$state<boolean>` + `setTimeout` — no persistence, no cross-component awareness. |
| Clipboard write IPC | Tauri plugin (`clipboard-manager`) | OS clipboard | Frontend calls `writeText` -> plugin invokes Rust side internally -> OS clipboard. No app-level Rust command needed. |
| Failure surfacing | Browser / Svelte component (via `showToast` facade) | — | Frontend-only; toast facade is already in `src/lib/toast.svelte.ts`. |
| Source of truth for copied text | Browser / Svelte component prop | — | `markdown: string` already flows in as a prop from `ReviewPanel` (`ReviewDocPreview.svelte:15`); same string the `<pre>` renders. No re-fetch. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@tauri-apps/plugin-clipboard-manager` | 2.3.2 [VERIFIED: `package.json:20` + on-disk `node_modules/@tauri-apps/plugin-clipboard-manager/package.json` + `index.d.ts`] | Plain-text clipboard write via `writeText(text): Promise<void>` | Already installed, already used at 3 other callsites, already permitted in capabilities. |
| `@lucide/svelte` | ^0.577.0 [VERIFIED: `package.json:19` + already imported at `ReviewPanel.svelte:8` (`FileText`, `MessageSquarePlus`)] | Icon library; provides `Clipboard` / `ClipboardCopy` named exports per `@lucide/svelte` standard surface | Already in dependencies, already used in the sibling component for icons in the same visual context. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `src/lib/toast.svelte.ts` | (in-repo) | `showToast(message, kind, ms?)` facade [VERIFIED: file read; signature is `(message: string, kind: ToastKind = "success", ms = 3000)`] | Failure case only (D-05). Success case stays local per D-03. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `await writeText(markdown)` with `try/catch` | `.then(...).catch(...)` chain | Both surface the failure; the `try/catch` form reads more linearly in an `async` handler. Discretion per CONTEXT.md. |
| `Clipboard` icon | `ClipboardCopy` icon | Both exist in Lucide. `Clipboard` is the plain icon; `ClipboardCopy` has a small ↩ accent. Discretion. |
| Hand-rolling Phase 71 helper module | Inline in `ReviewDocPreview.svelte` | Inline wins by Beck #4 (fewest elements); ~15 lines, single callsite, no other consumer. |
| Reusing the fire-and-forget pattern (`writeText(x).catch(() => {})`) | Awaited `writeText` with explicit failure surfacing | Intentionally diverging per CONTEXT.md "Established Patterns" — the artifact IS the product; silent failure is unacceptable here. |

**Installation:** None. All dependencies already in `package.json`.

**Version verification:**
```bash
node -e 'console.log(require("@tauri-apps/plugin-clipboard-manager/package.json").version)'
# -> 2.3.2 (verified on disk 2026-05-26)
node -e 'console.log(require("@lucide/svelte/package.json").version)'
# -> 0.577.x range per package.json
```

## Package Legitimacy Audit

> **Skip rationale:** This phase installs NO new packages. All required libraries are already in `package.json` and have been used by the project for prior phases (Phase 65 onward for the Tauri plugins, Phase 69 for `@lucide/svelte` in `ReviewPanel.svelte`). slopcheck protocol does not apply.

| Package | Disposition |
|---------|-------------|
| `@tauri-apps/plugin-clipboard-manager@2.3.2` | Already installed; verified in `node_modules`; published by official `tauri-apps` org. No change. |
| `@lucide/svelte@^0.577.0` | Already installed; already imported by `ReviewPanel.svelte`. No change. |

## Architecture Patterns

### System Architecture Diagram

```
ReviewPanel.svelte (host)
   |
   |-- passes markdown: string  prop ----------------+
   v                                                  |
ReviewDocPreview.svelte                              |
   +-- header                                         |
   |    +-- [← Back to comments] button (existing)  |
   |    +-- .preview-spacer  <-- DOCK POINT          |
   |    +-- [Copy] button (NEW, this phase)  -------+
   |          |                                      |
   |          v                                      |
   |       onCopy() handler                          |
   |          |                                      |
   |          +-- await writeText(markdown)  -------> @tauri-apps/plugin-clipboard-manager
   |          |       |                                |
   |          |       +-- success: copied = true      |
   |          |       |             setTimeout 1500   |
   |          |       |             -> copied = false |
   |          |       |                                |
   |          |       +-- failure: showToast(...)     v
   |          |                       toast.svelte.ts -> Toast UI
   v
   <pre class="preview-body">{markdown}</pre>
```

**Data-flow trace, primary use case ("user clicks Copy, succeeds"):**
1. User clicks the Copy button.
2. `onCopy()` calls `await writeText(markdown)` (the prop value, same string the `<pre>` renders).
3. Promise resolves -> set `copied = true`; schedule `setTimeout(() => copied = false, 1500)`.
4. Template reactively swaps button content from `[Clipboard icon] Copy` to `[✓] Copied`.
5. After 1.5s, content reverts. Button remained clickable throughout (D-04).

**Data-flow trace, failure case:**
1-2 as above.
3. Promise rejects with `err`.
4. `onCopy()` catches, calls `showToast(\`Failed to copy: ${err.message ?? String(err)}\`, "error")`.
5. Toast appears via `src/components/Toast.svelte`, auto-dismisses after 3000ms (toast facade default).
6. `copied` state is NOT flipped; button remains in default state. User can retry.

### Recommended Project Structure

No new files. Modifications:
```
src/components/
└── ReviewDocPreview.svelte   # ONE FILE MODIFIED — new button + handler + state + style block addition
```

### Pattern 1: Awaited `writeText` with explicit failure surfacing
**What:** Wrap the plugin call in `try`/`catch` inside an async click handler; surface the failure via `showToast`.
**When to use:** This phase only — diverges from the fire-and-forget callsites elsewhere because the artifact (the review doc) is the product, not a convenience copy.
**Example:**
```typescript
// Source: pattern derived from ReviewPanel.svelte:278-287 (onGenerateClick) +
//         App.svelte:133 (writeText import) + toast.svelte.ts (showToast signature)
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Clipboard } from "@lucide/svelte";  // or ClipboardCopy — discretion
import { showToast } from "../lib/toast.svelte.js";

let copied = $state(false);

async function onCopy() {
  try {
    await writeText(markdown);
    copied = true;
    setTimeout(() => { copied = false; }, 1500);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showToast(`Failed to copy: ${msg}`, "error");
  }
}
```

### Pattern 2: Mirror `← Back to comments` styling exactly (D-02)

The visual template lives in `ReviewDocPreview.svelte:50-64`:

```css
.back-button {
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  padding: 2px 8px;
  font-size: 12px;
  font-family: inherit;
}
.back-button:hover,
.back-button:focus-visible {
  color: var(--color-text);
  background: var(--color-hover);
}
```

The new `.copy-button` rule should be identical aside from layout helpers needed for icon+label flex (`display: inline-flex; align-items: center; gap: 4px;` — mirrors `.generate-button` in `ReviewPanel.svelte:742-760`). All colors come from CSS custom properties (`--color-border`, `--color-text`, `--color-text-muted`, `--color-hover`) per the project rule "Never inline colors".

### Pattern 3: Layout via flex (no positioning hacks)

The header is already `display: flex` (`ReviewDocPreview.svelte:40-48`). The `.preview-spacer` has `flex: 1`. The Copy button placed *after* the spacer naturally docks to the right edge — no `position: absolute`, no `margin-left: auto`. Matches CLAUDE.md "Never fight layout with positioning hacks".

### Anti-Patterns to Avoid
- **Re-deriving the markdown from `session`:** the prop `markdown: string` is already the canonical string the `<pre>` shows; copying anything else risks divergence. Use the prop verbatim.
- **Toast for the success case:** explicitly rejected by D-03 in favor of the in-button affordance. A toast would compete visually with the button's own swap.
- **Adding a Rust command:** the plugin handles the clipboard from frontend; a Rust command is pure surface-area inflation.
- **Inlining colors / hex values:** project rule. Always `var(--color-*)`.
- **Debouncing or disabling the button during the "Copied" window:** D-04 explicitly allows re-clicks during the window.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Clipboard write | Custom Rust command via `arboard` / `clipboard-rs` / shell | `@tauri-apps/plugin-clipboard-manager`'s `writeText` | Already installed, already permitted, already paved by 3 other callsites. |
| Toast / notification | Inline alert div, `window.alert`, native dialog | `showToast(msg, "error")` from `src/lib/toast.svelte.ts` | Already used by `ReviewPanel` for the analogous failure case. |
| Icon | Custom SVG inline | Lucide `Clipboard` / `ClipboardCopy` | Already in deps, already used in the same panel. |
| "Copied" feedback timer | `requestAnimationFrame` loop, `setInterval` | `setTimeout(() => copied = false, 1500)` once | Single transition, no cleanup complexity. |

**Key insight:** This phase is the textbook case for "use the platform." Three existing usage sites of `writeText`, one existing toast facade, one existing icon library, one existing dock point. Adding anything new here is over-engineering.

## Common Pitfalls

### Pitfall 1: Bare `Error.message` may be empty on opaque plugin errors
**What goes wrong:** `await writeText(...)` could reject with something that has no useful `.message`, leaving the toast as `"Failed to copy: "` with nothing after the colon.
**Why it happens:** Tauri plugin errors propagate as JS objects whose shape isn't strictly typed; `e` is `unknown` in TS strict mode.
**How to avoid:** Coerce defensively: `const msg = e instanceof Error ? e.message : String(e);`. `String({})` becomes `"[object Object]"` which is still better than empty. CLAUDE.md test bar would say: would the user see SOMETHING actionable? Yes — they see the toast appeared, even if the underlying reason is opaque.
**Warning signs:** Toast shows `"Failed to copy: "` with empty tail in dev. Fix at point of generation, not by mutating the toast facade.

### Pitfall 2: Stale `setTimeout` on rapid re-clicks
**What goes wrong:** User clicks Copy, succeeds, then clicks again 0.5s into the 1.5s window. Second click's timer fires at 2.0s and flips `copied = false` while the user is still mid-affordance from the second click.
**Why it happens:** No timer-tracking — both timeouts run in parallel.
**How to avoid:** Capture the timer handle and clear it before scheduling a new one:
```typescript
let copyTimer: ReturnType<typeof setTimeout> | null = null;
async function onCopy() {
  try {
    await writeText(markdown);
    if (copyTimer !== null) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => { copied = false; copyTimer = null; }, 1500);
  } catch (e) { /* … */ }
}
```
**Warning signs:** Affordance flickers off prematurely when re-copying. The unit-testable assertion: "two clicks in quick succession produce one continuous `copied=true` window that extends to the second click + 1500ms, not the first."

### Pitfall 3: TypeScript `unknown` on caught errors
**What goes wrong:** `catch (e)` types `e` as `unknown` under strict mode (TS 5.x default). Accessing `e.message` without narrowing is a type error.
**Why it happens:** TS strict mode + `useUnknownInCatchVariables` (default).
**How to avoid:** Always narrow: `e instanceof Error ? e.message : String(e)`. Matches Pitfall 1 fix.
**Warning signs:** `svelte-check` fails with `'e' is of type 'unknown'`.

### Pitfall 4: Importing `writeText` dynamically vs statically
**What goes wrong:** Inconsistency with the rest of the file or making the test mock unreachable.
**Why it happens:** `App.svelte:133` uses `await import(...)` (dynamic); `CommitDetail.svelte:3` and `CommitGraph.svelte:11` use static top-level imports. The mock at `CommitGraph.test.ts:56` uses `vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({...}))` which works for both shapes but is cleanest with the static form.
**How to avoid:** Use the **static** form: `import { writeText } from "@tauri-apps/plugin-clipboard-manager";` at the top of `ReviewDocPreview.svelte`. Matches the closer-by pattern (CommitDetail, CommitGraph), keeps the test mock idiomatic.
**Warning signs:** None in production; only matters for test cleanliness.

### Pitfall 5: Forgetting that `setTimeout` is non-deterministic under vitest fake timers
**What goes wrong:** Unit test asserts `copied === false` after 1500ms but the timer never advanced.
**Why it happens:** Default vitest doesn't fake timers.
**How to avoid:** In tests, use `vi.useFakeTimers()` in `beforeEach`, then `vi.advanceTimersByTime(1500)` to trigger the revert. The component-level test for `setTimeout(() => copied = false, 1500)` is a clean candidate. (Note: existing tests don't use fake timers — this would be the first.) Alternative: assert only the `copied = true` transition and treat the revert as integration-tested via observable DOM after time advance.

## Code Examples

### Full handler + state (verified pattern)

```typescript
// In ReviewDocPreview.svelte <script> block.
// Sources:
//   - writeText import:  src/components/CommitDetail.svelte:3
//   - try/catch + showToast: src/components/ReviewPanel.svelte:278-287 (onGenerateClick)
//   - icon usage: src/components/ReviewPanel.svelte:366-367 (<FileText size={14} />)
//   - state rune pattern: standard Svelte 5 (e.g., draftText in ReviewPanel.svelte:46)

import { Clipboard } from "@lucide/svelte";  // or ClipboardCopy (Discretion)
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { showToast } from "../lib/toast.svelte.js";

interface Props {
  markdown: string;
  onBack: () => void;
}

let { markdown, onBack }: Props = $props();

let copied = $state(false);
let copyTimer: ReturnType<typeof setTimeout> | null = null;

async function onCopy() {
  try {
    await writeText(markdown);
    if (copyTimer !== null) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => {
      copied = false;
      copyTimer = null;
    }, 1500);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showToast(`Failed to copy: ${msg}`, "error");
  }
}
```

### Template snippet (the dock point)

```svelte
<header class="preview-header">
  <button type="button" class="back-button" onclick={onBack}
    >← Back to comments</button>
  <span class="preview-spacer"></span>
  <button type="button" class="copy-button" onclick={onCopy}>
    {#if copied}
      <span aria-hidden="true">✓</span>
      <span>Copied</span>
    {:else}
      <Clipboard size={14} />
      <span>Copy</span>
    {/if}
  </button>
</header>
```

### Style block addition

```css
/* Source: mirrors .back-button (lines 50-64 of the same file) +
   .generate-button flex layout (ReviewPanel.svelte:742-760).
   All colors via theme tokens — CLAUDE.md rule. */
.copy-button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  cursor: pointer;
  padding: 2px 8px;
  font-size: 12px;
  font-family: inherit;
}
.copy-button:hover,
.copy-button:focus-visible {
  color: var(--color-text);
  background: var(--color-hover);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Browser `navigator.clipboard.writeText` directly | Tauri plugin `writeText` | Tauri 2 + plugin extraction | We use the plugin; the browser API would skip Tauri capability gating and is unavailable in the webview context anyway. |
| Fire-and-forget `.catch(() => {})` | Awaited `try/catch` with surfaced failure | This phase | Pattern divergence is intentional and isolated to this callsite. |

**Deprecated/outdated:** None applicable. `@tauri-apps/plugin-clipboard-manager@2.3.2` is the current 2.x line stable release.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The Lucide `Clipboard` and `ClipboardCopy` icons exist as named exports in `@lucide/svelte@^0.577.0`. | Standard Stack / Code Examples | LOW — Lucide is a stable, broad set; both names are canonical Lucide icons. If wrong, swap to any clipboard-ish Lucide name (e.g., `Files`). [ASSUMED — not verified against installed `@lucide/svelte` exports during research; planner can verify with `node -e 'console.log(Object.keys(require("@lucide/svelte")).filter(k => k.includes("lipboard")))'`]. |
| A2 | The Tauri plugin's `writeText` rejection's `.message` field is generally populated when the underlying OS clipboard write fails. | Pitfalls | LOW — Pitfall 1 already covers the empty-message case defensively. [ASSUMED — based on standard Tauri plugin error conventions; not exercised in this codebase's existing fire-and-forget callsites because they discard the error.] |

*All other claims are VERIFIED against on-disk files cited inline.*

## Open Questions

None. Every CONTEXT.md decision maps to a concrete code pattern that already exists in the repo. The two assumptions (A1, A2) are low-risk and self-mitigating.

## Environment Availability

> **Skipped:** This phase is purely a frontend code/config change inside files already in the project. No new tools, services, runtimes, CLIs, databases, or external dependencies. Existing `just check` (vitest + svelte-check + biome) is the only pipeline involved.

## Validation Architecture

> Status: nyquist_validation enabled (`.planning/config.json` -> `workflow.nyquist_validation: true`). tdd_mode enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest ^4.1.0 + @testing-library/svelte 5.3.1 + jsdom 29 [VERIFIED: `package.json:33-37`, `vite.config.ts` `test` block] |
| Config file | `vite.config.ts` (test config inlined via `/// <reference types="vitest/config" />`) [VERIFIED] |
| Setup | `vitest-setup.ts` (ResizeObserver + Element.prototype.animate stubs) [VERIFIED] |
| Quick run command | `npx vitest run src/components/ReviewDocPreview.test.ts` |
| Full suite command | `just check` (also runs fmt, biome, svelte-check, clippy, cargo-test) |
| Component mount supported? | YES — `@testing-library/svelte` `render`/`fireEvent`/`screen` already used by 20+ existing tests including `ReviewPanel.test.ts`. Components can be mounted, interacted with, and asserted in jsdom. |

### Phase Requirements -> Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OUT-01 | `onCopy` invokes `writeText` with the `markdown` prop verbatim | unit (component-mount, mocked plugin) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "writes markdown prop"` | ❌ Wave 0 |
| OUT-01 | Successful copy flips `copied` state to `true` (label/icon swap visible) | unit (component-mount) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "shows Copied affordance on success"` | ❌ Wave 0 |
| OUT-01 | `copied` reverts to `false` after ~1500ms (via `vi.useFakeTimers`) | unit (component-mount + fake timers) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "reverts after timeout"` | ❌ Wave 0 |
| OUT-01 | Button remains clickable during the "Copied" window (D-04); a second click during the window extends the window | unit (component-mount + fake timers) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "remains clickable during window"` | ❌ Wave 0 |
| OUT-01 | Failed `writeText` calls `showToast` with `"Failed to copy: <message>"` and `"error"` kind | unit (component-mount, mocked plugin rejects, mocked showToast) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "shows error toast on failure"` | ❌ Wave 0 |
| OUT-01 | On failure, `copied` state does NOT flip | unit (component-mount, mocked rejection) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "does not flip copied on failure"` | ❌ Wave 0 |
| OUT-01 | `← Back to comments` still works (regression — handler not displaced by new button) | unit (component-mount) | `npx vitest run src/components/ReviewDocPreview.test.ts -t "back button still invokes onBack"` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `npx vitest run src/components/ReviewDocPreview.test.ts`
- **Per wave merge:** `npm test` (full vitest suite — fast, all `src/**/*.test.ts`)
- **Phase gate:** `just check` (full pipeline: fmt + biome + svelte-check + clippy + cargo-test + vitest) before `/gsd:verify-work`

### Unit-testable behaviors (explicit list per request)

The Copy button's logic is unit-testable end-to-end with the established mock pattern (mock `@tauri-apps/plugin-clipboard-manager` -> `writeText: vi.fn()`, mock `../lib/toast.svelte.js` -> `showToast: vi.fn()`, both already used at `CommitGraph.test.ts:56` and `ReviewPanel.test.ts:25` respectively):

1. **`writeText` receives the markdown prop verbatim** — render `<ReviewDocPreview markdown="hello" onBack={...} />`, click the Copy button, assert `writeText` mock called with `"hello"` once.
2. **State transition `idle -> copied -> idle`** — assert the button DOM contains "Copy" pre-click, "Copied" immediately post-click (after `await tick()` + the resolved `writeText` mock microtask), and "Copy" again after `vi.advanceTimersByTime(1500)`.
3. **Re-click during window extends the window** — two clicks at t=0 and t=500 with fake timers; assert button still shows "Copied" at t=1600 (would have reverted at t=1500 had the first timer not been cleared) and reverts at t=2000.
4. **Failure path invokes `showToast` with formatted message** — make `writeText` mock reject with `new Error("plugin disabled")`; click; assert `showToast` mock called with `("Failed to copy: plugin disabled", "error")`.
5. **Failure path does NOT flip `copied`** — same scenario; assert button DOM still shows "Copy" (never "Copied").
6. **Unknown error coercion** — `writeText` rejects with `"raw string"` (not an Error); assert `showToast` called with `"Failed to copy: raw string"` (verifies the `String(e)` fallback).
7. **`onBack` regression** — click `← Back to comments`; assert the injected `onBack` callback fires once.

### Integration / UI-level behaviors

These are behaviors where component-mount unit tests give weaker signal and a `just dev` smoke or manual verification is more decisive:

1. **Actual clipboard contents** — the unit test mocks `writeText`; only `just dev` (or a Tauri integration test, which the project does not run) confirms the real OS clipboard contains the markdown after a click.
2. **Visual styling parity with `← Back to comments`** — pixel-level border/hover match. jsdom doesn't render CSS visually; even component tests can only assert classes, not painted appearance.
3. **Icon rendering** — Lucide SVGs render in jsdom (they're inline SVG), but the visual feel of `Clipboard` vs `ClipboardCopy` is a human-eyes decision per Discretion.
4. **Toast appearance + 3000ms auto-dismiss in real UI** — covered by existing `Toast.test.ts`; not re-tested here.
5. **Theme custom-property correctness** — the test can assert the class name `.copy-button` is present, but whether `var(--color-border)` resolves correctly across the light/dark theme is visual.

### Manual verification items

1. Click Copy in `just dev`, switch to another app (TextEdit, VS Code, etc.), paste — confirm full markdown lands.
2. Eyeball the "Copied" duration — 1500ms is the spec; tune to ~1200 or ~1800 if it feels off (Discretion in CONTEXT.md).
3. Visual parity: Copy button border, padding, hover color, font-size match `← Back to comments`. Take a screenshot if borderline.
4. Force a failure: revoke the `clipboard-manager:allow-write-text` capability temporarily in a `just dev` session and click — confirm the error toast surfaces with a readable message. **Restore the capability before commit.**
5. Rapid double-click test: spam Copy 3-4 times in 2s; confirm affordance behaves sanely (no flicker, no console errors).

### Out-of-scope for validation

- **Clipboard contents inside other apps** — once `writeText` resolves, the OS owns the data. Trust the plugin.
- **OS clipboard manager edge cases** — clipboard managers like Maccy/Paste/Clipy intercept writes. Out of scope; the user controls their own clipboard stack.
- **Race between `endSession` and `onCopy`** — Phase 70 D-11 + the preview-only mount path mean this isn't reachable; the preview can't be shown without an active session, and `markdown` is a prop snapshot at render time.
- **Re-render mid-copy with a different markdown prop** — also not reachable; ReviewPanel doesn't mutate `session.state.previewMarkdown` except via Generate (which re-mounts the preview anyway). If it ever became reachable, the in-flight `await` would resolve with the *first* markdown, which is the intended capture-at-click semantics.
- **OUT-02 (save-to-file)** — explicitly descoped.

### Wave 0 Gaps

- [ ] `src/components/ReviewDocPreview.test.ts` — file does not exist yet; needs creation alongside the component change.
- [ ] No fixture-level changes needed; existing `vitest-setup.ts` covers everything (no clipboard-specific setup required because the plugin is fully mocked).
- [ ] Framework install: NONE — vitest + @testing-library/svelte + jsdom already installed and proven by 20+ existing component tests.

**Recommended mock skeleton (verified against `CommitGraph.test.ts:56` and `ReviewPanel.test.ts:25`):**
```typescript
import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ReviewDocPreview from "./ReviewDocPreview.svelte";

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn().mockResolvedValue(undefined),
}));
vi.mock("../lib/toast.svelte.js", () => ({
  showToast: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
  vi.useFakeTimers();
});
afterEach(() => { vi.useRealTimers(); });
```

## Project Constraints (from CLAUDE.md)

| Directive | This phase's compliance plan |
|-----------|------------------------------|
| Never inline colors — always use CSS custom properties | All new `.copy-button` rules use `var(--color-*)` tokens (mirrored from `.back-button`). |
| Never fight layout with positioning hacks — use grid/flexbox | Header is already `display: flex`; `.preview-spacer { flex: 1 }` pushes the new button to the right naturally. No `position: absolute`, no negative margins. |
| All git operations through git2 (except editor escape) | N/A — no git operations in this phase. |
| Run `just check` before every commit and push | Phase plan must include `just check` as the verification gate. |

## Project Constraints (from user's global `~/.agents/rules/`)

| Directive | This phase's compliance plan |
|-----------|------------------------------|
| TDD (red -> green -> refactor) — tests before implementation | Plan tasks should write `ReviewDocPreview.test.ts` cases first, then add the button + handler, then refactor styling for parity. |
| No `as any` / `as unknown` casts | Use `e instanceof Error` narrowing in the catch block; never cast. |
| Never use `vi.fn()` for domain logic — use Fakes | Exception applies here: this is a **boundary mock** (the Tauri plugin and the toast facade are at the edge of the component), not domain logic. The existing tests (`CommitGraph.test.ts:56`, `ReviewPanel.test.ts:25`) already establish `vi.mock(...)` for these exact two boundaries; consistency wins. |
| Surgical execution — only touch what's directly relevant | Modify ONLY `ReviewDocPreview.svelte` and CREATE `ReviewDocPreview.test.ts`. No other files. |
| Don't defend against your own code | D-08 makes the empty-markdown case unreachable; no defensive `if (!markdown)` branch. |

## Sources

### Primary (HIGH confidence — verified on-disk)
- `package.json` — `@tauri-apps/plugin-clipboard-manager@^2.3.2`, `@lucide/svelte@^0.577.0`, `vitest@^4.1.0`, `@testing-library/svelte@5.3.1`, `jsdom@29.0.1`.
- `node_modules/@tauri-apps/plugin-clipboard-manager/package.json` — confirms installed version 2.3.2.
- `node_modules/@tauri-apps/plugin-clipboard-manager/dist-js/index.d.ts` — confirms `writeText(text: string, opts?: { label?: string }): Promise<void>`.
- `src-tauri/capabilities/default.json:14` — confirms `clipboard-manager:allow-write-text` already granted.
- `src/components/ReviewDocPreview.svelte` — host component, props, dock point, visual template all verified by direct file read.
- `src/components/ReviewPanel.svelte` — analog `try/catch` + `showToast` pattern at `onGenerateClick` (lines 278-287); analog button-with-icon pattern at `.generate-button` (lines 360-369, 742-760); `showToast` import (line 11).
- `src/lib/toast.svelte.ts` — `showToast(message: string, kind: ToastKind = "success", ms = 3000): void` signature directly read.
- `src/components/CommitGraph.test.ts:56-58` — exact pattern for mocking `@tauri-apps/plugin-clipboard-manager` in component tests.
- `src/components/ReviewPanel.test.ts:25-27` — exact pattern for mocking `../lib/toast.svelte.js` in component tests.
- `vite.config.ts` — vitest config (jsdom, setup file, `src/**/*.test.ts` pattern).
- `vitest-setup.ts` — global stubs (ResizeObserver, Element.prototype.animate).
- `.planning/config.json` — `nyquist_validation: true`, `tdd_mode: true`, `code_review_depth: "deep"`.

### Secondary (MEDIUM confidence)
- N/A — every claim above is grounded in a file on disk.

### Tertiary (LOW confidence — assumptions logged)
- A1 (Lucide exports `Clipboard` / `ClipboardCopy`) — based on standard Lucide naming convention; not directly grepped during research.
- A2 (Tauri plugin error `.message` populated) — based on Tauri plugin conventions; not exercised in existing fire-and-forget callsites.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every library/version verified on disk.
- Architecture: HIGH — all integration points exist; the patch is purely additive within one file.
- Pitfalls: HIGH — derived from existing code patterns (Pitfall 4 from import-style audit, Pitfall 5 from existing vitest setup, Pitfall 3 from TS strict default).
- Validation: HIGH — vitest infrastructure proven by 20+ existing component tests; mock pattern for the two boundaries is verbatim from existing tests.

**Research date:** 2026-05-26
**Valid until:** 2026-06-25 (30 days — stable Tauri 2.x line, no fast-moving deps in this phase's surface).

## RESEARCH COMPLETE
