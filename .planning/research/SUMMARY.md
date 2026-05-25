# Project Research Summary

**Project:** Trunk v0.13 Code Review Mode
**Domain:** Persisted line-anchored review-comment session + AI-targeted markdown export in a Tauri 2 + Svelte 5 + Rust desktop Git GUI
**Researched:** 2026-05-25
**Confidence:** HIGH

## Executive Summary

v0.13 Code Review Mode is a single-user, session-based feature for collecting commit/file/line-anchored comments and rendering them into one markdown document framed for an AI coding agent to act on. All four research tracks converge on the same conclusion: the feature is achievable entirely within the existing stack with zero new dependencies, because every required capability — diff line selection (v0.7), full-file-at-commit view (v0.12), commit graph context menus (v0.3/v0.5), clipboard write, native save dialog, language detection, git2 blob reads, serde_json — is already installed and proven. The genuinely new surface area is: a session persistence layer, the session panel UI, an anchor capture adapter, and a markdown renderer. Everything else is integration and wiring.

The recommended approach is to build in strict dependency order: get the data model and persistence exactly right in Phase 1 before touching any UI, because every other phase reads/writes the anchor schema. Two architectural decisions carry the most implementation risk and must be made correctly in Phase 1: (1) anchors must translate diff-array positions to stable `(commit_oid, file_path, side, start_line, end_line, source)` tuples at capture time — never persist hunk/line indices from the in-memory diff array, which become meaningless on re-fetch; (2) render-time excerpts for the `full_file` source must come from a fresh git2 tree→blob read, never from re-running the 100k-context diff. Both mistakes look correct in the same session but silently corrupt on restart or history rewrite.

The primary risks are schema design mistakes (recoverable only with a migration or session loss), a capability permission gap (`dialog:allow-save` is not yet in `capabilities/default.json`), and the interaction between the filesystem watcher and in-progress comment drafts (drafts must be persisted as soon as an anchor is attached, not on form submit). All risks are avoidable with correct foundation design and are well-understood from the codebase audit.

## Key Findings

### Recommended Stack

Zero new dependencies are required. Every capability the feature needs is in `Cargo.toml` or `package.json` today. The only action needed on the stack is: (a) add `dialog:allow-save` to `capabilities/default.json` for the save-to-file path, and (b) add a custom Rust `std::fs` write command for atomic file output — consistent with the project's established pattern of Rust commands + std::fs instead of the fs plugin.

Markdown generation belongs in Rust (not TypeScript): git blob bytes already live in Rust, and the renderer must handle unresolvable anchors by re-reading git2 state at render time. Hand-roll the markdown string — no template engine, no markdown crate. Fence collision is handled inline: scan each excerpt for the longest backtick run, open with `max(3, longest_run + 1)` backticks.

**Core technologies (all already installed):**
- `git2` 0.19: session seeding (revwalk), excerpt extraction (tree→blob read), anchor resolution at render time — the mandated git backend
- `serde` / `serde_json` 1: serialize/deserialize the per-repo `ReviewSession` struct with `schema_version` field; atomic tmp+rename write
- `tauri` 2.10.2 (`Manager::path().app_data_dir()`): resolve the OS-correct per-app data directory for session storage — no `dirs` crate needed
- `@tauri-apps/plugin-clipboard-manager` 2.3.2: `writeText(renderedMarkdown)` — already used for copy-SHA/message
- `@tauri-apps/plugin-dialog` 2.6.0: `save()` returns `string | null` — already in use for other dialogs
- `tauri-plugin-store` (LazyStore): UI preferences ONLY (panel open/closed, last-used export dir) — NOT session data

**What NOT to use:**
- LazyStore for the review session — wrong shape for nested mutating documents; non-atomic reads invite lost-update races across same-repo tabs (v0.9 allows duplicate-repo tabs)
- `DefaultHasher` for the repo filename key — output is not stable across Rust versions; a toolchain bump would orphan all existing session files
- `uuid`, `sha2`, `blake3` — per-session IDs need only a persisted monotonic counter; the repo path key needs only canonicalize + percent-encode
- `tauri-plugin-fs` — unjustified for a single user-chosen write path; `std::fs` via a Rust command is one line

### Expected Features

The output document recipient is an AI coding agent, not a human. This inverts several conventions: comment phrasing carries the instruction signal (no severity tags), code excerpts are load-bearing grounding anchors (not courtesy context), and location must be machine-locatable (`path:Lstart-Lend` heading). The document is a one-shot static artifact — no threading, no resolve/approve state, no posting to a forge.

**Must have (table stakes, all P1):**
- One persisted review session per repo; start/resume across restarts until render
- Seed from commit range (base→tip via git2 revwalk) + hand-pick individual commits from the graph
- Anchor comments to diff-view selections (`source=diff`) — reuses v0.7 line selection
- Anchor comments to full-file-at-commit selections (`source=full_file`) — reuses v0.12 full-file view
- Anchor model: tagged enum `Diff{commit_sha, file_path, side, old_range, new_range}` | `FullFile{commit_sha, file_path, start_line, end_line}` — the keystone everything reads/writes
- Optional commit-level comment with no code anchor
- Edit and delete comments
- Session panel: list all comments, jump-to-anchor, generate button
- Markdown render: preamble + commit list + file-grouped/line-sorted sections + location heading (`path:Lstart-Lend (sha)`) + short excerpt (5-10 lines) + diff-fence for `source=diff`, language-fence for `source=full_file` + verbatim instruction text
- Graceful unresolvable-anchor section at render (never drop a comment; never crash)
- Copy-to-clipboard + save-to-file output

**Should have (differentiators, P2):**
- Live markdown preview pane before export
- GitHub-style suggestion/exact-replacement blocks
- Per-comment include/exclude toggle

**Defer (v2+):**
- Configurable filename/output template DSL
- Multiple concurrent sessions per repo
- Auto-trim excerpt to tightest subrange
- Copy-single-comment-as-prompt

**Anti-features (explicitly excluded):**
- Threaded replies, severity tags, approve/request-changes state machine, re-anchoring on history rewrite, GitHub/GitLab posting, auto-generated review findings

### Architecture Approach

The feature slots into Trunk's existing inner-fn / managed-state / shared-$state-rune architecture with two new Rust modules (`commands/review.rs` thin wrappers, `git/review.rs` pure logic), one new managed state entry (`ReviewSessionsState: Mutex<HashMap<String, ReviewSession>>`), one new Svelte state module (`review-session.svelte.ts`), one new Svelte panel component (`ReviewPanel.svelte`), and targeted modifications to `HunkView`, `FullFileView`, `CommitRow`, and `RepoView`. The split mirrors the established `commands/diff.rs` + `git/` pattern and makes all review logic unit-testable via the GOOS harness without a Tauri runtime.

The Review mode toggle replaces the right-pane content with `ReviewPanel` — the right pane already swaps between `CommitDetail` and `DiffPanel`, making this the smallest possible surface change. The center diff/full-file view remains intact as the anchor-capture surface.

**Major components:**
1. `git/review.rs` (NEW) — pure functions: revwalk range seeding, excerpt extraction (diff hunk slice + blob slice), markdown assembly, anchor resolution
2. `commands/review.rs` (NEW) — thin `#[tauri::command]` wrappers following the inner-fn pattern; all commands unit-testable
3. `ReviewSessionsState` (NEW) — `Mutex<HashMap<repo_path, ReviewSession>>` managed state; mirrors `CommitCache`; load-on-resume, flush on every mutation
4. `review-session.svelte.ts` (NEW) — shared `$state` rune module; per-tab factory (like `remote-state.svelte.ts`); ReviewPanel + HunkView + FullFileView + CommitRow all share it without prop drilling
5. `ReviewPanel.svelte` (NEW) — right-pane review surface: included-commit list, comment list with jump/edit/delete, generate-markdown button, copy/save output
6. `HunkView.svelte` / `FullFileView.svelte` (MODIFIED) — add "Add comment" action; selection adapter translates diff-array indices to stable source-line anchors at capture time
7. `CommitRow.svelte` (MODIFIED) — context-menu item "Add commit(s) to review"
8. Persistence: `app_data_dir()/review-sessions/<canonicalize+percent-encode(path)>.json` — `serde_json`, `schema_version: 1`, atomic tmp+rename

### Critical Pitfalls

1. **Persisting diff-array indices as anchors** — `selectedLineIndices: Set<number>` in v0.7 is a position in the in-memory diff array, NOT a source line number. Persisting it breaks every anchor on re-fetch, option change, or restart. Translate to `(commit_oid, file_path, side, start_line, end_line)` via `old_lineno`/`new_lineno` at capture time. Never put `hunk_index` or `line_index` in the anchor struct.

2. **Treating full-file view as a blob** — The v0.12 full-file view is a `diff` with `context_lines(100_000)`, not a blob read. It does not include context past the last change; unchanged files produce zero hunks. Render-time excerpts for `source=full_file` must come from a NEW git2 `commit.tree().get_path(file).to_object().peel_to_blob()` read.

3. **Null `lineno` in mixed selections** — Diff lines are origin-tagged: `Add` has `new_lineno` only, `Delete` has `old_lineno` only. A selection spanning both yields `null` on the wrong side. Require a `side` discriminator on the anchor; derive line numbers from the matching side only.

4. **Non-atomic session writes and unstable repo keying** — LazyStore's read-modify-write is not atomic; concurrent same-repo tabs lose updates. Key derived from raw path string breaks on symlinks or `..` segments. Use Rust mutex-serialized writes + atomic tmp+rename; canonicalize the repo path before keying.

5. **Missing `dialog:allow-save` capability** — `capabilities/default.json` grants `dialog:allow-open`, `dialog:allow-ask`, and `clipboard-manager:allow-write-text` but NOT `dialog:allow-save`. Add it in Phase 7; verify in a release build.

6. **Stale anchors crashing render** — Every resolution step must return `Result`; on failure, emit the comment into an `## Unresolvable anchors` section. Never `unwrap`. Never drop a comment.

7. **Hardcoded 3-backtick fences** — Use `max(3, longest_backtick_run_in_excerpt + 1)` for fence length. Hardcoded triple-backtick fences break on any excerpt containing a triple-backtick.

## Implications for Roadmap

Based on combined research, the natural phase structure follows strict dependency order. Phases 3 and 4 are independent and can be swapped or parallelized. Phases 6 and 7 may merge.

### Phase 1: Data Model + Persistence + Session Lifecycle
**Rationale:** The anchor schema gates every subsequent phase. Getting it wrong is a high-cost schema migration; getting it right makes everything else straightforward. All critical pitfalls manifest as design decisions here.
**Delivers:** `ReviewSession` / `ReviewComment` / `Anchor` Rust structs + TS mirrors; `ReviewSessionsState` managed state; per-repo JSON file in `app_data_dir` (canonicalized path key, `schema_version: 1`, atomic write); `start_review` / `resume_review` / `end_review` / `save_review` commands; monotonic ID counter.
**Avoids:** Pitfalls 1, 3, 4 (correct anchor struct, blob-read strategy decided, canonicalized path key, atomic write, schema_version from day 1).

### Phase 2: Commit Selection
**Rationale:** Defines the review scope. Depends on Phase 1. Decides merge-commit policy.
**Delivers:** `seed_review_range` (git2 revwalk); `add_commits_to_review` from graph context menu (`CommitRow` modified); ReviewPanel skeleton showing included-commit list.

### Phase 3: Diff-Source Anchor Capture
**Rationale:** Primary review interaction — commenting on what changed. Adapts v0.7 selection to stable anchors. Independent of Phase 4.
**Delivers:** Selection adapter in `HunkView` (indices → `Anchor::Diff{side, old_range, new_range}`); "Add comment" action; `add_comment` command with attach-time excerpt caching.
**Avoids:** Pitfalls 1, 2, 3.

### Phase 4: Full-File-Source Anchor Capture
**Rationale:** Adapts v0.12 full-file view for comment anchoring. Independent of Phase 3.
**Delivers:** Line-range selection state in `FullFileView` (net-new); "Add comment" action producing `Anchor::FullFile{start_line, end_line}`; shares `add_comment` command from Phase 3.
**Avoids:** Pitfall 2.

### Phase 5: Comment Management UI
**Rationale:** Makes accumulated review visible and actionable. Depends on at least one anchor type (Phase 3 or 4).
**Delivers:** ReviewPanel comment list with edit/delete/jump-to-anchor; optional commit-level comments; delete confirmation; orphaned-anchor jump shows read-only state with reason badge.
**Avoids:** Pitfall 6.

### Phase 6: Excerpt Resolution + Markdown Render
**Rationale:** The primary deliverable. Requires all anchor types. Both excerpt strategies (diff hunk slice and blob slice) implemented here.
**Delivers:** `resolve_excerpt` (git2 blob slice for `FullFile`; diff hunk re-run for `Diff`); `render_review_markdown` (preamble, file-grouped/line-ascending ordering, `path:Lstart-Lend (sha)` headings, short excerpts, diff-fence vs language-fence by `source`, verbatim comment text, commit-level notes, `## Unresolvable anchors` section); dynamic fence-length; binary-file placeholder; CRLF normalization.
**Avoids:** Pitfalls 2, 5, 6, 7.

### Phase 7: Output (Clipboard + Save-to-File)
**Rationale:** Exposes the rendered artifact. Depends on Phase 6. May merge with Phase 6.
**Delivers:** Copy-to-clipboard (await + toast, NOT fire-and-forget); save-to-file (`save()` → null-check → custom Rust `std::fs` write with atomic tmp+rename); `dialog:allow-save` added to `capabilities/default.json`; filename convention `trunk-review-<repo-name>-<YYYYMMDD-HHMM>.md`.
**Avoids:** Pitfall 5.

### Phase Ordering Rationale

- Phase 1 first because the anchor schema gates everything. A wrong schema is the highest-cost mistake in the feature.
- Phases 3 and 4 can be swapped or parallelized — they share the `add_comment` command but touch different source views.
- Phase 5 after at least one anchor type so the panel has data to display.
- Phase 6 after all anchor types — must handle both `Diff` and `FullFile` variants.
- Phase 7 after Phase 6; may merge with 6.

### Research Flags

Phases with standard patterns (skip research-phase):
- **Phase 1:** `CommitCache` managed-state pattern is the direct template; inner-fn and serde_json are proven project-wide.
- **Phase 2:** git2 revwalk is a standard API; context-menu modification follows v0.3/v0.5 patterns.
- **Phase 5:** Svelte 5 rune + component patterns are established; delete confirmation reuses v0.6 dialog.
- **Phase 7:** Clipboard and dialog usage follows existing patterns; `dialog:allow-save` is a one-line capability entry.

Phases that may benefit from targeted research during planning:
- **Phase 3:** The selection adapter (diff-array indices → stable line numbers) is the most mechanically novel piece. Review `HunkView` selection state and `staging.rs` `stage_lines` before writing the plan.
- **Phase 6:** Excerpt extraction and markdown assembly are new logic without a direct predecessor. Review `diff.rs` walk_diff and `git/syntax.rs` before planning the render functions.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Verified against installed package sources (`node_modules/`, `~/.cargo/registry/`, `Cargo.toml`). Zero new dependencies confirmed. |
| Features | HIGH | Markdown-for-AI format grounded in real artifacts (openai/codex, reviewprompt). Session UX extrapolated from PR-tool patterns — MEDIUM for edge-case UX details. |
| Architecture | HIGH | Every integration point grounded in actual Trunk source files. Component boundaries verified against `src-tauri/src/` and `src/components/`. |
| Pitfalls | HIGH | Critical pitfalls verified against actual codebase: `selectedLineIndices` at `DiffViewer.svelte:27`, `HunkView.svelte:305`, `staging.rs:776`; full-file = 100k-context diff at `diff.rs:33-38`; capability gap confirmed in `capabilities/default.json`. |

**Overall confidence:** HIGH

### Gaps to Address

- **Merge-commit policy:** Two options (exclude from `diff` source vs. blob-read sidestep for `full_file` source). Decide in Phase 2 planning.
- **Draft comment slot:** A transient draft slot or debounced autosave in CommentEditor protects in-progress comments. Decide in Phase 1 — either a `draft_comment` field on the session or component-level persistence.
- **Same-repo multi-tab live coordination:** Rust mutex serializes writes but live updates across duplicate tabs require a `session-changed` Tauri event or tab-reload strategy. Flag for Phase 1 planning.
- **Windows path length:** Canonicalize + percent-encode can exceed 260 chars on Windows for deeply-nested repos. Needs a concrete truncation strategy if Windows support is a target.

## Sources

### Primary (HIGH confidence)
- Trunk source: `diff.rs`, `staging.rs`, `DiffViewer.svelte`, `HunkView.svelte`, `FullFileView.svelte`, `store.ts`, `capabilities/default.json`, `state.rs`, `lib.rs` — codebase audit confirming selection model, full-file = diff, LazyStore pattern, existing capabilities
- `node_modules/@tauri-apps/plugin-clipboard-manager/dist-js/index.d.ts` — confirmed `writeText`, v2.3.2
- `node_modules/@tauri-apps/plugin-dialog/dist-js/index.d.ts` — confirmed `save(): Promise<string | null>`, v2.6.0
- `~/.cargo/registry/.../tauri-2.10.2/src/path/desktop.rs` — confirmed `app_data_dir()` on `Manager::path()`
- `.planning/PROJECT.md` — v0.13 key decisions

### Secondary (MEDIUM confidence)
- [openai/codex review_prompt.md](https://github.com/openai/codex/blob/main/codex-rs/core/review_prompt.md) — AI review format: imperative instruction, `line_range`, 5-10 line max excerpts
- [dyoshikawa/reviewprompt](https://github.com/dyoshikawa/reviewprompt) — `./path:Lstart-Lend` heading convention confirmed
- [raf.xyz: How I keep up with AI-generated PRs](https://www.raf.xyz/blog/03-how-i-keep-up-with-ai-generated-prs) — file-grouped/line-sorted ordering confirmed
- Tauri 2 dialog plugin docs — `dialog:allow-save` permission identifier; `save()` returns `null` on cancel

---
*Research completed: 2026-05-25*
*Ready for roadmap: yes*
