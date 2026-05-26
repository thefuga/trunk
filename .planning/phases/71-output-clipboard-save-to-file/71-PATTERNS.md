# Phase 71: Output (Clipboard) — Pattern Map

**Mapped:** 2026-05-26
**Files analyzed:** 2 (1 modified, 1 created)
**Analogs found:** 2 / 2 (both exact matches in the same component family)

## File Classification

| File | New / Modified | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|---|
| `src/components/ReviewDocPreview.svelte` | Modified | Svelte 5 component (presentation + local interaction) | request-response (frontend → Tauri plugin → OS clipboard); transient local UI state | `src/components/ReviewPanel.svelte` (sibling component, same component family, established `try/catch` + `showToast` + Lucide-icon button pattern) | exact (role + data flow + colocation) |
| `src/components/ReviewDocPreview.test.ts` | Created | Component-mount unit test (vitest + @testing-library/svelte) | mounts component, mocks Tauri plugin + toast facade, drives DOM events, asserts mock calls + DOM | `src/components/ReviewPanel.test.ts` (sibling test file, mocks `../lib/toast.svelte.js`) + `src/components/CommitGraph.test.ts` (mocks `@tauri-apps/plugin-clipboard-manager`) | exact (combined: clipboard mock from CommitGraph, toast mock + flush/tick discipline from ReviewPanel) |

Notes:
- The "static `writeText` import" analog is `src/components/CommitDetail.svelte:3` — that file is NOT the closest *behavioral* analog (it's fire-and-forget); it's cited solely for the **import form**.
- The "dynamic `import(...)` form" used in `App.svelte:133` is the form to **avoid** per RESEARCH Pitfall 4.

## Pattern Assignments

### `src/components/ReviewDocPreview.svelte` (Svelte 5 component, request-response + local state)

The host file already exists with the dock point (`.preview-spacer`) and the visual template (`.back-button`). The modification is purely additive: new imports, new state, new handler, new button markup, new style rules.

#### Imports pattern

**Static `writeText` import — copy from `src/components/CommitDetail.svelte:3`:**
```typescript
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
```
The dynamic `await import(...)` form at `src/App.svelte:133` is intentionally NOT used (Pitfall 4).

**Lucide icon import + toast import — copy from `src/components/ReviewPanel.svelte:8` and `:12`:**
```typescript
import { FileText, MessageSquarePlus } from "@lucide/svelte";   // analog
import { showToast } from "../lib/toast.svelte.js";             // analog
```
For Phase 71 the icon is `Clipboard` (or `ClipboardCopy` — Discretion):
```typescript
import { Clipboard } from "@lucide/svelte";
import { showToast } from "../lib/toast.svelte.js";
```

#### Svelte 5 `$state<boolean>` rune for transient UI state

**Analog — `src/components/ReviewPanel.svelte` (line 44 area), `addNoteForCommit` / `editingCommentId` are `$state` runes for transient UI mode flags; the closest boolean-shape sibling is the `draftValid` derived (line 48). Plain boolean `$state` pattern verified elsewhere in same file.**

Phase 71 usage (copy this shape into `ReviewDocPreview.svelte` `<script>` block):
```typescript
let copied = $state(false);
let copyTimer: ReturnType<typeof setTimeout> | null = null;
```
`copyTimer` is plain (not `$state`) — it is not reactive, only a handle to clear (RESEARCH Pitfall 2).

#### Core pattern — `try`/`catch` + `showToast` on failure

**Analog — `src/components/ReviewPanel.svelte:278-287` (`onGenerateClick`):**
```typescript
async function onGenerateClick() {
	try {
		await session.generate(repoPath);
	} catch (e) {
		showToast(
			(e as TrunkError).message ?? "Failed to generate review doc",
			"error",
		);
	}
}
```

**Adaptation for Phase 71** (note the divergence — narrow with `instanceof Error` rather than the `TrunkError` cast, because Tauri plugin errors don't conform to `TrunkError`):
```typescript
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
Three intentional divergences from the analog:
1. **`instanceof Error` narrowing, not `as TrunkError`** — coding_style_typescript.md §1 bans casts; the plugin's error type is unknown so the only sound narrowing is `instanceof`. Matches RESEARCH Pitfall 1 + 3.
2. **`clearTimeout` before scheduling a new timer** — RESEARCH Pitfall 2 (rapid re-click).
3. **Template literal for the toast message** — matches the RESEARCH-mandated `"Failed to copy: <error message>"` format.

#### Button markup pattern — icon + label in flex layout

**Analog — `src/components/ReviewPanel.svelte:360-368` (`.generate-button`):**
```svelte
<button
	type="button"
	class="generate-button flex items-center"
	onclick={onGenerateClick}
	disabled={!hasAnyComment}
	title={hasAnyComment ? "" : "Add at least one comment to generate"}
>
	<FileText size={14} />
	<span>Generate</span>
</button>
```

**Adaptation for Phase 71** (no `disabled`, no `title` — D-04 keeps button always clickable; D-08 means there's no empty-doc case to gate against; the `{#if copied}` branch is the new affordance per D-03):
```svelte
<button type="button" class="copy-button" onclick={onCopy}>
	{#if copied}
		<span aria-hidden="true">✓</span>
		<span>Copied</span>
	{:else}
		<Clipboard size={14} />
		<span>Copy</span>
	{/if}
</button>
```
The button is placed **after** `<span class="preview-spacer"></span>` (line 26 of the existing file) so flexbox docks it to the right naturally (RESEARCH Pattern 3, CLAUDE.md "Never fight layout with positioning hacks").

#### Style block — `.copy-button` rule

**Analog — `src/components/ReviewDocPreview.svelte:50-64` (`.back-button`) for color/border/hover; `src/components/ReviewPanel.svelte:742-760` (`.generate-button`) for `gap: 4px` icon-row layout.**

Copy `.back-button` lines 50-64 verbatim, rename to `.copy-button`, and prepend the three flex helpers:
```css
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
All color values come from CSS custom properties — CLAUDE.md rule "Never inline colors".

---

### `src/components/ReviewDocPreview.test.ts` (component-mount unit test)

A new file. Combines two existing patterns: the `@tauri-apps/plugin-clipboard-manager` mock from `CommitGraph.test.ts` and the `../lib/toast.svelte.js` mock + render/flush discipline from `ReviewPanel.test.ts`.

#### Imports + boundary mocks

**Analog — `src/components/ReviewPanel.test.ts:1-3, 11, 24-26` (toast mock + render imports):**
```typescript
import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ReviewPanel from "./ReviewPanel.svelte";

vi.mock("../lib/toast.svelte.js", () => ({
	showToast: vi.fn(),
}));
```

**Analog — `src/components/CommitGraph.test.ts:56-58` (clipboard plugin mock):**
```typescript
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
	writeText: vi.fn().mockResolvedValue(undefined),
}));
```

**Combined skeleton for `ReviewDocPreview.test.ts`:**
```typescript
import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { showToast } from "../lib/toast.svelte.js";
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

afterEach(() => {
	vi.useRealTimers();
});
```
Note: existing tests do NOT use fake timers — Phase 71 is the first. RESEARCH Pitfall 5 already covers this. Use `vi.useFakeTimers()` + `vi.advanceTimersByTime(1500)` for the revert-after-timeout test.

#### Render + flush discipline

**Analog — `src/components/ReviewPanel.test.ts:115-118` (`flush` helper) and `:143-150` (render shape):**
```typescript
async function flush() {
	await new Promise((r) => setTimeout(r, 0));
	await tick();
}

// later, inside a test:
render(ReviewPanel, {
	props: {
		repoPath: "/repo",
		session: createReviewSession(),
		onJump: vi.fn(),
		onJumpToCommit: vi.fn(),
	},
});
await flush();
```

**Adaptation for `ReviewDocPreview` (much simpler props — just `markdown` and `onBack`):**
```typescript
function renderPreview(markdown: string, onBack = vi.fn()) {
	return render(ReviewDocPreview, { props: { markdown, onBack } });
}
```
**Caveat on fake timers + `flush`:** the existing `flush` helper uses `new Promise((r) => setTimeout(r, 0))`. Under `vi.useFakeTimers()`, that setTimeout will NOT advance unless you advance the timer. Use `await Promise.resolve()` (or `await tick()` alone) in fake-timer contexts to flush microtasks without waiting on the timer queue. The plan should call this out explicitly.

#### Assertion patterns

**Analog — `src/components/ReviewPanel.test.ts:263-267` (assert mock call args):**
```typescript
expect(calledCommands()).toContain("add_commit_comment");
const args = callArgs("add_commit_comment");
expect(args?.commitOid).toBe(COMMIT_A);
expect(args?.text).toBe("a fresh note");
```

**Adaptation for Phase 71** (assert `writeText` mock was called with the markdown prop verbatim):
```typescript
expect(vi.mocked(writeText)).toHaveBeenCalledWith("the test markdown");
expect(vi.mocked(writeText)).toHaveBeenCalledTimes(1);
```
And for the error path:
```typescript
expect(vi.mocked(showToast)).toHaveBeenCalledWith(
	"Failed to copy: plugin disabled",
	"error",
);
```

#### Test list (matches RESEARCH §"Unit-testable behaviors")

The seven test cases from RESEARCH.md §Validation Architecture map directly to vitest `it` blocks. Group under a single `describe("ReviewDocPreview", () => { ... })` per the existing pattern (`ReviewPanel.test.ts:133`).

---

## Shared Patterns

### TypeScript strict catch narrowing
**Source:** Coding-style rule (`coding_style_typescript.md §1`) — no `as` casts; use `instanceof Error`.
**Apply to:** Every `catch` block in this phase.
```typescript
} catch (e) {
	const msg = e instanceof Error ? e.message : String(e);
	showToast(`Failed to copy: ${msg}`, "error");
}
```
This is a **divergence** from the existing `(e as TrunkError).message` pattern in `ReviewPanel.svelte:283` — that pattern is acceptable there because the catch sits behind a domain-typed IPC; here `writeText` rejects with an opaque plugin error.

### CSS theme tokens only
**Source:** `CLAUDE.md` "Never inline colors — always use CSS custom properties from the theme."
**Apply to:** The entire `.copy-button` rule. Tokens used: `--color-text-muted`, `--color-text`, `--color-border`, `--color-hover` — all already present and used by the analog `.back-button` rule (lines 50-64).

### Flex layout, no positioning hacks
**Source:** `CLAUDE.md` "Never fight layout with positioning hacks — use grid/flexbox so elements flow naturally."
**Apply to:** Header dock. `.preview-header` is already `display: flex` (line 40-48); `.preview-spacer { flex: 1 }` (line 49) is already in place. Placing the new button after the spacer is the entire docking mechanism — no `position: absolute`, no `margin-left: auto`.

### Boundary mocks via `vi.mock` (test files only)
**Source:** `ReviewPanel.test.ts:24-26` (toast) + `CommitGraph.test.ts:56-58` (clipboard).
**Apply to:** `ReviewDocPreview.test.ts`.
This is a deliberate exception to the user rule "no spy/mock frameworks" — the rule applies to **domain logic dependencies**. The Tauri plugin and the toast facade are **boundary** dependencies; the project's established pattern is to `vi.mock` them. RESEARCH Validation §"Project Constraints from `~/.agents/rules/`" already documents this carve-out.

---

## No Analog Found

None. Every required pattern has a verified analog in the existing codebase.

---

## Metadata

**Analog search scope:**
- `src/components/` — every existing `.svelte` and `.test.ts` file in the same directory.
- Specifically read for excerpts:
  - `src/components/ReviewDocPreview.svelte` (1-81 — host file)
  - `src/components/ReviewPanel.svelte` (1-60, 270-300, 360-368, 735-761)
  - `src/components/CommitDetail.svelte` (1-15 — for the static import form)
  - `src/components/ReviewPanel.test.ts` (full — for toast mock, flush helper, render shape, assertion style)
  - `src/components/CommitGraph.test.ts` (1-80 — for clipboard plugin mock)

**Files scanned:** ~5 (highly focused — the phase is a single-component change with two well-known analogs).

**Pattern extraction date:** 2026-05-26
