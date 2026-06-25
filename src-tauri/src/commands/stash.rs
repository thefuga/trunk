use crate::error::TrunkError;
use crate::git::{
    command_runner, graph, read_model,
    types::{GraphResult, StashEntry},
};
use crate::state::{CommitCache, RepoState};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn git_command_error(code: &str, output: std::process::Output) -> TrunkError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    TrunkError::new(code, if stderr.is_empty() { stdout } else { stderr })
}

fn run_git(repo: &crate::git::types::RepoDescriptor, args: &[&str]) -> Result<(), TrunkError> {
    let output = command_runner::git_output(repo, args, "git_error")?;
    if output.status.success() {
        Ok(())
    } else {
        Err(git_command_error("git_error", output))
    }
}

fn wsl_has_conflicts(repo: &crate::git::types::RepoDescriptor) -> Result<bool, TrunkError> {
    let output = command_runner::git_output(repo, &["ls-files", "-u"], "git_error")?;
    if output.status.success() {
        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    } else {
        Err(git_command_error("git_error", output))
    }
}

fn refresh_graph_for_backend(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, crate::git::types::RepoDescriptor>,
) -> Result<GraphResult, TrunkError> {
    match read_model::backend_from_state(path, state_map, descriptor_map)? {
        read_model::ReadBackend::Local(path_buf) => {
            let mut repo = git2::Repository::open(path_buf)?;
            graph::walk_commits(&mut repo, 0, usize::MAX)
        }
        read_model::ReadBackend::Wsl(repo) => read_model::wsl_commit_graph(&repo),
    }
}

fn wsl_list_stashes(
    repo: &crate::git::types::RepoDescriptor,
) -> Result<Vec<StashEntry>, TrunkError> {
    Ok(read_model::wsl_refs(repo)?.stashes)
}

fn wsl_stash_save(
    repo: &crate::git::types::RepoDescriptor,
    message: &str,
) -> Result<(), TrunkError> {
    let status = command_runner::git_output(repo, &["status", "--porcelain=v1"], "git_error")?;
    if !status.status.success() {
        return Err(git_command_error("git_error", status));
    }
    if String::from_utf8_lossy(&status.stdout).trim().is_empty() {
        return Err(TrunkError::new(
            "nothing_to_stash",
            "Nothing to stash — working tree is clean",
        ));
    }

    if message.trim().is_empty() {
        run_git(repo, &["stash", "push"])
    } else {
        run_git(repo, &["stash", "push", "-m", message])
    }
}

fn wsl_stash_apply(
    repo: &crate::git::types::RepoDescriptor,
    index: usize,
) -> Result<(), TrunkError> {
    let stash_ref = format!("stash@{{{}}}", index);
    run_git(repo, &["stash", "apply", &stash_ref]).map_err(|e| {
        if wsl_has_conflicts(repo).unwrap_or(false) {
            TrunkError::new(
                "conflict_state",
                "Stash applied with conflicts — resolve conflicts before continuing",
            )
        } else {
            e
        }
    })?;
    if wsl_has_conflicts(repo)? {
        return Err(TrunkError::new(
            "conflict_state",
            "Stash applied with conflicts — resolve conflicts before continuing",
        ));
    }
    Ok(())
}

fn wsl_stash_pop(repo: &crate::git::types::RepoDescriptor, index: usize) -> Result<(), TrunkError> {
    let stash_ref = format!("stash@{{{}}}", index);
    run_git(repo, &["stash", "pop", &stash_ref]).map_err(|e| {
        if wsl_has_conflicts(repo).unwrap_or(false) {
            TrunkError::new("conflict_state", "Stash applied with conflicts — resolve conflicts before continuing. Note: stash was NOT removed.")
        } else {
            e
        }
    })?;
    if wsl_has_conflicts(repo)? {
        return Err(TrunkError::new("conflict_state", "Stash applied with conflicts — resolve conflicts before continuing. Note: stash was NOT removed."));
    }
    Ok(())
}

pub fn list_stashes_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<Vec<StashEntry>, TrunkError> {
    let mut repo = crate::commands::open_repo_from_state(path, state_map)?;
    let mut raw: Vec<(usize, String, git2::Oid)> = Vec::new();
    repo.stash_foreach(|idx, name, oid| {
        raw.push((idx, name.to_owned(), *oid));
        true
    })?;
    Ok(raw
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
        .collect())
}

pub fn stash_save_inner(
    path: &str,
    message: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let mut repo = crate::commands::open_repo_from_state(path, state_map)?;
    let sig = repo.signature().map_err(TrunkError::from)?;
    let msg = if message.trim().is_empty() {
        let branch = repo
            .head()
            .ok()
            .and_then(|h| h.shorthand().map(str::to_owned))
            .unwrap_or_else(|| "HEAD".to_owned());
        format!("WIP on {}", branch)
    } else {
        message.to_owned()
    };
    repo.stash_save(&sig, &msg, None).map_err(|e| {
        if e.message().contains("nothing to stash") {
            TrunkError::new(
                "nothing_to_stash",
                "Nothing to stash — working tree is clean",
            )
        } else {
            TrunkError::from(e)
        }
    })?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn stash_pop_inner(
    path: &str,
    index: usize,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let mut repo = crate::commands::open_repo_from_state(path, state_map)?;
    repo.stash_pop(index, None).map_err(|e| {
        if e.message().contains("conflict") || e.message().contains("merge") {
            TrunkError::new("conflict_state", "Stash applied with conflicts — resolve conflicts before continuing. Note: stash was NOT removed.")
        } else {
            TrunkError::from(e)
        }
    })?;
    // Check for post-apply conflicts (git2 may return Ok even with conflicts)
    {
        let statuses = repo.statuses(None).map_err(TrunkError::from)?;
        let has_conflicts = statuses
            .iter()
            .any(|s| s.status().contains(git2::Status::CONFLICTED));
        if has_conflicts {
            return Err(TrunkError::new("conflict_state", "Stash applied with conflicts — resolve conflicts before continuing. Note: stash was NOT removed."));
        }
    }
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn stash_apply_inner(
    path: &str,
    index: usize,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let mut repo = crate::commands::open_repo_from_state(path, state_map)?;
    repo.stash_apply(index, None).map_err(|e| {
        if e.message().contains("conflict") || e.message().contains("merge") {
            TrunkError::new(
                "conflict_state",
                "Stash applied with conflicts — resolve conflicts before continuing",
            )
        } else {
            TrunkError::from(e)
        }
    })?;
    {
        let statuses = repo.statuses(None).map_err(TrunkError::from)?;
        let has_conflicts = statuses
            .iter()
            .any(|s| s.status().contains(git2::Status::CONFLICTED));
        if has_conflicts {
            return Err(TrunkError::new(
                "conflict_state",
                "Stash applied with conflicts — resolve conflicts before continuing",
            ));
        }
    }
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

pub fn stash_drop_inner(
    path: &str,
    index: usize,
    state_map: &HashMap<String, PathBuf>,
) -> Result<GraphResult, TrunkError> {
    let mut repo = crate::commands::open_repo_from_state(path, state_map)?;
    repo.stash_drop(index).map_err(TrunkError::from)?;
    graph::walk_commits(&mut repo, 0, usize::MAX)
}

#[tauri::command]
pub async fn list_stashes(
    path: String,
    state: State<'_, RepoState>,
) -> Result<Vec<StashEntry>, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        match read_model::backend_from_state(&path, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => list_stashes_inner(&path, &state_map),
            read_model::ReadBackend::Wsl(repo) => wsl_list_stashes(&repo),
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())
}

#[tauri::command]
pub async fn stash_save(
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
        match read_model::backend_from_state(&path_clone, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => {
                stash_save_inner(&path_clone, &message, &state_map)
            }
            read_model::ReadBackend::Wsl(repo) => {
                wsl_stash_save(&repo, &message)?;
                refresh_graph_for_backend(&path_clone, &state_map, &descriptor_map)
            }
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn stash_pop(
    path: String,
    index: usize,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        match read_model::backend_from_state(&path_clone, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => stash_pop_inner(&path_clone, index, &state_map),
            read_model::ReadBackend::Wsl(repo) => {
                wsl_stash_pop(&repo, index)?;
                refresh_graph_for_backend(&path_clone, &state_map, &descriptor_map)
            }
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn stash_apply(
    path: String,
    index: usize,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        match read_model::backend_from_state(&path_clone, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => stash_apply_inner(&path_clone, index, &state_map),
            read_model::ReadBackend::Wsl(repo) => {
                wsl_stash_apply(&repo, index)?;
                refresh_graph_for_backend(&path_clone, &state_map, &descriptor_map)
            }
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn stash_drop(
    path: String,
    index: usize,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        match read_model::backend_from_state(&path_clone, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => stash_drop_inner(&path_clone, index, &state_map),
            read_model::ReadBackend::Wsl(repo) => {
                let stash_ref = format!("stash@{{{}}}", index);
                run_git(&repo, &["stash", "drop", &stash_ref])?;
                refresh_graph_for_backend(&path_clone, &state_map, &descriptor_map)
            }
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}
