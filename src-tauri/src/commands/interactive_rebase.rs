use std::collections::HashMap;
use std::path::PathBuf;
use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};
use crate::error::TrunkError;
use crate::git::{graph, types::RebaseTodoItem};
use crate::state::{CommitCache, RepoState};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RebaseTodoAction {
    pub oid: String,
    pub action: String, // "pick", "squash", "reword", "drop"
    pub summary: String,
    pub new_message: Option<String>,
}

fn open_repo(path: &str, state_map: &HashMap<String, PathBuf>) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

pub fn get_rebase_todo_inner(
    path: &str,
    base_oid: &str,
    inclusive: bool,
    state_map: &HashMap<String, PathBuf>,
) -> Result<Vec<RebaseTodoItem>, TrunkError> {
    let repo = open_repo(path, state_map)?;

    let base = git2::Oid::from_str(base_oid)
        .map_err(|e| TrunkError::new("invalid_oid", e.to_string()))?;

    let mut revwalk = repo.revwalk().map_err(TrunkError::from)?;
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME).map_err(TrunkError::from)?;
    revwalk.push_head().map_err(TrunkError::from)?;

    if inclusive {
        let commit = repo.find_commit(base).map_err(TrunkError::from)?;
        if commit.parent_count() > 0 {
            revwalk.hide(commit.parent_id(0).map_err(TrunkError::from)?).map_err(TrunkError::from)?;
        }
        // Root commit: don't hide anything — all commits included
    } else {
        revwalk.hide(base).map_err(TrunkError::from)?;
    }

    let mut items: Vec<RebaseTodoItem> = Vec::new();
    for oid_result in revwalk {
        let oid = oid_result.map_err(TrunkError::from)?;
        let commit = repo.find_commit(oid).map_err(TrunkError::from)?;
        let oid_str = oid.to_string();
        let short_oid = oid_str.chars().take(7).collect();
        let summary = commit.summary().unwrap_or("").to_owned();
        let author_name = commit.author().name().unwrap_or("").to_owned();
        let author_timestamp = commit.time().seconds();

        items.push(RebaseTodoItem {
            oid: oid_str,
            short_oid,
            summary,
            author_name,
            author_timestamp,
        });
    }

    // Revwalk returns newest-first; rebase todo needs oldest-first
    items.reverse();

    Ok(items)
}

pub fn get_fork_point_inner(
    path: &str,
    branch: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<String, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    let output = std::process::Command::new("git")
        .args(["merge-base", branch, "HEAD"])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| TrunkError::new("fork_point_error", e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("fork_point_error", stderr.to_string()));
    }

    let oid = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok(oid)
}

pub fn start_interactive_rebase_blocking(
    path: &str,
    base_oid: &str,
    todo_items: &[RebaseTodoAction],
    session_dir: &std::path::Path,
    state_map: &HashMap<String, PathBuf>,
) -> Result<crate::git::types::GraphResult, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;

    // 1. Write todo file (drop = omit from list, not the 'drop' keyword)
    let todo_path = session_dir.join("trunk-rebase-todo");
    let todo_content: String = todo_items
        .iter()
        .filter(|item| item.action != "drop")
        .map(|item| format!("{} {} {}", item.action, item.oid, item.summary))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&todo_path, &todo_content)
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;

    // 2. Write GIT_SEQUENCE_EDITOR script (script file for reliable $1 handling)
    let seq_editor_path = session_dir.join("trunk-seq-editor.sh");
    let seq_editor_script = format!(
        "#!/bin/sh\ncp \"{}\" \"$1\"\n",
        todo_path.display(),
    );
    std::fs::write(&seq_editor_path, &seq_editor_script)
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&seq_editor_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    }

    // 3. Write pre-edited message files (consumed by GIT_EDITOR in order)
    let msg_queue_dir = session_dir.join("msg-queue");
    let _ = std::fs::create_dir_all(&msg_queue_dir);
    let mut msg_index = 0u32;
    for item in todo_items.iter().filter(|i| i.action != "drop") {
        if item.action == "reword" || item.action == "squash" {
            if let Some(ref new_msg) = item.new_message {
                let msg_file = msg_queue_dir.join(format!("{:04}", msg_index));
                std::fs::write(&msg_file, new_msg)
                    .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
            }
            msg_index += 1;
        }
    }

    // 4. Write GIT_EDITOR script — consumes the first available message file, or accepts default
    let editor_script_path = session_dir.join("trunk-rebase-editor.sh");
    let editor_script = format!(
        r#"#!/bin/sh
MSG=$(ls -1 "{queue}/" 2>/dev/null | sort | head -1)
if [ -n "$MSG" ]; then
  cp "{queue}/$MSG" "$1"
  rm "{queue}/$MSG"
  exit 0
fi
exit 0
"#,
        queue = msg_queue_dir.display(),
    );
    std::fs::write(&editor_script_path, &editor_script)
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&editor_script_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    }

    // 5. Run git rebase -i (blocking — waits for completion)
    let output = std::process::Command::new("git")
        .args(["rebase", "-i", "--empty=keep", base_oid])
        .current_dir(path_buf)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_SEQUENCE_EDITOR", seq_editor_path.to_str().unwrap())
        .env("GIT_EDITOR", editor_script_path.to_str().unwrap())
        .output()
        .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;

    // 6. Handle result
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        // Conflicts leave the repo in rebase-in-progress state — that's expected
        if !stderr.to_lowercase().contains("conflict") && !stderr.to_lowercase().contains("could not apply") {
            return Err(TrunkError::new("rebase_error", stderr));
        }
    }

    let mut repo = git2::Repository::open(path_buf)?;
    graph::walk_commits(&mut repo, 0, usize::MAX).map_err(TrunkError::from)
}

#[tauri::command]
pub async fn get_rebase_todo(
    path: String,
    base_oid: String,
    inclusive: Option<bool>,
    state: State<'_, RepoState>,
) -> Result<Vec<RebaseTodoItem>, String> {
    let state_map = state.0.lock().unwrap().clone();
    let incl = inclusive.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
        get_rebase_todo_inner(&path, &base_oid, incl, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn get_fork_point(
    path: String,
    branch: String,
    state: State<'_, RepoState>,
) -> Result<String, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        get_fork_point_inner(&path, &branch, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}

#[tauri::command]
pub async fn start_interactive_rebase(
    path: String,
    base_oid: String,
    todo_items: Vec<RebaseTodoAction>,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let path_clone = path.clone();

    let session_dir = std::env::temp_dir().join(format!("trunk-rebase-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&session_dir);
    let session_dir_cleanup = session_dir.clone();

    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        start_interactive_rebase_blocking(
            &path_clone, &base_oid, &todo_items, &session_dir, &state_map,
        )
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())?;

    let _ = std::fs::remove_dir_all(&session_dir_cleanup);

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_repo() -> (TempDir, HashMap<String, PathBuf>, Vec<git2::Oid>) {
        let dir = TempDir::new().unwrap();
        let path_str = dir.path().to_str().unwrap().to_owned();
        let mut oids = Vec::new();

        {
            let repo = git2::Repository::init(dir.path()).unwrap();
            let mut config = repo.config().unwrap();
            config.set_str("user.name", "Test").unwrap();
            config.set_str("user.email", "test@test.com").unwrap();
            drop(config);

            let sig = repo.signature().unwrap();

            // Commit 1: Initial commit
            fs::write(dir.path().join("file.txt"), "initial").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let c1 = repo.commit(Some("refs/heads/main"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
            oids.push(c1);

            // Commit 2: Second commit
            fs::write(dir.path().join("file.txt"), "second").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let parent = repo.find_commit(c1).unwrap();
            let c2 = repo.commit(Some("refs/heads/main"), &sig, &sig, "Second commit", &tree, &[&parent]).unwrap();
            oids.push(c2);

            // Commit 3: Third commit
            fs::write(dir.path().join("file.txt"), "third").unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("file.txt")).unwrap();
            index.write().unwrap();
            let tree_oid = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let parent = repo.find_commit(c2).unwrap();
            let c3 = repo.commit(Some("refs/heads/main"), &sig, &sig, "Third commit", &tree, &[&parent]).unwrap();
            oids.push(c3);

            repo.set_head("refs/heads/main").unwrap();
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())).unwrap();
        }

        let mut state_map = HashMap::new();
        state_map.insert(path_str.clone(), dir.path().to_owned());
        (dir, state_map, oids)
    }

    #[test]
    fn get_rebase_todo_returns_commits_oldest_first() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();
        let base_oid = oids[0].to_string(); // Initial commit as base

        let items = get_rebase_todo_inner(path, &base_oid, false, &state_map).unwrap();

        assert_eq!(items.len(), 2, "Should return 2 commits (excluding base)");
        assert_eq!(items[0].summary, "Second commit", "First item should be oldest (Second commit)");
        assert_eq!(items[1].summary, "Third commit", "Second item should be newest (Third commit)");
    }

    #[test]
    fn get_rebase_todo_inclusive_includes_base_commit() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();
        let base_oid = oids[1].to_string(); // Second commit

        let items = get_rebase_todo_inner(path, &base_oid, true, &state_map).unwrap();

        assert_eq!(items.len(), 2, "Should return 2 commits (including base)");
        assert_eq!(items[0].summary, "Second commit", "Base commit should be included");
        assert_eq!(items[1].summary, "Third commit");
    }

    #[test]
    fn get_rebase_todo_returns_empty_when_base_equals_head() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();
        let base_oid = oids[2].to_string(); // HEAD commit as base

        let items = get_rebase_todo_inner(path, &base_oid, false, &state_map).unwrap();

        assert_eq!(items.len(), 0, "Should return empty vec when base equals HEAD");
    }

    #[test]
    fn get_rebase_todo_item_has_correct_fields() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();
        let base_oid = oids[0].to_string();

        let items = get_rebase_todo_inner(path, &base_oid, false, &state_map).unwrap();

        let item = &items[0];
        assert_eq!(item.oid, oids[1].to_string(), "OID should match second commit");
        assert_eq!(item.short_oid, &oids[1].to_string()[..7], "short_oid should be first 7 chars");
        assert_eq!(item.summary, "Second commit");
        assert_eq!(item.author_name, "Test");
        assert!(item.author_timestamp > 0, "author_timestamp should be positive");
    }

    #[test]
    fn get_fork_point_returns_merge_base() {
        let (dir, state_map, oids) = make_test_repo();
        let path = dir.path().to_str().unwrap();

        // Create a branch at the initial commit
        {
            let repo = git2::Repository::open(dir.path()).unwrap();
            let initial_commit = repo.find_commit(oids[0]).unwrap();
            repo.branch("feature", &initial_commit, false).unwrap();
        }

        let result = get_fork_point_inner(path, "feature", &state_map).unwrap();

        assert_eq!(result, oids[0].to_string(), "Fork point should be the initial commit (merge-base of feature and HEAD)");
    }
}
