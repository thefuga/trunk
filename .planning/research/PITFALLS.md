# Pitfalls Research: Trunk v0.10 CI/CD & Cross-Platform Releases

**Domain:** Adding CI/CD pipeline and cross-platform release publishing to an existing Tauri 2 + Svelte 5 + Rust desktop Git GUI
**Researched:** 2026-03-25
**Confidence:** HIGH -- based on official Tauri 2 distribution docs, tauri-action issue tracker, community post-mortems, and direct analysis of the project's Cargo.toml, package.json, and tauri.conf.json

---

## Context: What Is Changing in v0.10

v0.9 shipped multi-tab and tree view. The codebase is ~13,400 LOC TypeScript/Svelte, ~9,400 LOC Rust with 49 completed phases. All local development and testing has happened on macOS (darwin). No CI pipeline exists. No release binaries have been published.

v0.10 adds:
1. **CI checks** on every push/PR: cargo check, clippy, cargo test, cargo fmt, bun run check, bun run test, prettier
2. **Cross-platform release pipeline**: macOS (.dmg), Linux (.AppImage), Windows (.msi)
3. **Tag-triggered releases**: push `v*` tag -> build all platforms -> publish GitHub Release
4. **Changelog generation** from commit messages
5. **Dependabot** for Rust + npm dependency updates

---

## Critical Pitfalls

### Pitfall 1: Version Mismatch Across Three Manifests Breaks Releases

**What goes wrong:**
Trunk has version numbers in three places: `package.json` (currently `0.1.0`), `src-tauri/Cargo.toml` (currently `0.1.0`), and `src-tauri/tauri.conf.json` (currently `0.1.0`). The tauri-action uses `tauri.conf.json`'s version to generate the `tagName` (via `v__VERSION__` template). If you tag `v0.10.0` but `tauri.conf.json` still says `0.1.0`, the action creates a release named `v0.1.0` -- which does not match the tag. Artifacts upload to the wrong release (or fail to upload entirely). This is the single most common CI/CD failure reported by Tauri developers.

Additionally, after bumping versions, `Cargo.lock` must be regenerated (`cargo generate-lockfile`), or the CI build uses stale dependency resolution.

**Why it happens:**
There is no built-in version sync mechanism in Tauri. Developers bump one file and forget the others. The mismatch is invisible locally because `bun run dev` does not validate version alignment.

**How to avoid:**
1. Remove the `version` field from `tauri.conf.json` entirely. Tauri 2 falls back to `Cargo.toml`'s version when `tauri.conf.json` omits it. This reduces the sync points to two files (package.json + Cargo.toml).
2. Create a `scripts/bump-version.sh` that updates both `package.json` and `Cargo.toml`, runs `cargo generate-lockfile`, and creates the git tag. This is the single entry point for all version changes.
3. Add a CI check that verifies `package.json` version matches `Cargo.toml` version on every PR. Fail the build if they diverge.

**Warning signs:**
- Release workflow creates a release with unexpected version number
- Artifacts appear under a different release than the tag
- "No releaseId or tagName provided, skipping all uploads" error in CI logs

**Phase to address:**
Phase 1 (CI foundation) -- version sync validation must exist before the first release is attempted

---

### Pitfall 2: tauri-action Draft Release Race Condition Creates Duplicate Releases

**What goes wrong:**
The tauri-action runs as parallel matrix jobs (macOS-arm, macOS-intel, Linux, Windows). Each job calls the GitHub API to find or create the release. When the first job creates a draft release and subsequent jobs call `updateRelease()`, the API call inadvertently strips the tag association from the draft release. Later jobs cannot find the tagged release and create duplicates. The result: some artifacts on Release A, other artifacts on Release B, and manual cleanup required.

This is a known bug (tauri-apps/tauri-action#914) that has become more frequent. The root cause is in `create-release.ts` lines 142-150.

**Why it happens:**
GitHub's Release API has subtle behavior differences between draft and published releases. `updateRelease()` on a draft release can remove the `tag_name` association in certain race windows. Multiple parallel jobs hitting the API simultaneously create this race.

**How to avoid:**
Create the GitHub Release in a separate job BEFORE the build matrix starts. Use a two-phase workflow:
1. **Job 1 (create-release):** Create the draft release, output the `release_id`
2. **Job 2 (build, needs: create-release):** Matrix build across platforms, upload artifacts to the existing release using the `release_id` from Job 1

This eliminates the concurrent release creation race entirely. The build jobs only upload artifacts -- they never create or update the release metadata.

**Warning signs:**
- Two releases appear for the same tag version
- Some platform artifacts missing from the release
- CI logs show "Release already exists" warnings

**Phase to address:**
Phase 2 (release pipeline) -- the workflow structure must prevent this from the start

---

### Pitfall 3: Bun Not Found on GitHub Actions Runners

**What goes wrong:**
The project uses bun (evidenced by `bun.lockb` or `bun.lock`). Standard GitHub Actions runners do not have bun pre-installed. The tauri-action auto-detects the package manager by looking for lock files. When it finds `bun.lockb`, it assumes bun is available and runs `bun install` -- which fails with `sh: 1: bun: not found` (exit code 127). The entire build fails before Rust compilation even starts.

On Windows specifically, bun has additional compatibility issues: `bun run --bun` may fall back to Node instead of using the Bun runtime, and `bun install` can trigger assertion failures on Windows Server 2022/2025 runners.

**Why it happens:**
GitHub runners come with Node.js and npm pre-installed, but not bun. The tauri-action does not install package managers -- it only detects and uses them. Developers who use bun locally forget it needs explicit setup in CI.

**How to avoid:**
Add `oven-sh/setup-bun@v2` as an early step in every workflow, before any step that might trigger `beforeBuildCommand`. Pin a specific bun version to match local development. Example:
```yaml
- uses: oven-sh/setup-bun@v2
  with:
    bun-version: "1.1.39"  # match local version
```

Also ensure `tauri.conf.json`'s `beforeBuildCommand` uses `bun run build` (already correct in the project).

**Warning signs:**
- `bun: not found` or exit code 127 in CI logs
- Windows builds fail with "Bun is not defined" errors
- Lock file format mismatch warnings

**Phase to address:**
Phase 1 (CI foundation) -- bun setup must be in the workflow from the first CI run

---

### Pitfall 4: Linux Builds Missing System Dependencies

**What goes wrong:**
Tauri 2 on Linux requires system libraries that are not pre-installed on GitHub's Ubuntu runners: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, and `patchelf`. Without them, the Rust build fails during the Tauri framework compilation. Additionally, this project uses `git2` with `vendored-libgit2` which needs `cmake` and a C compiler to build libgit2 from source -- these are typically present on runners but worth verifying.

A subtler issue: Tauri v1 required `libwebkit2gtk-4.0-dev` while Tauri v2 requires `libwebkit2gtk-4.1-dev`. Copy-pasting a v1 workflow causes a confusing compilation failure.

**Why it happens:**
GitHub's Ubuntu runners have a base set of packages but not GTK/WebKit development headers. These are specific to building desktop GUI applications, which is not a common CI use case. Developers on macOS or Windows never encounter this because those platforms bundle their webview runtimes.

**How to avoid:**
Add a Linux-only step that installs all required system packages:
```yaml
- name: Install Linux dependencies
  if: matrix.platform == 'ubuntu-22.04'
  run: |
    sudo apt-get update
    sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

Use `ubuntu-22.04` (not `ubuntu-latest` which may point to 24.04) to ensure maximum AppImage compatibility with older glibc versions.

**Warning signs:**
- `Package libwebkit2gtk-4.1-dev is not available` (wrong Ubuntu version)
- `Package libwebkit2gtk-4.0-dev` in your config (Tauri v1 dependency, wrong for v2)
- Linker errors about missing GTK/WebKit symbols

**Phase to address:**
Phase 1 (CI foundation) for check/test workflows; Phase 2 (release pipeline) for build matrix

---

### Pitfall 5: Uncached Rust Builds Take 30-60 Minutes

**What goes wrong:**
A fresh Tauri build from scratch (no cache) compiles the entire Rust dependency tree: tauri framework, git2 (which vendored-compiles libgit2 from C source), serde, tokio, notify, and all transitive dependencies. On GitHub Actions runners, this takes 30-60 minutes per platform. With 4 platform targets (macOS arm, macOS Intel, Linux, Windows), a single release burns 2-4 hours of runner time. Without caching, every CI run pays this full cost.

Even with caching, the CI profile matters. Debug builds generate enormous artifacts (500MB+) that bloat the cache. Incremental compilation (the default) creates larger artifacts that are slower to cache and restore than non-incremental builds.

**Why it happens:**
Rust is a compiled language with no pre-built binary caches (unlike npm). The `vendored-libgit2` feature adds C compilation on top of Rust compilation. GitHub's 10GB cache limit per repository can be exhausted quickly with multiple platform caches.

**How to avoid:**
1. Use `swatinem/rust-cache@v2` with `workspaces: "src-tauri -> target"` to cache the Rust target directory. This brings subsequent builds down to 5-15 minutes.
2. Create a CI-specific Cargo profile that disables debug info and incremental compilation to reduce cache size:
   ```toml
   # src-tauri/Cargo.toml
   [profile.ci]
   inherits = "dev"
   debug = false
   incremental = false
   ```
3. Split CI checks from release builds. CI checks (clippy, fmt, test) need debug builds and should cache separately from release builds (which use `--release`).
4. Run `cargo fmt --check` and `cargo clippy` AFTER `cargo build` (not before). Clippy reuses build artifacts from cargo build, but not vice versa -- this saves ~5 minutes per run.
5. Set `fail-fast: false` on the build matrix so one platform failure does not cancel the others.

**Warning signs:**
- CI runs taking >30 minutes
- Cache size warnings from GitHub Actions
- Cache misses on every run (check the `swatinem/rust-cache` output logs)

**Phase to address:**
Phase 1 (CI foundation) -- caching must be configured from the first workflow; retrofitting after habits form wastes weeks of accumulated runner time

---

### Pitfall 6: AppImage glibc Compatibility -- Build System Determines Minimum Target

**What goes wrong:**
The AppImage format bundles dependencies but does NOT bundle glibc. The AppImage's minimum glibc requirement equals the glibc version on the build system. Building on Ubuntu 24.04 (glibc 2.39) produces an AppImage that fails on Ubuntu 22.04 (glibc 2.35) with `GLIBC_2.39 not found`. This is not fixable after the fact -- the binary must be rebuilt on an older system.

This is the most common Linux distribution complaint for Tauri apps. Users download the AppImage, run it, and get a cryptic glibc error with no actionable fix.

**Why it happens:**
`ubuntu-latest` on GitHub Actions currently points to Ubuntu 24.04 (or may shift over time). Developers use it without realizing it determines their minimum supported Linux version. The AppImage format gives a false sense of portability -- "it bundles everything" except the one thing that matters most (glibc).

**How to avoid:**
Pin the Linux runner to `ubuntu-22.04` explicitly (not `ubuntu-latest`). This ensures compatibility with Ubuntu 22.04+ and most current Linux distributions. Do NOT use `ubuntu-20.04` -- it is deprecated and missing `libwebkit2gtk-4.1-dev` needed for Tauri 2.

Document the minimum Linux requirement (Ubuntu 22.04 / glibc 2.35) in the release notes.

**Warning signs:**
- User reports "GLIBC_X.XX not found" when running the AppImage
- `ubuntu-latest` appears in your workflow file without a pinned version
- GitHub changes what `ubuntu-latest` points to (check their runner images changelog)

**Phase to address:**
Phase 2 (release pipeline) -- runner version must be pinned when configuring the build matrix

---

### Pitfall 7: Unsigned Builds Trigger OS Security Warnings on All Platforms

**What goes wrong:**
Without code signing:
- **macOS**: Gatekeeper shows "trunk is damaged and can't be opened" or "Apple cannot check it for malicious software." Users must right-click > Open (or run `xattr -cr trunk.app` from terminal) to bypass. This is a dealbreaker for non-technical users.
- **Windows**: SmartScreen shows "Windows protected your PC" with a scary blue warning. Users must click "More info" > "Run anyway." Browser downloads may be blocked entirely.
- **Linux**: No signing issues. AppImage runs after `chmod +x`.

The v0.10 scope explicitly excludes code signing ("No code signing (unsigned builds)"). This is a valid decision for a personal/early-stage project, but the UX impact must be documented for end users.

**Why it happens:**
Apple and Microsoft require paid developer accounts ($99/year and ~$200-400/year respectively) and per-build notarization/signing processes. For a personal learning project, this cost and complexity is premature.

**How to avoid:**
Since code signing is explicitly out of scope for v0.10:
1. Document the bypass instructions prominently in every GitHub Release's description:
   - macOS: "Right-click the app > Open, then click Open in the dialog"
   - Windows: "Click 'More info' then 'Run anyway' on the SmartScreen dialog"
2. Include these instructions in the README's installation section.
3. Do NOT set `releaseDraft: false` -- use draft releases so you can review and add installation instructions before publishing.
4. Plan code signing for a future milestone (v1.0 or later).

**Warning signs:**
- Users file issues saying the app "doesn't work" or "is broken" on macOS
- Download counts are high but active user count is low (people download, hit the warning, give up)

**Phase to address:**
Phase 2 (release pipeline) -- release notes template must include bypass instructions; Phase 3 (changelog/docs) should formalize installation docs

---

### Pitfall 8: GitHub Token Permissions Block Release Creation

**What goes wrong:**
The default `GITHUB_TOKEN` in GitHub Actions has read-only permissions. The tauri-action needs write access to create releases and upload artifacts. Without it, the workflow fails with "Resource not accessible by integration" -- a vague error that does not mention permissions.

**Why it happens:**
GitHub tightened default token permissions for security. New repositories default to read-only. The error message does not clearly state "you need write permissions" -- it says "resource not accessible" which developers misinterpret as a repository visibility issue.

**How to avoid:**
Add explicit permissions in the workflow file:
```yaml
permissions:
  contents: write
```

This is more reliable than changing repository settings (which can be reset) and is visible in code review.

**Warning signs:**
- "Resource not accessible by integration" in CI logs
- Release job succeeds (exit 0) but no release appears on GitHub
- "403 Forbidden" errors in tauri-action output

**Phase to address:**
Phase 2 (release pipeline) -- must be in the workflow file from the first release attempt

---

### Pitfall 9: Changelog Generation Fails With Shallow Clone

**What goes wrong:**
GitHub Actions checks out repositories with `fetch-depth: 1` by default (shallow clone). Changelog generators like git-cliff need the full commit history to generate meaningful release notes. With a shallow clone, git-cliff sees only the HEAD commit and produces an empty or single-entry changelog. The release goes out with no useful release notes.

**Why it happens:**
Shallow clones are a CI optimization -- they are faster and use less disk. Most CI tasks (build, test) do not need history. But changelog generation is specifically a history-dependent operation.

**How to avoid:**
Set `fetch-depth: 0` in the checkout step of any job that generates changelogs:
```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0
```

Only do this in the release/changelog job, not in the CI checks job (where shallow clone is fine and faster).

**Warning signs:**
- Changelog contains only the tag commit or is empty
- git-cliff warns about "no commits found" or "detached HEAD"
- Release notes say "No changes" despite many commits

**Phase to address:**
Phase 3 (changelog generation) -- must be configured when adding git-cliff to the workflow

---

### Pitfall 10: Dependabot Lockfile Corruption With Bun

**What goes wrong:**
Dependabot gained bun support in February 2025, but it has known issues: it may remove the `configVersion` line from `bun.lock` files (issue dependabot/dependabot-core#13623), and it does not support the legacy binary `bun.lockb` format. If the project uses `bun.lockb`, Dependabot's npm ecosystem handler may try to generate a `package-lock.json` instead, creating conflicting lock files.

Additionally, Dependabot's Rust support requires a Cargo version that matches the project's Rust edition. Since this project uses Rust 2021 edition, this is not currently an issue, but upgrading to Rust 2024 edition later would require Dependabot to use cargo 1.85.0+.

**Why it happens:**
Bun support in Dependabot is relatively new (GA February 2025). Edge cases with lockfile format versions are still being resolved. The bun lockfile format changed from binary (`bun.lockb`) to text (`bun.lock`) in bun 1.1.39+.

**How to avoid:**
1. Ensure the project uses text-based `bun.lock` (not binary `bun.lockb`). Run `bun install` with bun >= 1.1.39 to generate the text format.
2. In `.github/dependabot.yml`, configure both ecosystems explicitly:
   ```yaml
   version: 2
   updates:
     - package-ecosystem: "cargo"
       directory: "/src-tauri"
       schedule:
         interval: "weekly"
     - package-ecosystem: "npm"
       directory: "/"
       schedule:
         interval: "weekly"
   ```
3. Pin the Dependabot schedule to weekly (not daily) to avoid PR fatigue.
4. Add a CI check that runs `bun install --frozen-lockfile` to catch lockfile inconsistencies from Dependabot PRs.

**Warning signs:**
- Dependabot PRs that change `package-lock.json` instead of `bun.lock`
- Lockfile format version mismatch warnings after merging Dependabot PRs
- Dependabot fails to open PRs for bun dependencies (check Dependabot logs in repository settings)

**Phase to address:**
Phase 3 or 4 (Dependabot configuration) -- configure after CI checks are working so Dependabot PRs are validated automatically

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Using `ubuntu-latest` instead of pinned version | Less maintenance when GitHub updates runners | AppImage breaks on older distros when GitHub bumps the runner OS | Never for release builds; acceptable for CI checks |
| Skipping macOS Intel builds (arm-only) | Halves macOS build time | Excludes pre-2020 Mac users | Acceptable if targeting only Apple Silicon users |
| No version sync check in CI | Less CI configuration | Silent version mismatch causes release failures | Never -- the check is trivial and prevents the most common failure |
| Single workflow file for CI + releases | Simpler to maintain | CI check changes risk breaking release pipeline; harder to read | Acceptable for v0.10; split later when complexity grows |
| No Cargo.lock in repository | Avoids lock file merge conflicts | CI builds use different dependency versions than local; non-reproducible builds | Never for applications (Cargo.lock should be committed per Rust convention) |
| Relying on tauri-action to create releases | Less workflow code | Race condition creates duplicate releases (Pitfall 2) | Never -- create release in a separate job |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| tauri-action + bun | Forgetting `oven-sh/setup-bun` before tauri-action | Add setup-bun step before any step that triggers `beforeBuildCommand` |
| tauri-action + release | Letting parallel matrix jobs create the release | Create release in a dedicated job; matrix jobs only upload artifacts |
| swatinem/rust-cache + matrix | Using same cache key across platforms | rust-cache auto-scopes by OS; but add `shared-key` for related jobs (e.g., "ci-check" vs "release") |
| git-cliff + shallow clone | Default `fetch-depth: 1` produces empty changelog | Set `fetch-depth: 0` only in the release job |
| Dependabot + bun | Dependabot generates `package-lock.json` instead of `bun.lock` | Ensure text-based `bun.lock` exists; verify Dependabot detects bun ecosystem |
| GitHub token + release creation | Default read-only token cannot create releases | Add `permissions: contents: write` in workflow |
| macOS targets + rust-toolchain | Missing `aarch64-apple-darwin` or `x86_64-apple-darwin` target | Install both targets via `dtolnay/rust-toolchain` with `targets` input |
| Ubuntu version + Tauri 2 | Using `libwebkit2gtk-4.0-dev` (Tauri v1) instead of `4.1` (Tauri v2) | Use `libwebkit2gtk-4.1-dev` for Tauri 2 |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| No Rust compilation cache | Every CI run takes 30-60 min per platform | Use `swatinem/rust-cache@v2` with correct workspace path | First run always (cold cache); subsequent runs if cache key changes |
| Debug info in CI builds | Cache exceeds 10GB limit, slow cache upload/download | Set `debug = false` in CI profile; or use `[profile.ci]` | ~3rd platform cache added |
| Running clippy before cargo build | Clippy cannot reuse build artifacts from subsequent build | Run `cargo build` first, then `cargo clippy` -- clippy reuses build cache | Every CI run (wasted ~5 minutes) |
| Full history checkout in CI checks | Slower checkout for checks that do not need history | Use `fetch-depth: 1` for CI checks, `fetch-depth: 0` only for changelog | Large repos with 10k+ commits |
| Serial lint/test/build steps | CI takes 3x longer than needed | Parallelize: fmt+clippy in one job, tests in another, build in another | Every CI run |
| vendored-libgit2 C compilation | ~2-3 min extra compilation for libgit2 from C source | Cache handles this after first build; no way to avoid on cold cache | Cold cache only |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Committing signing certificates or secrets to repo | Private keys exposed in git history forever | Use GitHub Secrets for all credentials; add `.p12`, `.pfx`, `*.keystore` to `.gitignore` |
| Using `GITHUB_TOKEN` in PR workflows from forks | Token has write access in fork PRs (if misconfigured) | Use `pull_request` trigger (not `pull_request_target`) for CI checks |
| Dependabot PRs running release workflow | Tag push from Dependabot could trigger unintended release | Release workflow should only trigger on `v*` tags, which Dependabot never creates |
| Storing API tokens in workflow files | Tokens visible in repository history | Always use `${{ secrets.TOKEN_NAME }}`; never hardcode |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No installation instructions in release notes | Users download unsigned binary, hit OS security warning, assume app is broken | Include platform-specific bypass instructions in every release description |
| AppImage without execute permission note | Linux users download, double-click, nothing happens | Release notes: "Run `chmod +x trunk.AppImage` before launching" |
| No changelog in release | Users cannot tell what changed between versions | Generate changelog with git-cliff and include in release body |
| Releasing as non-draft without review | Broken build goes live, users download broken artifact | Always create as draft, review artifacts, then publish manually |
| No "latest" release designation | Users on the releases page do not know which version to download | Mark the published release as "latest" in GitHub |

## "Looks Done But Isn't" Checklist

- [ ] **Version sync:** All three files (package.json, Cargo.toml, tauri.conf.json) have matching versions -- or tauri.conf.json omits version to inherit from Cargo.toml
- [ ] **CI checks pass on all platforms:** Not just "the build succeeds" but clippy, fmt, and tests pass on Linux, macOS, AND Windows
- [ ] **Release artifacts downloadable:** After publishing, download each artifact on its target platform and verify it launches (unsigned warning is expected)
- [ ] **macOS both architectures:** Release has both `.dmg` files -- one for Apple Silicon (aarch64), one for Intel (x86_64)
- [ ] **Linux AppImage runs on Ubuntu 22.04:** Download the AppImage on a fresh Ubuntu 22.04 and verify it launches without glibc errors
- [ ] **Windows .msi installs correctly:** Run the .msi installer, verify the app appears in Start Menu and launches
- [ ] **Changelog is non-empty:** Release notes contain actual commit summaries, not "No changes" or empty body
- [ ] **Dependabot opens PRs:** Check the Dependabot tab in repo settings -- PRs should appear within a week of configuration
- [ ] **Cache working:** Second CI run is significantly faster than first (check swatinem/rust-cache logs for "cache hit")
- [ ] **Concurrency control:** Push two tags rapidly -- second run cancels first (or waits), no duplicate releases created
- [ ] **Fork PRs safe:** A PR from a fork runs CI checks but cannot trigger release workflow or access secrets

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Version mismatch broke release | LOW | Delete the incorrect release, fix versions in all files, re-tag, re-push |
| Duplicate releases created | LOW | Delete the duplicate release, re-run the workflow (or manually upload missing artifacts) |
| AppImage built on wrong Ubuntu | MEDIUM | Cannot fix the artifact; must rebuild on correct Ubuntu version; update workflow and re-release |
| Unsigned build complaints | LOW | Add installation instructions to release notes; plan code signing for future milestone |
| Cache bloated past 10GB | LOW | Delete cache entries via GitHub API (`gh cache delete`); add CI profile to reduce artifact size |
| Dependabot corrupted lockfile | LOW | Revert the PR; regenerate lockfile locally with `bun install`; push fix |
| Shallow clone empty changelog | LOW | Re-run with `fetch-depth: 0`; or generate changelog locally and paste into release notes |
| Token permissions block release | LOW | Add `permissions: contents: write` to workflow; re-run |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| P1: Version mismatch | Phase 1 (CI foundation) | CI check validates version alignment on every PR |
| P2: Duplicate release race | Phase 2 (release pipeline) | Push a tag, verify exactly one release with all platform artifacts |
| P3: Bun not found | Phase 1 (CI foundation) | CI workflow includes setup-bun; first run succeeds |
| P4: Missing Linux deps | Phase 1 (CI foundation) | Linux CI job passes cargo check + clippy |
| P5: Uncached slow builds | Phase 1 (CI foundation) | Second CI run takes <15 minutes (check cache hit logs) |
| P6: AppImage glibc compat | Phase 2 (release pipeline) | Download AppImage, run on Ubuntu 22.04 VM/container |
| P7: Unsigned build warnings | Phase 2 (release pipeline) | Release notes include bypass instructions for macOS and Windows |
| P8: Token permissions | Phase 2 (release pipeline) | Release workflow creates draft release successfully |
| P9: Shallow clone changelog | Phase 3 (changelog) | Release notes contain grouped commit summaries |
| P10: Dependabot lockfile | Phase 3-4 (Dependabot) | Dependabot PR passes CI checks including `bun install --frozen-lockfile` |

## Sources

- [Tauri 2 GitHub Actions Distribution Docs](https://v2.tauri.app/distribute/pipelines/github/) -- official workflow examples, Linux dependencies, runner matrix
- [Tauri 2 AppImage Distribution Docs](https://v2.tauri.app/distribute/appimage/) -- glibc compatibility warning, bundle size, GStreamer
- [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action) -- action configuration, release creation, package manager detection
- [tauri-action#914: Duplicate Releases Race Condition](https://github.com/tauri-apps/tauri-action/issues/914) -- root cause analysis, workaround (separate release job)
- [tauri-action#986: bun: not found](https://github.com/tauri-apps/tauri-action/issues/986) -- setup-bun requirement for bun projects
- [tauri-apps/tauri#8265: Version Sync Feature Request](https://github.com/tauri-apps/tauri/issues/8265) -- community discussion on version management
- [tauri-apps/tauri Discussion#6347: Version Sync](https://github.com/tauri-apps/tauri/discussions/6347) -- workaround patterns, single source of truth
- [VaultNote CI/CD Post-Mortem](https://dev.to/dev_michael/my-first-tauri-cicd-pipeline-lessons-from-building-vaultnote-with-sveltekit-17mp) -- version sync lessons, 60+ failed runs
- [Ship Tauri v2 Like a Pro: GitHub Actions (Part 2)](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) -- draft release pattern, concurrency control, fail-fast
- [How to Make Rust CI 2-3x Faster](https://www.reillywood.com/blog/rust-faster-ci/) -- cache strategy, clippy ordering, nextest
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) -- cache key strategy, workspace configuration
- [git-cliff](https://git-cliff.org/) -- changelog generation, GitHub Actions integration
- [orhun/git-cliff-action](https://github.com/orhun/git-cliff-action) -- fetch-depth requirement, configuration
- [Dependabot Bun Support (GA Feb 2025)](https://github.blog/changelog/2025-02-13-dependabot-version-updates-now-support-the-bun-package-manager-ga/) -- bun.lock requirements
- [dependabot-core#13623: Bun Lockfile configVersion Removal](https://github.com/dependabot/dependabot-core/issues/13623) -- known bun lockfile corruption
- [dependabot-core#11691: Rust 2024 Edition Support](https://github.com/dependabot/dependabot-core/issues/11691) -- cargo version requirements
- [Ship Tauri v2 Like a Pro: Code Signing (Part 1)](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-code-signing-for-macos-and-windows-part-12-3o9n) -- unsigned build UX impact

---
*Pitfalls research for: Trunk v0.10 -- CI/CD & Cross-Platform Releases*
*Researched: 2026-03-25*
