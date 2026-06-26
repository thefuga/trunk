use super::{GitBackend, PreparedOpenRepo};
use crate::commands::staging::DirtyCounts;
use crate::commands::{
    branches, commit, commit_actions, diff, merge_editor, operation_state, staging, stash,
};
use crate::error::TrunkError;
use crate::git::backend_fs::BackendTempDir;
use crate::git::command_runner::GitCommandSpec;
use crate::git::types::{
    CommitDetail, DiffRequestOptions, FileDiff, GraphResult, HeadCommitMessage, MergeSides,
    OperationInfo, RebaseTodoItem, RefsResponse, RepoDescriptor, RepoLocator, StashEntry,
    UndoResult, WorkingTreeStatus,
};
use crate::git::{graph, repository};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalBackend;

fn local_path(repo: &RepoDescriptor) -> Result<&str, TrunkError> {
    match &repo.locator {
        RepoLocator::Local { path } => Ok(path),
        _ => Err(TrunkError::new(
            "backend_descriptor_mismatch",
            "Local backend received a non-local descriptor",
        )),
    }
}

impl GitBackend for LocalBackend {
    fn prepare_open_repo(
        &self,
        descriptor: RepoDescriptor,
    ) -> Result<PreparedOpenRepo, TrunkError> {
        let execution_path = local_path(&descriptor)?.to_string();
        Ok(PreparedOpenRepo {
            descriptor,
            execution_path,
            use_native_watcher: true,
        })
    }

    fn command_spec(
        &self,
        repo: &RepoDescriptor,
        git_args: &[&str],
    ) -> Result<GitCommandSpec, TrunkError> {
        Ok(GitCommandSpec {
            program: "git".to_string(),
            args: git_args.iter().map(|arg| arg.to_string()).collect(),
            current_dir: Some(PathBuf::from(local_path(repo)?)),
            env: Vec::new(),
        })
    }

    fn read_repo_file(
        &self,
        repo: &RepoDescriptor,
        relative_path: &str,
    ) -> Result<String, TrunkError> {
        std::fs::read_to_string(Path::new(local_path(repo)?).join(relative_path))
            .map_err(|e| TrunkError::new("io_error", e.to_string()))
    }

    fn write_repo_file(
        &self,
        repo: &RepoDescriptor,
        relative_path: &str,
        content: &str,
    ) -> Result<(), TrunkError> {
        std::fs::write(Path::new(local_path(repo)?).join(relative_path), content)
            .map_err(|e| TrunkError::new("write_error", e.to_string()))
    }

    fn read_absolute_file(&self, repo: &RepoDescriptor, path: &str) -> Result<String, TrunkError> {
        local_path(repo)?;
        std::fs::read_to_string(path).map_err(|e| TrunkError::new("io_error", e.to_string()))
    }

    fn write_absolute_file(
        &self,
        repo: &RepoDescriptor,
        path: &str,
        content: &str,
    ) -> Result<(), TrunkError> {
        local_path(repo)?;
        std::fs::write(path, content).map_err(|e| TrunkError::new("write_error", e.to_string()))
    }

    fn delete_repo_file(
        &self,
        repo: &RepoDescriptor,
        relative_path: &str,
    ) -> Result<(), TrunkError> {
        match std::fs::remove_file(Path::new(local_path(repo)?).join(relative_path)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(TrunkError::new("io_error", e.to_string())),
        }
    }

    fn create_temp_dir(
        &self,
        repo: &RepoDescriptor,
        name: &str,
    ) -> Result<BackendTempDir, TrunkError> {
        local_path(repo)?;
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&path).map_err(|e| TrunkError::new("io_error", e.to_string()))?;
        Ok(BackendTempDir::Local(path))
    }

    fn poll_token(&self, repo: &RepoDescriptor) -> Result<Option<String>, TrunkError> {
        local_path(repo)?;
        Ok(None)
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
