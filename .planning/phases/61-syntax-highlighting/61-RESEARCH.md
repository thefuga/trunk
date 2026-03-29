# Phase 61: Syntax Highlighting - Research

**Researched:** 2026-03-28
**Domain:** Rust syntax highlighting with syntect, TextMate scope-to-CSS mapping, merged span rendering
**Confidence:** HIGH

## Summary

Phase 61 adds language-aware syntax coloring to all diff lines. The syntect crate (v5.3.0) provides one-shot line highlighting using TextMate grammars with 150+ bundled languages including Rust, TypeScript, JSON, Python, Go, and more. The key challenge is converting syntect's color-based output into CSS-class-based tokens (SyntaxToken with scope field carrying a CSS class name like `syn-keyword`), then merging those tokens with the existing word_spans into a single unified span array per line.

The approach involves: (1) adding syntect as a dependency, (2) building a syntax highlighting pass in the Rust backend that runs after diff line collection but before serialization, (3) implementing a scope-to-CSS-class mapping function that inspects the TextMate scope stack for each token and assigns a CSS class, (4) merging syntax tokens with word spans into a combined span array, and (5) updating the frontend to render the merged spans with CSS classes for text color and word-diff background.

**Primary recommendation:** Use `HighlightLines::highlight_line()` to get token boundaries with foreground colors from a VS Code Dark+ equivalent theme, then map those foreground colors to CSS class names via a color-to-class lookup table. This avoids the complexity of scope string parsing while leveraging syntect's theme engine for accurate token classification.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** syntect (Rust, TextMate grammars) -- confirmed. 150+ bundled languages, one-shot highlighting, VS Code Dark+ theme support built-in. Tree-sitter considered and rejected.
- **D-02:** VS Code Dark+ base theme. Blue keywords (#569cd6), orange strings (#ce9178), green comments (#6a9955), light green numbers (#b5cea8), teal types (#4ec9b0), yellow functions (#dcdcaa), light grey operators (#d4d4d4).
- **D-03:** All syntax colors defined as CSS custom properties (e.g., `--color-syn-keyword`, `--color-syn-string`). Never inline styles.
- **D-04:** Reduce opacity of syntax-colored text on add/delete line backgrounds (e.g., 0.7 alpha). Context lines get full-color syntax.
- **D-05:** Fine-grained mapping (15+ CSS classes) for precise syntax coloring close to a real editor experience.
- **D-06:** Mapping happens in Rust backend. SyntaxToken.scope field carries the mapped CSS class name (not raw TextMate scope string).
- **D-07:** Syntax tokens set text color (via CSS class). Word-diff spans set background color. Both render simultaneously.
- **D-08:** Backend merges syntax_tokens and word_spans into a single sorted span array per line. Frontend renders one loop.

### Claude's Discretion
- Exact syntect API usage (SyntaxSet loading strategy, ThemeSet selection)
- Performance caching for syntax sets (per-request vs singleton)
- Exact list of 15+ fine-grained scope-to-class mappings
- How to handle the merged span type (new combined type vs extending SyntaxToken)
- Fallback behavior for unrecognized file extensions (no highlighting, plain text)

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SYNT-01 | Diff lines are syntax-highlighted with language-aware coloring | syntect `HighlightLines::highlight_line()` produces styled token segments per line; scope-to-CSS mapping converts to CSS classes |
| SYNT-02 | Language is auto-detected from file extension | `SyntaxSet::find_syntax_by_extension()` resolves language from file path extension; falls back to plain text |
| SYNT-03 | Syntax colors are desaturated on add/delete line backgrounds to maintain readability | CSS `opacity: 0.7` on syntax-colored spans within add/delete lines; context lines get full opacity |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| syntect | 5.3.0 | TextMate-grammar syntax highlighting | De facto Rust syntax highlighting crate; used by bat, delta, xi-editor; 150+ bundled languages |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| two-face | 0.5.1 | Extra syntax definitions from bat project | If syntect defaults lack needed languages (e.g., Svelte, TOML, Dockerfile) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| two-face | syntect defaults only | Fewer languages (~50 vs ~200), but smaller binary. Start with defaults, add two-face if gaps found |
| default-onig (C regex) | default-fancy (pure Rust regex) | fancy is ~2x slower but avoids C dependency. Project already has vendored C deps (libgit2, openssl) so onig is fine |

**Installation:**
```bash
# In src-tauri/Cargo.toml
syntect = { version = "5", default-features = false, features = ["default-fancy"] }
```

**Recommendation on regex engine:** Use `default-fancy` (pure Rust). The project targets cross-platform builds including CI. Adding onig introduces a C library build dependency that complicates the matrix. For diff line highlighting (not real-time editor highlighting), the ~2x slowdown of fancy-regex is negligible -- we highlight individual lines, not entire files in a loop. The `default-fancy` feature avoids linking Oniguruma and keeps the build simpler.

**Version verification:** syntect 5.3.0 is the latest on crates.io (confirmed via `cargo search`). two-face 0.5.1 is the latest.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── commands/
│   └── diff.rs              # Add syntax highlighting pass after word spans
├── git/
│   ├── types.rs             # Add MergedSpan type, update DiffLine
│   └── syntax.rs            # NEW: syntect integration module
└── ...
```

### Pattern 1: Syntax Highlighting Module (`git/syntax.rs`)

**What:** A dedicated module that owns the SyntaxSet singleton, provides `highlight_line_tokens()`, and handles scope-to-CSS-class mapping.

**When to use:** Always -- isolates syntect dependency from diff command logic.

**Example:**
```rust
// src-tauri/src/git/syntax.rs
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use std::sync::LazyLock;

/// Global singleton -- SyntaxSet is immutable and thread-safe after construction.
/// LazyLock initializes on first access, lives for program lifetime.
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| {
    SyntaxSet::load_defaults_newlines()
});

/// Map foreground Color (from theme) to a CSS class name.
/// Uses the VS Code Dark+ color palette (D-02).
fn color_to_css_class(color: Color) -> &'static str {
    // Compare RGB values (ignore alpha) against known VS Code Dark+ colors
    match (color.r, color.g, color.b) {
        (86, 156, 214) => "syn-keyword",    // #569cd6 - blue keywords
        (206, 145, 120) => "syn-string",     // #ce9178 - orange strings
        (106, 153, 85) => "syn-comment",     // #6a9955 - green comments
        (181, 206, 168) => "syn-number",     // #b5cea8 - light green numbers
        (78, 201, 176) => "syn-type",        // #4ec9b0 - teal types
        (220, 220, 170) => "syn-function",   // #dcdcaa - yellow functions
        (212, 212, 212) => "syn-text",       // #d4d4d4 - default text
        // ... more mappings ...
        _ => "syn-text", // fallback
    }
}

pub struct SyntaxToken {
    pub start: u32,
    pub end: u32,
    pub scope: String, // CSS class name like "syn-keyword"
}

/// Highlight a single line of code, returning byte-offset-based SyntaxTokens.
pub fn highlight_line_tokens(content: &str, extension: &str) -> Vec<SyntaxToken> {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let theme = /* load VS Code Dark+ equivalent theme */;
    let mut highlighter = HighlightLines::new(syntax, theme);

    let line_with_newline = if content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{}\n", content)
    };

    let ranges = highlighter
        .highlight_line(&line_with_newline, &SYNTAX_SET)
        .unwrap_or_default();

    let mut tokens = Vec::new();
    let mut offset: u32 = 0;
    for (style, text) in &ranges {
        let len = text.len() as u32;
        // Trim trailing newline from last token if present
        let effective_len = if offset + len > content.len() as u32 {
            content.len() as u32 - offset
        } else {
            len
        };
        if effective_len > 0 {
            let class = color_to_css_class(style.foreground);
            if class != "syn-text" { // Skip default-colored tokens
                tokens.push(SyntaxToken {
                    start: offset,
                    end: offset + effective_len,
                    scope: class.to_string(),
                });
            }
        }
        offset += len;
    }
    tokens
}
```

### Pattern 2: Color-to-Class Mapping (Preferred over Scope String Parsing)

**What:** Map syntect foreground colors (from VS Code Dark+ theme) to CSS class names.

**Why preferred:** The `HighlightLines` API returns `Vec<(Style, &str)>` where `Style.foreground` is an RGBA `Color`. The theme already maps scopes to colors. Mapping colors to CSS classes is a simple lookup table (15-20 entries), whereas parsing TextMate scope strings involves string manipulation and matching against dozens of patterns.

**How it works:**
1. Load a VS Code Dark+ compatible theme (syntect bundles themes that approximate it, or load a custom `.tmTheme`)
2. `highlight_line()` returns colored token segments
3. Match foreground color RGB values to CSS class names
4. Tokens with default text color are omitted or given `syn-text`

**Alternative (scope-based mapping):** Use `ParseState::parse_line()` to get `ScopeStackOp` events, reconstruct scope stacks, and match scope prefixes (e.g., `keyword.` -> `syn-keyword`). This is more fragile but more semantically correct. The color-based approach is recommended because it leverages syntect's own theme engine which has already resolved all scope priorities.

### Pattern 3: Merged Span Type

**What:** A new `MergedSpan` struct that replaces separate `syntax_tokens` and `word_spans` arrays on `DiffLine`.

**Design:**
```rust
#[derive(Debug, Serialize, Clone, Default)]
pub struct MergedSpan {
    pub start: u32,
    pub end: u32,
    pub syntax_class: String,  // CSS class name, e.g., "syn-keyword" or "" for plain text
    pub emphasized: bool,       // true = word-diff highlight background
}
```

**Merge algorithm:** Given sorted syntax tokens and sorted word spans (both by `start`), sweep through both arrays simultaneously:
1. Start at byte 0
2. Advance to the nearest boundary (start or end of either syntax token or word span)
3. At each sub-range, determine: (a) which syntax class applies, (b) whether emphasized applies
4. Emit a `MergedSpan` for that sub-range
5. Continue until end of line content

**DiffLine update:** Replace `syntax_tokens: Vec<SyntaxToken>` and `word_spans: Vec<WordSpan>` with `spans: Vec<MergedSpan>`. The frontend renders a single loop.

### Pattern 4: Theme Loading Strategy

**What:** Load a custom `.tmTheme` that matches VS Code Dark+ colors or construct a theme programmatically.

**Options (in preference order):**
1. **Bundled `.tmTheme` file:** Download the VS Code Dark+ tmTheme, include it as a static asset compiled into the binary with `include_bytes!()`, and load with `ThemeSet::get_theme()`. Most accurate color mapping.
2. **Use syntect's closest bundled theme:** `base16-ocean.dark` is the closest to VS Code Dark+, but colors will not match exactly. Less maintenance.
3. **Build a Theme struct programmatically:** Construct scope-to-color mappings in code. Maximum control but most code.

**Recommendation:** Option 1 (bundled `.tmTheme`). VS Code's Dark+ theme is open source (MIT license). Download the `.tmTheme` equivalent once, embed it. This gives exact color mapping and the simplest `color_to_css_class()` lookup.

### Anti-Patterns to Avoid
- **Highlighting per-line without state continuity:** For diff context, each line is independent (different line numbers, possibly reordered). But for multi-line strings/comments, syntect's `HighlightLines` maintains parser state across `highlight_line()` calls. For diff lines, we cannot maintain state across non-contiguous lines. Use a **fresh highlighter per hunk** with lines fed in order. For lines from different positions in the file, accept that multi-line constructs may not highlight perfectly.
- **Parsing scope strings with regex:** Scope strings like `source.rust keyword.control.rust` are complex nested hierarchies. Do NOT parse them manually. Let the theme engine resolve them.
- **Inline styles from syntect:** syntect's HTML module can generate inline `style="color: #..."` attributes. The project rule (CLAUDE.md) says never inline colors. Use CSS classes exclusively.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Syntax grammar parsing | Custom regex-based tokenizer | syntect SyntaxSet + HighlightLines | TextMate grammars handle 1000+ edge cases per language |
| Language detection | File extension lookup table | syntect `find_syntax_by_extension()` | Handles multi-extension files, shebangs, special cases |
| Scope resolution | Manual scope priority matching | syntect Theme engine | Theme precedence rules are complex; syntect handles it |
| Span merging | Nested DOM elements | Backend merge into flat spans | Flat array avoids overlapping DOM spans and z-index issues |

**Key insight:** syntect's `HighlightLines` API does all the heavy lifting. The project only needs to: (1) map output colors to CSS classes, and (2) merge with word spans. Both are straightforward algorithms.

## Common Pitfalls

### Pitfall 1: Diff Lines Are Not Contiguous File Lines
**What goes wrong:** Using a single `HighlightLines` instance across an entire hunk assumes lines are contiguous in the source file. Context lines and add/delete lines may come from different positions.
**Why it happens:** Diff hunks interleave old-file and new-file lines. A delete line might be from old line 10, followed by an add line at new line 10 -- they are different text at the same position.
**How to avoid:** Create a fresh `HighlightLines` per line, OR accept slightly degraded multi-line construct highlighting. For most code (single-line statements), this is invisible. Multi-line strings/comments may lose color on continuation lines.
**Warning signs:** A multi-line string appears colored on its first line but plain on continuation lines within a hunk.

### Pitfall 2: Byte Offset Mismatch Between syntect and Content
**What goes wrong:** syntect's `highlight_line()` expects a line ending with `\n` (when using `load_defaults_newlines`). If the content field on DiffLine does NOT end with `\n`, byte offsets will mismatch.
**Why it happens:** git2's `line.content()` may or may not include trailing newlines depending on the line.
**How to avoid:** Normalize: append `\n` if missing before highlighting, then clamp token end offsets to `content.len()` when building SyntaxTokens.
**Warning signs:** Last token on a line extends past the content boundary, causing a slice panic in the frontend.

### Pitfall 3: SyntaxSet Loading Is Expensive
**What goes wrong:** Creating a new `SyntaxSet::load_defaults_newlines()` on every diff request takes ~10-50ms and allocates significant memory.
**Why it happens:** SyntaxSet deserializes ~200KB of compressed binary data.
**How to avoid:** Use `LazyLock<SyntaxSet>` (or `OnceLock`) as a global singleton. SyntaxSet is immutable and `Send + Sync` after construction.
**Warning signs:** Diff loading becomes noticeably slower after adding syntax highlighting.

### Pitfall 4: Theme Loading and Color Precision
**What goes wrong:** Color-to-class mapping uses exact RGB byte matching. If the theme's actual color values differ by even 1 from expected (e.g., #569CD6 vs #569BD6), the mapping misses.
**Why it happens:** Different theme files may have slightly different color values for "the same" semantic meaning.
**How to avoid:** Print all unique foreground colors from a test highlight run. Build the mapping table from actual observed colors, not assumed ones. Include a catch-all fallback to `syn-text`.
**Warning signs:** Code appears without syntax coloring even for recognized file types.

### Pitfall 5: Merged Span Array Must Cover Entire Content
**What goes wrong:** If the merged span array has gaps (byte ranges not covered), the frontend will skip rendering those characters.
**Why it happens:** Syntax tokens may not cover whitespace or newlines. Word spans may not exist for context lines.
**How to avoid:** The merge algorithm must generate spans covering byte 0 to `content.len()` with no gaps. Uncovered ranges get `syntax_class: ""` and `emphasized: false`.
**Warning signs:** Characters disappear from rendered diff lines.

### Pitfall 6: Binary Size Increase
**What goes wrong:** syntect's bundled syntax definitions add ~200KB to the binary. With `two-face`, this increases to ~700KB+.
**Why it happens:** Compressed TextMate grammar data is embedded in the binary.
**How to avoid:** Start with syntect defaults only. Add two-face only if specific language support gaps are found. Use `default-fancy` to avoid the Oniguruma C library.
**Warning signs:** CI build times increase, binary size jumps unexpectedly.

## Code Examples

### Example 1: SyntaxSet Singleton with LazyLock
```rust
// Source: syntect docs + Rust std LazyLock
use std::sync::LazyLock;
use syntect::parsing::SyntaxSet;

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| {
    SyntaxSet::load_defaults_newlines()
});
```

### Example 2: Highlight a Single Line
```rust
// Source: syntect HighlightLines docs
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;

fn highlight_one_line(content: &str, extension: &str) -> Vec<(syntect::highlighting::Style, String)> {
    let syntax = SYNTAX_SET
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    let line = if content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{}\n", content)
    };

    h.highlight_line(&line, &SYNTAX_SET)
        .unwrap_or_default()
        .into_iter()
        .map(|(style, text)| (style, text.to_string()))
        .collect()
}
```

### Example 3: Extract Extension from FileDiff Path
```rust
// Source: std::path
fn extension_from_path(path: &str) -> &str {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
}
```

### Example 4: Merge Syntax Tokens and Word Spans
```rust
// Pseudocode for the merge algorithm
fn merge_spans(
    syntax_tokens: &[SyntaxToken],
    word_spans: &[WordSpan],
    content_len: u32,
) -> Vec<MergedSpan> {
    let mut result = Vec::new();
    let mut pos: u32 = 0;

    // Collect all boundaries from both arrays
    let mut boundaries: Vec<u32> = Vec::new();
    for t in syntax_tokens {
        boundaries.push(t.start);
        boundaries.push(t.end);
    }
    for w in word_spans {
        boundaries.push(w.start);
        boundaries.push(w.end);
    }
    boundaries.push(0);
    boundaries.push(content_len);
    boundaries.sort_unstable();
    boundaries.dedup();

    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end || start >= content_len {
            continue;
        }
        let end = end.min(content_len);

        // Find which syntax token covers this range
        let syntax_class = syntax_tokens.iter()
            .find(|t| t.start <= start && t.end >= end)
            .map(|t| t.scope.as_str())
            .unwrap_or("");

        // Find which word span covers this range
        let emphasized = word_spans.iter()
            .any(|w| w.start <= start && w.end >= end && w.emphasized);

        result.push(MergedSpan {
            start,
            end,
            syntax_class: syntax_class.to_string(),
            emphasized,
        });
    }
    result
}
```

### Example 5: CSS Custom Properties for Syntax Colors
```css
/* Source: VS Code Dark+ theme colors (D-02) */
:root {
    --color-syn-keyword: #569cd6;
    --color-syn-string: #ce9178;
    --color-syn-comment: #6a9955;
    --color-syn-number: #b5cea8;
    --color-syn-type: #4ec9b0;
    --color-syn-function: #dcdcaa;
    --color-syn-variable: #9cdcfe;
    --color-syn-constant: #4fc1ff;
    --color-syn-operator: #d4d4d4;
    --color-syn-punctuation: #808080;
    --color-syn-attribute: #9cdcfe;
    --color-syn-tag: #569cd6;
    --color-syn-property: #9cdcfe;
    --color-syn-regex: #d16969;
    --color-syn-escape: #d7ba7d;
    --color-syn-text: var(--color-text);
}
```

### Example 6: Frontend Merged Span Rendering (Svelte 5)
```svelte
{#each line.spans as span}
  <span
    class="{span.syntax_class}{span.emphasized
      ? (line.origin === 'Add' ? ' word-add' : ' word-delete')
      : ''}"
  >{line.content.slice(span.start, span.end)}</span>
{/each}
```

### Example 7: CSS Opacity Reduction for Add/Delete Lines (SYNT-03)
```css
/* Context lines: full-color syntax */
.diff-line-context .syn-keyword { color: var(--color-syn-keyword); }

/* Add/Delete lines: desaturated syntax */
.diff-line-add [class^="syn-"],
.diff-line-delete [class^="syn-"] {
    opacity: 0.7;
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| syntect 4.x with onig default | syntect 5.x with default-fancy option | v5.0 (2022) | Pure Rust builds possible, no C dependency |
| SyntaxSet::load_defaults() | SyntaxSet::load_defaults_newlines() | v3.0+ | Must use newlines variant for line-by-line highlighting |
| lazy_static! macro | std::sync::LazyLock | Rust 1.80 (stable) | No external crate needed for lazy statics |

**Deprecated/outdated:**
- `syntect::easy::HighlightFile`: Still exists but not needed for our use case (we highlight individual lines, not files)
- `tokens_to_classed_spans()`: Deprecated in favor of `ClassedHTMLGenerator`
- `SyntaxSet::load_defaults()`: Use `load_defaults_newlines()` instead for line-by-line processing

## Open Questions

1. **Theme file availability**
   - What we know: VS Code Dark+ theme is MIT-licensed; syntect can load `.tmTheme` files
   - What's unclear: Whether a ready-made `.tmTheme` file exists that exactly matches VS Code Dark+ colors, or if we need to use a close approximation from syntect's bundled themes
   - Recommendation: Start with syntect's bundled `base16-ocean.dark` theme, extract the unique foreground colors, build the mapping table, then optionally swap to a more accurate theme later

2. **Svelte syntax support**
   - What we know: syntect's default syntax set may not include Svelte. two-face (bat's extra syntaxes) likely does.
   - What's unclear: Whether the default set covers all file types the user commonly diffs
   - Recommendation: Start with syntect defaults. Log a warning for unrecognized extensions. Add two-face in a follow-up if needed.

3. **Multi-line construct accuracy**
   - What we know: Highlighting individual diff lines without parser state from preceding lines means multi-line strings/comments may not highlight correctly on continuation lines
   - What's unclear: How noticeable this will be in practice for real diffs
   - Recommendation: Accept this limitation for now. Most diff lines are single statements. This is a known tradeoff in every diff tool that does syntax highlighting.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust: built-in #[test] with cargo test; Frontend: vitest 3.x |
| Config file | Cargo.toml (Rust); vite.config.ts (vitest) |
| Quick run command | `cargo test -p trunk --lib -- syntax && bun run test` |
| Full suite command | `cargo test -p trunk && bun run test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SYNT-01 | Rust/TS/JSON file lines produce non-empty syntax_tokens with known CSS classes | unit | `cargo test -p trunk test_diff::syntax_tokens_populated -x` | No - Wave 0 |
| SYNT-02 | Extension detection resolves correct language for .rs, .ts, .json, .svelte | unit | `cargo test -p trunk test_diff::syntax_extension_detection -x` | No - Wave 0 |
| SYNT-03 | Frontend applies opacity reduction class on add/delete lines | unit | `bun run test -- DiffPanel.test.ts` | Partially - existing DiffPanel.test.ts needs new cases |
| SYNT-01 | Merged spans cover entire content length (no gaps) | unit | `cargo test -p trunk test_diff::merged_spans_cover_content -x` | No - Wave 0 |
| SYNT-01 | DiffLine.spans serializes correctly over IPC | integration | `cargo test -p trunk test_integ_serde -x` | Partially - existing serde tests need update |

### Sampling Rate
- **Per task commit:** `cargo test -p trunk && bun run test`
- **Per wave merge:** Full suite (Rust + frontend + svelte-check)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/tests/test_diff.rs` -- add syntax highlighting test cases (SYNT-01, SYNT-02)
- [ ] `src/components/DiffPanel.test.ts` -- add merged span rendering tests with syntax classes
- [ ] `src-tauri/src/git/syntax.rs` -- new module needs unit tests for color-to-class mapping

## Project Constraints (from CLAUDE.md)

- Never inline colors -- always use CSS custom properties from the theme
- Never fight layout with positioning hacks -- use grid/flexbox
- All git operations go through git2 crate, no shelling out
- Run all 6 checks before push: fmt, clippy, test, vitest, svelte-check, biome
- Include regression tests with bug fixes

## Sources

### Primary (HIGH confidence)
- [syntect crates.io](https://crates.io/crates/syntect) - version 5.3.0 confirmed
- [syntect docs - HighlightLines](https://docs.rs/syntect/5.3.0/syntect/easy/struct.HighlightLines.html) - API: `highlight_line()` returns `Vec<(Style, &str)>`
- [syntect docs - Color](https://docs.rs/syntect/5.3.0/syntect/highlighting/struct.Color.html) - Color struct: `{ r: u8, g: u8, b: u8, a: u8 }`
- [syntect docs - SyntaxSet](https://docs.rs/syntect/5.3.0/syntect/parsing/struct.SyntaxSet.html) - `find_syntax_by_extension()`, `load_defaults_newlines()`
- [syntect docs - ThemeSet](https://docs.rs/syntect/5.3.0/syntect/highlighting/struct.ThemeSet.html) - bundled themes, `load_defaults()`
- [syntect docs - ClassStyle](https://docs.rs/syntect/5.3.0/syntect/html/enum.ClassStyle.html) - CSS class generation from scopes
- [Sublime Text Scope Naming](https://www.sublimetext.com/docs/scope_naming.html) - TextMate scope naming conventions
- [syntect GitHub](https://github.com/trishume/syntect) - feature flags, default-fancy vs default-onig

### Secondary (MEDIUM confidence)
- [two-face crate](https://docs.rs/two-face/latest/two_face/) - extra syntaxes from bat, ~200 languages
- [syntect docs - ParseState](https://docs.rs/syntect/5.3.0/syntect/parsing/struct.ParseState.html) - lower-level API for scope operations

### Tertiary (LOW confidence)
- Training data knowledge on VS Code Dark+ exact color values -- needs verification against actual theme file

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - syntect 5.3.0 is the clear choice, API verified from official docs
- Architecture: HIGH - color-to-class mapping pattern verified from docs, merge algorithm is a standard sweep-line approach
- Pitfalls: HIGH - byte offset issues, singleton loading, and state continuity are well-documented concerns

**Research date:** 2026-03-28
**Valid until:** 2026-04-28 (syntect is stable, no breaking changes expected)
