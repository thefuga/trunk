# Phase 53: Rust Unit Tests & Test Harness - Research

**Researched:** 2026-03-26
**Domain:** Rust testing architecture, git2 test fixtures, GOOS-style test harness
**Confidence:** HIGH

## Summary

This phase establishes a GOOS-style test harness for the Rust backend and migrates all 148 existing inline tests to use it. The codebase has 54 `_inner` functions across 10 command files (plus 2 git module files with tests), all following a consistent pattern: they take `path: &str`, `state_map: &HashMap<String, PathBuf>`, and optionally `cache_map: &mut HashMap<String, GraphResult>`, returning `Result<T, TrunkError>`.

The existing tests already use real git2 repositories via `tempfile::tempdir()`, so the foundation is solid. The main work is: (1) building the `TestContext` struct with builder and driver APIs, (2) making crate modules publicly accessible for integration tests, and (3) migrating all existing inline tests to the new harness while adding missing edge case coverage.

**Primary recommendation:** Create integration tests in `src-tauri/tests/` with a shared `common/` module containing `TestContext`, builders, and assertion helpers. Change `mod commands` to `pub mod commands` (and similarly for `error`, `git`, `state`) in `lib.rs` to make `_inner` functions accessible from integration tests.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Harness lives in a separate integration test crate (`src-tauri/tests/`) with shared `common/` module -- not inline `#[cfg(test)]` modules.
- **D-02:** `TestContext` struct is the Application Runner -- manages tempdir lifecycle, repo handle, and state_map. One struct orchestrates setup and teardown.
- **D-03:** Domain-level Drivers are methods on `TestContext` that fully wrap `_inner` functions. Tests never call `_inner` functions directly -- the driver is the API (e.g., `ctx.diff_unstaged("file.txt")`, `ctx.stage_file("README.md")`).
- **D-04:** Fluent builder pattern via `TestContext::builder()` with composable methods: `.with_file()`, `.with_commit()`, `.with_branch()`, `.checkout()`, `.merge()`, `.with_conflict()`, `.build()`.
- **D-05:** Full coverage upfront -- builder supports binary files (`.with_binary_file()`), stashes (`.with_stash()`), tags (`.with_tag()`), and remote setup (`.with_remote()`) from the start.
- **D-06:** All existing inline `#[cfg(test)]` tests (14 files) are migrated to the integration test crate and rewritten to use `TestContext`/Drivers. No dual test styles -- one consistent approach.
- **D-07:** After migration, inline `#[cfg(test)]` modules and the old `make_test_repo()` / `make_state_map()` helpers are removed.
- **D-08:** Descriptive action-result naming convention: `fn modified_file_shows_in_unstaged_diff()`, `fn checkout_dirty_workdir_returns_error()`.
- **D-09:** Custom assertion helpers on `TestContext` for domain-specific checks: `ctx.assert_file_staged("file.txt")`, `ctx.assert_branch_exists("feature")`, `ctx.assert_status_clean()`.

### Claude's Discretion
- Exact Driver method signatures and return types
- Which edge cases to cover beyond the required set (empty repos, merge commits, binary files, conflict states)
- Internal implementation of builder state machine
- Test file organization within `tests/` (one file per command module vs grouped by domain)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| HARN-01 | Application Runner manages app lifecycle (start, stop, cleanup) for each test | `TestContext` struct with `TempDir`, auto-cleanup on drop, `state_map` + `cache_map` management |
| HARN-02 | Domain-level Drivers wrap raw interactions behind intention-revealing methods | Driver methods on `TestContext` wrapping all 54 `_inner` functions |
| HARN-03 | Builders and fixtures provide reusable test data setup | `TestContextBuilder` with fluent API for repo construction |
| HARN-04 | Tests read like behavior specifications, not implementation details | Naming convention (D-08) + assertion helpers (D-09) + driver abstraction (D-03) |
| UNIT-01 | All Rust backend commands have unit tests via inner-fn pattern | All 54 `_inner` functions get test coverage via integration test crate |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tempfile | 3.26.0 | Tempdir-based git repo fixtures | Already in dev-dependencies; standard for FS-based tests |
| git2 | 0.19.0 | Real git repository operations in builder | Already in dependencies; all `_inner` functions use it |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none needed) | - | - | All dependencies already present |

No new dependencies are required. The existing `tempfile = "3"` and `git2 = "0.19"` provide everything needed for the test harness.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/
  tests/
    common/
      mod.rs              # Re-exports all common modules
      context.rs          # TestContext struct (Application Runner)
      builder.rs          # TestContextBuilder (fluent fixture API)
      drivers/
        mod.rs            # Re-exports all driver modules
        staging.rs        # Staging driver methods
        diff.rs           # Diff driver methods
        branches.rs       # Branch driver methods
        commit.rs         # Commit driver methods
        commit_actions.rs # Tag, cherry-pick, revert, reset driver methods
        stash.rs          # Stash driver methods
        history.rs        # Search driver methods
        operation_state.rs # Merge/rebase state driver methods
        merge_editor.rs   # Merge conflict resolution driver methods
        interactive_rebase.rs # Rebase todo driver methods
        repo.rs           # Repo open/close validation driver methods
      assertions.rs       # Custom assertion helpers
    test_staging.rs       # Staging tests
    test_diff.rs          # Diff tests
    test_branches.rs      # Branch tests
    test_commit.rs        # Commit tests
    test_commit_actions.rs # Tag, cherry-pick, revert, reset, undo tests
    test_stash.rs         # Stash tests
    test_history.rs       # Search/graph tests
    test_operation_state.rs # Merge/rebase operation tests
    test_merge_editor.rs  # Conflict resolution tests
    test_interactive_rebase.rs # Rebase todo tests
    test_repo.rs          # Repo validation tests
    test_graph.rs         # Graph algorithm tests (from git/graph.rs)
    test_repository.rs    # Ref map tests (from git/repository.rs)
```

### Pattern 1: Module Visibility for Integration Tests

**What:** Integration tests in `src-tauri/tests/` can only access `pub` items from the crate root. Currently ALL modules in `lib.rs` are private (`mod`, not `pub mod`).

**Required change:** In `src-tauri/src/lib.rs`, change:
```rust
// Before (private -- integration tests cannot access)
mod commands;
mod error;
mod git;
mod state;
mod watcher;

// After (public -- integration tests can access via trunk_lib::commands::*)
pub mod commands;
pub mod error;
pub mod git;
pub mod state;
mod watcher;  // watcher stays private (not needed by tests)
```

**Why:** The locked decision D-01 requires tests in `tests/` directory. Rust integration tests are separate crates that can only access `pub` items. The `_inner` functions are already `pub fn` within their modules, but the parent `commands` module must also be `pub` for the chain to work.

**Impact:** This is a visibility-only change -- it does not affect runtime behavior. The Tauri command wrappers (the `#[tauri::command]` functions) will also become visible, but since they require `State<'_>` parameters that only exist in a running Tauri app, they cannot be accidentally misused.

### Pattern 2: TestContext (Application Runner)

**What:** Single struct that manages the test lifecycle.
**When to use:** Every test.
```rust
pub struct TestContext {
    _dir: tempfile::TempDir,      // Dropped last (cleanup)
    path: String,                  // String key for state_map lookups
    state_map: HashMap<String, PathBuf>,
    cache_map: HashMap<String, GraphResult>,
}

impl TestContext {
    pub fn builder() -> TestContextBuilder { ... }

    // Accessors
    pub fn path(&self) -> &str { &self.path }
    pub fn repo_path(&self) -> &Path { self._dir.path() }
    pub fn repo(&self) -> git2::Repository {
        git2::Repository::open(self._dir.path()).unwrap()
    }

    // The state_map and cache_map are passed to _inner functions
    pub fn state_map(&self) -> &HashMap<String, PathBuf> { &self.state_map }
    pub fn cache_map(&mut self) -> &mut HashMap<String, GraphResult> { &mut self.cache_map }
}
```

### Pattern 3: Fluent Builder

**What:** Constructs git repositories with specific topologies.
**When to use:** Setting up test fixtures.
```rust
let ctx = TestContext::builder()
    .with_file("README.md", "hello")
    .with_commit("Initial commit")
    .with_branch("feature")
    .checkout("feature")
    .with_file("feature.txt", "work")
    .with_commit("Feature work")
    .checkout("main")
    .merge("feature")
    .build();
```

**Builder state:** The builder maintains internal state:
```rust
pub struct TestContextBuilder {
    files: Vec<(String, Vec<u8>)>,     // Pending file writes
    commits: Vec<BuilderCommit>,       // Commit sequence
    current_branch: String,            // Track which branch we're on
    // ... builder-internal state
}
```

The builder executes all operations in `.build()`, creating the tempdir, initializing the repo, configuring user identity, and replaying the build steps to produce the desired repository state.

### Pattern 4: Driver Methods

**What:** Intention-revealing methods that wrap `_inner` function calls.
**When to use:** Every test interaction with the system under test.
```rust
// Driver methods are implemented as trait impls or direct methods on TestContext
impl TestContext {
    // Staging drivers
    pub fn get_status(&self) -> Result<WorkingTreeStatus, TrunkError> {
        trunk_lib::commands::staging::get_status_inner(
            self.path(), self.state_map()
        )
    }
    pub fn stage_file(&self, file: &str) -> Result<(), TrunkError> {
        trunk_lib::commands::staging::stage_file_inner(
            self.path(), file, self.state_map()
        )
    }
    // ... etc
}
```

For functions that need `&mut cache_map`, the driver takes `&mut self`:
```rust
impl TestContext {
    pub fn checkout_branch(&mut self, name: &str) -> Result<(), TrunkError> {
        trunk_lib::commands::branches::checkout_branch_inner(
            self.path(), name, self.state_map(), self.cache_map()
        )
    }
}
```

### Pattern 5: Assertion Helpers

**What:** Domain-specific assertions with clear error messages.
```rust
impl TestContext {
    pub fn assert_file_staged(&self, file: &str) {
        let status = self.get_status().expect("get_status failed");
        assert!(
            status.staged.iter().any(|f| f.path == file),
            "expected '{}' to be staged, but staged files are: {:?}",
            file, status.staged.iter().map(|f| &f.path).collect::<Vec<_>>()
        );
    }

    pub fn assert_branch_exists(&self, name: &str) {
        let repo = self.repo();
        assert!(
            repo.find_branch(name, git2::BranchType::Local).is_ok(),
            "expected branch '{}' to exist", name
        );
    }

    pub fn assert_status_clean(&self) {
        let status = self.get_status().expect("get_status failed");
        assert!(status.staged.is_empty(), "expected no staged files");
        assert!(status.unstaged.is_empty(), "expected no unstaged files");
        assert!(status.conflicted.is_empty(), "expected no conflicted files");
    }

    pub fn assert_head_at(&self, branch: &str) {
        let repo = self.repo();
        let head = repo.head().expect("no HEAD");
        assert_eq!(
            head.shorthand().unwrap(), branch,
            "expected HEAD at '{}', got '{}'", branch, head.shorthand().unwrap()
        );
    }
}
```

### Anti-Patterns to Avoid
- **Calling `_inner` functions directly in tests:** Always use driver methods. This is locked decision D-03. If a test calls `staging::stage_file_inner()` directly, it bypasses the driver abstraction and couples the test to implementation details.
- **Building git repos inline in tests:** All repo setup goes through the builder (D-04). Tests that construct repos manually with raw git2 calls are test debt.
- **Generic assertions:** Use domain-specific assertion helpers (D-09) instead of bare `assert!` with manual status checking. `ctx.assert_file_staged("file.txt")` is clearer than `assert!(status.staged.iter().any(|f| f.path == "file.txt"))`.
- **Keeping old inline test modules:** After migration, every `#[cfg(test)] mod tests` block in command files must be removed (D-07). No dual test locations.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tempdir management | Manual `std::fs::create_dir_all` + cleanup | `tempfile::TempDir` (auto-cleanup on drop) | Tempdir cleanup on panic, test isolation |
| Git identity config | Per-test `config.set_str()` calls | Builder handles it once in `.build()` | 14 files currently duplicate this boilerplate |
| State map construction | Per-test `HashMap::new()` + `insert()` | `TestContext` auto-constructs on `.build()` | 12 files currently have identical `make_state_map()` helpers |
| Cache map construction | Per-test `HashMap::new()` for cache | `TestContext` provides `cache_map()` accessor | Several test files build this ad-hoc |

**Key insight:** Every existing test file has 10-30 lines of identical boilerplate (make_test_repo, make_state_map, open repo, configure identity). The builder collapses this to 2-4 lines.

## Common Pitfalls

### Pitfall 1: Module Visibility Chain
**What goes wrong:** Integration tests can't see `_inner` functions even though they're `pub fn`.
**Why it happens:** In Rust, if a parent module is private (`mod commands`), its children are not accessible from outside the crate -- even if the children are `pub mod` and functions are `pub fn`.
**How to avoid:** Change `lib.rs` to export `pub mod commands`, `pub mod error`, `pub mod git`, `pub mod state`. Verify with `cargo test` that integration tests compile.
**Warning signs:** Compiler error "module `commands` is private" in integration tests.

### Pitfall 2: TempDir Dropped Too Early
**What goes wrong:** Tests fail with "file not found" or "repository not found" errors.
**Why it happens:** `tempfile::TempDir` deletes its directory when dropped. If the `TempDir` is owned by a local variable that goes out of scope before the test assertions run, the repo is gone.
**How to avoid:** `TestContext` owns the `TempDir` (field `_dir`). The leading underscore prevents "unused" warnings while ensuring the directory lives for the entire test.
**Warning signs:** Tests that pass individually but fail when run in parallel, or tests that fail with "No such file or directory".

### Pitfall 3: Git User Identity Not Configured
**What goes wrong:** `repo.signature()` fails with "user.name not set" error.
**Why it happens:** `git2::Repository::init()` creates a repo with no local config. Tests that call commit/stash/tag functions need `user.name` and `user.email` set.
**How to avoid:** The builder must configure identity in every test repo. Current `make_test_repo()` does this -- the builder must replicate it.
**Warning signs:** Tests fail with `git2::Error` about missing signature config.

### Pitfall 4: Functions That Shell Out to Git CLI
**What goes wrong:** Tests require `git` CLI to be available in PATH.
**Why it happens:** Several `_inner` functions use `std::process::Command::new("git")` instead of git2: `cherry_pick_inner`, `revert_commit_inner`, `reset_to_commit_inner`, `undo_commit_inner`, `get_fork_point_inner`, `start_interactive_rebase_blocking`. Also `merge_editor` tests currently shell out for `git merge`.
**How to avoid:** Accept that these functions require `git` in PATH (it's available in CI and on dev machines). Don't try to mock the CLI -- test with real git. Ensure `GIT_TERMINAL_PROMPT=0` is set (already done in the functions themselves).
**Warning signs:** CI failures if git is not installed (but CI already has git for checkout).

### Pitfall 5: Cache Map Required for Branch Operations
**What goes wrong:** Compiler errors when calling branch `_inner` functions.
**Why it happens:** 5 functions require `&mut HashMap<String, GraphResult>` (cache_map): `delete_branch_inner`, `checkout_branch_inner`, `fast_forward_to_inner`, `create_branch_inner`, `rename_branch_inner`. And `search_commits_inner` requires `&HashMap<String, GraphResult>`.
**How to avoid:** `TestContext` must hold a `cache_map: HashMap<String, GraphResult>`. Driver methods for these functions take `&mut self`.
**Warning signs:** Compilation errors about missing cache_map argument.

### Pitfall 6: Integration Test Compilation Time
**What goes wrong:** Adding 10+ test files causes long compilation.
**Why it happens:** Each file in `tests/` compiles as a separate crate. But with a shared `common/` module, the common code is compiled once per test binary.
**How to avoid:** This is acceptable overhead. The `common/` module pattern (via `tests/common/mod.rs`) is compiled once per `cargo test` invocation and shared. Alternatively, all tests can be combined into fewer files if compilation time becomes an issue.
**Warning signs:** `cargo test` takes significantly longer than current 5.76s.

### Pitfall 7: Test Parallelism and Filesystem Conflicts
**What goes wrong:** Tests interfere with each other when run in parallel.
**Why it happens:** All tests create separate tempdirs (via `TempDir::new()`), so filesystem isolation is automatic. But tests that set up git remotes between two tempdirs could theoretically race.
**How to avoid:** Each test gets its own `TestContext` with its own `TempDir`. No shared state between tests. This is already the pattern.
**Warning signs:** Flaky tests that pass individually but fail under `cargo test` (parallel execution).

## Code Examples

### Example 1: TestContext Builder Usage
```rust
// Source: Designed based on D-04 decision and existing make_test_repo() pattern
use common::TestContext;

#[test]
fn modified_file_shows_in_unstaged_diff() {
    let ctx = TestContext::builder()
        .with_file("README.md", "initial content")
        .with_commit("Initial commit")
        .build();

    // Modify file without staging
    std::fs::write(ctx.repo_path().join("README.md"), "modified content").unwrap();

    let diffs = ctx.diff_unstaged("README.md").unwrap();
    assert!(!diffs.is_empty());
    assert!(!diffs[0].hunks.is_empty());
}
```

### Example 2: Complex Fixture with Branches and Merge
```rust
#[test]
fn merge_commit_shows_two_parents() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("feature.txt", "feature work")
        .with_commit("Feature commit")
        .checkout("main")
        .merge("feature")
        .build();

    let repo = ctx.repo();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    assert_eq!(head.parent_count(), 2);
}
```

### Example 3: Binary File and Stash Fixtures
```rust
#[test]
fn binary_file_diff_marks_is_binary() {
    let ctx = TestContext::builder()
        .with_binary_file("image.png", &[0x89, 0x50, 0x4E, 0x47])
        .with_commit("Add binary file")
        .build();

    // Modify the binary file
    std::fs::write(ctx.repo_path().join("image.png"), &[0xFF, 0xD8, 0xFF]).unwrap();

    let diffs = ctx.diff_unstaged("image.png").unwrap();
    assert!(!diffs.is_empty());
    assert!(diffs[0].is_binary);
}
```

### Example 4: Assertion Helpers
```rust
#[test]
fn stage_file_moves_to_staged() {
    let ctx = TestContext::builder()
        .with_file("README.md", "hello")
        .with_commit("Initial commit")
        .build();

    std::fs::write(ctx.repo_path().join("README.md"), "modified").unwrap();
    ctx.stage_file("README.md").unwrap();

    ctx.assert_file_staged("README.md");
}
```

### Example 5: Conflict Scenario
```rust
#[test]
fn merge_conflict_shows_three_sides() {
    let ctx = TestContext::builder()
        .with_file("file.txt", "original")
        .with_commit("Initial commit")
        .with_branch("feature")
        .checkout("feature")
        .with_file("file.txt", "feature version")
        .with_commit("Feature change")
        .checkout("main")
        .with_file("file.txt", "main version")
        .with_commit("Main change")
        .with_conflict("feature")   // Starts merge that will conflict
        .build();

    let sides = ctx.get_merge_sides("file.txt").unwrap();
    assert_eq!(sides.base, "original");
    assert_eq!(sides.ours, "main version");
    assert_eq!(sides.theirs, "feature version");
}
```

## Inventory of Existing Code

### _inner Functions by Command File (54 total)

| File | Functions | Needs cache_map | Has Existing Tests |
|------|-----------|-----------------|-------------------|
| branches.rs | 7 (list_refs, delete_branch, rename_branch, resolve_ref, checkout_branch, fast_forward_to, create_branch) | 5 of 7 | 14 tests |
| commit_actions.rs | 9 (checkout_commit, create_tag, delete_tag, cherry_pick, revert_commit, reset_to_commit, undo_commit, redo_commit, check_undo_available) | 0 | 11 tests |
| commit.rs | 3 (create_commit, amend_commit, get_head_commit_message) | 0 | 6 tests |
| diff.rs | 4 (diff_unstaged, diff_staged, diff_commit, get_commit_detail) | 0 | 10 tests |
| history.rs | 1 (search_commits) | 1 (read-only) | 14 tests |
| interactive_rebase.rs | 2 (get_rebase_todo, get_fork_point) | 0 | 5 tests |
| merge_editor.rs | 2 (get_merge_sides, save_merge_result) | 0 | 3 tests |
| operation_state.rs | 8 (get_operation_state, merge_continue, merge_abort, rebase_continue, rebase_skip, rebase_abort, merge_branch, rebase_branch) | 0 | 8 tests |
| staging.rs | 13 (get_status, stage_file, unstage_file, discard_file, discard_all, stage_all, stage_hunk, unstage_hunk, discard_hunk, unstage_all, stage_lines, unstage_lines, discard_lines) | 0 | 24 tests |
| stash.rs | 5 (list_stashes, stash_save, stash_pop, stash_apply, stash_drop) | 0 | 7 tests |

### Git Module Tests (also to migrate)

| File | Tests | Notes |
|------|-------|-------|
| git/graph.rs | 22 tests | Tests for walk_commits, lane allocation, stash placement |
| git/repository.rs | 2 tests | ref_map_head, ref_map_stash |

### Additional Non-Command Tests

| File | Tests | Notes |
|------|-------|-------|
| commands/repo.rs | 6 tests | validate_and_open, state management (these don't use _inner pattern but test repo validation) |
| commands/remote.rs | 16 tests | Tests for classify_git_error and remote operations |

### Total: 148 existing tests to migrate

### Functions That Shell Out to Git CLI
These `_inner` functions use `std::process::Command::new("git")` and require git in PATH:
- `cherry_pick_inner` (commit_actions.rs)
- `revert_commit_inner` (commit_actions.rs)
- `reset_to_commit_inner` (commit_actions.rs)
- `undo_commit_inner` (commit_actions.rs)
- `get_fork_point_inner` (interactive_rebase.rs)
- `start_interactive_rebase_blocking` (interactive_rebase.rs)
- `merge_continue_inner` (operation_state.rs) -- uses `git commit`
- `rebase_continue_inner` (operation_state.rs) -- uses `git rebase --continue`
- `rebase_skip_inner` (operation_state.rs) -- uses `git rebase --skip`
- `rebase_abort_inner` (operation_state.rs) -- uses `git rebase --abort`
- `merge_abort_inner` (operation_state.rs) -- uses `git merge --abort`
- `merge_branch_inner` (operation_state.rs) -- uses `git merge`
- `rebase_branch_inner` (operation_state.rs) -- uses `git rebase`
- Remote operations in `remote.rs` -- uses `git fetch/pull/push` (async, harder to test)

### Helper Duplication to Eliminate
The following helpers are duplicated across test modules and will be absorbed into the harness:
- `make_state_map()` -- appears in 8+ test modules, identical each time
- `make_test_repo()` -- appears in 4 variants (repository.rs canonical, stash.rs, commit_actions.rs, merge_editor.rs each have their own)
- `make_test_repo_two_commits()` -- in commit_actions.rs
- `make_conflicted_repo()` -- in merge_editor.rs
- `build_cache()` -- in history.rs

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Inline `#[cfg(test)]` with duplicated helpers | Integration test crate with shared harness | This phase | Eliminates ~200 lines of duplicated boilerplate |
| Raw git2 calls in every test setup | Builder pattern via `TestContextBuilder` | This phase | 2-4 line fixture setup instead of 20-30 |
| Direct `_inner()` calls in tests | Driver methods on TestContext | This phase | Tests read as specifications, not implementations |

## Open Questions

1. **Remote operation testing scope**
   - What we know: `remote.rs` operations (fetch/pull/push) are async and shell out to git CLI with subprocess management. They require a real remote (bare repo) to test meaningfully.
   - What's unclear: Whether to include remote operation driver methods in this phase, or defer to Phase 55 (Integration Testing).
   - Recommendation: Include basic `classify_git_error` testing (already exists) and skip async remote operations. The 16 existing `remote.rs` tests can be migrated as-is since they test error classification, not actual remote ops. Full remote integration testing belongs in Phase 55.

2. **Graph algorithm test organization**
   - What we know: `git/graph.rs` has 22 tests that test the lane allocation algorithm directly (not via `_inner` functions).
   - What's unclear: Whether these should use `TestContext` drivers or remain as direct function calls since they test internal graph logic.
   - Recommendation: Migrate graph tests to integration test crate (`test_graph.rs`) using the builder for fixture setup, but call `trunk_lib::git::graph::walk_commits()` directly since there's no command-level `_inner` wrapper. This is acceptable because graph tests are testing the algorithm, not a user-facing command.

## Project Constraints (from CLAUDE.md)

- All git operations go through git2 crate, no shelling out (except GIT_EDITOR for rebase/merge message editing) -- some existing `_inner` functions already violate this by using `std::process::Command`, but that's existing code, not something this phase introduces.
- Commands live in `src-tauri/src/commands/`
- Backend stack: Tauri 2, git2 0.19 (libgit2), notify 7, tokio 1
- `cargo test` runs tests (CI gate exists at `.github/workflows/ci.yml:95`)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in, Rust 1.93.1) |
| Config file | `src-tauri/Cargo.toml` [dev-dependencies] |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HARN-01 | TestContext manages lifecycle (tempdir, state_map, cleanup) | unit | `cargo test --manifest-path src-tauri/Cargo.toml --test test_staging` | Wave 0 |
| HARN-02 | Driver methods wrap all _inner functions | unit | `cargo test --manifest-path src-tauri/Cargo.toml --test test_*` | Wave 0 |
| HARN-03 | Builder creates repos with branches, merges, conflicts | unit | `cargo test --manifest-path src-tauri/Cargo.toml --test test_branches` | Wave 0 |
| HARN-04 | Tests read as behavior specs (naming + assertions) | code review | Manual review of test names and assertion usage | N/A |
| UNIT-01 | All _inner functions have test coverage | unit | `cargo test --manifest-path src-tauri/Cargo.toml` (148+ tests pass) | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml`
- **Per wave merge:** `cargo test --manifest-path src-tauri/Cargo.toml`
- **Phase gate:** All tests pass, old inline test modules removed, no compilation errors

### Wave 0 Gaps
- [ ] `src-tauri/tests/common/mod.rs` -- TestContext, builder, drivers (does not exist yet)
- [ ] `src-tauri/tests/test_*.rs` -- Integration test files (do not exist yet)
- [ ] `pub mod commands` in `src-tauri/src/lib.rs` -- Visibility change needed

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis of all 13 command files, `git/graph.rs`, `git/repository.rs`
- Verified 148 existing tests pass via `cargo test` (5.76s)
- Cargo.toml: tempfile 3.26.0, git2 0.19.0 confirmed via `cargo metadata`
- Rust 1.93.1 / Cargo 1.93.1 confirmed via `rustc --version`
- CI gate confirmed at `.github/workflows/ci.yml:95`

### Secondary (MEDIUM confidence)
- Rust integration test documentation (The Rust Programming Language, ch. 11.3) -- `tests/` directory convention, `common/mod.rs` sharing pattern

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, verified versions from lockfile
- Architecture: HIGH -- based on locked decisions + thorough codebase analysis of all 54 _inner functions
- Pitfalls: HIGH -- identified from actual code patterns (visibility chain, cache_map requirement, CLI shelling)

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (stable domain, no external dependencies changing)
