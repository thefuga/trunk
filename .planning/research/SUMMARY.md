# Project Research Summary

**Project:** Trunk v0.6 — UI Polish & Core Ops
**Domain:** Desktop Git GUI (Tauri 2 + Svelte 5 + Rust/git2)
**Researched:** 2026-03-15
**Confidence:** HIGH

## Executive Summary

Trunk v0.6 is a polish milestone that bridges the gap between "functional prototype" and "daily driver." The core architecture (commit graph, staging, commits, remotes) is proven from v0.5. This release adds the missing everyday operations users expect from any Git GUI — discard changes, branch/tag delete — while replacing amateur Unicode symbols with proper icons throughout the UI, unifying the commit/amend/stash workflow, and fixing visual bugs. The research confirms that **zero new Rust crates and only one new npm package (`@lucide/svelte`)** are needed. All new git operations use existing `git2 = "0.19"` APIs, and the dialog/notification system extends existing `@tauri-apps/plugin-dialog` patterns plus a lightweight custom toast (~30 LOC).

The recommended approach is to build in dependency order: icon system first (it's referenced by every subsequent UI change), then new Rust commands for destructive operations (discard, branch delete, tag delete), then staging UX improvements (three-way selector, button styling), then graph polish and bug fixes. Each new backend command follows the established `*_inner()` pattern with `state_map` and cache rebuild. The critical insight from architecture research is that this is an **integration milestone** — every feature plugs into existing patterns rather than requiring new architecture. The risk profile is "will additions break what works" rather than "will the architecture hold."

The top risks are: (1) discard implementation must branch by file status type — `checkout_head()` silently ignores untracked files, requiring `std::fs::remove_file()` as a separate path; (2) `git2::Branch::delete()` has NO merge-safety check (unlike `git branch -d`), so unmerged work can be silently lost without UI guardrails; (3) CSS `position: sticky` does not work inside the virtual list's `transform: translateY()` container, so the graph overflow feature must use flex/overflow layout instead; (4) the custom title bar merge is platform-specific and high-risk — defer full implementation, do CSS-only unification first.

## Key Findings

### Recommended Stack

One new dependency, zero Rust crate changes. The existing stack handles everything v0.6 needs.

**Core technologies:**
- **`@lucide/svelte@^0.577.0`**: SVG icon components — Svelte 5-native (`$props()` runes), 1500+ icons, tree-shakable, offline-compatible (critical for desktop app). Replaces inconsistent Unicode symbols across toolbar, sidebar, staging panel, and commit form.
- **`git2 = "0.19"` (existing)**: All new operations — `Branch::delete()`, `Repository::tag_delete()`, `checkout_head()` with `CheckoutBuilder::force().path()` for discard, `std::fs::remove_file()` for untracked files.
- **`@tauri-apps/plugin-dialog` (existing)**: `ask()` for destructive confirmations (discard, delete). Already used in 9+ places. Extend pattern, don't replace.
- **Custom `<Toast>` component (build it, ~30 LOC)**: Non-blocking feedback ("Branch deleted", "Changes discarded"). Uses existing `$state` rune + shared module pattern from `remote-state.svelte.ts`. Not worth a dependency.

**What NOT to add:** `svelte-sonner`/`svelte-french-toast` (SSR-oriented, overkill), `@iconify/svelte` (fetches from remote API — fails offline), `lucide-svelte` without `@lucide/` scope (Svelte 4, not 5), any animation library (built-in `svelte/transition` suffices), `tauri-plugin-notification` (too heavy for in-app feedback).

### Expected Features

**Must have (table stakes):**
- **Icon system throughout UI** — Every competitor uses icons; Unicode symbols look unpolished and render inconsistently cross-platform. Single highest-impact visual polish item (~35 icon placements).
- **Discard changes (file-level + all)** — GitKraken, Fork, Tower, Sublime Merge all have it. Most common "missing feature" for staging workflows. Must confirm before executing — destructive and unrecoverable.
- **Branch delete** — Context menu → confirmation → delete. Every Git GUI supports it. Must prevent deleting HEAD branch.
- **Tag delete** — Same pattern as branch delete. Table stakes for any GUI with tag display.
- **Confirmation dialogs for destructive ops** — Already partially implemented (stash drop). Extend to discard and delete.
- **Stage All / Unstage All with clear visual affordance** — Colored icon buttons replacing text-only buttons.

**Should have (differentiators):**
- **Three-way commit/amend/stash selector** — GitKraken's best UX pattern. Unifies "save my work" in one location. Replaces amend checkbox + separate stash action.
- **Click refs in sidebar → navigate graph** — Every competitor does this (double-click branch scrolls to its HEAD commit). Requires `scrollToIndex` on virtual list + OID lookup.
- **Graph overflow with clipped graph column** — Right-side columns (message, author, date) never scroll off-screen.
- **Right pane auto-opens on content change** — Prevents "I clicked but nothing happened" confusion.
- **Merged top bar (tab + actions)** — Saves 36px vertical space.

**Defer (v0.7+):**
- Hunk-level discard (ships with hunk staging)
- Remote branch/tag delete (too destructive, affects collaborators)
- Branch rename (separate feature with edge cases)
- Toast notification queue with animations (over-engineering for v0.6)
- Custom title bar with native window controls (high-risk, platform-specific)

### Architecture Approach

v0.6 is an integration milestone — features plug into the existing three-layer architecture (Svelte 5 frontend → IPC via `safeInvoke` → Rust/git2 backend with managed state). No new architectural patterns needed. New Rust commands follow the established `*_inner()` + `state_map` + `cache_map` + `app.emit('repo-changed')` pattern. New frontend state uses the proven `$state` rune shared module pattern. The icon system is a centralized `icons.ts` module with an `Icon.svelte` wrapper — raw SVG path data for maximum flexibility across HTML and SVG overlay contexts.

**New Rust commands (4):**
1. `discard_file` / `discard_all_unstaged` in `staging.rs` — revert working tree files via `checkout_head(force)` or `fs::remove_file()`
2. `delete_branch` in `branches.rs` — `Branch::delete()` with HEAD guard and cache rebuild
3. `delete_tag` in `commit_actions.rs` — `Repository::tag_delete(short_name)` with cache rebuild

**New frontend components (2):**
1. `Icon.svelte` — thin SVG wrapper using centralized `ICONS` path data
2. `Toast.svelte` — fixed-position notification stack, auto-dismiss

**New modules (2):**
1. `src/lib/icons.ts` — centralized icon SVG path data
2. `src/lib/toast-state.svelte.ts` — shared reactive toast state

**Modified components (10):** Toolbar, FileRow, StagingPanel, CommitForm, BranchSidebar, BranchRow, BranchSection, CommitGraph, TabBar, App.svelte

**Bug fixes (3, Rust):**
1. Add `WT_NEW` to `get_dirty_counts` unstaged bitmask (1-line fix)
2. Add `include_untracked(true)` to `diff_unstaged` options (1-line fix)
3. SVG `overflow="visible"` for ref pill overflow badge

### Critical Pitfalls

1. **Discard silently ignores untracked files** — `checkout_head()` restores tracked files but does nothing for `WT_NEW` files. Must branch by file status: tracked → `checkout_head(force, path)`, untracked → `std::fs::remove_file()`. Never use `remove_untracked(true)` without `path()` filtering. Always use `force()` not `safe()`.

2. **`Branch::delete()` has no merge safety check** — Unlike `git branch -d`, git2's `Branch::delete()` always succeeds for non-HEAD branches, silently deleting unmerged work. Must check `is_head()` before delete. Should verify merge status via `graph_descendant_of()` and warn user about unmerged commits.

3. **`tag_delete()` requires short name, not full ref** — Passing `refs/tags/v1.0.0` causes double-prefix error. Always use `short_name` from frontend. Add defensive `strip_prefix("refs/tags/")` in backend.

4. **`position: sticky` broken inside virtual list** — The virtual list's `transform: translateY()` creates a new containing block, breaking sticky positioning. Use `overflow: hidden` on graph column + flex layout instead.

5. **Custom title bar breaks macOS traffic lights and drag** — `decorations: false` removes traffic lights entirely. Use `titleBarStyle: "overlay"` instead, add `data-tauri-drag-region`, add 70px left padding. Start macOS-only, defer Windows/Linux. **Recommend CSS-only bar merge for v0.6.**

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation — Icons, Toast System & Bug Fixes
**Rationale:** Icon system is referenced by every subsequent UI change. Toast system is needed for feedback across all new features. WIP bug fix is trivial and high-value. These have zero dependencies on other v0.6 work.
**Delivers:** Consistent visual vocabulary across the entire app; non-blocking notification infrastructure; fix for missing WIP rows with untracked files.
**Addresses:** Icon system (table stakes), dialog/notification system, untracked files WIP bug.
**Avoids:** P7 (icon inconsistency) — migrating ALL Unicode symbols in one pass prevents mix of old and new styles.

### Phase 2: Destructive Operations — Discard, Branch Delete, Tag Delete
**Rationale:** These are the highest-value missing features (table stakes). All are independent Rust commands that follow established patterns. Each needs confirmation dialogs (from Phase 1 toast/dialog infrastructure). Should be built backend-first then wired to frontend.
**Delivers:** Discard file/all, branch delete with HEAD guard, tag delete with short-name handling.
**Addresses:** Discard changes, branch delete, tag delete, confirmation dialogs (all table stakes).
**Avoids:** P1 (discard untracked handling), P2 (HEAD branch guard + merge check), P3 (tag name format), P10 (discard confirmation required).

### Phase 3: Staging & Commit UX
**Rationale:** Depends on icons (Phase 1). The three-way selector is the highest-complexity frontend change and needs dedicated focus. Staging button improvements and equal-height lists are natural companions.
**Delivers:** Unified commit/amend/stash workflow, polished staging panel with colored buttons, stash name auto-fill.
**Addresses:** Three-way selector (differentiator), staging button improvements, equal-height lists, stash name defaults.
**Avoids:** P8 (state machine complexity — model as explicit discriminated state with 6 transition handlers), P11 (equal-height empty list — collapse empty sections to header-only).

### Phase 4: Graph Polish & Navigation
**Rationale:** Independent of Phases 2-3. Graph changes are CSS/layout-focused with some SVG work. Ref navigation requires adding `oid` to `BranchInfo` (Rust + TypeScript type change).
**Delivers:** Graph padding, graph overflow handling, sidebar ref → graph navigation, better tag icon, overflow badge z-index fix.
**Addresses:** Graph overflow/sticky columns (differentiator), click refs → navigate (differentiator), graph padding, tag pill icon, overflow pill z-index bug.
**Avoids:** P4 (no sticky positioning — use flex overflow), P9 (SVG overflow="visible" for badges), P13 (ref nav to unloaded commits — load in batches with progress).

### Phase 5: Layout & Remaining Polish
**Rationale:** Lowest priority, highest risk (title bar is platform-specific). Do last so it doesn't block other work. Bug fixes can be interleaved throughout.
**Delivers:** Merged top bar (CSS-only for v0.6), right pane auto-open, trailing header divider fix.
**Addresses:** Merged top bar (differentiator), right pane auto-open (differentiator), trailing divider bug.
**Avoids:** P5 (title bar merge — CSS-only approach avoids platform-specific breakage), P6 (dialog focus context — keep native dialogs for confirmations, only add toasts for feedback).

### Phase Ordering Rationale

- **Icons first** because every subsequent UI change references the icon system. Doing icons later means double-touching every component.
- **Destructive operations before UX polish** because they're the highest-value missing features and have well-established patterns across all competitor GUIs.
- **Three-way selector gets dedicated Phase 3** because it has 6 mode transition paths, each with different form-state semantics — highest frontend complexity.
- **Graph polish is independent** and can be parallelized with Phase 3 if resources allow.
- **Title bar merge is last** because it's the only feature with platform-specific risk. A CSS-only approach de-risks it; full custom title bar deferred to v0.7.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (Graph Overflow):** CSS interaction between virtual list, SVG overlay, and flex layout is non-trivial. May need experimentation with `overflow: hidden` vs `clip-path` approaches. Test with 8+ parallel branches.
- **Phase 4 (Ref Navigation):** Pagination interaction — target commit may not be loaded. Need to design load-until-found behavior with progress indicator for old tags.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Icons):** Well-documented `@lucide/svelte` API, straightforward component migration.
- **Phase 2 (Destructive Ops):** All git2 APIs verified, established `*_inner()` pattern, existing `ask()` dialog pattern.
- **Phase 3 (Staging UX):** Clear competitor patterns (GitKraken three-way selector). Implementation path is known, just needs careful state machine design.
- **Phase 5 (Layout):** CSS-only bar merge is low risk. Right pane auto-open is a simple `$effect`.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | One new dep (`@lucide/svelte`) verified on npm. All git2 APIs verified in 0.19.0 docs. Zero Rust crate changes. |
| Features | HIGH | All table-stakes features have clear competitor patterns (GitKraken, Fork, Sublime Merge, Tower). Feature scope well-defined. |
| Architecture | HIGH | Full codebase audit performed. All integration points mapped to existing files. Every new command follows established patterns. |
| Pitfalls | HIGH | 13 pitfalls identified with concrete prevention strategies. All based on direct codebase analysis + git2 API behavior. |

**Overall confidence:** HIGH

### Gaps to Address

- **Graph overflow CSS approach:** MEDIUM confidence on exact implementation. `overflow: hidden` on graph column is the leading approach, but interaction with SVG overlay width needs testing with a high-lane-count repo (8+ parallel branches). Validate during Phase 4 planning.
- **Ref navigation for unloaded commits:** Need to design the load-until-found UX. How many batches to load before giving up? Show a spinner? This is a UX decision, not a technical blocker.
- **Three-way selector visual design:** The state machine is clear, but the exact Svelte component design (segmented control vs tabs vs dropdown) needs iteration. GitKraken uses icons, Tower uses a dropdown.
- **macOS title bar inset:** Hardcoded 70px padding for traffic lights is fragile. Investigate whether Tauri 2 exposes the actual inset value. May need `@tauri-apps/plugin-os` platform detection.

## Sources

### Primary (HIGH confidence)
- [git2 0.19.0 Repository docs](https://docs.rs/git2/0.19.0/git2/struct.Repository.html) — `tag_delete()`, `checkout_head()`, `status_file()`
- [git2 0.19.0 Branch docs](https://docs.rs/git2/0.19.0/git2/struct.Branch.html) — `Branch::delete()`, `find_branch()`, `is_head()`
- [git2 0.19.0 CheckoutBuilder docs](https://docs.rs/git2/0.19.0/git2/build/struct.CheckoutBuilder.html) — `force()`, `path()`, `remove_untracked()`
- [Lucide Svelte docs](https://lucide.dev/guide/packages/lucide-svelte) — `@lucide/svelte` for Svelte 5
- [npm: @lucide/svelte@0.577.0](https://www.npmjs.com/package/@lucide/svelte) — confirmed on npm, 308K weekly downloads
- Trunk v0.5 codebase — full audit of all components, commands, types, and tests

### Secondary (MEDIUM confidence)
- GitKraken Desktop documentation — staging, stashing, tags, branching UX patterns
- Fork, Sublime Merge, Tower — feature patterns from domain knowledge (no docs fetched)
- CSS spec — `position: sticky` + `transform` interaction (containing block creation)

---
*Research completed: 2026-03-15*
*Ready for roadmap: yes*
