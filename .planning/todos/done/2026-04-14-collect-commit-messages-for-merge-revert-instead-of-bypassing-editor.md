---
created: 2026-04-14T00:00:00.000Z
title: Collect commit messages for merge/revert instead of bypassing editor
area: commands
resolves_phase: 76
files:
  - src-tauri/src/commands/operation_state.rs
  - src-tauri/src/commands/commit_actions.rs
---

## Problem

Three git operations currently bypass the editor with `GIT_EDITOR=true` or `--no-edit` because trunk has no UI to collect a commit message from the user:

- `merge_continue_inner` (else branch, `operation_state.rs`) — `git merge --continue` with `GIT_EDITOR=true`
- `merge_branch_inner` (`operation_state.rs`) — `git merge <branch> --no-edit` with `GIT_EDITOR=true`
- `revert_commit_inner` (`commit_actions.rs`) — `git revert <oid> --no-edit`

This violates the project principle that trunk operations should behave like running the same operation in the terminal. In the shell, each of these opens `$EDITOR` on the prepared message (`MERGE_MSG`, default merge message, or `"Revert \"<subject>\""`). Trunk silently uses the default, giving the user no chance to edit or confirm.

The defensive overrides exist because without them, git tries to launch the user's editor inside a TTY-less GUI subprocess and fails.

## Solution

Collect the commit message in trunk's UI *before* invoking git. Then pass it via a temp editor script — the same pattern `interactive_rebase.rs:157-172` already uses for the in-app rebase editor.

Sketch:

1. **UI flow**: when the user clicks "Continue Merge" / "Merge Branch" / "Revert Commit", open a message editor modal pre-filled with the default message git would use:
   - Merge continue: read `.git/MERGE_MSG`
   - Merge branch: construct `"Merge branch '<branch>'"` (and `Merge remote-tracking branch ...` for remotes)
   - Revert: construct `"Revert \"<original subject>\""` plus `"This reverts commit <oid>."` body
2. **Rust side**: accept the user-edited message as a parameter to each Tauri command. Write it to a temp file. Write a shell script that `cp`s that file into `$1`. Run git with `GIT_EDITOR=<script>` (no `--no-edit`, no `GIT_EDITOR=true`).
3. **Remove** `GIT_EDITOR=true` from `merge_continue_inner` (else branch), `merge_branch_inner`, and the `--no-edit` flag from `merge_branch_inner` and `revert_commit_inner`.

Once this lands, trunk's behavior matches terminal: the user sees and can edit the prepared message every time, exactly like `$EDITOR` would let them.

## Why deferred

This is a UI change: new modal, wiring in `StagingPanel` / merge dropdown / commit context menu, plus message-default construction in Rust. It was out of scope for the env-drift fix that introduced this TODO.
