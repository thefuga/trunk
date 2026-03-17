# Architecture Research: Trunk v0.7 — Hunk Staging & Commit Graph Search

**Domain:** Desktop Git GUI — hunk-level staging and commit graph search integration
**Researched:** 2026-03-17
**Confidence:** HIGH (based on full codebase audit of all Rust commands, Svelte components, types, and IPC patterns)

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

---

## Feature 1: Hunk-Level Staging

### Problem Analysis

Current staging operates at whole-file granularity:
- `stage_file_inner` → `index.add_path(file_path)` — stages entire file
- `unstage_file_inner` → `index.remove_path` or `repo.reset_default` — unstages entire file
- DiffPanel renders hunks with headers + lines but has no interactive elements per hunk

git2 does NOT provide a `stage_hunk` API. Hunk staging requires manually applying a patch to the index. Two approaches exist:

### Approach A: git2 Index Manipulation (Recommended)

Build a partial patch from the hunk data and apply it to the index using `git2::Repository::apply()` (available since git2 0.17).

```rust
// Signature for stage_hunk_inner
pub fn stage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: usize,        // which hunk (0-based) within the file's diff
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError>
```

**Implementation strategy:**

1. Compute the workdir-to-index diff for the file (same as `diff_unstaged_inner`)
2. Walk the diff to isolate the target hunk by index
3. Build a patch buffer containing only that hunk (with the correct unified diff header)
4. Apply the patch to the index using `repo.apply(&diff_for_hunk, ApplyLocation::Index, None)`

The key insight: `git2::Repository::apply()` can apply a `Diff` object to the index (`ApplyLocation::Index`). We need to construct a `Diff` from the single-hunk patch text. git2 provides `Diff::from_buffer()` for this.

```rust
use git2::{ApplyLocation, Diff};

pub fn stage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: usize,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    
    // Get the full workdir→index diff for this file
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file_path);
    let full_diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;
    
    // Build a patch containing only the target hunk
    let patch_buf = extract_hunk_patch(&full_diff, file_path, hunk_index)?;
    
    // Parse the patch back into a Diff object
    let hunk_diff = Diff::from_buffer(&patch_buf)?;
    
    // Apply to the index (not workdir)
    repo.apply(&hunk_diff, ApplyLocation::Index, None)?;
    
    Ok(())
}
```

**Unstage hunk** follows the inverse pattern:

```rust
pub fn unstage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: usize,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    
    // Get the HEAD→index diff (what's staged)
    let head_tree = repo.head()?.peel_to_tree()?;
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file_path);
    let full_diff = repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut opts))?;
    
    // Build a REVERSED patch for the target hunk
    let patch_buf = extract_hunk_patch_reversed(&full_diff, file_path, hunk_index)?;
    
    let hunk_diff = Diff::from_buffer(&patch_buf)?;
    repo.apply(&hunk_diff, ApplyLocation::Index, None)?;
    
    Ok(())
}
```

**Helper: `extract_hunk_patch`** — walks the diff callbacks, counts hunks for the target file, and when the target hunk is reached, builds a valid unified diff patch string:

```
--- a/{file_path}
+++ b/{file_path}
@@ -{old_start},{old_lines} +{new_start},{new_lines} @@{header_suffix}
 context line
-removed line
+added line
```

### Approach B: git CLI Subprocess (Fallback)

Use `git apply --cached` with a patch file. This matches the pattern used for remote operations (git CLI subprocess with `GIT_TERMINAL_PROMPT=0`). The patch must be written to a temp file or piped via stdin.

**Trade-off:** Approach A is pure Rust (testable via inner-fn pattern, no subprocess), consistent with all local operations using git2. Approach B is simpler to implement but introduces a new pattern (git CLI for local ops). **Recommend Approach A.**

### Approach A Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| `Diff::from_buffer()` may reject malformed patches | Extensive unit tests with various diff shapes (add-only, delete-only, mixed, binary, no-newline-at-eof) |
| `repo.apply()` may fail on edge cases (binary files, rename diffs) | Disable hunk buttons for binary files (already detected: `fd.is_binary`). Rename diffs: fall back to whole-file staging |
| Hunk index may become stale if file changes between diff fetch and stage action | Re-fetch diff immediately before staging, validate hunk exists. Show toast if stale |
| Unborn HEAD (no commits yet) for unstage_hunk | Use `diff_tree_to_index(None, ...)` when HEAD is unborn, same as existing `diff_staged_inner` pattern |

### New Rust Commands

| Command | Inner Function | Module | Purpose |
|---------|---------------|--------|---------|
| `stage_hunk` | `stage_hunk_inner(path, file_path, hunk_index, state_map)` | `staging.rs` | Stage a single hunk from workdir→index diff |
| `unstage_hunk` | `unstage_hunk_inner(path, file_path, hunk_index, state_map)` | `staging.rs` | Unstage a single hunk from HEAD→index diff |

### DiffPanel Changes

Current DiffPanel (`src/components/DiffPanel.svelte`) renders hunks as read-only with no interactivity. Changes needed:

1. **New props on DiffPanel:**
   ```typescript
   interface Props {
     fileDiffs: FileDiff[];
     commitDetail: CommitDetail | null;
     selectedPath?: string | null;
     onclose: () => void;
     // NEW for hunk staging:
     diffKind?: 'unstaged' | 'staged' | 'commit';
     onHunkAction?: (filePath: string, hunkIndex: number) => void;
   }
   ```

2. **Hunk action button** — rendered in the hunk header row (the `@@ ... @@` line), positioned to the right:
   - For `diffKind='unstaged'`: green "Stage Hunk" button (or `+` icon)
   - For `diffKind='staged'`: red "Unstage Hunk" button (or `-` icon)
   - For `diffKind='commit'`: no button (read-only)
   - Button hidden for binary files (`fd.is_binary`)

3. **Hunk header markup change:**
   ```svelte
   <!-- Current -->
   <div style="...">{hunk.header}</div>

   <!-- New -->
   <div style="...; display: flex; align-items: center;">
     <span style="flex: 1;">{hunk.header}</span>
     {#if diffKind && diffKind !== 'commit' && !fd.is_binary}
       <button onclick={() => onHunkAction?.(fd.path, hunkIdx)}
         style="... background: {diffKind === 'unstaged' ? '#22c55e' : '#f87171'}; ..."
       >
         {diffKind === 'unstaged' ? 'Stage Hunk' : 'Unstage Hunk'}
       </button>
     {/if}
   </div>
   ```

4. **Hunk index tracking:** The `{#each fd.hunks as hunk, hunkIdx}` loop already provides the index via the second parameter of `#each`. Pass `hunkIdx` to the action callback.

### App.svelte Wiring

Current file selection flow in App.svelte:

```
handleFileSelect(path, kind='unstaged'|'staged')
  → sets selectedFile = { path, kind }
  → fetches diff via 'diff_unstaged' or 'diff_staged'
  → sets stagingDiffFiles
  → DiffPanel renders with fileDiffs={stagingDiffFiles}
```

**New wiring for hunk actions:**

```typescript
async function handleHunkAction(filePath: string, hunkIndex: number) {
  if (!repoPath || !selectedFile) return;
  const command = selectedFile.kind === 'unstaged' ? 'stage_hunk' : 'unstage_hunk';
  try {
    await safeInvoke(command, { path: repoPath, filePath, hunkIndex });
    // Re-fetch the diff to show updated hunk state
    await refetchFileDiff(selectedFile.path, selectedFile.kind);
    // Trigger status refresh (file may have moved between staged/unstaged)
  } catch (e) {
    const err = e as TrunkError;
    showToast(err.message ?? 'Hunk staging failed', 'error');
  }
}
```

Pass to DiffPanel:
```svelte
<DiffPanel
  fileDiffs={currentDiffFiles}
  commitDetail={null}
  selectedPath={selectedCommitFile ?? selectedFile?.path ?? null}
  onclose={handleDiffClose}
  diffKind={selectedFile?.kind ?? (selectedCommitFile ? 'commit' : undefined)}
  onHunkAction={handleHunkAction}
/>
```

### Data Flow: Stage Hunk

```
User clicks "Stage Hunk" button on hunk header in DiffPanel
  → onHunkAction(filePath, hunkIndex) callback to App.svelte
  → safeInvoke('stage_hunk', { path: repoPath, filePath, hunkIndex })
  → Rust: extract_hunk_patch() → Diff::from_buffer() → repo.apply(Index)
  → Return Ok(())
  → Frontend: re-fetch diff for the file (diff may now have fewer hunks)
  → Filesystem watcher detects index change → 'repo-changed' event
  → StagingPanel re-fetches status (file may appear in both staged + unstaged)
  → CommitGraph: no refresh needed (no commit graph change)
```

### Data Flow: Unstage Hunk

```
User clicks "Unstage Hunk" on staged file's hunk in DiffPanel
  → onHunkAction(filePath, hunkIndex) callback to App.svelte
  → safeInvoke('unstage_hunk', { path: repoPath, filePath, hunkIndex })
  → Rust: extract reversed hunk patch → repo.apply(Index)
  → Return Ok(())
  → Frontend: re-fetch diff, StagingPanel auto-refreshes
```

---

## Feature 2: Commit Graph Search (Cmd+F)

### Problem Analysis

Current CommitGraph supports:
- Virtual scrolling with `VirtualList` (renders ~40 DOM nodes for any history size)
- `displayItems` array in memory (all loaded commits, paginated via `loadMore()`)
- `CommitCache` in Rust stores full `GraphResult` per repo (all commits, computed by `walk_commits`)
- `scrollToOid(oid)` already exists — scrolls graph to a specific commit

Search needs to find commits matching a query (SHA prefix, message substring, branch name) and navigate to results.

### Backend vs Client-Side Search

**Option A: Backend search command (Recommended)**

CommitCache already holds the full `GraphResult` with all `GraphCommit` structs. A backend search command can scan this in O(n) with zero serialization overhead for the scan itself, returning only matching OIDs.

```rust
#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub oid: String,
    pub match_type: String,  // "sha", "message", "ref"
}

pub fn search_commits_inner(
    path: &str,
    query: &str,
    cache_map: &HashMap<String, GraphResult>,
) -> Result<Vec<SearchResult>, TrunkError> {
    let graph = cache_map.get(path)
        .ok_or_else(|| TrunkError::new("repo_not_open", "Repository not open"))?;
    
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();
    
    for commit in &graph.commits {
        // Match by SHA prefix
        if commit.oid.starts_with(&query_lower) || commit.short_oid.starts_with(&query_lower) {
            results.push(SearchResult {
                oid: commit.oid.clone(),
                match_type: "sha".to_string(),
            });
            continue;
        }
        
        // Match by message (summary + body)
        if commit.summary.to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                oid: commit.oid.clone(),
                match_type: "message".to_string(),
            });
            continue;
        }
        if let Some(body) = &commit.body {
            if body.to_lowercase().contains(&query_lower) {
                results.push(SearchResult {
                    oid: commit.oid.clone(),
                    match_type: "message".to_string(),
                });
                continue;
            }
        }
        
        // Match by ref name
        for ref_label in &commit.refs {
            if ref_label.short_name.to_lowercase().contains(&query_lower) {
                results.push(SearchResult {
                    oid: commit.oid.clone(),
                    match_type: "ref".to_string(),
                });
                break;
            }
        }
    }
    
    Ok(results)
}
```

**Why backend over client-side:**
- CommitCache holds ALL commits (even those not yet loaded into the frontend `commits` array due to pagination). Frontend `displayItems` only has loaded batches.
- Backend scan is O(n) on owned Rust strings — no serialization/deserialization overhead for the scan.
- Client-side search would require loading ALL commits first (defeating pagination), or would miss unloaded commits.
- The existing `CommitCache` lock is brief (search is read-only, no mutation needed).

**Option B: Client-side search (NOT recommended)**

Would require: (a) loading all commit batches first via repeated `loadMore()`, or (b) searching only loaded commits (incomplete results). Also, author name search would need all commits deserialized to JS. Backend search is strictly better.

### New Rust Command

| Command | Inner Function | Module | Purpose |
|---------|---------------|--------|---------|
| `search_commits` | `search_commits_inner(path, query, cache_map)` | `history.rs` | Search CommitCache for matching commits |

Registration in `lib.rs`: `commands::history::search_commits`

**Note:** The command reads from `CommitCache` (not `RepoState`), so it only needs the `cache: State<'_, CommitCache>` parameter — same pattern as `get_commit_graph`.

### Search UI Component

New component: `src/components/SearchBar.svelte`

```
┌──────────────────────────────────────────┐
│  🔍 Search commits...    [2/15] [↑] [↓]  │
└──────────────────────────────────────────┘
```

**State:**
```typescript
let query = $state('');
let results = $state<SearchResult[]>([]);
let currentIndex = $state(0);
let visible = $state(false);
```

**Behavior:**
- `Cmd+F` toggles visibility (keyboard listener in CommitGraph or App.svelte)
- Debounced search: 200ms after typing stops, invoke `search_commits`
- Results are OIDs with match type
- Up/Down arrows or Enter cycle through results
- Each navigation calls `commitGraphRef.scrollToOid(results[currentIndex].oid)` + selects the commit
- `Escape` closes search bar
- Result count badge shows "N/M" (current index / total matches)

**Positioning:** Overlaid at top of CommitGraph area (below the column headers), absolute positioned with z-index above the graph content. Does NOT replace the graph — it floats on top.

### Search Integration with Virtual Scrolling

The key question: how does search navigate to results that aren't loaded yet?

**Answer:** `scrollToOid(oid)` already handles this. Looking at CommitGraph.svelte:569-593:

```typescript
export async function scrollToOid(oid: string): Promise<void> {
  let idx = displayItems.findIndex(c => c.oid === oid);
  // Load more batches until found or all commits exhausted
  while (idx < 0 && hasMore && !loading) {
    await loadMore();
    await tick();
    idx = displayItems.findIndex(c => c.oid === oid);
  }
  if (idx < 0 || !listRef) return;
  // Smooth scroll to center the row
  ...
}
```

This already lazy-loads batches until the target OID is found, then scrolls to it. Search navigation simply calls this existing method. No changes needed to `VirtualList.svelte` or the scroll machinery.

### Search Data Flow

```
User presses Cmd+F
  → SearchBar becomes visible, input focused
  → User types query (debounced 200ms)
  → safeInvoke('search_commits', { path: repoPath, query })
  → Rust: scan CommitCache, return Vec<SearchResult>
  → Frontend: set results, currentIndex = 0
  → Auto-navigate to first result: scrollToOid(results[0].oid)
  → CommitGraph loads batches if needed, scrolls to commit row
  → Commit row highlighted (selectedCommitOid set via handleCommitSelect)

User presses ↓ (next result) or ↑ (prev result)
  → currentIndex = (currentIndex + 1) % results.length
  → scrollToOid(results[currentIndex].oid)
  → handleCommitSelect(results[currentIndex].oid)

User presses Escape
  → SearchBar hidden, results cleared
  → Optional: clear commit selection or keep current
```

### Search UI Placement Decision

Two options:

**Option A: Inside CommitGraph (recommended)**
- SearchBar is a child of CommitGraph.svelte, positioned absolutely below the header row
- Cmd+F handler lives in CommitGraph
- Direct access to `scrollToOid` and `handleCommitSelect` via internal state
- No additional App.svelte wiring needed for scroll

**Option B: In App.svelte above CommitGraph**
- SearchBar is a sibling of CommitGraph
- Requires forwarding results to CommitGraph via props
- More separation of concerns but more prop plumbing

**Recommend Option A** — search is tightly coupled to the graph (navigation, highlighting, scrolling). Keeping it inside CommitGraph reduces prop drilling and makes the feature self-contained.

### Selected Commit Highlighting During Search

Current highlighting: `selectedCommitOid` in App.svelte is passed to CommitGraph, which passes `selected={commit.oid === selectedCommitOid}` to each CommitRow. This already works for search — navigating to a result calls `handleCommitSelect(oid)` which sets `selectedCommitOid`.

**Additional UX for multi-result awareness:** Optionally dim all non-matching rows or add a subtle marker on all matching rows. This is a stretch goal — basic search with navigation is the MVP.

---

## New Rust Commands (Combined)

| Command | Inner Function | Module | State Needed | Purpose |
|---------|---------------|--------|-------------|---------|
| `stage_hunk` | `stage_hunk_inner(path, file_path, hunk_index, state_map)` | `staging.rs` | `RepoState` | Stage a single diff hunk |
| `unstage_hunk` | `unstage_hunk_inner(path, file_path, hunk_index, state_map)` | `staging.rs` | `RepoState` | Unstage a single diff hunk |
| `search_commits` | `search_commits_inner(path, query, cache_map)` | `history.rs` | `CommitCache` | Search commit graph |

**Registration additions to `lib.rs`:**
```rust
commands::staging::stage_hunk,
commands::staging::unstage_hunk,
commands::history::search_commits,
```

## Modified Components

| Component | Changes | Risk |
|-----------|---------|------|
| **DiffPanel.svelte** | Add `diffKind` and `onHunkAction` props; render hunk action buttons in header rows | Low-Medium |
| **App.svelte** | Add `handleHunkAction` function, pass `diffKind`/`onHunkAction` to DiffPanel, add Cmd+F handler (if search is in App) | Low |
| **CommitGraph.svelte** | Add SearchBar child component, Cmd+F keyboard handler, search state management | Medium |

## New Components

| Component | Purpose |
|-----------|---------|
| `src/components/SearchBar.svelte` | Floating search input with result count and navigation buttons |

## New Rust Types

```rust
// In git/types.rs or inline in history.rs
#[derive(Debug, Serialize, Clone)]
pub struct SearchResult {
    pub oid: String,
    pub match_type: String,  // "sha", "message", "ref"
}
```

```typescript
// In lib/types.ts
export interface SearchResult {
  oid: string;
  match_type: 'sha' | 'message' | 'ref';
}
```

## Integration Points

### Hunk Staging Integration Points

1. **staging.rs ↔ diff.rs**: `stage_hunk_inner` reuses the same diff computation as `diff_unstaged_inner` (calls `repo.diff_index_to_workdir`). Could extract shared helper.

2. **DiffPanel ↔ App.svelte**: New callback prop (`onHunkAction`) follows same pattern as `onclose`. App.svelte orchestrates the IPC call and diff re-fetch.

3. **Filesystem watcher**: After `stage_hunk`, the index changes on disk. The filesystem watcher (watching `.git/index` changes via `notify`) fires `repo-changed`, which triggers StagingPanel to re-fetch status. This is the existing cache-repopulate-before-emit pattern — but for hunk staging, we may NOT need cache rebuild (no commit graph change). The `repo-changed` event is sufficient for StagingPanel refresh.

4. **DiffPanel re-render**: After staging a hunk, the diff is re-fetched. If only 1 hunk existed, the file moves fully to staged → `selectedFile` may need to be cleared or auto-switch to show the staged diff. Edge case to handle in `handleHunkAction`.

### Search Integration Points

1. **CommitCache**: Search reads from `CommitCache` (same Mutex lock as `get_commit_graph`). The lock is held briefly for a read-only scan. No contention concerns — search is read-only.

2. **scrollToOid**: Already exists in CommitGraph (line 569). Search navigation calls this directly. Handles lazy-loading batches automatically.

3. **selectedCommitOid**: Search navigation sets this via `handleCommitSelect(oid)`, which triggers right-pane CommitDetail display. This is the existing behavior — search reuses it.

4. **Cmd+F global keyboard handler**: Currently App.svelte has keyboard handlers for Cmd+J (toggle left pane), Cmd+K (toggle right pane), Cmd+0/+/- (zoom). Adding Cmd+F follows the same pattern. If search lives inside CommitGraph, the handler should be in CommitGraph (listening on `window` or the component's DOM element).

## Suggested Build Order

### Phase 1: Hunk Staging Backend
**Dependencies:** None
**Deliverables:**
1. `extract_hunk_patch` helper function in `staging.rs`
2. `stage_hunk_inner` function with unit tests
3. `unstage_hunk_inner` function with unit tests
4. `stage_hunk` / `unstage_hunk` Tauri commands registered in `lib.rs`

**Why first:** The backend is the riskiest part (git2 `Diff::from_buffer` + `repo.apply` may have edge cases). Get it working and tested before touching UI.

**Test strategy:**
```rust
#[test]
fn stage_hunk_stages_only_target_hunk() {
    // Create repo with file having 2+ distinct hunks
    // Stage hunk 0 only
    // Verify staged diff has 1 hunk, unstaged diff has remaining hunks
}

#[test]
fn unstage_hunk_unstages_only_target_hunk() {
    // Stage entire file, then unstage hunk 0
    // Verify staged diff has N-1 hunks, unstaged diff has 1 hunk
}

#[test]
fn stage_hunk_binary_file_error() {
    // Binary file → should return appropriate error
}

#[test]
fn stage_hunk_stale_index_error() {
    // Hunk index out of range → should return error, not panic
}
```

### Phase 2: Hunk Staging UI
**Dependencies:** Phase 1
**Deliverables:**
1. DiffPanel: add `diffKind` and `onHunkAction` props
2. DiffPanel: render "Stage Hunk" / "Unstage Hunk" buttons in hunk headers
3. App.svelte: add `handleHunkAction` function, pass props to DiffPanel
4. App.svelte: handle edge case where staging last hunk clears the diff view

**Test strategy:** Manual testing with files containing multiple hunks. Verify:
- Button appears only for unstaged/staged diffs (not commit diffs)
- Button hidden for binary files
- Staging a hunk re-renders with fewer hunks
- Staging last hunk clears the diff or switches view

### Phase 3: Search Backend
**Dependencies:** None (parallel with Phase 1-2)
**Deliverables:**
1. `SearchResult` type in `types.rs` and `types.ts`
2. `search_commits_inner` function in `history.rs` with unit tests
3. `search_commits` Tauri command registered in `lib.rs`

**Test strategy:**
```rust
#[test]
fn search_by_sha_prefix() { ... }

#[test]
fn search_by_message_substring() { ... }

#[test]
fn search_by_ref_name() { ... }

#[test]
fn search_case_insensitive() { ... }

#[test]
fn search_empty_query_returns_empty() { ... }
```

### Phase 4: Search UI
**Dependencies:** Phase 3, and CommitGraph's `scrollToOid` (already exists)
**Deliverables:**
1. `SearchBar.svelte` component (input, result count, prev/next buttons)
2. CommitGraph: integrate SearchBar, add Cmd+F keyboard handler
3. Search navigation: invoke `search_commits`, navigate results with `scrollToOid`
4. Escape to close, result cycling with arrow keys

**Test strategy:** Manual testing:
- Cmd+F opens search bar, focuses input
- Typing triggers search after debounce
- Results navigable with Enter/arrows
- Scroll position updates to show matching commit
- Escape closes search
- Works with large repos (10k+ commits, search result in unpaginated range)

### Dependency Graph

```
Phase 1 (Hunk Backend)  ──► Phase 2 (Hunk UI)
                                 │
Phase 3 (Search Backend) ──► Phase 4 (Search UI)
                                 │
                          (Phases 1-2 and 3-4 are independent tracks)
```

Phases 1 and 3 can be developed in parallel. Phase 2 depends on Phase 1. Phase 4 depends on Phase 3.

### Build Order Rationale

- **Hunk backend first** — highest technical risk (git2 `apply` API, patch construction). Early validation prevents wasted UI work.
- **Search backend is low risk** — simple string matching on cached data. Can run in parallel with hunk work.
- **Hunk UI before search UI** — hunk staging is the higher-value feature (users encounter this more often than graph search). Ship it first.
- **Search UI last** — builds on existing `scrollToOid` infrastructure. Lowest risk, cleanest integration.

## Anti-Patterns to Avoid

### Anti-Pattern 1: git CLI for Hunk Staging
**What people do:** Shell out to `git add -p` or `git apply --cached`
**Why it's wrong for Trunk:** Every local git operation uses git2. Introducing CLI for hunk staging creates an inconsistent pattern and requires temp file management.
**Do this instead:** Use `git2::Diff::from_buffer()` + `repo.apply(ApplyLocation::Index)`. Keep all local ops in git2.

### Anti-Pattern 2: Client-Side Search with Full Commit Load
**What people do:** Load all commits into JS arrays, search with `.filter()`
**Why it's wrong:** Defeats pagination purpose. 50k commits serialized to JS = ~100MB memory spike + multi-second parse time. CommitCache already has the data in Rust.
**Do this instead:** Backend `search_commits` command reads from `CommitCache`. Only matching OIDs are serialized to frontend.

### Anti-Pattern 3: Search That Replaces the Graph
**What people do:** Show search results in a filtered list view, hiding the graph
**Why it's wrong:** Users need graph context (which branch? which parent?) when searching. Search should highlight and navigate within the existing graph view.
**Do this instead:** Float SearchBar over the graph. Navigate results by scrolling the graph to matching commits.

### Anti-Pattern 4: Hunk Actions Without Diff Re-fetch
**What people do:** Optimistically remove the staged hunk from the UI without re-fetching the diff
**Why it's wrong:** After staging a hunk, git may coalesce adjacent hunks or change line numbers. The remaining diff is NOT simply "original minus staged hunk."
**Do this instead:** Always re-fetch the diff after a hunk action. The IPC round-trip is <5ms.

### Anti-Pattern 5: Breaking the Inner-Fn Test Pattern
**What people do:** Put git2 logic directly in `#[tauri::command]` functions
**Why it's wrong:** Can't unit test without Tauri runtime. Every existing command follows `*_inner()` pattern.
**Do this instead:** `stage_hunk_inner`, `unstage_hunk_inner`, `search_commits_inner` with `HashMap<String, PathBuf>` params. Test with `make_test_repo()`.

## Sources

- Full codebase audit of Trunk v0.6: all 10 Rust command modules, 20 Svelte components, 19 lib modules
- `git2` crate 0.19 API: `Repository::apply()`, `Diff::from_buffer()`, `ApplyLocation::Index` — confirmed available
- Existing `scrollToOid` implementation: CommitGraph.svelte:569-593 — handles lazy-load and smooth scroll
- CommitCache structure: `state.rs:16-17` — `HashMap<String, GraphResult>`, populated on open_repo
- DiffPanel hunk rendering: DiffPanel.svelte:123-148 — `{#each fd.hunks as hunk}` with header and lines
- Unified diff format spec: patch header must include `--- a/` / `+++ b/` / `@@ @@` for `Diff::from_buffer`

---
*Architecture research for: Trunk v0.7 Hunk Staging & Commit Graph Search*
*Researched: 2026-03-17*
