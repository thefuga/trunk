# Feature Landscape: Conflict Resolution & Interactive Rebase

**Domain:** Git GUI conflict resolution and history rewriting (GitKraken-parity)
**Researched:** 2026-03-20
**Target milestone:** v0.8

## Table Stakes

Features users expect from a Git GUI that claims to handle conflicts and rebase. Missing any of these makes the milestone feel incomplete.

### Conflict Resolution

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Conflicted file list in staging panel | Every Git GUI shows conflicted files distinctly from staged/unstaged. Users need to see what needs resolving at a glance. | Low | `WorkingTreeStatus.conflicted` field and `FileStatusType::Conflicted` already exist in the codebase. Need to render the conflicted list as a third section in StagingPanel with a distinct visual treatment (e.g., orange/yellow warning color). |
| Three-panel merge editor (current / incoming / output) | GitKraken, Sublime Merge, SmartGit, Tower all use this layout. It is the universal standard for visual conflict resolution. | High | Top-left shows "current" (ours) file version, top-right shows "incoming" (theirs) version, bottom shows the editable merged output. All three panels scroll in sync. |
| Per-hunk checkbox selection | GitKraken's core interaction: a checkbox next to each conflicting hunk lets the user add that entire chunk to the output with one click. | Med | Each conflict section (ours side and theirs side) gets a checkbox. Checking it adds those lines to the output panel. Unchecking removes them. |
| Per-line click selection | GitKraken allows clicking individual highlighted lines (not just whole hunks) to add them to the output. | Med | Click a line number or the line itself on either the current or incoming panel to toggle it into/out of the output. Finer granularity than per-hunk. |
| Editable output panel | The bottom output panel must be directly editable as a text editor. Users often need to manually tweak the merged result beyond what ours/theirs selection provides. | Med | Standard textarea/code editor in the output section. Users can type freely to adjust the merge result. |
| "Take All Current" / "Take All Incoming" buttons | Quick resolution for files where the user already knows which side wins entirely. GitKraken has this as right-click on conflicted file AND as buttons in the merge tool header. | Low | Two buttons in the merge tool toolbar. Also available as right-click options on conflicted files in the staging panel (resolve without opening the editor). |
| Conflict navigation arrows | GitKraken has arrow buttons (prev/next conflict) to jump between conflict sections within a file. Essential for large files with multiple conflicts. | Low | Up/Down arrow buttons in the merge tool toolbar. Jump to the next/previous conflict marker section within the current file. |
| "Save and Mark Resolved" button | After resolving, user clicks this single button to save the output, stage the file (mark resolved), and move to the next conflicted file. | Low | Single action: write merged content to disk, `git add` the file (removes it from conflict state), auto-open next conflicted file if any remain. |
| Conflict count badge / indicator | User needs to know how many files remain conflicted at all times. | Low | Badge on the conflicted section header in StagingPanel showing count. Update reactively as files are resolved. |
| Merge/rebase operation banner | When in a merge or rebase state, a persistent banner at the top of the app indicates the operation in progress with Continue/Abort buttons. | Med | Detect `.git/MERGE_HEAD` or `.git/rebase-merge/` or `.git/rebase-apply/` to show contextual banner. Buttons invoke `git merge --continue`, `git rebase --continue`, `git rebase --abort`, etc. |

### Interactive Rebase

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Rebase editor with commit list | The core interactive rebase UI: a modal/panel showing all commits that will be rebased, each with an action selector. Commits listed in application order (oldest first, matching `git rebase -i` todo format). | Med | Each row: commit SHA (short), commit message summary, action selector (dropdown or inline), drag handle. Default action is "Pick" for all commits. |
| Pick action | Keep the commit as-is. The default action on every commit when the rebase editor opens. | Low | No transformation needed. Commit is cherry-picked onto the new base. |
| Squash action | Combine the commit with its parent (the previous commit in the list). Merges commit messages. | Med | When set, the commit's changes fold into the previous commit. A message editor appears to let the user edit the combined commit message before starting. |
| Drop action | Remove the commit entirely from history. | Low | Commit is skipped during replay. Visual indicator (strikethrough or dimmed row) shows it will be removed. |
| Reword action | Edit the commit message without changing the commit's content. | Med | When the rebase executes and reaches a Reword commit, a dialog appears for the user to edit the commit summary and description. GitKraken opens a modal for this. |
| Drag-and-drop commit reordering | Reorder commits by dragging rows up/down in the rebase editor list. | Med | Standard drag-and-drop list reordering. Visual feedback during drag (insertion line indicator, row elevation). Updates the commit application order. |
| Keyboard shortcuts for actions | GitKraken: P=Pick, S=Squash, R=Reword, D=Drop. Quick action assignment without mouse. | Low | When a commit row is focused/selected, pressing the shortcut key changes its action. |
| "Start Rebase" / "Cancel" buttons | Confirmation before executing the rebase. Cancel discards the rebase plan. | Low | "Start Rebase" validates the plan (e.g., can't squash the first commit) and executes. "Cancel" closes the editor with no changes. |
| Reset button | Undo all changes in the rebase editor, returning all commits to their original "Pick" state and order. | Low | Single button resets the editor to its initial state without closing it. |

### Merge Workflow

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Merge via drag-and-drop on graph | GitKraken's primary merge initiation: drag source branch onto target branch, select "Merge" from context menu. | Med | Drag a branch ref pill (or branch row in sidebar) onto another branch. Show a context menu with "Merge [source] into [target]" option. |
| Merge via right-click context menu | Right-click a branch in the sidebar or graph, select "Merge [branch] into [current branch]". | Low | Add "Merge into current branch" to branch context menu. Only enabled when the branch is not the current branch. |
| Fast-forward merge handling | When merge is a fast-forward, just advance the branch pointer. No merge commit needed. | Low | Detect fast-forward possibility, perform it silently, refresh graph. Show toast: "Fast-forwarded [branch] to [target]". |
| Merge commit creation (no conflicts) | When merge has no conflicts, auto-create the merge commit with the standard merge message. | Low | Create merge commit with default message "Merge branch '[source]' into [target]". Refresh graph to show the new merge commit. |

### Rebase Workflow

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Rebase via drag-and-drop on graph | GitKraken's primary rebase initiation: drag source branch onto target, select "Rebase [source] onto [target]" from context menu. | Med | Same drag interaction as merge, but the context menu offers "Rebase" as an alternative to "Merge". |
| Rebase via right-click context menu | Right-click a branch, select "Rebase current branch onto [branch]". | Low | Context menu option on branches. |
| Interactive rebase from right-click on commit | Right-click a parent commit in the graph, select "Interactive Rebase" to rebase current branch onto that commit. | Low | Opens the rebase editor with commits from current HEAD down to (not including) the selected commit. |
| Mid-rebase conflict resolution | When a rebase hits a conflict, pause and show the merge tool for each conflicting file. After resolving all conflicts for the current commit, allow "Continue Rebase". | High | The rebase pauses at the conflicting commit. Conflicted files appear in the staging panel. User resolves each file using the merge tool. "Continue Rebase" button (in the operation banner) resumes. |
| Abort rebase | Cancel an in-progress rebase and restore the repository to its pre-rebase state. | Low | "Abort Rebase" button in the operation banner. Runs `git rebase --abort`. Refreshes graph. |
| Skip commit during rebase | Skip the current conflicting commit and continue with the next one. | Low | "Skip" button in the operation banner. Runs `git rebase --skip`. |

## Differentiators

Features that go beyond basic parity. Not strictly required but add significant value.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Fixup action in interactive rebase | Like Squash but discards the commit message. Standard git operation that GitKraken does NOT support (only Pick/Reword/Squash/Drop). Trunk can differentiate here. | Low | git2's `RebaseOperationType::Fixup` is fully supported. Add "Fixup" to the action selector with keyboard shortcut F. |
| Edit action in interactive rebase | Pause rebase at a specific commit to let user amend it. GitKraken does NOT support this. | Med | git2's `RebaseOperationType::Edit` is supported. When rebase reaches an Edit commit, it pauses. The operation banner shows "Amend this commit, then Continue Rebase". |
| Multi-commit selection for bulk actions | Select multiple commits in the rebase editor and set them all to the same action at once. | Low | Shift-click or Cmd-click to select multiple rows, then press a shortcut or use a dropdown to set all selected to the same action. |
| Base (ancestor) version toggle in merge editor | Show the common ancestor version alongside current/incoming. Helps understand what changed on each side. GitKraken mentions "3-way" but primarily shows current+incoming+output. SmartGit explicitly shows the base. | Med | Optional fourth panel or toggle that shows the ancestor version. Helps with complex conflicts where understanding the original is crucial. |
| Undo/redo for rebase operations | GitKraken 10.8+ added support for undoing interactive rebase. Trunk already has undo/redo infrastructure. | Med | Use existing undo/redo pattern. After a rebase completes, push the pre-rebase HEAD onto the undo stack so the user can `git reset --hard` back. |
| Conflict diff syntax highlighting | Syntax-highlight the code in the merge editor panels based on file extension. | Med | Reuse whatever syntax highlighting exists in the diff panel. Apply to all three merge editor panels. |
| "Rebase Last N Commits" shortcut | GitKraken 11.10 added this: shift-click a range of commits at the branch tip, then drag onto another branch to rebase just those commits. No editor, no list, no CLI. | Med | Select commit range in graph via shift-click, right-click or drag onto target branch, rebase only that range. Quick workflow for common case. |
| Squash message editor | When squash is selected, show a combined message editor before starting the rebase so the user can craft the final message. | Low | Modal with the concatenated messages from all squashed commits, editable before execution. |
| Cherry-pick multiple commits with rebase editor | GitKraken 10.8 added this: select multiple commits and cherry-pick them through the interactive rebase editor (reorder, squash, reword, drop). | Med | Reuse the rebase editor UI for cherry-pick operations on non-contiguous commits. |

## Anti-Features

Features to explicitly NOT build in v0.8.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| AI-powered conflict resolution | GitKraken's "Auto-resolve with AI" is a paid feature in Preview. Adds massive complexity (LLM integration, API keys, cost management) for questionable value. Not core UX. | Focus on making manual resolution fast and intuitive. Consider as a post-1.0 feature if there's demand. |
| Conflict prevention / team detection | GitKraken's conflict prevention requires organization/team infrastructure, cloud sync, and real-time monitoring of teammates' branches. Way beyond scope. | Out of scope entirely. This is a team collaboration feature, not a conflict resolution feature. |
| External merge tool integration | Spawning kdiff3, meld, Beyond Compare, etc. Adds configuration complexity and dependency on external tools. | Build the merge tool in-app. If users want external tools, they can use `git mergetool` in terminal. Consider as a future option. |
| `git rebase --exec` support | The Exec operation runs arbitrary shell commands during rebase. Niche use case, security concerns with arbitrary command execution. | Support Pick, Reword, Edit, Squash, Fixup, Drop. Exec is for CI/automation, not GUI users. |
| Rebase across remotes / force push flow | After rebase, the user needs to force push. Building a force push flow with safety checks adds scope. | Show a toast/warning after rebase: "Branch history has changed. You may need to force push." Let the user handle force push via toolbar or terminal. |
| In-memory rebase | git2 supports `inmemory` rebase mode. Adds complexity for marginal UX benefit. | Use standard on-disk rebase. Working directory changes are expected and visible. |

## Feature Dependencies

```
Conflicted file detection (staging panel) --> Merge editor (three-panel)
                                          --> Right-click "Take All" resolution

Operation state detection (.git/MERGE_HEAD, .git/rebase-*) --> Operation banner (Continue/Abort/Skip)

Merge via drag-drop / context menu --> Conflict detection --> Merge editor --> "Save and Mark Resolved" --> Merge commit

Rebase editor UI --> Start Rebase execution --> Mid-rebase conflict --> Merge editor --> Continue Rebase

Merge editor (three-panel) --> Per-hunk checkboxes --> Per-line selection --> Editable output
                           --> Conflict navigation arrows
                           --> "Save and Mark Resolved"

Drag-and-drop on graph --> Context menu with Merge/Rebase options
```

## MVP Recommendation

### Phase 1: Conflict Detection + Basic Resolution (foundation)

Prioritize:
1. **Conflicted file list in staging panel** -- renders the already-existing `conflicted` field from `WorkingTreeStatus`
2. **Operation state banner** -- detect merge/rebase in progress, show Continue/Abort buttons
3. **Right-click "Take All Current" / "Take All Incoming"** on conflicted files -- quickest path to functional conflict resolution without building the merge editor
4. **Merge via context menu** -- right-click branch, "Merge into current branch"
5. **Rebase via context menu** -- right-click branch, "Rebase onto"

Rationale: This gives users a complete merge/rebase workflow using quick resolution (take all ours/theirs). The three-panel merge editor is the biggest single component and can come in Phase 2.

### Phase 2: Three-Panel Merge Editor

Prioritize:
1. **Three-panel merge editor** with synced scrolling
2. **Per-hunk checkbox selection**
3. **Per-line click selection**
4. **Editable output panel**
5. **Conflict navigation arrows**
6. **"Save and Mark Resolved" button**
7. **"Take All" buttons in merge tool header**

Rationale: This is the highest-complexity, highest-value feature. It replaces the need for external merge tools.

### Phase 3: Interactive Rebase Editor

Prioritize:
1. **Rebase editor with commit list** (Pick/Squash/Drop/Reword actions)
2. **Drag-and-drop reordering**
3. **Keyboard shortcuts** (P/S/R/D)
4. **Start Rebase / Cancel / Reset buttons**
5. **Mid-rebase conflict resolution** (reuses merge editor from Phase 2)
6. **Fixup and Edit actions** (differentiators over GitKraken)
7. **Interactive rebase from right-click on commit**

Defer: Cherry-pick multiple commits via rebase editor, "Rebase Last N" shortcut, base ancestor toggle, undo/redo for rebase. These are differentiators that can ship in a later milestone.

## GitKraken UX Reference: Detailed Flows

### Merge Conflict Resolution Flow (Step-by-Step)

1. **Initiation**: User drags branch A onto branch B in graph (or right-clicks branch and selects "Merge"). A context menu appears: "Merge [A] into [B]" / "Rebase [A] onto [B]" / "Interactive Rebase".
2. **Conflict detection**: If merge produces conflicts, the commit panel (staging panel) shows conflicted files with a distinct warning icon. A banner appears at the top: "Merge in progress - resolve conflicts to continue."
3. **File selection**: User clicks a conflicted file in the staging panel to open the merge tool.
4. **Merge tool opens**: Three-panel layout appears:
   - **Top-left**: Current branch version (ours). Read-only. Conflicting hunks highlighted.
   - **Top-right**: Incoming branch version (theirs). Read-only. Conflicting hunks highlighted.
   - **Bottom**: Output panel (editable). Initially contains the conflicted file with markers OR empty sections for conflict regions.
5. **Hunk selection**: Each conflicting section on both left and right panels has a checkbox. Clicking the checkbox adds that entire hunk to the output. User can check hunks from both sides (selecting "both" for a conflict section).
6. **Line selection**: User can click individual highlighted lines to add just those lines to the output. Finer control than hunk-level.
7. **Direct editing**: User can type directly in the output panel to manually adjust the merged result.
8. **Navigation**: Arrow buttons in the toolbar jump between conflict sections within the file.
9. **"Take All" shortcut**: Button in toolbar to take all from current side or all from incoming side for the entire file.
10. **Save**: User clicks "Save and Mark Resolved". The file is saved to disk, staged (`git add`), and removed from the conflicted list. The next conflicted file auto-opens.
11. **Completion**: When all conflicted files are resolved, the user clicks "Continue Merge" (or "Commit") in the operation banner. The merge commit is created.

### Interactive Rebase Flow (Step-by-Step)

1. **Initiation**: User right-clicks a commit in the graph and selects "Interactive Rebase onto [commit]". OR drags branch onto target and selects "Interactive Rebase".
2. **Editor opens**: A modal/panel shows the list of commits that will be rebased. Each commit row shows:
   - Action selector (dropdown: Pick / Reword / Squash / Drop; with keyboard shortcuts P/R/S/D)
   - Short SHA
   - Commit message summary
   - Drag handle for reordering
3. **Default state**: All commits are set to "Pick".
4. **User configures**: User changes actions on individual commits (dropdown or keyboard shortcut). Reorders by dragging rows. Can select multiple and batch-change actions.
5. **Validation**: Can't squash the first commit (nothing to squash into). Visual warning if attempted.
6. **Start**: User clicks "Start Rebase". The editor closes.
7. **Execution**: Rebase begins applying commits in order.
   - For **Pick**: Cherry-pick and auto-commit.
   - For **Reword**: A dialog pops up for message editing, then commit.
   - For **Squash**: Changes fold into previous commit. If multiple squashes in sequence, a message editor appears with all combined messages.
   - For **Drop**: Commit is skipped.
   - For **Edit** (differentiator): Rebase pauses. Banner shows "Amend this commit, then Continue Rebase."
   - For **Fixup** (differentiator): Like squash but discards the commit message silently.
8. **Conflict during rebase**: If a commit conflicts:
   - Rebase pauses at the conflicting commit.
   - Operation banner: "Rebase in progress - resolve conflicts to continue. [Continue] [Skip] [Abort]"
   - Conflicted files appear in staging panel.
   - User resolves each file using the merge tool (same flow as merge conflicts).
   - User clicks "Continue Rebase" to proceed to the next commit.
9. **Abort**: At any point, "Abort Rebase" restores the repository to its pre-rebase state.
10. **Completion**: After all commits are processed, the rebase finishes. Graph refreshes showing the rewritten history. Toast: "Rebase complete."

### Merge Tool Component Anatomy

```
+------------------------------------------------------------------+
|  Merge Tool Toolbar                                               |
|  [<< Prev Conflict] [Next Conflict >>]  [Take All Current]       |
|  [Take All Incoming]                    [Save and Mark Resolved]  |
+-----------------------------------+------------------------------+
|  Current (Ours)                   |  Incoming (Theirs)            |
|                                   |                               |
|  [x] hunk 1 (lines 10-15)        |  [x] hunk 1 (lines 10-18)    |
|      context line                 |      context line             |
|  +   added line (clickable)       |  +   added line (clickable)   |
|  +   added line (clickable)       |  +   added line (clickable)   |
|      context line                 |      context line             |
|                                   |                               |
|  [ ] hunk 2 (lines 30-35)        |  [ ] hunk 2 (lines 30-33)    |
|      context line                 |      context line             |
|  -   removed line                 |  +   added line               |
|      context line                 |      context line             |
+-----------------------------------+------------------------------+
|  Output (Editable)                                                |
|                                                                   |
|      context line                                                 |
|      added line (from checked current hunk 1)                     |
|      added line (from checked current hunk 1)                     |
|      added line (from checked incoming hunk 1)                    |
|      added line (from checked incoming hunk 1)                    |
|      context line                                                 |
|                                                                   |
|      << unresolved conflict section (hunk 2) >>                   |
|                                                                   |
+------------------------------------------------------------------+
```

### Interactive Rebase Editor Anatomy

```
+------------------------------------------------------------------+
|  Interactive Rebase: [source-branch] onto [target-branch]         |
|                                                     [Reset]       |
+------------------------------------------------------------------+
|  Action    |  SHA      |  Message                    |  Drag     |
+------------------------------------------------------------------+
|  [Pick v]  |  a1b2c3d  |  Add user authentication    |  :::      |
|  [Squash v]|  e4f5g6h  |  Fix typo in auth module    |  :::      |
|  [Pick v]  |  i7j8k9l  |  Add password reset flow    |  :::      |
|  [Drop v]  |  m0n1o2p  |  WIP: debugging             |  :::      |
|  [Reword v]|  q3r4s5t  |  Update dependencies        |  :::      |
+------------------------------------------------------------------+
|  Keyboard: P=Pick  S=Squash  R=Reword  D=Drop  F=Fixup  E=Edit  |
+------------------------------------------------------------------+
|                              [Cancel]  [Start Rebase]             |
+------------------------------------------------------------------+
```

### Operation Banner Anatomy

```
+------------------------------------------------------------------+
|  MERGE IN PROGRESS                    [Continue]  [Abort]         |
+------------------------------------------------------------------+

+------------------------------------------------------------------+
|  REBASE IN PROGRESS (3/7 commits applied)                        |
|  Current commit: a1b2c3d "Add user auth"                         |
|                                       [Continue]  [Skip]  [Abort]|
+------------------------------------------------------------------+
```

## Implementation Notes (git2 / git CLI boundaries)

### What git2 can handle natively:
- Merge: `repo.merge()`, `index.conflicts()`, `index.conflict_get(path)`, `index.has_conflicts()`
- Conflict entries: `IndexConflict` provides ancestor/ours/theirs blob OIDs for each conflicted file
- Rebase: `repo.rebase()` with full `RebaseOperationType` support (Pick, Reword, Edit, Squash, Fixup, Exec)
- Rebase iteration: `Rebase` implements `Iterator<Item = Result<RebaseOperation>>` with `next()`, `commit()`, `abort()`, `finish()`

### What should use git CLI (subprocess):
- Mid-rebase continue/abort/skip: `git rebase --continue`, `git rebase --abort`, `git rebase --skip` (matches existing pattern of shelling out for complex state machine operations like cherry-pick/revert)
- Merge continue: `git merge --continue` or `git commit` after resolving all conflicts
- Interactive rebase: While git2 has the Rebase API, the interactive rebase flow is complex enough that using `git rebase -i` with a custom `GIT_SEQUENCE_EDITOR` script may be simpler and more reliable for the initial implementation

### Existing codebase hooks:
- `WorkingTreeStatus.conflicted: Vec<FileStatus>` already exists in types
- `FileStatusType::Conflicted` variant exists
- `classify_index()` already handles `Status::CONFLICTED`
- DiffPanel already has per-hunk navigation, hunk actions, line selection -- these patterns directly inform the merge editor design
- Context menu infrastructure exists for right-click actions on branches and commits
- Toast notification system exists for operation feedback
- Operation state (merge/rebase in progress) can be detected by checking for `.git/MERGE_HEAD`, `.git/rebase-merge/`, `.git/rebase-apply/` files

## Sources

- [GitKraken Merge Conflict Tool](https://www.gitkraken.com/features/merge-conflict-resolution-tool) -- MEDIUM confidence
- [GitKraken Branching and Merging docs](https://help.gitkraken.com/gitkraken-desktop/branching-and-merging/) -- HIGH confidence
- [GitKraken Interactive Rebase docs](https://help.gitkraken.com/gitkraken-desktop/interactive-rebase/) -- HIGH confidence
- [GitKraken Conflict Prevention docs](https://help.gitkraken.com/gitkraken-desktop/conflict-prevention/) -- MEDIUM confidence
- [GitKraken merge conflict blog post](https://www.gitkraken.com/blog/merge-conflict-tool) -- MEDIUM confidence
- [GitKraken merge conflict tutorial](https://www.gitkraken.com/learn/git/tutorials/how-to-resolve-merge-conflict-in-git) -- MEDIUM confidence
- [GitKraken rebase tutorial](https://www.gitkraken.com/learn/git/problems/git-rebase-branch) -- MEDIUM confidence
- [GitKraken interactive rebase tutorial](https://www.gitkraken.com/learn/git/problems/git-interactive-rebase) -- MEDIUM confidence
- [GitKraken Desktop 11.10 release](https://www.gitkraken.com/blog/gitkraken-desktop-11-10-from-top-requests-to-todays-release) -- MEDIUM confidence
- [GitKraken Desktop 10.x release notes](https://help.gitkraken.com/gitkraken-desktop/10x/) -- HIGH confidence
- [SmartGit conflict resolution](https://www.smartgit.dev/features/conflict-resolution/) -- MEDIUM confidence
- [git2 Index API (conflicts)](https://docs.rs/git2/latest/git2/struct.Index.html) -- HIGH confidence
- [git2 Rebase API](https://docs.rs/git2/latest/git2/struct.Rebase.html) -- HIGH confidence
- [git2 RebaseOperationType](https://docs.rs/git2/latest/git2/enum.RebaseOperationType.html) -- HIGH confidence
- [git2-rs rebase source](https://github.com/rust-lang/git2-rs/blob/master/src/rebase.rs) -- HIGH confidence
