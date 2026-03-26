use crate::common::context::TestContext;
use trunk_lib::commands::stash;
use trunk_lib::error::TrunkError;
use trunk_lib::git::types::{GraphResult, StashEntry};

impl TestContext {
    pub fn list_stashes(&self) -> Result<Vec<StashEntry>, TrunkError> {
        stash::list_stashes_inner(self.path(), self.state_map())
    }

    pub fn stash_save(&self, message: &str) -> Result<GraphResult, TrunkError> {
        stash::stash_save_inner(self.path(), message, self.state_map())
    }

    pub fn stash_pop(&self, index: usize) -> Result<GraphResult, TrunkError> {
        stash::stash_pop_inner(self.path(), index, self.state_map())
    }

    pub fn stash_apply(&self, index: usize) -> Result<GraphResult, TrunkError> {
        stash::stash_apply_inner(self.path(), index, self.state_map())
    }

    pub fn stash_drop(&self, index: usize) -> Result<GraphResult, TrunkError> {
        stash::stash_drop_inner(self.path(), index, self.state_map())
    }
}
