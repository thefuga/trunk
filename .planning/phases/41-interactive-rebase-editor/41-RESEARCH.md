# Phase 41: Interactive Rebase Editor - Research

**Researched:** 2026-03-21
**Domain:** Git interactive rebase UI, drag-and-drop reordering, Tauri IPC for subprocess control
**Confidence:** HIGH

## Summary

Phase 41 implements a visual interactive rebase editor that replaces the center pane (same pattern as MergeEditor in Phase 38). The editor displays commits between a base commit and HEAD with action selectors (Pick/Squash/Reword/Drop), drag-and-drop reordering, keyboard shortcuts, validation, and execution with mid-rebase conflict handling via the existing merge editor.

The core technical challenge is programmatic control of `git rebase -i`. The approach uses `GIT_SEQUENCE_EDITOR` to write a custom todo file (bypassing the normal editor), and a custom `GIT_EDITOR` helper script for reword/squash message editing pauses. The frontend components reuse the established column system from CommitGraph (resizable headers, visibility toggle, LazyStore persistence) and the center-pane swap pattern from MergeEditor.

**Primary recommendation:** Use `GIT_SEQUENCE_EDITOR` with a shell script that writes the user's todo plan to the rebase todo file, and `GIT_EDITOR` set to a helper script that communicates commit messages via temp files and Tauri events. Execute the rebase as a single `git rebase -i` subprocess, handling pauses (reword/squash) through event-driven IPC between the Rust backend and Svelte frontend.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Editor container**: Replaces center pane (same pattern as MergeEditor). Sidebar and staging panel remain visible and interactive
- **Entry points**: Commit right-click ("Interactive Rebase") AND branch context menus (sidebar + graph pills). For branch menus, auto-detect fork point as base commit
- **Commit scope**: Right-click commit X -> rebase all commits from X (exclusive) up to HEAD. Standard `git rebase -i` behavior
- **Column layout**: Reuses CommitGraph column system -- resizable columns, right-click header to toggle visibility, LazyStore-persisted widths/visibility
- **Columns (left to right)**: Action | SHA | Message | Author | Date. All visible by default. Action always visible
- **Same row height** as CommitGraph rows -- compact single-line
- **Action column**: Dropdown select per row with Pick/Squash/Reword/Drop options
- **Keyboard shortcuts**: P=Pick, S=Squash, R=Reword, D=Drop on focused row
- **Drag-and-drop**: Entire row is draggable, grab cursor on hover, row swap animation
- **Drop visual**: Reduced opacity + strikethrough text (GitKraken approach)
- **Toolbar buttons**: Start Rebase, Cancel, Reset
- **Message editing during execution**: Messages written when git pauses at each reword/squash commit, not pre-configured
- **Squash pre-fill**: Concatenated messages of all squashed commits (git default)
- **Editor closes on Start**: Returns to graph view immediately. OperationBanner shows progress
- **Conflict handling**: Merge editor opens via existing Phase 37-38 infrastructure
- **Reword/squash pauses**: Message dialog pops over normal graph view (editor does NOT reopen)
- **No success toast**: Graph refresh shows rewritten history (consistent with Phases 39-40)
- **Error toast** for failures
- **Validation with inline errors**: Block Start Rebase for invalid states (can't squash first commit, can't drop all commits)

### Claude's Discretion
- Message editor implementation (InputDialog reuse vs inline approach)
- Exact validation rules beyond the two specified
- Loading state during rebase execution
- How to detect fork point for branch menu entry point
- Drag-and-drop implementation approach (native HTML5 vs library)
- Exact dropdown styling and color coding for actions
- How to handle detached HEAD (hide menu items, like merge/rebase pattern)

### Deferred Ideas (OUT OF SCOPE)
- IREB-EX-01: Fixup action (like Squash but discards commit message)
- IREB-EX-02: Edit action (pause rebase at commit to amend)
- IREB-EX-03: Multi-commit selection for bulk action assignment
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| REB-03 | User can start interactive rebase by right-clicking a commit in the graph and selecting "Interactive Rebase" | Context menu wiring pattern established in CommitGraph.svelte (showCommitContextMenu) and BranchSidebar.svelte; Tauri Menu API pattern verified |
| IREB-01 | Interactive rebase opens a panel showing all commits with action selectors (Pick/Squash/Reword/Drop) | Backend: `get_rebase_todo` command using git2 revwalk to list commits between base and HEAD; Frontend: RebaseEditor component with column layout |
| IREB-02 | User can reorder commits by dragging rows up/down | Native HTML5 drag-and-drop API sufficient for row reordering with swap animation; no library needed |
| IREB-03 | Keyboard shortcuts assign actions to focused commit rows (P/S/R/D) | Standard keydown handler on focused row elements; existing keyboard shortcut pattern in CommitGraph |
| IREB-04 | Start Rebase validates the plan and executes the rebase | Validation logic in frontend; execution via `start_interactive_rebase` Rust command using GIT_SEQUENCE_EDITOR |
| IREB-05 | Cancel closes with no changes; Reset restores original state | Pure frontend state management; Cancel unmounts editor, Reset resets $state arrays |
| IREB-06 | Reword action shows message editor dialog when rebase reaches that commit | GIT_EDITOR helper script + Tauri event IPC; InputDialog reuse for message editing |
| IREB-07 | Squash action combines commit with predecessor; message editor shows concatenated messages | Same GIT_EDITOR mechanism; git natively concatenates squash messages in the COMMIT_EDITMSG file |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte | 5.53.6 | UI framework (runes, $state, $derived) | Project standard |
| Tauri | 2.x | Desktop app framework, IPC, subprocess | Project standard |
| git2 | 0.19 | Git repository access (revwalk, commit lookup) | Project standard |
| @tauri-apps/api | 2.10.1 | Frontend IPC (invoke, listen, emit) | Project standard |
| @tauri-apps/api/menu | 2.10.1 | Native context menus | Project standard |
| @tauri-apps/plugin-store | 2.4.2 | LazyStore for column width/visibility persistence | Project standard |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @lucide/svelte | 0.577.0 | Icons (GripVertical for drag hint if needed) | UI polish |
| vitest | 4.1.0 | Unit tests for validation logic | Testing |
| tempfile | 3 (Rust) | Temp directories for Rust tests | Backend testing |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Native HTML5 DnD | svelte-dnd-action / SortableJS | Adds dependency; native API sufficient for simple row reorder. Animation can be done with CSS transitions on transform |
| GIT_SEQUENCE_EDITOR script | git2 rebase API | git2's rebase API is low-level and doesn't support the full interactive rebase todo format easily; CLI approach is proven and matches existing rebase_branch_inner pattern |
| Custom GIT_EDITOR helper | Pre-collecting all messages | User decisions explicitly require messages during execution, not pre-configured |

## Architecture Patterns

### Recommended Project Structure
```
src/
  components/
    RebaseEditor.svelte       # Main editor component (center pane replacement)
  lib/
    rebase-editor.svelte.ts   # Module-level $state for rebase editor state
    rebase-validation.ts      # Pure validation functions (testable)
    rebase-validation.test.ts # Unit tests for validation
src-tauri/
  src/
    commands/
      interactive_rebase.rs   # New command module: get_rebase_todo, start_interactive_rebase, submit_rebase_message
    git/
      types.rs                # New types: RebaseTodoItem, RebaseTodoAction
```

### Pattern 1: Center Pane Swap (Established)
**What:** Conditional rendering in App.svelte to replace CommitGraph with RebaseEditor
**When to use:** When rebase editor state is active
**Example:**
```typescript
// In App.svelte — add third center-pane option
{#if showRebaseEditor}
  <RebaseEditor
    repoPath={repoPath!}
    commits={rebaseEditorCommits}
    baseOid={rebaseBaseOid}
    onclose={handleRebaseEditorClose}
    onstart={handleRebaseStart}
  />
{:else if showMergeEditor && selectedFile}
  <MergeEditor ... />
{:else if showDiff}
  <DiffPanel ... />
{:else}
  <CommitGraph ... />
{/if}
```

### Pattern 2: Module-Level $state for Cross-Component State
**What:** Svelte 5 $state rune at module level for reactive state without prop drilling
**When to use:** Rebase editor state shared between App.svelte, CommitGraph, StagingPanel
**Example:**
```typescript
// src/lib/rebase-editor.svelte.ts
export interface RebaseTodoItem {
  oid: string;
  shortOid: string;
  summary: string;
  authorName: string;
  authorTimestamp: number;
  action: 'pick' | 'squash' | 'reword' | 'drop';
}

export const rebaseEditorState = $state({
  active: false,
  baseOid: null as string | null,
  commits: [] as RebaseTodoItem[],
  originalCommits: [] as RebaseTodoItem[], // for Reset
});
```

### Pattern 3: GIT_SEQUENCE_EDITOR for Todo File Control
**What:** Write the user's rebase plan to the todo file via a custom sequence editor script
**When to use:** When starting the interactive rebase
**Example (Rust):**
```rust
// Write the todo plan to a temp file
let todo_content = todo_items.iter()
    .map(|item| format!("{} {} {}", item.action, item.oid, item.summary))
    .collect::<Vec<_>>()
    .join("\n");
let todo_path = std::env::temp_dir().join("trunk-rebase-todo");
std::fs::write(&todo_path, &todo_content)?;

// Build a GIT_SEQUENCE_EDITOR that copies our todo over git's todo
// On macOS/Linux: "cp /tmp/trunk-rebase-todo \"$1\""
let seq_editor = format!("cp {} \"$1\"", todo_path.display());

let output = std::process::Command::new("git")
    .args(["rebase", "-i", &base_oid])
    .current_dir(path_buf)
    .env("GIT_TERMINAL_PROMPT", "0")
    .env("GIT_SEQUENCE_EDITOR", &seq_editor)
    .env("GIT_EDITOR", &editor_script)  // for reword/squash pauses
    .output()?;
```

### Pattern 4: GIT_EDITOR Helper for Reword/Squash Message Pauses
**What:** A helper mechanism for handling git's editor pauses during reword/squash
**When to use:** When git pauses for commit message editing during interactive rebase execution
**Implementation approach:**

The simplest robust approach for a Tauri app:

1. **Write a small shell script** as GIT_EDITOR that:
   - Receives the commit message file path as `$1`
   - Copies the file content to a well-known temp location (e.g., `/tmp/trunk-rebase-msg-input`)
   - Creates a signal file (e.g., `/tmp/trunk-rebase-msg-needed`)
   - Polls/waits for a response file (e.g., `/tmp/trunk-rebase-msg-response`)
   - Copies the response back to `$1`
   - Cleans up and exits 0

2. **Rust backend** runs git rebase in a background thread via `spawn_blocking`. A separate polling mechanism or file watcher detects when the signal file appears, reads the input message, and emits a Tauri event (`rebase-message-needed`) to the frontend.

3. **Frontend** listens for `rebase-message-needed`, shows InputDialog with the pre-filled message, and on submit calls a Tauri command (`submit_rebase_message`) that writes the response file.

4. **Shell script** detects the response file, copies content back, exits. Git continues.

```rust
// Helper script template (written to temp dir)
let editor_script = format!(r#"#!/bin/sh
cp "$1" "{input_path}"
touch "{signal_path}"
while [ ! -f "{response_path}" ]; do sleep 0.1; done
cp "{response_path}" "$1"
rm -f "{signal_path}" "{response_path}"
exit 0
"#, input_path = input_path.display(),
    signal_path = signal_path.display(),
    response_path = response_path.display());
```

### Pattern 5: Commit Listing via git2 Revwalk
**What:** List commits between a base OID (exclusive) and HEAD for the rebase todo
**When to use:** Backend command `get_rebase_todo`
**Example:**
```rust
pub fn get_rebase_todo_inner(
    path: &str,
    base_oid: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<Vec<RebaseTodoItem>, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let base = git2::Oid::from_str(base_oid)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.hide(base)?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;

    let mut items = Vec::new();
    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        items.push(RebaseTodoItem {
            oid: oid.to_string(),
            short_oid: oid.to_string()[..7].to_string(),
            summary: commit.summary().unwrap_or("").to_string(),
            author_name: commit.author().name().unwrap_or("").to_string(),
            author_timestamp: commit.time().seconds(),
            action: "pick".to_string(),
        });
    }
    // Reverse to get oldest-first (bottom-to-top in rebase todo)
    items.reverse();
    Ok(items)
}
```

### Pattern 6: Fork Point Auto-Detection for Branch Menu
**What:** Detect the base commit when user selects "Interactive Rebase" from a branch context menu
**When to use:** Branch sidebar and graph pill menus
**Example:**
```rust
// Use git merge-base to find the common ancestor
let output = std::process::Command::new("git")
    .args(["merge-base", branch_name, "HEAD"])
    .current_dir(path_buf)
    .env("GIT_TERMINAL_PROMPT", "0")
    .output()?;
let fork_point = String::from_utf8_lossy(&output.stdout).trim().to_string();
```

### Anti-Patterns to Avoid
- **Pre-collecting reword/squash messages**: User decisions explicitly state messages are written during execution, not pre-configured. Do NOT show message editors in the planning phase.
- **Reopening RebaseEditor during execution**: The editor closes on Start. Reword/squash message dialogs pop over the normal graph view. The editor does NOT reopen.
- **Using git2's low-level rebase API**: The Rebase struct in git2 is complex, doesn't map cleanly to the interactive rebase todo format, and doesn't support the full git rebase -i workflow. Use CLI subprocess like all other operations.
- **Running rebase synchronously on the main thread**: Must use `spawn_blocking` + events, same as all other git operations in the codebase.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Interactive rebase execution | Custom commit-by-commit replay | `git rebase -i` with `GIT_SEQUENCE_EDITOR` | Git handles all edge cases (conflict detection, reflog, rerere, etc.) |
| Commit message concatenation for squash | Manual message merging | Git's native squash message in COMMIT_EDITMSG | Git generates the correct combined message format |
| Fork point detection | Manual ancestor search | `git merge-base <branch> HEAD` | Handles complex histories, octopus merges, etc. |
| Column resizing | Custom resize logic | Copy CommitGraph's `startColumnResize` pattern | Already proven, handles min/max widths, LazyStore persistence |
| Context menu construction | Custom dropdown | Tauri `Menu.new({ items: [...] })` | Native OS menus, established pattern throughout codebase |

**Key insight:** The git CLI handles all the complex edge cases of interactive rebase (conflict detection, reflog entries, rerere, merge strategies). Our role is to provide the UI for plan configuration and message editing, then delegate execution to git.

## Common Pitfalls

### Pitfall 1: Rebase Todo Order
**What goes wrong:** Git's rebase todo lists commits oldest-first (top = first applied). The UI might display them in the wrong order.
**Why it happens:** `git rev-list` returns newest-first by default; rebase todo is oldest-first.
**How to avoid:** After revwalk, reverse the list so oldest commits are at the top of the editor (matching git's todo format). When writing the todo file, maintain the same order.
**Warning signs:** Rebase produces unexpected results, wrong commit order.

### Pitfall 2: Squash After Drop
**What goes wrong:** User sets action to "squash" on a commit that follows a "drop" commit, meaning there's no predecessor to squash into.
**Why it happens:** Squash combines with the previous (surviving) commit, not necessarily the adjacent row.
**How to avoid:** Validation must check that each squash commit has at least one non-dropped predecessor above it.
**Warning signs:** Git error "cannot squash without a previous commit".

### Pitfall 3: GIT_EDITOR Script Permissions
**What goes wrong:** The helper script written to temp dir isn't executable.
**Why it happens:** `std::fs::write` doesn't set execute permissions on Unix.
**How to avoid:** After writing the script, use `std::fs::set_permissions` with mode 0o755.
**Warning signs:** "Permission denied" when git tries to call the editor.

### Pitfall 4: Race Condition in Message Exchange
**What goes wrong:** Rust backend emits the `rebase-message-needed` event before the frontend listener is ready, or the response file is written before the shell script starts polling.
**Why it happens:** Asynchronous event timing between processes.
**How to avoid:** Use file-based signaling with polling. The shell script creates the signal file, then polls for response. Backend polls for signal file, then emits event. Order is guaranteed.
**Warning signs:** Rebase hangs waiting for editor response.

### Pitfall 5: Temp File Cleanup
**What goes wrong:** Temp files from previous rebase attempts interfere with new ones.
**Why it happens:** Crash or abort doesn't clean up temp files.
**How to avoid:** Use unique temp directory per rebase session (e.g., `tempfile::TempDir` in Rust). Clean up at start of new session and on abort/completion.
**Warning signs:** Stale signal files cause immediate false triggers.

### Pitfall 6: Detached HEAD State
**What goes wrong:** "Interactive Rebase" appears in menus when HEAD is detached, leading to confusing behavior.
**Why it happens:** No branch to rebase.
**How to avoid:** Hide "Interactive Rebase" menu items when HEAD is detached, consistent with merge/rebase menu pattern (Phase 39-40 decisions).
**Warning signs:** Error on rebase start with no clear branch context.

### Pitfall 7: Keyboard Shortcut Conflicts
**What goes wrong:** P/S/R/D shortcuts fire when user is typing in other inputs (search bar, commit form).
**Why it happens:** Keydown handler captures events globally.
**How to avoid:** Only handle shortcuts when a rebase editor row is focused. Check `document.activeElement` or use event delegation from the editor container.
**Warning signs:** Typing "S" in search bar changes a rebase action.

## Code Examples

### RebaseEditor Column Header (mirrors CommitGraph pattern)
```svelte
<!-- Source: CommitGraph.svelte lines 837-899 -->
<div
  class="flex items-center flex-shrink-0"
  style="height: 24px; background: var(--color-surface); border-bottom: 1px solid var(--color-border); font-size: 11px; color: var(--color-text-muted); padding: 0 8px;"
  oncontextmenu={showHeaderContextMenu}
>
  <!-- Action column (always visible, no resize) -->
  <div class="flex-shrink-0" style="width: {columnWidths.action}px; padding: 0 8px;">
    Action
  </div>
  {#if columnVisibility.sha}
    <div class="flex-shrink-0 relative" style="width: {columnWidths.sha}px; padding: 0 8px;">
      SHA
      <div class="col-resize-handle" onmousedown={(e) => startColumnResize('sha', e)}></div>
    </div>
  {/if}
  <!-- Message is flex-1, always visible -->
  <div class="flex-1 relative" style="padding: 0 8px;">
    Message
    <div class="col-resize-handle" onmousedown={(e) => startColumnResize('author', e, true)}></div>
  </div>
  {#if columnVisibility.author}
    <div class="flex-shrink-0 relative" style="width: {columnWidths.author}px; padding: 0 8px;">
      Author
    </div>
  {/if}
  {#if columnVisibility.date}
    <div class="flex-shrink-0" style="width: {columnWidths.date}px; padding: 0 8px;">
      Date
    </div>
  {/if}
</div>
```

### Action Dropdown Select
```svelte
<select
  bind:value={item.action}
  style="
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    font-size: 11px;
    padding: 1px 4px;
    border-radius: 3px;
    cursor: pointer;
  "
>
  <option value="pick">Pick</option>
  <option value="reword">Reword</option>
  <option value="squash">Squash</option>
  <option value="drop">Drop</option>
</select>
```

### Native HTML5 Drag-and-Drop Row Reorder
```svelte
<script lang="ts">
  let dragIdx = $state<number | null>(null);

  function handleDragStart(e: DragEvent, idx: number) {
    dragIdx = idx;
    e.dataTransfer?.setData('text/plain', String(idx));
    if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
  }

  function handleDragOver(e: DragEvent, idx: number) {
    e.preventDefault();
    if (dragIdx === null || dragIdx === idx) return;
    // Swap items
    const items = [...commits];
    const [moved] = items.splice(dragIdx, 1);
    items.splice(idx, 0, moved);
    commits = items;
    dragIdx = idx;
  }

  function handleDragEnd() {
    dragIdx = null;
  }
</script>

{#each commits as item, idx}
  <div
    draggable="true"
    ondragstart={(e) => handleDragStart(e, idx)}
    ondragover={(e) => handleDragOver(e, idx)}
    ondragend={handleDragEnd}
    style="cursor: grab; {item.action === 'drop' ? 'opacity: 0.4; text-decoration: line-through;' : ''}"
  >
    <!-- row content -->
  </div>
{/each}
```

### Validation Logic (pure function)
```typescript
// src/lib/rebase-validation.ts
export interface ValidationError {
  index: number;
  message: string;
}

export function validateRebasePlan(
  items: { action: string }[]
): ValidationError[] {
  const errors: ValidationError[] = [];

  // Rule 1: Can't squash the first non-dropped commit
  const firstNonDrop = items.findIndex(i => i.action !== 'drop');
  if (firstNonDrop >= 0 && items[firstNonDrop].action === 'squash') {
    errors.push({
      index: firstNonDrop,
      message: "Cannot squash the first commit",
    });
  }

  // Rule 2: Can't drop all commits
  if (items.every(i => i.action === 'drop')) {
    errors.push({
      index: 0,
      message: "Cannot drop all commits",
    });
  }

  // Rule 3: Squash must have a non-dropped predecessor
  for (let i = 0; i < items.length; i++) {
    if (items[i].action === 'squash') {
      const hasPredecessor = items.slice(0, i).some(
        p => p.action !== 'drop'
      );
      if (!hasPredecessor) {
        errors.push({
          index: i,
          message: "No preceding commit to squash into",
        });
      }
    }
  }

  return errors;
}
```

### Tauri Command: get_rebase_todo
```rust
#[derive(Debug, Serialize, Clone)]
pub struct RebaseTodoItem {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
    pub author_name: String,
    pub author_timestamp: i64,
}

#[tauri::command]
pub async fn get_rebase_todo(
    path: String,
    base_oid: String,
    state: State<'_, RepoState>,
) -> Result<Vec<RebaseTodoItem>, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        get_rebase_todo_inner(&path, &base_oid, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual `git rebase -i` in terminal | GUI rebase editors (GitKraken, SourceTree, VS Code) | 2018+ | Users expect visual drag-and-drop and action selectors |
| `GIT_EDITOR=vim` for all editing | `GIT_SEQUENCE_EDITOR` separates todo editing from message editing | Git 1.7.8+ | Enables programmatic todo control while still supporting message editing |
| git2 Rebase struct for interactive | CLI subprocess with `GIT_SEQUENCE_EDITOR` | Proven pattern | CLI handles all edge cases; git2 rebase API is low-level and complex |

**Deprecated/outdated:**
- git2's `Rebase` struct: While it exists, it maps to low-level rebase internals, not the user-facing `git rebase -i` workflow. The CLI approach is universally preferred in GUI clients.

## Open Questions

1. **Shell Script Cross-Platform Compatibility**
   - What we know: macOS uses `/bin/sh` and `cp` command. The project currently targets macOS.
   - What's unclear: If Windows support is ever needed, shell scripts won't work directly.
   - Recommendation: Use `/bin/sh` for now (project is macOS-only based on current Cargo.toml and `libc` dependency). Document the platform assumption.

2. **Large Rebase Performance**
   - What we know: Rebasing 100+ commits is rare but possible. The editor doesn't use virtualization.
   - What's unclear: Whether a simple `{#each}` loop is sufficient or virtual scrolling is needed.
   - Recommendation: Start without virtualization. Rebase editors rarely have more than 20-50 commits. Add VirtualList later if performance issues arise.

3. **Concurrent Rebase Prevention**
   - What we know: Git prevents concurrent rebases at the filesystem level. The OperationBanner detects in-progress operations.
   - What's unclear: Whether the "Interactive Rebase" menu item should be hidden/disabled during an active operation.
   - Recommendation: Disable/hide the menu item when `get_operation_state` returns a non-None operation type.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.0 (frontend) + Rust #[test] (backend) |
| Config file | `vite.config.ts` (test section) |
| Quick run command | `npx vitest run src/lib/rebase-validation.test.ts` |
| Full suite command | `npx vitest run && cd src-tauri && cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IREB-01 | Commits listed between base and HEAD | unit (Rust) | `cd src-tauri && cargo test interactive_rebase` | Wave 0 |
| IREB-02 | Drag reorder updates array | manual-only | N/A (DOM interaction) | N/A |
| IREB-03 | Keyboard shortcuts change action | manual-only | N/A (DOM interaction) | N/A |
| IREB-04 | Validation blocks invalid plans | unit | `npx vitest run src/lib/rebase-validation.test.ts` | Wave 0 |
| IREB-05 | Cancel/Reset restore state | manual-only | N/A (component state) | N/A |
| IREB-06 | Reword shows message editor | integration | manual verification | N/A |
| IREB-07 | Squash combines with predecessor message | integration | manual verification | N/A |
| REB-03 | Context menu triggers interactive rebase | manual-only | N/A (native menu) | N/A |

### Sampling Rate
- **Per task commit:** `npx vitest run`
- **Per wave merge:** `npx vitest run && cd src-tauri && cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/lib/rebase-validation.ts` -- pure validation functions
- [ ] `src/lib/rebase-validation.test.ts` -- unit tests for validation rules
- [ ] `src-tauri/src/commands/interactive_rebase.rs` -- backend commands with #[test] module

## Sources

### Primary (HIGH confidence)
- App.svelte (lines 432-455) -- Center pane swap pattern for MergeEditor, to replicate for RebaseEditor
- CommitGraph.svelte (lines 62-66, 129-162, 576-594, 837-899) -- Column system, resize handles, header context menu, LazyStore persistence
- operation_state.rs -- Existing rebase_branch_inner, GIT_EDITOR=true, GIT_TERMINAL_PROMPT=0 patterns
- store.ts -- LazyStore usage for column widths/visibility persistence
- types.ts -- Existing DTO types (GraphCommit, OperationInfo)
- InputDialog.svelte -- Modal dialog pattern for reword/squash message editing
- OperationBanner.svelte -- Rebase progress display, Continue/Skip/Abort buttons

### Secondary (MEDIUM confidence)
- [Git rebase documentation](https://git-scm.com/docs/git-rebase) -- GIT_SEQUENCE_EDITOR behavior, todo file format, reword/squash semantics
- [Interactive git rebase with non-interactive editing](https://raimue.blog/2018/04/06/interactive-git-rebase-with-non-interactive-editing/) -- GIT_SEQUENCE_EDITOR technique for programmatic todo control
- [GitKraken Interactive Rebase docs](https://help.gitkraken.com/gitkraken-desktop/interactive-rebase/) -- UI/UX reference for action selectors, drop visual
- [Svelte 5 drag and drop patterns](https://dev.to/artxe2/implementing-drag-and-drop-using-svelte-5-767) -- Native HTML5 DnD with Svelte 5 runes

### Tertiary (LOW confidence)
- Shell script helper for GIT_EDITOR -- approach based on understanding of git internals and file-based IPC; needs validation through implementation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in use in the project
- Architecture: HIGH - All patterns established by prior phases (37-40)
- Pitfalls: MEDIUM - Some edge cases (race conditions, temp file cleanup) need implementation validation
- Drag-and-drop: MEDIUM - Native HTML5 DnD is straightforward but animation quality needs testing
- GIT_EDITOR helper: MEDIUM - File-based IPC approach is conceptually sound but needs careful implementation

**Research date:** 2026-03-21
**Valid until:** 2026-04-21 (stable domain, no moving targets)
