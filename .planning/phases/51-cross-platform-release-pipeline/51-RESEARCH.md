# Phase 51: Cross-Platform Release Pipeline - Research

**Researched:** 2026-03-25
**Domain:** GitHub Actions CI/CD, Tauri cross-platform builds, artifact distribution
**Confidence:** HIGH

## Summary

This phase creates a GitHub Actions release workflow triggered by `v*` tag pushes that builds Trunk for four targets: macOS ARM (aarch64-apple-darwin), macOS Intel (x86_64-apple-darwin), Linux x64, and Windows x64. The workflow produces platform-specific installers (.dmg, .AppImage, .msi) and portable .tar.gz archives, uploaded as workflow artifacts.

The `tauri-apps/tauri-action@v0` action is the standard tool for Tauri release builds. It handles the full build pipeline (frontend build + Rust compilation + bundling) and outputs installers per platform. The action supports a build-only mode (no GitHub Release creation) by omitting `tagName`/`releaseName`/`releaseId`. Portable .tar.gz archives require a post-build step since Tauri only generates them automatically when the updater is configured with signing keys.

**Primary recommendation:** Use `tauri-apps/tauri-action@v0` in build-only mode with a matrix strategy, then upload all artifacts (installers + manually-created tar.gz archives) via `actions/upload-artifact@v4`.

## Project Constraints (from CLAUDE.md)

- All git operations go through git2 crate, no shelling out (N/A for this phase -- CI only)
- Never inline colors (N/A for this phase -- no UI changes)
- `bun` is the package manager -- CI must use `oven-sh/setup-bun@v2`
- Build command: `bun run build` (frontend), Tauri handles the rest
- Rust crate at `src-tauri/` -- cargo commands need `--manifest-path src-tauri/Cargo.toml`

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Separate `release.yml` workflow (not part of ci.yml) triggered by tag push matching `v*`
- **D-02:** Matrix strategy with per-platform jobs running in parallel (macOS ARM, macOS Intel, Linux, Windows)
- **D-03:** No CI gate dependency -- tag push implies code on main is already CI-validated
- **D-04:** Use `tauri-apps/tauri-action@v0` for building
- **D-05:** Each matrix entry configures tauri-action with appropriate target and runner
- **D-06:** Separate ARM and Intel macOS builds (not universal binary)
- **D-07:** Rationale: REL-01 explicitly lists separate targets
- **D-08:** Post-build step wraps tauri-action output into .tar.gz archives per platform
- **D-09:** .tar.gz contains the platform-appropriate binary/app bundle
- **D-10:** All artifacts uploaded via `actions/upload-artifact`
- **D-11:** No automated GitHub Release creation
- **D-12:** Linux builds on `ubuntu-22.04` (REL-05)
- **D-13:** macOS builds on `macos-latest` (ARM) and `macos-13` (Intel, last Intel runner)
- **D-14:** Windows builds on `windows-latest`

### Claude's Discretion
- Exact matrix configuration format and variable naming
- Concurrency controls (cancel-in-progress for same tag, or allow all)
- Whether to cache Rust builds in release workflow
- Specific tauri-action configuration options and version pinning
- Artifact naming convention

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| REL-01 | Tag push (v*) triggers cross-platform builds for macOS ARM, macOS Intel, Linux x64, Windows x64 | Matrix strategy with 4 entries; tauri-action `args` for target selection; tag trigger via `on: push: tags: ['v*']` |
| REL-02 | Release workflow produces .dmg (macOS), .AppImage (Linux), .msi (Windows) installers | tauri-action with `bundle.targets: "all"` in tauri.conf.json produces all platform-appropriate installers automatically |
| REL-03 | Release workflow produces portable .tar.gz archives for each platform | Post-build shell step creates tar.gz from bundle output directory; different paths per platform |
| REL-04 | Build artifacts uploaded as workflow artifacts | `actions/upload-artifact@v4` with per-platform artifact names; tauri-action `artifactPaths` output or known bundle paths |
| REL-05 | Linux builds use ubuntu-22.04 for AppImage glibc compatibility | Matrix entry uses `ubuntu-22.04` runner explicitly (not `ubuntu-latest`) |
</phase_requirements>

## Standard Stack

### Core

| Library/Action | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| tauri-apps/tauri-action | @v0 (resolves to v0.6.x) | Build Tauri app + bundle installers | Official Tauri action; handles frontend build, Rust compilation, and bundling in one step |
| actions/checkout | @v6 | Clone repository | Standard; matches existing ci.yml |
| actions/upload-artifact | @v4 | Upload build artifacts | Standard GHA artifact upload; v4 is current |
| dtolnay/rust-toolchain | @stable | Install Rust toolchain + targets | Standard; matches existing ci.yml |
| oven-sh/setup-bun | @v2 | Install bun package manager | Required by project (bun, not npm); matches existing ci.yml |
| Swatinem/rust-cache | @v2 | Cache Rust compilation | Optional for release but saves time; matches existing ci.yml |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tauri-action | Manual `bun tauri build` | tauri-action handles bundling + artifact collection automatically; manual approach gives more control but much more YAML |
| Separate ARM/Intel builds | Universal binary (`--target universal-apple-darwin`) | D-06 locks separate builds; universal binaries are larger and harder to debug |
| upload-artifact | tauri-action uploadWorkflowArtifacts | Built-in option exists but may be removed; upload-artifact@v4 gives explicit control over naming and retention |

## Architecture Patterns

### Workflow Structure

```
.github/
  workflows/
    ci.yml          # Existing - push/PR quality gates
    release.yml     # NEW - tag-triggered release builds
```

### Pattern 1: Matrix Strategy with Platform-Specific Configuration

**What:** Single job definition with a matrix that varies runner, target, and platform-specific steps.
**When to use:** Cross-platform builds where each platform needs slightly different setup but the core build step is identical.

```yaml
# Source: Tauri v2 official docs + tauri-action README
strategy:
  fail-fast: false
  matrix:
    include:
      - platform: 'macos-latest'
        args: '--target aarch64-apple-darwin'
        rust_target: 'aarch64-apple-darwin'
      - platform: 'macos-15-intel'
        args: '--target x86_64-apple-darwin'
        rust_target: 'x86_64-apple-darwin'
      - platform: 'ubuntu-22.04'
        args: ''
        rust_target: ''
      - platform: 'windows-latest'
        args: ''
        rust_target: ''
```

### Pattern 2: Build-Only Mode (No Release Creation)

**What:** Use tauri-action without `tagName`/`releaseName` to just build, then handle artifacts separately.
**When to use:** When release creation is manual (D-11).

```yaml
# Source: tauri-action README
- uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  with:
    args: ${{ matrix.args }}
    # No tagName, releaseName, or releaseId = build-only mode
```

### Pattern 3: Post-Build tar.gz Creation

**What:** After tauri-action builds installers, create portable .tar.gz archives from the binary/app bundle output.
**When to use:** For REL-03 portable archive requirement.

```yaml
# macOS: tar the .app bundle
- name: Create portable archive (macOS)
  if: runner.os == 'macOS'
  run: |
    cd src-tauri/target/*/release/bundle/macos
    tar -czf trunk-macos-${{ matrix.rust_target }}.tar.gz *.app

# Linux: tar the AppImage
- name: Create portable archive (Linux)
  if: runner.os == 'Linux'
  run: |
    cd src-tauri/target/release/bundle/appimage
    tar -czf trunk-linux-x86_64.tar.gz *.AppImage

# Windows: tar the .exe
- name: Create portable archive (Windows)
  if: runner.os == 'Windows'
  shell: bash
  run: |
    cd src-tauri/target/release/bundle/nsis
    tar -czf trunk-windows-x86_64.tar.gz *.exe
```

### Pattern 4: Per-Platform Artifact Upload

**What:** Upload artifacts with platform-specific names so they are distinguishable in the workflow artifacts list.
**When to use:** When multiple matrix jobs produce different artifacts.

```yaml
- uses: actions/upload-artifact@v4
  with:
    name: trunk-${{ matrix.platform }}-${{ matrix.rust_target || 'x64' }}
    path: |
      src-tauri/target/*/release/bundle/**/*.dmg
      src-tauri/target/*/release/bundle/**/*.AppImage
      src-tauri/target/*/release/bundle/**/*.msi
      src-tauri/target/*/release/bundle/**/*.tar.gz
```

### Anti-Patterns to Avoid

- **Using `ubuntu-latest` for Linux builds:** This may resolve to a newer Ubuntu version with a newer glibc, breaking AppImage compatibility on older distros. Always pin to `ubuntu-22.04` (REL-05).
- **Using `macos-13` for Intel builds:** macOS 13 runners were deprecated September 2025 and fully removed December 2025. Use `macos-15-intel` instead.
- **Creating a GitHub Release in the workflow:** D-11 explicitly forbids this. Omit `tagName`/`releaseName` from tauri-action.
- **Using `actions/upload-artifact@v3`:** v3 is deprecated; v4 changed the artifact model (each artifact name must be unique, no merging).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tauri build + bundling | Manual cargo build + bundler invocation | `tauri-apps/tauri-action@v0` | Handles frontend build, Rust compilation, bundling, and artifact path resolution in one step |
| Rust toolchain setup | curl/rustup manual install | `dtolnay/rust-toolchain@stable` | Handles target installation, component setup, caching hints |
| Artifact upload | Manual gh CLI artifact upload | `actions/upload-artifact@v4` | Standard, handles compression, retention, multi-file upload |

## Common Pitfalls

### Pitfall 1: macOS-13 Runner No Longer Exists

**What goes wrong:** Workflow fails immediately with "runner not found" error.
**Why it happens:** CONTEXT.md D-13 specifies `macos-13` for Intel, but this runner was removed December 2025.
**How to avoid:** Use `macos-15-intel` instead. This is the last Intel runner, available until August 2027.
**Warning signs:** Workflow fails on macOS Intel job before any build step runs.

### Pitfall 2: macOS Cross-Compilation Target Not Installed

**What goes wrong:** Rust compilation fails with "can't find crate for `std`" when targeting x86_64 from an ARM runner.
**Why it happens:** `macos-latest` is an ARM runner. Building for x86_64-apple-darwin requires installing that target explicitly.
**How to avoid:** Install both targets on macOS runners: `targets: aarch64-apple-darwin,x86_64-apple-darwin` in rust-toolchain setup. Or install only the needed target per matrix entry: `targets: ${{ matrix.rust_target }}`.
**Warning signs:** Compilation error mentioning missing target or std library.

### Pitfall 3: Linux System Dependencies Missing

**What goes wrong:** Cargo build fails with missing header errors (webkit2gtk, appindicator, etc.).
**Why it happens:** Tauri Linux builds need system libraries not present on bare GitHub runners.
**How to avoid:** Install system deps before build:
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf
```
**Warning signs:** Compilation errors about missing `.h` files or pkg-config failures.

### Pitfall 4: Bundle Output Path Varies by Target

**What goes wrong:** Post-build tar.gz step can't find artifacts because paths differ between native and cross-compiled builds.
**Why it happens:** Cross-compiled macOS builds output to `target/{target_triple}/release/bundle/` while native builds output to `target/release/bundle/`.
**How to avoid:** Use glob patterns or the `artifactPaths` output from tauri-action. For macOS: `src-tauri/target/*/release/bundle/` or the specific `src-tauri/target/aarch64-apple-darwin/release/bundle/`.
**Warning signs:** "No such file or directory" errors in post-build steps.

### Pitfall 5: upload-artifact v4 Requires Unique Names

**What goes wrong:** Artifact upload fails with "name already exists" error.
**Why it happens:** v4 does not allow multiple uploads with the same artifact name (v3 did by appending). Each matrix job uploads with the same name.
**How to avoid:** Include platform/arch in the artifact name: `name: trunk-${{ matrix.platform }}-${{ matrix.rust_target || 'x64' }}`.
**Warning signs:** "An artifact with this name already exists" error in upload step.

### Pitfall 6: Windows Shell Differences

**What goes wrong:** Shell commands in post-build steps fail on Windows.
**Why it happens:** Windows runners default to PowerShell, not bash. Commands like `cd`, `tar`, `ls` may behave differently.
**How to avoid:** Use `shell: bash` on Windows steps that use Unix-style commands. Or use PowerShell-native commands.
**Warning signs:** Syntax errors or command-not-found on Windows jobs.

### Pitfall 7: Bun Not Detected by tauri-action

**What goes wrong:** tauri-action tries to use npm/yarn instead of bun for the frontend build.
**Why it happens:** tauri-action auto-detects the package manager but may not detect bun in all cases.
**How to avoid:** Set `tauriScript: bunx tauri` in tauri-action configuration to explicitly use bun.
**Warning signs:** "npm: command not found" or wrong lockfile warnings during build.

## Code Examples

### Complete Release Workflow Skeleton

```yaml
# Source: Tauri v2 official docs + tauri-action README, adapted for project decisions
name: Release

on:
  push:
    tags:
      - 'v*'

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: true

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
            rust_target: 'aarch64-apple-darwin'
          - platform: 'macos-15-intel'
            args: '--target x86_64-apple-darwin'
            rust_target: 'x86_64-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
            rust_target: ''
          - platform: 'windows-latest'
            args: ''
            rust_target: ''

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v6

      - name: Install system dependencies (Linux)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf

      - uses: oven-sh/setup-bun@v2
      - run: bun install --frozen-lockfile

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.rust_target }}

      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri -> target"

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          args: ${{ matrix.args }}
          tauriScript: bunx tauri

      # Post-build: create portable tar.gz archives
      # ... platform-specific steps ...

      - uses: actions/upload-artifact@v4
        with:
          name: trunk-${{ matrix.platform }}-${{ matrix.rust_target || 'x64' }}
          path: |
            src-tauri/target/**/release/bundle/**/*.dmg
            src-tauri/target/**/release/bundle/**/*.AppImage
            src-tauri/target/**/release/bundle/**/*.msi
            src-tauri/target/**/release/bundle/**/*.tar.gz
```

### Tauri Bundle Output Paths

```
# macOS (cross-compiled from ARM runner):
src-tauri/target/aarch64-apple-darwin/release/bundle/
  dmg/trunk_0.1.0_aarch64.dmg
  macos/trunk.app

src-tauri/target/x86_64-apple-darwin/release/bundle/
  dmg/trunk_0.1.0_x64.dmg
  macos/trunk.app

# Linux (native):
src-tauri/target/release/bundle/
  appimage/trunk_0.1.0_amd64.AppImage
  deb/trunk_0.1.0_amd64.deb

# Windows (native):
src-tauri/target/release/bundle/
  msi/trunk_0.1.0_x64_en-US.msi
  nsis/trunk_0.1.0_x64-setup.exe
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `macos-13` for Intel builds | `macos-15-intel` | December 2025 | macos-13 fully removed; macos-15-intel available until August 2027 |
| `actions/upload-artifact@v3` | `actions/upload-artifact@v4` | 2024 | v4 requires unique artifact names; no auto-merge of same-named uploads |
| `actions/checkout@v4` | `actions/checkout@v6` | 2025 | Project already uses v6 in ci.yml |
| tauri-action with release creation | Build-only mode + manual release | N/A (project decision) | Simpler workflow; release notes written manually |

**Critical update from CONTEXT.md D-13:** The context specifies `macos-13` for Intel builds, but this runner was deprecated September 2025 and removed December 2025. The correct replacement is `macos-15-intel`, which is an Intel (x86_64) runner on macOS 15, available until August 2027. This is the last Intel runner GitHub will offer.

## Open Questions

1. **Exact tauri-action `artifactPaths` output format**
   - What we know: It outputs "the paths of the generated artifacts" as a string
   - What's unclear: Whether it is newline-separated, JSON array, or glob pattern; exact paths per platform
   - Recommendation: Use known bundle output directory patterns (`src-tauri/target/**/release/bundle/`) rather than relying on the output variable. Glob patterns are more reliable.

2. **Whether `tauriScript: bunx tauri` works reliably on all platforms**
   - What we know: tauri-action auto-detects package manager from lockfile; project uses bun
   - What's unclear: Whether the auto-detection correctly finds `bun.lockb` and uses `bunx tauri`
   - Recommendation: Set `tauriScript: bunx tauri` explicitly to avoid any ambiguity. If it fails, fall back to letting it auto-detect.

3. **Rust cache effectiveness for release builds**
   - What we know: Release builds differ from debug builds (different optimization flags, different target dir)
   - What's unclear: Whether rust-cache@v2 meaningfully speeds up infrequent release builds vs. the cache storage cost
   - Recommendation: Include rust-cache but with `save-if: ${{ github.ref_type == 'tag' }}` to only save on tag pushes (releases are from main, and saving the cache is worthwhile for sequential tag pushes during testing).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | GitHub Actions workflow syntax + act (local runner, optional) |
| Config file | `.github/workflows/release.yml` (the deliverable itself) |
| Quick run command | `gh workflow view release.yml` (verify workflow exists) |
| Full suite command | Push a test tag: `git tag v0.0.0-test && git push origin v0.0.0-test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REL-01 | v* tag triggers 4-platform build | smoke | `gh workflow view release.yml` then `gh run list -w release.yml` after tag push | N/A (workflow file is the deliverable) |
| REL-02 | Produces .dmg, .AppImage, .msi | smoke | Inspect workflow artifacts after tag push: `gh run view <id> --json jobs` | N/A |
| REL-03 | Produces portable .tar.gz | smoke | Check artifacts list for .tar.gz entries | N/A |
| REL-04 | Artifacts uploaded as workflow artifacts | smoke | `gh run view <id> --json artifacts` | N/A |
| REL-05 | Linux on ubuntu-22.04 | manual-only | Read workflow file, verify `ubuntu-22.04` in matrix | N/A |

### Sampling Rate
- **Per task commit:** YAML lint check (`yamllint .github/workflows/release.yml` or manual review)
- **Per wave merge:** Not applicable (single-file deliverable)
- **Phase gate:** Push a test tag and verify all 4 platform jobs succeed with correct artifacts

### Wave 0 Gaps
None -- this phase produces a workflow file, not application code. Validation is done by pushing a tag and inspecting workflow run results.

## Sources

### Primary (HIGH confidence)
- [Tauri v2 GitHub Pipelines Guide](https://v2.tauri.app/distribute/pipelines/github/) - Official matrix configuration, system dependencies, runner labels
- [tauri-apps/tauri-action README](https://github.com/tauri-apps/tauri-action) - All input parameters, build-only mode, artifactPaths output, tauriScript option
- [actions/runner-images#13045](https://github.com/actions/runner-images/issues/13045) - macOS-15-intel runner availability (replacing deprecated macos-13)
- [GitHub Changelog: macOS 13 closing down](https://github.blog/changelog/2025-09-19-github-actions-macos-13-runner-image-is-closing-down/) - Deprecation timeline confirmed

### Secondary (MEDIUM confidence)
- [DeepWiki: tauri-action cross-platform builds](https://deepwiki.com/tauri-apps/tauri-action/4.3-cross-platform-builds) - Matrix strategy details, per-platform artifacts
- [DEV Community: Ship Tauri v2 App](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-github-actions-and-release-automation-part-22-2ef7) - Complete workflow example with code signing (we skip signing, but structure is useful)
- [actions/upload-artifact](https://github.com/actions/upload-artifact) - v4 API, retention-days, unique name requirement

### Tertiary (LOW confidence)
- tauri-action @v0 exact resolution version (likely v0.6.2 based on releases page, but not definitively confirmed what the v0 floating tag tracks)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Tauri docs prescribe tauri-action; GHA action versions verified against existing ci.yml
- Architecture: HIGH - Matrix strategy is well-documented; bundle output paths confirmed in multiple sources
- Pitfalls: HIGH - macOS-13 deprecation confirmed via GitHub changelog; upload-artifact v4 changes documented; system deps verified from existing ci.yml
- Runner selection: HIGH for all except macos-15-intel (MEDIUM - confirmed via GitHub issue but not tested in this project)

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable domain; GitHub runner deprecations are the main moving target)
