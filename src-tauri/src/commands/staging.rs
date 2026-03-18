use std::collections::HashMap;
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
    Err(TrunkError::new("not_implemented", "stage_hunk not yet implemented"))
}

pub fn unstage_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    Err(TrunkError::new("not_implemented", "unstage_hunk not yet implemented"))
}

pub fn discard_hunk_inner(
    path: &str,
    file_path: &str,
    hunk_index: u32,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    Err(TrunkError::new("not_implemented", "discard_hunk not yet implemented"))
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
}
