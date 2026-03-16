# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v0.1 — MVP

**Shipped:** 2026-03-09
**Phases:** 6 | **Plans:** 27 (26 complete) | **Commits:** 155 | **Timeline:** 7 days

### What Was Built
- Vite+Svelte SPA with Tailwind v4 dark theme and shared Rust/TypeScript primitives
- Visual commit graph with Rust lane algorithm, inline SVG, and virtual scrolling
- Branch sidebar with checkout, search, dirty-workdir error handling, and branch creation
- Working tree staging panel with filesystem watcher auto-refresh
- Commit creation/amendment with validation and immediate graph refresh
- Unified diff display for workdir/staged/commit diffs with commit metadata

### What Worked
- **TDD inner-fn pattern**: Separating Tauri state from pure git2 logic made all commands directly testable without the Tauri runtime
- **Phase execution speed**: 6 phases in 7 days — tight dependency chain kept each phase focused
- **safeInvoke wrapper**: Single IPC abstraction eliminated error handling boilerplate across all frontend components
- **Cache repopulate-before-emit**: Solved race conditions between mutation commands and graph remount elegantly
- **Gap closure plans**: When UAT revealed issues, adding targeted plans (02-07, 02-08, 02-09) was clean and traceable

### What Was Inefficient
- **Graph algorithm required 3 iterations**: Plans 02-07, 02-08, 02-09 were all gap closures for the lane algorithm — initial algorithm design underestimated edge cases (first-parent edges, column priority, pass-through edges)
- **GRAPH-04 never implemented**: Merge commit visual distinction was in the requirement and the Rust DTO but never wired in the Svelte template — slipped through because verification focused on algorithm correctness, not rendering completeness
- **Phase 06 VERIFICATION.md never created**: The diff feature was code-complete but formal verification was skipped
- **ROADMAP progress table got stale**: Phase 2 and Phase 4 showed incorrect completion counts in the progress table

### Patterns Established
- **inner-fn pattern**: `*_inner()` functions for pure git2 logic; thin Tauri wrappers handle state extraction and spawn_blocking
- **safeInvoke<T>**: All Tauri IPC goes through this wrapper; never raw `invoke()`
- **Sequence counter for async guards**: `loadSeq` pattern discards stale async responses
- **{#key graphKey} remount**: Forces full CommitGraph re-render after mutations; key resets on repo close
- **Cache repopulate before emit**: Any command that invalidates CommitCache must repopulate before emitting `repo-changed`
- **CSS flex truncation**: overflow:hidden on container + min-width:0 + flex:1 on text span

### Key Lessons
1. **Graph algorithms need more upfront design**: The lane algorithm went through 3 gap-closure iterations — investing more time in edge case analysis before implementation would have saved effort
2. **Verification must cover rendering, not just logic**: GRAPH-04 passed algorithm verification but the frontend rendering was never checked
3. **Keep progress tables in sync**: Stale ROADMAP progress counts cause confusion during milestone completion
4. **Svelte 5 runes require assignment for reactivity**: Mutating collections in place doesn't trigger updates — must use immutable patterns (new Set, spread)
5. **git2 API quirks need discovery**: `is_head_unborn()` missing in 0.19, `stash_foreach` requires `&mut Repository`, `peel_to_commit()` has lifetime conflicts — each required workaround discovery

---

## Milestone: v0.2 — Commit Graph

**Shipped:** 2026-03-10
**Phases:** 4 | **Plans:** 9 | **Commits:** 76 | **Timeline:** 2 days

### What Was Built
- Hardened Rust lane algorithm with ghost lane fix, octopus protection, max_columns, and branch color counter
- Three-layer SVG lane rendering (rails -> edges -> dots) with vivid 8-color palette
- Manhattan-routed merge/fork edges with 6px rounded corners
- Merge commits as hollow circles, WIP row in virtual list with dashed connector
- Lane-colored ref pills with remote dimming and connector lines
- 6-column resizable layout with LazyStore-persisted widths and column visibility toggles

### What Worked
- **Dedicated milestone for graph rendering**: Deferring lanes from v0.1 was the right call — focused effort produced a clean result in 2 days
- **GraphResult wrapper type**: Returning max_columns alongside commits eliminated SVG width inconsistency at the root
- **Three-layer SVG architecture**: Separating rails/edges/dots made it easy to add conditional rendering (hollow merge dots, dashed WIP) without conflicts
- **Gap closure as numbered plans**: Plans 10-03, 10-04, 10-05 cleanly addressed UAT findings without disrupting the main plan sequence
- **LazyStore pattern reuse**: ColumnWidths and ColumnVisibility both followed the same getter/setter/persistence pattern — second use was trivial

### What Was Inefficient
- **Phase 10 needed 5 plans for 2 requirements**: DIFF-01 and DIFF-02 spawned 3 gap-closure plans (10-03, 10-04, 10-05) — initial plans underestimated the visual integration complexity of connector lines spanning multiple columns
- **ROADMAP plan checkboxes got stale**: Plans 10-04 and 10-05 were marked `[ ]` even though SUMMARY files existed — same issue as v0.1
- **VIS-03 reinterpreted silently**: "Reduced opacity" became "hollow dot only" — the requirement text should have been updated to match the implementation decision

### Patterns Established
- **GraphResult wrapper**: walk_commits returns struct with commits + max_columns, not bare Vec
- **Branch color counter**: Monotonic counter, HEAD=0, freed columns remove entries
- **Sentinel oid pattern**: `'__wip__'` for synthetic items in GraphCommit array
- **displayItems derived**: Wrapping backend data with frontend-only synthetic items
- **LazyStore for UI state**: Column widths and visibility both use this pattern
- **Cross-column visual elements**: Absolute-positioned divs at row level, not within column SVGs
- **Native Tauri Menu API**: Preferred over custom Svelte context menus for native UX

### Key Lessons
1. **Visual integration is harder than rendering**: Drawing individual elements is straightforward; making them work together across column boundaries required 3 additional plans
2. **Keep ROADMAP checkboxes in sync**: This is the second milestone where plan checkboxes got stale — consider automating or removing them
3. **Update requirement text when reinterpreting**: When a design decision changes the meaning of a requirement, update REQUIREMENTS.md immediately
4. **52 tests, zero regressions**: The test suite from v0.1 protected against algorithm regressions effectively through all 4 phases

---

## Milestone: v0.3 — Actions

**Shipped:** 2026-03-12
**Phases:** 4 | **Plans:** 14 | **Commits:** 88 | **Timeline:** 3 days

### What Was Built
- Full stash management (create/pop/apply/drop) with graph-integrated stash rows and right-click context menu
- Commit context menu with copy, checkout, branch, tag, cherry-pick, revert (merge commits disabled)
- Remote fetch/pull/push via git CLI subprocess with per-line progress streaming
- Quick actions toolbar merged with tab bar into single top row
- Branch ahead/behind tracking computed inside list_refs
- Undo/redo last commit with race-condition guards

### What Worked
- **git CLI for remote ops**: Shelling out to git avoided libgit2 SSH/HTTPS auth issues entirely — clean subprocess pattern reused for cherry-pick/revert
- **Sentinel OID pattern extension**: `__stash_N__` reused the `__wip__` pattern for stash graph rows — zero new abstraction needed
- **$derived.by() for complex reactivity**: Cleaner than IIFE pattern for imperative splice logic in displayItems
- **Shared $state rune modules**: remote-state.svelte.ts provided clean cross-component communication without prop drilling
- **Ahead/behind in list_refs**: Computing inside existing map closure avoided extra IPC round-trip

### What Was Inefficient
- **Stash graph rendering failed and was redone**: Plan 11-02 was entirely removed during UAT and reimplemented as 11-05 — the initial approach had too many rendering bugs
- **ROADMAP plan checkboxes still getting stale**: Third milestone with this issue — plans 11-05, 11-06, 12-01, 12-02, 13-03, 14-03 all show `[ ]` despite being complete
- **Redo race condition**: clearRedoStack in repo-changed listener fired during undo/redo operations — required targeted gap closure plan (14-03)

### Patterns Established
- **git CLI subprocess pattern**: GIT_TERMINAL_PROMPT=0 + GIT_SSH_COMMAND for batch mode; PID stored in RunningOp for cancellation
- **$derived.by()**: Preferred over IIFE $derived for imperative reactive computations
- **Shared $state rune modules**: For cross-component state (remote-state.svelte.ts)
- **InputDialog $state dialogConfig**: Set config to show dialog, null to hide
- **Two-pass borrow pattern**: Collect into Vec first, then process — for git2 mutable borrow conflicts

### Key Lessons
1. **Plan for graph rendering failures**: Visual features that integrate with the virtual list need careful testing before merging — 11-02 was a complete redo
2. **Race conditions in event-driven architectures**: repo-changed events fire for both user actions and programmatic mutations — guards (isUndoing/isRedoing) are necessary
3. **Duplicating small helpers is OK**: open_repo/is_dirty duplicated in commit_actions.rs to avoid cross-module dependencies — pragmatic over DRY
4. **14 plans in 3 days**: Velocity increasing with established patterns — each plan averaged ~30 min

---

## Milestone: v0.4 — Graph Rework

**Shipped:** 2026-03-13
**Phases:** 3 | **Plans:** 5 | **Commits:** ~30 | **Timeline:** 1 day

### What Was Built
- GraphSvgData computing one SVG path per commit-to-commit edge with Manhattan routing
- ViewBox-clipped per-row rendering of full SVG paths (no per-row seams)
- Commit dot rendering as individual SVG elements (filled for regular, hollow for merges)
- WIP row with dashed connector and stash rows with square dots in new SVG model

### What Worked
- **Per-row viewBox approach**: Quick to implement, validated that continuous paths look better than per-row fragments
- **Clean phase scoping**: 3 focused phases (data engine, rendering, synthetic rows) with clear boundaries
- **Decision to carry phases forward**: Phases 18-19 (ref pills, interactions) correctly deferred to v0.5 rather than forcing into v0.4

### What Was Inefficient
- **Architecture replaced in next milestone**: Per-row viewBox clipping worked but was immediately superseded by the single SVG overlay in v0.5 — the intermediate step added code that was deleted 1 day later
- **Short-lived milestone**: v0.4 lasted 1 day before v0.5 planning began — could have been folded into v0.5

### Key Lessons
1. **Intermediate architectural steps can be wasteful**: v0.4's per-row approach was functional but immediately replaced — sometimes it's better to invest in the final architecture directly
2. **Decision gates are valuable**: Phase 20's decision gate in v0.5 validated the overlay approach before full investment — this should be standard for architectural changes

---

## Milestone: v0.5 — Graph Overlay

**Shipped:** 2026-03-15
**Phases:** 7 | **Plans:** 12 | **Commits:** 111 | **Timeline:** 2 days

### What Was Built
- Single SVG overlay spanning full graph height with native scroll sync (zero JS)
- TypeScript Active Lanes transformation with edge coalescing (O(commits x lanes) → O(lanes + merge_edges))
- Cubic bezier curve rendering with adaptive corner radius replacing Manhattan routing
- Three-layer z-ordered SVG (rails → edges → dots) with virtualized element filtering
- SVG ref pills with Canvas-based text measurement, connector lines, remote dimming, overflow +N hover expansion
- Clean integration: unified constants, ~1,000 lines dead code removed, all interactions preserved

### What Worked
- **Decision gate (Phase 20)**: Validating scroll sync and pointer-events passthrough before investing 5 more phases avoided a potential full rework
- **TDD throughout**: buildGraphData, buildOverlayPaths, getVisibleOverlayElements, buildRefPillData, measureTextWidth all built test-first — tests caught regressions during integration
- **Parallel phase design**: Phases 21+22 could execute in parallel (both depend only on Phase 20 types), and Phases 25+26 both depend on Phase 24 — well-structured dependency graph
- **Injectable measureFn**: OffscreenCanvas for production, mock function for tests — deterministic text measurement tests
- **Edge coalescing**: Reducing edge count made the overlay performant on large repos without explicit optimization
- **Gap closure plans scoped tightly**: 23-03 and 23-04 were small, targeted fixes (positioning, test constants) — resolved quickly

### What Was Inefficient
- **OVERLAY_ROW_HEIGHT confusion**: Initially set to 36px, then corrected to 26px (Phase 23-03), then 11 tests broke because they hardcoded 36 (Phase 23-04) — constants should have been settled before test-writing
- **Some ROADMAP plan checkboxes still unchecked**: Phases 22, 23, 25, 26 have `[ ]` in ROADMAP despite having SUMMARYs — persistent issue from v0.1
- **HTML hover overlay for ref pills**: SVG-only approach couldn't handle multi-ref expansion elegantly — needed a hybrid SVG+HTML solution, adding complexity

### Patterns Established
- **SVG overlay architecture**: Single SVG in scroll container with pointer-events:none, HTML handles interactions beneath
- **Pure transformation pipeline**: GraphCommit[] → buildGraphData → buildOverlayPaths → getVisibleOverlayElements → SVG rendering — each step is a pure function
- **Injectable dependencies for testing**: measureFn pattern for Canvas text measurement — can mock in tests
- **Range-intersection filtering**: minRow/maxRow metadata on paths enables O(1) visibility checks
- **translate-based multi-column SVG**: SVG spans ref+graph columns, existing graph groups offset via translate()

### Key Lessons
1. **Decision gates before architectural investment**: Phase 20's POC validation saved potentially wasted effort — make this standard for any architecture change
2. **Settle constants early**: OVERLAY_ROW_HEIGHT changing mid-milestone caused cascading test fixes — lock dimensions before writing tests
3. **Pure function pipelines are testable**: Every step in the overlay pipeline (data, paths, visibility, pills) was independently testable because they're pure functions with no side effects
4. **Edge coalescing is a performance multiplier**: Reducing O(commits x lanes) edges to O(lanes + merge_edges) made virtualization almost unnecessary — but having both is defense in depth
5. **12 plans in 2 days**: Fastest pace yet — established patterns from 4 prior milestones enabled rapid execution

---

## Milestone: v0.6 — UI Polish & Core Ops

**Shipped:** 2026-03-16
**Phases:** 6 (incl. 27.1) | **Plans:** 16 | **Commits:** ~129 | **Timeline:** 2 days

### What Was Built
- Lucide icon set replacing all Unicode symbols across Toolbar, FileRow, StagingPanel, CommitForm, BranchSidebar, TabBar, and SVG overlay pills
- Toast notification system: Svelte 5 $state store, auto-dismiss timer, fly transition, per-kind styling (error/success)
- Destructive operations: discard file/all (git2 checkout + fs::remove_file for untracked), delete branch/tag, rename branch, reset (soft/mixed/hard) — all with confirmation dialogs and toast feedback
- Three-way commit/amend/stash selector replacing amend checkbox; stash mode auto-populates subject as stash name
- Graph CSS polish: viewport padding, column overflow-hidden with fixed minimum, sidebar click scrolls to ref commit via resolve_ref backend command
- Unified title bar: decorations:false + drag region + macOS traffic light padding (78px); right pane auto-opens on commit/ref click
- Bug fixes from TODO.md: window rubber-band scroll (overflow:hidden on html/body), root commit downward branch tip (isBranchTip=true for root commits)

### What Worked
- **Established patterns enabled fast delivery**: All backend commands used the inner-fn pattern; all frontend IPC used safeInvoke — zero new architecture decisions needed
- **Wave 0 test scaffolds**: Writing failing tests before implementation (Phase 27-01) verified TDD discipline and caught the toast store import issue early
- **Phases 27-31 executed in 2 days**: Fastest multi-phase milestone yet — UI work is less risky than graph algorithm work
- **TODO.md as pre-archive checklist**: Reviewing TODO.md before closing found 2 real bugs (scroll gap, root commit tip) that were quick to fix and worth shipping in v0.6
- **Decimal phase (27.1) for urgent insertion**: Clean way to insert an urgent fix without renumbering subsequent phases

### What Was Inefficient
- **Phase 30 "Plans: TBD" in ROADMAP**: Phase 30 had `Plans: TBD` at roadmap creation time — should have been planned at the same time as other phases
- **STATE.md progress bar stale**: Progress bar in STATE.md showed `3/5 phases` even after all 5 completed — hand-maintained state drifted
- **REQUIREMENTS.md traceability stale**: GRAPH-01/02/03, LAYOUT-01/02 stayed "Pending" in the traceability table after phases 30/31 completed — not updated post-execution
- **gsd-tools couldn't find phase plans** (archived to v0.6-phases/ before CLI ran): CLI reported 0 phases/plans in MILESTONES.md — had to manually fill stats

### Patterns Established
- **`@lucide/svelte` not `lucide-svelte` for Svelte 5**: `lucide-svelte` causes SvelteComponent type errors; `@lucide/svelte` is the correct Svelte 5 package
- **`decorations: false` + drag region**: Native titlebar removal pattern; `data-tauri-drag-region` on non-interactive elements, `padding-left: 78px` for macOS traffic lights
- **`overflow: hidden` on html/body**: Required with `decorations: false` to prevent macOS rubber-band scroll from creating a window gap
- **`isBranchTip = true` for root commits**: Root commits have no parents but may have `active_lanes[col] = Some(...)` set by their child — must explicitly mark as tip
- **discard uses git2 checkout for tracked, fs::remove_file for untracked**: No git CLI subprocess needed; `discard` commands skip cache rebuild since FS watcher handles workdir detection
- **clearRedoStack guard for stash mode**: stash doesn't modify commit history so clearRedoStack must be skipped

### Key Lessons
1. **Archive phases before running milestone CLI**: gsd-tools `milestone complete` relies on finding phase files in `.planning/phases/` — archiving first causes 0-count MILESTONES.md entries
2. **TODO.md review before milestone close is high-value**: Found 2 bugs in 5 minutes that were straightforward to fix and improved the shipped quality
3. **UI polish milestones are faster than graph milestones**: 16 plans/2 days vs 12 plans/2 days for v0.5, with less uncertainty — established patterns do the heavy lifting
4. **Stale traceability tables are misleading**: Update requirement statuses immediately after phase completion, not at milestone close
5. **Root commit edge cases in graph algorithms**: The isBranchTip detection used `active_lanes[col].is_none()` but a child sets that slot before the root processes — always cross-check with `parent_count() == 0`

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Days | Phases | Plans | Key Change |
|-----------|------|--------|-------|------------|
| v0.1 | 7 | 6 | 27 | First milestone — established all patterns |
| v0.2 | 2 | 4 | 9 | Focused visual milestone — gap closure plans for UAT findings |
| v0.3 | 3 | 4 | 14 | Largest plan count — subprocess pattern for remote/cherry-pick/revert |
| v0.4 | 1 | 3 | 5 | Intermediate architecture — per-row viewBox SVG (superseded by v0.5) |
| v0.5 | 2 | 7 | 12 | Pure function pipeline — decision gate, TDD, SVG overlay architecture |
| v0.6 | 2 | 6 | 16 | UI polish milestone — established patterns enabled fastest per-plan pace yet |

### Top Lessons (Verified Across Milestones)

1. **Gap closure plans are a recurring pattern**: All 6 milestones needed additional plans for UAT findings — budget 1-2 per phase
2. **ROADMAP checkboxes get stale**: All 6 milestones had this issue — should automate or remove
3. **Test suite protects against regressions**: Tests caught zero regressions across all milestones — TDD investment pays dividends
4. **Visual rendering is the riskiest area**: v0.1 needed 3 graph iterations, v0.2 needed 3 gap closures for connectors, v0.3 had a full plan redo (11-02), v0.5 had constant confusion — visual features need more upfront design or spike plans
5. **Decision gates prevent wasted work**: v0.4's architecture was superseded in 1 day; v0.5's Phase 20 decision gate validated the approach before committing — gates should be standard for architecture changes
6. **Pure function pipelines scale**: v0.5's pipeline (data → paths → visibility → rendering) was independently testable and composable — prefer this over stateful approaches
7. **TODO.md review before milestone close is high-value**: v0.6 found 2 real bugs in pre-archive review — make this a standard step
