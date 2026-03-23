# Project Research Summary

**Project:** Trunk v0.9 — Multi-tab & Tree View
**Domain:** Desktop Git GUI — multi-repository tab management and directory tree file views
**Researched:** 2026-03-23
**Confidence:** HIGH

## Executive Summary

Trunk v0.9 adds two independent features to an existing, production-quality Tauri 2 + Svelte 5 + Rust Git GUI: multi-tab repository management and tree view file lists. The research finding that shapes the entire milestone is that **the Rust backend is already multi-repo capable** — all state structures (`RepoState`, `CommitCache`, `WatcherState`) are `HashMap<String, _>` keyed by repo path, and every command already accepts a `path` argument. The challenge is entirely on the frontend. The current `App.svelte` is a ~630-line god component holding ~30 `$state` variables for a single repo, and the core v0.9 work is extracting that into a per-tab `RepoView.svelte` component.

The recommended approach for multi-tab is HTML/JS tabs within a single Tauri window — the same approach used by GitKraken, Fork, SourceTree, and Spacedrive. Tauri's multi-webview support is experimental, has known panics, and would require all UI state to travel through Rust IPC. For tree view, a custom ~120-line recursive Svelte 5 component is the correct choice; the only credible third-party option (`@keenmate/svelte-treeview`) brings FlexSearch as a dependency and is overengineered for the <500-file lists in a staging panel. Both features require zero new npm or crate dependencies.

The critical risks are cross-tab state contamination from ES module singletons (`remoteState`, `undoRedoState`) and the `RunningOp` global Mutex in Rust. Left unaddressed, these produce data corruption: the undo/redo stack shared between tabs can replay a commit from Tab A into Tab B's repository. These must be scoped per-repo before tab switching is exposed to users. Secondary risks are filesystem watcher accumulation with many open tabs and tree view expand/collapse state being wiped on every status refresh.

## Key Findings

### Recommended Stack

No new dependencies are needed for either feature. The existing stack — Svelte 5 runes for component-local state, `tauri-plugin-store` LazyStore for preferences, `notify` debouncer for FS watching — fully covers v0.9. Tab state is managed via a new shared `$state` rune module (`tab-state.svelte.ts`) following the established pattern of `remote-state.svelte.ts` and `undo-redo.svelte.ts`. The tree view uses a pure frontend transformation (`buildTree()` in `file-tree.ts`) on the flat paths already returned by `git2`.

**Core technologies:**
- Svelte 5 runes (`$state`, `$derived`, `$effect`): per-tab state isolation via component instances — no extra library needed
- `tauri-plugin-store` 2.4.2: replace `open_repo` single-object persistence with `open_tabs` array
- Tauri 2 `invoke`/`listen`: no changes to the event system; path-based filtering already works correctly per-tab
- `notify` 7 / `notify-debouncer-mini` 0.5: existing per-path watcher infrastructure; each tab's watcher is already isolated by design

### Expected Features

**Must have (table stakes):**
- Tab bar with repo name, active indicator, close (X) button per tab, + button for new tab — every tabbed Git GUI shows this
- Cmd+T / Cmd+W for new/close tab, Cmd+1-9 for switching, Ctrl+Tab for next/prev — GitKraken/browser convention users already know
- Independent state per tab — each tab must be a fully isolated repo context with its own diff, selection, rebase state
- Tab order and active tab persisted and restored on app relaunch
- Dirty indicator (dot badge) on background tabs with uncommitted changes
- WelcomeScreen as new-tab page — consistent with existing splash screen; clicking a repo in the picker transitions that tab to the repo view
- Flat/tree view toggle on each file list section — GitKraken, Fork, VS Code, Lazygit all offer this
- Directory expand/collapse with chevrons
- Stage/unstage entire directory via hover action on directory nodes
- Keyboard navigation in tree (arrow keys, left/right to collapse/expand, enter to select)

**Should have (competitive differentiators):**
- Tab context menu (Close Others, Close All, Copy Path) — standard in editors
- Middle-click to close tab — browser convention power users expect
- Path compression (merge single-child `src/ > lib/` into `src/lib/`) — VS Code and Lazygit do this; reduces visual noise significantly
- File count badge on directory nodes
- Expand All / Collapse All buttons in file list header

**Defer to future milestone:**
- Drag tab to new OS window — requires Tauri multi-window, serialized state, cross-window IPC; very high complexity
- Workspace/group management — GitKraken-style cloud team feature; overkill for personal desktop use
- Tab search / fuzzy finder — Cmd+1-9 and Ctrl+Tab cover navigation for v0.9 scale
- Virtual scrolling for file tree — file counts in staging are rarely >500; not needed
- Auto-open submodules as child tabs — niche use case, complex to implement correctly

### Architecture Approach

The architecture split is clean: extract `RepoView.svelte` from `App.svelte` (moving all repo-scoped state into it), then make `App.svelte` a thin tab orchestrator that renders one `RepoView` per active tab using Svelte's `{#key activeTab.id}` destroy/recreate pattern. Tab state lives in a new `tab-state.svelte.ts` shared module. For the Rust side, only `RunningOp` requires a change (from `Option<u32>` to `HashMap<String, u32>`); all other backend state is already multi-repo. Tree view is a pure frontend concern: a `buildTree()` utility transforms flat git2 paths into a `TreeNode[]` tree, consumed by a `FileList.svelte` wrapper that toggles between the existing `FileRow` flat list and a new `TreeRow.svelte` recursive renderer.

**Major components:**
1. `tab-state.svelte.ts` — shared state module holding `tabs: TabInfo[]`, `activeTabId`, and mutation functions; consumed only by `App.svelte` and `TabBar.svelte`
2. `RepoView.svelte` — extracted repo workspace containing all ~30 state variables currently in `App.svelte`; one instance mounted per active tab
3. `repo-context.svelte.ts` — per-repo context factory (`createRepoContext(path)`) holding `remoteState` and `undoRedoState`; scoped to each tab via Svelte `setContext`/`getContext`
4. `file-tree.ts` — pure utility with `buildTree<T>()` and `flattenTree()` functions; no Svelte dependency; fully unit-testable
5. `FileList.svelte` — flat/tree toggle wrapper that replaces `{#each}` file loops in `StagingPanel` and `CommitDetail`
6. `TreeRow.svelte` — recursive tree node renderer using Svelte 5 self-import; wraps existing `FileRow` at leaf nodes

### Critical Pitfalls

1. **ES module singleton state leaks between tabs** (`remoteState`, `undoRedoState`) — Move per-repo state into a `createRepoContext(path)` factory; use Svelte `setContext`/`getContext` to scope it to each tab's component subtree. Without this fix, undo in Tab A can push a commit to Tab B's repository.

2. **`RunningOp` global Mutex blocks concurrent remote ops** — Change `RunningOp` from `Mutex<Option<u32>>` to `Mutex<HashMap<String, u32>>` keyed by repo path. Update `cancel_remote_op` to accept a `path` param. Without this, Tab B cannot fetch while Tab A is pushing.

3. **`App.svelte` monolithic state must be extracted before tabs exist** — Extract all repo-scoped `$state` variables (~30) and handler functions into `RepoView.svelte` first. Attempting to add tab switching before this extraction produces unfixable state bleed.

4. **Tree expand/collapse state lost on every `repo-changed` refresh** — Track expanded directory paths in a `Set<string>` separate from the tree data structure; reapply on rebuild. Must be designed in from day one; retrofitting is painful and tends to produce inconsistent behavior.

5. **Filesystem watcher accumulation at scale** — Verify `close_repo` cleanup runs on tab close. For large repos, consider increasing debounce interval for background tabs (300ms active → 2000ms background) to reduce CPU and fd pressure.

## Implications for Roadmap

Based on research, the dependency graph is clear: Rust fixes are prerequisites for tabs, tab extraction is a prerequisite for tab switching, tree utility is a prerequisite for tree UI. The natural phase structure follows these dependencies.

### Phase 1: Backend State Scoping (Rust)

**Rationale:** `RunningOp` is a global Mutex that blocks concurrent remote operations across tabs, and watcher cleanup must be verified before multi-tab goes live. These are Rust-only changes — low surface area, low risk, and must be done before frontend tab work exposes the bugs.

**Delivers:** A Rust backend where all state (including `RunningOp`) is keyed by repo path, `open_repo` is idempotent (skips commit walk if cache is already populated), and `close_repo` watcher cleanup is verified correct.

**Addresses:** Concurrent remote operations (two tabs can fetch/push independently without blocking each other)

**Avoids:** Pitfall 2 (RunningOp mutex), Pitfall 6 (watcher accumulation)

**Research flag:** Standard patterns — HashMap keying is already established in this codebase. Skip research-phase.

### Phase 2: Frontend Tab Architecture (RepoView Extraction + Tab State)

**Rationale:** The largest single refactor in v0.9 — extracting ~30 state variables and ~15 handler functions from `App.svelte` into `RepoView.svelte`. Doing this as a pure refactor first (zero behavior change) makes the diff reviewable and regressions catchable. Once extraction is done, wiring up multi-tab is additive rather than interleaved with structural changes.

**Delivers:** `RepoView.svelte` component (all current repo logic), `tab-state.svelte.ts` shared module, `repo-context.svelte.ts` per-tab factory for `remoteState`/`undoRedoState`, rewritten `TabBar.svelte` for multi-tab, updated `App.svelte` as thin shell, tab persistence via LazyStore (`open_tabs` replaces `open_repo`), full keyboard shortcuts (Cmd+T/W/1-9/Ctrl+Tab), dirty indicator on background tabs.

**Addresses:** Independent state per tab, tab bar UI, keyboard shortcuts, WelcomeScreen as new-tab page, tab persistence, dirty indicator.

**Avoids:** Pitfall 1 (singleton state leaks), Pitfall 3 (App.svelte monolith), Pitfall 4 (event listener cleanup), Pitfall 5 (LazyStore not tab-aware)

**Research flag:** Needs planning attention — the per-repo context scoping pattern for `remoteState`/`undoRedoState` needs a concrete design decision before coding starts. Specifically: whether to use Svelte's `setContext`/`getContext` or a factory function approach, and how this propagates through all 24 components.

### Phase 3: Tree View Data Layer (Pure Logic)

**Rationale:** Build and test `buildTree()` as a pure function with no UI dependencies. Unit tests validate correctness for edge cases (empty input, deeply nested paths, single-file repos, unicode paths) before any Svelte components are written. This phase has zero risk of UI regressions.

**Delivers:** `src/lib/file-tree.ts` with `buildTree<T>()`, path compression (merge single-child directory chains), sorting (directories first, then alpha), and `flattenTree()` for keyboard navigation. Full unit test coverage in `file-tree.test.ts`.

**Avoids:** Pitfall 7 (path splitting — always split on `/`, never OS path separator since git2 always returns forward-slash paths), Pitfall 9 (generic component boundary settled at the data layer design stage)

**Research flag:** Standard patterns — pure function trie construction is well-documented. Skip research-phase.

### Phase 4: Tree View UI Integration

**Rationale:** With the data layer in place and tested, wire the UI. Build `FileList.svelte` (flat/tree wrapper) and `TreeRow.svelte` (recursive renderer). Integrate into `StagingPanel` (unstaged, staged, conflicted) and `CommitDetail`, and add view mode toggle persisted in LazyStore.

**Delivers:** Tree view toggle visible in staging panel and commit detail, directory expand/collapse with chevrons, expand state preserved across status refreshes (`Set<string>` tracking), keyboard navigation (arrow keys), view mode persistence in LazyStore.

**Addresses:** All tree view table-stakes features.

**Avoids:** Pitfall 8 (expand state lost on refresh — designed in from the start), Pitfall 9 (single `FileTree` handles all contexts via generic interface)

**Research flag:** Standard patterns — recursive Svelte 5 self-import components are well-documented. Skip research-phase.

### Phase 5: Polish and Differentiators

**Rationale:** With core infrastructure solid, add the competitive differentiators that elevate UX. These are all additive and low-risk.

**Delivers:** Directory staging (sequential `stage_file` loop — no new backend command needed for v0.9), tab context menu via Tauri `Menu` API, middle-click tab close, file count badges on directory nodes, Expand All / Collapse All in file list header, duplicate repo detection (switching to existing tab instead of opening duplicate).

**Avoids:** Pitfall 10 (directory actions ambiguity — sequential calls with loading indicator, no atomicity risk for <100 files)

**Research flag:** Standard patterns. Skip research-phase.

### Phase Ordering Rationale

- Rust changes first because they unblock everything and are the safest to ship (backend-only, no visible UX changes until tabs exist)
- `RepoView` extraction as a pure refactor before adding tab switching — ensures the diff is auditable and regressions are immediately visible
- Tree data layer before tree UI — pure function testability reduces integration bugs in Phase 4
- Polish last — differentiators are valuable but not blocking

### Research Flags

Phases likely needing deeper investigation during planning:
- **Phase 2 (Frontend Tab Architecture):** The per-repo context scoping pattern for `remoteState`/`undoRedoState` needs a concrete design before coding. This decision propagates through all 24 components. Recommended: spike the `setContext`/`getContext` approach against the existing component tree to confirm no awkward prop-threading is required.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Rust Backend):** HashMap keying pattern already established in this codebase (`RepoState`, `CommitCache`, `WatcherState` all follow it). The `RunningOp` change is mechanical.
- **Phase 3 (Tree Data Layer):** Pure function trie construction is well-documented; unit tests validate correctness without architectural uncertainty.
- **Phase 4 (Tree View UI):** Recursive Svelte 5 self-import components are documented. Component structure is fully specified in ARCHITECTURE.md.
- **Phase 5 (Polish):** All features are additive with established patterns (Tauri Menu API, sequential IPC calls, existing tab infrastructure).

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Zero new dependencies confirmed by full codebase audit. Existing stack capabilities verified against both features. |
| Features | HIGH | Based on GitKraken, Fork, VS Code, Lazygit feature inventories. Priority ordering validated against the feature dependency graph. |
| Architecture | HIGH | Based on full codebase audit of all 21 commands, 24 components, and all Rust state structs. Component boundaries are concrete, not hypothetical. |
| Pitfalls | HIGH | All 10 pitfalls identified through direct code analysis — not inferred risks. Specific variable names, line ranges, and corruption scenarios documented. |

**Overall confidence:** HIGH

### Gaps to Address

- **Per-repo context scoping design:** The concrete mechanism for scoping `remoteState` and `undoRedoState` to individual tabs needs a design spike in Phase 2 planning. Both `setContext`/`getContext` and a factory constructor approach work technically; the choice affects how all child components import these modules. Resolve this before writing Phase 2 plans.

- **Keep-alive vs destroy/recreate:** ARCHITECTURE.md recommends destroy/recreate (`{#key}`) for memory efficiency; PITFALLS.md recommends keep-alive for better UX (preserved scroll position, instant switching). Recommendation: start with destroy/recreate (simpler, Rust cache makes re-mount fast), and switch to keep-alive in a later milestone if tab switching UX is judged unsatisfactory during verification. Decide in Phase 2 planning.

- **Watcher debounce for background tabs:** Increasing the background tab debounce interval from 300ms to 2000ms could meaningfully reduce CPU and fd pressure with many open tabs. This is an optimization worth validating during Phase 1 or Phase 5 rather than a blocking concern.

## Sources

### Primary (HIGH confidence)

- Trunk codebase audit: `state.rs`, `watcher.rs`, `remote.rs`, `repo.rs`, `App.svelte`, `TabBar.svelte`, `WelcomeScreen.svelte`, `store.ts`, `remote-state.svelte.ts`, `undo-redo.svelte.ts`, all 21 Tauri commands, all 24 Svelte components
- [Tauri 2 State Management docs](https://v2.tauri.app/develop/state-management/) — Mutex<HashMap> pattern, state management API
- [Tauri splittable tab discussion #6464](https://github.com/tauri-apps/tauri/discussions/6464) — Tauri maintainer confirms HTML/JS tabs; Spacedrive example
- [GitKraken Keyboard Shortcuts](https://help.gitkraken.com/gitkraken-desktop/keyboard-shortcuts/) — tab shortcuts (Cmd+T, Cmd+W, Cmd+1-9, Ctrl+Tab)
- [GitKraken Staging docs](https://help.gitkraken.com/gitkraken-desktop/staging/) — tree view toggle, folder-level staging
- [Recursive Svelte 5 components](https://scriptraccoon.dev/blog/recursive-svelte-components) — self-import pattern replaces `<svelte:self>`
- [Lazygit Tree View PR #1197](https://github.com/jesseduffield/lazygit/pull/1197) — path compression, status color aggregation, directory staging
- [PatternFly Tree View Design Guidelines](https://www.patternfly.org/components/tree-view/design-guidelines/) — expand/collapse, caret behavior, accessibility
- [Carbon Design System Tree View](https://carbondesignsystem.com/components/tree-view/usage/) — standard tree interaction, WAI-ARIA requirements

### Secondary (MEDIUM confidence)

- [Fork 1.0.69 Multi-window Support](https://fork.dev/blog/posts/fork-1.0.69/) — tab drag to new window feature (informs anti-feature decision for v0.9)
- [Fork Tab Dirty Indicator issue #515](https://github.com/fork-dev/Tracker/issues/515) — dot badge on tabs with uncommitted changes
- [Tauri multi-webview discussion #2975](https://github.com/tauri-apps/tauri/issues/2975) — experimental status, known panics on webview close
- [Svelte 5 shared state patterns](https://joyofcode.xyz/how-to-share-state-in-svelte-5) — module-level $state rune pattern and its limits in multi-context apps
- [@keenmate/svelte-treeview GitHub](https://github.com/KeenMate/svelte-treeview) — evaluated and rejected: FlexSearch dependency, overengineered for <500 file lists, own SCSS

### Tertiary (LOW confidence)

- [GitKraken tab performance feedback](https://feedback.gitkraken.com/suggestions/196509/improve-performance-when-switching-tabs) — user reports on tab switching latency; suggests warm cache mitigates perceived slowness with destroy/recreate
- [Zed Tree View Feature Request](https://github.com/zed-industries/zed/discussions/40052) — tree expand state persistence is an expected behavior, not a nice-to-have

---
*Research completed: 2026-03-23*
*Ready for roadmap: yes*
