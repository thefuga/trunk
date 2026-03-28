# Requirements: Trunk

**Defined:** 2026-03-28
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.

## v0.12 Requirements

Requirements for Better Diffs milestone. Each maps to roadmap phases.

### View Modes

- [ ] **VIEW-01**: User can toggle diff between hunk view, full file view, and split (side-by-side) view
- [ ] **VIEW-02**: Split view shows old content on left, new on right, with phantom/spacer rows for alignment
- [ ] **VIEW-03**: Split view panels scroll in sync (locked)
- [ ] **VIEW-04**: Full file view shows entire file with changed lines highlighted (via context_lines=MAX)
- [ ] **VIEW-05**: User can stage/unstage/discard hunks and lines in all view modes (disabled when whitespace ignore is active)

### Syntax Highlighting

- [ ] **SYNT-01**: Diff lines are syntax-highlighted with language-aware coloring
- [ ] **SYNT-02**: Language is auto-detected from file extension
- [ ] **SYNT-03**: Syntax colors are desaturated on add/delete line backgrounds to maintain readability

### Word-Level Diff

- [ ] **WORD-01**: Changed words/characters within modified lines are highlighted with a distinct background
- [ ] **WORD-02**: Word-level diff is skipped for lines over 500 chars or with >60% edit distance (performance guard)

### Whitespace

- [ ] **WHSP-01**: User can toggle whitespace ignore in the diff toolbar (re-fetches diff with ignore_whitespace_change)
- [ ] **WHSP-02**: Hunk/line staging is disabled with tooltip when whitespace ignore is active
- [ ] **WHSP-03**: User can toggle display of invisible characters (spaces as dots, tabs as arrows)

### Context Lines

- [ ] **CTXL-01**: User can select context line count from toolbar dropdown (3/5/10/25/All)
- [ ] **CTXL-02**: Selecting "All" activates full file view mode

### Display Options

- [ ] **DISP-01**: Line numbers shown in diff gutter (old lineno + new lineno)
- [ ] **DISP-02**: User can toggle word wrap in the diff viewer
- [ ] **DISP-03**: All diff display preferences persist across sessions via LazyStore

## Future Requirements

Deferred to future release. Tracked but not in current roadmap.

### Image Diffs

- **IMGD-01**: User can compare image changes with side-by-side, swipe, and onion skin modes
- **IMGD-02**: Pixel diff highlights exact changed pixels

### Advanced Display

- **ADVD-01**: Scrollbar minimap shows compressed diff overview with change markers
- **ADVD-02**: Per-hunk context expand buttons ("show N more lines")
- **ADVD-03**: External diff tool support (Beyond Compare, Kaleidoscope, etc.)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Editable diff (edit-in-diff) | Massive complexity; Trunk is a Git GUI, not an editor; users can double-click to open in external editor |
| Three-way diff for non-conflict files | Already have three-panel merge editor for conflicts (v0.8); two-way covers 99% of review needs |
| Structural/AST diff (difftastic-style) | Requires language-specific parsers; word-level diff covers 80% of value |
| Blame integration in diff | Separate view domain; belongs in own future milestone |
| Copy diff as patch | Low priority for personal use; defer |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| VIEW-01 | — | Pending |
| VIEW-02 | — | Pending |
| VIEW-03 | — | Pending |
| VIEW-04 | — | Pending |
| VIEW-05 | — | Pending |
| SYNT-01 | — | Pending |
| SYNT-02 | — | Pending |
| SYNT-03 | — | Pending |
| WORD-01 | — | Pending |
| WORD-02 | — | Pending |
| WHSP-01 | — | Pending |
| WHSP-02 | — | Pending |
| WHSP-03 | — | Pending |
| CTXL-01 | — | Pending |
| CTXL-02 | — | Pending |
| DISP-01 | — | Pending |
| DISP-02 | — | Pending |
| DISP-03 | — | Pending |

**Coverage:**
- v0.12 requirements: 18 total
- Mapped to phases: 0
- Unmapped: 18 ⚠️

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-28 after initial definition*
