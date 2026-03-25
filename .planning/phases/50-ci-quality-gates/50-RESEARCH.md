# Phase 50: CI Quality Gates - Research

**Researched:** 2026-03-25
**Domain:** GitHub Actions CI pipeline for Tauri 2 (Rust + Svelte 5) with Biome formatting/linting
**Confidence:** HIGH

## Summary

This phase creates a GitHub Actions CI workflow from scratch (no `.github/` directory exists) for a Tauri 2 desktop app with a Rust backend and Svelte 5 frontend. The workflow uses a two-gate architecture: fast checks (formatting, linting, type checking) run first, and heavier compilation/test jobs only start if Gate 1 passes.

The codebase currently fails several of the checks CI will enforce: `cargo fmt --check` has 251 formatting diffs across 15 files, `cargo clippy -D warnings` has 29 errors (mostly `useless conversion` and `map_or` simplifications), and `bun run check` (svelte-check) has 127 type errors concentrated in vendored virtual-list JS files. These must all be fixed before CI enforcement or CI will never pass. `cargo test` (148 tests) and `bun run test` (170 tests, vitest) both pass cleanly.

**Primary recommendation:** Create the CI workflow AND fix all pre-existing check failures in coordinated commits: format-all commits for Rust and frontend (Biome), clippy fix commit, and svelte-check fix (exclude vendored JS from type checking). Without these fixes, CI will fail on its first run.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Single `ci.yml` workflow with parallel jobs (rust, frontend, biome)
- **D-02:** Two-gate pipeline: Gate 1 runs fast checks (biome check, cargo fmt --check, svelte-check) in parallel; Gate 2 (cargo check + clippy, cargo test, vitest) only starts after Gate 1 passes
- **D-03:** Rationale for gating: fast checks (~5-10s) catch trivial issues before spending time on heavier tests
- **D-04:** Biome replaces Prettier (CI-03, CI-04 requirements updated). No Prettier in this project.
- **D-05:** Biome runs both formatting and linting (not format-only)
- **D-06:** Minimal/default Biome config -- no custom rules, no .biome config file if possible, or bare minimum config
- **D-07:** Single format-all commit before CI enforcement: `style: format codebase with biome`
- **D-08:** No fail-fast -- all jobs run to completion even if one fails (shows all problems at once)
- **D-09:** Clippy runs with `-D warnings` (all warnings are errors)
- **D-10:** All CI jobs run on `ubuntu-latest` (no multi-OS matrix; cross-platform is Phase 51's concern)
- **D-11:** Rust compilation cached with `swatinem/rust-cache` (CI-05 requirement)
- **D-12:** Bun dependencies cached between runs (`~/.bun/install/cache`)
- **D-13:** Keep current tools: cargo test (not nextest), svelte-check (not svelte-fast-check), vitest (not bun test)
- **D-14:** No sccache, no mold linker -- keep CI simple for now

### Claude's Discretion
- Biome job placement (separate job vs inside frontend job)
- GitHub Actions versions for setup-bun, rust toolchain, etc.
- Concurrency controls (cancel in-progress on same branch)
- Branch filter specifics

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CI-01 | CI runs cargo check, clippy (-D warnings), cargo test, cargo fmt --check on every push/PR | Gate 1 (fmt) + Gate 2 (check/clippy/test); needs Tauri system deps for compilation; 29 clippy errors and 15 files with fmt issues must be fixed first |
| CI-02 | CI runs bun install, svelte-check, vitest on every push/PR | Gate 1 (svelte-check) + Gate 2 (vitest); 127 svelte-check errors from vendored JS must be resolved (exclude from tsconfig) |
| CI-03 | CI runs formatting/linting check on all frontend files (updated: Biome, not Prettier) | `biome ci .` in Gate 1; Biome v2.4.9 supports .ts/.svelte; needs `biome.json` with Svelte overrides |
| CI-04 | Biome configured as devDependency with config, existing code pre-formatted (updated: Biome, not Prettier) | `@biomejs/biome` devDependency + `biome.json` + format-all commit |
| CI-05 | Rust builds cached with swatinem/rust-cache | `Swatinem/rust-cache@v2` with `workspaces: "src-tauri -> target"` |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Never inline colors -- use CSS custom properties from theme
- Never fight layout with positioning hacks -- use grid/flexbox
- All git operations through git2 crate, no shelling out (except GIT_EDITOR)
- Package manager: `bun` (not npm/pnpm)
- Test commands: `bun run test` (vitest), `bun run check` (svelte-check)

## Standard Stack

### Core CI Actions
| Action | Version | Purpose | Why Standard |
|--------|---------|---------|--------------|
| `actions/checkout` | v6 | Check out repository | Official GitHub action, latest stable |
| `dtolnay/rust-toolchain` | stable | Install Rust toolchain + components | De facto standard, replaces deprecated actions-rs |
| `Swatinem/rust-cache` | v2 | Cache Rust compilation artifacts | Most-used Rust caching action; v2.9.1 latest |
| `oven-sh/setup-bun` | v2 | Install Bun runtime | Official Bun GitHub action |
| `biomejs/setup-biome` | v2 | Install Biome CLI | Official Biome GitHub action; auto-detects version from package.json |

### Dev Dependencies to Add
| Package | Version | Purpose | Why |
|---------|---------|---------|-----|
| `@biomejs/biome` | 2.4.9 | Formatting + linting for frontend | Replaces Prettier per user decision; latest stable |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `biomejs/setup-biome` | `bunx @biomejs/biome ci .` | setup-biome is cleaner, auto-detects version, provides caching |
| `dtolnay/rust-toolchain` | `actions-rust-lang/setup-rust-toolchain` | dtolnay is simpler and more widely adopted |
| `actions/checkout@v6` | `actions/checkout@v4` | v6 is current; v4 still works but no reason to use older |

**Installation (local dev):**
```bash
bun add -d @biomejs/biome
```

**Version verification (2026-03-25):**
- `@biomejs/biome`: 2.4.9 (verified via `npm view`)
- `Swatinem/rust-cache`: v2.9.1 (verified via GitHub releases page)
- `oven-sh/setup-bun`: v2 (verified via GitHub)
- `biomejs/setup-biome`: v2 (verified via GitHub + official docs)
- `actions/checkout`: v6.0.2 (verified via GitHub releases)

## Architecture Patterns

### CI Workflow Structure (Two-Gate Pipeline)

```yaml
# .github/workflows/ci.yml
#
# Gate 1 (fast, ~10-30s): biome, cargo-fmt, svelte-check
#   - No system deps needed for biome or cargo-fmt
#   - svelte-check needs bun + node_modules
#
# Gate 2 (heavy, ~2-5min): cargo-check-clippy, cargo-test, vitest
#   - Needs Tauri system deps (libwebkit2gtk-4.1-dev, etc.)
#   - Only runs if Gate 1 passes
#
# All jobs: ubuntu-latest, no fail-fast
```

### Recommended File Layout
```
.github/
  workflows/
    ci.yml              # Single workflow file
biome.json              # Biome configuration (project root)
src-tauri/
  Cargo.toml            # Rust workspace (unchanged)
  Cargo.lock            # Committed to git (unchanged)
package.json            # + @biomejs/biome devDependency
bun.lock                # Updated after biome install
```

### Pattern 1: Two-Gate Pipeline with Job Dependencies
**What:** Fast checks gate slow checks using `needs:` in GitHub Actions
**When to use:** When you have checks with vastly different runtimes
**Example:**
```yaml
jobs:
  # ── Gate 1: Fast checks ──
  biome:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: biomejs/setup-biome@v2
      - run: biome ci .

  cargo-fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --manifest-path src-tauri/Cargo.toml --check

  svelte-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: oven-sh/setup-bun@v2
      - run: bun install --frozen-lockfile
      - run: bun run check

  # ── Gate 2: Heavy checks (only if Gate 1 passes) ──
  cargo-clippy:
    needs: [biome, cargo-fmt, svelte-check]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - # install system deps
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri -> target"
      - run: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

  cargo-test:
    needs: [biome, cargo-fmt, svelte-check]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - # install system deps
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: "src-tauri -> target"
      - run: cargo test --manifest-path src-tauri/Cargo.toml

  vitest:
    needs: [biome, cargo-fmt, svelte-check]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: oven-sh/setup-bun@v2
      - run: bun install --frozen-lockfile
      - run: bun run test
```

### Pattern 2: Concurrency Controls
**What:** Cancel in-progress CI runs when a new push arrives on the same branch
**When to use:** Always for PR workflows -- saves runner minutes
**Example:**
```yaml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```

### Pattern 3: System Dependencies for Tauri on Ubuntu
**What:** Tauri 2 requires system-level C libraries for compilation
**When to use:** Any job that runs `cargo check`, `cargo clippy`, or `cargo build` for a Tauri project
**Example:**
```yaml
- name: Install system dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libwebkit2gtk-4.1-dev \
      build-essential \
      curl \
      wget \
      file \
      libxdo-dev \
      libssl-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev
```
**Note:** `cargo fmt --check` does NOT need these deps (syntax-only parsing). `cargo clippy` and `cargo check` DO need them (they compile the code).

### Pattern 4: Biome CI Command
**What:** `biome ci` is the read-only, CI-optimized version of `biome check`
**When to use:** Always in CI -- it provides GitHub annotations, never modifies files, uses `--changed` with VCS integration
**Key difference from `biome check`:** Read-only (no `--write`), GitHub-native error annotations, optimized for CI runners

### Anti-Patterns to Avoid
- **Running `cargo check` AND `cargo clippy` separately:** Clippy is a superset of check. Running both wastes time. Use clippy alone (which includes check).
- **Using `actions-rs/*`:** Deprecated/unmaintained. Use `dtolnay/rust-toolchain` instead.
- **Using `npm`/`pnpm` for bun projects:** This project uses `bun`. Always use `bun install --frozen-lockfile`.
- **Caching `node_modules` manually:** `oven-sh/setup-bun@v2` handles bun cache automatically when lockfile is present.
- **Running `biome check --write` in CI:** Use `biome ci` which is read-only and CI-optimized.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Rust caching | Custom `actions/cache` with cargo paths | `Swatinem/rust-cache@v2` | Handles cache key generation, selective caching, workspace support |
| Bun setup + caching | Manual bun install + cache | `oven-sh/setup-bun@v2` | Handles version resolution, caching, PATH setup |
| Biome setup | `bunx @biomejs/biome` | `biomejs/setup-biome@v2` | Auto-detects version from package.json, faster than bunx |
| Rust toolchain install | `rustup` commands | `dtolnay/rust-toolchain@stable` | Handles component installation, caching hints |
| Concurrency control | Workflow-level logic | GitHub Actions `concurrency:` key | Built-in, reliable, no custom code |

**Key insight:** Every major tool in this stack has an official GitHub Action. Using them instead of manual setup scripts saves time, provides caching, and follows community standards.

## Common Pitfalls

### Pitfall 1: Tauri System Dependencies Missing
**What goes wrong:** `cargo check`/`cargo clippy` fail with `pkg-config` errors about missing webkit2gtk or gtk
**Why it happens:** Tauri 2 depends on system C libraries (webkit2gtk-4.1, gtk3, appindicator) that aren't on ubuntu-latest by default
**How to avoid:** Install system deps via `apt-get` BEFORE any cargo compilation step
**Warning signs:** Errors mentioning `pkg-config`, `Could not find`, `webkit2gtk-4.1`

### Pitfall 2: Codebase Fails Checks Before CI Is Added
**What goes wrong:** CI workflow is created but every single run fails because existing code doesn't pass checks
**Why it happens:** The codebase was developed without enforcing fmt/clippy/biome -- there's accumulated drift
**How to avoid:** Fix ALL pre-existing issues in dedicated commits BEFORE enabling CI:
  - `cargo fmt` (15 files need formatting)
  - `cargo clippy` fixes (29 errors: useless `.into()` calls, `map_or` -> `is_some_and`, etc.)
  - `biome format --write` (initial format of all frontend files)
  - `svelte-check` fixes (exclude vendored JS files from type checking)
**Warning signs:** CI fails on the very first push

### Pitfall 3: svelte-check Fails Due to Vendored JS Files
**What goes wrong:** `bun run check` exits with 127 errors, almost all from `src/components/virtual-list/`
**Why it happens:** `tsconfig.json` has `"checkJs": true` and `"allowJs": true`, which type-checks vendored JS files that have no type annotations
**How to avoid:** Exclude `src/components/virtual-list/**/*.js` from tsconfig's `include` or add to `exclude`. The 1 remaining error (`RefType` in CommitGraph.svelte) must be fixed separately.
**Warning signs:** svelte-check shows 100+ errors all in one directory

### Pitfall 4: cargo fmt Needs --manifest-path for Subdirectory Crates
**What goes wrong:** `cargo fmt --check` runs in the repo root and finds nothing
**Why it happens:** The Rust crate is in `src-tauri/`, not the repo root
**How to avoid:** Use `cargo fmt --manifest-path src-tauri/Cargo.toml --check` or `cd src-tauri && cargo fmt --check`
**Warning signs:** cargo fmt passes but Rust code is clearly unformatted

### Pitfall 5: clippy Superset Duplication
**What goes wrong:** CI runs both `cargo check` and `cargo clippy`, wasting 2-3 minutes of compilation
**Why it happens:** Not understanding that clippy already runs check internally
**How to avoid:** Run only `cargo clippy` in a single job. It covers both correctness checking and lint warnings.
**Warning signs:** Two separate compilation steps with nearly identical output

### Pitfall 6: Bun Lockfile Mismatch
**What goes wrong:** `bun install --frozen-lockfile` fails because `bun.lock` is stale
**Why it happens:** Adding `@biomejs/biome` updates `bun.lock`; if not committed, CI fails
**How to avoid:** Always commit `bun.lock` after adding dependencies
**Warning signs:** `bun install` error about frozen lockfile

### Pitfall 7: Biome Svelte False Positives
**What goes wrong:** Biome reports false-positive lint errors in `.svelte` files
**Why it happens:** Svelte support is experimental in Biome v2.4; control-flow syntax (`{#if}`) not parsed
**How to avoid:** Add Svelte-specific rule overrides in `biome.json` to disable rules that produce false positives
**Warning signs:** Lint errors about unused variables that are actually reactive `$:` bindings, or `useConst`/`useImportType` errors in Svelte script blocks

### Pitfall 8: rust-cache Workspace Path
**What goes wrong:** rust-cache doesn't find or cache the correct target directory
**Why it happens:** Cargo workspace is in `src-tauri/` subdirectory, not repo root
**How to avoid:** Set `workspaces: "src-tauri -> target"` in rust-cache config
**Warning signs:** Cache restore/save messages show wrong paths, or cache misses every run

## Code Examples

### Biome Configuration (biome.json)
```json
{
  "$schema": "https://biomejs.dev/schemas/2.4.9/schema.json",
  "organizeImports": {
    "enabled": true
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true
    }
  },
  "formatter": {
    "enabled": true,
    "indentStyle": "tab",
    "lineWidth": 100
  },
  "overrides": [
    {
      "includes": ["**/*.svelte"],
      "linter": {
        "rules": {
          "style": {
            "useConst": "off",
            "useImportType": "off"
          },
          "correctness": {
            "noUnusedVariables": "off",
            "noUnusedImports": "off"
          }
        }
      }
    }
  ]
}
```
Source: [Biome language support docs](https://biomejs.dev/internals/language-support/) + [Biome configuration reference](https://biomejs.dev/reference/configuration/)

**Note on D-06 (minimal config):** Biome requires a `biome.json` even if minimal. The Svelte overrides above are recommended by official docs to avoid false positives. The formatter settings (indentStyle, lineWidth) should match the project's existing conventions.

### Concurrency Configuration
```yaml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```
Source: [GitHub Actions concurrency docs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency)

### rust-cache with Tauri Subdirectory
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    workspaces: "src-tauri -> target"
    save-if: ${{ github.ref == 'refs/heads/main' }}
```
Source: [Swatinem/rust-cache README](https://github.com/Swatinem/rust-cache)

**Note:** `save-if` restricted to main branch prevents PR branches from polluting the cache. All branches still restore from the main branch cache.

### Biome CI Command
```bash
# In CI (read-only, GitHub annotations):
biome ci .

# Locally (with auto-fix):
biome check --write .
```
Source: [Biome CI recipes](https://biomejs.dev/recipes/continuous-integration/)

## Codebase Health Audit

Current state of checks that CI will enforce:

| Check | Status | Issues | Fix Effort |
|-------|--------|--------|------------|
| `cargo fmt --check` | FAILS | 251 diffs across 15 files | Low -- `cargo fmt` auto-fixes |
| `cargo clippy -D warnings` | FAILS | 29 errors (useless `.into()`, `map_or`, `last()`) | Low -- mechanical fixes |
| `cargo test` | PASSES | 148 tests, 0 failures | None |
| `bun run check` (svelte-check) | FAILS | 127 errors (126 in vendored virtual-list JS, 1 in CommitGraph.svelte) | Low -- exclude vendored JS from tsconfig |
| `bun run test` (vitest) | PASSES | 170 tests, 0 failures | None |
| `biome ci .` | NOT YET INSTALLED | N/A -- Biome not in project yet | Medium -- install + format-all commit |

**Critical path:** All failing checks must be fixed BEFORE the CI workflow is created, or CI will fail on its first run.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `actions-rs/toolchain` | `dtolnay/rust-toolchain` | 2023 | actions-rs unmaintained; dtolnay is standard |
| `actions/checkout@v4` | `actions/checkout@v6` | 2025 | v6 uses Node 24, v4 still works |
| Prettier for formatting | Biome for format+lint | 2024-2025 | Single tool replaces Prettier+ESLint; much faster |
| `biome check` in CI | `biome ci` in CI | Biome v2+ | Read-only, GitHub annotations, CI-optimized |
| Manual `actions/cache` for Rust | `Swatinem/rust-cache` | 2022+ | Smart key generation, workspace support |

**Deprecated/outdated:**
- `actions-rs/*`: Unmaintained since 2022. Use `dtolnay/rust-toolchain`.
- Prettier in this project: User explicitly rejected. Use Biome.
- `bun.lockb`: Binary lockfile format replaced by text-based `bun.lock` in Bun 1.2+.

## Open Questions

1. **Biome `indentStyle` and `lineWidth` -- what does the project currently use?**
   - What we know: Vite config uses 2-space indentation. TypeScript files appear to use 2-space. Biome defaults to tabs.
   - What's unclear: Whether the project should adopt Biome's tab defaults or configure spaces to match existing code
   - Recommendation: Check the existing indentation convention and configure Biome to match. If adopting tabs, the format-all commit will reformat everything. If keeping spaces, set `"indentStyle": "space"` in biome.json. **This is Claude's discretion per D-06.**

2. **Svelte control-flow formatting with Biome**
   - What we know: Biome's Svelte support is experimental and doesn't parse `{#if}` / `{#each}` / `{:else}` blocks
   - What's unclear: Whether formatting will break Svelte template readability
   - Recommendation: Run `biome format --write .` locally first, review the `.svelte` file changes, and adjust config if needed. LOW risk -- Biome formats the JS/TS in `<script>` blocks, not the template HTML.

3. **Should `cargo check` run as a separate job or be subsumed by `cargo clippy`?**
   - What we know: clippy is a superset of check. Running both is redundant.
   - What's unclear: The requirements (CI-01) list both `cargo check` and `clippy`
   - Recommendation: Run only `cargo clippy` (which includes check). This satisfies CI-01 while avoiding duplicate compilation. Note this in the plan.

## Environment Availability

> This section covers CI runner environment (ubuntu-latest), not local dev.

| Dependency | Required By | Available on ubuntu-latest | Version | Fallback |
|------------|------------|---------------------------|---------|----------|
| Rust toolchain | Gate 1 (fmt) + Gate 2 (clippy/test) | Via `dtolnay/rust-toolchain` | stable | -- |
| Bun | Gate 1 (svelte-check) + Gate 2 (vitest) | Via `oven-sh/setup-bun` | latest | -- |
| Biome CLI | Gate 1 (biome ci) | Via `biomejs/setup-biome` | From package.json | -- |
| libwebkit2gtk-4.1-dev | Gate 2 (cargo clippy/test) | Needs `apt-get install` | System | -- |
| libayatana-appindicator3-dev | Gate 2 (cargo clippy/test) | Needs `apt-get install` | System | -- |
| libssl-dev | Gate 2 (cargo clippy/test) | Needs `apt-get install` | System | -- |
| build-essential | Gate 2 (cargo clippy/test) | Pre-installed on ubuntu-latest | System | -- |
| pkg-config | Gate 2 (Rust sys crates) | Pre-installed on ubuntu-latest | System | -- |

**Missing dependencies with no fallback:**
- System libraries for Tauri (webkit, gtk, appindicator) must be installed via `apt-get` in Gate 2 jobs

**Missing dependencies with fallback:**
- None -- all tools have official GitHub Actions for installation

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Frontend framework | vitest 4.1.0 (configured in vite.config.ts) |
| Backend framework | cargo test (built-in) |
| Frontend config | `vite.config.ts` (test section) |
| Backend config | `#[cfg(test)]` modules in Rust source |
| Quick run (frontend) | `bun run test` |
| Quick run (backend) | `cd src-tauri && cargo test` |
| Lint check (Rust) | `cd src-tauri && cargo clippy -- -D warnings` |
| Format check (Rust) | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` |
| Format+lint (frontend) | `biome ci .` (after installation) |
| Type check (frontend) | `bun run check` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CI-01 | Rust checks run on push/PR | smoke (CI itself) | Push a commit and verify workflow runs | N/A -- CI workflow |
| CI-02 | Frontend checks run on push/PR | smoke (CI itself) | Push a commit and verify workflow runs | N/A -- CI workflow |
| CI-03 | Biome check runs on push/PR | smoke (CI itself) | `biome ci .` locally | Wave 0: install biome first |
| CI-04 | Biome configured, code pre-formatted | manual verification | `biome ci .` returns exit 0 locally | Wave 0: install + format |
| CI-05 | Rust caching active | manual verification | Check CI logs for cache hit/miss | N/A -- verify in Actions UI |

### Sampling Rate
- **Per task commit:** Run the specific check locally (e.g., `cargo clippy -- -D warnings`, `biome ci .`)
- **Per wave merge:** Push to branch, verify CI workflow triggers and passes
- **Phase gate:** All CI jobs green on a push to any branch

### Wave 0 Gaps
- [ ] Install `@biomejs/biome` as devDependency
- [ ] Create `biome.json` configuration
- [ ] Fix `cargo fmt` issues (run `cargo fmt`)
- [ ] Fix `cargo clippy` warnings (29 mechanical fixes)
- [ ] Fix `svelte-check` failures (exclude vendored JS, fix 1 CommitGraph error)
- [ ] Run `biome format --write .` to format all frontend files

## Sources

### Primary (HIGH confidence)
- [Biome language support](https://biomejs.dev/internals/language-support/) - Svelte experimental status, recommended overrides
- [Biome configuration reference](https://biomejs.dev/reference/configuration/) - biome.json schema, formatter/linter options
- [Biome CI recipes](https://biomejs.dev/recipes/continuous-integration/) - GitHub Actions integration, `biome ci` command
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) - v2.9.1, workspace config, save-if option
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) - Ubuntu system dependencies
- [Tauri 2 GitHub pipelines](https://v2.tauri.app/distribute/pipelines/github/) - Official CI/CD guidance
- [GitHub Actions concurrency docs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency) - Concurrency groups

### Secondary (MEDIUM confidence)
- [biomejs/setup-biome](https://github.com/biomejs/setup-biome) - v2, auto-detects version from package.json
- [oven-sh/setup-bun](https://github.com/oven-sh/setup-bun) - v2, official action
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) - Stable, component installation
- [actions/checkout releases](https://github.com/actions/checkout/releases) - v6.0.2 latest

### Tertiary (LOW confidence)
- None -- all findings verified with official sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All actions verified via official repos/npm registry
- Architecture: HIGH - Two-gate pattern is standard GitHub Actions; Tauri deps well-documented
- Pitfalls: HIGH - Verified by running actual checks locally; all failures documented with counts
- Codebase health: HIGH - Every check command run locally and output captured

**Research date:** 2026-03-25
**Valid until:** 2026-04-25 (stable domain; GitHub Actions and Biome move slowly)
