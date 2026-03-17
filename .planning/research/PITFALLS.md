# Pitfalls Research: Trunk v0.7

**Domain:** Hunk-level staging and commit graph search for a Tauri 2 + Svelte 5 + Rust desktop Git GUI
**Researched:** 2026-03-17
**Confidence:** HIGH — based on direct codebase analysis (staging.rs, diff.rs, graph.rs, CommitGraph.svelte, DiffPanel.svelte, VirtualList.svelte, App.svelte, watcher.rs), git2 0.19 API docs, and established patterns from v0.1–v0.6

---

## Context: What's Changing in v0.7

v0.6 shipped successfully: Lucide icons, toast system, discard/delete operations, three-way staging selector, unified title bar, graph polish. The codebase is now ~6k+ LOC Rust, ~5k+ LOC Svelte/TS with well-established patterns.

v0.7 adds two features that touch the core staging and graph subsystems:
1. **Hunk staging:** Stage/unstage individual diff hunks within a file (requires git2 index manipulation at the blob level)
2. **Commit graph search:** Find commits by hash, message, or branch name with Cmd+F (requires search across paginated/virtual commit list)

---

## Hunk Staging Pitfalls

### P1: git2 Index Blob Manipulation for Partial Staging

- **Risk:** The current `stage_file_inner` uses `index.add_path()` which stages the entire workdir file. Hunk staging requires a fundamentally different approach: read the HEAD blob (or empty for new files), apply selected hunk patches to produce a new blob, write that blob to the ODB, then update the index entry with the new blob OID. Getting any step wrong produces a corrupted index — the file appears staged but contains wrong content, leading to incorrect commits. The git2 API for this workflow is: `repo.blob(new_content)` → `IndexEntry { id: new_oid, path, ... }` → `index.add(&entry)` → `index.write()`. The `IndexEntry` struct has many fields (ctime, mtime, dev, ino, mode, uid, gid, file_size, id, path) and most must match the existing entry or git complains.
- **Prevention:** (1) For staging a hunk from unstaged: diff index-to-workdir, take the index blob as base, apply the selected hunk's lines to produce a new blob, write to ODB, update index entry with new blob OID and correct file_size. (2) For unstaging a hunk from staged: diff HEAD-to-index, take the HEAD blob as base, apply the inverse of the selected hunk, write new blob, update index. (3) Copy all other `IndexEntry` fields from the existing entry. (4) Always call `index.write()` after modification. (5) Unit test by staging one hunk, committing, then verifying the commit tree contains only the expected lines.
- **Phase:** Hunk staging backend (Rust command implementation)

### P2: Hunk Boundaries Shift After Staging One Hunk

- **Risk:** After staging hunk N of a file, the line numbers for all subsequent hunks in that file are invalidated. The diff between index and workdir has changed — what was hunk 3 at line 50 may now be at line 45 or may have merged with an adjacent hunk. If the frontend continues using the stale hunk data to stage another hunk, it applies the wrong lines. This is the single most common bug in hunk staging implementations.
- **Prevention:** (1) After every `stage_hunk` or `unstage_hunk` call, the frontend MUST re-fetch the diff for the file before allowing another hunk operation. The existing `refetchFileDiff` pattern in App.svelte (line 198-206) already does this for file-level operations. (2) On the backend, the `stage_hunk_inner` function should be atomic (stage one hunk, write index) — never accept a batch of hunks with pre-computed line numbers. (3) Disable hunk staging buttons with a loading state while the re-diff is in flight (reuse the `loadingFiles` Set pattern from StagingPanel.svelte). (4) Consider returning the updated diff from the staging command itself to avoid a round-trip.
- **Phase:** Hunk staging frontend (DiffPanel hunk actions)

### P3: Binary Files and "No Newline at EOF" Edge Cases

- **Risk:** (1) Binary files have no hunks — the current `walk_diff_into_file_diffs` skips the binary callback (`None` at diff.rs line 53). Attempting to hunk-stage a binary file would find zero hunks and fail silently or crash. (2) The `\ No newline at end of file` marker in unified diff is not a real diff line — it has origin `>` (GIT_DIFF_LINE_CONTEXT_EOFNL) or `<` or `=`, not `+`/`-`/` `. The current diff parser maps all non-`+`/`-` origins to `Context` (diff.rs line 72), which is correct for display but wrong for hunk application — the "no newline" marker means the preceding line should NOT have a trailing `\n`. If the hunk application code naively joins lines with `\n`, files that originally had no trailing newline will gain one (or vice versa).
- **Prevention:** (1) Guard hunk staging commands with `is_binary` check — return a clear error "Cannot partially stage binary files" if `delta.old_file().is_binary() || delta.new_file().is_binary()`. (2) When building the patched blob content, track the `GIT_DIFF_LINE_NOEOFNL` origin character and strip the trailing newline from the last line accordingly. (3) Test with files that have and don't have trailing newlines. (4) Test with a file that transitions from "has newline" to "no newline" in a hunk.
- **Phase:** Hunk staging backend (blob construction logic)

### P4: New (Untracked) File Hunk Staging — No HEAD Blob Exists

- **Risk:** For untracked files (`DiffStatus::Added`/`Untracked`), there's no existing blob in HEAD or the index to use as a base for patching. The entire file content is "new." Staging a single hunk of a new file means creating a blob that contains only the selected hunk's `+` lines, not the entire file. The index entry needs `mode: 0o100644` (or `0o100755` for executable), and there's no existing entry to copy fields from.
- **Prevention:** (1) For new files, the base is an empty string/blob. Apply the selected hunk's add-lines to produce partial content. (2) Build a fresh `IndexEntry` with: `id` = new blob OID, `mode` = file mode from `delta.new_file().mode()`, `path` = file path as bytes, and zero all stat fields (ctime, mtime, etc.) — git will update them on the next status check. (3) For unstaging a hunk from a partially-staged new file: if the result would be an empty blob, remove the index entry entirely (`index.remove_path()`). (4) Test: new file with 3 hunks, stage only hunk 2, verify index blob contains only hunk 2's lines.
- **Phase:** Hunk staging backend

### P5: Index Lock Contention Between Hunk Staging and Filesystem Watcher

- **Risk:** The filesystem watcher (watcher.rs) fires `repo-changed` every 300ms. When the user stages a hunk, the backend opens the repo, gets the index, modifies it, and calls `index.write()`. If a `repo-changed` event triggers `get_status` or `get_dirty_counts` at the same time, both operations call `repo.index()` — git2 acquires the index lock (`.git/index.lock`). If two operations race, one gets `ELOCKED` error: "failed to lock file '.git/index.lock'". The existing `stage_file_inner` has this same theoretical risk, but hunk staging is slower (read blob + apply patch + write blob + update index) so the window is wider.
- **Prevention:** (1) The current architecture opens a fresh `Repository` per command (RepoState stores PathBuf only, per Key Decision). This means each command gets its own index snapshot, but `index.write()` still uses the filesystem lock. (2) Option A: suppress the watcher during staging operations — call `stop_watcher`/`start_watcher` around the command. But this means the StagingPanel pauses the watcher while the command is running, then resumes — losing any external changes during that window. (3) Option B (recommended): catch `ELOCKED` errors and retry once after a short delay. The `TrunkError` already has structured error codes. (4) Option C: serialize all index-mutating commands through a Tokio mutex (separate from `RepoState`). This guarantees no concurrent index writes but adds complexity. (5) At minimum, the frontend should debounce or gate: don't auto-refresh status while a staging operation is in flight. The existing `loadSeq` pattern in StagingPanel.svelte (line 30) already guards against stale status results — extend it to skip `loadStatus` when a staging operation is pending.
- **Phase:** Hunk staging integration (watcher coordination)

### P6: Staged Diff Context for Hunk Unstaging Is HEAD-to-Index, Not Index-to-Workdir

- **Risk:** Staging and unstaging hunks require different diff bases. Staging uses the index→workdir diff (diff_unstaged_inner). Unstaging uses the HEAD→index diff (diff_staged_inner). If the backend uses the wrong diff direction, the hunk line numbers won't match the actual index content, and the patch application will produce corrupt data. The current DiffPanel already distinguishes these via `selectedFile.kind` ('unstaged' | 'staged'), but the hunk staging command must also receive this context.
- **Prevention:** (1) Create two separate commands: `stage_hunk` (applies workdir hunk to index) and `unstage_hunk` (reverts index hunk toward HEAD). (2) The frontend passes the file path AND the hunk index (or old_start/old_lines/new_start/new_lines header) to identify which hunk. (3) The backend re-diffs to find the exact hunk rather than trusting frontend-provided line numbers (which may be stale). (4) Test: modify a file, stage half via hunk staging, then unstage one of the staged hunks — verify the final index state matches expectations.
- **Phase:** Hunk staging backend (separate stage/unstage commands)

---

## Search Pitfalls

### P7: Cmd+F Conflicts with WebView's Built-in Find

- **Risk:** On macOS, Cmd+F triggers the WebView's built-in text search (browser-style find bar). Tauri 2 uses WKWebView on macOS, which has a native find bar that overlays the web content. If the app adds its own Cmd+F handler for commit search, both the native find bar AND the custom search UI appear simultaneously. The native find bar searches the rendered HTML text, which is useless for a virtualized list (most commits aren't in the DOM).
- **Prevention:** (1) Intercept Cmd+F at the JavaScript level with `e.preventDefault()` in a `keydown` handler — this MUST be registered on `window` and fire before the WebView processes it. The existing keyboard handling in App.svelte (lines 268-293) already uses this pattern for Cmd+=, Cmd+-, etc. (2) On macOS WKWebView, `e.preventDefault()` in `keydown` may NOT suppress the native find bar — it depends on the Tauri version and WebView configuration. Test this early. (3) If `preventDefault` doesn't work, disable the native find behavior by adding `"devtools": false` to the webview config (but this loses dev tools) or by intercepting at the Rust/native level using `on_webview_event`. (4) Alternative: use a different shortcut (Cmd+K for command palette style, or Cmd+G for go-to-commit). This avoids the conflict entirely but breaks user expectations. (5) The existing `tauri.conf.json` has `titleBarStyle: "Overlay"` and `hiddenTitle: true` — check if there's a WebView configuration option to disable built-in find.
- **Phase:** Search frontend (keyboard shortcut registration) — test this FIRST before building the search UI

### P8: Search Performance with Large Commit Histories (10k+ Commits)

- **Risk:** The commit graph loads 200 commits per batch. Searching for a commit message substring requires checking ALL commits, not just the loaded ones. Options: (1) search only loaded commits (fast but incomplete — user may not find what they need), (2) search on the backend across the full history (complete but potentially slow — linear scan of 100k+ commit messages), (3) load all commits then search in JS (memory explosion on huge repos). For SHA prefix search, the backend can use `repo.revparse_single(prefix)` which is O(1) via git's fanout table. For message/author search, there's no index — it's always a linear scan.
- **Prevention:** (1) Implement backend search command `search_commits(path, query, limit)` that walks the revwalk and checks each commit's summary/body/oid/refs against the query. Return first N matches (e.g., 50) to avoid serializing thousands of results. (2) For OID prefix search: use `git2::Repository::revparse_single(query)` first — if it resolves, return that single commit immediately. (3) For message search: walk commits with `revwalk` and check `commit.summary()` and `commit.body()` with case-insensitive substring match. Stop after `limit` matches. (4) Stream results: return matches incrementally (or paginated) so the UI can show results as they arrive. (5) Add a `cancel` mechanism: if the user types a new character while search is running, abort the old search. Use a sequence counter (existing `loadSeq` pattern) or `AbortController`-style cancellation. (6) Benchmark: git2's `revwalk` + `find_commit` on 100k commits takes ~200-500ms. The bottleneck is `find_commit` per OID (disk I/O). Caching commit summaries in memory (from the graph walk) would make re-searches instant.
- **Phase:** Search backend (Rust command) — benchmark with a 10k+ commit repo early

### P9: Search Results Must Navigate the Virtual Scroll Position

- **Risk:** When the user selects a search result, the commit graph must scroll to that row. The target commit may be: (1) already loaded and visible — scroll directly, (2) loaded but off-screen — scroll via VirtualList, (3) not yet loaded — need to load more batches first. The existing `scrollToOid` method in CommitGraph.svelte (line 569-594) already handles cases 1-3 with a load-until-found loop. But search introduces a new pattern: the user may jump between multiple search results rapidly (next/previous match). Each jump may trigger a load-more cascade, causing UI jank.
- **Prevention:** (1) Reuse `scrollToOid` for search navigation — it already handles the load-and-scroll pattern. (2) Cache search results as an array of OIDs. Navigation is just indexing into this array and calling `scrollToOid`. (3) Pre-load: when search returns N results, the frontend doesn't need to load all of them immediately. Load on-demand as the user navigates. (4) Debounce navigation: if the user holds Cmd+G (next match) rapidly, debounce the scroll calls to avoid triggering multiple concurrent `loadMore` cascades. (5) Show the match count and current position ("3 of 17 matches") so the user knows there are results even if scrolling takes a moment. (6) Highlight the matched row — add a `searchMatch` flag or `searchHighlightOid` state to CommitRow, distinct from `selected` (the user may search without wanting to select/load commit detail).
- **Phase:** Search frontend (navigation + virtual list integration)

### P10: Search Highlight in Virtualized SVG Overlay

- **Risk:** The commit graph renders rows via VirtualList with an SVG overlay for dots/rails/pills. Highlighting a search-matched row requires either: (1) a CSS highlight on the CommitRow HTML element, (2) an SVG highlight rect in the overlay, or (3) both. The VirtualList only renders ~40 DOM nodes. If a search match is outside the visible range, there's no DOM element to highlight — the highlight must be applied reactively when the row scrolls into view. The existing `selected` prop on CommitRow (CommitGraph.svelte line 997) already handles this pattern for commit selection.
- **Prevention:** (1) Follow the existing `selected` pattern: pass a `searchMatches: Set<string>` (OIDs) to CommitRow and apply a highlight style when `searchMatches.has(commit.oid)`. (2) Don't try to highlight in the SVG overlay — it's unnecessary visual complexity. The row background highlight is sufficient. (3) For the "current match" (the one the user navigated to), use a stronger highlight (e.g., border or brighter background) distinct from "other matches" (subtle background). (4) The `displayItems` derived store already includes all rendered items. Adding a `searchMatches` check is O(1) per rendered row.
- **Phase:** Search frontend (result highlighting)

---

## Integration Pitfalls

### P11: Stale Diff Data After Staging a Hunk — Watcher vs Manual Refresh Race

- **Risk:** When the user stages a hunk, the index changes but the workdir file does NOT change. The filesystem watcher watches the workdir, not `.git/index`. So the `repo-changed` event does NOT fire after `index.write()`. The frontend's diff display becomes stale — it still shows the old hunks. The user sees the hunk they just staged still listed as unstaged. The existing whole-file staging works around this because `stage_file` is followed by an explicit `loadStatus()` call in StagingPanel.svelte (line 47). For hunk staging, the same pattern is needed but for the DiffPanel.
- **Prevention:** (1) After `stage_hunk` succeeds, the frontend must: (a) re-fetch the diff for the file (`refetchFileDiff`), (b) reload the staging status (`loadStatus`), and (c) reload dirty counts (`loadDirtyCounts`). (2) The current App.svelte `repo-changed` listener (line 219-236) already calls `handleRefresh()` + `loadDirtyCounts()` + `refetchFileDiff()` — but this only fires on workdir changes, not index changes. (3) Option A: emit a custom `index-changed` event from the Rust backend after any `index.write()`. The frontend listens for both `repo-changed` and `index-changed`. (4) Option B (simpler): have the staging command return a signal, and the frontend refreshes manually after the command returns (like the current `stageFile` → `loadStatus` pattern). (5) The watcher ignoring `.git/` directory changes is correct (watching `.git/` would cause infinite loops from index writes). Don't try to watch `.git/index`.
- **Phase:** Hunk staging integration (frontend refresh flow)

### P12: Hunk Staging UI Must Coexist with Whole-File Staging

- **Risk:** The existing StagingPanel shows file-level `+`/`-` buttons for stage/unstage. Hunk staging adds per-hunk buttons in the DiffPanel. Both must work simultaneously and correctly. Edge cases: (1) User stages hunk 1 via DiffPanel, then clicks whole-file stage via StagingPanel `+` button — should stage remaining hunks. (2) User stages all hunks individually — file should move from unstaged to staged list (same as whole-file stage). (3) User has partially-staged file (some hunks staged, some not) — file should appear in BOTH unstaged and staged lists in StagingPanel. The current `get_status_inner` already handles this: a file with `INDEX_MODIFIED | WT_MODIFIED` appears in both lists.
- **Prevention:** (1) Whole-file staging (`stage_file_inner` with `index.add_path()`) must continue to work — it stages the entire workdir version, overwriting any partial staging. This is correct behavior. (2) The partially-staged state (file in both lists) is already supported by `classify_index` + `classify_workdir` in staging.rs — both can return Some for the same file. (3) The DiffPanel needs to know whether it's showing the staged or unstaged diff to show the correct action button ("Stage Hunk" vs "Unstage Hunk"). The `selectedFile.kind` already carries this info. (4) When a file is partially staged, clicking it in the unstaged list shows remaining unstaged hunks; clicking it in the staged list shows the staged hunks. Each view gets different hunk actions. (5) Test the full flow: partial stage via hunks → verify both lists → whole-file unstage → verify all hunks return to unstaged.
- **Phase:** Hunk staging UI (DiffPanel buttons + StagingPanel interaction)

### P13: DiffPanel Needs Hunk-Level Action Buttons Without Breaking Commit Diff View

- **Risk:** The DiffPanel currently serves three purposes: (1) unstaged file diff, (2) staged file diff, (3) commit diff (read-only). Hunk staging buttons should appear only for cases 1 and 2, never for case 3. The DiffPanel receives `fileDiffs` and `commitDetail` as props. When `commitDetail` is non-null, it's showing a commit diff (no staging actions). When `selectedPath` matches a staging file, it should show hunk actions. But the DiffPanel currently has no `kind` prop to distinguish unstaged from staged — it relies on the parent to pass the correct diffs.
- **Prevention:** (1) Add a `diffKind: 'unstaged' | 'staged' | 'commit'` prop to DiffPanel. Show "Stage Hunk" button when `diffKind === 'unstaged'`, "Unstage Hunk" button when `diffKind === 'staged'`, no buttons when `diffKind === 'commit'`. (2) Pass an `onstagethunk` callback prop from App.svelte that calls the backend command and triggers refresh. (3) The hunk button should appear in the hunk header row (next to the `@@ ... @@` line) — this is the standard UX from VS Code, GitKraken, etc. (4) Keep the existing DiffPanel simple: it receives data and renders. Hunk action logic lives in the parent (App.svelte) via callbacks.
- **Phase:** Hunk staging UI (DiffPanel component extension)

### P14: Cmd+F Search Bar Overlapping Title Bar or Graph Header

- **Risk:** The search bar UI needs to appear somewhere when Cmd+F is pressed. Common placement: a floating bar at the top of the commit graph area. But the graph already has a 24px header row (column labels) and the unified title bar above that. A search bar that overlays these elements will be partially hidden behind the macOS traffic lights (the title bar has `titleBarStyle: "Overlay"` with ~78px left padding). If the search bar pushes content down, the virtual list's scroll position and height calculations break.
- **Prevention:** (1) Render the search bar as an absolutely-positioned element INSIDE the commit graph container, below the column header, overlaying the first few rows of the virtual list. Use `z-index: 10` (above SVG overlay's `z-index: 1`). (2) Don't push the virtual list down — keep it the same size. The search bar floats on top. This avoids VirtualList height recalculation. (3) Position: `top: 24px` (below column header), `right: 0` (right-aligned to avoid traffic light area). Width: ~300px. (4) Include: text input, match count ("3 of 17"), prev/next buttons, close button (Escape). (5) The search bar should have a semi-transparent background so the user can still see graph content behind it.
- **Phase:** Search UI (CommitGraph search bar component)

---

## Summary

**Top 3 things to watch out for:**

1. **Hunk boundaries invalidate after each staging operation (P2).** This is the most likely source of data corruption bugs. Every `stage_hunk`/`unstage_hunk` call MUST be followed by a full re-diff of the file before the next hunk operation. Never batch hunk operations using pre-computed line numbers.

2. **Cmd+F conflicts with WebView's native find bar (P7).** On macOS WKWebView, `preventDefault()` may not suppress the native find bar. Test this before building any search UI — if it doesn't work, the workaround may require native Rust-level event interception or a different shortcut.

3. **Index changes don't trigger the filesystem watcher (P11).** After hunk staging, the workdir file is unchanged — only `.git/index` is modified. The `repo-changed` event won't fire. The frontend must explicitly refresh diffs, status, and dirty counts after every staging command, or the UI will show stale data.

---

*Pitfalls research for: Trunk v0.7 — Hunk Staging & Search*
*Researched: 2026-03-17*
