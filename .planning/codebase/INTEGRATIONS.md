# External Integrations

**Analysis Date:** 2026-05-14

## APIs & External Services

**None (network):**
- Trunk makes no HTTP calls to external APIs. There is no analytics, telemetry, crash reporting, or cloud sync.

## Data Storage

**Databases:**
- None — no SQL or NoSQL database.

**Persistent Preferences (local):**
- Tauri Plugin Store (`@tauri-apps/plugin-store`, `tauri-plugin-store 2.4.2`)
  - File: `trunk-prefs.json` stored in the OS app data directory (`~/Library/Application Support/com.joaofnds.trunk/` on macOS)
  - Client: `LazyStore` from `@tauri-apps/plugin-store`; wrapped in `src/lib/store.ts`
  - Keys stored: `recent_repos`, `zoom_level`, `left_pane_width`, `right_pane_width`, `left_pane_collapsed`, `right_pane_collapsed`, `open_repo`, `column_widths`, `column_visibility`, `rebase_column_widths`, `rebase_column_visibility`, `open_tabs`, `active_tab_id`, `tree_view_enabled`, `diff_context_lines`, `diff_ignore_whitespace`, `diff_show_full_file`, `diff_content_mode`, `diff_layout_mode`, `diff_show_invisibles`, `diff_word_wrap`, `fetch_interval_ms`

**Window State:**
- Tauri Plugin Window State (`tauri-plugin-window-state 2`)
  - Persists window size/position across restarts automatically
  - No explicit API calls needed; registered as plugin in `src-tauri/src/lib.rs`

**File Storage:**
- Local filesystem only — reads the user's local git repositories via `git2`
- Interactive rebase uses temp files: `std::env::temp_dir().join("trunk-rebase-{pid}")` during rebase operations (`src-tauri/src/commands/interactive_rebase.rs`)

**Caching:**
- In-memory commit graph cache: `CommitCache` (Mutex<HashMap<String, GraphResult>>) managed in `src-tauri/src/state.rs`
  - Populated on `open_repo`, cleared on `close_repo`, sliced by `get_commit_graph`
  - Stored in Tauri managed state (not persisted across restarts)

## Authentication & Identity

**Auth Provider:**
- None — no user accounts or authentication system.
- Git remote authentication (SSH, HTTPS) is handled entirely by the user's existing git credential helpers and SSH agent. Trunk spawns `git` as a subprocess for remote operations (`git fetch`, `git push`, `git pull`) and inherits credentials from the environment.
- Auth errors are detected by parsing git stderr output in `src-tauri/src/commands/remote.rs`: `classify_git_error()` matches strings like "authentication failed", "permission denied", "could not read from remote".

## Git Subprocess Integration

**Where:** Remote operations (fetch, pull, push, delete remote branch) and operations requiring GIT_EDITOR (interactive rebase, cherry-pick, revert, merge continue)

**How:**
- Async subprocess via `tokio::process::Command` for remote ops with streaming stderr progress (`remote-progress` Tauri events)
- Sync subprocess via `std::process::Command` for GIT_EDITOR-dependent ops (interactive rebase sequence editor, commit message editor)
- PATH is always overridden with `shell_env::system_path()` to ensure git is found on macOS GUI (Homebrew etc.); see `src-tauri/src/shell_env.rs`
- GIT_SEQUENCE_EDITOR and GIT_EDITOR are set to temp shell scripts during interactive rebase; see `src-tauri/src/commands/interactive_rebase.rs`
- Process PIDs are stored in `RunningOp` state for cancel support (SIGTERM on Unix, `taskkill` on Windows); see `src-tauri/src/state.rs`

**Subprocess callers by file:**
- `src-tauri/src/commands/remote.rs` — git fetch/pull/push (async, tokio)
- `src-tauri/src/commands/interactive_rebase.rs` — git rebase -i (sync, GIT_EDITOR)
- `src-tauri/src/commands/branches.rs` — git fast-forward (sync)
- `src-tauri/src/commands/commit_actions.rs` — cherry-pick, revert, reset (sync)
- `src-tauri/src/commands/operation_state.rs` — merge/rebase continue/abort (sync)
- `src-tauri/src/shell_env.rs` — `/usr/libexec/path_helper` (macOS only, once at startup)

## Filesystem Watching

**Library:** `notify 7` + `notify-debouncer-mini 0.5`

**What:** Watches open repository directories recursively with a 300ms debounce window.

**Event emitted:** `repo-changed` Tauri event (carries repo path string) → frontend components re-fetch via `invoke`.

**Implementation:** `src-tauri/src/watcher.rs` — `start_watcher()` / `stop_watcher()`; watcher handles stored in `WatcherState` managed state.

## Tauri Plugin Surface

All Tauri plugins registered in `src-tauri/src/lib.rs`:

| Plugin | Purpose | Frontend Import |
|--------|---------|-----------------|
| `tauri-plugin-dialog` | Native open/save/ask/confirm dialogs | `@tauri-apps/plugin-dialog` |
| `tauri-plugin-store` | JSON preferences persistence | `@tauri-apps/plugin-store` |
| `tauri-plugin-window-state` | Window geometry persistence | `@tauri-apps/plugin-window-state` |
| `tauri-plugin-clipboard-manager` | Clipboard write (copy SHA, commit message) | `@tauri-apps/plugin-clipboard-manager` |

## Frontend Tauri API Usage

Beyond plugins, the frontend uses Tauri core APIs directly:

| API | Usage | Files |
|-----|-------|-------|
| `@tauri-apps/api/core` `invoke` | All git commands via IPC | `src/lib/invoke.ts`, throughout components |
| `@tauri-apps/api/event` `listen` | `repo-changed`, `remote-progress`, `search-toggle` events | `src/App.svelte`, `src/components/CommitGraph.svelte`, `src/components/RepoView.svelte`, `src/components/Toolbar.svelte` |
| `@tauri-apps/api/window` `getCurrentWindow` | Window resize/move events, maximize/fullscreen state | `src/App.svelte` |
| `@tauri-apps/api/webview` `getCurrentWebview` | Drag-and-drop file open for repos | `src/App.svelte` |
| `@tauri-apps/api/menu` `Menu`, `MenuItem`, `CheckMenuItem` | Context menus (branch sidebar, rebase editor) | `src/components/BranchSidebar.svelte`, `src/components/RebaseEditor.svelte` |

## CI/CD & Deployment

**Hosting:**
- GitHub Releases (binaries) — triggered by `v*` tags
- Homebrew tap: `joaofnds/homebrew-tap` — Casks auto-updated by release workflow

**CI Pipeline:**
- GitHub Actions (`.github/workflows/`)
  - `ci.yml` — two-gate strategy: Gate 1 (biome, cargo-fmt, svelte-check in parallel), Gate 2 (clippy, cargo-test with coverage, vitest with coverage) — runs on ubuntu-latest
  - `e2e.yml` — WebdriverIO E2E tests on ubuntu-latest with xvfb; uses `tauri-driver`
  - `benchmarks.yml` — benchmark compile-check
  - `release.yml` — cross-platform matrix build (macOS arm64, macOS x86_64, Linux x86_64, Windows x86_64); produces `.dmg`, `.AppImage`, `.msi`, portable archives; pushes Homebrew cask update

**Coverage reporting:**
- Rust: cargo-llvm-cov → `rust-lcov.info`; reported on PRs via `zgosalvez/github-actions-report-lcov`
- TypeScript: vitest v8 → `coverage/lcov.info`; reported on PRs via same action

**Secrets used in CI:**
- `GITHUB_TOKEN` — release uploads, coverage PR comments
- `HOMEBREW_TAP_TOKEN` — push access to `joaofnds/homebrew-tap` repo (release workflow only)

## Monitoring & Observability

**Error Tracking:** None — no Sentry, Bugsnag, or equivalent.

**Logs:** None — no structured logging framework. Rust errors are serialized as `TrunkError { code, message }` JSON and surfaced to the frontend via Tauri IPC error responses (`src-tauri/src/error.rs`). Frontend catches and displays these as toast notifications.

## Webhooks & Callbacks

**Incoming:** None.

**Outgoing:** None.

---

*Integration audit: 2026-05-14*
