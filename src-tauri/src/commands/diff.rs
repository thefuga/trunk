// Diff commands — Phase 6 implementation

use crate::error::TrunkError;
use crate::git::syntax;
use crate::git::types::{
    CommitDetail, DiffHunk, DiffLine, DiffOrigin, DiffRequestOptions, DiffStatus, FileDiff,
    WordSpan,
};
use crate::state::RepoState;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;

fn open_repo_from_state(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

fn is_head_unborn(repo: &git2::Repository) -> bool {
    match repo.head() {
        Err(e) => e.code() == git2::ErrorCode::UnbornBranch,
        Ok(_) => false,
    }
}

fn apply_request_options(opts: &mut git2::DiffOptions, req: &DiffRequestOptions) {
    let context = if req.show_full_file {
        100_000 // practical cap for full-file view
    } else {
        req.context_lines
    };
    opts.context_lines(context);
    opts.ignore_whitespace_change(req.ignore_whitespace);
}

/// Compute word spans for a paired delete/add line.
/// Returns (delete_spans, add_spans) with byte-offset ranges and emphasis flags.
fn compute_word_spans_for_pair(
    old_content: &str,
    new_content: &str,
) -> (Vec<WordSpan>, Vec<WordSpan>) {
    // Normalize: ensure newline-terminated for from_lines (Pitfall 5)
    let old = if old_content.ends_with('\n') {
        old_content.to_string()
    } else {
        format!("{}\n", old_content)
    };
    let new = if new_content.ends_with('\n') {
        new_content.to_string()
    } else {
        format!("{}\n", new_content)
    };

    let diff = TextDiff::from_lines(&old, &new);
    let mut del_spans = Vec::new();
    let mut add_spans = Vec::new();

    for op in diff.ops() {
        for change in diff.iter_inline_changes(op) {
            let mut offset: u32 = 0;
            let mut spans = Vec::new();
            for (emphasized, value) in change.iter_strings_lossy() {
                let len = value.len() as u32;
                if len > 0 {
                    spans.push(WordSpan {
                        start: offset,
                        end: offset + len,
                        emphasized,
                    });
                }
                offset += len;
            }
            match change.tag() {
                ChangeTag::Delete => del_spans = spans,
                ChangeTag::Insert => add_spans = spans,
                ChangeTag::Equal => {}
            }
        }
    }

    (del_spans, add_spans)
}

/// Compute word spans for all paired Delete/Add lines within a hunk.
/// Returns a Vec parallel to `lines`, each entry being the word_spans for that line index.
/// Pairs consecutive Delete runs with following Add runs positionally (D-03, D-04).
/// Skips lines over 500 chars (WORD-02) and dissimilar pairs with ratio < 0.4 (WORD-02).
fn compute_word_spans_for_hunk(lines: &[DiffLine]) -> Vec<Vec<WordSpan>> {
    let mut word_spans: Vec<Vec<WordSpan>> = vec![Vec::new(); lines.len()];
    let mut i = 0;
    while i < lines.len() {
        // Find start of a Delete run
        if !matches!(lines[i].origin, DiffOrigin::Delete) {
            i += 1;
            continue;
        }

        // Collect consecutive Deletes
        let del_start = i;
        while i < lines.len() && matches!(lines[i].origin, DiffOrigin::Delete) {
            i += 1;
        }
        let del_end = i;

        // Collect consecutive Adds following the Deletes
        let add_start = i;
        while i < lines.len() && matches!(lines[i].origin, DiffOrigin::Add) {
            i += 1;
        }
        let add_end = i;

        // Pair positionally
        let pairs = (del_end - del_start).min(add_end - add_start);
        for p in 0..pairs {
            let del_idx = del_start + p;
            let add_idx = add_start + p;

            let del_content = &lines[del_idx].content;
            let add_content = &lines[add_idx].content;

            // Length threshold (WORD-02): skip lines over 500 chars
            if del_content.len() > 500 || add_content.len() > 500 {
                continue;
            }

            // Edit distance threshold (WORD-02): ratio < 0.4 means >60% different
            let check_diff = TextDiff::from_chars(del_content, add_content);
            if check_diff.ratio() < 0.4 {
                continue;
            }

            let (del_ws, add_ws) = compute_word_spans_for_pair(del_content, add_content);
            word_spans[del_idx] = del_ws;
            word_spans[add_idx] = add_ws;
        }
    }
    word_spans
}

fn walk_diff_into_file_diffs(diff: git2::Diff<'_>) -> Result<Vec<FileDiff>, TrunkError> {
    use std::cell::RefCell;

    let file_diffs: RefCell<Vec<FileDiff>> = RefCell::new(Vec::new());

    diff.foreach(
        &mut |delta, _progress| {
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            let is_binary = delta.old_file().is_binary() || delta.new_file().is_binary();
            let status = match delta.status() {
                git2::Delta::Added => DiffStatus::Added,
                git2::Delta::Deleted => DiffStatus::Deleted,
                git2::Delta::Modified => DiffStatus::Modified,
                git2::Delta::Renamed => DiffStatus::Renamed,
                git2::Delta::Copied => DiffStatus::Copied,
                git2::Delta::Untracked => DiffStatus::Untracked,
                _ => DiffStatus::Unknown,
            };
            file_diffs.borrow_mut().push(FileDiff {
                path,
                status,
                is_binary,
                hunks: Vec::new(),
            });
            true
        },
        None, // skip binary callbacks
        Some(&mut |_delta, hunk| {
            if let Some(fd) = file_diffs.borrow_mut().last_mut() {
                fd.hunks.push(DiffHunk {
                    header: String::from_utf8_lossy(hunk.header()).into_owned(),
                    old_start: hunk.old_start(),
                    old_lines: hunk.old_lines(),
                    new_start: hunk.new_start(),
                    new_lines: hunk.new_lines(),
                    lines: Vec::new(),
                });
            }
            true
        }),
        Some(&mut |_delta, _hunk, line| {
            let origin = match line.origin() {
                '+' => DiffOrigin::Add,
                '-' => DiffOrigin::Delete,
                _ => DiffOrigin::Context,
            };
            let content = String::from_utf8_lossy(line.content()).into_owned();
            let mut diffs = file_diffs.borrow_mut();
            if let Some(fd) = diffs.last_mut() {
                if let Some(hunk) = fd.hunks.last_mut() {
                    hunk.lines.push(DiffLine {
                        origin,
                        content,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                        spans: vec![],
                    });
                }
            }
            true
        }),
    )
    .map_err(TrunkError::from)?;

    let mut file_diffs = file_diffs.into_inner();

    // Pass 2: word-level diff enrichment
    // Pass 3: syntax highlighting + merge into unified spans
    for fd in &mut file_diffs {
        let ext = syntax::extension_from_path(&fd.path);
        for hunk in &mut fd.hunks {
            let word_spans_per_line = compute_word_spans_for_hunk(&hunk.lines);
            for (i, line) in hunk.lines.iter_mut().enumerate() {
                let syntax_tokens = syntax::highlight_line_tokens(&line.content, ext);
                let ws = &word_spans_per_line[i];
                line.spans = syntax::merge_spans(&syntax_tokens, ws, line.content.len() as u32);
            }
        }
    }

    Ok(file_diffs)
}

pub fn diff_unstaged_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
    options: &DiffRequestOptions,
) -> Result<Vec<FileDiff>, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file_path);
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);
    opts.show_untracked_content(true);
    apply_request_options(&mut opts, options);
    let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;
    walk_diff_into_file_diffs(diff)
}

pub fn diff_staged_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
    options: &DiffRequestOptions,
) -> Result<Vec<FileDiff>, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file_path);
    apply_request_options(&mut opts, options);
    let diff = if is_head_unborn(&repo) {
        repo.diff_tree_to_index(None, None, Some(&mut opts))?
    } else {
        let head_tree = repo.head()?.peel_to_tree()?;
        repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut opts))?
    };
    walk_diff_into_file_diffs(diff)
}

pub fn diff_commit_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
    options: &DiffRequestOptions,
) -> Result<Vec<FileDiff>, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let oid =
        git2::Oid::from_str(oid).map_err(|e| TrunkError::new("invalid_oid", e.to_string()))?;
    let commit = repo.find_commit(oid)?;
    let commit_tree = commit.tree()?;
    let mut opts = git2::DiffOptions::new();
    apply_request_options(&mut opts, options);
    let diff = if commit.parent_count() == 0 {
        repo.diff_tree_to_tree(None, Some(&commit_tree), Some(&mut opts))?
    } else {
        let parent_tree = commit.parent(0)?.tree()?;
        repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), Some(&mut opts))?
    };
    walk_diff_into_file_diffs(diff)
}

pub fn get_commit_detail_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<CommitDetail, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let oid =
        git2::Oid::from_str(oid).map_err(|e| TrunkError::new("invalid_oid", e.to_string()))?;
    let commit = repo.find_commit(oid)?;
    let author = commit.author();
    let committer = commit.committer();
    Ok(CommitDetail {
        oid: commit.id().to_string(),
        short_oid: commit.id().to_string()[..7].to_owned(),
        summary: commit.summary().unwrap_or("").to_owned(),
        body: commit.body().map(str::to_owned),
        author_name: author.name().unwrap_or("").to_owned(),
        author_email: author.email().unwrap_or("").to_owned(),
        author_timestamp: author.when().seconds(),
        committer_name: committer.name().unwrap_or("").to_owned(),
        committer_email: committer.email().unwrap_or("").to_owned(),
        committer_timestamp: committer.when().seconds(),
        parent_oids: commit.parent_ids().map(|id| id.to_string()).collect(),
    })
}

#[tauri::command]
pub async fn diff_unstaged(
    path: String,
    file_path: String,
    options: DiffRequestOptions,
    state: State<'_, RepoState>,
) -> Result<Vec<FileDiff>, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        diff_unstaged_inner(&path, &file_path, &state_map, &options)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn diff_staged(
    path: String,
    file_path: String,
    options: DiffRequestOptions,
    state: State<'_, RepoState>,
) -> Result<Vec<FileDiff>, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        diff_staged_inner(&path, &file_path, &state_map, &options)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn diff_commit(
    path: String,
    oid: String,
    options: DiffRequestOptions,
    state: State<'_, RepoState>,
) -> Result<Vec<FileDiff>, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        diff_commit_inner(&path, &oid, &state_map, &options)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn get_commit_detail(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
) -> Result<CommitDetail, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || get_commit_detail_inner(&path, &oid, &state_map))
        .await
        .map_err(|e| {
            serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
        })?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}
