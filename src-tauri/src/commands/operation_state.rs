use crate::error::TrunkError;
use crate::git::{
    graph,
    types::{GraphResult, OperationInfo, OperationType},
};
use crate::shell_env;
use crate::state::{CommitCache, RepoState};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn open_repo(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<git2::Repository, TrunkError> {
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
            let target = repo
                .head()
                .ok()
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
                .ok()
                .map(|s| s.trim().replace("refs/heads/", ""));
            let onto_oid = std::fs::read_to_string(rebase_dir.join("onto"))
                .ok()
                .map(|s| s.trim().to_owned());
            let onto_branch = onto_oid.and_then(|oid| resolve_oid_to_branch(&repo, &oid));
            let msgnum = std::fs::read_to_string(rebase_dir.join("msgnum"))
                .ok()
                .map(|s| s.trim().to_owned());
            let end = std::fs::read_to_string(rebase_dir.join("end"))
                .ok()
                .map(|s| s.trim().to_owned());
            let progress = match (msgnum, end) {
                (Some(m), Some(e)) => Some(format!("{}/{}", m, e)),
                _ => None,
            };
            let rebase_message = std::fs::read_to_string(rebase_dir.join("message"))
                .ok()
                .map(|s| s.trim().to_owned());
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
                source_branch: None,
                target_branch: None,
                progress: None,
                source_color_index: None,
                target_color_index: None,
                rebase_message: None,
            })
        }
        git2::RepositoryState::Revert | git2::RepositoryState::RevertSequence => {
            Ok(OperationInfo {
                op_type: OperationType::Revert,
                source_branch: None,
                target_branch: None,
                progress: None,
                source_color_index: None,
                target_color_index: None,
                rebase_message: None,
            })
        }
        _ => Ok(OperationInfo {
            op_type: OperationType::None,
            source_branch: None,
            target_branch: None,
            progress: None,
            source_color_index: None,
            target_color_index: None,
            rebase_message: None,
        }),
    }
}

// --- CLI operation inner functions ---

pub fn merge_continue_inner(
    path: &str,
    message: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = if let Some(msg) = message {
        // Custom message: use git commit directly (works during merge state)
        std::process::Command::new("git")
            .args(["commit", "-m", msg])
            .current_dir(path_buf)
            .env("PATH", shell_env::system_path())
            .output()
            .map_err(|e| TrunkError::new("merge_error", e.to_string()))?
    } else {
        std::process::Command::new("git")
            .args(["merge", "--continue"])
            .current_dir(path_buf)
            .env("PATH", shell_env::system_path())
            .env("GIT_EDITOR", "true")
            .output()
            .map_err(|e| TrunkError::new("merge_error", e.to_string()))?
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("merge_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn merge_abort_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["merge", "--abort"])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("merge_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("merge_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn rebase_continue_inner(
    path: &str,
    message: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    // Write edited message to .git/rebase-merge/message before continuing
    if let Some(msg) = message {
        let repo = git2::Repository::open(path_buf)?;
        let git_dir = repo.path();
        let rebase_dir = if git_dir.join("rebase-merge").exists() {
            git_dir.join("rebase-merge")
        } else {
            git_dir.join("rebase-apply")
        };
        let msg_file = rebase_dir.join("message");
        if msg_file.exists() {
            std::fs::write(&msg_file, msg)
                .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
        }
    }

    let output = std::process::Command::new("git")
        .args(["rebase", "--continue"])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        // Next commit hit a conflict — rebase paused at next step, not an error
        if !stderr.to_lowercase().contains("conflict")
            && !stderr.to_lowercase().contains("could not apply")
        {
            return Err(TrunkError::new("rebase_error", stderr));
        }
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn rebase_skip_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["rebase", "--skip"])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("rebase_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn rebase_abort_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["rebase", "--abort"])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("rebase_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

// --- Start merge/rebase ---

pub fn merge_branch_inner(
    path: &str,
    branch: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["merge", branch, "--no-edit"])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .env("GIT_EDITOR", "true")
        .output()
        .map_err(|e| TrunkError::new("merge_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.to_lowercase().contains("conflict") {
            // Conflicts: rebuild graph so UI picks up the merge state
            let mut repo = git2::Repository::open(path_buf)?;
            return graph::walk_commits(&mut repo, 0, usize::MAX);
        }
        return Err(TrunkError::new("merge_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn rebase_branch_inner(
    path: &str,
    onto_branch: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let output = std::process::Command::new("git")
        .args(["rebase", onto_branch])
        .current_dir(path_buf)
        .env("PATH", shell_env::system_path())
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.to_lowercase().contains("conflict") {
            let mut repo = git2::Repository::open(path_buf)?;
            return graph::walk_commits(&mut repo, 0, usize::MAX);
        }
        return Err(TrunkError::new("rebase_error", stderr.to_string()));
    }
    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
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
fn find_branch_color(
    commits: &[crate::git::types::GraphCommit],
    branch_name: &str,
) -> Option<usize> {
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
    let graph_result =
        tauri::async_runtime::spawn_blocking(move || merge_abort_inner(&path_clone, &state_map))
            .await
            .map_err(|e| {
                serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
            })?
            .map_err(|e| serde_json::to_string(&e).unwrap())?;
    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn rebase_continue(
    path: String,
    message: Option<String>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        rebase_continue_inner(&path_clone, message.as_deref(), &state_map)
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
    let graph_result =
        tauri::async_runtime::spawn_blocking(move || rebase_skip_inner(&path_clone, &state_map))
            .await
            .map_err(|e| {
                serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
            })?
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
    let graph_result =
        tauri::async_runtime::spawn_blocking(move || rebase_abort_inner(&path_clone, &state_map))
            .await
            .map_err(|e| {
                serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
            })?
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
    use git2::{Repository, Signature};
    use std::process::Command;
    use tempfile::TempDir;

    // Temp-repo harness (mirrors git/review.rs:662 make_repo). Real git2 +
    // tempfile, no mocks (classical TDD). The code-under-test shells out to
    // `git`, so the repo config must carry a committer identity — git2
    // Signatures do not satisfy the subprocess `git commit`.

    fn make_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        {
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_str("user.email", "test@example.com").unwrap();
        }
        (dir, repo)
    }

    fn sig() -> Signature<'static> {
        Signature::new("Test", "test@example.com", &git2::Time::new(0, 0)).unwrap()
    }

    /// Path string used to key the state_map (what open_repo looks up).
    fn path_str(dir: &TempDir) -> String {
        dir.path().to_str().unwrap().to_string()
    }

    fn state_map_for(dir: &TempDir) -> HashMap<String, PathBuf> {
        let mut map = HashMap::new();
        map.insert(path_str(dir), dir.path().to_path_buf());
        map
    }

    /// Commit `path`=`content` onto `parents`, leaving the index/worktree clean.
    /// Writes the blob through the object DB only (no worktree write), so the
    /// repo stays clean for subsequent subprocess `git merge`.
    fn commit_file(
        repo: &Repository,
        message: &str,
        parents: &[git2::Oid],
        file: &str,
        content: &[u8],
    ) -> git2::Oid {
        let blob_oid = repo.blob(content).unwrap();
        let mut builder = repo.treebuilder(None).unwrap();
        builder
            .insert(file, blob_oid, git2::FileMode::Blob.into())
            .unwrap();
        let tree = repo.find_tree(builder.write().unwrap()).unwrap();
        let parent_commits: Vec<_> = parents
            .iter()
            .map(|oid| repo.find_commit(*oid).unwrap())
            .collect();
        let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();
        let s = sig();
        repo.commit(Some("HEAD"), &s, &s, message, &tree, &parent_refs)
            .unwrap()
    }

    /// Point `branch` at `oid` (creating it) without moving HEAD.
    fn set_branch(repo: &Repository, branch: &str, oid: git2::Oid) {
        let commit = repo.find_commit(oid).unwrap();
        repo.branch(branch, &commit, true).unwrap();
    }

    fn merge_head_path(dir: &TempDir) -> PathBuf {
        dir.path().join(".git").join("MERGE_HEAD")
    }

    fn read_merge_msg(dir: &TempDir) -> String {
        std::fs::read_to_string(dir.path().join(".git").join("MERGE_MSG")).unwrap()
    }

    /// A repo where `feature` is strictly ahead of `main` (no divergence) so a
    /// merge is a pure fast-forward. HEAD stays on `main`.
    fn ff_repo() -> (TempDir, Repository) {
        let (dir, repo) = make_repo();
        let base = commit_file(&repo, "base", &[], "base.txt", b"base\n");
        // main stays at base; feature advances.
        repo.branch("main", &repo.find_commit(base).unwrap(), true)
            .unwrap();
        repo.set_head("refs/heads/main").unwrap();
        let ahead = commit_file(&repo, "ahead", &[base], "ahead.txt", b"ahead\n");
        set_branch(&repo, "feature", ahead);
        // Reset HEAD/main back to base so feature is strictly ahead.
        repo.reset(
            &repo.find_object(base, None).unwrap(),
            git2::ResetType::Hard,
            None,
        )
        .unwrap();
        (dir, repo)
    }

    /// Divergent branches that merge cleanly (different files). HEAD on `main`.
    fn clean_divergent_repo() -> (TempDir, Repository) {
        let (dir, repo) = make_repo();
        let base = commit_file(&repo, "base", &[], "base.txt", b"base\n");
        repo.branch("main", &repo.find_commit(base).unwrap(), true)
            .unwrap();
        repo.set_head("refs/heads/main").unwrap();
        // feature: add feature.txt
        let feat = commit_file(&repo, "feat", &[base], "feature.txt", b"feature\n");
        set_branch(&repo, "feature", feat);
        // main: add main.txt (diverges)
        repo.reset(
            &repo.find_object(base, None).unwrap(),
            git2::ResetType::Hard,
            None,
        )
        .unwrap();
        commit_file(&repo, "main change", &[base], "main.txt", b"main\n");
        (dir, repo)
    }

    /// Conflicting divergent branches (same file, different content). HEAD on `main`.
    fn conflict_divergent_repo() -> (TempDir, Repository) {
        let (dir, repo) = make_repo();
        let base = commit_file(&repo, "base", &[], "f.txt", b"base\n");
        repo.branch("main", &repo.find_commit(base).unwrap(), true)
            .unwrap();
        repo.set_head("refs/heads/main").unwrap();
        let feat = commit_file(&repo, "feat", &[base], "f.txt", b"feature side\n");
        set_branch(&repo, "feature", feat);
        repo.reset(
            &repo.find_object(base, None).unwrap(),
            git2::ResetType::Hard,
            None,
        )
        .unwrap();
        commit_file(&repo, "main change", &[base], "f.txt", b"main side\n");
        (dir, repo)
    }

    fn kind_of(result: &MergeBeginResult) -> String {
        serde_json::to_value(result).unwrap()["kind"]
            .as_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn merge_branch_begin_fast_forwards_without_editor() {
        let (dir, _repo) = ff_repo();
        let map = state_map_for(&dir);
        let result = merge_branch_begin_inner(&path_str(&dir), "feature", &map).unwrap();
        assert_eq!(kind_of(&result), "fast_forwarded");
        assert!(
            !merge_head_path(&dir).exists(),
            "fast-forward must not leave MERGE_HEAD"
        );
    }

    #[test]
    fn merge_branch_begin_non_ff_clean_returns_ready_with_verbatim_message() {
        let (dir, _repo) = clean_divergent_repo();
        let map = state_map_for(&dir);
        let result = merge_branch_begin_inner(&path_str(&dir), "feature", &map).unwrap();
        assert_eq!(kind_of(&result), "ready");
        let message = match result {
            MergeBeginResult::Ready { message, .. } => message,
            other => panic!("expected Ready, got {:?}", kind_of(&other)),
        };
        assert_eq!(message, read_merge_msg(&dir), "message must be MERGE_MSG verbatim");
        assert!(
            message.starts_with("Merge branch 'feature'"),
            "got: {message:?}"
        );
        assert!(merge_head_path(&dir).exists(), "non-ff merge must set MERGE_HEAD");
    }

    #[test]
    fn merge_branch_begin_into_non_default_branch_has_into_suffix() {
        let (dir, repo) = clean_divergent_repo();
        // Re-point HEAD onto a non-default branch `devel` at the current tip.
        let head_oid = repo.head().unwrap().target().unwrap();
        set_branch(&repo, "devel", head_oid);
        repo.set_head("refs/heads/devel").unwrap();
        let map = state_map_for(&dir);
        let result = merge_branch_begin_inner(&path_str(&dir), "feature", &map).unwrap();
        let message = match result {
            MergeBeginResult::Ready { message, .. } => message,
            other => panic!("expected Ready, got {:?}", kind_of(&other)),
        };
        assert!(
            message.contains("into devel"),
            "verbatim MERGE_MSG must carry the current-branch suffix; got: {message:?}"
        );
    }

    #[test]
    fn merge_branch_begin_conflict_returns_conflicts_not_err() {
        let (dir, _repo) = conflict_divergent_repo();
        let map = state_map_for(&dir);
        let result = merge_branch_begin_inner(&path_str(&dir), "feature", &map).unwrap();
        assert_eq!(kind_of(&result), "conflicts");
        assert!(
            merge_head_path(&dir).exists(),
            "conflicted merge leaves MERGE_HEAD for the continue UI"
        );
    }

    #[test]
    fn get_merge_message_returns_merge_msg_verbatim() {
        let (dir, _repo) = conflict_divergent_repo();
        // Start a conflicted merge so MERGE_MSG carries a `# Conflicts:` block.
        let _ = Command::new("git")
            .args(["merge", "--no-commit", "feature"])
            .current_dir(dir.path())
            .env("PATH", shell_env::system_path())
            .output()
            .unwrap();
        let raw = read_merge_msg(&dir);
        assert!(
            raw.contains("# Conflicts:"),
            "setup precondition: conflicted MERGE_MSG should carry the comment block; got: {raw:?}"
        );
        let map = state_map_for(&dir);
        let got = get_merge_message_inner(&path_str(&dir), &map).unwrap();
        assert_eq!(
            got,
            Some(raw),
            "get_merge_message returns MERGE_MSG verbatim, including # Conflicts: lines"
        );
    }
}
