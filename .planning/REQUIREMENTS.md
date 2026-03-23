# Requirements: Trunk

**Defined:** 2026-03-23
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.

## v0.9 Requirements

Requirements for milestone v0.9 Multi-tab & Tree View. Each maps to roadmap phases.

### Multi-tab

- [ ] **TAB-01**: User can open multiple repositories as separate tabs in a single window
- [ ] **TAB-02**: User can create a new tab via Cmd+T, which shows the splash/project picker
- [ ] **TAB-03**: User can close a tab via Cmd+W or the X button on the tab
- [ ] **TAB-04**: User can switch tabs via Cmd+1-9 and Ctrl+Tab/Ctrl+Shift+Tab
- [ ] **TAB-05**: Each tab has fully independent state (graph, staging, diffs, selection, rebase/merge)
- [ ] **TAB-06**: Open tabs and active tab are persisted and restored on app relaunch
- [ ] **TAB-07**: Background tabs with uncommitted changes show a dirty indicator (dot badge)
- [ ] **TAB-08**: User can right-click a tab for context menu (Close Others, Close All, Copy Path)
- [ ] **TAB-09**: User can middle-click a tab to close it
- [ ] **TAB-10**: Opening a repo that's already in a tab switches to the existing tab instead of duplicating

### Tree View

- [ ] **TREE-01**: User can toggle between flat file list and directory tree view in staging panel
- [ ] **TREE-02**: User can toggle between flat file list and directory tree view in commit diffs
- [ ] **TREE-03**: User can toggle between flat file list and directory tree view in merge editor
- [ ] **TREE-04**: Directory nodes expand/collapse with chevron indicators
- [ ] **TREE-05**: Expand/collapse state is preserved across status refreshes (staging a file doesn't collapse the tree)
- [ ] **TREE-06**: User can navigate the tree with arrow keys (up/down to move, left/right to collapse/expand)
- [ ] **TREE-07**: Single-child directory paths are compressed (e.g. src/lib/ instead of src > lib)
- [ ] **TREE-08**: User can stage/unstage an entire directory via action on the directory node
- [ ] **TREE-09**: Directory nodes show file count badges
- [ ] **TREE-10**: User can Expand All / Collapse All via buttons in the file list header

### Backend

- [x] **BACK-01**: Concurrent remote operations across tabs (each tab can fetch/push independently)

## Future Requirements

Deferred to future milestones. Tracked but not in current roadmap.

### Multi-tab (Future)

- **TAB-F01**: Drag tab to new OS window (requires Tauri multi-window)
- **TAB-F02**: Workspace/group management (team feature)
- **TAB-F03**: Tab search / fuzzy finder

### Tree View (Future)

- **TREE-F01**: Virtual scrolling for file tree (needed only for >500 files)
- **TREE-F02**: Auto-open submodules as child tabs

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multi-window (drag tab to new window) | Requires Tauri multi-window, state serialization, cross-window IPC — very high complexity |
| Workspace/group management | GitKraken-style cloud team feature; overkill for personal desktop use |
| Tab search/fuzzy finder | Cmd+1-9 and Ctrl+Tab cover navigation at v0.9 scale |
| Virtual scrolling for file tree | Staging file counts rarely exceed 500; not needed |
| Submodule auto-open | Niche use case, complex to implement correctly |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BACK-01 | Phase 44 | Complete |
| TAB-01 | Phase 45 | Pending |
| TAB-02 | Phase 45 | Pending |
| TAB-03 | Phase 45 | Pending |
| TAB-04 | Phase 45 | Pending |
| TAB-05 | Phase 45 | Pending |
| TAB-06 | Phase 45 | Pending |
| TAB-07 | Phase 45 | Pending |
| TREE-07 | Phase 46 | Pending |
| TREE-01 | Phase 47 | Pending |
| TREE-02 | Phase 47 | Pending |
| TREE-03 | Phase 47 | Pending |
| TREE-04 | Phase 47 | Pending |
| TREE-05 | Phase 47 | Pending |
| TREE-06 | Phase 47 | Pending |
| TAB-08 | Phase 48 | Pending |
| TAB-09 | Phase 48 | Pending |
| TAB-10 | Phase 48 | Pending |
| TREE-08 | Phase 48 | Pending |
| TREE-09 | Phase 48 | Pending |
| TREE-10 | Phase 48 | Pending |

**Coverage:**
- v0.9 requirements: 21 total
- Mapped to phases: 21
- Unmapped: 0

---
*Requirements defined: 2026-03-23*
*Last updated: 2026-03-23 after roadmap creation (traceability added)*
