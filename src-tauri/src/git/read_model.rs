use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::TrunkError;
use crate::git::command_runner;
use crate::git::types::{
    BranchInfo, CommitDetail, DiffHunk, DiffLine, DiffOrigin, DiffRequestOptions, DiffStatus,
    EdgeType, FileDiff, FileStatus, FileStatusType, GraphCommit, GraphEdge, GraphResult,
    OperationInfo, OperationType, RefLabel, RefType, RefsResponse, RepoDescriptor, RepoLocator,
    StashEntry, WorkingTreeStatus,
};

pub enum ReadBackend {
    Local(PathBuf),
    Wsl(RepoDescriptor),
}

pub fn backend_from_state(
    repo_id: &str,
    state_map: &HashMap<String, PathBuf>,
    descriptor_map: &HashMap<String, RepoDescriptor>,
) -> Result<ReadBackend, TrunkError> {
    let descriptor =
        crate::commands::repo_descriptor_from_state(repo_id, state_map, descriptor_map)?;
    match descriptor.locator {
        RepoLocator::Local { .. } => {
            let path = state_map.get(repo_id).ok_or_else(|| {
                TrunkError::new("not_open", format!("Repository not open: {}", repo_id))
            })?;
            Ok(ReadBackend::Local(path.clone()))
        }
        RepoLocator::Wsl { .. } => Ok(ReadBackend::Wsl(descriptor)),
    }
}

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

fn short_oid(oid: &str) -> String {
    oid.chars().take(7).collect()
}

fn parse_ref_type(full_name: &str) -> Option<(RefType, String)> {
    if let Some(short) = full_name.strip_prefix("refs/heads/") {
        Some((RefType::LocalBranch, short.to_string()))
    } else if let Some(short) = full_name.strip_prefix("refs/remotes/") {
        if short.ends_with("/HEAD") {
            None
        } else {
            Some((RefType::RemoteBranch, short.to_string()))
        }
    } else {
        full_name
            .strip_prefix("refs/tags/")
            .map(|short| (RefType::Tag, short.to_string()))
    }
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

fn find_free_column(active_lanes: &mut Vec<Option<String>>) -> usize {
    if let Some(index) = active_lanes.iter().position(Option::is_none) {
        index
    } else {
        active_lanes.push(None);
        active_lanes.len() - 1
    }
}

fn assign_graph_lanes(commits: &mut [GraphCommit]) -> usize {
    let mut active_lanes: Vec<Option<String>> = Vec::new();
    let mut pending_parents: HashMap<String, usize> = HashMap::new();
    let mut max_columns = 0usize;

    for commit in commits {
        let column = pending_parents
            .remove(&commit.oid)
            .unwrap_or_else(|| find_free_column(&mut active_lanes));
        if column >= active_lanes.len() {
            active_lanes.resize(column + 1, None);
        }
        max_columns = max_columns.max(active_lanes.len());

        let mut edges = Vec::new();
        for (other_column, occupant) in active_lanes.iter().enumerate() {
            if other_column != column && occupant.is_some() {
                edges.push(GraphEdge {
                    from_column: other_column,
                    to_column: other_column,
                    edge_type: EdgeType::Straight,
                    color_index: other_column,
                    dashed: false,
                });
            }
        }

        active_lanes[column] = None;
        for (parent_index, parent_oid) in commit.parent_oids.iter().enumerate() {
            let parent_column = if parent_index == 0 {
                column
            } else {
                pending_parents
                    .get(parent_oid)
                    .copied()
                    .unwrap_or_else(|| find_free_column(&mut active_lanes))
            };
            pending_parents
                .entry(parent_oid.clone())
                .or_insert(parent_column);
            if parent_column >= active_lanes.len() {
                active_lanes.resize(parent_column + 1, None);
            }
            active_lanes[parent_column] = Some(parent_oid.clone());
            max_columns = max_columns.max(active_lanes.len());

            let edge_type = if parent_column == column {
                EdgeType::Straight
            } else if parent_column < column {
                EdgeType::MergeLeft
            } else {
                EdgeType::MergeRight
            };
            edges.push(GraphEdge {
                from_column: column,
                to_column: parent_column,
                edge_type,
                color_index: parent_column,
                dashed: false,
            });
        }

        commit.column = column;
        commit.color_index = column;
        commit.edges = edges;
    }

    max_columns.max(1)
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

fn parse_status_type(code: char) -> Option<FileStatusType> {
    match code {
        'A' | '?' => Some(FileStatusType::New),
        'M' => Some(FileStatusType::Modified),
        'D' => Some(FileStatusType::Deleted),
        'R' | 'C' => Some(FileStatusType::Renamed),
        'T' => Some(FileStatusType::Typechange),
        'U' => Some(FileStatusType::Conflicted),
        _ => None,
    }
}

pub fn wsl_status(repo: &RepoDescriptor) -> Result<WorkingTreeStatus, TrunkError> {
    let output = git_output(repo, &["status", "--porcelain=v1", "-z"])?;
    Ok(parse_porcelain_status(&output))
}

fn parse_porcelain_status(output: &str) -> WorkingTreeStatus {
    let mut unstaged = Vec::new();
    let mut staged = Vec::new();
    let mut conflicted = Vec::new();
    let mut fields = output.split('\0').filter(|part| !part.is_empty());
    while let Some(entry) = fields.next() {
        if entry.len() < 3 {
            continue;
        }
        let x = entry.as_bytes()[0] as char;
        let y = entry.as_bytes()[1] as char;
        let mut path = entry[3..].to_string();
        if x == 'R' || x == 'C' {
            if let Some(new_path) = fields.next() {
                path = new_path.to_string();
            }
        }
        if x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D') {
            conflicted.push(FileStatus {
                path,
                status: FileStatusType::Conflicted,
                is_binary: false,
            });
            continue;
        }
        if let Some(status) = parse_status_type(x) {
            if x != '?' {
                staged.push(FileStatus {
                    path: path.clone(),
                    status,
                    is_binary: false,
                });
            }
        }
        if let Some(status) = parse_status_type(y) {
            unstaged.push(FileStatus {
                path,
                status,
                is_binary: false,
            });
        } else if x == '?' {
            unstaged.push(FileStatus {
                path,
                status: FileStatusType::New,
                is_binary: false,
            });
        }
    }
    WorkingTreeStatus {
        unstaged,
        staged,
        conflicted,
    }
}

pub fn status_dirty_counts(status: WorkingTreeStatus) -> (usize, usize, usize) {
    (
        status.staged.len(),
        status.unstaged.len(),
        status.conflicted.len(),
    )
}

fn diff_status_from_header(line: &str) -> DiffStatus {
    if line.starts_with("new file mode") {
        DiffStatus::Added
    } else if line.starts_with("deleted file mode") {
        DiffStatus::Deleted
    } else if line.starts_with("rename ") {
        DiffStatus::Renamed
    } else {
        DiffStatus::Modified
    }
}

fn parse_hunk_header(header: &str) -> (u32, u32, u32, u32) {
    let parts: Vec<&str> = header.split_whitespace().collect();
    let parse_part = |raw: Option<&&str>| -> (u32, u32) {
        let raw = raw.unwrap_or(&"").trim_start_matches(['-', '+']);
        let mut split = raw.splitn(2, ',');
        let start = split.next().unwrap_or("0").parse().unwrap_or(0);
        let lines = split.next().unwrap_or("1").parse().unwrap_or(1);
        (start, lines)
    };
    let (old_start, old_lines) = parse_part(parts.get(1));
    let (new_start, new_lines) = parse_part(parts.get(2));
    (old_start, old_lines, new_start, new_lines)
}

pub fn parse_unified_diff(text: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut current: Option<FileDiff> = None;
    let mut old_lineno = 0u32;
    let mut new_lineno = 0u32;

    for line in text.lines() {
        if line.starts_with("diff --git ") {
            if let Some(file) = current.take() {
                files.push(file);
            }
            let path = line
                .split(" b/")
                .nth(1)
                .or_else(|| line.split_whitespace().last())
                .unwrap_or("")
                .trim_start_matches("b/")
                .to_string();
            current = Some(FileDiff {
                path,
                status: DiffStatus::Modified,
                is_binary: false,
                hunks: Vec::new(),
            });
        } else if let Some(file) = current.as_mut() {
            if line.starts_with("Binary files ") {
                file.is_binary = true;
            } else if line.starts_with("new file mode")
                || line.starts_with("deleted file mode")
                || line.starts_with("rename ")
            {
                file.status = diff_status_from_header(line);
            } else if let Some(path) = line.strip_prefix("+++ b/") {
                file.path = path.to_string();
            } else if line.starts_with("@@ ") {
                let (os, ol, ns, nl) = parse_hunk_header(line);
                old_lineno = os;
                new_lineno = ns;
                file.hunks.push(DiffHunk {
                    header: format!("{}\n", line),
                    old_start: os,
                    old_lines: ol,
                    new_start: ns,
                    new_lines: nl,
                    lines: Vec::new(),
                });
            } else if let Some(hunk) = file.hunks.last_mut() {
                let (origin, content, old_line, new_line) =
                    if let Some(content) = line.strip_prefix('+') {
                        let line_no = new_lineno;
                        new_lineno += 1;
                        (
                            DiffOrigin::Add,
                            format!("{}\n", content),
                            None,
                            Some(line_no),
                        )
                    } else if let Some(content) = line.strip_prefix('-') {
                        let line_no = old_lineno;
                        old_lineno += 1;
                        (
                            DiffOrigin::Delete,
                            format!("{}\n", content),
                            Some(line_no),
                            None,
                        )
                    } else if let Some(content) = line.strip_prefix(' ') {
                        let old_line = old_lineno;
                        let new_line = new_lineno;
                        old_lineno += 1;
                        new_lineno += 1;
                        (
                            DiffOrigin::Context,
                            format!("{}\n", content),
                            Some(old_line),
                            Some(new_line),
                        )
                    } else {
                        continue;
                    };
                hunk.lines.push(DiffLine {
                    origin,
                    content,
                    old_lineno: old_line,
                    new_lineno: new_line,
                    spans: Vec::new(),
                });
            }
        }
    }
    if let Some(file) = current {
        files.push(file);
    }
    files
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

pub fn wsl_operation_state(repo: &RepoDescriptor) -> Result<OperationInfo, TrunkError> {
    let files = git_output(repo, &["ls-files", "-u"]).unwrap_or_default();
    let has_conflicts = !files.trim().is_empty();
    let status = git_output(repo, &["status"]).unwrap_or_default();
    let status_lower = status.to_lowercase();
    let op_type = if status_lower.contains("rebase") {
        OperationType::Rebase
    } else if status_lower.contains("cherry-pick") {
        OperationType::CherryPick
    } else if status_lower.contains("revert") {
        OperationType::Revert
    } else if has_conflicts
        || !git_output_owned(
            repo,
            &[
                "rev-parse".into(),
                "-q".into(),
                "--verify".into(),
                "MERGE_HEAD".into(),
            ],
        )
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        OperationType::Merge
    } else {
        OperationType::None
    };
    Ok(OperationInfo {
        op_type,
        source_branch: None,
        target_branch: git_output(repo, &["branch", "--show-current"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        progress: None,
        source_color_index: None,
        target_color_index: None,
        rebase_message: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_commit(oid: &str, parents: &[&str]) -> GraphCommit {
        GraphCommit {
            oid: oid.to_string(),
            short_oid: short_oid(oid),
            summary: String::new(),
            body: None,
            author_name: String::new(),
            author_email: String::new(),
            author_timestamp: 0,
            parent_oids: parents.iter().map(|parent| parent.to_string()).collect(),
            column: 0,
            color_index: 0,
            edges: Vec::new(),
            refs: Vec::new(),
            is_head: false,
            is_merge: parents.len() >= 2,
            is_branch_tip: false,
            is_stash: false,
        }
    }

    #[test]
    fn parses_porcelain_status_into_existing_dto_buckets() {
        let status = parse_porcelain_status("M  staged.txt\0 M unstaged.txt\0?? new.txt\0UU conflict.txt\0R  old.txt\0renamed.txt\0");

        assert_eq!(status.staged.len(), 2);
        assert_eq!(status.unstaged.len(), 2);
        assert_eq!(status.conflicted.len(), 1);
        assert_eq!(status.staged[0].path, "staged.txt");
        assert!(matches!(status.staged[0].status, FileStatusType::Modified));
        assert_eq!(status.unstaged[1].path, "new.txt");
        assert!(matches!(status.unstaged[1].status, FileStatusType::New));
        assert_eq!(status.conflicted[0].path, "conflict.txt");
        assert_eq!(status.staged[1].path, "renamed.txt");
        assert!(matches!(status.staged[1].status, FileStatusType::Renamed));
    }

    #[test]
    fn parses_unified_diff_hunks_into_file_diff_dtos() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
index 1111111..2222222 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,2 +1,2 @@
 fn main() {
-    println!(\"old\");
+    println!(\"new\");
 }
";

        let files = parse_unified_diff(diff);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/main.rs");
        assert!(matches!(files[0].status, DiffStatus::Modified));
        assert_eq!(files[0].hunks.len(), 1);
        assert_eq!(files[0].hunks[0].old_start, 1);
        assert_eq!(files[0].hunks[0].new_start, 1);
        assert_eq!(files[0].hunks[0].lines.len(), 4);
        assert!(matches!(
            files[0].hunks[0].lines[1].origin,
            DiffOrigin::Delete
        ));
        assert_eq!(files[0].hunks[0].lines[1].old_lineno, Some(2));
        assert!(matches!(files[0].hunks[0].lines[2].origin, DiffOrigin::Add));
        assert_eq!(files[0].hunks[0].lines[2].new_lineno, Some(2));
    }

    #[test]
    fn assigns_wsl_graph_edges_for_linear_history() {
        let mut commits = vec![
            graph_commit("bbbbbbb", &["aaaaaaa"]),
            graph_commit("aaaaaaa", &[]),
        ];

        let max_columns = assign_graph_lanes(&mut commits);

        assert_eq!(max_columns, 1);
        assert_eq!(commits[0].column, 0);
        assert_eq!(commits[1].column, 0);
        assert_eq!(commits[0].edges.len(), 1);
        assert!(matches!(commits[0].edges[0].edge_type, EdgeType::Straight));
    }

    #[test]
    fn assigns_wsl_graph_edges_for_merge_history() {
        let mut commits = vec![
            graph_commit("merge01", &["main001", "topic01"]),
            graph_commit("topic01", &["base001"]),
            graph_commit("main001", &["base001"]),
            graph_commit("base001", &[]),
        ];

        let max_columns = assign_graph_lanes(&mut commits);

        assert!(max_columns >= 2);
        assert_eq!(commits[0].edges.len(), 2);
        assert!(commits[0]
            .edges
            .iter()
            .any(|edge| matches!(edge.edge_type, EdgeType::MergeRight | EdgeType::MergeLeft)));
    }
}
