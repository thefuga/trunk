use crate::commands::staging::DirtyCounts;
use crate::commands::{
    branches, commit, commit_actions, diff, merge_editor, operation_state, staging, stash,
};
use crate::error::TrunkError;
#[cfg(target_os = "windows")]
use crate::git::command_runner;
#[cfg(target_os = "windows")]
use crate::git::read_model;
use crate::git::types::{
    CommitDetail, DiffRequestOptions, FileDiff, GraphResult, HeadCommitMessage, MergeSides,
    OperationInfo, RebaseTodoItem, RefsResponse, RepoDescriptor, RepoLocator, StashEntry,
    UndoResult, WorkingTreeStatus,
};
use crate::git::{graph, repository};
use std::collections::HashMap;
use std::path::PathBuf;

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
    match descriptor.locator {
        RepoLocator::Local { .. } => Ok(()),
        RepoLocator::Wsl { .. } => {
            #[cfg(target_os = "windows")]
            {
                Ok(())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err(wsl_unsupported_platform())
            }
        }
    }
}

pub fn resolve_backend(descriptor: RepoDescriptor) -> Result<Box<dyn GitBackend>, TrunkError> {
    match descriptor.locator {
        RepoLocator::Local { .. } => Ok(Box::new(LocalBackend)),
        RepoLocator::Wsl { .. } => {
            #[cfg(target_os = "windows")]
            {
                Ok(Box::new(WslBackend::new(descriptor)))
            }
            #[cfg(not(target_os = "windows"))]
            {
                Err(wsl_unsupported_platform())
            }
        }
    }
}

pub trait GitBackend: Send + Sync {
    fn prepare_open_repo(
        &self,
        _descriptor: RepoDescriptor,
    ) -> Result<PreparedOpenRepo, TrunkError> {
        backend_method_not_implemented("prepare_open_repo")
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

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalBackend;

impl GitBackend for LocalBackend {
    fn prepare_open_repo(
        &self,
        descriptor: RepoDescriptor,
    ) -> Result<PreparedOpenRepo, TrunkError> {
        let execution_path = match &descriptor.locator {
            RepoLocator::Local { path } => path.clone(),
            RepoLocator::Wsl { .. } => return Err(wsl_unsupported_platform()),
        };
        Ok(PreparedOpenRepo {
            descriptor,
            execution_path,
            use_native_watcher: true,
        })
    }

    fn status(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<WorkingTreeStatus, TrunkError> {
        staging::get_status_inner(repo_id, state_map)
    }

    fn commit_graph(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let path_buf = state_map.get(repo_id).ok_or_else(|| {
            TrunkError::new("not_open", format!("Repository not open: {}", repo_id))
        })?;
        repository::validate_and_open(path_buf)?;
        let mut repo = git2::Repository::open(path_buf)?;
        graph::walk_commits(&mut repo, 0, usize::MAX)
    }

    fn diff_unstaged(
        &self,
        repo_id: &str,
        file_path: &str,
        state_map: &HashMap<String, PathBuf>,
        options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        diff::diff_unstaged_inner(repo_id, file_path, state_map, options)
    }

    fn diff_staged(
        &self,
        repo_id: &str,
        file_path: &str,
        state_map: &HashMap<String, PathBuf>,
        options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        diff::diff_staged_inner(repo_id, file_path, state_map, options)
    }

    fn list_commit_files(
        &self,
        repo_id: &str,
        oid: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        diff::list_commit_files_inner(repo_id, oid, state_map)
    }

    fn diff_commit_file(
        &self,
        repo_id: &str,
        oid: &str,
        file_path: &str,
        state_map: &HashMap<String, PathBuf>,
        options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        diff::diff_commit_file_inner(repo_id, oid, file_path, state_map, options)
    }

    fn commit_detail(
        &self,
        repo_id: &str,
        oid: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<CommitDetail, TrunkError> {
        diff::get_commit_detail_inner(repo_id, oid, state_map)
    }

    fn refs(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<RefsResponse, TrunkError> {
        branches::list_refs_inner(repo_id, state_map)
    }

    fn resolve_ref(
        &self,
        repo_id: &str,
        ref_name: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<String, TrunkError> {
        branches::resolve_ref_inner(repo_id, ref_name, state_map)
    }

    fn checkout_branch(
        &self,
        repo_id: &str,
        branch_name: &str,
        state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let mut cache_map = HashMap::new();
        branches::checkout_branch_inner(repo_id, branch_name, state_map, &mut cache_map)?;
        cache_map
            .remove(repo_id)
            .ok_or_else(|| TrunkError::new("graph_error", "Branch checkout did not refresh graph"))
    }

    fn fast_forward_to(
        &self,
        repo_id: &str,
        target_oid: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let mut cache_map = HashMap::new();
        branches::fast_forward_to_inner(
            repo_id,
            target_oid,
            state_map,
            descriptor_map,
            &mut cache_map,
        )?;
        cache_map
            .remove(repo_id)
            .ok_or_else(|| TrunkError::new("graph_error", "Fast-forward did not refresh graph"))
    }

    fn create_branch(
        &self,
        repo_id: &str,
        name: &str,
        from_oid: Option<&str>,
        state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let mut cache_map = HashMap::new();
        branches::create_branch_inner(repo_id, name, from_oid, state_map, &mut cache_map)?;
        cache_map
            .remove(repo_id)
            .ok_or_else(|| TrunkError::new("graph_error", "Branch creation did not refresh graph"))
    }

    fn delete_branch(
        &self,
        repo_id: &str,
        branch_name: &str,
        state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let mut cache_map = HashMap::new();
        branches::delete_branch_inner(repo_id, branch_name, state_map, &mut cache_map)?;
        cache_map
            .remove(repo_id)
            .ok_or_else(|| TrunkError::new("graph_error", "Branch deletion did not refresh graph"))
    }

    fn rename_branch(
        &self,
        repo_id: &str,
        old_name: &str,
        new_name: &str,
        state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let mut cache_map = HashMap::new();
        branches::rename_branch_inner(repo_id, old_name, new_name, state_map, &mut cache_map)?;
        cache_map
            .remove(repo_id)
            .ok_or_else(|| TrunkError::new("graph_error", "Branch rename did not refresh graph"))
    }

    fn stage_files(
        &self,
        repo_id: &str,
        file_paths: &[String],
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::stage_files_inner(repo_id, file_paths, state_map)
    }

    fn unstage_files(
        &self,
        repo_id: &str,
        file_paths: &[String],
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::unstage_files_inner(repo_id, file_paths, state_map)
    }

    fn discard_file(
        &self,
        repo_id: &str,
        file_path: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::discard_file_inner(repo_id, file_path, state_map)
    }

    fn discard_all(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::discard_all_inner(repo_id, state_map)
    }

    fn stage_all(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::stage_all_inner(repo_id, state_map)
    }

    fn unstage_all(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::unstage_all_inner(repo_id, state_map)
    }

    fn stage_hunk(
        &self,
        repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::stage_hunk_inner(repo_id, file_path, hunk_index, state_map)
    }

    fn unstage_hunk(
        &self,
        repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::unstage_hunk_inner(repo_id, file_path, hunk_index, state_map)
    }

    fn discard_hunk(
        &self,
        repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::discard_hunk_inner(repo_id, file_path, hunk_index, state_map)
    }

    fn stage_lines(
        &self,
        repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::stage_lines_inner(repo_id, file_path, hunk_index, line_indices, state_map)
    }

    fn unstage_lines(
        &self,
        repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::unstage_lines_inner(repo_id, file_path, hunk_index, line_indices, state_map)
    }

    fn discard_lines(
        &self,
        repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::discard_lines_inner(repo_id, file_path, hunk_index, line_indices, state_map)
    }

    fn dirty_counts(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<DirtyCounts, TrunkError> {
        staging::get_dirty_counts_inner(repo_id, state_map)
    }

    fn create_commit(
        &self,
        repo_id: &str,
        subject: &str,
        body: Option<&str>,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        commit::create_commit_inner(repo_id, subject, body, state_map)
    }

    fn amend_commit(
        &self,
        repo_id: &str,
        subject: &str,
        body: Option<&str>,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        commit::amend_commit_inner(repo_id, subject, body, state_map)
    }

    fn head_commit_message(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<HeadCommitMessage, TrunkError> {
        commit::get_head_commit_message_inner(repo_id, state_map)
    }

    fn operation_state(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<OperationInfo, TrunkError> {
        operation_state::get_operation_state_inner(repo_id, state_map)
    }

    fn list_stashes(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<StashEntry>, TrunkError> {
        stash::list_stashes_inner(repo_id, state_map)
    }

    fn stash_save(
        &self,
        repo_id: &str,
        message: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        stash::stash_save_inner(repo_id, message, state_map)
    }

    fn stash_pop(
        &self,
        repo_id: &str,
        index: usize,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        stash::stash_pop_inner(repo_id, index, state_map)
    }

    fn stash_apply(
        &self,
        repo_id: &str,
        index: usize,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        stash::stash_apply_inner(repo_id, index, state_map)
    }

    fn stash_drop(
        &self,
        repo_id: &str,
        index: usize,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        stash::stash_drop_inner(repo_id, index, state_map)
    }

    fn checkout_commit(
        &self,
        repo_id: &str,
        oid: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        commit_actions::checkout_commit_inner(repo_id, oid, state_map)
    }

    fn create_tag(
        &self,
        repo_id: &str,
        oid: &str,
        tag_name: &str,
        message: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        commit_actions::create_tag_inner(repo_id, oid, tag_name, message, state_map)
    }

    fn delete_tag(
        &self,
        repo_id: &str,
        tag_name: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        commit_actions::delete_tag_inner(repo_id, tag_name, state_map)
    }

    fn cherry_pick(
        &self,
        repo_id: &str,
        oid: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        commit_actions::cherry_pick_inner(repo_id, oid, state_map, &HashMap::new())
    }

    fn reset_to_commit(
        &self,
        repo_id: &str,
        oid: &str,
        mode: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        commit_actions::reset_to_commit_inner(repo_id, oid, mode, state_map, descriptor_map)
    }

    fn undo_commit(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<UndoResult, TrunkError> {
        commit_actions::undo_commit_local_inner(repo_id, state_map, descriptor_map)
    }

    fn redo_commit(
        &self,
        repo_id: &str,
        subject: &str,
        body: Option<&str>,
        state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<(), TrunkError> {
        commit_actions::redo_commit_inner(repo_id, subject, body, state_map)
    }

    fn check_undo_available(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<bool, TrunkError> {
        commit_actions::check_undo_available_inner(repo_id, state_map)
    }

    fn merge_sides(
        &self,
        repo_id: &str,
        file_path: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<MergeSides, TrunkError> {
        merge_editor::get_merge_sides_inner(repo_id, file_path, state_map)
    }

    fn save_merge_result(
        &self,
        repo_id: &str,
        file_path: &str,
        content: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        merge_editor::save_merge_result_local_inner(repo_id, file_path, content, state_map)
    }

    fn rebase_todo(
        &self,
        repo_id: &str,
        base_oid: &str,
        inclusive: bool,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<RebaseTodoItem>, TrunkError> {
        crate::commands::interactive_rebase::get_rebase_todo_inner(
            repo_id, base_oid, inclusive, state_map,
        )
    }
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
pub struct WslBackend {
    repo: RepoDescriptor,
}

#[cfg(target_os = "windows")]
impl WslBackend {
    pub fn new(repo: RepoDescriptor) -> Self {
        Self { repo }
    }
}

#[cfg(target_os = "windows")]
impl GitBackend for WslBackend {
    fn prepare_open_repo(
        &self,
        _descriptor: RepoDescriptor,
    ) -> Result<PreparedOpenRepo, TrunkError> {
        let RepoLocator::Wsl { distro, linux_path } = &self.repo.locator else {
            return Err(TrunkError::new(
                "backend_descriptor_mismatch",
                "WSL backend received a non-WSL descriptor",
            ));
        };
        let validation =
            crate::commands::wsl::validate_repo_inner(distro.clone(), linux_path.clone())?;
        let mut descriptor = validation.descriptor;
        descriptor.id = descriptor.locator.stable_id();
        let execution_path =
            crate::commands::wsl::unc_path(&validation.distro, &validation.repo_root);
        Ok(PreparedOpenRepo {
            descriptor,
            execution_path,
            use_native_watcher: false,
        })
    }

    fn status(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<WorkingTreeStatus, TrunkError> {
        read_model::wsl_status(&self.repo)
    }

    fn commit_graph(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        read_model::wsl_commit_graph(&self.repo)
    }

    fn diff_unstaged(
        &self,
        _repo_id: &str,
        file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
        options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        let mut diffs = read_model::wsl_diff_unstaged(&self.repo, file_path, options)?;
        diff::enrich_file_diffs(&mut diffs);
        Ok(diffs)
    }

    fn diff_staged(
        &self,
        _repo_id: &str,
        file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
        options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        let mut diffs = read_model::wsl_diff_staged(&self.repo, file_path, options)?;
        diff::enrich_file_diffs(&mut diffs);
        Ok(diffs)
    }

    fn list_commit_files(
        &self,
        _repo_id: &str,
        oid: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        read_model::wsl_list_commit_files(&self.repo, oid)
    }

    fn diff_commit_file(
        &self,
        _repo_id: &str,
        oid: &str,
        file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
        options: &DiffRequestOptions,
    ) -> Result<Vec<FileDiff>, TrunkError> {
        let mut diffs = read_model::wsl_diff_commit(&self.repo, oid, Some(file_path), options)?;
        diff::enrich_file_diffs(&mut diffs);
        Ok(diffs)
    }

    fn commit_detail(
        &self,
        _repo_id: &str,
        oid: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<CommitDetail, TrunkError> {
        read_model::wsl_commit_detail(&self.repo, oid)
    }

    fn refs(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<RefsResponse, TrunkError> {
        read_model::wsl_refs(&self.repo)
    }

    fn resolve_ref(
        &self,
        _repo_id: &str,
        ref_name: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<String, TrunkError> {
        branches::git_stdout(&self.repo, &["rev-parse", "--verify", ref_name])
    }

    fn checkout_branch(
        &self,
        repo_id: &str,
        branch_name: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        branches::run_git(&self.repo, &["checkout", branch_name])?;
        self.commit_graph(repo_id, state_map, descriptor_map)
    }

    fn fast_forward_to(
        &self,
        repo_id: &str,
        target_oid: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let output = command_runner::git_output(
            &self.repo,
            &["merge", "--ff-only", target_oid],
            "merge_error",
        )?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(TrunkError::new("not_fast_forward", stderr));
        }
        self.commit_graph(repo_id, state_map, descriptor_map)
    }

    fn create_branch(
        &self,
        repo_id: &str,
        name: &str,
        from_oid: Option<&str>,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        if let Some(from_oid) = from_oid {
            branches::run_git(&self.repo, &["branch", name, from_oid])?;
        } else {
            branches::run_git(&self.repo, &["branch", name])?;
        }

        let dirty = command_runner::git_output(
            &self.repo,
            &["diff-index", "--quiet", "HEAD", "--"],
            "git_error",
        )?;
        if !dirty.status.success() {
            return Err(TrunkError::new(
                "dirty_workdir",
                "Branch created but working tree has uncommitted changes — checkout skipped",
            ));
        }

        branches::run_git(&self.repo, &["checkout", name])?;
        self.commit_graph(repo_id, state_map, descriptor_map)
    }

    fn delete_branch(
        &self,
        repo_id: &str,
        branch_name: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        let head =
            branches::git_stdout(&self.repo, &["branch", "--show-current"]).unwrap_or_default();
        if head == branch_name {
            return Err(TrunkError::new(
                "cannot_delete_head",
                "Cannot delete the currently checked-out branch",
            ));
        }
        branches::run_git(&self.repo, &["branch", "-D", branch_name])?;
        self.commit_graph(repo_id, state_map, descriptor_map)
    }

    fn rename_branch(
        &self,
        repo_id: &str,
        old_name: &str,
        new_name: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<GraphResult, TrunkError> {
        branches::run_git(&self.repo, &["branch", "-m", old_name, new_name])?;
        self.commit_graph(repo_id, state_map, descriptor_map)
    }

    fn stage_files(
        &self,
        _repo_id: &str,
        file_paths: &[String],
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_stage_files(&self.repo, file_paths)
    }

    fn unstage_files(
        &self,
        _repo_id: &str,
        file_paths: &[String],
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_unstage_files(&self.repo, file_paths)
    }

    fn discard_file(
        &self,
        _repo_id: &str,
        file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        let status = read_model::wsl_status(&self.repo)?;
        let file_status = status
            .unstaged
            .iter()
            .find(|entry| entry.path == file_path)
            .ok_or_else(|| {
                TrunkError::new(
                    "file_not_found",
                    format!("File not in working tree changes: {}", file_path),
                )
            })?;
        if matches!(file_status.status, crate::git::types::FileStatusType::New) {
            staging::git_write(&self.repo, &["clean", "-f", "--", file_path], "io_error")
        } else {
            staging::git_write(&self.repo, &["checkout", "--", file_path], "discard_error")
        }
    }

    fn discard_all(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::git_write(&self.repo, &["checkout", "--", "."], "discard_error")?;
        staging::git_write(&self.repo, &["clean", "-fd"], "discard_error")
    }

    fn stage_all(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::git_write(&self.repo, &["add", "-A"], "stage_error")
    }

    fn unstage_all(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        if staging::wsl_head_exists(&self.repo) {
            staging::git_write(&self.repo, &["restore", "--staged", "."], "unstage_error")
        } else {
            staging::git_write(
                &self.repo,
                &["rm", "--cached", "-r", "--ignore-unmatch", "."],
                "unstage_error",
            )
        }
    }

    fn stage_hunk(
        &self,
        _repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_stage_hunk(&self.repo, file_path, hunk_index)
    }

    fn unstage_hunk(
        &self,
        _repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_unstage_hunk(&self.repo, file_path, hunk_index)
    }

    fn discard_hunk(
        &self,
        _repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_discard_hunk(&self.repo, file_path, hunk_index)
    }

    fn stage_lines(
        &self,
        _repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_stage_lines(&self.repo, file_path, hunk_index, line_indices)
    }

    fn unstage_lines(
        &self,
        _repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_unstage_lines(&self.repo, file_path, hunk_index, line_indices)
    }

    fn discard_lines(
        &self,
        _repo_id: &str,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        staging::wsl_discard_lines(&self.repo, file_path, hunk_index, line_indices)
    }

    fn operation_state(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<OperationInfo, TrunkError> {
        read_model::wsl_operation_state(&self.repo)
    }

    fn list_stashes(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<StashEntry>, TrunkError> {
        Ok(read_model::wsl_refs(&self.repo)?.stashes)
    }

    fn create_commit(
        &self,
        _repo_id: &str,
        subject: &str,
        body: Option<&str>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        commit::wsl_create_commit_inner(&self.repo, subject, body)
    }

    fn amend_commit(
        &self,
        _repo_id: &str,
        subject: &str,
        body: Option<&str>,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        commit::wsl_amend_commit_inner(&self.repo, subject, body)
    }

    fn head_commit_message(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<HeadCommitMessage, TrunkError> {
        commit::wsl_get_head_commit_message_inner(&self.repo)
    }

    fn stash_save(
        &self,
        repo_id: &str,
        message: &str,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        stash::wsl_stash_save(&self.repo, message)?;
        self.commit_graph(repo_id, state_map, &HashMap::new())
    }

    fn stash_pop(
        &self,
        repo_id: &str,
        index: usize,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        stash::wsl_stash_pop(&self.repo, index)?;
        self.commit_graph(repo_id, state_map, &HashMap::new())
    }

    fn stash_apply(
        &self,
        repo_id: &str,
        index: usize,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        stash::wsl_stash_apply(&self.repo, index)?;
        self.commit_graph(repo_id, state_map, &HashMap::new())
    }

    fn stash_drop(
        &self,
        repo_id: &str,
        index: usize,
        state_map: &HashMap<String, PathBuf>,
    ) -> Result<GraphResult, TrunkError> {
        let stash_ref = format!("stash@{{{}}}", index);
        stash::run_git(&self.repo, &["stash", "drop", &stash_ref])?;
        self.commit_graph(repo_id, state_map, &HashMap::new())
    }

    fn undo_commit(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<UndoResult, TrunkError> {
        let parents = command_runner::git_output(
            &self.repo,
            &["rev-list", "--parents", "-n", "1", "HEAD"],
            "undo_error",
        )?;
        if !parents.status.success() {
            let stderr = String::from_utf8_lossy(&parents.stderr);
            return Err(TrunkError::new("undo_error", stderr.to_string()));
        }
        let parent_count = String::from_utf8_lossy(&parents.stdout)
            .split_whitespace()
            .skip(1)
            .count();
        if parent_count == 0 {
            return Err(TrunkError::new(
                "nothing_to_undo",
                "Cannot undo the initial commit",
            ));
        }
        if parent_count > 1 {
            return Err(TrunkError::new(
                "merge_commit",
                "Cannot undo a merge commit",
            ));
        }

        let message = command_runner::git_output(
            &self.repo,
            &["log", "-1", "--format=%s%x00%b"],
            "undo_error",
        )?;
        if !message.status.success() {
            let stderr = String::from_utf8_lossy(&message.stderr);
            return Err(TrunkError::new("undo_error", stderr.to_string()));
        }
        let message = String::from_utf8_lossy(&message.stdout);
        let mut parts = message.splitn(2, '\0');
        let subject = parts.next().unwrap_or("").trim_end().to_string();
        let body = parts
            .next()
            .map(str::trim_end)
            .filter(|body| !body.is_empty())
            .map(str::to_owned);

        let output =
            command_runner::git_output(&self.repo, &["reset", "--soft", "HEAD~1"], "undo_error")?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TrunkError::new("undo_error", stderr.to_string()));
        }

        Ok(UndoResult { subject, body })
    }

    fn redo_commit(
        &self,
        _repo_id: &str,
        subject: &str,
        body: Option<&str>,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<(), TrunkError> {
        let message = match body {
            Some(body) if !body.trim().is_empty() => format!("{subject}\n\n{body}"),
            _ => subject.to_owned(),
        };
        let output =
            command_runner::git_output(&self.repo, &["commit", "-m", &message], "commit_error")?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(TrunkError::new("commit_error", stderr.to_string()))
        }
    }

    fn check_undo_available(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
        _descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<bool, TrunkError> {
        let output = command_runner::git_output(
            &self.repo,
            &["rev-list", "--parents", "-n", "1", "HEAD"],
            "undo_error",
        )?;
        if !output.status.success() {
            return Ok(false);
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .skip(1)
            .count()
            == 1)
    }

    fn merge_sides(
        &self,
        _repo_id: &str,
        file_path: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<MergeSides, TrunkError> {
        let read_stage = |stage: &str| -> Result<Option<String>, TrunkError> {
            let spec = format!(":{stage}:{file_path}");
            let output =
                command_runner::git_output(&self.repo, &["show", &spec], "conflict_error")?;
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

    fn save_merge_result(
        &self,
        _repo_id: &str,
        file_path: &str,
        content: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<(), TrunkError> {
        crate::git::backend_fs::write_repo_file(&self.repo, file_path, content)?;
        let output =
            command_runner::git_output(&self.repo, &["add", "--", file_path], "stage_error")?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(TrunkError::new("stage_error", stderr.to_string()))
        }
    }

    fn rebase_todo(
        &self,
        _repo_id: &str,
        base_oid: &str,
        inclusive: bool,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<Vec<RebaseTodoItem>, TrunkError> {
        let range = if inclusive {
            let parent = command_runner::git_output(
                &self.repo,
                &["rev-parse", &format!("{base_oid}^")],
                "rebase_error",
            )?;
            if parent.status.success() {
                format!("{}..HEAD", String::from_utf8_lossy(&parent.stdout).trim())
            } else {
                "HEAD".to_owned()
            }
        } else {
            format!("{base_oid}..HEAD")
        };
        let output = command_runner::git_output(
            &self.repo,
            &[
                "log",
                "--reverse",
                "--format=%H%x00%s%x00%an%x00%at",
                &range,
            ],
            "rebase_error",
        )?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TrunkError::new("rebase_error", stderr.to_string()));
        }
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let mut fields = line.splitn(4, '\0');
                let oid = fields.next()?.to_string();
                let summary = fields.next().unwrap_or("").to_string();
                let author_name = fields.next().unwrap_or("").to_string();
                let author_timestamp = fields
                    .next()
                    .and_then(|value| value.parse::<i64>().ok())
                    .unwrap_or(0);
                Some(RebaseTodoItem {
                    short_oid: oid.chars().take(7).collect(),
                    oid,
                    summary,
                    author_name,
                    author_timestamp,
                })
            })
            .collect())
    }

    fn dirty_counts(
        &self,
        _repo_id: &str,
        _state_map: &HashMap<String, PathBuf>,
    ) -> Result<DirtyCounts, TrunkError> {
        let (staged, unstaged, conflicted) =
            read_model::status_dirty_counts(read_model::wsl_status(&self.repo)?);
        Ok(DirtyCounts {
            staged,
            unstaged,
            conflicted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
