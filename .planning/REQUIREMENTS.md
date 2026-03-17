# Requirements: Trunk

**Defined:** 2026-03-17
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.

## v0.7 Requirements

Requirements for v0.7 Hunk Staging & Search. Each maps to roadmap phases.

### Hunk Staging

- [ ] **HUNK-01**: User can stage individual hunks from the unstaged diff view via a button on each hunk header
- [ ] **HUNK-02**: User can unstage individual hunks from the staged diff view via a button on each hunk header
- [ ] **HUNK-03**: User can discard individual hunks from the working tree with a confirmation prompt
- [ ] **HUNK-04**: DiffPanel displays context-appropriate actions (Stage Hunk for unstaged, Unstage Hunk for staged, no buttons for commit diffs)
- [ ] **HUNK-05**: Diff view refreshes immediately after each hunk stage/unstage/discard operation, reflecting updated hunk boundaries
- [ ] **HUNK-06**: Hunk action buttons are hidden for binary file diffs
- [ ] **HUNK-07**: User can select and stage individual lines within a diff hunk
- [ ] **HUNK-08**: User can select and unstage individual lines within a diff hunk
- [ ] **HUNK-09**: User can navigate between hunks in the diff view using keyboard shortcuts

### Commit Graph Search

- [ ] **SRCH-01**: User can open a search overlay bar in the commit graph with Cmd+F (macOS) / Ctrl+F
- [ ] **SRCH-02**: User can search commits by message text (case-insensitive substring match)
- [ ] **SRCH-03**: User can search commits by SHA or SHA prefix
- [ ] **SRCH-04**: User can search commits by branch or ref name
- [ ] **SRCH-05**: User can search commits by author name
- [ ] **SRCH-06**: Matching commits are visually highlighted in the graph while non-matching commits remain visible
- [ ] **SRCH-07**: Search overlay displays match count (e.g., "3 of 17 matches")
- [ ] **SRCH-08**: User can navigate between matches with Enter (next) and Shift+Enter (previous)
- [ ] **SRCH-09**: Commit graph auto-scrolls to show the current match
- [ ] **SRCH-10**: User can close the search overlay with Escape, preserving scroll position
- [ ] **SRCH-11**: Search results update incrementally as the user types (debounced live search)

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
| HUNK-01 | — | Pending |
| HUNK-02 | — | Pending |
| HUNK-03 | — | Pending |
| HUNK-04 | — | Pending |
| HUNK-05 | — | Pending |
| HUNK-06 | — | Pending |
| HUNK-07 | — | Pending |
| HUNK-08 | — | Pending |
| HUNK-09 | — | Pending |
| SRCH-01 | — | Pending |
| SRCH-02 | — | Pending |
| SRCH-03 | — | Pending |
| SRCH-04 | — | Pending |
| SRCH-05 | — | Pending |
| SRCH-06 | — | Pending |
| SRCH-07 | — | Pending |
| SRCH-08 | — | Pending |
| SRCH-09 | — | Pending |
| SRCH-10 | — | Pending |
| SRCH-11 | — | Pending |

**Coverage:**
- v0.7 requirements: 20 total
- Mapped to phases: 0
- Unmapped: 20 ⚠️

---
*Requirements defined: 2026-03-17*
*Last updated: 2026-03-17 after initial definition*
