# Phase 32: Hunk Staging Backend - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement Rust commands for staging, unstaging, and discarding individual hunks using git2's apply API with single-hunk patch extraction. Backend only — no UI changes. Requirements: HUNK-01, HUNK-02, HUNK-03, HUNK-05.

</domain>

<decisions>
## Implementation Decisions

### Hunk identification strategy
- Frontend identifies hunks by **array index** (0-based position in `Vec<DiffHunk>`)
- Backend re-fetches the diff internally and picks hunk at position N
- No stable hunk IDs needed — frontend re-fetches full diff after each operation and re-renders with current indices
- Command signature follows existing pattern: `stage_hunk(path: String, file_path: String, hunk_index: u32)` — same param names as `stage_file`/`diff_unstaged`

### Response after hunk operations
- Commands return `Result<(), String>` (success/failure only) — matches existing mutation commands
- Frontend re-fetches diff and status separately after each operation
- Mutation and query stay separated — no bundled response data
- Local IPC round-trip is sub-millisecond, no perceived delay

### Stale hunk behavior
- Backend returns typed error on apply failure — frontend re-fetches diff as part of error handler
- No backend auto-retry or closest-match logic — explicit fail, no magic
- FS watcher (300ms debounce) may already trigger status refresh for external changes
- On failure, backend does NOT emit 'repo-changed' event (nothing changed) — matches Phase 28 discard pattern

### Error codes
- Distinct error codes for different failure modes:
  - `stale_hunk_index` — hunk index out of bounds (file has fewer hunks than expected)
  - `hunk_apply_failed` — patch doesn't match working tree (file changed since diff was fetched)
  - `file_not_found` — file no longer exists
- Frontend can show targeted toast messages per error code

### Discard hunk safety
- Backend trusts the frontend has confirmed — no backend-side safeguard
- Matches existing `discard_file` pattern where frontend shows confirmation dialog before invoking

### Claude's Discretion
- Single-hunk patch extraction implementation (how to slice a Diff into a single-hunk patch for `repo.apply()`)
- Whether to use `Patch::from_diff()`, manual patch construction, or `Diff::from_buffer()` approach
- Test fixture design for multi-hunk files, single-hunk new files, and no-newline-at-EOF
- Whether hunk commands go in `staging.rs` or a new `hunk.rs` file
- `unstage_hunk` reversed patch construction approach

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing staging implementation
- `src-tauri/src/commands/staging.rs` — Current stage/unstage/discard commands with inner-fn pattern. New hunk commands should follow the same structure.
- `src-tauri/src/commands/diff.rs` — Diff generation using `diff.foreach()` with file/hunk/line callbacks. The `walk_diff_into_file_diffs` function shows how hunks are extracted from git2.

### Type definitions
- `src-tauri/src/git/types.rs` §119-161 — Rust `DiffHunk`, `DiffLine`, `FileDiff` structs that define the data model
- `src/lib/types.ts` §85-108 — TypeScript mirror types (`DiffHunk`, `DiffLine`, `FileDiff`)

### Command registration
- `src-tauri/src/lib.rs` §21-65 — `invoke_handler` list where new commands must be registered
- `src-tauri/src/commands/mod.rs` — Module declarations for command files

### git2 crate
- `Cargo.toml` §25 — `git2 = { version = "0.19", features = ["vendored-libgit2"] }`
- git2 docs for `Repository::apply()`, `Patch::from_diff()`, `ApplyLocation::Index`, `ApplyLocation::Workdir`

### Frontend integration points
- `src/components/DiffPanel.svelte` — Current diff rendering (no interactive hunk elements yet — Phase 33 scope)
- `src/components/StagingPanel.svelte` — Stage/unstage/discard flow with `safeInvoke` calls and `loadStatus()` refresh pattern
- `src/components/App.svelte` §120-133 — `handleFileSelect` showing how diff kind (unstaged/staged) flows to DiffPanel

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `open_repo_from_state()`: Opens git2 Repository from path + state map — used by every command
- `walk_diff_into_file_diffs()`: Converts git2 Diff to Vec<FileDiff> using foreach callbacks — can be used to re-fetch diff for hunk extraction
- `TrunkError`: Structured error type with code + message, JSON-serialized at command boundary
- `safeInvoke<T>`: Frontend IPC wrapper with error handling and type safety
- Inner-fn pattern: `stage_file_inner` / `unstage_file_inner` / `discard_file_inner` as templates

### Established Patterns
- Inner-fn pattern: sync `_inner` function with `&str`/`&HashMap` params → async `#[tauri::command]` wrapper with `spawn_blocking`
- Error handling: `Result<T, TrunkError>` in inner fns → `Result<T, String>` (JSON-serialized) at command boundary
- Index manipulation: `repo.index()` + `index.add_path()` + `index.write()` for staging
- Unstaging: `repo.reset_default()` for HEAD repos, `index.remove_path()` for unborn HEAD
- Discard: `git2::build::CheckoutBuilder` with `.path().force()` for tracked files, `fs::remove_file` for untracked
- Tests: `make_test_repo()` helper with `tempfile::TempDir`, `make_state_map()` for HashMap construction

### Integration Points
- New commands register in `lib.rs` invoke_handler
- Commands live in `commands/staging.rs` (or new file declared in `commands/mod.rs`)
- Frontend will call new commands via `safeInvoke('stage_hunk', { path, filePath, hunkIndex })` (Phase 33)
- DiffOptions with `.pathspec(file_path)` filters diff to single file — reuse this for hunk extraction

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The git2 `repo.apply()` API with `ApplyLocation::Index` / `ApplyLocation::Workdir` is the established approach for hunk-level operations.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 32-hunk-staging-backend*
*Context gathered: 2026-03-17*
