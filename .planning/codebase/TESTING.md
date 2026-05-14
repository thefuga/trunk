# Testing Patterns

**Analysis Date:** 2026-05-14

## Test Framework

### Frontend (Vitest)

**Runner:**
- Vitest v4.1.0
- Config: `vite.config.ts` (inline `test` block — no separate `vitest.config.ts`)
- Environment: jsdom v29.0.1
- Setup file: `vitest-setup.ts`

**Assertion Library:**
- Vitest built-in `expect`
- `@testing-library/jest-dom` v6.9.1 (DOM matchers: `toBeInTheDocument`, `toHaveTextContent`, etc.)
- Types registered in `tsconfig.json`: `"types": ["vitest/globals", "@testing-library/jest-dom/vitest"]`

**Component rendering:**
- `@testing-library/svelte` v5.3.1
- Plugin: `svelteTesting()` from `@testing-library/svelte/vite` registered in `vite.config.ts`

**Run Commands:**
```bash
just vitest          # bun run test (vitest run — single pass)
just vitest-cov      # bun run test -- --coverage.enabled (v8 coverage)
just check           # fmt + biome + svelte-check + clippy + cargo-test + vitest (pre-commit)
```

### Backend (Cargo test)

**Runner:**
- Cargo test (standard `#[test]` harness)
- No additional test framework dependencies

**Coverage:**
```bash
just cargo-test-cov  # cargo llvm-cov + HTML report in rust-coverage-html/
```

**Run Commands:**
```bash
just cargo-test      # cargo test --manifest-path src-tauri/Cargo.toml
```

## Test File Organization

### Frontend

**Location:** Co-located with source files

**Naming:**
- TypeScript lib tests: `src/lib/{name}.test.ts` — `src/lib/build-tree.test.ts`, `src/lib/toast.svelte.test.ts`
- Svelte component tests: `src/components/{Name}.test.ts` — `src/components/BranchSidebar.test.ts`
- Svelte state module tests use `.svelte.test.ts` suffix — `src/lib/toast.svelte.test.ts`, `src/lib/undo-redo.svelte.test.ts`, `src/lib/remote-state.svelte.test.ts`

**Shared helpers:**
- `src/__tests__/helpers/factories.ts` — data factory functions (`makeCommit`, `makeEdge`, `makeFile`, `makeRef`)
- `src/components/__tests__/helpers/tauri-mock` — (referenced by many component tests but file not found at scan time; likely auto-generated or vitest plugin)

**Glob pattern:** `src/**/*.test.ts` (configured in `vite.config.ts`)

### Backend (Rust)

**Location:** Separate `src-tauri/tests/` directory — NOT inline `#[cfg(test)]` modules

**Structure:**
```
src-tauri/tests/
├── common/
│   ├── mod.rs          # pub mod declarations; #![allow(dead_code)]
│   ├── context.rs      # TestContext struct — holds tempdir + state_map + cache_map
│   ├── builder.rs      # TestContextBuilder (fluent builder for git fixture repos)
│   ├── assertions.rs   # TestContext assertion methods (assert_file_staged, assert_head_at, etc.)
│   └── drivers/        # TestContext driver methods (wrap _inner functions)
│       ├── mod.rs
│       ├── branches.rs
│       ├── commit.rs
│       ├── staging.rs
│       └── ...         # one driver per command module
├── test_branches.rs
├── test_staging.rs
├── test_commit.rs
├── test_graph.rs       # 1245 lines
├── test_integ_workflows.rs  # 535 lines
├── test_integ_serde.rs      # 880 lines
└── ...                 # one file per domain
```

## Test Structure

### Frontend Suite Pattern

```typescript
import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ComponentName from "./ComponentName.svelte";

vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));

const mockInvoke = vi.mocked(invoke);

describe("ComponentName", () => {
    beforeEach(() => {
        mockInvoke.mockReset();
        mockInvoke.mockImplementation((cmd: string) => {
            if (cmd === "list_refs") return Promise.resolve(defaultData);
            return Promise.resolve(undefined);
        });
    });

    it("renders without crashing", () => {
        const { container } = render(ComponentName, { props: { repoPath: "/test/repo" } });
        expect(container).toBeTruthy();
    });

    it("displays data after fetch", async () => {
        render(ComponentName, { props: { repoPath: "/test/repo" } });
        await waitFor(() => {
            expect(screen.getByText("main")).toBeInTheDocument();
        });
    });
});
```

### Backend Test Pattern

```rust
mod common;

use common::context::TestContext;

#[test]
fn test_name_describes_behavior() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    let result = ctx.list_refs().expect("list_refs failed");

    assert!(!result.local.is_empty(), "expected at least 1 local branch");
    ctx.assert_head_at("main");
}
```

## Mocking

### Frontend Mocking

**Framework:** Vitest `vi.mock()` with hoisting

**Tauri IPC mock (used in nearly every component test):**
```typescript
vi.mock("@tauri-apps/api/core", () => ({
    invoke: vi.fn().mockResolvedValue(undefined),
}));

const mockInvoke = vi.mocked(invoke);

// Per-test setup in beforeEach:
mockInvoke.mockReset();
mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "list_refs") return Promise.resolve(data);
    return Promise.resolve(undefined);
});
```

**Plugin-level mocks (declared inline per test file):**
```typescript
vi.mock("@tauri-apps/plugin-store", () => {
    const store = new Map<string, unknown>();
    class MockLazyStore {
        get(key: string) { return Promise.resolve(store.get(key) ?? null); }
        set(key: string, value: unknown) { store.set(key, value); return Promise.resolve(); }
        save() { return Promise.resolve(); }
    }
    return { LazyStore: MockLazyStore };
});

vi.mock("@tauri-apps/plugin-dialog", () => ({
    open: vi.fn(),
    ask: vi.fn().mockResolvedValue(false),
    message: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
    listen: vi.fn().mockResolvedValue(() => {}),
}));
```

**Important:** `vi.mock()` calls are hoisted by Vitest — they must appear at module scope, not inside `beforeEach`. The `vi.mocked(invoke)` cast is done once at module level.

**What to mock:**
- All `@tauri-apps/*` modules (IPC, plugins, event, path, window, menu)
- `invoke` is always mocked — never calls real Tauri backend in tests

**What NOT to mock:**
- Pure utility functions (`buildGraphData`, `measureTextWidth`, `buildTree`, etc.)
- Svelte stores and rune-based state — test via rendered component

**Timer mocking (for toast/timeout tests):**
```typescript
beforeEach(() => { vi.useFakeTimers(); });
afterEach(() => { vi.useRealTimers(); });

vi.advanceTimersByTime(3000); // advance time to trigger toast removal
```

### Backend Mocking

No mocking framework — tests use real git repos in `tempfile::TempDir`.

`TestContext` creates a real `git2::Repository` in a temp directory and operates directly on `_inner` functions (which take `&HashMap<String, PathBuf>` instead of Tauri `State`).

## Fixtures and Factories

### Frontend Factories (`src/__tests__/helpers/factories.ts`)

```typescript
// Make a GraphCommit with defaults, only oid required
export function makeCommit(overrides: Partial<GraphCommit> & { oid: string }): GraphCommit {
    return {
        oid: overrides.oid,
        short_oid: overrides.oid.slice(0, 7),
        summary: overrides.summary ?? "test commit",
        author_name: overrides.author_name ?? "Test",
        author_timestamp: overrides.author_timestamp ?? 0,
        parent_oids: overrides.parent_oids ?? [],
        column: overrides.column ?? 0,
        color_index: overrides.color_index ?? 0,
        edges: overrides.edges ?? [],
        refs: overrides.refs ?? [],
        is_head: overrides.is_head ?? false,
        // ...all required fields have defaults
    };
}

export function makeEdge(overrides: Partial<GraphEdge> & { edge_type: GraphEdge["edge_type"] }): GraphEdge
export function makeFile(path: string, status: FileStatus["status"] = "Modified"): FileStatus
export function makeRef(overrides: Partial<RefLabel> & { short_name: string }): RefLabel
```

### Backend Fixtures (`TestContextBuilder`)

Fluent builder in `src-tauri/tests/common/builder.rs`:

```rust
let ctx = TestContext::builder()
    .with_file("README.md", "hello")    // write file to working tree
    .with_commit("Initial commit")       // stage pending files + commit
    .with_branch("feature")             // create branch at HEAD
    .checkout("feature")                // switch branches
    .with_file("feature.txt", "work")
    .with_commit("Feature work")
    .checkout("main")
    .merge("feature")                   // perform merge commit
    .with_conflict("feature")           // leave repo in conflict state
    .with_tag("v1.0.0")
    .with_stash(Some("stash message"))
    .with_remote("origin")              // add bare repo as remote
    .build();                           // returns TestContext
```

`TestContext::new_empty()` creates a repo with no commits (HEAD not yet pointing anywhere).

## vitest-setup.ts

Located at `vitest-setup.ts` (repo root). Imported by all tests via `setupFiles` in `vite.config.ts`:

```typescript
import "@testing-library/jest-dom/vitest";

// Stub ResizeObserver (jsdom does not implement it — needed by VirtualList)
globalThis.ResizeObserver = class { observe() {} unobserve() {} disconnect() {} };

// Stub Element.prototype.animate (needed by Svelte transitions)
Element.prototype.animate = () => ({ finished: Promise.resolve(), cancel() {}, ... });
```

Additional per-file stubs exist for:
- `OffscreenCanvas` (needed by `text-measure.ts` — stubbed in `CommitGraph.test.ts`)
- `Element.prototype.scrollTo` (needed by VirtualList — stubbed in `CommitGraph.test.ts`)

## Coverage

**Frontend (v8):**
- Provider: `@vitest/coverage-v8`
- Reports: `text`, `lcov`, `html`
- Output: `./coverage/`
- Includes: `src/**/*.ts`, `src/**/*.svelte`
- Excludes: `src/**/*.test.ts`

**Backend (llvm-cov):**
- Output: `rust-lcov.info` (lcov), `rust-coverage-html/` (HTML)

**Requirements:** No enforced coverage threshold — no `coverageThreshold` configured.

## Test Types

**Frontend Unit Tests (lib):**
- Test pure functions in isolation
- Examples: `src/lib/build-tree.test.ts`, `src/lib/active-lanes.test.ts`, `src/lib/merge-parser.test.ts`
- No mocking needed — pure functions with typed inputs

**Frontend Component Tests:**
- Render Svelte components with `@testing-library/svelte`
- Mock all Tauri IPC (`invoke` + plugins)
- Assert DOM state via `screen.getByText()`, `screen.getByTestId()`, `toBeInTheDocument()`
- Use `waitFor()` for async state updates after `invoke` resolves

**Backend Integration Tests:**
- All in `src-tauri/tests/test_*.rs`
- Use real git repos in `tempfile::TempDir`
- Test `_inner` functions directly (no Tauri runtime needed)
- `test_integ_serde.rs`: validates JSON serialization shape matches frontend types
- `test_integ_workflows.rs`: multi-step git workflow tests (branch, merge, rebase, stash)

**E2E Tests:** Not used.

## Common Patterns

**Async Component Testing:**
```typescript
render(BranchSidebar, { props: { repoPath: "/test/repo" } });
await waitFor(() => {
    expect(screen.getByText("main")).toBeInTheDocument();
});
```

**Error Path Testing (frontend):**
```typescript
mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "create_branch")
        return Promise.reject(
            JSON.stringify({ code: "branch_exists", message: "branch 'feature' already exists" })
        );
    return Promise.resolve(undefined);
});
```

**Error Path Testing (Rust):**
```rust
let result = ctx.checkout_branch("nonexistent");
assert!(result.is_err(), "expected error for nonexistent branch");
```

**data-testid usage:**
Components expose `data-testid` attributes for test selectors:
- `data-testid="branch-row"` on branch row elements
- `data-testid="branch-section-remote"` on remote section container
- Used as: `screen.getByTestId("branch-section-remote")`

**Inline mock data (no shared factory for Rust types):**
Frontend tests define local helper functions that return typed mock objects (e.g., `mockListRefs()` in `BranchSidebar.test.ts`), providing override support via optional partial objects.

---

*Testing analysis: 2026-05-14*
