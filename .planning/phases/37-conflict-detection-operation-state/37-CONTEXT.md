# Phase 37: Conflict Detection & Operation State - Context

**Gathered:** 2026-03-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can see which files are conflicted and know when a merge or rebase operation is in progress. This phase adds a distinct conflicted files section in the staging panel and persistent operation banners with action buttons. The merge editor (Phase 38) and merge/rebase initiation (Phases 39-40) are out of scope.

</domain>

<decisions>
## Implementation Decisions

### Conflict section layout
- Conflicted files appear as the **top section** in the staging panel, above unstaged and staged
- Section header uses **yellow warning icon + yellow count badge** on neutral background (same bg as unstaged/staged headers, not a colored background)
- Section is **collapsible**, consistent with unstaged/staged section behavior
- **No action buttons** on conflicted file rows in Phase 37 (no stage, discard, or resolve buttons)

### Operation banner design
- Banner appears at the **top of the staging panel**, above the conflict section
- Shows **operation type + branch names**: "Merging feature into main" / "Rebasing main onto origin/main"
- **Color-coded by operation type**: yellow-tinted background with yellow left border for merge, blue-tinted background with blue left border for rebase
- Banner is persistent — only disappears when the operation completes or is aborted

### Conflicted file interaction
- Clicking a conflicted file opens DiffPanel showing **raw file content with conflict markers** (<<<<<<< / ======= / >>>>>>>), read-only, no hunk action buttons
- Right-click context menu is **minimal**: only Copy Relative Path and Copy Absolute Path (no Stage/Discard)

### Banner button behavior
- **Merge banner**: Continue and Abort buttons
- **Rebase banner**: Continue, Skip, and Abort buttons
- **Abort requires confirmation dialog** before executing ("Abort merge? This will discard all merge progress...")
- **Continue does not require confirmation** — if conflicts remain, git errors and we show feedback
- **Skip does not require confirmation** — non-destructive forward action

### Claude's Discretion
- Feedback mechanism after Continue/Abort/Skip (toast, banner update, or combination)
- Exact banner padding, spacing, and typography
- How to detect operation state (git2 repository state API vs. filesystem checks for MERGE_HEAD / rebase-merge/)
- Confirmation dialog wording and button labels
- Whether to disable Continue button when conflicts still exist vs. letting git error

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above and REQUIREMENTS.md.

### Requirements
- `.planning/REQUIREMENTS.md` — CONF-01 (conflicted file display), OPS-01 (merge banner), OPS-02 (rebase banner), OPS-03 (banner button actions)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `WorkingTreeStatus.conflicted` field: Already exists in both Rust (`src-tauri/src/git/types.rs`) and TypeScript (`src/lib/types.ts`)
- `FileStatusType::Conflicted`: Already defined with yellow `FileWarning` icon (`#facc15`) in `FileRow.svelte`
- `Status::CONFLICTED` detection: Already implemented in `get_status_inner` (`src-tauri/src/commands/staging.rs:60`)
- Git CLI subprocess pattern: Proven in `commit_actions.rs` for cherry-pick/revert — reuse for merge/rebase continue/abort/skip
- `safeInvoke<T>` wrapper: Type-safe IPC for all frontend-to-backend calls
- `InputDialog` confirmation pattern: Existing dialog infrastructure for destructive action confirmation
- Toast system (`src/lib/toast.svelte.ts`): For post-action feedback notifications

### Established Patterns
- Collapsible section headers in StagingPanel: Unstaged/staged use chevron toggle with count badges
- Context menu via `@tauri-apps/api/menu`: Native Tauri menu API, used throughout StagingPanel
- `$state` rune modules: Cross-component state (e.g., `remote-state.svelte.ts`) for shared reactive state
- Cache-repopulate-before-emit: Mutation commands refresh cache then emit `repo-changed` event

### Integration Points
- `StagingPanel.svelte`: Conflicted files currently rendered inside unstaged section (lines 284-293) — needs separation into own section
- `get_status` Tauri command: Already returns `conflicted` array — no backend changes needed for file detection
- New Tauri commands needed: `get_operation_state`, `merge_continue`, `merge_abort`, `rebase_continue`, `rebase_skip`, `rebase_abort`
- Command registration in `src-tauri/src/lib.rs`: Add new commands to `generate_handler!` macro
- CSS custom properties in `src/app.css`: Add warning/info banner colors (yellow-tinted, blue-tinted backgrounds)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 37-conflict-detection-operation-state*
*Context gathered: 2026-03-20*
