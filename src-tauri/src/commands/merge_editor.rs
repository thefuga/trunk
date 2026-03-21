use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use crate::error::TrunkError;
use crate::git::types::MergeSides;
use crate::state::{CommitCache, RepoState};
use crate::git::graph;

fn open_repo_from_state(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

pub fn get_merge_sides_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<MergeSides, TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let index = repo.index()?;

    // Find the conflict entry for this file by iterating all conflicts
    let mut conflicts = index.conflicts()
        .map_err(|e| TrunkError::new("conflict_error", e.to_string()))?;

    let conflict = conflicts
        .find(|entry| {
            if let Ok(ref c) = entry {
                let entry_path = c.our.as_ref()
                    .or(c.their.as_ref())
                    .or(c.ancestor.as_ref())
                    .map(|e| String::from_utf8_lossy(&e.path).into_owned());
                entry_path.as_deref() == Some(file_path)
            } else {
                false
            }
        })
        .ok_or_else(|| TrunkError::new("not_conflicted", format!("File not in conflict: {}", file_path)))?
        .map_err(|e| TrunkError::new("conflict_error", e.to_string()))?;

    let read_blob = |entry: &Option<git2::IndexEntry>| -> Result<String, TrunkError> {
        match entry {
            Some(e) => {
                let blob = repo.find_blob(e.id)?;
                Ok(String::from_utf8_lossy(blob.content()).into_owned())
            }
            None => Ok(String::new()),
        }
    };

    Ok(MergeSides {
        base: read_blob(&conflict.ancestor)?,
        ours: read_blob(&conflict.our)?,
        theirs: read_blob(&conflict.their)?,
    })
}

pub fn save_merge_result_inner(
    path: &str,
    file_path: &str,
    content: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let repo_path = repo.workdir()
        .ok_or_else(|| TrunkError::new("no_workdir", "Bare repository"))?;

    // Write merged content to disk
    let full_path = repo_path.join(file_path);
    std::fs::write(&full_path, content)
        .map_err(|e| TrunkError::new("write_error", e.to_string()))?;

    // Stage the file (clears conflict entry from index)
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new(file_path))?;
    index.write()?;

    Ok(())
}

// --- Tauri command wrappers ---

#[tauri::command]
pub async fn get_merge_sides(
    path: String,
    file_path: String,
    state: State<'_, RepoState>,
) -> Result<MergeSides, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        get_merge_sides_inner(&path, &file_path, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn save_merge_result(
    path: String,
    file_path: String,
    content: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();
    let state_map_clone = state_map.clone();
    tauri::async_runtime::spawn_blocking(move || {
        save_merge_result_inner(&path_clone, &file_path, &content, &state_map_clone)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    // Repopulate cache and emit repo-changed (same pattern as merge_continue)
    let path_for_cache = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        let path_buf = state_map.get(&path_for_cache)
            .ok_or_else(|| TrunkError::new("not_open", "Repository not open"))?;
        let mut repo = git2::Repository::open(path_buf)?;
        graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create a repo with two branches that conflict on the same file.
    /// Returns (TempDir, state_map, path_string).
    fn make_conflicted_repo() -> (TempDir, HashMap<String, PathBuf>, String) {
        let dir = TempDir::new().unwrap();
        let path_str = dir.path().to_str().unwrap().to_owned();

        {
            let repo = git2::Repository::init(dir.path()).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
            drop(config);

            let sig = repo.signature().unwrap();

            // Initial commit with file.txt = "hello"
            fs::write(dir.path().join("file.txt"), "hello").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let initial = repo
                .commit(Some("refs/heads/main"), &sig, &sig, "Initial commit", &tree, &[])
                .unwrap();
            let initial_commit = repo.find_commit(initial).unwrap();

            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();

            // Create feature branch from initial
            repo.branch("feature", &initial_commit, false).unwrap();

            // On main: change file.txt to "main content"
            fs::write(dir.path().join("file.txt"), "main content").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(
                Some("refs/heads/main"),
                &sig,
                &sig,
                "Main commit",
                &tree,
                &[&initial_commit],
            )
            .unwrap();

            // Switch to feature: change file.txt to "feature content"
            repo.set_head("refs/heads/feature").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();

            fs::write(dir.path().join("file.txt"), "feature content").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(
                Some("refs/heads/feature"),
                &sig,
                &sig,
                "Feature commit",
                &tree,
                &[&initial_commit],
            )
            .unwrap();

            // Switch back to main
            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();
        }

        // Start a merge that will conflict (using git CLI)
        let output = std::process::Command::new("git")
            .args(["merge", "feature"])
            .current_dir(dir.path())
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .unwrap();
        assert!(!output.status.success(), "merge should fail with conflict");

        let mut state_map = HashMap::new();
        state_map.insert(path_str.clone(), dir.path().to_owned());
        (dir, state_map, path_str)
    }

    #[test]
    fn get_merge_sides_returns_conflict_content() {
        let (_dir, state_map, path) = make_conflicted_repo();

        let result = get_merge_sides_inner(&path, "file.txt", &state_map);
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);

        let sides = result.unwrap();
        assert_eq!(sides.ours, "main content", "ours should be main content");
        assert_eq!(sides.theirs, "feature content", "theirs should be feature content");
        assert_eq!(sides.base, "hello", "base should be original content");
    }

    #[test]
    fn get_merge_sides_no_ancestor() {
        // Create a repo where both branches add a new file with the same name (no common ancestor)
        let dir = TempDir::new().unwrap();
        let path_str = dir.path().to_str().unwrap().to_owned();

        {
            let repo = git2::Repository::init(dir.path()).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
            drop(config);

            let sig = repo.signature().unwrap();

            // Initial commit with only a placeholder file
            fs::write(dir.path().join("placeholder.txt"), "init").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("placeholder.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let initial = repo
                .commit(Some("refs/heads/main"), &sig, &sig, "Initial", &tree, &[])
                .unwrap();
            let initial_commit = repo.find_commit(initial).unwrap();

            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();

            // Create feature branch from initial
            repo.branch("feature", &initial_commit, false).unwrap();

            // On main: add new_file.txt with "main version"
            fs::write(dir.path().join("new_file.txt"), "main version").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("new_file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(
                Some("refs/heads/main"),
                &sig,
                &sig,
                "Add new_file on main",
                &tree,
                &[&initial_commit],
            )
            .unwrap();

            // Switch to feature: add new_file.txt with "feature version"
            repo.set_head("refs/heads/feature").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();

            fs::write(dir.path().join("new_file.txt"), "feature version").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("new_file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            repo.commit(
                Some("refs/heads/feature"),
                &sig,
                &sig,
                "Add new_file on feature",
                &tree,
                &[&initial_commit],
            )
            .unwrap();

            // Switch back to main
            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .unwrap();
        }

        // Start a merge that will conflict
        let output = std::process::Command::new("git")
            .args(["merge", "feature"])
            .current_dir(dir.path())
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .unwrap();
        assert!(!output.status.success(), "merge should fail with conflict");

        let mut state_map = HashMap::new();
        state_map.insert(path_str.clone(), dir.path().to_owned());

        let result = get_merge_sides_inner(&path_str, "new_file.txt", &state_map);
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);

        let sides = result.unwrap();
        assert_eq!(sides.base, "", "base should be empty for file added on both sides");
        assert_eq!(sides.ours, "main version", "ours should be main version");
        assert_eq!(sides.theirs, "feature version", "theirs should be feature version");
    }

    #[test]
    fn save_merge_result_writes_and_stages() {
        let (dir, state_map, path) = make_conflicted_repo();

        let result = save_merge_result_inner(&path, "file.txt", "resolved content", &state_map);
        assert!(result.is_ok(), "expected Ok, got: {:?}", result);

        // Assert the file on disk contains "resolved content"
        let content = fs::read_to_string(dir.path().join("file.txt")).unwrap();
        assert_eq!(content, "resolved content", "file on disk should contain resolved content");

        // Assert the file is staged in the index (no longer in conflict entries)
        let repo = git2::Repository::open(dir.path()).unwrap();
        let index = repo.index().unwrap();
        assert!(!index.has_conflicts(), "index should have no conflicts after staging");
    }
}
