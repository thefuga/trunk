use super::read_model;
use crate::error::TrunkError;
use crate::git::command_runner;
use crate::git::types::{RepoDescriptor, StashEntry};

fn git_command_error(code: &str, output: std::process::Output) -> TrunkError {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    TrunkError::new(code, if stderr.is_empty() { stdout } else { stderr })
}

pub(crate) fn run_git(repo: &RepoDescriptor, args: &[&str]) -> Result<(), TrunkError> {
    let output = command_runner::git_output(repo, args, "git_error")?;
    if output.status.success() {
        Ok(())
    } else {
        Err(git_command_error("git_error", output))
    }
}

fn wsl_has_conflicts(repo: &RepoDescriptor) -> Result<bool, TrunkError> {
    let output = command_runner::git_output(repo, &["ls-files", "-u"], "git_error")?;
    if output.status.success() {
        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    } else {
        Err(git_command_error("git_error", output))
    }
}

fn wsl_list_stashes(repo: &RepoDescriptor) -> Result<Vec<StashEntry>, TrunkError> {
    Ok(read_model::wsl_refs(repo)?.stashes)
}

pub(crate) fn wsl_stash_save(repo: &RepoDescriptor, message: &str) -> Result<(), TrunkError> {
    let status = command_runner::git_output(repo, &["status", "--porcelain=v1"], "git_error")?;
    if !status.status.success() {
        return Err(git_command_error("git_error", status));
    }
    if String::from_utf8_lossy(&status.stdout).trim().is_empty() {
        return Err(TrunkError::new(
            "nothing_to_stash",
            "Nothing to stash — working tree is clean",
        ));
    }

    if message.trim().is_empty() {
        run_git(repo, &["stash", "push"])
    } else {
        run_git(repo, &["stash", "push", "-m", message])
    }
}

pub(crate) fn wsl_stash_apply(repo: &RepoDescriptor, index: usize) -> Result<(), TrunkError> {
    let stash_ref = format!("stash@{{{}}}", index);
    run_git(repo, &["stash", "apply", &stash_ref]).map_err(|e| {
        if wsl_has_conflicts(repo).unwrap_or(false) {
            TrunkError::new(
                "conflict_state",
                "Stash applied with conflicts — resolve conflicts before continuing",
            )
        } else {
            e
        }
    })?;
    if wsl_has_conflicts(repo)? {
        return Err(TrunkError::new(
            "conflict_state",
            "Stash applied with conflicts — resolve conflicts before continuing",
        ));
    }
    Ok(())
}

pub(crate) fn wsl_stash_pop(repo: &RepoDescriptor, index: usize) -> Result<(), TrunkError> {
    let stash_ref = format!("stash@{{{}}}", index);
    run_git(repo, &["stash", "pop", &stash_ref]).map_err(|e| {
        if wsl_has_conflicts(repo).unwrap_or(false) {
            TrunkError::new("conflict_state", "Stash applied with conflicts — resolve conflicts before continuing. Note: stash was NOT removed.")
        } else {
            e
        }
    })?;
    if wsl_has_conflicts(repo)? {
        return Err(TrunkError::new("conflict_state", "Stash applied with conflicts — resolve conflicts before continuing. Note: stash was NOT removed."));
    }
    Ok(())
}
