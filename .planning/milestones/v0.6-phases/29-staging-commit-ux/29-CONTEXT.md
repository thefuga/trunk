# Phase 29: Staging & Commit UX - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Users get a unified "save my work" workflow through a three-way mode selector (commit / amend / stash) replacing the current amend checkbox, polished staging controls with colored buttons, and equal-height file lists. Requirements: STAGE-01 through STAGE-05. All backend commands already exist (`create_commit`, `amend_commit`, `stash_save`) — this phase is frontend-only.

</domain>

<decisions>
## Implementation Decisions

### Three-way selector design
- Tab-style row with underline indicator (Commit | Amend | Stash) — not segmented control, not dropdown
- Positioned at the top of CommitForm, above the subject input — sets the mode before the user starts typing
- Active tab gets underline indicator; replaces the current amend checkbox entirely
- Message carries across mode switches — switching from commit to stash keeps whatever you typed. Switching to amend still pre-fills from HEAD commit message (existing behavior). Switching away from amend does NOT clear — your prior draft stays.
- Submit button label is dynamic: "Commit", "Amend", or "Stash" based on selected mode (plus "Committing..." / "Amending..." / "Stashing..." during operation)

### Stash mode behavior
- Stash mode stashes staged files only (git stash push --staged) — not everything dirty
- Stash name is optional — subject field auto-populates with current commit form message (STAGE-02), falls back to git default "WIP on {branch}" if empty
- Validation: requires at least one staged file; name is optional
- All three existing stash triggers kept (toolbar stashes everything dirty, sidebar has its own form, commit form stashes staged only) — different scopes, different use cases
- After successful stash: clear form fields and reset mode to commit

### Stage/Unstage button styling
- "Stage All Changes" button gets filled green background with white text — clearly actionable, prominent
- "Unstage All" button gets filled red background with white text — clearly distinguishable from stage
- "Discard All" button also gets filled red background with white text — same red styling as Unstage All for visual consistency
- Individual file row action icons: Plus (+) icon gets green tint, Minus (-) icon gets red tint — reinforces stage=green, unstage=red throughout the panel

### File list equal-height layout
- Fixed 50/50 split of available space when both sections are expanded — each list gets exactly half
- Each section is collapsible via chevron toggle (existing behavior preserved)
- When one section is collapsed, the remaining section expands to take 100% of available space
- Each half has its own independent scroll container — scrolling one list doesn't affect the other
- Section headers always visible even when section has 0 files — consistent layout, no shifting

### Claude's Discretion
- Whether body textarea is hidden or shown in stash mode
- Tab underline exact styling (thickness, color, animation/transition)
- Button border-radius, padding, and exact green/red color values
- How the 50/50 split is implemented (CSS flex, grid, or explicit heights)
- Toast messages for stash success/error from commit form
- Loading/disabled states for the tab row during operations

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — STAGE-01 through STAGE-05 define acceptance criteria for this phase

### Prior phase context
- `.planning/phases/11-stash-operations/11-CONTEXT.md` — Stash creation patterns, sidebar stash form, stash naming behavior
- `.planning/phases/28-destructive-operations/28-CONTEXT.md` — Discard All button placement, `ask()` dialog pattern, FileRow context menu wiring

### Key source files
- `src/components/CommitForm.svelte` — Amend checkbox to be replaced, submit logic, message field handling
- `src/components/StagingPanel.svelte` — Stage All / Unstage All / Discard All buttons, file list sections, layout structure
- `src/components/FileRow.svelte` — Individual file action buttons (Plus/Minus icons), row styling

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CommitForm.svelte`: Already has amend toggle with HEAD message pre-fill via `get_head_commit_message` — this logic stays, just triggered by tab switch instead of checkbox
- `StagingPanel.svelte`: File section collapse already works with chevron toggles — reuse collapse state, add height constraints
- `Toolbar.svelte btn-group CSS`: Joined button pattern with shared border-radius — reference for tab styling (though tabs will be underline style, not button style)
- `safeInvoke<T>`: All IPC wrapper, already imported in CommitForm — stash_save call follows same pattern
- `showToast()` from `src/lib/toast.svelte.ts`: Success/error feedback after stash from commit form
- `@lucide/svelte` Plus/Minus icons: Already in FileRow.svelte — just need color prop added

### Established Patterns
- `$state` rune for component state: CommitForm uses `let amend = $state(false)` — extend to `let mode = $state<'commit' | 'amend' | 'stash'>('commit')`
- `$derived.by()` for imperative reactive computations — use for dynamic button label, validation logic
- `clearRedoStack()` call at submit start — preserve for commit/amend, skip for stash
- `onsubjectchange` callback: Propagated to parent for WIP row message display — keep working across all modes

### Integration Points
- `CommitForm.svelte` lines 133-144: Amend checkbox DOM to be replaced with tab row
- `CommitForm.svelte` lines 45-84: `handleSubmit` needs third branch for stash mode calling `stash_save`
- `StagingPanel.svelte` lines 181: Single scroll wrapper needs splitting into two 50% containers
- `StagingPanel.svelte` lines 221-235: Stage All button styling change (background, color)
- `StagingPanel.svelte` lines 289-303: Unstage All button styling change
- `StagingPanel.svelte` lines 206-220: Discard All button styling change
- `FileRow.svelte` lines ~85-95: Action button icon needs conditional color (green for +, red for -)

</code_context>

<specifics>
## Specific Ideas

- Stash from commit form stashes staged only — distinct from toolbar stash (everything dirty) and sidebar stash (everything dirty with optional name). Three triggers, three scopes.
- Tab-style, not segmented control — should feel lightweight like mode tabs, not like buttons to press
- Discard All gets same red treatment as Unstage All — user wants visual consistency across "remove" actions, even though discard is destructive with confirmation dialog

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 29-staging-commit-ux*
*Context gathered: 2026-03-15*
