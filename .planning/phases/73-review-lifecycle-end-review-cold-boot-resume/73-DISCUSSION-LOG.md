# Phase 73: Review Lifecycle (End-review + cold-boot resume) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-27
**Phase:** 73-review-lifecycle-end-review-cold-boot-resume
**Areas discussed:** Resume trigger & timing, End-review placement & shape, End-review confirmation pattern, Empty-state copy

**Pre-discussion setup:** Phase 73 did not exist in ROADMAP.md when discussion started (v0.13 had been marked complete after Phase 72). User chose "Add Phase 73 to v0.13, then discuss" — milestone status reopened to `in-progress`, phase added with goal + success criteria derived from `.planning/todos/pending/phase-73-review-lifecycle.md`.

---

## Phase-level area selection

| Option | Description | Selected |
|--------|-------------|----------|
| Resume trigger & timing | Where/when does cold-boot resume fire? | ✓ |
| End-review placement & shape | Where does the End button live? | ✓ |
| End-review confirmation pattern | How do we guard the destructive action? | ✓ |
| Empty-state copy (cold vs warm-empty) | Distinguish "no session" from "session active, no comments" | ✓ |

User selected all four areas.

---

## Resume trigger & timing

### Q1 — Where should cold-boot resume be triggered?

| Option | Description | Selected |
|--------|-------------|----------|
| ReviewPanel mount (status === ResumeAvailable) | Smallest, most local change: reload() already calls get_review_session_status; just act on status.state. Loads only when user opens panel. Downside: Toolbar's active-state lies until panel mounts. | ✓ |
| Repo-open effect in RepoView | Resume eagerly when repo opens, so reviewActive truly reflects on-disk state. More honest, but loads sessions for repos the user may never review. | |
| Only on user-initiated review toggle | First toggle calls resume_or_start. Doesn't fully fix Bug 3 — user still sees 'no comments' until they toggle. | |

**Notes:** Backend discriminator (`SessionState::ResumeAvailable`) already exists; cold-boot resume is a frontend wiring change in `ReviewPanel.reload()`.

### Q2 — How to handle the Toolbar active-state lag?

| Option | Description | Selected |
|--------|-------------|----------|
| Ignore — Toolbar reflects runtime, not disk | reviewActive is a runtime concept; the Toolbar button is a toggle, not a status light. | (default) |
| Cheap probe on repo-open: status only, no resume | One IPC for status, no session data load. Toolbar truthful, no eager load cost. | |
| Eager resume on repo-open | Toolbar truthful, panel loads instantly when opened. Cost is one resume per repo on open. | |

**User's choice:** Free-text: *"I don't know. Use your judgment on this one. Don't assume things will perform in the way that you think. Just do benchmarks and actually measure things before deciding."*

**Notes:** Captured as Claude's discretion in CONTEXT.md (D-02) with the hard constraint that any eager-resume path **MUST be benchmarked** before lock-in. Default for now is "ignore — Toolbar reflects runtime." Saved as durable feedback memory `[[measure-before-assuming]]`.

---

## End-review placement & shape

### Q1 — Where should the End-review affordance live?

| Option | Description | Selected |
|--------|-------------|----------|
| Session-state header strip inside ReviewPanel | New row at top: 'N comments across M commits' + End button. Session feels first-class but ~32px of new chrome. | |
| ReviewPanel header, next to Copy | Minimal new chrome. Pairs destructive + non-destructive (mitigated by confirmation + iconography). | ✓ |
| Toolbar dropdown on the Review button | Click = toggle, long-click/chevron = menu with End. Inconsistent with rest of Toolbar. | |
| Use my judgment with constraints | Defer to plan-phase. | |

### Q2 — Should the panel also show a session-summary line?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — small caption above the comments list | Muted-text line ('3 comments · 2 commits'). Frames what End will destroy. Reuses rune state — no new IPC. | ✓ |
| No — keep panel terse | End button is enough; comments list shows scope. | |
| Yes, but only when not empty | Hide '0 comments · 0 commits' in warm-empty state. | |

**Notes:** The summary line landed in CONTEXT.md as D-04; chrome cost was acceptable to the user because it makes the session legible.

---

## End-review confirmation pattern

### Q1 — How should we guard End-review against accidental clicks?

| Option | Description | Selected |
|--------|-------------|----------|
| Inline two-step button | Click → 'Click again to confirm' for 3s, then auto-reverts. No modal, no overlay. Matches the app's vibe. | ✓ |
| Modal dialog with comment count | Standard confirmation modal with cancel/end. Heavyweight but matches gravity. | |
| No confirmation — trust the user | Speed-wins-friction. Risk: misclick deletes work. | |
| Toast undo affordance instead | End immediately + toast 'Review ended. [Undo]' for 5–10s. Requires tombstone in backend; new complexity. | |

**Notes:** No existing two-step button precedent in the codebase, but the 1500ms ✓ Copied timer pattern (`ReviewPanel.svelte:130-133`) is the close analog to extend — same `clearTimeout` before `setTimeout` discipline.

---

## Empty-state copy

### Q1 — How should the panel distinguish 'no session active' from 'session active, no comments yet'?

| Option | Description | Selected |
|--------|-------------|----------|
| Distinct copy, no CTA button | Cold: 'No active review. Toggle review mode in the toolbar to start.' Warm-empty: 'Review started. Select diff lines or add a commit note to comment.' | ✓ |
| Distinct copy + inline CTA for cold state | Add [Start review] button to cold state. Duplicates toolbar's Review button (third start path). | |
| Distinct copy + small illustration / icon for each | Lucide icon above each empty message. More visual differentiation. | |
| Single empty state, ignore the distinction | One generic message. Doesn't solve the design concern. | |

**Notes:** Exact wording is plan-phase copywriting; the structural decision is distinct copy for the two states.

---

## Final check

| Option | Description | Selected |
|--------|-------------|----------|
| I'm ready for context | Write CONTEXT.md now. Remaining open choices captured as Claude-discretion. | ✓ |
| Explore more gray areas | Candidates: Copy+End combined, multi-tab live coordination, summary placement, LoadOutcome handling. | |

**Notes:** Multi-tab coordination (D-09) and LoadOutcome handling (D-07) were resolved by Claude in CONTEXT.md using existing primitives — flagged for plan-phase to verify, not re-decided.

---

## Claude's Discretion

- **D-02 toolbar sync** — defer eager-resume decision to plan-phase, with mandatory benchmark gate.
- Exact Lucide icons (Trash2 candidate for End; MessageSquareOff/Plus for empty states).
- Exact copy wording for empty states + two-step button.
- Auto-revert duration for the two-step button (~3s suggested).
- Whether to surface an explicit "Review was ended in another window" toast for multi-tab tab B.

## Deferred Ideas

- **Combined "Copy & End" button** — flow suggests it; coupling destructive + non-destructive too risky for v1.
- **Review history / archival** — Phase 74 candidate per carry-forward todo.
- **Keyboard shortcut for End-review** — explicitly out of scope; do not re-bind Cmd+Shift+R without UAT (REQ-72-1b retraction context).
- **Eager-resume on repo-open** — plan-phase exploration only, with mandatory benchmark.
- **Explicit multi-tab "Review was ended" toast** — D-09 reuses existing reload path silently; revisit if testing shows confusion.
