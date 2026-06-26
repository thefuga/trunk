use super::{GitBackend, PreparedOpenRepo};
use crate::commands::staging::DirtyCounts;
use crate::commands::{branches, diff};
use crate::error::TrunkError;
use crate::git::backend_fs::BackendTempDir;
use crate::git::command_runner;
use crate::git::command_runner::GitCommandSpec;
use crate::git::types::{
    CommitDetail, DiffRequestOptions, FileDiff, GraphResult, HeadCommitMessage, MergeSides,
    OperationInfo, RebaseTodoItem, RefsResponse, RepoDescriptor, RepoLocator, StashEntry,
    UndoResult, WorkingTreeStatus,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(not(all(target_os = "windows", feature = "wsl")))]
compile_error!("git::backend::wsl requires Windows and the `wsl` feature");

pub(crate) mod command;
mod commit;
pub(crate) mod fs;
pub(super) mod poller;
mod read_model;
mod staging;
mod stash;

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

    fn command_spec(
        &self,
        _repo: &RepoDescriptor,
        git_args: &[&str],
    ) -> Result<GitCommandSpec, TrunkError> {
        Ok(command::spec_for_repo(&self.repo, git_args))
    }

    fn with_interactive_rebase_editor_env(
        &self,
        spec: GitCommandSpec,
        _repo: &RepoDescriptor,
    ) -> GitCommandSpec {
        command::with_interactive_rebase_editor_env(spec)
    }

    fn read_repo_file(
        &self,
        _repo: &RepoDescriptor,
        relative_path: &str,
    ) -> Result<String, TrunkError> {
        fs::read_repo_file(&self.repo, relative_path)
    }

    fn write_repo_file(
        &self,
        _repo: &RepoDescriptor,
        relative_path: &str,
        content: &str,
    ) -> Result<(), TrunkError> {
        fs::write_repo_file(&self.repo, relative_path, content)
    }

    fn read_absolute_file(&self, _repo: &RepoDescriptor, path: &str) -> Result<String, TrunkError> {
        fs::read_absolute_file(&self.repo, path)
    }

    fn write_absolute_file(
        &self,
        _repo: &RepoDescriptor,
        path: &str,
        content: &str,
    ) -> Result<(), TrunkError> {
        fs::write_absolute_file(&self.repo, path, content)
    }

    fn delete_repo_file(
        &self,
        _repo: &RepoDescriptor,
        relative_path: &str,
    ) -> Result<(), TrunkError> {
        fs::delete_repo_file(&self.repo, relative_path)
    }

    fn create_temp_dir(
        &self,
        _repo: &RepoDescriptor,
        name: &str,
    ) -> Result<BackendTempDir, TrunkError> {
        fs::create_temp_dir(&self.repo, name)
    }

    fn poll_token(&self, _repo: &RepoDescriptor) -> Result<Option<String>, TrunkError> {
        fs::poll_token(&self.repo)
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
            crate::git::read_model::status_dirty_counts(read_model::wsl_status(&self.repo)?);
        Ok(DirtyCounts {
            staged,
            unstaged,
            conflicted,
        })
    }
}
