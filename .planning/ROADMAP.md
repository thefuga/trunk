# Roadmap: Trunk

## Milestones

- ✅ **v0.1 MVP** — Phases 1-6 (shipped 2026-03-09)
- ✅ **v0.2 Commit Graph** — Phases 7-10 (shipped 2026-03-10)
- ✅ **v0.3 Actions** — Phases 11-14 (shipped 2026-03-12)
- ✅ **v0.4 Graph Rework** — Phases 15-17 (shipped 2026-03-13)
- ✅ **v0.5 Graph Overlay** — Phases 20-26 (shipped 2026-03-15)
- 🔵 **v0.6 UI Polish & Core Ops** — Phases 27-31 (active)

## Phases

<details>
<summary>✅ v0.1 MVP (Phases 1-6) — SHIPPED 2026-03-09</summary>

- [x] Phase 1: Foundation (3/3 plans) — completed 2026-03-03
- [x] Phase 2: Repository Open + Commit Graph (8/9 plans) — completed 2026-03-09
- [x] Phase 3: Branch Sidebar + Checkout (5/5 plans) — completed 2026-03-04
- [x] Phase 4: Working Tree + Staging (4/4 plans) — completed 2026-03-07
- [x] Phase 5: Commit Creation (3/3 plans) — completed 2026-03-07
- [x] Phase 6: Diff Display (3/3 plans) — completed 2026-03-07

Full details: [milestones/v0.1-ROADMAP.md](milestones/v0.1-ROADMAP.md)

</details>

<details>
<summary>✅ v0.2 Commit Graph (Phases 7-10) — SHIPPED 2026-03-10</summary>

- [x] Phase 7: Lane Algorithm Hardening (2/2 plans) — completed 2026-03-09
- [x] Phase 8: Straight Rail Rendering (1/1 plans) — completed 2026-03-09
- [x] Phase 9: WIP Row + Visual Polish (1/1 plans) — completed 2026-03-09
- [x] Phase 10: Differentiators (5/5 plans) — completed 2026-03-10

Full details: [milestones/v0.2-ROADMAP.md](milestones/v0.2-ROADMAP.md)

</details>

<details>
<summary>✅ v0.3 Actions (Phases 11-14) — SHIPPED 2026-03-12</summary>

- [x] Phase 11: Stash Operations (6/6 plans) — completed 2026-03-12
- [x] Phase 12: Commit Context Menu (2/2 plans) — completed 2026-03-12
- [x] Phase 13: Remote Operations (3/3 plans) — completed 2026-03-12
- [x] Phase 14: Toolbar + Tracking (3/3 plans) — completed 2026-03-12

Full details: [milestones/v0.3-ROADMAP.md](milestones/v0.3-ROADMAP.md)

</details>

<details>
<summary>✅ v0.4 Graph Rework (Phases 15-17) — SHIPPED 2026-03-13</summary>

- [x] Phase 15: Graph Data Engine (2/2 plans) — completed 2026-03-12
- [x] Phase 16: Core Graph Rendering (1/1 plans) — completed 2026-03-12
- [x] Phase 17: Synthetic Row Adaptation (2/2 plans) — completed 2026-03-13

Phases 18-19 (Ref Pill Migration, Interaction Preservation) carried into v0.5 as PILL-01–04 and INTR-01–03.

Full details: [milestones/v0.4-ROADMAP.md](milestones/v0.4-ROADMAP.md)

</details>

<details>
<summary>✅ v0.5 Graph Overlay (Phases 20-26) — SHIPPED 2026-03-15</summary>

- [x] Phase 20: Foundation — Types, Constants & Overlay Container (2/2 plans) — completed 2026-03-14
- [x] Phase 21: Active Lanes Transformation (1/1 plans) — completed 2026-03-14
- [x] Phase 22: Bezier Path Builder (1/1 plans) — completed 2026-03-14
- [x] Phase 23: SVG Rendering (4/4 plans) — completed 2026-03-14
- [x] Phase 24: Integration (1/1 plans) — completed 2026-03-14
- [x] Phase 25: Interaction Preservation (1/1 plans) — completed 2026-03-14
- [x] Phase 26: SVG Ref Pills (2/2 plans) — completed 2026-03-14

Full details: [milestones/v0.5-ROADMAP.md](milestones/v0.5-ROADMAP.md)

</details>

### v0.6 UI Polish & Core Ops (Active)

- [x] **Phase 27: Foundation — Icons, Toast & Bug Fixes** — Icon system, toast notifications, and critical bug fixes that unblock all subsequent phases (5 plans) (completed 2026-03-15)
- [x] **Phase 28: Destructive Operations** — Discard changes, branch/tag delete, branch rename, and reset — all with confirmation dialogs (completed 2026-03-15)
- [ ] **Phase 29: Staging & Commit UX** — Three-way commit/amend/stash selector, colored stage/unstage buttons, equal-height file lists
- [ ] **Phase 30: Graph Polish & Navigation** — Graph padding, column overflow/shrink, sidebar ref navigation, and right pane auto-open
- [ ] **Phase 31: Layout Polish** — Merge window top bar with tab/actions bar into unified bar

## Phase Details

### Phase 27: Foundation — Icons, Toast & Bug Fixes
**Goal**: App has a consistent visual vocabulary, non-blocking operation feedback, and correct dirty-state behavior for untracked files
**Depends on**: Nothing (first v0.6 phase)
**Requirements**: ICON-01, TOAST-01, FIX-01, FIX-02
**Success Criteria** (what must be TRUE):
  1. Every toolbar button, file row, sidebar section, tab bar item, and commit form element displays an SVG icon instead of a Unicode symbol
  2. Successful operations (e.g. stash created, branch checked out) and errors show a non-blocking toast notification that auto-dismisses
  3. Creating a new untracked file in the repo causes the WIP row to appear in the commit graph and the file to appear in the unstaged list
  4. The last visible column in the commit graph header renders without a trailing resize divider on its right edge
**Plans**: 5 plans
Plans:
- [ ] 27-01-PLAN.md — Wave 0: Failing test scaffolds for toast store and dirty-counts fix
- [ ] 27-02-PLAN.md — Toast system: store, overlay component, wired into operations
- [ ] 27-03-PLAN.md — Icon system: install @lucide/svelte, replace Unicode symbols in 7 components
- [ ] 27-04-PLAN.md — Bug fixes: FIX-01 untracked files in dirty counts, FIX-02 last-column resize handle
- [ ] 27-05-PLAN.md — Gap closure: guard message column resize handle with lastVisibleColumn check

### Phase 27.1: Add icons to commit graph pills (INSERTED)

**Goal:** All four ref pill types (local branch, remote branch, tag, stash) in the commit graph SVG overlay display Lucide icons matching their sidebar/toolbar counterparts, replacing hand-drawn SVG path icons
**Requirements**: PILL-ICON-01
**Depends on:** Phase 27
**Plans:** 1/1 plans complete

Plans:
- [ ] 27.1-01-PLAN.md — Add Lucide icons to all pill types, remove hand-drawn SVG paths

### Phase 28: Destructive Operations
**Goal**: Users can perform common destructive git operations (discard, delete, rename, reset) with clear confirmation safeguards
**Depends on**: Phase 27 (icons for buttons/menus, toast for operation feedback)
**Requirements**: GITOP-01, GITOP-02, GITOP-03, GITOP-04, GITOP-05, GITOP-06
**Success Criteria** (what must be TRUE):
  1. User can right-click an unstaged file and discard its changes — a confirmation dialog appears, and after confirming, the file reverts to its last committed state (or is deleted if untracked)
  2. User can click "Discard all" to revert all unstaged changes at once, with a confirmation dialog showing the count of affected files
  3. User can right-click a local branch in the sidebar, choose Delete, confirm in a dialog, and the branch is removed (HEAD branch deletion is prevented)
  4. User can right-click a tag in the commit graph or sidebar, choose Delete, confirm, and the tag is removed
  5. User can right-click a local branch, choose Rename, enter a new name, and the branch is renamed in place
  6. User can right-click any commit, choose Reset, pick soft/mixed/hard mode, confirm, and the current branch tip moves to that commit
**Plans**: 3 plans
Plans:
- [ ] 28-01-PLAN.md — Backend commands: discard file/all, delete branch/tag, rename branch + unit tests
- [ ] 28-02-PLAN.md — Discard frontend: file context menu + Discard All button in StagingPanel
- [ ] 28-03-PLAN.md — Branch/Tag frontend: sidebar + graph pill context menus for delete/rename

### Phase 29: Staging & Commit UX
**Goal**: Users have a unified "save my work" workflow through a three-way selector and polished staging controls
**Depends on**: Phase 27 (icons for selector/buttons)
**Requirements**: STAGE-01, STAGE-02, STAGE-03, STAGE-04, STAGE-05
**Success Criteria** (what must be TRUE):
  1. Commit form displays a three-way selector (commit / amend / stash) that replaces the old amend checkbox, and switching modes updates form state (amend pre-fills last message, stash changes button label)
  2. When stash mode is selected, the subject field auto-populates with the current commit form message as the default stash name
  3. "Stage all changes" button is visually green and "Unstage all changes" button is visually red, clearly distinguishable at a glance
  4. When both unstaged and staged file lists have files, they render at equal height (50/50 split of available space)
**Plans**: TBD

### Phase 30: Graph Polish & Navigation
**Goal**: Commit graph handles dense histories gracefully and users can jump to any ref from the sidebar
**Depends on**: Phase 27 (icons for navigation affordances)
**Requirements**: GRAPH-01, GRAPH-02, GRAPH-03, LAYOUT-01
**Success Criteria** (what must be TRUE):
  1. Commit graph has visible padding above the first commit row and below the last commit row, preventing content from touching container edges
  2. Graph column can shrink narrower than its full lane content width — lanes compress rather than overflowing or causing horizontal scroll
  3. Clicking a branch or tag name in the left sidebar scrolls the commit graph to center that ref's commit row, loading additional history if needed
  4. When the right pane is collapsed and the user clicks a commit, branch, or tag that would show detail, the right pane automatically opens to display it
**Plans**: TBD

### Phase 31: Layout Polish
**Goal**: App window uses vertical space efficiently with a single unified top bar
**Depends on**: Phase 27 (icons in merged bar)
**Requirements**: LAYOUT-02
**Success Criteria** (what must be TRUE):
  1. The window's title bar area and the tab/actions toolbar are visually merged into one continuous bar — no visible boundary or wasted vertical space between them
  2. All existing toolbar actions (pull, push, branch, stash, pop, undo, redo) and tab controls remain fully functional in the merged bar
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v0.1 | 3/3 | Complete | 2026-03-03 |
| 2. Repository Open + Commit Graph | v0.1 | 8/9 | Complete | 2026-03-09 |
| 3. Branch Sidebar + Checkout | v0.1 | 5/5 | Complete | 2026-03-04 |
| 4. Working Tree + Staging | v0.1 | 4/4 | Complete | 2026-03-07 |
| 5. Commit Creation | v0.1 | 3/3 | Complete | 2026-03-07 |
| 6. Diff Display | v0.1 | 3/3 | Complete | 2026-03-07 |
| 7. Lane Algorithm Hardening | v0.2 | 2/2 | Complete | 2026-03-09 |
| 8. Straight Rail Rendering | v0.2 | 1/1 | Complete | 2026-03-09 |
| 9. WIP Row + Visual Polish | v0.2 | 1/1 | Complete | 2026-03-09 |
| 10. Differentiators | v0.2 | 5/5 | Complete | 2026-03-10 |
| 11. Stash Operations | v0.3 | 6/6 | Complete | 2026-03-12 |
| 12. Commit Context Menu | v0.3 | 2/2 | Complete | 2026-03-12 |
| 13. Remote Operations | v0.3 | 3/3 | Complete | 2026-03-12 |
| 14. Toolbar + Tracking | v0.3 | 3/3 | Complete | 2026-03-12 |
| 15. Graph Data Engine | v0.4 | 2/2 | Complete | 2026-03-12 |
| 16. Core Graph Rendering | v0.4 | 1/1 | Complete | 2026-03-12 |
| 17. Synthetic Row Adaptation | v0.4 | 2/2 | Complete | 2026-03-13 |
| 20. Foundation — Types, Constants & Overlay Container | v0.5 | 2/2 | Complete | 2026-03-14 |
| 21. Active Lanes Transformation | v0.5 | 1/1 | Complete | 2026-03-14 |
| 22. Bezier Path Builder | v0.5 | 1/1 | Complete | 2026-03-14 |
| 23. SVG Rendering | v0.5 | 4/4 | Complete | 2026-03-14 |
| 24. Integration | v0.5 | 1/1 | Complete | 2026-03-14 |
| 25. Interaction Preservation | v0.5 | 1/1 | Complete | 2026-03-14 |
| 26. SVG Ref Pills | v0.5 | 2/2 | Complete | 2026-03-14 |
| 27. Foundation — Icons, Toast & Bug Fixes | 4/4 | Complete    | 2026-03-15 | - |
| 28. Destructive Operations | 3/3 | Complete   | 2026-03-15 | - |
| 29. Staging & Commit UX | v0.6 | 0/? | Not started | - |
| 30. Graph Polish & Navigation | v0.6 | 0/? | Not started | - |
| 31. Layout Polish | v0.6 | 0/? | Not started | - |

---
*Roadmap created: 2026-03-13*
*Last updated: 2026-03-15 — v0.6 roadmap created*
