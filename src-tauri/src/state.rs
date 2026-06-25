use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::git::types::RepoDescriptor;

// CRITICAL: Store PathBuf ONLY — git2::Repository is not Sync.
// Each Tauri command opens a fresh Repository::open(path) inside spawn_blocking.
// Storing Repository handles here would cause cargo build to fail with "not Sync".
// Keyed by stable repo id for descriptor-aware opens. Legacy local-only opens
// keep using their raw path as the key for backwards compatibility.
pub struct RepoState(
    pub Mutex<HashMap<String, PathBuf>>,
    pub Mutex<HashMap<String, RepoDescriptor>>,
);

impl Default for RepoState {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()), Mutex::new(HashMap::new()))
    }
}

/// Stores the PID of the currently running remote operation per repo.
/// Key: stable repo id or legacy repo path (String), Value: PID (u32).
/// Used for: (a) cancel button kills the subprocess, (b) mutual exclusion prevents
/// concurrent ops on the SAME repo.
pub struct RunningOp(pub Mutex<HashMap<String, u32>>);

/// Terminate a process by PID. Uses SIGTERM on Unix and taskkill on Windows.
pub fn kill_process(pid: u32) {
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output();
    }
}

// Caches the full commit graph per open repo id.
// Populated on open_repo, cleared on close_repo, sliced by get_commit_graph.
pub struct CommitCache(pub Mutex<HashMap<String, crate::git::types::GraphResult>>);

// In-memory cache of the active review session per open repo.
// Keyed by stable backend-aware repo id, so local and WSL repos do not depend on
// host canonicalization or UNC spellings. ReviewSession is owned plain data,
// satisfying the top-of-file "PathBuf/owned only — git2::Repository is not Sync"
// constraint.
pub struct ReviewSessionsState(pub Mutex<HashMap<String, crate::git::types::ReviewSession>>);
