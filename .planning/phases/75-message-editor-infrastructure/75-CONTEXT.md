# Phase 75: Message Editor Infrastructure - Context

**Gathered:** 2026-05-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the two reusable primitives Phase 76 needs to wire commit-message editing into `merge --continue`, `merge <branch>`, and `revert <oid>`:

1. **`MessageEditor.svelte`** — modal that exposes a host-owned `open(default) → Promise<string | null>` API
2. **Rust temp-editor helper** (new module `src-tauri/src/git/editor.rs`) — single-shot `GIT_EDITOR=<script>` plumbing extracted from the queue-based pattern in `src-tauri/src/commands/interactive_rebase.rs:140-180`

Maps requirements MSG-04 (pre-fill from backend-built defaults) and MSG-05 (clean cancel). MSG-06 (empty → abort) building block lands here too — the modal's `null` return is the abort signal.

**No production code path changes this phase.** All three callers stay on their current `GIT_EDITOR=true` / `--no-edit` bypasses until Phase 76.

</domain>

<decisions>
## Implementation Decisions

### Modal API & UX
- **D-01:** Single textarea, pre-filled with the full default string. Matches `$EDITOR` semantics (REQUIREMENTS.md explicitly calls this out). No split summary/body — `.git/MERGE_MSG` contents are arbitrary multi-line text we hand to git verbatim.
- **D-02:** `open(default: string) → Promise<string | null>` host-owned API. Modal renders only while a call is pending; resolves on Save with the edited text, on Cancel/Esc with `null`.
- **D-03:** Per-operation `title` prop. Phase 76 callers pass strings like `"Merge commit message"`, `"Revert commit message"`. Trivial to implement, helps disambiguate in the UI.
- **D-04:** Empty/whitespace-only input resolves to `null` (treated identically to Cancel). Single uniform abort signal for Phase 76 consumers — `if (result === null) abort()`. Save button stays clickable; the trimmed-empty check lives in the resolve handler.
- **D-05:** `Cmd/Ctrl+Enter` from inside the textarea saves. `Esc` cancels. Plain `Enter` inserts a newline (textarea default — copy InputDialog's `!(e.target instanceof HTMLTextAreaElement)` guard).
- **D-06:** `Tab` uses default browser behavior (moves focus). Do not override to insert `\t` — commit messages rarely need tabs and overriding traps keyboard navigation.

### Rust Helper Scope
- **D-07:** New module `src-tauri/src/git/editor.rs` exposes a **single-shot** helper only. Returns a handle that holds the temp script path and message file; `Drop` cleans up both on success and error paths.
- **D-08:** `interactive_rebase.rs:140-180` keeps its inline queue-based script for now. No consolidation in Phase 75. Two patterns coexist briefly; that's the smaller blast radius and protects the 19-plan rebase test history.
- **D-09:** Helper script writes the user-edited content into git's commit-message file argument (`cp $temp_msg "$1"`). Same shape as the rebase queue script, just without the queue indirection. Unix `chmod 0755` via `PermissionsExt` mirrors existing pattern.

### Testing Strategy
- **D-10:** Component tests (`MessageEditor.test.ts`) cover: default pre-fill, edit + Save round-trip returns string, Esc returns null, Cancel button returns null, empty/whitespace-only input returns null on Save (D-04), `Cmd+Enter` saves, `Esc` cancels.
- **D-11:** Rust unit tests (`editor.rs` `#[cfg(test)]`) cover: script file is created with executable permissions, temp message file written, helper struct's `Drop` removes both files (happy path), drop on early return / explicit drop removes both files (error path). No subprocess invocation needed — assert on filesystem state.
- **D-12:** No integration test of the full git-launches-script-edits-message loop in Phase 75. That's a Phase 76 concern once a real production path uses it.

### Folded Todos
- **`.planning/todos/pending/2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md`** — directly describes the Phase 75 + 76 work. Phase 75 builds the infrastructure; Phase 76 does the wiring the todo prescribes. The todo's `resolves_phase: 76` is accurate; this phase enables it. Keep todo open; Phase 76 closes it.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Roadmap & requirements
- `.planning/ROADMAP.md` §"Phase 75: Message Editor Infrastructure" — phase goal, 4 success criteria
- `.planning/REQUIREMENTS.md` — MSG-04 (pre-fill source-of-truth), MSG-05 (clean cancel), MSG-06 (empty-aborts building block); v0.14 Out of Scope table

### Reference implementations to extract from / mirror
- `src-tauri/src/commands/interactive_rebase.rs:140-200` — queue-based `GIT_EDITOR=<script>` pattern; Phase 75 builds the single-shot variant of this
- `src/components/InputDialog.svelte` — existing modal styling, backdrop, Esc handling, `!(e.target instanceof HTMLTextAreaElement)` Enter-vs-newline guard. MessageEditor mirrors its visual treatment (CSS custom properties, no inline colors per project rule)
- `src/components/RebaseEditor.svelte:357-440` — inline message editor reference for how `editingSummary`/`editingBody` are handled today (we intentionally do NOT split — D-01)

### Phase 76 consumers (downstream — do not modify in Phase 75)
- `src-tauri/src/commands/operation_state.rs:171` — `merge_continue_inner` `GIT_EDITOR=true` (Phase 76 swaps for MessageEditor helper)
- `src-tauri/src/commands/operation_state.rs:301,304` — `merge_branch_inner` `--no-edit` + `GIT_EDITOR=true` (Phase 76)
- `src-tauri/src/commands/commit_actions.rs:153` — `revert_commit_inner` `--no-edit` (Phase 76)

### Related todo
- `.planning/todos/pending/2026-04-14-collect-commit-messages-for-merge-revert-instead-of-bypassing-editor.md` — folded (see D in Folded Todos)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`InputDialog.svelte`** — provides the modal shell pattern (backdrop, keydown, autofocus action, CSS custom-property styling). MessageEditor can mirror its structure but uses a single textarea + per-op title prop, and `Promise<string | null>` instead of `onsubmit`/`oncancel` callbacks.
- **`interactive_rebase.rs:157-172`** — the exact temp-editor script shell + `PermissionsExt` chmod pattern to extract. Single-shot variant drops the msg-queue directory and `ls | sort | head -1` indirection.

### Established Patterns
- **CSS custom properties only** (CLAUDE.md "Never inline colors"). Modal must use `var(--color-backdrop)`, `var(--color-surface)`, `var(--color-border)`, `var(--color-text)` etc. — same set InputDialog uses.
- **git2 for git ops, except editor-message editing** (CLAUDE.md). The temp-editor-script + subprocess pattern is the codebase-sanctioned exception, already used for rebase. We extend that exception to merge/revert.
- **TrunkError struct** for Rust error propagation. New helper returns `Result<EditorHandle, TrunkError>`.
- **`#[cfg(unix)]` + `PermissionsExt`** for chmod. Tauri is Unix-only for the rebase path today; helper follows the same gating.

### Integration Points
- **Phase 76 consumers** (not touched here): three call sites in `operation_state.rs` and `commit_actions.rs`. Helper API needs to feel natural for each — `let handle = editor::prepare(message)?; cmd.env("GIT_EDITOR", handle.script_path());` pattern.
- **Host of MessageEditor** (Phase 76 wires this): likely `RepoView.svelte` (already hosts RebaseEditor at `:36, :173, :754`). Host owns a single `let messageEditorRef = $state(...)` and exposes `messageEditorRef.open(default)` to children via callback props — same shape as `onopenrebaseeditor` (`RepoView.svelte:567,888`).

</code_context>

<specifics>
## Specific Ideas

- **No git-style `#` comment lines** in the pre-filled default. Git CLI strips lines starting with `#` from `$EDITOR` output; trunk's textarea is plain so we never add them, never strip them.
- **No "cut here" marker** ("# Please enter the commit message... # Lines starting with '#' will be ignored.") — the textarea is the editor; no commentary needed.
- **No draft persistence on Cancel.** REQUIREMENTS Out of Scope table is explicit: "Persisting draft messages across modal cancels: Cancel = throw away."
- **Modal title prop labels** Phase 76 will use (informational, not locked here): `"Merge commit message"`, `"Revert commit message"`. Phase 76 plan decides final wording.

</specifics>

<deferred>
## Deferred Ideas

- **Consolidate `interactive_rebase.rs` queue onto the new helper.** Logged as a tech-debt candidate for a future milestone — not Phase 76 either (Phase 76 only wires merge/revert). If the rebase queue script ever needs changes, the cleanup happens then.
- **`commit.template` support / Settings UI** — already deferred to v1.0 in REQUIREMENTS.md v2 (TMPL-01).
- **Commit signing UI** — deferred to v2 (SIGN-01..03 in REQUIREMENTS.md).
- **Pre-commit / commit-msg hook stdout streaming** — out of v0.14 scope per REQUIREMENTS.md.

</deferred>

---

*Phase: 75-message-editor-infrastructure*
*Context gathered: 2026-05-28*
