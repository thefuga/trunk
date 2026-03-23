# Phase 38: Merge Editor - Research

**Researched:** 2026-03-20
**Domain:** Three-panel merge editor UI + git2 conflict resolution backend
**Confidence:** HIGH

## Summary

Phase 38 builds a three-panel merge editor that replaces the DiffPanel when a conflicted file is clicked. The backend must extract ours/theirs/base file versions from git's index conflict entries (stages 1-3) using git2's `Index::conflict_get()` API and `Repository::find_blob()`. The frontend must render three synchronized scroll panels (Current + Incoming side-by-side on top, editable Output spanning bottom) with per-hunk and per-line selection toggles, real-time output computation, and a resolution workflow that writes the merged result to disk and stages the file.

The architecture splits cleanly into: (1) a Rust backend command `get_merge_sides` that returns ours/theirs/base file content as strings, (2) a Rust backend command `save_merge_result` that writes the output to disk and stages the file, (3) a frontend `MergeEditor.svelte` component that parses the content into conflict regions and non-conflict context, manages selection state, computes the output in real-time, and handles synchronized scrolling. The existing `DiffPanel.svelte` patterns for line rendering, hunk toolbars, and CSS custom property usage provide a strong reference.

**Primary recommendation:** Extract ours/theirs/base content via git2 index stages in the backend, perform all conflict region parsing and line-level merge logic on the frontend in TypeScript, and keep the Output panel as a plain `<textarea>` for the editable mode.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Merge editor **replaces DiffPanel** in the same right-pane area -- clicking a conflicted file swaps the right pane content, consistent with existing DiffPanel pattern
- Three panels: **Current (ours) and Incoming (theirs) side-by-side on top**, **Output (editable) spanning full width on bottom**
- **50/50 vertical split** between top row and output, **50/50 horizontal split** between current and incoming
- **Fixed splits** -- no draggable dividers, no resizing
- **Colored header bars** on each panel: Current (ours) with blue-ish tint, Incoming (theirs) with green-ish tint, Output with neutral tint
- **Synchronized scroll** across all three panels (GitKraken-style)
- Conflict regions use **standard diff-style background coloring** (green for additions, red for deletions) -- same colors as normal diffs, not panel-specific colors
- **Non-conflict regions shown in both** current and incoming panels as neutral context -- keeps line numbers aligned and provides surrounding context
- **Line numbers displayed in all three panels** in a gutter on the left
- **Taken hunks at full brightness**, untaken hunks **dimmed to ~50% opacity**
- Three levels of selection granularity: **whole file**, **per-hunk**, and **per-line**
- **Whole file**: "Take All Current" / "Take All Incoming" buttons in panel headers
- **Per-hunk**: Each conflict section has a **clickable hunk header row** (separator line). Clicking toggles all lines in that hunk. Shows green check icon when all taken, empty when none
- **Per-line**: Click any line within a conflict region to toggle it. **Green check icon in gutter** when line is taken. **Hover over a taken line** switches icon to a red remove icon. Click again to remove from output
- **Output updates in real-time** as selections change
- **Toolbar split between panel headers**: Take All Current in Current panel header, Take All Incoming in Incoming panel header, Prev/Next conflict arrows and Save and Mark Resolved in Output panel header
- **Save and Mark Resolved** saves output to disk, stages the file, then **returns to the staging panel** (does not auto-open next file)
- **Right-click on conflicted file in staging panel** shows "Take All Current" and "Take All Incoming" as context menu items -- quick resolution without opening the editor
- **Prev/Next conflict navigation stops at boundaries** -- Prev disabled at first conflict, Next disabled at last. Does not wrap around

### Claude's Discretion
- Exact CSS custom property names and color values for panel header tints
- How to extract ours/theirs/base versions from git (git2 merge_file API vs index stages vs conflict marker parsing)
- Synchronized scroll implementation approach
- Output panel editing implementation (textarea, contenteditable, etc.)
- Conflict hunk header row design (separator style, label text)
- Keyboard shortcuts for navigation and selection
- Loading/error states

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CONF-02 | Three-panel merge editor opens when user clicks a conflicted file (current/incoming on top, editable output on bottom) | Backend: git2 Index::conflict_get() extracts ours/theirs/base. Frontend: MergeEditor.svelte with CSS grid layout. App.svelte routing via selectedFile.kind === 'conflicted' |
| CONF-03 | Merge editor panels scroll in sync across all three panels | Synchronized scroll via shared scrollTop state + onscroll handlers with guard flag to prevent feedback loops |
| CONF-04 | Per-hunk checkboxes on current and incoming panels add/remove hunk content to/from the output | Frontend conflict region parser identifies hunk boundaries; selection state per-hunk drives real-time output recomputation |
| CONF-05 | Per-line click selection on current and incoming panels toggles individual lines into/out of the output | Per-line toggle state stored in Set<string> keyed by side+hunkIdx+lineIdx; click handler toggles, output recomputes |
| CONF-06 | Output panel is directly editable as a text editor for manual merge adjustments | Plain textarea with monospace font; switching to manual edit mode disables auto-recompute; edit state tracked |
| CONF-07 | "Take All Current" and "Take All Incoming" buttons resolve entire file with one click (toolbar + right-click) | Buttons in panel headers; right-click context menu items on conflicted files in StagingPanel |
| CONF-08 | Prev/Next conflict navigation arrows jump between conflict sections within a file | Conflict region index array; focusedConflictIdx state; scroll-into-view on navigation; disable at boundaries |
| CONF-09 | "Save and Mark Resolved" saves the output, stages the file, and returns to staging panel | Backend save_merge_result command writes file content + stages via git2; frontend calls then clears merge editor state |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 (Rust) | 0.19 | Extract conflict entries (ours/theirs/base blobs) from index stages | Already in use; Index::conflict_get() + find_blob() is the canonical way to get conflict content |
| Svelte | 5.x | MergeEditor component with reactive state | Already in use; $state/$derived runes for real-time output computation |
| @lucide/svelte | 0.577.0 | Icons (Check, CircleCheck, CircleX, ChevronUp, ChevronDown) | Already in use throughout the project |
| Tauri IPC | 2.x | safeInvoke wrapper for backend commands | Established pattern; all backend calls use this |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/api/menu | 2.x | Context menu for right-click Take All actions | Already used in StagingPanel for file context menus |
| @tauri-apps/plugin-dialog | 2.x | Confirmation dialogs if needed | Already used for discard confirmations |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| textarea for output | contenteditable div | contenteditable is fragile with line-by-line rendering; textarea is simpler, works with monospace text, and handles keyboard input natively |
| git2 index stages | Parsing conflict markers from workdir file | Marker parsing is fragile (nested conflicts, custom markers); index stages are authoritative and clean |
| git2 merge_file() | Manual three-way diff | merge_file() would produce a merged result but we need per-hunk granularity; the index stage approach gives raw ours/theirs/base for client-side merge |

## Architecture Patterns

### Recommended Project Structure
```
src/
  components/
    MergeEditor.svelte      # Main merge editor component (replaces DiffPanel for conflicted files)
  lib/
    merge-parser.ts          # Parse ours/theirs/base into conflict regions and context
    types.ts                 # New MergeSides, ConflictRegion, MergeLine types
src-tauri/
  src/
    commands/
      merge_editor.rs        # get_merge_sides + save_merge_result commands
    lib.rs                   # Register new commands
```

### Pattern 1: Backend - Extract Ours/Theirs/Base via Index Stages
**What:** Use git2's Index conflict API to extract the three versions of a conflicted file as raw string content.
**When to use:** Every time a conflicted file is opened in the merge editor.
**Example:**
```rust
// Source: git2 Index::conflict_get() + Repository::find_blob()
pub fn get_merge_sides_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<MergeSides, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let index = repo.index()?;
    let conflict = index.conflict_get(file_path)
        .map_err(|e| TrunkError::new("conflict_error", e.to_string()))?;

    let read_blob = |entry: Option<git2::IndexEntry>| -> Result<String, TrunkError> {
        match entry {
            Some(e) => {
                let blob = repo.find_blob(e.id)?;
                Ok(String::from_utf8_lossy(blob.content()).into_owned())
            }
            None => Ok(String::new()), // File didn't exist in this version
        }
    };

    Ok(MergeSides {
        base: read_blob(conflict.ancestor)?,
        ours: read_blob(conflict.our)?,
        theirs: read_blob(conflict.their)?,
    })
}
```

### Pattern 2: Frontend - Conflict Region Parsing
**What:** Parse ours/theirs/base text into structured conflict regions (conflict hunks + shared context) for rendering in the three-panel UI.
**When to use:** After receiving MergeSides from backend, before rendering.
**Approach:** Use a line-by-line diff algorithm (longest common subsequence) to identify conflict regions between ours and theirs relative to base. Each region is either:
- **Context:** Lines identical in ours, theirs, and base -- shown in all panels at full brightness
- **Conflict:** Lines that differ between ours and theirs -- shown with diff coloring, toggleable

```typescript
// Conceptual structure
interface ConflictRegion {
  type: 'context' | 'conflict';
  baseLines: string[];       // Lines from base (ancestor)
  oursLines: string[];       // Lines from current (ours)
  theirsLines: string[];     // Lines from incoming (theirs)
  oursStartLine: number;     // Line number offset in ours
  theirsStartLine: number;   // Line number offset in theirs
}

interface MergeState {
  regions: ConflictRegion[];
  // For each conflict region, track which lines are "taken"
  // Key: `ours-${regionIdx}-${lineIdx}` or `theirs-${regionIdx}-${lineIdx}`
  takenLines: Set<string>;
  manualEdit: boolean;       // True if user has typed in output
  outputText: string;        // Current output content
}
```

### Pattern 3: Real-Time Output Computation
**What:** Recompute the merged output whenever selection state changes.
**When to use:** Every time a line or hunk toggle fires.
**Approach:** Walk the regions array. For context regions, include all lines. For conflict regions, include taken lines from ours and theirs in the order they appear. Use Svelte's `$derived` to make this reactive.

```typescript
let outputText = $derived.by(() => {
  if (mergeState.manualEdit) return mergeState.outputText;
  const lines: string[] = [];
  for (const region of mergeState.regions) {
    if (region.type === 'context') {
      lines.push(...region.baseLines);
    } else {
      // Include taken ours lines first, then taken theirs lines
      region.oursLines.forEach((line, i) => {
        if (mergeState.takenLines.has(`ours-${regionIdx}-${i}`)) lines.push(line);
      });
      region.theirsLines.forEach((line, i) => {
        if (mergeState.takenLines.has(`theirs-${regionIdx}-${i}`)) lines.push(line);
      });
    }
  }
  return lines.join('\n');
});
```

### Pattern 4: Synchronized Scroll
**What:** All three panels scroll together.
**When to use:** Any scroll event on any panel.
**Approach:** Use a guard flag to prevent feedback loops. When one panel scrolls, set scrollTop on the other two panels directly via DOM refs.

```typescript
let scrolling = false;
let panelRefs: HTMLDivElement[] = [];  // [current, incoming, output]

function handleScroll(sourceIdx: number) {
  if (scrolling) return;
  scrolling = true;
  const scrollTop = panelRefs[sourceIdx].scrollTop;
  panelRefs.forEach((el, i) => {
    if (i !== sourceIdx) el.scrollTop = scrollTop;
  });
  requestAnimationFrame(() => { scrolling = false; });
}
```

### Pattern 5: App.svelte Routing
**What:** When selectedFile.kind === 'conflicted', show MergeEditor instead of DiffPanel in the center pane.
**When to use:** Integration point in App.svelte.
**Approach:** Add conditional rendering:

```svelte
{#if selectedFile?.kind === 'conflicted'}
  <MergeEditor
    repoPath={repoPath}
    filePath={selectedFile.path}
    onclose={handleDiffClose}
    onresolved={handleFileResolved}
  />
{:else if showDiff}
  <DiffPanel ... />
{:else}
  <CommitGraph ... />
{/if}
```

### Anti-Patterns to Avoid
- **Parsing conflict markers from the working directory file:** The workdir file contains `<<<<<<<`/`=======`/`>>>>>>>` markers which are fragile to parse (nested conflicts, custom marker lengths). Use git2 index stages instead for clean separation.
- **Using contenteditable for the output panel:** contenteditable divs have cross-browser inconsistencies with whitespace, line breaks, and cursor positioning. A textarea is more reliable for plain-text editing.
- **Bidirectional scroll binding via Svelte reactivity:** Using `bind:scrollTop` on multiple panels creates infinite loops. Use imperative DOM manipulation with a guard flag instead.
- **Recomputing output on every keystroke in manual edit mode:** Once the user types in the output textarea, stop auto-recomputing from selections. Track `manualEdit` boolean.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Extract ours/theirs/base content | Custom conflict marker parser | git2 `Index::conflict_get()` + `find_blob()` | Index stages are authoritative; marker parsing is fragile with edge cases |
| File write + stage | Manual fs::write + index manipulation | Single Tauri command that does both atomically | Ensures consistency; prevents partial state |
| Context menu | Custom right-click handler | `@tauri-apps/api/menu` Menu/MenuItem | Established pattern in the project; native OS menus |
| Toast notifications | Custom notification system | `showToast()` from existing toast.svelte.ts | Already built and used throughout the app |
| Diff computation for conflict regions | Custom LCS algorithm | Simple line comparison between base/ours/theirs | For a merge editor, we don't need full diff -- we just need to identify which lines differ from base |

**Key insight:** The heaviest algorithmic work (three-way merge) is already done by git during the merge operation. Our job is just to present the three versions and let the user pick lines. We do NOT need to implement a diff algorithm -- we compare lines against base to identify conflict vs context regions.

## Common Pitfalls

### Pitfall 1: Scroll Feedback Loops
**What goes wrong:** Setting scrollTop on panel B triggers its onscroll, which sets scrollTop on panel A, creating an infinite loop.
**Why it happens:** Browser fires scroll events whenever scrollTop is set programmatically.
**How to avoid:** Use a boolean guard flag. Set it before updating other panels, clear it in the next animation frame.
**Warning signs:** Panels jitter or freeze when scrolling.

### Pitfall 2: Index Conflict Entries Disappearing After Stage
**What goes wrong:** After `git add` on a conflicted file, `conflict_get()` returns no conflict for that path.
**Why it happens:** Git clears conflict entries when a file is staged (added to index). The conflict data moves to the REUC (resolve undo) section.
**How to avoid:** Extract ours/theirs/base content when the merge editor opens, not lazily. Cache the content in component state.
**Warning signs:** Opening a file that was previously resolved shows empty panels.

### Pitfall 3: Binary Files in Conflicts
**What goes wrong:** `find_blob().content()` returns raw bytes that can't be converted to valid UTF-8.
**Why it happens:** Binary files can be conflicted too (e.g., images modified on both branches).
**How to avoid:** Check `String::from_utf8()` result. If it fails, show a "Binary file -- cannot merge in editor" message. Let user resolve via "Take All Current" or "Take All Incoming" only.
**Warning signs:** Garbled text in panels, or backend error on `from_utf8_lossy`.

### Pitfall 4: New Files in Conflict (File Added on Both Sides)
**What goes wrong:** `conflict.ancestor` is `None` because the file didn't exist in the common ancestor.
**Why it happens:** Both branches added a file with the same name but different content.
**How to avoid:** Handle `None` ancestor gracefully -- treat all lines as conflict (no shared context). The read_blob helper already handles this by returning empty string.
**Warning signs:** Crash or empty base panel when ancestor is None.

### Pitfall 5: File Deleted on One Side
**What goes wrong:** `conflict.our` or `conflict.their` is `None` because one side deleted the file.
**Why it happens:** One branch modified the file while the other deleted it.
**How to avoid:** Handle None by showing "File deleted on this side" message in that panel. Let user choose to keep (Take All from the side that has it) or accept deletion.
**Warning signs:** Empty panel with no explanation.

### Pitfall 6: Output Textarea vs Computed Output State Conflict
**What goes wrong:** User types in output, then clicks a hunk toggle, overwriting their manual edits.
**Why it happens:** The auto-computed output and manual edits share the same state.
**How to avoid:** Track `manualEdit` flag. Once user types in textarea, set it to true. Show a warning or confirmation before switching back to auto-compute mode if they click a toggle.
**Warning signs:** Lost manual edits.

### Pitfall 7: CONF-09 Says Auto-Open Next vs CONTEXT.md Says Return to Staging
**What goes wrong:** Implementing auto-open next file per REQUIREMENTS.md when CONTEXT.md explicitly says "returns to staging panel (does not auto-open next file)."
**Why it happens:** Requirements document and context document have a discrepancy.
**How to avoid:** Follow CONTEXT.md -- it represents the user's explicit decision. Save and Mark Resolved returns to the staging panel.
**Warning signs:** None if caught during planning.

## Code Examples

### Backend: MergeSides DTO (Rust)
```rust
// Source: New type for merge_editor.rs
#[derive(Debug, Serialize, Clone)]
pub struct MergeSides {
    pub base: String,    // Common ancestor content (may be empty)
    pub ours: String,    // Current branch (HEAD) content
    pub theirs: String,  // Incoming branch content
}
```

### Backend: Save and Stage Resolved File
```rust
// Source: Pattern from existing staging.rs stage_file_inner
pub fn save_merge_result_inner(
    path: &str,
    file_path: &str,
    content: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let repo_path = repo.workdir()
        .ok_or_else(|| TrunkError::new("no_workdir", "Bare repository"))?;

    // Write merged content to disk
    let full_path = repo_path.join(file_path);
    std::fs::write(&full_path, content)
        .map_err(|e| TrunkError::new("write_error", e.to_string()))?;

    // Stage the file (clears conflict entry from index)
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new(file_path))?;
    index.write()?;

    Ok(())
}
```

### Frontend: CSS Custom Properties for Panel Headers
```css
/* New variables in app.css */
--color-merge-current-header: rgba(56, 139, 253, 0.15);   /* Blue-ish tint for Current (ours) */
--color-merge-current-border: #58a6ff;
--color-merge-incoming-header: rgba(74, 222, 128, 0.15);   /* Green-ish tint for Incoming (theirs) */
--color-merge-incoming-border: #4ade80;
--color-merge-output-header: rgba(139, 148, 158, 0.1);     /* Neutral tint for Output */
--color-merge-output-border: #8b949e;
--color-merge-taken-check: #4ade80;                          /* Green check icon */
--color-merge-remove-icon: #f87171;                          /* Red remove icon on hover */
--color-merge-dimmed: 0.5;                                   /* Opacity for untaken hunks */
```

### Frontend: MergeEditor Component Structure
```svelte
<!-- Conceptual layout structure -->
<div class="merge-editor" style="height: 100%; display: flex; flex-direction: column;">
  <!-- Top row: Current + Incoming side by side (50% height) -->
  <div style="flex: 1; display: flex; min-height: 0;">
    <!-- Current (Ours) Panel -->
    <div style="flex: 1; display: flex; flex-direction: column; min-width: 0;">
      <div class="panel-header" style="background: var(--color-merge-current-header);">
        Current (Ours)
        <button>Take All Current</button>
      </div>
      <div class="panel-content" bind:this={panelRefs[0]} onscroll={() => handleScroll(0)}>
        <!-- Line-by-line rendering with gutter + check icons -->
      </div>
    </div>
    <!-- Incoming (Theirs) Panel -->
    <div style="flex: 1; display: flex; flex-direction: column; min-width: 0;">
      <div class="panel-header" style="background: var(--color-merge-incoming-header);">
        Incoming (Theirs)
        <button>Take All Incoming</button>
      </div>
      <div class="panel-content" bind:this={panelRefs[1]} onscroll={() => handleScroll(1)}>
        <!-- Line-by-line rendering with gutter + check icons -->
      </div>
    </div>
  </div>
  <!-- Bottom: Output panel (50% height) -->
  <div style="flex: 1; display: flex; flex-direction: column; min-height: 0;">
    <div class="panel-header" style="background: var(--color-merge-output-header);">
      Output
      <button>Prev</button> <button>Next</button>
      <button>Save and Mark Resolved</button>
    </div>
    <textarea bind:value={outputText} style="flex: 1; font-family: var(--font-mono);" />
  </div>
</div>
```

### Frontend: Conflict Region Identification (Simplified)
```typescript
// Compare lines of ours and theirs against base to identify conflict regions
function parseConflictRegions(base: string, ours: string, theirs: string): ConflictRegion[] {
  const baseLines = base.split('\n');
  const oursLines = ours.split('\n');
  const theirsLines = theirs.split('\n');

  // Strategy: Walk through lines, group consecutive lines that are
  // identical across all three as 'context', and lines that differ as 'conflict'.
  // Use a simple LCS-based approach or even direct comparison when base exists.
  // When base is empty (new file on both sides), treat entire file as one conflict.

  // ... implementation details in merge-parser.ts
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Parse conflict markers from workdir | Extract clean ours/theirs/base from git index stages | Always available in git2 | Cleaner data, no parsing edge cases |
| Two-panel merge (ours vs theirs) | Three-panel with editable output (VS Code, GitKraken pattern) | Industry standard ~2020+ | Better UX for complex conflicts |
| contenteditable for rich editing | textarea for plain-text merge output | Ongoing best practice | Simpler, more reliable for plain text |

**Deprecated/outdated:**
- git2 `merge_file()` for producing auto-merged output: Not needed here because we want granular per-line control, not auto-merge
- Svelte `bind:scrollTop`: Works for single elements but causes feedback loops with multiple synced panels; use imperative DOM manipulation instead

## Open Questions

1. **Diff algorithm for conflict region detection**
   - What we know: We need to identify which lines are conflicts vs context by comparing ours/theirs against base
   - What's unclear: Whether a simple line-by-line comparison suffices or if we need a proper LCS/Myers diff algorithm for alignment
   - Recommendation: Start with simple sequential comparison (walk base, ours, theirs simultaneously). If line alignment breaks (insertions/deletions shift line numbers), fall back to treating entire differing sections as conflict regions. A full diff algorithm may be needed but can be added iteratively.

2. **Textarea scrollTop synchronization with rendered panels**
   - What we know: The top two panels render lines as individual divs; the bottom textarea is a single element
   - What's unclear: Whether scrollTop values will match between div-based rendering and textarea rendering when line heights differ
   - Recommendation: Use consistent line-height (e.g., `line-height: 1.5` at `font-size: 12px` = 18px per line) in all three panels. For the textarea, set the same font-size and line-height. If pixel-perfect sync is hard, sync scroll as a percentage of total scrollHeight.

3. **Large file performance**
   - What we know: Rendering thousands of lines in three panels simultaneously could be slow
   - What's unclear: At what file size does performance degrade
   - Recommendation: For v1, render all lines without virtualization. Most conflicts occur in reasonably-sized files. Add virtualization later if needed.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust: `cargo test` (built-in), TypeScript: vitest 4.1.0 |
| Config file | vite.config.ts (test section), Cargo.toml |
| Quick run command | `cd src-tauri && cargo test merge_editor -- --nocapture` |
| Full suite command | `cd src-tauri && cargo test && cd .. && npx vitest run` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONF-02 | get_merge_sides returns ours/theirs/base content | unit (Rust) | `cd src-tauri && cargo test merge_editor -x` | Wave 0 |
| CONF-02 | MergeSides DTO serializes correctly | unit (Rust) | `cd src-tauri && cargo test merge_editor -x` | Wave 0 |
| CONF-03 | Synchronized scroll (DOM interaction) | manual-only | N/A -- requires browser DOM | N/A |
| CONF-04 | Per-hunk toggle updates output (merge-parser logic) | unit (TS) | `npx vitest run src/lib/merge-parser.test.ts` | Wave 0 |
| CONF-05 | Per-line toggle updates output (merge-parser logic) | unit (TS) | `npx vitest run src/lib/merge-parser.test.ts` | Wave 0 |
| CONF-06 | Output textarea editable (DOM interaction) | manual-only | N/A -- requires browser DOM | N/A |
| CONF-07 | Take All Current/Incoming selects all lines from one side | unit (TS) | `npx vitest run src/lib/merge-parser.test.ts` | Wave 0 |
| CONF-08 | Conflict navigation index tracking | unit (TS) | `npx vitest run src/lib/merge-parser.test.ts` | Wave 0 |
| CONF-09 | save_merge_result writes file and stages | unit (Rust) | `cd src-tauri && cargo test merge_editor -x` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test merge_editor -- --nocapture`
- **Per wave merge:** `cd src-tauri && cargo test && cd .. && npx vitest run`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/commands/merge_editor.rs` -- new module with tests for get_merge_sides and save_merge_result
- [ ] `src/lib/merge-parser.ts` -- conflict region parsing + output computation logic
- [ ] `src/lib/merge-parser.test.ts` -- unit tests for parsing and selection logic

## Sources

### Primary (HIGH confidence)
- git2-rs source code (Index::conflict_get, IndexConflict struct) -- https://github.com/rust-lang/git2-rs/blob/master/src/index.rs
- git2-rs merge.rs (MergeFileInput, merge_file) -- https://github.com/rust-lang/git2-rs/blob/master/src/merge.rs
- Existing codebase: DiffPanel.svelte, StagingPanel.svelte, App.svelte, diff.rs, staging.rs, operation_state.rs, types.ts, types.rs -- direct code reading
- git2 Index documentation -- https://docs.rs/git2/latest/git2/struct.Index.html
- Lucide Svelte icons -- https://lucide.dev/guide/packages/lucide-svelte

### Secondary (MEDIUM confidence)
- git2 Blob::content() API -- https://docs.rs/git2/latest/git2/struct.Blob.html (confirmed pattern: find_blob(oid) then .content() returns &[u8])
- Svelte scroll binding patterns -- https://svelte.dev/repl/97366319dc7c477989fe744d01d81391 (use imperative DOM, not bind:scrollTop for sync)

### Tertiary (LOW confidence)
- None -- all critical patterns verified against source code or official documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in use in the project; git2 Index conflict API verified against source
- Architecture: HIGH -- follows established project patterns (inner_fn + Tauri command wrapper, safeInvoke, CSS custom properties, component structure)
- Pitfalls: HIGH -- scroll feedback loops and index stage behavior verified against git2 documentation; other pitfalls from direct code analysis

**Research date:** 2026-03-20
**Valid until:** 2026-04-20 (stable domain; git2 0.19 and Svelte 5 are mature)
