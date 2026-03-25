# Trunk

## What This Is

Trunk is a fast, native, cross-platform desktop Git GUI built with Tauri 2 + Svelte 5 + Rust. It provides a GitKraken-quality visual commit graph with cubic bezier curves and SVG overlay architecture, branch management, staging workflow, remote operations, and file diffs — without the performance penalties or licensing costs of existing tools like GitKraken or Fork.

## Core Value

A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.

## Requirements

### Validated

- ✓ Migrate scaffold from SvelteKit to plain Vite+Svelte — v0.1
- ✓ Open a Git repository via native file dialog — v0.1
- ✓ Display paginated commit history with visual lane graph — v0.1
- ✓ List branches, remote branches, tags, and stashes in sidebar — v0.1
- ✓ Show working tree status (unstaged and staged files) — v0.1
- ✓ Stage and unstage individual files (whole-file only) — v0.1
- ✓ Create commits with message and optional description — v0.1
- ✓ Show file diffs (workdir, staged, and commit diffs) — v0.1
- ✓ Show full commit detail (metadata + diff) when a commit is clicked — v0.1
- ✓ Checkout branches with dirty-workdir error handling — v0.1
- ✓ Watch filesystem and auto-refresh status on external changes — v0.1
- ✓ GitKraken-quality commit graph with lane rendering — v0.2
- ✓ Continuous vertical colored lines per branch with lane packing — v0.2
- ✓ Manhattan-routed merge/fork edges with vivid 8-color palette — v0.2
- ✓ Merge commit visual distinction (hollow dots) and WIP dashed connector — v0.2
- ✓ Lane-colored ref pills and resizable 6-column layout — v0.2
- ✓ Quick actions toolbar (Pull, Push, Branch, Stash, Pop, Undo, Redo) — v0.3
- ✓ Push / Pull / Fetch with SSH/HTTPS auth + ahead/behind counts in sidebar — v0.3
- ✓ Stash create/pop/apply/drop — v0.3
- ✓ Commit row right-click context menu (copy SHA/message, checkout, branch, tag, cherry-pick, revert) — v0.3
- ✓ Undo last commit (soft reset) and Redo (re-commit with original message) — v0.3
- ✓ Click and context menu interactions preserved through SVG overlay architecture — v0.5
- ✓ Selected commit row persistent visual highlight — v0.5
- ✓ Stash-specific context menu routing (Pop/Apply/Drop) in graph — v0.5
- ✓ Single SVG overlay spanning full graph height with native scroll sync — v0.5
- ✓ TypeScript Active Lanes transformation with edge coalescing — v0.5
- ✓ Cubic bezier curves replacing Manhattan routing — v0.5
- ✓ Virtualized SVG element filtering with DOM node cap — v0.5
- ✓ SVG ref pills with Canvas text measurement, lane colors, connectors, dimming, overflow badges — v0.5
- ✓ Three-layer z-ordered SVG rendering (rails → edges → dots) — v0.5
- ✓ Lucide SVG icons replacing Unicode symbols across all components — v0.6
- ✓ Toast notification system with auto-dismiss and per-kind styling — v0.6
- ✓ Discard file/all changes (git2 checkout + fs::remove_file) with confirmation — v0.6
- ✓ Delete local branch/tag and rename branch via context menu — v0.6
- ✓ Reset current branch to any commit (soft/mixed/hard) via context menu — v0.6
- ✓ Three-way commit/amend/stash selector replacing amend checkbox — v0.6
- ✓ Colored stage (green) / unstage (red) buttons, equal-height file lists — v0.6
- ✓ Commit graph padding, column shrink with sticky dots, sidebar ref navigation — v0.6
- ✓ Unified title bar (decorations:false + drag region), right pane auto-opens — v0.6
- ✓ Lucide icons on all SVG overlay ref pill types — v0.6

- ✓ Stage/unstage individual hunks within a file diff — v0.7
- ✓ Discard individual hunks from working tree with confirmation — v0.7
- ✓ Context-aware hunk actions (Stage Hunk for unstaged, Unstage Hunk for staged, none for commit diffs) — v0.7
- ✓ Binary file diffs show no hunk action buttons — v0.7
- ✓ Diff refreshes immediately after hunk operations with updated boundaries — v0.7
- ✓ Keyboard navigation between hunks with [/] shortcuts — v0.7
- ✓ Select and stage/unstage individual lines within a diff hunk — v0.7
- ✓ Search commit graph for hashes, messages, refs, and author with Cmd+F — v0.7
- ✓ Search overlay with live match count, debounced 200ms — v0.7
- ✓ Match highlighting (amber current, yellow others) with non-match dimming — v0.7
- ✓ Enter/Shift+Enter navigation with auto-scroll and wrap-around — v0.7
- ✓ Escape closes search, clears highlights, restores full opacity — v0.7

- ✓ Conflicted file detection and display in staging panel — v0.8
- ✓ Operation state banner (merge/rebase in progress) with Continue/Abort/Skip — v0.8
- ✓ Read-only diff for conflicted files showing conflict markers — v0.8
- ✓ Three-panel merge editor (current/incoming/output) with per-hunk/per-line selection, editable output, sync scroll — v0.8
- ✓ Take All Current / Take All Incoming quick resolution (toolbar + context menu) — v0.8
- ✓ Conflict navigation (Prev/Next) and "Save and Mark Resolved" auto-advance — v0.8
- ✓ Merge via branch context menu (sidebar + graph pill + overflow ref) — v0.8
- ✓ Fast-forward and non-conflicting merge auto-handling — v0.8
- ✓ Rebase initiation via commit and branch context menus — v0.8
- ✓ Mid-rebase conflict resolution with pause/continue/skip/abort — v0.8
- ✓ Interactive rebase editor with Pick/Squash/Reword/Drop actions and drag-and-drop reordering — v0.8
- ✓ Squash message pre-editing with combined messages — v0.8
- ✓ Reword pauses with message editing dialog — v0.8
- ✓ Skip conflicting commit during rebase from inline UI — v0.8
- ✓ Tech debt cleanup: removed orphaned diff_conflicted command, fixed rebaseBaseName lookup, cleaned dead imports — v0.8

- ✓ Multi-tab: independent repo tabs with keep-alive rendering, Cmd+T/W/1-9 shortcuts, dirty indicators, persisted state — v0.9
- ✓ Tab interactions: context menu (Close Others/All, Copy Path), middle-click close, duplicate detection, drag-and-drop reorder — v0.9
- ✓ Tree view: directory tree toggle in staging panel, commit diffs, and merge editor with keyboard navigation — v0.9
- ✓ Tree data layer: trie-based flat-to-tree with path compression, directory-before-file sorting — v0.9
- ✓ Tree features: directory staging, file count badges, Expand All / Collapse All, directory right-click context menus — v0.9
- ✓ Backend: per-repo remote operation isolation via HashMap-keyed RunningOp — v0.9

### Active

(No active requirements — next milestone not yet defined)

### Planned

- **v0.10**: CI/CD & Releases — GitHub Actions CI, cross-platform release publishing (macOS, Linux, Windows)
- **v1.0**: Infrastructure — E2E test harness (GOOS-style), performance benchmarks

### Out of Scope

- Settings/preferences UI — deferred to v1.0
- Commit signing — deferred to v1.0
- Auto-updates — deferred to v1.0
- Mobile / web versions — desktop only
- Tab drag to new OS window — requires Tauri multi-window, state serialization
- Workspace/group management — GitKraken-style cloud team feature; overkill for personal use
- Virtual scrolling for file tree — staging file counts rarely exceed 500

## Context

- **Stack**: Tauri 2 + Svelte 5 (Vite SPA, not SvelteKit) + Rust with `git2` crate (libgit2 bindings)
- **Current state**: Shipped v0.9 with 49 phases across 9 milestones. ~13,400 LOC TypeScript/Svelte, ~9,400 LOC Rust.
- **Architecture**: Svelte UI communicates with Rust backend via Tauri `invoke` (commands) and `listen` (events). Rust holds `RepoState` (path-keyed PathBuf registry), `CommitCache` (cached GraphResult with max_columns), `WatcherState` (filesystem watchers), and `RunningOp` (active remote process PID) in managed state.
- **Remote ops**: `git2` for all local read/write; git CLI subprocess for remote operations (fetch/pull/push) and cherry-pick/revert with `GIT_TERMINAL_PROMPT=0` + `GIT_SSH_COMMAND=ssh -o BatchMode=yes`
- **Graph rendering (v0.5)**: Single SVG overlay spanning full graph height inside virtual list scroll container. Rust lane algorithm (O(n), ~5ms for 10k commits) outputs GraphCommit[]; TypeScript Active Lanes transformation computes global grid coordinates with edge coalescing. Cubic bezier curves for cross-lane connections, continuous vertical rails for same-lane. Three-layer z-ordered `<g>` groups (rails → edges → dots). Virtualized element filtering with O(1) range-intersection. SVG ref pills with Canvas text measurement and hover expansion.
- **Graph UI**: 6-column resizable layout (ref, graph, message, author, date, SHA) with LazyStore-persisted widths, native Tauri context menu for column visibility, lane-colored ref pills
- **Patterns established**: inner-fn pattern for testable Tauri commands, safeInvoke<T> for all IPC, sequence counter for stale async guard, cache-repopulate-before-emit for mutation commands, LazyStore for UI state persistence, sentinel oid ('__wip__', '__stash_N__') for synthetic virtual list items, $derived.by() for imperative reactive computations, shared $state rune modules for cross-component communication, InputDialog $state dialogConfig pattern
- **Motivation**: Personal learning project (Tauri/Rust/Svelte) + building a better tool for personal use + eventual open source release

## Constraints

- **Tech stack**: Tauri 2 + Svelte 5 + Rust — already chosen, non-negotiable
- **Frontend framework**: Plain Vite+Svelte (not SvelteKit) — desktop app has no routing/SSR needs
- **Git backend**: `git2 = "0.19"` for all local operations
- **Filesystem watching**: `notify = "7"` + `notify-debouncer-mini = "0.5"` with 300ms debounce
- **Styling**: Tailwind CSS v4 + forced dark theme via CSS custom properties
- **Graph**: Virtual scrolling — render only visible rows + dynamic buffer; ~40 DOM nodes for any history size

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Vite+Svelte over SvelteKit | Desktop app has no routing or SSR needs; SvelteKit adds unnecessary complexity | ✓ Good — eliminated entire class of build issues |
| git2 for reads/writes, git CLI for remotes (future) | libgit2 has unreliable SSH/HTTPS auth; all major Tauri git clients shell out for push/pull | ✓ Good — all local ops work reliably |
| Graph lane algorithm in Rust | O(n), avoids serializing intermediate data, doesn't block JS thread | ✓ Good — ~5ms for 10k commits, required 3 gap-closure iterations |
| Inline SVG per row (not Canvas) | Free scrolling, text selection, accessibility; simple enough geometry | ✓ Good — works well with virtual list |
| Virtual scrolling with dynamic buffer | Constant DOM nodes regardless of history size | ✓ Good — smooth performance on large repos |
| `dirty_workdir` error code for checkout | Structured error codes let frontend show contextual UI without string matching | ✓ Good — clean error handling pattern |
| RepoState stores PathBuf only | git2::Repository is not Sync; open fresh per command in spawn_blocking | ✓ Good — avoids lifetime issues, minimal overhead |
| inner-fn pattern for Tauri commands | Separates Tauri state from pure git logic, enables direct unit testing | ✓ Good — all commands testable without Tauri runtime |
| Cache repopulate before emit | Prevents CommitGraph remount from racing a cleared cache | ✓ Good — eliminated race conditions in commit/amend flows |
| DiffPanel replaces CommitGraph (toggle not split) | User feedback found split pane confusing | ✓ Good — simpler UX |

| Lanes removed, dots only for v0.1 | v0.1 lane rendering had visual bugs; simpler to ship dots and revisit with dedicated focus | ✓ Good — shipped v0.1 clean, dedicated v0.2 milestone for graph |
| GraphResult wrapper return type | walk_commits returns struct with commits + max_columns metadata instead of bare Vec | ✓ Good — enables consistent SVG widths, clean separation |
| GraphResponse IPC struct at command boundary | Separate from internal GraphResult; slices commits for pagination | ✓ Good — clean internal/external type separation |
| Branch color counter with deterministic color_index | HEAD gets 0, new branches get incrementing colors, freed columns remove entries | ✓ Good — enables consistent per-branch coloring in frontend |
| Three-layer SVG rendering (rails -> edges -> dots) | Correct z-stacking: rails behind edges behind dots; each layer is a separate SVG group | ✓ Good — clean visual layering, easy to add new element types |
| Manhattan routing for merge/fork edges | Horizontal + arc + vertical path segments with 6px corner radius; simpler than full bezier | ✓ Good — clean visual appearance, straightforward path math |
| Vivid 8-color dark-theme palette | GitHub-dark-inspired high-contrast colors replacing low-contrast originals | ✓ Good — all colors readable against #0d1117 |
| WIP sentinel oid ('__wip__') | Synthetic virtual list item rather than extending GraphCommit type | ✓ Good — keeps TypeScript type aligned with Rust backend struct |
| LazyStore for UI state persistence | Column widths and visibility persisted via Tauri store with lazy load | ✓ Good — consistent pattern for all UI state |
| Native Tauri Menu API over custom Svelte component | Replaced HeaderContextMenu.svelte with @tauri-apps/api/menu | ✓ Good — native look and feel, simpler code |
| git CLI for cherry-pick/revert (not git2) | Avoids reimplementing conflict state machine; consistent with remote ops pattern | ✓ Good — conflict detection via exit code |
| Two-pass stash OID resolution | stash_foreach collects into Vec, parent resolution after foreach releases mutable borrow | ✓ Good — clean borrow checker compliance |
| Stash sentinel OID (__stash_N__) | Extends WIP sentinel pattern for synthetic graph rows; square dots differentiate from commits | ✓ Good — reuses established pattern |
| Store child PID (u32) not tokio::process::Child | Child is !Sync; storing PID in RunningOp enables kill from any thread | ✓ Good — clean async cancellation |
| Ahead/behind inside list_refs_inner | Compute in existing map closure to avoid extra IPC round-trip | ✓ Good — no performance impact on sidebar refresh |
| isUndoing/isRedoing flags for redo stack | Prevents clearRedoStack during undo/redo-triggered repo-changed events | ✓ Good — eliminated race condition |
| Shared $state rune module (remote-state.svelte.ts) | Cross-component state for StatusBar/Toolbar without props/bindings | ✓ Good — clean Svelte 5 pattern |
| Unicode symbols for toolbar icons | Simple, no SVG assets needed, consistent with dark theme | ✓ Good — minimal complexity |
| Reverse "no full-height SVG" (v0.4 out-of-scope) | v0.4 per-row viewBox clipping worked but limited; single overlay enables continuous bezier paths, eliminates row-boundary seams | ✓ Good — single overlay cleaner than per-row, native scroll sync eliminated JS sync code |
| Rust lane algorithm stays, TS transformation added | Rust O(n) algorithm is proven (~5ms/10k commits); TS layer transforms output into global grid coords for SVG rendering | ✓ Good — edge coalescing reduces O(commits x lanes) to O(lanes + merge_edges) |
| Canvas measureText for SVG ref pills | OffscreenCanvas for DOM-free text measurement with injectable mock for testing | ✓ Good — deterministic tests, accurate text sizing |
| SVG ref pills with HTML hover overlay | SVG handles static pills, HTML sibling handles hover expansion for reliable multi-ref display | ✓ Good — avoids SVG text layout complexity |
| git2 apply API for hunk staging | Single-hunk patch extraction via hunk_callback counter, applied to Index/WorkDir | ✓ Good — precise hunk targeting, no shell dependency |
| Partial patch text construction for line staging | Build new diff text from selected lines, recalculate @@ header counts | ✓ Good — handles edge cases (no-newline-at-EOF, context lines) |
| Capture-phase Cmd+F handler | `{ capture: true }` on window keydown to intercept before WKWebView native find | ✓ Good — suppresses native find bar on macOS |
| Pure presentation SearchBar | SearchBar is stateless — parent (CommitGraph) owns all search state and IPC | ✓ Good — clean separation, easy to test and restyle |
| SVG global dimming over per-element | Apply opacity to entire SVG overlay vs tracking per-element match state | ✓ Good — single style change, rails/edges span multiple rows making per-element impractical |

| File-based IPC for interactive rebase | Shell script touches signal file, Rust polls, frontend writes response — avoids stdin piping and works with GIT_EDITOR | ✓ Good — reliable cross-process communication |
| git2 Index::conflicts() iterator for merge sides | git2 0.19 only exposes iterator API, not conflict_get() | ✓ Good — clean extraction of ours/theirs/base content |
| Sequential scan for three-way merge parsing | Simple sync-point search instead of full LCS/Myers diff | ✓ Good — adequate for conflict region detection, minimal complexity |
| No success toast on merge/rebase/skip | Graph refresh via repo-changed event is sufficient feedback | ✓ Good — consistent silent-success pattern across all operations |
| Destroy/recreate over keep-alive for tab switching | Simpler than caching, Rust cache makes remount fast | ✓ Revised — ended up using keep-alive (display:contents/none) for zero-cost hidden tabs |
| Per-tab state via factory functions | $state() factories replacing global singletons for remote/undo-redo state isolation | ✓ Good — backward-compat singleton aliases kept consumers compiling during migration |
| Trie-based flat-to-tree algorithm | O(n) conversion with path compression; compression guard checks child type === directory | ✓ Good — 19 TDD tests, clean separation of data/UI layers |
| SortableJS for drag reorder | Same library already used in RebaseEditor; forceFallback:true for cross-platform | ✓ Good — {#key} wrapper caused bug (orphaned Sortable), fixed by removing it |
| Dynamic imports for Tauri menu/dialog | @tauri-apps/api/menu and plugin-dialog loaded on demand, not at module level | ✓ Good — reduces initial bundle, matches existing pattern |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-25 after v0.9 milestone (Multi-tab & Tree View)*
