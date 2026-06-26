use crate::error::TrunkError;
use crate::git::{
    backend,
    backend_fs::BackendTempDir,
    command_runner,
    types::{RebaseTodoItem, RepoDescriptor},
};
use crate::state::{CommitCache, RepoState};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RebaseTodoAction {
    pub oid: String,
    pub action: String, // "pick", "squash", "reword", "drop"
    pub summary: String,
    pub new_message: Option<String>,
}

fn interactive_rebase_command_spec(
    repo: &RepoDescriptor,
    base_oid: &str,
    seq_editor_path: &str,
    editor_script_path: &str,
) -> command_runner::GitCommandSpec {
    command_runner::GitCommandSpec::for_repo(repo, &["rebase", "-i", base_oid])
        .with_interactive_rebase_editor_env(repo, seq_editor_path, editor_script_path)
}

pub fn get_rebase_todo_inner(
    path: &str,
    base_oid: &str,
    inclusive: bool,
    state_map: &HashMap<String, PathBuf>,
) -> Result<Vec<RebaseTodoItem>, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;

    let base =
        git2::Oid::from_str(base_oid).map_err(|e| TrunkError::new("invalid_oid", e.to_string()))?;

    let mut revwalk = repo.revwalk().map_err(TrunkError::from)?;
    revwalk
        .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
        .map_err(TrunkError::from)?;
    revwalk.push_head().map_err(TrunkError::from)?;

    if inclusive {
        let commit = repo.find_commit(base).map_err(TrunkError::from)?;
        if commit.parent_count() > 0 {
            revwalk
                .hide(commit.parent_id(0).map_err(TrunkError::from)?)
                .map_err(TrunkError::from)?;
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

pub fn get_rebase_todo_inner_with_descriptors(
    path: &str,
    base_oid: &str,
    inclusive: bool,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, crate::git::types::RepoDescriptor>,
) -> Result<Vec<RebaseTodoItem>, TrunkError> {
    let descriptor = crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    backend::resolve_backend(descriptor)?.rebase_todo(path, base_oid, inclusive, state_map)
}

pub fn get_fork_point_inner(
    path: &str,
    branch: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, crate::git::types::RepoDescriptor>,
) -> Result<String, TrunkError> {
    let repo = crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    let output =
        command_runner::git_output(&repo, &["merge-base", branch, "HEAD"], "fork_point_error")?;

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
    session_dir: &BackendTempDir,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, crate::git::types::RepoDescriptor>,
) -> Result<crate::git::types::GraphResult, TrunkError> {
    let repo_descriptor =
        crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    backend::ensure_backend_supported(&repo_descriptor)?;

    // 1. Write todo file (drop = omit from list, not the 'drop' keyword)
    let todo_path = session_dir.join_display("trunk-rebase-todo");
    let todo_content: String = todo_items
        .iter()
        .filter(|item| item.action != "drop")
        .map(|item| format!("{} {} {}", item.action, item.oid, item.summary))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    session_dir.write_file("trunk-rebase-todo", &todo_content, false)?;

    // 2. Write GIT_SEQUENCE_EDITOR script (script file for reliable $1 handling)
    let seq_editor_path = session_dir.join_display("trunk-seq-editor.sh");
    // T-75-T04 parity: POSIX single-quote the path so $TMPDIR-controlled `"` or `'` cannot
    // terminate the quoted segment. Shared helper lives in `git::editor`.
    let seq_editor_script = format!(
        "#!/bin/sh\ncp {} \"$1\"\n",
        crate::git::editor::shell_single_quote(&todo_path),
    );
    session_dir.write_file("trunk-seq-editor.sh", &seq_editor_script, true)?;

    // 3. Write pre-edited message files (consumed by GIT_EDITOR in order)
    let msg_queue_dir = session_dir.join_display("msg-queue");
    let _ = session_dir.create_dir_all("msg-queue");
    let mut msg_index = 0u32;
    for item in todo_items.iter().filter(|i| i.action != "drop") {
        if item.action == "reword" || item.action == "squash" {
            if let Some(ref new_msg) = item.new_message {
                session_dir.write_file(&format!("msg-queue/{:04}", msg_index), new_msg, false)?;
            }
            msg_index += 1;
        }
    }

    // 4. Write GIT_EDITOR script — consumes the first available message file, or accepts default.
    // T-75-T04 parity: bind the queue path into a single-quoted shell variable so embedded
    // `"` or `'` in $TMPDIR cannot break out into shell syntax.
    let editor_script_path = session_dir.join_display("trunk-rebase-editor.sh");
    let editor_script = format!(
        r#"#!/bin/sh
QUEUE={queue}
MSG=$(ls -1 "$QUEUE/" 2>/dev/null | sort | head -1)
if [ -n "$MSG" ]; then
  cp "$QUEUE/$MSG" "$1"
  rm "$QUEUE/$MSG"
  exit 0
fi
exit 0
"#,
        queue = crate::git::editor::shell_single_quote(&msg_queue_dir),
    );
    session_dir.write_file("trunk-rebase-editor.sh", &editor_script, true)?;

    // 5. Run git rebase -i (blocking — waits for completion)
    let output = interactive_rebase_command_spec(
        &repo_descriptor,
        base_oid,
        &seq_editor_path,
        &editor_script_path,
    )
    .command()
    .output()
    .map_err(|e| TrunkError::new("rebase_error", e.to_string()))?;

    // 6. Handle result
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        // Conflicts leave the repo in rebase-in-progress state — that's expected
        if !stderr.to_lowercase().contains("conflict")
            && !stderr.to_lowercase().contains("could not apply")
        {
            return Err(TrunkError::new("rebase_error", stderr));
        }
    }

    crate::commands::refresh_graph_from_state(path, state_map, descriptor_map)
}

#[tauri::command]
pub async fn get_rebase_todo(
    path: String,
    base_oid: String,
    inclusive: Option<bool>,
    state: State<'_, RepoState>,
) -> Result<Vec<RebaseTodoItem>, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let incl = inclusive.unwrap_or(false);
    tauri::async_runtime::spawn_blocking(move || {
        get_rebase_todo_inner_with_descriptors(&path, &base_oid, incl, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e: TrunkError| e.to_json())
}

#[tauri::command]
pub async fn get_fork_point(
    path: String,
    branch: String,
    state: State<'_, RepoState>,
) -> Result<String, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        get_fork_point_inner(&path, &branch, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e: TrunkError| e.to_json())
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
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();

    let repo_descriptor =
        crate::commands::repo_descriptor_from_state(&path, &state_map, &descriptor_map)
            .map_err(|e| e.to_json())?;
    let session_dir =
        BackendTempDir::create(&repo_descriptor, "trunk-rebase").map_err(|e| e.to_json())?;

    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        let result = start_interactive_rebase_blocking(
            &path_clone,
            &base_oid,
            &todo_items,
            &session_dir,
            &state_map,
            &descriptor_map,
        );
        session_dir.cleanup();
        result
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e: TrunkError| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    fn wsl_repo() -> RepoDescriptor {
        let locator = crate::git::types::RepoLocator::Wsl {
            distro: "Ubuntu".to_string(),
            linux_path: "/home/me/project".to_string(),
        };
        RepoDescriptor {
            id: locator.stable_id(),
            display_name: "project".to_string(),
            display_path: r"\\wsl.localhost\Ubuntu\home\me\project".to_string(),
            locator,
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn wsl_interactive_rebase_command_propagates_editor_env() {
        let spec = interactive_rebase_command_spec(
            &wsl_repo(),
            "abc123",
            "/tmp/trunk-rebase/trunk-seq-editor.sh",
            "/tmp/trunk-rebase/trunk-rebase-editor.sh",
        );

        assert_eq!(spec.program, "wsl.exe");
        assert_eq!(
            spec.args,
            vec![
                "-d",
                "Ubuntu",
                "--cd",
                "/home/me/project",
                "git",
                "rebase",
                "-i",
                "abc123",
            ]
        );
        assert_eq!(
            spec.env,
            vec![
                (
                    "GIT_SEQUENCE_EDITOR".to_string(),
                    "/tmp/trunk-rebase/trunk-seq-editor.sh".to_string(),
                ),
                (
                    "GIT_EDITOR".to_string(),
                    "/tmp/trunk-rebase/trunk-rebase-editor.sh".to_string(),
                ),
                (
                    "WSLENV".to_string(),
                    "GIT_SEQUENCE_EDITOR:GIT_EDITOR".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn local_interactive_rebase_command_does_not_set_wslenv() {
        let spec = interactive_rebase_command_spec(
            &RepoDescriptor::local("/repo".to_string()),
            "abc123",
            "/tmp/trunk-rebase/trunk-seq-editor.sh",
            "/tmp/trunk-rebase/trunk-rebase-editor.sh",
        );

        assert_eq!(spec.program, "git");
        assert_eq!(spec.args, vec!["rebase", "-i", "abc123"]);
        assert_eq!(spec.current_dir, Some(PathBuf::from("/repo")));
        assert_eq!(
            spec.env,
            vec![
                (
                    "GIT_SEQUENCE_EDITOR".to_string(),
                    "/tmp/trunk-rebase/trunk-seq-editor.sh".to_string(),
                ),
                (
                    "GIT_EDITOR".to_string(),
                    "/tmp/trunk-rebase/trunk-rebase-editor.sh".to_string(),
                ),
            ]
        );
    }
}
