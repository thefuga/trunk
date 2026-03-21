# Phase 38: Merge Editor - Context

**Gathered:** 2026-03-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Three-panel merge editor for resolving conflicted files with per-hunk and per-line selection, editable output, navigation, and resolution workflow. Opens when a user clicks a conflicted file in the staging panel (replacing the Phase 37 read-only conflict marker view). Merge/rebase initiation is out of scope (Phase 39-40).

</domain>

<decisions>
## Implementation Decisions

### Editor placement & panel layout
- Merge editor **replaces DiffPanel** in the same right-pane area — clicking a conflicted file swaps the right pane content, consistent with existing DiffPanel pattern
- Three panels: **Current (ours) and Incoming (theirs) side-by-side on top**, **Output (editable) spanning full width on bottom**
- **50/50 vertical split** between top row and output, **50/50 horizontal split** between current and incoming
- **Fixed splits** — no draggable dividers, no resizing
- **Colored header bars** on each panel: Current (ours) with blue-ish tint, Incoming (theirs) with green-ish tint, Output with neutral tint
- **Synchronized scroll** across all three panels (GitKraken-style)

### Conflict region styling
- Conflict regions use **standard diff-style background coloring** (green for additions, red for deletions) — same colors as normal diffs, not panel-specific colors
- **Non-conflict regions shown in both** current and incoming panels as neutral context — keeps line numbers aligned and provides surrounding context
- **Line numbers displayed in all three panels** in a gutter on the left
- **Taken hunks at full brightness**, untaken hunks **dimmed to ~50% opacity**

### Hunk & line selection interaction
- Three levels of selection granularity: **whole file**, **per-hunk**, and **per-line**
- **Whole file**: "Take All Current" / "Take All Incoming" buttons in panel headers
- **Per-hunk**: Each conflict section has a **clickable hunk header row** (separator line). Clicking it toggles all lines in that hunk. Shows green check icon when all taken, empty when none
- **Per-line**: Click any line within a conflict region to toggle it. **Green check icon in gutter** when line is taken. **Hover over a taken line** switches icon to a red remove icon. Click again to remove from output
- **Output updates in real-time** as selections change — user sees the merged result take shape immediately

### Resolution flow
- **Toolbar split between panel headers**: Take All Current in Current panel header, Take All Incoming in Incoming panel header, Prev/Next conflict arrows and Save and Mark Resolved in Output panel header
- **Save and Mark Resolved** saves output to disk, stages the file, then **returns to the staging panel** (does not auto-open next file)
- **Right-click on conflicted file in staging panel** shows "Take All Current" and "Take All Incoming" as context menu items — quick resolution without opening the editor
- **Prev/Next conflict navigation stops at boundaries** — Prev disabled at first conflict, Next disabled at last. Does not wrap around

### Claude's Discretion
- Exact CSS custom property names and color values for panel header tints
- How to extract ours/theirs/base versions from git (git2 merge_file API vs index stages vs conflict marker parsing)
- Synchronized scroll implementation approach
- Output panel editing implementation (textarea, contenteditable, etc.)
- Conflict hunk header row design (separator style, label text)
- Keyboard shortcuts for navigation and selection
- Loading/error states

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — CONF-02 (three-panel editor), CONF-03 (sync scroll), CONF-04 (per-hunk checkboxes), CONF-05 (per-line selection), CONF-06 (editable output), CONF-07 (Take All buttons), CONF-08 (Prev/Next navigation), CONF-09 (Save and Mark Resolved)

### Prior phase context
- `.planning/phases/37-conflict-detection-operation-state/37-CONTEXT.md` — Conflict detection decisions, conflicted file section design, operation banner design

### Existing code
- `src/components/DiffPanel.svelte` — Current diff display component that merge editor replaces for conflicted files; line selection and hunk action patterns to reference
- `src/components/StagingPanel.svelte` — Integration point where conflicted file click triggers merge editor; context menu for right-click Take All actions
- `src-tauri/src/commands/operation_state.rs` — Operation state detection and merge/rebase CLI commands
- `src-tauri/src/commands/diff.rs` — Current diff infrastructure
- `src-tauri/src/commands/staging.rs` — Staging commands, conflict detection in get_status
- `src/lib/types.ts` — TypeScript DTOs (FileDiff, DiffHunk, DiffLine, WorkingTreeStatus)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `DiffPanel.svelte` line selection model: Click-to-select, shift-click range, selected line highlighting — reference for per-line toggle UX
- `safeInvoke<T>` IPC wrapper: Use for all new merge editor backend calls
- `showToast()`: For resolution feedback notifications
- CSS custom properties in `app.css`: Existing diff colors (`--color-diff-add-bg`, `--color-diff-delete-bg`, etc.) to reuse for conflict region styling
- `FileRow.svelte` with `FileStatusType::Conflicted`: Already renders conflicted files with yellow FileWarning icon
- Native Tauri context menu: For right-click Take All actions on conflicted files

### Established Patterns
- `diffKind` prop on DiffPanel: Currently supports 'unstaged' | 'staged' | 'commit' — merge editor is a separate component, not a new diffKind
- `onfileselect` callback in StagingPanel: Already passes `kind: 'conflicted'` — can be used to switch to merge editor
- Cache-repopulate-before-emit: After staging resolved files, refresh cache then emit `repo-changed`
- Inner-fn pattern for Tauri commands: New backend commands for extracting ours/theirs versions

### Integration Points
- `StagingPanel.svelte` `onfileselect`: Currently dispatches `('conflicted')` kind — parent switches to merge editor instead of DiffPanel
- New Tauri commands needed: `get_merge_sides` (extract ours/theirs/base content), `save_merge_result` (write output + stage)
- Context menu on conflicted files: Add "Take All Current" / "Take All Incoming" items
- `app.css`: New CSS custom properties for panel header tints (blue for current, green for incoming)

</code_context>

<specifics>
## Specific Ideas

- Synchronized scroll across all three panels, like GitKraken
- Line toggle interaction: green check icon in gutter when taken, switches to red remove icon on hover
- Hunk header row acts as a "select all lines in hunk" toggle
- Standard diff coloring (not panel-specific colors) for conflict regions

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 38-merge-editor*
*Context gathered: 2026-03-20*
