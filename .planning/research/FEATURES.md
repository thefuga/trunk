# Feature Landscape: v0.6 UI Polish & Core Ops

**Domain:** Desktop Git GUI — UI polish, missing core operations, staging UX, graph polish, dialog system
**Researched:** 2026-03-15

## Context

Trunk v0.6 is a polish milestone. The core architecture (graph, staging, commits, remotes) is proven. This milestone addresses the gap between "functional prototype" and "daily driver" — adding icons, missing destructive operations (discard, delete), UX refinements (commit mode selector, staging buttons), graph polish (overflow, navigation), bug fixes, and infrastructure (dialog system, layout merge).

Research focuses on: how do GitKraken, Fork, Sublime Merge, and Tower handle these features? What are user expectations? What patterns avoid pitfalls?

---

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Icon system throughout UI** | Every competitor uses icons. Unicode symbols (current: `↓ ↑ ⟗ 📦 📥 ↩ ↪`) look unpolished and render inconsistently across platforms. Icons are the single highest-impact visual polish item. | MEDIUM | Affects Toolbar, FileRow, BranchSidebar, StagingPanel, CommitForm, TabBar, context menus. ~30-40 icon placements. |
| **Discard changes (file-level)** | GitKraken, Fork, Tower, Sublime Merge all have discard. Users expect to revert unstaged changes without terminal. This is the most common "missing feature" for staging workflows. | MEDIUM | Backend: `git2::Repository::checkout_index` or `checkout_head` with path spec for tracked files; `std::fs::remove_file` for untracked. Frontend: trash icon on hover per file + "Discard All" button. **Must confirm** — destructive, unrecoverable. |
| **Branch delete** | Every Git GUI supports right-click → Delete on local branches. Without it, users must go to terminal for basic branch hygiene. | LOW | Backend: `git2::Branch::delete()`. Frontend: context menu item in BranchSidebar + confirmation dialog. Must prevent deleting HEAD branch. Consider "also delete remote" checkbox like GitKraken. |
| **Tag delete** | Same as branch delete — table stakes for any Git GUI with tag display. GitKraken shows "Delete locally" and "Delete from remote" as separate options. | LOW | Backend: `git2::Reference::delete()` for local tags. Remote tag delete needs `git push --delete origin <tag>` (CLI). Frontend: context menu in sidebar Tags section. |
| **Confirmation dialogs for destructive ops** | GitKraken, Fork, Tower all confirm before discard/delete. Users will accidentally lose work without confirmation. | LOW | Trunk already uses `@tauri-apps/plugin-dialog` `ask()` for stash drop (see BranchSidebar.svelte line 207). Extend this pattern to discard and delete operations. |
| **Stage All / Unstage All with clear visual affordance** | Currently text-only buttons ("Stage All Changes" / "Unstage All"). Every competitor uses colored icon buttons. GitKraken: green ↑ arrow. Fork: green + button. | LOW | Replace text buttons with icon buttons. Green background for "Stage All", red/muted for "Unstage All". Matches universal color semantics (green = add/approve, red = remove/danger). |
| **Discard All Changes button** | GitKraken shows a trash icon at the top of the unstaged section. Fork has "Discard All" button. Essential companion to per-file discard. | LOW | Trash icon button in unstaged section header, next to "Stage All". Must confirm before executing. |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Three-way commit/amend/stash selector** | GitKraken does this: the commit panel bottom has tabs/icons that switch between Commit, Amend, and Stash modes. Tower has a similar dropdown. Most GUIs keep these as separate actions (checkbox for amend, toolbar button for stash). Unifying them in one selector is cleaner. | MEDIUM | Replaces current `amend` checkbox + separate stash in toolbar. Selector changes button label and behavior. "Commit" (default), "Amend" (pre-fills HEAD message), "Stash" (creates stash, optionally with name from message field). Depends on: existing CommitForm refactor. |
| **Stash name defaults to commit message** | GitKraken does this — the WIP node name becomes the stash name. If user has typed a commit message and switches to Stash mode, the message becomes the stash name. Seamless context switching. | LOW | Read `subject` state from CommitForm when stash mode is selected. Pass as `message` to `stash_save`. |
| **Click refs in left pane → navigate graph** | GitKraken: double-click branch in left panel scrolls graph to that branch's HEAD commit. Fork: same. Sublime Merge: same. This is expected in apps that have both sidebar and graph. | MEDIUM | Requires: mapping branch name → commit OID → row index in virtual list → programmatic scroll to that index. Backend already returns `oid` per branch in `RefsResponse`. Frontend needs `scrollToIndex(idx)` on virtual list. |
| **Commit graph overflow with sticky right-side** | When graph has many lanes, the graph column should be scrollable horizontally but the right-side columns (message, author, date, SHA) should remain visible. Fork and GitKraken both handle this — message column never scrolls off-screen. | MEDIUM | CSS: graph column gets `overflow-x: auto` while remaining columns stay fixed. Or: the graph SVG column shrinks to a min-width with horizontal scroll, while right columns remain anchored. Tricky with current 6-column resizable layout. |
| **Right pane auto-opens when content changes** | If user clicks a file in staging panel or a commit in graph, and right pane is collapsed, it should auto-open. Fork does this. Prevents "I clicked but nothing happened" confusion. | LOW | Check `rightPaneCollapsed` state; if true and content arrives, set to false. Simple reactive guard in App.svelte. |
| **Merge window top bar with tab+actions bar** | Currently two bars: TabBar (repo name + close) and Toolbar (actions). Merging them saves 36px vertical space — valuable for a desktop app where the graph is the primary content area. | LOW | Layout change only. Move toolbar buttons into same flex row as tab. Fork and Sublime Merge both use single toolbar rows. |
| **Top/bottom padding in commit graph** | Without padding, the first and last commits touch the container edges. Fork and GitKraken both add whitespace padding. Small visual polish. | LOW | Add padding rows (empty space) to virtual list, or CSS padding on scroll container. |
| **Dialog system for errors/warnings/notifications** | Currently errors go to `console.error` or inline text. A proper dialog system shows modal/toast notifications for errors, warnings, and confirmations. GitKraken uses modal dialogs for errors + toast for progress. | MEDIUM | Generalized dialog component beyond current InputDialog. Needs: title, message, icon (error/warning/info), actions (OK / OK+Cancel / custom). Toast variant for non-blocking notifications. |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Discard individual hunks/lines** | v0.7 scope (Hunk Staging). Mixing hunk-level operations into v0.6 adds complexity to what should be a polish milestone. | File-level discard only. Hunk-level discard ships with hunk staging in v0.7. |
| **Remote branch delete** | Requires `git push --delete`, which is destructive and affects other collaborators. Easy to accidentally delete production branches. | Local branch delete only for v0.6. Remote delete can be added later with extra safety guards. |
| **Branch rename** | Separate feature with its own edge cases (tracking branch updates, remote rename). Not in v0.6 scope. | Defer to v0.7+. Users can delete + recreate. |
| **Undo discard** | `git checkout -- <file>` is irreversible for untracked files and unstaged changes. No Git GUI provides undo for discard. | Confirmation dialog is the safety mechanism. Do NOT promise or attempt undo. |
| **Toast notification system with queue/animation** | Over-engineering for v0.6. Simple modal dialogs cover all error/warning needs. | Modal dialog only. Toast/snackbar can come later if needed. |
| **Custom icon font / SVG sprite sheet** | Over-engineering. Tree-shaking individual imports is simpler and more maintainable. | Direct Lucide component imports. |
| **Tag push from sidebar** | Requires knowing which remote to push to, handling auth, error states. Separate concern from delete. | Tags are pushed via `git push` (already works). Tag push UI deferred. |

---

## Feature Dependencies

```
[Icon System]
  └── ALL other visual features depend on having icons available
  └── Should be first phase: establishes visual vocabulary

[Dialog System]
  └── Required by: Discard (confirmation), Branch Delete (confirmation), Tag Delete (confirmation)
  └── Extends: existing InputDialog pattern (backdrop + modal)
  └── Does NOT depend on: icon system (can use text-only initially)

[Discard Changes]
  └── Requires: Dialog System (confirmation before destructive action)
  └── Requires: Icon System (trash icon on hover per file)
  └── Backend: new Tauri command `discard_file` + `discard_all`
  └── Modifies: FileRow.svelte (add discard button), StagingPanel.svelte (add discard all)

[Branch Delete]
  └── Requires: Dialog System (confirmation)
  └── Modifies: BranchSidebar.svelte (context menu), BranchRow.svelte (context menu handler)
  └── Backend: new Tauri command `delete_branch`
  └── Safety: must check if branch == HEAD, refuse with error

[Tag Delete]
  └── Requires: Dialog System (confirmation)
  └── Modifies: BranchSidebar.svelte (Tags section context menu)
  └── Backend: new Tauri command `delete_tag`

[Three-Way Selector]
  └── Requires: Icon System (mode icons: commit, amend, stash)
  └── Modifies: CommitForm.svelte (replace checkbox with selector)
  └── Inherits: existing amend logic + stash_save from toolbar

[Staging Button Improvements]
  └── Requires: Icon System (+ and - icons, or checkmark/x)
  └── Modifies: StagingPanel.svelte (section headers)

[Graph Overflow/Sticky Columns]
  └── Independent of other features
  └── Modifies: CommitGraph.svelte, column layout CSS
  └── Risk: complex CSS interaction with virtual list + SVG overlay

[Click Refs → Navigate Graph]
  └── Requires: virtual list `scrollToIndex` capability
  └── Modifies: BranchSidebar.svelte (click handler), App.svelte (scroll communication)

[Graph Padding]
  └── Independent, simple CSS/virtual list change
  └── Modifies: CommitGraph.svelte or VirtualList.svelte

[Right Pane Auto-Open]
  └── Independent
  └── Modifies: App.svelte (reactive guard on rightPaneCollapsed)

[Layout Merge (top bars)]
  └── Independent
  └── Modifies: App.svelte, TabBar.svelte, Toolbar.svelte

[Bug Fixes]
  └── Independent of features, can be interleaved
  └── Overflow pill z-index: SVG layering fix
  └── Trailing header divider: CSS fix
  └── Untracked files not showing WIP: backend logic fix
```

---

## Detailed Feature Analysis

### 1. Icon System

**How competitors do it:**
- **GitKraken:** Custom icon font with proprietary glyphs. Consistent but not reusable.
- **Fork:** Native macOS SF Symbols + custom SVG icons. Platform-native look.
- **Sublime Merge:** Custom SVG icons, minimal set, very consistent sizing.
- **Tower:** Custom icon font, consistent weight and style throughout.

**Recommendation for Trunk:** Use **Lucide** (`@lucide/svelte` for Svelte 5). Reasons:
- 1,500+ icons, covers all Git GUI needs (git-branch, git-commit, git-merge, tag, trash, plus, minus, check, x, undo, redo, download, upload, archive, etc.)
- Official Svelte 5 package (`@lucide/svelte`), tree-shakable individual imports
- Consistent 24x24 grid, 2px stroke width — scales cleanly to 12-16px used in Trunk's UI
- ISC license (permissive, compatible with any future open source release)
- `currentColor` default — works with CSS custom properties for theming

**Key icon mappings needed (~35 icons):**
- Toolbar: `Undo2`, `Redo2`, `ArrowDown` (Pull), `ArrowUp` (Push), `GitBranch`, `Archive` (Stash), `ArchiveRestore` (Pop)
- FileRow status: `Plus`, `Pencil`, `Minus`/`Trash2`, `ArrowRight` (Renamed), `RefreshCw` (Typechange), `AlertTriangle` (Conflicted)
- FileRow actions: `Plus` (stage), `Minus` (unstage), `Trash2` (discard)
- StagingPanel: `ChevronDown`/`ChevronRight` (expand/collapse), `Plus` (stage all), `Minus` (unstage all), `Trash2` (discard all)
- CommitForm: `GitCommit` (commit mode), `PenLine` (amend mode), `Archive` (stash mode)
- Sidebar: `GitBranch` (local), `Globe` (remote), `Tag`, `Layers` (stashes), `Plus` (create), `Search` (filter)
- TabBar: `X` (close)
- Ref pills: `Tag` (better icon for tag pill — currently using generic symbol)

**Confidence:** HIGH — Lucide has official Svelte 5 package, verified via lucide.dev docs.

### 2. Discard Changes

**How competitors do it:**
- **GitKraken:** Trash icon on hover per file (unstaged section). "Discard all changes" button (trash icon) in section header. Confirmation dialog: "Are you sure you want to discard all changes?" Right-click context menu also has "Discard changes".
- **Fork:** Right-click file → "Discard Changes". Also has "Discard All" in toolbar. Confirmation dialog before executing.
- **Sublime Merge:** Right-click file → "Discard File". Keyboard shortcut available. Confirmation dialog.
- **Tower:** Right-click → "Discard Changes". Batch discard for multi-selected files. Confirmation.

**Universal pattern:** Every GUI confirms before discard. All support per-file and all-files. Most show discard on hover (icon) and in context menu (text).

**Implementation for Trunk:**
- Per-file: hover shows trash icon (third action after stage +/unstage −). Click → confirm dialog → `git checkout HEAD -- <path>` for tracked files, `rm` for untracked new files.
- Discard All: button in unstaged section header (trash icon). Click → confirm dialog → restore all.
- Backend: `git2::Repository::checkout_head()` with `CheckoutBuilder::path()` for tracked files. `std::fs::remove_file()` for untracked (New) files.
- Staged files: Do NOT show discard on staged files. Discard operates on unstaged changes only. This matches GitKraken/Fork behavior.

**Confidence:** HIGH — well-established pattern across all competitors.

### 3. Branch Delete

**How competitors do it:**
- **GitKraken:** Right-click branch → "Delete [branch-name]". Confirmation dialog. Cannot delete HEAD branch (menu item greyed out). Also offers "Delete [branch] from [remote]" as separate action.
- **Fork:** Right-click → Delete. Asks "Delete branch 'X'?" with option "Also delete remote tracking branch". Cannot delete current branch.
- **Sublime Merge:** Right-click → Delete Branch. Requires switching away from branch first.
- **Tower:** Right-click → Delete. Multi-select delete supported.

**Universal pattern:** Context menu → confirmation → delete. HEAD branch cannot be deleted (disabled/greyed, not hidden). Some offer remote delete as checkbox or separate option.

**Implementation for Trunk:**
- Add "Delete" to existing BranchRow context menu (right-click handling needed — currently only has checkout via click)
- Confirmation dialog using `@tauri-apps/plugin-dialog` `ask()` (same pattern as stash drop)
- Backend: `git2::Branch::delete()` after checking it's not HEAD
- Grey out/hide for HEAD branch
- v0.6 scope: local delete only (remote delete deferred — too destructive for v0.6)

**Confidence:** HIGH — straightforward pattern.

### 4. Tag Delete

**How competitors do it:**
- **GitKraken:** Right-click tag → "Delete [tag] locally" and "Delete [tag] from [remote]" as separate options. Confirmation: "Deleting a tag is permanent and cannot be undone."
- **Fork:** Right-click → Delete Tag. Confirmation.
- **Tower:** Right-click → Delete. Option to also delete from remote.

**Implementation for Trunk:**
- Tags section in sidebar currently has `BranchRow` with no context menu. Need to add right-click → "Delete tag" context menu.
- Backend: `git2::Reference::delete()` — find ref by `refs/tags/<name>` and delete.
- Confirmation dialog. "Delete tag '[name]'? This cannot be undone."
- v0.6 scope: local delete only.

**Confidence:** HIGH — straightforward.

### 5. Three-Way Commit/Amend/Stash Selector

**How competitors do it:**
- **GitKraken:** The commit panel has three mode icons at the bottom. Clicking the Stash icon switches the panel to "Stash mode" — the button says "Stash Changes", subject field becomes stash name, description becomes stash message. Amend pre-fills HEAD message. Very smooth mode switching.
- **Tower:** Dropdown selector above commit message: "Commit" / "Amend" / "Stash". Changes button text and behavior.
- **Fork:** Amend is a checkbox. Stash is a separate toolbar action. No unified selector.
- **Sublime Merge:** Amend via commit menu. Stash via menu/toolbar. No unified selector.

**Analysis:** GitKraken's unified approach is the best UX. It acknowledges that commit, amend, and stash are three variations of "save my work" and belong in the same UI location. Fork/Sublime Merge's approach of scattering them is functional but less discoverable.

**Implementation for Trunk:**
- Replace `amend` checkbox with segmented control or tab bar: `[Commit] [Amend] [Stash]`
- Default: "Commit" mode (current behavior)
- "Amend" mode: pre-fills subject/body from HEAD commit (current checkbox logic)
- "Stash" mode: button says "Stash Changes", subject field is stash name (optional), body is stash message
- When switching to "Stash" mode: if subject has text, it becomes the stash name (the "stash name defaults to commit message" feature)
- Stash mode should work with or without staged files (stashes everything)
- Visually: three small buttons/tabs above the commit message area, or a segmented control

**Confidence:** MEDIUM — the UX design is clear from competitors, but the exact Svelte component design needs iteration. The concept is well-proven.

### 6. Staging Button Improvements

**Current state:** Text buttons "Stage All Changes" and "Unstage All" in section headers. Individual files show `+` or `−` text on hover.

**How competitors do it:**
- **GitKraken:** Green "Stage all changes" button with icon. Individual files show "Stage File" label on hover. Unstage has corresponding button.
- **Fork:** Green `+` icon button for stage all, red `−` for unstage all. Per-file has same colored icons.
- **Sublime Merge:** Buttons in toolbar area, not inline.

**Implementation for Trunk:**
- "Stage All" button: green background, `Plus` or `PlusCircle` icon, prominent
- "Unstage All" button: red/danger background, `Minus` or `MinusCircle` icon
- Per-file: keep hover action icons but use proper Lucide icons instead of `+`/`−` text
- "Discard All" button: trash icon, danger color, in unstaged header (next to Stage All)
- Equal height for unstaged/staged sections when both expanded: CSS `flex: 1` on both sections, giving 50/50 split of available space

**Confidence:** HIGH — well-understood visual patterns.

### 7. Graph Overflow/Sticky Columns

**How competitors do it:**
- **GitKraken:** Graph column scrolls horizontally when many lanes push it wider than its allocated space. Message/author/date columns remain fixed in place. Horizontal scrollbar appears only on graph column.
- **Fork:** Similar — graph area is independently scrollable horizontally. Right columns (message, author, date, SHA) never scroll off-screen.
- **Sublime Merge:** Different approach — graph never gets too wide because it limits lane depth. But it does scroll the graph column independently.

**Implementation for Trunk:**
- Current: 6-column resizable layout with graph as one column. If graph needs more space than allocated, content is clipped.
- Fix: make graph column `overflow-x: auto` with its own horizontal scrollbar. OR: implement sticky right columns using CSS `position: sticky` on the right cells.
- The tricky part: SVG overlay spans the graph column. Horizontal scrolling of the graph column must also scroll the SVG overlay's x-position.
- Alternative simpler approach: instead of horizontal scroll, shrink graph lines when space is tight (reduce `LANE_WIDTH` dynamically based on `max_columns` vs available width). This is what some GUIs do for very dense graphs.

**Confidence:** MEDIUM — the UX goal is clear, but implementation with virtual list + SVG overlay is non-trivial. May need experimentation.

### 8. Click Refs → Navigate Graph

**How competitors do it:**
- **GitKraken:** Double-click branch in left panel → graph scrolls to that branch's HEAD commit and highlights it.
- **Fork:** Single-click branch → graph scrolls to commit, selects it.
- **Tower:** Same pattern.
- **Sublime Merge:** Same pattern.

**Implementation for Trunk:**
- Branch/tag entries in sidebar already have `oid` (from `RefsResponse`).
- On click: find the row index for that OID in the commit list. Call `virtualList.scrollToIndex(rowIndex)`.
- The virtual list (VirtualList.svelte) needs a `scrollToIndex(idx)` method exposed. This is a standard virtual list feature.
- After scrolling, select the commit (reuse `handleCommitSelect` from App.svelte).
- For branches: the click currently triggers checkout. Need to differentiate: single-click → navigate, double-click → checkout. Or: left-click → navigate, right-click → context menu with checkout/delete.

**Confidence:** HIGH — standard pattern, implementation path is clear.

### 9. Dialog System

**How competitors do it:**
- **GitKraken:** Modal dialogs for errors (red border, error icon), warnings (yellow), confirmations (neutral). Toast notifications for progress (push/pull). Activity log for history.
- **Fork:** OS-native dialogs for most confirmations. Custom modals for complex operations.
- **Tower:** Custom modal dialogs with icon, title, message, actions. Snackbar for transient notifications.

**Current Trunk state:** Uses `@tauri-apps/plugin-dialog` `ask()` for stash drop confirmation. Has `InputDialog.svelte` for form input (branch creation). Errors go to `console.error` or inline text (`checkoutError`, `stashCreateError`).

**Implementation for Trunk:**
- Generalize `InputDialog.svelte` into a `Dialog.svelte` component that supports:
  - **Confirm dialog:** title, message, "Cancel" / "Delete" buttons (destructive variant)
  - **Error dialog:** title, error message, "OK" button, error styling
  - **Warning dialog:** title, warning message, "OK" / "Cancel"
  - **Input dialog:** current InputDialog functionality (backwards-compatible)
- Use a shared `$state` rune module (`dialog-state.svelte.ts`) following the established `remote-state.svelte.ts` pattern
- Functions: `showConfirm(title, message): Promise<boolean>`, `showError(title, message): void`, `showInput(config): Promise<Record<string, string> | null>`
- This replaces direct `@tauri-apps/plugin-dialog` calls with in-app styled dialogs (consistent visual language)
- OR keep using `@tauri-apps/plugin-dialog` for confirmations (OS-native) and only build custom dialogs for errors/warnings

**Recommendation:** Use **native OS dialogs** (`@tauri-apps/plugin-dialog`) for confirmations (discard, delete) — they're already proven and feel native. Build a custom **error/notification dialog** for error display (replacing console.error). This hybrid approach gives native feel for confirmations + custom styling for errors.

**Confidence:** HIGH — patterns are well-established, Trunk already has 80% of the infrastructure.

### 10. Bug Fixes

| Bug | Root Cause (likely) | Fix Approach | Complexity |
|-----|---------------------|--------------|------------|
| **Branch overflow pill z-index behind graph** | SVG overlay has higher z-index than HTML ref pills, or SVG `<g>` rendering order. | Ensure overflow badge SVG elements render in the topmost `<g>` group. Or increase z-index of the overflow badge container. | LOW |
| **Trailing header divider on last visible column** | CSS `:last-child` doesn't account for hidden columns (display:none doesn't remove from DOM). | Use `:last-of-type` or calculate visible columns and apply class to actual last visible column. | LOW |
| **New untracked files not showing WIP row or diff** | `get_status` may not detect untracked files, or WIP row rendering logic doesn't trigger when only untracked files exist (no staged or modified). | Check `status.unstaged` includes untracked files. Ensure WIP row appears when `unstaged.length > 0` regardless of file status type. | LOW-MEDIUM |

---

## MVP Recommendation

### Phase 1: Foundation (enables everything else)
1. **Icon system** — install `@lucide/svelte`, replace all Unicode symbols throughout app
2. **Dialog system** — generalize confirmation dialogs for destructive operations

### Phase 2: Core Operations
3. **Discard changes** — per-file + discard all, with confirmation
4. **Branch delete** — context menu + confirmation
5. **Tag delete** — context menu + confirmation

### Phase 3: Staging & Commit UX
6. **Three-way selector** — commit/amend/stash unified control
7. **Staging button improvements** — colored icon buttons for stage all/unstage all
8. **Stash name from commit message** — wire up subject field to stash name

### Phase 4: Graph & Navigation
9. **Graph padding** — top/bottom whitespace
10. **Click refs → navigate** — sidebar click scrolls to commit
11. **Graph overflow/sticky columns** — horizontal scroll for wide graphs

### Phase 5: Layout & Polish
12. **Right pane auto-open** — reactive guard
13. **Merge top bars** — layout consolidation
14. **Bug fixes** — overflow z-index, trailing divider, untracked files

**Defer:**
- Remote branch/tag delete: too destructive for v0.6, needs extra safety
- Hunk-level discard: ships with v0.7 hunk staging
- Branch rename: separate feature, not in scope
- Toast notifications: over-engineering for v0.6

---

## Sources

- GitKraken Desktop documentation (Feb 2026): staging, stashing, tags, branching — help.gitkraken.com (accessed 2026-03-15) [HIGH confidence]
- Lucide Svelte package documentation: lucide.dev/guide/packages/lucide-svelte (accessed 2026-03-15) [HIGH confidence]
- Fork Git client: feature observation from known UX patterns [MEDIUM confidence — no docs fetched, based on domain knowledge]
- Sublime Merge, Tower: feature observation from known UX patterns [MEDIUM confidence — based on domain knowledge]
- Trunk codebase: App.svelte, CommitForm.svelte, StagingPanel.svelte, Toolbar.svelte, BranchSidebar.svelte, FileRow.svelte, InputDialog.svelte [HIGH confidence — direct code review]
- git2 crate: Repository::checkout_head, Branch::delete, Reference::delete APIs [MEDIUM confidence — based on training data, verify against current docs]

---
*Feature landscape for: Trunk v0.6 — UI Polish & Core Ops*
*Researched: 2026-03-15*
