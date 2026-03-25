# Feature Research

**Domain:** CI/CD pipeline and cross-platform release publishing for a Tauri 2 desktop app
**Researched:** 2026-03-25
**Confidence:** HIGH

## Feature Landscape

### Table Stakes (Users Expect These)

Features that any serious open-source Tauri 2 project must have for CI/CD and release publishing. Missing these means the project looks unfinished or untrustworthy to contributors and users downloading releases.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| CI lint/check on every push and PR | Prevents regressions; standard for any Rust+TS project. Contributors expect immediate feedback on PRs. | LOW | Separate jobs for fast feedback: `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --all -- --check`, `cargo test`, `bun run check` (svelte-check), `bun run test` (vitest), prettier `--check`. Run on ubuntu-latest only (cheapest). |
| Cross-platform release builds (macOS, Linux, Windows) | A desktop app that only ships one platform is not a real release. Users expect native installers for their OS. | MEDIUM | `tauri-apps/tauri-action@v0` handles build + bundle. Matrix: macOS (universal binary via `--target universal-apple-darwin`), Ubuntu 22.04 x64, Windows x64. Outputs: .dmg, .AppImage, .msi. |
| Tag-triggered release workflow | Standard release mechanism for open-source projects. Push `v*` tag, get binaries. Predictable and auditable. | LOW | Trigger on `push: tags: ['v*']`. tauri-action creates GitHub Release and uploads platform artifacts. The `__VERSION__` placeholder in tagName/releaseName is auto-replaced with version from tauri.conf.json. |
| GitHub Release with downloadable artifacts | Users expect a Releases page with clearly labeled platform-specific downloads they can grab. | LOW | Built into tauri-action. Each matrix job uploads its artifacts to the same GitHub Release. Release body includes changelog. |
| Dependabot for automated dependency updates | Standard GitHub practice. Keeps Cargo crates, npm packages, and GitHub Actions versions current. Catches security vulnerabilities. | LOW | Single `.github/dependabot.yml` with three ecosystem entries: `cargo` (directory: `/src-tauri`), `npm` (directory: `/`), `github-actions` (directory: `/`). Weekly schedule for all three. |
| Rust build caching in CI | Without caching, Rust compilation takes 10-15 minutes. With `swatinem/rust-cache@v2`, subsequent runs take 2-4 minutes. Uncached CI is a dealbreaker for contributor DX. | LOW | Config: `workspaces: './src-tauri -> target'`. Also cache bun dependencies via `actions/cache@v4` with `~/.bun/install/cache` keyed on `bun.lock` hash. |
| Auto-generated changelog in release notes | Users and contributors expect release notes describing what changed. Manual notes are error-prone and tedious. | LOW | `git-cliff` (Rust-native, ~120ms for large repos) with conventional commit parsing. Generates markdown changelog between tags. Output injected into GitHub Release body. |

### Differentiators (Competitive Advantage)

Features that go beyond minimum expectations and signal a well-run, thoughtful CI/CD setup.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Separate fast CI (lint+test) from slow release build | Lint failures caught in ~1-2 min instead of waiting for 8-15 min full build. Better contributor DX. Most Tauri projects combine everything into one slow workflow. | LOW | Two workflow files: `ci.yml` (lint/test on push+PR, runs on ubuntu-latest only, ~2 min) and `release.yml` (build+publish on tag push, matrix of 3 platforms, ~8-15 min). |
| Universal macOS binary (.dmg) | Single download works on both Intel and Apple Silicon instead of forcing users to pick the right architecture. Cleaner release page. | LOW | Use `--target universal-apple-darwin` with both Rust targets (`aarch64-apple-darwin,x86_64-apple-darwin`) installed. One macOS matrix entry instead of two. |
| Release as draft for review | Catch mistakes (wrong version, missing artifacts, bad changelog) before users see a broken release. Maintainer reviews, then publishes. | LOW | `releaseDraft: true` in tauri-action config. Workflow builds + uploads everything, but release stays draft until manually published. |
| Cargo test running in CI | 14 Rust source files already have `#[cfg(test)]` modules with real unit tests. Running them catches backend regressions that vitest cannot. | LOW | Already written. Just add `cargo test --manifest-path src-tauri/Cargo.toml` to CI workflow. Free quality gate. |
| Prettier formatting enforcement | Consistent frontend code style across all contributions. No style debates in PRs. | LOW | Add `prettier` as devDependency, create `.prettierrc`, run `bunx prettier --write .` once to format existing codebase, then `bunx prettier --check .` in CI. |
| Fail-fast disabled in release matrix | If Windows build fails, macOS and Linux still complete and upload. Partial release is better than no release for the platforms that did succeed. | LOW | `fail-fast: false` in matrix strategy. Default is true (one failure cancels all). |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems. Explicitly choosing NOT to build these in v0.10.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Auto-updater (tauri-plugin-updater) | "Apps should update themselves" | Requires code signing keys (prerequisite), update signature keypair, latest.json hosting, and significantly increases release complexity. Adds plugin dependency + Rust + frontend code. PROJECT.md explicitly defers to v1.0. | Ship clean GitHub Releases page. Users download new versions manually. Add auto-updater after code signing is established. |
| macOS code signing + notarization | "Unsigned apps show scary Gatekeeper warnings" | Requires Apple Developer Program ($99/year), certificate management as CI secrets, P12 export as base64, adds 2-5 min per build for Apple notarization round-trip. Overkill for personal/early open-source project. | Unsigned builds with clear install instructions: right-click > Open to bypass Gatekeeper once. Document in README. Add signing when user base warrants the cost. |
| Windows code signing (EV certificate) | "SmartScreen blocks unsigned installers" | Since June 2023, OV certs require HSM storage. Most accessible option is Azure Key Vault + relic signing tool. $200-400/year for cert. Complex CI setup. | Unsigned builds. SmartScreen warning disappears after enough user downloads build reputation. Document the "More info > Run anyway" click in README. |
| Nightly / dev channel builds | "I want bleeding edge builds from main" | Doubles CI compute costs. Requires branch management and naming conventions. Confuses users about stability. No user base yet to benefit. | Release on tags only. Contributors can build from source via `bun run build`. |
| Multi-architecture Linux builds (arm64) | "Raspberry Pi / ARM server support" | `ubuntu-22.04-arm` runners are only free for public repos. Adds another matrix entry and test surface. Tiny user base for a Git GUI on ARM Linux. | Ship x64 AppImage only. Add arm64 if demand emerges after open-source release. |
| Homebrew tap / AUR package / Winget manifest | "I want to install via my package manager" | Each package manager has its own submission process, review cycle, update automation, and maintenance burden. Homebrew needs a tap repo; AUR needs PKGBUILD maintenance; Winget needs manifest PRs. | Defer to post v1.0. GitHub Releases with direct downloads is sufficient for initial distribution. |
| release-please or semantic-release | "Automate version bumps and changelogs via bot PRs" | Adds significant complexity: bot-created PRs, version file sync across three files (package.json, Cargo.toml, tauri.conf.json), conventional commit enforcement as hard requirement, merge strategy constraints. | Manual version bump across the three files + `git-cliff` for changelog. Simple, predictable, human-controlled. Tag push triggers release. |
| E2E testing in CI | "Test the actual app end-to-end" | Tauri E2E requires WebDriver (tauri-driver) + platform-specific WebKitGTK/WebView2 setup. Notoriously flaky in CI. PROJECT.md explicitly plans E2E test harness for v1.0 as a dedicated infrastructure milestone. | Unit tests (vitest + cargo test) catch most regressions. 14 TS test files + 14 Rust test modules provide good coverage. E2E is a v1.0 concern. |
| Portable .tar.gz alongside installers | "Power users want unzip-and-run" | tauri-action's `uploadPlainBinary` is marked "ONLY ENABLE THIS IF YOU KNOW WHAT YOU'RE DOING" -- Tauri does NOT officially support portable mode. May cause issues with runtime dependencies. | Stick with official bundle formats (.dmg, .AppImage, .msi). AppImage on Linux already functions as "download and run" without install. |

## Feature Dependencies

```
[Prettier config + devDependency]
    └──required by──> [Prettier check in CI workflow]

[git-cliff config (cliff.toml)]
    └──required by──> [Changelog generation in release workflow]

[CI lint/check workflow (ci.yml)]
    (no dependencies -- standalone, can be built first)

[Dependabot config (.github/dependabot.yml)]
    (no dependencies -- standalone YAML file)

[Tag-triggered release workflow (release.yml)]
    └──requires──> [git-cliff config] (release body includes changelog)
    └──benefits from──> [CI workflow] (should pass checks before releasing)
    └──uses──> [tauri-action] (builds all platforms + uploads artifacts)
    └──uses──> [oven-sh/setup-bun] (bun must be installed before tauri-action runs)
    └──produces──> [GitHub Release with platform artifacts]
```

### Dependency Notes

- **Release workflow benefits from CI workflow existing first:** While not a hard technical dependency (release workflow can include its own checks), having CI running on PRs means the code on main/tags has already been validated. Build the CI workflow first.
- **Changelog generation requires cliff.toml:** A `cliff.toml` configuration file in the repo root configures how git-cliff parses commit messages into changelog sections. One-time setup.
- **Prettier CI check requires setup first:** Need to (a) add `prettier` to devDependencies, (b) create `.prettierrc` config, (c) run initial `bunx prettier --write .` to format existing codebase, (d) commit formatted code. Only then can CI enforce `--check` without failing on pre-existing formatting inconsistencies.
- **tauri-action requires bun to be installed:** Known issue (tauri-apps/tauri-action#986): tauri-action cannot find bun if `oven-sh/setup-bun@v2` has not run first. The `beforeBuildCommand` in tauri.conf.json runs `bun run build`, which fails without bun on PATH.
- **Version sync is manual but critical:** Three files contain the version: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`. Before pushing a release tag, all three must match. This is a process discipline, not a technical dependency.

## MVP Definition

### Launch With (v0.10)

The minimum viable CI/CD setup -- automated quality gates and publishable cross-platform releases.

- [ ] **CI workflow (`ci.yml`)** -- cargo check, clippy, fmt, cargo test, bun install, bun run check, bun run test, prettier check. Triggered on push to main and PRs. Runs on ubuntu-latest. ~2 min.
- [ ] **Release workflow (`release.yml`)** -- tag-triggered cross-platform builds. macOS universal .dmg, Linux x64 .AppImage, Windows x64 .msi. Draft GitHub Release with auto-generated changelog. ~8-15 min.
- [ ] **Dependabot config (`dependabot.yml`)** -- weekly automated PRs for cargo, npm, and github-actions dependency updates.
- [ ] **git-cliff config (`cliff.toml`)** -- conventional commit parsing, changelog generation between tags, injected into release body.
- [ ] **Prettier setup** -- `.prettierrc` config, `prettier` devDependency, initial format pass on codebase, CI enforcement.

### Add After Validation (v1.0)

Features to add once the pipeline is running and the project has real users downloading releases.

- [ ] **macOS code signing + notarization** -- requires Apple Developer Program enrollment and CI secrets setup
- [ ] **Windows code signing** -- requires EV cert + Azure Key Vault + relic signing tool
- [ ] **Auto-updater (tauri-plugin-updater)** -- requires code signing to be in place first (signed updates are mandatory)
- [ ] **E2E test harness in CI** -- already planned for v1.0 in PROJECT.md

### Future Consideration (v2+)

Features to defer until the project has an established user base and contributor community.

- [ ] **Homebrew tap / AUR / Winget manifest** -- when users request package manager distribution
- [ ] **Nightly builds from main** -- when contributor activity warrants bleeding-edge testing
- [ ] **Linux arm64 builds** -- when ARM desktop Linux demand is demonstrated
- [ ] **release-please automation** -- when release frequency makes manual version bumps painful

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| CI lint/check workflow | HIGH | LOW | P1 |
| Tag-triggered release builds | HIGH | MEDIUM | P1 |
| GitHub Release with artifacts | HIGH | LOW (built into tauri-action) | P1 |
| Dependabot config | MEDIUM | LOW | P1 |
| git-cliff changelog | MEDIUM | LOW | P1 |
| Prettier setup + CI check | MEDIUM | LOW | P1 |
| Universal macOS binary | MEDIUM | LOW | P2 |
| Release draft mode | MEDIUM | LOW | P2 |
| fail-fast: false in release matrix | LOW | LOW | P2 |
| Code signing (macOS + Windows) | HIGH | HIGH | P3 (defer to v1.0) |
| Auto-updater | HIGH | HIGH | P3 (defer to v1.0) |
| Package manager distribution | MEDIUM | HIGH | P3 (defer to v2+) |

**Priority key:**
- P1: Must have for v0.10 launch
- P2: Should have, fold into P1 work since the cost is trivial
- P3: Explicitly deferred to future milestones

## Competitor Feature Analysis

| Feature | GitKraken | Fork | Sublime Merge | Our Approach (Trunk v0.10) |
|---------|-----------|------|---------------|---------------------------|
| Release cadence | Frequent, auto-update | Regular, manual download | Stable + Dev channels | Tag-triggered GitHub Releases |
| Platform coverage | macOS, Linux, Windows | macOS, Windows only | macOS, Linux, Windows | macOS (universal), Linux (AppImage), Windows (.msi) |
| Code signing | Yes (commercial product) | Yes (commercial) | Yes (commercial) | Deferred -- unsigned for v0.10 |
| Auto-update | Yes (built-in) | Yes (built-in) | Yes (built-in) | Deferred to v1.0 |
| Changelog | Blog/release notes (manual) | Blog (manual) | Changelog page (manual) | Auto-generated via git-cliff |
| Package managers | Homebrew, Snap, Flatpak, apt | Homebrew (macOS only) | apt, pacman, Homebrew, others | Direct download only for v0.10 |
| CI pipeline | Private (commercial) | Private (commercial) | Private (commercial) | Public GitHub Actions (open-source advantage) |

**Note:** All competitors are mature commercial products with paid teams. Trunk is a personal open-source project. The comparison shows where to invest later (signing, auto-update) vs. what to skip now (package managers, dev channels). The open-source advantage is that CI/CD is public and transparent.

## Existing Project State

Key facts about the current codebase that affect feature implementation.

| Aspect | Current State | Implication for CI/CD |
|--------|---------------|----------------------|
| Package manager | bun with `bun.lock` (text format) | Use `oven-sh/setup-bun@v2` in CI. Cache `~/.bun/install/cache` keyed on `bun.lock` hash. tauri-action needs bun installed first. |
| Frontend tests | 14 vitest test files in `src/lib/` | `bun run test` already works. Just add to CI. |
| Backend tests | 14 Rust files with `#[cfg(test)]` modules | `cargo test --manifest-path src-tauri/Cargo.toml` works. Just add to CI. |
| Prettier | Not installed, no config exists | Need to: add devDependency, create `.prettierrc`, run initial format pass, commit changes. |
| Icons | Full set exists (`src-tauri/icons/`) -- .icns, .ico, .png at all required sizes | No icon work needed for bundling. tauri.conf.json already references them. |
| Bundle config | `"targets": "all"` in tauri.conf.json | Builds all platform-appropriate formats automatically. No target restriction needed. |
| Version files | `0.1.0` in package.json, Cargo.toml, tauri.conf.json | Three files to keep in sync on release. Manual for now. |
| .github directory | Does not exist | Create from scratch: `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `.github/dependabot.yml`. |
| Cargo.lock | Exists at `src-tauri/Cargo.lock` (committed) | Dependabot will create PRs updating it. `swatinem/rust-cache` uses it for cache key. |
| git2 vendored | `features = ["vendored-libgit2"]` in Cargo.toml | CI does not need system libgit2 installed. Simplifies Linux dependency setup (only need webkit2gtk, appindicator, librsvg, patchelf). |
| Tauri plugins | dialog, store, window-state, clipboard-manager | All bundled via Cargo. No extra CI setup needed. |
| Rust edition | 2021 | Compatible with Dependabot's current Rust support. Rust 2024 edition has known Dependabot issues (#11691). |

## Sources

- [Tauri 2 GitHub Actions Pipeline Documentation](https://v2.tauri.app/distribute/pipelines/github/) -- official CI/CD guide with full workflow YAML examples (HIGH confidence)
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) -- official GitHub Action, 25+ config inputs, release creation, artifact upload (HIGH confidence)
- [tauri-apps/tauri-action#986](https://github.com/tauri-apps/tauri-action/issues/986) -- known "bun: not found" issue, must install bun before tauri-action (HIGH confidence)
- [Tauri 2 Configuration Reference](https://v2.tauri.app/reference/config/) -- bundle targets: deb, rpm, appimage, nsis, msi, app, dmg, or "all" (HIGH confidence)
- [oven-sh/setup-bun](https://github.com/oven-sh/setup-bun) -- official bun setup action v2 for GitHub Actions (HIGH confidence)
- [swatinem/rust-cache](https://github.com/Swatinem/rust-cache) -- Rust/Cargo caching, `workspaces` config for Tauri projects (HIGH confidence)
- [git-cliff](https://git-cliff.org/) -- Rust-native changelog generator, conventional commits, ~120ms generation (HIGH confidence)
- [Dependabot Supported Ecosystems](https://docs.github.com/en/code-security/dependabot/ecosystems-supported-by-dependabot/supported-ecosystems-and-repositories) -- cargo, npm, github-actions all supported (HIGH confidence)
- [Dependabot Rust Toolchain Updates](https://github.blog/changelog/2025-08-19-dependabot-now-supports-rust-toolchain-updates/) -- also supports rust-toolchain.toml updates (HIGH confidence)
- [Clippy CI Documentation](https://doc.rust-lang.org/nightly/clippy/continuous_integration/github_actions.html) -- official `cargo clippy -- -D warnings` CI pattern (HIGH confidence)
- [Prettier CI Documentation](https://prettier.io/docs/ci) -- official `--check` flag for CI enforcement (HIGH confidence)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) -- standard Rust toolchain setup action, supports target specification (HIGH confidence)
- [Tauri 2 macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) -- Apple Developer Program, P12 cert, notarization requirements (MEDIUM confidence -- read but deferring)
- [Tauri 2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) -- HSM requirement since 2023, Azure Key Vault + relic approach (MEDIUM confidence -- read but deferring)
- [Ship Your Tauri v2 App Like a Pro (Part 2/2)](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) -- real-world Tauri v2 release pipeline walkthrough (MEDIUM confidence)

---
*Feature research for: CI/CD pipeline and cross-platform release publishing (Tauri 2 desktop app)*
*Researched: 2026-03-25*
