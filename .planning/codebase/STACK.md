# Technology Stack

**Analysis Date:** 2026-05-14

## Languages

**Primary:**
- Rust 1.93.1 — Tauri backend, all git operations, file watching, syntax highlighting
- TypeScript 5.6 (strict) — Frontend logic, lib utilities, Tauri IPC wrappers

**Secondary:**
- Svelte 5 (`.svelte` files with runes) — UI component layer
- CSS (Tailwind CSS 4) — Styling via utility classes and CSS custom properties only (no inline colors)
- JavaScript — E2E test specs (`e2e/specs/*.e2e.js`)

## Runtime

**Environment:**
- Desktop application (Tauri 2) — webview embeds the Svelte frontend; native Rust process handles git
- Node-equivalent: Bun 1.3.8 (runtime and package manager for frontend)

**Package Manager:**
- Bun 1.3.8 — lockfile at `bun.lock` (present; also `package-lock.json` for npm compatibility)
- Cargo — lockfile at `src-tauri/Cargo.lock`

## Frameworks

**Core:**
- Tauri 2 — desktop shell, IPC, native menus, plugin system; config at `src-tauri/tauri.conf.json`
- Svelte 5 — UI framework using runes (`$state`, `$derived`); entry at `src/App.svelte`, `src/main.ts`
- Vite 6 — frontend bundler / dev server (port 1420); config at `vite.config.ts`
- Tailwind CSS 4 — utility-first CSS via `@tailwindcss/vite` plugin; global styles at `src/app.css`

**Testing:**
- Vitest 4.1 — unit and component tests; config embedded in `vite.config.ts`
- @testing-library/svelte 5.3.1 — Svelte component test helpers
- jsdom 29.0.1 — DOM environment for vitest
- @wdio/cli 9.27 (WebdriverIO + Mocha) — E2E tests via `tauri-driver`; config at `e2e/wdio.conf.js`
- cargo-llvm-cov — Rust coverage (lcov + HTML output)

**Build/Dev:**
- just 1.48.1 — task runner; `justfile` at repo root
- mise 2 — version management for bun, just, rust; config at `mise.toml`
- @tauri-apps/cli 2 — `bunx tauri` CLI for dev/build

**Linting/Formatting:**
- Biome 2.4.9 — TypeScript/JS/Svelte linting and formatting; config at `biome.json`
- svelte-check 4 — Svelte TypeScript type checking
- cargo fmt — Rust formatting
- cargo clippy — Rust linting (`-D warnings` in CI)

## Key Dependencies

**Critical (Rust):**
- `git2 0.19` (`vendored-libgit2`, `vendored-openssl`) — all git operations; `src-tauri/src/git/`
- `notify 7` + `notify-debouncer-mini 0.5` — filesystem watching for repo change events; `src-tauri/src/watcher.rs`
- `tokio 1` (`process`, `io-util` features) — async runtime, async subprocess for remote ops; `src-tauri/src/commands/remote.rs`
- `similar 2.7` (`inline` feature) — word-level diff computation; `src-tauri/src/commands/diff.rs`
- `syntect 5` (`default-onig`) — syntax highlighting for diff view (base16-ocean.dark theme); `src-tauri/src/git/syntax.rs`
- `serde 1` + `serde_json 1` — JSON serialization for Tauri IPC
- `libc 0.2` — SIGTERM for cancelling remote processes on Unix

**Critical (Frontend):**
- `@tauri-apps/api 2` — IPC (`invoke`, `listen`, `event`), menus, window, webview
- `@tauri-apps/plugin-store 2.4.2` — persistent JSON preferences store (`trunk-prefs.json`); `src/lib/store.ts`
- `@tauri-apps/plugin-dialog 2.6` — native open/save/ask/message dialogs
- `@tauri-apps/plugin-clipboard-manager 2.3.2` — clipboard write access
- `@tauri-apps/plugin-window-state 2.4.1` — persists window size/position across restarts
- `@humanspeak/svelte-virtual-list 0.4.2` — virtual scrolling for large commit graphs
- `@lucide/svelte 0.577` — icon library
- `sortablejs 1.15.7` — drag-and-drop for rebase editor row reordering

**Dev/Testing (Rust):**
- `tempfile 3` — temporary directories in Rust tests
- `tauri 2` (test feature) — Tauri test harness
- `criterion 0.8` (html_reports) — micro-benchmarks; `src-tauri/benches/`

## Configuration

**Build:**
- `src-tauri/tauri.conf.json` — app identifier `com.joaofnds.trunk`, version `0.12.7`, window config (title bar overlay, 800×600 default), macOS entitlements, bundle targets
- `src-tauri/Cargo.toml` — Rust deps, three bench targets (`bench_graph`, `bench_commands`, `bench_ipc`)
- `tsconfig.json` — strict TS, `bundler` resolution, `$lib` path alias → `src/lib`
- `vite.config.ts` — dev server on port 1420 (strict), `svelte()` + `tailwindcss()` + `svelteTesting()` plugins, vitest config (jsdom, v8 coverage)
- `biome.json` — recommended rules, import organization, Svelte-specific rule overrides
- `mise.toml` — pins bun 1.3.8, just 1.48.1, rust 1.93.1

**Environment:**
- No `.env` files required for development — app reads local git repos directly
- Env vars set by build process: `TAURI_DEV_HOST` (optional, for mobile/remote dev)
- macOS PATH fix: `shell_env::system_path()` reads `/usr/libexec/path_helper` to resolve Homebrew-installed git

## Platform Requirements

**Development:**
- macOS, Linux, or Windows with Rust 1.93.1+, bun 1.3.8, just 1.48.1
- Linux additionally requires: `libwebkit2gtk-4.1-dev`, `libxdo-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
- E2E tests on Linux additionally need: `webkit2gtk-driver`, `xvfb`

**Production:**
- macOS: `.app` bundle (`.dmg`), arm64 and x86_64 targets; distributed via Homebrew tap (`joaofnds/homebrew-tap`)
- Linux: `.AppImage`
- Windows: `.msi` + portable `.exe`
- App signing: macOS uses `-` (ad-hoc signing); `src-tauri/Entitlements.plist` required

---

*Stack analysis: 2026-05-14*
