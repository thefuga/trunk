# Pitfalls Research

**Domain:** Persisted, line-anchored review-comment + AI-targeted markdown export in a Tauri 2 + Svelte 5 + git2 desktop Git GUI (Trunk v0.13 Code Review Mode)
**Researched:** 2026-05-25
**Confidence:** HIGH (grounded in the actual Trunk source: `src-tauri/src/commands/diff.rs`, `staging.rs`, `src/components/diff/*.svelte`, `src/lib/store.ts`, `src-tauri/capabilities/default.json`, verified against Tauri 2 docs)

> **Build-phase vocabulary used below:** `foundation` (session model, schema, persistence, repo keying), `selection` (anchor capture from diff/full-file views), `render` (resolve anchors → excerpts → markdown), `output` (clipboard + save-to-file). These map to the four obvious phase groupings the roadmapper will create.

---

## The one finding that reframes everything

**The existing v0.7 line selection is `(hunk_index, Set<line_index_within_hunk>)` — a position in the in-memory diff array, NOT a source line number.** (`src/components/diff/DiffViewer.svelte:27` `selectedLineIndices: Set<number>`; `HunkView.svelte:305` keys on `selectedHunkKey === hunkKey && selectedLineIndices.has(lineIdx)`; backend `stage_lines(hunk_index, line_indices: Vec<u32>)` at `staging.rs:776`.)

**The existing "full file at commit" view is NOT a blob read — it is a diff against the first parent with `context_lines(100_000)`** (`diff.rs:33-38` `if req.show_full_file { 100_000 }`; `FullFileView.svelte:49` `fd.hunks.flatMap(h => h.lines)`). Line numbers shown are `old_lineno`/`new_lineno` **from the diff**, which are **nullable** (`HunkView.svelte:327` `{line.new_lineno ?? ''}`) and **do not include context past the last change** (libgit2 only emits context around hunks). An unchanged file produces zero hunks and renders nothing.

Both selection sources therefore hand you **diff-relative, option-dependent, ephemeral coordinates**. If you persist those directly as anchors, every anchor breaks the moment the diff is re-fetched with different `context_lines`/`ignore_whitespace`, on restart, or for full-file views of large/binary files. **The foundation phase must define an anchor schema that converts diff-relative selection into a stable `(commit_oid, file_path, side, start_line, end_line, source)` tuple at capture time, and the render phase must re-derive excerpts from git2 — never trust the in-memory diff array to still exist.**

---

## Critical Pitfalls

### Pitfall 1: Persisting the diff-array index instead of a stable source-line anchor

**What goes wrong:**
An anchor is saved as `(hunk_index, line_index)` (the only thing v0.7 selection currently produces). After restart, an option change, or a re-fetch, the diff array is regenerated with different length/ordering, and the anchor points at the wrong line or out of bounds — silently mis-attributing every comment.

**Why it happens:**
The path of least resistance is to reuse v0.7's `selectedLineIndices: Set<number>` verbatim. It works in the same session because the array is identical. The breakage only appears across re-fetch/restart.

**How to avoid:**
At capture time, translate the selected diff-array positions into source line numbers via the `DiffLine.old_lineno`/`new_lineno` already carried in each line. Persist `(commit_oid, file_path, side, start_line, end_line, source)`. Never persist `hunk_index`/`line_index`. Diff options (`context_lines`, `ignore_whitespace`) must not appear in the anchor.

**Warning signs:**
Anchor struct contains `hunk_index` or `line_index`. A comment "jumps" to a different line after toggling whitespace-ignore or context-line count. Out-of-bounds panics when re-opening a session.

**Phase to address:** foundation (schema), selection (capture/translation)

---

### Pitfall 2: Off-by-one and null `lineno` — collapsing a mixed selection to one line range

**What goes wrong:**
A selection spanning Add + Delete + Context lines is collapsed to a single `(start, end)` tuple, but Delete lines have `new_lineno = null` and Add lines have `old_lineno = null` (`HunkView.svelte:327` renders `?? ''`). Picking the wrong side yields `null`, an off-by-one, or a range that doesn't exist in the blob you later read.

**Why it happens:**
Developers assume every line has both numbers. git2 diff lines are origin-tagged: `Context` has both, `Add` has only `new_lineno`, `Delete` has only `old_lineno` (`diff.rs:222-238`).

**How to avoid:**
Add a `side ∈ {old, new}` discriminator to the anchor (in addition to `source ∈ {diff, full_file}`). Derive line numbers from the matching side only: `new` side uses `new_lineno`, skipping Delete lines; `old` side uses `old_lineno`, skipping Add lines. For diff selections that mix both, default the anchor to the **new** side (the post-change state the AI should act on) and drop pure-Delete lines from the line range, but keep them in the rendered excerpt as `-` context. Reject/clamp empty ranges. Full-file selections are always `new` side.

**Warning signs:**
Anchor has a single line range with no `side`. Excerpt at render time is off by one or empty. Crash on `null` lineno. AI feedback references a line number that doesn't exist in the file.

**Phase to address:** foundation (schema: `side`), selection (capture logic)

---

### Pitfall 3: Treating "full file at commit" as a complete blob

**What goes wrong:**
The full-file view is a 100k-context diff, so (a) files with no changes vs. parent render nothing, (b) `new_lineno` skips at the boundary if a hunk is dropped, (c) very large files may be emitted as Binary by libgit2 and produce no lines, (d) content is `String::from_utf8_lossy` (`diff.rs:228`) — non-UTF8 bytes are already replacement chars, so byte-exact excerpts are impossible from this path. Anchors captured here can reference lines that the render-time blob read won't match.

**Why it happens:**
The name "full file view" implies the whole file is present. It isn't — it's a diff with a big context window, optimized for display, not for stable line addressing.

**How to avoid:**
Decide the render strategy in foundation and write it down: **render-time excerpts come from a NEW git2 tree→blob read** (`commit.tree()?.get_path(path)?.to_object()?.as_blob()`), NOT from re-issuing the 100k diff. The blob is the source of truth for `full_file` line numbers. The diff view at capture time is only used to *let the user pick* lines; the persisted anchor stores absolute blob line numbers on the `new` side. At render, read the blob at `commit_oid`, split on `\n`, slice `[start..=end]`. This decouples excerpts from diff-option drift entirely. (Trade-off of the rejected alternative — re-issuing the diff — is exactly the drift in Pitfall 1 plus the Binary/empty-file gaps above.)

**Warning signs:**
Render reuses `diff_commit_file_inner` to get excerpt text. Full-file anchors on an unchanged file resolve to nothing. Excerpt line count doesn't match the file. No new blob-read command in `commands/`.

**Phase to address:** foundation (decide blob-read strategy), render (implement git2 blob read)

---

### Pitfall 4: Non-UTF8, CRLF, and no-newline-at-EOF in excerpt extraction

**What goes wrong:**
Blob bytes are decoded assuming UTF-8 (corrupt output on Latin-1/binary-ish files); CRLF files get `\r` embedded in every excerpt line (visible junk in the markdown, throws off line counts if you split on `\r\n` vs `\n`); a final line with no trailing newline is dropped or mis-counted, shifting the end of the range.

**Why it happens:**
git2 blobs are raw `&[u8]`. Naive `String::from_utf8(content).unwrap()` panics or `from_utf8_lossy` silently mangles. Splitting strategy for line counting is rarely thought through.

**How to avoid:**
At blob read: detect binary up front (reuse the existing `is_binary` signal pattern from `FileDiff`, or git2 `Blob::is_binary()`) and refuse to excerpt binary blobs — emit a `[binary file, no excerpt]` placeholder. For text, decode with `from_utf8_lossy` but flag if replacement chars were introduced. Split lines on `\n` after stripping a single trailing `\r` per line (normalize CRLF→LF for display). Count lines consistently: N `\n` characters → N or N+1 lines depending on trailing newline — pick "lines = split('\n'), drop trailing empty if blob ends in \n" and use that everywhere (capture gutter, render slice). Preserve a no-final-newline file by not appending one.

**Warning signs:**
`.unwrap()` on `String::from_utf8`. `\r` visible in copied markdown. Last-line comments render one line short. Replacement chars (U+FFFD) in excerpts with no warning.

**Phase to address:** render (blob decode + line-split helper), with the line-counting convention fixed in foundation so capture and render agree

---

### Pitfall 5: Code excerpts containing ``` fences break the markdown code block

**What goes wrong:**
The reviewed code contains a fenced block (common in markdown files, docstrings, this very planning repo). Emitting it inside a 3-backtick fence terminates the block early; the AI sees broken structure and the rest of the excerpt leaks as prose.

**Why it happens:**
Hardcoding three backticks. Works until the first excerpt that itself contains ```` ``` ````.

**How to avoid:**
Scan each excerpt for the longest run of consecutive backticks; use `max(3, longest_run + 1)` backticks for the opening and closing fence. Choose the info string by source: `diff` selections → ```` ```diff ````; `full_file` selections → the language id from the existing v0.12 language detection (`syntax::extension_from_path`). Never indent the fence (indented fences are interpreted as code spans). Preserve exact indentation of the code inside — do not trim or re-indent.

**Warning signs:**
Fence length is a constant `"```"`. Copy a review of a markdown/docstring-heavy file and the output renders half as prose. Indentation lost in excerpts.

**Phase to address:** render (markdown generation)

---

### Pitfall 6: Stale anchors that no longer resolve — crash or silent drop instead of graceful surface

**What goes wrong:**
By explicit design there is NO re-anchoring. So at render time a `commit_oid` may be gone (history rewritten/GC'd), a `file_path` may not exist in that tree (rename/delete), or `start_line..end_line` may exceed the blob length. Naive code either panics (`.unwrap()` on `find_commit`/`get_path`) or silently omits the comment, losing review feedback the user wrote.

**Why it happens:**
The happy path (session built and rendered in one sitting on a stable range) always resolves. The unresolvable case only appears after a rebase/amend/branch-switch between session creation and render, which is easy to never hit in dev.

**How to avoid:**
Split the concern across two phases. **Foundation:** the schema must carry enough to *attempt* resolution and to *describe* a failure — store the resolved `file_path` at the anchored commit (post-rename), `commit_oid` (full, not short), `side`, line range, `source`, and the comment text independent of resolvability. **Render:** every resolution step returns `Result`; on failure, emit the comment into the markdown with an explicit `> [!warning] Unresolvable anchor: <reason>` block (commit not found / file not in tree / line range out of bounds) plus the original comment text and best-known location. Never `unwrap`. Never drop. A render must always succeed and always include every comment.

**Warning signs:**
`find_commit(...).unwrap()` or `get_path(...).unwrap()` in render. Rendered doc has fewer comments than the session panel shows. Render throws instead of producing a doc. No "unresolvable" branch in tests.

**Phase to address:** foundation (schema carries resolution inputs + comment independence), render (graceful degradation + warning blocks)

---

### Pitfall 7: Merge commits silently lose second-parent changes

**What goes wrong:**
`diff_commit_inner` diffs against `parent(0)` only (`diff.rs:410-414`). A comment anchored to a merge commit captures the first-parent diff exclusively; changes brought in from the second parent are invisible in selection and in the rendered excerpt — the AI reviews an incomplete picture.

**Why it happens:**
First-parent diff is the standard simplification; it's correct for linear history and quietly wrong for merges.

**How to avoid:**
Decide policy in selection phase and surface it: either (a) exclude merge commits from the seed/hand-pick selection (simplest, matches "single user reviewing AI-written code" where merges are rare), or (b) allow them but render-time blob excerpts still come from the merge commit's own tree (Pitfall 3 path sidesteps the parent-diff problem entirely, since blob read at the merge commit is complete). Option (b) is nearly free given the blob-read strategy — recommend (b) for full-file source and exclude merges only for `diff` source.

**Warning signs:**
Selection lets the user pick a merge commit with `>1` parent and shows a first-parent-only diff with no indication. Excerpt for a merge omits second-parent hunks.

**Phase to address:** selection (policy + UI constraint), render (blob read sidesteps it for full_file)

---

### Pitfall 8: Renamed / added / deleted files within the reviewed range

**What goes wrong:**
For a Renamed file, `delta.old_file().path() != delta.new_file().path()` — anchoring on the wrong path makes the render-time blob lookup fail. For an Added file there is no old-side content; for a Deleted file there is no new-side content. Selecting the impossible side yields an unresolvable anchor or empty excerpt.

**Why it happens:**
A single `path` field assumes the file has one stable name and both sides. git2 deltas carry separate old/new paths and a `DiffStatus` (already modeled: `diff.rs:191-199`, `DiffStatus::{Added,Deleted,Renamed,Copied}`).

**How to avoid:**
Store the path **as it exists at the anchored commit on the anchor's `side`** — for `new` side, `delta.new_file().path()`; for `old` side, `delta.old_file().path()`. Constrain selection by `DiffStatus`: Added files allow only `new`-side anchors; Deleted files allow only `old`-side; Renamed files store the new path with `new` side by default. Render reads the blob at `(commit, stored_path)` and falls back to the Pitfall-6 warning if absent.

**Warning signs:**
Anchor has one `path` with no awareness of rename. Render fails on a file that was renamed in the range. UI lets you anchor the `old` side of an Added file.

**Phase to address:** foundation (path-at-commit + side in schema), selection (DiffStatus constraints)

---

### Pitfall 9: Session persistence — lost updates, corrupt JSON, and repo keying

**What goes wrong:**
Three failures: (a) **Lost update** — the existing LazyStore pattern is read-modify-write (`store.ts:16-19` `addRecentRepo`: get → spread → set → save). Two tabs on the same repo (the v0.13 decision is *one session per repo*, and tabs can duplicate a repo — v0.9) racing this lose one tab's comments. (b) **Corrupt/partial JSON** — a crash mid-`save()` leaves a truncated file; next load throws and the whole session is gone. (c) **Repo keying** — `RepoState` keys by `PathBuf` as opened; symlinks, trailing slashes, and `..` segments produce different key strings for the same repo, so the session "disappears" when the repo is opened via a different path.

**Why it happens:**
LazyStore is convenient and already in use; its read-modify-write and non-atomic save are invisible until concurrency or a crash hits. Path canonicalization is easy to forget.

**How to avoid:**
(a) Serialize session writes through a **Rust command holding a mutex** (matches the existing `RepoState`/`RunningOp` managed-state pattern) rather than racing LazyStore from multiple tabs; or, if staying in LazyStore, reload-before-write and merge by anchor id. Coordinate live updates across tabs via the existing filesystem-watcher/event pattern or a `session-changed` event. (b) **Atomic write**: serialize to a temp file, then rename over the target (`std::fs::rename` is atomic on same volume). (c) **Canonicalize the repo path** (`std::fs::canonicalize` / `dunce` on Windows) before using it as the session key; store sessions in the app data dir keyed by the canonical path hash, per the v0.13 decision ("app data dir, keyed by repo — not `.git/`, not the working tree").

**Warning signs:**
Session writes go straight to LazyStore from Svelte with no merge. No temp-file+rename. Key derived from raw `path` string. Comments vanish when the same repo is opened twice or via a symlink.

**Phase to address:** foundation (storage location, keying, atomic write, write-serialization)

---

### Pitfall 10: Schema with no version field

**What goes wrong:**
The anchor/comment format will evolve (you'll add `side`, then maybe a category, then a resolved-path cache). Without a version field, the loader can't tell v1 from v2 and either crashes on old sessions or silently misreads fields.

**Why it happens:**
"It's just JSON, I'll add versioning later." Later is after users (you) have persisted v1 sessions.

**How to avoid:**
Put `schema_version: 1` in the session file from the first commit, even with zero migrations. On load, switch on the version; unknown future version → refuse gracefully with a message, never partial-parse. serde `#[serde(default)]` on new optional fields eases additive migrations.

**Warning signs:**
Session JSON has no version. Adding a field breaks loading old sessions in testing.

**Phase to address:** foundation

---

### Pitfall 11: Save-to-file uses a capability/permission that isn't granted

**What goes wrong:**
`capabilities/default.json` currently grants only `dialog:allow-open`, `dialog:allow-ask`, and `clipboard-manager:allow-write-text` — **no `dialog:allow-save` and no `fs:` write permission**. Calling the save dialog or writing the file fails at runtime with a permission error that's easy to misread as a logic bug.

**Why it happens:**
Tauri 2's capability model denies by default; the missing permission only surfaces when the new code path runs, not at build time.

**How to avoid:**
Pick ONE save strategy and grant exactly its permissions:
- **Recommended (matches Trunk's "git2/std for local writes, plugins for UI" pattern):** `dialog:allow-save` to pick the path (returns `null` on cancel — verified against Tauri 2 dialog docs), then write via a **custom Rust command using `std::fs`** with atomic temp+rename. This needs only `dialog:allow-save` added to capabilities — no `fs:` plugin scope to configure.
- Alternative: `dialog:allow-save` + `@tauri-apps/plugin-fs` `writeTextFile` with an explicit fs scope. Two permissions plus scope config; rejected as more surface area for no benefit.

Add `dialog:allow-save` to `capabilities/default.json` in the output phase and verify the dialog actually opens in a built (not just dev) binary.

**Warning signs:**
Save throws a permissions/denied error. Code calls `save()` or `writeTextFile` with no matching capability entry. Works in dev with `withGlobalTauri` but fails in release.

**Phase to address:** output

---

### Pitfall 12: Save-dialog cancel and clipboard failure handled as success

**What goes wrong:**
(a) Save dialog cancel returns `null`; code that doesn't check writes to `null`/empty path or shows a false "Saved!" toast. (b) Clipboard `writeText` is fire-and-forget across the codebase (`.catch(() => {})` — `CommitGraph.svelte:651`, `CommitDetail.svelte:72`). For an *artifact the user intends to paste into an AI*, a silent clipboard failure means they paste stale content and never know.

**Why it happens:**
The existing pattern copies low-stakes strings (a SHA, a path) where silent failure is fine. The review artifact is the product — silence is a trap.

**How to avoid:**
After `save()`, branch on `null` → no-op (user cancelled), do not toast success. For clipboard, await `writeText` and toast success on resolve / **error toast on reject** (use the existing v0.6 toast system) — do not copy the fire-and-forget `.catch(() => {})` pattern here.

**Warning signs:**
"Saved" toast appears after cancelling the dialog. Clipboard call is `writeText(doc).catch(() => {})`. No success/failure feedback on copy.

**Phase to address:** output

---

### Pitfall 13: Losing in-progress comments and selection on diff re-fetch

**What goes wrong:**
(a) A comment being typed is lost when the user clicks another file/commit, switches view mode, or the filesystem watcher fires a `repo-changed` re-fetch (Trunk auto-refreshes on fs changes — v0.1). (b) The diff re-fetch replaces the in-memory array, clearing `selectedLineIndices` (it's transient Svelte `$state`), so an in-progress selection vanishes before the user attaches a comment.

**Why it happens:**
Selection and draft text live in component-local `$state`, which is reset on remount/refetch. The watcher and view-mode toggles trigger refetch underneath the user.

**How to avoid:**
Persist the draft comment to the session (or a transient draft slot) on blur/change, not only on explicit save. Once a selection is converted to an anchor, it is persisted immediately and survives refetch (it no longer depends on the diff array — Pitfall 1). Guard destructive refetches while a comment editor is open, or re-open the editor with its draft after refetch. Confirm before discarding an unsaved draft.

**Warning signs:**
Typing a comment, an external file change fires, and the text is gone. Selection clears on view-mode toggle. No draft persistence.

**Phase to address:** selection (anchor-immediately + draft persistence), foundation (draft slot in schema)

---

### Pitfall 14: No delete confirmation; no jump-to-anchor when anchor is stale

**What goes wrong:**
(a) Comment delete with no confirmation (the codebase has a confirmation pattern for discard — v0.6/v0.7 — so its absence here would be inconsistent and destructive). (b) "Jump to anchor" assumes the anchor resolves; if the commit/file is gone (Pitfall 6) the jump throws or navigates nowhere.

**Why it happens:**
Delete is a one-liner; confirmation is extra work. Jump-to-anchor is written for the happy path.

**How to avoid:**
Reuse the existing confirmation-dialog pattern for delete. Jump-to-anchor must check resolvability first and, when unresolvable, show the comment in a read-only "orphaned" state in the panel instead of navigating.

**Warning signs:**
One-click irreversible delete. Jump-to-anchor errors on a rewritten range.

**Phase to address:** selection/management (delete confirm), render+management (orphaned-anchor handling)

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Persist `(hunk_index, line_index)` directly from v0.7 selection | Reuses existing selection state as-is | Every anchor breaks on re-fetch/restart/option change | **Never** — this is the core correctness bug (Pitfall 1) |
| Extract excerpts by re-running the 100k-context diff | Reuses `diff_commit_file_inner`, no new command | Drift, empty unchanged files, Binary gaps, lossy UTF-8 (Pitfall 3/4) | **Never** for excerpts — diff is fine for *display during capture* only |
| Single `path` field on the anchor (no side/rename awareness) | Simpler struct | Renamed/added/deleted files resolve wrong or empty (Pitfall 8) | Never |
| Write session JSON directly via LazyStore from Svelte | Matches existing `store.ts` pattern | Lost updates across same-repo tabs; corrupt-partial on crash (Pitfall 9) | Acceptable only if single-tab-per-repo is enforced AND atomic write is added |
| Hardcode 3-backtick fences | Trivial | Breaks on code containing fences (Pitfall 5) | Only if you guarantee no fenced content ever — you can't |
| Omit `schema_version` | One less field | No clean migration path; old sessions crash loader (Pitfall 10) | Never — costs nothing to add |
| `find_commit(...).unwrap()` in render | Shorter code | Render crashes on rewritten history (Pitfall 6) | Never in render |
| Fire-and-forget clipboard `.catch(() => {})` for the artifact | Consistent with existing copy-SHA code | User pastes stale content unknowingly (Pitfall 12) | Never for the review artifact (fine for copy-SHA) |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| v0.7 line selection (`selectedLineIndices: Set<number>` + `hunk_index`) | Persisting the index as the anchor | Translate to `(commit, file, side, start_line, end_line)` at capture via `old/new_lineno` |
| v0.12 full-file view (100k-context diff, not a blob) | Treating it as the whole file / stable line numbers | Capture for selection only; render excerpts from a NEW git2 tree→blob read |
| git2 `diff_commit_inner` (first-parent only, `diff.rs:410`) | Anchoring merge commits and trusting the diff | Blob-read at the commit's own tree; constrain or document merge handling (Pitfall 7) |
| git2 blob bytes (`&[u8]`, may be non-UTF8 / CRLF / no trailing `\n`) | `String::from_utf8(...).unwrap()` and split naively | Binary-check, `from_utf8_lossy` with flag, normalize CRLF, fixed line-count convention |
| Tauri 2 capabilities (deny-by-default; only open/ask/clipboard granted today) | Calling `save()`/fs write with no permission | Add `dialog:allow-save`; write file via custom Rust `std::fs` command (Pitfall 11) |
| Tauri 2 `dialog.save()` | Treating `null` (cancel) as a path | Branch on `null`; no success toast on cancel (Pitfall 12) |
| Tauri 2 IPC for excerpts | Returning huge per-line span-enriched payloads for the whole render | Render builds the markdown string in Rust and returns one string, not a giant struct |
| LazyStore (`store.ts` read-modify-write, non-atomic save) | Multi-tab same-repo writes | Serialize through a mutex-held Rust command + atomic temp+rename |
| `RepoState` PathBuf keying | Keying session by raw opened path | Canonicalize path before keying (symlinks/`..`/trailing slash) |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Rendering excerpts with full syntax/word-span enrichment (the v0.12 `enrich_file_diffs` path) | Slow render, huge payload | Excerpts are plain text in a fenced block; AI doesn't need spans. Skip enrichment entirely for export | Any session with many anchors |
| Re-reading the blob per line instead of per file | O(lines) tree lookups | Read each `(commit, file)` blob once, cache, slice all its anchors' ranges | Files with many anchors |
| Including full file content per anchor in the markdown | Doc balloons past clipboard/AI context limits | Excerpt only the anchored line range (+ small fixed context, e.g. 2 lines); summarize commit refs once at top | Large files / many anchors |
| Loading every repo's session at startup | Slow launch | Lazy-load the session only for the active repo (matches LazyStore lazy pattern) | Many repos with saved sessions |
| Unbounded doc size copied to clipboard | Copy silently truncated/failed on very large docs | Show doc size; warn past a threshold; offer save-to-file as the path for large docs | Reviews spanning many commits/files |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Save-to-file path not validated / fs scope too broad | Write outside intended dir if using broad `fs:` scope | Use `dialog:allow-save` to get a user-chosen path + custom Rust write; avoid a wide `fs:scope` |
| Session files in the working tree or `.git/` | Private review drafts leak into commits / shared repo | Store in app data dir per v0.13 decision; never in repo |
| Reading arbitrary blob path from anchor without bounding to the repo | Path traversal via crafted session file | Resolve paths only through `commit.tree().get_path()` (tree-relative); never `std::fs` the working tree by anchor path |
| Trusting persisted session JSON without schema/version validation | Malformed/old file crashes or mis-parses | Version + serde validation on load (Pitfall 10) |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| In-progress comment lost on file switch / fs refresh | Lost work, frustration | Persist drafts on change; anchor-immediately so selection survives refetch (Pitfall 13) |
| Selection cleared by view-mode toggle / re-fetch | Have to re-select before commenting | Convert selection to a persisted anchor at attach time, independent of diff array |
| No confirmation on comment delete | Accidental loss of written feedback | Reuse existing confirmation dialog (Pitfall 14) |
| Silent clipboard / false save-success | User pastes stale or saves nothing, unaware | Await + toast success/failure; handle cancel as no-op (Pitfall 12) |
| Orphaned anchor jump throws / goes nowhere | Confusing dead UI | Show orphaned comments read-only with a reason badge (Pitfall 6/14) |
| Markdown unreadable to AI (broken fences, wrong/absent line numbers) | AI can't act on feedback — defeats the feature's purpose | Dynamic fence length, correct file path + line numbers per excerpt, diff vs language info-string (Pitfall 5) |

## "Looks Done But Isn't" Checklist

- [ ] **Anchor schema:** Often missing `side` and `source` discriminators and the path-at-commit — verify a Renamed-file and a Delete-line-only selection both round-trip.
- [ ] **Render resolution:** Often missing the unresolvable branch — verify rendering a session after `git commit --amend`/rebase still produces a doc with warning blocks for every orphaned comment, and never panics.
- [ ] **Full-file source:** Often silently reuses the diff path — verify excerpts come from a git2 blob read and match the file exactly (including an unchanged file and a file with no trailing newline).
- [ ] **Fence escaping:** Often hardcoded — verify a review of a file containing a ```` ``` ```` fence renders intact.
- [ ] **Capabilities:** Often missing — verify `dialog:allow-save` is in `capabilities/default.json` and save works in a *release* build, not just dev.
- [ ] **Cancel/failure paths:** Verify cancelling the save dialog shows no success, and a forced clipboard failure shows an error toast.
- [ ] **Persistence robustness:** Verify a session survives restart, the same repo opened via symlink/different path resolves the same session, and a kill mid-save doesn't corrupt the file (atomic write).
- [ ] **Schema version:** Verify the file has `schema_version` and loading a hand-edited "future version" file degrades gracefully.
- [ ] **Binary files:** Verify a binary file in the range can't be anchored / renders a placeholder, reusing the existing `is_binary` signal.
- [ ] **Merge commits:** Verify the chosen policy (exclude or blob-read) actually holds for a merge commit in the range.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Persisted diff-index anchors (Pitfall 1) | HIGH | Schema change + migration; existing sessions' anchors are unrecoverable to exact lines — must re-anchor manually. Avoid by getting foundation right first. |
| Corrupt session JSON (Pitfall 9b) | MEDIUM | Atomic write prevents it; if it happens, keep a `.bak` of the last good save and offer restore on load failure |
| Wrong repo key after path change (Pitfall 9c) | LOW | Canonicalize and, on miss, scan for a session whose stored canonical path matches the now-canonical path |
| Broken fences in output (Pitfall 5) | LOW | Dynamic fence length is a localized render fix; regenerate the doc |
| Missing `dialog:allow-save` (Pitfall 11) | LOW | Add the permission, rebuild |
| Orphaned anchors crashing render (Pitfall 6) | LOW–MEDIUM | Wrap resolution in `Result`, emit warning blocks; no data loss since comment text is stored independently |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| 1. Diff-index vs stable anchor | foundation (schema) + selection (translate) | Anchor struct has no `hunk_index`; comment stays put after toggling whitespace/context and after restart |
| 2. Off-by-one / null lineno / mixed side | foundation (`side`) + selection | Delete-only and Add+Delete+Context selections round-trip to correct new-side lines |
| 3. Full-file ≠ blob | foundation (strategy) + render (blob read) | New git2 blob-read command exists; full-file excerpt matches an unchanged file exactly |
| 4. Encoding / CRLF / EOF | foundation (line convention) + render | CRLF file has no `\r` in output; last-line comment renders complete |
| 5. Fence breakage | render | File containing ```` ``` ```` renders intact with longer fence |
| 6. Stale anchors | foundation (schema carries inputs + comment) + render (degrade) | Post-rebase render produces doc with warning blocks, no panic, no dropped comment |
| 7. Merge first-parent loss | selection (policy) + render (blob sidesteps) | Merge-commit anchor behaves per chosen policy |
| 8. Rename/add/delete files | foundation (path-at-commit) + selection (status constraint) | Renamed-file anchor resolves; can't anchor old side of Added file |
| 9. Persistence (lost update / corrupt / keying) | foundation | Same-repo two-tab writes don't lose data; kill-mid-save leaves valid file; symlink path resolves same session |
| 10. No schema version | foundation | File has `schema_version`; future-version file degrades gracefully |
| 11. Missing save capability | output | `dialog:allow-save` present; save works in release build |
| 12. Cancel/failure as success | output | Cancel → no success toast; clipboard failure → error toast |
| 13. Lost drafts / selection | foundation (draft slot) + selection | Draft survives fs-refresh; selection survives view-mode toggle once attached |
| 14. No delete confirm / stale jump | selection/management + render | Delete confirms; jump on orphaned anchor shows read-only state |

## Sources

- Trunk source (primary, HIGH): `src-tauri/src/commands/diff.rs` (full-file = 100k-context diff `:33-38`, first-parent diff `:410-414`, `from_utf8_lossy` `:228`, `is_binary` `:190`), `src-tauri/src/commands/staging.rs` (`stage_lines(hunk_index, line_indices)` `:776`), `src/components/diff/{DiffViewer,HunkView,FullFileView,SplitView}.svelte` (`selectedLineIndices: Set<number>`, nullable `?? ''` linenos, `flatMap` over hunks), `src/lib/store.ts` (LazyStore read-modify-write `addRecentRepo` `:16-19`), `src-tauri/capabilities/default.json` (granted permissions), `src/components/{CommitGraph,CommitDetail}.svelte` (fire-and-forget `writeText().catch(() => {})`).
- Tauri 2 dialog plugin docs (HIGH): `dialog:allow-save` permission identifier; `save()` returns `null`/`None` on cancel — https://v2.tauri.app/plugin/dialog/
- `.planning/PROJECT.md` v0.13 key decisions (anchor = `(commit, file, line-range, source)`; one session per repo; no re-anchoring; app-data-dir storage; single comment per anchor).
- CommonMark fenced-code-block rules (fence length = longest inner run + 1; indented fences become code spans) — established markdown behavior, MEDIUM.

---
*Pitfalls research for: persisted line-anchored review-comment + AI-markdown-export in Trunk (Tauri 2 + Svelte 5 + git2)*
*Researched: 2026-05-25*
