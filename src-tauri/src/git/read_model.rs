#![cfg_attr(
    not(any(target_os = "windows", test)),
    allow(dead_code, unused_imports)
)]

use std::collections::HashMap;

use crate::git::types::{
    DiffHunk, DiffLine, DiffOrigin, DiffStatus, EdgeType, FileDiff, FileStatus, FileStatusType,
    GraphCommit, GraphEdge, RefType, WorkingTreeStatus,
};

pub(crate) fn short_oid(oid: &str) -> String {
    oid.chars().take(7).collect()
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub(crate) fn parse_ref_type(full_name: &str) -> Option<(RefType, String)> {
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

fn find_free_column(active_lanes: &mut Vec<Option<String>>) -> usize {
    if let Some(index) = active_lanes.iter().position(Option::is_none) {
        index
    } else {
        active_lanes.push(None);
        active_lanes.len() - 1
    }
}

pub(crate) fn assign_graph_lanes(commits: &mut [GraphCommit]) -> usize {
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

pub(crate) fn parse_porcelain_status(output: &str) -> WorkingTreeStatus {
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

pub(crate) fn parse_unified_diff(text: &str) -> Vec<FileDiff> {
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
    fn assigns_graph_edges_for_linear_history() {
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
    fn assigns_graph_edges_for_merge_history() {
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
