# Stack Research: Trunk v0.7

**Domain:** v0.7 Hunk Staging & Search additions to Tauri 2 + Svelte 5 Git GUI
**Researched:** 2026-03-17
**Confidence:** HIGH

## Existing Stack (DO NOT change)

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Framework | Tauri 2 | 2.x | Desktop shell, IPC via `invoke`/`listen` |
| Frontend | Svelte 5 | 5.x | Vite SPA, runes (`$state`, `$derived`, `$effect`) |
| Styling | Tailwind CSS v4 | 4.x | Forced dark theme via CSS custom properties |
| Git backend | `git2` (libgit2) | 0.19 | `vendored-libgit2` feature, all local ops |
| Git CLI | subprocess | — | Remote ops (fetch/pull/push), cherry-pick/revert |
| Icons | `@lucide/svelte` | ^0.577 | SVG icon components (added in v0.6) |
| FS watching | `notify` + `notify-debouncer-mini` | 7 / 0.5 | 300ms debounce |
| State persistence | `tauri-plugin-store` | 2.4.2 | LazyStore for column widths, visibility |
| Async runtime | `tokio` | 1 | `process`, `io-util` features |
| Graph rendering | SVG overlay + virtual list | — | Rust lane algorithm + TS Active Lanes transform |

## New Dependencies

| Dependency | Version | Purpose | Why needed |
|-----------|---------|---------|------------|
| **None** | — | — | Both features are implementable with the existing stack. No new crates or npm packages required. |

## Core Finding: Zero New Dependencies

Both v0.7 features (hunk staging and commit graph search) require **zero new dependencies**. All APIs needed already exist in `git2 = "0.19"` and the existing frontend stack.

---

## Stack Decisions

### 1. Hunk Staging: `Repository::apply()` with `ApplyOptions::hunk_callback`

**Decision:** Use `git2`'s `apply()` + `ApplyLocation::Index` with a `hunk_callback` to selectively stage/unstage individual hunks.

**Key git2 0.19 APIs for hunk-level staging:**

#### Staging a hunk (workdir → index)

```rust
// 1. Get diff of workdir changes for a file (index vs workdir)
let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;

// 2. Apply only the selected hunk to the INDEX
let mut apply_opts = git2::ApplyOptions::new();
let target_hunk_idx: usize = hunk_index; // from frontend
let mut current_hunk: usize = 0;

apply_opts.hunk_callback(move |_hunk| {
    let dominated = current_hunk == target_hunk_idx;
    current_hunk += 1;
    dominated  // true = apply this hunk, false = skip
});

repo.apply(&diff, git2::ApplyLocation::Index, Some(&mut apply_opts))?;
```

**API methods used:**
- `Repository::diff_index_to_workdir()` — already used in `diff_unstaged_inner` (diff.rs:101)
- `Repository::apply(&diff, location, opts)` — applies a diff. `ApplyLocation::Index` modifies only the index (staging area), leaving the working directory untouched. This is exactly `git add -p` behavior.
- `ApplyOptions::hunk_callback(cb)` — called per hunk; return `true` to include, `false` to skip. This is the core filtering mechanism.
- `ApplyOptions::delta_callback(cb)` — called per file delta; can additionally filter by file path.

#### Unstaging a hunk (index → HEAD)

Unstaging is the reverse: apply the *inverse* diff (HEAD vs index) to the index.

```rust
// 1. Get diff of staged changes (HEAD tree vs index)
let head_tree = repo.head()?.peel_to_tree()?;
let diff = repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?;

// 2. Apply the selected hunk in REVERSE to remove it from the index
// Since diff is HEAD→index, applying it reversed (index→HEAD) unstages.
// However, git2's apply() doesn't have a reverse flag.
// Instead, use apply_to_tree on the index tree and reconstruct:

// Alternative approach for unstage — rebuild the index entry:
let index_tree_oid = repo.index()?.write_tree()?;
let index_tree = repo.find_tree(index_tree_oid)?;
let mut apply_opts = git2::ApplyOptions::new();
// ... hunk filter as above ...
let new_tree = repo.apply_to_tree(&index_tree, &reverse_diff, Some(&mut apply_opts))?;
// Read new_tree back into index
let mut index = repo.index()?;
index.read_tree(&new_tree)?;
index.write()?;
```

**Practical approach for unstage:** Generate the diff as `diff_tree_to_index(HEAD, index)`, then use `Diff::from_buffer()` to reverse the patch (swap +/- lines), or use `apply_to_tree()` on the HEAD tree with the forward diff filtered to just the target hunk, then read the resulting tree back into the index for that file path.

**Simpler alternative for unstage:** Use `Index::add_frombuffer()` to write a reconstructed file blob to the index. This is how `git reset -p` works internally — it reads the HEAD version + applies selected hunks to compute the desired index content, then writes that blob back.

**Recommended approach (both stage and unstage):**

1. **Stage hunk:** `repo.apply(&workdir_diff, ApplyLocation::Index, &apply_opts_with_hunk_filter)` — straightforward, one call.
2. **Unstage hunk:** Reconstruct the desired index content by reading the HEAD blob for the file, applying all hunks *except* the one being unstaged from the staged diff, then writing the result to the index via `Index::add_frombuffer()`. This avoids reverse-patch complexity.

**Confidence:** HIGH — `Repository::apply()`, `ApplyLocation::Index`, and `ApplyOptions::hunk_callback()` are all confirmed in git2 0.19.0 docs. The `hunk_callback` is specifically designed for selective hunk application (git add -p equivalent).

#### Unborn HEAD edge case

When HEAD is unborn (no commits yet), there's no HEAD tree to diff against. For unstaging:
- Use `diff_tree_to_index(None, None, opts)` — None tree means empty tree
- Same hunk-filter approach works; just treat "all content" as the diff

Already handled in `diff_staged_inner` (diff.rs:113-114) with the same pattern.

#### New file (untracked) hunks

Untracked files show as a single hunk (all lines are additions). Staging the single hunk is equivalent to staging the whole file — `Index::add_path()` suffices. The frontend should show the single hunk with a stage button, and the backend can detect this case and use the simpler path.

---

### 2. Commit Graph Search: Client-Side Filtering

**Decision:** Pure client-side filtering on the already-loaded `GraphCommit[]` array. No backend search command needed.

**Rationale:**
- The `CommitCache` in Rust state already holds the **full** `GraphResult` with all commits (graph.rs populates all commits via `walk_commits(repo, 0, usize::MAX)` in `refresh_commit_graph`).
- Each `GraphCommit` already contains: `oid`, `short_oid`, `summary`, `body`, `author_name`, `author_email`, and `refs: Vec<RefLabel>` (with `name`, `short_name`).
- The frontend already loads all commits into the `commits` $state array (CommitGraph.svelte:51).
- For a typical repo (10k commits), filtering is O(n) string matching — sub-millisecond in JavaScript.

**Search implementation:**
```typescript
// In CommitGraph.svelte or a new search module
const searchResults = $derived.by(() => {
  if (!searchQuery) return null;
  const q = searchQuery.toLowerCase();
  return commits
    .map((c, i) => ({ commit: c, index: i }))
    .filter(({ commit: c }) =>
      c.oid.startsWith(q) ||
      c.short_oid.startsWith(q) ||
      c.summary.toLowerCase().includes(q) ||
      (c.body?.toLowerCase().includes(q) ?? false) ||
      c.author_name.toLowerCase().includes(q) ||
      c.refs.some(r => r.short_name.toLowerCase().includes(q))
    );
});
```

**Why NOT a backend search command:**
- All data is already in memory (frontend has the full array, backend has the cache).
- A backend command would require IPC round-trips for each keystroke.
- Client-side filtering is instant for any reasonable repo size.
- The search needs to highlight/navigate within the virtual list, which is purely a frontend concern.
- No need for git2's `revwalk` or `git log --grep` — those search the ODB, but we already have all data loaded.

**Search UX integration:**
- Cmd+F opens a search bar overlaying the commit graph (standard pattern).
- Results are navigable with Enter/Shift+Enter (next/prev match).
- Current match scrolls into view using the existing `listRef.scroll()` API.
- Matched rows get visual highlighting (a CSS class on `CommitRow`).
- Escape dismisses the search bar.

**Performance:** For repos with 100k+ commits, consider debouncing input by ~100ms. For repos with 1M+ commits (extreme edge case), a backend command returning matching indices could be added later — but this is out of scope for v0.7.

---

## Integration Notes

### Hunk Staging: Rust Backend

**New Tauri commands (in staging.rs):**
```
stage_hunk_inner(path, file_path, hunk_index, state_map) -> Result<(), TrunkError>
unstage_hunk_inner(path, file_path, hunk_index, state_map) -> Result<(), TrunkError>
```

**Parameters:**
- `file_path: &str` — relative path of the file in the repo
- `hunk_index: usize` — zero-based index of the hunk within the file's diff

These follow the established `inner-fn` pattern. The frontend already has `DiffHunk` with `old_start`, `old_lines`, `new_start`, `new_lines` from the diff commands — the hunk index maps directly to the hunk order returned by `diff_index_to_workdir()`.

**Important:** The hunk index must correspond to the same diff the frontend is displaying. Since the diff is computed fresh each time and hunks are returned in file-order, the zero-based index is stable for a given file state.

### Hunk Staging: Frontend

**DiffPanel changes:**
- Add a "Stage Hunk" / "Unstage Hunk" button per hunk header (the `@@ ... @@` line).
- Button visibility depends on context: shown for workdir/staged diffs, hidden for commit diffs.
- After staging/unstaging a hunk, refresh both the diff display and the staging panel status.

**StagingPanel integration:**
- No changes needed to StagingPanel itself — it already calls `loadStatus()` after stage/unstage operations.
- The file-level stage/unstage buttons remain alongside hunk-level buttons.

### Commit Graph Search: Frontend

**New components/modules:**
- `SearchBar.svelte` — floating input bar with match count, next/prev navigation
- Search state: `$state` rune module or inline in CommitGraph.svelte

**CommitGraph.svelte changes:**
- Add keyboard listener for Cmd+F (Ctrl+F on non-macOS)
- Pass `highlightedOids: Set<string>` to CommitRow for visual highlighting
- Use `listRef.scroll()` to navigate to matched rows

**CommitRow.svelte changes:**
- Accept optional `isSearchMatch` prop for highlight styling

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `diffy` / `similar` / `imara-diff` crates | git2's `apply()` + `hunk_callback` handles hunk-level staging natively; no need for external diff/patch libraries | `git2::Repository::apply()` with `ApplyOptions` |
| `git2-patch` or custom patch parsing | git2 0.19 already has `Diff::from_buffer()` for parsing patches and `apply()` for applying them | Built-in `git2` APIs |
| `fuse.js` or `minisearch` (fuzzy search libs) | Commit graph search is exact substring/prefix matching; fuzzy search adds complexity without clear UX benefit for git hashes/messages | Simple `String.includes()` / `String.startsWith()` |
| Backend search command (Tauri `invoke`) | All commit data already loaded in frontend memory; IPC adds latency | Client-side `$derived` filtering |
| `regex` crate for Rust-side search | Search is client-side; if later needed, JS `RegExp` suffices | JavaScript string methods |
| New npm packages | No frontend library needed for either feature | Existing Svelte 5 reactivity + Tailwind |
| `git add -p` via CLI subprocess | Would require interactive stdin/stdout parsing; git2's `apply()` API is cleaner and testable | `git2::Repository::apply()` |

---

## git2 0.19 API Reference for v0.7

| API | Signature | Used For |
|-----|-----------|----------|
| `Repository::apply()` | `fn apply(&self, diff: &Diff, location: ApplyLocation, opts: Option<&mut ApplyOptions>) -> Result<(), Error>` | Stage/unstage hunks by applying filtered diffs |
| `ApplyLocation::Index` | enum variant | Apply diff to index only (staging area), not working directory |
| `ApplyOptions::hunk_callback()` | `fn hunk_callback<F>(&mut self, cb: F) -> &mut Self where F: FnMut(Option<DiffHunk>) -> bool` | Filter which hunks to include (true=include, false=skip) |
| `ApplyOptions::delta_callback()` | `fn delta_callback<F>(&mut self, cb: F) -> &mut Self where F: FnMut(Option<DiffDelta>) -> bool` | Filter which file deltas to include |
| `ApplyOptions::check()` | `fn check(&mut self, check: bool) -> &mut Self` | Dry-run mode — validate patch applies without modifying index |
| `Repository::apply_to_tree()` | `fn apply_to_tree(&self, tree: &Tree, diff: &Diff, opts: Option<&mut ApplyOptions>) -> Result<Index, Error>` | Apply diff to a tree object (useful for unstaging) |
| `Index::add_frombuffer()` | `fn add_frombuffer(&mut self, entry: &IndexEntry, data: &[u8]) -> Result<(), Error>` | Write computed file content directly to index (alternative unstage approach) |
| `Index::read_tree()` | `fn read_tree(&mut self, tree: &Tree) -> Result<(), Error>` | Replace index contents from tree (used with apply_to_tree result) |
| `Repository::diff_index_to_workdir()` | already used in diff.rs | Get workdir diff for stage-hunk |
| `Repository::diff_tree_to_index()` | already used in diff.rs | Get staged diff for unstage-hunk |

---

## Sources

- [git2 0.19.0 Repository::apply](https://docs.rs/git2/0.19/git2/struct.Repository.html#method.apply) — `apply()`, `apply_to_tree()` (HIGH confidence)
- [git2 0.19.0 ApplyOptions](https://docs.rs/git2/0.19/git2/struct.ApplyOptions.html) — `hunk_callback`, `delta_callback`, `check` (HIGH confidence)
- [git2 0.19.0 ApplyLocation](https://docs.rs/git2/0.19/git2/enum.ApplyLocation.html) — `WorkDir`, `Index`, `Both` variants (HIGH confidence)
- [git2 0.19.0 Index](https://docs.rs/git2/0.19/git2/struct.Index.html) — `add_frombuffer`, `read_tree`, `write` (HIGH confidence)
- Existing codebase: `diff.rs` (diff_unstaged_inner, diff_staged_inner), `staging.rs` (stage_file_inner, unstage_file_inner), `CommitGraph.svelte` (commits array, virtual list scroll), `types.ts` (DiffHunk, GraphCommit) (HIGH confidence)

---
*Stack research for: Trunk v0.7 Hunk Staging & Search*
*Researched: 2026-03-17*
