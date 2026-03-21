# Requirements: Trunk

**Defined:** 2026-03-20
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.

## v0.8 Requirements

Requirements for v0.8 Conflict & Rebase milestone. GitKraken-parity scope.

### Conflict Resolution

- [x] **CONF-01**: Conflicted files display as a distinct third section in the staging panel with warning styling and count badge
- [x] **CONF-02**: Three-panel merge editor opens when user clicks a conflicted file (current/incoming on top, editable output on bottom)
- [x] **CONF-03**: Merge editor panels scroll in sync across all three panels
- [x] **CONF-04**: Per-hunk checkboxes on current and incoming panels add/remove hunk content to/from the output
- [x] **CONF-05**: Per-line click selection on current and incoming panels toggles individual lines into/out of the output
- [x] **CONF-06**: Output panel is directly editable as a text editor for manual merge adjustments
- [x] **CONF-07**: "Take All Current" and "Take All Incoming" buttons resolve the entire file with one click (available in merge tool toolbar and as right-click on conflicted file)
- [x] **CONF-08**: Prev/Next conflict navigation arrows jump between conflict sections within a file
- [x] **CONF-09**: "Save and Mark Resolved" saves the output, stages the file, and auto-opens the next conflicted file

### Operation State

- [x] **OPS-01**: Persistent banner displays when a merge is in progress (detected via .git/MERGE_HEAD) with Continue and Abort buttons
- [x] **OPS-02**: Persistent banner displays when a rebase is in progress (detected via .git/rebase-merge/ or .git/rebase-apply/) with Continue, Skip, and Abort buttons
- [x] **OPS-03**: Continue/Abort/Skip buttons invoke the corresponding git CLI commands and refresh the UI

### Merge Workflow

- [x] **MERGE-01**: User can merge a branch into the current branch via right-click context menu on a branch (sidebar or graph pill)
- [ ] **MERGE-02**: ~~User can initiate merge by dragging a branch onto another branch in the graph~~ — Dropped per user decision (no drag-and-drop)
- [x] **MERGE-03**: Fast-forward merges advance the branch pointer without creating a merge commit silently (no toast), graph refreshes
- [x] **MERGE-04**: Non-conflicting merges auto-create a merge commit with standard message and refresh the graph

### Rebase Workflow

- [x] **REB-01**: User can rebase current branch onto another branch via right-click context menu on a branch
- [ ] **REB-02**: ~~User can initiate rebase by dragging a branch onto another branch in the graph, selecting "Rebase" from the resulting context menu~~ — Dropped per user decision
- [ ] **REB-03**: User can start interactive rebase by right-clicking a commit in the graph and selecting "Interactive Rebase"
- [x] **REB-04**: Mid-rebase conflicts pause the rebase and show conflicted files in the staging panel for resolution via the merge editor
- [x] **REB-05**: User can abort an in-progress rebase to restore the repository to its pre-rebase state
- [x] **REB-06**: User can skip a conflicting commit during rebase and continue with the next commit

### Interactive Rebase Editor

- [x] **IREB-01**: Interactive rebase opens a modal/panel showing all commits to be rebased with action selectors (Pick/Squash/Reword/Drop)
- [ ] **IREB-02**: User can reorder commits by dragging rows up/down in the rebase editor
- [ ] **IREB-03**: Keyboard shortcuts assign actions to focused commit rows (P=Pick, S=Squash, R=Reword, D=Drop)
- [x] **IREB-04**: Start Rebase button validates the plan (e.g., can't squash first commit) and executes the rebase
- [x] **IREB-05**: Cancel button closes the editor with no changes; Reset button restores all commits to original Pick state and order
- [ ] **IREB-06**: Reword action shows a message editor dialog when the rebase reaches that commit
- [ ] **IREB-07**: Squash action combines the commit with its predecessor; a message editor shows the concatenated messages before execution

## Future Requirements

Deferred differentiators beyond GitKraken parity.

### Rebase Extras

- **IREB-EX-01**: Fixup action (like Squash but discards commit message)
- **IREB-EX-02**: Edit action (pause rebase at commit to amend it)
- **IREB-EX-03**: Multi-commit selection for bulk action assignment
- **IREB-EX-04**: Base (ancestor) version toggle in merge editor
- **IREB-EX-05**: Undo/redo for rebase operations
- **IREB-EX-06**: Squash message editor before starting rebase
- **IREB-EX-07**: Cherry-pick multiple commits via rebase editor

## Out of Scope

| Feature | Reason |
|---------|--------|
| AI-powered conflict resolution | Massive complexity (LLM integration), questionable value, GitKraken treats as paid preview |
| Conflict prevention / team detection | Requires organization infrastructure and cloud sync — team collaboration feature, not conflict resolution |
| External merge tool integration (kdiff3, meld, etc.) | Configuration complexity; users can use `git mergetool` in terminal |
| `git rebase --exec` support | Niche CI/automation use case, security concerns with arbitrary commands |
| Force push flow after rebase | Adds scope; toast warning is sufficient, user handles force push via toolbar |
| In-memory rebase | Marginal UX benefit over standard on-disk rebase |
| Syntax highlighting in merge editor | Nice-to-have, not table stakes; defer to future polish |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONF-01 | Phase 37 | Complete |
| CONF-02 | Phase 38 | Complete |
| CONF-03 | Phase 38 | Complete |
| CONF-04 | Phase 38 | Complete |
| CONF-05 | Phase 38 | Complete |
| CONF-06 | Phase 38 | Complete |
| CONF-07 | Phase 38 | Complete |
| CONF-08 | Phase 38 | Complete |
| CONF-09 | Phase 38 | Complete |
| OPS-01 | Phase 37 | Complete |
| OPS-02 | Phase 37 | Complete |
| OPS-03 | Phase 37 | Complete |
| MERGE-01 | Phase 39 | Complete |
| MERGE-02 | Phase 39 | Dropped |
| MERGE-03 | Phase 39 | Complete |
| MERGE-04 | Phase 39 | Complete |
| REB-01 | Phase 40 | Complete |
| REB-02 | Phase 40 | Dropped |
| REB-03 | Phase 41 | Pending |
| REB-04 | Phase 40 | Complete |
| REB-05 | Phase 40 | Complete |
| REB-06 | Phase 40 | Complete |
| IREB-01 | Phase 41 | Complete |
| IREB-02 | Phase 41 | Pending |
| IREB-03 | Phase 41 | Pending |
| IREB-04 | Phase 41 | Complete |
| IREB-05 | Phase 41 | Complete |
| IREB-06 | Phase 41 | Pending |
| IREB-07 | Phase 41 | Pending |

**Coverage:**
- v0.8 requirements: 29 total
- Mapped to phases: 29
- Unmapped: 0

---
*Requirements defined: 2026-03-20*
*Last updated: 2026-03-20 after roadmap creation*
