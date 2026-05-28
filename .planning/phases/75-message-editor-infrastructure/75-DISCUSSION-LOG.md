# Phase 75: Message Editor Infrastructure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-28
**Phase:** 75-message-editor-infrastructure
**Areas discussed:** Editor surface shape, Rust helper scope, Save-gating on empty input, Keyboard ergonomics, Modal title

---

## Modal title

| Option | Description | Selected |
|--------|-------------|----------|
| Per-op title prop | Caller passes a `title` prop; Phase 76 disambiguates per operation. Trivial to implement. | ✓ |
| Fixed title | Modal always shows the same heading. Simpler API; relies on context to tell operations apart. | |

**User's choice:** Per-op title prop
**Notes:** Locked up-front before opening the four-area deep-dive.

---

## Editor surface shape

| Option | Description | Selected |
|--------|-------------|----------|
| Single textarea | One textarea pre-filled with the full default. Matches `$EDITOR` semantics; aligned with REQUIREMENTS note. Simplest contract for Phase 76 callers. | ✓ |
| Split summary + body | Two inputs like RebaseEditor's inline reword editor. Better visual hierarchy but requires splitting/joining arbitrary defaults (`.git/MERGE_MSG` can be messy) and diverges from `$EDITOR` semantics. | |

**User's choice:** Single textarea
**Notes:** Decisive alignment with the REQUIREMENTS.md Out-of-Scope row ("Modal is plain textarea (matches `$EDITOR` semantics)") and CLI parity.

---

## Rust helper scope

| Option | Description | Selected |
|--------|-------------|----------|
| Single-shot only | New `git/editor.rs` exposes a single-message helper for Phase 76's three callers. `interactive_rebase.rs` keeps its queue inline. Smallest blast radius. | ✓ |
| Consolidate with rebase queue | Helper handles both single-shot and N-message queue. Migrate `interactive_rebase.rs` in Phase 75. One pattern in the codebase, but Phase 75 grows and risks rebase regressions. | |
| Single-shot now + todo for rebase | Same as single-shot, but explicitly file a pending todo to migrate rebase later. | |

**User's choice:** Single-shot only (no extra todo filed)
**Notes:** Cleanest interpretation of the roadmap "extracted from `interactive_rebase.rs:157-172`" wording — extracted ≠ migrated.

---

## Save-gating on empty input

| Option | Description | Selected |
|--------|-------------|----------|
| Modal returns null on empty | Save remains clickable; empty/whitespace input resolves to `null` (same as Cancel). Aligns with Phase 75 success criterion #3's wording. Uniform consumer logic. | ✓ |
| Disable Save when empty | Save button greyed out until non-whitespace content. Friendlier but doesn't match the CLI "kill editor on empty" semantics MSG-06 references. | |
| Return literal empty string | Modal returns `""` for empty, `null` only for Esc/Cancel. More fidelity but redundant — both paths abort identically per MSG-06. | |

**User's choice:** Modal returns null on empty
**Notes:** Aligns with success criterion #3's exact wording and gives Phase 76 a single `if (result === null) abort()` pattern.

---

## Keyboard ergonomics

### Save shortcut

| Option | Description | Selected |
|--------|-------------|----------|
| Cmd/Ctrl+Enter saves | Standard convention in commit editors (GitHub Desktop, VS Code SCM). | ✓ |
| Click Save only | No keyboard save. Slower for keyboard-driven workflows. | |

**User's choice:** Cmd/Ctrl+Enter saves

### Tab behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Default browser behavior | Tab moves focus to next element. Standard accessibility; commit messages rarely need literal tabs. | ✓ |
| Insert literal `\t` | Override Tab to insert tab character. Traps keyboard navigation; non-standard. | |

**User's choice:** Default browser behavior

---

## Claude's Discretion

- Exact `chmod` invocation, `Drop` impl shape, temp-file naming convention in the Rust helper — standard Rust idioms, no user preference recorded.
- Modal CSS values (sizing, padding, z-index) — mirror `InputDialog.svelte`'s established treatment.
- Phase 76's final modal title strings ("Merge commit message" etc.) — Phase 76 planning decides.

## Deferred Ideas

- Consolidate `interactive_rebase.rs` queue onto the new helper — tech-debt candidate for a future milestone (not Phase 76 either; that phase only wires merge/revert).
