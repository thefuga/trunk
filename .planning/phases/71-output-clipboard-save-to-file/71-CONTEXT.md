# Phase 71: Output (Clipboard) — Context

**Gathered:** 2026-05-26
**Status:** Ready for planning

<domain>
## Phase Boundary

User can copy the generated review markdown to the system clipboard from the `ReviewDocPreview` view, with explicit success/failure feedback.

**Descope (this discussion):** Save-to-file (OUT-02 and ROADMAP success criteria #2/#3) is dropped. Rationale: the dialog/permission/atomic-write/filename/default-dir/overwrite/release-build-verification surface area was disproportionate to the marginal value over `Cmd+V` into the user's own editor. Users who want a file can save the pasted content themselves. ROADMAP.md and REQUIREMENTS.md need follow-up edits (see Deferred Ideas → Follow-up edits).

</domain>

<decisions>
## Implementation Decisions

### Affordance
- **D-01:** Single `Copy` button. Lives in the existing `.preview-spacer` flex cell in `ReviewDocPreview.svelte` header (Phase 70 left it intentionally empty for this).
- **D-02:** Icon + label, Lucide icon (`Clipboard` or `ClipboardCopy`). Match the existing `← Back to comments` button styling exactly (border + transparent bg + hover transition) — no primary-filled accent.

### Click feedback
- **D-03:** On successful copy, button swaps its label+icon to `✓ Copied` for ~1.5s, then reverts. Local component state, no global toast for the success case.
- **D-04:** Button remains clickable during the "Copied" window — user can re-copy without waiting for the revert.

### Failure handling
- **D-05:** On clipboard write failure, show an error toast with the underlying reason: `showToast("Failed to copy: <error message>", "error")` using the existing `src/lib/toast.svelte.ts` facade (same pattern as `ReviewPanel`).
- **D-06:** No fallback modal / no retry — surface the error and let the user act.

### Keyboard
- **D-07:** No app-level keyboard binding. `Cmd/Ctrl+C` is left entirely to native text selection inside the preview `<pre>` (the user can select a region and copy it natively).

### Empty-doc edge case
- **D-08:** Not reachable. Phase 70 D-11 gates `Generate` on `comment_count > 0`, so the preview view is never shown with an empty markdown string. No defensive handling.

### Claude's Discretion
- Exact Lucide icon name (`Clipboard` vs `ClipboardCopy`) — pick whichever reads cleaner in context.
- The exact "Copied" revert duration (target ~1.5s; tune if it feels off in dev).
- Whether to await the `writeText` Promise in a `try/catch` vs a `.then/.catch` chain — both fine; pick the one that matches the local component style.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — OUT-01 (copy). OUT-02 dropped this phase; see Deferred Ideas.

### Roadmap
- `.planning/ROADMAP.md` §"Phase 71" — success criterion #1 (copy with explicit feedback) is in scope; #2 and #3 (save dialog, cancel-noop) are descoped this phase.

### Immediate surface area
- `src/components/ReviewDocPreview.svelte` — host component. The `.preview-spacer` cell is the docking point.
- `src/lib/toast.svelte.ts` + `src/components/Toast.svelte` — toast facade for failure case.
- `src-tauri/capabilities/default.json` — already grants `clipboard-manager:allow-write-text`. **No capability changes needed** (the originally-anticipated `dialog:allow-save` is now N/A).
- `@tauri-apps/plugin-clipboard-manager` `writeText` — established pattern; see `src/components/CommitGraph.svelte:757`, `src/components/CommitDetail.svelte:72`, `src/App.svelte:133`. Note: those callsites use the fire-and-forget `.catch(() => {})` pattern — **this phase intentionally diverges** (await + surface error).

### Prior phase context
- `.planning/phases/70-excerpt-resolution-markdown-render/70-CONTEXT.md` — Phase 70 D-11 (zero-comment gate) + the preview-spacer-reserved-for-71 note.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `showToast(message, "error")` from `src/lib/toast.svelte.ts` — exact pattern already used by `ReviewPanel` for command failures.
- Lucide icons via `@lucide/svelte` — already in use throughout `ReviewPanel` (`FileText`, `MessageSquarePlus`).
- `ReviewDocPreview.svelte` header back-button is the visual template for the Copy button (same border/hover/font-size).

### Established Patterns
- Inner-state-driven button label swap is a familiar Svelte 5 rune pattern (`$state<boolean>` + `setTimeout`).
- Clipboard `writeText` is a Promise — divergence from the fire-and-forget calls elsewhere is **intentional** for this phase (the artifact IS the product; silent failure is unacceptable).

### Integration Points
- `ReviewDocPreview.svelte` receives `markdown: string` as a prop. The Copy button reads from that same prop — no new IPC, no new Rust commands.
- No backend changes. No `src-tauri/` edits. No new Tauri capability permissions.

</code_context>

<specifics>
## Specific Ideas

- Success toast text was discussed but superseded by the in-button affordance. If we ever add a toast for success, candidate copy was `"Review copied to clipboard"`.
- Save dialog defaults were partially discussed (repo parent dir, `trunk-review-<YYYYMMDD-HHMM>.md` filename, local time, trust native overwrite dialog) before the descope. **Preserved here for the deferred OUT-02 if it ever comes back.**

</specifics>

<deferred>
## Deferred Ideas

### Save-to-file (OUT-02, descoped from this phase)
Dropped entirely from REQUIREMENTS.md per user decision. If revisited later, the design ground covered in this discussion was:
- Default directory: parent of the repo (`dirname(repoPath)`)
- Filename: `trunk-review-<YYYYMMDD-HHMM>.md` (no repo-name slug; parent-dir colocation disambiguates)
- Timestamp: local time
- Overwrite: trust the native save dialog (no app-level recheck)
- Rust write: custom `std::fs` command with atomic tmp+rename (per existing project pattern); no `fs:` plugin needed
- Capability: would need `dialog:allow-save` added to `src-tauri/capabilities/default.json`
- Verify in a **release** build, not just dev

### Follow-up edits (must happen before plan-phase)
These are NOT discussion items — they are bookkeeping that needs to land alongside this CONTEXT.md so downstream agents see consistent state:
1. `.planning/REQUIREMENTS.md`: remove **OUT-02**.
2. `.planning/ROADMAP.md` Phase 71:
   - Title: `Output (Clipboard + Save-to-File)` → `Output (Clipboard)`
   - Goal: drop the "or file" clause
   - Success criteria: remove #2 (save dialog) and #3 (cancel-noop)
   - Notes: remove the `dialog:allow-save`, save-strategy, suggested-filename, and release-build-save bullets
3. Phase 71 directory slug (`71-output-clipboard-save-to-file`) is now misleading. Optional rename to `71-output-clipboard`; leaving as-is is also fine since the directory name is internal.

</deferred>

---

*Phase: 71-Output (Clipboard)*
*Context gathered: 2026-05-26*
