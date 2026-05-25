# Phase 69: Comment Management UI - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-26
**Phase:** 69-comment-management-ui
**Areas discussed:** Commit-level comments, Edit/delete identity, Jump-to-anchor & in-diff, Panel presentation

---

## Commit-level comments (ANCH-03)

### What does a commit-level comment attach to?

| Option | Description | Selected |
|--------|-------------|----------|
| Tied to a commit | Attached to a specific commit (no file/line); renders with the SHA. Matches "commit-level" + Phase 70 `(sha)` contract. Needs to store commit_oid without lines. | ✓ |
| Free-floating note | anchor=None as schema allows — a general note, no commit. Zero schema change but loses per-commit framing. | |
| You decide | Delegate to Claude. | |

**User's choice:** Tied to a commit

### Where does the user create one?

| Option | Description | Selected |
|--------|-------------|----------|
| Per-commit in panel | Each commit row gets an "Add note" affordance. | ✓ |
| Right-click in graph | Reuse the commit-row context menu (Phase 66 pattern). | |
| Global add button | One add button; prompt which commit. | |
| You decide | Delegate to Claude. | |

**User's choice:** Per-commit in panel
**Notes:** Creation lives next to the commit the note is about; aligns with group-by-commit panel layout.

---

## Edit/delete identity (CMT-02/03)

| Option | Description | Selected |
|--------|-------------|----------|
| Stable id field | Add `id` to each Comment; edit/delete target by id. Robust to reordering + multi-tab races. Costs schema bump to v2 + id backfill on load. | ✓ |
| Array index | Reference by Vec position. Zero schema change, but a stale index edits/deletes the wrong comment. | |
| You decide | Delegate to Claude. | |

**User's choice:** Stable id field
**Notes:** Index is the weak link given Phase 65's multi-tab correctness investment. The id rides the same schema v2 bump already needed for commit-level comments.

---

## Jump-to-anchor & in-diff (CMT-04)

### Resolvability check timing/depth

| Option | Description | Selected |
|--------|-------------|----------|
| Eager, at panel load | Rust git2-backed command resolves every comment on load; orphan badges + reasons show up-front. Matches success criterion wording. | ✓ |
| Lazy, on jump click | Check only on click; cheaper load, but orphans look normal until clicked. | |
| You decide | Delegate to Claude. | |

**User's choice:** Eager, at panel load

### In-diff comment markers (deferred from Phase 67)

| Option | Description | Selected |
|--------|-------------|----------|
| Defer it again | Keep Phase 69 panel-side; gutter markers go to backlog. | ✓ |
| Include gutter markers | Anchored lines show markers in diff/full-file; click opens the comment. Larger scope. | |
| You decide | Delegate to Claude. | |

**User's choice:** Defer it again
**Notes:** Keeps the phase focused on the 5 success criteria; markers are a separate review-UX slice.

---

## Panel presentation

### List organization

| Option | Description | Selected |
|--------|-------------|----------|
| Group by commit | Comments nested under their commit. Matches per-commit "Add note" + review mental model. | ✓ |
| Flat capture-order | One flat list in add order. Simplest, harder to scan. | |
| Group by file | Nested under file path; commit-level needs a separate bucket. | |
| You decide | Delegate to Claude. | |

**User's choice:** Group by commit

### Edit UX

| Option | Description | Selected |
|--------|-------------|----------|
| Inline in panel | Click edit → textarea in place in the panel. Works for line-anchored, commit-level, and orphaned alike. | ✓ |
| Reopen diff composer | Reuse the Phase 67 composer in the diff. Couples edit to a working anchor. | |
| You decide | Delegate to Claude. | |

**User's choice:** Inline in panel

### Jump layout (panel replaced the diff)

| Option | Description | Selected |
|--------|-------------|----------|
| Jump reveals the diff | Select commit + file, swap right pane to diff/full-file per Source, scroll + highlight; persistent "Review" toggle returns. | ✓ |
| Panel becomes a sidebar | Panel shrinks alongside the diff; contradicts the locked "replaces diff content" note. | |
| You decide | Delegate to Claude. | |

**User's choice:** Jump reveals the diff

---

## Claude's Discretion

- Commit-level storage mechanism in v2 (new `Source::Commit` variant vs optional Anchor line fields vs dedicated shape) — keep Phase 67/68 anchors unchanged.
- `id` generation strategy (uuid / monotonic / hash).
- New command surface & naming: read-comments command (CMT-01), `edit_comment`, `delete_comment`, commit-level writer (extend `add_comment` vs sibling), and whether the resolvability check is its own command or folded into the read.
- `review-session.svelte.ts` rune module shape and Review-mode right-pane swap wiring.
- Save/attach feedback (silent-via-`session-changed` vs optional toast) and empty-text validation point.

## Deferred Ideas

- In-diff comment markers / browser (gutter badges, click-to-edit) — backlog.
- Render-time excerpt re-resolution + "unresolvable" section — Phase 70.
- Comment metadata (severity, author, threading) — out of scope per REQUIREMENTS.
- Reviewed-not-folded todo: `2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` (unrelated; declined again).
