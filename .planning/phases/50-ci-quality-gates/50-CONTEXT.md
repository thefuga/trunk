# Phase 50: CI Quality Gates - Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Every push and PR is automatically validated for Rust correctness, frontend type safety, test coverage, and code formatting/linting. This phase sets up GitHub Actions CI with Biome (replacing Prettier from original requirements) for frontend formatting and linting.

</domain>

<decisions>
## Implementation Decisions

### Workflow Structure
- **D-01:** Single `ci.yml` workflow with parallel jobs (rust, frontend, biome)
- **D-02:** Two-gate pipeline: Gate 1 runs fast checks (biome check, cargo fmt --check, svelte-check) in parallel; Gate 2 (cargo check + clippy, cargo test, vitest) only starts after Gate 1 passes
- **D-03:** Rationale for gating: fast checks (~5-10s) catch trivial issues before spending time on heavier tests

### Formatter/Linter
- **D-04:** Biome replaces Prettier (CI-03, CI-04 requirements updated). No Prettier in this project.
- **D-05:** Biome runs both formatting and linting (not format-only)
- **D-06:** Minimal/default Biome config — no custom rules, no .biome config file if possible, or bare minimum config
- **D-07:** Single format-all commit before CI enforcement: `style: format codebase with biome`

### CI Failure Strategy
- **D-08:** No fail-fast — all jobs run to completion even if one fails (shows all problems at once)
- **D-09:** Clippy runs with `-D warnings` (all warnings are errors)

### Runner & Caching
- **D-10:** All CI jobs run on `ubuntu-latest` (no multi-OS matrix; cross-platform is Phase 51's concern)
- **D-11:** Rust compilation cached with `swatinem/rust-cache` (CI-05 requirement)
- **D-12:** Bun dependencies cached between runs (`~/.bun/install/cache`)

### Tooling Decisions
- **D-13:** Keep current tools: cargo test (not nextest), svelte-check (not svelte-fast-check), vitest (not bun test)
- **D-14:** No sccache, no mold linker — keep CI simple for now

### Claude's Discretion
- Biome job placement (separate job vs inside frontend job)
- GitHub Actions versions for setup-bun, rust toolchain, etc.
- Concurrency controls (cancel in-progress on same branch)
- Branch filter specifics

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — CI-01 through CI-05 define success criteria (note: CI-03 and CI-04 now use Biome instead of Prettier)

### Tauri CI Patterns
- No external specs — Tauri does not prescribe CI configuration. Their own repo uses Prettier + cargo fmt + taplo as parallel jobs, but we diverge by using Biome instead of Prettier.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `package.json` scripts: `bun run check` (svelte-check), `bun run test` (vitest run) — CI can invoke these directly
- 14 vitest test files in `src/lib/` — existing test suite for frontend
- 14 Rust source files with `#[cfg(test)]` blocks — existing test suite for backend

### Established Patterns
- `bun` as package manager (not npm/pnpm) — CI must use `setup-bun` action
- Rust crate at `src-tauri/` — cargo commands need `--manifest-path src-tauri/Cargo.toml` or `cd src-tauri`
- No `.github/` directory exists yet — CI is greenfield

### Integration Points
- Biome needs to be added as devDependency in `package.json`
- A `biome.json` may be needed at project root (or rely on defaults if biome supports no-config)
- GitHub Actions workflow file at `.github/workflows/ci.yml`

</code_context>

<specifics>
## Specific Ideas

- User explicitly rejected Prettier — "I don't even want prettier NEAR this project"
- User preference for fast-first CI: run cheap checks (format, lint, types) before expensive ones (compile, test)
- User evaluated cargo-nextest, sccache, mold, svelte-fast-check, bun test — deliberately chose to keep current tools for simplicity

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 50-ci-quality-gates*
*Context gathered: 2026-03-25*
