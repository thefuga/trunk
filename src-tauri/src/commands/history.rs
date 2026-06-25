use crate::error::TrunkError;
use crate::git::{
    graph, read_model,
    types::{GraphCommit, GraphResult, MatchType, SearchResult},
};
use crate::state::{CommitCache, RepoState};
use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

#[derive(Debug, Serialize, Clone)]
pub struct GraphResponse {
    pub commits: Vec<GraphCommit>,
    pub max_columns: usize,
}

#[tauri::command]
pub async fn get_commit_graph(
    path: String,
    offset: usize,
    cache: State<'_, CommitCache>,
) -> Result<GraphResponse, String> {
    let lock = cache.0.lock().unwrap();
    let graph_result = lock
        .get(&path)
        .ok_or_else(|| TrunkError::new("repo_not_open", "Repository not open").to_json())?;

    let len = graph_result.commits.len();
    let start = offset.min(len);
    let end = (offset + 200).min(len);
    Ok(GraphResponse {
        commits: graph_result.commits[start..end].to_vec(),
        max_columns: graph_result.max_columns,
    })
}

#[tauri::command]
pub async fn refresh_commit_graph(
    path: String,
    state: State<'_, RepoState>,
    cache: State<'_, CommitCache>,
) -> Result<GraphResponse, String> {
    let state_map = state.0.lock().unwrap().clone();
    let descriptor_map = state.1.lock().unwrap().clone();
    let path_clone = path.clone();

    let graph_result = tauri::async_runtime::spawn_blocking(move || {
        match read_model::backend_from_state(&path_clone, &state_map, &descriptor_map)? {
            read_model::ReadBackend::Local(path_buf) => {
                let mut repo = git2::Repository::open(path_buf).map_err(TrunkError::from)?;
                graph::walk_commits(&mut repo, 0, usize::MAX)
            }
            read_model::ReadBackend::Wsl(repo) => read_model::wsl_commit_graph(&repo),
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
    .map_err(|e| e.to_json())?;

    let len = graph_result.commits.len();
    let end = 200.min(len);
    let response = GraphResponse {
        commits: graph_result.commits[..end].to_vec(),
        max_columns: graph_result.max_columns,
    };

    cache.0.lock().unwrap().insert(path, graph_result);

    Ok(response)
}

pub fn search_commits_inner(
    path: &str,
    query: &str,
    cache_map: &HashMap<String, GraphResult>,
) -> Result<Vec<SearchResult>, TrunkError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(vec![]);
    }
    let q = query.to_lowercase();

    let graph_result = cache_map.get(path).ok_or_else(|| {
        TrunkError::new("repo_not_open", format!("Repository not open: {}", path))
    })?;

    let mut results = Vec::new();
    for commit in &graph_result.commits {
        let mut match_types = Vec::new();

        // SHA prefix match
        if commit.oid.to_lowercase().starts_with(&q) {
            match_types.push(MatchType::Sha);
        }

        // Message match (summary + body)
        if commit.summary.to_lowercase().contains(&q) {
            match_types.push(MatchType::Message);
        } else if let Some(ref body) = commit.body {
            if body.to_lowercase().contains(&q) {
                match_types.push(MatchType::Message);
            }
        }

        // Ref match (short_name)
        if commit
            .refs
            .iter()
            .any(|r| r.short_name.to_lowercase().contains(&q))
        {
            match_types.push(MatchType::Ref);
        }

        // Author match
        if commit.author_name.to_lowercase().contains(&q) {
            match_types.push(MatchType::Author);
        }

        if !match_types.is_empty() {
            results.push(SearchResult {
                oid: commit.oid.clone(),
                match_types,
            });
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn search_commits(
    path: String,
    query: String,
    cache: State<'_, CommitCache>,
) -> Result<Vec<SearchResult>, String> {
    let cache_map = cache.0.lock().unwrap().clone();
    search_commits_inner(&path, &query, &cache_map).map_err(|e| e.to_json())
}
