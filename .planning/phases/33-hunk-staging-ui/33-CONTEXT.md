# Phase 33: Hunk Staging UI - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Add context-aware hunk action buttons to DiffPanel with binary file guards and keyboard navigation between hunks. Requirements: HUNK-04, HUNK-06, HUNK-09. Backend hunk commands (stage_hunk, unstage_hunk, discard_hunk) already exist from Phase 32 — this phase wires the UI.

</domain>

<decisions>
## Implementation Decisions

### Hunk button placement & style
- Buttons appear in a **toolbar row above each hunk**, replacing the current `@@` header line
- Toolbar row contains: line range info (from `@@` header) on the left, action buttons on the right
- Buttons use **text labels only** — "Stage Hunk", "Unstage Hunk", "Discard Hunk" (no icons)
- Toolbar row has a **subtle distinct background** (same muted style as current `@@` header) to visually separate from diff lines

### Context-dependent button sets
- **Unstaged diff**: "Stage Hunk" + "Discard Hunk" buttons
- **Staged diff**: "Unstage Hunk" button only (no discard)
- **Commit diff**: No hunk buttons at all
- **Binary files**: No hunk buttons (existing `fd.is_binary` guard)
- DiffPanel needs a new prop to receive the diff kind ('unstaged' | 'staged' | 'commit') from App.svelte

### In-flight operation feedback
- **All hunk buttons in the file** are disabled during any hunk operation (prevents stale-index races since indices may shift)
- Disabled state: **reduced opacity + cursor: not-allowed** — standard disabled pattern, no spinners
- Buttons re-enable after diff re-fetch completes

### Hunk keyboard navigation
- `]` jumps to next hunk, `[` jumps to previous hunk
- Navigation **scrolls the diff view** so the target hunk's toolbar row is at the top, and **briefly highlights** it (flash or border)
- At edges: **stop** — no wrap-around (] on last hunk does nothing, [ on first does nothing)
- Shortcuts **only active when DiffPanel is visible** — prevents conflicts with commit message textarea or other inputs

### Discard hunk flow
- Discard Hunk button appears alongside Stage Hunk in the toolbar row (unstaged diffs only)
- Confirmation uses **Tauri ask() dialog** — "Discard this hunk? This cannot be undone." — consistent with existing discard_file pattern
- Backend trusts frontend confirmation (decided in Phase 32)

### Claude's Discretion
- Exact CSS styling of toolbar row (padding, font size, button spacing)
- Highlight animation implementation for hunk navigation (CSS transition or brief class toggle)
- Whether to track focused hunk index in component state or derive from scroll position
- How to pass diff kind prop through (direct prop vs context)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### DiffPanel (primary integration point)
- `src/components/DiffPanel.svelte` — Current read-only hunk rendering. Toolbar rows replace existing `@@` header lines. Must add diff kind prop and hunk action buttons.

### Backend hunk commands (Phase 32 output)
- `src-tauri/src/commands/staging.rs` — `stage_hunk_inner`, `unstage_hunk_inner`, `discard_hunk_inner` functions. Command signatures: `(path, file_path, hunk_index, state)`. Error codes: `stale_hunk_index`, `hunk_apply_failed`, `file_not_found`.

### Diff type definitions
- `src-tauri/src/git/types.rs` — Rust `DiffHunk`, `DiffLine`, `FileDiff` structs
- `src/lib/types.ts` — TypeScript mirror types (`DiffHunk`, `DiffLine`, `FileDiff`, `is_binary` field)

### File selection & diff routing
- `src/components/App.svelte` — `handleFileSelect(path, kind)` determines diff command. `selectedFile.kind` holds 'unstaged' | 'staged'. Must pass kind to DiffPanel. Also: keyboard shortcut pattern (global `$effect` listener).

### Staging patterns
- `src/components/StagingPanel.svelte` — `safeInvoke` + `loadStatus()` refresh pattern, `loadingFiles` Set for disabled state, `ask()` for destructive confirmation

### IPC & error handling
- `src/lib/invoke.ts` — `safeInvoke<T>` with `TrunkError` parsing (code + message)
- `src/lib/toast.svelte.ts` — `showToast(message, kind)` for error/success feedback

### Command registration
- `src-tauri/src/lib.rs` — `invoke_handler` list (hunk commands already registered in Phase 32)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `safeInvoke<T>`: IPC wrapper with TrunkError parsing — use for all hunk commands
- `showToast`: Toast feedback for success/error — use for hunk operation results
- `ask()` from `@tauri-apps/plugin-dialog`: Native confirmation dialog — use for discard hunk confirmation
- `loadStatus()` in StagingPanel: Refresh file list after mutations — must be called after hunk ops
- `loadingFiles` Set pattern: Disable UI during operations — adapt for hunk-level disabling

### Established Patterns
- Inner-fn pattern for Tauri commands (already done in Phase 32)
- `selectedFile: { path, kind }` tracks current file and diff source in App.svelte
- Keyboard shortcuts via global `$effect` with `window.addEventListener('keydown', ...)`
- Binary detection via `fd.is_binary` — skip interactive UI for binary files
- Stale-request guard via sequence counter (`++loadSeq` pattern)

### Integration Points
- DiffPanel receives `fileDiffs` and `selectedPath` from App.svelte — needs new `diffKind` prop
- After hunk op: must re-fetch diff (call `refetchFileDiff`) AND refresh status (trigger `loadStatus`)
- Hunk index is 0-based array position — passed directly as `hunk_index` to backend commands

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 33-hunk-staging-ui*
*Context gathered: 2026-03-17*
