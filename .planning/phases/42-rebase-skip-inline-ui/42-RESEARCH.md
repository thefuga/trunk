# Phase 42: Rebase Skip in Inline UI - Research

**Researched:** 2026-03-23
**Domain:** Svelte 5 UI (StagingPanel inline rebase form), Tauri IPC
**Confidence:** HIGH

## Summary

This is a narrowly scoped UI-only phase. The backend `rebase_skip` IPC command already exists and is registered. The CSS custom properties (`--color-btn-skip-*`) already exist. The `rebaseLoading` state already exists and is shared between Continue and Abort. The only work is: (1) add a Skip Commit button to StagingPanel's inline rebase button row between Continue and Abort, (2) write a `skipRebase()` handler following the established pattern, and (3) remove the success toast from OperationBanner's `handleSkip()` for consistency with Phase 40's silent pattern.

All building blocks are in place. No new libraries, no new Rust code, no new CSS tokens. The implementation is copy-adapt from existing handlers.

**Primary recommendation:** Follow the `continueRebase()` handler pattern exactly -- set `rebaseLoading`, call `safeInvoke('rebase_skip', { path: repoPath })`, error toast on failure, `loadStatus()` in finally. Insert the button between Continue and Abort with `flex: 1`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Skip button goes **between Continue and Abort**: Continue Rebase | Skip Commit | Abort Rebase
- Flex ratios: Continue flex:3, Skip flex:1, Abort flex:2 (Skip is the narrowest -- secondary action)
- Label: **"Skip Commit"** -- explicit about what is being skipped
- Same height as Continue/Abort (34px)
- Reuse existing **--color-btn-skip-bg, --color-btn-skip, --color-btn-skip-border** CSS custom properties
- Same font-size (12px) and font-weight (600) as Continue/Abort
- Skip is **always enabled** during rebase -- not gated on conflict state (no `allResolved` check)
- Only disabled when `rebaseLoading` is true (same as Abort)
- No confirmation dialog
- **Silent -- no success toast** for Skip in StagingPanel
- **Also remove** the existing "Commit skipped" toast from OperationBanner's `handleSkip()` for consistency
- Error toast on failure (consistent with all rebase actions)

### Claude's Discretion
- Whether `skipRebase()` function in StagingPanel calls `rebase_skip` directly or delegates to OperationBanner
- Loading state management (can share `rebaseLoading` flag)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| REB-06 | User can skip a conflicting commit during rebase and continue with the next commit | Backend `rebase_skip` IPC already exists; needs StagingPanel UI button + handler; OperationBanner toast removal for consistency |
</phase_requirements>

## Standard Stack

No new dependencies. This phase uses only what is already in the project.

### Core (already installed)
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| Svelte 5 | runes | StagingPanel component | Already in use |
| Tauri 2 | IPC | `safeInvoke('rebase_skip', ...)` | Already registered |
| CSS custom properties | -- | `--color-btn-skip-*` tokens | Already defined in app.css |

### Installation
None required.

## Architecture Patterns

### Pattern 1: Rebase Action Handler (copy from `continueRebase`)

**What:** The StagingPanel has an established pattern for rebase action handlers.
**When to use:** For the new `skipRebase()` function.
**Source:** `src/components/StagingPanel.svelte` lines 268-286 (`continueRebase`)

```typescript
async function skipRebase() {
  rebaseLoading = true;
  try {
    await safeInvoke('rebase_skip', { path: repoPath });
    // No success toast -- Phase 40 silent pattern
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Skip failed', 'error');
  } finally {
    rebaseLoading = false;
    await loadStatus();
  }
}
```

Key differences from `continueRebase()`:
- No message parameter (skip does not need a commit message)
- No success toast (silent pattern)
- No `allResolved` gate on the button (skip is always enabled)

Key differences from `abortRebase()`:
- No confirmation dialog (skip is non-destructive forward action)

### Pattern 2: Button Row Layout (existing flex pattern)

**What:** The button row at StagingPanel line 777 uses flex with gap for action buttons.
**Current:** Two buttons -- Continue (flex:3) and Abort (flex:2).
**New:** Three buttons -- Continue (flex:3), Skip (flex:1), Abort (flex:2).

The Skip button uses the same styling template as Continue/Abort but with `--color-btn-skip-*` tokens.

### Pattern 3: OperationBanner Toast Removal

**What:** OperationBanner's `handleSkip()` at line 49 currently shows `showToast('Commit skipped', 'success')`.
**Change:** Remove this line to align with Phase 40's silent success pattern (no toast on rebase operations).
**Impact:** Only the success toast is removed. Error toast via catch block remains.

### Anti-Patterns to Avoid
- **Gating Skip on `allResolved`:** Skip must NOT check `allResolved`. Its purpose is to skip the current conflicting commit entirely, including its conflicts. Only `rebaseLoading` gates it.
- **Adding confirmation dialog to Skip:** Per Phase 37 decision, Skip is a non-destructive forward action and needs no confirmation (unlike Abort).
- **Delegating to OperationBanner:** StagingPanel should call `rebase_skip` directly via `safeInvoke`, not route through OperationBanner. This keeps the pattern consistent with how `continueRebase()` and `abortRebase()` work -- they call IPC directly.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Skip IPC call | New Rust command | Existing `rebase_skip` | Already implemented and registered |
| Button styling | Custom CSS | `--color-btn-skip-*` tokens | Already defined in app.css line 45-47 |
| Loading state | New state variable | Existing `rebaseLoading` | Already shared by Continue/Abort |
| UI refresh after skip | Manual refresh logic | Existing `loadStatus()` in finally block | Established pattern from continueRebase/abortRebase |

## Common Pitfalls

### Pitfall 1: Forgetting to Remove OperationBanner Toast
**What goes wrong:** Skip works in StagingPanel (no toast) but OperationBanner still shows "Commit skipped" toast, creating inconsistent behavior.
**Why it happens:** Easy to focus only on StagingPanel and forget the consistency fix.
**How to avoid:** Plan explicitly includes OperationBanner modification.
**Warning signs:** Testing skip from banner shows toast but skip from staging panel does not.

### Pitfall 2: Gating Skip Button on allResolved
**What goes wrong:** Skip button is disabled when conflicts exist, defeating its purpose.
**Why it happens:** Copy-pasting from Continue button which has `disabled={rebaseLoading || !allResolved}`.
**How to avoid:** Skip button disabled condition is ONLY `rebaseLoading` (same as Abort, not Continue).
**Warning signs:** Skip button grayed out when there are conflicted files.

### Pitfall 3: Wrong Flex Order
**What goes wrong:** Button order is wrong (e.g., Skip after Abort instead of between Continue and Abort).
**Why it happens:** Adding the new button at the end of the flex container instead of between.
**How to avoid:** Insert the Skip button markup between the Continue and Abort button elements in the template.

## Code Examples

### Exact Button Markup (to insert between Continue and Abort)

```svelte
<button
  onclick={skipRebase}
  disabled={rebaseLoading}
  style="
    flex: 1;
    height: 34px;
    background: var(--color-btn-skip-bg);
    color: var(--color-btn-skip);
    border: 1px solid var(--color-btn-skip-border);
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: {rebaseLoading ? 'not-allowed' : 'pointer'};
    opacity: {rebaseLoading ? 0.4 : 1};
  "
>
  Skip Commit
</button>
```

### OperationBanner handleSkip After Fix

```typescript
async function handleSkip() {
  loading = true;
  try {
    await safeInvoke('rebase_skip', { path: repoPath });
    // No success toast -- silent pattern (Phase 40)
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Skip failed', 'error');
  } finally {
    loading = false;
    onaction?.();
  }
}
```

### Existing CSS Tokens (no changes needed)

```css
/* src/app.css lines 45-47 */
--color-btn-skip: #fbbf24;
--color-btn-skip-bg: rgba(251, 191, 36, 0.15);
--color-btn-skip-border: rgba(251, 191, 36, 0.3);
```

## State of the Art

No changes in approach -- this phase uses established patterns from Phase 37 and Phase 40.

| Component | Pattern Source | Phase |
|-----------|---------------|-------|
| Handler structure | `continueRebase()` | Phase 40 |
| No confirmation | Skip = non-destructive | Phase 37 decision |
| Silent success | No toast on rebase ops | Phase 40 decision |
| CSS tokens | `--color-btn-skip-*` | Phase 37 |

## Open Questions

None. All implementation details are fully specified by the CONTEXT.md decisions and existing code patterns.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | vitest (via vite.config.ts) |
| Config file | `vite.config.ts` test section |
| Quick run command | `bun run test` |
| Full suite command | `bun run test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REB-06 | Skip button appears in StagingPanel rebase form | manual-only | Visual inspection in dev mode | N/A |
| REB-06 | Skip invokes rebase_skip IPC | manual-only | Trigger during active rebase in dev mode | N/A |
| REB-06 | OperationBanner skip toast removed | manual-only | Trigger skip from banner, verify no toast | N/A |

**Manual-only justification:** This phase modifies Svelte component templates and event handlers in StagingPanel and OperationBanner. The project's test suite (vitest with node environment) tests utility functions and pure logic modules, not component rendering or IPC integration. These changes are best validated by visual inspection during an active rebase.

### Sampling Rate
- **Per task commit:** `bun run test` (ensure no regressions)
- **Per wave merge:** `bun run test && bun run check` (full type check)
- **Phase gate:** `bun run test && bun run check` + manual verification of Skip button behavior

### Wave 0 Gaps
None -- no new test files needed. Existing test infrastructure covers all utility code; the new code is UI template + thin IPC handler with no unit-testable logic.

## Sources

### Primary (HIGH confidence)
- **Source code inspection:** `src/components/StagingPanel.svelte` -- lines 266-307 (rebase handlers), lines 730-815 (rebase UI template)
- **Source code inspection:** `src/components/OperationBanner.svelte` -- lines 45-57 (handleSkip), lines 130-174 (button markup)
- **Source code inspection:** `src-tauri/src/commands/operation_state.rs` -- lines 221-240 (rebase_skip_inner), lines 420-438 (rebase_skip command)
- **Source code inspection:** `src/app.css` -- lines 45-47 (--color-btn-skip-* tokens)
- **Phase 42 CONTEXT.md** -- all implementation decisions locked

### Secondary (MEDIUM confidence)
- **Phase 37 CONTEXT.md** -- referenced for Skip = non-destructive, no confirmation decision
- **Phase 40 CONTEXT.md** -- referenced for silent success pattern (no toast on rebase operations)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all components already exist in the codebase, verified via source inspection
- Architecture: HIGH -- exact handler pattern and button template derived from existing code
- Pitfalls: HIGH -- based on direct analysis of the code to be modified

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable -- no external dependencies, all internal code)
