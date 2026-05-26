---
phase: 70
slug: excerpt-resolution-markdown-render
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-26
---

# Phase 70 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` + Vitest (Svelte) |
| **Config file** | `src-tauri/Cargo.toml`, `vitest.config.ts` |
| **Quick run command** | `just check` (fmt + biome + svelte-check + clippy + cargo-test + vitest) |
| **Full suite command** | `just check` |
| **Estimated runtime** | ~60s |

---

## Sampling Rate

- **After every task commit:** Run `just check` (the project rule from CLAUDE.md)
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~60s

---

## Per-Task Verification Map

*Populated by planner from PLAN.md tasks; one row per task. See RESEARCH.md `## Validation Architecture` for the test surface inventory.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD     | TBD  | TBD  | DOC-01..04  | —          | N/A             | unit / component | `just check` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/git/review.rs` — new module with `#[cfg(test)] mod tests` and golden-test fixture helpers
- [ ] Reuse `commit_with_file` / `make_file_repo` patterns from `src-tauri/src/commands/review.rs:2026-2047`
- [ ] `src/components/ReviewDocPreview.test.ts` (or inline `*.test.ts` next to component) — Vitest + `@testing-library/svelte` (both already in use)

*If the planner inlines preview into ReviewPanel.svelte instead of a new component, the component test target shifts to `src/components/ReviewPanel.test.ts`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Generate button disabled tooltip ("Add at least one comment to generate") visible on hover | D-01 | Tooltip hover state — covered by component test but final visual check belongs to UAT | Hover Generate with empty session, confirm tooltip text |
| Preview view legibility (theme variables, monospace font, scroll behavior on long docs) | D-02 | Visual / theme integration | Open preview against a session with ≥20 comments; verify scroll, mono font, theme tokens |
| AI agent consumability of generated markdown | DOC-01..04 | LLM-recipient quality is judgmental | Paste rendered doc into Claude / Cursor; confirm it understands structure and acts on it |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
