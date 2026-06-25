use crate::error::TrunkError;
use crate::git::{
    graph, repository,
    types::{RepoDescriptor, RepoLocator},
};
use crate::state::{kill_process, CommitCache, RepoState, ReviewSessionsState, RunningOp};
use crate::watcher::{self, WatcherState};
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
    let execution_path = match &descriptor.locator {
        RepoLocator::Local { path } => path.clone(),
        RepoLocator::Wsl { distro, linux_path } => {
            let validation =
                crate::commands::wsl::validate_repo_inner(distro.clone(), linux_path.clone())
                    .map_err(|e| e.to_json())?;
            descriptor = validation.descriptor;
            descriptor.id = descriptor.locator.stable_id();
            crate::commands::wsl::unc_path(&validation.distro, &validation.repo_root)
        }
    };
    let repo_key = descriptor.id.clone();
    let is_local_repo = matches!(descriptor.locator, RepoLocator::Local { .. });
    let path_clone = execution_path.clone();

    let result = tauri::async_runtime::spawn_blocking(
        move || -> Result<crate::git::types::GraphResult, TrunkError> {
            let path_buf = std::path::PathBuf::from(&path_clone);
            repository::validate_and_open(&path_buf)?;
            let mut repo = git2::Repository::open(&path_buf)?;
            graph::walk_commits(&mut repo, 0, usize::MAX)
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
    if is_local_repo {
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
