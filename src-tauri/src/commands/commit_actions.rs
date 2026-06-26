use crate::error::TrunkError;
use crate::git::{
    backend, command_runner, graph,
    types::{GraphResult, RepoDescriptor, UndoResult},
};
use crate::state::{CommitCache, RepoState};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

/// Outcome of a clean two-step revert begin. The async wrapper emits
/// `repo-changed` (REVERT_HEAD is set before the editor opens), so a later
/// cancel still surfaces the in-progress UI. A conflicted revert returns
/// `Err(conflict_state)` instead — there is a single editor outcome, so a
/// 2-field struct is simpler than a tagged enum here.
#[derive(Debug, serde::Serialize)]
pub struct RevertBeginResult {
    pub graph: GraphResult,
    pub message: Option<String>,
}

fn run_git_action(
    path: &str,
    args: &[&str],
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
    spawn_error_code: &str,
) -> Result<std::process::Output, TrunkError> {
    let repo = crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    command_runner::git_output(&repo, args, spawn_error_code)
}

fn refresh_graph(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<GraphResult, TrunkError> {
    crate::commands::refresh_graph_from_state(path, state_map, descriptor_map)
}

pub fn checkout_commit_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;

    if crate::git::repository::is_repo_dirty(&repo)? {
        return Err(TrunkError::new(
            "dirty_workdir",
            "Working tree has uncommitted changes",
        ));
    }

    let obj = repo.revparse_single(oid)?;
    repo.checkout_tree(&obj, Some(&mut git2::build::CheckoutBuilder::new().safe()))?;
    repo.set_head_detached(obj.id())?;
    drop(obj);
    drop(repo);

    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo2, 0, usize::MAX)
}

pub fn create_tag_inner(
    path: &str,
    oid: &str,
    tag_name: &str,
    message: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let obj = repo.revparse_single(oid)?;
    let sig = repo.signature().map_err(TrunkError::from)?;
    let msg = if message.trim().is_empty() {
        tag_name.to_owned()
    } else {
        message.to_owned()
    };
    repo.tag(tag_name, &obj, &sig, &msg, false)?;
    drop(obj);
    drop(repo);

    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo2, 0, usize::MAX)
}

pub fn delete_tag_inner(
    path: &str,
    tag_name: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let tag_ref_name = format!("refs/tags/{}", tag_name);
    let mut reference = repo.find_reference(&tag_ref_name)?;
    reference.delete()?;
    drop(reference);
    drop(repo);

    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo2, 0, usize::MAX)
}

pub fn cherry_pick_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<GraphResult, TrunkError> {
    let output = run_git_action(
        path,
        &["cherry-pick", oid],
        state_map,
        descriptor_map,
        "cherry_pick_error",
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = if stderr.to_lowercase().contains("conflict") {
            "conflict_state"
        } else {
            "cherry_pick_error"
        };
        return Err(TrunkError::new(code, stderr.to_string()));
    }

    refresh_graph(path, state_map, descriptor_map)
}

pub fn revert_commit_begin_inner(
    path: &str,
    oid: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<RevertBeginResult, TrunkError> {
    // Stage the revert without committing so the editor can edit the message.
    // git writes the default message (Revert "<subject>" + full 40-char OID) to
    // .git/MERGE_MSG. REVERT_HEAD is set; the wrapper emits repo-changed.
    let output = run_git_action(
        path,
        &["revert", "--no-commit", oid],
        state_map,
        descriptor_map,
        "revert_error",
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = if stderr.to_lowercase().contains("conflict") {
            "conflict_state"
        } else {
            "revert_error"
        };
        return Err(TrunkError::new(code, stderr.to_string()));
    }

    let graph = refresh_graph(path, state_map, descriptor_map)?;
    // Verbatim default — `# Conflicts:` lines (conflicted revert) are stripped at
    // commit time via --cleanup=strip, never here.
    let message = super::operation_state::get_merge_message_inner_with_descriptors(
        path,
        state_map,
        descriptor_map,
    )?;
    Ok(RevertBeginResult { graph, message })
}

pub fn revert_continue_inner(
    path: &str,
    message: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<GraphResult, TrunkError> {
    // --cleanup=strip drops git's `# Conflicts:` comment block so conflicted
    // revert bodies stay clean (MSG-03 fidelity). git commit -m clears REVERT_HEAD.
    let output = run_git_action(
        path,
        &["commit", "-m", message, "--cleanup=strip"],
        state_map,
        descriptor_map,
        "revert_error",
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("revert_error", stderr.to_string()));
    }
    refresh_graph(path, state_map, descriptor_map)
}

pub fn revert_abort_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<GraphResult, TrunkError> {
    // The MSG-06 recovery path for revert: clears REVERT_HEAD + restores a clean
    // tree. Without it a cancelled revert traps the user (RESEARCH finding 4).
    let output = run_git_action(
        path,
        &["revert", "--abort"],
        state_map,
        descriptor_map,
        "revert_error",
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("revert_error", stderr.to_string()));
    }
    refresh_graph(path, state_map, descriptor_map)
}

pub fn reset_to_commit_inner(
    path: &str,
    oid: &str,
    mode: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<GraphResult, TrunkError> {
    let valid_modes = ["soft", "mixed", "hard"];
    if !valid_modes.contains(&mode) {
        return Err(TrunkError::new(
            "invalid_mode",
            format!("Invalid reset mode: {}", mode),
        ));
    }

    let reset_mode = format!("--{}", mode);
    let output = run_git_action(
        path,
        &["reset", &reset_mode, oid],
        state_map,
        descriptor_map,
        "reset_error",
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("reset_error", stderr.to_string()));
    }

    refresh_graph(path, state_map, descriptor_map)
}

#[tauri::command]
pub async fn reset_to_commit(
    path: String,
    oid: String,
    mode: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        reset_to_commit_inner(&path_clone, &oid, &mode, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn checkout_commit(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        checkout_commit_inner(&path_clone, &oid, &state_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn create_tag(
    path: String,
    oid: String,
    tag_name: String,
    message: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        create_tag_inner(&path_clone, &oid, &tag_name, &message, &state_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn delete_tag(
    path: String,
    tag_name: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        delete_tag_inner(&path_clone, &tag_name, &state_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn cherry_pick(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        cherry_pick_inner(&path_clone, &oid, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn revert_commit_begin(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<RevertBeginResult, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        revert_commit_begin_inner(&path_clone, &oid, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    // begin mutated the repo (REVERT_HEAD set + staged) before the editor opens,
    // so cache the rebuilt graph and emit repo-changed — a later cancel must
    // still surface the in-progress banner (RESEARCH Pitfall 2/4).
    cache
        .0
        .lock()
        .unwrap()
        .insert(path.clone(), result.graph.clone());
    let _ = app.emit("repo-changed", path);
    Ok(result)
}

#[tauri::command]
pub async fn revert_continue(
    path: String,
    message: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        revert_continue_inner(&path_clone, &message, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn revert_abort(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        revert_abort_inner(&path_clone, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

pub fn undo_commit_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<UndoResult, TrunkError> {
    let descriptor = crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    backend::resolve_backend(descriptor)?.undo_commit(path, state_map, descriptor_map)
}

pub fn undo_commit_local_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<UndoResult, TrunkError> {
    let repo_descriptor =
        crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    backend::ensure_backend_supported(&repo_descriptor)?;
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let head = repo.head()?.peel_to_commit()?;

    if head.parent_count() == 0 {
        return Err(TrunkError::new(
            "nothing_to_undo",
            "Cannot undo the initial commit",
        ));
    }
    if head.parent_count() > 1 {
        return Err(TrunkError::new(
            "merge_commit",
            "Cannot undo a merge commit",
        ));
    }

    let subject = head.summary().unwrap_or("").to_owned();
    let body = head.body().map(str::to_owned);

    let output = run_git_action(
        path,
        &["reset", "--soft", "HEAD~1"],
        state_map,
        descriptor_map,
        "undo_error",
    )?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("undo_error", stderr.to_string()));
    }

    Ok(UndoResult { subject, body })
}

pub fn redo_commit_inner(
    path: &str,
    subject: &str,
    body: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    super::commit::create_commit_inner(path, subject, body, state_map)
}

pub fn redo_commit_inner_with_descriptors(
    path: &str,
    subject: &str,
    body: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<(), TrunkError> {
    let descriptor = crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    backend::resolve_backend(descriptor)?.redo_commit(
        path,
        subject,
        body,
        state_map,
        descriptor_map,
    )
}

pub fn check_undo_available_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<bool, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let head = match repo.head() {
        Ok(h) => match h.peel_to_commit() {
            Ok(c) => c,
            Err(_) => return Ok(false),
        },
        Err(_) => return Ok(false),
    };
    // Can undo if exactly one parent (not initial, not merge)
    Ok(head.parent_count() == 1)
}

pub fn check_undo_available_inner_with_descriptors(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<bool, TrunkError> {
    let descriptor = crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    backend::resolve_backend(descriptor)?.check_undo_available(path, state_map, descriptor_map)
}

#[tauri::command]
pub async fn undo_commit(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<UndoResult, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let (undo_result, graph_result) = tauri::async_runtime::spawn_blocking(move || {
        let undo = undo_commit_inner(&path_clone, &state_map, &descriptor_map)?;
        let graph = refresh_graph(&path_clone, &state_map, &descriptor_map)?;
        Ok::<(UndoResult, GraphResult), TrunkError>((undo, graph))
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(undo_result)
}

#[tauri::command]
pub async fn redo_commit(
    path: String,
    subject: String,
    body: Option<String>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        redo_commit_inner_with_descriptors(
            &path_clone,
            &subject,
            body.as_deref(),
            &state_map,
            &descriptor_map,
        )?;
        refresh_graph(&path_clone, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn check_undo_available(
    path: String,
    state: State<'_, RepoState>,
) -> Result<bool, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        check_undo_available_inner_with_descriptors(&path, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Repository, Signature};
    use tempfile::TempDir;

    // Temp-repo harness (mirrors operation_state.rs tests / git/review.rs:662).
    // Real git2 + tempfile, no mocks (classical TDD). The code-under-test shells
    // out to `git`, so the repo config must carry a committer identity.

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

    fn path_str(dir: &TempDir) -> String {
        dir.path().to_str().unwrap().to_string()
    }

    fn state_map_for(dir: &TempDir) -> HashMap<String, PathBuf> {
        let mut map = HashMap::new();
        map.insert(path_str(dir), dir.path().to_path_buf());
        map
    }

    /// Commit `file`=`content` onto `parents`, carrying the first parent's tree
    /// forward so existing files persist (faithful linear history). Returns OID.
    fn commit_file(
        repo: &Repository,
        message: &str,
        parents: &[git2::Oid],
        file: &str,
        content: &[u8],
    ) -> git2::Oid {
        let blob_oid = repo.blob(content).unwrap();
        let base_tree = parents
            .first()
            .map(|oid| repo.find_commit(*oid).unwrap().tree().unwrap());
        let mut builder = repo.treebuilder(base_tree.as_ref()).unwrap();
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

    fn revert_head_path(dir: &TempDir) -> PathBuf {
        dir.path().join(".git").join("REVERT_HEAD")
    }

    fn head_body(repo: &Repository) -> String {
        repo.head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .message()
            .unwrap()
            .to_string()
    }

    /// A two-commit linear repo: base then a "v2" change to `f.txt`. Worktree
    /// synced to HEAD so subprocess `git revert` sees a clean tree.
    fn two_commit_repo() -> (TempDir, Repository, git2::Oid) {
        let (dir, repo) = make_repo();
        let base = commit_file(&repo, "Important change to x", &[], "f.txt", b"base\n");
        let v2 = commit_file(&repo, "change to v2", &[base], "f.txt", b"v2\n");
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .unwrap();
        (dir, repo, v2)
    }

    /// A repo where reverting the tip commit conflicts: the tip changes `f.txt`,
    /// then a later commit changes `f.txt` again, so reverting the middle commit
    /// can't apply cleanly. HEAD's tip is returned as the OID to revert.
    fn conflicting_revert_repo() -> (TempDir, Repository, git2::Oid) {
        let (dir, repo) = make_repo();
        let base = commit_file(&repo, "base", &[], "f.txt", b"base\n");
        let mid = commit_file(&repo, "mid change to v2", &[base], "f.txt", b"v2\n");
        // A later commit rewrites the same file so reverting `mid` conflicts.
        commit_file(&repo, "later change to v3", &[mid], "f.txt", b"v3\n");
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .unwrap();
        (dir, repo, mid)
    }

    #[test]
    fn revert_commit_begin_returns_default_message_with_full_oid() {
        let (dir, _repo, oid) = two_commit_repo();
        let oid_str = oid.to_string();
        let map = state_map_for(&dir);
        let descriptors = HashMap::new();
        let result =
            revert_commit_begin_inner(&path_str(&dir), &oid_str, &map, &descriptors).unwrap();
        let message = result.message.expect("clean revert must carry a message");
        assert!(
            message.starts_with("Revert \"change to v2\""),
            "got: {message:?}"
        );
        assert!(
            message.contains(&format!("This reverts commit {}.", oid_str)),
            "message must carry the FULL 40-char OID, not a short OID; got: {message:?}"
        );
        assert_eq!(
            oid_str.len(),
            40,
            "OID under test must be the full 40-char form"
        );
        assert!(
            revert_head_path(&dir).exists(),
            "begin sets REVERT_HEAD (not committed yet)"
        );
    }

    #[test]
    fn revert_continue_clears_revert_head_and_commits_edited_body() {
        let (dir, repo, oid) = two_commit_repo();
        let oid_str = oid.to_string();
        let map = state_map_for(&dir);
        let descriptors = HashMap::new();
        revert_commit_begin_inner(&path_str(&dir), &oid_str, &map, &descriptors).unwrap();

        let edited = "Revert \"change to v2\"\n\nedited body";
        revert_continue_inner(&path_str(&dir), edited, &map, &descriptors).unwrap();

        assert!(
            !revert_head_path(&dir).exists(),
            "git commit -m clears REVERT_HEAD"
        );
        let body = head_body(&repo);
        // git appends a trailing newline; the body is otherwise the edited text.
        assert_eq!(
            body.trim_end(),
            edited,
            "HEAD body must equal the edited message"
        );
        assert!(
            !body.lines().any(|l| l.starts_with('#')),
            "no commit body line may start with #; got: {body:?}"
        );
    }

    #[test]
    fn revert_commit_begin_conflict_returns_conflict_state_err() {
        let (dir, _repo, oid) = conflicting_revert_repo();
        let oid_str = oid.to_string();
        let map = state_map_for(&dir);
        let descriptors = HashMap::new();
        let err = revert_commit_begin_inner(&path_str(&dir), &oid_str, &map, &descriptors)
            .expect_err("conflicted revert must return Err, never open the editor");
        assert_eq!(err.code, "conflict_state");
    }

    #[test]
    fn revert_abort_recovers_clean_tree() {
        let (dir, repo, oid) = two_commit_repo();
        let oid_str = oid.to_string();
        let map = state_map_for(&dir);
        let descriptors = HashMap::new();
        revert_commit_begin_inner(&path_str(&dir), &oid_str, &map, &descriptors).unwrap();
        assert!(
            revert_head_path(&dir).exists(),
            "precondition: begin set REVERT_HEAD"
        );

        revert_abort_inner(&path_str(&dir), &map, &descriptors).unwrap();

        assert!(
            !revert_head_path(&dir).exists(),
            "revert --abort clears REVERT_HEAD"
        );
        assert!(
            !crate::git::repository::is_repo_dirty(&repo).unwrap(),
            "revert --abort must leave a clean tree"
        );
    }

    #[test]
    fn revert_continue_strips_conflict_comment_block_from_commit_body() {
        let (dir, repo, oid) = conflicting_revert_repo();
        let oid_str = oid.to_string();
        let map = state_map_for(&dir);
        let descriptors = HashMap::new();
        // Conflicted begin leaves REVERT_HEAD set; resolve by staging a fix.
        let _ = revert_commit_begin_inner(&path_str(&dir), &oid_str, &map, &descriptors);
        let blob = repo.blob(b"resolved\n").unwrap();
        let mut index = repo.index().unwrap();
        index
            .add(&git2::IndexEntry {
                ctime: git2::IndexTime::new(0, 0),
                mtime: git2::IndexTime::new(0, 0),
                dev: 0,
                ino: 0,
                mode: 0o100644,
                uid: 0,
                gid: 0,
                file_size: 0,
                id: blob,
                flags: 0,
                flags_extended: 0,
                path: b"f.txt".to_vec(),
            })
            .unwrap();
        index.write().unwrap();

        // Finish with a message that carries a trailing `# Conflicts:` block.
        let msg = "Revert \"mid change to v2\"\n\n# Conflicts:\n#\tf.txt";
        revert_continue_inner(&path_str(&dir), msg, &map, &descriptors).unwrap();

        let body = head_body(&repo);
        assert!(
            !body.lines().any(|l| l.starts_with('#')),
            "--cleanup=strip must remove the # Conflicts: block; got: {body:?}"
        );
    }
}
