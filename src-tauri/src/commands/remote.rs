use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::error::TrunkError;
use crate::git::{
    backend, command_runner,
    types::{GraphResult, OperationType},
};
use crate::state::{kill_process, CommitCache, RepoState, RunningOp};

/// Classifies git stderr output into structured error codes.
pub fn classify_git_error(stderr: &str) -> TrunkError {
    let lower = stderr.to_lowercase();

    if lower.contains("authentication failed")
        || lower.contains("permission denied")
        || lower.contains("could not read from remote")
        || lower.contains("host key verification failed")
        || lower.contains("connection refused")
    {
        TrunkError::new("auth_failure", stderr)
    } else if lower.contains("non-fast-forward")
        || lower.contains("fetch first")
        || lower.contains("failed to push some refs")
    {
        TrunkError::new("non_fast_forward", stderr)
    } else if lower.contains("no upstream") || lower.contains("has no upstream branch") {
        TrunkError::new("no_upstream", stderr)
    } else {
        TrunkError::new("remote_error", stderr)
    }
}

/// Spawns a git subprocess with async stderr streaming and progress events.
///
/// Stores child PID in `running` for cancel support.
/// Emits `remote-progress` Tauri events per stderr line.
/// On failure, classifies the error using `classify_git_error`.
async fn run_git_remote(
    args: &[&str],
    repo: &crate::git::types::RepoDescriptor,
    app: &AppHandle,
    repo_id: &str,
    running: &Mutex<HashMap<String, u32>>,
) -> Result<(), TrunkError> {
    // Check mutual exclusion (per-repo)
    {
        let guard = running.lock().unwrap();
        if guard.contains_key(repo_id) {
            return Err(TrunkError::new(
                "op_in_progress",
                "A remote operation is already running for this repository",
            ));
        }
    }

    let mut child = command_runner::git_tokio_piped(repo, args)
        .spawn()
        .map_err(|e| TrunkError::new("remote_error", e.to_string()))?;

    // Store PID for cancel support keyed by stable repo id.
    if let Some(pid) = child.id() {
        let mut guard = running.lock().unwrap();
        guard.insert(repo_id.to_owned(), pid);
    }

    // Read stderr lines and emit progress events
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| TrunkError::new("remote_error", "Failed to capture stderr"))?;

    let mut reader = BufReader::new(stderr).lines();
    let mut collected_stderr = Vec::new();

    while let Ok(Some(line)) = reader.next_line().await {
        collected_stderr.push(line.clone());

        // Git progress uses \r for in-place updates; take the last segment
        let display = line
            .split('\r')
            .rfind(|s| !s.trim().is_empty())
            .unwrap_or("")
            .trim();

        if !display.is_empty() {
            let _ = app.emit(
                "remote-progress",
                serde_json::json!({"path": repo_id, "repoId": repo_id, "line": display}),
            );
        }
    }

    let status = child
        .wait()
        .await
        .map_err(|e| TrunkError::new("remote_error", e.to_string()))?;

    // Clear RunningOp for this repo regardless of outcome
    {
        let mut guard = running.lock().unwrap();
        guard.remove(repo_id);
    }

    if !status.success() {
        let full_stderr = collected_stderr.join("\n");
        return Err(classify_git_error(&full_stderr));
    }

    Ok(())
}

/// Rebuild the commit graph and update the cache after a successful remote operation.
async fn refresh_graph(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, crate::git::types::RepoDescriptor>,
    cache: &State<'_, CommitCache>,
    app: &AppHandle,
) -> Result<(), String> {
    let path_owned = path.to_owned();
    let path_for_refresh = path_owned.clone();
    let state_map = state_map.clone();
    let descriptor_map = descriptor_map.clone();
    let graph_result: GraphResult = tauri::async_runtime::spawn_blocking(move || {
        let descriptor = crate::commands::repo_descriptor_from_state(
            &path_for_refresh,
            &state_map,
            &descriptor_map,
        )?;
        backend::resolve_backend(descriptor)?.commit_graph(
            &path_for_refresh,
            &state_map,
            &descriptor_map,
        )
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache
        .0
        .lock()
        .unwrap()
        .insert(path_owned.clone(), graph_result);
    let _ = app.emit("repo-changed", path_owned);
    Ok(())
}

#[tauri::command]
pub async fn git_fetch(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    running: State<'_, RunningOp>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let repo = crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)
        .map_err(|e| e.to_json())?;

    run_git_remote(
        &["fetch", "--all", "--progress"],
        &repo,
        &app,
        &path,
        &running.0,
    )
    .await
    .map_err(|e| e.to_json())?;

    refresh_graph(&path, &state_map, &descriptor_map, &cache, &app).await
}

/// Silent periodic fetch. Best-effort: skips when the repo is mid-operation
/// (rebase/merge/cherry-pick/revert) or another remote op is already running,
/// and swallows any error so the UI never surfaces a popup or toast.
#[tauri::command]
pub async fn git_fetch_background(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    running: State<'_, RunningOp>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let Ok(repo) = crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)
    else {
        return Ok(());
    };

    let path_for_state = path.clone();
    let state_map_for_state = state_map.clone();
    let descriptor_map_for_state = descriptor_map.clone();
    let is_clean = tauri::async_runtime::spawn_blocking(move || {
        let Ok(descriptor) = crate::commands::repo_descriptor_from_state(
            &path_for_state,
            &state_map_for_state,
            &descriptor_map_for_state,
        ) else {
            return false;
        };
        backend::resolve_backend(descriptor)
            .and_then(|backend| backend.operation_state(&path_for_state, &state_map_for_state))
            .map(|info| matches!(info.op_type, OperationType::None))
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false);
    if !is_clean {
        return Ok(());
    }

    if run_git_remote(
        &["fetch", "--all", "--tags", "--prune", "--progress"],
        &repo,
        &app,
        &path,
        &running.0,
    )
    .await
    .is_err()
    {
        return Ok(());
    }

    let _ = refresh_graph(&path, &state_map, &descriptor_map, &cache, &app).await;
    Ok(())
}

#[tauri::command]
pub async fn git_pull(
    path: String,
    strategy: Option<String>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    running: State<'_, RunningOp>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let repo = crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)
        .map_err(|e| e.to_json())?;

    let args: Vec<&str> = match strategy.as_deref() {
        Some("ff") => vec!["pull", "--ff", "--progress"],
        Some("ff-only") => vec!["pull", "--ff-only", "--progress"],
        Some("rebase") => vec!["pull", "--rebase", "--progress"],
        _ => vec!["pull", "--progress"],
    };

    run_git_remote(&args, &repo, &app, &path, &running.0)
        .await
        .map_err(|e| e.to_json())?;

    refresh_graph(&path, &state_map, &descriptor_map, &cache, &app).await
}

#[tauri::command]
pub async fn git_push(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    running: State<'_, RunningOp>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let repo = crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)
        .map_err(|e| e.to_json())?;

    run_git_remote(&["push", "--progress"], &repo, &app, &path, &running.0)
        .await
        .map_err(|e| e.to_json())?;

    refresh_graph(&path, &state_map, &descriptor_map, &cache, &app).await
}

#[tauri::command]
pub async fn delete_remote_branch(
    path: String,
    branch_name: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    running: State<'_, RunningOp>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let repo = crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)
        .map_err(|e| e.to_json())?;

    // Parse "origin/feature" into remote="origin", branch="feature"
    let slash = branch_name.find('/').ok_or_else(|| {
        TrunkError::new(
            "invalid_ref",
            format!("Invalid remote branch name: {}", branch_name),
        )
        .to_json()
    })?;
    let remote = &branch_name[..slash];
    let branch = &branch_name[slash + 1..];

    run_git_remote(
        &["push", "--delete", "--progress", remote, branch],
        &repo,
        &app,
        &path,
        &running.0,
    )
    .await
    .map_err(|e| e.to_json())?;

    refresh_graph(&path, &state_map, &descriptor_map, &cache, &app).await
}

#[tauri::command]
pub async fn cancel_remote_op(path: String, running: State<'_, RunningOp>) -> Result<(), String> {
    let mut guard = running.0.lock().unwrap();
    if let Some(pid) = guard.remove(&path) {
        kill_process(pid);
    }
    Ok(())
}
