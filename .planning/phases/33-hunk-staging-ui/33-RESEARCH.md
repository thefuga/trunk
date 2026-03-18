# Phase 33: Hunk Staging UI - Research

**Researched:** 2026-03-17
**Domain:** Svelte 5 UI components, Tauri IPC, keyboard navigation, diff rendering
**Confidence:** HIGH

## Summary

This phase wires hunk staging UI into the existing DiffPanel component. The backend commands (`stage_hunk`, `unstage_hunk`, `discard_hunk`) already exist from Phase 32 and are registered in the invoke handler. The work is purely frontend: (1) replace the plain `@@` header line with a toolbar row containing context-dependent action buttons, (2) guard binary files from showing hunk buttons, (3) add `[`/`]` keyboard navigation between hunks, and (4) disable all hunk buttons during in-flight operations.

The existing codebase provides all necessary patterns: `safeInvoke` for IPC, `showToast` for feedback, `ask()` from `@tauri-apps/plugin-dialog` for destructive confirmation, and the `$effect`-based keyboard listener pattern in App.svelte. DiffPanel is currently a simple read-only component (155 lines) that needs a new `diffKind` prop to determine which buttons to show.

**Primary recommendation:** Modify DiffPanel.svelte to accept a `diffKind` prop (`'unstaged' | 'staged' | 'commit'`), replace the hunk header `<div>` with a toolbar row containing action buttons, add a `hunkOperationInFlight` state flag to disable all buttons during operations, and add a scoped `keydown` listener for `[`/`]` navigation.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Buttons appear in a **toolbar row above each hunk**, replacing the current `@@` header line
- Toolbar row contains: line range info (from `@@` header) on the left, action buttons on the right
- Buttons use **text labels only** -- "Stage Hunk", "Unstage Hunk", "Discard Hunk" (no icons)
- Toolbar row has a **subtle distinct background** (same muted style as current `@@` header) to visually separate from diff lines
- **Unstaged diff**: "Stage Hunk" + "Discard Hunk" buttons
- **Staged diff**: "Unstage Hunk" button only (no discard)
- **Commit diff**: No hunk buttons at all
- **Binary files**: No hunk buttons (existing `fd.is_binary` guard)
- DiffPanel needs a new prop to receive the diff kind ('unstaged' | 'staged' | 'commit') from App.svelte
- **All hunk buttons in the file** are disabled during any hunk operation (prevents stale-index races since indices may shift)
- Disabled state: **reduced opacity + cursor: not-allowed** -- standard disabled pattern, no spinners
- Buttons re-enable after diff re-fetch completes
- `]` jumps to next hunk, `[` jumps to previous hunk
- Navigation **scrolls the diff view** so the target hunk's toolbar row is at the top, and **briefly highlights** it (flash or border)
- At edges: **stop** -- no wrap-around (] on last hunk does nothing, [ on first does nothing)
- Shortcuts **only active when DiffPanel is visible** -- prevents conflicts with commit message textarea or other inputs
- Discard Hunk button appears alongside Stage Hunk in the toolbar row (unstaged diffs only)
- Confirmation uses **Tauri ask() dialog** -- "Discard this hunk? This cannot be undone." -- consistent with existing discard_file pattern
- Backend trusts frontend confirmation (decided in Phase 32)

### Claude's Discretion
- Exact CSS styling of toolbar row (padding, font size, button spacing)
- Highlight animation implementation for hunk navigation (CSS transition or brief class toggle)
- Whether to track focused hunk index in component state or derive from scroll position
- How to pass diff kind prop through (direct prop vs context)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| HUNK-04 | DiffPanel displays context-appropriate actions (Stage Hunk for unstaged, Unstage Hunk for staged, no buttons for commit diffs) | New `diffKind` prop on DiffPanel + conditional button rendering in hunk toolbar row. Backend commands already exist. |
| HUNK-06 | Hunk action buttons are hidden for binary file diffs | Existing `fd.is_binary` guard in DiffPanel already skips hunk rendering for binary files -- no hunk toolbar will be rendered inside the binary fallback block. |
| HUNK-09 | User can navigate between hunks in the diff view using keyboard shortcuts | `[`/`]` keydown listener scoped to DiffPanel visibility, `scrollIntoView` on hunk toolbar DOM elements, CSS flash animation for highlight. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| svelte | ^5.0.0 | Component framework | Project standard -- Svelte 5 runes ($state, $derived, $effect, $props) |
| @tauri-apps/api | ^2 | IPC invoke for hunk commands | Project standard -- safeInvoke wrapper |
| @tauri-apps/plugin-dialog | ^2.6.0 | ask() for discard confirmation | Project standard -- same pattern as discard_file |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none needed) | -- | -- | All supporting code exists in-project |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Direct prop for diffKind | Svelte context | Direct prop is simpler, no context needed -- DiffPanel is used in one place |
| Track focused hunk index in state | Derive from scroll position | State tracking is simpler and more reliable -- scroll-position derivation requires IntersectionObserver overhead |

## Architecture Patterns

### Current DiffPanel Structure (Before Phase 33)
```
DiffPanel.svelte (155 lines, read-only)
  Props: fileDiffs, commitDetail, selectedPath, onclose
  Template: toolbar -> scrollable area -> file loop -> binary guard -> hunk loop -> lines
```

### Target DiffPanel Structure (After Phase 33)
```
DiffPanel.svelte (~250 lines estimated)
  Props: fileDiffs, commitDetail, selectedPath, onclose, diffKind, repoPath, onhunkaction
  State: hunkOperationInFlight (boolean)
  Template: toolbar -> scrollable area -> file loop -> binary guard -> hunk loop:
    - Hunk toolbar row (replaces plain @@ header)
      - Left: line range text from @@ header
      - Right: conditional action buttons (Stage/Unstage/Discard based on diffKind)
    - Diff lines (unchanged)
```

### Pattern 1: Hunk Toolbar Row Replacing @@ Header
**What:** Transform the existing hunk header `<div>` into a flex row with line info left, buttons right.
**When to use:** Every hunk in non-binary, non-commit diffs.
**Example:**
```svelte
<!-- Current hunk header (line 124-133 of DiffPanel.svelte) -->
<div style="background: var(--color-bg); color: var(--color-text-muted); font-size: 11px; font-family: monospace; padding: 2px 8px;">
  {hunk.header}
</div>

<!-- Replacement: toolbar row with buttons -->
<div
  bind:this={hunkElements[`${fd.path}-${hunkIdx}`]}
  style="
    background: var(--color-bg);
    display: flex;
    align-items: center;
    padding: 2px 8px;
    gap: 6px;
  "
>
  <span style="flex: 1; color: var(--color-text-muted); font-size: 11px; font-family: monospace;">
    {hunk.header}
  </span>
  {#if diffKind === 'unstaged'}
    <button disabled={hunkOperationInFlight} onclick={() => handleStageHunk(fd.path, hunkIdx)}>Stage Hunk</button>
    <button disabled={hunkOperationInFlight} onclick={() => handleDiscardHunk(fd.path, hunkIdx)}>Discard Hunk</button>
  {:else if diffKind === 'staged'}
    <button disabled={hunkOperationInFlight} onclick={() => handleUnstageHunk(fd.path, hunkIdx)}>Unstage Hunk</button>
  {/if}
  <!-- No buttons for diffKind === 'commit' -->
</div>
```

### Pattern 2: In-Flight Operation Disabling
**What:** Single boolean flag disables ALL hunk buttons during any hunk operation. Re-enables after diff re-fetch completes.
**When to use:** All hunk mutation operations.
**Example:**
```svelte
<script lang="ts">
  let hunkOperationInFlight = $state(false);

  async function handleStageHunk(filePath: string, hunkIndex: number) {
    hunkOperationInFlight = true;
    try {
      await safeInvoke('stage_hunk', { path: repoPath, filePath, hunkIndex });
      showToast(`Staged hunk`, 'success');
      await onhunkaction(filePath);  // triggers re-fetch + status refresh in parent
    } catch (e) {
      const err = e as TrunkError;
      showToast(err.message ?? 'Stage hunk failed', 'error');
    } finally {
      hunkOperationInFlight = false;
    }
  }
</script>
```

### Pattern 3: Keyboard Navigation Between Hunks
**What:** `[`/`]` keys scroll to previous/next hunk toolbar row. Only active when DiffPanel is visible.
**When to use:** When DiffPanel is rendered and user focus is not in a text input.
**Example:**
```svelte
<script lang="ts">
  let focusedHunkIndex = $state(0);
  let hunkElements: Record<string, HTMLDivElement> = {};

  function scrollToHunk(index: number) {
    const keys = Object.keys(hunkElements);
    if (index < 0 || index >= keys.length) return;
    focusedHunkIndex = index;
    const el = hunkElements[keys[index]];
    el?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    // Brief highlight flash
    el?.classList.add('hunk-highlight');
    setTimeout(() => el?.classList.remove('hunk-highlight'), 600);
  }

  $effect(() => {
    function handleKeydown(e: KeyboardEvent) {
      // Skip if focus is in an input/textarea
      const tag = (e.target as HTMLElement)?.tagName;
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

      if (e.key === ']') {
        e.preventDefault();
        scrollToHunk(focusedHunkIndex + 1);
      } else if (e.key === '[') {
        e.preventDefault();
        scrollToHunk(focusedHunkIndex - 1);
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });
</script>
```

### Pattern 4: Discard Hunk Confirmation
**What:** Use `ask()` from `@tauri-apps/plugin-dialog` for destructive discard confirmation.
**When to use:** Before calling `discard_hunk` backend command.
**Example:**
```svelte
async function handleDiscardHunk(filePath: string, hunkIndex: number) {
  const { ask } = await import('@tauri-apps/plugin-dialog');
  const confirmed = await ask('Discard this hunk? This cannot be undone.', {
    title: 'Discard Hunk',
    kind: 'warning',
  });
  if (!confirmed) return;

  hunkOperationInFlight = true;
  try {
    await safeInvoke('discard_hunk', { path: repoPath, filePath, hunkIndex });
    showToast('Discarded hunk', 'success');
    await onhunkaction(filePath);
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Discard hunk failed', 'error');
  } finally {
    hunkOperationInFlight = false;
  }
}
```

### Anti-Patterns to Avoid
- **Passing hunk index as `data-` attribute and parsing from DOM:** Use the `{#each}` loop index directly -- Svelte closures capture it correctly.
- **Separate loading state per hunk:** A single `hunkOperationInFlight` boolean is correct because hunk indices shift after any mutation, making concurrent operations on different hunks dangerous.
- **Wrapping keyboard listener in DiffPanel without checking visibility:** The DiffPanel may be unmounted when CommitGraph shows. Since Svelte `$effect` cleanup removes the listener on unmount, this is handled automatically. However, still guard against input/textarea focus.
- **Using `Element.scrollTo` with manual offset calculation:** Use `scrollIntoView({ block: 'start' })` instead -- simpler and correct for scrollable containers.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| IPC error handling | Custom try/catch with string parsing | `safeInvoke` from `src/lib/invoke.ts` | Already handles Tauri string error format |
| Toast notifications | Custom notification system | `showToast` from `src/lib/toast.svelte.ts` | Existing project-wide toast system |
| Destructive confirmation dialogs | Custom modal component | `ask()` from `@tauri-apps/plugin-dialog` | Native OS dialog, consistent with discard_file pattern |
| Scroll-to-element | Manual scrollTop calculation | `element.scrollIntoView()` | Browser native, handles nested scrollable containers |

**Key insight:** This phase has zero new dependencies. Every utility and pattern already exists in the project.

## Common Pitfalls

### Pitfall 1: Stale Hunk Index After Mutation
**What goes wrong:** User clicks "Stage Hunk" on hunk 2, while the operation runs the file changes, and the hunk indices shift. A concurrent click on another hunk button would use a stale index.
**Why it happens:** Git diff hunk indices are positional -- staging one hunk can change the index of subsequent hunks.
**How to avoid:** Disable ALL hunk buttons (`hunkOperationInFlight = true`) before any operation. Only re-enable after diff re-fetch completes (which provides new, correct hunk indices).
**Warning signs:** Two hunk operations running simultaneously, or buttons still enabled during an in-flight operation.

### Pitfall 2: DiffPanel Not Re-Fetching After Hunk Operation
**What goes wrong:** Hunk is staged but the diff view still shows the old hunks with stale indices.
**Why it happens:** DiffPanel does not own the diff data -- App.svelte fetches it via `refetchFileDiff`. If DiffPanel does not signal the parent after a hunk operation, the diff data becomes stale.
**How to avoid:** DiffPanel must call a callback prop (e.g., `onhunkaction`) that triggers `refetchFileDiff` in App.svelte AND `loadStatus()` in StagingPanel (to update file counts). The repo-changed FS watcher handles status refresh automatically but diff re-fetch must be explicit.
**Warning signs:** Diff view not updating after a hunk operation, or file counts not changing.

### Pitfall 3: Keyboard Shortcuts Firing in Text Inputs
**What goes wrong:** User types `[` or `]` in the commit message textarea and the diff jumps instead.
**Why it happens:** Keyboard listener is attached to `window` and captures all keystrokes.
**How to avoid:** Guard with `(e.target as HTMLElement)?.tagName` check -- skip `INPUT`, `TEXTAREA`, `SELECT`. This matches the project's existing keyboard shortcut pattern in App.svelte (which uses `e.metaKey` guard instead, but the principle is the same).
**Warning signs:** Typing brackets in commit form causes unexpected scroll behavior.

### Pitfall 4: Hunk Element References Breaking on Re-Render
**What goes wrong:** After a hunk operation, the diff re-fetches and Svelte re-renders the hunk list. The `hunkElements` references become stale.
**Why it happens:** `bind:this` references are cleared when the DOM element is destroyed during re-render.
**How to avoid:** Use a `Record<string, HTMLDivElement>` object that gets populated via `bind:this` on each render. Svelte 5 handles this correctly -- new `bind:this` assignments replace old ones. Reset `focusedHunkIndex` to 0 when fileDiffs change (use `$effect`).
**Warning signs:** `scrollToHunk` targeting a null element after a diff refresh.

### Pitfall 5: Forgetting diffKind Prop in All DiffPanel Call Sites
**What goes wrong:** DiffPanel renders without diffKind, defaulting to undefined, and no buttons appear.
**Why it happens:** DiffPanel is used in App.svelte line 405. The new prop must be passed there.
**How to avoid:** Derive diffKind from existing state: if `selectedCommitFile` is set, it is `'commit'`; otherwise use `selectedFile?.kind` which is already `'unstaged' | 'staged'`.
**Warning signs:** DiffPanel renders but never shows hunk action buttons.

## Code Examples

### Deriving diffKind in App.svelte
```svelte
<!-- In App.svelte, line ~405 where DiffPanel is used -->
<DiffPanel
  fileDiffs={currentDiffFiles}
  commitDetail={null}
  selectedPath={selectedCommitFile ?? selectedFile?.path ?? null}
  diffKind={selectedCommitFile ? 'commit' : selectedFile?.kind ?? 'commit'}
  repoPath={repoPath!}
  onhunkaction={async (filePath) => {
    if (selectedFile) {
      await refetchFileDiff(filePath, selectedFile.kind);
    }
  }}
  onclose={handleDiffClose}
/>
```

### Hunk Button Disabled Styling
```svelte
<button
  disabled={hunkOperationInFlight}
  style="
    background: none;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    color: var(--color-text);
    font-size: 11px;
    padding: 1px 6px;
    cursor: {hunkOperationInFlight ? 'not-allowed' : 'pointer'};
    opacity: {hunkOperationInFlight ? 0.4 : 1};
    white-space: nowrap;
  "
  onclick={() => handleStageHunk(fd.path, hunkIdx)}
>
  Stage Hunk
</button>
```

### Highlight Flash CSS
```svelte
<style>
  :global(.hunk-highlight) {
    animation: hunk-flash 0.6s ease-out;
  }
  @keyframes hunk-flash {
    0% { background-color: rgba(96, 165, 250, 0.3); }
    100% { background-color: transparent; }
  }
</style>
```

### Resetting Focused Hunk on File Change
```svelte
$effect(() => {
  // Reset focused hunk when the displayed diffs change
  fileDiffs;  // dependency tracking
  focusedHunkIndex = 0;
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Custom modal for destructive confirm | `ask()` from @tauri-apps/plugin-dialog | Phase 28 | Consistent native OS dialog |
| Boolean loading per-file | Single boolean for all hunk buttons | Phase 33 (new) | Prevents stale-index races across hunks |
| Plain @@ header text | Toolbar row with buttons | Phase 33 (new) | Enables hunk-level staging from diff view |

## Open Questions

1. **Should `onhunkaction` also trigger StagingPanel's `loadStatus()`?**
   - What we know: The `repo-changed` FS watcher event fires when git index changes, which triggers `loadStatus()` in StagingPanel automatically (via the `listen('repo-changed')` in StagingPanel). However, `refetchFileDiff` must be called explicitly because it requires the specific file path.
   - What's unclear: Whether the FS watcher fires fast enough for immediate UI feedback, or if there is a 200ms debounce delay (App.svelte line 228 has `setTimeout(..., 200)`).
   - Recommendation: Rely on the FS watcher for status refresh (it works today for file-level staging). Only call `refetchFileDiff` explicitly from `onhunkaction`. If timing feels slow during testing, add an explicit `loadStatus` call.

2. **Should focusedHunkIndex be a flat counter across all files, or per-file?**
   - What we know: For staging diffs, DiffPanel shows exactly one file's hunks (filtered by `selectedPath`). For commit diffs, it could show multiple files' hunks if `selectedCommitFile` filters to one file.
   - What's unclear: Whether a user would ever see multiple files in the hunk view simultaneously.
   - Recommendation: Use a flat array of all visible hunk elements across all files. In practice, only one file's hunks are shown at a time due to the selectedPath filtering.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest 4.1.0 (frontend), cargo test (backend) |
| Config file | `vite.config.ts` (test section) |
| Quick run command | `npx vitest run --reporter=verbose` |
| Full suite command | `npx vitest run && cd src-tauri && cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HUNK-04 | Context-appropriate buttons per diffKind | manual-only | Visual verification in app | N/A |
| HUNK-06 | Binary files show no hunk buttons | manual-only | Visual verification with binary file | N/A |
| HUNK-09 | [/] keyboard navigation between hunks | manual-only | Interactive keyboard testing | N/A |

**Note on test types:** All three requirements are UI-interaction behaviors (button rendering, DOM scrolling, keyboard events) that require a running Tauri app with a real git repository. The project's Vitest setup uses `environment: "node"` (not jsdom), making DOM-dependent component tests impractical. The backend hunk commands are already thoroughly tested (Tests 13-18 in `staging.rs`).

### Sampling Rate
- **Per task commit:** `npx vitest run`
- **Per wave merge:** `npx vitest run && cd src-tauri && cargo test`
- **Phase gate:** Full suite green + manual UAT of all three requirements

### Wave 0 Gaps
None -- no new test files needed. The phase requirements are UI-interaction behaviors verified through manual testing. Backend commands are already tested.

## Sources

### Primary (HIGH confidence)
- `src/components/DiffPanel.svelte` -- Current 155-line read-only component, direct inspection
- `src/App.svelte` -- DiffPanel usage site (line 405), `handleFileSelect`, `refetchFileDiff`, `selectedFile` state
- `src/components/StagingPanel.svelte` -- `safeInvoke` pattern, `loadingFiles` disabled pattern, `ask()` for discard
- `src-tauri/src/commands/staging.rs` -- Backend hunk commands (stage_hunk, unstage_hunk, discard_hunk) with signatures and error codes
- `src/lib/types.ts` -- FileDiff, DiffHunk type definitions with `is_binary` field
- `src/lib/invoke.ts` -- `safeInvoke<T>` and `TrunkError` interface
- `src/lib/toast.svelte.ts` -- `showToast(message, kind)` API

### Secondary (MEDIUM confidence)
- MDN `Element.scrollIntoView()` -- standard Web API, well-supported
- CSS `@keyframes` animation -- standard, no browser compatibility concerns

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use, no new dependencies
- Architecture: HIGH -- DiffPanel is simple (155 lines), modification path is clear
- Pitfalls: HIGH -- directly observed from code inspection (stale index, re-fetch, input guard)

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable -- no external dependencies changing)
