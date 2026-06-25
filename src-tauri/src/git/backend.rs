use crate::commands::staging::DirtyCounts;
use crate::commands::{branches, commit, commit_actions, diff, operation_state, staging, stash};
use crate::error::TrunkError;
use crate::git::read_model;
use crate::git::types::{
    CommitDetail, DiffRequestOptions, FileDiff, GraphResult, HeadCommitMessage, OperationInfo,
    RefsResponse, RepoDescriptor, StashEntry, UndoResult, WorkingTreeStatus,
};
use std::collections::HashMap;
use std::path::PathBuf;

fn backend_method_not_implemented<T>(method: &str) -> Result<T, TrunkError> {
    Err(TrunkError::new(
        "backend_method_not_implemented",
        format!("Git backend method `{method}` is not implemented"),
    ))
}

pub trait GitBackend: Send + Sync {
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
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalBackend;

impl GitBackend for LocalBackend {
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
        crate::commands::refresh_graph_from_state(repo_id, state_map, &HashMap::new())
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
        commit_actions::undo_commit_inner(repo_id, state_map, descriptor_map)
    }

    fn redo_commit(
        &self,
        repo_id: &str,
        subject: &str,
        body: Option<&str>,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<(), TrunkError> {
        commit_actions::redo_commit_inner_with_descriptors(
            repo_id,
            subject,
            body,
            state_map,
            descriptor_map,
        )
    }

    fn check_undo_available(
        &self,
        repo_id: &str,
        state_map: &HashMap<String, PathBuf>,
        descriptor_map: &HashMap<String, RepoDescriptor>,
    ) -> Result<bool, TrunkError> {
        commit_actions::check_undo_available_inner_with_descriptors(
            repo_id,
            state_map,
            descriptor_map,
        )
    }
}

#[derive(Debug, Clone)]
pub struct WslBackend {
    repo: RepoDescriptor,
}

impl WslBackend {
    pub fn new(repo: RepoDescriptor) -> Self {
        Self { repo }
    }
}

impl GitBackend for WslBackend {
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
