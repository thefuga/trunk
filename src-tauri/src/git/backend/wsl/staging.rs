use crate::error::TrunkError;
use crate::git::command_runner;
use crate::git::read_model;
use crate::git::types::{DiffHunk, DiffLine, DiffOrigin, DiffStatus, FileDiff, RepoDescriptor};
use std::collections::HashSet;

pub(crate) fn git_write(
    repo: &RepoDescriptor,
    args: &[&str],
    code: &str,
) -> Result<(), TrunkError> {
    let output = command_runner::git_output(repo, args, code)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(TrunkError::new(
        code,
        if stderr.is_empty() {
            "Git write operation failed".to_string()
        } else {
            stderr
        },
    ))
}

fn git_apply(
    repo: &RepoDescriptor,
    args: &[&str],
    patch: &str,
    code: &str,
) -> Result<(), TrunkError> {
    let output = command_runner::git_output_with_stdin(repo, args, patch.as_bytes(), code)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(TrunkError::new(
        code,
        if stderr.is_empty() {
            "Git patch apply failed".to_string()
        } else {
            stderr
        },
    ))
}

fn git_output_text(repo: &RepoDescriptor, args: &[&str], code: &str) -> Result<String, TrunkError> {
    let output = command_runner::git_output(repo, args, code)?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(TrunkError::new(
        code,
        if stderr.is_empty() {
            "Git command failed".to_string()
        } else {
            stderr
        },
    ))
}

pub(crate) fn wsl_head_exists(repo: &RepoDescriptor) -> bool {
    command_runner::git_output(repo, &["rev-parse", "--verify", "HEAD"], "git_write_error")
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn select_hunk_patch(
    raw: &str,
    hunk_index: u32,
    file_path: &str,
    empty_code: &str,
) -> Result<String, TrunkError> {
    let mut header = Vec::new();
    let mut selected = Vec::new();
    let mut current: Option<u32> = None;

    for line in raw.split_inclusive('\n') {
        if line.starts_with("@@ ") {
            let next = current.map_or(0, |idx| idx + 1);
            current = Some(next);
        }

        match current {
            None => header.push(line.to_string()),
            Some(idx) if idx == hunk_index => selected.push(line.to_string()),
            Some(_) => {}
        }
    }

    if raw.trim().is_empty() {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No changes for: {}", file_path),
        ));
    }
    if selected.is_empty() {
        let count = current.map_or(0, |idx| idx + 1);
        return Err(TrunkError::new(
            "stale_hunk_index",
            format!(
                "Hunk index {} out of range (file has {} hunks)",
                hunk_index, count
            ),
        ));
    }
    if header.is_empty() {
        return Err(TrunkError::new(empty_code, "Patch header not found"));
    }
    Ok(format!("{}{}", header.join(""), selected.join("")))
}

fn wsl_unstaged_diff_text(repo: &RepoDescriptor, file_path: &str) -> Result<String, TrunkError> {
    let output = command_runner::git_output(
        repo,
        &["diff", "--no-color", "--find-renames", "--", file_path],
        "git_diff_error",
    )?;
    if output.status.success() && !output.stdout.is_empty() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    let untracked = git_output_text(
        repo,
        &[
            "ls-files",
            "--others",
            "--exclude-standard",
            "--",
            file_path,
        ],
        "git_diff_error",
    )?;
    if untracked.trim().is_empty() {
        return Ok(String::new());
    }

    let output = command_runner::git_output(
        repo,
        &["diff", "--no-color", "--no-index", "/dev/null", file_path],
        "git_diff_error",
    )?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn wsl_staged_diff_text(repo: &RepoDescriptor, file_path: &str) -> Result<String, TrunkError> {
    git_output_text(
        repo,
        &[
            "diff",
            "--cached",
            "--no-color",
            "--find-renames",
            "--",
            file_path,
        ],
        "git_diff_error",
    )
}

pub(crate) fn wsl_stage_files(
    repo: &RepoDescriptor,
    file_paths: &[String],
) -> Result<(), TrunkError> {
    if file_paths.is_empty() {
        return Ok(());
    }
    let mut args = vec!["add", "--"];
    args.extend(file_paths.iter().map(String::as_str));
    git_write(repo, &args, "stage_error")
}

pub(crate) fn wsl_unstage_files(
    repo: &RepoDescriptor,
    file_paths: &[String],
) -> Result<(), TrunkError> {
    if file_paths.is_empty() {
        return Ok(());
    }
    if wsl_head_exists(repo) {
        let mut args = vec!["restore", "--staged", "--"];
        args.extend(file_paths.iter().map(String::as_str));
        git_write(repo, &args, "unstage_error")
    } else {
        let mut args = vec!["rm", "--cached", "-r", "--ignore-unmatch", "--"];
        args.extend(file_paths.iter().map(String::as_str));
        git_write(repo, &args, "unstage_error")
    }
}

pub(crate) fn wsl_stage_hunk(
    repo: &RepoDescriptor,
    file_path: &str,
    hunk_index: u32,
) -> Result<(), TrunkError> {
    let raw = wsl_unstaged_diff_text(repo, file_path)?;
    let patch = select_hunk_patch(&raw, hunk_index, file_path, "hunk_apply_failed")?;
    git_apply(
        repo,
        &["apply", "--cached", "--"],
        &patch,
        "hunk_apply_failed",
    )
}

pub(crate) fn wsl_unstage_hunk(
    repo: &RepoDescriptor,
    file_path: &str,
    hunk_index: u32,
) -> Result<(), TrunkError> {
    let raw = wsl_staged_diff_text(repo, file_path)?;
    let patch = select_hunk_patch(&raw, hunk_index, file_path, "hunk_apply_failed")?;
    git_apply(
        repo,
        &["apply", "--cached", "--reverse", "--"],
        &patch,
        "hunk_apply_failed",
    )
}

pub(crate) fn wsl_discard_hunk(
    repo: &RepoDescriptor,
    file_path: &str,
    hunk_index: u32,
) -> Result<(), TrunkError> {
    let raw = wsl_unstaged_diff_text(repo, file_path)?;
    let patch = select_hunk_patch(&raw, hunk_index, file_path, "hunk_apply_failed")?;
    git_apply(
        repo,
        &["apply", "--reverse", "--"],
        &patch,
        "hunk_apply_failed",
    )
}

fn build_partial_patch_from_diff(
    file_path: &str,
    file_diff: &FileDiff,
    hunk: &DiffHunk,
    selected_indices: &[u32],
    reverse: bool,
) -> String {
    let selected_set: HashSet<u32> = selected_indices.iter().copied().collect();
    let mut patch_lines = Vec::new();
    let mut old_count = 0u32;
    let mut new_count = 0u32;

    for (line_idx, line) in hunk.lines.iter().enumerate() {
        let content = line_content(line);
        if reverse {
            match line.origin {
                DiffOrigin::Add => {
                    if selected_set.contains(&(line_idx as u32)) {
                        patch_lines.push(format!("-{}", content));
                        old_count += 1;
                    } else {
                        patch_lines.push(format!(" {}", content));
                        old_count += 1;
                        new_count += 1;
                    }
                }
                DiffOrigin::Delete => {
                    if selected_set.contains(&(line_idx as u32)) {
                        patch_lines.push(format!("+{}", content));
                        new_count += 1;
                    }
                }
                DiffOrigin::Context => {
                    patch_lines.push(format!(" {}", content));
                    old_count += 1;
                    new_count += 1;
                }
            }
        } else {
            match line.origin {
                DiffOrigin::Add => {
                    if selected_set.contains(&(line_idx as u32)) {
                        patch_lines.push(format!("+{}", content));
                        new_count += 1;
                    }
                }
                DiffOrigin::Delete => {
                    if selected_set.contains(&(line_idx as u32)) {
                        patch_lines.push(format!("-{}", content));
                        old_count += 1;
                    } else {
                        patch_lines.push(format!(" {}", content));
                        old_count += 1;
                        new_count += 1;
                    }
                }
                DiffOrigin::Context => {
                    patch_lines.push(format!(" {}", content));
                    old_count += 1;
                    new_count += 1;
                }
            }
        }
    }

    let (old_start, new_start) = if reverse {
        (hunk.new_start, hunk.old_start)
    } else {
        (hunk.old_start, hunk.new_start)
    };

    let old_header = if (!reverse
        && matches!(file_diff.status, DiffStatus::Added | DiffStatus::Untracked))
        || (reverse && matches!(file_diff.status, DiffStatus::Deleted))
    {
        "--- /dev/null".to_string()
    } else {
        format!("--- a/{}", file_path)
    };
    let new_header = if (!reverse && matches!(file_diff.status, DiffStatus::Deleted))
        || (reverse && matches!(file_diff.status, DiffStatus::Added | DiffStatus::Untracked))
    {
        "+++ /dev/null".to_string()
    } else {
        format!("+++ b/{}", file_path)
    };

    format!(
        "diff --git a/{path} b/{path}\n{old_header}\n{new_header}\n@@ -{old_start},{old_count} +{new_start},{new_count} @@\n{lines}",
        path = file_path,
        old_header = old_header,
        new_header = new_header,
        old_start = old_start,
        old_count = old_count,
        new_start = new_start,
        new_count = new_count,
        lines = patch_lines.join(""),
    )
}

fn line_content(line: &DiffLine) -> String {
    if line.content.ends_with('\n') {
        line.content.clone()
    } else {
        format!("{}\n", line.content)
    }
}

fn single_wsl_diff(
    diffs: Vec<FileDiff>,
    file_path: &str,
    hunk_index: u32,
    missing_message: &str,
) -> Result<(FileDiff, DiffHunk), TrunkError> {
    let file_diff = diffs
        .into_iter()
        .next()
        .ok_or_else(|| TrunkError::new("file_not_found", missing_message.to_string()))?;
    let hunk = file_diff
        .hunks
        .get(hunk_index as usize)
        .cloned()
        .ok_or_else(|| {
            TrunkError::new(
                "stale_hunk_index",
                format!(
                    "Hunk index {} out of range (file has {} hunks)",
                    hunk_index,
                    file_diff.hunks.len()
                ),
            )
        })?;
    if file_diff.path != file_path && !file_diff.path.ends_with(file_path) {
        return Err(TrunkError::new(
            "file_not_found",
            format!("No changes for: {}", file_path),
        ));
    }
    Ok((file_diff, hunk))
}

pub(crate) fn wsl_stage_lines(
    repo: &RepoDescriptor,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,
) -> Result<(), TrunkError> {
    let raw = wsl_unstaged_diff_text(repo, file_path)?;
    let diffs = read_model::parse_unified_diff(&raw);
    let (file_diff, hunk) = single_wsl_diff(
        diffs,
        file_path,
        hunk_index,
        &format!("No unstaged changes for: {}", file_path),
    )?;
    let patch = build_partial_patch_from_diff(file_path, &file_diff, &hunk, &line_indices, false);
    git_apply(
        repo,
        &["apply", "--cached", "--"],
        &patch,
        "line_apply_failed",
    )
}

pub(crate) fn wsl_unstage_lines(
    repo: &RepoDescriptor,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,
) -> Result<(), TrunkError> {
    let raw = wsl_staged_diff_text(repo, file_path)?;
    let diffs = read_model::parse_unified_diff(&raw);
    let (file_diff, hunk) = single_wsl_diff(
        diffs,
        file_path,
        hunk_index,
        &format!("No staged changes for: {}", file_path),
    )?;
    let patch = build_partial_patch_from_diff(file_path, &file_diff, &hunk, &line_indices, true);
    git_apply(
        repo,
        &["apply", "--cached", "--"],
        &patch,
        "line_apply_failed",
    )
}

pub(crate) fn wsl_discard_lines(
    repo: &RepoDescriptor,
    file_path: &str,
    hunk_index: u32,
    line_indices: Vec<u32>,
) -> Result<(), TrunkError> {
    let raw = wsl_unstaged_diff_text(repo, file_path)?;
    let diffs = read_model::parse_unified_diff(&raw);
    let (file_diff, hunk) = single_wsl_diff(
        diffs,
        file_path,
        hunk_index,
        &format!("No unstaged changes for: {}", file_path),
    )?;
    let patch = build_partial_patch_from_diff(file_path, &file_diff, &hunk, &line_indices, true);
    git_apply(repo, &["apply", "--"], &patch, "line_apply_failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn run_git(dir: &std::path::Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git command should spawn");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    fn repo_with_committed_file() -> (tempfile::TempDir, RepoDescriptor) {
        let dir = tempfile::TempDir::new().unwrap();
        run_git(dir.path(), &["init", "-q"]);
        std::fs::write(dir.path().join("file.txt"), "a\n").unwrap();
        run_git(dir.path(), &["add", "file.txt"]);
        run_git(
            dir.path(),
            &[
                "-c",
                "user.name=Trunk Test",
                "-c",
                "user.email=trunk@example.test",
                "commit",
                "-q",
                "-m",
                "initial",
            ],
        );
        let repo = RepoDescriptor::local(dir.path().to_string_lossy().into_owned());
        (dir, repo)
    }

    fn diff_line(origin: DiffOrigin, content: &str) -> DiffLine {
        DiffLine {
            origin,
            content: format!("{content}\n"),
            old_lineno: None,
            new_lineno: None,
            spans: Vec::new(),
        }
    }

    #[test]
    fn select_hunk_patch_keeps_headers_and_only_target_hunk() {
        let raw = "\
diff --git a/file.txt b/file.txt
--- a/file.txt
+++ b/file.txt
@@ -1,1 +1,1 @@
-old
+new
@@ -10,1 +10,1 @@
-older
+newer
";

        let patch = select_hunk_patch(raw, 1, "file.txt", "hunk_apply_failed").unwrap();

        assert!(patch.contains("diff --git a/file.txt b/file.txt"));
        assert!(!patch.contains("-old\n"));
        assert!(patch.contains("-older\n"));
        assert!(patch.contains("+newer\n"));
    }

    #[test]
    fn partial_wsl_line_patch_preserves_unselected_delete_as_context() {
        let file_diff = FileDiff {
            path: "file.txt".to_string(),
            status: DiffStatus::Modified,
            is_binary: false,
            hunks: Vec::new(),
        };
        let hunk = DiffHunk {
            header: "@@ -1,2 +1,2 @@\n".to_string(),
            old_start: 1,
            old_lines: 2,
            new_start: 1,
            new_lines: 2,
            lines: vec![
                diff_line(DiffOrigin::Delete, "keep old"),
                diff_line(DiffOrigin::Add, "stage new"),
            ],
        };

        let patch = build_partial_patch_from_diff("file.txt", &file_diff, &hunk, &[1], false);

        assert!(patch.contains("--- a/file.txt"));
        assert!(patch.contains("+++ b/file.txt"));
        assert!(patch.contains(" keep old\n"));
        assert!(patch.contains("+stage new\n"));
        assert!(!patch.contains("-keep old\n"));
    }

    #[test]
    fn wsl_unstage_lines_applies_reverse_partial_patch_to_index_once() {
        let (dir, repo) = repo_with_committed_file();
        std::fs::write(dir.path().join("file.txt"), "a\nb\nc\n").unwrap();
        run_git(dir.path(), &["add", "file.txt"]);

        wsl_unstage_lines(&repo, "file.txt", 0, vec![1]).unwrap();

        let staged = run_git(dir.path(), &["diff", "--cached", "--", "file.txt"]);
        let unstaged = run_git(dir.path(), &["diff", "--", "file.txt"]);
        assert!(!staged.contains("+b\n"));
        assert!(staged.contains("+c\n"));
        assert!(unstaged.contains("+b\n"));
        assert!(!unstaged.contains("+c\n"));
    }

    #[test]
    fn wsl_discard_lines_applies_reverse_partial_patch_to_workdir_once() {
        let (dir, repo) = repo_with_committed_file();
        std::fs::write(dir.path().join("file.txt"), "a\nb\nc\n").unwrap();

        wsl_discard_lines(&repo, "file.txt", 0, vec![1]).unwrap();

        let content = std::fs::read_to_string(dir.path().join("file.txt")).unwrap();
        assert_eq!(content, "a\nc\n");
    }
}
