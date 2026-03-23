# Phase 37: Conflict Detection & Operation State - Research

**Researched:** 2026-03-20
**Domain:** Git operation state detection, conflict file display, Tauri IPC for git CLI commands
**Confidence:** HIGH

## Summary

This phase adds two major UI features to the staging panel: (1) a dedicated conflicted files section separated from unstaged files, and (2) persistent operation banners for merge/rebase in-progress states with action buttons. The backend needs a new Tauri command to detect operation state (merge vs. rebase vs. clean) and six new commands for merge/rebase continue/abort/skip actions.

The project already has strong foundations for this work. The `WorkingTreeStatus.conflicted` array and `FileStatusType::Conflicted` are already implemented and returning data. The conflicted files currently render inside the unstaged section (StagingPanel.svelte lines 284-293) -- they need extraction into their own collapsible section above unstaged. For operation detection, git2 0.19.0's `Repository::state()` method returns a `RepositoryState` enum with `Merge`, `Rebase`, `RebaseInteractive`, and `RebaseMerge` variants, providing a clean API-based approach. Branch name extraction requires reading `.git/MERGE_MSG` (for merge) and `.git/rebase-merge/head-name` + `.git/rebase-merge/onto` (for rebase) from the filesystem. The git CLI subprocess pattern from `commit_actions.rs` (cherry-pick/revert) provides a proven template for the continue/abort/skip commands.

**Primary recommendation:** Use `repo.state()` from git2 for operation type detection, augmented with filesystem reads for branch name extraction. Follow the existing `commit_actions.rs` pattern for all git CLI operations (continue/abort/skip). Create a new `operation_state.rs` command module to keep concerns separated.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Conflicted files appear as the **top section** in the staging panel, above unstaged and staged
- Section header uses **yellow warning icon + yellow count badge** on neutral background (same bg as unstaged/staged headers, not a colored background)
- Section is **collapsible**, consistent with unstaged/staged section behavior
- **No action buttons** on conflicted file rows in Phase 37 (no stage, discard, or resolve buttons)
- Banner appears at the **top of the staging panel**, above the conflict section
- Shows **operation type + branch names**: "Merging feature into main" / "Rebasing main onto origin/main"
- **Color-coded by operation type**: yellow-tinted background with yellow left border for merge, blue-tinted background with blue left border for rebase
- Banner is persistent -- only disappears when the operation completes or is aborted
- Clicking a conflicted file opens DiffPanel showing **raw file content with conflict markers** (<<<<<<< / ======= / >>>>>>>), read-only, no hunk action buttons
- Right-click context menu is **minimal**: only Copy Relative Path and Copy Absolute Path (no Stage/Discard)
- **Merge banner**: Continue and Abort buttons
- **Rebase banner**: Continue, Skip, and Abort buttons
- **Abort requires confirmation dialog** before executing ("Abort merge? This will discard all merge progress...")
- **Continue does not require confirmation** -- if conflicts remain, git errors and we show feedback
- **Skip does not require confirmation** -- non-destructive forward action

### Claude's Discretion
- Feedback mechanism after Continue/Abort/Skip (toast, banner update, or combination)
- Exact banner padding, spacing, and typography
- How to detect operation state (git2 repository state API vs. filesystem checks for MERGE_HEAD / rebase-merge/)
- Confirmation dialog wording and button labels
- Whether to disable Continue button when conflicts still exist vs. letting git error

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CONF-01 | Conflicted files display as a distinct third section in the staging panel with warning styling and count badge | Existing `WorkingTreeStatus.conflicted` array already returns data; extract from unstaged section into new collapsible section with yellow warning styling |
| OPS-01 | Persistent banner displays when a merge is in progress (detected via .git/MERGE_HEAD) with Continue and Abort buttons | Use `repo.state() == RepositoryState::Merge` for detection; read `.git/MERGE_MSG` for branch names; new `get_operation_state` command |
| OPS-02 | Persistent banner displays when a rebase is in progress (detected via .git/rebase-merge/ or .git/rebase-apply/) with Continue, Skip, and Abort buttons | Use `repo.state()` matching `Rebase/RebaseInteractive/RebaseMerge` variants; read `.git/rebase-merge/head-name` and `onto` for branch info |
| OPS-03 | Continue/Abort/Skip buttons invoke the corresponding git CLI commands and refresh the UI | Follow `cherry_pick_inner` subprocess pattern from `commit_actions.rs`; six new Tauri commands with cache-repopulate-before-emit pattern |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.19.0 | Repository state detection via `repo.state()` | Already in project; provides `RepositoryState` enum with Merge/Rebase/RebaseInteractive/RebaseMerge variants |
| git CLI | system | merge/rebase continue/abort/skip commands | Already used for cherry-pick/revert in `commit_actions.rs`; more reliable than git2 API for mutating operations |
| @tauri-apps/plugin-dialog | 2.6.0 | Confirmation dialog for Abort action | Already used for discard confirmations in StagingPanel |
| @lucide/svelte | 0.577.0 | Warning icons for conflict section | Already used; `FileWarning` icon already mapped for Conflicted status |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/plugin-clipboard-manager | 2.3.2 | Copy path from context menu | Already used in StagingPanel for Copy Relative/Absolute Path |
| @tauri-apps/api/menu | 2.x | Native context menu for conflicted files | Already used in StagingPanel for unstaged/staged context menus |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `repo.state()` (git2 API) | Filesystem checks for `.git/MERGE_HEAD`, `.git/rebase-merge/` | git2 API is cleaner but gives same info; filesystem needed anyway for branch names. Use both: git2 for state type, filesystem for metadata. |
| git CLI for continue/abort/skip | git2 `Rebase` struct API | git2 rebase API requires maintaining a `Rebase` handle across calls, which is complex with the project's "open-per-call" architecture. CLI is simpler and proven. |

**No additional npm or cargo packages needed.** Everything required is already in the project.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/commands/
  operation_state.rs       # NEW: get_operation_state, merge_continue, merge_abort,
                           #      rebase_continue, rebase_skip, rebase_abort
src-tauri/src/git/types.rs # ADD: OperationState enum + OperationInfo struct
src-tauri/src/commands/mod.rs  # ADD: pub mod operation_state;

src/lib/types.ts           # ADD: OperationState type + OperationInfo interface
src/components/
  StagingPanel.svelte      # MODIFY: add operation banner + conflict section
  OperationBanner.svelte   # NEW: persistent merge/rebase banner component
  FileRow.svelte           # MINOR: no changes needed (already handles Conflicted)
```

### Pattern 1: Operation State Detection (Backend)
**What:** New Tauri command `get_operation_state` that returns the current operation type and metadata.
**When to use:** Called on StagingPanel mount, on `repo-changed` events, and after any continue/abort/skip action.
**Example:**
```rust
// New types in git/types.rs
#[derive(Debug, Serialize, Clone)]
pub enum OperationType {
    None,
    Merge,
    Rebase,
    CherryPick,
    Revert,
}

#[derive(Debug, Serialize, Clone)]
pub struct OperationInfo {
    pub op_type: OperationType,
    pub source_branch: Option<String>,  // branch being merged/rebased
    pub target_branch: Option<String>,  // branch being merged into / rebased onto
    pub progress: Option<String>,       // e.g. "3/7" for rebase step
}

// In operation_state.rs
pub fn get_operation_state_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<OperationInfo, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let state = repo.state();

    match state {
        git2::RepositoryState::Merge => {
            // Read .git/MERGE_MSG for branch name
            let git_dir = repo.path(); // returns .git/ directory
            let merge_msg = std::fs::read_to_string(git_dir.join("MERGE_MSG")).ok();
            let source = extract_merge_branch_from_msg(merge_msg.as_deref());
            let target = repo.head().ok()
                .and_then(|h| h.shorthand().map(String::from));
            Ok(OperationInfo {
                op_type: OperationType::Merge,
                source_branch: source,
                target_branch: target,
                progress: None,
            })
        }
        git2::RepositoryState::Rebase
        | git2::RepositoryState::RebaseInteractive
        | git2::RepositoryState::RebaseMerge => {
            let git_dir = repo.path();
            // Try rebase-merge first (default backend), then rebase-apply
            let rebase_dir = if git_dir.join("rebase-merge").exists() {
                git_dir.join("rebase-merge")
            } else {
                git_dir.join("rebase-apply")
            };
            let head_name = std::fs::read_to_string(rebase_dir.join("head-name"))
                .ok().map(|s| s.trim().replace("refs/heads/", ""));
            let onto_oid = std::fs::read_to_string(rebase_dir.join("onto"))
                .ok().map(|s| s.trim().to_owned());
            // Resolve onto OID to branch name if possible
            let onto_branch = onto_oid.and_then(|oid| resolve_oid_to_branch(&repo, &oid));
            let msgnum = std::fs::read_to_string(rebase_dir.join("msgnum"))
                .ok().map(|s| s.trim().to_owned());
            let end = std::fs::read_to_string(rebase_dir.join("end"))
                .ok().map(|s| s.trim().to_owned());
            let progress = match (msgnum, end) {
                (Some(m), Some(e)) => Some(format!("{}/{}", m, e)),
                _ => None,
            };
            Ok(OperationInfo {
                op_type: OperationType::Rebase,
                source_branch: head_name,
                target_branch: onto_branch,
                progress,
            })
        }
        git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
            Ok(OperationInfo {
                op_type: OperationType::CherryPick,
                source_branch: None,
                target_branch: None,
                progress: None,
            })
        }
        git2::RepositoryState::Revert | git2::RepositoryState::RevertSequence => {
            Ok(OperationInfo {
                op_type: OperationType::Revert,
                source_branch: None,
                target_branch: None,
                progress: None,
            })
        }
        _ => {
            Ok(OperationInfo {
                op_type: OperationType::None,
                source_branch: None,
                target_branch: None,
                progress: None,
            })
        }
    }
}
```

### Pattern 2: Git CLI Subprocess for Operations (Backend)
**What:** Continue/Abort/Skip commands follow the existing `cherry_pick_inner` pattern: spawn `git` subprocess, check exit status, refresh graph cache, emit `repo-changed`.
**When to use:** All mutating merge/rebase operations.
**Example:**
```rust
// Follows cherry_pick_inner pattern from commit_actions.rs
pub fn merge_continue_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    let output = std::process::Command::new("git")
        .args(["merge", "--continue"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| TrunkError::new("merge_error", e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("merge_error", stderr.to_string()));
    }

    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

// Same pattern for: merge_abort, rebase_continue, rebase_skip, rebase_abort
```

### Pattern 3: Operation Banner Component (Frontend)
**What:** New `OperationBanner.svelte` component that sits at the top of StagingPanel, fetches operation state, and renders action buttons.
**When to use:** Rendered conditionally when `operationInfo.op_type !== 'None'`.
**Example:**
```svelte
<!-- In StagingPanel.svelte, above file sections -->
{#if operationInfo && operationInfo.op_type !== 'None'}
  <OperationBanner
    info={operationInfo}
    {repoPath}
    onaction={handleOperationAction}
  />
{/if}
```

### Pattern 4: Conflicted File Section (Frontend)
**What:** Separate collapsible section for conflicted files, mimicking the existing unstaged/staged section pattern but with warning styling and no action buttons on rows.
**When to use:** When `status?.conflicted.length > 0`.
**Key differences from unstaged/staged sections:**
- Yellow warning icon instead of chevron in header
- Yellow count badge
- No "Stage All" / "Discard All" header buttons
- FileRow rendered without the hover action button (no `onaction` prop or `actionLabel`)
- Click opens DiffPanel in a special read-only 'conflicted' mode
- Context menu has only Copy Relative Path and Copy Absolute Path

### Pattern 5: Conflicted File Diff Display
**What:** When a conflicted file is clicked, open DiffPanel showing raw file content with conflict markers.
**When to use:** User clicks on a file in the conflicted section.
**Implementation approach:** Use `diff_unstaged` for the file -- git2 will return the diff showing conflict markers as content. DiffPanel already supports read-only mode via `diffKind='commit'` (no hunk action buttons). Add a new `diffKind='conflicted'` value to explicitly disable all action buttons and line selection.

### Anti-Patterns to Avoid
- **Don't use git2's Rebase API for continue/abort/skip:** The project opens a fresh `Repository` per command call. git2's `Rebase` struct needs to be opened and maintained within the same repo handle, which adds complexity. The git CLI subprocess pattern is proven and simpler.
- **Don't poll for operation state:** Rely on the existing `repo-changed` event pattern for reactivity. Load operation state alongside file status on mount and on every `repo-changed` event.
- **Don't store operation state globally in a `$state` rune module:** Operation state is per-repo and should be loaded fresh each time. Keep it as local state in StagingPanel, fetched alongside `loadStatus()`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Operation state detection | Manual filesystem checks for all state files | `repo.state()` from git2 | Handles all edge cases (bisect, cherry-pick, revert, mailbox) in one call; 12 variants properly classified |
| Confirmation dialogs | Custom modal component | `@tauri-apps/plugin-dialog` `ask()` | Already used for discard confirmations; native OS dialog, consistent UX |
| Branch name from MERGE_MSG | Regex parsing of arbitrary merge messages | Simple prefix strip `"Merge branch '"` | MERGE_MSG follows a standard format: `Merge branch 'name' into target` |
| Resolving OID to branch name | Custom ref iteration | `repo.references()` iteration or `git branch --contains` | git2 provides direct API for this |

**Key insight:** Operation state detection has many edge cases (interactive vs. normal rebase, rebase-merge vs. rebase-apply directories, cherry-pick sequences, etc.). Using git2's `RepositoryState` enum handles all variants correctly with a single API call, avoiding brittle filesystem checks for the primary detection.

## Common Pitfalls

### Pitfall 1: MERGE_MSG Branch Name Extraction
**What goes wrong:** Assuming MERGE_MSG always has a consistent format.
**Why it happens:** MERGE_MSG format varies: `Merge branch 'feature'` vs. `Merge remote-tracking branch 'origin/feature'` vs. custom merge messages.
**How to avoid:** Use a permissive parser. Try to extract branch name but fall back to showing just "Merge in progress" if parsing fails. Never crash or error on unparseable MERGE_MSG.
**Warning signs:** Tests with non-standard merge messages failing.

### Pitfall 2: Rebase Directory Selection (rebase-merge vs. rebase-apply)
**What goes wrong:** Only checking `.git/rebase-merge/` and missing `.git/rebase-apply/`.
**Why it happens:** Modern git defaults to the merge backend (`rebase-merge`), but `rebase-apply` is used when `--apply` flag is passed or for older git versions.
**How to avoid:** Check both directories. Try `rebase-merge` first (more common), fall back to `rebase-apply`.
**Warning signs:** Rebase banner not appearing when user initiated rebase with `--apply`.

### Pitfall 3: Operation State Stale After Action
**What goes wrong:** Banner stays visible after successful continue/abort because state was not refreshed.
**Why it happens:** The `repo-changed` event triggers async refresh, but the UI may not re-fetch operation state.
**How to avoid:** After any continue/abort/skip action, explicitly call `loadOperationState()` in addition to relying on the `repo-changed` event. The action handler should refresh state immediately.
**Warning signs:** Banner flickering or staying after operation completes.

### Pitfall 4: `git merge --continue` vs. `git commit`
**What goes wrong:** `git merge --continue` may not exist on older git versions.
**Why it happens:** `git merge --continue` was added in Git 2.12. Before that, completing a merge required `git commit`.
**How to avoid:** Use `git merge --continue` as the primary command. If it fails with an unrecognized-option error, fall back. However, for a desktop app where users likely have modern git, this is low risk. The minimum git version should be documented.
**Warning signs:** Error messages about unrecognized options.

### Pitfall 5: Conflict Section Flex Layout
**What goes wrong:** Adding a third section breaks the 50/50 flex layout of unstaged/staged.
**Why it happens:** The current layout uses `flex: 1` to split space between two sections. Adding a third section with `flex: 1` would give each 33%.
**How to avoid:** The conflict section should NOT flex-grow to fill space equally. It should have `flex-shrink: 0` and only take as much height as it needs (up to a reasonable max), leaving the remaining space for unstaged/staged. When conflict section has files, it should behave like a compact list (not half the panel).
**Warning signs:** Conflict section taking 33% of panel space when it only has 2 files.

### Pitfall 6: Concurrent State Reads During Rapid Changes
**What goes wrong:** Operation state and file status get out of sync.
**Why it happens:** `loadStatus()` and `loadOperationState()` are separate async calls. If `repo-changed` fires rapidly, they may return for different repo states.
**How to avoid:** Combine operation state fetch with file status fetch in a single Tauri command, or use the existing `loadSeq` counter pattern to discard stale responses. The simplest approach: add operation state to the existing `get_status` response, or use the existing `loadSeq` pattern for both calls.
**Warning signs:** Banner showing "merge in progress" but no conflicted files, or vice versa.

## Code Examples

### Example 1: OperationInfo Type (Rust)
```rust
// Source: New types for src-tauri/src/git/types.rs
#[derive(Debug, Serialize, Clone)]
pub enum OperationType {
    None,
    Merge,
    Rebase,
    CherryPick,
    Revert,
}

#[derive(Debug, Serialize, Clone)]
pub struct OperationInfo {
    pub op_type: OperationType,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    pub progress: Option<String>,
}
```

### Example 2: OperationInfo Type (TypeScript)
```typescript
// Source: New types for src/lib/types.ts
export type OperationType = 'None' | 'Merge' | 'Rebase' | 'CherryPick' | 'Revert';

export interface OperationInfo {
  op_type: OperationType;
  source_branch: string | null;
  target_branch: string | null;
  progress: string | null;
}
```

### Example 3: Tauri Command Pattern (Follows commit_actions.rs)
```rust
// Source: Pattern from commit_actions.rs cherry_pick/revert_commit
#[tauri::command]
pub async fn merge_abort(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        merge_abort_inner(&path_clone, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}
```

### Example 4: Confirmation Dialog (Follows StagingPanel Pattern)
```typescript
// Source: Pattern from StagingPanel.svelte handleDiscardFile
async function handleAbort(opType: OperationType) {
  const { ask } = await import('@tauri-apps/plugin-dialog');
  const opName = opType === 'Merge' ? 'merge' : 'rebase';
  const confirmed = await ask(
    `Abort ${opName}? This will discard all ${opName} progress and return to the previous state.`,
    { title: `Abort ${opName.charAt(0).toUpperCase() + opName.slice(1)}`, kind: 'warning' }
  );
  if (!confirmed) return;
  // Execute abort...
}
```

### Example 5: CSS Custom Properties for Banners
```css
/* Source: New properties for src/app.css following existing pattern */
/* Operation banner colors */
--color-banner-merge-bg: rgba(250, 204, 21, 0.08);
--color-banner-merge-border: #facc15;
--color-banner-rebase-bg: rgba(96, 165, 250, 0.08);
--color-banner-rebase-border: #60a5fa;

/* Operation banner button colors */
--color-btn-continue: #4ade80;
--color-btn-continue-bg: rgba(74, 222, 128, 0.15);
--color-btn-continue-border: rgba(74, 222, 128, 0.3);
--color-btn-abort: #f87171;
--color-btn-abort-bg: rgba(248, 113, 113, 0.15);
--color-btn-abort-border: rgba(248, 113, 113, 0.3);
--color-btn-skip: #fbbf24;
--color-btn-skip-bg: rgba(251, 191, 36, 0.15);
--color-btn-skip-border: rgba(251, 191, 36, 0.3);

/* Conflict section warning badge */
--color-badge-warning: #facc15;
--color-badge-warning-bg: rgba(250, 204, 21, 0.15);
```

### Example 6: Conflict Section Header (Follows Existing Pattern)
```svelte
<!-- Source: Pattern from StagingPanel.svelte unstaged/staged headers -->
<!-- Conflict section header -->
<div
  role="button"
  tabindex="0"
  onclick={() => (conflicted_expanded = !conflicted_expanded)}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') conflicted_expanded = !conflicted_expanded; }}
  style="
    height: 28px;
    border-bottom: 1px solid var(--color-border);
    padding: 0 8px;
    display: flex;
    align-items: center;
    cursor: pointer;
    flex-shrink: 0;
  "
>
  <span style="color: var(--color-badge-warning); display: inline-flex; align-items: center; margin-right: 4px;">
    <AlertTriangle size={12} />
  </span>
  <span style="color: var(--color-text); font-size: 12px; font-weight: 500; flex: 1;">
    Conflicted Files
  </span>
  <span style="
    background: var(--color-badge-warning-bg);
    color: var(--color-badge-warning);
    font-size: 10px;
    font-weight: 700;
    border-radius: 9999px;
    padding: 0 6px;
    line-height: 16px;
  ">
    {status?.conflicted.length ?? 0}
  </span>
</div>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Check `.git/MERGE_HEAD` exists (filesystem) | `repo.state()` returns `RepositoryState::Merge` | git2 has had this since early versions | Single API call covers all 12 operation states |
| `git rebase --apply` (legacy backend) | `git rebase` (merge backend, default since Git 2.34) | Git 2.34 (2021) | `.git/rebase-merge/` is the common directory; still check `rebase-apply/` as fallback |
| `git commit` to complete merge | `git merge --continue` | Git 2.12 (2017) | Clearer semantics; safe to assume for desktop app users |

**Deprecated/outdated:**
- `rebase-apply` backend: Still supported but not the default. Must handle both paths.

## Open Questions

1. **Should operation state be bundled into `get_status` response or be a separate command?**
   - What we know: Separate command is simpler to implement but risks stale state (Pitfall 6). Bundling avoids sync issues.
   - What's unclear: Whether adding a field to `WorkingTreeStatus` is cleaner than a separate call.
   - Recommendation: **Add `operation: OperationInfo` field to `WorkingTreeStatus`** so both are fetched atomically in one IPC call. This eliminates Pitfall 6 entirely. Single source of truth.

2. **How to show conflicted file diff with conflict markers?**
   - What we know: `diff_unstaged` returns git2's diff, which may or may not include raw conflict markers. The file on disk contains the conflict markers.
   - What's unclear: Whether `diff_unstaged` on a conflicted file shows useful content or errors.
   - Recommendation: For conflicted files, read the raw file content from disk and display it as plain text (not as a diff). This guarantees conflict markers are visible. Add a new Tauri command `read_file_content` or handle in the frontend by passing a 'conflicted' diffKind that triggers raw file read.

3. **Feedback mechanism after Continue/Abort/Skip**
   - What we know: Project has toast system (`showToast`) and the operation banner itself.
   - Recommendation: Use **toast for success** ("Merge aborted", "Rebase continued -- 3/7 commits applied") and **toast for errors** ("Cannot continue: conflicts remain"). The banner automatically disappears on success because operation state refreshes to `None`.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Vitest 4.1.0 (frontend), cargo test (backend) |
| Config file | `vite.config.ts` (test section), Cargo.toml |
| Quick run command | `npm run test` / `cargo test -p trunk` |
| Full suite command | `npm run test && cargo test -p trunk` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONF-01 | Conflicted files appear as distinct section with warning styling | manual | Visual inspection in Tauri dev | N/A |
| OPS-01 | Merge operation detection + banner display | unit (backend) | `cargo test -p trunk operation_state` | Wave 0 |
| OPS-02 | Rebase operation detection + banner display | unit (backend) | `cargo test -p trunk operation_state` | Wave 0 |
| OPS-03 | Continue/Abort/Skip execute git commands | unit (backend) | `cargo test -p trunk operation_state` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p trunk`
- **Per wave merge:** `npm run test && cargo test -p trunk`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/commands/operation_state.rs` -- covers OPS-01, OPS-02, OPS-03 (new module with tests)
- [ ] Backend tests for `get_operation_state_inner` with merge, rebase, and clean states
- [ ] Backend tests for `merge_continue_inner`, `merge_abort_inner`, `rebase_continue_inner`, `rebase_skip_inner`, `rebase_abort_inner`
- [ ] Backend tests for MERGE_MSG branch name parsing edge cases

## Sources

### Primary (HIGH confidence)
- [git2 0.19.0 RepositoryState docs](https://docs.rs/git2/0.19.0/git2/enum.RepositoryState.html) -- confirmed all 12 variants including Merge, Rebase, RebaseInteractive, RebaseMerge
- [git2 Repository::state() docs](https://docs.rs/git2/latest/git2/struct.Repository.html) -- confirmed `state()`, `cleanup_state()`, `message()`, `merge()` methods
- Codebase: `src-tauri/src/commands/commit_actions.rs` -- proven git CLI subprocess pattern for cherry-pick/revert
- Codebase: `src-tauri/src/commands/staging.rs` -- existing `Status::CONFLICTED` detection, `WorkingTreeStatus` struct
- Codebase: `src/components/StagingPanel.svelte` -- current conflicted files rendering (lines 284-293), collapsible section pattern
- Codebase: `src/components/FileRow.svelte` -- `FileWarning` icon with `#facc15` already mapped for Conflicted
- Codebase: `src/app.css` -- CSS custom property naming convention

### Secondary (MEDIUM confidence)
- [git-merge documentation](https://git-scm.com/docs/git-merge) -- MERGE_HEAD, MERGE_MSG file contents and behavior
- [git-rebase documentation](https://git-scm.com/docs/git-rebase) -- rebase-merge/rebase-apply directory structure, head-name/onto/msgnum/end files

### Tertiary (LOW confidence)
- None -- all findings verified with official sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, all libraries already in project and verified
- Architecture: HIGH -- patterns directly derived from existing codebase (commit_actions.rs, StagingPanel.svelte)
- Pitfalls: HIGH -- identified from both git internals knowledge and codebase analysis
- Operation detection: HIGH -- git2 RepositoryState enum verified against docs.rs for version 0.19.0

**Research date:** 2026-03-20
**Valid until:** 2026-04-20 (stable domain, no expected breaking changes)
