# Architecture Research: CI/CD Pipeline & Cross-Platform Release Publishing

**Domain:** Desktop app CI/CD for Tauri 2 + Svelte 5 + Rust
**Researched:** 2026-03-25
**Confidence:** HIGH

## System Overview

```
                         GitHub Repository
                               |
              +----------------+----------------+
              |                                 |
         Push/PR trigger                  Tag push (v*)
              |                                 |
    +---------v----------+          +-----------v-----------+
    |   CI Workflow       |          |   Release Workflow     |
    |   (ci.yml)          |          |   (release.yml)        |
    +---------------------+          +------------------------+
    | ubuntu-latest only  |          | Matrix: 3 OS x targets |
    |                     |          |                        |
    | Rust checks:        |          | macos-latest:          |
    |  - cargo check      |          |   aarch64-apple-darwin |
    |  - cargo clippy      |          |   x86_64-apple-darwin  |
    |  - cargo test        |          |                        |
    |  - cargo fmt --check |          | ubuntu-22.04:          |
    |                     |          |   x86_64 (default)     |
    | Frontend checks:    |          |                        |
    |  - bun run check    |          | windows-latest:        |
    |  - bun run test     |          |   x86_64-pc-windows-   |
    |  - prettier --check |          |   msvc                 |
    +---------------------+          +------------------------+
              |                                 |
              v                                 v
         Pass/Fail gate              tauri-apps/tauri-action@v0
                                                |
                                    +-----------v-----------+
                                    | Per-platform artifacts |
                                    | macOS: .dmg            |
                                    | Linux: .AppImage, .deb |
                                    | Windows: .msi, .nsis   |
                                    +-----------+-----------+
                                                |
                                    +-----------v-----------+
                                    | GitHub Release (draft) |
                                    | + auto-generated notes |
                                    +-----------------------+

                         Dependabot
                    +-------------------+
                    | dependabot.yml    |
                    | - bun (/)         |
                    | - cargo (src-tauri)|
                    | - github-actions  |
                    +-------------------+
```

## Component Responsibilities

| Component | Responsibility | New vs Modified |
|-----------|---------------|-----------------|
| `.github/workflows/ci.yml` | Run lint, type-check, and test on every push/PR | **NEW file** |
| `.github/workflows/release.yml` | Build cross-platform binaries, publish GitHub Release on tag push | **NEW file** |
| `.github/dependabot.yml` | Automated dependency update PRs for bun, cargo, and GH Actions | **NEW file** |
| `.github/release.yml` | Categorize PRs for auto-generated release notes | **NEW file** |
| `cliff.toml` | git-cliff configuration for changelog from conventional commits | **NEW file** |
| `src-tauri/tauri.conf.json` | Bundle targets configuration (already `"all"`) | **No change needed** -- `"targets": "all"` already set |
| `package.json` | Add `format:check` script for Prettier CI | **MODIFIED** -- add script |
| `mise.toml` | Pinned tool versions (bun 1.3.8, rust 1.93.1) -- CI uses these as reference | **No change** -- read-only reference |

## New File Structure

```
.github/
├── workflows/
│   ├── ci.yml              # Lint + test on push/PR
│   └── release.yml         # Build + publish on tag push
├── dependabot.yml          # Dependency update automation
└── release.yml             # Release notes categories
cliff.toml                  # git-cliff changelog config (repo root)
```

### Structure Rationale

- **Two separate workflows** rather than one: CI runs on every push/PR (fast feedback, single runner), release runs only on tag push (expensive, multi-platform matrix). Separating them avoids wasting build minutes on every PR.
- **cliff.toml at repo root**: git-cliff expects it there by default. No flag needed.
- **`.github/release.yml`** (not a workflow): GitHub's built-in release notes categorization. Works with tauri-action's `generateReleaseNotes: true` input.

## Architectural Patterns

### Pattern 1: Tag-Triggered Release Pipeline

**What:** Release builds trigger on `push: tags: ['v*']`, not on branch push or manual dispatch.
**When to use:** When you want releases to be a deliberate act (create tag) rather than automatic on every merge.
**Trade-offs:** Requires discipline to tag properly. Prevents accidental releases. Cleanly separates "code ready" (merge to main) from "release ready" (push tag).

**Flow:**
```
Developer: git tag v0.10.0 && git push origin v0.10.0
    |
    v
GitHub Actions: release.yml triggers
    |
    v
3-4 parallel jobs (macOS ARM, macOS Intel, Linux, Windows)
    |
    v
All artifacts uploaded to draft GitHub Release
    |
    v
Developer: reviews draft, publishes
```

### Pattern 2: Matrix Strategy with Platform-Specific Config

**What:** Single workflow file uses `strategy.matrix.include` to define per-platform runner OS, Rust target, and build args.
**When to use:** Always for cross-platform Tauri builds. This is the canonical pattern from Tauri's official docs.
**Trade-offs:** `fail-fast: false` means a macOS failure does not cancel Windows/Linux builds. Slightly longer total CI time, but you get all platform results.

**Configuration:**
```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - platform: macos-latest
        args: '--target aarch64-apple-darwin'
        rust_targets: 'aarch64-apple-darwin,x86_64-apple-darwin'
      - platform: macos-latest
        args: '--target x86_64-apple-darwin'
        rust_targets: 'aarch64-apple-darwin,x86_64-apple-darwin'
      - platform: ubuntu-22.04
        args: ''
        rust_targets: ''
      - platform: windows-latest
        args: ''
        rust_targets: ''
```

**Why `ubuntu-22.04` not `ubuntu-latest`:** Tauri's WebKit2GTK dependency requires specific system packages. Ubuntu 22.04 is the explicitly tested and documented runner for Tauri builds. Using `ubuntu-latest` risks breakage when GitHub rolls forward the default runner image.

### Pattern 3: Bun Setup Before tauri-action

**What:** `oven-sh/setup-bun@v2` must be called before `tauri-apps/tauri-action@v0` because the project uses `bun.lock`. tauri-action auto-detects the lockfile and tries to run `bun` commands, but bun is not pre-installed on GitHub runners.
**When to use:** Always when the project uses bun as package manager.
**Trade-offs:** Extra setup step (~5s). Without it, tauri-action fails with `bun: not found` (documented issue #986).

**Required step order:**
```yaml
- uses: actions/checkout@v4
- uses: oven-sh/setup-bun@v2          # Must come before tauri-action
  with:
    bun-version: '1.3.8'              # Pin to mise.toml version
- uses: dtolnay/rust-toolchain@stable
  with:
    targets: ${{ matrix.rust_targets }}
- uses: swatinem/rust-cache@v2
  with:
    workspaces: './src-tauri -> target'
- run: bun install                     # Explicit install
- uses: tauri-apps/tauri-action@v0     # Detects bun.lock, uses bun
```

### Pattern 4: Concurrency Groups for Release Safety

**What:** `concurrency: { group: release-${{ github.ref }}, cancel-in-progress: true }` prevents duplicate release builds if a tag is pushed, deleted, and re-pushed quickly.
**When to use:** Always on the release workflow.

### Pattern 5: CI-Only Checks on Single Runner

**What:** Run all lint and test checks on `ubuntu-latest` only, not a full matrix. Rust and TypeScript checks do not need cross-platform validation at the lint level.
**When to use:** For the CI workflow (not release). Saves significant GitHub Actions minutes.
**Trade-offs:** A platform-specific compilation bug in Rust would not be caught until release build. Acceptable because: (a) git2 vendored-libgit2 builds are well-tested cross-platform, (b) the app has no platform-specific Rust code paths currently.

## Data Flow

### CI Workflow Trigger Flow

```
Push to any branch OR Pull Request opened/synchronized
    |
    v
ci.yml starts on ubuntu-latest
    |
    +---> Install bun (oven-sh/setup-bun@v2, bun-version: 1.3.8)
    +---> Install Rust (dtolnay/rust-toolchain@stable)
    +---> Cache: swatinem/rust-cache@v2 (workspaces: ./src-tauri -> target)
    |
    v
Parallel check groups (can use separate jobs or sequential steps):

    Rust checks (in src-tauri/):
    ├── cargo check
    ├── cargo clippy -- -D warnings
    ├── cargo test
    └── cargo fmt --all -- --check

    Frontend checks:
    ├── bun install
    ├── bun run check          (svelte-check)
    ├── bun run test           (vitest run)
    └── bunx prettier --check "src/**/*.{ts,svelte,css,html}"
```

### Release Workflow Trigger Flow

```
git tag v0.10.0 && git push origin v0.10.0
    |
    v
release.yml starts (4 parallel matrix jobs)
    |
    +---> Each job: checkout, setup bun, setup rust, install deps
    |
    +---> ubuntu-22.04: apt-get install system deps
    |         (libwebkit2gtk-4.1-dev, libappindicator3-dev,
    |          librsvg2-dev, patchelf)
    |
    +---> bun install (all jobs)
    |
    +---> tauri-apps/tauri-action@v0
    |         - Builds frontend (bun run build via tauri.conf.json beforeBuildCommand)
    |         - Compiles Rust backend
    |         - Bundles platform artifacts
    |         - Uploads to GitHub Release
    |
    v
Draft GitHub Release created with:
    - Auto-generated release notes (from PR titles since last tag)
    - macOS: 2 .dmg files (ARM + Intel)
    - Linux: .AppImage + .deb
    - Windows: .msi + .exe (NSIS)
```

### Version Synchronization Points

Three files contain version numbers that must stay in sync:

```
package.json          --> "version": "0.10.0"
src-tauri/Cargo.toml  --> version = "0.10.0"
src-tauri/tauri.conf.json --> "version": "0.10.0"
```

tauri-action reads the version from `tauri.conf.json` and substitutes `__VERSION__` in `tagName` and `releaseName`. The tag version and config version should match.

## Key Integration Points

### tauri-apps/tauri-action@v0 (core release action)

| Input | Value | Purpose |
|-------|-------|---------|
| `tagName` | `v__VERSION__` | Git tag name (version from tauri.conf.json) |
| `releaseName` | `Trunk v__VERSION__` | Display name on GitHub Release |
| `releaseDraft` | `true` | Create as draft -- manual publish required |
| `prerelease` | `false` | Not a prerelease |
| `generateReleaseNotes` | `true` | Use GitHub's auto-generated notes from PR titles |
| `args` | `${{ matrix.args }}` | Platform-specific `--target` flag |

**Outputs used:**
- `releaseId` -- all matrix jobs upload to same release
- `releaseHtmlUrl` -- URL for the draft release

### oven-sh/setup-bun@v2

| Input | Value | Purpose |
|-------|-------|---------|
| `bun-version` | `1.3.8` | Match mise.toml pinned version |

Note: Bun installs faster than Node.js. Cache is usually not needed (bun install is faster than GH Actions cache restore for most projects), but setup-bun@v2 does cache the binary itself.

### dtolnay/rust-toolchain@stable

| Input | Value | Purpose |
|-------|-------|---------|
| `targets` | Platform-dependent | macOS needs both `aarch64-apple-darwin,x86_64-apple-darwin`; others use default |

### swatinem/rust-cache@v2

| Input | Value | Purpose |
|-------|-------|---------|
| `workspaces` | `./src-tauri -> target` | Tauri's Cargo.toml is in subdirectory |

This is critical for build performance. First build: ~15-20 min for Rust compilation. Cached builds: ~3-5 min.

### Dependabot Integration

Three ecosystems configured in single `dependabot.yml`:

| Ecosystem | Directory | Schedule | Notes |
|-----------|-----------|----------|-------|
| `bun` | `/` | weekly | Reads `bun.lock` (text format, supported since bun 1.1.39) |
| `cargo` | `/src-tauri` | weekly | Reads `Cargo.toml` + `Cargo.lock` |
| `github-actions` | `/` | weekly | Keeps action versions current |

## Secrets & Environment Variables

### Required for Unsigned Builds (Phase 1)

| Secret | Required | Purpose |
|--------|----------|---------|
| `GITHUB_TOKEN` | Auto-provided | Upload release artifacts, create releases |

No additional secrets needed for unsigned builds. `GITHUB_TOKEN` is automatically available.

### Required for Code Signing (Future -- Out of Scope for v0.10)

| Secret | Platform | Purpose |
|--------|----------|---------|
| `APPLE_CERTIFICATE` | macOS | Base64-encoded .p12 signing certificate |
| `APPLE_CERTIFICATE_PASSWORD` | macOS | Certificate password |
| `APPLE_SIGNING_IDENTITY` | macOS | Certificate identity string |
| `APPLE_TEAM_ID` | macOS | Apple Developer team ID |
| `APPLE_ID` | macOS | Apple ID email for notarization |
| `APPLE_PASSWORD` | macOS | App-specific password (not Apple ID password) |
| `AZURE_CLIENT_ID` | Windows | Azure Key Vault for code signing |
| `AZURE_TENANT_ID` | Windows | Azure tenant |
| `AZURE_CLIENT_SECRET` | Windows | Azure client secret |

Code signing is explicitly out of scope for v0.10. Unsigned builds work fine for personal use and GitHub distribution. Users will see Gatekeeper/SmartScreen warnings but can bypass them.

## Changelog Generation

### Recommended: Two-Layer Approach

**Layer 1 -- GitHub Auto-Generated Release Notes** (for GitHub Release page)
- tauri-action's `generateReleaseNotes: true` input
- Configured via `.github/release.yml` categories
- Groups PRs by label into sections (Features, Bug Fixes, Other)
- Zero maintenance, works immediately

**Layer 2 -- git-cliff** (for `CHANGELOG.md` file in repo)
- Configured via `cliff.toml` at repo root
- Parses conventional commit messages
- Run locally before tagging: `git-cliff -o CHANGELOG.md`
- Groups: Features (feat), Bug Fixes (fix), Performance (perf), Refactor, etc.
- Skips: chore(deps), chore(release), CI commits

Both layers serve different purposes: GitHub Release notes are for people browsing releases on GitHub; CHANGELOG.md is for people reading the repo.

### git-cliff Configuration

```toml
[changelog]
header = """
# Changelog\n
All notable changes to this project will be documented in this file.\n
"""
body = """
{% if version %}\
## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}\
## [Unreleased]
{% endif %}\
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
- {{ commit.message | split(pat="\n") | first | upper_first | trim }}\
{% endfor %}
{% endfor %}\n
"""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
commit_parsers = [
  { message = "^feat", group = "Features" },
  { message = "^fix", group = "Bug Fixes" },
  { message = "^perf", group = "Performance" },
  { message = "^refactor", group = "Refactor" },
  { message = "^doc", group = "Documentation" },
  { message = "^test", group = "Testing" },
  { message = "^style", group = "Styling" },
  { message = "^chore\\(deps\\)", skip = true },
  { message = "^chore\\(release\\)", skip = true },
  { message = "^chore|^ci", group = "Miscellaneous" },
]
sort_commits = "oldest"
```

## Bundle Artifacts Per Platform

The existing `tauri.conf.json` has `"targets": "all"`, which produces all platform-appropriate formats automatically:

| Platform | Runner | Artifacts | Location |
|----------|--------|-----------|----------|
| macOS ARM | `macos-latest` | `.app`, `.dmg` | `target/aarch64-apple-darwin/release/bundle/` |
| macOS Intel | `macos-latest` | `.app`, `.dmg` | `target/x86_64-apple-darwin/release/bundle/` |
| Linux x64 | `ubuntu-22.04` | `.AppImage`, `.deb` | `target/release/bundle/` |
| Windows x64 | `windows-latest` | `.msi`, `.exe` (NSIS) | `target/release/bundle/` |

tauri-action automatically uploads all bundle artifacts to the GitHub Release.

## Anti-Patterns

### Anti-Pattern 1: Universal macOS Binary in CI

**What people do:** Try to build a universal macOS binary (`--target universal-apple-darwin`) in CI.
**Why it's wrong:** Universal binaries require both ARM and Intel Rust toolchains and lipo. It is simpler and more reliable to build separate ARM and Intel .dmg files. GitHub runners are all x86_64, so cross-compiling to ARM is the standard approach.
**Do this instead:** Two separate macOS matrix entries, one per architecture. Users download the correct .dmg for their Mac.

### Anti-Pattern 2: Running Full Build Matrix for CI Checks

**What people do:** Run `cargo check`, `cargo test`, `bun run check` on all 4 platform runners.
**Why it's wrong:** Wastes 4x the GitHub Actions minutes. Lint and type checks are platform-independent for this codebase.
**Do this instead:** Single `ubuntu-latest` runner for CI checks. Platform-specific compilation is validated at release time.

### Anti-Pattern 3: Using `ubuntu-latest` for Tauri Builds

**What people do:** Use `ubuntu-latest` for the release build.
**Why it's wrong:** `ubuntu-latest` tracks the newest Ubuntu LTS. When GitHub bumps it from 22.04 to 24.04, WebKit2GTK package names or versions may change, breaking builds silently.
**Do this instead:** Pin `ubuntu-22.04` for release builds. Explicit > implicit.

### Anti-Pattern 4: Auto-Publishing Releases

**What people do:** Set `releaseDraft: false` so releases go live immediately when the tag is pushed.
**Why it's wrong:** If one platform build fails, you have a published release with incomplete artifacts. No chance to review.
**Do this instead:** `releaseDraft: true`. Review the draft, verify all 4 artifacts are present, then publish manually.

### Anti-Pattern 5: Forgetting Bun Setup

**What people do:** Rely on tauri-action to handle package manager setup.
**Why it's wrong:** tauri-action detects `bun.lock` and tries to use bun, but bun is not installed on GitHub runners. Fails with `bun: not found`.
**Do this instead:** Always add `oven-sh/setup-bun@v2` before tauri-action when project uses bun.

### Anti-Pattern 6: Version Drift Between Config Files

**What people do:** Update version in `package.json` but forget `tauri.conf.json` or `Cargo.toml`.
**Why it's wrong:** tauri-action reads version from `tauri.conf.json`. If it does not match the git tag, the release will have wrong version metadata. Cargo.toml drift causes `cargo build` warnings.
**Do this instead:** Use a release script that bumps all three files atomically, or document the manual steps clearly.

## Build Performance Expectations

| Scenario | Estimated Time | Notes |
|----------|---------------|-------|
| CI workflow (all checks) | 3-5 min | Single runner, Rust cache warm |
| CI workflow (cold cache) | 10-15 min | First run or cache miss |
| Release build (per platform, warm) | 5-8 min | Rust cache + bun cache |
| Release build (per platform, cold) | 15-25 min | Full Rust compilation |
| Release total (all platforms parallel) | 15-25 min | Bottleneck is slowest platform |
| macOS with notarization (future) | +2-5 min | Apple's notarization service |

**Cache key factors:**
- `swatinem/rust-cache` uses `Cargo.lock` hash. Dependency updates invalidate cache.
- Bun install is typically faster than cache restore, so bun caching is optional.
- GitHub Actions cache has 10GB limit per repository.

## Suggested Build Order

Based on dependency analysis, the CI/CD milestone should be built in this order:

1. **CI workflow first** -- provides immediate value on every push/PR. No external dependencies or secrets needed. Validates that existing code passes all checks.

2. **Dependabot configuration** -- trivial to add alongside CI. Dependabot PRs will exercise the CI workflow, validating both.

3. **Release workflow** -- depends on CI workflow patterns being validated. Requires understanding of build matrix and tauri-action configuration. Start with unsigned builds (no secrets beyond GITHUB_TOKEN).

4. **Changelog generation** -- depends on release workflow. git-cliff can be set up locally first, then integrated into release flow. `.github/release.yml` for GitHub auto-notes is trivial.

5. **Version bump scripting** -- optional tooling to keep version numbers in sync across `package.json`, `Cargo.toml`, and `tauri.conf.json`. Can be a simple shell script.

## Sources

- [Tauri v2 Official GitHub Pipeline Guide](https://v2.tauri.app/distribute/pipelines/github/) -- HIGH confidence
- [tauri-apps/tauri-action Repository](https://github.com/tauri-apps/tauri-action) -- HIGH confidence
- [tauri-action Issue #986: bun not found](https://github.com/tauri-apps/tauri-action/issues/986) -- HIGH confidence (documents critical bun gotcha)
- [oven-sh/setup-bun Action](https://github.com/oven-sh/setup-bun) -- HIGH confidence
- [Bun CI/CD Guide](https://bun.com/docs/guides/runtime/cicd) -- HIGH confidence
- [Dependabot Supported Ecosystems](https://docs.github.com/en/code-security/dependabot/ecosystems-supported-by-dependabot/supported-ecosystems-and-repositories) -- HIGH confidence
- [Dependabot bun support GA announcement](https://github.blog/changelog/2025-02-13-dependabot-version-updates-now-support-the-bun-package-manager-ga/) -- HIGH confidence
- [GitHub Auto-Generated Release Notes](https://docs.github.com/en/repositories/releasing-projects-on-github/automatically-generated-release-notes) -- HIGH confidence
- [git-cliff Repository](https://github.com/orhun/git-cliff) -- HIGH confidence
- [git-cliff GitHub Integration](https://git-cliff.org/docs/integration/github/) -- HIGH confidence
- [Ship Your Tauri v2 App Like a Pro (Part 2)](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) -- MEDIUM confidence (community tutorial, verified against official docs)
- [Tauri v2 macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) -- HIGH confidence (reference for future milestone)
- [Tauri v2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) -- HIGH confidence (reference for future milestone)

---
*Architecture research for: CI/CD Pipeline & Cross-Platform Release Publishing for Tauri 2 desktop app*
*Researched: 2026-03-25*
