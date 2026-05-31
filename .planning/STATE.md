---
gsd_state_version: 1.0
milestone: v0.14
milestone_name: milestone
status: Awaiting next milestone
stopped_at: Completed 76-03-PLAN.md (frontend merge/revert wiring)
last_updated: "2026-05-31T01:17:29.489Z"
last_activity: 2026-05-31 — Quick task 260531-l02: hunk Comment + review fixes (lag/button/GC) + staged commenting via index snapshot (code complete, in-app UAT pending)
progress:
  total_phases: 2
  completed_phases: 2
  total_plans: 6
  completed_plans: 6
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-05-28 after v0.14 defined)

**Core value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits -- all without touching the terminal.
**Current focus:** v0.14 Commit Message UX — both phases (75, 76) complete & verified; ready to complete-milestone

## Current Position

Phase: Milestone v0.14 complete
Plan: —
Status: Awaiting next milestone
Last activity: 2026-05-29 — Milestone v0.14 completed and archived

## Performance Metrics

| Metric | v0.1 | v0.2 | v0.3 | v0.4 | v0.5 | v0.6 | v0.7 | v0.8 | v0.9 | v0.10 | v0.11 |
|--------|------|------|------|------|------|------|------|------|------|-------|-------|
| Phases | 6 | 4 | 4 | 3 | 7 | 5 | 5 | 7 | 6 | 3 | 6 |
| Plans | 27 | 9 | 14 | 5 | 12 | 16 | 8 | 19 | 13 | 4 | 16 |
| Days | 7 | 2 | 3 | 1 | 2 | 2 | 2 | 4 | 3 | 2 | 2 |
| Phase 59 P01 | 5min | 3 tasks | 7 files |
| Phase 59 P02 | 5min | 3 tasks | 5 files |
| Phase 60 P01 | 5min | 1 tasks | 7 files |
| Phase 60 P02 | 3min | 2 tasks | 3 files |
| Phase 61 P01 | 17min | 2 tasks | 8 files |
| Phase 61 P02 | 5min | 2 tasks | 4 files |
| Phase 62 P01 | 5min | 2 tasks | 8 files |
| Phase 62 P02 | 4min | 2 tasks | 1 files |
| Phase 63 P01 | 7min | 2 tasks | 8 files |
| Phase 63 P02 | 10min | 2 tasks | 5 files |
| Phase 63 P03 | 11min | 2 tasks | 4 files |
| Phase 64 P01 | 7min | 2 tasks | 9 files |
| Phase 64 P02 | 10min | 2 tasks | 3 files |
| Phase 65 P01 | 15min | 2 tasks | 3 files |
| Phase 65 P02 | 5min | 3 tasks | 4 files |
| Phase 65 P03 | ~12min | 3 tasks | 6 files |
| Phase 65 P04 | ~10min | 2 tasks | 3 files |
| Phase 67 P04 | 25min | 2 tasks | 7 files |
| Phase 68 P01 | 6min | 2 tasks | 2 files |
| Phase 68 P02 | 6min | 2 tasks | 5 files |
| Phase 69 P01 | 18min | 2 tasks | 7 files |
| Phase 69 P02 | 12min | 2 tasks | 2 files |
| Phase 69 P04 | 6min | 2 tasks | 2 files |
| Phase 69 P03 | ~25min | 2 tasks | 2 files |
| Phase 73 P01 | 7min | 3 tasks | 2 files |
| Phase 73 P02 | 6min | 2 tasks | 2 files |
| Phase 73 P03 | 6min | 3 tasks | 2 files |
| Phase 76 P01 | 30min | 2 tasks + 1 ownership | 5 files |
| Phase 76 P02 | 18min | 2 tasks | 4 files |
| Phase 76 P03 | 38min | 2 tasks | 7 files |

## Accumulated Context

### Roadmap Evolution

- Phase 74 added: Address v0.13 tech debt: 66/WR-01..04, 70/CR-01, 70/WR-01, biome warnings

### Decisions

- syntect (Rust) chosen over Shiki (JS) for syntax highlighting -- CSS custom properties compliance, no inline styles
- similar crate chosen for word-level diff -- runs on Rust thread pool, purpose-built iter_inline_changes() API
- Whitespace-ignored diffs disable staging (GitHub Desktop pattern) -- never attempt hunk index remapping
- [Phase 59]: Byte offset ranges (u32 start/end) for WordSpan/SyntaxToken enrichment fields -- compact IPC, frontend slices content
- [Phase 59]: 100,000 context_lines cap for show_full_file instead of u32::MAX -- avoids IPC payload issues
- [Phase 59]: DiffLine enrichment fields use snake_case matching Rust Serialize default; DiffRequestOptions uses camelCase matching serde rename_all
- [Phase 59]: Default diff preferences: contextLines=3, ignoreWhitespace=false, showFullFile=false matching Rust defaults
- [Phase 60]: Used TextDiff::from_lines with iter_inline_changes for two-level word diff (line then word)
- [Phase 60]: Newline normalization before from_lines to handle content with/without trailing newlines
- [Phase 60]: Alpha 0.35 for word-diff highlights provides visible contrast atop line-level alpha 0.1 backgrounds
- [Phase 60]: Origin symbol rendered as separate span element outside word-span loop to keep symbol distinct from content slicing
- [Phase 61]: Used base16-ocean.dark bundled theme with 7 discovered RGB color mappings (keyword, string, comment, number, function, type, variable)
- [Phase 61]: DiffLine.spans: Vec<MergedSpan> replaces separate word_spans and syntax_tokens -- single unified field for frontend
- [Phase 61]: Sweep-line boundary merge algorithm: collect all boundary points, sort+dedup, iterate pairs for zero-gap coverage
- [Phase 61]: 15 CSS custom properties for syntax colors (--color-syn-*) even though backend produces 7 colors -- future-proofs for richer themes
- [Phase 61]: Opacity 0.7 via CSS [class*=syn-] on diff-line-add/delete containers for syntax desaturation on diff backgrounds
- [Phase 62]: Graceful .catch() on getDiffViewMode $effect for test environment compatibility
- [Phase 62]: diff-line-content wrapper span to maintain getByText test compatibility with gutter columns
- [Phase 62]: hunkElements as $state<Record> for cross-boundary reactivity between DiffPanel and HunkView
- [Phase 62]: Stateful store mock: getDiffViewMode/setDiffViewMode share mutable state to match real store behavior in tests
- [Phase 62]: tick() before fireEvent.click to let initial $effect settle before testing mode changes
- [Phase 63]: LazyStore-first-then-callback pattern: DiffPanel persists new value before calling ondiffoptionschange so RepoView buildDiffOptions reads updated values
- [Phase 63]: FullFileView prop pass deferred to Plan 02 since it is still a stub component
- [Phase 63]: Stateful store mock for test isolation: getDiffIgnoreWhitespace/getDiffWordWrap share mutable state to match real store behavior
- [Phase 63]: DISP-02 tested via toggle click + store call verification instead of inline style assertion (Svelte 5 dynamic styles invisible to jsdom)
- [Phase 63]: prefsLoaded gate: defer DiffPanel content rendering until async LazyStore preferences resolve to eliminate toggle flicker
- [Phase 63]: flushPrefs test helper: setTimeout(0) + tick() to properly handle async  initialization in DiffPanel tests
- [Phase 64]: ContentMode + LayoutMode replace ViewMode as independent type unions for 2D mode dispatch
- [Phase 64]: Legacy diff_view_mode store key migration: 'full' -> contentMode='full', 'split' -> layoutMode='split'
- [Phase 64]: Two independent scroll panels with syncScroll for vertical sync, independent horizontal scroll per panel
- [Phase 64]: Hunk headers in split view: left panel shows header text, right panel shows staging buttons
- [Phase 64]: Line selection only on right panel Add lines using original lineIdx from PairedRow for correct staging indices
- [Phase 65]: Review-session DTOs derive Deserialize and serialize PascalCase enums (Source/Side) with no rename_all; struct fields snake_case. Frozen keystone schema for phases 66/67/68/70.
- [Phase 65]: review_store persists per-repo sessions via atomic tmp+sync_all+rename (D-10), FNV-1a filename hash as path-traversal mitigation (D-11), and a load state machine that quarantines corrupt files (D-15) and refuses newer schema_version untouched (D-16).
- [Phase 65]: Canonical-path keying lives only in the session layer (D-11); RepoState/CommitCache keep raw-String keys
- [Phase 65]: Three-state status merge happens in the thin command, never in _inner (_inner is disk-only and can never report Active)
- [Phase 65]: start_review_session rejects with session_exists when a file already exists; client must Resume or End first (no silent overwrite)
- [Phase 65]: ReviewPanel is a D-12 throwaway 3-state lifecycle stub (Start/Resume/Discard/End), replaced by the real panel in Phase 69.
- [Phase 65]: Derived state var renamed to sessionState — naming it 'state' shadows the Svelte $state rune and breaks svelte-check.
- [Phase 67]: Auto-start a review session at the comment chokepoint (DiffPanel.ensureActiveSession) when none is active; add_comment/save_draft_comment stay dumb writers (L-08), the Comment affordance stays enabled, only merge commits disable it (D-04)
- [Phase 68]: buildFullFileAnchor is a sibling pure adapter (src/lib/full-file-anchor.ts), not an extension of diff-anchor — once D-02/D-04 diverge they share no logic (no side resolution, no diff prefixing); side=New/source=FullFile constants, new-side coords only, plain-content excerpt, gap marker N=next-prev-1
- [Phase 68]: 68-02: FullFileView owns selection state (anchorIndex/focusIndex) and exports clearSelection(); host receives flat indices on the affordance click — reconciles the plan's ownership ambiguity
- [Phase 68]: 68-02: CommentComposer reused via an optional injected captured result (lower-coupling seam over a source mode); diff-path buildDiffAnchor fallback intact
- [Phase 68]: 68-02: merge commits keep the full-file Comment affordance ENABLED (L-05) — HunkView isMerge disable not copied
- [Phase 69]: Review schema v2 — Comment gains stable id: String (#[serde(default)] empty-string sentinel) + commit_oid: Option<String>; CURRENT_SCHEMA_VERSION=2 (D-04, one bump for both)
- [Phase 69]: v1->v2 lazy load-path migration backfills uuid ids and re-saves via the atomic writer; version-gate stays BEFORE from_value/migration so D-16 (refuse newer untouched) and D-15 (corrupt quarantine) both hold
- [Phase 69]: line-anchored comments mint a real uuid id at WRITE time (add_comment_inner), not empty, so edit/delete-by-id never misses before a reload
- [Phase ?]: 69-02: edit_comment surfaces not_found via a found-flag captured inside the infallible mutate_session_rmw closure (no helper signature change, no TOCTOU)
- [Phase ?]: 69-02: comment edit/delete target by stable uuid id (D-03), not list position — multi-tab-safe; missing id -> not_found (edit) / no-op (delete)
- [Phase ?]: 69-04: TS Comment DTO mirrors Rust v2 wire shape (id, commit_oid?); OrphanReason/CommentResolution added ahead of their 69-03 backend
- [Phase ?]: 69-04: review-session.svelte.ts rune owns center-pane Review state (rightPaneMode panel|diff, reviewActive) + a RepoView-decoupled jumpTo(comment, deps) via injected JumpDeps callbacks (D-07/D-08)
- [Phase ?]: 69-03: resolve_all reads the anchor's own commit_oid for line-anchored comments (top-level commit_oid only for commit-level); a (None,None) no-target arm is CommitGone, never a panic
- [Phase ?]: 69-03: 1-based inclusive line bounds (last line resolvable) per Phase 67/68 capture; resolvability runs git2 off-lock in spawn_blocking, read-only (no emit)
- [Phase 73]: 73-01: cold-boot resume gated on sessionState === 'resume-available'; recursion-safe because post-resume status is 'active' and the listener-triggered second reload skips the resume branch (RESEARCH Pitfall 1)
- [Phase 73]: 73-01: errorMessage() returns extracted .message only; "Failed to resume review: " prefix added via template literal at the call site (matches RESEARCH §Pattern 2 canonical shape)
- [Phase 73]: 73-01: installReads default status = 'active' so 26 pre-existing ReviewPanel tests stay on the warm path without modification; statusAfterResume dispatcher flip models the backend "resume-available" -> "active" transition for recursion-safety assertions
- [Phase 73]: 73-02: button order shipped as [End review] [Copy] — End BEFORE Copy in the header flex row; UI-SPEC doesn't pin the order; destructive reads naturally to the left of the affirmative action
- [Phase 73]: 73-02: endConfirming stays TRUE during the IPC await (label frozen at "Click again to confirm" per UI-SPEC § In-flight); success collapses via session-changed listener round-trip → sessionState === 'none' → {#if} gate hides the button; failure explicitly reverts + showToast
- [Phase 73]: 73-02: toast prefix added via template literal (`Failed to end review: ${errorMessage(e, "unknown error")}`) — same shape as Plan 01's resume-fail toast; errorMessage extracts only .message, the prefix is added at the call site
- [Phase 73]: 73-03: three-way empty-state branching specificity-first (cold → warm-no-commits → warm-with-commits-zero-comments) consumes Plan 01 sessionState rune + groups + hasAnyComment; no new state shapes; warm-no-commits copy preserved verbatim
- [Phase 73]: 73-03: session-summary caption '{N} comments · {M} commits' (U+00B7) above empty-state block, gated on sessionState !== 'none' — single span with var(--color-text-muted)/font-size:11px, no new IPC, no PII (T-73-11 accepted)
- [Phase 73]: 73-03: multi-tab coordination emergent — Plan 01 sessionState rune + untouched session-changed listener (D-09 byte-for-byte across all three plans) + Plan 03 empty-state branching compose into tab-A End → tab-B cold render. Test 1 uses per-test closure dispatcher; Test 2 asserts safeInvoke call-count delta to prove canonicalPath filter held without coupling to reload() internals
- [Phase 76]: 76-01: MergeBeginResult is a serde internally-tagged enum (tag="kind", snake_case: fast_forwarded/conflicts/ready) — first such enum in the codebase, frozen as the Plan 03 frontend discriminant; tests assert the serialized `kind` string, not just the Rust variant
- [Phase 76]: 76-01: merge_branch_begin wrapper emits repo-changed UNCONDITIONALLY after an all-arm match that extracts .graph from every variant (ff/conflicts/ready) — a cancelled begin must still surface the in-progress UI (RESEARCH finding 7)
- [Phase 76]: 76-01: git merge writes the CONFLICT notice to STDOUT not stderr (git 2.54.0) — conflict detection must scan both streams; the old merge_branch stderr-only check would misclassify conflicts as Err
- [Phase 76]: 76-01: merge_continue commit uses --cleanup=strip (drops # Conflicts: block); GIT_EDITOR=true else-branch removed — None message now returns Err (contract violation), not a silent bypass
- [Phase 76]: 76-01: lib.rs dropped merge_branch registration while frontend still invokes it (CommitGraph:592, BranchSidebar:397) — expected wave seam, repointed in Plan 03
- [Phase 76]: 76-01: renaming merge_branch_inner broke the pre-existing src-tauri/tests integration suite (driver + 4 call sites); migrated to merge_branch_begin two-step semantics and INVERTED merge_branch_with_conflict_returns_error (which asserted the now-fixed stdout-conflict bug) — gate is `just check`, not filtered cargo test
- [Phase 76]: 76-02: revert-side backend — revert_commit_begin/continue/abort (RevertBeginResult struct, --cleanup=strip, git revert --abort recovery for MSG-06)
- [Phase 76]: 76-03: single RepoView-hosted MessageEditor with a reactive $state title flipped per-operation before await ref.open() (D-03/D-04); onopenmessageeditor threaded to CommitGraph/BranchSidebar/StagingPanel mirroring onopenrebaseeditor
- [Phase 76]: 76-03: merge/revert trigger sites route begin -> editor -> continue with a single null guard (cancel/empty leaves the recoverable in-progress state, no commit — D-02/MSG-06); merge ff/conflicts open no editor (MSG-02 ff skip)
- [Phase 76]: 76-03: context-menu-wired handlers tested via menu-action capture — the @tauri-apps/api/menu MenuItem.new mock records {text -> action} so firing the captured callback exercises the real user gesture in jsdom; BranchSidebar.test.ts must declare ALL Tauri mocks locally (a shared named-import helper reorders vi.mock hoisting and detaches the invoke mock the component sees)

### Pending Todos

1 — `.planning/todos/pending/2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` (real future work, not v0.13-blocking).

### Known Limitations

- SSH_AUTH_SOCK absent when app launched from Finder (not `cargo tauri dev`). Documented as known limitation.

### Blockers/Concerns

(None)

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260320-wz2 | Simplify splash screen project cards to single-line items, save 10 projects instead of 5 | 2026-03-21 | ca345bf | [260320-wz2-simplify-splash-screen-project-cards-to-](./quick/260320-wz2-simplify-splash-screen-project-cards-to-/) |
| 260325-j3y | Remove left tab offset when window is maximized/fullscreen | 2026-03-25 | 13597f5 | [260325-j3y-remove-left-tab-offset-when-window-is-ma](./quick/260325-j3y-remove-left-tab-offset-when-window-is-ma/) |
| 260325-jb3 | Prevent Ctrl+Cmd+F from being captured by Cmd+F search handler | 2026-03-25 | 8b3dcd9 | [260325-jb3-prevent-ctrl-cmd-f-from-being-captured-b](./quick/260325-jb3-prevent-ctrl-cmd-f-from-being-captured-b/) |
| 260325-lkj | Fix graph column width too wide for linear repos (auto-fit to content) | 2026-03-25 | 51add5e | [260325-lkj-fix-graph-column-width-too-wide-for-line](./quick/260325-lkj-fix-graph-column-width-too-wide-for-line/) |
| 260325-up9 | Add keyboard arrow navigation to commit graph pane | 2026-03-26 | a048c3f | [260325-up9-when-the-commit-graph-pane-has-focus-the](./quick/260325-up9-when-the-commit-graph-pane-has-focus-the/) |
| 260402-wea | Show full repo path tooltip on tab hover | 2026-04-03 | 12fdf14 | [260402-wea-show-full-repo-path-tooltip-on-tab-hover](./quick/260402-wea-show-full-repo-path-tooltip-on-tab-hover/) |
| 260402-x1v | Double-click remote branch to create+checkout local tracking branch | 2026-04-03 | b31a968 | [260402-x1v-double-click-remote-branch-to-checkout-l](./quick/260402-x1v-double-click-remote-branch-to-checkout-l/) |
| 260403-1yi | Delete remote branches from sidebar and graph context menus | 2026-04-03 | 5c48d8d | [260403-1yi-delete-remote-branches-from-sidebar-and-](./quick/260403-1yi-delete-remote-branches-from-sidebar-and-/) |
| 260403-uy4 | Auto-advance focus to next file after staging/unstaging/discarding | 2026-04-04 | 753358e | [260403-uy4-auto-advance-focus-to-next-file-after-st](./quick/260403-uy4-auto-advance-focus-to-next-file-after-st/) |
| 260405-ik1 | Fix tree view folder staging to stage all files (bulk stage_files/unstage_files) | 2026-04-05 | 96013e7 | [260405-ik1-fix-tree-view-folder-staging-to-stage-al](./quick/260405-ik1-fix-tree-view-folder-staging-to-stage-al/) |
| 260405-j41 | Fix diff view horizontal scroll to scroll entire view together | 2026-04-05 | c458fd3 | [260405-j41-fix-diff-view-horizontal-scroll-scroll-e](./quick/260405-j41-fix-diff-view-horizontal-scroll-scroll-e/) |
| 260514-356 | VSCode-style recent projects picker (Cmd/Ctrl+R) | 2026-05-14 | 045de1f | [260514-356-build-a-recent-projects-picker-vscode-ct](./quick/260514-356-build-a-recent-projects-picker-vscode-ct/) |
| 260531-4kk | Review uncommitted changes — working tree as a review target (dangling-commit snapshot) | 2026-05-31 | fe5b679 | [260531-4kk-add-review-uncommitted-changes-to-the-co](./quick/260531-4kk-add-review-uncommitted-changes-to-the-co/) |
| 260531-k4j | Move uncommitted-changes Comment into the working-tree diff toolbar (get-or-create snapshot); remove v1 panel button — code complete, in-app UAT pending | 2026-05-31 | 6349d2d | [260531-k4j-move-uncommitted-changes-comment-afforda](./quick/260531-k4j-move-uncommitted-changes-comment-afforda/) |
| 260531-l02 | Hunk-level Comment button + adversarial-review fixes: Infinity-anchor (capture-at-open), click-lag (defer snapshot+session to submit, open synchronously), Comment button moved left of staging, GC-orphan fix (snapshot keepalive refs under refs/trunk/review-snapshots). See ADVERSARIAL-REVIEW.md. Code complete, in-app UAT pending | 2026-05-31 | dde9107 | [260531-l02-hunk-level-comment-button-in-the-no-sele](./quick/260531-l02-hunk-level-comment-button-in-the-no-sele/) |

## Deferred Items

Items acknowledged and deferred at milestone close on 2026-05-28. Audited again on 2026-05-28 — 7 entries (5 debug + 2 todos) were already fixed in code; status files moved to `.planning/debug/resolved/` and `.planning/todos/done/` with verification evidence.

| Category | Item | Status |
|----------|------|--------|
| debug_sessions | graph-dot-zoom-drift | awaiting_human_verify (fix landed; needs in-app smoke at non-100% zoom) |
| quick_tasks | 21 entries (most already shipped, slug-index marker stale) | missing |
| todos | 2026-04-14-collect-commit-messages-for-merge-revert | open (real future work; needs commit-message modal + temp-editor-script plumbing) |
| uat_gaps | Phase 65 (1 open scenario) | partial |
| uat_gaps | Phase 66 (10 open scenarios) | partial |
| uat_gaps | Phase 70 (3 open scenarios) | partial |
| verification_gaps | Phases 65, 66, 70, 71, 72, 73, 74 | human_needed |

Closed on 2026-05-28 cleanup pass:

- debug/merge-editor-manual-edit-overridden — fixed in Phase 38-07 (commit 6c76bbf)
- debug/context-menu-take-all-no-resolve — fixed by StagingPanel `onfileresolved`/`onfileadvance` callbacks wired in RepoView
- debug/shift-click-text-selection — fixed via `onmousedown` shift+preventDefault in HunkView/SplitView/FullFileView
- debug/overflow-pill-context-menu — fixed by per-ref oncontextmenu wiring in CommitGraph overflow expansion
- debug/stash-dashed-branch-lines — fixed structurally by graph algorithm rewrite (stash_lanes HashSet removed)
- todos/relocate-copy-action-off-preview-pane — closed by Phase 72 (preview pane deleted, Copy on ReviewPanel)
- todos/review-pane-navigation-and-dead-review-button — closed by Phases 72 + 73 (toolbar/menu entry, in-pane End-review button)

v0.13-specific outstanding work:

- 1 human UAT — cross-repo session-changed isolation across two Tauri windows (Phase 73)
- INT-W1 (stylistic, seed_review_range canonical path resolution location)
- INT-W2 (edge case, save_draft_comment under cross-tab End-review)

## Session Continuity

Last activity: 2026-05-29
Last session: 2026-05-29T00:03:37.511Z
Stopped at: Completed 76-03-PLAN.md (frontend merge/revert wiring)
Resume file: None
Next action: execute 76-04 (StagingPanel merge-continue + OperationBanner revert continue/abort + UAT)

## Operator Next Steps

- Start the next milestone with /gsd:new-milestone
