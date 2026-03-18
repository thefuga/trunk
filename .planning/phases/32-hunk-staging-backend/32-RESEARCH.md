# Phase 32: Hunk Staging Backend - Research

**Researched:** 2026-03-17
**Domain:** git2 hunk-level patch application in Rust
**Confidence:** HIGH

## Summary

Hunk-level staging, unstaging, and discarding in git2 can be achieved using `repo.apply()` with `ApplyOptions::hunk_callback()` to selectively apply individual hunks from a full diff. This eliminates the need to construct single-hunk patch text or use the `Patch::from_diff()` + `to_buf()` + `Diff::from_buffer()` pipeline. The hunk_callback approach is the cleanest path: generate the diff for a single file (using pathspec), then apply it with a callback that returns `true` only for the target hunk index.

For **unstaging** and **discarding**, `DiffOptions::reverse(true)` inverts the diff's a/b sides, so the same `repo.apply()` + `hunk_callback` pattern works -- just with a reversed diff. Staging applies a forward diff to the index; unstaging applies a reversed diff to the index; discarding applies a reversed diff to the workdir.

**Primary recommendation:** Use `repo.apply()` with `ApplyOptions::hunk_callback()` for all three operations (stage, unstage, discard). Use `DiffOptions::reverse(true)` for unstage and discard. Do not construct patch text manually.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Frontend identifies hunks by **array index** (0-based position in `Vec<DiffHunk>`)
- Backend re-fetches the diff internally and picks hunk at position N
- Command signature: `stage_hunk(path: String, file_path: String, hunk_index: u32)`
- Commands return `Result<(), String>` (success/failure only)
- Backend returns typed error on apply failure -- frontend re-fetches diff
- Distinct error codes: `stale_hunk_index`, `hunk_apply_failed`, `file_not_found`
- Backend trusts frontend has confirmed discard (no backend-side safeguard)
- No backend auto-retry or closest-match logic

### Claude's Discretion
- Single-hunk patch extraction implementation approach
- Whether to use `Patch::from_diff()`, manual patch construction, or `Diff::from_buffer()` approach
- Test fixture design for multi-hunk files, single-hunk new files, and no-newline-at-EOF
- Whether hunk commands go in `staging.rs` or a new `hunk.rs` file
- `unstage_hunk` reversed patch construction approach

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| HUNK-01 | User can stage individual hunks from the unstaged diff view | `stage_hunk_inner` using `repo.apply(&diff, ApplyLocation::Index, apply_opts)` with hunk_callback filtering |
| HUNK-02 | User can unstage individual hunks from the staged diff view | `unstage_hunk_inner` using reversed diff + `repo.apply(&diff, ApplyLocation::Index, apply_opts)` |
| HUNK-03 | User can discard individual hunks from the working tree | `discard_hunk_inner` using reversed diff + `repo.apply(&diff, ApplyLocation::WorkDir, apply_opts)` |
| HUNK-05 | Diff view refreshes after hunk operations | Commands return `Result<(), String>`; frontend re-fetches diff after each operation (no backend changes needed) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.19.0 | All git operations (diff, apply, index manipulation) | Already in Cargo.toml; provides `Repository::apply()`, `ApplyOptions`, `DiffOptions::reverse()` |
| tempfile | 3.x | Test fixtures with temp directories | Already in dev-dependencies; used by all existing test modules |

### Supporting
No additional libraries needed. All required functionality exists in git2 0.19.0.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| hunk_callback filtering | Patch::from_diff + to_buf + Diff::from_buffer | Extra serialize/deserialize; more code; same result |
| hunk_callback filtering | Manual patch text construction | Fragile, error-prone, must handle all edge cases git2 already handles |
| DiffOptions::reverse(true) | Manual patch reversal (swap +/- lines) | Fragile, misses header line count adjustments |

**Decision:** Use `ApplyOptions::hunk_callback()` for selective hunk application. This is the approach used in git2's own test suite.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/commands/
    staging.rs          # Existing file -- add hunk commands here
```

**Recommendation: Keep hunk commands in `staging.rs`.** The hunk operations (stage_hunk, unstage_hunk, discard_hunk) are staging operations. They share `open_repo_from_state`, `is_head_unborn`, and the error handling pattern. A separate `hunk.rs` would require duplicating these helpers or extracting them to a shared module. The existing file is ~400 lines; adding ~120 lines for 3 inner functions + 3 command wrappers is well within reason.

### Pattern 1: Hunk Callback Filtering
**What:** Use `ApplyOptions::hunk_callback()` to apply only a specific hunk from a multi-hunk diff.
**When to use:** All three hunk operations (stage, unstage, discard).
**How it works:** The hunk_callback receives each hunk in order during `repo.apply()`. By tracking a counter, return `true` only when the counter matches the target hunk_index. Return `false` for all other hunks (skips them without aborting).

**Example:**
```rust
// Source: git2 0.19.0 src/apply.rs (verified from source)
// hunk_cb_c maps: true -> 0 (apply), false -> 1 (skip)
let mut current_hunk: usize = 0;
let target = hunk_index as usize;
let mut apply_opts = git2::ApplyOptions::new();
apply_opts.hunk_callback(move |_hunk| {
    let dominated = current_hunk == target;
    current_hunk += 1;
    dominated
});
repo.apply(&diff, location, Some(&mut apply_opts))?;
```

### Pattern 2: Three Operations, One Core Pattern
**What:** stage_hunk, unstage_hunk, and discard_hunk all follow the same structure but with different diff source and apply location.

| Operation | Diff Source | Reverse? | ApplyLocation |
|-----------|------------|----------|---------------|
| stage_hunk | `diff_index_to_workdir` | No | `Index` |
| unstage_hunk | `diff_tree_to_index` | Yes | `Index` |
| discard_hunk | `diff_index_to_workdir` | Yes | `WorkDir` |

### Pattern 3: Inner-fn + async wrapper (existing pattern)
**What:** Sync `_inner` function with `&str`/`&HashMap` params, async `#[tauri::command]` wrapper with `spawn_blocking`.
**When to use:** All new commands.
**Example:**
```rust
pub fn stage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    // ... implementation
}

#[tauri::command]
pub async fn stage_hunk(
    path: String,
    file_path: String,
    hunk_index: u32,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        stage_hunk_inner(&path, &file_path, hunk_index, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())
}
```

### Anti-Patterns to Avoid
- **Manual patch text construction:** Do not build patch strings by concatenating headers and hunk lines. This is fragile and misses edge cases (binary markers, no-newline-at-EOF, mode changes).
- **Patch::from_diff roundtrip for hunk filtering:** Using `Patch::from_diff(diff, delta_idx)?.to_buf()` followed by `Diff::from_buffer()` adds unnecessary serialization. The hunk_callback approach is simpler.
- **Sharing mutable state between diff generation and apply:** The diff must remain immutable while apply uses it. Generate the diff once, then apply with callbacks.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Single-hunk extraction from diff | Manual patch text with header manipulation | `ApplyOptions::hunk_callback()` filtering | libgit2 handles all edge cases: binary, no-newline-at-EOF, mode changes |
| Reversed patch for unstage/discard | Swapping +/- lines and adjusting header counts | `DiffOptions::reverse(true)` | Header old/new counts, line number adjustments are subtle and error-prone |
| Applying patch to index | Direct index entry manipulation | `repo.apply(&diff, ApplyLocation::Index, ...)` | libgit2 handles tree entry creation, OID computation, index write |
| Applying patch to workdir | File read + text manipulation + file write | `repo.apply(&diff, ApplyLocation::WorkDir, ...)` | libgit2 handles atomic writes, permissions, line endings |
| Error detection for stale hunks | Custom diff comparison | hunk_index bounds check against diff's hunk count | Simple bounds check is sufficient; apply errors cover content mismatch |

**Key insight:** The hunk_callback approach delegates ALL patch mechanics to libgit2. The only custom logic needed is a counter to match the target hunk index.

## Common Pitfalls

### Pitfall 1: Hunk Index Mismatch Between Frontend and Backend
**What goes wrong:** Frontend sends hunk_index=2 but the file has been modified since the diff was fetched, changing hunk boundaries.
**Why it happens:** Race condition between user viewing diff and executing stage/unstage/discard.
**How to avoid:** Re-fetch the diff in the backend and validate hunk_index < num_hunks before applying. Return `stale_hunk_index` error if out of bounds.
**Warning signs:** Apply succeeds but stages the wrong hunk, or panics on out-of-bounds.

### Pitfall 2: Untracked (New) Files Cannot Be Staged via apply()
**What goes wrong:** `repo.apply()` with `ApplyLocation::Index` may fail or produce unexpected results for files with `Delta::Untracked` status, because the index has no entry for the file.
**Why it happens:** `diff_index_to_workdir` marks new files as `Delta::Untracked`. The apply mechanism expects to modify an existing index entry.
**How to avoid:** For new/untracked files, fall back to `index.add_path()` (stage the whole file). This is acceptable because a new file is typically a single add-all-lines hunk. Alternatively, detect untracked status and return an appropriate error directing the frontend to use `stage_file` instead.
**Warning signs:** Apply returns error for new files that have never been tracked.

### Pitfall 3: Unborn HEAD (No Commits Yet) for Unstage
**What goes wrong:** `unstage_hunk` needs `diff_tree_to_index(HEAD_tree, ...)` but HEAD doesn't exist on an unborn branch.
**Why it happens:** No HEAD tree to diff against.
**How to avoid:** Check `is_head_unborn()` before unstage_hunk. For unborn HEAD, use `diff_tree_to_index(None, None, opts)` (empty tree vs index). The existing `diff_staged_inner` already handles this pattern.
**Warning signs:** Panic or error when trying to unstage a hunk in a brand-new repo with no commits.

### Pitfall 4: hunk_callback Counter Must Track All Hunks, Not Just Matching
**What goes wrong:** If the diff contains multiple files (despite pathspec), the hunk counter tracks hunks across all files.
**Why it happens:** Pathspec filters to a single file, but if the pathspec is wrong or file path has special characters, multiple files might match.
**How to avoid:** Always use exact pathspec. Assert that `diff.deltas().len() == 1` after generating the diff. If 0, return `file_not_found`. If >1, return an error.
**Warning signs:** Wrong hunk gets staged when file path matching is ambiguous.

### Pitfall 5: Deleted Files Have No Workdir Content
**What goes wrong:** Attempting `discard_hunk` on a deleted file's diff might fail because the workdir file doesn't exist.
**Why it happens:** `Delta::Deleted` means the file exists in the index but not on disk.
**How to avoid:** For `discard_hunk`, the reverse of a deletion is a creation. `ApplyLocation::WorkDir` with a reversed delete-diff should recreate the file. However, this edge case should be tested explicitly.
**Warning signs:** "File not found" errors when trying to discard a deletion hunk.

### Pitfall 6: apply() Error vs Hunk Callback Skip Confusion
**What goes wrong:** If the hunk_callback returns `false` for ALL hunks (e.g., hunk_index is valid but all hunks are skipped), `repo.apply()` still returns `Ok(())`. Nothing is applied but no error is raised.
**Why it happens:** Skipping hunks via callback is not an error condition in libgit2.
**How to avoid:** This is actually fine for our use case -- if the target hunk doesn't exist (already validated by bounds check), we never reach this state. The bounds check before apply catches the stale_hunk_index case.
**Warning signs:** Silent no-op if bounds check is missing.

## Code Examples

### stage_hunk_inner (complete implementation pattern)
```rust
// Source: git2 0.19.0 API + existing codebase patterns
pub fn stage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    // Generate diff for this file (index -> workdir)
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(file_path);
    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;

    // Validate: exactly one delta expected
    let num_deltas = diff.deltas().len();
    if num_deltas == 0 {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No unstaged changes for: {}", file_path),
        ));
    }

    // Count hunks via Patch to validate hunk_index
    let patch = git2::Patch::from_diff(&diff, 0)?
        .ok_or_else(|| TrunkError::new("file_not_found", "Binary or unchanged file"))?;
    let num_hunks = patch.num_hunks();
    if (hunk_index as usize) >= num_hunks {
        return Err(TrunkError::new(
            "stale_hunk_index",
            format!("Hunk index {} out of range (file has {} hunks)", hunk_index, num_hunks),
        ));
    }
    drop(patch); // Release borrow on diff

    // Apply only the target hunk to the index
    let target = hunk_index as usize;
    let mut current: usize = 0;
    let mut apply_opts = git2::ApplyOptions::new();
    apply_opts.hunk_callback(move |_hunk| {
        let dominated = current == target;
        current += 1;
        dominated
    });

    repo.apply(&diff, git2::ApplyLocation::Index, Some(&mut apply_opts))
        .map_err(|e| TrunkError::new("hunk_apply_failed", e.message().to_owned()))?;

    Ok(())
}
```

### unstage_hunk_inner (reversed diff pattern)
```rust
// Source: git2 0.19.0 API
pub fn unstage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    // Generate reversed diff (index -> HEAD) so applying it to index undoes the staged change
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(file_path).reverse(true);

    let diff = if is_head_unborn(&repo) {
        repo.diff_tree_to_index(None, None, Some(&mut diff_opts))?
    } else {
        let head_tree = repo.head()?.peel_to_tree()?;
        repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?
    };

    // Validate delta exists
    if diff.deltas().len() == 0 {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No staged changes for: {}", file_path),
        ));
    }

    // Validate hunk_index
    let patch = git2::Patch::from_diff(&diff, 0)?
        .ok_or_else(|| TrunkError::new("file_not_found", "Binary or unchanged file"))?;
    let num_hunks = patch.num_hunks();
    if (hunk_index as usize) >= num_hunks {
        return Err(TrunkError::new(
            "stale_hunk_index",
            format!("Hunk index {} out of range (file has {} hunks)", hunk_index, num_hunks),
        ));
    }
    drop(patch);

    // Apply reversed hunk to index
    let target = hunk_index as usize;
    let mut current: usize = 0;
    let mut apply_opts = git2::ApplyOptions::new();
    apply_opts.hunk_callback(move |_hunk| {
        let apply = current == target;
        current += 1;
        apply
    });

    repo.apply(&diff, git2::ApplyLocation::Index, Some(&mut apply_opts))
        .map_err(|e| TrunkError::new("hunk_apply_failed", e.message().to_owned()))?;

    Ok(())
}
```

### discard_hunk_inner (reversed diff to workdir)
```rust
// Source: git2 0.19.0 API
pub fn discard_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    // Generate reversed diff (workdir -> index) so applying to workdir undoes the change
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(file_path).reverse(true);
    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;

    if diff.deltas().len() == 0 {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No unstaged changes for: {}", file_path),
        ));
    }

    // Validate hunk_index
    let patch = git2::Patch::from_diff(&diff, 0)?
        .ok_or_else(|| TrunkError::new("file_not_found", "Binary or unchanged file"))?;
    let num_hunks = patch.num_hunks();
    if (hunk_index as usize) >= num_hunks {
        return Err(TrunkError::new(
            "stale_hunk_index",
            format!("Hunk index {} out of range (file has {} hunks)", hunk_index, num_hunks),
        ));
    }
    drop(patch);

    // Apply reversed hunk to workdir
    let target = hunk_index as usize;
    let mut current: usize = 0;
    let mut apply_opts = git2::ApplyOptions::new();
    apply_opts.hunk_callback(move |_hunk| {
        let apply = current == target;
        current += 1;
        apply
    });

    repo.apply(&diff, git2::ApplyLocation::WorkDir, Some(&mut apply_opts))
        .map_err(|e| TrunkError::new("hunk_apply_failed", e.message().to_owned()))?;

    Ok(())
}
```

### Test fixture: multi-hunk file
```rust
// Create a file with content that produces multiple hunks when modified
fn create_multi_hunk_file(dir: &std::path::Path) {
    // Original content: 30+ lines to ensure context separation between hunks
    let original = (1..=30).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
    std::fs::write(dir.join("multi.txt"), &original).unwrap();

    // Stage and commit the original
    let repo = git2::Repository::open(dir).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("multi.txt")).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = repo.signature().unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Add multi.txt", &tree, &[&head]).unwrap();
    drop(index);
    drop(tree);
    drop(head);
    drop(repo);

    // Modify lines near the top AND near the bottom (creates 2 hunks)
    let mut lines: Vec<String> = original.split('\n').map(|s| s.to_string()).collect();
    lines[1] = "MODIFIED line 2".to_string();   // Near top -> hunk 0
    lines[28] = "MODIFIED line 29".to_string();  // Near bottom -> hunk 1
    std::fs::write(dir.join("multi.txt"), lines.join("\n")).unwrap();
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual patch text construction | `ApplyOptions::hunk_callback()` filtering | git2 0.14+ (libgit2 1.3+) | Eliminates brittle string manipulation |
| Patch::from_diff + to_buf roundtrip | Direct hunk_callback on repo.apply() | Always available in git2 | Simpler, fewer allocations |
| `git add -p` subprocess | Native git2 `repo.apply()` | git2 0.14+ | No subprocess, no parsing |

**Deprecated/outdated:**
- `Patch::from_diff` + `to_buf` + `Diff::from_buffer` pipeline: Still works but is unnecessarily complex when hunk_callback is available.

## Open Questions

1. **Untracked file hunk staging via apply()**
   - What we know: `diff_index_to_workdir` includes untracked files as `Delta::Untracked`. The `repo.apply()` with `ApplyLocation::Index` may not handle this delta type.
   - What's unclear: Whether libgit2 can apply an "add new file" patch to the index via `git_apply`.
   - Recommendation: Test this during implementation. If it fails, fall back to `index.add_path()` for untracked files (detected via delta status check before apply). This is acceptable since new files are typically one hunk.

2. **Reversed diff for newly staged file (all content added)**
   - What we know: For a file that was entirely newly staged (appeared in index for the first time via stage_file), the staged diff is all additions. Reversing this produces all deletions.
   - What's unclear: Whether applying a "delete entire file" reversed patch to the index via `ApplyLocation::Index` correctly removes the index entry.
   - Recommendation: Test explicitly in Wave 0. If it fails, fall back to `unstage_file_inner` (which uses `reset_default` or `index.remove_path`).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)]` + `#[test]` |
| Config file | None -- Cargo test runner |
| Quick run command | `cd src-tauri && cargo test --lib commands::staging -- --test-threads=1` |
| Full suite command | `cd src-tauri && cargo test --lib -- --test-threads=1` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HUNK-01 | Stage single hunk from 2-hunk file; verify only that hunk is staged | unit | `cd src-tauri && cargo test --lib commands::staging::tests::stage_hunk_stages_single_hunk -- --test-threads=1` | Wave 0 |
| HUNK-01 | Stage hunk with out-of-bounds index returns stale_hunk_index error | unit | `cd src-tauri && cargo test --lib commands::staging::tests::stage_hunk_stale_index -- --test-threads=1` | Wave 0 |
| HUNK-02 | Unstage single hunk from 2-hunk staged diff; verify only that hunk is unstaged | unit | `cd src-tauri && cargo test --lib commands::staging::tests::unstage_hunk_unstages_single_hunk -- --test-threads=1` | Wave 0 |
| HUNK-03 | Discard single hunk from 2-hunk working tree diff; verify only that hunk is discarded | unit | `cd src-tauri && cargo test --lib commands::staging::tests::discard_hunk_discards_single_hunk -- --test-threads=1` | Wave 0 |
| HUNK-05 | Commands return Result<(), String>; re-fetch shows updated state | unit | Covered by above tests (they verify state after operation) | Wave 0 |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test --lib commands::staging -- --test-threads=1`
- **Per wave merge:** `cd src-tauri && cargo test --lib -- --test-threads=1`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `stage_hunk_stages_single_hunk` test -- multi-hunk fixture + stage hunk 0, verify hunk 0 is staged and hunk 1 is not
- [ ] `stage_hunk_stale_index` test -- attempt hunk_index=5 on 2-hunk file, expect stale_hunk_index error
- [ ] `unstage_hunk_unstages_single_hunk` test -- stage file, then unstage one hunk, verify partial staging
- [ ] `discard_hunk_discards_single_hunk` test -- modify two regions, discard one hunk, verify the other remains
- [ ] `stage_hunk_file_not_found` test -- attempt on non-existent file, expect file_not_found error
- [ ] Multi-hunk file creation helper function (shared fixture)

## Sources

### Primary (HIGH confidence)
- git2 0.19.0 source code (`patch.rs`, `apply.rs`, `diff.rs`, `repo.rs`) -- read directly from `/Users/joaofnds/.local/share/mise/installs/rust/1.93.1/registry/src/index.crates.io-1949cf8c6b5b557f/git2-0.19.0/src/`
- git2 `ApplyOptions::hunk_callback()` implementation: `true` -> 0 (apply), `false` -> 1 (skip) -- verified from `apply.rs` line 79-84
- git2 `ApplyOptions` test `apply_hunks_and_delta()` -- confirms `ApplyLocation::Index` works with `diff_index_to_workdir` and callbacks
- git2 `DiffOptions::reverse()` -- verified from `diff.rs` line 694-697, maps to `GIT_DIFF_REVERSE`
- git2 `Diff::from_buffer()` returns `Diff<'static>` -- verified from `diff.rs` line 315
- Existing codebase: `staging.rs`, `diff.rs`, `types.rs`, `error.rs`, `lib.rs` patterns

### Secondary (MEDIUM confidence)
- [libgit2 git_apply_options docs](https://libgit2.org/docs/reference/main/apply/git_apply_options.html) -- hunk_cb return value semantics (0=apply, >0=skip, <0=abort)
- [Rust forum: applying git patches](https://users.rust-lang.org/t/getting-an-error-when-trying-to-apply-git-patch-with-git2/63258) -- confirms Diff::from_buffer + repo.apply workflow
- [libgit2 issue #5153](https://github.com/libgit2/libgit2/issues/5153) -- no-newline-at-EOF bug in Diff::from_buffer, fixed in libgit2 via PR #5159 (2019)

### Tertiary (LOW confidence)
- Untracked file behavior with `repo.apply(ApplyLocation::Index)` -- not verified; flagged for implementation-time testing
- Reversed diff behavior for "entire new file" unstaging -- not verified; needs explicit test

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- git2 0.19.0 already in use; all APIs verified from source
- Architecture: HIGH -- hunk_callback approach verified from git2 source and test suite; inner-fn pattern matches existing codebase
- Pitfalls: HIGH -- identified from source code analysis and real-world issue reports
- Untracked file edge case: LOW -- needs implementation-time verification

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable APIs, no expected changes in git2 0.19)
