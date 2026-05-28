# Phase 76: Wire MessageEditor into merge/continue, merge, and revert - Context

**Gathered:** 2026-05-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Route the three git operations — `merge --continue`, `merge <branch>` (non-fast-forward), and `revert <oid>` — through the Phase 75 `MessageEditor` modal so the user can edit a pre-filled commit message, then remove the `GIT_EDITOR=true` / `--no-edit` bypasses that currently swallow the message. Empty/whitespace-only message aborts the operation cleanly, leaving the repo in a recoverable state. Fast-forward merges continue to skip the editor (no merge commit → no message).

Maps requirements MSG-01 (merge --continue), MSG-02 (merge <branch>), MSG-03 (revert), MSG-06 (empty → abort, recoverable). MSG-04/MSG-05 were delivered as infrastructure in Phase 75.

**This phase wires existing primitives.** The MessageEditor modal and its `open(default) → Promise<string|null>` contract are frozen (Phase 75-01). This phase changes production code paths in `operation_state.rs`, `commit_actions.rs`, and the frontend trigger sites.

</domain>

<decisions>
## Implementation Decisions

### Message-injection mechanism (D-01) — the spine of the phase

**D-01: Direct/unified `git commit -m` approach. Phase 75's `editor.rs` GIT_EDITOR-script helper goes UNUSED.**

All three operations follow one pattern:
1. Put git into the operation's in-progress state **without committing**:
   - `merge --continue`: already in-progress (conflicts resolved on disk) — `.git/MERGE_MSG` already present.
   - `merge <branch>` (non-ff): `git merge --no-commit <branch>` → sets `MERGE_HEAD`, writes `.git/MERGE_MSG`.
   - `revert <oid>`: `git revert --no-commit <oid>` → sets `REVERT_HEAD`, writes `.git/MERGE_MSG`.
2. Backend reads `.git/MERGE_MSG` to get the **default message built by git itself** (satisfies MSG-04 "built by Rust backend, never hardcoded in frontend" — and is *more* faithful than a constructed string).
3. Frontend opens `MessageEditor.open(default)`.
4. On Save (non-null): backend runs `git commit -m <edited_msg>` — finalizes the merge/revert commit and clears `MERGE_HEAD`/`REVERT_HEAD`.
5. On null (Cancel/Esc/backdrop/empty/whitespace): backend does NOT commit; repo stays in its recoverable in-progress state (see D-02).

**Why this over the Phase 75 GIT_EDITOR script:**
- Verified (scratch-repo test, 2026-05-28): `git revert --no-commit` and `git merge --no-commit --no-ff` BOTH set `REVERT_HEAD`/`MERGE_HEAD` (the MSG-06 recoverable states) AND write `.git/MERGE_MSG` with the *exact* spec'd defaults verbatim:
  - revert → `Revert "<subject>"\n\nThis reverts commit <full-oid>.` (matches MSG-03)
  - merge → `Merge branch '<branch>'` (matches MSG-02; remote-tracking variant produced by git the same way)
- `merge_continue_inner` (operation_state.rs:158-165) ALREADY has a working `git commit -m <msg>` path. The direct approach reuses it.
- The GIT_EDITOR script would make git re-launch an "editor" that just dumps a message we already collected in the GUI — pure indirection.
- `git revert` has no `-m`-for-message flag (`-m` selects the mainline parent), so the script OR `--no-commit`+`git commit` are the only revert options; `--no-commit` is the simpler of the two.

**editor.rs fate (D-01a):** `src-tauri/src/git/editor.rs` + its `tempfile` runtime-dep promotion become dead code after this phase. **Do NOT delete editor.rs in Phase 76** (keeps the blast radius tight, preserves the green Phase 75 test suite). Log it as a tech-debt removal candidate for a future cleanup pass. Researcher/planner must explicitly acknowledge editor.rs is intentionally unused so a reviewer doesn't "fix" the phase by wiring it in.

### Cancel / empty-message repo state (D-02)

**D-02: On null (cancel/empty/whitespace), leave the repo in its recoverable in-progress state for ALL cases, including clean (conflict-free) merges and reverts.** Do not auto-abort.
- merge --continue: stays mid-merge (conflicts already resolved on disk).
- clean merge <branch>: stays with `MERGE_HEAD` set + changes staged.
- revert: stays with `REVERT_HEAD` set + changes staged.
- Recovery is via the existing `OperationBanner` abort affordances (merge_abort / a revert-abort path) — the user is never trapped.
- **Rationale:** matches git CLI exactly (the milestone goal — "match git's terminal `$EDITOR` behavior"), and gives one uniform cancel code path instead of a special-case auto-abort branch (which couldn't apply to the conflict-resolved merge-continue case anyway).

### Modal titles & host (D-03)

**D-03: Per-operation modal titles via the Phase 75 `title` prop (D-03 in 75-CONTEXT):**
- merge --continue AND merge <branch> → `"Merge commit message"`
- revert → `"Revert commit message"`

**D-04: Single `MessageEditor` instance hosted in `RepoView.svelte`**, exposed to the 4 scattered trigger sites via a callback prop threaded down — mirroring the existing `onopenrebaseeditor` pattern (`RepoView.svelte:567,805,888`). Trigger sites that must route through the editor:
- `CommitGraph.svelte:544` (revert_commit context menu)
- `CommitGraph.svelte:592` and `BranchSidebar.svelte:397` (merge_branch — TWO sites, both must get the editor)
- `StagingPanel.svelte:593` and `OperationBanner.svelte:33` (merge_continue — TWO sites)

### Open questions for the researcher (mechanism is decided; these are sub-mechanics)

- **OQ-1 — Fast-forward detection (MSG criterion 5).** Must NOT use `--no-ff` (that would force a merge commit on a ff-able merge and wrongly trigger the editor). Decide the detection approach: e.g., try `git merge --ff-only <branch>` first (succeeds silently = ff done, no editor; fails cleanly = non-ff, then `git merge --no-commit <branch>` → editor), OR an ancestry check before merging. Pick the one that leaves the repo untouched when detection says "ff" and never half-starts a merge. Verify in a scratch repo.
- **OQ-2 — Default-surfacing flow / IPC shape.** merge --continue can read `.git/MERGE_MSG` from the *already* in-progress state (one read; `get_operation_state_inner` already reads MERGE_MSG at operation_state.rs:59 — extend or add a small command). merge <branch> and revert generate the default only AFTER running `--no-commit`, so they need a two-step "begin (run --no-commit, return default) → finish (commit with message) / abort (leave state)" command shape. Decide whether to add new commands (e.g. `merge_branch_begin`/`revert_begin` + reuse `merge_continue` for the commit step) or restructure the existing ones. Keep the existing `merge_continue(message: Option<String>)` signature — it already does `git commit -m` when message is `Some`.
- **OQ-3 — Confirm `git commit -m` clears `REVERT_HEAD`.** After `git revert --no-commit` then `git commit -m msg`, confirm `REVERT_HEAD` is cleared and the revert is fully committed (expected yes; verify). Same check for the merge case clearing `MERGE_HEAD`.
- **OQ-4 — Removing the bypasses.** Criterion requires: drop `GIT_EDITOR=true` at operation_state.rs:171 (merge_continue else-branch — when no message, the frontend should abort and never call commit, so the else-branch is removed); drop `--no-edit` + `GIT_EDITOR=true` at operation_state.rs:301,304 (merge_branch); drop `--no-edit` at commit_actions.rs:153 (revert). Ensure no remaining call path silently bypasses the editor.

### Folded Todos

- **`.planning/todos/pending/2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md`** — its `resolves_phase: 76` is accurate. Phase 75 built the infra; Phase 76 does the wiring this todo prescribes. **This phase closes the todo** (move to `.planning/todos/done/` at phase completion).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & requirements
- `.planning/ROADMAP.md` §"Phase 76: Wire MessageEditor into merge/continue, merge, and revert" — phase goal, 6 success criteria
- `.planning/REQUIREMENTS.md` — MSG-01 (merge --continue), MSG-02 (merge <branch>), MSG-03 (revert), MSG-06 (empty → abort, recoverable); v0.14 Out of Scope table (ff-merge editing, cherry-pick, hooks UI, `--no-verify`, rich preview, draft persistence — all excluded)

### Phase 75 infrastructure (the primitives this phase wires)
- `.planning/phases/75-message-editor-infrastructure/75-CONTEXT.md` — D-01..D-12 (modal API, helper scope); D-04 (uniform null abort)
- `.planning/phases/75-message-editor-infrastructure/75-01-SUMMARY.md` — frozen `MessageEditor.open(default) → Promise<string|null>` contract; `title` prop; `bind:this={ref}` host usage; the worked Phase 76 wiring snippet
- `.planning/phases/75-message-editor-infrastructure/75-02-SUMMARY.md` — `editor.rs` API (intentionally UNUSED per D-01); documents the `tempfile` runtime-dep promotion that also becomes removable tech debt
- `src/components/MessageEditor.svelte` — the modal (CSS custom properties only, no inline colors)

### Production code paths this phase modifies (the bypasses to remove)
- `src-tauri/src/commands/operation_state.rs:150-181` — `merge_continue_inner`; line 158 has the working `git commit -m` path; line 171 sets the `GIT_EDITOR=true` to remove
- `src-tauri/src/commands/operation_state.rs:292-318` — `merge_branch_inner`; lines 301,304 use `--no-edit` + `GIT_EDITOR=true` to remove; note the existing conflict-detection branch at 309-313
- `src-tauri/src/commands/operation_state.rs:22-32` — `extract_merge_source` (parses MERGE_MSG; reference for branch-name handling)
- `src-tauri/src/commands/commit_actions.rs:143-171` — `revert_commit_inner`; line 153 uses `--no-edit` to remove; note existing `conflict_state` error code at 161-165
- Frontend trigger sites (D-04): `src/components/CommitGraph.svelte:544,592`, `src/components/BranchSidebar.svelte:397`, `src/components/StagingPanel.svelte:593`, `src/components/OperationBanner.svelte:33`
- `src/components/RepoView.svelte:567,805,888` — `onopenrebaseeditor` host+callback-thread pattern to mirror for the MessageEditor host

### Related todo
- `.planning/todos/pending/2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` — folded; this phase closes it

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`MessageEditor.svelte` (Phase 75)** — the modal; consumed via `bind:this`, `await ref.open(default)`, single `if (result === null) abort()` check covers all cancel paths.
- **`merge_continue_inner` `git commit -m` path (operation_state.rs:158-165)** — already finalizes a merge commit from a custom message; the direct approach reuses it for merge --continue and is the template for merge <branch>/revert finish-steps.
- **`onopenrebaseeditor` callback-thread (RepoView → BranchSidebar/CommitGraph)** — exact host+prop pattern for wiring the scattered trigger sites to a single RepoView-owned modal.
- **`OperationBanner.svelte` + `merge_abort`** — existing recoverable-abort UX; D-02's "leave recoverable" relies on it. Check whether a revert-abort affordance exists or needs adding (a `git revert --abort` command may be missing — researcher to confirm).

### Established Patterns
- **`*_inner` fn + `#[tauri::command]` async wrapper + `spawn_blocking` + `cache.insert` + `app.emit("repo-changed")`** — every op in operation_state.rs follows this; new begin/finish commands must too.
- **Subprocess git for editor-touching ops** (CLAUDE.md exception): merge/rebase/revert/cherry-pick already shell out to `git` with `.env("PATH", shell_env::system_path())`. The `--no-commit` + `git commit -m` steps stay on this sanctioned subprocess path. (Default-message READ from `.git/MERGE_MSG` is a plain file read, like operation_state.rs:59.)
- **`TrunkError` with a `code` string** (`merge_error`, `revert_error`, `conflict_state`) for typed frontend handling.
- **CSS custom properties only**; MessageEditor already complies.

### Integration Points
- **Two-step IPC for merge <branch> / revert** (OQ-2): begin (`--no-commit`, return default) → finish (`git commit -m`) / leave-on-cancel. merge --continue is one-step (read existing MERGE_MSG → editor → commit).
- **Graph refresh:** every finish/abort must rebuild the graph (`graph::walk_commits`) and emit `repo-changed`, matching the existing wrappers.

</code_context>

<specifics>
## Specific Ideas

- **Default message comes verbatim from `.git/MERGE_MSG`** — no frontend construction, no `#`-comment lines, no "cut here" marker. The textarea is byte-equal to git's own default (Phase 75 D-01/specifics).
- **The full OID appears in the revert default** (`This reverts commit <full-40-char-oid>.`) because that's what git writes — do not truncate to short OID.
- **Both merge_branch trigger sites and both merge_continue trigger sites** must route through the editor — no second silent path may remain.

</specifics>

<deferred>
## Deferred Ideas

- **Remove `editor.rs` + revert the `tempfile` runtime-dep promotion** — becomes dead code this phase (D-01a). Tech-debt cleanup candidate for a future milestone; not Phase 76 (keeps blast radius tight).
- **Consolidate `interactive_rebase.rs` queue script onto a shared helper** — already deferred in Phase 75 (75-CONTEXT deferred). If editor.rs is removed, this consolidation target disappears too.
- **`commit.template` / commit signing / hook-output streaming / `--no-verify`** — all deferred per REQUIREMENTS.md v2 + Out of Scope.

### Reviewed Todos (not folded)
None beyond the one folded above.

</deferred>

---

*Phase: 76-wire-messageeditor-into-merge-continue-merge-and-revert*
*Context gathered: 2026-05-28*
</content>
</invoke>
