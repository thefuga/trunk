# Requirements: Trunk — v0.14 Commit Message UX

**Defined:** 2026-05-28
**Core Value:** A developer can open any Git repository, browse its full commit history as a visual graph, stage files, and create commits — all without touching the terminal.
**Milestone goal:** Make `merge --continue`, `merge <branch>`, and `revert <oid>` open a pre-populated commit-message editor matching git's terminal `$EDITOR` behavior, eliminating the `GIT_EDITOR=true` / `--no-edit` bypasses that silently swallow the message.

## v1 Requirements

Requirements for v0.14. Each maps to a roadmap phase.

### Message Editor (MSG)

- [ ] **MSG-01**: User can edit the commit message at the commit step of `merge --continue` (after resolving all conflicts), with the editor pre-filled from `.git/MERGE_MSG`.
- [ ] **MSG-02**: User can edit the commit message at the commit step of `merge <branch>` (non-fast-forward merges that produce a merge commit), with the editor pre-filled with `"Merge branch '<branch>'"` (or `"Merge remote-tracking branch 'origin/<branch>'"` for remote branches).
- [ ] **MSG-03**: User can edit the commit message at the commit step of `revert <oid>`, with the editor pre-filled with `Revert "<original subject>"` followed by `This reverts commit <oid>.`
- [x] **MSG-04**: The message editor pre-fills with git's default message for each operation. Source-of-truth defaults: `.git/MERGE_MSG` for continue, constructed string for merge/revert built by the Rust backend (never hardcoded in the frontend).
- [x] **MSG-05**: User can cancel the message editor (Esc or Cancel button), leaving the repo in the same state as before opening it — no commit created, no half-finished state, no orphan temp files.
- [ ] **MSG-06**: Empty or whitespace-only message aborts the operation (matches git CLI behavior where saving an empty `$EDITOR` buffer aborts the commit). Operation must leave repo in a clean state: for merge, repo stays mid-merge with conflicts already resolved on disk (user can edit message and retry, or run Abort); for revert, repo stays in `REVERT_HEAD` state (user can retry or abort).

## v2 Requirements

Acknowledged but deferred to a future milestone.

### Commit Signing

- **SIGN-01**: User can configure commit signing (GPG/SSH) via UI
- **SIGN-02**: User can sign individual commits with `-S` flag
- **SIGN-03**: User can add `--signoff` to commits via UI toggle

### Message Templates

- **TMPL-01**: User can configure `commit.template` via Settings UI (depends on Settings UI deferred to v1.0)

## Out of Scope

Explicitly excluded from v0.14 to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Fast-forward merge message editing | git CLI doesn't open `$EDITOR` for fast-forward merges; no commit is created. Trunk should match. |
| Cherry-pick message editing | Already works via the interactive-rebase `GIT_EDITOR=<script>` pattern shipped in v0.8. Out of v0.14 scope; revisit if user reports drift. |
| Squash / reword message editing during interactive rebase | Already shipped in v0.8 (RebaseEditor + file-based IPC). |
| Pre-commit / commit-msg hook output display | Separate concern; hooks run via git CLI and their stdout/stderr would need streaming UI. Out of v0.14. |
| `--no-verify` toggle | Hook bypassing is a power-user feature; not part of message UX. Defer with Settings UI. |
| Multi-line message editor with rich preview | Modal is plain textarea (matches `$EDITOR` semantics); no markdown preview, no diff preview alongside. Keep simple. |
| Persisting draft messages across modal cancels | Cancel = throw away. Matches CLI where killing `$EDITOR` discards changes. |

## Traceability

Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| MSG-01 | Phase 76 | In Progress (76-01 backend: merge_continue --cleanup=strip; frontend wiring pending 76-03) |
| MSG-02 | Phase 76 | In Progress (76-01 backend: merge_branch_begin + verbatim MERGE_MSG; frontend wiring pending 76-03) |
| MSG-03 | Phase 76 | Pending |
| MSG-04 | Phase 75 | Complete |
| MSG-05 | Phase 75 | Complete |
| MSG-06 | Phase 76 | Pending |

**Coverage:**

- v1 requirements: 6 total
- Mapped to phases: 6
- Unmapped: 0 ✓

---
*Requirements defined: 2026-05-28*
*Last updated: 2026-05-28 after initial definition*
