use crate::common::context::TestContext;
use trunk_lib::commands::diff;
use trunk_lib::error::TrunkError;
use trunk_lib::git::types::{CommitDetail, FileDiff};

impl TestContext {
    pub fn diff_unstaged(&self, file_path: &str) -> Result<Vec<FileDiff>, TrunkError> {
        diff::diff_unstaged_inner(self.path(), file_path, self.state_map())
    }

    pub fn diff_staged(&self, file_path: &str) -> Result<Vec<FileDiff>, TrunkError> {
        diff::diff_staged_inner(self.path(), file_path, self.state_map())
    }

    pub fn diff_commit(&self, oid: &str) -> Result<Vec<FileDiff>, TrunkError> {
        diff::diff_commit_inner(self.path(), oid, self.state_map())
    }

    pub fn get_commit_detail(&self, oid: &str) -> Result<CommitDetail, TrunkError> {
        diff::get_commit_detail_inner(self.path(), oid, self.state_map())
    }
}
