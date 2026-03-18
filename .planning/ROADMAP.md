# Roadmap: Trunk

## Milestones

- ✅ **v0.1 MVP** — Phases 1-6 (shipped 2026-03-09)
- ✅ **v0.2 Commit Graph** — Phases 7-10 (shipped 2026-03-10)
- ✅ **v0.3 Actions** — Phases 11-14 (shipped 2026-03-12)
- ✅ **v0.4 Graph Rework** — Phases 15-17 (shipped 2026-03-13)
- ✅ **v0.5 Graph Overlay** — Phases 20-26 (shipped 2026-03-15)
- ✅ **v0.6 UI Polish & Core Ops** — Phases 27-31 (shipped 2026-03-16)
- 🔲 **v0.7 Hunk Staging & Search** — Phases 32-36 (active)

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

<details>
<summary>✅ v0.6 UI Polish & Core Ops (Phases 27-31) — SHIPPED 2026-03-16</summary>

- [x] Phase 27: Foundation — Icons, Toast & Bug Fixes (5/5 plans) — completed 2026-03-15
- [x] Phase 27.1: Add icons to commit graph pills (1/1 plans) — completed 2026-03-15 (INSERTED)
- [x] Phase 28: Destructive Operations (4/4 plans) — completed 2026-03-15
- [x] Phase 29: Staging & Commit UX (2/2 plans) — completed 2026-03-15
- [x] Phase 30: Graph Polish & Navigation (3/3 plans) — completed 2026-03-16
- [x] Phase 31: Layout Polish (1/1 plans) — completed 2026-03-16

Full details: [milestones/v0.6-ROADMAP.md](milestones/v0.6-ROADMAP.md)

</details>

## Current: v0.7 Hunk Staging & Search (Phases 32-36)

- [x] Phase 32: Hunk Staging Backend (4 requirements) (completed 2026-03-18)
  **Goal:** Implement Rust commands for staging, unstaging, and discarding individual hunks using git2's apply API with single-hunk patch extraction.
  **Requirements:** HUNK-01, HUNK-02, HUNK-03, HUNK-05
  **Plans:** 1 plan
  Plans:
  - [ ] 32-01-PLAN.md — TDD: implement stage_hunk, unstage_hunk, discard_hunk with tests and Tauri wiring
  **Success criteria:**
  1. `stage_hunk` command applies a single hunk to the index via `repo.apply(&diff, ApplyLocation::Index)` and returns success
  2. `unstage_hunk` command applies a reversed patch to remove a single hunk from the index
  3. `discard_hunk` command reverts a single hunk in the working directory with confirmation required from frontend
  4. After any hunk operation, re-fetching the diff shows updated hunk boundaries (no stale indices)
  5. All commands follow the inner-fn pattern and have unit tests covering multi-hunk files, single-hunk new files, and no-newline-at-EOF

- [x] Phase 33: Hunk Staging UI (3 requirements) (completed 2026-03-18)
  **Goal:** Add context-aware hunk action buttons to DiffPanel with binary file guards and keyboard navigation between hunks.
  **Requirements:** HUNK-04, HUNK-06, HUNK-09
  **Plans:** 1 plan
  Plans:
  - [ ] 33-01-PLAN.md — Add hunk toolbar rows with action buttons and [/] keyboard navigation to DiffPanel
  **Success criteria:**
  1. DiffPanel shows "Stage Hunk" button on each `@@` header for unstaged diffs, "Unstage Hunk" button for staged diffs, and no buttons for commit diffs
  2. Binary file diffs display no hunk action buttons
  3. User can press `]` to jump to next hunk and `[` to jump to previous hunk within the diff view
  4. Hunk buttons are disabled during in-flight operations to prevent stale-index races

- [ ] Phase 34: Line-Level Staging (2 requirements)
  **Goal:** Enable selecting and staging/unstaging individual lines within a diff hunk, constructing partial patches from line selections.
  **Requirements:** HUNK-07, HUNK-08
  **Plans:** 2 plans
  Plans:
  - [ ] 34-01-PLAN.md — TDD: partial patch construction + stage_lines/unstage_lines/discard_lines backend
  - [ ] 34-02-PLAN.md — Line selection UI + toolbar mode switching + IPC handlers in DiffPanel
  **Success criteria:**
  1. User can click or shift-click to select individual lines within a diff hunk
  2. "Stage Lines" action constructs a valid partial patch from selected added/removed lines and applies it to the index
  3. "Unstage Lines" action constructs a reversed partial patch from selected lines in the staged diff
  4. Selected lines are visually highlighted and the selection clears after the operation completes

- [ ] Phase 35: Search Backend (5 requirements)
  **Goal:** Implement a backend search command that queries CommitCache for commits matching SHA, message, branch/ref, or author — returning all matches regardless of frontend pagination.
  **Requirements:** SRCH-02, SRCH-03, SRCH-04, SRCH-05, SRCH-11
  **Success criteria:**
  1. `search_commits` command returns `Vec<SearchResult>` (OID + match type) from CommitCache
  2. SHA prefix search matches via `oid.starts_with(query)`
  3. Message search performs case-insensitive substring match on summary and body fields
  4. Ref search matches branch names, tag names, and remote ref short names
  5. Author search performs case-insensitive substring match on author name

- [ ] Phase 36: Search UI (6 requirements)
  **Goal:** Build a floating search overlay bar in CommitGraph with Cmd+F activation, match highlighting, prev/next navigation with auto-scroll, and Escape to close.
  **Requirements:** SRCH-01, SRCH-06, SRCH-07, SRCH-08, SRCH-09, SRCH-10
  **Success criteria:**
  1. Cmd+F (macOS) / Ctrl+F opens a floating SearchBar overlay inside CommitGraph (with WebView native find suppressed)
  2. Matching commit rows are visually highlighted in the graph while non-matching rows remain visible but un-highlighted
  3. Search overlay displays match count (e.g., "3 of 17 matches") updating live as user types (debounced 200ms)
  4. Enter navigates to next match, Shift+Enter to previous match, with the graph auto-scrolling via existing `scrollToOid`
  5. Escape closes the search overlay and preserves the current scroll position

---
*Roadmap created: 2026-03-13*
*Last updated: 2026-03-18 — Phase 34 planned (2 plans)*
