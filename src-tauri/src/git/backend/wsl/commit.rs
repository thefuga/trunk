use crate::error::TrunkError;
use crate::git::command_runner;
use crate::git::types::{HeadCommitMessage, RepoDescriptor};

fn git_write(repo: &RepoDescriptor, args: &[String], code: &str) -> Result<(), TrunkError> {
    let output = command_runner::git_output_owned(repo, args, code)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(TrunkError::new(
        code,
        if stderr.is_empty() {
            "Git commit operation failed".to_string()
        } else {
            stderr
        },
    ))
}

pub(crate) fn wsl_create_commit_inner(
    repo: &RepoDescriptor,
    subject: &str,
    body: Option<&str>,
) -> Result<(), TrunkError> {
    let mut args = vec!["commit".to_string(), "-m".to_string(), subject.to_string()];
    if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
        args.push("-m".to_string());
        args.push(body.to_string());
    }
    git_write(repo, &args, "commit_error")
}

pub(crate) fn wsl_amend_commit_inner(
    repo: &RepoDescriptor,
    subject: &str,
    body: Option<&str>,
) -> Result<(), TrunkError> {
    let mut args = vec![
        "commit".to_string(),
        "--amend".to_string(),
        "-m".to_string(),
        subject.to_string(),
    ];
    if let Some(body) = body.filter(|value| !value.trim().is_empty()) {
        args.push("-m".to_string());
        args.push(body.to_string());
    }
    git_write(repo, &args, "commit_error")
}

pub(crate) fn wsl_get_head_commit_message_inner(
    repo: &RepoDescriptor,
) -> Result<HeadCommitMessage, TrunkError> {
    let output = command_runner::git_output(
        repo,
        &["log", "-1", "--format=%s%x00%b"],
        "commit_message_error",
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(TrunkError::new(
            "commit_message_error",
            if stderr.is_empty() {
                "Could not read HEAD commit message".to_string()
            } else {
                stderr
            },
        ));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut parts = text.trim_end_matches('\n').splitn(2, '\0');
    Ok(HeadCommitMessage {
        subject: parts.next().unwrap_or("").to_string(),
        body: parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string),
    })
}
