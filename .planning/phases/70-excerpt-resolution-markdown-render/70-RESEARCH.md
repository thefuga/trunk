# Phase 70: Excerpt Resolution + Markdown Render — Research

**Researched:** 2026-05-26
**Domain:** Rust pure render (git2 0.19) + Svelte 5 panel swap
**Confidence:** HIGH

## Summary

Phase 70 is a small, well-bounded surface: one new pure Rust module
(`src-tauri/src/git/review.rs`), one new Tauri command in
`src-tauri/src/commands/review.rs` mirroring the existing `_inner` + thin wrapper
pattern, and a Svelte 5 in-panel view swap in `ReviewPanel.svelte` driven by an
added boolean on the existing `review-session.svelte.ts` rune. Every load-bearing
question CONTEXT.md surfaced has a concrete codebase answer; **no new
dependencies are required** for this phase.

**Primary recommendation:** build the pure render against a `&git2::Repository`
+ a `&ReviewSession` taken by reference, return `String`, and route every per-
comment resolution failure (including binary, blob-read failure, diff-replay
failure, no-hunks-for-anchor) into the unresolvable section using
`Comment.text` + `Comment.cached_excerpt`. The command lives one layer up; it
gates on `session.comments.is_empty()` (D-11) and is the only place that can
return `Err`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Trigger surface**
- **D-01:** Generate button in the ReviewPanel header. Lives next to the panel's
  existing controls. Disabled until the session has ≥1 comment with a tooltip
  ("Add at least one comment to generate"). Resolvable AND unresolvable
  comments both count toward enablement.
- **D-02:** Click → swap the panel to a markdown preview view. A panel-internal
  swap (comment-list view ↔ generated-doc view), mirroring the Phase 69 D-07
  right-pane swap pattern. The preview renders the markdown in a monospace,
  scrollable container; a "Back to comments" affordance returns to the list.
  Phase 71 attaches Copy/Save buttons to this same preview view — its shape
  must leave room for those controls.

**Doc anatomy**
- **D-03:** Top section: H1 title + short prose framing + commit refs list. H1
  reads "Code review: <repo-name>" (planner picks exact wording). Framing is
  one or two sentences explicitly telling the AI what the doc is — a human-
  authored code review with anchored excerpts; address each comment in context.
  Then the commit refs list.
- **D-04:** Section order: resolved per-file content → commit-level section →
  unresolvable section.
- **D-05:** Per-(file, commit) grouping. The same file in different commits
  gets its own group containing all anchors in that file at that commit,
  ordered by line (DOC-03).
- **D-06:** Within one anchor block: excerpt first, comment text after.

**Commit refs detail**
- **D-07:** Bullet list of `- <short-sha> — <subject>`. 7-char short SHA + the
  commit's subject line.
- **D-08:** 7-char short SHA EVERYWHERE — top refs list AND locked
  `path:Lstart-Lend (sha)` per-anchor headings (DOC-03) AND commit-level
  section's commit reference.

**Unresolvable + empty**
- **D-09:** Human-readable reason phrasing:
  - `CommitGone` → "commit no longer exists in the repository"
  - `FileGone` → "file no longer exists at this commit/side"
  - `LineOutOfRange` → "anchor line range is outside the current file bounds"
- **D-10:** Unresolvable excerpt fences by `Source` per DOC-02, labelled as
  cached at attach time (e.g., a one-line note above the fence: "Anchor no
  longer resolves; excerpt is the cached snapshot from attach time."). Diff-
  fenced for `Source::Diff`, language-fenced for `Source::FullFile`. The
  comment body is `cached_excerpt`.
- **D-11:** No zero-comment render path. The D-01 trigger gating is the
  contract; the Rust renderer is invoked only when the session has ≥1 comment.

### Locked Carry-forwards (do not re-litigate)
- **L-01:** Render in Rust, pure logic in `src-tauri/src/git/review.rs`,
  returning ONE markdown string.
- **L-02:** Re-resolution mechanics — `Source::FullFile` slices a fresh blob
  from `commit.tree().get_path(file).peel_to_blob()`; `Source::Diff` re-runs
  `diff_tree_to_tree(parent, commit)` and slices overlapping hunk lines
  keeping `-` lines per Phase 67 L-03.
- **L-03:** Fence length = `max(3, longest_backtick_run_in_excerpt + 1)`.
  Never indent the fence. Preserve exact indentation INSIDE the fence.
- **L-04:** Every resolution step returns `Result`. Never `unwrap`. On failure
  the comment routes to the unresolvable section using independently-stored
  `Comment.text` + `Comment.cached_excerpt`. Render NEVER crashes.
- **L-05:** Binary files → `[binary file, no excerpt]` placeholder. Detection
  mechanism is planner's discretion (see Item 1).
- **L-06:** Normalize CRLF → LF. Reconcile capture's line-counting convention
  with the renderer's. Fix the divergence in ONE place; document it.
- **L-07:** DOC-02 fencing: diff-fenced for `Source::Diff`, language-fenced
  for `Source::FullFile`.
- **L-08:** DOC-03 layout: comments grouped by file, ordered by line within a
  file/commit group, each under `path:Lstart-Lend (sha)`. Commit-level
  comments render in the trailing section.
- **L-09:** DOC-04: unresolvable section is mandatory, never silently dropped,
  never crashes the render.
- **L-10:** No syntax/word-span enrichment. The renderer does NOT call into
  `src-tauri/src/git/syntax.rs`. Plain text in a fence is what the AI needs.

### Claude's Discretion (delegated to planner)
- Module placement: introduce `src-tauri/src/git/review.rs`. Helpers for
  blob-slice and diff-replay-slice may live next to the render or in
  `review_store.rs` — keep them pure, no `tauri::*` imports.
- Tauri command surface: likely `generate_review_doc(path)` in
  `commands/review.rs`. No `session-changed` emit. No `mutate_session_rmw`.
- `OrphanReason` reuse vs render-side classifier — planner decision (see
  Item 7).
- File-group heading text + level — planner's call. H1 doc title → H2 per-
  (file, commit) → H3 per-anchor seems natural but is not locked.
- Exact prose framing text in D-03's preamble.
- Language detection for full-file language fences — extension-based mapping
  with `text` fallback.
- Line-counting reconciliation (L-06) — see Item 2.
- Panel preview view: new component file vs. inline branch — either is fine.
  Leave space for Phase 71's Copy/Save buttons.
- `review-session.svelte.ts`: add generate action + preview-state-vs-list-
  state discriminator.
- Empty / one-only states: D-11 lets the renderer assume ≥1 comment, but
  cases like "all comments unresolvable" or "only commit-level, no anchored"
  are valid — render with whatever sections apply; skip empty section
  headings.

### Deferred Ideas (OUT OF SCOPE)
- Clipboard / Save-to-file actions (Phase 71 / OUT-01, OUT-02).
- Re-anchoring on history rewrite (REQUIREMENTS Out of Scope).
- New comment metadata (severity, author, threading).
- Inline gutter badges on anchored diff/full-file lines.
- Per-line word-span / syntax-token enrichment in excerpts.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOC-01 | User can generate one markdown document from the session (commit refs + code excerpts + comments) | Pure renderer in `git/review.rs` + thin `generate_review_doc` Tauri command. Section ordering per D-04. Item 7 below specifies the wire shape. |
| DOC-02 | Excerpts render diff-fenced for diff-source, language-fenced for full-file-source, with fence length computed to avoid backtick collisions | Item 5 (fence-length formula) + Item 3 (language detection table). Diff fence: `` ```diff ``. Full-file fence: `` ```{language} `` from a small `.ext → name` table. |
| DOC-03 | Comments group by file and order by line, each under `path:Lstart-Lend (sha)`; commit-level in a trailing section | Per-(file, commit) grouping per D-05, sorted by `start_line` within a group. 7-char short SHA per D-08. |
| DOC-04 | Comments whose anchor can no longer be resolved render in a dedicated "unresolvable" section, never silently dropped and never crashing the render | `L-04` + Item 7 — `Result<Excerpt, RenderResolutionError>` per comment; failure routes to the unresolvable section using `Comment.text` + `Comment.cached_excerpt`. Phase 69's `classify_anchor` is the short-circuit gate. |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- **Stack:** Svelte 5 runes (`$state`, `$derived`, `$effect`); Vite 6; TypeScript 5.6 strict; Tailwind CSS 4.
- **Theme tokens only — no inline raw colors.** All preview-view styling uses
  `var(--color-*)` from `src/app.css`.
- **Layout discipline:** No positioning hacks; grid/flexbox so elements flow
  naturally.
- **git2-only for local reads — no shelling out.** The render path is pure
  git2 calls (`commit.tree().get_path(file).peel_to_blob()`,
  `diff_tree_to_tree(parent, commit)`, `DiffOptions::pathspec`).
- **Path aliases:** `$lib` → `src/lib` for frontend imports.
- **`just check` is the pre-commit gate** — runs fmt, biome, svelte-check,
  clippy, cargo-test, vitest.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Pure markdown rendering from session + repo | Backend (Rust, `git/review.rs`) | — | L-01 lock; the only place that can re-resolve excerpts against git2. |
| Excerpt re-resolution (blob slice / diff replay) | Backend (Rust, `git/review.rs`) | — | git2 is Rust-only; `git2::Repository` is not `Sync`, so it MUST be opened per-call inside the command. |
| Tauri IPC adapter (`generate_review_doc`) | Backend (`commands/review.rs`) | — | Mirrors `_inner` + thin wrapper pattern (CONVENTIONS line 20). No `mutate_session_rmw`; no `session-changed` emit (read-only). |
| D-11 zero-comment gate | Backend (command, not renderer) | — | Discretion: the command returns `TrunkError::new("no_comments", ...)`; the pure renderer assumes ≥1. |
| Generate-button trigger + disabled-state derivation | Frontend (`ReviewPanel.svelte`) | — | UI affordance; mirrors Phase 69 inline edit/delete affordances. |
| Preview view-state (list vs preview) | Frontend (`review-session.svelte.ts` rune) | Component | The rune already owns `rightPaneMode` view state — add a sibling flag for panel-internal mode. |
| IPC call adapter | Frontend (`lib/invoke.ts::safeInvoke`) | — | Already exists. Phase 70 just adds a call site. |

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `git2` | 0.19.0 [VERIFIED: Cargo.lock:1] | Blob slicing, diff replay, tree walk | Already pinned project-wide; no other Rust git library is in scope (CLAUDE.md "All git operations go through git2 crate"). |
| `serde` / `serde_json` | 1.x [VERIFIED: Cargo.toml:24-25] | Error wire shape (`TrunkError → String`) | Already used by every existing command. |
| `tauri` | 2.x [VERIFIED: Cargo.toml:22] | `#[tauri::command]` macro + `AppHandle::path().app_data_dir()` | Project framework. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tempfile` | 3.x [VERIFIED: Cargo.toml:39] | Test fixtures (`TempDir` + `Repository::init` + commit-with-file helper) | Backend unit tests for the pure renderer (mirrors `commands/review.rs:2049-2066`). |
| `tokio` | 1.x [VERIFIED: Cargo.toml:32] | `tauri::async_runtime::spawn_blocking` for git2 work in the command wrapper | Standard pattern — `git2::Repository` is not `Sync`. |
| `@testing-library/svelte` | — [VERIFIED: src/components/ReviewPanel.test.ts:1] | Component testing for ReviewPanel + preview swap | Project convention (every existing panel has a `.test.ts` alongside it). |
| `vitest` | — | TS unit + Svelte component tests | Per CLAUDE.md `just check` includes vitest. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled markdown emission | `pulldown-cmark` / `comrak` | NEW DEPENDENCY — rejected. We are EMITTING markdown (one direction, controlled), not parsing it. A `Write`-based string builder is simpler, smaller, and avoids supply-chain surface (Engineering Judgment §2: "Every new dependency needs a strong case"). |
| `String::from_utf8(blob.content().to_vec())` (strict) | `String::from_utf8_lossy` | The strict version errors on any non-UTF-8 byte. We want lossy + binary placeholder for non-text, not a hard error (L-04 + L-05). The classifier already chose lossy (`commands/review.rs:358`); mirror it. |
| `git2`-rs `Patch` API (lines as `String`s) | Manual blob slice + diff hunk walk | The `Patch` API does not preserve raw byte fidelity well; we already use `Diff::foreach` + `blob.content()` everywhere in the project (`commands/diff.rs`). Stay consistent. |

**Installation:** No new dependencies. All required crates and packages are
already pinned.

**Version verification:**
```bash
grep -A2 "^name = \"git2\"" src-tauri/Cargo.lock | head -3   # → 0.19.0
grep '"vitest"' package.json                                  # already present
```

## Package Legitimacy Audit

> Not applicable to this phase. **Zero external packages are added.** All
> required dependencies (`git2 0.19.0`, `serde 1`, `tauri 2`, `tempfile 3`,
> `tokio 1`, `vitest`, `@testing-library/svelte`) are already pinned in the
> project lockfiles and have been used in shipped phases. No
> `npm view` / `pip index versions` / `cargo search` / slopcheck step is
> warranted.

## Architecture Patterns

### System Architecture Diagram

```
                 ┌──────────────────────────────────────┐
                 │ ReviewPanel.svelte (Phase 69 host)   │
                 │                                      │
                 │  Header: [Generate] disabled?        │  D-01: button gated on
                 │     │  (comments.length >= 1)        │       comments.length>=1
                 │     ▼  click                         │
                 │  rune.showPreview()  ◀───────┐       │  D-02: panel-internal
                 │     │                        │       │       view swap
                 │     ▼                        │       │
                 │  safeInvoke("generate_review_doc",   │  TS → Rust IPC via
                 │      { path: repoPath })  ──────┐    │  safeInvoke<string>
                 │     │ awaits Result<String,…>   │    │
                 │     ▼                           │    │
                 │  rune.previewMarkdown = result  │    │
                 │     │                           │    │
                 │     ▼                           │    │
                 │  {#if previewMode}              │    │
                 │     <pre> preview </pre>        │    │
                 │  {:else}                        │    │
                 │     <ul> comment list </ul>     │    │
                 │  {/if}                          │    │
                 │     │                           │    │
                 │  [Back to comments] ──swap back─┘    │
                 └─────────────────────────────────┬────┘
                                                   │
                                                   │ Tauri IPC
                                                   ▼
        ┌──────────────────────────────────────────────────────────┐
        │ commands/review.rs::generate_review_doc                  │
        │                                                          │
        │ 1. canonical_repo_path(path, &state_map) ──── not_open?  │
        │ 2. spawn_blocking:                                       │
        │      a. load session (from in-memory ReviewSessionsState │
        │         by canonical key — same as list_session_comments)│
        │      b. session.comments.is_empty() → Err(no_comments)   │
        │              (D-11 gate)                                 │
        │      c. git2::Repository::open(&path)                    │
        │      d. review::render(&session, &repo) → String         │
        │ 3. Ok(doc_string)                                        │
        └────────────────────────────┬─────────────────────────────┘
                                     │
                                     ▼
       ┌─────────────────────────────────────────────────────────────┐
       │ git/review.rs::render(session, repo) -> String              │
       │ PURE — no tauri::*, no IPC, no async                        │
       │                                                             │
       │  For each comment in session.comments:                      │
       │    if anchor is Some:                                       │
       │       classify_anchor(anchor, repo)  ◀── reuse Phase 69     │
       │         ├ Ok(()) → attempt fresh excerpt re-resolution      │
       │         │            ├ Source::FullFile → blob_slice(...)   │
       │         │            └ Source::Diff     → diff_replay(...)  │
       │         │               ├ Ok(excerpt) → render anchor block │
       │         │               │   (D-06: excerpt then comment)    │
       │         │               └ Err(_)      → unresolvable+cached │
       │         └ Err(reason) → unresolvable + cached_excerpt       │
       │    else (commit-level comment):                             │
       │       commit_exists? → commit-level section                 │
       │                else → unresolvable (CommitGone)             │
       │                                                             │
       │  Assemble in D-04 order:                                    │
       │    H1 + framing + commit refs                               │
       │    → per-(file, commit) anchored sections                   │
       │    → commit-level section                                   │
       │    → unresolvable section                                   │
       └─────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src-tauri/src/
├── git/
│   ├── mod.rs                 # add: pub mod review;
│   ├── review.rs              # NEW — pure render + excerpt re-resolution helpers
│   ├── review_store.rs        # unchanged (Phase 65)
│   ├── syntax.rs              # unchanged; L-10 forbids invoking the highlighter
│   └── types.rs               # unchanged (Comment/Anchor/Source/Side schema)
└── commands/
    └── review.rs              # ADD: generate_review_doc + generate_review_doc_inner

src/
├── components/
│   ├── ReviewPanel.svelte     # ADD: Generate header button + preview view
│   ├── ReviewPanel.test.ts    # ADD: generate-button-disabled, preview-swap tests
│   └── ReviewDocPreview.svelte (optional)  # OR inline branch in ReviewPanel
└── lib/
    └── review-session.svelte.ts  # ADD: previewMode flag + previewMarkdown + actions
```

### Pattern 1: `_inner(data_dir, …)` + thin `#[tauri::command]` wrapper

**What:** Every command in `commands/review.rs` separates the logic-bearing
`_inner` (takes `&Path` + plain args, returns `Result<T, TrunkError>`) from the
thin `#[tauri::command]` wrapper (resolves `data_dir`, clones state, calls
`spawn_blocking`, serializes `TrunkError` to JSON string).

**When to use:** Every Tauri command. No exception.

**Example (lift from `commands/review.rs:891-916`, `list_session_comments`):**
```rust
#[tauri::command]
pub async fn generate_review_doc(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
) -> Result<String, String> {
    let state_map = state.0.lock().unwrap().clone();
    let canonical = canonical_repo_path(&path, &state_map)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;

    // Clone comments under the lock; never hold the lock across spawn_blocking.
    let session = {
        let map = sessions.0.lock().unwrap();
        map.get(&canonical)
            .ok_or_else(|| serde_json::to_string(&TrunkError::new(
                "no_session", "No active review session for this repository"
            )).unwrap())?
            .clone()
    };

    // D-11 gate: zero-comment is the command's contract violation, not the
    // renderer's. The pure renderer assumes >=1 and has no defensive branch.
    if session.comments.is_empty() {
        return Err(serde_json::to_string(&TrunkError::new(
            "no_comments",
            "Generate requires at least one comment in the session"
        )).unwrap());
    }

    let doc = tauri::async_runtime::spawn_blocking(
        move || -> Result<String, TrunkError> {
            let repo = git2::Repository::open(&path).map_err(TrunkError::from)?;
            Ok(crate::git::review::render(&session, &repo))
        }
    )
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    Ok(doc)
}
```

### Pattern 2: Pure render builds a `String` via `std::fmt::Write`

**What:** The renderer takes references and returns a `String`. No `Result`
return value (per L-04: resolution failures route INTO the document, not the
error channel). All formatting via `writeln!(out, "…")` against a `String`
buffer — no allocations beyond what the doc needs.

**Why:** Pure function = trivially testable (golden tests against synthesized
session+repo fixtures). Matches Engineering Judgment §3: "Seek the simplest
thing that could work."

### Anti-Patterns to Avoid

- **Building a per-line enriched payload and rendering in the frontend.**
  Locked out by L-01. The renderer returns ONE markdown string.
- **Calling into `syntax.rs::highlight_*`.** Locked out by L-10. Language fence
  tags come from a tiny `.ext → name` table inside `review.rs`.
- **Using `unwrap()` anywhere in `review.rs`.** Locked out by L-04. Every blob
  read / diff replay / slice MUST return `Result`; failures map to the
  unresolvable section.
- **Trying to share a `git2::Repository` across the lock boundary.**
  `git2::Repository` is not `Sync` (CONVENTIONS §git2 ownership) — open it
  fresh inside the `spawn_blocking` closure, never stash in state.
- **Emitting `session-changed` from the generate command.** Read-only path; an
  emit would trigger a reload storm on every Generate click.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Binary file detection | A custom NUL-byte scanner | `git2::Blob::is_binary()` | libgit2's heuristic checks the first ~8000 bytes for NUL; identical to what every text-aware tool does. Already used at `commands/diff.rs:190`. |
| Filter `diff_tree_to_tree` to one file | A post-walk pathspec filter on results | `DiffOptions::pathspec(file_path)` | libgit2 applies the filter during the walk — used at `commands/diff.rs:355,369,386`. |
| Markdown emission | A markdown AST builder | `String` + `writeln!()` | We emit, we don't parse. A library is pure overhead and a supply-chain surface. |
| Line counting | `lines().collect::<Vec<_>>()` + index | Direct `lines()` iteration with `enumerate()` | Avoids materializing the full Vec when most slices are small ranges. |
| Atomic file writes | Custom tmp+rename | Not applicable to Phase 70 | The doc is in-memory only; Phase 71 owns Save-to-File. |

**Key insight:** The phase touches three boundaries (git2, Tauri IPC, Svelte
runes) and each has an established codebase pattern. There is nothing to
hand-roll — every piece has a precedent in the same file or an adjacent one.

## Domain Investigation

### Item 1 — git2 binary-file detection (L-05)

**`git2::Blob::is_binary()` semantics, version 0.19** [CITED:
https://docs.rs/git2/0.19.0/git2/struct.Blob.html]

Signature: `pub fn is_binary(&self) -> bool`
Docs (verbatim): *"Determine if the blob content is most certainly binary or
not."*

The implementation calls libgit2's `git_blob_is_binary`, which scans the
**first ~8000 bytes** of the blob for a NUL byte. Presence of any NUL is
treated as "binary." This is the same heuristic git itself uses for the
"Binary files … differ" diff message. Already in use in this codebase at
`commands/diff.rs:190` (`delta.old_file().is_binary() || delta.new_file().is_binary()`).

**Interaction with L-05:** Use `blob.is_binary()` as the **primary** detector.
If it returns false but the UTF-8 lossy conversion of `blob.content()` then
produces a replacement-character path that breaks line slicing, that's a
secondary failure — route the comment to the unresolvable section (NOT the
binary placeholder; the user got a normal text-binary intermediate, which is
something the AI agent should see described, not silently swapped).

**What about non-UTF-8 bytes that survive `is_binary() == false`?** Files in
legacy encodings (Latin-1, Shift-JIS) often have no NUL bytes and will pass
`is_binary() == false`. `String::from_utf8_lossy` will replace invalid bytes
with U+FFFD and continue. `lines()` works on the resulting `Cow<str>`. **Do
NOT route these to unresolvable** — render the lossy excerpt; the AI agent
should see what's there.

**Recommendation:**
```rust
fn read_blob_text(blob: &git2::Blob<'_>) -> Result<String, ExcerptError> {
    if blob.is_binary() {
        return Err(ExcerptError::Binary);
    }
    Ok(String::from_utf8_lossy(blob.content()).into_owned())
}
```
Then the `Binary` variant maps to the `[binary file, no excerpt]`
placeholder; any subsequent slice failure maps to the unresolvable section.

**Confidence:** HIGH (docs verbatim + existing codebase pattern).

### Item 2 — Line-counting reconciliation (L-06)

**Critical correction to the CONTEXT.md framing.** I read the capture-side TS
code (`src/lib/diff-anchor.ts`, `src/lib/full-file-anchor.ts`). **TypeScript
does NOT do its own line counting.** Both files consume `DiffLine` objects
whose `new_lineno` / `old_lineno` fields come from libgit2's diff parser
(emitted by Rust in `commands/diff.rs:235-236` via `line.new_lineno()` /
`line.old_lineno()`). The TS adapter just picks min/max over those
already-computed line numbers (`diff-anchor.ts:69-73`,
`full-file-anchor.ts:50-52`).

So the actual divergence risk is **render-internal**, not capture↔render:
- **Phase 69 classifier** (`commands/review.rs:358`):
  `String::from_utf8_lossy(blob.content()).lines().count() as u32` — 1-based
  inclusive, no trailing-empty-line, `\r\n` collapses to one separator.
- **Phase 70 render must do the same.**

`str::lines()` semantics (Rust stdlib): splits on `\n`, strips an optional
preceding `\r` (so `\r\n` is one boundary), and does NOT emit a final empty
slice if the string ends with `\n`. Exactly what we want for "human line
count."

**Recommendation — lock ONE convention, document it, apply both at render's
blob slice and diff replay:**

```rust
/// L-06: ALL line-indexing in the renderer goes through this iterator.
/// 1-based inclusive bounds, `str::lines()` semantics (no trailing empty
/// line, `\r\n` collapses). Mirrors classify_anchor at commands/review.rs:358
/// so a comment that resolves at classification also resolves at render.
fn iter_lines(content: &str) -> impl Iterator<Item = &str> {
    content.lines()
}
```

**CRLF→LF normalization (L-06 second clause):** treat as **content
normalization** for what goes INSIDE the fence, not line-indexing
normalization. Two valid options for the planner:

| Option | What | Pro | Con |
|---|---|---|---|
| (a) Normalize before slicing | Replace `\r\n` with `\n` on the lossy string, then slice | Simpler downstream; the fence body is uniform | A second pass over the string before slicing |
| (b) Strip `\r` per emitted line | Emit `line.trim_end_matches('\r')` per slice line | One pass | Easier to misapply (forget to strip somewhere) |

**Recommendation:** Option (a). Do it once at the top of each excerpt
extraction; the slice indices are unchanged because `str::lines()` already
handles `\r\n` as one boundary. The body bytes inside the fence become LF-only,
which is what the AI consumer expects.

**Nothing to retrofit on the capture side** — CONTEXT.md's L-06 framing
("retrofit `buildDiffAnchor` / `buildFullFileAnchor` if needed") is moot.
State this explicitly in the planning doc so the planner doesn't add a
no-op TS task.

**Confidence:** HIGH (read both files; verified line-number provenance is
libgit2-side).

### Item 3 — Language detection for full-file fences (L-07)

**Codebase audit:** `src-tauri/src/git/syntax.rs` has NO `.ext → markdown-
language-tag` table. It has:
- `fn extension_from_path(path: &str) -> &str` (lines 60-65) — pure path
  helper, no syntect coupling. **Lift-able under L-10.**
- `fn fallback_extension(ext: &str) -> &str` (lines 41-46) — `"ts" | "mts" |
  "cts" | "tsx" | "jsx" | "svelte" | "vue" => "js"` — this maps to syntect
  syntax IDs, NOT to markdown fence language tags. **Do not lift this** — for
  example we want `ts` (not `js`) inside the fence so the AI agent gets the
  right hint.

**Recommendation — minimal hardcoded table in `review.rs`, fallback `text`:**

```rust
/// Extension → markdown fence language tag for `Source::FullFile` excerpts (L-07).
/// Distinct from syntax.rs's syntect-fallback table: we want the AI-facing
/// name, not a highlighter syntax id. `text` fallback ensures the fence is
/// always valid (no untagged ```; D-02 fences are always tagged).
fn fence_language(file_path: &str) -> &'static str {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs"                                 => "rust",
        "ts" | "mts" | "cts"                 => "typescript",
        "tsx"                                => "tsx",
        "js" | "mjs" | "cjs"                 => "javascript",
        "jsx"                                => "jsx",
        "svelte"                             => "svelte",
        "json"                               => "json",
        "md" | "markdown"                    => "markdown",
        "toml"                               => "toml",
        "yaml" | "yml"                       => "yaml",
        "css"                                => "css",
        "html" | "htm"                       => "html",
        "sh" | "bash"                        => "bash",
        "py"                                 => "python",
        "go"                                 => "go",
        _                                    => "text",
    }
}
```

This is a literal lookup table — no syntect, no library call, fully L-10
compliant.

**Confidence:** HIGH (read `syntax.rs` end-to-end).

### Item 4 — Diff replay-slice mechanics (L-02)

**Re-running `diff_tree_to_tree(parent, commit)` per anchored file is the
established pattern.** The codebase already does this for the full per-commit
diff at `commands/diff.rs:411-417`:

```rust
let diff = if is_head_unborn(&repo) {
    repo.diff_tree_to_tree(None, Some(&commit_tree), Some(&mut opts))?
} else {
    repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Some(&mut opts))?
};
```

**Filter to one file via pathspec** — same file, line 354-356:
```rust
let mut opts = git2::DiffOptions::new();
opts.pathspec(file_path);
```

**Walk hunks and lines via `diff.foreach`** — the same callback pattern at
`commands/diff.rs:182-243` emits `DiffLine { origin, content, old_lineno,
new_lineno, .. }`. The renderer needs only a subset:

```rust
/// L-02 + Phase 67 L-03: slice diff lines overlapping the anchor.
/// Keeps `-` lines per Phase 67 L-03; the excerpt body matches what the
/// capture-time cached_excerpt would have looked like.
fn slice_diff(
    repo: &git2::Repository,
    anchor: &Anchor,
) -> Result<String, ExcerptError> {
    let commit = repo.find_commit(git2::Oid::from_str(&anchor.commit_oid)?)?;
    // Side::Old on a root commit is FileGone (matches classify_anchor at
    // commands/review.rs:339-346 — already handled by the gate above).
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
    let commit_tree = commit.tree()?;

    let mut opts = git2::DiffOptions::new();
    opts.pathspec(&anchor.file_path);

    let diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&commit_tree),
        Some(&mut opts),
    )?;

    let mut out = String::new();
    diff.foreach(
        &mut |_d, _p| true,
        None,
        Some(&mut |_d, _h| true),
        Some(&mut |_d, _h, line| {
            // Pick the line-number on the anchor's side.
            let lineno = match anchor.side {
                Side::New => line.new_lineno(),
                Side::Old => line.old_lineno(),
            };
            // Overlap test: keep lines on the anchor's side within [start, end];
            // ALSO keep '-' (Delete) lines, which carry no new_lineno but appear
            // in the diff body — Phase 67 L-03.
            let in_range = lineno
                .map(|n| n >= anchor.start_line && n <= anchor.end_line)
                .unwrap_or_else(|| {
                    // Lines with no side-lineno: keep iff they're '-' on a
                    // New-side anchor or '+' on an Old-side anchor (the
                    // opposing-side change lines that visually anchor the
                    // range).
                    matches!(
                        (anchor.side, line.origin()),
                        (Side::New, '-') | (Side::Old, '+'),
                    )
                });
            if in_range {
                let origin = line.origin();
                let prefix = match origin {
                    '+' | '-' | ' ' => origin,
                    _ => ' ',
                };
                out.push(prefix);
                out.push_str(&String::from_utf8_lossy(line.content()));
            }
            true
        }),
    )?;
    Ok(out)
}
```

**One edge case CONTEXT.md doesn't address — file unchanged from parent.** If
the comment anchors a `Source::Diff` line in a file that is identical to its
parent at this commit (e.g. the user anchored on a line that was unchanged but
visible in the diff context, then later the file was reverted), the `pathspec`-
filtered diff emits zero hunks. The `classify_anchor` gate will say "resolves"
(file exists), but the slice produces an empty excerpt. **Recommended
disposition:** if `out.is_empty()` after the walk, return
`Err(ExcerptError::NoHunks)` and route to the unresolvable section with a reason
like "diff hunk no longer exists at this commit." The cached_excerpt fallback
covers the user — see Item 7 for the error type.

**Confidence:** HIGH (lifted directly from existing `commands/diff.rs` walker).

### Item 5 — Fence-length formula (L-03)

CONTEXT.md says `max(3, longest_backtick_run_in_excerpt + 1)`. The "longest
backtick run" is the **longest contiguous sequence of backtick bytes anywhere
in the excerpt body**, not the longest run on a single line and not lines
starting with backticks.

**Recommended one-liner:**
```rust
/// L-03: fence length = max(3, longest_contiguous_backtick_run + 1).
/// Scans the entire body byte-by-byte; line boundaries are irrelevant.
fn fence_length(body: &str) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for b in body.as_bytes() {
        if *b == b'`' {
            current += 1;
            if current > longest { longest = current; }
        } else {
            current = 0;
        }
    }
    std::cmp::max(3, longest + 1)
}
```

This is a linear scan; no allocation. Counter resets at every non-backtick byte
including newlines, so `` ``` `` followed by newline followed by `` ``` `` does
NOT compose into a longer run — which is the intent (those are two separate
fences in the body).

**Confidence:** HIGH (the formula is industry-standard; CommonMark §4.5 fenced
code blocks require an opening fence STRICTLY LONGER than any backtick run
inside the body).

### Item 6 — Markdown preview in Svelte 5 (D-02)

The preview is **a monospace, scrollable container showing the markdown string
as-is.** The user inspects the raw doc before Phase 71 ships Copy/Save. The
recipient of the doc is an AI agent — the user does not need a rendered HTML
preview. Per Engineering Judgment §3 ("Seek the simplest thing that could
work"), `<pre>` wrapped in a scrollable flex container is correct.

**Recommended shape:**

```svelte
<div class="preview-wrap">
  <header class="preview-header">
    <button onclick={() => session.showList()}>← Back to comments</button>
    <!-- Phase 71 will add Copy / Save buttons here. Leave the slot. -->
    <span class="preview-spacer"></span>
  </header>
  <pre class="preview-body">{previewMarkdown}</pre>
</div>

<style>
  .preview-wrap {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    background: var(--color-bg);
    color: var(--color-text);
  }
  .preview-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    font-size: 12px;
  }
  .preview-spacer { flex: 1; }
  .preview-body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    margin: 0;
    padding: 12px;
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre;       /* preserve all whitespace, do not wrap */
    background: var(--color-bg);
    color: var(--color-text);
  }
</style>
```

**Theme-token compliance:** all `var(--color-*)` — matches CONVENTIONS line
144 + ReviewPanel.svelte's existing style block (lines 519-657).

**Why `<pre>` not rendered HTML:** (a) the doc's recipient is an AI agent —
the user reviews the raw text the AI will see; (b) rendering markdown to HTML
needs a parser (new dependency); (c) Phase 71 ships Copy/Save and the user
copies the raw text, not HTML.

**Rune extension** (`review-session.svelte.ts`):
```ts
export interface ReviewSessionState {
  reviewActive: boolean;
  rightPaneMode: RightPaneMode;
  // ADD:
  panelMode: "list" | "preview";       // D-02 panel-internal view-state
  previewMarkdown: string | null;      // cached doc; null when no preview
}

// Add to the returned manager:
showList()    : void;                  // panelMode = "list"
showPreview() : void;                  // panelMode = "preview"
async generate(repoPath: string): Promise<void>;  // safeInvoke + set state
```

`endSession` must clear `previewMarkdown` and reset `panelMode` to `"list"` —
the doc is a static snapshot of the session that just ended, no reason to
keep it.

**Confidence:** HIGH (Svelte 5 rune file read; theme tokens checked against
CONVENTIONS).

### Item 7 — Tauri command return shape (L-04)

Existing review commands return `Result<T, String>` where `String` is a
JSON-encoded `TrunkError` (CONVENTIONS line 78: "Tauri command wrappers return
`Result<T, String>` — the `String` is a JSON-encoded `TrunkError`"). Mirror
that:

```rust
pub async fn generate_review_doc(path: String, ...) -> Result<String, String>;
```

**Error contract (the L-04 contract):**

| Failure | Where | Returns |
|---------|-------|---------|
| Repo not open | `canonical_repo_path` | `Err({code:"not_open",…})` |
| No session for repo | session lookup | `Err({code:"no_session",…})` |
| Zero comments (D-11 violation) | D-11 gate in the command | `Err({code:"no_comments",…})` |
| `Repository::open` fails | spawn_blocking | `Err({code: from git2 error, …})` |
| Anything per-comment (binary, blob read fail, slice fail, hunks-empty, CommitGone, FileGone, LineOutOfRange) | inside the renderer | **NEVER errors.** Routed to the unresolvable section in the returned `String`. |

**Internal error type — render-only, do NOT extend `OrphanReason`.**
`OrphanReason` is serialized to the frontend wire and consumed by
ReviewPanel.svelte (orphan badge labels at lines 46-50). Extending it risks
cascading TS-side churn for an internal-only concern. Instead:

```rust
// In git/review.rs — pure, never crosses the IPC boundary.
enum ExcerptError {
    /// blob.is_binary() == true — render the L-05 placeholder.
    Binary,
    /// Anchor classified as orphaned by classify_anchor (CommitGone /
    /// FileGone / LineOutOfRange). The classifier's reason maps to the
    /// D-09 user-facing string.
    Orphaned(OrphanReason),
    /// Blob read, slice, or diff replay failed for some other reason —
    /// route to unresolvable with a generic "could not be re-resolved"
    /// reason and the cached_excerpt fallback (L-04).
    ResolutionFailed,
    /// diff_tree_to_tree filtered to this file emitted zero hunks
    /// (file unchanged from parent at this commit). Route to unresolvable.
    NoHunks,
}
```

Mapping to D-09 user-facing strings:

| `ExcerptError` | User-facing reason in the markdown |
|---|---|
| `Orphaned(CommitGone)` | "commit no longer exists in the repository" |
| `Orphaned(FileGone)` | "file no longer exists at this commit/side" |
| `Orphaned(LineOutOfRange)` | "anchor line range is outside the current file bounds" |
| `ResolutionFailed` | "excerpt could not be re-resolved from the repository" |
| `NoHunks` | "diff hunk no longer exists at this commit" |
| `Binary` | (not unresolvable — renders the `[binary file, no excerpt]` placeholder INSIDE the resolved per-file section) |

**Confidence:** HIGH (read CONVENTIONS + existing commands).

### Item 8 — `safeInvoke` integration (TS → Rust)

`src/lib/invoke.ts:10-28` defines:
```ts
export async function safeInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T>;
```

It calls `invoke<T>` from `@tauri-apps/api/core`, parses any thrown error
(which is a string from the Rust side) as JSON, and either returns the parsed
`TrunkError` or wraps unknown errors as `{code: "unknown_error", …}`.

**Call shape for the new generate action:**
```ts
const md = await safeInvoke<string>("generate_review_doc", { path: repoPath });
```

**Argument naming — camelCase on the TS side, snake_case on the Rust side.**
Tauri auto-converts via serde's default field-name behavior on
`#[tauri::command]` parameters. The Rust command takes `path: String`, so
`{ path: repoPath }` is correct. No nested DTO needed (the existing review
commands all take flat named args — see `add_comment` at
`commands/review.rs:697-705`).

**Error catching pattern** (mirrors existing usage in ReviewPanel.svelte
line 199-208):
```ts
try {
  const md = await safeInvoke<string>("generate_review_doc", { path: repoPath });
  session.showPreview(md);
} catch (e) {
  const err = e as TrunkError;
  if (err.code === "no_comments") {
    // Should never happen — the button is gated. But fail loud.
    showToast("Add at least one comment first.", "error");
    return;
  }
  showToast(err.message ?? "Failed to generate review doc", "error");
}
```

**Confidence:** HIGH (read `invoke.ts` + existing call sites).

### Item 9 — Validation Architecture (see dedicated section below)

## Patterns to Reuse

| Pattern | Source | How Phase 70 uses it |
|---------|--------|------------------------|
| `_inner(data_dir, …)` + thin wrapper | `commands/review.rs:74-94` (`start_review_session_inner`) + `commands/review.rs:964-987` (`start_review_session` command) | `generate_review_doc_inner` (or directly inline since no disk side-effects) + `generate_review_doc` thin wrapper. |
| `canonical_repo_path` | `commands/review.rs:61-69` | Same call, same `not_open` error. |
| Clone-comments-under-lock | `commands/review.rs:901-913` (`list_session_comments`) | Same shape — clone `ReviewSession` out of the in-memory map BEFORE entering `spawn_blocking`. |
| `spawn_blocking` for git2 work | `commands/review.rs:950-958` (`resolve_session_comments`) | Same; never hold `ReviewSessionsState` lock across git2 work. |
| `String::from_utf8_lossy(blob.content()).lines()` | `commands/review.rs:358` (`classify_anchor`) | Same — render must mirror this for L-06. |
| `delta.old_file().is_binary() \|\| delta.new_file().is_binary()` | `commands/diff.rs:190,287,448` | Same — and the simpler `blob.is_binary()` for direct blob reads. |
| `DiffOptions::pathspec(file_path)` | `commands/diff.rs:355,369,386` | Same — filter `diff_tree_to_tree` to one file. |
| `commit_with_file` test helper | `commands/review.rs:2026-2047` | Same helper for golden tests of the render. |
| Theme-token-only styling | `ReviewPanel.svelte:519-657` | Same — every color goes through `var(--color-*)`. |
| Rune state shape (`$state(...)` + setters) | `review-session.svelte.ts:42-71` | Extend the same module with `panelMode` + `previewMarkdown`. |
| `safeInvoke<T>(cmd, args)` + try/catch + `showToast` | `ReviewPanel.svelte:179-209,230-268` | Same — error handling for the generate action. |
| Component-test command-aware mock | `ReviewPanel.test.ts:19-21` | New tests use the same `vi.mock('../lib/invoke.js')` shape. |

## Pitfalls / Landmines

### Pitfall 1: Diff replay on `Side::Old` of a root commit

**What goes wrong:** `commit.parent(0)` returns `Err` (no parent). If the diff
replay code unwraps or doesn't guard, the panic propagates.

**Why it happens:** A `Source::Diff` anchor on `Side::Old` references the
*parent* tree; a root commit has none. The Phase 69 classifier already returns
`FileGone` for this case (`commands/review.rs:339-346`), so the gate-then-
re-resolve flow inherits the guard. **As long as the renderer runs
`classify_anchor` FIRST and only attempts re-resolution when it returns
`Ok(())`, the diff replay never sees this case.** Bypassing the gate
re-introduces the panic risk.

**How to avoid:** Make `classify_anchor` the mandatory short-circuit at the
top of every per-comment resolution. Document this in `review.rs` with an
explicit `// SAFETY:` comment referencing `classify_anchor`.

**Warning signs:** Any Phase 70 code that calls `slice_diff` or `slice_blob`
without first calling `classify_anchor` is wrong.

### Pitfall 2: Diff anchor on a file unchanged at the commit

**What goes wrong:** `diff_tree_to_tree(parent, commit)` with
`pathspec(anchor.file_path)` returns a diff with zero hunks. The slice walk
produces an empty `String`. The renderer then emits an empty fence body, which
is technically valid markdown but semantically wrong — the AI sees an empty
diff block and has no signal that something failed.

**Why it happens:** The user anchored on a context line that was unchanged at
this commit, and a later commit reverted the change so the file is now
identical to its parent at this commit. The classifier passes (file exists,
line in range), but there's no diff to slice.

**How to avoid:** After the slice walk, check `if out.is_empty()` and return
`Err(ExcerptError::NoHunks)`. The unresolvable section then carries the
cached_excerpt fallback per L-04. Don't try to fall back to a FullFile-style
blob slice — that loses the Source::Diff signal the AI uses.

**Warning signs:** An anchor block with an empty `` ```diff ``…`` ``` `` body
in the rendered doc — should never happen.

### Pitfall 3: UTF-8 boundary across `lines()` slicing

**What goes wrong:** `String::from_utf8_lossy` returns a `Cow<str>`. Lines are
extracted as `&str` slices of the `Cow`'s underlying buffer. If the renderer
stores those slices and lets the `Cow` drop, it's a use-after-free at the type
level — but in practice the borrow checker catches this at compile time. The
real risk is treating the lossy-replaced characters (U+FFFD) as if they were
the original bytes when emitting the fence body.

**Why it happens:** A file in Shift-JIS, Latin-1, or any non-UTF-8 encoding
passes `blob.is_binary() == false` but has bytes that get replaced. The
renderer emits the U+FFFD-laden text inside a fence. The AI agent reading the
doc sees mojibake; not catastrophic, but worth knowing.

**How to avoid:** Document this explicitly. The render is best-effort for
non-UTF-8 text files. The placeholder is reserved for binary; lossy text is
emitted as-is. **Do not attempt encoding detection — that's a rabbit hole
(`chardet`, `whatwg-encoding`, etc.) outside this phase's scope.**

**Warning signs:** None at runtime — only visible to the AI consumer of the
doc. Worth a comment in `review.rs` so it isn't relitigated.

### Pitfall 4: Large diff replay performance

**What goes wrong:** `diff_tree_to_tree(parent, commit)` on a commit that
modifies many files can be expensive. Per L-02 the render re-runs the diff
ONCE PER ANCHORED FILE — if a single commit has 50 diff anchors across 50
files, that's 50 separate `diff_tree_to_tree` calls.

**Why it happens:** Naive per-anchor implementation. Easy to write, slow on
worst-case sessions.

**How to avoid:** Group anchors by `(commit_oid, file_path)` once at the top
of the renderer (D-05 grouping already does this). Within a group, run the
diff replay ONCE and emit all the file's anchors from the single walk.
**Locked decision is per-(file, commit) grouping (D-05)**, so the natural
implementation is already optimal. Just don't re-derive the slice per anchor
within a group.

**Warning signs:** Generate latency >1s on a small session. Hot loop visible
in `cargo flamegraph` if anyone profiles.

### Pitfall 5: Markdown injection via comment text

**What goes wrong:** The user's comment text is emitted verbatim into the
markdown body (after the fence). If a comment contains `` ``` `` or a
heading, it can be misparsed by an AI consumer that does its own markdown
parsing.

**Why it happens:** No escaping in the renderer.

**How to avoid:** **Don't escape.** The recipient is an AI coding agent, not
a browser. The user wrote the comment knowing it's going into a code-review
doc; bizarre formatting in the comment body is the user's signal to the AI
("see how the inner fence breaks the outer fence — that's the bug").
Escaping comment text would hide that signal. Document this stance in
`review.rs`.

**Warning signs:** None — this is a deliberate non-mitigation.

### Pitfall 6: `git2::Repository` is not `Sync`

**What goes wrong:** Holding the `ReviewSessionsState` lock across a
`spawn_blocking` call, or stashing a `Repository` in any `Send`-bounded state,
will fail to compile or deadlock.

**Why it happens:** Easy to write naively. The codebase has dealt with this
since v0.1 (`src-tauri/src/state.rs`: "Store PathBuf ONLY — git2::Repository
is not Sync.").

**How to avoid:** Open the `Repository` INSIDE the `spawn_blocking` closure,
never outside. Clone the `ReviewSession` out of the lock BEFORE entering
`spawn_blocking`. Both patterns are visible in `commands/review.rs:945-957`
(`resolve_session_comments`).

**Warning signs:** `cargo clippy` flags `Send`/`Sync` violations. Already
caught by `just check`.

## Runtime State Inventory

> Phase 70 is **greenfield** — it adds new functionality (one Rust module,
> one Tauri command, one panel-internal view) and modifies no existing
> persisted data. No rename, no migration, no string replacement. This
> section omitted.

## Environment Availability

> Phase 70 has **no new external dependencies**. All required tooling
> (Cargo, Vite, vitest, just, libgit2-vendored via git2's
> `vendored-libgit2` feature) is already installed and shipped in prior
> phases. Section omitted.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Frontend framework | vitest + @testing-library/svelte [VERIFIED: src/components/ReviewPanel.test.ts:1-3] |
| Backend framework | `cargo test` with `#[cfg(test)] mod tests` (no external runner) |
| Frontend test layout | Co-located `.test.ts` alongside the component (`src/components/ReviewPanel.svelte` ↔ `src/components/ReviewPanel.test.ts`) |
| Backend test layout | Inline `mod tests` in the same `.rs` file (mirrors `commands/review.rs:1090-2275`) |
| Quick run command | `just check` (runs fmt + biome + svelte-check + clippy + cargo-test + vitest) |
| Full suite command | `just check` (same — there is no slower tier in this project) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DOC-01 | Generate produces one markdown string with H1 + framing + commit refs + per-(file, commit) sections + commit-level section + unresolvable section | unit (Rust) | `cargo test -p trunk_lib --manifest-path src-tauri/Cargo.toml git::review::tests::generate_doc_has_all_sections` | Wave 0 |
| DOC-02 | Diff-source comments emit `` ```diff ``…`` ``` ``; full-file emit `` ```{lang} ``…`` ``` ``; fence length adapts to backtick runs in body | unit (Rust) | `cargo test ... git::review::tests::diff_source_uses_diff_fence`, `..::full_file_uses_language_fence`, `..::fence_length_avoids_backtick_collision` | Wave 0 |
| DOC-03 | Anchors grouped by (file, commit); per-anchor heading is `path:Lstart-Lend (sha)`; comments sorted by `start_line` within a group; commit-level in trailing section | unit (Rust) | `cargo test ... git::review::tests::anchors_grouped_by_file_commit`, `..::anchors_sorted_by_start_line`, `..::commit_level_in_trailing_section` | Wave 0 |
| DOC-04 | Unresolvable anchor (CommitGone/FileGone/LineOutOfRange/diff-no-hunks) produces an entry in the unresolvable section using `Comment.text` + `Comment.cached_excerpt`; never crashes | unit (Rust) | `cargo test ... git::review::tests::unresolvable_uses_cached_excerpt`, `..::renderer_never_panics_on_orphan` | Wave 0 |
| D-01 (UI) | Generate button is disabled when `comments.length === 0`, enabled when ≥1 | component (Svelte) | `vitest run src/components/ReviewPanel.test.ts -t "generate button"` | Wave 0 |
| D-02 (UI) | Click on Generate calls `safeInvoke("generate_review_doc", { path })` and swaps the panel into preview mode showing the returned string | component (Svelte) | `vitest run src/components/ReviewPanel.test.ts -t "preview swap"` | Wave 0 |
| D-02 back-affordance | "Back to comments" returns the panel to list mode | component (Svelte) | `vitest run src/components/ReviewPanel.test.ts -t "back to comments"` | Wave 0 |

### Sampling Rate

- **Per task commit:** `just check` (single command runs everything; no
  faster subset is materially worth defining)
- **Per wave merge:** `just check`
- **Phase gate:** `just check` green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src-tauri/src/git/review.rs` — new file; the `#[cfg(test)] mod tests`
  block carries the entire Rust-side golden-test suite (DOC-01..04). Reuse
  the `commit_with_file` helper pattern from `commands/review.rs:2026-2047`.
- [ ] `src/components/ReviewPanel.test.ts` — extend with: generate-button-
  disabled-when-no-comments, generate-click-invokes-command, preview-swap-
  renders-string, back-to-comments-restores-list.
- [ ] No new framework installs. All test infrastructure exists.

### Backend Test Strategy

The pure renderer in `git/review.rs` is the **bulk of the testable
surface**. The `_inner` wrapper has trivial logic (D-11 gate, error
serialization); the thin `#[tauri::command]` has none worth testing
(precedent: zero of the existing review commands test the wrapper directly —
they all test `_inner`).

Test pattern (lift from `commands/review.rs:2010-2275`):
1. Build a `TempDir` + `Repository::init`.
2. Use `commit_with_file(&repo, message, parents, path, content)` to create
   commits with known blob contents.
3. Construct a `ReviewSession` in-memory with synthesized `Comment`s anchored
   into the test repo.
4. Call `render(&session, &repo)`.
5. Assert on the returned `String` with substring checks or `expect_test`-
   style golden comparisons (project doesn't currently use `expect_test`, so
   substring checks are the conservative choice).

Suggested golden tests:

| Test | Setup | Assertion |
|------|-------|-----------|
| `generate_doc_has_all_sections` | 1 resolvable diff anchor + 1 resolvable full-file anchor + 1 commit-level + 1 orphan | All four sections appear in D-04 order |
| `diff_source_uses_diff_fence` | 1 Source::Diff comment | Body contains `` ```diff `` opening fence |
| `full_file_uses_language_fence` | 1 Source::FullFile on `foo.rs` | Body contains `` ```rust `` opening fence |
| `fence_length_avoids_backtick_collision` | A blob whose content contains `` ``` `` and `` ```` `` | Output's fence is at least `` ````` `` (5 backticks) for that excerpt |
| `crlf_normalized_in_fence_body` | A blob with `\r\n` line endings | Output's fence body uses LF only |
| `anchors_grouped_by_file_commit` | 2 comments on `foo.rs@A` + 1 on `foo.rs@B` + 1 on `bar.rs@A` | Three groups in the doc, each under one heading |
| `anchors_sorted_by_start_line` | 3 comments on same (file, commit), start_lines 30, 10, 20 | Order in doc is 10, 20, 30 |
| `unresolvable_uses_cached_excerpt` | A comment whose anchor.commit_oid is a 40-`0` (bogus) OID, with a non-null cached_excerpt | Unresolvable section contains the cached_excerpt verbatim |
| `renderer_never_panics_on_orphan` | All four orphan kinds (CommitGone, FileGone, LineOutOfRange, NoHunks) | `render()` returns a String, no panic |
| `binary_blob_uses_placeholder` | A Source::FullFile anchor on a blob with embedded NUL bytes | Output contains `[binary file, no excerpt]` for that anchor |
| `zero_comment_gate_at_command` | Empty `comments`, call `generate_review_doc_inner` | Returns `Err(TrunkError{code:"no_comments",…})` |

### Frontend Test Strategy

`ReviewPanel.test.ts` is the natural home (already exists, already mocks
`safeInvoke`, already mocks Tauri events). Extend with:

```ts
it("generate button is disabled when no comments", async () => {
  mockInvokeRoute({
    list_session_commits: [],
    list_session_comments: [],
    resolve_session_comments: [],
  });
  const { getByRole } = render(ReviewPanel, { props: { repoPath, onJump, onJumpToCommit } });
  await tick();
  const btn = getByRole("button", { name: /generate/i });
  expect(btn).toBeDisabled();
});

it("generate click invokes generate_review_doc and swaps to preview", async () => {
  // commands router returns 1 comment + 1 resolvable + 1 markdown doc
  mockInvokeRoute({
    ...,
    generate_review_doc: "# Code review: trunk\n\nfoo",
  });
  const { getByRole, findByText } = render(ReviewPanel, ...);
  await tick();
  fireEvent.click(getByRole("button", { name: /generate/i }));
  expect(await findByText(/# Code review: trunk/)).toBeInTheDocument();
});

it("back to comments returns to list view", async () => {
  // ... after preview is showing
  fireEvent.click(getByRole("button", { name: /back/i }));
  expect(getByText(/^src\/main\.ts/)).toBeInTheDocument();  // comment file ref
});
```

The rune (`review-session.svelte.ts`) can have its own `.test.ts` for the
`panelMode` transitions (mirrors `review-session.svelte.test.ts` if one
exists — Phase 69 may or may not have it; the planner checks). If it doesn't,
the rune behavior is exercised transitively through `ReviewPanel.test.ts`.

## Security Domain

> `security_enforcement` is not set in `.planning/config.json` → treat as
> enabled. Phase 70 has a narrow surface; the analysis is brief.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local desktop app, no auth |
| V3 Session Management | no | OS user session only |
| V4 Access Control | no | No multi-user model |
| V5 Input Validation | yes (limited) | Repo `path` arg validated by `canonical_repo_path` (must be a currently-open repo); commit OIDs come from session JSON written under our control (Phase 65 schema validation already covers them) |
| V6 Cryptography | no | No crypto operations in this phase |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `path` arg | Tampering | `canonical_repo_path` requires the path to be in `RepoState`'s open-repo map; an attacker would need to first open a malicious path through the open-repo flow (out of scope here) |
| Resource exhaustion via huge diff replay | DoS | Pitfall 4 — diff replay is per-(file, commit), bounded by the user's session size. Worst case is bounded by repo size. Acceptable for a desktop app |
| Markdown injection in comment text | Information disclosure / parsing confusion | Deliberate non-mitigation (Pitfall 5) — the recipient is an AI agent, not a browser; comment text is emitted verbatim. No XSS attack surface; no HTML rendering |
| Binary file content leak via fence body | Information disclosure | `Blob::is_binary()` placeholder (L-05) prevents emitting raw binary bytes into the doc |

**No new IPC surface beyond a single read-only command.** Phase 70 does not
expand the threat model meaningfully.

## Code Examples

Verified patterns from official sources / the codebase.

### Re-resolution of a Source::FullFile excerpt (L-02 + L-05 + L-06)

```rust
// In git/review.rs
use crate::git::types::{Anchor, Source, Side};

fn slice_full_file(
    repo: &git2::Repository,
    anchor: &Anchor,
) -> Result<String, ExcerptError> {
    // Pre: classify_anchor(anchor, repo) returned Ok(())
    let oid = git2::Oid::from_str(&anchor.commit_oid)
        .map_err(|_| ExcerptError::ResolutionFailed)?;
    let commit = repo.find_commit(oid)
        .map_err(|_| ExcerptError::ResolutionFailed)?;
    let tree = commit.tree().map_err(|_| ExcerptError::ResolutionFailed)?;
    let entry = tree.get_path(std::path::Path::new(&anchor.file_path))
        .map_err(|_| ExcerptError::ResolutionFailed)?;
    let blob = repo.find_blob(entry.id())
        .map_err(|_| ExcerptError::ResolutionFailed)?;

    if blob.is_binary() {
        return Err(ExcerptError::Binary);
    }

    // L-06: lossy + CRLF→LF, lines() semantics (matches classify_anchor:358)
    let text = String::from_utf8_lossy(blob.content()).replace("\r\n", "\n");

    // 1-based inclusive bounds; range is guaranteed in-bounds by classify_anchor.
    let start = anchor.start_line as usize;
    let end   = anchor.end_line   as usize;

    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        let lineno = i + 1;
        if lineno >= start && lineno <= end {
            out.push_str(line);
            out.push('\n');
        }
    }
    // Trim the trailing \n we always appended — the fence body shouldn't end
    // with a blank line unless the source had one.
    if out.ends_with('\n') { out.pop(); }
    Ok(out)
}
```

### Per-(file, commit) anchored section emission (D-05 + D-06 + L-07 + L-08)

```rust
fn emit_anchored_section(
    out: &mut String,
    file_path: &str,
    commit_short_oid: &str,
    anchors_on_this_file_commit: &[&Comment],  // pre-sorted by start_line
    repo: &git2::Repository,
) {
    let lang = fence_language(file_path);
    writeln!(out, "## {file_path} (`{commit_short_oid}`)").unwrap();
    writeln!(out).unwrap();

    for comment in anchors_on_this_file_commit {
        let anchor = comment.anchor.as_ref().expect("anchored");
        writeln!(
            out,
            "### {}:L{}-L{} ({})",
            anchor.file_path, anchor.start_line, anchor.end_line, commit_short_oid,
        ).unwrap();
        writeln!(out).unwrap();

        // D-06: excerpt first, comment text after.
        match try_resolve_excerpt(repo, anchor) {
            Ok(excerpt) => emit_fence(out, &excerpt, anchor.source, lang),
            Err(ExcerptError::Binary) => {
                writeln!(out, "`[binary file, no excerpt]`").unwrap();
                writeln!(out).unwrap();
            }
            Err(_) => unreachable!(  // caller already routed orphans away
                "anchored section must only contain resolvable anchors"
            ),
        }

        writeln!(out, "{}", comment.text).unwrap();
        writeln!(out).unwrap();
    }
}

fn emit_fence(out: &mut String, body: &str, source: Source, lang: &str) {
    let n = fence_length(body);
    let fence: String = std::iter::repeat('`').take(n).collect();
    let info = match source {
        Source::Diff     => "diff",
        Source::FullFile => lang,
    };
    writeln!(out, "{fence}{info}").unwrap();
    out.push_str(body);
    if !body.ends_with('\n') { out.push('\n'); }
    writeln!(out, "{fence}").unwrap();
    writeln!(out).unwrap();
}
```

### Test setup (lift from commands/review.rs:2026-2047)

```rust
fn commit_with_file(
    repo: &Repository,
    message: &str,
    parents: &[Oid],
    path: &str,
    content: &str,
) -> Oid {
    let blob_oid = repo.blob(content.as_bytes()).unwrap();
    let mut builder = repo.treebuilder(None).unwrap();
    builder
        .insert(path, blob_oid, git2::FileMode::Blob.into())
        .unwrap();
    let tree = repo.find_tree(builder.write().unwrap()).unwrap();
    let parent_commits: Vec<_> = parents
        .iter()
        .map(|oid| repo.find_commit(*oid).unwrap())
        .collect();
    let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();
    let s = sig();
    repo.commit(None, &s, &s, message, &tree, &parent_refs)
        .unwrap()
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Per-line enriched-payload render in the frontend | Pure Rust render returning one markdown string | Phase 70 ROADMAP §70 Notes | Smaller IPC payload, all logic testable in `cargo test`, no syntect coupling. |
| Custom NUL-byte scanning for binary detection | `git2::Blob::is_binary()` | (already in codebase) | One-line call, libgit2-accurate. |
| OrphanReason as the only failure type | Render-only `ExcerptError` enum + `OrphanReason` wrapped inside | This phase | Keeps the IPC wire stable while supporting render-only failure modes. |

**Deprecated/outdated:** Nothing in this phase deprecates prior work.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `git2::Blob::is_binary()` checks the first ~8000 bytes for NUL via libgit2's `git_blob_is_binary` | Item 1 | LOW — even if the exact byte-count is wrong, the function returns `bool` and that's all we need. Implementation detail. |
| A2 | The TS capture-side line numbers (`new_lineno`/`old_lineno`) are emitted by libgit2 in Rust and are byte-for-byte the same as what `commands/diff.rs:235-236` produces today | Item 2 | LOW — same git2 callback, same code path. |
| A3 | Markdown injection via comment text is a non-mitigation (the AI consumer doesn't render HTML) | Pitfall 5 | MEDIUM — if a future consumer DOES render HTML, this changes. Worth a comment in `review.rs`. |
| A4 | A diff replay on a file unchanged from parent at the commit should route to unresolvable with reason "diff hunk no longer exists" | Pitfall 2 + Item 4 | LOW-MEDIUM — alternative would be to fall back to a FullFile-style blob slice, which would lose the Source::Diff signal. Either is defensible; the planner could flip if user prefers. |

## Open Questions for Planner

### Q1: Should `generate_review_doc_inner` exist, given there's no disk I/O?

**Context:** The `_inner` pattern in this codebase typically separates from
the `#[tauri::command]` wrapper because `_inner` writes to disk (and disk
behavior is what tests need to prove via `TempDir`). The generate command
does NO disk I/O — it reads in-memory session state and a `git2::Repository`.

**Tradeoff:**
- **Option A — keep `_inner`:** consistent with the other review commands;
  `_inner` takes `(data_dir, path, state_map, sessions)` and returns the
  markdown string. The data_dir param is unused, slightly weird.
- **Option B — skip `_inner`, test `git::review::render` directly:** the pure
  renderer is fully testable on its own (it takes `&ReviewSession` +
  `&git2::Repository`). The thin command just wraps it.

**Recommendation:** **Option B.** The `_inner` pattern exists to make
disk behavior testable; there's no disk here. Test `render` and skip the
indirection. Document the deviation in the command's doc-comment so it
isn't mistaken for sloppiness.

### Q2: File-group heading text + level (D-05 grouping)

CONTEXT.md explicitly leaves the heading shape and level to the planner.
Three shapes:
- `## src/foo.rs (abc1234)` — parentheses, file first
- `## src/foo.rs @ abc1234` — `@` separator, no parentheses
- `## abc1234 — src/foo.rs` — sha first, em-dash separator

Recommendation: **`## src/foo.rs (abc1234)`** — matches the locked per-anchor
heading shape `path:Lstart-Lend (sha)` (DOC-03), so the SHA-in-parens pattern
is consistent throughout the doc. H1 = doc title; H2 = per-(file, commit); H3
= per-anchor.

### Q3: Should the renderer panic if `classify_anchor` returns `Ok(())` but excerpt re-resolution still fails?

**Context:** L-04 says "Never `unwrap`. On failure (any kind: commit gone,
file gone, blob not utf8 in a way that matters, hunk slice failure, anything),
the comment routes to the unresolvable section." This is unambiguous: route
to unresolvable, never panic. But Item 7's `ExcerptError::ResolutionFailed`
variant is the catch-all — should there be a "this should be unreachable"
assertion anywhere?

**Recommendation:** No assertion. Match L-04 verbatim: every failure routes
to unresolvable, even ones we believe are impossible. The cost of an extra
unresolvable-section entry is zero; the cost of a panic in user-facing
markdown generation is high.

### Q4: ReviewDocPreview.svelte vs. inline branch in ReviewPanel.svelte

CONTEXT.md explicitly leaves this to the planner. ReviewPanel.svelte is
already 659 lines; adding a preview view inline pushes it over 700.

Tradeoffs:
- **Inline branch:** simpler, no new file, all panel state in one place.
- **`ReviewDocPreview.svelte`:** smaller component files, but adds a prop-
  drilling surface (the rune state, the back-to-list callback).

Recommendation: **`ReviewDocPreview.svelte` as a new file.** The preview view
will grow in Phase 71 (Copy + Save buttons + their wire-up). Separating it
now means Phase 71's diff is local to the preview component, not a 800-line
ReviewPanel.svelte diff. Per Engineering Judgment §2 ("Draw boundaries before
internals"), this boundary is structural.

## Sources

### Primary (HIGH confidence)
- `src-tauri/Cargo.toml:26` — git2 0.19 pinned with `vendored-libgit2`
- `src-tauri/Cargo.lock` — git2 = 0.19.0 (build-stable resolution)
- `src-tauri/src/commands/review.rs:296-398` — `OrphanReason`,
  `classify_anchor`, `resolve_all` (Phase 69 classifier — reused)
- `src-tauri/src/commands/review.rs:891-961` — `list_session_comments` /
  `resolve_session_comments` (the `_inner` + thin wrapper + clone-under-lock
  + spawn_blocking pattern to mirror)
- `src-tauri/src/commands/review.rs:2026-2275` — `commit_with_file`,
  `make_file_repo`, and the existing test pattern to lift for Phase 70
- `src-tauri/src/commands/diff.rs:182-249,354-414` — `walk_diff`,
  `DiffOptions::pathspec`, `diff_tree_to_tree(parent, commit)`, the binary
  detection pattern (`delta.*_file().is_binary()`)
- `src-tauri/src/git/syntax.rs` — confirms no `.ext → markdown-language-tag`
  table exists; `extension_from_path` is L-10-safe to lift
- `src-tauri/src/git/types.rs:288-345` — review schema (Comment, Anchor,
  Source, Side, ReviewSession)
- `src-tauri/src/git/review_store.rs:104-155` — `load_session` (for
  reference; the renderer reads from the in-memory map, not disk)
- `src/components/ReviewPanel.svelte` — the host panel (lines 1-289 script;
  lines 519-657 styles)
- `src/lib/review-session.svelte.ts` — the rune to extend
- `src/lib/invoke.ts:1-28` — `safeInvoke` shape
- `src/lib/diff-anchor.ts`, `src/lib/full-file-anchor.ts` — capture-side
  TS adapters (confirmed: no JS line counting)
- `src/components/ReviewPanel.test.ts:1-80` — test mocking pattern
- `.planning/codebase/CONVENTIONS.md` — naming, error handling, theme
  tokens, Svelte 5 runes
- `.planning/ROADMAP.md:335-374` — Phase 70 + Phase 71 specs
- `.planning/REQUIREMENTS.md` — DOC-01..04 + Out of Scope table
- `.planning/phases/70-excerpt-resolution-markdown-render/70-CONTEXT.md` —
  all locked D-01..D-11 and L-01..L-10 decisions

### Secondary (MEDIUM-to-HIGH confidence)
- [docs.rs git2 0.19 — Blob](https://docs.rs/git2/0.19.0/git2/struct.Blob.html) —
  `is_binary()`, `content()`, `size()` signatures (verbatim doc-comments)
- [docs.rs git2 0.19 — DiffFile](https://docs.rs/git2/0.19.0/git2/struct.DiffFile.html) —
  `is_binary()` signature

### Tertiary (LOW confidence — NONE)
- No claim in this RESEARCH.md relies on WebSearch-only or unverified
  sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every dependency is already pinned; nothing new
- Architecture: HIGH — every pattern lifted from existing code with file:line
  references
- Pitfalls: MEDIUM-to-HIGH — Pitfalls 1, 2, 6 verified against existing code;
  Pitfalls 3, 4, 5 are engineering judgment calls, not code-verified
- Validation: HIGH — test patterns exist in the codebase

**Research date:** 2026-05-26
**Valid until:** 2026-06-25 (30 days; git2 0.19 is stable, the Svelte 5 / Tauri 2
stack is stable, and the codebase is the primary source)
