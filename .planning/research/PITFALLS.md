# Pitfalls Research

**Domain:** UI polish, icon system, core git operations, dialog system, and layout improvements for an existing Tauri 2 + Svelte 5 + Rust desktop Git GUI
**Researched:** 2026-03-15
**Confidence:** HIGH — based on direct codebase analysis (all Svelte components, all Rust commands, git2 0.19 API docs, Tauri 2 config), v0.5 shipped patterns, and CSS/SVG behavior knowledge

---

## Context: What's Changing in v0.6

v0.5 shipped successfully: single SVG overlay with cubic bezier curves, ref pills, three-layer z-ordering, virtualized element filtering. The codebase is now ~6,038 LOC Rust, ~4,417 LOC Svelte, ~1,102 LOC TypeScript with established patterns.

v0.6 is a **polish + extension** milestone — adding features TO an existing working system rather than building new architecture. The risk profile is different from v0.5: instead of "will this architecture work?", it's "will these additions break what already works?"

Key additions:
- Icon system throughout the app (replacing Unicode symbols)
- git2 destructive operations: discard changes, branch delete, tag delete
- Three-way commit/amend/stash selector replacing amend checkbox
- Graph CSS polish: overflow/shrink, sticky columns, padding
- Dialog/notification system
- Merged window title bar + tab/actions bar
- Bug fixes (z-index, trailing divider, untracked files WIP)

---

## Critical Pitfalls

### Pitfall 1: git2 `checkout_index` for Discard Doesn't Handle Untracked Files

**What goes wrong:**
Implementing "discard changes" for a tracked modified file using `repo.checkout_index()` or `repo.checkout_head()` with `CheckoutBuilder::path()` works for modified files but **silently does nothing for untracked (new) files**. The user clicks "discard" on a new file, no error occurs, but the file remains on disk. They think the discard failed or click repeatedly.

**Why it happens:**
`git checkout -- <file>` restores a file to its HEAD state. For untracked files, there IS no HEAD state — the file never existed in any commit. `checkout_index`/`checkout_head` with a path filter simply skips untracked files without error. The git2 API:
- `CheckoutBuilder::path("file.txt")` + `repo.checkout_head()` → restores tracked files to HEAD, ignores untracked
- `CheckoutBuilder::remove_untracked(true)` → deletes untracked files, but applies to ALL untracked files if used without path scoping

To discard changes for different file types requires different code paths:
- **Modified tracked file:** `checkout_head()` with path filter — restores from HEAD
- **Deleted tracked file:** `checkout_head()` with path filter — restores from HEAD  
- **Untracked new file:** `std::fs::remove_file()` — physically delete from disk
- **Staged new file (index only):** `index.remove_path()` + `index.write()` + `std::fs::remove_file()`

**How to avoid:**
1. Branch discard logic by `FileStatusType`: Modified/Deleted use `checkout_head()` with `CheckoutBuilder::path()` and `.force()`, New uses `std::fs::remove_file()` on the workdir path.
2. For staged files being discarded, first unstage (using existing `unstage_file_inner` logic), then discard.
3. Handle the "file was both staged and has workdir changes" case: `reset_default()` to unstage + `checkout_head()` to restore workdir.
4. Always use `CheckoutBuilder::force()` not `safe()` for discard — `safe()` will refuse to overwrite workdir changes, which is the opposite of what discard means.
5. **Never** use `remove_untracked(true)` without `path()` filtering — it would delete ALL untracked files in the repo.

**Warning signs:**
- Discard on new/untracked files silently does nothing
- Tests pass for modified files but skip new-file case
- No `std::fs::remove_file` in the discard implementation

**Phase to address:**
Discard changes phase. Must have tests for: modified file, deleted file, new untracked file, staged-only new file, file with both staged and unstaged changes.

---

### Pitfall 2: Branch Delete on HEAD Branch or Branch with Upstream Tracking

**What goes wrong:**
Using `branch.delete()` in git2 fails silently or panics when:
1. **Deleting the currently checked-out branch (HEAD):** git2 returns an error, but the error message from libgit2 is generic ("Cannot delete branch 'X' as it is the current HEAD"). The UI must prevent this or handle the error cleanly.
2. **Deleting an unmerged branch without force:** `branch.delete()` DOES NOT check for unmerged commits — that's a git CLI safety check. git2's `Branch::delete()` succeeds even if the branch has unmerged work. This is MORE dangerous than the CLI, not less.
3. **Remote tracking refs remain:** Deleting a local branch does NOT delete its remote tracking branch (`refs/remotes/origin/X`). The sidebar still shows the remote ref, confusing users into thinking the delete failed.

**Why it happens:**
git2's `Branch::delete()` is a low-level ref deletion. It doesn't implement the safety checks that `git branch -d` provides (merge check) or the cleanup that `git branch -D` implies (user explicitly choosing force). The git2 API:
- `Branch::delete()` — deletes the ref. No merge check. Fails if HEAD points to it.
- `repo.find_branch(name, BranchType::Local)` — returns `Branch` object
- The `Branch` must be mutable: `branch.delete()` requires `&mut self`

**How to avoid:**
1. **Guard against HEAD deletion:** Before calling `branch.delete()`, check `branch.is_head()`. If true, return `TrunkError::new("cannot_delete_head", "Cannot delete the currently checked-out branch")`.
2. **Implement merge check for safety:** Use `repo.graph_descendant_of(head_oid, branch_tip_oid)` to check if the branch tip is reachable from HEAD. If not reachable, the branch has unmerged commits — show a confirmation dialog with "This branch has unmerged changes. Delete anyway?".
3. **Borrow checker issue:** `repo.find_branch()` returns a `Branch<'_>` borrowing `&repo`. Calling `branch.delete()` requires `&mut Branch` (which has `&mut Reference` internally). You may need to resolve the branch OID first, drop the branch, then delete the reference directly: `repo.find_reference(&format!("refs/heads/{}", name))?.delete()`.
4. Follow the existing pattern: the `create_branch_inner` function already handles borrow complications by dropping objects before re-borrowing the repo.
5. After deletion, rebuild the graph cache (existing pattern in all mutation commands).

**Warning signs:**
- Deleting HEAD branch shows raw git2 error instead of friendly message
- Unmerged branches deleted without warning
- Sidebar still shows remote tracking ref after local branch delete
- Borrow checker errors when trying to call `branch.delete()`

**Phase to address:**
Branch delete phase. Add to both sidebar context menu and commit graph context menu.

---

### Pitfall 3: Tag Delete Requires Name Without `refs/tags/` Prefix — But `tag_foreach` Returns Full Ref Name

**What goes wrong:**
`repo.tag_delete(name)` expects the short tag name (e.g., "v1.0.0"), but the existing `list_refs_inner` function stores tags with the full ref path (e.g., "refs/tags/v1.0.0") in `RefLabel.name` and the short name in `RefLabel.short_name`. If the wrong field is passed to `tag_delete`, it fails with a cryptic libgit2 error: "Reference 'refs/tags/refs/tags/v1.0.0' not found" (double-prefixed).

Additionally, git2's `tag_delete` works differently for lightweight vs annotated tags:
- **Annotated tags:** `tag_delete` deletes the ref AND the tag object
- **Lightweight tags:** `tag_delete` deletes only the ref (no tag object exists)

Both cases work with `repo.tag_delete(short_name)`, but the inconsistency can cause confusion during testing.

**Why it happens:**
The existing code in `list_refs_inner` (branches.rs line 112-126) stores:
```rust
tags.push(RefLabel {
    name,           // "refs/tags/v1.0.0"  
    short_name,     // "v1.0.0"
    // ...
});
```
The frontend passes `tag.short_name` or `tag.name` — easy to use the wrong one.

**How to avoid:**
1. Use `repo.tag_delete(&short_name)` — always strip `refs/tags/` prefix before calling.
2. The command should accept the short name from the frontend, not the full ref path.
3. Add a defensive check: `let name = name.strip_prefix("refs/tags/").unwrap_or(name);`
4. Require confirmation dialog before deletion — tags are often release markers.
5. Rebuild graph cache after deletion (existing pattern).

**Warning signs:**
- "Reference not found" errors when deleting tags
- Tag appears to delete but reappears on next refresh (wrong ref deleted)

**Phase to address:**
Tag delete phase. Simple once the name format is correct — the git2 API is straightforward.

---

### Pitfall 4: `position: sticky` Doesn't Work Inside the Virtual List's Scroll Container Architecture

**What goes wrong:**
The v0.6 feature "Commit graph overflow/shrink with sticky right-side commits" needs the right-side columns (message, author, date, SHA) to remain visible when the graph overflows its column width. The natural CSS solution is `position: sticky` on the right-side columns. But `position: sticky` **does not work** inside the virtual list's DOM structure because:

1. The virtual list uses `position: absolute` on `.virtual-list-items` with `transform: translateY()` — sticky elements inside a transformed container are confined to that container's bounds, not the scroll viewport.
2. The commit rows are children of `.virtual-list-items` which is absolutely positioned inside `.virtual-list-content`. Sticky positioning only works relative to the nearest scrolling ancestor, but the transform creates a new containing block.
3. The horizontal scroll (if any) would need to be on the row content, but the vertical scroll is on `.virtual-list-viewport`. These are different elements.

**Why it happens:**
CSS spec: "`position: sticky` is relative to its nearest scrolling ancestor and containing block." When `transform` is applied to any ancestor, it creates a new containing block, breaking sticky behavior. The virtual list's `translateY` transform is fundamental to how virtualization works — it can't be removed.

**How to avoid:**
1. **Don't use `position: sticky`.** Instead, implement the graph overflow/shrink differently:
   - Option A: Make the graph column `overflow: hidden` with a fixed width. The SVG overlay already clips to the graph column width via its `width` attribute. When the graph needs more space than the column width, it simply clips (shrink-to-fit).
   - Option B: Let the graph column have `overflow: visible` and use CSS `clip-path` or `overflow: hidden` on the containing row, with the right-side columns using `position: relative` and `z-index` to paint on top of any overflowing graph content.
   - Option C: The commit row is a flex container. Set `flex-shrink: 0` on right-side columns and let the graph column absorb overflow by shrinking. The SVG overlay already adapts to `columnWidths.graph`.
2. For "sticky right-side commits," the actual need is likely: when the graph has many lanes and the graph column is narrow, the columns to the right of the graph should NOT get pushed off-screen. This is a **flex layout** problem, not a sticky positioning problem. Use `overflow: hidden` on the graph column cell within each row.
3. The existing column layout already uses `flex-shrink: 0` on fixed-width columns and `flex-1` on the message column. The graph column just needs `overflow: hidden` to prevent its SVG from pushing siblings.

**Warning signs:**
- `position: sticky` in CSS but it has no visible effect
- Right-side columns shift when the graph has many lanes
- SVG overlay extends beyond the graph column boundary

**Phase to address:**
Graph polish phase. Test with repos that have 8+ parallel branches to verify the layout handles many lanes.

---

### Pitfall 5: Merging Window Title Bar With App UI Breaks macOS Traffic Lights and Drag Region

**What goes wrong:**
The v0.6 feature "Merge window top bar with tab+actions bar" means removing the native window title bar and integrating the tab/toolbar into the window chrome area. On macOS, this involves:
1. Setting `decorations: false` in Tauri config to remove the native title bar
2. Implementing a custom drag region for window movement
3. Positioning the macOS traffic lights (close/minimize/maximize) within the custom bar

Common failures:
- **No drag region:** Window can't be moved by dragging the top bar
- **Traffic lights disappear:** On macOS, `decorations: false` removes traffic lights entirely
- **Traffic lights overlap content:** Using `hiddenTitle` + `titleBarStyle: "overlay"` shows traffic lights but they overlap the tab bar content
- **Double-click behavior breaks:** macOS users expect double-click on title bar to maximize — custom drag regions don't implement this by default
- **Windows/Linux inconsistency:** The title bar approach differs per platform

**Why it happens:**
Tauri 2 WindowConfig has several title bar options:
- `decorations: true` (default) — native title bar, all platform behaviors work
- `decorations: false` — no title bar at all, no traffic lights, no drag
- `titleBarStyle: "overlay"` (macOS only) — native title bar overlaps content, traffic lights visible
- `titleBarStyle: "transparent"` (macOS only) — title bar area is transparent, traffic lights visible

The current config (`tauri.conf.json`) uses defaults (decorations: true, no titleBarStyle). Changing this requires understanding the platform-specific implications.

**How to avoid:**
1. **Use `titleBarStyle: "overlay"` on macOS** — not `decorations: false`. This keeps the traffic lights and native drag behavior while allowing your UI to render behind the title bar area.
2. Add `data-tauri-drag-region` attribute to the merged bar element for drag functionality.
3. Add padding-left (~70px on macOS) to account for traffic light buttons. Use platform detection: `import { platform } from '@tauri-apps/plugin-os';`
4. On Windows, consider keeping decorations or implementing custom minimize/maximize/close buttons.
5. **Start with macOS only.** Get the overlay title bar working on macOS first. Windows and Linux can keep native decorations initially.
6. The existing 36px bar height (App.svelte line 333) may need to increase to accommodate the taller macOS title bar area (~28px system bar + content).
7. Test: window drag, traffic light clicks, double-click to maximize, full-screen mode, split-screen (macOS).

**Warning signs:**
- Window can't be dragged after title bar merge
- Traffic lights missing or overlapping content
- Layout shifts between macOS and Windows/Linux
- Full-screen mode leaves a gap where the title bar was

**Phase to address:**
Layout improvements phase. This is platform-specific and should be tested on all target platforms. Consider deferring Windows/Linux customization.

---

### Pitfall 6: Dialog System That Blocks the Main Thread or Loses Focus Context

**What goes wrong:**
The v0.6 dialog system needs to handle errors, warnings, and notifications. The existing patterns use:
1. `@tauri-apps/plugin-dialog` for `ask()` and `message()` — these are **native OS dialogs** that block the Tauri event loop
2. `InputDialog.svelte` — a custom HTML overlay dialog with z-index 9999

Problems emerge when:
- **Native dialogs block async operations:** `await ask(...)` pauses all JS execution. If a remote operation triggers an error while another native dialog is open, the error is queued and appears AFTER the user dismisses the first dialog — confusing timing.
- **Custom dialogs don't block:** `InputDialog` uses a `dialogConfig` state pattern. If a custom dialog is open and a `repo-changed` event fires, the dialog may lose its context (e.g., the file it was about to discard no longer exists).
- **Multiple error sources compete:** Remote operations, git2 operations, and filesystem watcher events can all produce errors simultaneously. Without a queue, dialogs stack or overwrite each other.
- **Focus trap missing:** `InputDialog.svelte` doesn't trap focus — Tab can move focus behind the dialog to toolbar buttons. Keyboard users can accidentally trigger actions while a dialog is open.

**Why it happens:**
The current codebase uses native dialogs for confirmations (stash drop, hard reset, checkout commit) and `InputDialog` for input (branch name, tag name). v0.6 adds a general-purpose dialog system. Mixing native and custom dialogs, plus adding notifications, creates a more complex state machine than a single `dialogConfig` can handle.

**How to avoid:**
1. **Choose one dialog approach and commit to it.** Recommendation: use custom Svelte dialogs for everything. Native dialogs (`ask`/`message`) are simpler but block the event loop and can't be styled.
2. If keeping native dialogs for confirmations (they're already used and work), use custom dialogs only for notifications/toasts that don't need user interaction.
3. **Dialog queue:** If using custom dialogs, implement a queue. Only one dialog renders at a time. New dialogs queue behind the current one.
4. **Guard against stale context:** When a dialog opens for "discard file X", capture the file path at dialog-open time. Verify the file still exists when the user confirms. If it doesn't (external change), show "File was already removed" instead of crashing.
5. **Focus trap:** Add `inert` attribute to the main content when a dialog is open, or implement Tab key trapping in the dialog component.
6. **Notifications vs dialogs:** Errors that need acknowledgment → dialog. Transient info (push succeeded, branch created) → toast/notification that auto-dismisses. Don't use dialogs for success messages.

**Warning signs:**
- Multiple dialogs stacking or overwriting each other
- Dialog appears for an already-resolved error
- Focus escapes the dialog to background elements
- Native dialog blocks a UI update that the user is waiting for

**Phase to address:**
Dialog system phase. Design the queuing and categorization (dialog vs toast) before implementing.

---

## Moderate Pitfalls

### Pitfall 7: Icon System Inconsistency Between SVG Inline, SVG File Import, and Existing Unicode Symbols

**What goes wrong:**
v0.6 adds an icon system throughout the app. The existing codebase uses three different icon approaches:
1. **Unicode symbols in Toolbar.svelte:** `↩ Undo`, `↓ Pull`, `↑ Push`, `📦 Stash`, `📥 Pop`, `⎇ Branch` (lines 178-207)
2. **Inline SVG paths in CommitGraph.svelte:** Tag icon (`<path d="M ... l 4 -4 l 4 4 l -4 4 z"/>`) and stash icon (`<path d="M ..."/>`) for ref pills (lines 615-632)
3. **Text characters:** `▼`/`▶` for expand/collapse in StagingPanel, `×` for close button in TabBar, `+`/`−` for stage/unstage in FileRow

Adding a formal icon system means either:
- Keeping all three approaches plus the new system (inconsistent)
- Migrating all existing icons to the new system (risk of breaking existing styling)
- A partial migration that leaves some Unicode and some new icons (visually inconsistent)

**Why it happens:**
Each icon approach was added incrementally. The project decisions doc notes "Unicode symbols for toolbar icons — Simple, no SVG assets needed" as a deliberate v0.3 decision. Now that the project has matured, a consistent system is needed, but migration isn't free.

**How to avoid:**
1. **Choose one icon approach:** Inline SVG components (e.g., `<Icon name="pull" />`) using a single SVG sprite sheet or individual `.svelte` icon components. This works for both the HTML UI and the SVG graph overlay.
2. **Migrate all existing Unicode/text icons in the same phase.** Don't leave a mix. The toolbar, sidebar section headers, file action buttons, and tab close button all need updating.
3. For the SVG overlay ref pill icons (tag diamond, stash icon): these are already inline `<path>` elements inside the SVG. These should stay as inline SVG paths — they can't use HTML icon components since they're inside an `<svg>` element.
4. Keep icon sizes consistent: 14-16px for toolbar, 12px for file rows, 10px for section headers.
5. Use `currentColor` for SVG fill/stroke so icons inherit their parent's text color.

**Warning signs:**
- Some buttons have icons, others have Unicode symbols
- Icon sizes inconsistent across components
- SVG icons in the graph overlay don't match the HTML icon style

**Phase to address:**
Icon system phase. Should be one of the first phases since it touches many components.

---

### Pitfall 8: Three-Way Selector (Commit/Amend/Stash) State Machine Complexity

**What goes wrong:**
Replacing the amend checkbox with a three-way commit/amend/stash selector introduces a state machine with more transitions than expected. The current `CommitForm.svelte` has two states: commit mode and amend mode. Adding stash mode creates interactions:

- **Commit → Amend:** Load HEAD commit message into fields (existing `handleAmendToggle`)
- **Amend → Commit:** Clear fields (existing)
- **Commit → Stash:** Change button text, allow empty staged (stash can include unstaged), optionally use commit message as stash name
- **Stash → Commit:** Clear stash-specific state
- **Amend → Stash:** Clear HEAD message, switch to stash mode
- **Stash → Amend:** Load HEAD message, switch to amend mode

Each transition must handle:
- Whether the commit message fields are preserved or cleared
- Whether staged files are required (commit requires staged; amend may not; stash doesn't require staged but needs dirty workdir)
- The button label and submit action
- Error state clearing

**Why it happens:**
A checkbox is binary. A three-way selector has 6 transition paths (3×2 directed transitions). Each transition has different semantics for the form fields. The existing `handleAmendToggle` already has non-trivial logic (loading HEAD message on toggle-on, clearing on toggle-off).

**How to avoid:**
1. Model the three modes as an explicit discriminated state:
   ```typescript
   type CommitMode = 'commit' | 'amend' | 'stash';
   let mode = $state<CommitMode>('commit');
   ```
2. Handle mode transitions in a single function that switches on `(oldMode, newMode)` pairs.
3. The stash mode submit should call existing `stash_save` command (already implemented in `stash.rs`).
4. The PROJECT.md requirement "Stash name defaults to commit form message" means: when switching to stash mode, pre-fill the stash name with the current commit subject. When switching FROM stash mode, clear the stash name but preserve the commit subject.
5. Validate per-mode: commit requires staged + subject, amend requires subject, stash requires dirty workdir.
6. Test every mode transition combination (6 total) and verify form state is correct after each.

**Warning signs:**
- Switching modes leaves stale error messages
- Amend mode message persists when switching to stash
- Submit calls the wrong command for the current mode
- Stash from amend mode accidentally amends instead

**Phase to address:**
Staging UX phase. Extract the mode state machine into a separate module for testability.

---

### Pitfall 9: SVG Ref Pill Overflow Badge Z-Index Behind Graph Lines

**What goes wrong:**
The v0.6 bug fix "branch overflow pill z-index behind graph" refers to the existing issue where the `+N` overflow badge on ref pills renders behind the graph line layer. The current SVG rendering has three layers in CommitGraph.svelte (lines 537-580):
1. `<g class="overlay-rails">` — background lane lines
2. `<g class="overlay-connections">` — merge/fork bezier curves
3. `<g class="overlay-dots">` — commit dots

The ref pills layer (`<g class="overlay-pills">`, line 582) renders AFTER dots, which should put it on top. But the `+N` overflow badge within the pills group may be clipped by the SVG's `width` attribute if the badge extends beyond the ref column width.

**Why it happens:**
SVG has no `overflow: visible` by default on child groups — the root `<svg>` element clips to its `width`/`height`. The overflow badge is positioned at `pill.x + pill.width + PILL_GAP` (line 658-659), which may exceed the SVG's width. The SVG width is set to `refOffset + Math.max(maxColumns, 1) * displaySettings.laneWidth` (line 533) — this is the ref column width + graph column width. If a pill + badge extends beyond the ref column width, it gets clipped.

**How to avoid:**
1. Set `overflow="visible"` on the root `<svg>` element — this allows child elements to render outside the SVG's viewport.
2. Alternatively, increase the SVG width to account for overflow badges: add `+ maxBadgeWidth` to the width calculation.
3. If using `overflow="visible"`, be careful that paths don't visually extend into adjacent columns. Use `clip-path` on the graph layers but not on the pills layer.
4. The hover expansion overlay (HTML div at line 696-744) is already positioned absolutely outside the SVG, so it works. The issue is only with the SVG-rendered badge.
5. Verify with `pointer-events: auto` on the badge — it needs to receive hover events for the expansion behavior.

**Warning signs:**
- `+N` badge cut off at column boundary
- Badge visible in some positions but clipped in others (depends on pill X position)

**Phase to address:**
Graph bug fix phase. Quick fix once root cause is identified.

---

### Pitfall 10: Discard Changes Without Confirmation Causes Data Loss

**What goes wrong:**
"Discard changes" is a **destructive, unrecoverable operation**. Unlike staging/unstaging, there's no way to get the discarded changes back (unless they were committed or stashed). If discard is triggered without confirmation — or if the confirmation dialog is dismissed by a keyboard shortcut the user didn't intend — work is permanently lost.

**Why it happens:**
The temptation is to implement discard as a simple file action button (like the existing `+` for stage, `−` for unstage in FileRow.svelte). But stage/unstage are safe operations. Discard is the most dangerous single-file operation in the UI.

**How to avoid:**
1. **Always require confirmation** for discard. Use `ask()` from `@tauri-apps/plugin-dialog` (existing pattern from stash drop, hard reset).
2. Show the file path in the confirmation message: "Discard changes to `src/app.css`? This cannot be undone."
3. For "discard all" (discarding all unstaged changes), make the confirmation MORE prominent: "Discard ALL unstaged changes (N files)? This cannot be undone."
4. Use `kind: 'warning'` on the dialog to show a caution icon.
5. Don't add a keyboard shortcut for discard until v0.7+ when undo-for-discard could be implemented (stash before discard).
6. Consider a UX pattern: discard actually stashes the changes to a hidden stash entry, allowing "undo discard" via stash pop. This is how some Git GUIs implement safe discard. Note: this adds complexity and is optional.

**Warning signs:**
- Discard button with no confirmation
- Discard-all with single-click action
- No distinction between discard (destructive) and unstage (safe) in the UI

**Phase to address:**
Discard changes phase. Confirmation dialog must be part of the initial implementation, not added later.

---

### Pitfall 11: Equal-Height File Lists Break When One List Is Empty

**What goes wrong:**
The v0.6 feature "Equal height for unstaged and staged file lists when not collapsed" means the unstaged and staged sections in StagingPanel should share available space equally. But when one list is empty (common: all files staged, or nothing staged), the equal-height CSS creates a large empty area for the empty list, wasting space.

**Why it happens:**
A naïve implementation uses `flex: 1` on both sections to split space equally. This works when both lists have content, but when one is empty, you get a tall empty box with just a header ("Staged Files (0)").

**How to avoid:**
1. Equal height should only apply when BOTH lists have content. When one is empty, the non-empty list should take all available space.
2. Use `flex: 1` on both sections, but add `min-height: 0` to allow shrinking and `max-height: {hasItems ? 'none' : '28px'}` to collapse empty sections to header-only.
3. Better approach: use CSS `flex-grow` only when expanded AND has items. A collapsed or empty section uses `flex-shrink: 0` with a fixed header height.
4. The existing expand/collapse toggle (`unstaged_expanded`, `staged_expanded` state in StagingPanel.svelte) already handles collapsing. The equal-height behavior should interact correctly with these toggles.

**Warning signs:**
- Large empty area when all files are staged
- Both lists 50% height even when one has 0 items
- Scroll appearing unnecessarily in a list with 1-2 items

**Phase to address:**
Staging UX phase. Test with: 0+N files, N+0 files, N+M files, 0+0 files.

---

### Pitfall 12: Untracked Files Not Showing WIP Row — Missing `WT_NEW` in Dirty Count

**What goes wrong:**
The v0.6 bug "new untracked files not showing WIP row or diff" likely stems from the dirty count calculation in `get_dirty_counts` (staging.rs lines 193-210). The `unstaged` counter checks for `WT_MODIFIED | WT_DELETED | WT_RENAMED | WT_TYPECHANGE` but **does NOT include `WT_NEW`**:

```rust
if s.intersects(
    Status::WT_MODIFIED
        | Status::WT_DELETED
        | Status::WT_RENAMED
        | Status::WT_TYPECHANGE,
) {
    unstaged += 1;
}
```

Untracked new files have `WT_NEW` status, which is NOT in this bitmask. So `unstaged` count stays 0 when only new untracked files exist. In App.svelte, `wipCount = dirtyCounts.staged + dirtyCounts.unstaged + dirtyCounts.conflicted` — if unstaged is 0 (because WT_NEW is excluded), the WIP row doesn't appear.

Meanwhile, `get_status_inner` (staging.rs line 29-35) DOES classify `WT_NEW` correctly via `classify_workdir()`. So the StagingPanel shows the files, but the WIP row in the commit graph doesn't appear.

**How to avoid:**
1. Add `Status::WT_NEW` to the `unstaged` bitmask in `get_dirty_counts`.
2. Verify that the `is_dirty()` function in branches.rs (used for checkout guards) intentionally excludes `WT_NEW` — it's correct there because git allows checkout with untracked files.
3. These are two DIFFERENT "dirty" concepts: "has changes that should show WIP row" (includes untracked) vs "has changes that would be lost on checkout" (excludes untracked).

**Warning signs:**
- Creating a new file in the repo doesn't show WIP row
- New files appear in staging panel but graph shows no WIP indicator
- `dirtyCounts.unstaged` is 0 when only new files exist

**Phase to address:**
Bug fix phase. This is a one-line fix but important because it affects the core visual indicator.

---

### Pitfall 13: Click on Sidebar Refs to Navigate Graph — Scroll Target May Not Be Loaded

**What goes wrong:**
The v0.6 feature "Click references in left pane to navigate graph" needs to scroll the commit graph to the commit that a branch or tag points to. But with pagination (200 commits per batch), the target commit may not be loaded yet. If the user clicks a branch that points to commit #500, and only 200 commits are loaded, there's nothing to scroll to.

**Why it happens:**
The commit graph uses lazy loading via `loadMore()` with 200-commit batches. The virtual list only loads more when scrolling near the bottom. Clicking a sidebar ref needs to:
1. Find the target commit's OID
2. Find its index in `displayItems`
3. Scroll to that index via `listRef.scroll()`

Step 2 fails if the commit hasn't been loaded yet.

**How to avoid:**
1. When a ref is clicked, first check if the target OID exists in the current `commits` array.
2. If found, scroll to it using the existing `listRef.scroll()` method.
3. If NOT found, load more commits until the target is found (or all commits are loaded). This requires a loop: `while (!found && hasMore) { await loadMore(); check again; }`.
4. Add a loading indicator while fetching ("Finding commit...").
5. For branches, the target commit is usually near HEAD (recently committed) — likely already loaded. For old tags, it may be far down the history.
6. The `list_refs_inner` function already returns branch info. Add the target OID to `BranchInfo` (it's not currently exposed to the frontend) so the frontend knows what to look for.
7. Alternative simpler approach: always scroll to HEAD for branch clicks (since checkout is the primary action). For tags, implement the search-and-scroll.

**Warning signs:**
- Clicking a branch/tag in sidebar does nothing (target not loaded)
- Clicking causes rapid loading of all commits (performance issue on large repos)
- Scroll animation targets wrong commit (index mismatch after loading)

**Phase to address:**
Ref navigation phase. Start with branches (usually near HEAD), then handle the tag case.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Using native `ask()` dialogs for all confirmations | Zero custom UI work | Blocks event loop, can't be styled, platform-inconsistent | Acceptable for v0.6 — replace with custom dialogs in v0.7+ if needed |
| Duplicating `open_repo_from_state()` across command files | No cross-file dependency | 5+ copies of the same 5-line function | Never ideal, but low cost. Consolidate when adding new command files |
| Hardcoded 70px left padding for macOS traffic lights | Quick title bar merge | Breaks if macOS changes traffic light size or user has RTL layout | Acceptable for initial implementation, should read actual inset from Tauri API |
| Not implementing "undo discard" | Simpler discard implementation | Users lose work with no recovery | Acceptable if confirmation dialog is robust; add stash-before-discard in v0.7 |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| git2 `Branch::delete()` | Calling on HEAD branch | Check `branch.is_head()` first, return friendly error |
| git2 `tag_delete()` | Passing full ref path (`refs/tags/X`) | Strip prefix: use `short_name` not `name` |
| git2 `checkout_head()` for discard | Using `safe()` checkout | Use `force()` — `safe()` refuses to overwrite (opposite of discard) |
| git2 discard untracked file | Using `checkout_head()` | Use `std::fs::remove_file()` — untracked files have no HEAD state |
| `data-tauri-drag-region` | Adding to child elements | Must be on the outermost draggable div, not nested children |
| `titleBarStyle: "overlay"` | Forgetting content padding | Add padding-left for traffic lights, padding-top for title bar height |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Loading all commits for ref navigation | UI freezes on large repos | Load in batches with progress indicator | Repos with 50k+ commits |
| `getBBox()` for icon sizing in SVG | Synchronous layout thrash | Use pre-measured constants or Canvas measureText | When icon count > 50 |
| Dialog renders behind SVG overlay | Dialog invisible but captures input | Ensure dialog z-index > SVG z-index (currently 9999 vs 1) | When SVG overlay is full-window width |
| FileRow rerender on every status refresh | Visible flicker during typing | Use `{#each ... (f.path)}` keying (already done) | When many files change rapidly |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Discard button looks like unstage button | User discards when they meant to unstage | Use red color and trash icon for discard, distinct from blue `−` for unstage |
| No distinction between delete-local and delete-remote branch | User accidentally removes remote branch | v0.6 should only delete local branches; remote branch delete deferred to later |
| Three-way selector is unclear | User doesn't realize they're in stash mode | Use clear visual states: blue for commit, orange for amend, purple for stash |
| Dialog blocks before error message is visible | User sees dialog, doesn't know what the error was | Show error details in the dialog body, not just "Operation failed" |
| Notification auto-dismisses before user reads it | Important info missed | Use 5s minimum display for notifications, longer for errors |

## "Looks Done But Isn't" Checklist

- [ ] **Discard modified file:** File reverts to HEAD content
- [ ] **Discard deleted file:** File is restored on disk
- [ ] **Discard new untracked file:** File is removed from disk
- [ ] **Discard staged file:** File is unstaged AND reverted
- [ ] **Discard file with both staged and unstaged changes:** Only unstaged changes discarded, staged changes preserved
- [ ] **Delete HEAD branch:** Shows friendly error, does not crash
- [ ] **Delete unmerged branch:** Shows confirmation warning about unmerged commits
- [ ] **Delete tag:** Tag disappears from sidebar AND graph ref pills
- [ ] **Tag delete with refs/tags/ prefix:** Defensive strip works, no double-prefix error
- [ ] **Three-way selector transitions:** All 6 mode pairs preserve/clear form state correctly
- [ ] **WIP row with only new files:** `WT_NEW` counted in dirty counts
- [ ] **Graph overflow with 8+ lanes:** Right-side columns remain visible, graph clips cleanly
- [ ] **Sidebar ref click → loaded commit:** Scrolls to commit in graph
- [ ] **Sidebar ref click → unloaded commit:** Shows loading state, loads batches, scrolls when found
- [ ] **Dialog over dialog:** Second dialog queues, first must be dismissed first
- [ ] **Dialog during remote operation:** Remote progress continues behind dialog
- [ ] **Icon consistency:** No remaining Unicode symbols in toolbar, sidebar headers, or file rows
- [ ] **macOS title bar merge:** Traffic lights visible, window draggable, double-click maximizes
- [ ] **Title bar on Windows:** Still functional (native decorations or custom buttons)
- [ ] **Trailing header divider:** Last visible column has no trailing divider line

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Discard implementation wrong (P1) | LOW | Fix status-type branching logic; tests catch all cases |
| Branch delete crashes (P2) | LOW | Add HEAD guard and borrow-drop pattern; unit test |
| Tag delete double-prefix (P3) | LOW | Add `strip_prefix` defensive check |
| Sticky positioning fails (P4) | MEDIUM | Redesign as flex overflow layout instead of sticky |
| Title bar merge breaks (P5) | HIGH | Revert to `decorations: true`, keep merged layout for macOS only |
| Dialog system inadequate (P6) | MEDIUM | Add queue system; worst case, keep using native dialogs |
| Icon inconsistency (P7) | LOW | Batch-migrate remaining Unicode in a single phase |
| Three-way selector bugs (P8) | MEDIUM | Explicit state machine with transition tests |
| Overflow badge z-index (P9) | LOW | Add `overflow="visible"` to SVG root |
| Missing WT_NEW in counts (P12) | LOW | One-line bitmask fix |
| Ref nav unloaded commit (P13) | MEDIUM | Implement load-until-found with progress indicator |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| P1: git2 discard gotchas | Discard Changes | Unit tests for all 4 file status types |
| P2: Branch delete edge cases | Branch Delete | Unit test: HEAD delete fails, unmerged shows warning |
| P3: Tag name format | Tag Delete | Unit test: short name and full ref path both work |
| P4: Sticky in virtual list | Graph Polish | Visual test with 8+ lane repo |
| P5: Title bar merge | Layout Improvements | Manual test on macOS + Windows |
| P6: Dialog system | Dialog System | Test simultaneous error scenarios |
| P7: Icon consistency | Icon System (first phase) | Visual audit of all components |
| P8: Three-way selector | Staging UX | State transition matrix test |
| P9: Overflow badge z-index | Graph Bug Fixes | Visual test with multi-ref commits |
| P10: Discard confirmation | Discard Changes | Verify no code path skips confirmation |
| P11: Equal height empty lists | Staging UX | Test 0+N, N+0, N+M, 0+0 file states |
| P12: WT_NEW in dirty counts | Bug Fixes (early) | Unit test: create untracked file → dirty count > 0 |
| P13: Ref nav pagination | Ref Navigation | Test clicking tag on commit #500 with 200 loaded |

## Sources

### HIGH confidence (direct codebase analysis)
- `staging.rs` — `get_dirty_counts` bitmask analysis, `classify_workdir` includes `WT_NEW` but `get_dirty_counts` doesn't
- `branches.rs` — `list_refs_inner`, `checkout_branch_inner`, `create_branch_inner` patterns for cache/emit
- `commit_actions.rs` — `create_tag_inner` uses `repo.tag()`, existing `tag_delete` API found in git2 docs
- `CommitGraph.svelte` — SVG overlay architecture, three-layer rendering, overlay snippet inside VirtualList
- `VirtualList.svelte` — DOM structure (container → viewport → content → items with translateY), scroll handling
- `StagingPanel.svelte` — current file list layout, expand/collapse state
- `CommitForm.svelte` — current amend toggle logic, submit flow
- `InputDialog.svelte` — z-index 9999, backdrop click, keydown handling
- `App.svelte` — 36px top bar, three-pane layout, dirty count flow
- `TabBar.svelte` — current title bar content (repo name + close)
- `Toolbar.svelte` — Unicode symbols for all buttons
- `tauri.conf.json` — current config with default decorations

### HIGH confidence (official documentation)
- git2 0.19 `Repository` docs — `checkout_head()`, `checkout_index()`, `tag_delete()`, `Branch::delete()` API signatures and behavior
- git2 `CheckoutBuilder` docs — `path()`, `force()`, `safe()`, `remove_untracked()` semantics
- Tauri 2 WindowConfig — `decorations`, `titleBarStyle`, `hiddenTitle` options

### MEDIUM confidence (CSS spec / established web standards)
- CSS `position: sticky` + `transform` interaction — spec states transform creates new containing block, breaking sticky relative to scroll ancestor
- SVG `overflow` attribute — default is `hidden` on `<svg>` elements, `visible` must be explicitly set

---

*Pitfalls research for: Trunk v0.6 — UI Polish & Core Ops*
*Researched: 2026-03-15*
