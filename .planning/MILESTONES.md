# Milestones

## v0.8 Conflict & Rebase (Shipped: 2026-03-23)

**Phases completed:** 7 phases, 19 plans, 36 tasks

**Key accomplishments:**

- Tauri IPC layer for merge/rebase operation detection with git2 repo.state() and 5 CLI action commands (continue/abort/skip)
- Conflict detection UI with collapsible conflicted files section, color-coded merge/rebase operation banners, and Continue/Skip/Abort action buttons
- diff_conflicted backend command using git2 diff_tree_to_workdir to show conflict markers when clicking conflicted files
- Two Tauri commands (get_merge_sides, save_merge_result) extracting conflict content from git2 index stages and persisting resolved merge output with staging
- Pure TypeScript merge parser with three-way conflict region detection, Set-based selection state, real-time output computation, and navigation helpers
- Three-panel MergeEditor.svelte with synchronized scroll, per-hunk/per-line toggle selection, editable output textarea, and Prev/Next conflict navigation
- MergeEditor routing in App.svelte for conflicted files and Take All Current/Incoming context menu in StagingPanel for quick resolution
- Wired output textarea into three-way scroll sync via panelRefs[2] binding and handleScroll(2) handler
- handleFileResolved queries get_status after resolution to auto-open next conflicted file or return to CommitGraph when none remain
- Fix manualEdit override in toggle/take handlers and wire StagingPanel context menu resolution to App handleFileResolved
- Merge context menu items wired to all 6 branch surfaces using Tauri native menus, with silent success and error-only toasts
- Rebase context menu items on all 6 branch surfaces (pills, overflow refs, sidebar) with toast-free success handling
- get_rebase_todo/get_fork_point backend commands, validateRebasePlan with 9 test cases, CSS rebase tokens, InputDialog configurable labels, LazyStore rebase column persistence
- RebaseEditor Svelte component with 5-column layout, native DnD reordering, P/S/R/D keyboard shortcuts, color-coded action dropdowns, inline validation errors, and LazyStore-persisted column widths/visibility
- Interactive rebase execution engine with GIT_SEQUENCE_EDITOR for custom todo and GIT_EDITOR shell script for file-based IPC message editing
- RebaseEditor wired into App center pane with context menu entry points in CommitGraph (commit, pill, overflow ref menus) and BranchSidebar, plus reword/squash message dialog via event listener
- Squash message pre-editing with combined messages, corrected column order (Message before SHA), and stable squash arrow positioning
- Skip Commit button added to StagingPanel inline rebase form with silent-skip behavior aligned across both StagingPanel and OperationBanner
- Removed orphaned diff_conflicted command/tests, fixed rebaseBaseName branch resolution via resolve_ref IPC, cleaned dead InputDialog import, narrowed diffKind type

---

## v0.7 Hunk Staging & Search (Shipped: 2026-03-19)

**Phases completed:** 5 phases, 8 plans, 0 tasks

**Key accomplishments:**

- (none recorded)

---

## v0.6 UI Polish & Core Ops (Shipped: 2026-03-16)

**Phases:** 6 (incl. 27.1) | **Plans:** 16 | **Commits:** ~129 | **Timeline:** 2 days
**Git range:** 9ff6c95 (feat(27-01)) → b1ef642 (fix: root commit branch tip + scroll)

**Delivered:** Lucide icon set replacing Unicode symbols across all components, toast notification system for operation feedback, discard/delete/rename/reset destructive operations with confirmation dialogs, three-way commit/amend/stash selector, graph polish (padding, column shrink, sticky dots), sidebar ref navigation, unified title bar, and two bug fixes from TODO.md.

**Key accomplishments:**

1. Icon system: @lucide/svelte replacing all Unicode symbols in Toolbar, FileRow, StagingPanel, CommitForm, BranchSidebar, TabBar, and SVG overlay pills
2. Toast notification system with Svelte 5 $state store, auto-dismiss, fly transition, and per-kind styling (error/success)
3. Destructive operations: discard file/all (git2 checkout + fs::remove_file), delete branch/tag, rename branch, reset — all with confirmation dialogs and toast feedback
4. Three-way commit/amend/stash selector replacing amend checkbox; stash mode auto-populates subject as stash name
5. Graph CSS polish: viewport padding, column overflow-hidden with fixed minimum, sidebar click scrolls to ref commit with resolve_ref backend command
6. Unified title bar: decorations:false + drag region + traffic light padding; right pane auto-opens on commit/ref click

---

## v0.5 Graph Overlay (Shipped: 2026-03-15)

**Phases:** 7 | **Plans:** 12 | **Commits:** 111 | **Timeline:** 2 days
**LOC:** ~6,038 Rust / ~4,417 Svelte / ~1,102 TypeScript / ~1,463 Tests
**Git range:** 1144693 (feat(20-01)) → 85af781 (test: complete UAT)

**Delivered:** Single SVG overlay architecture replacing per-row viewBox-clipped SVGs, with cubic bezier curve rendering, TypeScript active lanes transformation with edge coalescing, virtualized element filtering, Canvas-measured SVG ref pills with hover expansion, and full interaction preservation.

**Key accomplishments:**

1. Single SVG overlay spanning full graph height with native scroll sync and pointer-events passthrough — zero JS scroll synchronization
2. TypeScript Active Lanes transformation with edge coalescing reducing edge count from O(commits x lanes) to O(lanes + merge_edges)
3. Cubic bezier curve rendering with adaptive corner radius replacing Manhattan routing, three-layer z-ordered SVG (rails → edges → dots)
4. Virtualized SVG element filtering with O(1) viewport range-intersection via minRow/maxRow metadata
5. SVG ref pills with Canvas-based text measurement, capsule shapes, lane colors, connector lines, remote dimming, and overflow +N hover expansion
6. Clean integration: unified constants (16px lanes, 36px rows), ~1,000 lines dead code removed, all click/context-menu interactions preserved

---

## v0.3 Actions (Shipped: 2026-03-12)

**Phases:** 4 | **Plans:** 14 | **Commits:** 88 | **Timeline:** 3 days
**LOC:** ~5,009 Rust / ~3,553 Svelte / ~345 TypeScript
**Git range:** c30394be (start milestone) → 100a1b90 (merge TabBar/Toolbar)

**Delivered:** Full stash management, commit row context menu with cherry-pick/revert, remote push/pull/fetch with progress streaming, quick actions toolbar with undo/redo, and branch ahead/behind tracking.

**Key accomplishments:**

1. Stash create/pop/apply/drop via git2 with graph-integrated synthetic stash rows (square dots, dashed connectors) and right-click context menu
2. Commit context menu with copy SHA/message, checkout, create branch/tag, cherry-pick, and revert (merge commits disabled)
3. Remote fetch/pull/push via git CLI subprocess with per-line progress streaming and structured auth/rejection error taxonomy
4. Quick actions toolbar (Pull, Push, Branch, Stash, Pop, Undo, Redo) merged into single top bar
5. Branch ahead/behind counts computed inside list_refs to avoid extra IPC, auto-refreshed after remote ops
6. Undo/redo commit with isUndoing/isRedoing race-condition guards and clearRedoStack moved to user-initiated sites

---

## v0.2 Commit Graph (Shipped: 2026-03-10)

**Phases:** 4 | **Plans:** 9 | **Commits:** 76 | **Timeline:** 2 days
**LOC:** ~3,344 Rust / ~2,458 Svelte / ~290 TypeScript
**Git range:** 80a714c (test(07-01)) → 2c18aa2 (docs(phase-07))

**Delivered:** GitKraken-quality commit graph with continuous vertical lane rails, Manhattan-routed merge/fork edges, vivid 8-color palette, hollow merge dots, WIP dashed connector, lane-colored ref pills, resizable 6-column layout, and column visibility toggles.

**Key accomplishments:**

1. Hardened lane algorithm with ghost lane fix, octopus merge protection, max_columns tracking, and deterministic branch color counter
2. Three-layer SVG lane rendering with vivid 8-color palette, continuous vertical rails, and Manhattan-routed merge/fork edges
3. Merge commits display as hollow circles, WIP row integrated into virtual list with dashed connector to HEAD
4. Lane-colored ref pills with remote dimming and horizontal connector lines to commit dots
5. Spreadsheet-style 6-column resizable layout with LazyStore-persisted column widths
6. Header right-click context menu with per-column visibility toggles via native Tauri Menu API

---

## v0.1 MVP (Shipped: 2026-03-09)

**Phases:** 6 | **Plans:** 27 (26 complete) | **Commits:** 155 | **Timeline:** 7 days
**LOC:** ~53,990 Rust / ~2,043 Svelte / ~193 TypeScript
**Git range:** 5e8a251 (initial) → 80d0151 (UAT re-test)

**Delivered:** A native desktop Git GUI where a developer can open any repository, browse its full commit history as a visual lane graph, manage branches, stage files, create/amend commits, and inspect diffs — all without touching the terminal.

**Key accomplishments:**

1. Vite+Svelte SPA with Tailwind v4 dark theme and shared Rust/TypeScript primitives
2. Visual commit graph with Rust lane algorithm (O(n)), inline SVG per row, virtual scrolling with 200-commit pagination
3. Branch sidebar with checkout, dirty-workdir error handling, and client-side search
4. Working tree staging panel with real-time file status, whole-file stage/unstage, and filesystem watcher auto-refresh
5. Commit creation and amendment with subject+body form, validation, and immediate graph refresh
6. Unified diff display for workdir/staged/commit diffs with commit metadata header

### Known Gaps

- **GRAPH-04**: Merge commits not visually distinct in CommitRow.svelte (Rust DTO carries `is_merge` correctly, frontend never reads it)
- **DIFF-01–04**: Phase 06 VERIFICATION.md never created (code-complete and wiring-verified, lacks formal report)
- **Plan 02-09**: No SUMMARY.md (active_lanes[0] None initialization fix — plan created, not executed)
- **Checkout → StagingPanel**: Non-deterministic refresh after checkout/create-branch (relies on watcher, not explicit event emit)

---
