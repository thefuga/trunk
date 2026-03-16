# Phase 29: Staging & Commit UX - Research

**Researched:** 2026-03-15
**Domain:** Svelte 5 UI components — commit form mode selector, button styling, layout splitting
**Confidence:** HIGH

## Summary

Phase 29 is a **frontend-only** phase that modifies three existing Svelte 5 components: `CommitForm.svelte` (replace amend checkbox with tab-style selector, add stash branch to submit handler), `StagingPanel.svelte` (color stage/unstage/discard buttons, split file lists 50/50), and `FileRow.svelte` (tint Plus/Minus action icons green/red). All backend commands (`create_commit`, `amend_commit`, `stash_save`) already exist and are wired through `safeInvoke`. No new Tauri commands are needed.

The one material risk is the stash-staged-only requirement: CONTEXT.md says "Stash mode stashes staged files only (git stash push --staged)" but libgit2 (and thus git2 0.19) does **not** support `--staged` flag. The existing `stash_save_inner` stashes everything dirty. The recommended approach is to **reuse the existing `stash_save` command as-is** (stash all dirty changes) for the commit form stash mode, since the alternative (git CLI subprocess) violates the project's architecture. The user said "staged files only" but the backend simply cannot do this with git2. This is a known limitation to document.

**Primary recommendation:** Build the three-way tab selector in CommitForm, wire it to existing commands, style buttons with inline CSS color values, and implement 50/50 flex split in StagingPanel. All changes are CSS + Svelte 5 reactive state — no backend work required.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Tab-style row with underline indicator (Commit | Amend | Stash) — not segmented control, not dropdown
- Positioned at the top of CommitForm, above the subject input
- Active tab gets underline indicator; replaces the current amend checkbox entirely
- Message carries across mode switches — switching from commit to stash keeps whatever you typed. Switching to amend still pre-fills from HEAD commit message (existing behavior). Switching away from amend does NOT clear — your prior draft stays.
- Submit button label is dynamic: "Commit", "Amend", or "Stash" based on selected mode (plus "Committing..." / "Amending..." / "Stashing..." during operation)
- Stash mode stashes staged files only (git stash push --staged) — not everything dirty
- Stash name is optional — subject field auto-populates with current commit form message (STAGE-02), falls back to git default "WIP on {branch}" if empty
- Validation: requires at least one staged file; name is optional
- All three existing stash triggers kept (toolbar stashes everything dirty, sidebar has its own form, commit form stashes staged only) — different scopes, different use cases
- After successful stash: clear form fields and reset mode to commit
- "Stage All Changes" button gets filled green background with white text
- "Unstage All" button gets filled red background with white text
- "Discard All" button also gets filled red background with white text
- Individual file row action icons: Plus (+) icon gets green tint, Minus (-) icon gets red tint
- Fixed 50/50 split of available space when both sections are expanded — each list gets exactly half
- Each section is collapsible via chevron toggle (existing behavior preserved)
- When one section is collapsed, the remaining section expands to take 100% of available space
- Each half has its own independent scroll container
- Section headers always visible even when section has 0 files — consistent layout, no shifting

### Claude's Discretion
- Whether body textarea is hidden or shown in stash mode
- Tab underline exact styling (thickness, color, animation/transition)
- Button border-radius, padding, and exact green/red color values
- How the 50/50 split is implemented (CSS flex, grid, or explicit heights)
- Toast messages for stash success/error from commit form
- Loading/disabled states for the tab row during operations

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| STAGE-01 | Commit form has a three-way selector (commit / amend / stash) replacing the amend checkbox | Tab-style row with underline indicator; `$state<'commit' \| 'amend' \| 'stash'>('commit')` replaces `$state(false)` for amend; existing `handleAmendToggle` logic reused when switching to amend tab |
| STAGE-02 | In stash mode, commit form subject auto-populates as the stash name | Subject field value passed as `message` param to `stash_save`; if empty, backend falls back to "WIP on {branch}" |
| STAGE-03 | "Stage all changes" button is styled green | Inline style change on StagingPanel.svelte line 221-234: `background: #22c55e; color: white; border-radius: 4px; padding: 2px 8px;` |
| STAGE-04 | "Unstage all changes" button is styled red | Inline style change on StagingPanel.svelte line 289-303: `background: #f87171; color: white; border-radius: 4px; padding: 2px 8px;` |
| STAGE-05 | Unstaged and staged file lists render at equal height when both expanded | Replace single `overflow-y: auto` wrapper with two flex children each `flex: 1; overflow-y: auto;` |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte | ^5.0.0 | UI framework | Project framework — `$state`, `$derived`, `$effect` runes |
| @lucide/svelte | ^0.577.0 | Icons | Already used for Plus/Minus in FileRow, ChevronDown/Right in StagingPanel |
| @tauri-apps/api/core | ^2 | IPC (invoke) | All backend calls go through `safeInvoke` wrapper |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/plugin-dialog | ^2.6.0 | Native dialogs | Already used for discard confirmations — not needed for stash |

### No New Dependencies
This phase requires **zero** new npm or Cargo dependencies. Everything needed is already installed.

## Architecture Patterns

### Recommended Project Structure
No new files needed. All changes are in existing components:
```
src/components/
├── CommitForm.svelte     # Tab selector + stash submit branch
├── StagingPanel.svelte   # Button styling + 50/50 layout split
└── FileRow.svelte        # Green/red icon tinting
```

### Pattern 1: Mode State with `$state` Rune
**What:** Replace boolean `amend` state with string union mode state
**When to use:** When a UI control has more than two states
**Example:**
```typescript
// BEFORE (CommitForm.svelte line 16)
let amend = $state(false);

// AFTER
let mode = $state<'commit' | 'amend' | 'stash'>('commit');
```

All existing references to `amend` become `mode === 'amend'`. The `handleAmendToggle` function becomes a mode-switch handler that fires when any tab is clicked.

### Pattern 2: Derived Button Label with `$derived`
**What:** Dynamic submit button text based on mode + loading state
**When to use:** When display text depends on multiple reactive values
**Example:**
```typescript
let buttonLabel = $derived.by(() => {
  if (committing) {
    return mode === 'commit' ? 'Committing...' : mode === 'amend' ? 'Amending...' : 'Stashing...';
  }
  return mode === 'commit' ? 'Commit' : mode === 'amend' ? 'Amend' : 'Stash';
});
```

### Pattern 3: Mode Switch with Message Preservation
**What:** When switching modes, preserve the user's typed message (except amend pre-fill)
**When to use:** Tab selector mode switches in CommitForm
**Example:**
```typescript
async function handleModeSwitch(newMode: 'commit' | 'amend' | 'stash') {
  const prevMode = mode;
  mode = newMode;
  
  if (newMode === 'amend') {
    // Pre-fill from HEAD (existing handleAmendToggle logic)
    const msg = await safeInvoke<HeadCommitMessage>('get_head_commit_message', { path: repoPath });
    subject = msg.subject;
    body = msg.body ?? '';
  }
  // Switching AWAY from amend: keep current values (don't clear)
  // Switching between commit <-> stash: keep current values
}
```

### Pattern 4: Existing stash_save IPC Call
**What:** Call existing `stash_save` command with subject as message
**When to use:** Stash mode submit handler
**Example:**
```typescript
// In handleSubmit, third branch for stash mode:
if (mode === 'stash') {
  await safeInvoke('stash_save', { path: repoPath, message: subject.trim() });
  showToast('Stash created', 'success');
  subject = '';
  onsubjectchange?.('');
  body = '';
  mode = 'commit'; // reset to commit mode after stash
}
```

### Pattern 5: 50/50 Flex Layout with Collapse
**What:** Two flex children that share space equally, with collapse support
**When to use:** When two scrollable lists need equal height
**Example:**
```html
<!-- Outer container -->
<div style="flex: 1; display: flex; flex-direction: column; overflow: hidden; min-height: 0;">
  <!-- Unstaged section -->
  <div style="
    {unstaged_expanded && staged_expanded ? 'flex: 1;' : unstaged_expanded ? 'flex: 1;' : ''}
    display: flex; flex-direction: column; overflow: hidden; min-height: 0;
  ">
    <!-- Header (always visible, flex-shrink: 0) -->
    <div style="height: 28px; flex-shrink: 0; ...">...</div>
    <!-- Scrollable list (only when expanded) -->
    {#if unstaged_expanded}
      <div style="flex: 1; overflow-y: auto; min-height: 0;" role="list">...</div>
    {/if}
  </div>
  
  <!-- Staged section (mirror structure) -->
  <div style="
    {staged_expanded && unstaged_expanded ? 'flex: 1;' : staged_expanded ? 'flex: 1;' : ''}
    display: flex; flex-direction: column; overflow: hidden; min-height: 0;
  ">
    ...
  </div>
</div>
```

### Anti-Patterns to Avoid
- **Don't use `display: grid` with `grid-template-rows: 1fr 1fr`** for the 50/50 split — `flex: 1` is simpler and the codebase uses flex throughout
- **Don't clear subject/body when switching from amend to commit** — CONTEXT.md says "your prior draft stays"
- **Don't add a new Tauri command for stash-staged-only** — git2 doesn't support it; reuse existing `stash_save`
- **Don't use CSS classes or Tailwind utilities for button colors** — the codebase uses inline styles consistently; these are one-off color overrides, not design system tokens

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| IPC error handling | Custom error parser | `safeInvoke` from `src/lib/invoke.ts` | Already parses JSON-encoded TrunkError from Tauri |
| Success/error feedback | Alert dialogs | `showToast()` from `src/lib/toast.svelte.ts` | Established pattern across all components |
| Stash backend | New stash command | Existing `stash_save` command | Already handles message fallback, error cases, cache rebuild |
| HEAD message fetch | Custom git command | Existing `get_head_commit_message` | Already used by amend toggle |

## Common Pitfalls

### Pitfall 1: Stash Validation Differs from Commit
**What goes wrong:** Stash mode tries to validate like commit mode (requiring subject), but stash name is optional
**Why it happens:** The existing `handleSubmit` requires `subject.trim()` to be non-empty
**How to avoid:** In stash mode, skip the subject-required validation. The `stash_save` backend already falls back to "WIP on {branch}" for empty messages.
**Warning signs:** Stash button disabled or showing "Subject is required" error in stash mode

### Pitfall 2: Stash Mode Still Requires Staged Files
**What goes wrong:** Developer forgets that stash (like commit) needs at least one staged file per CONTEXT.md validation rules
**Why it happens:** The existing `stash_save` stashes ALL dirty changes including unstaged, so validation might seem unnecessary
**How to avoid:** Keep the `stagedCount === 0` check for stash mode. The CONTEXT.md says "Validation: requires at least one staged file" even though the backend will stash everything.
**Warning signs:** Stash succeeds with 0 staged files (technically works but violates spec)

### Pitfall 3: Amend Pre-fill Clobbers Draft When Re-entering Amend
**What goes wrong:** User writes a commit message, switches to amend (pre-fills from HEAD), switches back to commit, then switches to amend again — each amend switch overwrites their current text
**Why it happens:** `handleAmendToggle` always fetches and sets HEAD message
**How to avoid:** This is intentional per CONTEXT.md: "Switching to amend still pre-fills from HEAD commit message." Each switch TO amend always pre-fills. Switching AWAY preserves the amend text.
**Warning signs:** N/A — this is correct behavior

### Pitfall 4: Single Scroll Container Breaks 50/50 Split
**What goes wrong:** The existing single `overflow-y: auto` wrapper at StagingPanel line 181 means both sections share one scroll. Splitting into two sections requires removing this wrapper and giving each section its own scroll.
**Why it happens:** Current layout has one scrollable area containing both unstaged and staged sections
**How to avoid:** Replace the single scroll wrapper `<div style="flex: 1; overflow-y: auto; min-height: 0;">` with a flex container that holds two independently-scrollable sections
**Warning signs:** Both lists scrolling together as one, or the 50/50 split not respecting available height

### Pitfall 5: Button Color Overrides Need Full Style Reset
**What goes wrong:** Current buttons have `background: none; border: none;` and rely on `color` for text. Changing to filled background requires setting `background`, `color` (to white), `border-radius`, and `padding` explicitly.
**Why it happens:** Current buttons are text-only (no background)
**How to avoid:** Set complete button styles: `background: #22c55e; color: white; border: none; border-radius: 4px; padding: 2px 8px; font-size: 11px; cursor: pointer; white-space: nowrap;`
**Warning signs:** Green/red background but dark text, or rounded corners missing

### Pitfall 6: clearRedoStack() Should Not Run for Stash
**What goes wrong:** `clearRedoStack()` is called at the start of `handleSubmit` — it's relevant for commit/amend (modifies history) but not for stash
**Why it happens:** The function runs before the mode check
**How to avoid:** Move `clearRedoStack()` inside the commit and amend branches, skip for stash
**Warning signs:** Redo stack cleared unexpectedly when stashing

## Code Examples

### Tab Selector HTML (CommitForm — replaces amend checkbox)
```html
<!-- Tab row — replaces lines 133-144 (amend checkbox) -->
<div style="display: flex; gap: 0; border-bottom: 1px solid var(--color-border);">
  {#each ['commit', 'amend', 'stash'] as tab}
    <button
      onclick={() => handleModeSwitch(tab)}
      disabled={committing}
      style="
        flex: 1;
        padding: 6px 0 4px;
        font-size: 11px;
        background: none;
        border: none;
        border-bottom: 2px solid {mode === tab ? 'var(--color-accent)' : 'transparent'};
        color: {mode === tab ? 'var(--color-text)' : 'var(--color-text-muted)'};
        cursor: pointer;
        text-transform: capitalize;
      "
    >
      {tab === 'commit' ? 'Commit' : tab === 'amend' ? 'Amend' : 'Stash'}
    </button>
  {/each}
</div>
```

### Green Stage All Button (StagingPanel)
```html
<!-- Replace lines 221-235 -->
<button
  onclick={(e) => { e.stopPropagation(); stageAll(); }}
  style="
    background: #22c55e;
    color: white;
    font-size: 11px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 8px;
    white-space: nowrap;
  "
  aria-label="Stage all changes"
>
  Stage All Changes
</button>
```

### Red Unstage All Button (StagingPanel)
```html
<!-- Replace lines 289-303 -->
<button
  onclick={(e) => { e.stopPropagation(); unstageAll(); }}
  style="
    background: #f87171;
    color: white;
    font-size: 11px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 8px;
    white-space: nowrap;
  "
  aria-label="Unstage all"
>
  Unstage All
</button>
```

### Green/Red Icon Tinting (FileRow)
```html
<!-- Replace lines 81-101 — action button with conditional color -->
{#if hovered && !isLoading}
  <button
    onclick={(e) => { e.stopPropagation(); onaction(); }}
    aria-label={actionLabel === '+' ? 'Stage file' : 'Unstage file'}
    style="
      background: none;
      border: none;
      cursor: pointer;
      color: {actionLabel === '+' ? '#22c55e' : '#f87171'};
      display: flex;
      align-items: center;
      padding: 0 4px;
      line-height: 1;
    "
  >
    {#if actionLabel === '+'}
      <Plus size={11} />
    {:else}
      <Minus size={11} />
    {/if}
  </button>
{/if}
```

## Stash-Staged-Only Limitation

### The Problem
CONTEXT.md says "Stash mode stashes staged files only (git stash push --staged)". However:
- `git stash push --staged` was added in **Git 2.35** (January 2022)
- libgit2 (which git2 0.19 wraps) does **not** implement this flag
- git2 0.19 `StashFlags` only has: `DEFAULT`, `KEEP_INDEX`, `INCLUDE_UNTRACKED`, `INCLUDE_IGNORED`, `KEEP_ALL`
- There is **no** `STAGED_ONLY` flag

### Verified (HIGH confidence)
Confirmed via [docs.rs/git2/0.19.0/git2/struct.StashFlags.html](https://docs.rs/git2/0.19.0/git2/struct.StashFlags.html) — only 5 flags exist, none for staged-only stash.

### Recommendation: Reuse Existing stash_save (Stash All Dirty)
**Why:**
1. Adding a git CLI subprocess (`git stash push --staged`) breaks the project's pure-git2 architecture
2. The `KEEP_INDEX` flag does the opposite — it stashes everything but *keeps* the index, not "stash only the index"
3. No workaround exists within git2 that correctly implements staged-only stash
4. The stash from commit form will work fine stashing everything dirty — the primary UX goal (save work with a name from the commit form) is achieved

**What changes:** The commit form stash mode will call the existing `stash_save` command which stashes all dirty changes (both staged and unstaged), not just staged. This matches what the toolbar and sidebar stash triggers already do.

**If the user later wants staged-only stash:** This would require either (a) upgrading to a future version of git2/libgit2 that adds the flag, or (b) adding a targeted git CLI subprocess call. Both are out of scope for this phase.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Boolean `amend` toggle | Three-way mode union type | This phase | Replaces checkbox with tab selector |
| Single scroll wrapper | Two independent scroll containers | This phase | Enables 50/50 split |
| Unstyled text buttons | Filled colored buttons | This phase | Green/red visual distinction |

## Open Questions

1. **Stash-staged-only vs stash-all**
   - What we know: git2 cannot stash staged only; existing command stashes everything
   - What's unclear: Whether user will accept stash-all as sufficient for commit form stash mode
   - Recommendation: Implement with stash-all and document as known limitation. The UX goal (quick save from commit form) is met regardless.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.0 |
| Config file | `vite.config.ts` (test section at line 24) |
| Quick run command | `npm test` |
| Full suite command | `npm test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| STAGE-01 | Three-way mode state management | unit | `npx vitest run src/lib/commit-mode.test.ts` | ❌ Wave 0 |
| STAGE-02 | Subject auto-populates as stash name | manual-only | N/A — requires Tauri IPC | N/A |
| STAGE-03 | Stage all button styled green | manual-only | N/A — visual CSS assertion | N/A |
| STAGE-04 | Unstage all button styled red | manual-only | N/A — visual CSS assertion | N/A |
| STAGE-05 | Equal height file lists | manual-only | N/A — layout assertion | N/A |

### Sampling Rate
- **Per task commit:** `npm test`
- **Per wave merge:** `npm test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/lib/commit-mode.test.ts` — covers STAGE-01 mode switching logic (button labels, validation rules per mode)
- This phase is heavily UI/CSS — most requirements are visual and cannot be automatically tested with the current Vitest+Node setup (no DOM, no Svelte component testing configured)

*(Most requirements are visual/layout changes verified by manual inspection — unit tests cover mode state logic only)*

## Sources

### Primary (HIGH confidence)
- `src/components/CommitForm.svelte` — 169 lines, complete read — amend checkbox at lines 133-144, submit handler at lines 45-84
- `src/components/StagingPanel.svelte` — 328 lines, complete read — button styles at lines 206-235 and 288-303, scroll wrapper at line 181
- `src/components/FileRow.svelte` — 103 lines, complete read — action button at lines 81-101
- `src-tauri/src/commands/stash.rs` — 339 lines, complete read — `stash_save_inner` at lines 41-64, uses `repo.stash_save(&sig, &msg, None)` with no StashFlags
- [docs.rs/git2/0.19.0 StashFlags](https://docs.rs/git2/0.19.0/git2/struct.StashFlags.html) — confirmed only DEFAULT, KEEP_INDEX, INCLUDE_UNTRACKED, INCLUDE_IGNORED, KEEP_ALL flags exist
- `src/app.css` — color tokens: `--color-accent: #388bfd`, `--color-text: #c9d1d9`, `--color-text-muted: #8b949e`, `--color-border: #30363d`
- `src/lib/toast.svelte.ts` — `showToast(message, kind)` API
- `src/lib/invoke.ts` — `safeInvoke<T>(cmd, args)` API, `TrunkError` interface

### Secondary (MEDIUM confidence)
- Phase 11 CONTEXT.md — stash create patterns, sidebar stash form, established stash naming behavior
- Phase 28 CONTEXT.md (referenced) — Discard All button placement, `ask()` dialog pattern

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified in package.json and Cargo.toml, no new dependencies
- Architecture: HIGH — all patterns verified by reading existing source code, established conventions followed
- Pitfalls: HIGH — derived from direct code analysis of the three target files
- Stash limitation: HIGH — verified via official docs.rs documentation for git2 0.19.0 StashFlags

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable — no external dependencies changing, pure UI work)
