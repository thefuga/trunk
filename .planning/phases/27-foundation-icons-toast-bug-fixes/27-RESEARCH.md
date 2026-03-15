# Phase 27: Foundation — Icons, Toast & Bug Fixes - Research

**Researched:** 2026-03-15
**Domain:** Svelte 5 UI — icon library, toast notifications, Rust/git2 dirty-state, CSS header layout
**Confidence:** HIGH

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| ICON-01 | App uses Lucide SVG icons replacing Unicode symbols across all components (Toolbar, FileRow, StagingPanel, CommitForm, BranchSidebar, TabBar) | `@lucide/svelte` package, icon mapping table in Architecture Patterns |
| TOAST-01 | App displays toast notifications for operation success and error feedback | Custom Svelte 5 toast store pattern (no external lib); singleton overlay component |
| FIX-01 | New untracked files included in dirty counts and trigger WIP row in graph | `get_dirty_counts` in `staging.rs` omits `WT_NEW` status flag — one-line Rust fix |
| FIX-02 | Last visible column header does not render a trailing resize divider | `CommitGraph.svelte` header always renders a `.col-resize-handle` div on every column; last-visible detection needed |
</phase_requirements>

---

## Summary

Phase 27 is the foundation phase of v0.6. It has four independent workstreams: (1) replacing all Unicode symbols with `@lucide/svelte` SVG icons across six components, (2) building a non-blocking toast notification system, (3) fixing a one-line Rust bug where `get_dirty_counts` misses untracked files, and (4) fixing a CSS artifact where the last visible commit-graph column header still renders a resize divider on its right edge.

All four items are self-contained and well-understood from reading the codebase. The Lucide library is not currently installed — `@lucide/svelte` (for Svelte 5) must be added as a dependency. No existing toast infrastructure exists; a minimal Svelte 5 `$state`-based store + overlay component is the correct approach given the project's no-external-library-for-simple-things pattern. Both bug fixes are trivial code changes (< 5 lines each).

**Primary recommendation:** Install `@lucide/svelte`, create a `Toast.svelte` overlay component backed by a `toast.svelte.ts` store, fix `get_dirty_counts` to include `WT_NEW`, and conditionally suppress the `.col-resize-handle` on the last visible column header.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `@lucide/svelte` | latest (0.487+) | SVG icon components for Svelte 5 | Official Lucide package for Svelte 5; tree-shakable, typed, uses `currentColor` |

> **Note:** `lucide-svelte` is the Svelte 4 package. **This project uses Svelte 5** (see `package.json`: `"svelte": "^5.0.0"`). The correct package is `@lucide/svelte`.

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Svelte 5 `$state` runes | built-in | Toast store reactive state | All new reactive state in this project |
| Tailwind CSS v4 | already installed | Positioning/animation classes for toast overlay | Already used extensively in the project |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `@lucide/svelte` | Custom inline SVGs | Custom SVGs are maintenance burden; Lucide is the ecosystem standard |
| Custom toast store | `svelte-sonner`, `svelte-hot-toast` | Third-party libs add bundle weight; custom is ~30 lines and fits the project pattern |
| CSS-only last-column fix | JS computed `isLast` prop | CSS `:last-child` doesn't work because columns are conditionally rendered with `{#if}`; JS is needed |

**Installation:**
```bash
npm install @lucide/svelte
```

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── components/
│   ├── Toast.svelte         # NEW: fixed overlay, renders active toasts
│   └── [existing files]     # icons injected inline via import
├── lib/
│   └── toast.svelte.ts      # NEW: $state store + show/dismiss helpers
└── App.svelte               # mounts <Toast /> once at top level
```

### Pattern 1: Lucide Icon Import (Svelte 5)
**What:** Import named icon components from `@lucide/svelte`; render inline with `size` and `color` props.
**When to use:** Everywhere a Unicode symbol or HTML entity currently appears.
**Example:**
```typescript
// Source: https://lucide.dev/guide/packages/lucide-svelte
<script lang="ts">
  import { Undo2, Redo2, ArrowDown, ArrowUp, GitBranch, Archive, PackageOpen } from '@lucide/svelte';
</script>

<button class="toolbar-btn" disabled={!canUndo} onclick={handleUndo}>
  <Undo2 size={14} />
  Undo
</button>
```

Props accepted: `size` (number, default 24), `color` (string, default `currentColor`), `strokeWidth` (number, default 2). All SVG attributes also accepted.

### Pattern 2: Toast Store (Svelte 5 `$state`)
**What:** A module-level `$state` array of toast objects; `showToast()` pushes, auto-dismiss timer pops.
**When to use:** After any `safeInvoke` that represents a user-facing operation (stash, checkout, branch create, etc.).
**Example:**
```typescript
// src/lib/toast.svelte.ts
export interface Toast {
  id: number;
  message: string;
  kind: 'success' | 'error';
}

let _toasts = $state<Toast[]>([]);
let _nextId = 0;

export const toasts = {
  get items() { return _toasts; }
};

export function showToast(message: string, kind: 'success' | 'error' = 'success', durationMs = 3000) {
  const id = _nextId++;
  _toasts = [..._toasts, { id, message, kind }];
  setTimeout(() => {
    _toasts = _toasts.filter(t => t.id !== id);
  }, durationMs);
}
```

```svelte
<!-- src/components/Toast.svelte -->
<script lang="ts">
  import { toasts } from '../lib/toast.svelte.js';
</script>

<div class="fixed bottom-4 right-4 flex flex-col gap-2 z-50 pointer-events-none">
  {#each toasts.items as toast (toast.id)}
    <div
      role="status"
      class="px-4 py-2 rounded-lg text-sm font-medium shadow-lg pointer-events-auto"
      style="background: {toast.kind === 'error' ? '#3d1c1c' : 'var(--color-surface)'}; 
             border: 1px solid {toast.kind === 'error' ? '#6b2a2a' : 'var(--color-border)'};
             color: {toast.kind === 'error' ? '#f87171' : 'var(--color-text)'};"
    >
      {toast.message}
    </div>
  {/each}
</div>
```

Mount once in `App.svelte`:
```svelte
<!-- App.svelte (inside the top-level div) -->
<Toast />
```

### Pattern 3: FIX-01 — Add `WT_NEW` to `get_dirty_counts`
**What:** The `get_dirty_counts` Tauri command in `src-tauri/src/commands/staging.rs` checks `Status::WT_MODIFIED | WT_DELETED | WT_RENAMED | WT_TYPECHANGE` but **omits `WT_NEW`** (untracked files). Adding it makes new files increment `unstaged`.
**When to use:** One-line fix in the `unstaged` accumulator block.

Current code (line 204-210 in `staging.rs`):
```rust
if s.intersects(
    Status::WT_MODIFIED
        | Status::WT_DELETED
        | Status::WT_RENAMED
        | Status::WT_TYPECHANGE,
) {
    unstaged += 1;
}
```

Fix — add `Status::WT_NEW`:
```rust
if s.intersects(
    Status::WT_NEW
        | Status::WT_MODIFIED
        | Status::WT_DELETED
        | Status::WT_RENAMED
        | Status::WT_TYPECHANGE,
) {
    unstaged += 1;
}
```

Also requires `StatusOptions` to have `include_untracked(true)` when creating the `statuses`. The current code passes `None` to `repo.statuses(None)` which uses default options — by default git2 does NOT include untracked files. Fix must also pass explicit `StatusOptions`:

```rust
let mut opts = StatusOptions::new();
opts.include_untracked(true)
    .include_ignored(false)
    .recurse_untracked_dirs(true);
let statuses = repo.statuses(Some(&mut opts)).map_err(TrunkError::from)?;
```

> **Note:** `get_status_inner` already does this correctly (line 44-47 in `staging.rs`). Only `get_dirty_counts` is broken.

### Pattern 4: FIX-02 — Last Visible Column Header Has No Trailing Divider
**What:** In `CommitGraph.svelte`, every column header div contains a `.col-resize-handle` div. When the last column is hidden via context menu, the new last visible column now shows a resize divider on its right edge (visually dangling).

**Root cause:** The header block uses `{#if columnVisibility.X}` to show/hide individual columns. Each shown column unconditionally renders `.col-resize-handle`. There is no concept of "last visible column."

**Fix approach:** Compute an array of currently visible column keys in order; compare each column to the last entry; only render the handle when it's not last.

```svelte
<!-- In CommitGraph.svelte script block -->
const ORDERED_COLUMNS = ['ref', 'graph', 'message', 'author', 'date', 'sha'] as const;

const visibleColumns = $derived(
  ORDERED_COLUMNS.filter(k => columnVisibility[k])
);
const lastVisibleColumn = $derived(visibleColumns[visibleColumns.length - 1]);
```

In the template, each column's resize handle is guarded:
```svelte
{#if columnVisibility.sha && 'sha' !== lastVisibleColumn}
  <div class="col-resize-handle" onmousedown={(e) => startColumnResize('sha', e, true)}></div>
{/if}
```

> Note: The `message` column has no dedicated resize slot — its handle targets `author` with `invert=true`. This must be preserved (it's the left edge of the message column, not right edge).

### Icon Mapping Table (Unicode → Lucide)

| Component | Current Symbol | HTML Entity | Lucide Icon | Notes |
|-----------|---------------|-------------|-------------|-------|
| `Toolbar` | ↵ Undo | `&#8617;` | `Undo2` | `Undo2` is the standard "undo arrow" |
| `Toolbar` | ↺ Redo | `&#8618;` | `Redo2` | |
| `Toolbar` | ↓ Pull | `&#8595;` | `ArrowDown` | |
| `Toolbar` | ↑ Push | `&#8593;` | `ArrowUp` | |
| `Toolbar` | ⎇ Branch | `&#9095;` | `GitBranch` | |
| `Toolbar` | 📦 Stash | `&#128230;` | `Archive` | |
| `Toolbar` | 📥 Pop | `&#128229;` | `PackageOpen` | or `ArchiveRestore` |
| `PullDropdown` | ▾ chevron | `&#9662;` | `ChevronDown` | |
| `FileRow` | + (New) | literal `+` | `FilePlus` | colored by status |
| `FileRow` | ✎ (Modified) | literal `✎` | `FilePen` | or `FileEdit` |
| `FileRow` | − (Deleted) | literal `−` | `FileMinus` | |
| `FileRow` | → (Renamed) | literal `→` | `FileSymlink` | |
| `FileRow` | ⇄ (Typechange) | literal `⇄` | `FileType2` | |
| `FileRow` | ! (Conflicted) | literal `!` | `FileWarning` | |
| `FileRow` action | + (stage btn) | literal `+` | `Plus` (small) | hover action button |
| `FileRow` action | − (unstage btn) | literal `−` | `Minus` (small) | hover action button |
| `StagingPanel` | ▼/▶ expand | literals | `ChevronDown`/`ChevronRight` | section toggles |
| `CommitForm` | (none to replace) | — | — | no Unicode in CommitForm |
| `BranchSection` | ▼/▶ expand | literals | `ChevronDown`/`ChevronRight` | section toggle |
| `BranchSection` | + (create btn) | literal `+` | `Plus` | create branch/stash |
| `BranchRow` | ↓ behind | `\u2193` | `ArrowDown` | upstream tracking |
| `BranchRow` | ↑ ahead | `\u2191` | `ArrowUp` | upstream tracking |
| `TabBar` | × close | literal `×` | `X` | close repo tab |

### Anti-Patterns to Avoid
- **Import `* from '@lucide/svelte'`:** Imports the entire icon set, massively bloating the bundle. Always use named imports.
- **Using `lucide-svelte` (Svelte 4 package):** This project is Svelte 5. Use `@lucide/svelte`.
- **Toast with HTML `alert()` or Tauri `dialog` plugin:** These are blocking. Toast must be non-blocking.
- **Placing toast logic inside individual components:** Toast store must be a singleton; components call `showToast()`, they don't manage their own notification state.
- **Post-processing graph layers:** Per `.claude/rules/commit-graph.md`, never post-process a layer's output to fix something a prior layer should handle. The FIX-02 is purely a template/CSS change — it does not touch graph data.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SVG icon paths | Custom `<svg>` paths per icon | `@lucide/svelte` named imports | 1000+ icons, consistent stroke, tree-shakable |
| Unicode/emoji icons | HTML entities like `&#8617;` | Lucide SVG components | Inconsistent rendering across platforms/fonts |

**Key insight:** The project already uses emoji/HTML entities due to historical lack of an icon library. The entire `STATUS_ICONS` and `fileStatusIcon` pattern across FileRow, CommitDetail, etc. can be replaced with a tiny Lucide import map.

---

## Common Pitfalls

### Pitfall 1: Wrong Lucide Package for Svelte 5
**What goes wrong:** Installing `lucide-svelte` instead of `@lucide/svelte` — gets Svelte 4 components that don't compile under Svelte 5's runes mode.
**Why it happens:** Lucide has two Svelte packages; documentation defaults show `lucide-svelte` in many older guides.
**How to avoid:** Always install `@lucide/svelte` for Svelte 5 projects. The official docs at lucide.dev/guide/packages/lucide-svelte are explicit about this.
**Warning signs:** `SvelteComponent` type errors, `export let` instead of `$props()`.

### Pitfall 2: `get_dirty_counts` Still Misses Untracked Files After Adding `WT_NEW`
**What goes wrong:** Adding `WT_NEW` to the status check in `get_dirty_counts` but forgetting to also change `repo.statuses(None)` to use `StatusOptions` with `include_untracked(true)`. The `WT_NEW` flag is only set when git2 is told to scan untracked files.
**Why it happens:** git2's default `statuses(None)` excludes untracked files entirely — no `WT_NEW` entries will ever appear without the option.
**How to avoid:** Mirror the `get_status_inner` approach: create `StatusOptions::new()` with `.include_untracked(true).recurse_untracked_dirs(true)` and pass it to `repo.statuses(Some(&mut opts))`.
**Warning signs:** Tests pass when checking tracked-file modifications but fail when checking new untracked files.

### Pitfall 3: Toast Store Using `let` Instead of `$state`
**What goes wrong:** A plain `let toasts = []` in a `.svelte.ts` file — mutations won't trigger Svelte 5 reactivity, so the `Toast.svelte` overlay never updates.
**Why it happens:** Svelte 5 runes are required in `.svelte.ts` files for reactivity; regular JS variables don't have reactive tracking.
**How to avoid:** Use `$state` for the toasts array in `toast.svelte.ts`. The file extension must be `.svelte.ts` (not `.ts`) for runes to be enabled.
**Warning signs:** `showToast()` doesn't cause re-render; toasts never appear.

### Pitfall 4: FIX-02 Breaking the Message Column's Inverted Resize Handle
**What goes wrong:** When suppressing resize handles for the last visible column, incorrectly suppressing the `message` column's handle (which targets `author` with `invert=true`) — this removes the resize affordance for the entire right side of the layout.
**Why it happens:** The `message` column's resize handle is logically the *left* edge of the author column, not the *right* edge of message. It is attached to the message `div` but controls `author` width. The last-column logic only needs to suppress trailing handles, not these inverted ones.
**How to avoid:** The `message` column handle uses `invert=true`; it should never be suppressed since it's always the leftmost resize boundary of the right-side columns.

### Pitfall 5: Lucide Icon Size Mismatch in Toolbar Buttons
**What goes wrong:** Default Lucide `size={24}` makes icons too large for the 26px-tall toolbar buttons.
**Why it happens:** Lucide defaults to 24px, but the toolbar uses 12px font and 26px button height.
**How to avoid:** Use `size={14}` for toolbar buttons and `size={12}` for inline status icons (FileRow, BranchRow).

---

## Code Examples

Verified patterns from official sources:

### Lucide Svelte 5 Named Import
```svelte
<!-- Source: https://lucide.dev/guide/packages/lucide-svelte -->
<script lang="ts">
  import { ArrowDown, GitBranch, Archive } from '@lucide/svelte';
</script>

<button class="toolbar-btn" onclick={handlePull}>
  <ArrowDown size={14} />
  Pull
</button>
```

### Lucide with currentColor (inherits button color)
```svelte
<!-- color defaults to "currentColor" — inherits from parent CSS color -->
<GitBranch size={14} />
```

### Toast store (.svelte.ts runes file)
```typescript
// src/lib/toast.svelte.ts
export type ToastKind = 'success' | 'error';

export interface Toast {
  id: number;
  message: string;
  kind: ToastKind;
}

let _toasts = $state<Toast[]>([]);
let _nextId = 0;

export const toasts = {
  get items(): Toast[] { return _toasts; }
};

export function showToast(message: string, kind: ToastKind = 'success', ms = 3000): void {
  const id = _nextId++;
  _toasts = [..._toasts, { id, message, kind }];
  setTimeout(() => dismiss(id), ms);
}

function dismiss(id: number): void {
  _toasts = _toasts.filter(t => t.id !== id);
}
```

### git2 StatusOptions with untracked files (Rust)
```rust
// Pattern from get_status_inner (already correct in codebase):
let mut opts = StatusOptions::new();
opts.include_untracked(true)
    .include_ignored(false)
    .recurse_untracked_dirs(true);
let statuses = repo.statuses(Some(&mut opts))?;
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `lucide-svelte` (Svelte 4) | `@lucide/svelte` (Svelte 5) | 2024 (Svelte 5 release) | Must use `@lucide/svelte` for Svelte 5 runes compatibility |
| Plain `let` reactive stores | Svelte 5 `$state` runes in `.svelte.ts` | Svelte 5 | Module-level reactive state requires runes syntax |

**Deprecated/outdated:**
- `lucide-svelte`: Svelte 4 package; do not use with Svelte 5

---

## Open Questions

1. **Animation for toast enter/leave**
   - What we know: Tailwind v4 is available; Svelte has `transition:` directives
   - What's unclear: Whether CSS-only fade is sufficient or if Svelte `fly`/`fade` transitions are desired
   - Recommendation: Use Svelte's built-in `transition:fly` for slide-in; it's in the standard lib, zero cost

2. **Which operations should trigger toasts?**
   - What we know: TOAST-01 says "operation success and error feedback" for operations like stash create, branch checkout
   - What's unclear: Exhaustive list is not specified
   - Recommendation: Wire toasts into Toolbar operations (stash/pop/push/pull/branch create) and BranchSidebar checkout/stash as the primary locations for v0.6

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.0 |
| Config file | `vite.config.ts` (implicit) |
| Quick run command | `npx vitest run` |
| Full suite command | `npx vitest run` |
| Rust tests | `cd src-tauri && cargo test --lib` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ICON-01 | Icons render without Unicode text | visual/manual | `npx vitest run` (no icon unit tests needed) | ✅ manual only |
| TOAST-01 | `showToast` adds to store; auto-dismiss removes after timeout | unit | `npx vitest run src/lib/toast.svelte.test.ts` | ❌ Wave 0 |
| FIX-01 | `get_dirty_counts` counts untracked files | Rust unit test | `cd src-tauri && cargo test --lib staging` | ❌ Wave 0 |
| FIX-02 | Last visible column header has no resize handle | visual/manual | manual verification | ✅ manual only |

### Sampling Rate
- **Per task commit:** `npx vitest run` (TypeScript) + `cd src-tauri && cargo test --lib` (Rust)
- **Per wave merge:** Full suite green
- **Phase gate:** Both suites green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/lib/toast.svelte.test.ts` — covers TOAST-01 (store push/pop/auto-dismiss)
- [ ] `src-tauri/src/commands/staging.rs` new test: `get_dirty_counts_includes_untracked` — covers FIX-01

*(Rust test for FIX-01 should follow the existing pattern in `staging.rs` `#[cfg(test)]` block using `make_test_repo`.)*

---

## Sources

### Primary (HIGH confidence)
- `lucide.dev/guide/packages/lucide-svelte` — confirmed `@lucide/svelte` is the Svelte 5 package; API verified
- `/Users/joaofnds/code/trunk/src-tauri/src/commands/staging.rs` — source of FIX-01 bug (line 188-189, 204-210)
- `/Users/joaofnds/code/trunk/src/components/CommitGraph.svelte` — source of FIX-02 bug (lines 459-498)
- `/Users/joaofnds/code/trunk/package.json` — confirms Svelte 5, Vitest 4, Tailwind 4; no `@lucide/svelte` installed
- Svelte 5 runes docs — `$state` in `.svelte.ts` files verified

### Secondary (MEDIUM confidence)
- Lucide icon name choices (Undo2, Redo2, Archive, etc.) — browsed lucide.dev icon search; names verified to exist

### Tertiary (LOW confidence)
- Toast animation approach — recommended Svelte `transition:fly` based on Svelte docs knowledge; not formally verified against this project's constraints

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — `@lucide/svelte` confirmed from official docs; package.json confirms Svelte 5
- Architecture: HIGH — bugs found directly in source; patterns derived from existing codebase conventions
- Pitfalls: HIGH for Rust/git2 (verified by reading code); MEDIUM for toast reactivity (standard Svelte 5 pattern)

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable ecosystem; Lucide API unlikely to change)
