# Phase 43: Tech Debt Cleanup - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Clean up orphaned code and dead imports accumulated during v0.8 phases (37-42). No new features — purely removing dead code and fixing cosmetic issues.

</domain>

<decisions>
## Implementation Decisions

### Cleanup scope
- **D-01:** Remove `diff_conflicted` backend command — both the registration in `lib.rs` and the implementation in `commands/diff.rs` (including `diff_conflicted_inner` and its tests). Frontend uses `get_merge_sides` instead (Phase 38 decision).
- **D-02:** Remove dead `InputDialog` import from `App.svelte` line 11 — component is used in CommitGraph, Toolbar, BranchSidebar but NOT in App.svelte itself.
- **D-03:** Fix `rebaseBaseName` lookup in `App.svelte` lines 411-416 — the `allBranches.find()` callback always `return false`, so it always falls through to short OID. Should resolve the base ref to an actual branch name when possible.
- **D-04:** Clean up `submit_rebase_message` dead references — grep shows no references remain in `src-tauri/src/`. Verify no frontend references either; if already clean, mark as done.
- **D-05:** Fix type mismatch at `App.svelte` line 590 — `selectedFile?.kind` can be `'conflicted'` but DiffPanel's `diffKind` prop type is `'unstaged' | 'staged' | 'commit'`. Since MergeEditor intercepts `'conflicted'` files (line 67 `showMergeEditor` derived), DiffPanel never actually receives `'conflicted'` — but the type needs to be correct. Either narrow the expression to exclude `'conflicted'` or guard the path.

### Approach
- **D-06:** Each cleanup item is independent — can be done in any order, no dependencies between items.
- **D-07:** Tests for `diff_conflicted` should be removed along with the implementation (tests 10 and 11 in `diff.rs`).

### Claude's Discretion
- Exact approach for fixing `rebaseBaseName` lookup (iterate refs comparing OIDs, or use a git2 API call)
- Whether to narrow the type union at the DiffPanel call site or add a guard

</decisions>

<canonical_refs>
## Canonical References

No external specs — requirements fully captured in decisions above.

### Audit source
- `.planning/v0.8-MILESTONE-AUDIT.md` — Original audit that identified these tech debt items

</canonical_refs>

<code_context>
## Existing Code Insights

### Files to modify
- `src-tauri/src/lib.rs` — Line 53: `diff_conflicted` command registration
- `src-tauri/src/commands/diff.rs` — Lines 124-216+: `diff_conflicted_inner` and `diff_conflicted` implementations, tests at lines 452-574
- `src/App.svelte` — Line 11 (dead InputDialog import), lines 411-416 (dead rebaseBaseName lookup), line 590 (type mismatch)

### Established Patterns
- Command registration: `lib.rs` registers all Tauri commands in `invoke_handler`
- Dead code removal: straightforward deletion, no feature flags or deprecation needed

### Integration Points
- `diff_conflicted` removal: only registered in `lib.rs` and implemented in `diff.rs`. No frontend callers exist (verified by grep).
- `InputDialog`: actively used in CommitGraph, Toolbar, BranchSidebar — only the App.svelte import is dead.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. This is a mechanical cleanup phase.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 43-tech-debt-cleanup*
*Context gathered: 2026-03-23*
