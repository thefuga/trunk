# Stack Research: CI/CD & Cross-Platform Release Publishing

**Domain:** Desktop app CI/CD pipeline (Tauri 2 + Svelte 5 + Rust)
**Researched:** 2026-03-25
**Confidence:** HIGH

## Recommended Stack

### Core CI/CD Actions

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `tauri-apps/tauri-action` | `@v0` (v0.6.2) | Build + bundle + upload Tauri app per platform | Official Tauri action. Handles Rust compilation, frontend build, bundling (.dmg/.AppImage/.nsis), and GitHub Release upload in one step. v0 is the stable release line (v0.6.2 latest patch, March 2025). A v1 exists on the dev branch but is NOT stable-released -- do not use it. |
| `orhun/git-cliff-action` | `@v4` (v4.7.1) | Generate changelog from conventional commits | git-cliff v2.12.0 is the Rust-native changelog generator. 120ms for 10k commits vs 30s for Node-based alternatives. Project already uses conventional commits (`feat:`, `fix:`, `chore:`, `docs:`), so it maps directly with zero workflow changes. |
| `actions/checkout` | `@v4` | Repository checkout | Standard. Must use `fetch-depth: 0` for release workflow (git-cliff needs full history to generate changelog). CI workflow can use default shallow clone. |
| `oven-sh/setup-bun` | `@v2` (v2.2.0) | Install Bun runtime on runners | Project uses `bun.lock` (text format). GitHub runners do NOT pre-install Bun. tauri-action auto-detects `bun.lock` and expects Bun on PATH. Without this step, builds fail with `bun: not found`. |
| `dtolnay/rust-toolchain` | `@stable` | Install Rust toolchain | Industry standard replacement for the deprecated/archived `actions-rs/toolchain`. Concise, actively maintained. `@stable` resolves to current stable (1.93.x as of March 2026). |
| `Swatinem/rust-cache` | `@v2` | Cache Cargo build artifacts | Saves 2-5 minutes per build. Uses Cargo.lock hash for cache keys. Essential given git2's vendored-libgit2 compile time (~3-5 min uncached). Must be placed AFTER `dtolnay/rust-toolchain`. |

### Dependabot

| Technology | Config Version | Purpose | Why Recommended |
|------------|----------------|---------|-----------------|
| GitHub Dependabot | v2 (config format) | Automated dependency PRs for Cargo + npm | Built into GitHub, zero maintenance overhead. Covers both `cargo` (Rust crates) and `npm` (package.json) ecosystems. Note: Bun is not a separate Dependabot ecosystem -- use `npm` ecosystem with the root directory containing `package.json`. |

### Bundle Targets per Platform

| Platform | GitHub Runner | Bundle Format | Output File | Notes |
|----------|---------------|---------------|-------------|-------|
| macOS ARM (Apple Silicon) | `macos-latest` | dmg | `trunk_x.y.z_aarch64.dmg` | Native build on ARM runner |
| macOS Intel | `macos-latest` | dmg | `trunk_x.y.z_x64.dmg` | Cross-compiled via `--target x86_64-apple-darwin` on same ARM runner |
| Linux x64 | `ubuntu-22.04` | appimage, deb | `.AppImage`, `.deb` | Use 22.04 (not 24.04) for widest glibc compatibility |
| Windows x64 | `windows-latest` | nsis | `trunk_x.y.z_x64-setup.exe` | NSIS over MSI: cross-compilable, per-user install, consumer-friendly |

### Supporting Tools

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `git-cliff` | 2.12.0 | Local changelog generation | Optional local install for previewing changelogs during development. CI uses the GitHub Action. |
| `prettier` | latest | Frontend code formatting | New devDependency. CI runs `prettier --check`. Not yet configured (no `.prettierrc` found). |
| `prettier-plugin-svelte` | latest | Svelte file formatting for Prettier | Required companion for Prettier to handle `.svelte` files. |

### Development Tools (CI Steps)

| Tool | CI Command | Purpose | Notes |
|------|------------|---------|-------|
| `cargo fmt` | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Rust formatting check | Check-only mode, no writes. No `rustfmt.toml` exists -- defaults are fine. |
| `cargo clippy` | `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | Rust linting | `-D warnings` fails CI on any clippy warning. Catches common mistakes. |
| `cargo test` | `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit tests | Project has `#[cfg(test)]` modules in 12+ files (staging, branches, graph, etc.). |
| `cargo check` | `cargo check --manifest-path src-tauri/Cargo.toml` | Rust type checking | Faster than full build. Use as first Rust CI step for quick failure. |
| `bun run check` | `bun run check` | svelte-check (TypeScript) | Already in package.json. Runs `svelte-check --tsconfig ./tsconfig.json`. |
| `bun run test` | `bun run test` | Vitest frontend tests | Already in package.json. Runs `vitest run`. |
| `prettier` | `bunx prettier --check "src/**/*.{ts,svelte,css}"` | Frontend formatting | New addition. Needs `.prettierrc` and `.prettierignore` files created. |

## Installation

No new runtime dependencies. All additions are CI-only (GitHub Actions config) or dev tooling.

```bash
# Dev dependency for formatting checks (only new npm packages)
bun add -D prettier prettier-plugin-svelte

# Optional: git-cliff for local changelog preview
cargo install git-cliff
# Or without installing: bunx git-cliff
```

### New Files to Create

```
.github/workflows/ci.yml              # Lint + test on every push and PR
.github/workflows/release.yml         # Build all platforms + publish on tag push
.github/dependabot.yml                # Automated dependency update PRs
cliff.toml                            # git-cliff changelog configuration
.prettierrc                           # Prettier formatting config
.prettierignore                       # Exclude build artifacts from formatting
```

## Architecture: Two-Workflow Design

### Workflow 1: CI (`ci.yml`) -- Every Push + PR

```
Trigger: push to any branch, pull_request targeting main
Runner: ubuntu-latest (single platform, fastest feedback)
Purpose: Code quality gate -- fail fast on lint/test/type errors

Steps:
  1. actions/checkout@v4
  2. oven-sh/setup-bun@v2
  3. bun install
  4. dtolnay/rust-toolchain@stable
  5. Swatinem/rust-cache@v2
  6. cargo fmt --check
  7. cargo clippy -- -D warnings
  8. cargo test
  9. bun run check (svelte-check)
  10. bun run test (vitest)
  11. bunx prettier --check
```

No matrix needed. Single `ubuntu-latest` runner catches all lint and test failures. Cross-platform compilation is only needed for releases.

### Workflow 2: Release (`release.yml`) -- Tag Push

```
Trigger: push tags matching 'v*' (e.g., v0.10.0)
Purpose: Build all platforms, generate changelog, publish GitHub Release

Job 1 -- Changelog:
  1. actions/checkout@v4 (fetch-depth: 0)
  2. orhun/git-cliff-action@v4 (generate release body)
  3. Create draft GitHub Release with changelog body

Job 2 -- Build (matrix, depends on Job 1):
  Matrix:
    - { platform: macos-latest, args: '--target aarch64-apple-darwin' }
    - { platform: macos-latest, args: '--target x86_64-apple-darwin' }
    - { platform: ubuntu-22.04, args: '' }
    - { platform: windows-latest, args: '' }
  Steps:
    1. actions/checkout@v4
    2. oven-sh/setup-bun@v2
    3. bun install
    4. dtolnay/rust-toolchain@stable (with targets for macOS cross-compile)
    5. Swatinem/rust-cache@v2
    6. Install Linux deps (Ubuntu only)
    7. tauri-apps/tauri-action@v0 (build + upload to release from Job 1)
```

Release is created as **draft** so the developer can review artifacts and changelog before publishing.

### Dependabot Config (`dependabot.yml`)

```
Ecosystems:
  - cargo (directory: /src-tauri, weekly, label: dependencies)
  - npm (directory: /, weekly, label: dependencies)
  - github-actions (directory: /, weekly, label: dependencies)
```

Include `github-actions` ecosystem to keep action versions updated (checkout, setup-bun, rust-toolchain, etc.).

## Key Integration Points

### tauri.conf.json -- No Changes Needed

Current config has `"targets": "all"` in the bundle section. This is correct -- tauri-action automatically selects platform-appropriate targets. The `args` matrix parameter handles target architecture (`--target x86_64-apple-darwin` for Intel Mac cross-compile).

### Version Management

Version is currently `0.1.0` in `package.json`, `tauri.conf.json`, and `Cargo.toml`. Before creating the first release tag:
1. Bump all three files to the actual release version (e.g., `0.10.0`)
2. Commit the version bump
3. Tag with `v0.10.0`
4. Push the tag -- this triggers the release workflow

The `__VERSION__` placeholder in tauri-action reads from `tauri.conf.json`'s `version` field.

### Tag Convention

Use `v{VERSION}` tags (e.g., `v0.10.0`). Configure tauri-action with:
- `tagName: v__VERSION__` -- creates/finds the tag matching the app version
- `releaseName: Trunk v__VERSION__` -- human-readable release title

Configure git-cliff's `tag_pattern` to match `v[0-9]*`.

### Linux Build Dependencies

The Ubuntu runner needs system packages for Tauri 2 (WebKit2GTK 4.1):

```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

### Bun Lock File Detection

The project uses `bun.lock` (Bun's newer text format, not the old binary `bun.lockb`). Tauri CLI detection was fixed in PR #12998 (March 2025) to recognize both formats. tauri-action inherits this fix. The `oven-sh/setup-bun@v2` step must come before `tauri-action` so Bun is on PATH when detected.

## cliff.toml Configuration

The project's conventional commit style maps cleanly to changelog groups:

| Commit Prefix | Changelog Group | Include in Changelog |
|---------------|-----------------|----------------------|
| `feat:` / `feat(scope):` | Features | Yes |
| `fix:` / `fix(scope):` | Bug Fixes | Yes |
| `chore:` | Miscellaneous | Skip (internal) |
| `docs:` | Documentation | Skip (GSD planning artifacts) |
| `refactor:` | Refactoring | Skip (internal) |

Skip `docs:` commits because the project's commit history is heavily docs-commit-laden from GSD planning workflow -- these are not user-facing changes.

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `tauri-apps/tauri-action@v0` | Manual `cargo tauri build` steps | Never for this project. tauri-action handles bundling, artifact naming, release upload, and updater JSON in one step. Manual approach requires 50+ lines of YAML duplicating all of this. |
| `git-cliff` (Rust) | `conventional-changelog` (Node) | If project were Node-only. git-cliff is ~250x faster and Rust-native, matching this project's stack. |
| `git-cliff` | GitHub auto-generated release notes | If you want zero configuration. GitHub's built-in notes list PR titles only and don't group by conventional commit type. git-cliff produces a properly grouped, customizable changelog. |
| NSIS (`.exe`) for Windows | MSI (`.msi`) | For enterprise/Group Policy deployment. MSI requires a Windows-only build (WiX Toolset cannot cross-compile). NSIS works on all host platforms, supports per-user install, and is the better choice for a consumer desktop app. |
| `ubuntu-22.04` for Linux builds | `ubuntu-24.04` | When minimum glibc compatibility is not a concern. 22.04 produces binaries compatible with older Linux systems. 24.04 raises the glibc floor, reducing compatibility. Stick with 22.04 for widest user reach. |
| `Swatinem/rust-cache@v2` | `actions/cache` (generic) | Never for Rust. rust-cache understands Cargo.lock, target directory structure, and registry cache layout. Generic cache requires manual key configuration and misses optimizations. |
| Dependabot | Renovate | For monorepo organizations needing grouped PRs, automerge rules, and custom merge strategies. Dependabot is simpler and built into GitHub with zero setup beyond the YAML file. Sufficient for this project. |
| Draft release (manual publish) | Auto-publish | If you trust the pipeline fully and want zero manual steps. Starting with draft releases lets you review artifacts and changelog before they go public. Can switch to auto-publish later. |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `actions-rs/toolchain` | Archived October 2023, known bugs, unmaintained | `dtolnay/rust-toolchain@stable` |
| `actions-rs/cargo` | Same: archived, unmaintained | Direct `cargo` commands via `run:` steps |
| `tauri-apps/tauri-action@v1` | Dev branch only, not stable-released. Using it risks breaking changes. | `tauri-apps/tauri-action@v0` (v0.6.2 stable) |
| MSI bundle target | WiX Toolset requires Windows host. Cannot cross-compile. Limits CI flexibility. | NSIS -- builds on all platforms |
| `actions/setup-node` | Project uses Bun. tauri-action detects `bun.lock` and calls Bun. Installing Node creates confusion and doesn't help. | `oven-sh/setup-bun@v2` |
| `semantic-release` (Node) | Overkill for a personal desktop app. Adds Node runtime, complex plugin config, assumes npm publishing. | `git-cliff` for changelog + `tauri-action` for release -- two simple tools covering the same flow |
| `cargo-release` | Designed for publishing Rust libraries to crates.io. This is a desktop app. Version lives in `tauri.conf.json`. | Manual version bump in 3 files before tagging |
| `release-please` (Google) | Designed for monorepos publishing multiple packages. Over-automated for a single-app desktop project. | Manual tag push to trigger release |
| CrabNebula Cloud | Paid SaaS for Tauri distribution with auto-updates. Out of scope (auto-updates deferred to v1.0). | GitHub Releases for now |

## Version Compatibility

| Component A | Compatible With | Notes |
|-------------|-----------------|-------|
| `tauri-apps/tauri-action@v0` | Tauri 2.x | Reads version from `tauri.conf.json`. Supports `bun.lock` detection (tauri-cli PR #12998, March 2025). |
| `tauri-apps/tauri-action@v0` | `oven-sh/setup-bun@v2` | tauri-action auto-detects Bun from lock file. Bun must be installed first via setup-bun. |
| `orhun/git-cliff-action@v4` | `actions/checkout@v4` | Requires `fetch-depth: 0` on checkout. Without full history, git-cliff produces empty or errored changelogs. |
| `Swatinem/rust-cache@v2` | `dtolnay/rust-toolchain@stable` | rust-cache MUST come after toolchain setup. Cache keys incorporate toolchain version + Cargo.lock hash. |
| `git2 = "0.19"` (vendored-libgit2) | `ubuntu-22.04` runner | Vendored libgit2 compiles from source. Slow first build (~3-5 min), fast with cache (<30s). Runner needs `libwebkit2gtk-4.1-dev` for Tauri bundling, not for git2. |
| NSIS bundle | All runner platforms | NSIS cross-compiles on macOS/Linux/Windows. MSI does NOT cross-compile. |
| AppImage bundle | `ubuntu-22.04` only | Requires `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf` system packages. |
| DMG bundle | `macos-latest` only | Both `aarch64-apple-darwin` and `x86_64-apple-darwin` targets build on the same Apple Silicon runner via cross-compilation. |
| Dependabot `npm` ecosystem | `bun.lock` | Dependabot reads `package.json` for version info. It does not need to understand the lock file format -- it creates PRs that update `package.json` ranges. |

## Sources

- [Tauri 2 GitHub Actions guide](https://v2.tauri.app/distribute/pipelines/github/) -- Official workflow examples with matrix strategy (HIGH confidence)
- [tauri-apps/tauri-action repository](https://github.com/tauri-apps/tauri-action) -- Input/output spec, Tauri v2 support (HIGH confidence)
- [tauri-action releases](https://github.com/tauri-apps/tauri-action/releases) -- v0.6.2 is latest stable (March 2025) (HIGH confidence)
- [Tauri Windows Installer docs](https://v2.tauri.app/distribute/windows-installer/) -- NSIS vs MSI trade-offs (HIGH confidence)
- [git-cliff repository](https://github.com/orhun/git-cliff) -- v2.12.0, January 2026 (HIGH confidence)
- [git-cliff-action repository](https://github.com/orhun/git-cliff-action) -- v4.7.1, February 2026 (HIGH confidence)
- [oven-sh/setup-bun](https://github.com/oven-sh/setup-bun) -- v2.2.0, verified on GitHub Marketplace (HIGH confidence)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) -- Stable, actively maintained, recommended by Rust community (HIGH confidence)
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) -- v2.x, smart Cargo caching (HIGH confidence)
- [GitHub Dependabot configuration docs](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuring-dependabot-version-updates) -- v2 config format (HIGH confidence)
- [Tauri bun.lock detection fix (issue #12914)](https://github.com/tauri-apps/tauri/issues/12914) -- PR #12998 merged March 2025, both bun.lock and bun.lockb supported (HIGH confidence)
- [tauri-action bun issue #986](https://github.com/tauri-apps/tauri-action/issues/986) -- Workaround: install Bun via setup-bun before tauri-action (HIGH confidence)

---
*Stack research for: CI/CD & Cross-Platform Release Publishing (Trunk v0.10)*
*Researched: 2026-03-25*
