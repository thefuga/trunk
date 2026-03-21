# Phase 41: Interactive Rebase Editor - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can rewrite commit history through a visual interactive rebase editor. Right-click a commit or branch to open an editor (replacing center pane) with a commit list showing Pick/Squash/Reword/Drop actions, drag-and-drop reordering, keyboard shortcuts, validation, and execution with mid-conflict resolution via the existing merge editor.

</domain>

<decisions>
## Implementation Decisions

### Editor container & trigger
- **Replaces center pane** — editor takes over the CommitGraph area (same pattern as MergeEditor). Sidebar and staging panel remain visible and interactive
- **Entry points**: commit right-click ("Interactive Rebase") AND branch context menus (sidebar + graph pills). For branch menus, auto-detect fork point as base commit
- **Commit scope**: right-click commit X → rebase all commits from X (exclusive) up to HEAD. Standard `git rebase -i` behavior — clicked commit is the base
- **No isolation** — sidebar and staging remain fully interactive while the editor is open

### Commit row design — column layout
- **Reuses CommitGraph column system** — resizable columns, right-click header to toggle visibility, LazyStore-persisted widths/visibility
- **Columns (left to right)**: Action | SHA | Message | Author | Date
- **All columns visible by default** — user can hide any except Action (always visible)
- **No graph column, no ref column** — not applicable in rebase editor
- **Same row height** as CommitGraph rows — compact single-line
- **Action column**: dropdown select per row with Pick/Squash/Reword/Drop options
- **Keyboard shortcuts**: P=Pick, S=Squash, R=Reword, D=Drop on focused row

### Drag-and-drop reordering
- **Entire row is draggable** — grab cursor on hover, no dedicated grip column
- **Row swap animation** — rows animate and swap positions in real-time as you drag over them

### Drop action visual
- **Dropped commits visible but dimmed** — reduced opacity + strikethrough text. User can change action back. GitKraken approach

### Toolbar buttons
- **Start Rebase** — validates plan, executes, closes editor
- **Cancel** — closes editor with no changes
- **Reset** — restores all commits to original Pick state and order

### Reword & Squash message editing
- **During execution** — messages are written when git pauses at each reword/squash commit, not pre-configured
- **Squash pre-fill**: concatenated messages of all squashed commits (git default), user edits to final form

### Execution feedback
- **Editor closes on Start** — returns to graph view immediately. Existing OperationBanner shows rebase progress ("2/5")
- **Conflict handling**: merge editor opens via existing Phase 37-38 infrastructure. After resolving + Continue, stays in normal graph flow
- **Reword/squash pauses**: message dialog pops over normal graph view (editor does NOT reopen)
- **No success toast** — graph refresh shows rewritten history (consistent with Phases 39-40)
- **Error toast** for failures, consistent with existing pattern

### Validation
- **Validate with inline errors before execution** — block Start Rebase for invalid states:
  - Can't squash first commit
  - Can't have all commits dropped
  - Other git-invalid configurations
- Show inline error messages near the problematic rows

### Claude's Discretion
- Message editor implementation (InputDialog reuse vs inline approach)
- Exact validation rules beyond the two specified
- Loading state during rebase execution
- How to detect fork point for branch menu entry point
- Drag-and-drop implementation approach (native HTML5 vs library)
- Exact dropdown styling and color coding for actions
- How to handle detached HEAD (hide menu items, like merge/rebase pattern)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — REB-03 (interactive rebase via commit right-click), IREB-01 through IREB-07 (editor features, drag reorder, keyboard shortcuts, validation, reword, squash)

### Prior phase context
- `.planning/phases/37-conflict-detection-operation-state/37-CONTEXT.md` — Operation banner design, abort confirmation pattern, conflict section layout
- `.planning/phases/38-merge-editor/38-CONTEXT.md` — Center pane replacement pattern (MergeEditor), merge editor placement, resolution flow
- `.planning/phases/39-merge-workflow/39-CONTEXT.md` — Silent success pattern (no toast), context menu wiring across all surfaces
- `.planning/phases/40-rebase-workflow/40-CONTEXT.md` — Rebase context menu pattern, no confirmation for initiation, backend commands already implemented

### Existing rebase backend
- `src-tauri/src/commands/operation_state.rs` — `rebase_branch_inner()` (template for interactive rebase), `rebase_continue/skip/abort` commands, all registered in lib.rs

### Column system to replicate
- `src/components/CommitGraph.svelte` — Column header with right-click visibility toggle, resizable drag handles, LazyStore-persisted widths/visibility. Reuse this pattern for the editor's Action|SHA|Message|Author|Date columns

### Dialog patterns
- `src/components/InputDialog.svelte` — Modal dialog with multiline textarea, keyboard shortcuts, auto-focus. Candidate for reword/squash message editor

### Center pane replacement pattern
- `src/App.svelte` — How MergeEditor replaces CommitGraph in center pane (conditional rendering based on app state)

### State management
- `src/lib/toast.svelte.ts` — Module-level $state pattern for cross-component reactive state
- `src/lib/undo-redo.svelte.ts` — $state rune module pattern

### Operation state UI
- `src/components/OperationBanner.svelte` — Rebase progress display ("2/5"), Continue/Skip/Abort buttons
- `src/components/StagingPanel.svelte` — Operation state integration point

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **CommitGraph column system**: Resizable headers, visibility toggle, LazyStore persistence — replicate for editor columns
- **InputDialog**: Modal with multiline textarea — candidate for reword/squash message editing during execution
- **OperationBanner**: Already handles rebase progress display and Continue/Skip/Abort
- **safeInvoke<T>**: Type-safe IPC wrapper for all Tauri commands
- **Tauri Menu API**: Native context menus for commit and branch right-click
- **MergeEditor center-pane pattern**: App.svelte conditional rendering to swap CommitGraph with editor
- **LazyStore**: UI state persistence for column widths and visibility

### Established Patterns
- **Center pane swap**: MergeEditor replaces CommitGraph via conditional in App.svelte — same pattern for rebase editor
- **Context menu**: `Menu.new({ items: [...] })` → `menu.popup()` via `@tauri-apps/api/menu`
- **Git CLI subprocess**: `std::process::Command::new("git")` with `GIT_TERMINAL_PROMPT=0` and `GIT_EDITOR=true`
- **Cache-repopulate-before-emit**: After rebase, refresh CommitCache then emit `repo-changed`
- **$state rune modules**: Cross-component state without props drilling

### Integration Points
- **App.svelte**: Add rebase editor as third center-pane option (CommitGraph / MergeEditor / RebaseEditor)
- **CommitGraph.svelte**: Add "Interactive Rebase" to commit context menu
- **BranchSidebar.svelte**: Add "Interactive Rebase" to local branch menus (auto-detect fork point)
- **CommitGraph pill menus**: Add to local/remote pill and overflow ref menus
- **New Rust command**: `get_rebase_todo(path, base_oid)` to list commits between base and HEAD
- **New Rust command**: `start_interactive_rebase(path, base_oid, todo_list)` to execute with custom todo

</code_context>

<specifics>
## Specific Ideas

- Column layout mirrors CommitGraph exactly — same header behavior, same resize handles, same LazyStore persistence pattern, same row height
- Drag-and-drop with row swap animation (rows animate into new position as you drag over them)
- Editor closes immediately on Start — OperationBanner takes over for progress and pause handling
- Fork point auto-detection for branch menu entry enables "Interactive Rebase" without manually picking a base commit

</specifics>

<deferred>
## Deferred Ideas

- IREB-EX-01: Fixup action (like Squash but discards commit message) — Future requirements
- IREB-EX-02: Edit action (pause rebase at commit to amend) — Future requirements
- IREB-EX-03: Multi-commit selection for bulk action assignment — Future requirements

</deferred>

---

*Phase: 41-interactive-rebase-editor*
*Context gathered: 2026-03-21*
