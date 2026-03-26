use crate::common::context::TestContext;
use trunk_lib::commands::history;
use trunk_lib::error::TrunkError;
use trunk_lib::git::types::SearchResult;

impl TestContext {
    /// Search commits by query string (matches SHA, message, ref, author).
    /// Requires cache_map to be populated first via `populate_cache()`.
    pub fn search_commits(&self, query: &str) -> Result<Vec<SearchResult>, TrunkError> {
        history::search_commits_inner(self.path(), query, &self.cache_map)
    }

    /// Populate the graph cache by running walk_commits on the test repo.
    /// Must be called before search_commits to have data to search.
    pub fn populate_cache(&mut self) {
        let mut repo = self.repo();
        let result =
            trunk_lib::git::graph::walk_commits(&mut repo, 0, usize::MAX).expect("walk_commits failed");
        self.cache_map
            .insert(self.path.clone(), result);
    }
}
