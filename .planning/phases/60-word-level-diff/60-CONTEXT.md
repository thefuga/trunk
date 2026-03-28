# Phase 60: Word-Level Diff - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Highlight changed words and characters within modified lines with background highlighting, so users can instantly see what changed within a line. Uses the `similar` crate for inline change detection. Populates the existing `word_spans` field on DiffLine (added empty in Phase 59).

</domain>

<decisions>
## Implementation Decisions

### Diff Algorithm Granularity
- **D-01:** Word-level tokenization — split lines on whitespace and punctuation boundaries for comparison. Character-level is too noisy (highlights every letter in reworded sections). Language-aware token-level belongs in Phase 61 (syntax highlighting).
- **D-02:** Use `similar` crate's `TextDiff` with word-level splitting via `ChangeTag` iteration on paired lines.

### Line Pairing Strategy
- **D-03:** Sequential pairing within hunks — pair consecutive Delete/Add runs by position (first Delete with first Add, second with second, etc.). Unpaired lines (more Deletes than Adds or vice versa) get empty `word_spans`.
- **D-04:** Pairing happens per-hunk, not across hunks.

### Performance Thresholds
- **D-05:** Both threshold checks run in Rust backend before populating `word_spans`. If a line exceeds 500 characters or the paired lines have >60% edit distance, leave `word_spans` empty.
- **D-06:** Frontend simply checks if `word_spans` is non-empty — no frontend-side threshold logic. Empty spans = render line-level coloring only.

### Highlight Colors
- **D-07:** CSS custom properties for word-diff backgrounds: `--color-diff-word-add-bg` (highlight on Add lines) and `--color-diff-word-delete-bg` (highlight on Delete lines). Never inline colors.
- **D-08:** Word-diff highlights render as background-color spans within the line text, overlaid on the existing line-level add/delete background.

### Claude's Discretion
- Exact `similar` API usage (Algorithm::Patience vs Myers vs LCS — pick best for code diffs)
- How to compute edit distance ratio for the 60% threshold
- Whether to use `iter_inline_changes()` or manual word splitting + diff
- Frontend rendering approach (inline `<span>` elements vs CSS ranges)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Rust types and diff pipeline
- `src-tauri/src/git/types.rs` — WordSpan struct (start: u32, end: u32, emphasized: bool), DiffLine with word_spans field
- `src-tauri/src/commands/diff.rs` — `walk_diff_into_file_diffs` central converter where word_spans must be populated

### Frontend types and rendering
- `src/lib/types.ts` — TypeScript WordSpan interface mirror
- `src/components/DiffPanel.svelte` — Current diff line rendering (needs word-span highlight rendering)

### Requirements
- `.planning/REQUIREMENTS.md` — WORD-01 (word/char highlighting), WORD-02 (performance guard thresholds)

### Prior phase context
- `.planning/phases/59-backend-data-model-diff-options/59-CONTEXT.md` — D-05 (enrichment field structure), accumulated decisions on byte offset ranges

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `WordSpan { start: u32, end: u32, emphasized: bool }` (types.rs:141): Already defined — `emphasized: true` marks changed segments, `false` marks unchanged segments within the line
- `walk_diff_into_file_diffs` (diff.rs:39): Central function that builds DiffLine structs — word-span population hooks in here
- `DiffRequestOptions` (types.rs:154): Already wired through all diff commands — no new plumbing needed

### Established Patterns
- Byte offset ranges (u32 start/end) for enrichment fields — decided in Phase 59, frontend slices `content` string using these
- All diff computation in Rust, frontend only renders — word-diff computation follows this pattern
- CSS custom properties for all colors — word-diff highlights must use theme variables

### Integration Points
- `walk_diff_into_file_diffs` → after building DiffLine list per hunk, pair Delete/Add lines and compute word_spans
- `DiffPanel.svelte` line rendering → check `word_spans.length > 0`, render `<span>` elements with highlight classes
- Theme CSS → add `--color-diff-word-add-bg` and `--color-diff-word-delete-bg` variables

</code_context>

<specifics>
## Specific Ideas

- `similar` crate was explicitly chosen over Shiki/JS alternatives — runs on Rust thread pool, purpose-built `iter_inline_changes()` API
- WordSpan `emphasized` field distinguishes changed (true) vs unchanged (false) segments — frontend highlights only emphasized spans
- The 500-char and 60% thresholds from WORD-02 are hard limits, not configurable by user

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 60-word-level-diff*
*Context gathered: 2026-03-28*
