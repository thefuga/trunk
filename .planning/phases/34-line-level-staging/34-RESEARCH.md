# Phase 34: Line-Level Staging - Research

**Researched:** 2026-03-18
**Domain:** Git partial patch construction, Svelte 5 interactive line selection UI
**Confidence:** HIGH

## Summary

Line-level staging requires two major capabilities: (1) a backend that constructs valid partial unified diff patches from a subset of selected lines within a hunk and applies them to the git index or working directory, and (2) a frontend that lets users click individual add/delete lines, tracks selection state per-hunk, and dynamically swaps the hunk toolbar between hunk-mode and selection-mode buttons.

The backend approach is to manually construct a unified diff patch string from the original hunk data, including only the user-selected lines (with unselected add lines removed entirely and unselected delete lines converted to context lines), then parse it with `git2::Diff::from_buffer()` and apply it via `repo.apply()`. This is the standard approach used by tools like `git add -e` and GUI clients. The existing `stage_hunk_inner`/`unstage_hunk_inner`/`discard_hunk_inner` functions provide the pattern for diff generation, error handling, and `ApplyLocation` targeting.

The frontend is a straightforward extension of the existing `DiffPanel.svelte` -- add `$state` for selected line indices scoped per hunk, click/shift-click handlers on add/delete line divs, conditional toolbar button rendering based on selection count, and Escape key to clear. CSS custom properties for brighter selected-line backgrounds go in `app.css`.

**Primary recommendation:** Construct partial patches as text strings (unified diff format), feed to `Diff::from_buffer()`, apply with `repo.apply()`. Do not attempt to use `hunk_callback` filtering for line-level granularity -- it only works at hunk granularity.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Click to toggle** individual add/delete lines -- each click adds/removes a line from the selection
- **Shift+click** to select a range of lines between last-clicked and shift-clicked line
- **Context lines are not selectable** -- only add and delete lines respond to clicks
- **Selection scoped to a single hunk** -- clicking a line in a different hunk clears the previous selection
- **Symmetric for staged diffs** -- same click-to-select interaction works in staged diffs for "Unstage Lines"
- **Commit diffs** remain read-only (no selection)
- Selected lines get a **brighter/more saturated version** of their add/delete background color
- **Pointer cursor** on add/delete lines to signal clickability; context lines keep default cursor
- Selection clears **after operation completes** and **when navigating** to a different file or clicking a line in another hunk
- Toolbar has two modes: **hunk mode** (no selection) and **selection mode** (lines selected)
- Buttons **replace** the hunk-level ones -- not added alongside
- Button labels **include count**: "Stage Lines (3)", "Discard Lines (2)"
- **Escape key clears** selection and restores hunk-level buttons
- "Discard Lines (N)" replaces "Discard Hunk" when lines selected (unstaged diffs only)
- Confirmation dialog: **"Discard N selected lines? This cannot be undone."** using `ask()` from `@tauri-apps/plugin-dialog`

### Claude's Discretion
- Partial patch construction algorithm (how to build a valid git patch from selected lines within a hunk)
- CSS custom properties for selected-line highlight colors (brighter variants of existing diff colors)
- Backend command signatures (`stage_lines`, `unstage_lines`, `discard_lines`) and parameter design
- Whether to send line indices or line content to the backend
- Escape key handler integration with existing keyboard shortcut system

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| HUNK-07 | User can select and stage individual lines within a diff hunk | Partial patch construction algorithm, `stage_lines` backend command, line selection UI, toolbar mode switching |
| HUNK-08 | User can select and unstage individual lines within a diff hunk | Same algorithm with reversed diff and `ApplyLocation::Index`, `unstage_lines` backend command |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.19 | Diff generation, `Diff::from_buffer`, `repo.apply()` | Already in project; `from_buffer` enables text-based partial patch application |
| Svelte 5 | (existing) | `$state` for line selection, `$derived` for selection count, `$effect` for Escape key | Already in project; reactive state is ideal for selection tracking |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/plugin-dialog | (existing) | `ask()` for discard lines confirmation | Destructive discard-lines operation |
| serde_json | (existing) | Serialize line indices from frontend to backend | IPC parameter passing |

No new dependencies needed. All functionality builds on existing libraries.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/commands/staging.rs  # Add stage_lines_inner, unstage_lines_inner, discard_lines_inner + Tauri commands
src/components/DiffPanel.svelte     # Add selection state, click handlers, toolbar mode switching
src/app.css                         # Add CSS custom properties for selected line colors
src-tauri/src/lib.rs               # Register new commands in invoke_handler
```

### Pattern 1: Partial Patch Construction Algorithm
**What:** Given a hunk's lines and a set of selected line indices, construct a valid unified diff patch text that stages/unstages only the selected lines.
**When to use:** Every `stage_lines`, `unstage_lines`, and `discard_lines` call.

**Algorithm (staging selected lines from unstaged diff):**
1. Get the full diff for the file (index -> workdir)
2. Extract the target hunk by index using `Patch::from_diff`
3. Build a unified diff text string:
   - Write the diff header: `diff --git a/{path} b/{path}`
   - Write `--- a/{path}` and `+++ b/{path}`
   - Write the hunk header `@@ -old_start,OLD_LINES +new_start,NEW_LINES @@`
   - For each line in the hunk:
     - **Context line:** Always include (counts toward both old and new)
     - **Delete line (selected):** Include as `-` line (counts toward old only)
     - **Delete line (NOT selected):** Convert to context line (space prefix, counts toward both)
     - **Add line (selected):** Include as `+` line (counts toward new only)
     - **Add line (NOT selected):** Remove entirely (counts toward neither)
   - Recalculate `OLD_LINES` = count of context + selected deletes
   - Recalculate `NEW_LINES` = count of context + unselected deletes (now context) + selected adds
4. Parse with `Diff::from_buffer(patch_text.as_bytes())`
5. Apply with `repo.apply(&diff, ApplyLocation::Index, None)`

**Critical detail on recalculation:**
- `old_start` stays the same as the original hunk
- `new_start` stays the same as the original hunk
- `OLD_LINES` = (context lines) + (selected delete lines) + (unselected delete lines converted to context) = total lines that exist in old file within this hunk range = always equals the original `old_lines` value
- `NEW_LINES` = (context lines) + (unselected delete lines as context) + (selected add lines) = what the new file will look like after applying only selected changes

Wait -- let me be precise. The hunk header counts work like this:
- `old_lines` = number of lines prefixed with ` ` (context) + number of lines prefixed with `-` (delete)
- `new_lines` = number of lines prefixed with ` ` (context) + number of lines prefixed with `+` (add)

So in the constructed partial patch:
- **Context lines** (original context + unselected deletes converted to context): count toward both old_lines and new_lines
- **Selected delete lines** (kept as `-`): count toward old_lines only
- **Selected add lines** (kept as `+`): count toward new_lines only
- **Unselected add lines** (removed entirely): count toward nothing

Therefore:
- `old_lines` = (original context) + (unselected deletes as context) + (selected deletes) = always equals original old_lines
- `new_lines` = (original context) + (unselected deletes as context) + (selected adds)

**Example:**
```
Original hunk:
@@ -5,7 +5,8 @@
 context line 1
 context line 2
-deleted line A     (selected)
-deleted line B     (NOT selected)
+added line X       (selected)
+added line Y       (NOT selected)
+added line Z       (selected)
 context line 3

Partial patch (stage only selected):
@@ -5,7 +5,7 @@
 context line 1
 context line 2
-deleted line A
 deleted line B
+added line X
+added line Z
 context line 3

old_lines = 3 context + 1 unselected-delete-as-context + 1 selected-delete + 2 trailing = 7 (same as original)
new_lines = 3 context + 1 unselected-delete-as-context + 2 selected-adds + 0 trailing = wait...
```

Let me recalculate carefully:
- ` context line 1` -> context (old=1, new=1)
- ` context line 2` -> context (old=1, new=1)
- `-deleted line A` -> delete, selected (old=1, new=0)
- ` deleted line B` -> unselected delete converted to context (old=1, new=1)
- `+added line X` -> add, selected (old=0, new=1)
- `+added line Z` -> add, selected (old=0, new=1)
- ` context line 3` -> context (old=1, new=1)

old_lines = 1+1+1+1+0+0+1 = 5... but original was 7.

Actually this is wrong. Let me reconsider. The original hunk has 7 old lines and 8 new lines:
- context line 1 (old, new)
- context line 2 (old, new)
- deleted line A (old only)
- deleted line B (old only)
- added line X (new only)
- added line Y (new only)
- added line Z (new only)
- context line 3 (old, new)

old = 3 context + 2 deletes = 5, not 7. So original header would be `@@ -5,5 +5,6 @@`.

The key algorithm is correct:
- old_lines = count of lines starting with ` ` or `-` in the constructed patch
- new_lines = count of lines starting with ` ` or `+` in the constructed patch

```rust
// Pseudocode for the core algorithm
fn build_partial_patch(
    file_path: &str,
    hunk: &DiffHunk,  // from Patch iteration
    selected_indices: &[usize],  // indices into hunk.lines
) -> String {
    let selected_set: HashSet<usize> = selected_indices.iter().copied().collect();
    let mut patch_lines: Vec<String> = Vec::new();
    let mut old_count = 0u32;
    let mut new_count = 0u32;

    for (i, line) in hunk_lines.iter().enumerate() {
        match line.origin {
            '+' => {
                if selected_set.contains(&i) {
                    patch_lines.push(format!("+{}", line.content));
                    new_count += 1;
                }
                // else: skip entirely
            }
            '-' => {
                if selected_set.contains(&i) {
                    patch_lines.push(format!("-{}", line.content));
                    old_count += 1;
                } else {
                    // Convert to context line
                    patch_lines.push(format!(" {}", line.content));
                    old_count += 1;
                    new_count += 1;
                }
            }
            ' ' => {
                patch_lines.push(format!(" {}", line.content));
                old_count += 1;
                new_count += 1;
            }
        }
    }

    format!(
        "diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n@@ -{},{} +{},{} @@\n{}\n",
        path = file_path,
        hunk.old_start, old_count,
        hunk.new_start, new_count,
        patch_lines.join("\n")
    )
}
```

### Pattern 2: Unstage Lines (Reversed Partial Patch)
**What:** For unstaging selected lines from the staged diff, generate a reversed diff (HEAD -> index, with `.reverse(true)`) and construct a partial patch from it, then apply to Index.
**When to use:** `unstage_lines` command.

The algorithm is identical to staging, but:
- The diff source is `diff_tree_to_index` with `.reverse(true)` (same as `unstage_hunk_inner`)
- Apply location is `ApplyLocation::Index`
- The `+` and `-` semantics are already reversed by the `.reverse(true)` flag, so the same partial patch algorithm works

### Pattern 3: Discard Lines (Reversed Partial Patch to WorkDir)
**What:** For discarding selected lines from the unstaged diff, generate a reversed diff (workdir -> index, with `.reverse(true)`) and construct a partial patch, then apply to WorkDir.
**When to use:** `discard_lines` command.

Same algorithm but:
- Diff source is `diff_index_to_workdir` with `.reverse(true)` (same as `discard_hunk_inner`)
- Apply location is `ApplyLocation::WorkDir`

### Pattern 4: Frontend Line Selection State
**What:** Track which lines are selected within a single hunk using Svelte 5 reactive state.
**When to use:** DiffPanel component.

```typescript
// Selection state
let selectedHunkKey = $state<string | null>(null);  // e.g., "src/main.rs-2"
let selectedLineIndices = $state<Set<number>>(new Set());
let lastClickedIndex = $state<number | null>(null);

// Derived count for button labels
let selectedCount = $derived(selectedLineIndices.size);

// Click handler
function handleLineClick(hunkKey: string, lineIndex: number, origin: DiffOrigin, e: MouseEvent) {
  if (origin === 'Context') return;  // Context lines not selectable
  if (diffKind === 'commit') return;  // Commit diffs read-only

  // If clicking in different hunk, clear selection
  if (hunkKey !== selectedHunkKey) {
    selectedHunkKey = hunkKey;
    selectedLineIndices = new Set([lineIndex]);
    lastClickedIndex = lineIndex;
    return;
  }

  if (e.shiftKey && lastClickedIndex !== null) {
    // Range select
    const start = Math.min(lastClickedIndex, lineIndex);
    const end = Math.max(lastClickedIndex, lineIndex);
    const newSet = new Set(selectedLineIndices);
    for (let i = start; i <= end; i++) {
      // Only add selectable lines (add/delete)
      if (hunkLines[i].origin !== 'Context') {
        newSet.add(i);
      }
    }
    selectedLineIndices = newSet;
  } else {
    // Toggle single line
    const newSet = new Set(selectedLineIndices);
    if (newSet.has(lineIndex)) {
      newSet.delete(lineIndex);
    } else {
      newSet.add(lineIndex);
    }
    selectedLineIndices = newSet;
    lastClickedIndex = lineIndex;
  }
}
```

### Pattern 5: Backend Command Parameter Design
**What:** Send line indices (not line content) from frontend to backend. Indices are positions within the hunk's `lines` array.
**When to use:** All three line-level commands.

**Recommendation: Send line indices.** The frontend already has the hunk index and knows which lines within that hunk are selected by position. Sending indices is simpler, smaller, and unambiguous. The backend can validate indices against the actual hunk from a freshly-generated diff.

```rust
// Command signature
pub fn stage_lines_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,  // indices into hunk.lines array
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError>
```

### Anti-Patterns to Avoid
- **Using `hunk_callback` for line-level filtering:** `hunk_callback` on `ApplyOptions` only accepts/rejects entire hunks. It cannot filter individual lines within a hunk. You MUST construct a new diff with only the desired lines.
- **Modifying the Diff object in place:** `git2::Diff` does not support mutation. You must construct a new diff text and parse it with `Diff::from_buffer()`.
- **Sending line content instead of indices:** Line content can be ambiguous (duplicate lines). Indices are unambiguous and smaller.
- **Forgetting newline handling:** Ensure each line in the patch text ends properly. The `content` field from `DiffLine` may or may not include a trailing newline -- check and handle consistently.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Diff generation | Custom diff algorithm | `git2::Repository::diff_index_to_workdir` / `diff_tree_to_index` | Must match git's own diff to get correct line numbers |
| Patch application | Manual file manipulation | `git2::Repository::apply` with `Diff::from_buffer` | Handles edge cases (binary, permissions, CRLF, merge conflicts) |
| Confirmation dialogs | Custom modal | `ask()` from `@tauri-apps/plugin-dialog` | Native OS dialog, consistent with existing discard patterns |
| Toast notifications | Custom notification | `showToast()` | Already implemented, tested |

**Key insight:** The partial patch text construction is the ONE thing that must be hand-rolled -- there is no library function that takes "a hunk + selected line indices" and produces a partial patch. But everything around it (diff generation, patch parsing, patch application) uses existing `git2` functions.

## Common Pitfalls

### Pitfall 1: Newline Handling in Patch Text
**What goes wrong:** The constructed patch text has incorrect or missing newlines, causing `Diff::from_buffer` to fail or `repo.apply` to produce wrong results.
**Why it happens:** `DiffLine.content` from git2's `line.content()` includes the content but may or may not have a trailing newline. The `String::from_utf8_lossy` conversion preserves whatever bytes are there.
**How to avoid:** When iterating hunk lines via `Patch::from_diff`, check whether content ends with `\n`. If not (e.g., "No newline at end of file"), handle accordingly. Always ensure the patch text itself is well-formed.
**Warning signs:** "patch does not apply" errors, silent data corruption in applied files.

### Pitfall 2: Stale Hunk Index After Concurrent Operations
**What goes wrong:** User selects lines, but by the time the backend processes the command, the file's diff has changed (another operation completed), making the hunk index invalid.
**Why it happens:** The existing `hunkOperationInFlight` flag prevents concurrent operations, but if the flag is not properly extended to cover line operations, race conditions can occur.
**How to avoid:** Reuse the existing `hunkOperationInFlight` boolean to disable ALL hunk and line buttons during any operation. The backend should also validate the hunk index and return `stale_hunk_index` error (already implemented pattern).
**Warning signs:** "Hunk index out of range" errors after rapid clicking.

### Pitfall 3: Selection State Not Cleared After File Navigation
**What goes wrong:** User selects lines in file A's diff, then clicks file B in the staging panel. The selection visuals disappear (new diff renders), but the state variables retain stale values. When returning to file A, ghost selections appear.
**Why it happens:** The `$effect` that resets `focusedHunkIndex` when `fileDiffs` changes needs to also clear the selection state.
**How to avoid:** Clear `selectedHunkKey`, `selectedLineIndices`, and `lastClickedIndex` in the existing `$effect` that watches `fileDiffs`.
**Warning signs:** Selections appearing on wrong lines after switching files.

### Pitfall 4: Diff Header Format for New/Deleted Files
**What goes wrong:** The constructed patch text uses `a/{path}` and `b/{path}`, but for newly added files the "old" side is `/dev/null`, and for deleted files the "new" side is `/dev/null`.
**Why it happens:** Line-level staging on a file whose entire content is added (status = Added) or deleted (status = Deleted) has different diff header requirements.
**How to avoid:** Check the file's `DiffStatus`. For `Added` files, use `--- /dev/null`. For `Deleted` files, use `+++ /dev/null`. For `Modified` files, use the normal `a/` and `b/` prefixes.
**Warning signs:** "patch does not apply" errors on new or deleted files.

### Pitfall 5: Inline Colors Violation
**What goes wrong:** Selected line highlight colors are hardcoded in the component instead of using CSS custom properties.
**Why it happens:** Quick implementation shortcut.
**How to avoid:** Per project convention (feedback_no_inline_colors.md), ALL colors must use CSS custom properties defined in `app.css`. Add `--color-diff-add-bg-selected` and `--color-diff-delete-bg-selected` to the theme.
**Warning signs:** Code review rejection, inconsistent with theme system.

## Code Examples

### Backend: Constructing and Applying a Partial Patch
```rust
// Source: Algorithm derived from git add -e rules + git2 API
fn build_partial_patch_text(
    file_path: &str,
    patch: &git2::Patch<'_>,
    hunk_idx: usize,
    selected_indices: &[u32],
) -> Result<String, TrunkError> {
    let selected_set: std::collections::HashSet<u32> =
        selected_indices.iter().copied().collect();

    let (hunk, _) = patch.hunk(hunk_idx)?;
    let num_lines = patch.num_lines_in_hunk(hunk_idx)?;

    let mut patch_lines: Vec<String> = Vec::new();
    let mut old_count: u32 = 0;
    let mut new_count: u32 = 0;

    for line_idx in 0..num_lines {
        let line = patch.line_in_hunk(hunk_idx, line_idx)?;
        let content = String::from_utf8_lossy(line.content());
        // Ensure content ends with newline for patch format
        let content_str = if content.ends_with('\n') {
            content.into_owned()
        } else {
            format!("{}\n", content)
        };

        match line.origin() {
            '+' => {
                if selected_set.contains(&(line_idx as u32)) {
                    patch_lines.push(format!("+{}", content_str));
                    new_count += 1;
                }
                // Unselected add: skip entirely
            }
            '-' => {
                if selected_set.contains(&(line_idx as u32)) {
                    patch_lines.push(format!("-{}", content_str));
                    old_count += 1;
                } else {
                    // Unselected delete: convert to context
                    patch_lines.push(format!(" {}", content_str));
                    old_count += 1;
                    new_count += 1;
                }
            }
            _ => {
                // Context line
                patch_lines.push(format!(" {}", content_str));
                old_count += 1;
                new_count += 1;
            }
        }
    }

    let patch_text = format!(
        "diff --git a/{path} b/{path}\n--- a/{path}\n+++ b/{path}\n@@ -{},{} +{},{} @@\n{}",
        path = file_path,
        hunk.old_start(), old_count,
        hunk.new_start(), new_count,
        patch_lines.join("")
    );

    Ok(patch_text)
}

// Usage in stage_lines_inner:
pub fn stage_lines_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(file_path);
    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;

    if diff.deltas().len() == 0 {
        return Err(TrunkError::new("file_not_found",
            format!("No unstaged changes for: {}", file_path)));
    }

    let patch = git2::Patch::from_diff(&diff, 0)?
        .ok_or_else(|| TrunkError::new("file_not_found", "Binary or unchanged file"))?;

    if (hunk_index as usize) >= patch.num_hunks() {
        return Err(TrunkError::new("stale_hunk_index",
            format!("Hunk index {} out of range", hunk_index)));
    }

    let patch_text = build_partial_patch_text(
        file_path, &patch, hunk_index as usize, &line_indices
    )?;
    drop(patch);
    drop(diff);

    let partial_diff = git2::Diff::from_buffer(patch_text.as_bytes())
        .map_err(|e| TrunkError::new("patch_parse_failed", e.message().to_owned()))?;

    repo.apply(&partial_diff, git2::ApplyLocation::Index, None)
        .map_err(|e| TrunkError::new("line_apply_failed", e.message().to_owned()))?;

    Ok(())
}
```

### Frontend: Selected Line Background Color
```css
/* Source: app.css theme extension */
/* Selected line highlights -- brighter variants of existing diff colors */
--color-diff-add-bg-selected: rgba(74, 222, 128, 0.25);
--color-diff-delete-bg-selected: rgba(248, 113, 113, 0.25);
```

```typescript
// Source: DiffPanel.svelte lineBackground extension
function lineBackground(origin: string, isSelected: boolean): string {
  if (origin === 'Add') return isSelected ? 'var(--color-diff-add-bg-selected)' : 'var(--color-diff-add-bg)';
  if (origin === 'Delete') return isSelected ? 'var(--color-diff-delete-bg-selected)' : 'var(--color-diff-delete-bg)';
  return 'transparent';
}
```

### Frontend: IPC Call for Stage Lines
```typescript
// Source: DiffPanel.svelte handler
async function handleStageLines(filePath: string, hunkIndex: number) {
  hunkOperationInFlight = true;
  try {
    await safeInvoke('stage_lines', {
      path: repoPath,
      filePath,
      hunkIndex,
      lineIndices: Array.from(selectedLineIndices),
    });
    await onhunkaction?.(filePath);
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Stage lines failed', 'error');
  } finally {
    hunkOperationInFlight = false;
    clearSelection();
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `git add -e` (text editor) | GUI line selection | Always available in GUIs | Much better UX; no manual patch editing |
| `ApplyOptions::hunk_callback` (hunk level) | `Diff::from_buffer` (line level) | N/A -- different granularity | `hunk_callback` only does hunk-level; `from_buffer` enables line-level |

**Note:** `git2` 0.19 is the version in use. The `Diff::from_buffer`, `Patch::from_diff`, `Patch::line_in_hunk`, and `repo.apply` APIs are stable and well-tested at this version.

## Open Questions

1. **"No newline at end of file" marker**
   - What we know: Git diffs include a `\ No newline at end of file` marker when the last line lacks a trailing newline. The `DiffLine` origin for this is `>` (git2 uses `GIT_DIFF_LINE_CONTEXT_EOFNL`, `GIT_DIFF_LINE_ADD_EOFNL`, `GIT_DIFF_LINE_DEL_EOFNL`).
   - What's unclear: Whether `from_buffer` + `apply` handle this correctly when the partial patch includes the last line.
   - Recommendation: Test this edge case in the backend tests. If needed, include the `\ No newline at end of file` marker in the constructed patch text.

2. **Diff header for new files**
   - What we know: For files with status `Added`, the diff header should use `--- /dev/null` instead of `--- a/path`.
   - What's unclear: Whether line-level staging of a brand-new file is a realistic use case (the entire file is one add hunk).
   - Recommendation: Handle it for correctness. Check delta status from the `Patch` and adjust headers accordingly.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust: `cargo test` (built-in), Frontend: Vitest 4.1.0 |
| Config file | Rust: `src-tauri/Cargo.toml`, Frontend: `package.json` (`vitest run`) |
| Quick run command | `cd src-tauri && cargo test stage_lines -- --nocapture` |
| Full suite command | `cd src-tauri && cargo test && cd .. && npm test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HUNK-07 | Stage selected add lines from unstaged hunk | unit | `cd src-tauri && cargo test stage_lines_stages_selected_adds -x` | Wave 0 |
| HUNK-07 | Stage selected delete lines from unstaged hunk | unit | `cd src-tauri && cargo test stage_lines_stages_selected_deletes -x` | Wave 0 |
| HUNK-07 | Stage lines with mixed add/delete selection | unit | `cd src-tauri && cargo test stage_lines_mixed_selection -x` | Wave 0 |
| HUNK-07 | Stage lines -- stale hunk index error | unit | `cd src-tauri && cargo test stage_lines_stale_index -x` | Wave 0 |
| HUNK-08 | Unstage selected lines from staged hunk | unit | `cd src-tauri && cargo test unstage_lines_unstages_selected -x` | Wave 0 |
| HUNK-07+08 | Discard selected lines from unstaged hunk | unit | `cd src-tauri && cargo test discard_lines_discards_selected -x` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test`
- **Per wave merge:** `cd src-tauri && cargo test && npm test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/commands/staging.rs` -- tests for `stage_lines_inner`, `unstage_lines_inner`, `discard_lines_inner` (add to existing test module)
- [ ] No new test files needed -- tests go in existing `staging.rs#[cfg(test)]` module
- [ ] No new framework install needed

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/commands/staging.rs` -- Existing `stage_hunk_inner`, `unstage_hunk_inner`, `discard_hunk_inner` patterns (verified by reading source)
- `src-tauri/src/commands/diff.rs` -- `walk_diff_into_file_diffs` showing how lines are extracted from git2 diffs (verified by reading source)
- `src/components/DiffPanel.svelte` -- Current toolbar rendering, `lineBackground()`, `hunkOperationInFlight` pattern (verified by reading source)
- `src/app.css` -- Current CSS custom properties for diff colors (verified by reading source)
- [git2 ApplyOptions docs](https://docs.rs/git2/latest/git2/struct.ApplyOptions.html) -- `hunk_callback` and `delta_callback` API
- [git2 Diff::from_buffer docs](https://docs.rs/git2/latest/git2/struct.Diff.html) -- Parse unified diff text into Diff object
- [git2 Patch API](https://docs.rs/git2/latest/git2/struct.Patch.html) -- `from_diff`, `num_hunks`, `line_in_hunk`

### Secondary (MEDIUM confidence)
- [Git Line Staging & Patch Editing - Ken Muse](https://www.kenmuse.com/blog/git-line-staging-editor/) -- Rules for editing unified diff hunks (add line removal, delete line conversion to context)
- [Rust forum: Diff::from_buffer + repo.apply usage](https://users.rust-lang.org/t/getting-an-error-when-trying-to-apply-git-patch-with-git2/63258) -- Confirmed API works with text patches
- [Git Interactive Staging docs](https://git-scm.com/book/en/v2/Git-Tools-Interactive-Staging) -- Official git documentation on patch editing rules

### Tertiary (LOW confidence)
- [GitButler hunk management](https://deepwiki.com/gitbutlerapp/gitbutler/2.4-workspace-and-hunk-management) -- High-level architecture only; their approach differs (assignment model, not traditional staging)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new libraries needed; all APIs verified in existing codebase
- Architecture: HIGH -- partial patch construction algorithm is well-understood (same as `git add -e` rules); backend pattern follows existing `stage_hunk_inner` structure exactly
- Pitfalls: HIGH -- identified from direct code reading and understanding of git unified diff format
- Frontend interaction: HIGH -- straightforward Svelte 5 reactive state; follows existing patterns in DiffPanel

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable domain; git2 0.19 is pinned)
