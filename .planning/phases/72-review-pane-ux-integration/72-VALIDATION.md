---
phase: 72
slug: review-pane-ux-integration
status: populated
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-27
populated: 2026-05-27
---

# Phase 72 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend) + cargo test (backend, no new tests this phase) |
| **Config file** | `vite.config.ts` / `vitest.config.ts` (project-managed) |
| **Quick run command** | `bun run vitest run src/components/ReviewPanel.test.ts src/components/Toolbar.test.ts src/lib/review-session.svelte.test.ts` |
| **Full suite command** | `just check` |
| **Estimated runtime** | ~25 seconds (component subset) / ~90 seconds (`just check`) |

---

## Sampling Rate

- **After every task commit:** Run `bun run vitest run <touched file>.test.ts`
- **After every plan wave:** Run `bun run vitest run` + `cargo test`
- **Before `/gsd:verify-work`:** `just check` must exit 0
- **Max feedback latency:** ~25 seconds

---

## Per-Task Verification Map

Tasks are listed in execution order (Wave → Plan → Task). Threat refs come from CONTEXT.md `## Threat model` — most are N/A for this refactor (no new IPC/identity/capability); T-72-I is `accept (LOW)` for the error-toast surface (carry-forward from Phase 71).

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 72-01-T1 | 01 | 1 | REQ-72-4b, REQ-72-4c | T-72-S/T/E (all N/A) | RED tests fail before rune is simplified — no behavior change yet, no security impact | unit (RED) | `bun run vitest run src/lib/review-session.svelte.test.ts` | ✅ src/lib/review-session.svelte.test.ts | ⬜ pending |
| 72-01-T2 | 01 | 1 | REQ-72-4b, REQ-72-4c | T-72-T (N/A) | Rune `generate` becomes pure return; no state mutation, no side-effect leakage across clicks | unit (GREEN) | `bun run vitest run src/lib/review-session.svelte.test.ts` | ✅ src/lib/review-session.svelte.ts | ⬜ pending |
| 72-02-T1 | 02 | 1 | REQ-72-1a, REQ-72-2 | T-72-S (N/A: in-process event bus) | RED tests assert emit + active state; aria-pressed exposes toggle state to assistive tech | unit (RED) | `bun run vitest run src/components/Toolbar.test.ts` | ✅ src/components/Toolbar.test.ts | ⬜ pending |
| 72-02-T2 | 02 | 1 | REQ-72-1a, REQ-72-2 | T-72-S (N/A); T-72-D (N/A: user-click-bounded) | Toolbar emits `review-toggle` onto existing in-process bus; no new IPC/capability; class-based active styling (no inline-color anti-pattern) | unit (GREEN) | `bun run vitest run src/components/Toolbar.test.ts` | ✅ src/components/Toolbar.svelte, src/App.svelte | ⬜ pending |
| 72-03-T1 | 03 | 2 | REQ-72-3a, REQ-72-3b, REQ-72-3c, REQ-72-3d, REQ-72-3e | T-72-I (accept LOW: same error-toast surface as Phase 71); T-72-D (mitigated: clearTimeout before setTimeout) | Copy handler uses `instanceof Error` narrowing (never `as`); clipboard write uses pre-granted capability; error toast carries no new data shapes | unit (RED+GREEN atomic — Pitfall 3) | `bun run vitest run src/components/ReviewPanel.test.ts` | ✅ src/components/ReviewPanel.svelte, src/components/ReviewPanel.test.ts | ⬜ pending |
| 72-04-T1 | 04 | 3 | REQ-72-4a | T-72-T (N/A: source deletion only) | `ReviewDocPreview.svelte` + `.test.ts` removed; no dangling imports; full suite still green after deletion | file-absence + smoke | `test ! -f src/components/ReviewDocPreview.svelte && test ! -f src/components/ReviewDocPreview.test.ts && rg -n "ReviewDocPreview" src/ \| (grep -v '^$' \|\| true) \| wc -l \| grep -qE '^\s*0\s*$' && bun run vitest run` | ✅ paths to be deleted | ⬜ pending |
| 72-04-T2 | 04 | 3 | REQ-72-5a, REQ-72-6 | T-72-E (N/A: no new capability) | Blue-strip deletion reduces UI attack surface; accelerator binds to existing menu item (no new handler); CLAUDE.md grid/flexbox discipline preserved | smoke (phase gate) | `just check` | ✅ src/components/RepoView.svelte, src-tauri/src/lib.rs | ⬜ pending |
| 72-04-T3 | 04 | 3 | ~~REQ-72-1b~~ (RETRACTED — see 72-05), REQ-72-1c, REQ-72-5b | (no new threat surface) | Manual confirmation that View menu and DiffPanel close route to the same `review-toggle` / `showPanel()` paths (Cmd+Shift+R retracted in 72-05) | manual UAT (blocking checkpoint) | n/a — checkpoint:human-verify | n/a | n/a — RETRACTED in 72-05 (for REQ-72-1b); other reqs still ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

### Coverage cross-check (Nyquist)

- Every task either runs an `<automated>` verify or sits behind a blocking `checkpoint:human-verify` (72-04-T3).
- No 3 consecutive tasks lack an automated verify (only 72-04-T3 is checkpoint-only, sandwiched between 72-04-T2's `just check` and the SUMMARY write).
- ~~`REQ-72-1b`~~ (RETRACTED — see 72-05), `REQ-72-1c`, `REQ-72-5b` are intentionally manual per VALIDATION.md `## Manual-Only Verifications` (Tauri menu config + interaction).
- `REQ-72-5a` is manual per Wave 0 finding (RepoView.test.ts contains no assertions on the blue-strip; grep verified at planning time).

### Phase requirements → Test Map (reference, from RESEARCH.md §Validation Architecture)

| Req ID | Behavior | Test Type | Automated Command |
|--------|----------|-----------|-------------------|
| REQ-72-1a | Toolbar Review button click emits `review-toggle` | unit | `bun run vitest run src/components/Toolbar.test.ts -t "review-toggle"` |
| ~~REQ-72-1b~~ | ~~Cmd+Shift+R triggers the menu item~~ (RETRACTED — see 72-05) | manual UAT | — |
| REQ-72-1c | View menu "Start/End Code Review" regression | manual UAT | — |
| REQ-72-2 | Toolbar button active styling when `reviewActive === true` | unit | `bun run vitest run src/components/Toolbar.test.ts -t "active state"` |
| REQ-72-3a | Copy click calls `generate` then `writeText` with returned markdown | unit | `bun run vitest run src/components/ReviewPanel.test.ts -t "copy click invokes generate and writeText"` |
| REQ-72-3b | ✓ Copied affordance for 1500ms with re-arm | unit | `bun run vitest run src/components/ReviewPanel.test.ts -t "remains clickable during window"` |
| REQ-72-3c | Error toast on failure with `instanceof Error` narrowing | unit | `bun run vitest run src/components/ReviewPanel.test.ts -t "shows error toast on failure"` |
| REQ-72-3d | Non-`Error` rejection coerced via `String(e)` | unit | `bun run vitest run src/components/ReviewPanel.test.ts -t "coerces non-Error rejection"` |
| REQ-72-3e | Copy button stays in "Copy" on failure | unit | `bun run vitest run src/components/ReviewPanel.test.ts -t "does not flip copied on failure"` |
| REQ-72-4a | `ReviewDocPreview.svelte` deleted | file-absence | `test ! -f src/components/ReviewDocPreview.svelte` |
| REQ-72-4b | `panelMode` / `previewMarkdown` / `showList` / `showPreview` removed | unit | `bun run vitest run src/lib/review-session.svelte.test.ts` |
| REQ-72-4c | `generate(repoPath)` returns markdown string | unit | `bun run vitest run src/lib/review-session.svelte.test.ts -t "generate returns the markdown string"` |
| REQ-72-5a | Blue-button header strip removed from RepoView | manual UAT (Wave 0 finding) | — |
| REQ-72-5b | DiffPanel close returns to ReviewPanel (regression) | manual UAT | — |
| REQ-72-6 | `just check` green | smoke | `just check` |
| G-71-A | Copy lives on comments view (not preview pane) | covered by REQ-72-3a + REQ-72-4a | — |
| G-71-B | Smooth entry/exit + no dead button | covered by REQ-72-1a + REQ-72-2 + REQ-72-5a | — |

---

## Wave 0 Findings (resolved at planning time)

- [x] **Confirmed:** `src/components/RepoView.test.ts` does NOT assert on the deleted header strip. Verified at planning time via `rg -n "Review|toolbar-btn-active|813|815|header strip|blue" src/components/RepoView.test.ts` (returned no matches in test bodies). REQ-72-5a is therefore **manual UAT** (covered in 72-04-T3), not an automated DOM-absence test. No new test needed.
- [x] **Confirmed:** No framework install required. `bun run vitest`, `bunx svelte-check`, `cargo test`, and `just check` are all wired and functional in the existing toolchain.
- [x] **Confirmed:** No new fixtures required. The existing `installReads` dispatcher in `ReviewPanel.test.ts:92-113` already handles `generate_review_doc` (returns the seeded `generateDoc` string). Plan 03 uses it verbatim.

---

## Manual-Only Verifications

Bundled into 72-04-T3 (single blocking `checkpoint:human-verify`) for atomic sign-off. The checkpoint covers all manual UAT requirements at once so the user only has to run the dev build once.

| Behavior | Requirement | Why Manual | Test Instructions (full script in 72-04-PLAN.md Task 3) |
|----------|-------------|------------|---------------------------------------------------------|
| ~~Cmd+Shift+R toggles review mode~~ | ~~REQ-72-1b~~ | ~~OS-level menu accelerator binding (Tauri config only)~~ | ~~Press `Cmd+Shift+R` → Toolbar Review button toggles active state.~~ (RETRACTED — see 72-05) |
| View → Start/End Code Review still works | REQ-72-1c | Native macOS menu regression | Open View menu → confirm item present with `⌘⇧R` hint → click → toggles. |
| DiffPanel close returns to ReviewPanel | REQ-72-5b | Surviving wiring (`reviewSession.showPanel()` at RepoView.svelte:~839) is the post-72 sole back-affordance | Enter review → jump from a comment to a diff → close DiffPanel → returns to ReviewPanel. |
| Blue header strip removed | REQ-72-5a | Wave 0 finding: no existing automated coverage; visual confirmation | Inspect RepoView pane during review mode → no blue button above ReviewPanel/DiffPanel. |
| Toolbar Review button visual sanity | (covers Plan 02 visuals beyond unit assertions) | Visual contrast of `var(--color-accent)` fill | Inspect Toolbar → button flat when off, accent-blue with white text when on. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify (only 72-04-T3 is manual-only; sandwiched between Task 2's `just check` and the SUMMARY write)
- [x] Wave 0 covers all MISSING references (RepoView.test.ts grepped at planning time; result documented above)
- [x] No watch-mode flags (all commands use `vitest run`, never `vitest`)
- [x] Feedback latency < 30s (component subset ~25s; full `just check` ~90s only at phase gate)
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved at plan-write time (2026-05-27)
