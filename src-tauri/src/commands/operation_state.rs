use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use crate::error::TrunkError;
use crate::git::{graph, types::{GraphResult, OperationInfo, OperationType}};
use crate::state::{CommitCache, RepoState};

fn open_repo(path: &str, state_map: &HashMap<String, PathBuf>) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

fn extract_merge_source(merge_msg: Option<&str>) -> Option<String> {
    let msg = merge_msg?;
    // Patterns: "Merge branch 'feature'" or "Merge remote-tracking branch 'origin/feature'"
    if let Some(rest) = msg.strip_prefix("Merge branch '") {
        return rest.split('\'').next().map(String::from);
    }
    if let Some(rest) = msg.strip_prefix("Merge remote-tracking branch '") {
        return rest.split('\'').next().map(String::from);
    }
    None // Unparseable -- caller shows generic "Merge in progress"
}

fn resolve_oid_to_branch(repo: &git2::Repository, oid_str: &str) -> Option<String> {
    let oid = git2::Oid::from_str(oid_str).ok()?;
    for reference in repo.references().ok()?.flatten() {
        if reference.is_branch() {
            if let Some(target) = reference.target() {
                if target == oid {
                    return reference.shorthand().map(String::from);
                }
            }
        }
    }
    // Fallback: return short OID
    Some(oid_str.chars().take(7).collect())
}

pub fn get_operation_state_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<OperationInfo, TrunkError> {
    let repo = open_repo(path, state_map)?;
    let state = repo.state();

    match state {
        git2::RepositoryState::Merge => {
            let git_dir = repo.path();
            let merge_msg = std::fs::read_to_string(git_dir.join("MERGE_MSG")).ok();
            let source = extract_merge_source(merge_msg.as_deref());
            let target = repo.head().ok()
                .and_then(|h| h.shorthand().map(String::from));
            Ok(OperationInfo {
                op_type: OperationType::Merge,
                source_branch: source,
                target_branch: target,
                progress: None,
                source_color_index: None,
                target_color_index: None,
                rebase_message: None,
            })
        }
        git2::RepositoryState::Rebase
        | git2::RepositoryState::RebaseInteractive
        | git2::RepositoryState::RebaseMerge => {
            let git_dir = repo.path();
            let rebase_dir = if git_dir.join("rebase-merge").exists() {
                git_dir.join("rebase-merge")
            } else {
                git_dir.join("rebase-apply")
            };
            let head_name = std::fs::read_to_string(rebase_dir.join("head-name"))
                .ok().map(|s| s.trim().replace("refs/heads/", ""));
            let onto_oid = std::fs::read_to_string(rebase_dir.join("onto"))
                .ok().map(|s| s.trim().to_owned());
            let onto_branch = onto_oid.and_then(|oid| resolve_oid_to_branch(&repo, &oid));
            let msgnum = std::fs::read_to_string(rebase_dir.join("msgnum"))
                .ok().map(|s| s.trim().to_owned());
            let end = std::fs::read_to_string(rebase_dir.join("end"))
                .ok().map(|s| s.trim().to_owned());
            let progress = match (msgnum, end) {
                (Some(m), Some(e)) => Some(format!("{}/{}", m, e)),
                _ => None,
            };
            let rebase_message = std::fs::read_to_string(rebase_dir.join("message"))
                .ok().map(|s| s.trim().to_owned());
            Ok(OperationInfo {
                op_type: OperationType::Rebase,
                source_branch: head_name,
                target_branch: onto_branch,
                progress,
                source_color_index: None,
                target_color_index: None,
                rebase_message,
            })
        }
        git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
            Ok(OperationInfo {
                op_type: OperationType::CherryPick,
                source_branch: None, target_branch: None, progress: None,
                source_color_index: None, target_color_index: None, rebase_message: None,
            })
        }
        git2::RepositoryState::Revert | git2::RepositoryState::RevertSequence => {
            Ok(OperationInfo {
                op_type: OperationType::Revert,
                source_branch: None, target_branch: None, progress: None,
                source_color_index: None, target_color_index: None, rebase_message: None,
            })
        }
        _ => {
            Ok(OperationInfo {
                op_type: OperationType::None,
                source_branch: None, target_branch: None, progress: None,
                source_color_index: None, target_color_index: None, rebase_message: None,
            })
        }
    }
}

// --- CLI operation inner functions ---

pub fn merge_continue_inner(
    path: &str,
    message: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = if let Some(msg) = message {
        // Custom message: use git commit directly (works during merge state)
        std::process::Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(path_buf)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| TrunkError::new("merge_error", e.to_string()))?
    } else {
        std::process::Command::new("git")
            .args(["merge", "--continue"])
            .current_dir(path_buf)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_EDITOR", "true")
            .output()
            .map_err(|e| TrunkError::new("merge_error", e.to_string()))?
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("merge_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

pub fn merge_abort_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["merge", "--abort"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .output()
        .map_err(|e| TrunkError::new("merge_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("merge_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

pub fn rebase_continue_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["rebase", "--continue"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("rebase_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

pub fn rebase_skip_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["rebase", "--skip"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("rebase_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

pub fn rebase_abort_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["rebase", "--abort"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("rebase_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

// --- Start merge/rebase ---

pub fn merge_branch_inner(
    path: &str,
    branch: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["merge", branch, "--no-edit"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .output()
        .map_err(|e| TrunkError::new("merge_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.to_lowercase().contains("conflict") {
            // Conflicts: rebuild graph so UI picks up the merge state
            let mut repo = git2::Repository::open(path_buf)?;
            return graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from);
        }
        return Err(TrunkError::new("merge_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

pub fn rebase_branch_inner(
    path: &str,
    onto_branch: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map.get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["rebase", onto_branch])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.to_lowercase().contains("conflict") {
            let mut repo = git2::Repository::open(path_buf)?;
            return graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from);
        }
        return Err(TrunkError::new("rebase_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

// --- Tauri command wrappers ---

#[tauri::command]
pub async fn get_operation_state(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
) -> Result<OperationInfo, String> {
    let state_map = state.0.lock().unwrap().clone();
    let graph_cache = cache.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut info = get_operation_state_inner(&path, &state_map)?;
        // Look up branch color indexes from the cached graph
        if let Some(graph) = graph_cache.get(&path) {
            if let Some(ref src) = info.source_branch {
                info.source_color_index = find_branch_color(&graph.commits, src);
            }
            if let Some(ref tgt) = info.target_branch {
                info.target_color_index = find_branch_color(&graph.commits, tgt);
            }
        }
        Ok(info)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}

/// Find a branch's color_index by searching ref labels in the cached graph.
fn find_branch_color(commits: &[crate::git::types::GraphCommit], branch_name: &str) -> Option<usize> {
    for commit in commits {
        for r in &commit.refs {
            if r.short_name == branch_name {
                return Some(r.color_index);
            }
        }
    }
    None
}

#[tauri::command]
pub async fn merge_continue(
    path: String,
    message: Option<String>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        merge_continue_inner(&path_clone, message.as_deref(), &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn merge_abort(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        merge_abort_inner(&path_clone, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn rebase_continue(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        rebase_continue_inner(&path_clone, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn rebase_skip(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        rebase_skip_inner(&path_clone, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn rebase_abort(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        rebase_abort_inner(&path_clone, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn merge_branch(
    path: String,
    branch: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        merge_branch_inner(&path_clone, &branch, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn rebase_branch(
    path: String,
    onto_branch: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        rebase_branch_inner(&path_clone, &onto_branch, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_repo() -> (TempDir, HashMap<String, PathBuf>) {
        let dir = TempDir::new().unwrap();
        let path_str = dir.path().to_str().unwrap().to_owned();
        {
            let repo = git2::Repository::init(dir.path()).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
            drop(config);

            let sig = repo.signature().unwrap();
            fs::write(dir.path().join("file.txt"), "hello").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(Some("refs/heads/main"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
            drop(tree);

            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();
        }
        let mut state_map = HashMap::new();
        state_map.insert(path_str.clone(), dir.path().to_owned());
        (dir, state_map)
    }

    // --- extract_merge_source tests ---

    #[test]
    fn extract_merge_source_local_branch() {
        let result = extract_merge_source(Some("Merge branch 'feature'"));
        assert_eq!(result, Some("feature".to_string()));
    }

    #[test]
    fn extract_merge_source_remote_tracking_branch() {
        let result = extract_merge_source(Some("Merge remote-tracking branch 'origin/feature'"));
        assert_eq!(result, Some("origin/feature".to_string()));
    }

    #[test]
    fn extract_merge_source_unparseable() {
        let result = extract_merge_source(Some("Some random commit message"));
        assert_eq!(result, None);
    }

    #[test]
    fn extract_merge_source_none_input() {
        let result = extract_merge_source(None);
        assert_eq!(result, None);
    }

    // --- get_operation_state_inner tests ---

    #[test]
    fn operation_state_clean_repo_returns_none() {
        let (dir, state_map) = make_test_repo();
        let path = dir.path().to_str().unwrap();

        let result = get_operation_state_inner(path, &state_map);
        assert!(result.is_ok(), "get_operation_state_inner should succeed on clean repo");

        let info = result.unwrap();
        assert!(matches!(info.op_type, OperationType::None));
        assert!(info.source_branch.is_none());
        assert!(info.target_branch.is_none());
        assert!(info.progress.is_none());
    }

    #[test]
    fn operation_state_merge_in_progress() {
        let (dir, state_map) = make_test_repo();
        let path = dir.path().to_str().unwrap();

        // Create a branch with a conflicting change
        {
            let repo = git2::Repository::open(dir.path()).unwrap();
            let sig = repo.signature().unwrap();
            let head = repo.head().unwrap().peel_to_commit().unwrap();

            // Create feature branch from HEAD
            repo.branch("feature", &head, false).unwrap();
            repo.set_head("refs/heads/feature").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();

            fs::write(dir.path().join("file.txt"), "feature content").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(
                Some("refs/heads/feature"),
                &sig, &sig,
                "Feature commit",
                &tree,
                &[&head],
            ).unwrap();

            // Switch back to main and make a conflicting change
            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();

            fs::write(dir.path().join("file.txt"), "main content").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let main_head = repo.head().unwrap().peel_to_commit().unwrap();
            repo.commit(
                Some("refs/heads/main"),
                &sig, &sig,
                "Main commit",
                &tree,
                &[&main_head],
            ).unwrap();

            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();
        }

        // Start a merge that will conflict (using git CLI to create proper MERGE_HEAD/MERGE_MSG)
        let output = std::process::Command::new("git")
            .args(["merge", "feature"])
            .current_dir(dir.path())
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .unwrap();
        // Merge should fail with conflict
        assert!(!output.status.success(), "merge should fail with conflict");

        let result = get_operation_state_inner(path, &state_map);
        assert!(result.is_ok(), "get_operation_state_inner should succeed during merge");

        let info = result.unwrap();
        assert!(matches!(info.op_type, OperationType::Merge));
        assert_eq!(info.source_branch, Some("feature".to_string()));
        assert_eq!(info.target_branch, Some("main".to_string()));
        assert!(info.progress.is_none());
    }

    // --- resolve_oid_to_branch tests ---

    #[test]
    fn resolve_oid_to_branch_known_branch() {
        let (dir, _state_map) = make_test_repo();
        let repo = git2::Repository::open(dir.path()).unwrap();
        let head_oid = repo.head().unwrap().target().unwrap().to_string();

        let result = resolve_oid_to_branch(&repo, &head_oid);
        assert_eq!(result, Some("main".to_string()));
    }

    #[test]
    fn resolve_oid_to_branch_unknown_oid_returns_short_hash() {
        let (dir, _state_map) = make_test_repo();
        let repo = git2::Repository::open(dir.path()).unwrap();
        // Use a valid but non-existent OID format
        let fake_oid = "abcdef1234567890abcdef1234567890abcdef12";

        let result = resolve_oid_to_branch(&repo, fake_oid);
        assert_eq!(result, Some("abcdef1".to_string()));
    }
}
