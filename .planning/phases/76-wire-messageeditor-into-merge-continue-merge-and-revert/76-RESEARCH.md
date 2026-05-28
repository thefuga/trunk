# Phase 76: Wire MessageEditor into merge/continue, merge, and revert - Research

**Researched:** 2026-05-28
**Domain:** git plumbing behavior (subprocess) + Tauri command IPC shape + Svelte 5 modal wiring
**Confidence:** HIGH (all git-behavior claims re-verified in scratch repos with git 2.54.0; all code line citations re-read against current source)

## Summary

The implementation mechanism is locked (D-01: `--no-commit` → read `.git/MERGE_MSG` → `git commit -m`). Phase 75's `editor.rs` is intentionally dead and stays in the tree (D-01a). This research resolves the four open sub-mechanics empirically and surfaces **five scope items the CONTEXT under-specifies or gets wrong** that the planner MUST account for or requirements MSG-01/MSG-06 fail.

The headline empirical findings, all re-verified live with git 2.54.0:

1. **OQ-1: Use `git merge --ff-only <branch>` as the ff probe — NOT `--no-ff`, and NOT plain `--no-commit`.** A plain `git merge --no-commit <branch>` on a fast-forwardable merge **silently fast-forwards** (no `MERGE_HEAD`, no merge commit, 1 parent) — so it cannot be the editor path. `--ff-only` succeeds silently on ff and already-up-to-date (exit 0, no editor needed), and on a non-ff merge it **fails with exit 128 leaving `MERGE_HEAD` absent and the working tree clean** (never half-starts). After a failed `--ff-only`, the branches are provably divergent, so a subsequent `git merge --no-commit <branch>` is guaranteed to create a real merge — `--no-ff` is unnecessary.
2. **OQ-3: `git commit -m` clears both `MERGE_HEAD` and `REVERT_HEAD` and fully finalizes the operation.** Verified for clean merge, conflicted merge, and revert. Default messages match the spec verbatim including the full 40-char OID in the revert message and the `Merge remote-tracking branch 'origin/<branch>'` variant.
3. **NEW — conflicted MERGE_MSG carries a `# Conflicts:` comment block, and `git commit -m` does NOT strip it (default cleanup=`whitespace`).** Git's terminal `$EDITOR` uses cleanup=`strip` which removes `#` lines. To match git's behavior (the milestone goal), the finish-commit must pass `--cleanup=strip` OR the comment lines must be stripped before pre-fill. This is the MSG-01 path and is the most likely fidelity bug if missed.
4. **NEW — revert has no recovery path.** There is no `revert_abort` backend command, and `OperationBanner.svelte` renders action buttons only for `isRebase` (lines 135–180); a `Revert` state shows a bare label with zero buttons. Under D-02 (leave-recoverable on cancel), a user who cancels a revert is trapped in `REVERT_HEAD`. **This phase must add `revert_abort` (`git revert --abort`) + a banner affordance**, or MSG-06 is unsatisfiable for revert.
5. **NEW — `StagingPanel.svelte` already has an inline merge-commit editor and it is the ONLY live merge-continue editor.** State at 574–607 (`mergeSubject`/`mergeBody` + a pre-fill `$effect` that hardcodes `Merge branch '<src>' into <tgt>`); **full UI at 1302–1360** (`{:else if isMerge}`: subject input + body textarea + `commitMerge` button gated `disabled={!allResolved}`). Routing merge --continue through `MessageEditor` means **replacing** this inline UI, not adding a modal beside it.
6. **NEW — `OperationBanner.svelte`'s `merge_continue` branch (:33) is DEAD for merge — the CONTEXT's "two merge_continue trigger sites" is wrong.** For a Merge state the banner renders only a label (its entire Continue/Skip/Abort block is `{#if isRebase}`, lines 135–180); merge-continue's live UI is StagingPanel's inline form (:1302–1360). So there is **ONE live merge-continue site (StagingPanel)**; `OperationBanner:33`'s `isMerge ? "merge_continue"` is vestigial and must NOT be wired as a real editor path. (For revert, the banner likewise shows only a label with zero buttons — see finding 4.)
7. **NEW — the two-step `begin` mutates the repo, so the wrapper MUST emit `repo-changed` on `begin` itself, not only on finish/abort.** `--no-commit` sets `MERGE_HEAD`/`REVERT_HEAD` and stages changes *before* the editor opens. If `begin` returns silently and the user cancels (D-02 = no commit call), no event fires → the UI never re-fetches operation state → the OperationBanner / StagingPanel merge form never appears → the user is trapped (same failure as the revert gap). **Every `begin` outcome (ff / conflicts / clean) must rebuild the graph (`graph::walk_commits`) and `app.emit("repo-changed", path)`** so the in-progress UI renders behind/after the modal regardless of save-or-cancel.

**Primary recommendation:** Add `revert_commit_begin` + `revert_continue` + `revert_abort` and `merge_branch_begin` (returns a tagged enum: ff-done / conflicts / clean+default-message — **all variants carry a rebuilt graph; the async wrapper emits `repo-changed` for every variant**) to the backend; pass `--cleanup=strip` on every finish `git commit -m`; host one `MessageEditor` in `RepoView.svelte` threaded to the trigger sites via callback props; **replace** the StagingPanel inline merge form with a modal-routed flow; add Revert Continue/Abort affordances to OperationBanner.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| ff vs non-ff detection | API / Backend (Rust subprocess) | — | git decides; frontend never inspects refs |
| Default-message generation | API / Backend (git writes `.git/MERGE_MSG`) | — | MSG-04: never constructed in frontend |
| `.git/MERGE_MSG` read | API / Backend (plain file read) | — | matches existing `operation_state.rs:59` |
| Message editing UI | Browser / Client (Svelte modal) | — | Phase 75 `MessageEditor` owns this |
| Finalize commit / clear HEAD-state files | API / Backend (`git commit -m`) | — | subprocess path, CLAUDE.md sanctioned exception |
| **Surface in-progress state after `begin`** | **API / Backend (emit `repo-changed` on begin)** | Browser (re-fetch op state → banner/form) | begin mutates repo before editor opens; UI must learn even if user then cancels |
| Cancel/abort recovery | API / Backend (`git merge/revert --abort`) | Browser (OperationBanner) | recovery is a git op; banner is the affordance |
| Cancel = leave-recoverable (D-02) | Browser (return early, no commit call) | API (already emitted on begin) | no finish round-trip; the begin emit already made state visible |

## Standard Stack

No new packages. This phase wires existing primitives only.

### Core (already present)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git (CLI subprocess) | 2.54.0 (verified on this machine) | merge/revert ops touching `$EDITOR` | CLAUDE.md sanctioned exception; existing ops already shell out |
| git2 (libgit2) | 0.19 | repo open + `walk_commits` graph rebuild | every `*_inner` already uses it |
| tauri | 2 | `#[tauri::command]` IPC | existing pattern |
| Svelte 5 | runes | `MessageEditor` modal (frozen Phase 75) | existing |
| tempfile | 3 (runtime dep, promoted Phase 75) | only used by the now-dead `editor.rs` | becomes removable tech debt (D-01a) — DO NOT touch this phase |

**Installation:** none.

## Package Legitimacy Audit

Not applicable — this phase installs no external packages. No registry verification required.

## editor.rs is intentionally dead (D-01a) — DO NOT wire it in

`src-tauri/src/git/editor.rs` (`prepare()` + `EditorHandle`, built in Phase 75-02) is **deliberately unused** under D-01. The direct `git commit -m` approach makes the GIT_EDITOR-script helper pure indirection. **A reviewer must NOT "fix" this phase by routing through `editor.rs`.** Do not delete it either (keeps blast radius tight; preserves Phase 75's 8 green editor tests). Its `tempfile` runtime-dep promotion also stays. Both are logged as future-cleanup tech debt in 76-CONTEXT Deferred Ideas.

## Empirical git-behavior findings (verified in scratch repos, git 2.54.0)

All commands run with `mktemp -d` + `git init`. Author/committer env set. Raw outputs observed, not asserted from memory.

### OQ-1 — Fast-forward detection

| Scenario | Command | Result | `MERGE_HEAD` after | HEAD parents |
|----------|---------|--------|--------------------|--------------|
| ff-able merge, **plain `--no-commit`** | `git merge --no-commit feature` | `Updating..Fast-forward`, exit 0 | **absent** | 1 (no merge commit) |
| ff-able merge, `--ff-only` | `git merge --ff-only feature` | `Updating..Fast-forward`, exit 0 | absent | 1 |
| already-up-to-date, `--ff-only` | `git merge --ff-only other` | `Already up to date.`, exit 0 | absent | — |
| **non-ff merge, `--ff-only`** | `git merge --ff-only feature` | `fatal: Not possible to fast-forward, aborting.`, **exit 128** | **absent** | unchanged (HEAD untouched, status clean) |
| non-ff merge, plain `--no-commit` (after `--ff-only` failed) | `git merge --no-commit feature` | enters merge | **present** | (becomes 2 on commit) |

**Decision: probe with `git merge --ff-only <branch>` first.**
- exit 0 → fast-forward already happened (or up-to-date). **Skip the editor.** Rebuild graph, emit `repo-changed`, done. (Out-of-scope per REQUIREMENTS.md: ff merges create no commit → no message.)
- exit ≠ 0 with stderr containing `fast-forward` / `Not possible to fast-forward` → non-ff. Branches are now provably divergent and `MERGE_HEAD` is absent (clean) → run `git merge --no-commit <branch>`, then open editor. **`--no-ff` is unnecessary** because divergence is already established.

**Why not `--no-ff`:** it would force a merge commit on an ff-able merge — wrongly triggering the editor (CONTEXT explicitly forbids this; matches REQUIREMENTS.md Out-of-Scope row).
**Why not ancestry check (`merge-base --is-ancestor`):** the already-up-to-date case (`other` identical to `main`) would still report "ancestor" and could send you into a no-op `--no-commit` state transition; `--ff-only` handles it as a clean exit-0 no-op. The probe is also more faithful to what git itself decides (e.g. respects `merge.ff` config). `--ff-only` proven to never half-start (MERGE_HEAD absent after the exit-128 failure).

**Conflict branch (non-ff WITH conflicts):** when `git merge --no-commit <branch>` hits a conflict, git stops with `MERGE_HEAD` set and exit 1 (stderr contains `conflict`). There is **no message to edit yet** — the merge isn't ready to commit. This must NOT open the editor on `begin`. The flow defers to the existing merge-continue UI (StagingPanel conflict view → resolve → finish reads MERGE_MSG). See OQ-2 return shape.

### OQ-3 — `git commit -m` clears HEAD-state files + default messages

| Operation | After `--no-commit` | Default `.git/MERGE_MSG` (verbatim) | After `git commit -m` |
|-----------|---------------------|--------------------------------------|------------------------|
| merge onto `main` (default branch) | `MERGE_HEAD` set | `Merge branch 'feature'` | `MERGE_HEAD` removed, 2 parents, clean |
| merge onto `devel` (non-default) | `MERGE_HEAD` set | `Merge branch 'feature' into devel` | (same) |
| merge `origin/feature` (remote-tracking) | `MERGE_HEAD` set | `Merge remote-tracking branch 'origin/feature'` | (same) |
| conflicted merge (MSG-01 path) | `MERGE_HEAD` set | `Merge branch 'feature'\n\n# Conflicts:\n#\tf.txt\n` | `MERGE_HEAD` removed, clean |
| revert | `REVERT_HEAD` set | `Revert "Important change to x"\n\nThis reverts commit 1009ba34…b50941.\n` (full 40-char OID) | `REVERT_HEAD` removed, clean |
| conflicted revert | `REVERT_HEAD` set | `Revert "change to v2"\n\nThis reverts commit b497fa32….\n\n# Conflicts:\n#\tf\n` | (clears on commit) |

**The `into <branch>` suffix is current-branch-dependent.** git appends `into <name>` only when the current branch is NOT `main`/`master`. This is exactly why reading `MERGE_MSG` verbatim beats constructing a string. Note: `StagingPanel.svelte:582` currently hardcodes `into <tgt>` unconditionally — the new MERGE_MSG-sourced flow fixes that divergence.

### NEW critical finding — `# Conflicts:` comment lines survive `git commit -m`

Verified: `git commit -m "$MSG"` uses cleanup mode `whitespace` by default, which **keeps `#` lines**. The conflicted-merge / conflicted-revert MERGE_MSG (the MSG-01 path) contains:
```
Merge branch 'feature'

# Conflicts:
#	f.txt
```
Committing that verbatim lands the `# Conflicts:` lines **in the commit body** — git's terminal `$EDITOR` strips them (cleanup=`strip`). Confirmed `git commit --cleanup=strip -m "$MSG"` removes the `#` lines and trailing blanks; on a clean (no-`#`) message it is a safe no-op.

**Recommendation:** pass `--cleanup=strip` on every finish `git commit -m` in this phase. Caveat to flag for the planner: cleanup=`strip` removes **any** line beginning with `#`, including a legitimate user body line like `#42`. Git's terminal editor has the same behavior by default (comment char is `#`), so `--cleanup=strip` is the faithful match. Alternative (lower fidelity, more code): strip only the trailing `# Conflicts:` block before pre-fill and keep cleanup=`whitespace`. Recommend `--cleanup=strip` for parity with git CLI. **[VERIFIED: scratch repo, git 2.54.0]**

### OQ-4 — Removing the bypasses (exact current lines, re-read)

Line citations from CONTEXT verified against current source (they match):

| Bypass | File:line (current) | Current code | Action |
|--------|---------------------|--------------|--------|
| `GIT_EDITOR=true` in merge_continue else-branch | `operation_state.rs:166-174` (the `else` of `if let Some(msg)`; `GIT_EDITOR` at :171) | runs `git merge --continue` with `GIT_EDITOR=true` when `message` is `None` | **Remove the else-branch.** Under the new flow, merge --continue always passes a message (frontend aborts on null and never calls commit). The `Some(msg)` arm at :158-165 (`git commit -m`) is the only path kept — add `--cleanup=strip`. |
| `--no-edit` + `GIT_EDITOR=true` in merge_branch | `operation_state.rs:300-305` (`--no-edit` at :301, `GIT_EDITOR` at :304) | `git merge <branch> --no-edit` with `GIT_EDITOR=true` | **Restructure into `merge_branch_begin`** (probe ff → no-commit → return tagged result). The conflict-detection branch at :307-315 stays (conflicts → rebuild graph, no error). |
| `--no-edit` in revert | `commit_actions.rs:152-157` (`--no-edit` at :153) | `git revert <oid> --no-edit` | **Restructure into `revert_commit_begin`** (`git revert --no-commit <oid>` → return default MERGE_MSG). The `conflict_state` error code at :159-166 stays. |

**No remaining silent bypass check:** after removal, confirm grep finds no `GIT_EDITOR` / `--no-edit` in `operation_state.rs` or `commit_actions.rs` merge/revert paths. (Note `interactive_rebase.rs` legitimately keeps its `GIT_EDITOR=<script>` queue — out of scope, do not touch; v0.8 cherry-pick/rebase reword.)

## Architecture Patterns

### System Architecture Diagram

```
                         FRONTEND (Svelte 5)
  ┌───────────────────────────────────────────────────────────────┐
  │ Trigger sites (live):                                          │
  │  CommitGraph:541 handleRevert     → onopenmessageeditor        │
  │  CommitGraph:590 handleMergeBranch→ onopenmessageeditor        │
  │  BranchSidebar:397 handleMergeBranch → onopenmessageeditor     │
  │  StagingPanel:1302-1360 inline merge form → REPLACE w/ modal   │
  │  (OperationBanner:33 merge branch = DEAD for merge — ignore)   │
  │            │ callback prop (mirror onopenrebaseeditor)         │
  │            ▼                                                    │
  │  RepoView.svelte  ── hosts ──►  <MessageEditor bind:this>      │
  │     handleOpenMessageEditor(default, title) : Promise<str|null>│
  └────────────┬──────────────────────────────────────────────────┘
               │ invoke(...)                       ▲ default msg string
               ▼                                   │
                         BACKEND (Rust / Tauri)
  ┌───────────────────────────────────────────────────────────────┐
  │ MERGE --CONTINUE (1-step, already in-progress):                │
  │   get_merge_message(path) ── read .git/MERGE_MSG ──► default   │
  │   editor → merge_continue(path, Some(msg))  [git commit -m     │
  │            --cleanup=strip]  → clears MERGE_HEAD               │
  │                                                                │
  │ MERGE <branch> (2-step; wrapper EMITS repo-changed each begin):│
  │   merge_branch_begin(path,branch):                             │
  │     ├─ git merge --ff-only → exit0  ⇒ FastForwarded{graph}     │
  │     │       (NO editor; emit; done)                            │
  │     └─ fail ⇒ git merge --no-commit:                           │
  │            ├─ conflict ⇒ Conflicts{graph} (NO editor; emit;    │
  │            │             existing merge-continue UI takes over)│
  │            └─ clean   ⇒ Ready{graph?,message} (emit; editor)   │
  │   editor → merge_continue(path, Some(msg))  (reuses commit path)│
  │                                                                │
  │ REVERT <oid> (2-step; wrapper EMITS repo-changed on begin):    │
  │   revert_commit_begin(path,oid):                               │
  │     git revert --no-commit <oid>                               │
  │       ├─ conflict ⇒ Err(conflict_state)  (existing handling)   │
  │       └─ clean    ⇒ Ok{message}  (REVERT_HEAD set; emit)       │
  │   editor → revert_continue(path, msg)  [git commit -m          │
  │            --cleanup=strip] → clears REVERT_HEAD               │
  │                                                                │
  │ CANCEL (null) for all: frontend returns early, NO commit call. │
  │   Begin already emitted → banner/form visible → recovery via   │
  │   merge_abort / revert_abort (NEW) through OperationBanner.     │
  └───────────────────────────────────────────────────────────────┘
```

### Component Responsibilities
| File | Responsibility this phase |
|------|---------------------------|
| `operation_state.rs` | `get_merge_message` (read `.git/MERGE_MSG`, return string), `merge_branch_begin` (tagged enum; **async wrapper emits `repo-changed` for every variant**), keep `merge_continue` finish path (+`--cleanup=strip`; remove the `None` else-branch) |
| `commit_actions.rs` | `revert_commit_begin` (**emit on begin**), `revert_continue` (`git commit -m --cleanup=strip`), `revert_abort` (`git revert --abort`) — all NEW; register all in the tauri handler list |
| `RepoView.svelte` | host single `<MessageEditor bind:this>`, expose `handleOpenMessageEditor(default,title)`, thread `onopenmessageeditor` to children (mirror `onopenrebaseeditor` at :805/:888) |
| `CommitGraph.svelte` | route `handleRevert` (:541) + `handleMergeBranch` (:590) through callback |
| `BranchSidebar.svelte` | route `handleMergeBranch` (:397) through callback |
| `StagingPanel.svelte` | **replace inline merge form** (state 574–607 + UI 1302–1360 + `commitMerge`/`mergeSubject`/`mergeBody`); the merge-continue commit now routes through the modal (default from `get_merge_message`) |
| `OperationBanner.svelte` | **add Revert Continue + Abort buttons** (currently only `{#if isRebase}`); Continue routes through host modal, Abort calls new `revert_abort`. (Do NOT wire the dead `merge_continue` :33 branch — merge-continue lives in StagingPanel.) |

### Pattern: existing `*_inner` + `#[tauri::command]` wrapper
Every op in `operation_state.rs` / `commit_actions.rs` follows: `fn xxx_inner(path, …, state_map) -> Result<GraphResult, TrunkError>` (subprocess + `git2::Repository::open` + `graph::walk_commits`), wrapped by an `async #[tauri::command]` that does `spawn_blocking` → `cache.insert` → `app.emit("repo-changed", path)`. New begin/continue/abort commands MUST follow this shape — **including the emit on `begin`** (so the in-progress UI surfaces before the user decides to save or cancel).
```rust
// Source: src-tauri/src/commands/operation_state.rs:150-181, 389-408 (current)
let output = std::process::Command::new("git")
    .args(["commit", "-m", msg, "--cleanup=strip"])   // add --cleanup=strip
    .current_dir(path_buf)
    .env("PATH", shell_env::system_path())
    .output()?;
```

### Pattern: callback-thread host (mirror `onopenrebaseeditor`)
`RepoView.svelte:805` passes `onopenrebaseeditor={handleOpenRebaseEditor}` to `BranchSidebar`; `:888` to `CommitGraph`. `handleOpenRebaseEditor` (:567) is the host-owned handler. Mirror this exactly: add `onopenmessageeditor` prop threaded to the trigger components, with `RepoView` owning the single `<MessageEditor bind:this={ref}>` and the `await ref.open(default)` call.
```svelte
<!-- Source: 75-01-SUMMARY.md Next Phase Readiness snippet -->
<MessageEditor bind:this={messageEditorRef} title={editorTitle} />
const message = await messageEditorRef.open(defaultFromBackend);
if (message === null) return;  // D-02: leave recoverable; begin already emitted state
await invoke("merge_continue", { path, message });
```
Note: `MessageEditor` takes `title` as a `$props()` (not per-`open()` arg, verified `MessageEditor.svelte:1-6`). Title must be set before `open()`. Single-instance + reactive-title (`$state` var the host flips per operation: `"Merge commit message"` vs `"Revert commit message"`) matches D-03/D-04.

### Anti-Patterns to Avoid
- **`begin` that doesn't emit `repo-changed`.** Repo is already mutated; without the emit the UI can't surface the in-progress state and the cancel path traps the user.
- **Opening the editor on a conflicted `begin`.** No message exists yet; the merge/revert isn't ready to commit. Return a Conflicts/Err variant instead.
- **Using `--no-ff` to "force" the editor.** Wrongly creates a merge commit on ff-able merges. Forbidden by CONTEXT + REQUIREMENTS Out-of-Scope.
- **Reusing `merge_continue` to finish a revert.** Semantically wrong; add a symmetric `revert_continue` (matches the `merge_continue`/`rebase_continue` family).
- **Wiring `OperationBanner:33`'s merge branch as a real editor path.** It's dead for merge; the live UI is StagingPanel.
- **Leaving the StagingPanel inline merge form alongside the modal** — two competing message editors.
- **Wiring `editor.rs` back in.** Intentionally dead (D-01a).
- **Omitting `--cleanup=strip`** — leaves `# Conflicts:` lines in conflicted-merge commit bodies (MSG-01 fidelity bug).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Default merge/revert message | A frontend-constructed string | Read `.git/MERGE_MSG` written by git | MSG-04; git handles the `into <branch>` / remote-tracking / OID variants correctly; construction drifts |
| ff detection | An ancestry walk in Rust | `git merge --ff-only` probe | git respects `merge.ff` config + already-up-to-date; proven to never half-start |
| Stripping conflict comments | Custom `#`-line regex | `git commit --cleanup=strip` | git's own cleanup mode; exact terminal-editor parity |
| Revert recovery | Custom index reset | `git revert --abort` | exists, verified clean recovery |

**Key insight:** git already produces every default and handles every variant correctly when you let `--no-commit` write `MERGE_MSG`. The phase's whole value is *not* reconstructing what git already wrote.

## Runtime State Inventory

Not a rename/refactor/migration phase — this wires UI to existing ops. No stored data, live-service config, OS-registered state, secrets, or build artifacts carry phase-specific strings. **None — verified: this is additive command + UI wiring.**

## Common Pitfalls

### Pitfall 1: `# Conflicts:` lines in conflicted-merge commits
**What goes wrong:** the committed merge message contains git's `# Conflicts:` comment block.
**Why:** `git commit -m` defaults to cleanup=`whitespace` (keeps `#` lines); the terminal editor uses cleanup=`strip`.
**How to avoid:** `--cleanup=strip` on every finish commit. **Warning sign:** a conflicted-merge test asserts a clean body but the actual body has `# Conflicts:`.

### Pitfall 2: revert cancel traps the user
**What goes wrong:** user cancels the revert editor (D-02 leaves `REVERT_HEAD`), but there's no UI to abort or retry.
**Why:** no `revert_abort` command; OperationBanner shows buttons only for rebase.
**How to avoid:** add `revert_abort` + banner Continue/Abort for Revert state. **Warning sign:** MSG-06 acceptance "user can retry or abort" has no clickable path for revert.

### Pitfall 3: plain `--no-commit` swallows ff merges
**What goes wrong:** using `git merge --no-commit <branch>` as the universal entry fast-forwards ff-able merges silently — the editor never opens AND no merge commit is made (correct for ff), but you can't tell ff from non-ff without the probe.
**How to avoid:** probe `--ff-only` first; only run `--no-commit` after it fails.

### Pitfall 4: begin mutates the repo but never tells the UI
**What goes wrong:** `merge_branch_begin` / `revert_commit_begin` runs `--no-commit` (sets `MERGE_HEAD`/`REVERT_HEAD`, stages changes), returns the default message, the editor opens, the user cancels → no `repo-changed` ever fired → the in-progress banner/form never renders → the user can't see or recover the half-done state.
**Why:** the begin command returned silently (only finish/abort emitted in the old single-step flow).
**How to avoid:** the begin command's async wrapper MUST rebuild the graph and `app.emit("repo-changed", path)` for **every** outcome (ff/conflict/clean). **Warning sign:** a cancelled clean-merge or clean-revert leaves the UI showing the normal graph with no operation banner.

### Pitfall 5: wiring the dead OperationBanner merge path
**What goes wrong:** treating `OperationBanner:33` (`isMerge ? "merge_continue"`) as a merge-continue trigger to route through the modal — but that branch never renders a button for merge (the action block is `{#if isRebase}`), so the wiring is invisible/untestable.
**How to avoid:** the live merge-continue editor is StagingPanel (:1302–1360); wire that. Leave OperationBanner's merge branch alone (or remove it as dead code if the planner scopes a cleanup).

## Code Examples

### Tagged begin result (recommended Rust enum)
```rust
// merge_branch_begin return type — distinguishes the three outcomes.
// All variants carry the rebuilt graph so the async wrapper can cache+emit
// uniformly; the wrapper emits repo-changed for EVERY variant.
#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MergeBeginResult {
    FastForwarded { graph: GraphResult },   // ff happened, no editor
    Conflicts { graph: GraphResult },       // merge --continue UI takes over
    Ready { graph: GraphResult, message: String }, // open editor with `message`
}
```
(Frontend: `if kind === "ready"` → open editor; `"fast_forwarded"`/`"conflicts"` → no editor. The `repo-changed` emit drives the operation-state re-fetch in all three.)

### merge --continue default read (extend existing pattern)
```rust
// Source: existing operation_state.rs:59 — same read, exposed as a command
let merge_msg = std::fs::read_to_string(repo.path().join("MERGE_MSG")).ok();
// return merge_msg (frontend pre-fills editor) — strip nothing here;
// cleanup happens at commit via --cleanup=strip
```

## State of the Art

| Old Approach (current code) | New Approach (this phase) | Impact |
|-----------------------------|---------------------------|--------|
| `GIT_EDITOR=true` swallows merge/revert message | editor pre-filled from `MERGE_MSG`, `git commit -m --cleanup=strip` | user edits the real message (MSG-01/02/03) |
| `merge <branch> --no-edit` | `--ff-only` probe → `--no-commit` → editor | ff merges skip editor (criterion 5); non-ff get editor |
| revert has no cancel recovery | `revert_abort` + banner affordance | MSG-06 satisfiable for revert |
| StagingPanel inline form constructs `Merge … into <tgt>` | merge-continue routes through modal, default from `MERGE_MSG` verbatim | correct default-branch / remote-tracking variants |
| begin emits nothing | begin emits `repo-changed` so in-progress UI surfaces | cancel leaves a *visible*, recoverable state |

**Deprecated/outdated (do not act on this phase):** `editor.rs` + `tempfile` runtime dep — dead per D-01a, removal deferred. `OperationBanner:33` merge-continue branch — dead for merge, candidate for cleanup but out of strict scope.

## Validation Architecture

**Nyquist validation ENABLED** (`config.json` `workflow.nyquist_validation: true`).

### Test Framework
| Property | Value |
|----------|-------|
| Rust framework | built-in `#[cfg(test)]` + `cargo test --lib`; temp-repo pattern `make_repo()` exists at `src-tauri/src/git/review.rs:662` (`TempDir::new()` + `Repository::init`) — **no tests exist yet in `operation_state.rs` / `commit_actions.rs`** (Wave 0 gap) |
| Frontend framework | vitest + @testing-library/svelte + tauri-mock (`mockIPC`); all trigger components already have `*.test.ts` (RepoView, CommitGraph, StagingPanel, BranchSidebar, OperationBanner, MessageEditor) |
| Quick run (Rust) | `cd src-tauri && cargo test --lib operation_state commit_actions` |
| Quick run (frontend) | `npx vitest run src/components/RepoView.test.ts src/components/StagingPanel.test.ts` |
| Full suite | `just check` (fmt + biome + svelte-check + clippy + cargo-test + vitest) |

### Phase Requirements → Test Map
| Req | Behavior | Test type | Layer / command | Exists? |
|-----|----------|-----------|-----------------|---------|
| MSG-01 | merge --continue editor pre-filled from MERGE_MSG; commit uses it | Rust integration (temp repo): set up conflicted merge, resolve, call `merge_continue` w/ msg, assert HEAD body == msg, `--cleanup=strip` removes `# Conflicts:`, `MERGE_HEAD` gone | `cargo test --lib` | ❌ Wave 0 |
| MSG-01 | frontend opens editor with backend default | frontend vitest: mock `get_merge_message`→msg, assert `MessageEditor.open` called with it, assert `merge_continue` invoked w/ edited msg | vitest (StagingPanel.test.ts) | ❌ Wave 0 |
| MSG-02 | non-ff merge editor pre-filled `Merge branch '<b>'` (+ remote-tracking variant); ff merge skips editor | Rust integration: `merge_branch_begin` returns `Ready{message}` for non-ff, `FastForwarded` for ff (assert no MERGE_HEAD), message verbatim incl. `into <branch>` and `origin/` variants | `cargo test --lib` | ❌ Wave 0 |
| MSG-02 | ff probe never opens editor; begin emits repo-changed | frontend vitest: mock `merge_branch_begin`→`fast_forwarded`, assert `MessageEditor.open` NOT called; mock `ready`, assert editor opens | vitest | ❌ Wave 0 |
| MSG-03 | revert editor pre-filled `Revert "<subj>"` + full OID; commit uses it | Rust integration: `revert_commit_begin` returns message with full 40-char OID; `revert_continue` clears `REVERT_HEAD`, body matches | `cargo test --lib` | ❌ Wave 0 |
| MSG-06 | empty/whitespace → null → no commit, repo stays recoverable AND visible | frontend vitest: `MessageEditor` resolves null (Phase 75 covers resolve); assert no `*_continue` invoke fired; Rust: assert begin leaves MERGE_HEAD/REVERT_HEAD when no finish called | vitest + Rust | partial (Phase 75 covers null resolve) |
| MSG-06 | begin emits repo-changed so cancel state is visible | frontend vitest: spy on tauri `emit`/event listener — after `merge_branch_begin`/`revert_commit_begin`, operation-state re-fetch fires | vitest | ❌ Wave 0 |
| MSG-06 | revert recovery path exists | Rust: `revert_abort` clears `REVERT_HEAD` + clean tree; frontend: OperationBanner renders Abort for Revert state | `cargo test --lib` + vitest `OperationBanner.test.ts` | ❌ Wave 0 |

### Success-criterion → git-state coverage decisions
| Behavior | Covered by | Why |
|----------|------------|-----|
| `MERGE_HEAD`/`REVERT_HEAD` cleared by `git commit -m` | **Rust integration test** (temp repo) | deterministic, fast, no GUI; assert `repo.path().join("MERGE_HEAD")` absence |
| MERGE_MSG contents (clean / conflicted / remote-tracking / revert) | **Rust integration test** | byte-level assertion on `.git/MERGE_MSG` after begin |
| ff/non-ff boundary (no half-start) | **Rust integration test** | assert `MERGE_HEAD` absent after `--ff-only` failure; assert `FastForwarded` variant on ff |
| `--cleanup=strip` removes `# Conflicts:` | **Rust integration test** | assert committed body has no `#` lines |
| begin emits `repo-changed` | **frontend vitest** (event spy) | the emit is an IPC/UI concern; backend test only proves state mutation |
| modal open/null/edited flows | **frontend vitest** (mockIPC) | UI behavior; backend mocked |
| full end-to-end UX (real GUI merge/revert + cancel-then-recover) | **manual UAT** | final smoke; integration + vitest already prove state + emit, but the visible banner-after-cancel loop is best confirmed live |

### Sampling Rate
- **Per task commit:** `cargo test --lib operation_state commit_actions` (Rust) or targeted `vitest run <file>`.
- **Per wave merge:** `just check`.
- **Phase gate:** `just check` green before `/gsd:verify-work`.

### Wave 0 Gaps
- [ ] `src-tauri/src/commands/operation_state.rs` — add `#[cfg(test)]` module with temp-repo helper (mirror `git/review.rs:662` `make_repo`); covers MSG-01/02 git-state.
- [ ] `src-tauri/src/commands/commit_actions.rs` — add `#[cfg(test)]` module; covers MSG-03/06 revert git-state + `revert_abort`.
- [ ] Frontend: extend `RepoView.test.ts`, `StagingPanel.test.ts`, `OperationBanner.test.ts`, `CommitGraph.test.ts`, `BranchSidebar.test.ts` for editor-routing, null-abort, and begin-emit assertions.
- [ ] No new framework install needed.

## Security Domain

`security_enforcement` is absent in `config.json` (treated as enabled). This phase is a local desktop git GUI with **no new network, auth, or session surface** — it shells to local `git` (existing sanctioned path with `PATH` pinned via `shell_env::system_path()`).

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | edited commit message passed as a single `git commit -m <arg>` argv element (no shell interpolation — `std::process::Command` argv array, not a shell string), so no command injection via message content. Branch/oid args already validated by existing flows. |
| V6 Cryptography | no | — |

| Pattern | STRIDE | Mitigation |
|---------|--------|------------|
| Command injection via message text | Tampering | `Command::args([...])` argv array (no shell); already the existing pattern — keep it |
| `PATH` hijack of `git` subprocess | Elevation | existing `.env("PATH", shell_env::system_path())` — preserve on all new commands |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Phase 75 `MessageEditor` `title` is a `$props()` set before `open()`, not a per-call arg — host needs a reactive `title` var | Architecture Patterns | LOW — verified by reading `MessageEditor.svelte:1-6`; a per-op title needs the host to flip a `$state` before `await open()` |
| A2 | `--cleanup=strip` is the desired fidelity (matches git terminal editor) vs. stripping only the `# Conflicts:` block | NEW finding / Pitfall 1 | MEDIUM — strip removes ANY `#`-leading body line; if users intentionally use `#issue` in bodies this differs. Git CLI default has the same behavior, so this is parity. Planner should surface to user in discuss if unsure. |
| A3 | Single `MessageEditor` instance with reactive title (D-04) is preferred over two instances | Architecture Patterns | LOW — D-04 locks single instance; reactive title is the mechanism |
| A4 | `Ready` variant should carry the graph too (so the wrapper emits uniformly) | Code Examples enum | LOW — alternative is to emit before returning `Ready{message}` without a graph payload; either works as long as the emit happens |

## Open Questions

1. **Reactive title vs. two instances for the per-op title (D-03)?**
   - Known: `title` is a `$props()`. D-04 mandates a single instance.
   - Unclear: cleanest Svelte 5 way to set title before `open()` — bind `title={editorTitle}` to a `$state` the host sets, then `await ref.open(default)`.
   - Recommendation: single instance + `$state` title var; set it in `handleOpenMessageEditor(default, title)` before calling `open`.

2. **Where does the Revert OperationBanner Continue handler reach the host modal?** OperationBanner is rendered inside StagingPanel (:909), which is inside RepoView. The new Revert Continue/Abort buttons need: Abort → new `revert_abort` invoke (self-contained); Continue → must reach `RepoView`'s `handleOpenMessageEditor`. Recommendation: thread the `onopenmessageeditor` callback down through StagingPanel → OperationBanner, OR give OperationBanner an `oncontinuerevert` callback prop the parent wires. Confirm the prop-threading path during planning.

3. **Does the conflicted-merge path also need `--cleanup=strip` for MSG-01 specifically?** Yes (verified: conflicted MERGE_MSG has `# Conflicts:`). The merge-continue finish (`merge_continue` `Some(msg)` arm) is the MSG-01 commit — it must get `--cleanup=strip` just like the begin-flow finishes. Single shared treatment.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| git CLI | all three ops (subprocess) | ✓ | 2.54.0 | none (hard requirement; already assumed by existing ops) |
| git2 / libgit2 | graph rebuild | ✓ | 0.19 (Cargo) | — |
| bun / node_modules | vitest | ✓ (may need `bun install` in worktree) | — | — |

**Missing with no fallback:** none. **Missing with fallback:** none.

## Sources

### Primary (HIGH confidence)
- Scratch git repos, git 2.54.0 (this session) — all OQ-1/OQ-3 behaviors, conflicted MERGE_MSG, `--cleanup=strip`, remote-tracking variant, `git revert --abort`. Raw command outputs captured.
- `src-tauri/src/commands/operation_state.rs` (read in full) — line citations verified.
- `src-tauri/src/commands/commit_actions.rs` (read in full) — revert path verified.
- `src/components/MessageEditor.svelte`, `OperationBanner.svelte` (full), `StagingPanel.svelte` (574-627, 852-914, 1300-1360), `CommitGraph.svelte:525-621`, `RepoView.svelte:540-908` — read directly.
- `git/review.rs:662` — existing temp-repo test pattern.
- `.planning/config.json` — nyquist + workflow flags.
- 76-CONTEXT.md, REQUIREMENTS.md, 75-01/75-02 SUMMARY.md.

### Secondary (MEDIUM confidence)
- git `commit`/`merge` cleanup-mode semantics — cross-checked against observed `--cleanup=strip` vs default behavior in scratch repo (not from docs alone).

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- git behavior (OQ-1/3, conflicts, cleanup, abort): HIGH — re-verified live, raw outputs.
- IPC shape / removals (OQ-2/4): HIGH — line citations re-read against source.
- Frontend wiring (trigger sites, dead OperationBanner merge path, StagingPanel inline form, begin-emit gap): HIGH — all components read directly and the dead/live paths traced.
- The five NEW scope items: HIGH — empirically + code-verified.

**Research date:** 2026-05-28
**Valid until:** 2026-06-27 (stable; git CLI behavior + local code)
