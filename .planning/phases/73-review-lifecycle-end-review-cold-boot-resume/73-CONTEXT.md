# Phase 73: Review Lifecycle (End-review + cold-boot resume) - Context

**Gathered:** 2026-05-27
**Status:** Ready for planning

<domain>
## Phase Boundary

Give the review session both lifecycle endpoints in the UI:

1. **Cold-boot resume.** Comments appear on the first ReviewPanel open after app boot, without requiring any mutation (Bug 3 from `72-VERIFICATION.md`). On-disk session data exists already (Phase 65); the runtime session must be loaded before the panel renders.
2. **Explicit End-review affordance.** The user can end a review session from the panel — terminating the runtime session and deleting the on-disk file — so the review no longer feels implicitly permanent. Supports the stated workflow: write code → review in trunk → Copy markdown → paste into agent → revise → start fresh.

Bundled per the carry-forward todo: shipping resume without End would compound the permanence problem (advisor concern recorded during 72-05 planning).

**In scope:**
- ReviewPanel auto-resumes when status indicates a disk-only session
- Visible End-review button in ReviewPanel header
- Distinct empty-state copy for cold (no session) vs warm-empty (session active, zero comments)
- Session summary line ("N comments · M commits") above the comments list

**Out of scope (deferred per `.planning/todos/pending/phase-73-review-lifecycle.md`):**
- Keyboard shortcut for End-review (REQ-72-1b retracted in 72-05 due to UAT clash with launcher tools; do not re-bind Cmd+Shift+R without UAT)
- Review history / archival of past sessions (Phase 74 candidate if asked)
- Combined "Copy & End" single-button affordance (kept separate to avoid coupling destructive + non-destructive ops)

</domain>

<decisions>
## Implementation Decisions

### Resume mechanics

- **D-01: Resume trigger** — `ReviewPanel.reload()` (the existing function at `src/components/ReviewPanel.svelte:210`) acts on `status.state`. When `state === 'ResumeAvailable'`, call `resume_review_session` before the parallel `list_session_*` reads. The status discriminator is already present in `SessionStatus` (`src-tauri/src/commands/review.rs:43`) and already returned by `get_review_session_status` — no backend schema change.
- **D-02: Toolbar active-state sync (lag)** — Default: ignore the lag — `reviewActive` reflects runtime, not disk; the Toolbar Review button is a toggle, not a status light. If plan-phase considers an eager `resume-on-repo-open` path so the toolbar lights up before the panel is opened, the plan **MUST benchmark the cost first** (per `[[measure-before-assuming]]`). Do not lock in any "this IPC is cheap" assumption without measurement.
- **D-07: LoadOutcome handling on cold-boot resume** — Reuse the existing branching in the `resume_review_session` thin command (`Loaded` / `RecoveredCorrupt` → toast / `RefusedNewer` → warn). Cold-boot resume must not crash the panel on a non-`Loaded` outcome; the toast surface is the existing one.

### End-review affordance

- **D-03: Placement** — ReviewPanel header, next to the existing Copy button. Use a visually distinct icon (proposed: `Trash2` from `@lucide/svelte`, contrasting with `Clipboard` for Copy). No new toolbar entry, no new menu item.
- **D-04: Session summary line** — Small caption above the comments list: "N comments · M commits" (muted text). Uses `comments.length` and `commits.length` from the existing rune state — no new IPC.
- **D-05: Confirmation pattern** — Inline two-step button. First click swaps the button label to "Click again to confirm" with a clear danger color; auto-reverts to "End review" after ~3s if not re-clicked. No modal. Extends the `setTimeout`/`clearTimeout` pattern already used for the 1500ms ✓ Copied affordance in `ReviewPanel.svelte:130-133` (same `clearTimeout` before `setTimeout` discipline carry-forward from Phase 71).
- **D-08: Post-End behavior** — On confirmed End: call `end_review_session` (deletes on-disk file + clears runtime state via the existing `_inner` primitive at `src-tauri/src/commands/review.rs:110`). Panel re-renders into the cold empty state.

### Empty-state copy

- **D-06: Distinct copy, no CTA button**
  - **Cold (`status.state === 'None'` and no runtime session):** "No active review. Toggle review mode in the toolbar to start."
  - **Warm-empty (session active, zero comments):** "Review started. Select diff lines or add a commit note to comment."
  - No inline [Start review] button — would duplicate the Toolbar Review button and add a third start path beyond menu + toolbar + Cmd+Shift+R.
  - Exact wording is a copywriting nit for plan-phase; the structural decision is that the two states have distinct copy.

### Multi-tab live coordination

- **D-09: Reuse existing `session-changed` event** — When tab A ends a review, the existing emission path (already used for comment mutations) triggers tab B's `ReviewPanel.reload()`, which sees `status.state === 'None'` and renders the cold empty state. No new event type. Verify this works as expected during plan-phase (the existing reload may need a small branch to handle the "session disappeared mid-session" case gracefully, e.g., clearing `canonicalPath` or showing a toast).

### Claude's Discretion

- Exact Lucide icons (Trash2 for End / MessageSquareOff/Plus for empty states / etc.) — pick during implementation by visual fit. All proposed icons should be verified present in `@lucide/svelte` (the Phase 72 CONTEXT used the same approach for `MessagesSquare`).
- Exact copy wording for the empty states, the two-step button, and the danger color for the "Click again to confirm" state — structural decision (distinct copy, inline two-step) is locked; word-level polish is plan-phase.
- Auto-revert duration for the two-step button (~3s suggested) — tune for testability and feel.
- Whether to surface a "Review was ended in another window" toast for tab B specifically, or just let the empty state speak for itself.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 73 origin
- `.planning/todos/pending/phase-73-review-lifecycle.md` — Carry-forward bundle: bundling rationale, advisor concern, open design questions, backend primitives inventory, out-of-scope guardrails. This is the phase brief.
- `.planning/phases/72-review-pane-ux-integration/72-VERIFICATION.md` — Source of truth for Bug 3 description and the RETRACTED REQ-72-1b context (do not re-bind Cmd+Shift+R).
- `.planning/phases/72-review-pane-ux-integration/72-CONTEXT.md` — Post-Phase-72 state machine (`reviewActive × rightPaneMode`, no `panelMode`); Copy patterns carried forward into ReviewPanel.

### Phase 65 (lifecycle foundation)
- `.planning/phases/65-data-model-persistence-session-lifecycle/65-CONTEXT.md` — Anchor schema, per-repo JSON store, start/resume/end lifecycle commands; SC #4 ("end and clear … restart shows no session for that repo") is the contract End-review must preserve.

### Backend primitives (already in place; no schema changes expected)
- `src-tauri/src/commands/review.rs:43-47` — `SessionStatus { state, file_exists, canonical_path }`
- `src-tauri/src/commands/review.rs:26-35` — `SessionState { Active | ResumeAvailable | None }`
- `src-tauri/src/commands/review.rs:74-93` — `start_review_session_inner`
- `src-tauri/src/commands/review.rs:99-107` — `resume_review_session_inner` (returns `LoadOutcome`)
- `src-tauri/src/commands/review.rs:110-118` — `end_review_session_inner` (hard-deletes the file)
- `src-tauri/src/commands/review.rs:123-140` — `get_review_session_status_inner` (returns disk-half status)
- `src-tauri/src/commands/review.rs:145-151` — `merge_status` (the disk + in-memory → final state merger)

### Frontend integration points
- `src/components/ReviewPanel.svelte:210-253` — `reload()` function: already calls `get_review_session_status`, currently uses only `canonical_path`. The cold-boot resume goes here.
- `src/components/ReviewPanel.svelte:130-133` — Existing `copied` state + `setTimeout` handle. The two-step End button extends this exact pattern.
- `src/components/ReviewPanel.svelte:347-358` — Existing `session-changed` listener wiring (the basis for D-09 multi-tab coordination).
- `src/lib/review-session.svelte.ts` — `state.reviewActive × state.rightPaneMode` (Phase 72 simplification). D-02's "Toolbar reflects runtime" decision lives or dies here.
- `src/lib/types.ts:338` — TS mirror of `SessionStatus`.

### Style / pattern carry-forward
- Phase 71 patterns: `await writeText`, `instanceof Error` narrowing, `clearTimeout` before `setTimeout`, `vi.useFakeTimers` test pattern — all live in `ReviewPanel.test.ts` post-72.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`get_review_session_status` IPC** — Already returns the discriminator we need (`Active | ResumeAvailable | None`). Cold-boot resume is a frontend wiring change, not a backend command addition.
- **`resume_review_session` IPC** — Already handles `LoadOutcome` variants (`Loaded` / `RecoveredCorrupt` / `RefusedNewer`) and is already called by `DiffPanel` on first line-comment add (which is *why* the bug currently looks like "mutations make comments appear"). Cold-boot resume just calls it from `ReviewPanel.reload()` instead, gated on `status.state === 'ResumeAvailable'`.
- **`end_review_session` IPC** — Already hard-deletes the on-disk file and clears runtime state. End-review wiring is purely a UI affordance + confirmation pattern.
- **`copied` / `setTimeout` pattern** at `ReviewPanel.svelte:130-133` — The two-step End button is the same `clearTimeout` before `setTimeout` discipline applied to a different label transition.
- **`session-changed` event** — Already emitted on session mutations and consumed by `ReviewPanel`'s `$effect` at line 347. Reuse for End-review multi-tab coordination.

### Established Patterns
- **`reload()` swallows `no_session` silently** — A missing session is a normal state, not an error. Cold-boot resume must preserve this — a session that genuinely doesn't exist on disk (cold state) shouldn't trip an error toast.
- **Three-state SessionState merge happens in the thin command, not `_inner`** (`merge_status` at `review.rs:145`) — `_inner` cannot promote to `Active` because it has no Tauri state. Plan-phase should not try to add new states without understanding this layering.
- **Phase 72 dropped `panelMode`** — Don't reintroduce panel-internal mode swaps. The session summary line and empty-state copy live in the existing list-render path.
- **Awaited destructive IPCs with try/catch + `showToast`** — End-review's `try`/`catch` should match: `instanceof Error` narrowing, error toast on failure, no half-success state.

### Integration Points
- `ReviewPanel.reload()` is the single chokepoint where cold-boot resume + post-End cleanup + multi-tab session-changed all converge. Keep the new logic concentrated here.
- The Toolbar Review button (Phase 72) doesn't need changes for Phase 73 unless D-02 plan-phase research argues for eager repo-open resume (which would require benchmark evidence).

</code_context>

<specifics>
## Specific Ideas

- User flow that motivates End-review: "write code → review in trunk → Copy markdown → paste into agent → agent revises → start fresh review." Today step 5 has no clean affordance — user manually deletes each comment.
- User specifically rejected the "session feels permanent" affordance gap; the End button is what addresses that, not just the Bug 3 fix.
- Advisor concern is locked: "auto-resuming the session at panel mount makes the permanence problem worse" — this is exactly why End-review is shipped in the same phase, not later.

</specifics>

<deferred>
## Deferred Ideas

- **Combined "Copy & End" button** — discussed and intentionally deferred. The user's flow suggests it, but coupling a destructive op to a non-destructive one is risky and confirmation friction would dominate. Revisit only if usage data shows users almost always End right after Copy.
- **Review history / archival** — Phase 74 candidate per the carry-forward todo. End-review will hard-delete the on-disk file; if archival is later desired, that becomes a backend change in its own phase.
- **Keyboard shortcut for End-review** — explicitly out of scope per the carry-forward todo. REQ-72-1b (Cmd+Shift+R) was retracted in 72-05 due to UAT clash with launcher tools; do not re-bind without UAT.
- **Eager-resume on repo-open for Toolbar accuracy** — flagged as plan-phase exploration *only* with mandatory benchmarking. Default ships with "Toolbar reflects runtime."
- **Explicit "Review was ended in another window" toast for multi-tab** — D-09 reuses the existing reload path silently; plan-phase may surface this if testing shows the empty state is confusing.

### Reviewed Todos (not folded)
- None — the only matching todo (`phase-73-review-lifecycle.md`) IS the phase brief; it's fully folded by definition.

</deferred>

---

*Phase: 73-review-lifecycle-end-review-cold-boot-resume*
*Context gathered: 2026-05-27*
