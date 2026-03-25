# Phase 50: CI Quality Gates - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-25
**Phase:** 50-ci-quality-gates
**Areas discussed:** Workflow structure, CI failure strategy, Runner & caching, Biome config

---

## Workflow Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Single workflow | One ci.yml with parallel jobs (rust, frontend, format). Simpler to maintain, one status check per PR. | ✓ |
| Two workflows | Separate ci-rust.yml and ci-frontend.yml. Independent triggers, two status checks. | |

**User's choice:** Single workflow
**Notes:** None

### Formatter Tool

| Option | Description | Selected |
|--------|-------------|----------|
| Biome | Mature, fast, Rust-based. Handles formatting + linting. Used by tauri-action. | ✓ |
| oxlint (OXC) | Very fast Rust-based linter. Formatting support newer/experimental. | |
| Neither | No JS/TS formatter in CI. | |

**User's choice:** Biome
**Notes:** User explicitly rejected Prettier: "I don't even want prettier NEAR this project. If we want to add a javascript formatter, use either biome or oxc"

### Biome Job Placement

**User's choice:** "You decide" — Claude has discretion
**Notes:** None

### Biome Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Format + lint | Enable both formatter and linter. One tool, one config. | ✓ |
| Format only | Biome purely as formatter. No lint rules. | |

**User's choice:** Format + lint
**Notes:** None

### Tooling Research

User requested research into faster alternatives to current tooling. Research covered:
- cargo-nextest (~2-3x faster than cargo test)
- sccache (15-35% faster Rust compilation)
- mold linker (2-5x faster linking on Linux)
- svelte-fast-check (10-24x faster than svelte-check)
- bun test (3-10x faster than vitest)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep current tools | Stick with cargo test, svelte-check, vitest. Keep it simple. | ✓ |
| cargo-nextest | Drop-in replacement for cargo test | |
| sccache | Compilation cache for Rust | |
| mold linker | Faster linker for Linux CI | |
| svelte-fast-check | Replace svelte-check with tsgo-based alternative | |
| bun test | Replace vitest for unit tests | |

**User's choice:** Keep current tools (both Rust and JS sides)
**Notes:** Evaluated all options, chose simplicity

---

## CI Failure Strategy

### Fail Mode

| Option | Description | Selected |
|--------|-------------|----------|
| Fail-fast | Cancel other jobs when one fails. Saves CI minutes. | |
| Run all checks | Always run all jobs to completion. Shows ALL problems at once. | ✓ |

**User's choice:** Run all checks
**Notes:** None

### Clippy Strictness

| Option | Description | Selected |
|--------|-------------|----------|
| -D warnings | All clippy warnings are errors. Clean codebase. | ✓ |
| -W warnings | Warnings shown but don't fail CI. | |

**User's choice:** -D warnings
**Notes:** None

### Job Ordering

| Option | Description | Selected |
|--------|-------------|----------|
| All parallel | Rust, frontend, biome start simultaneously. | |
| Format first, then rest | Two-gate pipeline: fast checks first, then heavy tests. | ✓ |

**User's choice:** Two-gate pipeline
**Notes:** "Let's check formatting and linting and type checking and then we run everything else. The idea behind there is we run this three easy and fast jobs. They should take only five to ten seconds and then we're good with moving on to these lower tests and more involved tests."

---

## Runner & Caching

### Runner OS

| Option | Description | Selected |
|--------|-------------|----------|
| ubuntu-latest | Cheapest, fastest spin-up. Release pipeline handles cross-platform. | ✓ |
| Multi-OS matrix | Run checks on ubuntu + macos + windows. 3x CI cost. | |

**User's choice:** ubuntu-latest
**Notes:** None

### Rust Cache

| Option | Description | Selected |
|--------|-------------|----------|
| swatinem/rust-cache | Purpose-built for Rust CI. Handles cache invalidation well. | ✓ |
| actions/cache manually | Generic cache action with manual config. | |

**User's choice:** swatinem/rust-cache
**Notes:** None

### Bun Cache

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, cache bun install | Cache ~/.bun/install/cache between runs. | ✓ |
| No caching | Run bun install fresh each time. | |

**User's choice:** Yes, cache bun install
**Notes:** None

---

## Biome Config

### Formatting Style

| Option | Description | Selected |
|--------|-------------|----------|
| Biome defaults | Tabs, no semicolons. Biome's opinionated defaults. | ✓ |
| Custom config | Any custom rules or overrides. | |

**User's choice:** Biome defaults — minimal/no config
**Notes:** "Leave the defaults in the file. Don't try to add your own rules or anything. Just use the most possible minimal config on Biome. If you can leave it just a blank config. Or even better, no config at all."

### Initial Format Pass

| Option | Description | Selected |
|--------|-------------|----------|
| Single format commit | One commit: 'style: format codebase with biome'. Easy to git blame --ignore-rev. | ✓ |
| Gradual formatting | Only enforce on new/changed files. | |

**User's choice:** Single format commit
**Notes:** None

---

## Claude's Discretion

- Biome job placement (separate job vs inside frontend job)
- GitHub Actions versions for setup-bun, rust toolchain, etc.
- Concurrency controls
- Branch filter specifics

## Deferred Ideas

None — discussion stayed within phase scope
