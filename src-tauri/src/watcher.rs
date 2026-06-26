use notify_debouncer_mini::notify::RecommendedWatcher;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult, Debouncer};
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};

pub enum WatchHandle {
    Native(Debouncer<RecommendedWatcher>),
    #[cfg(target_os = "windows")]
    WslPoller(mpsc::Sender<()>),
}

pub type WatcherMap = HashMap<String, WatchHandle>;
pub struct WatcherState(pub Mutex<WatcherMap>);

impl Default for WatcherState {
    fn default() -> Self {
        WatcherState(Mutex::new(HashMap::new()))
    }
}

pub fn start_watcher<R: Runtime>(path: PathBuf, app: AppHandle<R>, state: &WatcherState) {
    let repo_id = path.to_string_lossy().to_string();
    start_watcher_for_repo(path, repo_id, app, state);
}

pub fn start_watcher_for_repo<R: Runtime>(
    path: PathBuf,
    repo_id: String,
    app: AppHandle<R>,
    state: &WatcherState,
) {
    let event_repo_id = repo_id.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        move |res: DebounceEventResult| {
            if res.is_ok() {
                let _ = app.emit("repo-changed", event_repo_id.clone());
            }
        },
    )
    .expect("failed to create debouncer");

    debouncer
        .watcher()
        .watch(&path, RecursiveMode::Recursive)
        .expect("failed to watch path");

    state
        .0
        .lock()
        .unwrap()
        .insert(repo_id, WatchHandle::Native(debouncer));
}

#[cfg(target_os = "windows")]
pub fn start_wsl_poller_for_repo<R: Runtime>(
    repo: crate::git::types::RepoDescriptor,
    app: AppHandle<R>,
    state: &WatcherState,
) {
    let repo_id = repo.id.clone();
    let event_repo_id = repo_id.clone();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut last = crate::git::backend_fs::wsl_poll_token(&repo)
            .ok()
            .flatten()
            .unwrap_or_default();
        loop {
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(_) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            let Ok(Some(next)) = crate::git::backend_fs::wsl_poll_token(&repo) else {
                continue;
            };
            if next != last {
                last = next;
                let _ = app.emit("repo-changed", event_repo_id.clone());
            }
        }
    });

    state
        .0
        .lock()
        .unwrap()
        .insert(repo_id, WatchHandle::WslPoller(tx));
}

#[cfg(not(target_os = "windows"))]
pub fn start_wsl_poller_for_repo<R: Runtime>(
    _repo: crate::git::types::RepoDescriptor,
    _app: AppHandle<R>,
    _state: &WatcherState,
) {
}

pub fn stop_watcher(repo_id: &str, state: &WatcherState) {
    #[cfg(target_os = "windows")]
    if let Some(WatchHandle::WslPoller(stop)) = state.0.lock().unwrap().remove(repo_id) {
        let _ = stop.send(());
        return;
    }
    let _ = state.0.lock().unwrap().remove(repo_id);
}
