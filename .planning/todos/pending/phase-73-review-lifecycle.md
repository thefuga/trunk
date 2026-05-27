# Phase 73 — Review lifecycle redesign (End review + cold-boot resume)

**Created:** 2026-05-27
**Source:** Phase 72 gap closure (72-05). Carry-forward bundle.
**Predecessor todos resolved by Phase 72:**
- `2026-05-26-relocate-copy-action-off-preview-pane.md` (G-71-A — closed)
- `2026-05-26-review-pane-navigation-and-dead-review-button.md` (G-71-B — closed)

---

## Why this is bundled, not split

UAT for Phase 72 surfaced two issues that, taken together, point at the same
structural gap: the review session has no explicit lifecycle in the UI.

1. **Bug 3 — comments don't appear on first open after app boot.** The on-disk
   session data exists (Phase 65) but the runtime session isn't loaded until
   the user adds a line comment (which implicitly triggers `resume_review_session`
   via DiffPanel). Until that mutation, `ReviewPanel.reload()` queries
   `get_review_session_status` and `list_session_*`, gets `no_session`, and
   renders empty.

2. **User design ask — no End-review affordance.** The user's workflow is:
   write code → review in trunk → Copy markdown → paste into agent → agent
   revises → start fresh review. Today step 5 has no clean affordance — the
   user manually deletes each comment one by one. The review feels
   permanent, which is the underlying friction.

**Advisor concern (recorded during 72-05 planning):** Auto-resuming the
session at panel mount — the obvious fix for Bug 3 in isolation — makes the
permanence problem worse. The session would now visibly persist across app
restarts with no clean way to end it. The structurally honest move is to
ship the End-review affordance in the same wave as the resume, so the
lifecycle gains both endpoints together.

---

## Scope for Phase 73

### Bug 3 — verbatim from the 72-05 session

> Comments don't appear on first ReviewPanel open after app boot.
> `ReviewPanel.reload()` queries `get_review_session_status` then
> `list_session_*`; if the runtime session isn't loaded for this repo, the
> lists throw `no_session` and the panel renders empty. The on-disk session
> data exists (Phase 65), but no one calls `resume_review_session` at boot —
> DiffPanel does it implicitly when the user adds a line-comment, which is
> why mutations make all existing comments appear.

### User lifecycle ask

> Add an End review affordance so a review is not implicitly permanent. The
> user workflow is: write code → review in trunk → Copy markdown → paste
> into agent → agent revises → start fresh review. Today step 5 has no
> clean affordance — the user manually deletes each comment.

### Advisor concern

> Auto-resume without End-review would compound the permanence problem.
> Bundle Bug 3 with the End-review redesign so they ship together.

---

## Open design questions (for /gsd:discuss-phase)

- **Resume semantics:**
  - Option A: `get_review_session_status` returns a `has_saved_session: boolean`
    discriminator; ReviewPanel calls `resume_review_session` automatically when
    `state === 'idle' && has_saved_session === true`.
  - Option B: A dedicated `resume_or_start_review_session` command that the
    frontend calls unconditionally on first panel mount.
  - Tradeoff: A keeps the existing command set small; B removes a frontend
    branching responsibility but adds a command.

- **End review semantics:**
  - Archive (move to history list, surfaceable later) vs. wipe (delete the
    on-disk session file entirely).
  - The user's workflow ("start fresh") implies wipe is acceptable; history
    might be a separate Phase 74 ask.

- **Empty-state copy:**
  - Distinguish "no session active" (cold) vs. "session active, no comments
    yet" (warm-empty). They look identical today.

- **Copy-and-End in one action vs. separate buttons:**
  - The user's flow is copy → paste → revise → start fresh, so a combined
    "Copy + End review" could fit. But it's a destructive op coupled to a
    non-destructive one; confirmation friction may dominate.

- **Session-state header in ReviewPanel:**
  - "N comments across M commits" + an explicit End button would make the
    session feel like a first-class object, not an invisible mode.

---

## Backend primitives already available

- `start_review_session_inner` — creates a fresh session.
- `resume_review_session_inner` — loads an existing on-disk session into runtime.
- `end_review_session_inner` — terminates runtime + (presumably) on-disk state.
- `get_review_session_status_inner` — returns `{ state, file_exists, canonical_path }`.

The `SessionStatus` shape likely needs a `has_saved_session` discriminator
added for the Phase 73 work (or the equivalent surfaced in `state`).

---

## Out of scope

- **Keyboard shortcuts for Start/End review.** REQ-72-1b was retracted in
  72-05 (UAT clash with launcher tools). If Phase 73's End-review affordance
  proves to want a shortcut, revisit with a shortcut that doesn't collide
  with common launcher tools (Spotlight, Raycast, Recents, etc.). Do not
  re-bind Cmd+Shift+R without UAT.

- **Review history / archival of past sessions.** The user's stated workflow
  is "start fresh"; persistent history can be a Phase 74 candidate if asked.

---

## Acceptance hints for Phase 73 planning

- Cold boot → open ReviewPanel → on-disk comments appear without any
  mutation needed.
- ReviewPanel has a visible End review affordance; clicking it (with
  confirmation) terminates the runtime session and removes the on-disk
  session file.
- Empty state distinguishes "no session" from "session active, no comments".
- Existing copy flow still works; Copy and End are independently usable.
- `just check` exits 0.
