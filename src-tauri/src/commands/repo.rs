use crate::error::TrunkError;
use crate::git::{backend, repository, types::RepoDescriptor};
use crate::state::{kill_process, CommitCache, RepoState, ReviewSessionsState, RunningOp};
use crate::watcher::{self, WatcherState};
use std::collections::HashMap;
use tauri::{AppHandle, State};

/// Drop ONLY the in-memory session entry for `repo_id`. The file on disk is left
/// untouched so resume works on reopen — only `end_review_session` hard-deletes.
fn drop_in_memory_session(repo_id: &str, sessions: &State<'_, ReviewSessionsState>) {
    sessions.0.lock().unwrap().remove(repo_id);
}

#[tauri::command]
pub async fn open_repo(
    path: String,
    repo: Option<RepoDescriptor>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut descriptor = repo.unwrap_or_else(|| RepoDescriptor::local(path.clone()));
    descriptor.id = descriptor.locator.stable_id();
    let prepared = backend::resolve_backend(descriptor.clone())
        .and_then(|backend| backend.prepare_open_repo(descriptor))
        .map_err(|e| e.to_json())?;
    descriptor = prepared.descriptor;
    let execution_path = prepared.execution_path;
    let use_native_watcher = prepared.use_native_watcher;
    let repo_key = descriptor.id.clone();
    let descriptor_for_graph = descriptor.clone();
    let path_clone = execution_path.clone();
    let validate_native_path = use_native_watcher;

    let result = tauri::async_runtime::spawn_blocking(
        move || -> Result<crate::git::types::GraphResult, TrunkError> {
            if validate_native_path {
                let path_buf = std::path::PathBuf::from(&path_clone);
                repository::validate_and_open(&path_buf)?;
            }
            backend::resolve_backend(descriptor_for_graph)?.commit_graph(
                &path_clone,
                &HashMap::from([(path_clone.clone(), std::path::PathBuf::from(&path_clone))]),
                &HashMap::new(),
            )
        },
    )
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    let path_buf = std::path::PathBuf::from(&execution_path);
    state
        .0
        .lock()
        .unwrap()
        .insert(repo_key.clone(), path_buf.clone());
    state
        .1
        .lock()
        .unwrap()
        .insert(repo_key.clone(), descriptor.clone());
    cache.0.lock().unwrap().insert(repo_key.clone(), result);
    if use_native_watcher {
        watcher::start_watcher_for_repo(path_buf, repo_key, app, &watcher_state);
    } else {
        watcher::start_wsl_poller_for_repo(descriptor, app, &watcher_state);
    }

    Ok(())
}

#[tauri::command]
pub async fn close_repo(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
    sessions: State<'_, ReviewSessionsState>,
) -> Result<(), String> {
    let execution_path = state.0.lock().unwrap().remove(&path);
    state.1.lock().unwrap().remove(&path);
    cache.0.lock().unwrap().remove(&path);
    watcher::stop_watcher(&path, &watcher_state);
    let _ = execution_path;
    drop_in_memory_session(&path, &sessions);
    Ok(())
}

#[tauri::command]
pub async fn force_close_repo(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    watcher_state: State<'_, WatcherState>,
    running: State<'_, RunningOp>,
    sessions: State<'_, ReviewSessionsState>,
) -> Result<(), String> {
    // Cancel running remote op first (D-03)
    {
        let mut guard = running.0.lock().unwrap();
        if let Some(pid) = guard.remove(&path) {
            kill_process(pid);
        }
    }
    // Then clean up all other state (same as close_repo)
    let execution_path = state.0.lock().unwrap().remove(&path);
    state.1.lock().unwrap().remove(&path);
    cache.0.lock().unwrap().remove(&path);
    watcher::stop_watcher(&path, &watcher_state);
    let _ = execution_path;
    drop_in_memory_session(&path, &sessions);
    Ok(())
}
