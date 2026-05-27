---
phase: 74
slug: address-v0-13-tech-debt-66-wr-01-04-70-cr-01-70-wr-01-biome
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-05-27
---

# Phase 74 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (TS) + cargo test (Rust) |
| **Config file** | `vitest.config.ts`, `src-tauri/Cargo.toml` |
| **Quick run command** | `bunx vitest run --bail=1` (TS) / `cargo test -p trunk --lib --quiet` (Rust) |
| **Full suite command** | `just check` |
| **Estimated runtime** | ~45s `just check` full; ~6s per-file vitest; ~15s cargo test |

---

## Sampling Rate

- **After every task commit:** Run the scoped test for the changed file (`bunx vitest run src/components/CommitGraph.test.ts` or `cargo test slice_diff`)
- **After every plan wave:** Run `just check`
- **Before `/gsd:verify-work`:** `just check` must be green AND `bunx biome ci` must show 0 warnings for CommentComposer.svelte
- **Max feedback latency:** 45s

---

## Per-Task Verification Map

> Populated by `/gsd:plan-phase` when PLAN.md files are created. One row per PLAN (the column is `Plan ID`, not per-task — TDD plans 74-01 / 74-04 / 74-06 each contain RED/GREEN/REFACTOR subtasks; this map covers the plan as a whole). Each row maps to a finding from `74-RESEARCH.md` §Scope Summary.

| Plan ID | Plan | Wave | Finding | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|---------|------------|-----------------|-----------|-------------------|-------------|--------|
| 74-01-01 | 01 | 1 | 66/WR-01 + WR-02 (CommitGraph listener + reload error handling) | — | Listener fail-closed when canonicalPath null; non-`no_session` IPC errors surface toast | unit (vitest) | `bunx vitest run src/components/CommitGraph.test.ts` | ❌ W0 (verify CommitGraph.test.ts exists) | ⬜ pending |
| 74-02-01 | 02 | 1 | biome warnings | — | 3× noNonNullAssertion replaced with runtime guard helper; `bunx biome ci` exits 0 with no warnings on CommentComposer.svelte | unit (vitest) + lint | `bunx vitest run src/components/diff/CommentComposer.test.ts && bunx biome ci src/components/diff/CommentComposer.svelte` | ❓ verify CommentComposer.test.ts | ⬜ pending |
| 74-03-01 | 03 | 1 | 70/WR-01 drop-from-scope evidence | — | One-line SUMMARY note + grep evidence (`previewMarkdown` returns 0 matches) | docs-only | `! grep -rn "previewMarkdown\|panelMode\|ReviewDocPreview" src/` | ✅ | ⬜ pending |
| 74-04-01 | 04 | 2 | 66/WR-03 (seed_review_range precheck) | INT-W1 (incidental close) | Returns `no_session` before walking when session absent; canonical-path resolution moved outside spawn_blocking to match siblings | unit (cargo test) | `cargo test -p trunk --lib seed_review_range_requires_session` | ❌ W0 (new test) | ⬜ pending |
| 74-05-01 | 05 | 2 | 66/WR-04 (app.emit failure logging) | — | All 10 call sites of `app.emit("session-changed")` log a warning when emit fails | source assertion (grep) | `! grep -nP "let\\s+_\\s+=\\s+app\\.emit" src-tauri/src/commands/review.rs` | ✅ | ⬜ pending |
| 74-06-01 | 06 | 3 | 70/CR-01 (slice_diff multi-hunk leak) | — | Multi-hunk file: opposing-side lines from non-anchored hunks excluded from excerpt | unit (cargo test) | `cargo test -p trunk --lib slice_diff_multi_hunk_isolates_opposing_side` | ❌ W0 (new test) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

*Threat Ref: 66/WR-01..04, 70/CR-01, 70/WR-01, biome warnings — no security threats; tech debt only.*

---

## Wave 0 Requirements

- [ ] **Verify `src/components/CommitGraph.test.ts` exists** — if absent, scaffold it from `src/components/ReviewPanel.test.ts` (per RESEARCH §1 Assumption A2). Mock targets: `safeInvoke`, `listen` from `@tauri-apps/api/event`.
- [ ] **Verify `src/components/diff/CommentComposer.test.ts` exists** — for the biome fix RED→GREEN cycle.
- [ ] **Confirm pinned `git2` version** — read `src-tauri/Cargo.toml` and verify `Patch::from_diff` API (or RefCell-over-`diff.foreach` fallback per RESEARCH §5 Assumption A1).

*If none of the above is missing: "Existing infrastructure covers all phase tasks."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cross-repo session-changed event arriving in CommitGraph does NOT trigger reload when canonicalPath is null | 66/WR-01 | Two-window Tauri smoke; jsdom can't simulate window-level event isolation | Open two trunk windows on different repos; start a review in window A; observe window B's CommitGraph does NOT spuriously reload |
| Toast surfaces when reloadSession hits non-`no_session` IPC error | 66/WR-02 | IPC failure injection is gated by real Tauri runtime | Force-quit `tauri dev` mid-load; reopen → observe toast wording, not silent empty graph |
| Multi-hunk diff excerpt shows ONLY opposing-side lines from the anchored hunk | 70/CR-01 | End-to-end verification in DiffPanel; cargo test covers the slicer, not the render path | Anchor a comment to line in hunk 2 of a 3-hunk file; generate markdown; assert hunk 1 and hunk 3 opposing-side lines absent from excerpt |
| Biome warnings: `bunx biome ci` exits 0 with no `noNonNullAssertion` warnings touching `CommentComposer.svelte` | biome cleanup | Lint output observation | Run `bunx biome ci src/components/diff/CommentComposer.svelte` and confirm zero `noNonNullAssertion` lines |

*Two TS-side items (WR-01, WR-02) and the CR-01 render-path verification require running app observation; the slicer itself is fully unit-tested. Wave 0 tests cover the automated portion.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (CommitGraph.test.ts, CommentComposer.test.ts, git2 Patch API check)
- [ ] No watch-mode flags
- [ ] Feedback latency < 45s
- [ ] `nyquist_compliant: true` set in frontmatter (post-execution, post-Wave 0)

**Approval:** pending
