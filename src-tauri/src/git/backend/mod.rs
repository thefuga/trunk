use crate::commands::staging::DirtyCounts;
use crate::error::TrunkError;
use crate::git::command_runner::GitCommandSpec;
use crate::git::types::{
    CommitDetail, DiffRequestOptions, FileDiff, GraphResult, HeadCommitMessage, MergeSides,
    OperationInfo, RebaseTodoItem, RefsResponse, RepoDescriptor, RepoLocator, StashEntry,
    UndoResult, WorkingTreeStatus,
};
use std::collections::HashMap;
use std::path::PathBuf;

mod local;
#[cfg(all(target_os = "windows", feature = "wsl"))]
pub(crate) mod wsl;

pub use local::LocalBackend;
fn backend_method_not_implemented<T>(method: &str) -> Result<T, TrunkError> {
    Err(TrunkError::new(
        "backend_method_not_implemented",
        format!("Git backend method `{method}` is not implemented"),
    ))
}

pub struct PreparedOpenRepo {
    pub descriptor: RepoDescriptor,
    pub execution_path: String,
    pub use_native_watcher: bool,
}

pub fn wsl_unsupported_platform() -> TrunkError {
    TrunkError::new(
        "wsl_unsupported_platform",
        "WSL repositories can only be used on Windows.",
    )
}

pub fn ensure_backend_supported(descriptor: &RepoDescriptor) -> Result<(), TrunkError> {
    resolve_backend(descriptor.clone()).map(|_| ())
}

#[cfg(all(target_os = "windows", feature = "wsl"))]
pub fn start_wsl_poller_for_repo<R: tauri::Runtime>(
    repo: RepoDescriptor,
    app: tauri::AppHandle<R>,
    state: &crate::watcher::WatcherState,
) {
    wsl::poller::start_for_repo(repo, app, state);
}

#[cfg(not(all(target_os = "windows", feature = "wsl")))]
pub fn start_wsl_poller_for_repo<R: tauri::Runtime>(
    _repo: RepoDescriptor,
    _app: tauri::AppHandle<R>,
    _state: &crate::watcher::WatcherState,
) {
}

pub fn resolve_backend(descriptor: RepoDescriptor) -> Result<Box<dyn GitBackend>, TrunkError> {
    match descriptor.locator {
        RepoLocator::Local { .. } => Ok(Box::new(LocalBackend)),
        RepoLocator::Wsl { .. } => resolve_wsl_backend(descriptor),
    }
}

#[cfg(all(target_os = "windows", feature = "wsl"))]
fn resolve_wsl_backend(descriptor: RepoDescriptor) -> Result<Box<dyn GitBackend>, TrunkError> {
    Ok(Box::new(wsl::WslBackend::new(descriptor)))
}

#[cfg(not(all(target_os = "windows", feature = "wsl")))]
fn resolve_wsl_backend(_descriptor: RepoDescriptor) -> Result<Box<dyn GitBackend>, TrunkError> {
    Err(wsl_unsupported_platform())
}

pub trait GitBackend: Send + Sync {
    fn prepare_open_repo(
        &self,
        _descriptor: RepoDescriptor,
    ) -> Result<PreparedOpenRepo, TrunkError> {
        backend_method_not_implemented("prepare_open_repo")
    }

    fn command_spec(
        &self,
        _repo: &RepoDescriptor,
        _git_args: &[&str],
    ) -> Result<GitCommandSpec, TrunkError> {
        backend_method_not_implemented("command_spec")
    }

    fn with_interactive_rebase_editor_env(
        &self,
        spec: GitCommandSpec,
        _repo: &RepoDescriptor,
    ) -> GitCommandSpec {
        spec
    }

    fn read_repo_file(
        &self,
        _repo: &RepoDescriptor,
        _relative_path: &str,
    ) -> Result<String, TrunkError> {
        backend_method_not_implemented("read_repo_file")
    }

    fn write_repo_file(
        &self,
        _repo: &RepoDescriptor,
        _relative_path: &str,
        _content: &str,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("write_repo_file")
    }

    fn read_absolute_file(
        &self,
        _repo: &RepoDescriptor,
        _path: &str,
    ) -> Result<String, TrunkError> {
        backend_method_not_implemented("read_absolute_file")
    }

    fn write_absolute_file(
        &self,
        _repo: &RepoDescriptor,
        _path: &str,
        _content: &str,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("write_absolute_file")
    }

    fn delete_repo_file(
        &self,
        _repo: &RepoDescriptor,
        _relative_path: &str,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("delete_repo_file")
    }

    fn create_temp_dir(
        &self,
        _repo: &RepoDescriptor,
        _name: &str,
    ) -> Result<crate::git::backend_fs::BackendTempDir, TrunkError> {
        backend_method_not_implemented("create_temp_dir")
    }

    fn poll_token(&self, _repo: &RepoDescriptor) -> Result<Option<String>, TrunkError> {
        backend_method_not_implemented("poll_token")
    }

    fn status(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<WorkingTreeStatus, TrunkError> {
        backend_method_not_implemented("status")
    }

    fn commit_graph(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("commit_graph")
    }

    fn diff_unstaged(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
        _options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        backend_method_not_implemented("diff_unstaged")
    }

    fn diff_staged(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
        _options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        backend_method_not_implemented("diff_staged")
    }

    fn list_commit_files(
        &self,
        _repo_id: &str,
        _oid: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        backend_method_not_implemented("list_commit_files")
    }

    fn diff_commit_file(
        &self,
        _repo_id: &str,
        _oid: &str,
        _file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
        _options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        backend_method_not_implemented("diff_commit_file")
    }

    fn commit_detail(
        &self,
        _repo_id: &str,
        _oid: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<CommitDetail, TrunkError> {
        backend_method_not_implemented("commit_detail")
    }

    fn refs(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<RefsResponse, TrunkError> {
        backend_method_not_implemented("refs")
    }

    fn resolve_ref(
        &self,
        _repo_id: &str,
        _ref_name: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<String, TrunkError> {
        backend_method_not_implemented("resolve_ref")
    }

    fn checkout_branch(
        &self,
        _repo_id: &str,
        _branch_name: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("checkout_branch")
    }

    fn fast_forward_to(
        &self,
        _repo_id: &str,
        _target_oid: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("fast_forward_to")
    }

    fn create_branch(
        &self,
        _repo_id: &str,
        _name: &str,
        _from_oid: Option<&str>,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("create_branch")
    }

    fn delete_branch(
        &self,
        _repo_id: &str,
        _branch_name: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("delete_branch")
    }

    fn rename_branch(
        &self,
        _repo_id: &str,
        _old_name: &str,
        _new_name: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("rename_branch")
    }

    fn stage_files(
        &self,
        _repo_id: &str,
        _file_paths: &[String],
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("stage_files")
    }

    fn unstage_files(
        &self,
        _repo_id: &str,
        _file_paths: &[String],
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("unstage_files")
    }

    fn discard_file(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("discard_file")
    }

    fn discard_all(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("discard_all")
    }

    fn stage_all(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("stage_all")
    }

    fn unstage_all(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("unstage_all")
    }

    fn stage_hunk(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _hunk_index: u32,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("stage_hunk")
    }

    fn unstage_hunk(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _hunk_index: u32,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("unstage_hunk")
    }

    fn discard_hunk(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _hunk_index: u32,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("discard_hunk")
    }

    fn stage_lines(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _hunk_index: u32,
        _line_indices: Vec<u32>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("stage_lines")
    }

    fn unstage_lines(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _hunk_index: u32,
        _line_indices: Vec<u32>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("unstage_lines")
    }

    fn discard_lines(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _hunk_index: u32,
        _line_indices: Vec<u32>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("discard_lines")
    }

    fn dirty_counts(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<DirtyCounts, TrunkError> {
        backend_method_not_implemented("dirty_counts")
    }

    fn create_commit(
        &self,
        _repo_id: &str,
        _subject: &str,
        _body: Option<&str>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("create_commit")
    }

    fn amend_commit(
        &self,
        _repo_id: &str,
        _subject: &str,
        _body: Option<&str>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("amend_commit")
    }

    fn head_commit_message(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<HeadCommitMessage, TrunkError> {
        backend_method_not_implemented("head_commit_message")
    }

    fn operation_state(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<OperationInfo, TrunkError> {
        backend_method_not_implemented("operation_state")
    }

    fn list_stashes(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<StashEntry>, TrunkError> {
        backend_method_not_implemented("list_stashes")
    }

    fn stash_save(
        &self,
        _repo_id: &str,
        _message: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("stash_save")
    }

    fn stash_pop(
        &self,
        _repo_id: &str,
        _index: usize,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("stash_pop")
    }

    fn stash_apply(
        &self,
        _repo_id: &str,
        _index: usize,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("stash_apply")
    }

    fn stash_drop(
        &self,
        _repo_id: &str,
        _index: usize,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("stash_drop")
    }

    fn checkout_commit(
        &self,
        _repo_id: &str,
        _oid: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("checkout_commit")
    }

    fn create_tag(
        &self,
        _repo_id: &str,
        _oid: &str,
        _tag_name: &str,
        _message: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("create_tag")
    }

    fn delete_tag(
        &self,
        _repo_id: &str,
        _tag_name: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("delete_tag")
    }

    fn cherry_pick(
        &self,
        _repo_id: &str,
        _oid: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("cherry_pick")
    }

    fn reset_to_commit(
        &self,
        _repo_id: &str,
        _oid: &str,
        _mode: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        backend_method_not_implemented("reset_to_commit")
    }

    fn undo_commit(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<UndoResult, TrunkError> {
        backend_method_not_implemented("undo_commit")
    }

    fn redo_commit(
        &self,
        _repo_id: &str,
        _subject: &str,
        _body: Option<&str>,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("redo_commit")
    }

    fn check_undo_available(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<bool, TrunkError> {
        backend_method_not_implemented("check_undo_available")
    }

    fn merge_sides(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<MergeSides, TrunkError> {
        backend_method_not_implemented("merge_sides")
    }

    fn save_merge_result(
        &self,
        _repo_id: &str,
        _file_path: &str,
        _content: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        backend_method_not_implemented("save_merge_result")
    }

    fn rebase_todo(
        &self,
        _repo_id: &str,
        _base_oid: &str,
        _inclusive: bool,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<RebaseTodoItem>, TrunkError> {
        backend_method_not_implemented("rebase_todo")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "windows"))]
    fn wsl_descriptor() -> RepoDescriptor {
        let locator = RepoLocator::Wsl {
            distro: "Ubuntu".to_string(),
            linux_path: "/home/me/project".to_string(),
        };
        RepoDescriptor {
            id: locator.stable_id(),
            display_name: "project".to_string(),
            display_path: "Ubuntu:/home/me/project".to_string(),
            locator,
        }
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn resolver_rejects_wsl_descriptors_off_windows() {
        let error = match resolve_backend(wsl_descriptor()) {
            Ok(_) => panic!("expected WSL resolver to reject non-Windows target"),
            Err(error) => error,
        };
        assert_eq!(error.code, "wsl_unsupported_platform");
    }
}
