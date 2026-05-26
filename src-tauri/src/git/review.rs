//! Phase 70: pure markdown renderer for review sessions.
//!
//! Pure Rust logic: takes `&ReviewSession` + `&git2::Repository`, returns a
//! single `String`. No `tauri::*` imports (L-01), no calls into
//! `crate::git::syntax` (L-10), never panics (L-04).
//!
//! This module is `tauri`-free and exposes ONE public function: [`render`].
//! All resolution failures are routed INTO the returned markdown (per L-04 +
//! L-09); the renderer NEVER returns an error.

use crate::commands::review::{classify_anchor, OrphanReason};
use crate::git::types::{Anchor, ReviewSession, Side, Source};

/// Render-only failure kinds. Does NOT cross the IPC wire (the Phase 69
/// `OrphanReason` does — never extend it). All variants funnel into either the
/// resolved per-file section (via the `[binary file, no excerpt]` placeholder
/// for `Binary`) or the unresolvable trailing section (everything else).
#[allow(dead_code)] // wired up in task 2 / task 3
#[derive(Debug)]
pub(crate) enum ExcerptError {
    /// `blob.is_binary()` returned true; emit `[binary file, no excerpt]`
    /// INSIDE the resolved per-file section (L-05, not the unresolvable
    /// section).
    Binary,
    /// `classify_anchor` rejected the anchor — wraps the Phase 69 reason.
    Orphaned(OrphanReason),
    /// Generic re-resolution failure (git2 error during slicing).
    ResolutionFailed,
    /// Diff replay-slice produced an empty body (file unchanged from parent at
    /// the anchored commit; Pitfall 2).
    NoHunks,
}

/// L-03: fence length is `max(3, longest_contiguous_backtick_run + 1)`.
/// Linear byte-scan over the entire body — counter resets on any non-backtick
/// byte (including newlines), so two separate `` ``` `` runs split by a
/// newline do NOT compose into a longer run. CommonMark §4.5 requires the
/// opening fence be strictly longer than any inner backtick run.
#[allow(dead_code)] // wired up in task 3
pub(crate) fn fence_length(body: &str) -> usize {
    let mut longest = 0usize;
    let mut current = 0usize;
    for b in body.as_bytes() {
        if *b == b'`' {
            current += 1;
            if current > longest {
                longest = current;
            }
        } else {
            current = 0;
        }
    }
    std::cmp::max(3, longest + 1)
}

/// L-07: extension → markdown fence language tag for `Source::FullFile`
/// excerpts. Hand-rolled per L-10 (no syntect call). Distinct from
/// `syntax::fallback_extension` which targets syntect syntax IDs.
#[allow(dead_code)] // wired up in task 3
pub(crate) fn fence_language(file_path: &str) -> &'static str {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust",
        "ts" | "mts" | "cts" => "typescript",
        "tsx" => "tsx",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "svelte" => "svelte",
        "json" => "json",
        "md" | "markdown" => "markdown",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "css" => "css",
        "html" | "htm" => "html",
        "sh" | "bash" => "bash",
        "py" => "python",
        "go" => "go",
        _ => "text",
    }
}

/// L-06 line-indexing convention: 1-based inclusive bounds over
/// `str::lines()` semantics, with CRLF→LF normalisation applied to the body
/// BEFORE slicing. Mirrors `classify_anchor` at `commands/review.rs:358` so a
/// comment that resolves at classification time also resolves at render time —
/// one convention applies on both sides (capture and render). RESEARCH Item 2
/// Option (a): `str::lines()` already handles `\r\n` as one boundary, so line
/// indices are unchanged; only the bytes inside the fence become LF-only.
fn slice_lines(content: &str, start_line: u32, end_line: u32) -> Option<String> {
    if start_line == 0 || end_line < start_line {
        return None;
    }
    let normalised = content.replace("\r\n", "\n");
    let lines: Vec<&str> = normalised.lines().collect();
    let line_count = lines.len() as u32;
    if end_line > line_count {
        return None;
    }
    let start_idx = (start_line - 1) as usize;
    let end_idx = end_line as usize;
    Some(lines[start_idx..end_idx].join("\n"))
}

/// L-02 + L-05 + L-06: re-resolve a `Source::FullFile` excerpt by reading the
/// blob fresh from git2. Side semantics mirror `classify_anchor`
/// (`commands/review.rs:339-346`): `New` reads the commit's tree, `Old` reads
/// the parent's. `blob.is_binary()` short-circuits to `ExcerptError::Binary`
/// BEFORE any slicing (L-05). Caller MUST have run `classify_anchor` first
/// (Pitfall 1) — `slice_full_file` does NOT re-gate.
#[allow(dead_code)] // wired up in task 3
pub(crate) fn slice_full_file(
    repo: &git2::Repository,
    anchor: &Anchor,
) -> Result<String, ExcerptError> {
    let oid =
        git2::Oid::from_str(&anchor.commit_oid).map_err(|_| ExcerptError::ResolutionFailed)?;
    let commit = repo
        .find_commit(oid)
        .map_err(|_| ExcerptError::ResolutionFailed)?;
    let tree = match anchor.side {
        Side::New => commit.tree().map_err(|_| ExcerptError::ResolutionFailed)?,
        Side::Old => commit
            .parent(0)
            .map_err(|_| ExcerptError::ResolutionFailed)?
            .tree()
            .map_err(|_| ExcerptError::ResolutionFailed)?,
    };
    let entry = tree
        .get_path(std::path::Path::new(&anchor.file_path))
        .map_err(|_| ExcerptError::ResolutionFailed)?;
    let blob = repo
        .find_blob(entry.id())
        .map_err(|_| ExcerptError::ResolutionFailed)?;
    if blob.is_binary() {
        return Err(ExcerptError::Binary);
    }
    let content = String::from_utf8_lossy(blob.content()).into_owned();
    slice_lines(&content, anchor.start_line, anchor.end_line).ok_or(ExcerptError::ResolutionFailed)
}

/// L-02 + Phase 67 L-03: re-resolve a `Source::Diff` excerpt by replaying
/// `diff_tree_to_tree(parent, commit)` with `pathspec(file_path)` and keeping
/// lines whose side-lineno overlaps `[start_line, end_line]`. Lines with no
/// side-lineno (the opposing-side `-`/`+` rows) are kept per Phase 67 L-03 so
/// the body matches what the cached_excerpt looked like at capture. Empty
/// walk → `ExcerptError::NoHunks` (Pitfall 2 — file unchanged from parent at
/// this commit). Root-commit guard mirrors `commands/diff.rs:410-414`.
#[allow(dead_code)] // wired up in task 3
pub(crate) fn slice_diff(repo: &git2::Repository, anchor: &Anchor) -> Result<String, ExcerptError> {
    let oid =
        git2::Oid::from_str(&anchor.commit_oid).map_err(|_| ExcerptError::ResolutionFailed)?;
    let commit = repo
        .find_commit(oid)
        .map_err(|_| ExcerptError::ResolutionFailed)?;
    let commit_tree = commit.tree().map_err(|_| ExcerptError::ResolutionFailed)?;

    let mut opts = git2::DiffOptions::new();
    opts.pathspec(&anchor.file_path);

    let diff = if commit.parent_count() == 0 {
        repo.diff_tree_to_tree(None, Some(&commit_tree), Some(&mut opts))
            .map_err(|_| ExcerptError::ResolutionFailed)?
    } else {
        let parent_tree = commit
            .parent(0)
            .map_err(|_| ExcerptError::ResolutionFailed)?
            .tree()
            .map_err(|_| ExcerptError::ResolutionFailed)?;
        repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Some(&mut opts))
            .map_err(|_| ExcerptError::ResolutionFailed)?
    };

    let side = anchor.side.clone();
    let start_line = anchor.start_line;
    let end_line = anchor.end_line;
    let mut out = String::new();
    diff.foreach(
        &mut |_d, _p| true,
        None,
        Some(&mut |_d, _h| true),
        Some(&mut |_d, _h, line| {
            let lineno = match side {
                Side::New => line.new_lineno(),
                Side::Old => line.old_lineno(),
            };
            // Lines with a side-lineno: keep if in [start_line, end_line].
            // Lines WITHOUT one (the opposing-side change row): keep iff
            // origin matches the opposing-direction change marker (Phase 67
            // L-03 — visually anchors the range).
            let in_range = match lineno {
                Some(n) => n >= start_line && n <= end_line,
                None => matches!(
                    (side.clone(), line.origin()),
                    (Side::New, '-') | (Side::Old, '+')
                ),
            };
            if in_range {
                let prefix = match line.origin() {
                    '+' | '-' | ' ' => line.origin(),
                    _ => ' ',
                };
                out.push(prefix);
                out.push_str(&String::from_utf8_lossy(line.content()));
            }
            true
        }),
    )
    .map_err(|_| ExcerptError::ResolutionFailed)?;

    if out.is_empty() {
        Err(ExcerptError::NoHunks)
    } else {
        // L-06 second clause: CRLF→LF normalise the body inside the fence.
        Ok(out.replace("\r\n", "\n"))
    }
}

/// Gate-then-resolve dispatch (Pitfall 1): `classify_anchor` is the MANDATORY
/// first call. On `Ok(())`, dispatch to `slice_full_file` or `slice_diff` by
/// `anchor.source`. On `Err(reason)`, wrap into `ExcerptError::Orphaned`
/// WITHOUT entering the slicers — a `Side::Old` anchor on a root commit would
/// otherwise hit `commit.parent(0)` and surface as `ResolutionFailed`
/// (wrong: the correct reason is `FileGone`).
#[allow(dead_code)] // wired up in task 3
pub(crate) fn try_resolve_excerpt(
    repo: &git2::Repository,
    anchor: &Anchor,
) -> Result<String, ExcerptError> {
    classify_anchor(anchor, repo).map_err(ExcerptError::Orphaned)?;
    match anchor.source {
        Source::FullFile => slice_full_file(repo, anchor),
        Source::Diff => slice_diff(repo, anchor),
    }
}

/// Top-level pure renderer. Placeholder scaffold — task 3 implements the full
/// D-03..D-10 section assembly. Always returns a `String`, never panics.
#[allow(dead_code)] // task 3 fleshes this out
pub fn render(_session: &ReviewSession, _repo: &git2::Repository) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Oid, Repository, Signature};
    use tempfile::TempDir;

    // ── Test harness lifted from commands/review.rs:1135-2102 ─────────────
    // Real git2 + tempfile (classical TDD: no mocks). `_dir` field keeps the
    // TempDir alive for the test's duration; drop deletes it.

    fn sig() -> Signature<'static> {
        Signature::new("Test", "test@example.com", &git2::Time::new(0, 0)).unwrap()
    }

    fn commit_with_file(
        repo: &Repository,
        message: &str,
        parents: &[Oid],
        path: &str,
        content: &[u8],
    ) -> Oid {
        let blob_oid = repo.blob(content).unwrap();
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

    /// Empty-tree commit (no files). Used as the parent of `commit_with_file`
    /// commits so the diff-replay walks see a single added file.
    fn empty_commit(repo: &Repository, message: &str, parents: &[Oid]) -> Oid {
        let tree_oid = repo.treebuilder(None).unwrap().write().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let parent_commits: Vec<_> = parents
            .iter()
            .map(|oid| repo.find_commit(*oid).unwrap())
            .collect();
        let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();
        let s = sig();
        repo.commit(None, &s, &s, message, &tree, &parent_refs)
            .unwrap()
    }

    fn make_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        (dir, repo)
    }

    fn anchor(
        commit_oid: Oid,
        file_path: &str,
        source: Source,
        side: Side,
        start_line: u32,
        end_line: u32,
    ) -> Anchor {
        Anchor {
            commit_oid: commit_oid.to_string(),
            file_path: file_path.to_string(),
            source,
            side,
            start_line,
            end_line,
        }
    }

    // ── Task 1: fence_length unit tests (L-03) ────────────────────────────

    #[test]
    fn fence_length_floor_with_no_backticks() {
        assert_eq!(fence_length("hello world\n"), 3);
    }

    #[test]
    fn fence_length_floor_on_empty_body() {
        // Triangulation: empty body → still the max(3, …) floor.
        assert_eq!(fence_length(""), 3);
    }

    #[test]
    fn fence_length_avoids_backtick_collision() {
        // A 3-backtick run forces the opening fence to be at least 4
        // backticks so CommonMark §4.5 closes the outer fence correctly.
        assert_eq!(fence_length("foo ``` bar"), 4);
    }

    #[test]
    fn fence_length_handles_four_backtick_run() {
        assert_eq!(fence_length("foo ```` bar"), 5);
    }

    #[test]
    fn fence_length_resets_across_newlines() {
        // Two separate 3-runs split by a newline must NOT compose; longest
        // contiguous run is 3, so the fence is 3 + 1 = 4.
        assert_eq!(fence_length("```\n```"), 4);
    }

    #[test]
    fn fence_length_finds_longest_run_anywhere_in_body() {
        // The 5-run lives in the middle of a longer line; the scan must find
        // it regardless of line position. 5 + 1 = 6.
        assert_eq!(fence_length("a\nbbb`````ccc\nd"), 6);
    }

    // ── Task 2: slice_full_file + slice_diff + try_resolve_excerpt ────────

    #[test]
    fn slice_full_file_returns_requested_range() {
        let (_dir, repo) = make_repo();
        let b = commit_with_file(
            &repo,
            "B",
            &[],
            "foo.rs",
            b"fn a() {}\nfn b() {}\nfn c() {}\n",
        );
        let a = anchor(b, "foo.rs", Source::FullFile, Side::New, 2, 3);

        let body = slice_full_file(&repo, &a).expect("resolvable FullFile slice");

        assert_eq!(body, "fn b() {}\nfn c() {}");
    }

    #[test]
    fn slice_full_file_normalizes_crlf() {
        // L-06: CRLF in the blob collapses to LF inside the fence body.
        let (_dir, repo) = make_repo();
        let b = commit_with_file(&repo, "B", &[], "foo.txt", b"a\r\nb\r\nc\r\n");
        let a = anchor(b, "foo.txt", Source::FullFile, Side::New, 1, 3);

        let body = slice_full_file(&repo, &a).expect("resolvable FullFile slice");

        assert_eq!(body, "a\nb\nc");
    }

    #[test]
    fn slice_full_file_returns_binary_for_nul_byte_blob() {
        // L-05: a blob with a NUL byte → blob.is_binary() == true → Binary
        // variant. The placeholder is task 3's concern; here we assert the
        // dispatch ends in Binary, not ResolutionFailed.
        let (_dir, repo) = make_repo();
        let b = commit_with_file(&repo, "B", &[], "bin.dat", b"abc\0def\n");
        let a = anchor(b, "bin.dat", Source::FullFile, Side::New, 1, 1);

        let err = slice_full_file(&repo, &a).expect_err("binary blob must error");

        assert!(
            matches!(err, ExcerptError::Binary),
            "expected ExcerptError::Binary, got {err:?}"
        );
    }

    #[test]
    fn slice_full_file_passes_through_non_utf8_bytes_with_lossy_substitution() {
        // RESEARCH Pitfall 3: Latin-1 bytes (>=0x80) with no NUL pass
        // is_binary() == false; from_utf8_lossy emits U+FFFD substitutions
        // rather than erroring. The line stays sliceable.
        let (_dir, repo) = make_repo();
        // 0xC3 alone (no follow byte) is invalid UTF-8 but has no NUL.
        let b = commit_with_file(&repo, "B", &[], "latin1.txt", b"hello \xC3 world\nsecond\n");
        let a = anchor(b, "latin1.txt", Source::FullFile, Side::New, 1, 1);

        let body = slice_full_file(&repo, &a).expect("lossy UTF-8 still resolves");

        // U+FFFD = "\u{FFFD}" — the lossy substitution char.
        assert!(
            body.contains('\u{FFFD}'),
            "expected lossy substitution char in body, got {body:?}"
        );
        assert!(!body.is_empty());
    }

    #[test]
    fn slice_diff_returns_requested_range() {
        // Parent A has foo.rs = "old\n"; commit B has foo.rs = "new\n".
        // Side::New anchor on line 1 keeps the `+new` line; Phase 67 L-03
        // keeps the opposing-side `-old` line too.
        let (_dir, repo) = make_repo();
        let a = commit_with_file(&repo, "A", &[], "foo.rs", b"old\n");
        let b = commit_with_file(&repo, "B", &[a], "foo.rs", b"new\n");
        let an = anchor(b, "foo.rs", Source::Diff, Side::New, 1, 1);

        let body = slice_diff(&repo, &an).expect("resolvable Diff slice");

        assert!(
            body.contains("+new"),
            "diff body must contain the +new line, got {body:?}"
        );
        assert!(
            body.contains("-old"),
            "Phase 67 L-03: opposing-side `-` line must be kept, got {body:?}"
        );
    }

    #[test]
    fn slice_diff_returns_no_hunks_when_file_unchanged() {
        // Pitfall 2: the file is byte-identical to its parent's version. The
        // pathspec-filtered diff emits zero hunks → NoHunks (not an empty fence).
        let (_dir, repo) = make_repo();
        // Two-file parent so we can keep foo.rs unchanged at the child:
        let a = commit_with_file(&repo, "A", &[], "foo.rs", b"same\n");
        // Child B adds an unrelated file; foo.rs is byte-identical to A's.
        let blob_a = repo.blob(b"same\n").unwrap();
        let blob_other = repo.blob(b"unrelated\n").unwrap();
        let mut builder = repo.treebuilder(None).unwrap();
        builder
            .insert("foo.rs", blob_a, git2::FileMode::Blob.into())
            .unwrap();
        builder
            .insert("other.rs", blob_other, git2::FileMode::Blob.into())
            .unwrap();
        let tree = repo.find_tree(builder.write().unwrap()).unwrap();
        let parent = repo.find_commit(a).unwrap();
        let b = repo
            .commit(None, &sig(), &sig(), "B", &tree, &[&parent])
            .unwrap();
        let an = anchor(b, "foo.rs", Source::Diff, Side::New, 1, 1);

        let err = slice_diff(&repo, &an).expect_err("unchanged file must yield NoHunks");

        assert!(
            matches!(err, ExcerptError::NoHunks),
            "expected ExcerptError::NoHunks, got {err:?}"
        );
    }

    #[test]
    fn slice_diff_handles_root_commit() {
        // Root commit R adds foo.rs from nothing. Diff against None (no
        // parent) per the root-commit guard at commands/diff.rs:410-414.
        let (_dir, repo) = make_repo();
        let r = commit_with_file(&repo, "R (root)", &[], "foo.rs", b"hello\n");
        let an = anchor(r, "foo.rs", Source::Diff, Side::New, 1, 1);

        let body = slice_diff(&repo, &an).expect("root-commit Side::New must resolve");

        assert!(
            body.contains("+hello"),
            "root-commit diff body must contain +hello, got {body:?}"
        );
    }

    #[test]
    fn try_resolve_excerpt_short_circuits_on_missing_commit() {
        // classify_anchor must be the first call: a 40-zero OID is unknown
        // to the repo. The dispatcher returns Orphaned(CommitGone) WITHOUT
        // entering slice_full_file or slice_diff (Pitfall 1).
        let (_dir, repo) = make_repo();
        // Repo has SOMETHING valid so we know it's not a "repo is broken" case.
        let _b = commit_with_file(&repo, "B", &[], "foo.rs", b"hi\n");
        let missing_oid = Oid::from_str(&"0".repeat(40)).unwrap();
        let an = anchor(missing_oid, "foo.rs", Source::FullFile, Side::New, 1, 1);

        let err = try_resolve_excerpt(&repo, &an).expect_err("missing commit must orphan");

        assert!(
            matches!(err, ExcerptError::Orphaned(OrphanReason::CommitGone)),
            "expected Orphaned(CommitGone), got {err:?}"
        );
    }

    // Suppress unused-helper warning while task 3 is still pending.
    #[test]
    fn _empty_commit_helper_is_used() {
        let (_dir, repo) = make_repo();
        let _ = empty_commit(&repo, "R", &[]);
    }
}
