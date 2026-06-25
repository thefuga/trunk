use crate::error::TrunkError;
use crate::git::{command_runner, graph, read_model, types::HeadCommitMessage};
use crate::state::{CommitCache, RepoState};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

fn refresh_commit_cache(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, crate::git::types::RepoDescriptor>,
) -> Result<crate::git::types::GraphResult, TrunkError> {
    match read_model::backend_from_state(path, state_map, descriptor_map)? {
        read_model::ReadBackend::Local(path_buf) => {
            let mut repo = git2::Repository::open(path_buf).map_err(TrunkError::from)?;
            graph::walk_commits(&mut repo, 0, usize::MAX)
        }
        read_model::ReadBackend::Wsl(repo) => read_model::wsl_commit_graph(&repo),
    }
}

fn build_message(subject: &str, body: Option<&str>) -> String {
    match body {
        Some(b) if !b.trim().is_empty() => format!("{}\n\n{}", subject, b),
        _ => subject.to_owned(),
    }
}

pub fn create_commit_inner(
    path: &str,
    subject: &str,
    body: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let sig = repo.signature()?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let message = build_message(subject, body);

    let parents = match repo.head() {
        Ok(h) => vec![h.peel_to_commit()?],
        Err(e) if e.code() == git2::ErrorCode::UnbornBranch => vec![],
        Err(e) => return Err(TrunkError::from(e)),
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &parent_refs)?;
    Ok(())
}

pub fn amend_commit_inner(
    path: &str,
    subject: &str,
    body: Option<&str>,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let head_commit = repo.head()?.peel_to_commit()?;
    let sig = repo.signature()?;
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let message = build_message(subject, body);

    head_commit.amend(
        Some("HEAD"),
        Some(&sig),
        Some(&sig),
        None,
        Some(&message),
        Some(&tree),
    )?;
    Ok(())
}

pub fn get_head_commit_message_inner(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<HeadCommitMessage, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let commit = repo.head()?.peel_to_commit()?;
    Ok(HeadCommitMessage {
        subject: commit.summary().unwrap_or("").to_owned(),
        body: commit.body().map(str::to_owned),
    })
}

fn git_write(
    repo: &crate::git::types::RepoDescriptor,
    args: &[String],
    code: &str,
) -> Result<(), TrunkError> {
    let output = command_runner::git_output_owned(repo, args, code)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(TrunkError::new(
        code,
        if stderr.is_empty() {
            "Git commit operation failed".to_string()
        } else {
            stderr
        },
    ))
}

fn wsl_create_commit_inner(
    repo: &crate::git::types::RepoDescriptor,
    subject: &str,
    body: Option<&str>,
) -> Result<(), TrunkError> {
    let mut args = vec!["commit".to_string(), "-m".to_string(), subject.to_string()];
    if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
        args.push("-m".to_string());
        args.push(body.to_string());
    }
    git_write(repo, &args, "commit_error")
}

fn wsl_amend_commit_inner(
    repo: &crate::git::types::RepoDescriptor,
    subject: &str,
    body: Option<&str>,
) -> Result<(), TrunkError> {
    let mut args = vec![
        "commit".to_string(),
        "--amend".to_string(),
        "-m".to_string(),
        subject.to_string(),
    ];
    if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
        args.push("-m".to_string());
        args.push(body.to_string());
    }
    git_write(repo, &args, "commit_error")
}

fn wsl_get_head_commit_message_inner(
    repo: &crate::git::types::RepoDescriptor,
) -> Result<HeadCommitMessage, TrunkError> {
    let output = command_runner::git_output(
        repo,
        &["log", "-1", "--format=%s%x00%b"],
        "commit_message_error",
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(TrunkError::new(
            "commit_message_error",
            if stderr.is_empty() {
                "Could not read HEAD commit message".to_string()
            } else {
                stderr
            },
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut parts = text.trim_end_matches('\n').splitn(2, '\0');
    Ok(HeadCommitMessage {
        subject: parts.next().unwrap_or("").to_string(),
        body: parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string),
    })
}

#[tauri::command]
pub async fn create_commit(
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
        match read_model::backend_from_state(&path_clone, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => {
                create_commit_inner(&path_clone, &subject, body.as_deref(), &state_map)?;
            }
            read_model::ReadBackend::Wsl(repo) => {
                wsl_create_commit_inner(&repo, &subject, body.as_deref())?;
            }
        }
        refresh_commit_cache(&path_clone, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn amend_commit(
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
        match read_model::backend_from_state(&path_clone, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => {
                amend_commit_inner(&path_clone, &subject, body.as_deref(), &state_map)?;
            }
            read_model::ReadBackend::Wsl(repo) => {
                wsl_amend_commit_inner(&repo, &subject, body.as_deref())?;
            }
        }
        refresh_commit_cache(&path_clone, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[tauri::command]
pub async fn get_head_commit_message(
    path: String,
    state: State<'_, RepoState>,
) -> Result<HeadCommitMessage, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        match read_model::backend_from_state(&path, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(_) => get_head_commit_message_inner(&path, &state_map),
            read_model::ReadBackend::Wsl(repo) => wsl_get_head_commit_message_inner(&repo),
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())
}
