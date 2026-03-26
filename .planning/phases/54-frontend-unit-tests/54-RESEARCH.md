# Phase 54: Frontend Unit Tests - Research

**Researched:** 2026-03-26
**Domain:** Svelte 5 component testing, TypeScript utility unit testing, vitest + @testing-library/svelte
**Confidence:** HIGH

## Summary

Phase 54 covers two distinct workstreams: (1) reviewing and expanding the existing 14 TypeScript utility test files (~2,582 lines, 170 passing tests) for edge case coverage gaps, and (2) adding component tests for all 26 Svelte components using `@testing-library/svelte`. The existing test infrastructure (vitest 4.1.1, Svelte 5.53.6) is solid but currently configured with `environment: "node"` and lacks DOM testing support.

The primary configuration change is adding `@testing-library/svelte` (5.3.1), `@testing-library/jest-dom` (6.9.1), `jsdom` (29.0.1), and the `svelteTesting()` vite plugin. The `svelteTesting()` plugin from `@testing-library/svelte/vite` is critical -- it resolves the browser condition so Svelte components compile to client-side (not SSR) code in the jsdom environment. Without it, `mount()` calls fail with "not available on the server."

**Primary recommendation:** Use jsdom (not happy-dom) as the test environment, add the `svelteTesting()` vite plugin, create a shared Tauri invoke mock in `src/__tests__/helpers/tauri-mock.ts`, extract factory functions to `src/__tests__/helpers/factories.ts`, and write component tests using `render()` + `screen` queries from `@testing-library/svelte`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Use `@testing-library/svelte` for component tests. Renders components in jsdom, queries by accessible roles/text. Svelte 5 runes support required.
- **D-02:** All 20+ Svelte components get tests -- not just logic-heavy ones.
- **D-03:** Even simple components (RefPill, CommitRow, BranchRow, FileRow, OperationBanner) get render + props + events testing depth -- verify rendering, prop display, click handlers, event emissions, and conditional rendering.
- **D-04:** Mock `invoke` at module level via `vi.mock('$lib/invoke')` so all component tests get a working Tauri mock. Test the mock setup once, components just work.
- **D-05:** Skip testing `types.ts` (pure type definitions, no runtime logic) and `tab-types.ts` (already covered by `store.test.ts`).
- **D-06:** Review and expand all existing utility tests (10 modules, ~2,357 lines) for missing edge cases. Not just adding new tests -- also auditing existing coverage.
- **D-07:** Extract factory functions (`makeCommit`, `makeEdge`, `makeFile`, etc.) to `src/__tests__/helpers/`. Reduces duplication across test files and makes factories available to component tests.
- **D-08:** Shared Tauri invoke mock lives in `src/__tests__/helpers/tauri-mock.ts`. Component tests import it for consistent mocking.
- **D-09:** Existing store tests (toast, undo-redo, remote-state) remain as pure logic tests. Component tests will exercise the rendering side. No duplicate rendering tests for stores.

### Claude's Discretion
- Exact jsdom/happy-dom environment choice for vitest
- Which edge cases to add when reviewing existing utility tests
- Component test file naming convention (collocated vs `__tests__/` directory)
- How to structure @testing-library/svelte imports and render patterns

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UNIT-02 | All TypeScript utilities and state management modules have unit tests | Existing 14 test files cover 10 modules; need edge case audit per D-06. Utility modules without tests: `invoke.ts`, `store.ts` (has tab-type tests but not full store coverage). `types.ts` and `tab-types.ts` excluded per D-05. |
| UNIT-03 | All Svelte components have unit tests for behavior and state transitions | 26 components need tests via @testing-library/svelte. Configuration changes (jsdom env, svelteTesting plugin, jest-dom matchers) enable component rendering in vitest. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- `bun run test` runs `vitest run` -- all tests must pass via this command
- `bun run check` runs `svelte-check` -- test files must be type-safe
- Never inline colors -- always use CSS custom properties from the theme
- `$lib` maps to `src/lib` (path alias)
- Frontend uses Svelte 5 runes: `$state`, `$derived`, `$effect`, `$props`
- Frontend-to-backend communication via `invoke("command_name", args)` from `@tauri-apps/api/core`

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| vitest | 4.1.1 (installed) | Test runner | Already in project, built into vite config |
| @testing-library/svelte | 5.3.1 | Component testing | Official testing-library for Svelte; supports Svelte 5 runes via reactive $state props |
| @testing-library/jest-dom | 6.9.1 | DOM assertion matchers | Provides `toBeInTheDocument()`, `toHaveTextContent()`, etc. |
| jsdom | 29.0.1 | DOM emulation | Stable, well-tested; Svelte official docs recommend jsdom for component tests |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @testing-library/user-event | 14.6.1 | Advanced user interaction simulation | For complex interactions (typing, tabbing, focus) beyond simple click/fireEvent |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| jsdom | happy-dom (20.8.8) | happy-dom is faster but less complete DOM emulation; jsdom is safer for components using transitions, focus management, scroll APIs |
| @testing-library/svelte | vitest-browser-svelte | Requires real browser process; heavier setup; better for runes reactivity testing but overkill for this phase |

**Installation:**
```bash
bun add -d @testing-library/svelte@5.3.1 @testing-library/jest-dom@6.9.1 jsdom@29.0.1
```

## Architecture Patterns

### Recommended Project Structure
```
src/
  __tests__/
    helpers/
      factories.ts          # makeCommit, makeEdge, makeFile, etc.
      tauri-mock.ts          # vi.mock for @tauri-apps/api/core invoke
  components/
    RefPill.svelte
    RefPill.test.ts          # collocated component test
    CommitRow.svelte
    CommitRow.test.ts
    ...
  lib/
    active-lanes.ts
    active-lanes.test.ts     # existing, audit for gaps
    ...
```

### Pattern 1: Vitest Configuration for Component Tests
**What:** Update `vite.config.ts` to support both utility tests (node env) and component tests (jsdom env)
**When to use:** Required for all component tests

The recommended approach is to set the global environment to `jsdom` since all existing utility tests are pure functions that work fine in jsdom. The `svelteTesting()` plugin must be added to resolve browser conditions correctly.

```typescript
// vite.config.ts
/// <reference types="vitest/config" />
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte(), svelteTesting(), tailwindcss()],
  clearScreen: false,
  server: { /* ...existing... */ },
  test: {
    include: ["src/**/*.test.ts"],
    environment: "jsdom",
    setupFiles: ["./vitest-setup.ts"],
  },
});
```

```typescript
// vitest-setup.ts
import "@testing-library/jest-dom/vitest";
```

**Source:** [Official @testing-library/svelte setup docs](https://testing-library.com/docs/svelte-testing-library/setup/)

### Pattern 2: Shared Tauri Invoke Mock
**What:** Module-level mock for `@tauri-apps/api/core` that all component tests share
**When to use:** Any component that calls `safeInvoke` or imports from `$lib/invoke`

```typescript
// src/__tests__/helpers/tauri-mock.ts
import { vi } from "vitest";

// Mock the Tauri core invoke function
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

// Mock Tauri plugin modules that components import
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  ask: vi.fn().mockResolvedValue(false),
}));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-store", () => {
  const store = new Map<string, unknown>();
  return {
    LazyStore: vi.fn().mockImplementation(() => ({
      get: vi.fn((key: string) => Promise.resolve(store.get(key) ?? null)),
      set: vi.fn((key: string, value: unknown) => { store.set(key, value); return Promise.resolve(); }),
      save: vi.fn().mockResolvedValue(undefined),
    })),
  };
});

vi.mock("@tauri-apps/api/path", () => ({
  homeDir: vi.fn().mockResolvedValue("/Users/test"),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@tauri-apps/plugin-window-state", () => ({}));
```

### Pattern 3: Component Test Pattern
**What:** Standard render + query + assert pattern for Svelte 5 components
**When to use:** All component tests

```typescript
// src/components/RefPill.test.ts
import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import RefPill from "./RefPill.svelte";

// Import shared mock (applies vi.mock calls)
import "../__tests__/helpers/tauri-mock";

describe("RefPill", () => {
  const refs = [
    {
      name: "refs/heads/main",
      short_name: "main",
      ref_type: "LocalBranch" as const,
      is_head: true,
      color_index: 0,
    },
  ];

  it("renders the primary ref short name", () => {
    render(RefPill, { props: { refs } });
    expect(screen.getByText("main")).toBeInTheDocument();
  });

  it("applies bold styling to HEAD ref", () => {
    render(RefPill, { props: { refs } });
    const pill = screen.getByText("main");
    expect(pill.className).toContain("font-bold");
  });
});
```

### Pattern 4: Factory Functions
**What:** Shared test data builders extracted from existing test files
**When to use:** Any test that needs GraphCommit, GraphEdge, FileStatus, etc.

```typescript
// src/__tests__/helpers/factories.ts
import type { FileStatus, GraphCommit, GraphEdge, RefLabel } from "$lib/types";

export function makeCommit(
  overrides: Partial<GraphCommit> & { oid: string },
): GraphCommit {
  return {
    oid: overrides.oid,
    short_oid: overrides.oid.slice(0, 7),
    summary: "test commit",
    body: null,
    author_name: "Test",
    author_email: "test@test.com",
    author_timestamp: 0,
    parent_oids: overrides.parent_oids ?? [],
    column: overrides.column ?? 0,
    color_index: overrides.color_index ?? 0,
    edges: overrides.edges ?? [],
    refs: overrides.refs ?? [],
    is_head: overrides.is_head ?? false,
    is_merge: overrides.is_merge ?? false,
    is_branch_tip: overrides.is_branch_tip ?? false,
    is_stash: overrides.is_stash ?? false,
  };
}

export function makeEdge(
  overrides: Partial<GraphEdge> & { edge_type: GraphEdge["edge_type"] },
): GraphEdge {
  return {
    from_column: overrides.from_column ?? 0,
    to_column: overrides.to_column ?? 0,
    edge_type: overrides.edge_type,
    color_index: overrides.color_index ?? 0,
    dashed: overrides.dashed ?? false,
  };
}

export function makeFile(
  path: string,
  status: FileStatus["status"] = "Modified",
): FileStatus {
  return { path, status, is_binary: false };
}

export function makeRef(
  overrides: Partial<RefLabel> & { short_name: string },
): RefLabel {
  return {
    name: overrides.name ?? `refs/heads/${overrides.short_name}`,
    short_name: overrides.short_name,
    ref_type: overrides.ref_type ?? "LocalBranch",
    is_head: overrides.is_head ?? false,
    color_index: overrides.color_index ?? 0,
  };
}
```

### Pattern 5: Testing Components with Svelte 5 Runes ($state, $derived)
**What:** How to handle reactive state in component tests
**When to use:** Components that use $state internally or display derived state

Key insight: `@testing-library/svelte` v5.3.x uses `$state` internally to manage props. When you call `rerender()` with new props, the library uses `Object.assign` on the reactive props object, triggering Svelte's reactivity system. After rerender, always `await` the returned promise (it calls `tick()` internally).

```typescript
import { render, screen } from "@testing-library/svelte";
import { fireEvent } from "@testing-library/svelte";

it("updates when props change", async () => {
  const { rerender } = render(MyComponent, { props: { count: 0 } });
  expect(screen.getByText("0")).toBeInTheDocument();

  await rerender({ count: 5 });
  expect(screen.getByText("5")).toBeInTheDocument();
});

it("handles click events", async () => {
  const onclick = vi.fn();
  render(MyComponent, { props: { onclick } });

  await fireEvent.click(screen.getByRole("button"));
  expect(onclick).toHaveBeenCalledOnce();
});
```

### Anti-Patterns to Avoid
- **Testing implementation details:** Do not query by CSS class names or internal state. Query by role, text, or label (accessible queries). Use `screen.getByRole("button")`, not `container.querySelector('.my-btn')`.
- **Not awaiting rerender:** Always `await rerender()` -- without it, the DOM may not reflect updated props.
- **Mocking invoke per-test:** Use the shared mock module; per-test mocking leads to inconsistency and forgotten mock resets.
- **Testing CSS values in jsdom:** jsdom does not compute CSS -- do not assert `getComputedStyle()`. Test conditional classes and style attributes instead.
- **Importing Tauri modules directly in test files:** Always mock Tauri modules. They rely on the Tauri runtime IPC bridge which does not exist in jsdom.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| DOM environment | Custom DOM stubs | jsdom 29 | Thousands of edge cases in DOM spec compliance |
| Component rendering in tests | Manual `mount()` + DOM queries | `@testing-library/svelte` render + screen | Handles Svelte 5 lifecycle, cleanup, reactivity, tick flushing |
| DOM assertions | Manual `textContent` checks | `@testing-library/jest-dom` matchers | `toBeInTheDocument()`, `toHaveTextContent()`, `toBeVisible()` etc. |
| User interaction simulation | Manual `dispatchEvent` | `fireEvent` from @testing-library/svelte | Correct event bubbling, default prevention, focus management |
| Tauri IPC mock | Per-test mock objects | Shared `vi.mock` module | Single source of truth; all components get consistent mock behavior |

**Key insight:** The Svelte 5 runes system (`$state`, `$derived`, `$effect`) adds complexity to test setup. The `svelteTesting()` vite plugin and `@testing-library/svelte` abstract this away. Without the vite plugin, Svelte files compile to SSR mode in jsdom, causing `mount()` to fail.

## Common Pitfalls

### Pitfall 1: Svelte Components Compile to SSR in Tests
**What goes wrong:** Tests fail with "mount(...) is not available on the server"
**Why it happens:** Without the `svelteTesting()` vite plugin, vitest resolves Svelte imports using SSR entry points even in jsdom environment
**How to avoid:** Add `svelteTesting()` from `@testing-library/svelte/vite` to the plugins array in `vite.config.ts`
**Warning signs:** Error message mentions "server" or "SSR" in component test output

### Pitfall 2: Tauri Plugin Imports Crash Tests
**What goes wrong:** Tests crash immediately with "Failed to import module" or "window.__TAURI_INTERNALS__ is not defined"
**Why it happens:** Tauri API modules (`@tauri-apps/api/core`, `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-store`, etc.) check for the Tauri IPC bridge at import time
**How to avoid:** Mock ALL Tauri modules at the module level using `vi.mock()`. The shared tauri-mock.ts file must cover every Tauri module used by any component.
**Warning signs:** Import errors mentioning `@tauri-apps`, `__TAURI__`, or `window.__TAURI_INTERNALS__`

### Pitfall 3: Svelte Transitions Fail in jsdom
**What goes wrong:** Components using `transition:fly`, `transition:slide` etc. may throw errors or behave unexpectedly
**Why it happens:** jsdom does not support Web Animations API fully; Svelte transitions rely on `requestAnimationFrame` and `getComputedStyle`
**How to avoid:** Transitions will generally be no-ops in jsdom (which is fine). If a component test needs to verify transition-related state, use `vi.useFakeTimers()` and advance past the transition duration. The Toast component uses `transition:fly` and SearchBar uses `transition:slide` -- neither should block testing.
**Warning signs:** Flaky tests that pass sometimes, or transition-related console warnings

### Pitfall 4: $effect Not Running in Unit Tests
**What goes wrong:** Effects don't execute when testing `.svelte.ts` modules outside component context
**Why it happens:** `$effect` only runs inside a Svelte component render cycle or `$effect.root()`
**How to avoid:** For `.svelte.ts` store tests (toast, undo-redo, remote-state), the existing pattern of testing the API directly (not effects) is correct. Effects are tested indirectly through component tests.
**Warning signs:** Store state not updating after calling mutation functions in tests

### Pitfall 5: Lucide Icon Components in jsdom
**What goes wrong:** Components that import `@lucide/svelte` icons may fail if the icon component renders SVG that jsdom cannot handle
**Why it happens:** Lucide components render SVG elements; jsdom supports basic SVG but may have edge cases
**How to avoid:** If icon rendering causes issues, mock `@lucide/svelte` with stub components: `vi.mock("@lucide/svelte", () => new Proxy({}, { get: () => () => "" }))`. Only do this if actual failures occur -- in most cases icons render fine in jsdom.
**Warning signs:** Errors about SVG elements, `createElementNS`, or unknown elements

### Pitfall 6: Test File Naming for Svelte Runes
**What goes wrong:** Test files using `$state` or `$derived` directly don't compile
**Why it happens:** Svelte runes only work in `.svelte.ts` files, not regular `.ts` files
**How to avoid:** Component tests (using `@testing-library/svelte` render) do NOT need runes in the test file -- they test runes indirectly through the component. Use `.test.ts` (not `.svelte.test.ts`) for component tests. Only use `.svelte.test.ts` when the test itself needs to create reactive state (like the existing store tests).
**Warning signs:** Compilation errors about `$state` or `$derived` not being valid identifiers

## Code Examples

### Vitest Setup File
```typescript
// vitest-setup.ts
// Extends vitest with jest-dom matchers like toBeInTheDocument()
import "@testing-library/jest-dom/vitest";
```

### Component Test with Event Handlers
```typescript
// Source: @testing-library/svelte official docs
import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import BranchRow from "./BranchRow.svelte";
import "../__tests__/helpers/tauri-mock";

describe("BranchRow", () => {
  it("renders branch name", () => {
    render(BranchRow, { props: { name: "feature/login" } });
    expect(screen.getByText("feature/login")).toBeInTheDocument();
  });

  it("calls onclick when clicked", async () => {
    const onclick = vi.fn();
    render(BranchRow, { props: { name: "main", onclick } });

    await fireEvent.click(screen.getByRole("button"));
    expect(onclick).toHaveBeenCalledOnce();
  });

  it("shows ahead/behind counts when provided", () => {
    render(BranchRow, { props: { name: "main", ahead: 3, behind: 1 } });
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByText("1")).toBeInTheDocument();
  });

  it("shows error message when isError is true", () => {
    render(BranchRow, {
      props: { name: "main", isError: true, errorText: "Checkout failed" },
    });
    expect(screen.getByText("Checkout failed")).toBeInTheDocument();
  });
});
```

### Component Test with Conditional Rendering
```typescript
import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import OperationBanner from "./OperationBanner.svelte";
import "../__tests__/helpers/tauri-mock";

describe("OperationBanner", () => {
  it("shows 'Merging' for merge operations", () => {
    render(OperationBanner, {
      props: {
        info: {
          op_type: "Merge",
          source_branch: "feature",
          target_branch: "main",
          progress: null,
          source_color_index: 1,
          target_color_index: 0,
          rebase_message: null,
        },
        repoPath: "/test/repo",
      },
    });
    expect(screen.getByText("Merging")).toBeInTheDocument();
    expect(screen.getByText("feature")).toBeInTheDocument();
    expect(screen.getByText("main")).toBeInTheDocument();
  });

  it("shows Continue/Skip/Abort buttons for rebase", () => {
    render(OperationBanner, {
      props: {
        info: {
          op_type: "Rebase",
          source_branch: "feature",
          target_branch: "main",
          progress: "2/5",
          source_color_index: 1,
          target_color_index: 0,
          rebase_message: null,
        },
        repoPath: "/test/repo",
      },
    });
    expect(screen.getByText("Continue")).toBeInTheDocument();
    expect(screen.getByText("Skip")).toBeInTheDocument();
    expect(screen.getByText("Abort")).toBeInTheDocument();
  });
});
```

## Inventory: Existing Test Coverage vs. Modules

### Utility Modules (src/lib/)
| Module | Has Tests | Test File Lines | Gap Assessment |
|--------|-----------|-----------------|----------------|
| active-lanes.ts | Yes | 449 | Thorough; may need WIP node edge cases |
| build-tree.ts | Yes | 250 | Good; check `collectFilePaths` and `countFiles` coverage |
| flatten-tree.ts | Yes | 245 | Good; check `migrateExpanded` and `findFocusIndex` edge cases |
| graph-constants.ts | Yes | 18 | Minimal (just default values); sufficient |
| merge-parser.ts | Yes | 266 | Good; may need empty-line and large-input edge cases |
| overlay-paths.ts | Yes | 418 | Thorough |
| overlay-visible.ts | Yes | 193 | Good |
| ref-pill-data.ts | Yes | 270 | Good |
| text-measure.ts | Yes | 62 | Needs edge cases for empty string, very long text, cache behavior |
| rebase-validation.ts | Yes | 88 | Good; cover empty input edge case |
| toast.svelte.ts | Yes | 60 | Good |
| undo-redo.svelte.ts | Yes | 45 | Good |
| remote-state.svelte.ts | Yes | 32 | Good |
| store.ts | Partial (tab-types only) | 49 | **GAP**: store.ts itself is async (depends on Tauri plugin-store); test with LazyStore mock |
| invoke.ts | No | -- | **GAP**: `safeInvoke` error parsing logic needs unit tests |
| types.ts | N/A (D-05) | -- | Skip -- pure types |
| tab-types.ts | N/A (D-05) | -- | Skip -- covered by store.test.ts |

### Components (src/components/)
| Component | Lines | Complexity | Key Test Focus |
|-----------|-------|------------|----------------|
| Toast.svelte | 19 | Low | Renders toast messages, `role="status"` |
| RefPill.svelte | 60 | Medium | Conditional rendering (showAll, expanded), pill styling |
| RemoteGroup.svelte | 52 | Low | Renders remote name + branch list |
| BranchSection.svelte | 64 | Low | Expand/collapse, count display, create button |
| BranchRow.svelte | 85 | Medium | Click/dblclick/contextmenu handlers, ahead/behind, error state |
| DirectoryRow.svelte | 96 | Medium | Toggle expand, action button, file count |
| CommitRow.svelte | 111 | Medium | Column visibility, search match highlighting, WIP/stash rendering |
| FileRow.svelte | 125 | Medium | Status icons, action button (stage/unstage), depth |
| WelcomeScreen.svelte | 132 | High | Tauri invoke for repo validation, recent repos list |
| InputDialog.svelte | 154 | High | Form validation, keyboard handling, backdrop click |
| SearchBar.svelte | 167 | Medium | Query input, next/prev navigation, keyboard shortcuts |
| PullDropdown.svelte | 175 | Medium | Dropdown open/close, option selection, remote operations |
| OperationBanner.svelte | 181 | High | Merge vs rebase rendering, continue/skip/abort actions |
| TabBar.svelte | 198 | High | Tab rendering, sortable drag, close/activate/reorder |
| CommitForm.svelte | 214 | High | Mode switching (commit/amend/stash), validation, submit |
| TreeFileList.svelte | 228 | Medium | Tree rendering with expand/collapse, file actions |
| CommitDetail.svelte | 245 | Medium | Commit metadata display, diff file list |
| Toolbar.svelte | 284 | High | Button states, remote operations, zoom, keyboard shortcuts |
| DiffPanel.svelte | 631 | High | Hunk rendering, staging actions, scroll sync |
| RepoView.svelte | 660 | Very High | Orchestrates most other components; complex state |
| VirtualList.svelte | 734 | Very High | Scroll virtualization, resize observer |
| BranchSidebar.svelte | 764 | Very High | Branch CRUD, checkout, context menus |
| MergeEditor.svelte | 837 | Very High | Conflict resolution, line selection, output preview |
| RebaseEditor.svelte | 1044 | Very High | Drag reorder, action assignment, validation |
| StagingPanel.svelte | 1340 | Very High | File staging, tree view, commit form integration |
| CommitGraph.svelte | 1826 | Very High | SVG graph, virtual scrolling, overlay rendering |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `new Component({ target })` constructor | `mount(Component, { target, props })` | Svelte 5 (Oct 2024) | @testing-library/svelte handles this internally |
| Assignment-based reactivity `count = count + 1` | Runes: `$state`, `$derived`, `$effect` | Svelte 5 (Oct 2024) | Tests interact with runes through rendered components |
| `@testing-library/svelte` < 5.0 | `@testing-library/svelte` 5.3.1 | 2024-2025 | Built-in Svelte 5 support, `svelteTesting()` vite plugin |

## Open Questions

1. **Very large components (CommitGraph, StagingPanel, RebaseEditor)**
   - What we know: These are 800-1800 lines with deep Tauri integration, multiple child components, and complex state management
   - What's unclear: How deep to test -- full render with all children or shallow render strategy
   - Recommendation: Test key behaviors (renders, click handlers, conditional sections) with mocked children where needed. @testing-library/svelte always does full renders (no shallow rendering), so focus tests on the most important user-facing behaviors. Very complex components can have thinner test coverage initially -- focus on rendering states and primary interactions.

2. **SortableJS in TabBar tests**
   - What we know: TabBar uses `sortablejs` for drag-and-drop tab reordering via `$effect`
   - What's unclear: Whether SortableJS works in jsdom (it manipulates DOM directly)
   - Recommendation: Mock `sortablejs` in TabBar tests. The drag-reorder behavior is a library concern; test that reorder callback fires with correct data.

3. **VirtualList scroll behavior**
   - What we know: VirtualList uses ResizeObserver and scroll events heavily
   - What's unclear: jsdom's ResizeObserver support
   - Recommendation: Mock ResizeObserver globally in vitest-setup.ts if needed: `global.ResizeObserver = vi.fn().mockImplementation(() => ({ observe: vi.fn(), unobserve: vi.fn(), disconnect: vi.fn() }))`. Test that items render for given scroll positions.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest 4.1.1 |
| Config file | `vite.config.ts` (test section) |
| Quick run command | `bun run test` |
| Full suite command | `bun run test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UNIT-02 | TypeScript utility modules have unit tests | unit | `bun run test` | Partial -- 14 files exist, need gaps filled |
| UNIT-03 | Svelte components have unit tests | unit | `bun run test` | None -- all 26 component test files are Wave 0 |

### Sampling Rate
- **Per task commit:** `bun run test`
- **Per wave merge:** `bun run test && bun run check`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `vitest-setup.ts` -- jest-dom matchers setup; ResizeObserver global mock
- [ ] `src/__tests__/helpers/factories.ts` -- shared test factories (extracted from existing tests)
- [ ] `src/__tests__/helpers/tauri-mock.ts` -- shared Tauri module mocks
- [ ] `vite.config.ts` update -- add jsdom environment + svelteTesting() plugin + setupFiles
- [ ] Package installation -- @testing-library/svelte, @testing-library/jest-dom, jsdom
- [ ] 26 component test files -- all new
- [ ] `src/lib/invoke.test.ts` -- new; safeInvoke error parsing tests

## Sources

### Primary (HIGH confidence)
- [Svelte 5 official testing docs](https://svelte.dev/docs/svelte/testing) -- jsdom setup, rune testing, vitest configuration
- [@testing-library/svelte setup docs](https://testing-library.com/docs/svelte-testing-library/setup/) -- svelteTesting plugin, vitest-setup.ts pattern
- [@testing-library/svelte npm](https://www.npmjs.com/package/@testing-library/svelte) -- version 5.3.1, peer deps
- npm registry -- verified versions: jsdom 29.0.1, @testing-library/jest-dom 6.9.1, @testing-library/svelte 5.3.1
- Existing codebase -- 14 test files, 170 passing tests, 26 components examined

### Secondary (MEDIUM confidence)
- [DeepWiki @testing-library/svelte Svelte 5 features](https://deepwiki.com/testing-library/svelte-testing-library/4.1-svelte-5+-features) -- Svelte 5 mount API, rerender, prop reactivity details
- [vitest issue #8633](https://github.com/vitest-dev/vitest/issues/8633) -- SSR transform issue resolved by svelteTesting plugin (closed as not planned -- the plugin is the fix)

### Tertiary (LOW confidence)
- None -- all findings verified against official sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all packages verified against npm registry; versions confirmed current
- Architecture: HIGH -- patterns derived from official docs + existing codebase conventions
- Pitfalls: HIGH -- SSR compilation issue verified via official sources; Tauri mock pattern tested in existing codebase

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable ecosystem, 30-day validity)
