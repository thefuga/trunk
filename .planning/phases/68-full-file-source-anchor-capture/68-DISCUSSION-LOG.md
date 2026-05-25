# Phase 68: Full-File-Source Anchor Capture - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 68-full-file-source-anchor-capture
**Areas discussed:** Selection model, Delete-line selectability, Gap-crossing selections, Cached excerpt format

---

## Selection Model

📊 Research presented (GitHub/GitLab full-file commenting UX): GitHub uses gutter
line-number click + shift-click/drag for a contiguous range in both full-file
(`#Lstart-Lend`) and diff views; non-contiguous selection for comments is an
unsupported long-standing request; the full-file frame maps cleanly to a single
contiguous span.

| Option | Description | Selected |
|--------|-------------|----------|
| Contiguous range (shift-click) | Click + shift-click highlights a contiguous span; matches GitHub full-file UX; maps 1:1 to the anchor's single start..end | ✓ |
| Per-line toggle Set | Click toggles individual lines like the diff view; non-contiguous toggling adds invisible gaps since anchor collapses to min..max anyway | |
| Native text selection | Derive start..end from a DOM text selection; native feel but fiddly/error-prone mapping back to gutter line numbers | |

**User's choice:** Contiguous range (shift-click)
**Notes:** FullFileView is a flat continuous document with no hunk boundaries, so the diff view's single-hunk Set<index> model does not apply — net-new selection state.

---

## Delete-Line Selectability

| Option | Description | Selected |
|--------|-------------|----------|
| Non-selectable | Full-file = "the file AT this commit" = new side; Delete lines render but aren't selection endpoints; spans pass over them | ✓ |
| Selectable, dropped from range | Carry Phase 67 L-03: deletes allowed in span, dropped from start..end, kept in excerpt | |

**User's choice:** Non-selectable
**Notes:** Intentional divergence from Phase 67 L-03 — full-file semantics are new-side blob coordinates, so deletes are excluded from both range and excerpt.

---

## Gap-Crossing Selections

| Option | Description | Selected |
|--------|-------------|----------|
| Cache visible lines + marker | Allow gap-crossing; start..end stays correct (blob coords); excerpt holds visible lines with a "… N lines unchanged …" marker; render re-resolves from blob anyway | ✓ |
| Reject gap-crossing selection | Block selections straddling a dropped region; guarantees hole-free excerpt but adds friction | |
| Cache visible lines, silent skip | Allow it, no marker — only the gutter line-number jump hints at the gap | |

**User's choice:** Cache visible lines + marker
**Notes:** The 100k-context full-file view can drop large unchanged regions; the cached excerpt is an offline fallback only (Phase 70 re-resolves from a fresh blob read), so low friction wins.

---

## Cached Excerpt Format

| Option | Description | Selected |
|--------|-------------|----------|
| Plain new-side content | Selected lines as plain code (no +/- prefixes); matches DOC-02 language-fenced render and new-side blob semantics | ✓ |
| Diff-format (+/- prefixes) | Reuse Phase 67's prefixLine style; consistent with diff-source but mismatches DOC-02 and reintroduces excluded deletes | |

**User's choice:** Plain new-side content
**Notes:** Diverges from Phase 67 L-06 (diff-format excerpt) to match the full-file render contract (DOC-02).

---

## Claude's Discretion

- Full-file adapter API shape & location (`buildFullFileAnchor` analog; new file vs. extend `diff-anchor.ts`).
- `CommentComposer` adaptation strategy (parameterize by injected captured result vs. add a full-file mode).
- Selection-state ownership & wiring through `DiffViewer` → `FullFileView`.
- Empty/zero-hunk file → affordance simply doesn't appear.
- Attach-success feedback (silent `session-changed` reload vs. optional toast).
- Empty-text submit disabled; exact validation point.

## Deferred Ideas

- Full-file gutter markers / in-place comment browser — Phase 69 (CMT-04).
- Fresh-blob excerpt re-resolution + "unresolvable" section — Phase 70 (DOC-04).
- Reviewed-not-folded todo: `2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` (unrelated; declined in Phases 66/67).
