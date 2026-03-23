# Phase 43: Tech Debt Cleanup - Research

**Researched:** 2026-03-23
**Domain:** Dead code removal and cosmetic fixes (Rust/Svelte/TypeScript)
**Confidence:** HIGH

## Summary

Phase 43 is a mechanical cleanup phase with five independent items identified by the v0.8 milestone audit. Each item involves removing dead code or fixing minor bugs -- no new features, no new libraries, no architectural changes.

All five items are straightforward with zero risk of introducing regressions, provided the changes stay within the boundaries described. The `diff_conflicted` removal is the largest (deleting ~100 lines of Rust implementation + ~130 lines of tests + 1 registration line). The remaining four items are single-file edits of 1-10 lines each.

**Primary recommendation:** Execute all five items as independent tasks in a single plan, verified by `cargo test`, `bun run check`, and `bun run build`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Remove `diff_conflicted` backend command -- both the registration in `lib.rs` and the implementation in `commands/diff.rs` (including `diff_conflicted_inner` and its tests). Frontend uses `get_merge_sides` instead (Phase 38 decision).
- **D-02:** Remove dead `InputDialog` import from `App.svelte` line 11 -- component is used in CommitGraph, Toolbar, BranchSidebar but NOT in App.svelte itself.
- **D-03:** Fix `rebaseBaseName` lookup in `App.svelte` lines 411-416 -- the `allBranches.find()` callback always `return false`, so it always falls through to short OID. Should resolve the base ref to an actual branch name when possible.
- **D-04:** Clean up `submit_rebase_message` dead references -- grep shows no references remain in `src-tauri/src/`. Verify no frontend references either; if already clean, mark as done.
- **D-05:** Fix type mismatch at `App.svelte` line 590 -- `selectedFile?.kind` can be `'conflicted'` but DiffPanel's `diffKind` prop type is `'unstaged' | 'staged' | 'commit'`. Since MergeEditor intercepts `'conflicted'` files (line 67 `showMergeEditor` derived), DiffPanel never actually receives `'conflicted'` -- but the type needs to be correct. Either narrow the expression to exclude `'conflicted'` or guard the path.
- **D-06:** Each cleanup item is independent -- can be done in any order, no dependencies between items.
- **D-07:** Tests for `diff_conflicted` should be removed along with the implementation (tests 10 and 11 in `diff.rs`).

### Claude's Discretion
- Exact approach for fixing `rebaseBaseName` lookup (iterate refs comparing OIDs, or use a git2 API call)
- Whether to narrow the type union at the DiffPanel call site or add a guard

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

## Project Constraints (from CLAUDE.md)

- All git operations go through git2 crate, no shelling out (except GIT_EDITOR for rebase/merge message editing)
- Never inline colors -- always use CSS custom properties from the theme
- Never fight layout with positioning hacks -- use grid/flexbox so elements flow naturally
- Test commands: `bun run test` (vitest), `bun run check` (svelte-check), Rust tests via `cargo test` in `src-tauri/`
- Frontend paths: `$lib` maps to `src/lib`, commands in `src-tauri/src/commands/`

## Standard Stack

No new libraries needed. This phase only modifies existing code.

### Core (unchanged)
| Library | Version | Purpose | Relevance to Phase |
|---------|---------|---------|-------------------|
| Svelte 5 | current | Frontend framework | App.svelte modifications (D-02, D-03, D-05) |
| TypeScript 5.6 | strict | Type checking | D-05 type narrowing fix |
| git2 | 0.19 | Git operations | D-01 code removal |
| Tauri 2 | current | Backend framework | D-01 command deregistration |

## Architecture Patterns

### Files to Modify

```
src/
  App.svelte               # D-02: remove dead import (line 11)
                            # D-03: fix rebaseBaseName lookup (lines 410-420)
                            # D-05: fix diffKind type at line 590
src-tauri/src/
  lib.rs                    # D-01: remove diff_conflicted from invoke_handler (line 53)
  commands/diff.rs          # D-01: remove diff_conflicted_inner (lines 124-135),
                            #        diff_conflicted (lines 208-219),
                            #        test 10 (lines 452-562),
                            #        test 11 (lines 564-583)
```

### Pattern: Command Registration in lib.rs

Commands are registered in the `invoke_handler` macro call. Removing `diff_conflicted` means deleting the single line:
```rust
commands::diff::diff_conflicted,
```

### Pattern: Tauri Command Structure

Each command has an `_inner` function (synchronous, testable) and a `#[tauri::command]` async wrapper. Removing `diff_conflicted` means removing both `diff_conflicted_inner` (lines 124-135) and `diff_conflicted` (lines 208-219).

## Detailed Analysis Per Item

### D-01: Remove diff_conflicted

**What to delete:**
1. `src-tauri/src/lib.rs` line 53: `commands::diff::diff_conflicted,`
2. `src-tauri/src/commands/diff.rs` lines 124-135: `diff_conflicted_inner` function
3. `src-tauri/src/commands/diff.rs` lines 208-219: `diff_conflicted` Tauri command wrapper
4. `src-tauri/src/commands/diff.rs` test 10 (lines 452-562): `diff_conflicted_shows_conflict_markers`
5. `src-tauri/src/commands/diff.rs` test 11 (lines 564-583): `diff_conflicted_clean_file`

**Verification:** No frontend callers exist. Confirmed by grep -- zero references in `src/`. The MergeEditor loads data via `get_merge_sides` instead (Phase 38 decision).

**Confidence:** HIGH -- grep confirms zero frontend references; this is dead code.

### D-02: Remove dead InputDialog import

**What to delete:**
- `src/App.svelte` line 11: `import InputDialog from './components/InputDialog.svelte';`

**Verification:** `InputDialog` is used in:
- `src/components/CommitGraph.svelte` (line 21, 1270)
- `src/components/BranchSidebar.svelte` (line 8, 591)
- `src/components/Toolbar.svelte` (line 9, 256)

It is NOT used anywhere in `App.svelte` beyond the import itself.

**Confidence:** HIGH -- grep confirms the component is imported but never referenced in App.svelte template or script.

### D-03: Fix rebaseBaseName lookup

**Current broken code (App.svelte lines 410-420):**
```typescript
const refs = await safeInvoke<RefsResponse>('list_refs', { path: repoPath! });
const allBranches = [...refs.local, ...refs.remote];
const baseRef = allBranches.find(b => {
  // Try to match OID - need to resolve ref to OID
  return false; // fallback below
});
rebaseBaseName = baseOid.slice(0, 7);
```

The `find()` callback always returns `false`, so `baseRef` is always undefined, and `rebaseBaseName` always falls through to the short OID.

**Root cause:** `BranchInfo` does not include an OID field. The code was left as a stub because there was no direct way to compare `baseOid` against branch tip OIDs using only the `list_refs` response.

**Recommended fix (Claude's discretion -- using `resolve_ref`):** Use the existing `resolve_ref` Tauri command to resolve each branch name to an OID and find a match:

```typescript
try {
  const refs = await safeInvoke<RefsResponse>('list_refs', { path: repoPath! });
  const allBranches = [...refs.local, ...refs.remote];
  let foundName: string | null = null;
  for (const b of allBranches) {
    try {
      const branchOid = await safeInvoke<string>('resolve_ref', { path: repoPath!, refName: b.name });
      if (branchOid === baseOid) {
        foundName = b.name;
        break;
      }
    } catch {
      // ref resolution failed -- skip
    }
  }
  rebaseBaseName = foundName ?? baseOid.slice(0, 7);
} catch {
  rebaseBaseName = baseOid.slice(0, 7);
}
```

**Performance note:** This is acceptable because:
- It only runs once when opening the rebase editor (not on every render)
- Most repos have <50 branches
- `resolve_ref` is fast (single `revparse_single` call)
- It short-circuits on first match

**Where `baseName` is displayed:** `RebaseEditor.svelte` line 406 renders it as `<span class="rebase-branch-pill">{baseName}</span>` in the toolbar text "Rebasing {branchName} onto {baseName}".

**Confidence:** HIGH -- clear understanding of root cause and fix path.

### D-04: submit_rebase_message dead references

**Status: Already clean in source code.**

Grep results:
- `src/` directory: 0 references
- `src-tauri/src/` directory: 0 references
- Only references exist in `.planning/` documentation files (plans, summaries, research docs)

The function was planned but never implemented. The `newMessage` field in the `onstart` payload superseded it (Phase 41 decision).

**Action:** Verify and mark as done. No code changes needed.

**Confidence:** HIGH -- confirmed by grep across all source directories.

### D-05: Fix diffKind type mismatch

**Current code (App.svelte line 590):**
```typescript
diffKind={selectedCommitFile ? 'commit' : (selectedFile?.kind ?? 'commit')}
```

**Problem:** `selectedFile?.kind` is typed as `'unstaged' | 'staged' | 'conflicted'`, but `DiffPanel`'s `diffKind` prop accepts `'unstaged' | 'staged' | 'commit'`. If `kind` is `'conflicted'`, this would pass an invalid value.

**Why it doesn't crash at runtime:** The `showMergeEditor` derived (line 67) is `true` when `selectedFile?.kind === 'conflicted'`, so the `{#if showMergeEditor}` branch (line 578) catches it first. The DiffPanel branch at line 585 (`{:else if showDiff}`) only executes when `kind` is NOT `'conflicted'`.

**Recommended fix (Claude's discretion -- narrow the type):** Since `showMergeEditor` guarantees `selectedFile.kind !== 'conflicted'` in the `else if` branch, we can safely exclude `'conflicted'` with a type assertion or a ternary guard:

```typescript
diffKind={selectedCommitFile ? 'commit' : (selectedFile?.kind === 'conflicted' ? 'commit' : selectedFile?.kind ?? 'commit')}
```

This makes the type flow correct for TypeScript while being functionally identical (the `'conflicted'` case is unreachable, but the guard satisfies the type checker).

**Confidence:** HIGH -- the control flow guarantees this is unreachable, just a type-level fix.

## Don't Hand-Roll

Not applicable -- this phase removes code, it does not build anything.

## Common Pitfalls

### Pitfall 1: Removing wrong test boundaries in diff.rs
**What goes wrong:** Deleting test 10 or 11 but accidentally cutting into test 9 or leaving dangling brackets.
**Why it happens:** The tests are sequential in the file and use complex setup code.
**How to avoid:** Delete from the `// Test 10:` comment (line 452) through the end of `// Test 11`'s closing brace (line 583). Verify remaining tests still compile.
**Warning signs:** `cargo test` fails with syntax errors.

### Pitfall 2: Breaking the diffKind type fix at the wrong layer
**What goes wrong:** Changing DiffPanel's Props to accept `'conflicted'` instead of narrowing at the call site.
**Why it happens:** Seems simpler to widen the accepting type.
**How to avoid:** DiffPanel has no handling for `'conflicted'` kind -- its hunk action buttons (stage/unstage/discard) would be wrong. The fix must be at the App.svelte call site, not in DiffPanel.
**Warning signs:** `bun run check` passes but conflict files show wrong actions.

### Pitfall 3: rebaseBaseName fix creating N+1 IPC calls
**What goes wrong:** Calling `resolve_ref` in a loop for every branch on every render.
**Why it happens:** Putting the resolution in a reactive block instead of the one-time `handleOpenRebaseEditor` function.
**How to avoid:** Keep the resolution inside `handleOpenRebaseEditor` where it currently lives -- it only runs once when the user opens the editor.
**Warning signs:** Performance degradation on repos with many branches.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework (TS) | vitest 4.1.0 |
| Framework (Rust) | cargo test (built-in) |
| Config file (TS) | none (vitest defaults via package.json) |
| Quick run command (TS) | `bun run test` |
| Quick run command (Rust) | `cd src-tauri && cargo test` |
| Type check | `bun run check` |

### Phase Requirements to Test Map
| Item | Behavior | Test Type | Automated Command | File Exists? |
|------|----------|-----------|-------------------|-------------|
| D-01 | diff_conflicted removed, remaining diff tests pass | unit (Rust) | `cd src-tauri && cargo test diff` | Existing tests minus 2 deleted |
| D-02 | Dead import removed | type-check | `bun run check` | N/A |
| D-03 | rebaseBaseName shows branch name | manual | Open interactive rebase, verify toolbar shows branch name not OID | N/A |
| D-04 | No submit_rebase_message refs in source | grep | `grep -r submit_rebase_message src/ src-tauri/src/` | N/A |
| D-05 | No type error on diffKind | type-check | `bun run check` | N/A |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test` + `bun run check`
- **Phase gate:** `cd src-tauri && cargo test` + `bun run check` + `bun run build`

### Wave 0 Gaps
None -- existing test infrastructure covers all phase requirements.

## Code Examples

### D-01: Lines to remove from lib.rs

```rust
// DELETE this line from invoke_handler:
commands::diff::diff_conflicted,
```

### D-01: Lines to remove from diff.rs

```rust
// DELETE: diff_conflicted_inner (lines 124-135)
pub fn diff_conflicted_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<Vec<FileDiff>, TrunkError> {
    // ... entire function
}

// DELETE: diff_conflicted Tauri wrapper (lines 208-219)
#[tauri::command]
pub async fn diff_conflicted(
    path: String,
    file_path: String,
    state: State<'_, RepoState>,
) -> Result<Vec<FileDiff>, String> {
    // ... entire function
}

// DELETE: Test 10 "diff_conflicted_shows_conflict_markers" (lines 452-562)
// DELETE: Test 11 "diff_conflicted_clean_file" (lines 564-583)
```

### D-02: Line to remove from App.svelte

```typescript
// DELETE line 11:
import InputDialog from './components/InputDialog.svelte';
```

### D-03: Fixed rebaseBaseName lookup

```typescript
// REPLACE lines 410-420 in handleOpenRebaseEditor:
try {
  const refs = await safeInvoke<RefsResponse>('list_refs', { path: repoPath! });
  const allBranches = [...refs.local, ...refs.remote];
  let foundName: string | null = null;
  for (const b of allBranches) {
    try {
      const branchOid = await safeInvoke<string>('resolve_ref', { path: repoPath!, refName: b.name });
      if (branchOid === baseOid) {
        foundName = b.name;
        break;
      }
    } catch {
      // skip
    }
  }
  rebaseBaseName = foundName ?? baseOid.slice(0, 7);
} catch {
  rebaseBaseName = baseOid.slice(0, 7);
}
```

### D-05: Fixed diffKind expression

```typescript
// REPLACE line 590:
diffKind={selectedCommitFile ? 'commit' : (selectedFile?.kind === 'conflicted' ? 'commit' : selectedFile?.kind ?? 'commit')}
```

## Open Questions

None -- all five items are well-understood with clear implementation paths.

## Sources

### Primary (HIGH confidence)
- Direct source code inspection of `src/App.svelte`, `src-tauri/src/lib.rs`, `src-tauri/src/commands/diff.rs`, `src/components/DiffPanel.svelte`, `src/components/RebaseEditor.svelte`, `src-tauri/src/commands/branches.rs`, `src/lib/types.ts`
- Grep verification across `src/` and `src-tauri/src/` for all dead reference claims
- `.planning/phases/43-tech-debt-cleanup/43-CONTEXT.md` for locked decisions

## Metadata

**Confidence breakdown:**
- D-01 (diff_conflicted removal): HIGH -- dead code confirmed by grep, straightforward deletion
- D-02 (InputDialog import): HIGH -- dead import confirmed by grep
- D-03 (rebaseBaseName fix): HIGH -- root cause understood, fix uses existing `resolve_ref` command
- D-04 (submit_rebase_message): HIGH -- already clean, confirmed by grep
- D-05 (diffKind type fix): HIGH -- control flow guarantees safety, type-level fix only

**Research date:** 2026-03-23
**Valid until:** Indefinite -- this is a cleanup phase with no external dependencies
