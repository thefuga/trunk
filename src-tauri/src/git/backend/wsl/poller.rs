use crate::git::types::RepoDescriptor;
use crate::watcher::{self, WatcherState};
use tauri::{AppHandle, Runtime};

pub fn start_for_repo<R: Runtime>(repo: RepoDescriptor, app: AppHandle<R>, state: &WatcherState) {
    let repo_id = repo.id.clone();
    watcher::start_polling_watcher_for_repo(repo_id, app, state, move || {
        crate::git::backend_fs::poll_token(&repo).ok().flatten()
    });
}
