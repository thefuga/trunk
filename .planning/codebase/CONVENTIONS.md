# Coding Conventions

**Analysis Date:** 2026-05-14

## Naming Patterns

**Files:**
- Svelte components: `PascalCase.svelte` — `BranchSidebar.svelte`, `CommitGraph.svelte`
- TypeScript lib modules: `kebab-case.ts` — `build-tree.ts`, `active-lanes.ts`, `overlay-paths.ts`
- Svelte state modules: `kebab-case.svelte.ts` — `toast.svelte.ts`, `undo-redo.svelte.ts`, `remote-state.svelte.ts`
- Test files: co-located, same name + `.test.ts` — `BranchSidebar.test.ts`, `build-tree.test.ts`
- Rust modules: `snake_case.rs` — `branches.rs`, `commit_actions.rs`, `interactive_rebase.rs`

**Functions (TypeScript):**
- camelCase everywhere — `safeInvoke`, `buildGraphData`, `measureTextWidth`, `truncateWithEllipsis`
- Factory helpers in `src/__tests__/helpers/factories.ts` use `make` prefix — `makeCommit`, `makeEdge`, `makeFile`, `makeRef`

**Functions (Rust):**
- snake_case everywhere (Rust standard)
- Inner (testable) functions follow `{command_name}_inner` pattern — `list_refs_inner`, `checkout_branch_inner`, `get_status_inner`
- Tauri command wrappers have the same name without `_inner` — `list_refs`, `checkout_branch`

**Variables:**
- TypeScript: camelCase — `mockInvoke`, `recentRepos`, `columnWidths`
- Rust: snake_case — `state_map`, `path_buf`, `graph_result`

**Types/Interfaces:**
- TypeScript: PascalCase interfaces — `GraphCommit`, `BranchInfo`, `TrunkError`, `RefsResponse`
- TypeScript: `type` (not `enum`) for string literal unions — `RefType = "LocalBranch" | "RemoteBranch" | "Tag" | "Stash"` (see `src/lib/types.ts`)
- Rust: PascalCase structs/enums — `TrunkError`, `GraphCommit`, `BranchInfo`, `RefType`

**Constants (TypeScript):**
- SCREAMING_SNAKE_CASE for layout/display constants — `LANE_WIDTH`, `ROW_HEIGHT`, `DOT_RADIUS`, `PILL_HEIGHT` (see `src/lib/graph-constants.ts`)

## Code Style

**Formatting:**
- Biome (v2.4.9) handles formatting and linting for all frontend files
- Config: `biome.json` at repo root
- Applies to: `src/**`, `*.ts`, `*.js`, `*.json`, `*.svelte`
- Import organization: Biome `organizeImports` action set to `"on"`
- `src/components/virtual-list/**` is excluded from all Biome rules

**Linting:**
- Biome recommended rules enabled
- `clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` (warnings are errors)
- No `.eslintrc` — Biome replaces ESLint

**Svelte-specific Biome overrides** (in `biome.json`):
- `useConst` and `useImportType` are turned off for `.svelte` files
- `noUnusedVariables` and `noUnusedImports` are turned off for `.svelte` files (Svelte template binding prevents static analysis)

**Rust formatting:**
- `cargo fmt` enforced in CI via `just fmt`

## Import Organization

**TypeScript/Svelte order (Biome auto-organizes):**
1. External packages — `@tauri-apps/api/core`, `@lucide/svelte`, `svelte`
2. Internal `$lib` alias — `$lib/types`, `$lib/invoke`
3. Relative path imports — `../lib/active-lanes.js`, `./CommitGraph.svelte`

**Note:** `.js` extension is used in relative imports even for `.ts` source files (ESM bundler requirement). Example from `src/lib/invoke.test.ts`:
```typescript
import { safeInvoke } from "./invoke.js";
```

**Path Aliases:**
- `$lib` → `src/lib` (configured in `tsconfig.json` and used throughout)
- `$lib/*` → `src/lib/*`

## Error Handling

**Rust — all git operations:**
- Internal functions return `Result<T, TrunkError>` where `TrunkError` is `{ code: String, message: String }`
- `TrunkError` derives `Serialize` — serialized to JSON string for IPC
- `From<git2::Error> for TrunkError` converts libgit2 errors automatically (see `src-tauri/src/error.rs`)
- Tauri command wrappers return `Result<T, String>` — the `String` is a JSON-encoded `TrunkError`
- Pattern used uniformly:
  ```rust
  .map_err(|e| serde_json::to_string(&e).unwrap())
  ```

**Frontend — Tauri IPC:**
- All Tauri calls go through `safeInvoke<T>(cmd, args?)` in `src/lib/invoke.ts`
- Raw Tauri errors are JSON strings; `safeInvoke` parses them into `TrunkError { code, message }`
- Non-JSON or non-string errors become `{ code: "unknown_error", message: "..." }`
- Components `catch` errors from `safeInvoke` and show toast notifications (`showToast` from `src/lib/toast.svelte.ts`)

**Example pattern in components:**
```typescript
try {
  await safeInvoke("checkout_branch", { path: repoPath, name });
} catch (e) {
  const err = e as TrunkError;
  showToast(err.message, "error");
}
```

## Logging

**Framework:** No logging library — no console.log in production paths
- Rust: no tracing/log crate is used; errors propagate via `Result`
- Frontend: Tauri dev tools used for debugging; no structured logging

## Comments

**When to Comment:**
- Rust: doc comments (`///`) on public functions and structs that are non-obvious
- Rust: module-level `//` comments explaining invariants (see `src-tauri/src/git/types.rs` header: "CRITICAL: All fields use owned types...")
- Rust: inline comments on non-trivial filtering logic or safety rationale
- TypeScript: block comments above file-level constants explaining purpose
- No JSDoc on TypeScript — types are self-documenting via TypeScript strict mode

**Key doc comment examples:**
- `src-tauri/src/state.rs`: "CRITICAL: Store PathBuf ONLY — git2::Repository is not Sync."
- `src-tauri/src/git/types.rs`: "CRITICAL: All fields use owned types... NO git2 types"
- `src-tauri/src/commands/branches.rs`: `/// Inner implementation of list_refs — separated for testability without Tauri state.`

## Svelte Component Design

**Props declaration pattern (Svelte 5):**
```svelte
<script lang="ts">
interface Props {
  name: string;
  kind?: "local" | "remote" | "tag";
  isHead?: boolean;
  onclick?: () => void;
}

let { name, kind = "local", isHead = false, onclick }: Props = $props();
```

**Reactive state:**
- `$state()` for mutable local values — `let hovered = $state(false);`
- `$derived()` for computed values — `const KindIcon = $derived(kindIcon[kind]);`
- `$effect()` for side effects / lifecycle — used for data fetching and event listeners

**No reactive stores from Svelte 4** — all state management uses Svelte 5 runes.

## Styling

**Rule: Never inline raw colors.** Always use CSS custom properties from `src/app.css`.

All color tokens defined in `src/app.css` under `:root`:
- Base: `--color-bg`, `--color-surface`, `--color-border`, `--color-text`, `--color-text-muted`, `--color-accent`
- Diff: `--color-diff-add`, `--color-diff-delete`, `--color-diff-add-bg`, `--color-diff-delete-bg`
- Semantic: `--color-success`, `--color-danger`, `--color-warning`, `--color-accent-alt`
- Syntax: `--color-syn-keyword`, `--color-syn-string`, `--color-syn-comment`, etc.
- Graph lanes: `--lane-0` through `--lane-7`

**Violations to avoid (currently exist in codebase):**
- Hardcoded `#f87171`, `#3d1c1c`, `#6b2a2a` in `BranchRow.svelte`, `CommitForm.svelte`, `WelcomeScreen.svelte`, `CommitGraph.svelte`, `BranchSidebar.svelte`
- These should use `var(--color-danger)`, `var(--color-danger-bg)`, `var(--color-danger-border)`

**Layout:**
- Prefer inline `style=` attribute with CSS properties for layout-specific values
- Tailwind utility classes used sparingly alongside inline styles
- No CSS positioning hacks — use grid/flexbox

## Module Design

**Exports:**
- Named exports only — no default exports in `.ts` files
- Svelte components export themselves as default (implicit in `.svelte` files)

**Barrel files:** Not used — import directly from specific files

## TypeScript DTOs

All types in `src/lib/types.ts` mirror Rust DTOs from `src-tauri/src/git/types.rs`.
- Comment at top: "Use string literal unions (not enum) — matches serde default serialization"
- Field names use `snake_case` to match serde JSON output from Rust (e.g., `author_name`, `is_head`, `color_index`)

---

*Convention analysis: 2026-05-14*
