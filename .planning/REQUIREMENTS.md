# Requirements: Trunk — v0.13 Code Review Mode

**Defined:** 2026-05-25
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.
**Milestone Goal:** Collect commit/file/line-anchored comments in a review session, then render one markdown file — framed for an AI coding agent — to copy or save.

## v1 Requirements

Requirements for v0.13. Each maps to a roadmap phase. Recipient of the generated doc is an AI coding agent, not a human reviewer — this drives format decisions throughout.

### Session

- [x] **SESS-01**: User can start a code review session for the current repository
- [x] **SESS-02**: User can resume an in-progress review session after restarting the app (session state persists per repo)
- [x] **SESS-03**: User can end and clear the active review session

### Commit Selection

- [x] **SEL-01**: User can seed a review session from a commit range (base → tip)
- [x] **SEL-02**: User can add individual commits to the session from the graph context menu
- [x] **SEL-03**: User can remove a commit from the session
- [x] **SEL-04**: User can see the list of commits included in the session

### Anchor Capture

- [x] **ANCH-01**: User can select a line range in the diff view and attach a comment, anchored to commit + file + diff line range with a side discriminator
- [ ] **ANCH-02**: User can select a line range in the full-file-at-commit view and attach a comment, anchored to commit + file + source line range
- [ ] **ANCH-03**: User can attach a commit-level comment with no code anchor

### Comment Management

- [ ] **CMT-01**: User can view all comments in the active session in a review panel
- [ ] **CMT-02**: User can edit a comment's text
- [ ] **CMT-03**: User can delete a comment, with confirmation
- [ ] **CMT-04**: User can jump from a comment to its anchored code location

### Markdown Render

- [ ] **DOC-01**: User can generate one markdown document from the session (commit refs + code excerpts + comments)
- [ ] **DOC-02**: Excerpts render diff-fenced for diff-source comments and language-fenced for full-file-source comments, with fence length computed to avoid backtick collisions
- [ ] **DOC-03**: Comments group by file and order by line, each under a `path:Lstart-Lend (sha)` location heading; commit-level comments render in a trailing section
- [ ] **DOC-04**: Comments whose anchor can no longer be resolved render in a dedicated "unresolvable" section, never silently dropped and never crashing the render

### Output

- [ ] **OUT-01**: User can copy the generated markdown to the clipboard
- [ ] **OUT-02**: User can save the generated markdown to a file via a native save dialog

## Future Requirements

Deferred to a later milestone. Tracked but not in the v0.13 roadmap.

### Render & Output Enhancements

- **DOC-F1**: Live markdown preview pane before export
- **DOC-F2**: Per-comment include/exclude toggle (omit from doc without deleting)
- **DOC-F3**: GitHub-style suggestion/replacement blocks carrying a proposed edit
- **DOC-F4**: Auto-trim excerpts to the tightest meaningful subrange
- **OUT-F1**: Configurable filename / output template
- **OUT-F2**: Copy a single comment as a standalone prompt

### Session Enhancements

- **SESS-F1**: Multiple concurrent review sessions per repo

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Threaded comment replies | Single-user AI-review tool; threading is overkill for a one-shot artifact |
| Severity tags (nit/blocker/etc.) | Recipient is an AI agent — the comment phrasing IS the instruction; tags add noise |
| Approve / request-changes state machine | Not a forge review workflow; the deliverable is a static markdown doc |
| Re-anchoring comments on history rewrite | Sessions assume a stable range; the generated file is a static snapshot, never re-synced |
| GitHub / GitLab review posting | Local markdown only — copy or save, no forge integration |
| Auto-generated review findings | The human writes the comments; this tool collects and renders them |

## Traceability

Which phases cover which requirements. Phase numbers continue from v0.12 (last phase 64), so v0.13 starts at Phase 65. Finalized by the roadmapper.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SESS-01 | Phase 65 | Complete |
| SESS-02 | Phase 65 | Complete |
| SESS-03 | Phase 65 | Complete |
| SEL-01 | Phase 66 | Complete |
| SEL-02 | Phase 66 | Complete |
| SEL-03 | Phase 66 | Complete |
| SEL-04 | Phase 66 | Complete |
| ANCH-01 | Phase 67 | Complete |
| ANCH-02 | Phase 68 | Pending |
| ANCH-03 | Phase 69 | Pending |
| CMT-01 | Phase 69 | Pending |
| CMT-02 | Phase 69 | Pending |
| CMT-03 | Phase 69 | Pending |
| CMT-04 | Phase 69 | Pending |
| DOC-01 | Phase 70 | Pending |
| DOC-02 | Phase 70 | Pending |
| DOC-03 | Phase 70 | Pending |
| DOC-04 | Phase 70 | Pending |
| OUT-01 | Phase 71 | Pending |
| OUT-02 | Phase 71 | Pending |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-25*
*Last updated: 2026-05-25 — traceability finalized by roadmapper (Phases 65-71); all 20 v1 requirements mapped*
