# Requirements: Trunk

**Defined:** 2026-03-17
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.

## v0.7 Requirements

Requirements for v0.7 Hunk Staging & Search. Each maps to roadmap phases.

### Hunk Staging

- [x] **HUNK-01**: User can stage individual hunks from the unstaged diff view via a button on each hunk header
- [x] **HUNK-02**: User can unstage individual hunks from the staged diff view via a button on each hunk header
- [x] **HUNK-03**: User can discard individual hunks from the working tree with a confirmation prompt
- [x] **HUNK-04**: DiffPanel displays context-appropriate actions (Stage Hunk for unstaged, Unstage Hunk for staged, no buttons for commit diffs)
- [x] **HUNK-05**: Diff view refreshes immediately after each hunk stage/unstage/discard operation, reflecting updated hunk boundaries
- [x] **HUNK-06**: Hunk action buttons are hidden for binary file diffs
- [x] **HUNK-07**: User can select and stage individual lines within a diff hunk
- [x] **HUNK-08**: User can select and unstage individual lines within a diff hunk
- [x] **HUNK-09**: User can navigate between hunks in the diff view using keyboard shortcuts

### Commit Graph Search

- [x] **SRCH-01**: User can open a search overlay bar in the commit graph with Cmd+F (macOS) / Ctrl+F
- [x] **SRCH-02**: User can search commits by message text (case-insensitive substring match)
- [x] **SRCH-03**: User can search commits by SHA or SHA prefix
- [x] **SRCH-04**: User can search commits by branch or ref name
- [x] **SRCH-05**: User can search commits by author name
- [ ] **SRCH-06**: Matching commits are visually highlighted in the graph while non-matching commits remain visible
- [x] **SRCH-07**: Search overlay displays match count (e.g., "3 of 17 matches")
- [ ] **SRCH-08**: User can navigate between matches with Enter (next) and Shift+Enter (previous)
- [ ] **SRCH-09**: Commit graph auto-scrolls to show the current match
- [ ] **SRCH-10**: User can close the search overlay with Escape, preserving scroll position
- [x] **SRCH-11**: Search results update incrementally as the user types (debounced live search)

## Future Requirements

Deferred to future releases. Tracked but not in current roadmap.

### Diff Enhancement

- **DIFF-01**: Split (side-by-side) diff view mode
- **DIFF-02**: Expandable context lines around hunks (show more/less)
- **DIFF-03**: Drag-to-expand context boundaries (Sublime Merge style)

### Search Enhancement

- **SRCH-E01**: Scoped search prefixes (`author:`, `sha:`, `branch:`)
- **SRCH-E02**: Regex search toggle
- **SRCH-E03**: Persist search state across panel switches

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Graph filtering (hide non-matching commits) | Extremely complex — breaks virtual list + SVG overlay + lane algorithm. Fork/Sublime Merge took years. |
| Fuzzy search (fzf-style) | Commit messages and SHAs are not fuzzy targets. Substring match is the standard. |
| Inline editing in diff view | Separate complex feature. Mixing editing with staging creates ambiguous state. |
| Explicit "Split Hunk" button | No GUI tool has this. Line-level staging solves the same need. |
| Search by file path | Requires walking commit trees — expensive. Not a core search use case. |
| Saved searches / search history | Over-engineering for v0.7. |
| Cross-panel search | Each panel should have its own search. Commit graph search only. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| HUNK-01 | 32 | Planned |
| HUNK-02 | 32 | Planned |
| HUNK-03 | 32 | Planned |
| HUNK-04 | 33 | Planned |
| HUNK-05 | 32 | Planned |
| HUNK-06 | 33 | Planned |
| HUNK-07 | 34 | Planned |
| HUNK-08 | 34 | Planned |
| HUNK-09 | 33 | Planned |
| SRCH-01 | 36 | Planned |
| SRCH-02 | 35 | Planned |
| SRCH-03 | 35 | Planned |
| SRCH-04 | 35 | Planned |
| SRCH-05 | 35 | Planned |
| SRCH-06 | 36 | Planned |
| SRCH-07 | 36 | Planned |
| SRCH-08 | 36 | Planned |
| SRCH-09 | 36 | Planned |
| SRCH-10 | 36 | Planned |
| SRCH-11 | 35 | Planned |

**Coverage:**
- v0.7 requirements: 20 total
- Mapped to phases: 20 ✅
- Unmapped: 0

---
*Requirements defined: 2026-03-17*
*Last updated: 2026-03-17 after roadmap mapping*
