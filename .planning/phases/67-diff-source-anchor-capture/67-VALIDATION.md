---
phase: 67
slug: diff-source-anchor-capture
status: ready
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-25
---

# Phase 67 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from 67-RESEARCH.md §"Validation Architecture".

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.x (frontend, jsdom) + Cargo test (backend) |
| **Config file** | `vite.config.ts` (inline `test` block) + Cargo workspace |
| **Quick run command** | `just vitest` (frontend) / `just cargo-test` (backend) |
| **Full suite command** | `just check` (fmt + biome + svelte-check + clippy + cargo-test + vitest) |
| **Estimated runtime** | ~60–120 seconds (`just check`) |

---

## Sampling Rate

- **After every task commit:** Run `just vitest` (or `just cargo-test` for Rust-only tasks)
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** `just check` must be green
- **Max feedback latency:** ~120 seconds

---

## Per-Task Verification Map

> Task IDs are assigned by the planner; this map is keyed by success criterion until plans exist.
> Planner MUST map each row to a concrete `<automated>` verify in a task.

| SC | Requirement | Layer | Secure Behavior | Test Type | Automated Command | Status |
|----|-------------|-------|-----------------|-----------|-------------------|--------|
| SC-1 | ANCH-01 | TS adapter | pure-Add selection → `{ side:"New", start:min, end:max }` | unit | `just vitest` (`diff-anchor.test.ts`) | ⬜ pending |
| SC-1 | ANCH-01 | Rust | `add_comment_inner` pushes `Comment` (anchor+excerpt), clears `draft_comment`, persists | unit | `just cargo-test` | ⬜ pending |
| SC-1 | ANCH-01 | Vitest component | guard-lift: clicking a line in a commit diff updates selection state | component | `just vitest` | ⬜ pending |
| SC-2 | ANCH-01 | Rust | store round-trip: `save_session` w/ non-trivial `Anchor` → `load_session` → identical anchor fields | unit | `just cargo-test` | ⬜ pending |
| SC-2 | ANCH-01 | Type/review | anchor schema has no array-index/options field that could shift (types.rs Anchor) | static | review | ⬜ pending |
| SC-3 | ANCH-01 | TS adapter | mixed Add+Delete → `{ side:"New", start:min(new), end:max(new) }`, Delete dropped from range, `-` lines kept in excerpt | unit | `just vitest` | ⬜ pending |
| SC-3 | ANCH-01 | TS adapter | pure-Delete → `Old`; non-contiguous → `min..max`; `Added` forces `New`; `Renamed` stores new path | unit | `just vitest` | ⬜ pending |
| SC-3 | ANCH-01 | Rust | `add_comment_inner` accepts `Source::FullFile` too (proves L-08 sharing) | unit | `just cargo-test` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

> **SC-2 scope guard:** Phase 70 owns render-time re-resolution (L-06). For Phase 67, SC-2 = persist anchor → reload from disk → assert anchor fields unchanged. The anchor is source coordinates only (no `hunk_index`/`context_lines` field exists that could shift). Do NOT pull Phase 70 re-resolution into this phase.

---

## Wave 0 Requirements

- [ ] `src/lib/diff-anchor.test.ts` — adapter unit tests (SC-1 / SC-3), pure function, no Tauri mock
- [ ] Rust tests for `add_comment_inner` / `save_draft_comment_inner` in `review.rs` `#[cfg(test)]` (SC-1 / SC-2 / SC-3) using existing `tempfile::TempDir` + git fixture pattern
- [ ] Component test additions for guard-lift + inline composer (`DiffPanel.test.ts` or new `CommentComposer.test.ts`)
- [ ] No framework install needed — Vitest + Cargo test already configured

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Attach-then-submit UX feel + "Comments on lines N–M" preview correctness | ANCH-01 | Subjective interaction quality not assertable in jsdom | Select a line range in a commit diff, click Comment, type, submit; confirm preview matches selection and composer clears |
| Anchor metadata intact after whitespace toggle + app restart | ANCH-01 | Full re-resolution display is Phase 70; visual confirm only | Attach comment, toggle ignore-whitespace, restart app, confirm comment metadata persists |
| Merge-commit disabled affordance + tooltip | ANCH-01 | Tooltip hover/disabled-state UX | Open a merge commit's diff, confirm Comment affordance is disabled with explanatory tooltip |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-25
