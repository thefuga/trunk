# Phase 42: Rebase Skip in Inline UI - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a Skip button to the StagingPanel's inline rebase form so users can skip a conflicting commit during rebase without using the OperationBanner. The OperationBanner already has Skip wired — this phase adds it to the inline rebase UI (the commit message form at the bottom of StagingPanel) and aligns feedback behavior with Phase 40's silent pattern.

</domain>

<decisions>
## Implementation Decisions

### Button placement & sizing
- Skip button goes **between Continue and Abort**: Continue Rebase | Skip Commit | Abort Rebase
- Flex ratios: Continue flex:3, Skip flex:1, Abort flex:2 (Skip is the narrowest — secondary action)
- Label: **"Skip Commit"** — explicit about what's being skipped
- Same height as Continue/Abort (34px)

### Button styling
- Reuse existing **--color-btn-skip-bg, --color-btn-skip, --color-btn-skip-border** CSS custom properties
- Consistent with OperationBanner's Skip button styling
- Same font-size (12px) and font-weight (600) as Continue/Abort

### Skip behavior
- Skip is **always enabled** during rebase — not gated on conflict state
- Only disabled when `rebaseLoading` is true (same as Abort)
- No confirmation dialog (Phase 37 decision: Skip is non-destructive forward action)

### Skip feedback
- **Silent — no success toast** for Skip in StagingPanel (Phase 40 pattern: no success toasts for rebase operations)
- **Also remove** the existing "Commit skipped" toast from OperationBanner's `handleSkip()` for consistency
- Error toast on failure (consistent with all rebase actions)
- Graph refresh + banner progress update provide sufficient visual feedback

### Claude's Discretion
- Whether `skipRebase()` function in StagingPanel calls `rebase_skip` directly or delegates to OperationBanner
- Loading state management (can share `rebaseLoading` flag)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — REB-06 (skip conflicting commit during rebase)

### Prior phase context
- `.planning/phases/37-conflict-detection-operation-state/37-CONTEXT.md` — Banner button behavior: Skip needs no confirmation, Abort requires confirmation
- `.planning/phases/40-rebase-workflow/40-CONTEXT.md` — Silent success pattern (no toast on rebase success), rebase feedback decisions

### Existing implementation
- `src/components/StagingPanel.svelte` — Inline rebase form (lines 730-815): Continue/Abort buttons, `continueRebase()` and `abortRebase()` handlers, `rebaseLoading` state
- `src/components/OperationBanner.svelte` — Already has `handleSkip()` (lines 45-57) calling `rebase_skip` IPC with toast — toast to be removed
- `src-tauri/src/commands/operation_state.rs` — `rebase_skip` command already implemented and registered

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `rebase_skip` IPC command: Already implemented in Rust, registered in lib.rs — no backend work needed
- `safeInvoke('rebase_skip', { path: repoPath })`: Already used in OperationBanner — exact same call for StagingPanel
- `--color-btn-skip-*` CSS tokens: Already defined for OperationBanner's Skip button
- `rebaseLoading` state in StagingPanel: Already used by Continue/Abort — Skip can share it

### Established Patterns
- `continueRebase()` handler in StagingPanel (line 267): Pattern for Skip — set loading, invoke IPC, catch errors, refresh
- `abortRebase()` handler in StagingPanel (line 288): Confirmation dialog pattern (Skip does NOT use this)
- OperationBanner `handleSkip()`: Reference implementation for the IPC call

### Integration Points
- StagingPanel button row (lines 777-814): Add Skip button between existing Continue and Abort buttons
- OperationBanner `handleSkip()` (line 49): Remove success toast for consistency

</code_context>

<specifics>
## Specific Ideas

- This is a very small phase — primarily adding one button to StagingPanel and removing one toast from OperationBanner
- Backend work is zero — `rebase_skip` command already exists and is registered
- The OperationBanner toast removal is a consistency fix bundled with this phase

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 42-rebase-skip-inline-ui*
*Context gathered: 2026-03-23*
