# Codebase Structure

**Analysis Date:** 2026-05-14

## Directory Layout

```
trunk/
├── src/                              # Frontend: Svelte 5 + TypeScript
│   ├── App.svelte                    # Root component: tab manager, global layout
│   ├── app.css                       # Global CSS: Tailwind + CSS custom properties
│   ├── main.ts                       # Entry point: mounts App.svelte to #app
│   ├── components/                   # Svelte UI components (co-located with tests)
│   │   ├── diff/                     # Diff viewer sub-components
│   │   │   ├── DiffToolbar.svelte
│   │   │   ├── DiffViewer.svelte
│   │   │   ├── FullFileView.svelte
│   │   │   ├── HunkView.svelte
│   │   │   └── SplitView.svelte
│   │   ├── virtual-list/             # Vendored virtual list implementation
│   │   │   ├── reactive-list-manager/
│   │   │   └── utils/
│   │   ├── BranchRow.svelte          # Single branch row in sidebar
│   │   ├── BranchSection.svelte      # Collapsible group (Local/Remote/Tags/Stashes)
│   │   ├── BranchSidebar.svelte      # Full branch panel
│   │   ├── CommitDetail.svelte       # Commit metadata display
│   │   ├── CommitForm.svelte         # Commit message input + stage/commit button
│   │   ├── CommitGraph.svelte        # Graph view + SVG overlay (main graph component)
│   │   ├── CommitRow.svelte          # Single row in graph
│   │   ├── DiffPanel.svelte          # Diff display orchestrator
│   │   ├── DirectoryRow.svelte       # Tree-view directory row in file list
│   │   ├── FileRow.svelte            # Single file row in staging/diff
│   │   ├── InputDialog.svelte        # Reusable text-input dialog
│   │   ├── MergeEditor.svelte        # Three-way merge conflict editor
│   │   ├── OperationBanner.svelte    # In-progress merge/rebase banner
│   │   ├── PullDropdown.svelte       # Pull strategy picker dropdown
│   │   ├── RebaseEditor.svelte       # Interactive rebase todo list
│   │   ├── RefPill.svelte            # Branch/tag label pill in graph
│   │   ├── RemoteGroup.svelte        # Remote branch group in sidebar
│   │   ├── RepoView.svelte           # Per-repo state orchestrator (main view)
│   │   ├── SearchBar.svelte          # Commit search input
│   │   ├── StagingPanel.svelte       # Working tree status + stage/unstage UI
│   │   ├── TabBar.svelte             # Tab strip at top
│   │   ├── Toast.svelte              # Toast notification display
│   │   ├── Toolbar.svelte            # Fetch/pull/push/undo controls
│   │   ├── TreeFileList.svelte       # Tree-view file list (alternative to flat list)
│   │   ├── VirtualList.svelte        # Windowed list renderer (SVG overlay aware)
│   │   └── WelcomeScreen.svelte      # Repo picker for empty tabs
│   ├── lib/                          # Pure utilities and reactive stores
│   │   ├── __tests__/                # Shared test helpers
│   │   │   └── rebase-validation.test.ts
│   │   ├── active-lanes.ts           # Graph pipeline layer 2: GraphCommit[] → OverlayGraphData
│   │   ├── build-tree.ts             # Build file tree from flat path list
│   │   ├── diff-utils.ts             # Diff display helpers
│   │   ├── flatten-tree.ts           # Flatten tree for virtual list rendering
│   │   ├── graph-constants.ts        # SVG layout constants (lane width, row height, etc.)
│   │   ├── invoke.ts                 # safeInvoke wrapper — ALL IPC calls go here
│   │   ├── merge-parser.ts           # Parse three-way merge conflict markers
│   │   ├── overlay-paths.ts          # Graph pipeline layer 3: OverlayGraphData → SVG paths
│   │   ├── overlay-visible.ts        # Filter overlay elements to visible viewport
│   │   ├── path.ts                   # Path string utilities
│   │   ├── rebase-validation.ts      # Validate interactive rebase todo before submit
│   │   ├── ref-pill-data.ts          # Graph pipeline: compute ref pill geometry
│   │   ├── remote-state.svelte.ts    # Per-tab remote operation reactive state
│   │   ├── store.ts                  # All persistent prefs (plugin-store wrapper)
│   │   ├── tab-types.ts              # TabInfo, PersistedTab interfaces
│   │   ├── text-measure.ts           # Canvas-based text width measurement for pills
│   │   ├── toast.svelte.ts           # Global toast store (module-level $state)
│   │   ├── types.ts                  # All TypeScript DTO interfaces (mirrors Rust types.rs)
│   │   └── undo-redo.svelte.ts       # Per-tab redo stack reactive state
│   └── __tests__/                    # Cross-component test helpers
│       ├── helpers/factories.ts      # Test data factories
│       └── helpers/tauri-mock.ts     # Mock safeInvoke for component tests
│
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── main.rs                   # Binary entry: calls trunk_lib::run()
│   │   ├── lib.rs                    # App setup: plugins, state, full invoke_handler table
│   │   ├── error.rs                  # TrunkError type (code + message, JSON-serializable)
│   │   ├── shell_env.rs              # macOS PATH resolution via /usr/libexec/path_helper
│   │   ├── state.rs                  # RepoState, CommitCache, RunningOp managed state
│   │   ├── watcher.rs                # notify_debouncer_mini fs watcher, emits "repo-changed"
│   │   ├── commands/                 # Tauri command handlers (one file per domain)
│   │   │   ├── mod.rs                # Re-exports all command modules
│   │   │   ├── repo.rs               # open_repo, close_repo, force_close_repo
│   │   │   ├── history.rs            # get_commit_graph, refresh_commit_graph, search_commits
│   │   │   ├── branches.rs           # list_refs, checkout_branch, create_branch, etc.
│   │   │   ├── staging.rs            # get_status, stage/unstage/discard file/hunk/lines
│   │   │   ├── commit.rs             # create_commit, amend_commit, get_head_commit_message
│   │   │   ├── diff.rs               # diff_unstaged, diff_staged, diff_commit, etc.
│   │   │   ├── stash.rs              # list_stashes, stash_save/pop/apply/drop
│   │   │   ├── commit_actions.rs     # checkout_commit, cherry_pick, reset, undo/redo
│   │   │   ├── remote.rs             # git_fetch, git_pull, git_push (subprocess + progress)
│   │   │   ├── operation_state.rs    # get_operation_state, merge/rebase continue/abort
│   │   │   ├── interactive_rebase.rs # get_rebase_todo, start_interactive_rebase
│   │   │   └── merge_editor.rs       # get_merge_sides, save_merge_result
│   │   └── git/                      # Pure git2-based logic
│   │       ├── mod.rs                # Re-exports: graph, repository, syntax, types
│   │       ├── graph.rs              # walk_commits(): lane algorithm, color assignment
│   │       ├── repository.rs         # validate_and_open(), build_ref_map()
│   │       ├── syntax.rs             # Syntax highlighting via syntect
│   │       └── types.rs              # All Rust DTOs (GraphCommit, FileDiff, etc.)
│   ├── Cargo.toml                    # Rust dependencies
│   ├── Cargo.lock
│   ├── tauri.conf.json               # Tauri app configuration
│   └── capabilities/                 # Tauri security capability files
│
├── e2e/                              # End-to-end tests (WebDriver / tauri-driver)
├── static/                           # Static assets (icons, etc.)
├── docs/                             # Documentation and design specs
├── .planning/                        # GSD planning files
│   ├── STATE.md                      # Current milestone and phase progress
│   ├── PROJECT.md                    # Requirements and architecture decisions
│   ├── ROADMAP.md                    # All phases with success criteria
│   ├── COMMIT-GRAPH-ARCHITECTURE.md  # Deep reference for graph pipeline
│   ├── codebase/                     # Codebase map documents (this directory)
│   ├── phases/                       # Per-phase CONTEXT, PLAN, SUMMARY docs
│   └── milestones/                   # Archived milestone docs
├── .claude/                          # Claude/GSD tooling
│   ├── rules/                        # Project-specific coding rules
│   └── agents/                       # Agent definitions
├── index.html                        # HTML shell (loads src/main.ts via Vite)
├── package.json                      # Frontend dependencies
├── vite.config.ts                    # Vite build config (Svelte plugin, $lib alias)
├── tsconfig.json                     # TypeScript strict config
├── biome.json                        # Biome linter/formatter config
├── vitest-setup.ts                   # Vitest global setup
├── justfile                          # Build recipes (dev, build, check)
└── mise.toml                         # Runtime version pins (Node, Rust)
```

## Directory Purposes

**`src/components/`:**
- Purpose: All Svelte UI components
- Contains: `.svelte` files + co-located `.test.ts` files
- Key files: `App.svelte` (root), `RepoView.svelte` (per-repo orchestrator), `CommitGraph.svelte` (graph), `StagingPanel.svelte`, `Toolbar.svelte`
- Naming: PascalCase for all component files

**`src/components/diff/`:**
- Purpose: Diff display sub-components used by `DiffPanel.svelte`
- Contains: `DiffViewer.svelte`, `HunkView.svelte`, `SplitView.svelte`, `FullFileView.svelte`, `DiffToolbar.svelte`

**`src/components/virtual-list/`:**
- Purpose: Vendored virtual list implementation, not project-authored code
- Contains: `ReactiveListManager`, height calculation utils, scroll calculation utils
- Note: `VirtualList.svelte` is the public interface; internals are vendored

**`src/lib/`:**
- Purpose: Pure TypeScript utilities, IPC wrapper, reactive stores
- Contains: Business logic with no Svelte rendering
- Key files: `invoke.ts` (IPC), `types.ts` (all DTOs), `store.ts` (persistence), `active-lanes.ts` + `overlay-paths.ts` (graph pipeline)
- Naming: kebab-case for all library files; `.svelte.ts` suffix for files using Svelte runes outside components

**`src-tauri/src/commands/`:**
- Purpose: Tauri command handlers — one file per feature domain
- Contains: `#[tauri::command]` async functions only; no git logic
- Naming: snake_case module files matching domain (`staging.rs`, `branches.rs`, etc.)

**`src-tauri/src/git/`:**
- Purpose: Pure git2 business logic; no Tauri dependencies
- Contains: `graph.rs` (lane algorithm), `repository.rs` (repo helpers), `types.rs` (owned DTOs), `syntax.rs` (highlighting)

## Key File Locations

**Entry Points:**
- `src/main.ts`: Frontend mount point
- `src/App.svelte`: Frontend root component
- `src-tauri/src/main.rs`: Rust binary entry (delegates to `lib.rs`)
- `src-tauri/src/lib.rs`: Full app initialization and command registration table

**IPC Bridge:**
- `src/lib/invoke.ts`: `safeInvoke()` — single point for all frontend→backend calls
- `src-tauri/src/lib.rs:68-140`: `invoke_handler!` — all 47 registered commands

**Type Definitions:**
- `src/lib/types.ts`: All TypeScript DTO interfaces (mirrors Rust)
- `src-tauri/src/git/types.rs`: All Rust DTOs (source of truth for data shapes)

**Persistence:**
- `src/lib/store.ts`: All persistent preferences via `LazyStore("trunk-prefs.json")`

**Graph Pipeline:**
- `src-tauri/src/git/graph.rs`: Layer 1 — `walk_commits()`, lane algorithm
- `src/lib/active-lanes.ts`: Layer 2 — `buildGraphData()`, OverlayGraphData
- `src/lib/overlay-paths.ts`: Layer 3 — `buildOverlayPaths()`, SVG paths
- `src/components/CommitGraph.svelte`: Layer 4 — rendering

**Shared Backend State:**
- `src-tauri/src/state.rs`: `RepoState`, `CommitCache`, `RunningOp`
- `src-tauri/src/watcher.rs`: `WatcherState`, `start_watcher`, `stop_watcher`

**Configuration:**
- `vite.config.ts`: Vite + Svelte plugin, `$lib` alias to `src/lib`
- `tsconfig.json`: TypeScript strict mode config
- `biome.json`: Linter/formatter rules
- `src-tauri/tauri.conf.json`: Tauri app config (bundle ID, window settings, plugins)
- `justfile`: All development commands (`just dev`, `just check`, `just build`)

## Naming Conventions

**Files:**
- Svelte components: PascalCase — `CommitGraph.svelte`, `BranchSidebar.svelte`
- Component tests: Same name + `.test.ts` — `CommitGraph.test.ts`
- Library TypeScript: kebab-case — `active-lanes.ts`, `ref-pill-data.ts`
- Library files using Svelte runes: `.svelte.ts` suffix — `remote-state.svelte.ts`, `toast.svelte.ts`
- Rust command modules: snake_case — `commit_actions.rs`, `interactive_rebase.rs`

**Directories:**
- Frontend components: `src/components/` (flat, except `diff/` subdirectory)
- Frontend library: `src/lib/` (flat)
- Rust commands: `src-tauri/src/commands/` (flat)
- Rust git logic: `src-tauri/src/git/` (flat)

**Symbols:**
- Svelte components: PascalCase
- TypeScript functions: camelCase
- TypeScript interfaces: PascalCase
- Rust functions/commands: snake_case
- Rust types/structs: PascalCase
- Tauri command names (invoke strings): snake_case — `"open_repo"`, `"get_commit_graph"`

## Where to Add New Code

**New Tauri command:**
1. Add the `#[tauri::command]` function to the appropriate file in `src-tauri/src/commands/` (or create a new module file)
2. If new module: add `pub mod your_module;` to `src-tauri/src/commands/mod.rs`
3. Register in `tauri::generate_handler![...]` in `src-tauri/src/lib.rs`
4. Add TypeScript call via `safeInvoke("your_command", args)` in the relevant component
5. Add TypeScript DTO interface to `src/lib/types.ts` if new data shape is returned

**New git operation (local, git2-based):**
- Implementation: `src-tauri/src/git/` (pure git2 function, no Tauri types)
- Called from: the appropriate file in `src-tauri/src/commands/`
- New DTO: `src-tauri/src/git/types.rs` (Rust) + `src/lib/types.ts` (TypeScript mirror)

**New Svelte component:**
- Implementation: `src/components/YourComponent.svelte`
- Test: `src/components/YourComponent.test.ts` (co-located)
- Use `safeInvoke()` from `src/lib/invoke.ts` for all backend calls
- Use CSS custom properties for all colors (never inline color values)

**New persistent preference:**
- Add getter/setter pair to `src/lib/store.ts` using the existing `LazyStore` pattern
- Add a key constant (e.g., `const YOUR_KEY = "your_pref_key"`)

**New reactive store (non-persisted):**
- If per-tab: add to the `TabState` interface in `src/App.svelte` and the `getOrCreateTabState` factory; expose via props
- If global singleton: add to an existing `.svelte.ts` file in `src/lib/` or create a new `.svelte.ts` file using module-level `$state`

**New frontend library utility:**
- Location: `src/lib/your-utility.ts`
- Test: `src/lib/your-utility.test.ts` (co-located)

## Special Directories

**`src/components/virtual-list/`:**
- Purpose: Vendored virtual list implementation (not project-authored)
- Generated: No
- Committed: Yes
- Note: Do not modify; treat as a dependency

**`src-tauri/target/`:**
- Purpose: Rust build artifacts
- Generated: Yes
- Committed: No

**`node_modules/`:**
- Purpose: npm/bun dependencies
- Generated: Yes
- Committed: No

**`.planning/`:**
- Purpose: GSD planning documents (milestones, phases, codebase maps, state)
- Generated: Partially (via GSD commands)
- Committed: Yes

**`.claude/`:**
- Purpose: Claude/GSD tooling, coding rules, agent definitions
- Generated: No
- Committed: Yes

**`e2e/`:**
- Purpose: End-to-end tests (Tauri WebDriver)
- Generated: No
- Committed: Yes

---

*Structure analysis: 2026-05-14*
