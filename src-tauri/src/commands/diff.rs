// Diff commands — Phase 6 implementation

use crate::error::TrunkError;
use crate::git::types::{
    CommitDetail, DiffHunk, DiffLine, DiffOrigin, DiffRequestOptions, DiffStatus, FileDiff,
};
use crate::state::RepoState;
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
                        word_spans: vec![],
                        syntax_tokens: vec![],
                    });
                }
            }
            true
        }),
    )
    .map_err(TrunkError::from)?;

    Ok(file_diffs.into_inner())
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
    .map_err(|e| {
        serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
    })?
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
    .map_err(|e| {
        serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
    })?
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
    .map_err(|e| {
        serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
    })?
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
