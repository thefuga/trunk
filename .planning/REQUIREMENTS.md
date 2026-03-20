# Requirements: Trunk

**Defined:** 2026-03-20
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.

## v0.8 Requirements

Requirements for v0.8 Conflict & Rebase milestone. GitKraken-parity scope.

### Conflict Resolution

- [ ] **CONF-01**: Conflicted files display as a distinct third section in the staging panel with warning styling and count badge
- [ ] **CONF-02**: Three-panel merge editor opens when user clicks a conflicted file (current/incoming on top, editable output on bottom)
- [ ] **CONF-03**: Merge editor panels scroll in sync across all three panels
- [ ] **CONF-04**: Per-hunk checkboxes on current and incoming panels add/remove hunk content to/from the output
- [ ] **CONF-05**: Per-line click selection on current and incoming panels toggles individual lines into/out of the output
- [ ] **CONF-06**: Output panel is directly editable as a text editor for manual merge adjustments
- [ ] **CONF-07**: "Take All Current" and "Take All Incoming" buttons resolve the entire file with one click (available in merge tool toolbar and as right-click on conflicted file)
- [ ] **CONF-08**: Prev/Next conflict navigation arrows jump between conflict sections within a file
- [ ] **CONF-09**: "Save and Mark Resolved" saves the output, stages the file, and auto-opens the next conflicted file

### Operation State

- [ ] **OPS-01**: Persistent banner displays when a merge is in progress (detected via .git/MERGE_HEAD) with Continue and Abort buttons
- [ ] **OPS-02**: Persistent banner displays when a rebase is in progress (detected via .git/rebase-merge/ or .git/rebase-apply/) with Continue, Skip, and Abort buttons
- [ ] **OPS-03**: Continue/Abort/Skip buttons invoke the corresponding git CLI commands and refresh the UI

### Merge Workflow

- [ ] **MERGE-01**: User can merge a branch into the current branch via right-click context menu on a branch (sidebar or graph pill)
- [ ] **MERGE-02**: User can initiate merge by dragging a branch onto another branch in the graph, selecting "Merge" from the resulting context menu
- [ ] **MERGE-03**: Fast-forward merges advance the branch pointer without creating a merge commit, with toast confirmation
- [ ] **MERGE-04**: Non-conflicting merges auto-create a merge commit with standard message and refresh the graph

### Rebase Workflow

- [ ] **REB-01**: User can rebase current branch onto another branch via right-click context menu on a branch
- [ ] **REB-02**: User can initiate rebase by dragging a branch onto another branch in the graph, selecting "Rebase" from the resulting context menu
- [ ] **REB-03**: User can start interactive rebase by right-clicking a commit in the graph and selecting "Interactive Rebase"
- [ ] **REB-04**: Mid-rebase conflicts pause the rebase and show conflicted files in the staging panel for resolution via the merge editor
- [ ] **REB-05**: User can abort an in-progress rebase to restore the repository to its pre-rebase state
- [ ] **REB-06**: User can skip a conflicting commit during rebase and continue with the next commit

### Interactive Rebase Editor

- [ ] **IREB-01**: Interactive rebase opens a modal/panel showing all commits to be rebased with action selectors (Pick/Squash/Reword/Drop)
- [ ] **IREB-02**: User can reorder commits by dragging rows up/down in the rebase editor
- [ ] **IREB-03**: Keyboard shortcuts assign actions to focused commit rows (P=Pick, S=Squash, R=Reword, D=Drop)
- [ ] **IREB-04**: Start Rebase button validates the plan (e.g., can't squash first commit) and executes the rebase
- [ ] **IREB-05**: Cancel button closes the editor with no changes; Reset button restores all commits to original Pick state and order
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
| CONF-01 | — | Pending |
| CONF-02 | — | Pending |
| CONF-03 | — | Pending |
| CONF-04 | — | Pending |
| CONF-05 | — | Pending |
| CONF-06 | — | Pending |
| CONF-07 | — | Pending |
| CONF-08 | — | Pending |
| CONF-09 | — | Pending |
| OPS-01 | — | Pending |
| OPS-02 | — | Pending |
| OPS-03 | — | Pending |
| MERGE-01 | — | Pending |
| MERGE-02 | — | Pending |
| MERGE-03 | — | Pending |
| MERGE-04 | — | Pending |
| REB-01 | — | Pending |
| REB-02 | — | Pending |
| REB-03 | — | Pending |
| REB-04 | — | Pending |
| REB-05 | — | Pending |
| REB-06 | — | Pending |
| IREB-01 | — | Pending |
| IREB-02 | — | Pending |
| IREB-03 | — | Pending |
| IREB-04 | — | Pending |
| IREB-05 | — | Pending |
| IREB-06 | — | Pending |
| IREB-07 | — | Pending |

**Coverage:**
- v0.8 requirements: 29 total
- Mapped to phases: 0
- Unmapped: 29 ⚠️

---
*Requirements defined: 2026-03-20*
*Last updated: 2026-03-20 after initial definition*
