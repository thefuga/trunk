# Project Research Summary

**Project:** Trunk v0.10 — CI/CD & Cross-Platform Release Publishing
**Domain:** Desktop app CI/CD pipeline (Tauri 2 + Svelte 5 + Rust)
**Researched:** 2026-03-25
**Confidence:** HIGH

## Executive Summary

Trunk v0.10 is a CI/CD and release infrastructure milestone for an existing, feature-complete Tauri 2 desktop Git GUI. The project already has 49 completed phases of product work (~13,400 LOC TypeScript/Svelte + ~9,400 LOC Rust), a full test suite (14 vitest files + 14 Rust `#[cfg(test)]` modules), and a complete icon set — the missing piece is automated quality gates and distributable release artifacts for all three major platforms. Research consensus is clear: this is well-trodden territory with official Tauri 2 documentation, a dedicated GitHub Action (`tauri-apps/tauri-action`), and a known set of pitfalls from community post-mortems.

The recommended approach is a two-workflow design: a fast CI workflow (lint, type-check, test on a single Ubuntu runner, ~2-5 min) that runs on every push and PR; and a release workflow (cross-platform build matrix, ~15-25 min) that triggers only on `v*` tag pushes. The release workflow produces a draft GitHub Release with auto-generated changelog and platform-specific installers (.dmg for macOS ARM + Intel, .AppImage + .deb for Linux, .msi + .exe for Windows). Changelog generation uses git-cliff (Rust-native, 120ms for large repos) fed by the project's existing conventional commit style. Dependabot covers all three dependency ecosystems (cargo, npm, github-actions) with a trivial config file.

The principal risks are operational, not architectural. Three pitfalls account for the majority of real-world CI/CD failures on Tauri projects: version drift across the three version files (`package.json`, `Cargo.toml`, `tauri.conf.json`); a race condition in `tauri-action` when parallel matrix jobs all try to create the release simultaneously; and bun not being pre-installed on GitHub runners. All three have known, documented fixes and must be addressed in the initial workflow setup. Code signing (macOS Gatekeeper, Windows SmartScreen) is explicitly deferred to a future milestone — unsigned builds with documented bypass instructions are the correct v0.10 approach.

## Key Findings

### Recommended Stack

All CI/CD tooling is GitHub Actions-based with zero new runtime dependencies. The stack was selected by cross-referencing official Tauri 2 distribution docs against the project's existing lock files, config files, and commit history.

**Core technologies:**
- `tauri-apps/tauri-action@v0` (v0.6.2): Cross-platform build, bundle, and GitHub Release upload in one step — the official Tauri CI action; v1 exists on a dev branch but is not stable-released
- `oven-sh/setup-bun@v2` (v2.2.0): Installs Bun on GitHub runners — required because runners do not pre-install Bun; tauri-action detects `bun.lock` and expects Bun on PATH
- `dtolnay/rust-toolchain@stable`: Rust toolchain setup — replaces the archived `actions-rs/toolchain`; must come before `Swatinem/rust-cache`
- `Swatinem/rust-cache@v2`: Cargo build caching — understands Cargo.lock and target directory layout; saves 2-5 min per build (critical given `vendored-libgit2` C compilation)
- `orhun/git-cliff-action@v4` (git-cliff v2.12.0): Changelog from conventional commits — Rust-native, 120ms for large repos; requires `fetch-depth: 0` checkout
- GitHub Dependabot (v2 config): Automated dependency PRs for cargo, npm, github-actions — built into GitHub, zero maintenance overhead
- `prettier` + `prettier-plugin-svelte`: Frontend formatting — new devDependency; requires initial format pass before CI enforcement
- `ubuntu-22.04` runner for Linux builds: Pinned (not `ubuntu-latest`) to control AppImage glibc floor at 2.35 for maximum distribution compatibility

### Expected Features

**Must have (table stakes — v0.10):**
- CI lint/check workflow on every push and PR — prevents regressions; contributors expect immediate feedback
- Cross-platform release builds (macOS ARM + Intel, Linux x64, Windows x64) — a desktop app that ships one platform is not a real release
- Tag-triggered release workflow (`v*` tags) — standard open-source release mechanism; predictable and auditable
- GitHub Release with downloadable platform-specific artifacts — users expect a Releases page
- Dependabot for automated dependency updates — standard GitHub practice for security and currency
- Rust build caching — without caching CI takes 30-60 min; with `swatinem/rust-cache`, 5-15 min
- Auto-generated changelog in release notes — users and contributors expect to know what changed

**Should have (low-cost differentiators, fold into v0.10 work):**
- Separate fast CI from slow release build — better contributor DX; most Tauri projects combine these into one slow workflow
- Separate macOS ARM and Intel .dmg files — simpler than universal binary in CI; users download the correct one for their Mac
- Draft release mode — catch broken builds or wrong versions before users download them
- Cargo tests running in CI — 14 Rust test modules already exist; free quality gate
- Prettier formatting enforcement — consistent frontend code style, no style debates in PRs
- `fail-fast: false` on build matrix — if Windows fails, macOS and Linux still complete

**Defer (v1.0):**
- macOS code signing + notarization — requires Apple Developer Program ($99/year) and per-build notarization round-trip
- Windows code signing (EV cert) — requires Azure Key Vault + HSM; $200-400/year
- Auto-updater (`tauri-plugin-updater`) — requires code signing to be in place first; PROJECT.md explicitly defers to v1.0
- E2E test harness — already planned for v1.0 as a dedicated infrastructure milestone

**Defer (v2+):**
- Homebrew tap / AUR / Winget manifest — each has its own submission process and maintenance burden
- Nightly builds from main — doubles compute costs; no user base yet to benefit
- Linux arm64 builds — tiny user base for a Git GUI on ARM Linux

### Architecture Approach

The system is two parallel, independent workflows sharing no state except the GitHub Release object. The CI workflow (`ci.yml`) runs on a single `ubuntu-latest` runner — Rust and TypeScript checks are platform-independent for this codebase. The release workflow (`release.yml`) runs a matrix of 4 jobs in parallel across 3 runners (macOS arm64, macOS x86_64, Ubuntu 22.04, Windows). A critical structural decision: the release workflow must have a dedicated "create release" job that completes before the build matrix starts, because tauri-action's parallel matrix jobs can corrupt the release object if they race to create it simultaneously (known bug tauri-apps/tauri-action#914).

**Major components:**
1. `.github/workflows/ci.yml` — lint + test on every push/PR; single ubuntu-latest runner; ~2-5 min; reuses bun + Rust cache patterns that release workflow also uses
2. `.github/workflows/release.yml` — tag-triggered; two-phase: dedicated "create release" job first, then parallel 4-job build matrix uploading artifacts to the pre-created release
3. `.github/dependabot.yml` — three ecosystems (cargo at `/src-tauri`, npm at `/`, github-actions at `/`); weekly schedule
4. `cliff.toml` — git-cliff changelog config; maps project's conventional commit prefixes to changelog groups; skips `docs:` commits (GSD planning artifacts dominate history)
5. `.prettierrc` + `prettier` devDependency — one-time setup: add config, run initial format pass, then CI enforces `--check`
6. `scripts/bump-version.sh` (recommended) — atomic version bump across `package.json`, `Cargo.toml`, and `cargo generate-lockfile`

### Critical Pitfalls

1. **Version drift across three manifest files** — `package.json`, `Cargo.toml`, and `tauri.conf.json` all carry the version; tauri-action reads from `tauri.conf.json`; mismatch causes releases to upload to the wrong release or fail entirely. Fix: remove `version` from `tauri.conf.json` (Tauri 2 falls back to `Cargo.toml`), add a CI version alignment check, and create a version bump script.

2. **tauri-action duplicate release race condition** — parallel matrix jobs each call the GitHub Release API to find or create the release; concurrent calls can strip the tag association from a draft release, creating duplicate releases with split artifacts (bug tauri-apps/tauri-action#914). Fix: create the release in a dedicated job before the matrix starts; matrix jobs only upload artifacts using the pre-created `release_id`.

3. **Bun not found on GitHub runners** — GitHub runners do not pre-install Bun; tauri-action detects `bun.lock` and assumes Bun is on PATH; fails with `bun: not found`. Fix: always add `oven-sh/setup-bun@v2` as an early step before tauri-action.

4. **AppImage glibc compatibility** — AppImage does not bundle glibc; building on Ubuntu 24.04 produces an AppImage that fails on Ubuntu 22.04 with `GLIBC_X.XX not found`. Fix: pin the Linux runner to `ubuntu-22.04` explicitly, never `ubuntu-latest`.

5. **GitHub Token default read-only permissions** — default `GITHUB_TOKEN` cannot create releases; fails with "Resource not accessible by integration." Fix: add `permissions: contents: write` to the release workflow.

6. **Shallow clone breaks changelog** — default `fetch-depth: 1` gives git-cliff only the HEAD commit; generates empty release notes. Fix: use `fetch-depth: 0` on the checkout step in the release job only.

7. **Uncached Rust builds (30-60 min per platform)** — `vendored-libgit2` adds C compilation on top of Rust; without caching a 4-platform release burns 2-4 hours. Fix: `Swatinem/rust-cache@v2` with `workspaces: './src-tauri -> target'` must be configured from the first workflow run.

## Implications for Roadmap

Based on the dependency graph from FEATURES.md and the build order from ARCHITECTURE.md, four phases cover v0.10 entirely. All pitfall mitigations are woven into the phase where they first appear, not addressed retroactively.

### Phase 1: CI Foundation

**Rationale:** CI must exist before the release pipeline. It validates that existing code is clean, provides immediate value on every PR, and its patterns (bun setup, Rust cache, step ordering) are reused verbatim in the release workflow. Build this first so Dependabot PRs are auto-validated. Caching must be configured here — retrofitting after habits form wastes accumulated runner time.

**Delivers:** `ci.yml` — cargo check, cargo clippy (-D warnings), cargo test, cargo fmt --check, bun install, bun run check (svelte-check), bun run test (vitest), prettier --check. Triggered on push to any branch and PR to main. Single ubuntu-latest runner. Also: `prettier` + `prettier-plugin-svelte` devDependencies, `.prettierrc`, `.prettierignore`, initial `bunx prettier --write .` pass, CI version alignment check (package.json vs Cargo.toml must match).

**Addresses:** CI lint/check workflow (table stakes), Prettier enforcement (differentiator), Cargo tests (differentiator)

**Avoids:** Pitfall 3 (bun not found — setup-bun step required), Pitfall 5 (uncached builds — rust-cache from day one), Pitfall 7 (uncached builds), Pitfall 1 (version drift — CI check catches it before it can break a release)

**Research flag:** No additional research needed — standard patterns with official documentation.

### Phase 2: Release Pipeline

**Rationale:** Builds directly on the bun + Rust cache patterns validated in Phase 1. The two-job structure (create release first, then build matrix) is the only correct approach given the known race condition in tauri-action. All pitfalls in this phase have specific, documented fixes.

**Delivers:** `release.yml` — tag-triggered (`v*`); Job 1 creates draft GitHub Release; Job 2 runs 4-job parallel matrix (macOS ARM, macOS Intel, ubuntu-22.04, Windows) using tauri-action to build and upload artifacts; `fail-fast: false`; `permissions: contents: write`; Linux system dependency installation step.

**Addresses:** Cross-platform release builds (table stakes), tag-triggered workflow, GitHub Release with artifacts, separate macOS ARM + Intel .dmg files (differentiator), draft release mode (differentiator), `fail-fast: false` (differentiator)

**Avoids:** Pitfall 2 (duplicate release race — two-job structure eliminates the race), Pitfall 4 (Linux deps — ubuntu-22.04 with explicit apt-get), Pitfall 6 (AppImage glibc — ubuntu-22.04 pinned), Pitfall 7 (unsigned build UX — draft mode + release notes template), Pitfall 8 (token permissions — explicit `permissions: contents: write`)

**Research flag:** No additional research needed — official Tauri docs cover this exactly. Known bugs have documented workarounds with specific issue numbers.

### Phase 3: Changelog & Dependabot

**Rationale:** Changelog requires a working release pipeline. Dependabot is a standalone YAML file but benefits from being added after CI exists so its PRs are auto-validated. Both are low-complexity, high-value additions that complete the "table stakes" feature list.

**Delivers:** `cliff.toml` — conventional commit groups mapping to changelog sections; skips `docs:` (GSD planning history), `chore:` (internal); changelog injected into release workflow with `fetch-depth: 0`. `.github/dependabot.yml` — cargo (weekly), npm (weekly), github-actions (weekly). `bun install --frozen-lockfile` CI check to catch Dependabot lockfile corruption.

**Addresses:** Auto-generated changelog (table stakes), Dependabot coverage (table stakes)

**Avoids:** Pitfall 9 (shallow clone empty changelog — `fetch-depth: 0` on release checkout), Pitfall 10 (Dependabot bun lockfile corruption — frozen-lockfile CI check)

**Research flag:** No additional research needed — git-cliff and Dependabot are well-documented with clear configuration examples.

### Phase 4: Version Management & First Release

**Rationale:** Version sync is the most common Tauri CI/CD failure — structurally fixing it (remove version from `tauri.conf.json`, add bump script) prevents an entire class of release failures. This phase also produces the first actual release of Trunk v0.10.0, including release documentation so users succeed with unsigned builds.

**Delivers:** Remove `version` from `tauri.conf.json` (inherits from `Cargo.toml` in Tauri 2). `scripts/bump-version.sh` — atomic bump across `package.json` + `Cargo.toml` + `cargo generate-lockfile` + git tag. GitHub Release description template with macOS ("right-click > Open") and Windows ("More info > Run anyway") bypass instructions. README installation section. First `v0.10.0` release tag pushed and draft reviewed.

**Addresses:** Version management (structural fix), unsigned build UX, first actual release

**Avoids:** Pitfall 1 (version drift — structural fix, not just process discipline)

**Research flag:** No additional research needed — well-understood problem with documented solutions.

### Phase Ordering Rationale

- Phase 1 before Phase 2: CI patterns (bun setup, Rust cache) are copy-pasted into the release workflow; a passing CI run on main before the first release tag ensures the code is clean
- Phase 2 before Phase 3: Changelog integration requires the release workflow to exist; Dependabot PRs must flow through CI, which must exist first
- Phase 4 last: Version file cleanup and bump script are technically independent but the actual first release tag should only be pushed after all three prior phases are validated end-to-end
- All pitfall mitigations land in the phase where they first appear — no "cleanup" phase needed

### Research Flags

Phases with standard patterns (skip `/gsd:research-phase`):
- **All four phases:** The entire v0.10 scope is covered by official Tauri 2 distribution documentation, the tauri-action repository, and documented community post-mortems. All four research files rated HIGH confidence. No niche integrations, no sparse documentation areas, no experimental APIs. The pitfalls researcher identified root causes to specific line numbers in tauri-action source code — this level of specificity means planning can proceed directly to implementation.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All tools sourced from official documentation and verified release tags. No community-only sources for any recommended tool. Explicit "do not use" list backed by specific deprecation/bug records. |
| Features | HIGH | Table stakes derived from official Tauri 2 docs and analysis of comparable open-source desktop apps. Defer list backed by explicit PROJECT.md decisions and cost/complexity analysis. |
| Architecture | HIGH | Two-workflow pattern is the canonical Tauri 2 CI/CD design from official docs. Race condition fix verified against tauri-action issue tracker with root cause analysis to specific source lines. |
| Pitfalls | HIGH | Each pitfall references official docs, specific GitHub issue numbers, or post-mortems with quantified failure counts (e.g., "60+ failed runs" for version sync). Recovery costs documented. |

**Overall confidence:** HIGH

### Gaps to Address

- **Prettier initial format pass scope:** Research calls for running `bunx prettier --write .` on the codebase before enabling CI enforcement. The actual diff size and whether any files need manual attention is unknown until the pass runs. Low risk — Prettier is deterministic — but it may touch many files that should be reviewed before committing.

- **`tauri.conf.json` version field removal:** Tauri 2 documentation says it falls back to `Cargo.toml` version when `tauri.conf.json` omits the version field. This should be verified against the actual Tauri version in use before removing the field.

- **cliff.toml `docs:` skip filter:** Research recommends skipping `docs:` commits because the project's git history is heavily GSD-planning-laden. The ratio of product `feat:`/`fix:` commits to planning `docs:` commits should be spot-checked before writing the `cliff.toml` to confirm this exclusion produces a useful changelog.

- **macOS runner compute cost:** Research recommends two separate matrix entries for macOS ARM and Intel. Both run on `macos-latest` (Apple Silicon), so each build takes ~8 min. If free-tier build minutes become a concern after the first release, the Intel .dmg can be dropped — but this is not a blocking concern for v0.10.

## Sources

### Primary (HIGH confidence)

- [Tauri 2 GitHub Actions Guide](https://v2.tauri.app/distribute/pipelines/github/) — official workflow examples, Linux dependencies, matrix strategy
- [tauri-apps/tauri-action repository](https://github.com/tauri-apps/tauri-action) — input/output spec, v0.6.2 stable, bun detection, release creation
- [tauri-action#914: Duplicate Release Race](https://github.com/tauri-apps/tauri-action/issues/914) — root cause analysis, two-job workaround
- [tauri-action#986: bun not found](https://github.com/tauri-apps/tauri-action/issues/986) — setup-bun requirement
- [oven-sh/setup-bun](https://github.com/oven-sh/setup-bun) — v2.2.0, bun CI setup
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) — stable, replaces archived actions-rs
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) — v2.x, workspace config for Tauri
- [orhun/git-cliff-action](https://github.com/orhun/git-cliff-action) — v4.7.1, fetch-depth requirement
- [git-cliff repository](https://github.com/orhun/git-cliff) — v2.12.0, cliff.toml config format
- [GitHub Dependabot configuration docs](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuring-dependabot-version-updates) — v2 format, ecosystem options
- [Dependabot Bun Support GA (Feb 2025)](https://github.blog/changelog/2025-02-13-dependabot-version-updates-now-support-the-bun-package-manager-ga/) — bun.lock support
- [Tauri 2 AppImage Distribution Docs](https://v2.tauri.app/distribute/appimage/) — glibc compatibility warning
- [Tauri 2 Windows Installer Docs](https://v2.tauri.app/distribute/windows-installer/) — NSIS vs MSI trade-offs
- [Clippy CI docs](https://doc.rust-lang.org/nightly/clippy/continuous_integration/github_actions.html) — `-D warnings` pattern
- [Prettier CI docs](https://prettier.io/docs/ci) — `--check` flag
- [Tauri 2 macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) — future milestone reference
- [Tauri 2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) — future milestone reference

### Secondary (MEDIUM confidence)

- [Ship Tauri v2 Like a Pro Part 2](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) — real-world draft release pattern, concurrency control, fail-fast (verified against official docs)
- [How to Make Rust CI 2-3x Faster](https://www.reillywood.com/blog/rust-faster-ci/) — clippy ordering, cache strategy
- [VaultNote CI/CD Post-Mortem](https://dev.to/dev_michael/my-first-tauri-cicd-pipeline-lessons-from-building-vaultnote-with-sveltekit-17mp) — version sync failures, 60+ failed runs

### Tertiary (LOW confidence / known open issues)

- [dependabot-core#13623: Bun lockfile configVersion removal](https://github.com/dependabot/dependabot-core/issues/13623) — open bug; mitigated by `--frozen-lockfile` CI check
- [dependabot-core#11691: Rust 2024 edition support](https://github.com/dependabot/dependabot-core/issues/11691) — not relevant now (project uses 2021 edition); note for future if edition is upgraded

---
*Research completed: 2026-03-25*
*Ready for roadmap: yes*
