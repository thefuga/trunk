use crate::error::TrunkError;
use crate::git::types::GraphResult;
use crate::git::{backend, command_runner};
use crate::git::{
    graph,
    types::{BranchInfo, RefLabel, RefType, RefsResponse, StashEntry},
};
use crate::state::{CommitCache, RepoState};
use git2::BranchType;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[cfg(target_os = "windows")]
fn git_command_error(code: &str, output: std::process::Output) -> TrunkError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    TrunkError::new(code, if stderr.is_empty() { stdout } else { stderr })
}

#[cfg(target_os = "windows")]
pub(crate) fn run_git(
    repo: &crate::git::types::RepoDescriptor,
    args: &[&str],
) -> Result<(), TrunkError> {
    let output = command_runner::git_output(repo, args, "git_error")?;
    if output.status.success() {
        Ok(())
    } else {
        Err(git_command_error("git_error", output))
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn git_stdout(
    repo: &crate::git::types::RepoDescriptor,
    args: &[&str],
) -> Result<String, TrunkError> {
    let output = command_runner::git_output(repo, args, "git_error")?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(git_command_error("git_error", output))
    }
}

/// Inner implementation of list_refs — separated for testability without Tauri state.
pub fn list_refs_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<RefsResponse, TrunkError> {
    let mut repo = crate::commands::open_repo_from_state(path, state_map)?;

    // Resolve HEAD name before any mutable borrows
    let head_name: Option<String> = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(str::to_owned));

    let local: Vec<BranchInfo> = repo
        .branches(Some(BranchType::Local))?
        .filter_map(|b| b.ok())
        .map(|(branch, _)| {
            let name = branch.name().ok().flatten().unwrap_or("").to_owned();
            let is_head = head_name.as_deref() == Some(name.as_str());
            let upstream = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(str::to_owned));
            let last_commit_timestamp = branch
                .get()
                .peel_to_commit()
                .map(|c| c.author().when().seconds())
                .unwrap_or(0);
            let (ahead, behind) = match (&upstream, branch.get().target()) {
                (Some(_), Some(local_oid)) => branch
                    .upstream()
                    .ok()
                    .and_then(|ub| ub.get().target())
                    .map(|remote_oid| {
                        repo.graph_ahead_behind(local_oid, remote_oid)
                            .unwrap_or((0, 0))
                    })
                    .unwrap_or((0, 0)),
                _ => (0, 0),
            };
            BranchInfo {
                name,
                is_head,
                upstream,
                ahead,
                behind,
                last_commit_timestamp,
            }
        })
        .collect();

    // Remote branches — filter out entries where name ends with "/HEAD"
    let remote: Vec<BranchInfo> = repo
        .branches(Some(BranchType::Remote))?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| {
            let name = branch.name().ok().flatten()?.to_owned();
            if name.ends_with("/HEAD") {
                return None;
            }
            Some(BranchInfo {
                name,
                is_head: false,
                upstream: None,
                ahead: 0,
                behind: 0,
                last_commit_timestamp: 0,
            })
        })
        .collect();

    // Tags
    let mut tags: Vec<RefLabel> = Vec::new();
    repo.tag_foreach(|_oid, name_bytes| {
        let name = std::str::from_utf8(name_bytes).unwrap_or("").to_owned();
        let short_name = name.strip_prefix("refs/tags/").unwrap_or(&name).to_owned();
        tags.push(RefLabel {
            name,
            short_name,
            ref_type: RefType::Tag,
            is_head: false,
            color_index: 0,
        });
        true
    })?;

    // Stashes — requires &mut repo
    // Collect raw OIDs first (foreach holds mutable borrow), then resolve parents in second pass
    let mut raw_stashes: Vec<(usize, String, git2::Oid)> = Vec::new();
    repo.stash_foreach(|idx, name, oid| {
        raw_stashes.push((idx, name.to_owned(), *oid));
        true
    })?;
    let stashes: Vec<StashEntry> = raw_stashes
        .into_iter()
        .map(|(idx, name, stash_oid)| {
            let parent_oid = repo
                .find_commit(stash_oid)
                .ok()
                .and_then(|c| c.parent_id(0).ok())
                .map(|o| o.to_string());
            StashEntry {
                index: idx,
                short_name: format!("stash@{{{}}}", idx),
                name,
                oid: stash_oid.to_string(),
                parent_oid,
            }
        })
        .collect();

    Ok(RefsResponse {
        local,
        remote,
        tags,
        stashes,
    })
}

/// Delete a local branch. Rejects deletion of the currently checked-out (HEAD) branch.
pub fn delete_branch_inner(
    path: &str,
    branch_name: &str,
    state_map: &HashMap<String, PathBuf>,
    cache_map: &mut HashMap<String, GraphResult>,
) -> Result<(), TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;

    // Check if this is the HEAD branch
    let head_name = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(str::to_owned));
    if head_name.as_deref() == Some(branch_name) {
        return Err(TrunkError::new(
            "cannot_delete_head",
            "Cannot delete the currently checked-out branch",
        ));
    }

    let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
    branch.delete()?;
    drop(branch);
    drop(repo);

    // Rebuild graph cache
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    let graph_result = graph::walk_commits(&mut repo2, 0, usize::MAX)?;
    cache_map.insert(path.to_owned(), graph_result);

    Ok(())
}

/// Rename a local branch. Fails if `new_name` already exists.
pub fn rename_branch_inner(
    path: &str,
    old_name: &str,
    new_name: &str,
    state_map: &HashMap<String, PathBuf>,
    cache_map: &mut HashMap<String, GraphResult>,
) -> Result<(), TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let mut branch = repo.find_branch(old_name, BranchType::Local)?;
    branch.rename(new_name, false)?; // false = no force (fail if new_name exists)
    drop(branch);
    drop(repo);

    // Rebuild graph cache
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    let graph_result = graph::walk_commits(&mut repo2, 0, usize::MAX)?;
    cache_map.insert(path.to_owned(), graph_result);

    Ok(())
}

#[tauri::command]
pub async fn list_refs(path: String, state: State<'_, RepoState>) -> Result<RefsResponse, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let descriptor =
            crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)?;
        backend::resolve_backend(descriptor)?.refs(&path, &state_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())
}

/// Inner implementation of resolve_ref — separated for testability.
pub fn resolve_ref_inner(
    path: &str,
    ref_name: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<String, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let obj = repo.revparse_single(ref_name).map_err(TrunkError::from)?;
    let commit = obj.peel_to_commit().map_err(TrunkError::from)?;
    Ok(commit.id().to_string())
}

#[tauri::command]
pub async fn resolve_ref(
    path: String,
    ref_name: String,
    state: State<'_, RepoState>,
) -> Result<String, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let descriptor =
            crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)?;
        backend::resolve_backend(descriptor)?.resolve_ref(&path, &ref_name, &state_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())
}

/// Inner implementation of checkout_branch — separated for testability.
pub fn checkout_branch_inner(
    path: &str,
    branch_name: &str,
    state_map: &HashMap<String, PathBuf>,
    cache_map: &mut HashMap<String, GraphResult>,
) -> Result<(), TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;

    let branch_ref = format!("refs/heads/{}", branch_name);
    {
        let (object, _reference) = repo.revparse_ext(&branch_ref)?;
        repo.checkout_tree(
            &object,
            Some(&mut git2::build::CheckoutBuilder::new().safe()),
        )?;
    }
    repo.set_head(&branch_ref)?;
    drop(repo);

    // Rebuild graph cache after checkout
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    let graph_result = graph::walk_commits(&mut repo2, 0, usize::MAX)?;
    cache_map.insert(path.to_owned(), graph_result);

    Ok(())
}

#[tauri::command]
pub async fn checkout_branch(
    path: String,
    branch_name: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let mut cache_map = cache.0.lock().unwrap().clone();

    let path_clone = path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let descriptor =
            crate::commands::repo_descriptor_from_state(&path_clone, &state_map, &descriptor_map)?;
        let graph = backend::resolve_backend(descriptor)?.checkout_branch(
            &path_clone,
            &branch_name,
            &state_map,
            &descriptor_map,
        )?;
        cache_map.insert(path_clone.clone(), graph);
        Ok::<_, TrunkError>(cache_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    // Update cache in main thread with rebuilt data
    *cache.0.lock().unwrap() = result;

    let _ = app.emit("repo-changed", path);

    Ok(())
}

pub fn fast_forward_to_inner(
    path: &str,
    target_oid: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, crate::git::types::RepoDescriptor>,
    cache_map: &mut HashMap<String, GraphResult>,
) -> Result<(), TrunkError> {
    let repo = crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    let output =
        command_runner::git_output(&repo, &["merge", "--ff-only", target_oid], "merge_error")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(TrunkError::new("not_fast_forward", stderr));
    }

    let graph = crate::commands::refresh_graph_from_state(path, state_map, descriptor_map)?;
    cache_map.insert(path.to_owned(), graph);

    Ok(())
}

#[tauri::command]
pub async fn fast_forward_to(
    path: String,
    target_oid: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let mut cache_map = cache.0.lock().unwrap().clone();

    let path_clone = path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let descriptor =
            crate::commands::repo_descriptor_from_state(&path_clone, &state_map, &descriptor_map)?;
        let graph = backend::resolve_backend(descriptor)?.fast_forward_to(
            &path_clone,
            &target_oid,
            &state_map,
            &descriptor_map,
        )?;
        cache_map.insert(path_clone.clone(), graph);
        Ok::<_, TrunkError>(cache_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    *cache.0.lock().unwrap() = result;
    let _ = app.emit("repo-changed", path);

    Ok(())
}

/// Inner implementation of create_branch — separated for testability.
/// When `from_oid` is Some, branches from that OID; when None, branches from HEAD.
/// Creates the branch first (always safe), then checks out. If dirty workdir at checkout time,
/// returns dirty_workdir error (branch exists but HEAD didn't move).
pub fn create_branch_inner(
    path: &str,
    name: &str,
    from_oid: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
    cache_map: &mut HashMap<String, GraphResult>,
) -> Result<(), TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;

    let target_oid = match from_oid {
        Some(oid_str) => repo.revparse_single(oid_str)?.id(),
        None => repo
            .head()?
            .target()
            .ok_or_else(|| TrunkError::new("git_error", "HEAD has no target (unborn branch?)"))?,
    };
    let target_commit = repo.find_commit(target_oid)?;
    // false = no force; fails if name already exists
    repo.branch(name, &target_commit, false)?;
    // Drop target_commit (and its borrow on repo) before mutable operations
    drop(target_commit);

    // Check dirty workdir before checkout (branch already created above)
    if crate::git::repository::is_repo_dirty(&repo)? {
        drop(repo);
        // Rebuild cache even though checkout didn't happen — branch was created
        let path_buf = state_map
            .get(path)
            .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
        let mut repo2 = git2::Repository::open(path_buf)?;
        let graph_result = graph::walk_commits(&mut repo2, 0, usize::MAX)?;
        cache_map.insert(path.to_owned(), graph_result);
        return Err(TrunkError::new(
            "dirty_workdir",
            "Branch created but working tree has uncommitted changes — checkout skipped",
        ));
    }

    // Auto-checkout the new branch (checkout_tree updates index + working tree, then set_head moves HEAD)
    let branch_ref = format!("refs/heads/{}", name);
    {
        let (object, _reference) = repo.revparse_ext(&branch_ref)?;
        repo.checkout_tree(
            &object,
            Some(&mut git2::build::CheckoutBuilder::new().safe()),
        )?;
    }
    repo.set_head(&branch_ref)?;
    drop(repo);

    // Rebuild graph cache after branch creation
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    let mut repo2 = git2::Repository::open(path_buf)?;
    let graph_result = graph::walk_commits(&mut repo2, 0, usize::MAX)?;
    cache_map.insert(path.to_owned(), graph_result);

    Ok(())
}

#[tauri::command]
pub async fn create_branch(
    path: String,
    name: String,
    from_oid: Option<String>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let mut cache_map = cache.0.lock().unwrap().clone();

    let path_clone = path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let descriptor =
            crate::commands::repo_descriptor_from_state(&path_clone, &state_map, &descriptor_map)?;
        let graph = backend::resolve_backend(descriptor)?.create_branch(
            &path_clone,
            &name,
            from_oid.as_deref(),
            &state_map,
            &descriptor_map,
        )?;
        cache_map.insert(path_clone.clone(), graph);
        Ok::<_, TrunkError>(cache_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    // Update cache in main thread with rebuilt data
    *cache.0.lock().unwrap() = result;

    let _ = app.emit("repo-changed", path);

    Ok(())
}

#[tauri::command]
pub async fn delete_branch(
    path: String,
    branch_name: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let mut cache_map = cache.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let descriptor =
            crate::commands::repo_descriptor_from_state(&path_clone, &state_map, &descriptor_map)?;
        let graph = backend::resolve_backend(descriptor)?.delete_branch(
            &path_clone,
            &branch_name,
            &state_map,
            &descriptor_map,
        )?;
        cache_map.insert(path_clone.clone(), graph);
        Ok::<_, TrunkError>(cache_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    *cache.0.lock().unwrap() = result;
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn rename_branch(
    path: String,
    old_name: String,
    new_name: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let mut cache_map = cache.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let descriptor =
            crate::commands::repo_descriptor_from_state(&path_clone, &state_map, &descriptor_map)?;
        let graph = backend::resolve_backend(descriptor)?.rename_branch(
            &path_clone,
            &old_name,
            &new_name,
            &state_map,
            &descriptor_map,
        )?;
        cache_map.insert(path_clone.clone(), graph);
        Ok::<_, TrunkError>(cache_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    *cache.0.lock().unwrap() = result;
    let _ = app.emit("repo-changed", path);
    Ok(())
}
