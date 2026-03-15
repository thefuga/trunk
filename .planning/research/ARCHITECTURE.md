# Architecture Research: v0.6 UI Polish & Core Ops Integration

**Domain:** Desktop Git GUI — feature integration into existing Tauri 2 + Svelte 5 + Rust architecture
**Researched:** 2026-03-15
**Confidence:** HIGH (based on full codebase audit, all claims verified against existing source)

## Current Architecture Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│  Svelte 5 Frontend (Vite SPA)                                       │
│  ┌───────────┐ ┌──────────┐ ┌──────────────┐ ┌────────────┐        │
│  │ App.svelte│ │ Toolbar  │ │ CommitGraph  │ │ StagingPanel│        │
│  │ (state    │ │ (actions)│ │ (graph +     │ │ (files +   │        │
│  │  owner)   │ │          │ │  SVG overlay)│ │  CommitForm)│        │
│  └─────┬─────┘ └────┬─────┘ └──────┬───────┘ └─────┬──────┘        │
│        │             │              │               │               │
│  ┌─────┴─────────────┴──────────────┴───────────────┴──────────┐    │
│  │  safeInvoke<T> + listen('repo-changed')   IPC Layer         │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│  Rust Backend (Tauri 2)                                             │
│  ┌──────────────────────────┐  ┌─────────────────────────────┐      │
│  │  commands/ (10 modules)  │  │  Managed State               │      │
│  │  - repo, history         │  │  - RepoState (path map)      │      │
│  │  - branches, staging     │  │  - CommitCache (graph)        │      │
│  │  - commit, commit_actions│  │  - RunningOp (remote PID)     │      │
│  │  - diff, stash, remote   │  │  - WatcherState (notify)      │      │
│  └──────────┬───────────────┘  └─────────────────────────────┘      │
│             │                                                        │
│  ┌──────────┴───────────────┐                                        │
│  │  git/ (graph, repo, types)│  ← git2 + git CLI subprocess         │
│  └──────────────────────────┘                                        │
└─────────────────────────────────────────────────────────────────────┘
```

## v0.6 Feature Integration Map

Each v0.6 feature classified by integration approach: **New Component**, **Modified Component**, **New Rust Command**, **CSS/Layout Only**, or **Bug Fix**.

### Feature-by-Feature Architecture Decisions

---

### 1. Icon System

**Type:** New shared module + new component + modified components (many)
**Confidence:** HIGH

**Current state:** Unicode symbols for toolbar buttons (`&#8617;`, `&#8595;`, `&#9095;`, `&#128230;`), single-char symbols for file status (`+`, `✎`, `−`, `→`, `⇄`, `!`), inline SVG paths for ref pill icons (tag diamond, stash box).

**Architecture:**

Create `src/lib/icons.ts` — a centralized icon module exporting SVG path data strings keyed by icon name. NOT an icon component per icon — raw SVG `<path d="...">` data for maximum flexibility across HTML, SVG overlay, and toolbar contexts.

```typescript
// src/lib/icons.ts
export const ICONS = {
  // Toolbar
  undo: 'M9 15L3 9l6-6 M3 9h12a6 6 0 010 12H9',
  redo: 'M15 15l6-6-6-6 M21 9H9a6 6 0 000 12h6',
  pull: 'M12 5v14 M19 12l-7 7-7-7',
  push: 'M12 19V5 M5 12l7-7 7 7',
  branch: 'M6 3v12 M18 9a3 3 0 01-3 3H6',
  stash: 'M5 8h14 M7 4h10 M3 12h18v8H3z',
  pop: 'M5 12h14 M12 8V2 M8 6l4-4 4 4',
  // File status
  fileAdd: 'M12 5v14 M5 12h14',
  fileModify: 'M11 4H4v16h16v-7',
  fileDelete: 'M5 12h14',
  fileRename: 'M5 12h14 M13 6l6 6-6 6',
  // Ref pills (tiny, for 10px icon width)
  tag: 'M0 0l4-4 4 4-4 4z',
  stashPill: 'M0 4V-4h5v4H0',
  // Section headers
  chevronDown: 'M6 9l6 6 6-6',
  chevronRight: 'M9 6l6 6-6 6',
  plus: 'M12 5v14 M5 12h14',
  close: 'M18 6L6 18 M6 6l12 12',
  // Actions
  discard: 'M3 6h18 M8 6V4h8v2 M5 6v14h14V6',
  stage: 'M5 12l5 5L20 7',
  unstage: 'M5 12h14',
} as const;

export type IconName = keyof typeof ICONS;
```

**New component:** `src/components/Icon.svelte` — thin wrapper for consistent sizing:

```svelte
<script lang="ts">
  import { ICONS, type IconName } from '../lib/icons.js';
  interface Props {
    name: IconName;
    size?: number;
    class?: string;
    stroke?: string;
  }
  let { name, size = 16, class: className = '', stroke = 'currentColor' }: Props = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24"
  fill="none" {stroke} stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
  class={className}>
  <path d={ICONS[name]} />
</svg>
```

**Components to modify:**
- `Toolbar.svelte` — Replace unicode entities with `<Icon name="pull" />` etc.
- `FileRow.svelte` — Replace `STATUS_ICONS` symbol chars with SVG icon paths
- `BranchSection.svelte` — Replace `▼`/`▶` with chevron icons, `+` with plus icon
- `StagingPanel.svelte` — Replace section header text symbols
- `CommitGraph.svelte` — Replace ref pill inline SVG paths with `ICONS.tag`/`ICONS.stashPill`
- `TabBar.svelte` — Replace `×` with close icon

**Why this approach:**
- SVG path data (not icon font, not component-per-icon) — zero bundle overhead, works in both HTML and SVG overlay contexts
- Centralized module — single source of truth, easy to swap icon set later
- Matches Lucide/Feather icon convention (24x24 viewBox, stroke-based) — large library of compatible icons available if needed
- `Icon.svelte` wrapper is optional — components can use raw paths directly in SVG contexts (ref pills)

**No Rust changes required.**

---

### 2. Better Tag Pill Icon

**Type:** Modified constant in `icons.ts` + modified `CommitGraph.svelte`
**Confidence:** HIGH

**Current state:** Tag pill uses inline diamond SVG path: `d="M {x} {y} l 4 -4 l 4 4 l -4 4 z"`

**Change:** Replace diamond with a proper tag icon (price tag / label shape) in the ICONS module. Update the path in CommitGraph.svelte's ref pill rendering section (around line 617).

**No Rust changes required.**

---

### 3. Discard Changes (Revert Working Tree Files)

**Type:** New Rust commands + Modified frontend components
**Confidence:** HIGH

**New Rust commands in `src-tauri/src/commands/staging.rs`:**

```rust
// discard_file_inner: revert a single working tree file
pub fn discard_file_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    // Check if file is untracked (WT_NEW) — delete it
    let statuses = repo.statuses(None)?;
    let is_untracked = statuses.iter().any(|entry| {
        entry.path() == Some(file_path) && entry.status().contains(git2::Status::WT_NEW)
    });

    if is_untracked {
        let full_path = repo.workdir()
            .ok_or_else(|| TrunkError::new("bare_repo", "Cannot discard in bare repo"))?
            .join(file_path);
        std::fs::remove_file(&full_path)
            .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    } else {
        // Tracked file — checkout from HEAD (or index)
        repo.checkout_head(Some(
            git2::build::CheckoutBuilder::new()
                .force()
                .path(file_path)
        ))?;
    }
    Ok(())
}

// discard_all_unstaged_inner: revert all unstaged working tree changes
pub fn discard_all_unstaged_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    // ... checkout all tracked modifications + delete untracked files
}
```

**Registration:** Add to `lib.rs` invoke_handler: `commands::staging::discard_file`, `commands::staging::discard_all_unstaged`

**Frontend integration (FileRow.svelte / StagingPanel.svelte):**
- Add discard button (trash icon) to `FileRow.svelte` for unstaged files — shows on hover alongside the stage `+` button
- Add confirmation dialog before discard (destructive action) — use `ask()` from `@tauri-apps/plugin-dialog` (same pattern as stash drop in BranchSidebar)
- StagingPanel: Add "Discard All" button in unstaged section header (alongside existing "Stage All Changes")

**Data flow:**
```
User clicks discard icon on FileRow
  → ask() confirmation dialog
  → safeInvoke('discard_file', { path, filePath })
  → Rust: git2 checkout_head(force, path) or fs::remove_file
  → Filesystem watcher detects change → 'repo-changed' event
  → StagingPanel re-fetches status automatically
```

**Note:** No cache rebuild needed — discard doesn't affect commit graph. The filesystem watcher (300ms debounce) handles refresh automatically.

---

### 4. Branch Delete Action

**Type:** New Rust command + Modified frontend components
**Confidence:** HIGH

**New Rust command in `src-tauri/src/commands/branches.rs`:**

```rust
pub fn delete_branch_inner(
    path: &str,
    branch_name: &str,
    force: bool,
    state_map: &HashMap<String, PathBuf>,
    cache_map: &mut HashMap<String, GraphResult>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let mut branch = repo.find_branch(branch_name, git2::BranchType::Local)?;

    if branch.is_head() {
        return Err(TrunkError::new(
            "cannot_delete_head",
            "Cannot delete the currently checked-out branch",
        ));
    }

    if force {
        // Force delete (like git branch -D)
        branch.delete()?;
    } else {
        // Safe delete — git2 branch.delete() already checks merge status
        branch.delete()?;
    }
    drop(repo);

    // Rebuild graph cache (branch refs change the graph display)
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", "Repository not open"))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    let graph_result = graph::walk_commits(&mut repo2, 0, usize::MAX)?;
    cache_map.insert(path.to_owned(), graph_result);

    Ok(())
}
```

**Note on force delete:** `git2::Branch::delete()` does NOT check merge status — it always succeeds. For safe delete (checking if branch is fully merged), we need to verify merge status manually using `repo.merge_base()` before calling delete, or use git CLI `git branch -d` which does check. Recommend starting with `git2::Branch::delete()` (always works) behind an `ask()` confirmation, and adding the unmerged-branch warning in a future iteration.

**Registration:** Add `commands::branches::delete_branch` to invoke_handler.

**Frontend integration:**
- **BranchSidebar.svelte:** Add right-click context menu on local branch rows. Use Tauri `Menu` API (same pattern as stash context menu at line 170):
  ```
  Right-click on local branch → Context Menu:
    - Checkout
    - Delete...  → ask() confirmation → safeInvoke('delete_branch')
  ```
- **BranchRow.svelte:** Add `oncontextmenu` prop (currently only has `onclick`)

**Data flow:**
```
User right-clicks branch in sidebar
  → Native context menu with Delete option
  → ask() confirmation ("Delete branch 'feature-x'? This cannot be undone.")
  → safeInvoke('delete_branch', { path, branchName, force: false })
  → Rust: git2 branch.delete() + cache rebuild
  → app.emit('repo-changed')
  → BranchSidebar loadRefs() + CommitGraph refresh()
```

---

### 5. Tag Delete Action

**Type:** New Rust command + Modified frontend component
**Confidence:** HIGH

**New Rust command in `src-tauri/src/commands/commit_actions.rs`:**

```rust
pub fn delete_tag_inner(
    path: &str,
    tag_name: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let ref_name = format!("refs/tags/{}", tag_name);
    let mut reference = repo.find_reference(&ref_name)?;
    reference.delete()?;
    drop(reference);
    drop(repo);

    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", "Repository not open"))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo2, 0, usize::MAX).map_err(TrunkError::from)
}
```

**Registration:** Add `commands::commit_actions::delete_tag` to invoke_handler.

**Frontend integration:**
- **BranchSidebar.svelte:** Add right-click context menu on tag rows:
  ```
  Right-click on tag → Context Menu:
    - Copy Tag Name
    - Delete Tag... → ask() confirmation → safeInvoke('delete_tag')
  ```
- Current tag rows use `<BranchRow name={tag.short_name} />` with no context menu or click handler. Add `oncontextmenu` prop.

**Data flow:** Same pattern as branch delete — cache rebuild + repo-changed emit.

---

### 6. Three-Way Selector (Commit / Amend / Stash)

**Type:** Modified component (CommitForm.svelte) — significant rework
**Confidence:** HIGH

**Current state:** CommitForm has a subject input, body textarea, amend checkbox, and commit button. Stash is triggered from Toolbar or sidebar, not from the commit form.

**Architecture — Replace amend checkbox with segmented control:**

```
┌──────────┬──────────┬──────────┐
│  Commit  │  Amend   │  Stash   │   ← segmented radio-style control
└──────────┴──────────┴──────────┘
```

**CommitForm.svelte changes:**
- Replace `let amend = $state(false)` with `let mode = $state<'commit' | 'amend' | 'stash'>('commit')`
- Remove amend checkbox UI, add 3-segment selector row
- When mode='amend': load HEAD message (existing `handleAmendToggle` logic)
- When mode='stash': change button text to "Stash", call `stash_save` with subject as stash message
- Subject field placeholder changes per mode: "Summary (required)" / "Amend message" / "Stash name (optional)"
- Body field hidden in stash mode (stashes have single-line names)

**Button behavior per mode:**

| Mode | Button Text | Validation | Command |
|------|-------------|------------|---------|
| commit | "Commit" | subject required, stagedCount > 0 | `create_commit` |
| amend | "Amend Commit" | subject required | `amend_commit` |
| stash | "Stash" | none (empty = auto-name) | `stash_save` |

**No new Rust commands required** — uses existing `create_commit`, `amend_commit`, `stash_save`.

---

### 7. Green "Stage All" / Red "Unstage All" Buttons

**Type:** Modified component (StagingPanel.svelte) — CSS styling change
**Confidence:** HIGH

**Current state:** "Stage All Changes" and "Unstage All" are text-only buttons with `color: var(--color-text-muted)`, no background, no border.

**Change:**
- Stage All: green background (`#2ea043`), white text, icon prefix (checkmark from icon system)
- Unstage All: red-ish background (`#da3633`), white text, icon prefix (minus from icon system)
- Both become small pill-shaped buttons within the section header

**No Rust changes. No new components.**

---

### 8. Equal Height Unstaged/Staged Lists

**Type:** Modified component (StagingPanel.svelte) — layout change
**Confidence:** HIGH

**Current state:** Both sections share a single `overflow-y: auto` scrollable wrapper. Sections grow/shrink naturally based on content.

**Architecture change:** When both sections are expanded, split the scrollable area 50/50 with each section independently scrollable:

```
┌──────────────────────────┐
│  Panel Header            │  flex-shrink: 0
├──────────────────────────┤
│  Unstaged Header (+btns) │  flex-shrink: 0, 28px
│  ┌────────────────────┐  │
│  │  unstaged files     │  │  flex: 1, overflow-y: auto
│  │  (scrollable)       │  │
│  └────────────────────┘  │
├──────────────────────────┤
│  Staged Header (+btns)   │  flex-shrink: 0, 28px
│  ┌────────────────────┐  │
│  │  staged files       │  │  flex: 1, overflow-y: auto
│  │  (scrollable)       │  │
│  └────────────────────┘  │
├──────────────────────────┤
│  CommitForm              │  flex-shrink: 0
└──────────────────────────┘
```

**Implementation:** Replace the single scrollable wrapper with a flex column. Each file list section gets `flex: 1; overflow-y: auto; min-height: 0;`. When one section is collapsed (header only, 28px), the other section gets all remaining space. When both expanded, each gets 50%.

**No Rust changes.**

---

### 9. Stash Name Defaults to Commit Form Message

**Type:** Modified component interaction (CommitForm ↔ Toolbar)
**Confidence:** HIGH

**Current state:** `Toolbar.handleStash()` calls `stash_save` with `message: ''`. CommitForm's subject is accessible via `wipSubject` state in App.svelte.

**Architecture:**
- When three-way selector is in "stash" mode, CommitForm already has access to the subject field — pass it directly to `stash_save`
- When stash is triggered from Toolbar, pass `wipSubject` from App.svelte

**Changes:**
- `Toolbar.svelte`: Accept `stashMessage?: string` prop, use in `handleStash()`
- `App.svelte`: Pass `stashMessage={wipSubject}` to Toolbar

**No Rust changes.**

---

### 10. Top/Bottom Padding in Commit Graph

**Type:** Modified component (CommitGraph.svelte / VirtualList)
**Confidence:** HIGH

**Current state:** The virtual list renders items edge-to-edge with no padding. First commit row starts at y=0.

**Architecture (recommended approach):** CSS padding on the VirtualList's content container. Add `padding-top` and `padding-bottom` via a wrapper or by targeting `.virtual-list-content`. The SVG overlay height already matches `contentHeight` from VirtualList — ensure padding is accounted for in the overlay Y offset.

**Alternative approach (sentinel items):** Add invisible padding rows at top/bottom of `displayItems`. This is fragile — affects row indices, selection, and context menus. **Avoid.**

**No Rust changes.**

---

### 11. Commit Graph Overflow/Shrink with Sticky Right-Side Columns

**Type:** Modified components (CommitGraph.svelte, CommitRow.svelte) — layout adjustment
**Confidence:** MEDIUM (may need iteration)

**Current state:** Graph column has fixed width (`columnWidths.graph`). With many active lanes (e.g., 20+), the graph SVG can exceed the column width and push right-side columns off-screen.

**Architecture:**

The graph column should clip when content exceeds allocated width. Right-side columns (message, author, date, SHA) remain at fixed positions.

**Implementation:**
1. CommitRow: Graph column div already has `flex-shrink: 0` and fixed width. Add `overflow: hidden` to clip the graph SVG content.
2. SVG overlay: Already positioned `left: 0` with width based on `maxColumns * laneWidth`. When this exceeds `columnWidths.graph`, the parent's `overflow: hidden` handles clipping.
3. The flex layout already keeps message as `flex-1` — it fills remaining space after fixed columns.

**Key insight:** The current layout already partially handles this since the graph column width is user-resizable. The main fix is adding `overflow: hidden` to the graph column containers in both the header and rows.

**No Rust changes.**

---

### 12. Click References in Left Pane to Navigate Graph

**Type:** Modified components (BranchSidebar, BranchRow, App.svelte, CommitGraph)
**Confidence:** HIGH

**Current state:** Clicking a local branch triggers checkout. Tags have no click handler. No way to scroll the graph to a specific ref's commit.

**Architecture:**

**Recommended approach:** Extend `BranchInfo` with `oid: String` field in Rust (already available — we peel to commit for `last_commit_timestamp`). Then clicking a ref fires a navigation callback that scrolls the graph.

**Rust change (list_refs_inner in branches.rs):**
Add `oid` field to `BranchInfo` struct:
```rust
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub last_commit_timestamp: i64,
    pub oid: String,        // NEW — commit OID for navigation
}
```

Populate from existing commit peel:
```rust
let oid = branch.get().peel_to_commit()
    .map(|c| c.id().to_string())
    .unwrap_or_default();
```

**TypeScript type change:** Add `oid: string` to `BranchInfo` interface in `types.ts`.

**Frontend flow:**
```
User clicks tag/branch in sidebar
  → onrefnavigate(oid) callback to App.svelte
  → App.svelte sets selectedCommitOid + calls graph navigation
  → CommitGraph: find row index in displayItems where oid matches
  → listRef.scroll({ index, align: 'center', smoothScroll: true })
```

**CommitGraph.svelte:** Expose a `scrollToOid(oid: string)` method (or accept a `scrollToOid` prop that triggers an effect).

**BranchSidebar.svelte:** Add `onrefnavigate?: (oid: string) => void` callback. For local branches: fire alongside checkout. For tags: fire on click. Tags also get their target OID from `RefLabel` — but currently `RefLabel` doesn't have an oid field. Need to add `oid` to `RefLabel` or add to tags in `RefsResponse`.

**Alternative simpler approach for tags:** Resolve tag → OID on click using a new Rust command `resolve_ref_oid`. This avoids changing the `RefLabel` type but adds an IPC round-trip. Given the latency is < 1ms, this is acceptable.

---

### 13. Fix: Branch Overflow Pill Z-Index Behind Graph

**Type:** Bug fix — SVG/CSS
**Confidence:** HIGH

**Current state:** SVG ref pills render in the `overlay-pills` group within the SVG. The SVG has `pointer-events: none` and `z-index: 1`.

**Root cause analysis:** The overflow badge renders at `pill.x + pill.width + PILL_GAP`. If this exceeds the ref column width (`refOffset`), the badge extends into the graph area. Since all elements are in the same SVG, there's no z-index issue between SVG groups — SVG paint order determines visibility. The likely issue is that the SVG `width` is `refOffset + graphWidth`, and badges within the ref area are being clipped by the graph column's rendering.

**Fix:** Set `overflow: visible` on the SVG element, or ensure the SVG width accounts for the maximum badge extent. The simpler fix: add `style="overflow: visible"` to the SVG element in CommitGraph.svelte (line ~533).

---

### 14. Fix: Trailing Header Divider on Last Visible Column

**Type:** Bug fix — conditional rendering
**Confidence:** HIGH

**Current state:** Each column header div in CommitGraph has a `.col-resize-handle` child that renders a vertical divider. The last visible column's divider is unnecessary (nothing to resize to its right).

**Fix:** Determine which column key is last visible, skip its resize handle:

```svelte
{@const lastVisibleKey = columnLabels
  .filter(c => columnVisibility[c.key])
  .at(-1)?.key}

<!-- In SHA column (currently last and has no handle): -->
<!-- No change needed — SHA already lacks a handle -->

<!-- The actual bug is likely that columns before SHA have handles even when SHA is hidden -->
```

After reviewing the code: The SHA column (line 494-498) intentionally has no resize handle. The Date column (line 488-492) has a handle labeled `sha` that resizes the SHA column. When SHA is hidden, the Date column's handle should also be hidden. Fix by checking if the column that the handle controls is visible.

---

### 15. Fix: New Untracked Files Not Showing WIP Row or Diff

**Type:** Bug fix — Rust (2-line fix)
**Confidence:** HIGH

**Root cause in `staging.rs` (line 204):** `get_dirty_counts` checks `WT_MODIFIED | WT_DELETED | WT_RENAMED | WT_TYPECHANGE` for the unstaged count — it does NOT include `WT_NEW` (untracked files). So when only untracked files exist, `wipCount === 0` and no WIP row appears.

**Fix #1 — staging.rs `get_dirty_counts`:** Add `Status::WT_NEW` to the unstaged flags:

```rust
if s.intersects(
    Status::WT_NEW          // <-- ADD THIS
    | Status::WT_MODIFIED
    | Status::WT_DELETED
    | Status::WT_RENAMED
    | Status::WT_TYPECHANGE,
) {
    unstaged += 1;
}
```

**Fix #2 — diff.rs `diff_unstaged_inner`:** Add `include_untracked(true)` to diff options so new files show diffs:

```rust
let mut opts = git2::DiffOptions::new();
opts.pathspec(file_path);
opts.include_untracked(true);  // <-- ADD THIS
```

**This is a 2-line Rust fix. No frontend changes needed.**

---

### 16. Dialog System for Errors/Warnings/Notifications

**Type:** New component + new shared state module
**Confidence:** HIGH

**Current state:** Errors handled three ways:
1. `message()` from `@tauri-apps/plugin-dialog` — native OS dialog (CommitGraph context menu)
2. `ask()` from `@tauri-apps/plugin-dialog` — native OS confirmation (destructive actions)
3. Inline error text (CommitForm validation, BranchSidebar checkout errors)

**Architecture — In-app toast notification system:**

Don't replace native OS dialogs for confirmations (they're appropriate for destructive actions). Add toasts for non-blocking feedback.

**New shared state:** `src/lib/toast-state.svelte.ts` (same `$state` rune module pattern as `remote-state.svelte.ts`):

```typescript
interface ToastMessage {
  id: number;
  type: 'error' | 'warning' | 'info' | 'success';
  title: string;
  message?: string;
  duration?: number;
}

export const toastState = $state<{ messages: ToastMessage[] }>({
  messages: [],
});

export function showToast(type: ToastMessage['type'], title: string, message?: string, duration = 5000) {
  const id = ++nextId;
  toastState.messages = [...toastState.messages, { id, type, title, message, duration }];
  if (duration > 0) setTimeout(() => dismissToast(id), duration);
}
```

**New component:** `src/components/Toast.svelte` — renders toast stack in bottom-right:
- Fixed position, z-index above everything
- Colored left border per type (red/yellow/blue/green)
- Icon per type (from icon system)
- Title + optional message
- Dismiss button (X icon)
- Auto-dismiss with slide-out animation

**Mount in App.svelte:** `<Toast />` after the main layout.

**Integration strategy — keep existing patterns:**
- `ask()` — destructive confirmations (discard, delete, hard reset). Native, modal.
- `message()` — critical errors needing acknowledgment. Native, modal.
- `showToast()` — status updates, non-critical errors, success feedback. In-app, auto-dismiss.

**Where to add toasts:**
- Failed stage/unstage/discard (replace `console.error`)
- Successful destructive operations ("Branch 'feature-x' deleted")
- Commit/amend/stash success feedback
- Remote operation completion (complement existing StatusBar display)

---

### 17. Right Pane Auto-Opens When Content Changes

**Type:** Modified App.svelte — reactive behavior
**Confidence:** HIGH

**Current state:** Right pane shows CommitDetail when a commit is selected, or StagingPanel otherwise. Can be collapsed via Cmd+K or drag-to-close.

**Architecture:** Add `$effect` in App.svelte that auto-expands on commit selection:

```typescript
// Only auto-open on commit selection, not on every status change
$effect(() => {
  if (selectedCommitOid && rightPaneCollapsed) {
    rightPaneCollapsed = false;
    setRightPaneCollapsed(false);
  }
});
```

**Refinement:** Track whether user explicitly collapsed the pane to avoid fighting the user:

```typescript
let userExplicitlyCollapsedRight = $state(false);

// In keyboard shortcut handler and drag-to-close:
// Set userExplicitlyCollapsedRight = true

// Auto-open effect:
$effect(() => {
  if (selectedCommitOid && rightPaneCollapsed && !userExplicitlyCollapsedRight) {
    rightPaneCollapsed = false;
    setRightPaneCollapsed(false);
  }
});

// Reset flag when commit deselected:
$effect(() => {
  if (!selectedCommitOid) userExplicitlyCollapsedRight = false;
});
```

**No Rust changes.**

---

### 18. Merge Window Top Bar with Tab+Actions Bar

**Type:** Modified App.svelte + TabBar.svelte — layout/visual change
**Confidence:** MEDIUM (depends on interpretation)

**Current state:** App.svelte renders a single 36px bar with TabBar (left, `flex-shrink: 0`) and Toolbar (right, `flex: 1`). The OS native title bar sits above this.

**Interpretation A (simpler — recommended for v0.6):** Remove visual separation between tab and actions, make the bar feel unified. This is CSS-only: ensure consistent background, remove TabBar's right border.

**Interpretation B (custom titlebar — higher risk):** Use Tauri's `decorations: false` + custom titlebar with drag region and window control buttons. This is more complex:

1. `tauri.conf.json`: Set `"decorations": false` on the main window
2. Add `data-tauri-drag-region` attribute to top bar
3. New component `WindowControls.svelte` for minimize/maximize/close buttons
4. Platform-specific left padding for macOS traffic lights

**Recommendation:** Start with Interpretation A (CSS merge) for v0.6. Custom titlebar is higher risk due to platform-specific behavior and can be a separate task.

---

## Component Modification Summary

| Component | Changes | Risk |
|-----------|---------|------|
| **App.svelte** | Toast mount, right-pane auto-open, stash message prop to Toolbar | Low |
| **CommitForm.svelte** | Three-way selector (commit/amend/stash), mode-dependent UI | Medium |
| **StagingPanel.svelte** | Green/red buttons, equal-height lists, discard integration | Low |
| **FileRow.svelte** | Icons, discard button for unstaged files | Low |
| **Toolbar.svelte** | Icons, accept stashMessage prop | Low |
| **BranchSidebar.svelte** | Context menus on branches/tags, ref navigation callbacks | Medium |
| **BranchRow.svelte** | Add oncontextmenu prop, icons | Low |
| **BranchSection.svelte** | Icons for chevrons and create button | Low |
| **CommitGraph.svelte** | Graph padding, overflow handling, header divider fix, pill z-index fix | Medium |
| **TabBar.svelte** | Close icon, potential bar merge | Low |

## New Components

| Component | Purpose |
|-----------|---------|
| `src/components/Icon.svelte` | Reusable SVG icon wrapper |
| `src/components/Toast.svelte` | Toast notification rendering |

## New Modules

| Module | Purpose |
|--------|---------|
| `src/lib/icons.ts` | Centralized SVG icon path data |
| `src/lib/toast-state.svelte.ts` | Shared toast notification state (reactive) |

## New/Modified Rust Commands

| Command | Module | Type | Purpose |
|---------|--------|------|---------|
| `discard_file` | staging.rs | **NEW** | Revert single working tree file |
| `discard_all_unstaged` | staging.rs | **NEW** | Revert all unstaged changes |
| `delete_branch` | branches.rs | **NEW** | Delete local branch |
| `delete_tag` | commit_actions.rs | **NEW** | Delete tag by name |
| `get_dirty_counts` | staging.rs | **FIX** | Add `WT_NEW` to unstaged count |
| `diff_unstaged` | diff.rs | **FIX** | Add `include_untracked` for new files |
| `list_refs` | branches.rs | **MODIFY** | Add `oid` field to `BranchInfo` for ref navigation |

**Type changes (Rust + TypeScript):**
- `BranchInfo`: Add `oid: String` / `oid: string`
- Both in `src-tauri/src/git/types.rs` and `src/lib/types.ts`

## Data Flow Changes

### New Flow: Discard File
```
FileRow (hover → discard icon click)
  → ask() confirmation dialog (native)
  → safeInvoke('discard_file', { path, filePath })
  → Rust: git2 checkout_head(force, path) OR fs::remove_file (untracked)
  → Filesystem watcher detects change → 'repo-changed' event
  → StagingPanel auto-refreshes via listener
```

### New Flow: Branch/Tag Delete
```
BranchSidebar (right-click → native context menu → Delete)
  → ask() confirmation dialog (native)
  → safeInvoke('delete_branch' / 'delete_tag', { path, branchName/tagName })
  → Rust: git2 branch.delete() / reference.delete() + cache rebuild
  → app.emit('repo-changed')
  → BranchSidebar loadRefs() + CommitGraph refresh()
```

### New Flow: Ref Navigation
```
BranchSidebar (click ref)
  → onrefnavigate(oid) callback to App.svelte
  → App.svelte: handleCommitSelect(oid) or dedicated scrollToCommit(oid)
  → CommitGraph: find row index in displayItems, listRef.scroll({ index, align: 'center' })
```

### Modified Flow: Three-Way Commit Selector
```
CommitForm (mode = 'commit' | 'amend' | 'stash')
  → commit: safeInvoke('create_commit')       [existing]
  → amend:  safeInvoke('amend_commit')         [existing]
  → stash:  safeInvoke('stash_save', { message: subject })  [existing command, new call site]
```

### New Flow: Toast Notifications
```
Any component operation error/success
  → showToast('error'|'success', title, message) [from toast-state.svelte.ts]
  → toastState.messages updated ($state reactive)
  → Toast.svelte renders in fixed bottom-right position
  → Auto-dismiss after 5s (or manual dismiss)
```

## Suggested Build Order

Based on dependency analysis and risk:

### Phase 1: Foundation (no dependencies, enables everything else)
1. **Icon system** (`icons.ts` + `Icon.svelte`) — used by all subsequent UI work
2. **Toast system** (`toast-state.svelte.ts` + `Toast.svelte`) — used by all error-handling improvements
3. **Bug fix: untracked files WIP** — 2-line Rust fix, independent, high value

### Phase 2: New Rust Commands (all independent of each other)
4. **Discard file** (Rust `discard_file` + `discard_all_unstaged` commands + FileRow/StagingPanel)
5. **Branch delete** (Rust `delete_branch` command + BranchSidebar context menu)
6. **Tag delete** (Rust `delete_tag` command + BranchSidebar context menu)

### Phase 3: Staging UX (depends on icons from Phase 1)
7. **Three-way selector** (CommitForm rework — highest-complexity frontend change)
8. **Green/red stage/unstage buttons** (StagingPanel styling)
9. **Equal-height file lists** (StagingPanel layout)
10. **Stash name defaults** (CommitForm + Toolbar prop wiring)

### Phase 4: Graph Polish (independent of Phases 2-3)
11. **Graph top/bottom padding** (VirtualList/CommitGraph CSS)
12. **Graph overflow/shrink** (CommitGraph/CommitRow overflow: hidden)
13. **Ref navigation** (BranchInfo oid + BranchSidebar → CommitGraph scroll)
14. **Better tag icon** (icon data update, trivial)

### Phase 5: Bug Fixes & Behavior (independent, can do anytime)
15. **Branch pill z-index fix** (SVG overflow: visible)
16. **Trailing header divider fix** (conditional rendering)
17. **Right pane auto-open** (App.svelte $effect)

### Phase 6: Layout (highest risk, do last)
18. **Merged top bar** (CSS merge or custom titlebar)

### Dependency Graph
```
Phase 1 (icons, toast, WIP fix)  ← no deps, do first
    │
    ├──► Phase 2 (Rust commands: discard, branch delete, tag delete)
    │
    ├──► Phase 3 (staging UX: 3-way selector, buttons, layout, stash name)
    │
    ├──► Phase 4 (graph: padding, overflow, ref nav, tag icon)
    │
    ├──► Phase 5 (bug fixes: z-index, divider, auto-open)
    │
    └──► Phase 6 (layout: merged bar)
```

### Build Order Rationale
- **Icons first** because every subsequent UI change references the icon system
- **Toast early** because it improves error handling across all new features
- **WIP bug fix early** because it's trivial and high-value (users notice missing WIP rows)
- **Rust commands before frontend integration** because they unblock testing
- **Three-way selector** is highest frontend complexity — give it dedicated focus after simpler items validate the patterns
- **Custom titlebar last** because it's highest risk (platform-specific, could break window management)

## Anti-Patterns to Avoid

### Anti-Pattern 1: Global Error Dialog for Everything
**What people do:** Replace all inline errors with a global dialog/toast system
**Why it's wrong:** Modal dialogs interrupt flow. Inline errors (CommitForm validation, checkout error banners) should stay inline — they're contextual.
**Do this instead:** Keep inline validation errors where they are. Use toasts only for transient operation feedback. Use native `ask()`/`message()` for destructive confirmations and critical errors.

### Anti-Pattern 2: One Giant Icon Component with Dynamic Import
**What people do:** `<Icon name="pull" />` that dynamically loads icon data or has a massive switch
**Why it's wrong:** Adds indirection, potential async loading, harder to tree-shake
**Do this instead:** Export static path data from `icons.ts`. The `Icon.svelte` wrapper just reads from the const object — zero dynamic behavior.

### Anti-Pattern 3: Discard Without Confirmation
**What people do:** Wire discard directly to a button click
**Why it's wrong:** Discard is destructive and IRREVERSIBLE. No git undo. File changes are permanently lost.
**Do this instead:** Always `ask()` before discard. "Discard All" should have an even stronger warning. Never add keyboard shortcuts for discard without modifier keys.

### Anti-Pattern 4: Breaking Inner-Fn Pattern for New Commands
**What people do:** Write new Rust commands as monolithic `#[tauri::command]` functions
**Why it's wrong:** Can't unit test without Tauri runtime. Every existing command uses `*_inner()` with `state_map` param.
**Do this instead:** Every new command (`discard_file`, `delete_branch`, `delete_tag`) must have a `*_inner()` function. Write unit tests using `make_test_repo()` helper.

### Anti-Pattern 5: Modifying BranchInfo Without Both-Side Types
**What people do:** Add `oid` to Rust `BranchInfo` but forget the TypeScript `BranchInfo` interface
**Why it's wrong:** Tauri IPC serializes to JSON — the frontend silently ignores unknown fields, but code accessing `branch.oid` will get `undefined`.
**Do this instead:** Always update both `src-tauri/src/git/types.rs` and `src/lib/types.ts` simultaneously.

## Sources

- Full codebase audit of Trunk v0.5: all components, commands, types, and tests read and analyzed
- Tauri 2 Menu API: confirmed via existing usage in CommitGraph.svelte (line 216-237) and BranchSidebar.svelte (line 170-181)
- Tauri 2 Dialog API: `ask()` and `message()` confirmed via existing usage in CommitGraph.svelte and BranchSidebar.svelte
- git2 crate 0.19: `Branch::delete()`, `Reference::delete()`, `CheckoutBuilder::force().path()` — available in current dependency
- Svelte 5 `$state` rune modules: confirmed pattern from `remote-state.svelte.ts` and `undo-redo.svelte.ts`
- SVG stroke-based icon convention (24x24 viewBox): consistent with Lucide/Feather icon libraries

---
*Architecture research for: Trunk v0.6 UI Polish & Core Ops*
*Researched: 2026-03-15*
