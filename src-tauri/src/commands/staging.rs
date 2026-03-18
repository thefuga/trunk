use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use git2::{Status, StatusOptions};
use tauri::State;
use crate::error::TrunkError;
use crate::git::types::{FileStatus, FileStatusType, WorkingTreeStatus};
use crate::state::RepoState;

fn open_repo_from_state(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

fn classify_index(s: Status) -> Option<FileStatusType> {
    if s.contains(Status::INDEX_NEW)        { return Some(FileStatusType::New); }
    if s.contains(Status::INDEX_MODIFIED)   { return Some(FileStatusType::Modified); }
    if s.contains(Status::INDEX_DELETED)    { return Some(FileStatusType::Deleted); }
    if s.contains(Status::INDEX_RENAMED)    { return Some(FileStatusType::Renamed); }
    if s.contains(Status::INDEX_TYPECHANGE) { return Some(FileStatusType::Typechange); }
    if s.contains(Status::CONFLICTED)       { return Some(FileStatusType::Conflicted); }
    None
}

fn classify_workdir(s: Status) -> Option<FileStatusType> {
    if s.contains(Status::WT_NEW)        { return Some(FileStatusType::New); }
    if s.contains(Status::WT_MODIFIED)   { return Some(FileStatusType::Modified); }
    if s.contains(Status::WT_DELETED)    { return Some(FileStatusType::Deleted); }
    if s.contains(Status::WT_RENAMED)    { return Some(FileStatusType::Renamed); }
    if s.contains(Status::WT_TYPECHANGE) { return Some(FileStatusType::Typechange); }
    None
}

pub fn get_status_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<WorkingTreeStatus, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    let mut unstaged: Vec<FileStatus> = Vec::new();
    let mut staged: Vec<FileStatus> = Vec::new();
    let mut conflicted: Vec<FileStatus> = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();
        let file_path = entry.path().unwrap_or("").to_owned();

        // Check for conflicts first
        if status.contains(Status::CONFLICTED) {
            conflicted.push(FileStatus {
                path: file_path.clone(),
                status: FileStatusType::Conflicted,
                is_binary: false,
            });
            continue;
        }

        // Index (staged) entries
        if let Some(status_type) = classify_index(status) {
            staged.push(FileStatus {
                path: file_path.clone(),
                status: status_type,
                is_binary: false,
            });
        }

        // Working directory (unstaged) entries — a file can appear in both
        if let Some(status_type) = classify_workdir(status) {
            unstaged.push(FileStatus {
                path: file_path,
                status: status_type,
                is_binary: false,
            });
        }
    }

    Ok(WorkingTreeStatus { unstaged, staged, conflicted })
}

pub fn stage_file_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let mut index = repo.index()?;
    index.add_path(Path::new(file_path))?;
    index.write()?;
    Ok(())
}

fn is_head_unborn(repo: &git2::Repository) -> bool {
    match repo.head() {
        Err(e) => e.code() == git2::ErrorCode::UnbornBranch,
        Ok(_) => false,
    }
}

pub fn unstage_file_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    if is_head_unborn(&repo) {
        // No commits yet — just remove from index
        let mut index = repo.index()?;
        index.remove_path(Path::new(file_path))?;
        index.write()?;
    } else {
        // Reset the file to HEAD state using reset_default
        let head_commit = repo.head()?.peel_to_commit()?;
        repo.reset_default(
            Some(head_commit.as_object()),
            std::iter::once(file_path),
        )?;
    }

    Ok(())
}

pub fn discard_file_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    let mut opts = StatusOptions::new();
    opts.pathspec(file_path)
        .include_untracked(true)
        .include_ignored(false);

    let statuses = repo.statuses(Some(&mut opts))?;

    if statuses.is_empty() {
        return Err(TrunkError::new(
            "file_not_found",
            format!("File not in working tree changes: {}", file_path),
        ));
    }

    let status = statuses.get(0).unwrap().status();

    if status.contains(Status::WT_NEW) {
        // Untracked file — delete from disk
        let full_path = repo.workdir().unwrap().join(file_path);
        std::fs::remove_file(&full_path).map_err(|e| {
            TrunkError::new("io_error", format!("Failed to delete {}: {}", file_path, e))
        })?;
    } else if status.intersects(
        Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED | Status::WT_TYPECHANGE,
    ) {
        // Tracked file with working tree changes — checkout from HEAD
        let mut checkout = git2::build::CheckoutBuilder::new();
        checkout.path(file_path).force();
        repo.checkout_head(Some(&mut checkout))?;
    } else {
        return Err(TrunkError::new(
            "file_not_found",
            format!("File not in working tree changes: {}", file_path),
        ));
    }

    Ok(())
}

pub fn discard_all_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    // Collect untracked file paths before checkout
    let untracked_paths: Vec<PathBuf> = statuses
        .iter()
        .filter(|entry| entry.status().contains(Status::WT_NEW))
        .filter_map(|entry| entry.path().map(|p| repo.workdir().unwrap().join(p)))
        .collect();

    // Force checkout HEAD to restore all tracked modifications
    let mut checkout = git2::build::CheckoutBuilder::new();
    checkout.force();
    repo.checkout_head(Some(&mut checkout))?;

    // Delete untracked files
    for file_path in &untracked_paths {
        let _ = std::fs::remove_file(file_path);
        // Try to remove empty parent directories
        if let Some(parent) = file_path.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }

    Ok(())
}

pub fn stage_all_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    Ok(())
}

pub fn stage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    // Generate diff for this file (index -> workdir)
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(file_path);
    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;

    // Validate: at least one delta expected
    if diff.deltas().len() == 0 {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No unstaged changes for: {}", file_path),
        ));
    }

    // Count hunks via Patch to validate hunk_index
    let patch = git2::Patch::from_diff(&diff, 0)?
        .ok_or_else(|| TrunkError::new("file_not_found", "Binary or unchanged file"))?;
    let num_hunks = patch.num_hunks();
    if (hunk_index as usize) >= num_hunks {
        return Err(TrunkError::new(
            "stale_hunk_index",
            format!("Hunk index {} out of range (file has {} hunks)", hunk_index, num_hunks),
        ));
    }
    drop(patch); // Release borrow on diff

    // Apply only the target hunk to the index
    let target = hunk_index as usize;
    let mut current: usize = 0;
    let mut apply_opts = git2::ApplyOptions::new();
    apply_opts.hunk_callback(move |_hunk| {
        let apply = current == target;
        current += 1;
        apply
    });

    repo.apply(&diff, git2::ApplyLocation::Index, Some(&mut apply_opts))
        .map_err(|e| TrunkError::new("hunk_apply_failed", e.message().to_owned()))?;

    Ok(())
}

pub fn unstage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    // Generate reversed diff (index -> HEAD) so applying it to index undoes the staged change
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(file_path).reverse(true);

    let diff = if is_head_unborn(&repo) {
        repo.diff_tree_to_index(None, None, Some(&mut diff_opts))?
    } else {
        let head_tree = repo.head()?.peel_to_tree()?;
        repo.diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))?
    };

    // Validate delta exists
    if diff.deltas().len() == 0 {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No staged changes for: {}", file_path),
        ));
    }

    // Validate hunk_index
    let patch = git2::Patch::from_diff(&diff, 0)?
        .ok_or_else(|| TrunkError::new("file_not_found", "Binary or unchanged file"))?;
    let num_hunks = patch.num_hunks();
    if (hunk_index as usize) >= num_hunks {
        return Err(TrunkError::new(
            "stale_hunk_index",
            format!("Hunk index {} out of range (file has {} hunks)", hunk_index, num_hunks),
        ));
    }
    drop(patch);

    // Apply reversed hunk to index
    let target = hunk_index as usize;
    let mut current: usize = 0;
    let mut apply_opts = git2::ApplyOptions::new();
    apply_opts.hunk_callback(move |_hunk| {
        let apply = current == target;
        current += 1;
        apply
    });

    repo.apply(&diff, git2::ApplyLocation::Index, Some(&mut apply_opts))
        .map_err(|e| TrunkError::new("hunk_apply_failed", e.message().to_owned()))?;

    Ok(())
}

pub fn discard_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    // Generate reversed diff (workdir -> index) so applying to workdir undoes the change
    let mut diff_opts = git2::DiffOptions::new();
    diff_opts.pathspec(file_path).reverse(true);
    let diff = repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;

    if diff.deltas().len() == 0 {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No unstaged changes for: {}", file_path),
        ));
    }

    // Validate hunk_index
    let patch = git2::Patch::from_diff(&diff, 0)?
        .ok_or_else(|| TrunkError::new("file_not_found", "Binary or unchanged file"))?;
    let num_hunks = patch.num_hunks();
    if (hunk_index as usize) >= num_hunks {
        return Err(TrunkError::new(
            "stale_hunk_index",
            format!("Hunk index {} out of range (file has {} hunks)", hunk_index, num_hunks),
        ));
    }
    drop(patch);

    // Apply reversed hunk to workdir
    let target = hunk_index as usize;
    let mut current: usize = 0;
    let mut apply_opts = git2::ApplyOptions::new();
    apply_opts.hunk_callback(move |_hunk| {
        let apply = current == target;
        current += 1;
        apply
    });

    repo.apply(&diff, git2::ApplyLocation::WorkDir, Some(&mut apply_opts))
        .map_err(|e| TrunkError::new("hunk_apply_failed", e.message().to_owned()))?;

    Ok(())
}

pub fn unstage_all_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;

    if is_head_unborn(&repo) {
        let mut index = repo.index()?;
        index.clear()?;
        index.write()?;
    } else {
        let head_commit = repo.head()?.peel_to_commit()?;
        // Collect all staged paths first
        let staged_paths: Vec<String> = get_status_inner(path, state_map)?
            .staged
            .into_iter()
            .map(|f| f.path)
            .collect();
        if !staged_paths.is_empty() {
            repo.reset_default(
                Some(head_commit.as_object()),
                staged_paths.iter().map(String::as_str),
            )?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DirtyCounts {
    pub staged: usize,
    pub unstaged: usize,
    pub conflicted: usize,
}

fn get_dirty_counts_inner(
    path: &str,
    state_map: &std::collections::HashMap<String, std::path::PathBuf>,
) -> Result<DirtyCounts, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .recurse_untracked_dirs(true);
    let statuses = repo.statuses(Some(&mut opts)).map_err(TrunkError::from)?;
    let mut staged = 0usize;
    let mut unstaged = 0usize;
    let mut conflicted = 0usize;
    for entry in statuses.iter() {
        let s = entry.status();
        if s.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }
        if s.intersects(
            Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE,
        ) {
            unstaged += 1;
        }
        if s.intersects(Status::CONFLICTED) {
            conflicted += 1;
        }
    }
    Ok(DirtyCounts { staged, unstaged, conflicted })
}

#[tauri::command]
pub async fn discard_file(
    path: String,
    file_path: String,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || discard_file_inner(&path, &file_path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn discard_all(
    path: String,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || discard_all_inner(&path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn get_dirty_counts(
    path: String,
    state: State<'_, RepoState>,
) -> Result<DirtyCounts, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || get_dirty_counts_inner(&path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn get_status(
    path: String,
    state: State<'_, RepoState>,
) -> Result<WorkingTreeStatus, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || get_status_inner(&path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn stage_file(
    path: String,
    file_path: String,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || stage_file_inner(&path, &file_path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn unstage_file(
    path: String,
    file_path: String,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || unstage_file_inner(&path, &file_path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn stage_all(
    path: String,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || stage_all_inner(&path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn unstage_all(
    path: String,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || unstage_all_inner(&path, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn stage_hunk(
    path: String,
    file_path: String,
    hunk_index: u32,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || stage_hunk_inner(&path, &file_path, hunk_index, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn unstage_hunk(
    path: String,
    file_path: String,
    hunk_index: u32,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || unstage_hunk_inner(&path, &file_path, hunk_index, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn discard_hunk(
    path: String,
    file_path: String,
    hunk_index: u32,
    state: State<'_, RepoState>,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || discard_hunk_inner(&path, &file_path, hunk_index, &state_map))
        .await
        .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
        .map_err(|e| serde_json::to_string(&e).unwrap())
}

fn build_partial_patch_text(
    file_path: &str,
    patch: &git2::Patch<'_>,
    hunk_idx: usize,
    selected_indices: &[u32],
) -> Result<String, TrunkError> {
    todo!()
}

pub fn stage_lines_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    todo!()
}

pub fn unstage_lines_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    todo!()
}

pub fn discard_lines_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use crate::commands::diff::{diff_unstaged_inner, diff_staged_inner};
    use crate::git::repository::tests::make_test_repo;
    use crate::git::types::FileStatusType;

    fn make_state_map(path: &Path) -> std::collections::HashMap<String, std::path::PathBuf> {
        let mut map = std::collections::HashMap::new();
        map.insert(path.to_string_lossy().to_string(), path.to_path_buf());
        map
    }

    // Test 1 — get_status_returns_unstaged
    #[test]
    fn get_status_returns_unstaged() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Modify existing tracked file without staging
        std::fs::write(dir.path().join("README.md"), "modified content").unwrap();

        let status = super::get_status_inner(&path, &state_map).expect("get_status_inner failed");

        assert!(!status.unstaged.is_empty(), "expected unstaged to be non-empty");
        assert!(status.staged.is_empty(), "expected staged to be empty");
    }

    // Test 2 — status_new_file
    #[test]
    fn status_new_file() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Write a brand new file (not previously tracked)
        std::fs::write(dir.path().join("brand_new.txt"), "new content").unwrap();

        let status = super::get_status_inner(&path, &state_map).expect("get_status_inner failed");

        let has_new = status.unstaged.iter().any(|f| matches!(f.status, FileStatusType::New));
        assert!(has_new, "expected at least one entry with status New in unstaged");
    }

    // Test 3 — status_modified_file
    #[test]
    fn status_modified_file() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Overwrite README.md (tracked file) without staging
        std::fs::write(dir.path().join("README.md"), "modified hello").unwrap();

        let status = super::get_status_inner(&path, &state_map).expect("get_status_inner failed");

        let has_modified = status.unstaged.iter().any(|f| matches!(f.status, FileStatusType::Modified));
        assert!(has_modified, "expected at least one entry with status Modified in unstaged");
    }

    // Test 4 — stage_file_moves_to_staged
    #[test]
    fn stage_file_moves_to_staged() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Modify README.md without staging
        std::fs::write(dir.path().join("README.md"), "staged content").unwrap();

        super::stage_file_inner(&path, "README.md", &state_map).expect("stage_file_inner failed");

        let status = super::get_status_inner(&path, &state_map).expect("get_status_inner failed");

        assert!(!status.staged.is_empty(), "expected staged to be non-empty after staging");
        let has_readme = status.staged.iter().any(|f| f.path == "README.md");
        assert!(has_readme, "expected README.md in staged list");
    }

    // Test 5 — unstage_file_moves_to_unstaged
    #[test]
    fn unstage_file_moves_to_unstaged() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Modify and stage README.md
        std::fs::write(dir.path().join("README.md"), "to be staged then unstaged").unwrap();
        let repo = git2::Repository::open(dir.path()).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        index.write().unwrap();
        drop(index);
        drop(repo);

        super::unstage_file_inner(&path, "README.md", &state_map).expect("unstage_file_inner failed");

        let status = super::get_status_inner(&path, &state_map).expect("get_status_inner failed");

        let readme_in_staged = status.staged.iter().any(|f| f.path == "README.md");
        assert!(!readme_in_staged, "expected README.md NOT in staged list after unstaging");
    }

    // Test 6 — unstage_on_unborn_head
    #[test]
    fn unstage_on_unborn_head() {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let repo = git2::Repository::init(dir.path()).expect("failed to init repo");
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Create a new file and stage it (no commits yet)
        std::fs::write(dir.path().join("new_file.txt"), "content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("new_file.txt")).unwrap();
        index.write().unwrap();
        drop(index);
        drop(repo);

        let result = super::unstage_file_inner(&path, "new_file.txt", &state_map);
        assert!(result.is_ok(), "expected Ok(()) for unstage on unborn HEAD, got: {:?}", result);
    }

    // Test 7 — stage_all_stages_everything
    #[test]
    fn stage_all_stages_everything() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Modify existing tracked file + write a new untracked file
        std::fs::write(dir.path().join("README.md"), "modified for stage all").unwrap();
        std::fs::write(dir.path().join("new_for_all.txt"), "new content").unwrap();

        super::stage_all_inner(&path, &state_map).expect("stage_all_inner failed");

        let status = super::get_status_inner(&path, &state_map).expect("get_status_inner failed");

        assert!(
            status.staged.len() >= 2,
            "expected at least 2 entries in staged after stage_all, got {}",
            status.staged.len()
        );
        assert!(status.unstaged.is_empty(), "expected unstaged to be empty after stage_all");
    }

    // Test 8 — unstage_all_clears_index
    #[test]
    fn unstage_all_clears_index() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Modify README.md and stage it manually
        std::fs::write(dir.path().join("README.md"), "staged for unstage_all test").unwrap();
        let repo = git2::Repository::open(dir.path()).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        index.write().unwrap();
        drop(index);
        drop(repo);

        super::unstage_all_inner(&path, &state_map).expect("unstage_all_inner failed");

        let status = super::get_status_inner(&path, &state_map).expect("get_status_inner failed");

        assert!(status.staged.is_empty(), "expected staged to be empty after unstage_all");
    }

    // Test 9 — discard_file_reverts_tracked
    #[test]
    fn discard_file_reverts_tracked() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Remember original content
        let original = std::fs::read_to_string(dir.path().join("README.md")).unwrap();

        // Modify README.md (tracked file)
        std::fs::write(dir.path().join("README.md"), "modified content for discard test").unwrap();

        // Discard the file
        super::discard_file_inner(&path, "README.md", &state_map).expect("discard_file_inner failed");

        // Verify content reverted to original
        let after = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
        assert_eq!(after, original, "expected README.md to revert to original content after discard");
    }

    // Test 10 — discard_file_deletes_untracked
    #[test]
    fn discard_file_deletes_untracked() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Create a brand new untracked file
        std::fs::write(dir.path().join("brand_new.txt"), "untracked content").unwrap();

        // Discard the file
        super::discard_file_inner(&path, "brand_new.txt", &state_map).expect("discard_file_inner failed");

        // Verify file no longer exists
        assert!(!dir.path().join("brand_new.txt").exists(), "expected brand_new.txt to be deleted after discard");
    }

    // Test 11 — discard_all_reverts_everything
    #[test]
    fn discard_all_reverts_everything() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Remember original content
        let original = std::fs::read_to_string(dir.path().join("README.md")).unwrap();

        // Modify tracked file + create untracked file
        std::fs::write(dir.path().join("README.md"), "modified for discard_all").unwrap();
        std::fs::write(dir.path().join("brand_new.txt"), "untracked for discard_all").unwrap();

        // Discard all
        super::discard_all_inner(&path, &state_map).expect("discard_all_inner failed");

        // Verify tracked file reverted
        let after = std::fs::read_to_string(dir.path().join("README.md")).unwrap();
        assert_eq!(after, original, "expected README.md to revert after discard_all");

        // Verify untracked file deleted
        assert!(!dir.path().join("brand_new.txt").exists(), "expected brand_new.txt deleted after discard_all");
    }

    // Test 12 — get_dirty_counts_includes_untracked
    #[test]
    fn get_dirty_counts_includes_untracked() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Create a brand new file (never tracked)
        std::fs::write(dir.path().join("untracked_new.txt"), "brand new").unwrap();

        let counts = super::get_dirty_counts_inner(&path, &state_map).expect("get_dirty_counts_inner failed");

        assert!(
            counts.unstaged >= 1,
            "expected unstaged >= 1 for untracked file, got {}",
            counts.unstaged
        );
    }

    // --- Multi-hunk test fixture helper ---

    fn create_multi_hunk_file(dir: &std::path::Path) {
        // Original content: 30 lines to ensure context separation between hunks
        let original = (1..=30)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        std::fs::write(dir.join("multi.txt"), &original).unwrap();

        // Stage and commit the original
        let repo = git2::Repository::open(dir).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("multi.txt")).unwrap();
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let sig = repo.signature().unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Add multi.txt", &tree, &[&head]).unwrap();
        drop(index);
        drop(tree);
        drop(head);
        drop(repo);

        // Modify lines near the top AND near the bottom (creates 2 hunks)
        let mut lines: Vec<String> = original.split('\n').map(|s| s.to_string()).collect();
        lines[1] = "MODIFIED line 2".to_string();   // Near top -> hunk 0
        lines[28] = "MODIFIED line 29".to_string();  // Near bottom -> hunk 1
        std::fs::write(dir.join("multi.txt"), lines.join("\n")).unwrap();
    }

    // Test 13 — stage_hunk_stages_single_hunk
    #[test]
    fn stage_hunk_stages_single_hunk() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_multi_hunk_file(dir.path());

        // Stage only hunk 0
        super::stage_hunk_inner(&path, "multi.txt", 0, &state_map)
            .expect("stage_hunk_inner failed");

        // Verify: staged diff should have 1 hunk (hunk 0 was staged)
        let staged = diff_staged_inner(&path, "multi.txt", &state_map)
            .expect("diff_staged_inner failed");
        assert_eq!(staged.len(), 1, "expected 1 file in staged diff");
        assert_eq!(staged[0].hunks.len(), 1, "expected 1 hunk in staged diff");

        // Verify: unstaged diff should have 1 hunk (hunk 1 remains)
        let unstaged = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        assert_eq!(unstaged.len(), 1, "expected 1 file in unstaged diff");
        assert_eq!(unstaged[0].hunks.len(), 1, "expected 1 hunk remaining in unstaged diff");
    }

    // Test 14 — stage_hunk_stale_index
    #[test]
    fn stage_hunk_stale_index() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_multi_hunk_file(dir.path());

        // Try to stage hunk 5, which is out of range for a 2-hunk file
        let result = super::stage_hunk_inner(&path, "multi.txt", 5, &state_map);
        assert!(result.is_err(), "expected Err for out-of-range hunk index");
        let err = result.unwrap_err();
        assert_eq!(err.code, "stale_hunk_index", "expected stale_hunk_index error code");
    }

    // Test 15 — stage_hunk_file_not_found
    #[test]
    fn stage_hunk_file_not_found() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Do NOT create any changes — clean working tree
        let result = super::stage_hunk_inner(&path, "README.md", 0, &state_map);
        assert!(result.is_err(), "expected Err for file with no unstaged changes");
        let err = result.unwrap_err();
        assert_eq!(err.code, "file_not_found", "expected file_not_found error code");
    }

    // Test 16 — unstage_hunk_unstages_single_hunk
    #[test]
    fn unstage_hunk_unstages_single_hunk() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_multi_hunk_file(dir.path());

        // Stage the entire file first
        super::stage_file_inner(&path, "multi.txt", &state_map)
            .expect("stage_file_inner failed");

        // Unstage hunk 0 only
        super::unstage_hunk_inner(&path, "multi.txt", 0, &state_map)
            .expect("unstage_hunk_inner failed");

        // Verify: staged diff should have 1 hunk remaining (hunk 1 stays staged)
        let staged = diff_staged_inner(&path, "multi.txt", &state_map)
            .expect("diff_staged_inner failed");
        assert_eq!(staged.len(), 1, "expected 1 file in staged diff");
        assert_eq!(staged[0].hunks.len(), 1, "expected 1 hunk remaining in staged diff");

        // Verify: unstaged diff should have 1 hunk (hunk 0 is back in unstaged)
        let unstaged = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        assert_eq!(unstaged.len(), 1, "expected 1 file in unstaged diff");
        assert_eq!(unstaged[0].hunks.len(), 1, "expected 1 hunk in unstaged diff");
    }

    // Test 17 — discard_hunk_discards_single_hunk
    #[test]
    fn discard_hunk_discards_single_hunk() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_multi_hunk_file(dir.path());

        // Discard hunk 0 only
        super::discard_hunk_inner(&path, "multi.txt", 0, &state_map)
            .expect("discard_hunk_inner failed");

        // Verify: unstaged diff should have 1 hunk remaining (hunk 1)
        let unstaged = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        assert_eq!(unstaged.len(), 1, "expected 1 file in unstaged diff");
        assert_eq!(unstaged[0].hunks.len(), 1, "expected 1 hunk remaining after discard");
    }

    // Test 18 — discard_hunk_file_not_found
    #[test]
    fn discard_hunk_file_not_found() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        // Do NOT create any changes — clean working tree
        let result = super::discard_hunk_inner(&path, "README.md", 0, &state_map);
        assert!(result.is_err(), "expected Err for file with no unstaged changes");
        let err = result.unwrap_err();
        assert_eq!(err.code, "file_not_found", "expected file_not_found error code");
    }

    // --- Line-level staging test fixture helper ---

    /// Creates a file where hunk 0 has both add and delete lines.
    /// Original: 30 lines ("line 1" through "line 30").
    /// Modified: line 2 is replaced ("line 2" -> "MODIFIED line 2", "INSERTED line 2.5"),
    ///           line 29 is replaced ("line 29" -> "MODIFIED line 29").
    /// This produces hunk 0 with: one '-' (delete old line 2), one '+' (add MODIFIED line 2),
    /// one '+' (add INSERTED line 2.5), and hunk 1 with changes at line 29.
    fn create_add_delete_hunk_file(dir: &std::path::Path) {
        // Original content: 30 lines to ensure context separation between hunks
        let original = (1..=30)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        std::fs::write(dir.join("multi.txt"), &original).unwrap();

        // Stage and commit the original
        let repo = git2::Repository::open(dir).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("multi.txt")).unwrap();
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let sig = repo.signature().unwrap();
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Add multi.txt", &tree, &[&head]).unwrap();
        drop(index);
        drop(tree);
        drop(head);
        drop(repo);

        // Modify: replace line 2 and insert a new line, plus modify line 29
        let mut lines: Vec<String> = original.lines().map(|s| s.to_string()).collect();
        // Replace line 2 (index 1) with modified version
        lines[1] = "MODIFIED line 2".to_string();
        // Insert a new line after position 1
        lines.insert(2, "INSERTED line 2.5".to_string());
        // Modify line 29 (now at index 29 due to insertion)
        lines[29] = "MODIFIED line 29".to_string();
        let modified = lines.join("\n") + "\n";
        std::fs::write(dir.join("multi.txt"), &modified).unwrap();
    }

    // Test 19 — stage_lines_stages_selected_adds
    #[test]
    fn stage_lines_stages_selected_adds() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_add_delete_hunk_file(dir.path());

        // Get the unstaged diff to find exact line indices in hunk 0
        let unstaged = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        assert!(!unstaged.is_empty(), "expected unstaged diff");
        let hunk0 = &unstaged[0].hunks[0];

        // Find indices of add lines ('+') in hunk 0
        let add_indices: Vec<u32> = hunk0.lines.iter().enumerate()
            .filter(|(_, l)| matches!(l.origin, crate::git::types::DiffOrigin::Add))
            .map(|(i, _)| i as u32)
            .collect();
        assert!(!add_indices.is_empty(), "expected at least one add line in hunk 0");

        // Stage only the add lines from hunk 0
        super::stage_lines_inner(&path, "multi.txt", 0, add_indices.clone(), &state_map)
            .expect("stage_lines_inner failed");

        // Verify: staged diff should have the add lines
        let staged = diff_staged_inner(&path, "multi.txt", &state_map)
            .expect("diff_staged_inner failed");
        assert!(!staged.is_empty(), "expected staged diff after staging add lines");
        let staged_hunk0 = &staged[0].hunks[0];
        let staged_adds: Vec<&crate::git::types::DiffLine> = staged_hunk0.lines.iter()
            .filter(|l| matches!(l.origin, crate::git::types::DiffOrigin::Add))
            .collect();
        assert!(!staged_adds.is_empty(), "expected add lines in staged diff");

        // The delete line should NOT be in the staged diff (it was not selected)
        let staged_deletes: Vec<&crate::git::types::DiffLine> = staged_hunk0.lines.iter()
            .filter(|l| matches!(l.origin, crate::git::types::DiffOrigin::Delete))
            .collect();
        assert!(staged_deletes.is_empty(), "expected no delete lines in staged diff when only adds were staged");
    }

    // Test 20 — stage_lines_stages_selected_deletes
    #[test]
    fn stage_lines_stages_selected_deletes() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_add_delete_hunk_file(dir.path());

        // Get the unstaged diff to find line indices in hunk 0
        let unstaged = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        let hunk0 = &unstaged[0].hunks[0];

        // Find indices of delete lines ('-') in hunk 0
        let del_indices: Vec<u32> = hunk0.lines.iter().enumerate()
            .filter(|(_, l)| matches!(l.origin, crate::git::types::DiffOrigin::Delete))
            .map(|(i, _)| i as u32)
            .collect();
        assert!(!del_indices.is_empty(), "expected at least one delete line in hunk 0");

        // Stage only the delete lines from hunk 0
        super::stage_lines_inner(&path, "multi.txt", 0, del_indices, &state_map)
            .expect("stage_lines_inner failed");

        // Verify: staged diff should have the delete lines
        let staged = diff_staged_inner(&path, "multi.txt", &state_map)
            .expect("diff_staged_inner failed");
        assert!(!staged.is_empty(), "expected staged diff after staging delete lines");
        let staged_hunk0 = &staged[0].hunks[0];
        let staged_deletes: Vec<&crate::git::types::DiffLine> = staged_hunk0.lines.iter()
            .filter(|l| matches!(l.origin, crate::git::types::DiffOrigin::Delete))
            .collect();
        assert!(!staged_deletes.is_empty(), "expected delete lines in staged diff");

        // Unselected add lines should NOT be in the staged diff
        let staged_adds: Vec<&crate::git::types::DiffLine> = staged_hunk0.lines.iter()
            .filter(|l| matches!(l.origin, crate::git::types::DiffOrigin::Add))
            .collect();
        assert!(staged_adds.is_empty(), "expected no add lines in staged diff when only deletes were staged");
    }

    // Test 21 — stage_lines_mixed_selection
    #[test]
    fn stage_lines_mixed_selection() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_add_delete_hunk_file(dir.path());

        let unstaged = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        let hunk0 = &unstaged[0].hunks[0];

        // Select ALL add and delete lines from hunk 0 (mixed)
        let mixed_indices: Vec<u32> = hunk0.lines.iter().enumerate()
            .filter(|(_, l)| matches!(l.origin, crate::git::types::DiffOrigin::Add | crate::git::types::DiffOrigin::Delete))
            .map(|(i, _)| i as u32)
            .collect();
        assert!(mixed_indices.len() >= 2, "expected at least 2 add/delete lines for mixed selection");

        super::stage_lines_inner(&path, "multi.txt", 0, mixed_indices, &state_map)
            .expect("stage_lines_inner failed");

        // Verify: staged diff should have both add and delete lines
        let staged = diff_staged_inner(&path, "multi.txt", &state_map)
            .expect("diff_staged_inner failed");
        assert!(!staged.is_empty(), "expected staged diff");
        let staged_hunk0 = &staged[0].hunks[0];
        let has_adds = staged_hunk0.lines.iter().any(|l| matches!(l.origin, crate::git::types::DiffOrigin::Add));
        let has_dels = staged_hunk0.lines.iter().any(|l| matches!(l.origin, crate::git::types::DiffOrigin::Delete));
        assert!(has_adds, "expected add lines in staged diff for mixed selection");
        assert!(has_dels, "expected delete lines in staged diff for mixed selection");
    }

    // Test 22 — stage_lines_stale_index
    #[test]
    fn stage_lines_stale_index() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_add_delete_hunk_file(dir.path());

        // Try to stage lines from a non-existent hunk index
        let result = super::stage_lines_inner(&path, "multi.txt", 99, vec![0], &state_map);
        assert!(result.is_err(), "expected Err for out-of-range hunk index");
        let err = result.unwrap_err();
        assert_eq!(err.code, "stale_hunk_index", "expected stale_hunk_index error code");
    }

    // Test 23 — unstage_lines_unstages_selected
    #[test]
    fn unstage_lines_unstages_selected() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_add_delete_hunk_file(dir.path());

        // Stage entire file first
        super::stage_file_inner(&path, "multi.txt", &state_map)
            .expect("stage_file_inner failed");

        // Get the staged diff to find line indices
        let staged = diff_staged_inner(&path, "multi.txt", &state_map)
            .expect("diff_staged_inner failed");
        assert!(!staged.is_empty(), "expected staged diff");
        let hunk0 = &staged[0].hunks[0];

        // Find indices of add lines in hunk 0 of staged diff
        let add_indices: Vec<u32> = hunk0.lines.iter().enumerate()
            .filter(|(_, l)| matches!(l.origin, crate::git::types::DiffOrigin::Add))
            .map(|(i, _)| i as u32)
            .collect();
        assert!(!add_indices.is_empty(), "expected add lines in staged hunk 0");

        // Unstage only the add lines from hunk 0
        super::unstage_lines_inner(&path, "multi.txt", 0, add_indices, &state_map)
            .expect("unstage_lines_inner failed");

        // Verify: unstaged diff should now contain add lines that were unstaged
        let unstaged_after = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        assert!(!unstaged_after.is_empty(), "expected unstaged diff after unstaging lines");
        let has_adds_unstaged = unstaged_after[0].hunks.iter()
            .flat_map(|h| &h.lines)
            .any(|l| matches!(l.origin, crate::git::types::DiffOrigin::Add));
        assert!(has_adds_unstaged, "expected add lines in unstaged diff after unstaging");
    }

    // Test 24 — discard_lines_discards_selected
    #[test]
    fn discard_lines_discards_selected() {
        let dir = make_test_repo();
        let path = dir.path().to_string_lossy().to_string();
        let state_map = make_state_map(dir.path());

        create_add_delete_hunk_file(dir.path());

        // Get content of add lines in hunk 0 before discard
        let unstaged = diff_unstaged_inner(&path, "multi.txt", &state_map)
            .expect("diff_unstaged_inner failed");
        let hunk0 = &unstaged[0].hunks[0];

        // Find indices and content of add lines
        let add_info: Vec<(u32, String)> = hunk0.lines.iter().enumerate()
            .filter(|(_, l)| matches!(l.origin, crate::git::types::DiffOrigin::Add))
            .map(|(i, l)| (i as u32, l.content.clone()))
            .collect();
        assert!(!add_info.is_empty(), "expected add lines in hunk 0");
        let add_indices: Vec<u32> = add_info.iter().map(|(i, _)| *i).collect();
        let add_contents: Vec<String> = add_info.iter().map(|(_, c)| c.clone()).collect();

        // Discard only the add lines from hunk 0
        super::discard_lines_inner(&path, "multi.txt", 0, add_indices, &state_map)
            .expect("discard_lines_inner failed");

        // Verify: file content no longer has the discarded add lines
        let file_content = std::fs::read_to_string(dir.path().join("multi.txt")).unwrap();
        for content in &add_contents {
            let trimmed = content.trim();
            assert!(!file_content.contains(trimmed),
                "expected discarded add line '{}' to be gone from file", trimmed);
        }
    }
}
