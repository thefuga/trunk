# Phase 66: Commit Selection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 66-commit-selection
**Areas discussed:** Range seeding UX, Selection visibility, Add/remove affordances, Merge-commit handling

---

## Area selection

| Option | Description | Selected |
|--------|-------------|----------|
| Range seeding UX | base→tip pick interaction + re-seed semantics | ✓ |
| Selection visibility | graph marker + panel list depth | ✓ |
| Add/remove affordances | where add/remove live | ✓ |
| Merge-commit handling | are merges selectable | ✓ |

**User's choice:** All four areas.

---

## Range Seeding UX

### Interaction

| Option | Description | Selected |
|--------|-------------|----------|
| Two right-clicks on graph | 'Set as review base' then 'Add range to review'; reuses native context menu, no modal | ✓ |
| Dialog with two revspec inputs | Modal with base + tip text fields accepting refs/SHAs; net-new modal | |
| Shift/multi-select two rows | Click base, shift-click tip span, then 'Add range'; new multi-select model | |

**User's choice:** Two right-clicks on graph.

### Re-seed semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Union — add to the set | Seeding adds; hand-picks survive; dedup keeps clean | ✓ |
| Replace — clear then seed | New range clears current selection first | |

**User's choice:** Union — add to the set.

### Base edge

| Option | Description | Selected |
|--------|-------------|----------|
| Exclusive base (base..tip) | Only commits after base; matches `git log base..tip` and existing hide(base) | |
| Inclusive base (base^..tip) | Base commit itself included; hide(base.parent) | ✓ |

**User's choice:** Inclusive base — both base and tip are in the set.
**Notes:** "Base" interpreted as "the first commit I want to look at," not "the point I'm diffing against."

---

## Selection Visibility

### Graph marker

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — mark in-graph | Subtle marker on in-session commits (theme CSS var) | ✓ |
| No — panel list only | Graph unchanged; panel list is sole source of truth | |

**User's choice:** Yes — mark in-graph.

### List depth

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal (SHA + summary) | Short SHA + summary, graph order, dedup'd; least throwaway code | ✓ |
| Richer (+ author + date) | More info now, more markup Phase 69 rebuilds | |

**User's choice:** Minimal (SHA + summary).
**Notes:** Panel is Phase 65's throwaway stub (D-12), replaced in Phase 69 — don't over-invest.

---

## Add / Remove Affordances

| Option | Description | Selected |
|--------|-------------|----------|
| Context-menu toggle + panel × | Right-click toggles Add↔Remove; plus × per panel row | ✓ |
| Context-menu toggle only | Add and remove both via graph toggle; panel read-only | |
| Panel-driven remove | Add via menu (add-only); remove only via panel × | |

**User's choice:** Context-menu toggle + panel ×.

---

## Merge-Commit Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Allow merges in selection | Selectable; diff-source restriction deferred to Phase 67 capture | ✓ |
| Block merges at selection | 'Add' disabled for merges; range seeding skips them | |
| Allow + visual hint | Selectable but flagged that diff-source won't apply | |

**User's choice:** Allow merges in selection.
**Notes:** Selection ≠ anchor source (chosen at capture, Phase 67/68). Full-file-source review of a merge is valid; blocking would diverge from `git log`. Recorded as a constraint for Phase 67.

---

## Final gate

| Option | Description | Selected |
|--------|-------------|----------|
| Ready for context | Lock decisions, write CONTEXT.md | ✓ |
| Discuss invalid-range handling | Talk through base-not-ancestor / empty / base==tip | |
| Explore more gray areas | Surface additional gray areas | |

**User's choice:** Ready for context.

## Claude's Discretion

- Invalid-range handling (base not ancestor of tip, base==tip, empty) — lean validate + toast.
- Selected-but-not-loaded commits given paginated history — ordering/display fallback.
- Command surface & naming; persistence/event wiring reusing the Phase 65 store + `session-changed`.

## Deferred Ideas

- Enforcing diff-source-only-on-non-merges → Phase 67 (anchor capture).
- Richer commit list (author/date) → Phase 69 (real panel).
- Reviewed-not-folded todo: `2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` (unrelated — merge/revert message editing, not selection).
