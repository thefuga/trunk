use crate::error::TrunkError;
use crate::git::{graph, read_model, types::GraphResult, types::RepoDescriptor};
use std::collections::HashMap;
use std::path::PathBuf;

/// Open the git repository registered for `path` in the app's repo-state map.
/// Returns a `not_open` error if the path was never opened. Shared by every
/// command module so the open/error contract lives in exactly one place.
pub(crate) fn open_repo_from_state(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<git2::Repository, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    git2::Repository::open(path_buf).map_err(TrunkError::from)
}

pub(crate) fn repo_descriptor_from_state(
    repo_id: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<RepoDescriptor, TrunkError> {
    if let Some(descriptor) = descriptor_map.get(repo_id) {
        return Ok(descriptor.clone());
    }

    let path_buf = state_map
        .get(repo_id)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", repo_id)))?;
    Ok(RepoDescriptor::local(
        path_buf.to_string_lossy().into_owned(),
    ))
}

pub(crate) fn refresh_graph_from_state(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<GraphResult, TrunkError> {
    match read_model::backend_from_state(path, state_map, descriptor_map)? {
        read_model::ReadBackend::Local(path_buf) => {
            let mut repo = git2::Repository::open(path_buf)?;
            graph::walk_commits(&mut repo, 0, usize::MAX)
        }
        read_model::ReadBackend::Wsl(repo) => read_model::wsl_commit_graph(&repo),
    }
}

pub mod branches;
pub mod commit;
pub mod commit_actions;
pub mod diff;
pub mod fs;
pub mod history;
pub mod interactive_rebase;
pub mod merge_editor;
pub mod operation_state;
pub mod remote;
pub mod repo;
pub mod review;
pub mod staging;
pub mod stash;
pub mod wsl;
