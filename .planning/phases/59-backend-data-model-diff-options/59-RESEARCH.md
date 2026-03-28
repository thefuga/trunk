# Phase 59: Backend Data Model & Diff Options - Research

**Researched:** 2026-03-28
**Domain:** Rust git2 DiffOptions, Tauri IPC data model extension, LazyStore persistence
**Confidence:** HIGH

## Summary

This phase extends the existing diff pipeline with configurable options (context lines, whitespace ignore, show full file) and adds enrichment fields to DiffLine for downstream word-level and syntax highlighting phases. The core work is in three layers: (1) a new `DiffRequestOptions` struct that flows through all three diff commands (unstaged, staged, commit), (2) new `word_spans` and `syntax_tokens` fields on the Rust `DiffLine` type (empty vecs now, populated in Phases 60-61), and (3) LazyStore persistence for diff display preferences on the frontend.

The git2 crate already supports `context_lines(u32)` and `ignore_whitespace_change(bool)` directly on `DiffOptions`. The existing code already creates `DiffOptions` in `diff_unstaged_inner` and `diff_staged_inner` -- those just need the new settings applied. `diff_commit_inner` currently passes `None` for options and needs to create a `DiffOptions` instance. The "show full file" feature maps to passing a very large `context_lines` value (e.g., `u32::MAX`) to git2, which returns the entire file with changed lines marked.

**Primary recommendation:** Add a `DiffRequestOptions` Rust struct with Deserialize, thread it through all `_inner` functions and `#[tauri::command]` wrappers, mirror it in TypeScript, and add LazyStore get/set pairs for each preference key.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Single boolean toggle using git2's `ignore_whitespace_change` option. No multi-level dropdown.
- **D-02:** Context lines is a bounded integer (0-10 range). No "All" option in context lines setting.
- **D-03:** Separate `show_full_file: bool` toggle on DiffRequestOptions. When true, backend passes a large context_lines value to git2.
- **D-04:** Context lines and show_full_file are independent settings.
- **D-05:** DiffLine gets `word_spans` and `syntax_tokens` fields (empty vecs in this phase).
- **D-06:** All diff display preferences are global (shared across tabs), not per-tab. Uses existing LazyStore pattern.
- **D-07:** Preferences persisted: context_lines, ignore_whitespace, show_full_file (plus word_wrap, show_invisibles, view_mode for later phases).

### Claude's Discretion
- Enrichment field structure (byte offset ranges vs pre-segmented strings) -- Claude picks what aligns best with `similar` crate's `iter_inline_changes()` output and `syntect`'s token ranges.

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CTXL-01 | User can select context line count from toolbar dropdown (3/5/10/25/All) | git2 `DiffOptions::context_lines(u32)` directly supports this. Backend accepts any u32 value. Phase 59 scope is bounded 0-10 per D-02; the dropdown UI choices are a later-phase concern. |
| CTXL-02 | Selecting "All" activates full file view mode | Implemented via `show_full_file: bool` on DiffRequestOptions (D-03). Backend passes `u32::MAX` to git2 context_lines when true. |
| WHSP-01 | User can toggle whitespace ignore in diff toolbar (re-fetches diff with ignore_whitespace_change) | git2 `DiffOptions::ignore_whitespace_change(bool)` is a direct 1:1 mapping. Already used pattern in existing DiffOptions setup. |
| DISP-03 | All diff display preferences persist across sessions via LazyStore | Existing LazyStore pattern (get/set/save per key) in `store.ts` -- extend with `diff_context_lines`, `diff_ignore_whitespace`, `diff_show_full_file` keys. |
</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.19 | Diff generation with configurable options | Already used; `DiffOptions` has `context_lines()` and `ignore_whitespace_change()` |
| serde | 1 | Serialize/Deserialize for IPC structs | Already used for all types |
| @tauri-apps/plugin-store | 2.4.2 | LazyStore persistence | Already used in `store.ts` |

### Future (not needed this phase, but informs field design)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| similar | 2.7.0 | Word-level diff (Phase 60) | `InlineChange::iter_strings_lossy()` returns `(bool, Cow<str>)` tuples -- emphasized flag + text segment |
| syntect | 5.3.0 | Syntax highlighting (Phase 61) | `HighlightLines::highlight_line()` returns `Vec<(Style, &str)>` -- styled string slices |

**No new dependencies needed for Phase 59.** The enrichment fields are empty vecs.

## Architecture Patterns

### Data Flow: DiffRequestOptions

```
Frontend (TypeScript)                    Backend (Rust)
--------------------------              --------------------------
DiffRequestOptions {                    DiffRequestOptions {
  contextLines: number;      --IPC-->     context_lines: u32,
  ignoreWhitespace: boolean;              ignore_whitespace: bool,
  showFullFile: boolean;                  show_full_file: bool,
}                                       }

safeInvoke("diff_unstaged", {           pub async fn diff_unstaged(
  path, filePath,                         path, file_path,
  options: { ... }                        options: DiffRequestOptions,
})                                        state: State<...>
                                        )
```

Tauri 2 `#[tauri::command]` automatically converts snake_case Rust parameter names to camelCase on the JS side. The existing codebase confirms this: `file_path` in Rust is called as `filePath` from JS. So `DiffRequestOptions` with `context_lines` in Rust will be received as `contextLines` from JS.

### Pattern 1: Options Struct with Defaults
**What:** A single `DiffRequestOptions` struct with `#[derive(Deserialize)]` and `impl Default` that all diff commands accept.
**When to use:** Every diff command invocation.
**Example:**
```rust
// Source: Verified against git2 0.19 docs.rs API
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffRequestOptions {
    #[serde(default = "default_context_lines")]
    pub context_lines: u32,
    #[serde(default)]
    pub ignore_whitespace: bool,
    #[serde(default)]
    pub show_full_file: bool,
}

fn default_context_lines() -> u32 { 3 }

impl Default for DiffRequestOptions {
    fn default() -> Self {
        Self {
            context_lines: 3,
            ignore_whitespace: false,
            show_full_file: false,
        }
    }
}
```

**Important:** Use `#[serde(rename_all = "camelCase")]` on the struct so the JS side can send `{ contextLines: 3, ignoreWhitespace: false, showFullFile: false }` matching the Tauri convention. This is the same pattern used in `interactive_rebase.rs` line 10.

### Pattern 2: Apply Options to git2::DiffOptions
**What:** A helper that configures a `git2::DiffOptions` from `DiffRequestOptions`.
**When to use:** Inside each `_inner` function before calling git2 diff methods.
**Example:**
```rust
// Source: Verified against git2 0.19 DiffOptions API
fn apply_request_options(opts: &mut git2::DiffOptions, req: &DiffRequestOptions) {
    let context = if req.show_full_file {
        u32::MAX  // git2 returns entire file contents
    } else {
        req.context_lines
    };
    opts.context_lines(context);
    opts.ignore_whitespace_change(req.ignore_whitespace);
}
```

### Pattern 3: Enrichment Fields on DiffLine (byte offset ranges)
**What:** `word_spans` and `syntax_tokens` use byte offset ranges into `DiffLine.content`.
**When to use:** Populated in Phases 60-61; empty vecs in Phase 59.

**Design decision (Claude's discretion):** Use byte offset ranges, not pre-segmented strings.

Rationale:
- `similar`'s `InlineChange::iter_strings_lossy()` returns `(bool, Cow<str>)` segments. These are substrings of the original line content. Converting to byte offsets is trivial: track cumulative byte length as you iterate.
- `syntect`'s `HighlightLines::highlight_line()` returns `Vec<(Style, &str)>` -- also substrings. Same conversion applies.
- Byte offsets are more compact over IPC (two u32s vs duplicating string content).
- The frontend can slice `content` using the offsets, avoiding double-storage of text.
- Both consumers (word-level and syntax) work with the same content string, so offsets into that string are the natural shared representation.

```rust
// Enrichment types -- empty in Phase 59, populated in Phases 60-61
#[derive(Debug, Serialize, Clone, Default)]
pub struct WordSpan {
    pub start: u32,       // byte offset into DiffLine.content
    pub end: u32,         // byte offset (exclusive)
    pub emphasized: bool, // true = this segment changed
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct SyntaxToken {
    pub start: u32,      // byte offset into DiffLine.content
    pub end: u32,        // byte offset (exclusive)
    pub scope: String,   // syntect scope string, e.g. "keyword.control"
}

#[derive(Debug, Serialize, Clone)]
pub struct DiffLine {
    pub origin: DiffOrigin,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub word_spans: Vec<WordSpan>,     // NEW
    pub syntax_tokens: Vec<SyntaxToken>, // NEW
}
```

TypeScript mirror:
```typescript
export interface WordSpan {
    start: number;
    end: number;
    emphasized: boolean;
}

export interface SyntaxToken {
    start: number;
    end: number;
    scope: string;
}

export interface DiffLine {
    origin: DiffOrigin;
    content: string;
    old_lineno: number | null;
    new_lineno: number | null;
    word_spans: WordSpan[];      // NEW
    syntax_tokens: SyntaxToken[]; // NEW
}
```

### Pattern 4: LazyStore Preference Persistence
**What:** Extend `store.ts` with get/set functions for diff preferences.
**When to use:** On app init (load saved preferences) and when user changes a setting.
**Example:**
```typescript
// Source: Existing pattern in store.ts (getZoomLevel/setZoomLevel)
const DIFF_CONTEXT_LINES_KEY = "diff_context_lines";
const DIFF_IGNORE_WHITESPACE_KEY = "diff_ignore_whitespace";
const DIFF_SHOW_FULL_FILE_KEY = "diff_show_full_file";

export async function getDiffContextLines(): Promise<number> {
    return (await store.get<number>(DIFF_CONTEXT_LINES_KEY)) ?? 3;
}

export async function setDiffContextLines(lines: number): Promise<void> {
    await store.set(DIFF_CONTEXT_LINES_KEY, lines);
    await store.save();
}

// Same pattern for ignoreWhitespace (default false) and showFullFile (default false)
```

### Recommended Change Scope
```
src-tauri/src/git/types.rs       -- Add WordSpan, SyntaxToken, DiffRequestOptions; extend DiffLine
src-tauri/src/commands/diff.rs   -- Thread DiffRequestOptions through all _inner + command fns
src/lib/types.ts                 -- Mirror WordSpan, SyntaxToken, DiffRequestOptions; extend DiffLine
src/lib/store.ts                 -- Add diff preference get/set functions
src/components/RepoView.svelte   -- Pass options to diff invoke calls (4 call sites)
src/components/DiffPanel.svelte  -- Gracefully handle word_spans/syntax_tokens (empty for now)
```

### Anti-Patterns to Avoid
- **Passing options as individual parameters:** Use a single struct, not `context_lines: u32, ignore_whitespace: bool, show_full_file: bool` as separate Tauri command params. A struct is cleaner, easier to extend, and matches the existing `serde(rename_all)` pattern.
- **Hardcoding u32::MAX in the frontend:** The "show full file" logic belongs in the backend. Frontend sends `showFullFile: true`, backend translates to `u32::MAX` context lines.
- **Storing pre-segmented strings in word_spans/syntax_tokens:** Byte offset ranges are more IPC-efficient and avoid duplicating content strings.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Context line configuration | Custom diff algorithm patches | `git2::DiffOptions::context_lines(u32)` | Libgit2 handles this natively with full hunk header recalculation |
| Whitespace ignore | Post-processing filter on diff output | `git2::DiffOptions::ignore_whitespace_change(bool)` | Libgit2 compares at the right level; post-filtering breaks hunk indices for staging |
| Full file as diff | Manual file-read + diff merging | `context_lines(u32::MAX)` on git2 | Git natively supports "infinite context" -- produces a valid diff with all lines as context |
| Key-value persistence | Custom file I/O | `@tauri-apps/plugin-store` LazyStore | Already integrated, handles atomic writes, JSON serialization |
| camelCase/snake_case conversion | Manual rename in JS | `#[serde(rename_all = "camelCase")]` | Serde handles bidirectional conversion at compile time |

**Key insight:** git2's `DiffOptions` already exposes every knob this phase needs. The work is pure plumbing -- threading an options struct through existing functions and extending data types.

## Common Pitfalls

### Pitfall 1: Forgetting to Create DiffOptions in diff_commit_inner
**What goes wrong:** `diff_commit_inner` currently passes `None` for the options parameter to `diff_tree_to_tree`. If you add the options struct to the function signature but forget to actually create and pass `git2::DiffOptions`, context lines and whitespace ignore silently don't work for commit diffs.
**Why it happens:** The other two functions (`diff_unstaged_inner`, `diff_staged_inner`) already create `git2::DiffOptions` and just need extension. `diff_commit_inner` needs a `DiffOptions` created from scratch.
**How to avoid:** Create a shared `apply_request_options()` helper that all three functions call. Write a test for `diff_commit_inner` with non-default options.
**Warning signs:** Tests pass with default options but fail when context_lines != 3 on commit diffs.

### Pitfall 2: IPC Serialization of Empty Vecs
**What goes wrong:** If `word_spans` and `syntax_tokens` serialize as `null` instead of `[]`, TypeScript code that does `line.word_spans.length` will throw.
**Why it happens:** Using `Option<Vec<T>>` instead of `Vec<T>` or incorrect serde defaults.
**How to avoid:** Use `Vec<WordSpan>` (not `Option<Vec<WordSpan>>`). `Vec` serializes as `[]` by default. Verify with a serde_json roundtrip test.
**Warning signs:** Frontend crashes when rendering diff lines with empty enrichment data.

### Pitfall 3: show_full_file with u32::MAX on Very Large Files
**What goes wrong:** Passing `u32::MAX` context lines to git2 for a 100K+ line file produces a massive diff payload that can freeze the IPC bridge.
**Why it happens:** `u32::MAX` is technically correct but produces enormous output for large files.
**How to avoid:** Use a practical cap like `100_000` instead of `u32::MAX`. This is enough for any reasonable file while avoiding IPC payload issues. The frontend will need virtualization in Phase 62 anyway.
**Warning signs:** App hangs when "show full file" is toggled on a large generated file.

### Pitfall 4: Tauri camelCase Mismatch with Options Struct
**What goes wrong:** If `DiffRequestOptions` in Rust uses `context_lines` without `#[serde(rename_all = "camelCase")]`, the frontend must send `context_lines` instead of `contextLines`, breaking the established convention.
**Why it happens:** Tauri auto-converts top-level command params but nested struct fields follow serde rules.
**How to avoid:** Add `#[serde(rename_all = "camelCase")]` to `DiffRequestOptions`. This is the pattern already used in `interactive_rebase.rs`.
**Warning signs:** Invoke calls fail with deserialization errors when sending options from JS.

### Pitfall 5: Breaking Existing Tests by Changing _inner Signatures
**What goes wrong:** All 10 existing diff tests call `_inner` functions through test driver methods. Changing the `_inner` signatures (adding `DiffRequestOptions` param) breaks all tests until drivers are updated.
**Why it happens:** Test drivers in `tests/common/drivers/diff.rs` directly call `_inner` functions.
**How to avoid:** Update test drivers to pass `DiffRequestOptions::default()` immediately. Keep existing tests passing first, then add new tests for non-default options.
**Warning signs:** `cargo test` shows 10 compilation errors in test_diff.rs.

## Code Examples

### git2 DiffOptions Configuration
```rust
// Source: Verified against git2 0.19 docs.rs API
let mut opts = git2::DiffOptions::new();
opts.pathspec(file_path);
opts.include_untracked(true);
opts.recurse_untracked_dirs(true);
opts.show_untracked_content(true);

// NEW: Apply user preferences
opts.context_lines(3);                   // default, or user's preference
opts.ignore_whitespace_change(false);    // default, or user's toggle
```

### Full File Mode via Context Lines
```rust
// Source: Verified against git2 0.19 docs.rs API
// When show_full_file is true, pass a large context value
let context = if request_options.show_full_file {
    100_000  // practical cap; enough for any reasonable file
} else {
    request_options.context_lines
};
opts.context_lines(context);
```

### Serde Roundtrip for Enrichment Fields
```rust
// Source: serde_json standard pattern
let line = DiffLine {
    origin: DiffOrigin::Add,
    content: "let x = 42;".to_string(),
    old_lineno: None,
    new_lineno: Some(5),
    word_spans: vec![],       // empty this phase
    syntax_tokens: vec![],    // empty this phase
};
let json = serde_json::to_string(&line).unwrap();
assert!(json.contains("\"word_spans\":[]"));
assert!(json.contains("\"syntax_tokens\":[]"));
```

### LazyStore Round-Trip Pattern
```typescript
// Source: Existing pattern from store.ts (getZoomLevel/setZoomLevel)
const DIFF_CONTEXT_LINES_KEY = "diff_context_lines";

export async function getDiffContextLines(): Promise<number> {
    return (await store.get<number>(DIFF_CONTEXT_LINES_KEY)) ?? 3;
}

export async function setDiffContextLines(lines: number): Promise<void> {
    await store.set(DIFF_CONTEXT_LINES_KEY, lines);
    await store.save();
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| git2 DiffOptions with defaults only | Configurable context_lines + whitespace ignore | Phase 59 | User controls diff display granularity |
| DiffLine with content only | DiffLine with word_spans + syntax_tokens enrichment | Phase 59 (fields), 60-61 (population) | Enables word-level and syntax highlighting without re-fetching |

**Deprecated/outdated:**
- None relevant. git2 0.19 API is current and stable.

## Open Questions

1. **Practical cap for show_full_file context_lines**
   - What we know: `u32::MAX` works with git2 but produces huge payloads for large files.
   - What's unclear: The exact threshold where IPC payload size becomes a problem.
   - Recommendation: Use `100_000` as the cap. This covers any reasonable source file while staying well within IPC limits. If a file exceeds 100K lines, it's likely generated/binary and the diff is useless anyway.

2. **SyntaxToken scope field representation**
   - What we know: syntect uses `Scope` objects with dotted-string representation (e.g., "keyword.control.rust").
   - What's unclear: Whether to store the full scope stack or just the top scope, and whether to use a string or an enum.
   - Recommendation: Use `String` for now. Phase 61 will populate these and can refine the representation. The field exists as a placeholder.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust: cargo test (integration tests in `src-tauri/tests/`); TS: vitest |
| Config file | Rust: `src-tauri/Cargo.toml` `[dev-dependencies]`; TS: `package.json` `"test": "vitest run"` |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml --test test_diff` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml && bun run test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CTXL-01 | Diff with custom context_lines returns correct number of context lines | integration | `cargo test --manifest-path src-tauri/Cargo.toml --test test_diff context_lines` | Wave 0 |
| CTXL-02 | show_full_file returns entire file as diff | integration | `cargo test --manifest-path src-tauri/Cargo.toml --test test_diff full_file` | Wave 0 |
| WHSP-01 | ignore_whitespace hides whitespace-only changes | integration | `cargo test --manifest-path src-tauri/Cargo.toml --test test_diff whitespace` | Wave 0 |
| DISP-03 | Diff preferences round-trip through LazyStore | unit | `bun run test -- store.test.ts` | Wave 0 |
| (D-05) | DiffLine word_spans/syntax_tokens serialize as empty arrays | unit | `cargo test --manifest-path src-tauri/Cargo.toml --test test_integ_serde` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml --test test_diff && bun run test`
- **Per wave merge:** `cargo test --manifest-path src-tauri/Cargo.toml && bun run test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] New Rust integration tests in `test_diff.rs` for: context_lines override, whitespace ignore, show_full_file, and enrichment field serialization
- [ ] Update test drivers in `tests/common/drivers/diff.rs` to accept `DiffRequestOptions`
- [ ] New vitest tests in `store.test.ts` for: getDiffContextLines, setDiffContextLines, getDiffIgnoreWhitespace, setDiffIgnoreWhitespace, getDiffShowFullFile, setDiffShowFullFile

## Sources

### Primary (HIGH confidence)
- git2 0.19 docs.rs -- DiffOptions API: `context_lines(u32)`, `ignore_whitespace_change(bool)` method signatures and descriptions
- similar 2.7.0 docs.rs -- InlineChange::iter_strings_lossy() returns `(bool, Cow<str>)` tuples
- syntect 5.3.0 docs.rs -- HighlightLines::highlight_line() returns `Vec<(Style, &str)>`
- Existing codebase -- diff.rs, types.rs, types.ts, store.ts directly inspected

### Secondary (MEDIUM confidence)
- Tauri 2 v2.tauri.app docs -- Command parameter naming (snake_case auto-converts to camelCase for top-level params; nested structs need `#[serde(rename_all)]`)

## Project Constraints (from CLAUDE.md)

- **All git operations through git2 crate** -- no shelling out. This phase is 100% git2.
- **Never inline colors** -- CSS custom properties only. Phase 59 is backend-focused; frontend changes are minimal wiring.
- **Run all 6 checks before push** -- fmt, clippy, test, vitest, svelte-check, biome.
- **TypeScript types mirror Rust 1:1** in `lib/types.ts`.
- **Frontend->Backend:** `invoke("command_name", args)` calls Rust `#[tauri::command]` fns.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - git2 0.19 API verified against docs.rs, existing codebase patterns confirmed
- Architecture: HIGH - Direct extension of existing patterns (DiffOptions, LazyStore, type mirroring)
- Pitfalls: HIGH - Based on direct code inspection of all affected files and test infrastructure
- Enrichment field design: MEDIUM - Informed by similar/syntect API inspection but those crates are Phase 60-61 concerns

**Research date:** 2026-03-28
**Valid until:** 2026-04-28 (stable domain -- git2, serde, and Tauri APIs are mature)
