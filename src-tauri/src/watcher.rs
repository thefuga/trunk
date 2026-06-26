use notify_debouncer_mini::notify::RecommendedWatcher;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult, Debouncer};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};

pub enum WatchHandle {
    Native(Debouncer<RecommendedWatcher>),
    Poller(mpsc::Sender<()>),
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

pub fn start_polling_watcher_for_repo<R, F>(
    repo_id: String,
    app: AppHandle<R>,
    state: &WatcherState,
    mut poll_token: F,
) where
    R: Runtime,
    F: FnMut() -> Option<String> + Send + 'static,
{
    let event_repo_id = repo_id.clone();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut last = poll_token().unwrap_or_default();
        loop {
            match rx.recv_timeout(Duration::from_secs(2)) {
                Ok(_) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            let Some(next) = poll_token() else {
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
        .insert(repo_id, WatchHandle::Poller(tx));
}

pub fn stop_watcher(repo_id: &str, state: &WatcherState) {
    if let Some(WatchHandle::Poller(stop)) = state.0.lock().unwrap().remove(repo_id) {
        let _ = stop.send(());
        return;
    }
    let _ = state.0.lock().unwrap().remove(repo_id);
}
