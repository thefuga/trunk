use std::sync::LazyLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, ThemeSet};
use syntect::parsing::SyntaxSet;

use super::types::{MergedSpan, SyntaxToken, WordSpan};

static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// Map a syntect foreground Color (from base16-ocean.dark theme) to a CSS class name.
/// Colors discovered by running highlight_line() on sample code and observing
/// the actual RGB values the theme produces for each token type.
/// Returns empty string for default/plain text colors (no syntax class needed).
fn color_to_css_class(color: Color) -> &'static str {
    match (color.r, color.g, color.b) {
        // Keywords: fn, let, if, return, class, def, pub, use, mod, etc.
        (180, 142, 173) => "syn-keyword",
        // Strings: "hello", 'world'
        (163, 190, 140) => "syn-string",
        // Comments: //, #, /* */
        (101, 115, 126) => "syn-comment",
        // Numbers and built-in constants: 42, true, false, null
        (208, 135, 112) => "syn-number",
        // Function names: main, foo
        (143, 161, 179) => "syn-function",
        // Type/class names: Bar, MyStruct (entity.name.class)
        (235, 203, 139) => "syn-type",
        // Variables/parameters (Python scope: variable.parameter)
        (191, 97, 106) => "syn-variable",
        // Default foreground text -- skip (punctuation, operators, identifiers)
        (192, 197, 206) | (239, 241, 245) => "",
        // Catch-all fallback for any unrecognized color
        _ => "",
    }
}

/// Map extensions that syntect doesn't bundle to a supported fallback.
/// TypeScript → JavaScript, Svelte/Vue/JSX/TSX → JavaScript.
fn fallback_extension(ext: &str) -> &str {
    match ext {
        "ts" | "mts" | "cts" | "tsx" | "jsx" | "svelte" | "vue" => "js",
        _ => ext,
    }
}

/// Returns true if syntect has a real syntax definition for this extension (not plain text).
pub fn has_syntax_for_extension(ext: &str) -> bool {
    if ext.is_empty() {
        return false;
    }
    let resolved = fallback_extension(ext);
    SYNTAX_SET
        .find_syntax_by_extension(resolved)
        .is_some_and(|s| s.name != "Plain Text")
}

/// Extract file extension from a path string.
pub fn extension_from_path(path: &str) -> &str {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
}

/// Create a reusable highlighter for a file extension.
/// Returns None if the extension has no syntax definition (plain text).
pub fn create_highlighter(extension: &str) -> Option<HighlightLines<'static>> {
    let resolved = fallback_extension(extension);
    let syntax = SYNTAX_SET.find_syntax_by_extension(resolved)?;
    if syntax.name == "Plain Text" {
        return None;
    }
    let theme = &THEME_SET.themes["base16-ocean.dark"];
    Some(HighlightLines::new(syntax, theme))
}

/// Highlight a single line using a reusable highlighter instance.
/// The highlighter maintains parse state across calls, giving correct multi-line highlighting.
pub fn highlight_line_with(
    highlighter: &mut HighlightLines<'_>,
    content: &str,
) -> Vec<SyntaxToken> {
    // syntect requires newline-terminated lines with load_defaults_newlines
    let normalized = if content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{}\n", content)
    };

    let ranges = highlighter
        .highlight_line(&normalized, &SYNTAX_SET)
        .unwrap_or_default();

    let mut tokens = Vec::new();
    let mut offset: u32 = 0;
    let content_len = content.len() as u32;

    for (style, text) in &ranges {
        let len = text.len() as u32;
        let end = (offset + len).min(content_len);
        if offset < content_len && end > offset {
            let class = color_to_css_class(style.foreground);
            if !class.is_empty() {
                tokens.push(SyntaxToken {
                    start: offset,
                    end,
                    scope: class.to_string(),
                });
            }
        }
        offset += len;
    }
    tokens
}

/// Highlight a single line of code (standalone — creates a fresh highlighter).
/// Prefer `create_highlighter` + `highlight_line_with` for batch processing.
pub fn highlight_line_tokens(content: &str, extension: &str) -> Vec<SyntaxToken> {
    let Some(mut hl) = create_highlighter(extension) else {
        return vec![];
    };
    highlight_line_with(&mut hl, content)
}

/// Merge syntax tokens and word spans into a single sorted span array.
/// The resulting array covers bytes 0 to content_len with no gaps.
pub fn merge_spans(
    syntax_tokens: &[SyntaxToken],
    word_spans: &[WordSpan],
    content_len: u32,
) -> Vec<MergedSpan> {
    if content_len == 0 {
        return vec![];
    }

    // Collect all boundary points
    let mut boundaries: Vec<u32> = Vec::new();
    boundaries.push(0);
    boundaries.push(content_len);
    for t in syntax_tokens {
        boundaries.push(t.start);
        boundaries.push(t.end);
    }
    for w in word_spans {
        boundaries.push(w.start);
        boundaries.push(w.end);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut result = Vec::new();
    for window in boundaries.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end || start >= content_len {
            continue;
        }
        let end = end.min(content_len);

        // Find which syntax token covers this range
        let syntax_class = syntax_tokens
            .iter()
            .find(|t| t.start <= start && t.end >= end)
            .map(|t| t.scope.clone())
            .unwrap_or_default();

        // Find which word span covers this range and is emphasized
        let emphasized = word_spans
            .iter()
            .any(|w| w.start <= start && w.end >= end && w.emphasized);

        result.push(MergedSpan {
            start,
            end,
            syntax_class,
            emphasized,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_rust_produces_tokens() {
        let tokens = highlight_line_tokens("fn main() {", "rs");
        assert!(!tokens.is_empty(), "Rust code should produce syntax tokens");
        // "fn" is a keyword
        let has_keyword = tokens.iter().any(|t| t.scope == "syn-keyword");
        assert!(
            has_keyword,
            "Rust 'fn' should be highlighted as syn-keyword"
        );
    }

    #[test]
    fn highlight_unknown_extension_produces_no_tokens() {
        let tokens = highlight_line_tokens("some content here", "xyz999");
        assert!(
            tokens.is_empty(),
            "Unknown extension should produce no syntax tokens"
        );
    }

    #[test]
    fn extension_from_path_extracts_correctly() {
        assert_eq!(extension_from_path("main.rs"), "rs");
        assert_eq!(extension_from_path("src/lib/types.ts"), "ts");
        assert_eq!(extension_from_path("data.json"), "json");
        assert_eq!(extension_from_path("noext"), "");
        assert_eq!(extension_from_path(""), "");
    }

    #[test]
    fn merge_spans_covers_full_range() {
        let syntax = vec![
            SyntaxToken {
                start: 0,
                end: 2,
                scope: "syn-keyword".to_string(),
            },
            SyntaxToken {
                start: 3,
                end: 7,
                scope: "syn-function".to_string(),
            },
        ];
        let word = vec![];
        let merged = merge_spans(&syntax, &word, 10);

        // Should cover 0..10 with no gaps
        assert_eq!(merged[0].start, 0);
        assert_eq!(merged.last().unwrap().end, 10);
        for w in merged.windows(2) {
            assert_eq!(w[0].end, w[1].start, "No gaps between merged spans");
        }
    }

    #[test]
    fn merge_spans_with_emphasis() {
        let syntax = vec![SyntaxToken {
            start: 0,
            end: 5,
            scope: "syn-keyword".to_string(),
        }];
        let word = vec![
            WordSpan {
                start: 0,
                end: 3,
                emphasized: false,
            },
            WordSpan {
                start: 3,
                end: 5,
                emphasized: true,
            },
        ];
        let merged = merge_spans(&syntax, &word, 5);

        // Should have at least 2 spans due to word span boundary at 3
        let emph_spans: Vec<_> = merged.iter().filter(|s| s.emphasized).collect();
        assert!(
            !emph_spans.is_empty(),
            "Should have emphasized spans from word diff"
        );
        // The emphasized span should also carry the syntax class
        for s in &emph_spans {
            assert_eq!(s.syntax_class, "syn-keyword");
        }
    }

    #[test]
    fn merge_spans_empty_content() {
        let merged = merge_spans(&[], &[], 0);
        assert!(merged.is_empty(), "Empty content should produce no spans");
    }

    #[test]
    fn color_mapping_keywords() {
        assert_eq!(
            color_to_css_class(Color {
                r: 180,
                g: 142,
                b: 173,
                a: 255
            }),
            "syn-keyword"
        );
    }

    #[test]
    fn color_mapping_strings() {
        assert_eq!(
            color_to_css_class(Color {
                r: 163,
                g: 190,
                b: 140,
                a: 255
            }),
            "syn-string"
        );
    }

    #[test]
    fn color_mapping_comments() {
        assert_eq!(
            color_to_css_class(Color {
                r: 101,
                g: 115,
                b: 126,
                a: 255
            }),
            "syn-comment"
        );
    }

    #[test]
    fn color_mapping_default_text_empty() {
        assert_eq!(
            color_to_css_class(Color {
                r: 192,
                g: 197,
                b: 206,
                a: 255
            }),
            ""
        );
    }

    #[test]
    fn highlight_typescript_uses_js_fallback() {
        let tokens = highlight_line_tokens("const x: number = 42;", "ts");
        assert!(
            !tokens.is_empty(),
            "TypeScript should produce tokens via JS fallback"
        );
        let has_keyword = tokens.iter().any(|t| t.scope == "syn-keyword");
        assert!(has_keyword, "'const' should be highlighted as syn-keyword");
    }

    #[test]
    fn highlight_tsx_uses_js_fallback() {
        assert!(has_syntax_for_extension("tsx"));
        let tokens = highlight_line_tokens("export default function App() {}", "tsx");
        assert!(
            !tokens.is_empty(),
            "TSX should produce tokens via JS fallback"
        );
    }

    #[test]
    fn highlight_svelte_uses_js_fallback() {
        assert!(has_syntax_for_extension("svelte"));
        let tokens = highlight_line_tokens("const count = 0;", "svelte");
        assert!(
            !tokens.is_empty(),
            "Svelte should produce tokens via JS fallback"
        );
    }

    #[test]
    fn color_mapping_unknown_fallback() {
        assert_eq!(
            color_to_css_class(Color {
                r: 1,
                g: 2,
                b: 3,
                a: 255
            }),
            ""
        );
    }
}
