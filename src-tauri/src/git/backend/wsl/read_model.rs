use crate::error::TrunkError;
use crate::git::read_model::{
    assign_graph_lanes, parse_porcelain_status, parse_ref_type, parse_unified_diff, short_oid,
};
use crate::git::types::{
    BranchInfo, CommitDetail, DiffRequestOptions, DiffStatus, FileDiff, GraphCommit, GraphResult,
    OperationInfo, OperationType, RefLabel, RefType, RefsResponse, RepoDescriptor, StashEntry,
    WorkingTreeStatus,
};
use crate::git::{backend_fs, command_runner};
use std::collections::HashMap;

fn git_output(repo: &RepoDescriptor, args: &[&str]) -> Result<String, TrunkError> {
    let output = command_runner::git_output(repo, args, "git_read_error")?;
    if !output.status.success() {
        return Err(TrunkError::new(
            "git_read_error",
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn git_output_owned(repo: &RepoDescriptor, args: &[String]) -> Result<String, TrunkError> {
    let output = command_runner::git_output_owned(repo, args, "git_read_error")?;
    if !output.status.success() {
        return Err(TrunkError::new(
            "git_read_error",
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn wsl_refs_by_oid(repo: &RepoDescriptor) -> Result<HashMap<String, Vec<RefLabel>>, TrunkError> {
    let head_name = git_output(repo, &["symbolic-ref", "-q", "HEAD"]).unwrap_or_default();
    let head_name = head_name.trim();
    let refs = git_output(
        repo,
        &[
            "for-each-ref",
            "--format=%(objectname)%00%(refname)",
            "refs/heads",
            "refs/remotes",
            "refs/tags",
        ],
    )?;
    let mut map: HashMap<String, Vec<RefLabel>> = HashMap::new();
    for line in refs.lines() {
        let mut parts = line.splitn(2, '\0');
        let oid = parts.next().unwrap_or("").trim();
        let name = parts.next().unwrap_or("").trim();
        if oid.is_empty() || name.is_empty() {
            continue;
        }
        let Some((ref_type, short_name)) = parse_ref_type(name) else {
            continue;
        };
        let is_head = matches!(ref_type, RefType::LocalBranch) && name == head_name;
        map.entry(oid.to_string()).or_default().push(RefLabel {
            name: name.to_string(),
            short_name,
            ref_type,
            is_head,
            color_index: 0,
        });
    }
    let stashes =
        git_output(repo, &["stash", "list", "--format=%H%x00%gd%x00%s"]).unwrap_or_default();
    for line in stashes.lines() {
        let mut parts = line.splitn(3, '\0');
        let oid = parts.next().unwrap_or("").trim();
        let short_name = parts.next().unwrap_or("").trim();
        let name = parts.next().unwrap_or("").trim();
        if oid.is_empty() || short_name.is_empty() {
            continue;
        }
        map.entry(oid.to_string()).or_default().push(RefLabel {
            name: name.to_string(),
            short_name: short_name.to_string(),
            ref_type: RefType::Stash,
            is_head: false,
            color_index: 0,
        });
    }
    Ok(map)
}

pub fn wsl_commit_graph(repo: &RepoDescriptor) -> Result<GraphResult, TrunkError> {
    let refs_by_oid = wsl_refs_by_oid(repo)?;
    let head_oid = git_output(repo, &["rev-parse", "--verify", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string());
    let log = git_output(
        repo,
        &[
            "log",
            "--all",
            "--topo-order",
            "--date-order",
            "--format=%H%x00%P%x00%s%x00%b%x00%an%x00%ae%x00%at%x1e",
        ],
    )?;
    let mut commits = Vec::new();
    for (idx, record) in log.split('\x1e').enumerate() {
        let record = record.trim_matches('\n');
        if record.is_empty() {
            continue;
        }
        let mut fields = record.splitn(7, '\0');
        let oid = fields.next().unwrap_or("").to_string();
        if oid.is_empty() {
            continue;
        }
        let parent_oids: Vec<String> = fields
            .next()
            .unwrap_or("")
            .split_whitespace()
            .map(ToString::to_string)
            .collect();
        let summary = fields.next().unwrap_or("").to_string();
        let body = fields
            .next()
            .map(str::trim)
            .filter(|body| !body.is_empty())
            .map(ToString::to_string);
        let author_name = fields.next().unwrap_or("").to_string();
        let author_email = fields.next().unwrap_or("").to_string();
        let author_timestamp = fields
            .next()
            .unwrap_or("0")
            .trim()
            .parse::<i64>()
            .unwrap_or(0);
        let refs = refs_by_oid.get(&oid).cloned().unwrap_or_default();
        let is_stash = refs
            .iter()
            .any(|label| matches!(label.ref_type, RefType::Stash));
        commits.push(GraphCommit {
            short_oid: short_oid(&oid),
            is_head: head_oid.as_deref() == Some(oid.as_str()),
            is_merge: parent_oids.len() >= 2,
            is_branch_tip: !refs.is_empty(),
            oid,
            summary,
            body,
            author_name,
            author_email,
            author_timestamp,
            parent_oids,
            column: idx,
            color_index: idx,
            edges: Vec::new(),
            refs,
            is_stash,
        });
    }
    let max_columns = assign_graph_lanes(&mut commits);
    Ok(GraphResult {
        commits,
        max_columns,
    })
}

pub fn wsl_status(repo: &RepoDescriptor) -> Result<WorkingTreeStatus, TrunkError> {
    let output = git_output(repo, &["status", "--porcelain=v1", "-z"])?;
    Ok(parse_porcelain_status(&output))
}

fn diff_args(base: &[&str], file_path: Option<&str>, options: &DiffRequestOptions) -> Vec<String> {
    let mut args: Vec<String> = base.iter().map(|arg| arg.to_string()).collect();
    args.push("--no-color".to_string());
    args.push("--find-renames".to_string());
    args.push(format!(
        "--unified={}",
        if options.show_full_file {
            100_000
        } else {
            options.context_lines
        }
    ));
    if options.ignore_whitespace {
        args.push("--ignore-all-space".to_string());
    }
    if let Some(file_path) = file_path {
        args.push("--".to_string());
        args.push(file_path.to_string());
    }
    args
}

pub fn wsl_diff_unstaged(
    repo: &RepoDescriptor,
    file_path: &str,
    options: &DiffRequestOptions,
) -> Result<Vec<FileDiff>, TrunkError> {
    let args = diff_args(&["diff"], Some(file_path), options);
    let output = git_output_owned(repo, &args)?;
    if !output.trim().is_empty() {
        return Ok(parse_unified_diff(&output));
    }

    let untracked_args = vec![
        "ls-files".to_string(),
        "--others".to_string(),
        "--exclude-standard".to_string(),
        "--".to_string(),
        file_path.to_string(),
    ];
    if git_output_owned(repo, &untracked_args)?.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut args = diff_args(&["diff"], None, options);
    args.push("--no-index".to_string());
    args.push("/dev/null".to_string());
    args.push(file_path.to_string());
    let output = command_runner::git_output_owned(repo, &args, "git_read_error")?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut diffs = parse_unified_diff(&stdout);
    for diff in &mut diffs {
        diff.status = DiffStatus::Untracked;
    }
    Ok(diffs)
}

pub fn wsl_diff_staged(
    repo: &RepoDescriptor,
    file_path: &str,
    options: &DiffRequestOptions,
) -> Result<Vec<FileDiff>, TrunkError> {
    let args = diff_args(&["diff", "--cached"], Some(file_path), options);
    Ok(parse_unified_diff(&git_output_owned(repo, &args)?))
}

pub fn wsl_diff_commit(
    repo: &RepoDescriptor,
    oid: &str,
    file_path: Option<&str>,
    options: &DiffRequestOptions,
) -> Result<Vec<FileDiff>, TrunkError> {
    let revision = format!("{}^!", oid);
    let args = diff_args(&["diff", &revision], file_path, options);
    Ok(parse_unified_diff(&git_output_owned(repo, &args)?))
}

pub fn wsl_list_commit_files(
    repo: &RepoDescriptor,
    oid: &str,
) -> Result<Vec<FileDiff>, TrunkError> {
    let output = git_output(
        repo,
        &[
            "diff-tree",
            "--root",
            "--no-commit-id",
            "--name-status",
            "-r",
            oid,
        ],
    )?;
    let mut files = Vec::new();
    for line in output.lines() {
        let mut parts = line.split('\t');
        let status_raw = parts.next().unwrap_or("");
        let path = parts.next_back().unwrap_or("").to_string();
        if path.is_empty() {
            continue;
        }
        let status = match status_raw.chars().next().unwrap_or('M') {
            'A' => DiffStatus::Added,
            'D' => DiffStatus::Deleted,
            'R' => DiffStatus::Renamed,
            'C' => DiffStatus::Copied,
            'M' => DiffStatus::Modified,
            _ => DiffStatus::Unknown,
        };
        files.push(FileDiff {
            path,
            status,
            is_binary: false,
            hunks: Vec::new(),
        });
    }
    Ok(files)
}

pub fn wsl_commit_detail(repo: &RepoDescriptor, oid: &str) -> Result<CommitDetail, TrunkError> {
    let output = git_output(
        repo,
        &[
            "show",
            "-s",
            "--format=%H%x00%s%x00%b%x00%an%x00%ae%x00%at%x00%cn%x00%ce%x00%ct%x00%P",
            oid,
        ],
    )?;
    let mut fields = output.trim_end_matches('\n').splitn(10, '\0');
    let oid = fields.next().unwrap_or("").to_string();
    let summary = fields.next().unwrap_or("").to_string();
    let body = fields
        .next()
        .map(str::trim)
        .filter(|body| !body.is_empty())
        .map(ToString::to_string);
    let author_name = fields.next().unwrap_or("").to_string();
    let author_email = fields.next().unwrap_or("").to_string();
    let author_timestamp = fields.next().unwrap_or("0").parse().unwrap_or(0);
    let committer_name = fields.next().unwrap_or("").to_string();
    let committer_email = fields.next().unwrap_or("").to_string();
    let committer_timestamp = fields.next().unwrap_or("0").parse().unwrap_or(0);
    let parent_oids = fields
        .next()
        .unwrap_or("")
        .split_whitespace()
        .map(ToString::to_string)
        .collect();
    Ok(CommitDetail {
        short_oid: short_oid(&oid),
        oid,
        summary,
        body,
        author_name,
        author_email,
        author_timestamp,
        committer_name,
        committer_email,
        committer_timestamp,
        parent_oids,
    })
}

pub fn wsl_refs(repo: &RepoDescriptor) -> Result<RefsResponse, TrunkError> {
    let head = git_output(repo, &["branch", "--show-current"]).unwrap_or_default();
    let head = head.trim();
    let branch_output = git_output(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)%00%(upstream:short)%00%(committerdate:unix)",
            "refs/heads",
        ],
    )?;
    let mut local = Vec::new();
    for line in branch_output.lines() {
        let mut fields = line.splitn(3, '\0');
        let name = fields.next().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let upstream = fields
            .next()
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
        let last_commit_timestamp = fields.next().unwrap_or("0").parse().unwrap_or(0);
        let (ahead, behind) = if let Some(upstream) = upstream.as_deref() {
            let counts = git_output(
                repo,
                &[
                    "rev-list",
                    "--left-right",
                    "--count",
                    &format!("{}...{}", name, upstream),
                ],
            )
            .unwrap_or_default();
            let mut counts = counts.split_whitespace();
            (
                counts.next().unwrap_or("0").parse().unwrap_or(0),
                counts.next().unwrap_or("0").parse().unwrap_or(0),
            )
        } else {
            (0, 0)
        };
        local.push(BranchInfo {
            is_head: name == head,
            name,
            upstream,
            ahead,
            behind,
            last_commit_timestamp,
        });
    }

    let remote_output = git_output(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)%00%(committerdate:unix)",
            "refs/remotes",
        ],
    )?;
    let remote = remote_output
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(2, '\0');
            let name = fields.next()?.to_string();
            if name.ends_with("/HEAD") {
                return None;
            }
            Some(BranchInfo {
                name,
                is_head: false,
                upstream: None,
                ahead: 0,
                behind: 0,
                last_commit_timestamp: fields.next().unwrap_or("0").parse().unwrap_or(0),
            })
        })
        .collect();

    let tags = git_output(repo, &["tag", "--list"])?
        .lines()
        .map(|name| RefLabel {
            name: format!("refs/tags/{}", name),
            short_name: name.to_string(),
            ref_type: RefType::Tag,
            is_head: false,
            color_index: 0,
        })
        .collect();
    let stashes = git_output(repo, &["stash", "list", "--format=%gd%x00%H%x00%P%x00%s"])
        .unwrap_or_default()
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let mut fields = line.splitn(4, '\0');
            let short_name = fields.next()?.to_string();
            let oid = fields.next()?.to_string();
            let parent_oid = fields
                .next()
                .unwrap_or("")
                .split_whitespace()
                .next()
                .map(ToString::to_string);
            let name = fields.next().unwrap_or("").to_string();
            Some(StashEntry {
                index,
                name,
                short_name,
                oid,
                parent_oid,
            })
        })
        .collect();
    Ok(RefsResponse {
        local,
        remote,
        tags,
        stashes,
    })
}

fn git_verify_ref(repo: &RepoDescriptor, name: &str) -> bool {
    git_output(repo, &["rev-parse", "-q", "--verify", name])
        .ok()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn read_wsl_git_file(repo: &RepoDescriptor, path: &str) -> Option<String> {
    backend_fs::read_git_file(repo, path)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn wsl_rebase_dir(repo: &RepoDescriptor) -> Option<&'static str> {
    if read_wsl_git_file(repo, "rebase-merge/head-name").is_some()
        || read_wsl_git_file(repo, "rebase-merge/msgnum").is_some()
    {
        Some("rebase-merge")
    } else if read_wsl_git_file(repo, "rebase-apply/head-name").is_some()
        || read_wsl_git_file(repo, "rebase-apply/msgnum").is_some()
    {
        Some("rebase-apply")
    } else {
        None
    }
}

fn resolve_wsl_oid_to_branch(repo: &RepoDescriptor, oid: &str) -> Option<String> {
    let refs = git_output(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname:short)%00%(objectname)",
            "refs/heads",
        ],
    )
    .ok()?;
    for line in refs.lines() {
        let mut fields = line.splitn(2, '\0');
        let name = fields.next().unwrap_or("");
        let object = fields.next().unwrap_or("");
        if object == oid {
            return Some(name.to_string());
        }
    }
    Some(short_oid(oid))
}

pub fn wsl_operation_state(repo: &RepoDescriptor) -> Result<OperationInfo, TrunkError> {
    let files = git_output(repo, &["ls-files", "-u"]).unwrap_or_default();
    let has_conflicts = !files.trim().is_empty();
    let target_branch = git_output(repo, &["branch", "--show-current"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    if let Some(rebase_dir) = wsl_rebase_dir(repo) {
        let head_name = read_wsl_git_file(repo, &format!("{rebase_dir}/head-name"))
            .map(|value| value.replace("refs/heads/", ""));
        let onto_branch = read_wsl_git_file(repo, &format!("{rebase_dir}/onto"))
            .and_then(|oid| resolve_wsl_oid_to_branch(repo, &oid));
        let msgnum = read_wsl_git_file(repo, &format!("{rebase_dir}/msgnum"));
        let end = read_wsl_git_file(repo, &format!("{rebase_dir}/end"));
        let progress = match (msgnum, end) {
            (Some(m), Some(e)) => Some(format!("{m}/{e}")),
            _ => None,
        };
        let rebase_message = read_wsl_git_file(repo, &format!("{rebase_dir}/message"));
        return Ok(OperationInfo {
            op_type: OperationType::Rebase,
            source_branch: head_name,
            target_branch: onto_branch.or(target_branch),
            progress,
            source_color_index: None,
            target_color_index: None,
            rebase_message,
        });
    }

    if git_verify_ref(repo, "CHERRY_PICK_HEAD") {
        return Ok(OperationInfo {
            op_type: OperationType::CherryPick,
            source_branch: None,
            target_branch,
            progress: None,
            source_color_index: None,
            target_color_index: None,
            rebase_message: None,
        });
    }

    if git_verify_ref(repo, "REVERT_HEAD") {
        return Ok(OperationInfo {
            op_type: OperationType::Revert,
            source_branch: None,
            target_branch,
            progress: None,
            source_color_index: None,
            target_color_index: None,
            rebase_message: None,
        });
    }

    if has_conflicts || git_verify_ref(repo, "MERGE_HEAD") {
        let merge_msg = backend_fs::read_git_file(repo, "MERGE_MSG").ok();
        let source_branch = merge_msg.as_deref().and_then(|message| {
            crate::commands::operation_state::extract_merge_source(Some(message))
        });
        Ok(OperationInfo {
            op_type: OperationType::Merge,
            source_branch,
            target_branch,
            progress: None,
            source_color_index: None,
            target_color_index: None,
            rebase_message: None,
        })
    } else {
        Ok(OperationInfo {
            op_type: OperationType::None,
            source_branch: None,
            target_branch,
            progress: None,
            source_color_index: None,
            target_color_index: None,
            rebase_message: None,
        })
    }
}
