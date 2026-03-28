# Phase 60: Word-Level Diff - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 60-word-level-diff
**Areas discussed:** Diff algorithm granularity, Line pairing strategy, Performance thresholds, Highlight colors
**Mode:** Auto (--auto flag, all recommended defaults selected)

---

## Diff Algorithm Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Word-level | Split on whitespace/punctuation boundaries | ✓ |
| Character-level | Compare individual characters | |
| Token-level | Language-aware tokenization | |

**User's choice:** [auto] Word-level (recommended default)
**Notes:** Character-level too noisy for code diffs. Token-level requires language awareness which is Phase 61's domain. Word-level is the standard approach (GitHub, VS Code).

---

## Line Pairing Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Sequential within hunks | Pair Delete/Add runs by position per-hunk | ✓ |
| Nearest-match | Use edit distance to find best match | |
| Cross-hunk | Pair across hunk boundaries | |

**User's choice:** [auto] Sequential within hunks (recommended default)
**Notes:** Sequential pairing is what GitHub Desktop and VS Code use. Nearest-match adds complexity with marginal benefit. Cross-hunk would break the hunk abstraction.

---

## Performance Thresholds

| Option | Description | Selected |
|--------|-------------|----------|
| Backend (Rust) | Check thresholds before populating word_spans | ✓ |
| Frontend (JS) | Check thresholds before rendering highlights | |
| Both | Double-check in both layers | |

**User's choice:** [auto] Backend (Rust) (recommended default)
**Notes:** Keeps frontend simple — just checks word_spans.length. No wasted IPC bandwidth sending spans that would be discarded.

---

## Highlight Colors

| Option | Description | Selected |
|--------|-------------|----------|
| Semantic CSS custom properties | --color-diff-word-add-bg / --color-diff-word-delete-bg | ✓ |
| Inline computed colors | Calculate from line background at render time | |

**User's choice:** [auto] Semantic CSS custom properties (recommended default)
**Notes:** Consistent with project rule: never inline colors, always use CSS custom properties from the theme.

---

## Claude's Discretion

- Exact `similar` API usage (algorithm choice, word splitting method)
- Edit distance ratio computation method
- Frontend rendering approach (inline spans vs CSS ranges)

## Deferred Ideas

None — discussion stayed within phase scope
