use crate::error::TrunkError;
use crate::git::read_model;
use crate::git::types::{MergeSides, RepoDescriptor, RepoLocator};
use crate::state::{CommitCache, RepoState};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

pub fn get_merge_sides_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<MergeSides, TrunkError> {
    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let index = repo.index()?;

    // Find the conflict entry for this file by iterating all conflicts
    let mut conflicts = index
        .conflicts()
        .map_err(|e| TrunkError::new("conflict_error", e.to_string()))?;

    let conflict = conflicts
        .find(|entry| {
            if let Ok(ref c) = entry {
                let entry_path = c
                    .our
                    .as_ref()
                    .or(c.their.as_ref())
                    .or(c.ancestor.as_ref())
                    .map(|e| String::from_utf8_lossy(&e.path).into_owned());
                entry_path.as_deref() == Some(file_path)
            } else {
                false
            }
        })
        .ok_or_else(|| {
            TrunkError::new(
                "not_conflicted",
                format!("File not in conflict: {}", file_path),
            )
        })?
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

fn get_merge_sides_inner_with_descriptors(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<MergeSides, TrunkError> {
    match read_model::backend_from_state(path, state_map, descriptor_map)? {
        read_model::ReadBackend::Local(_) => get_merge_sides_inner(path, file_path, state_map),
        read_model::ReadBackend::Wsl(repo) => {
            let read_stage = |stage: &str| -> Result<Option<String>, TrunkError> {
                let spec = format!(":{stage}:{file_path}");
                let output = crate::git::command_runner::git_output(
                    &repo,
                    &["show", &spec],
                    "conflict_error",
                )?;
                if output.status.success() {
                    Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
                } else {
                    Ok(None)
                }
            };
            let ours = read_stage("2")?;
            let theirs = read_stage("3")?;
            if ours.is_none() && theirs.is_none() {
                return Err(TrunkError::new(
                    "not_conflicted",
                    format!("File not in conflict: {}", file_path),
                ));
            }
            Ok(MergeSides {
                base: read_stage("1")?.unwrap_or_default(),
                ours: ours.unwrap_or_default(),
                theirs: theirs.unwrap_or_default(),
            })
        }
    }
}

pub fn save_merge_result_inner(
    path: &str,
    file_path: &str,
    content: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    save_merge_result_inner_with_descriptors(path, file_path, content, state_map, &HashMap::new())
}

fn save_merge_result_inner_with_descriptors(
    path: &str,
    file_path: &str,
    content: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<(), TrunkError> {
    let repo_descriptor =
        crate::commands::repo_descriptor_from_state(path, state_map, descriptor_map)?;
    if matches!(repo_descriptor.locator, RepoLocator::Wsl { .. }) {
        crate::git::backend_fs::write_repo_file(&repo_descriptor, file_path, content)?;
        let output = crate::git::command_runner::git_output(
            &repo_descriptor,
            &["add", "--", file_path],
            "stage_error",
        )?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TrunkError::new("stage_error", stderr.to_string()));
        }
        return Ok(());
    }

    let repo = crate::commands::open_repo_from_state(path, state_map)?;
    let repo_path = repo
        .workdir()
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
    let descriptor_map = state.1.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        get_merge_sides_inner_with_descriptors(&path, &file_path, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())
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
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();
    let state_map_clone = state_map.clone();
    let descriptor_map_clone = descriptor_map.clone();
    tauri::async_runtime::spawn_blocking(move || {
        save_merge_result_inner_with_descriptors(
            &path_clone,
            &file_path,
            &content,
            &state_map_clone,
            &descriptor_map_clone,
        )
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    // Repopulate cache and emit repo-changed (same pattern as merge_continue)
    let path_for_cache = path.clone();
    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        crate::commands::refresh_graph_from_state(&path_for_cache, &state_map, &descriptor_map)
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    cache.0.lock().unwrap().insert(path.clone(), graph_result);
    let _ = app.emit("repo-changed", path);
    Ok(())
}
