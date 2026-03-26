use crate::common::context::TestContext;
use trunk_lib::commands::commit;
use trunk_lib::error::TrunkError;
use trunk_lib::git::types::HeadCommitMessage;

impl TestContext {
    pub fn create_commit(&self, subject: &str, body: Option<&str>) -> Result<(), TrunkError> {
        commit::create_commit_inner(self.path(), subject, body, self.state_map())
    }

    pub fn amend_commit(&self, subject: &str, body: Option<&str>) -> Result<(), TrunkError> {
        commit::amend_commit_inner(self.path(), subject, body, self.state_map())
    }

    pub fn get_head_commit_message(&self) -> Result<HeadCommitMessage, TrunkError> {
        commit::get_head_commit_message_inner(self.path(), self.state_map())
    }
}
