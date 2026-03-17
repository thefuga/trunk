# Features Research: Trunk v0.7 — Hunk Staging & Commit Graph Search

**Domain:** Desktop Git GUI — granular staging, commit search
**Researched:** 2026-03-17

## Context

Trunk v0.7 adds two features: (1) stage/unstage individual hunks within file diffs, and (2) search the commit graph by SHA, message, and branch name via cmd+f. Trunk already has whole-file staging, a DiffPanel that renders hunks with headers and colored +/- lines, a commit graph with SVG overlay and virtualized scrolling, and a branch sidebar with search filtering.

This research covers how GitKraken, Fork, Sublime Merge, lazygit, GitUI, and VS Code (Source Control) implement these features — what's table stakes, what differentiates, and what to avoid.

---

## Hunk Staging

### How Each Tool Does It

**GitKraken:**
- Click a file in WIP node to open diff view. Diff shows hunks separated by hunk headers.
- Three diff view modes: **Hunk View** (collapsed context, shows only changed blocks), **Inline View** (full file with inline changes), **Split View** (side-by-side).
- In Hunk View, each hunk has a **"Revert"** button to revert that hunk.
- To stage specific lines: highlight lines in the diff, right-click → "Stage selected lines". Same for unstaging.
- "Stage File" / "Unstage File" buttons remain on each file header for whole-file operations.
- Discard hunks: right-click in diff → "Discard hunk". Available since late 2024.
- No explicit "Stage Hunk" button per hunk — staging granularity is line-based via selection, or whole-file.

**Fork:**
- Diff view shows unified diff with hunk headers.
- Features **line-by-line staging**: click a line to select it, shift-click for range, then stage selection.
- "Stage / unstage changes line-by-line" is listed as a headline feature on their marketing page.
- Stage Hunk: click the hunk header area to stage the entire hunk.
- Split (side-by-side) diff view supported.
- No per-hunk stage/unstage buttons visible in the UI — staging is selection-based.

**Sublime Merge:**
- Diff is shown per file with collapsible hunks. Three staging granularities with explicit buttons:
  - **"Stage file"** button at file level
  - **"Stage hunk"** button at each hunk header
  - **"Stage lines"**: select individual lines, then click "Stage lines" button
- Context dragging: click and drag the edge of a hunk to expand/collapse context lines around changes.
- Full file context toggle: button on hunk header toggles between hunk view and full file view.
- This is the gold standard for hunk staging UX — explicit buttons at every level, no ambiguity.

**lazygit (TUI):**
- In the files panel, press enter to view diff. Diff shows hunks.
- Press **space** on a line to stage/unstage that single line.
- Press **v** to enter visual (range) selection mode, then space to stage the range.
- Press **a** to stage/unstage the entire current hunk.
- Navigation: arrow keys move between lines, tab switches between staged/unstaged panels.
- No mouse support for hunk staging — entirely keyboard-driven.

**GitUI (TUI, Rust + git2):**
- Stage, unstage, revert and reset at file, hunk, and line granularity.
- In diff view: navigate to a hunk, press enter/space to stage it.
- Line-level staging via visual selection mode.
- Uses git2 crate (same as Trunk) for all operations.

**VS Code Source Control:**
- Gutter icons: each hunk in the diff editor has inline stage/unstage/revert gutter buttons.
- Can also select lines and use "Stage Selected Ranges" from context menu or command palette.
- Hunk headers shown as collapsible sections with "Stage Change" / "Revert Change" buttons in the gutter.

### Table Stakes (must have)

| Feature | Evidence | Notes |
|---------|----------|-------|
| **Per-hunk stage button** | Sublime Merge, VS Code, GitUI all have explicit hunk-level stage buttons. Fork and lazygit also support it (click header / press 'a'). | Button on each hunk header: "Stage Hunk" (unstaged view) or "Unstage Hunk" (staged view). Universally expected. |
| **Per-hunk unstage button** | Mirrors stage. All tools that support hunk staging also support hunk unstaging at the same granularity. | Button on each hunk header in staged diff view. |
| **Visual hunk boundaries** | Every tool shows hunk headers (`@@ -X,Y +A,B @@`) with distinct styling (muted color, background band). Hunks are visually separated sections. | Trunk already renders hunk headers — need to add interactive affordance (button) to them. |
| **Hunk header as clickable/actionable region** | Sublime Merge, Fork, VS Code all make the hunk header row interactive. It's the natural place for the stage/unstage button. | Hunk header row becomes a flex container: `[hunk header text] [Stage Hunk / Unstage Hunk button]`. |
| **Hover affordance on stage/unstage buttons** | All GUI tools show buttons on hover or always-visible. Not hidden behind right-click. Hunk staging must be discoverable. | Button always visible or appears on hover over hunk header. Green for stage, red for unstage — matching Trunk's existing color semantics. |
| **Diff context awareness (unstaged vs staged)** | When viewing an unstaged file, hunks can be staged. When viewing a staged file, hunks can be unstaged. Different diff source = different action. | Trunk already has `diff_workdir` and `diff_staged` commands. The DiffPanel needs to know which context it's in to show the right button. |
| **Discard hunk** | GitKraken ("Revert Hunk"), VS Code ("Revert Change"), Sublime Merge (context menu). Discarding individual hunks from working tree is expected alongside staging. | Third action on hunk header: "Discard Hunk" (only for unstaged hunks). Requires confirmation or undo capability. |
| **Refresh after stage/unstage** | After staging a hunk, the diff must refresh to reflect the change. If a file has 3 hunks and you stage 1, the unstaged diff now shows 2 hunks. | IPC round-trip: stage hunk → re-fetch diff → re-render. Must feel instant (<100ms). |

### Differentiators (nice to have)

| Feature | Evidence | Complexity | Notes |
|---------|----------|------------|-------|
| **Line-level staging** | GitKraken (select lines → right-click → "Stage selected lines"), Fork (click/shift-click lines), Sublime Merge ("Stage lines" button), lazygit (space on line, v for range). | HIGH | Requires: (1) making individual diff lines selectable/highlightable, (2) computing the partial patch from selected lines, (3) applying that patch via git2. Significantly more complex than hunk staging. Fork, GitKraken, and Sublime Merge all have it, making it a differentiator but approaching table stakes for premium GUIs. |
| **Split (side-by-side) diff view** | GitKraken, Fork, Sublime Merge all offer split view. Old file left, new file right. | MEDIUM-HIGH | Different rendering mode for DiffPanel. Requires computing line-to-line mapping between old and new. Not needed for hunk staging but pairs well with it. |
| **Context expansion (show more lines)** | Sublime Merge: drag hunk edges to reveal more context. VS Code: "Show More Lines" button between hunks. | MEDIUM | Requires backend support to re-fetch diff with different context line count. git2's DiffOptions has `context_lines()` setting. |
| **Hunk splitting** | `git add -p` in CLI has the `s` (split) command that breaks large hunks into smaller ones. None of the GUI tools expose explicit "split hunk" UI. Sublime Merge handles it via line-level staging. lazygit and GitUI handle it via line selection. | HIGH (if manual), N/A (via line staging) | git2 doesn't expose hunk splitting. The standard approach is: support line-level staging, which implicitly gives split-hunk capability. Explicit "split hunk" button is an anti-feature — nobody does it in GUIs. |
| **Keyboard shortcuts for hunk navigation** | lazygit: arrow keys between hunks. VS Code: `Alt+F3` / `Alt+F5` jump between changes. | LOW | Add `[` / `]` or `↑` / `↓` keyboard shortcuts to jump between hunks in diff view. Low effort, high value for keyboard users. |

### Anti-Features (avoid)

| Feature | Why Avoid |
|---------|-----------|
| **Explicit "Split Hunk" button** | No GUI tool has this. The CLI's `git add -p` split is a workaround for not having line-level staging. GUIs solve this with line selection instead. Adding a split button adds confusion. |
| **Inline editing in diff view** | Some tools (GitKraken, VS Code) allow editing files directly in the diff. This is a separate, complex feature. Mixing editing with staging creates ambiguous state. Defer entirely. |
| **Three diff view modes (hunk/inline/split)** | GitKraken has all three. Adding view mode switching adds UI complexity. For v0.7, ship one good unified diff view with hunk staging. Split view is a separate future feature. |
| **Drag-to-expand context lines** | Sublime Merge's context dragging is elegant but complex — requires re-diffing with different context sizes per hunk. Defer to future polish. |
| **Staging from commit diff view** | Staging should only work from WIP (working directory) diffs. Allowing staging from historical commit diffs creates confusion. Keep commit diffs read-only. |

---

## Commit Graph Search

### How Each Tool Does It

**GitKraken:**
- **Search bar** in upper-right corner of the app, always visible. Defaults to commit search.
- Searches by: **commit message**, **SHA**, and **author name/email**.
- Results update **live** as you type (incremental search).
- Matching commit is **highlighted in the graph** — the graph scrolls to show the match.
- Does NOT filter/hide non-matching commits — the full graph remains visible with the match highlighted.
- No prev/next navigation for multiple results mentioned in docs — appears to highlight the first match.
- Also has a **Command Palette** (Cmd+P) for actions, but commit search is the dedicated search bar.

**Fork:**
- **Filter bar** at the top of the commit list. Typing filters the visible commits.
- Searches by: commit message, SHA, author, branch name.
- Approach: **filters the commit list** to show only matching commits. Non-matches are hidden.
- Can toggle between "Filter" mode (hide non-matches) and "Search" mode (highlight matches).
- Supports regex in search queries.

**Sublime Merge:**
- **Cmd+F** (or Ctrl+F) opens a **search overlay bar** at the top of the commit graph.
- `toggle_search` command bound to `super+f`.
- Searches by: commit message, SHA, author, file path, branch name.
- Rich query syntax: can prefix with `author:`, `path:`, `sha:`, `branch:` for scoped search.
- Results: **filters the graph** to show only matching commits. The graph topology is preserved (connecting lines drawn between visible matching commits).
- Has prev/next navigation buttons (or Enter/Shift+Enter) to jump between results.
- Also supports `--` for regex and various git-log-style options.

**lazygit (TUI):**
- Press **`/`** in any panel to enter **filter mode**. Types filter the visible list.
- In commits panel: filters commits by message text.
- In branches panel: filters branches by name.
- Separate **search commits** feature: press `Ctrl+S` to search commits by message/SHA with prev/next navigation.
- Does NOT search across panels simultaneously — each panel has its own filter.

**GitUI (TUI):**
- Search/filter in commit log panel.
- Searches by commit message text.
- Filter mode narrows visible commits.

**VS Code Source Control (Timeline/Git Graph extension):**
- Git Graph extension: Cmd+F opens search bar. Searches by message, SHA, author. Highlights matches in graph, with prev/next jump.
- Does NOT filter the graph — highlights within it.

### Table Stakes (must have)

| Feature | Evidence | Notes |
|---------|----------|-------|
| **Cmd+F keyboard shortcut** | Sublime Merge, VS Code, and web browsers all use Cmd+F. Universal muscle memory. GitKraken has a dedicated search bar always visible but Cmd+F is the expected shortcut. | Must intercept Cmd+F when commit graph has focus. |
| **Search overlay bar** | Sublime Merge and VS Code show a floating search bar at the top of the content area (not a sidebar, not a modal). This is the standard UX for in-place search. | Overlay bar at top of CommitGraph: text input + match count + prev/next buttons + close. |
| **Search by commit message** | Every tool searches messages. This is the primary use case: "find the commit where I changed X". | Substring match (case-insensitive) against `summary` and optionally `body`. |
| **Search by SHA (full or prefix)** | Every tool supports SHA search. Developers copy SHAs from GitHub, CI logs, error messages. Must find commits by hash. | Match against `oid` or `short_oid`. Prefix matching (typing `abc` matches `abc1234...`). |
| **Search by branch/ref name** | GitKraken, Fork, Sublime Merge all support branch name search. Essential for finding where a branch points in a large graph. | Match against ref labels on commits. If a commit's refs contain a match, highlight that commit. |
| **Highlight matches in graph** | GitKraken highlights matching commit. VS Code/Git Graph extension highlights. This preserves graph context — user sees where the match sits in the topology. | Highlight the matching row (background color change, or ring around the dot). Do NOT hide other commits. |
| **Scroll to first match** | When search finds a result, the graph must scroll to show it. Otherwise the user has no feedback. | Auto-scroll virtual list to the row index of the first match. |
| **Match count display** | "3 of 17 matches" — standard for search UIs (browsers, VS Code, Sublime Text). Gives user confidence about search completeness. | Display next to search input: `N of M` or `N results`. |
| **Prev/Next navigation** | Enter (or Down arrow) → next match. Shift+Enter (or Up arrow) → previous match. Sublime Merge and VS Code both support this. Standard search pattern. | Prev/Next buttons in search bar + keyboard shortcuts. Wraps around at ends. |
| **Escape to close** | Universal: Escape closes the search overlay and returns focus to the graph. | Clear search state, remove highlighting, close overlay. |
| **Dismiss without clearing** | Pressing Escape should close the overlay but ideally keep the current scroll position (don't jump back to where user was before search). | Close overlay, preserve scroll position, remove highlights. |

### Differentiators (nice to have)

| Feature | Evidence | Complexity | Notes |
|---------|----------|------------|-------|
| **Search by author** | GitKraken, Sublime Merge, Fork all support author search. | LOW | Match against `author_name` or `author_email`. Easy to add since data already exists in GraphCommit. |
| **Live/incremental search** | GitKraken updates results as you type. Sublime Merge also live-filters. | MEDIUM | Debounce input (150-200ms), re-search on each keystroke. Must be fast enough for large repos (searching 10k+ commit messages). Consider running search in Rust backend for speed. |
| **Scoped search prefixes** | Sublime Merge: `author:jane`, `sha:abc`, `branch:feature/`. Explicit scope narrows results. | LOW-MEDIUM | Parse input for `key:value` prefix. Route to appropriate field match. Good power-user feature. |
| **Filter mode (hide non-matches)** | Fork and Sublime Merge can filter the graph to show only matching commits. Topology is preserved with connecting lines. | HIGH | Requires re-computing visible rows, adjusting SVG overlay to draw connections between non-adjacent visible rows. Complex interaction with virtual list and lane algorithm. |
| **Regex support** | Fork supports regex. Sublime Merge has regex toggle. | LOW | Use Rust `regex` crate in backend search. Toggle button in search bar. |
| **Search by file path** | Sublime Merge supports `path:` scoped search. "Find the commit that last changed this file." | HIGH | Requires walking commit trees to find path changes — expensive. Not a v0.7 priority. |
| **Persist search across panel switches** | If user switches to diff panel and back, search state should be preserved. | LOW | Store search state in a shared `$state` module. |

### Anti-Features (avoid)

| Feature | Why Avoid |
|---------|-----------|
| **Full graph filtering (hide non-matches)** | Extremely complex to implement with SVG overlay + virtual list + lane algorithm. Hiding rows breaks the virtual list's index assumptions and requires re-rendering all SVG paths. Fork and Sublime Merge took years to get this right. Not worth it for v0.7. Highlight-in-place is sufficient. |
| **Search replacing sidebar filter** | The branch sidebar already has a text filter for branch names. Commit graph search is a separate feature. Don't merge them — they serve different purposes (find branch vs find commit). |
| **Search modal/dialog** | A search modal (centered popup) blocks the graph view. The user needs to see the graph while searching. An overlay bar at the top (like Ctrl+F in browsers) is correct. |
| **Fuzzy matching** | Fuzzy search (like fzf) sounds cool but commit messages and SHAs are not fuzzy targets. Substring match is what every tool uses and what users expect. Fuzzy matching would produce confusing results. |
| **Saved searches / search history** | Over-engineering for v0.7. Users can type the same search again. |
| **Cross-panel search** | Searching commits, branches, and files simultaneously creates ambiguous results. Each panel should have its own search. For v0.7, search is commit-graph-only. |

---

## Complexity Notes

### Hunk Staging

| Task | Complexity | Why |
|------|-----------|-----|
| **Backend: stage hunk (Rust/git2)** | MEDIUM | git2 has `Repository::apply()` for applying patches. Need to: (1) construct a `git2::Diff` for a single hunk, (2) apply it to the index. Alternative: use `git2::Index::add_frombuffer()` to write the partially-staged file content to the index. Another approach: shell out to `git apply --cached` with a patch constructed from the hunk. git2's apply API is the cleanest but requires careful patch construction. |
| **Backend: unstage hunk** | MEDIUM | Reverse of stage: construct a reverse patch for the hunk, apply to index. Or: recompute the file content without the hunk and write to index. |
| **Backend: discard hunk** | MEDIUM | Apply reverse patch to working directory (not index). Or write the original content (from HEAD) for just those lines back to the file. |
| **Frontend: hunk header buttons** | LOW | Add a button element to the existing hunk header div in DiffPanel.svelte. Wire to new Tauri command. |
| **Frontend: diff refresh after hunk op** | LOW-MEDIUM | After staging a hunk, re-invoke `diff_workdir` / `diff_staged` and re-render. Must handle the case where staging the last hunk removes the file from the unstaged list. |
| **Frontend: context awareness** | LOW | DiffPanel needs a prop or context to know if it's showing unstaged or staged diff. This determines whether buttons say "Stage" or "Unstage". |

**Total hunk staging estimate: MEDIUM complexity.** The main challenge is the git2 patch construction for partial staging. Once the backend command works, the frontend is straightforward.

### Commit Graph Search

| Task | Complexity | Why |
|------|-----------|-----|
| **Backend: search command** | LOW | Search is a filter over `GraphCommit[]` data that's already cached in `CommitCache`. New Tauri command `search_commits(repo, query)` iterates cached commits and returns matching indices/OIDs. Fast: 10k commits × string contains check = <1ms. |
| **Frontend: search overlay component** | MEDIUM | New Svelte component: SearchBar.svelte. Text input, match count, prev/next buttons, close button. Positioned absolutely at top of CommitGraph container. Cmd+F keybinding to toggle. |
| **Frontend: highlight matching rows** | LOW | Pass `Set<string>` of matching OIDs to CommitRow. Matching rows get a subtle background highlight (yellow/amber tint). |
| **Frontend: scroll to match** | LOW | Virtual list `scrollToIndex()` already needed (and may already exist from v0.6 ref navigation). Jump to first match on search, advance on prev/next. |
| **Frontend: keyboard handling** | LOW | Cmd+F to open, Escape to close, Enter for next, Shift+Enter for previous. Standard event handlers. |

**Total search estimate: LOW-MEDIUM complexity.** Mostly frontend work. The search itself is trivial (string matching on cached data). The UX component is the main effort.

---

## Dependencies on Existing Features

### Hunk Staging depends on:

| Existing Feature | How Used | Status |
|-----------------|----------|--------|
| **DiffPanel.svelte** | Already renders hunks with headers and colored lines. Hunk staging adds buttons to each hunk header div. | Ready — extend, not replace |
| **DiffHunk type (Rust + TS)** | Already has `header`, `old_start`, `old_lines`, `new_start`, `new_lines`, `lines[]`. Contains all data needed to construct a patch for staging. | Ready — no changes needed |
| **`diff_workdir` / `diff_staged` commands** | Already return `FileDiff[]` with hunks. Used to refresh diff after staging a hunk. | Ready — reuse as-is |
| **`stage_file` / `unstage_file` commands** | Existing whole-file staging. Hunk staging is a new parallel command (`stage_hunk`, `unstage_hunk`) — does not replace whole-file staging. | Ready — parallel path |
| **StagingPanel.svelte** | Shows unstaged/staged file lists. After hunk staging, file may split between lists (partially staged). StagingPanel already handles this case if the file appears in both lists. | Ready — but verify partial staging display |
| **Filesystem watcher + auto-refresh** | After staging a hunk, the watcher should trigger a status refresh. Existing `repo-changed` event flow handles this. | Ready |
| **Toast notifications (v0.6)** | Can show success/error toasts for stage/unstage/discard operations. | Ready |

### Commit Graph Search depends on:

| Existing Feature | How Used | Status |
|-----------------|----------|--------|
| **CommitGraph.svelte** | Container for the search overlay. Search results highlight rows within it. | Ready — add overlay child |
| **CommitRow.svelte** | Needs to accept a `highlighted` or `isSearchMatch` prop for visual styling. | Ready — add prop |
| **VirtualList.svelte** | Needs `scrollToIndex(idx)` method for jumping to search results. | May need enhancement — check if already exposed from v0.6 ref navigation feature |
| **CommitCache (Rust)** | Search queries run against cached commits. No additional data loading needed. | Ready — `GraphResult` has all commit data |
| **GraphCommit type** | Has `oid`, `short_oid`, `summary`, `author_name`, `refs[]` — all searchable fields. | Ready |
| **BranchSidebar search** | Sidebar already has a text filter for branches (implemented in v0.6). Separate from commit graph search but establishes the search UX pattern. | Ready — separate feature, consistent pattern |

---

## Implementation Recommendations

### Hunk Staging — Recommended Approach

**Backend (Rust):** The cleanest approach with git2 is:
1. `stage_hunk(repo_path, file_path, hunk_index)`: Read the working directory file. Read the index (staged) version. Construct the "desired index content" by taking the current index content and applying only the target hunk's changes. Write this content to the index via `Index::add_frombuffer()`.
2. Alternative: Build a minimal unified diff patch string for the single hunk and apply via `git2::Repository::apply()` with `ApplyLocation::Index`.
3. `unstage_hunk`: Same approach in reverse — construct index content with the hunk removed.
4. `discard_hunk`: Build reverse patch, apply to working directory file.

**Frontend (Svelte):** 
1. Add `diffContext: 'unstaged' | 'staged' | 'commit'` prop to DiffPanel.
2. When `diffContext` is `'unstaged'`: show "Stage Hunk" + "Discard Hunk" buttons on each hunk header.
3. When `diffContext` is `'staged'`: show "Unstage Hunk" button on each hunk header.
4. When `diffContext` is `'commit'`: no buttons (read-only).
5. On button click: invoke `stage_hunk` / `unstage_hunk` / `discard_hunk` → re-fetch diff → re-render.

### Commit Graph Search — Recommended Approach

**Backend (Rust):**
1. `search_commits(repo_path, query)` — iterate `CommitCache` entries, match `query` (case-insensitive substring) against `summary`, `oid`, `short_oid`, and ref names. Return `Vec<usize>` (matching row indices) + total count.
2. Consider: doing search on frontend since `GraphCommit[]` is already available in JS. Avoids IPC round-trip. 10k commits × 3 string comparisons is fast in JS.

**Frontend (Svelte):**
1. New `SearchOverlay.svelte` component: positioned absolute at top of CommitGraph.
2. Cmd+F toggles visibility. Escape closes.
3. Input field with debounced search (100-150ms).
4. Display: "N of M matches" + [Prev] [Next] buttons.
5. Pass `matchingOids: Set<string>` to CommitGraph for row highlighting.
6. On result navigation: call `virtualList.scrollToIndex(matchIndex)`.

---

## Sources

- GitKraken Desktop docs — staging (Feb 2026): explicit line staging via right-click, hunk revert in Hunk View [HIGH confidence — fetched directly]
- GitKraken Desktop docs — diff (Mar 2026): Hunk/Inline/Split views, revert hunk button per hunk [HIGH confidence — fetched directly]
- GitKraken Desktop docs — search (Jan 2026): search bar searches by message/SHA/author, highlights in graph, live results [HIGH confidence — fetched directly]
- Fork homepage: "Stage / unstage changes line-by-line" as headline feature [HIGH confidence — fetched directly]
- Sublime Merge docs — getting started: "Stage file" / "Stage hunk" / "Stage lines" explicit buttons, screenshot reference [HIGH confidence — fetched directly]
- Sublime Merge docs — key bindings: `toggle_search` bound to `super+f`, search_mode context key [HIGH confidence — fetched directly]
- Sublime Merge docs — diff context: context dragging and full-file toggle [HIGH confidence — fetched directly]
- lazygit README: space to stage line, `v` for range selection, `a` for whole hunk, `/` for filter mode [HIGH confidence — fetched directly]
- GitUI README: "Stage, unstage, revert and reset files, hunks and lines", search commit log [HIGH confidence — fetched directly]
- Trunk codebase: DiffPanel.svelte, types.ts (DiffHunk/DiffLine/FileDiff), src-tauri/src/commands/diff.rs, src-tauri/src/git/types.rs [HIGH confidence — direct code review]

---
*Feature landscape for: Trunk v0.7 — Hunk Staging & Commit Graph Search*
*Researched: 2026-03-17*
