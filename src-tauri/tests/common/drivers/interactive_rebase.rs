use crate::common::context::TestContext;
use trunk_lib::commands::interactive_rebase;
use trunk_lib::error::TrunkError;
use trunk_lib::git::types::RebaseTodoItem;

impl TestContext {
    pub fn get_rebase_todo(
        &self,
        base_oid: &str,
        inclusive: bool,
    ) -> Result<Vec<RebaseTodoItem>, TrunkError> {
        interactive_rebase::get_rebase_todo_inner(
            self.path(),
            base_oid,
            inclusive,
            self.state_map(),
        )
    }

    pub fn get_fork_point(&self, branch: &str) -> Result<String, TrunkError> {
        interactive_rebase::get_fork_point_inner(
            self.path(),
            branch,
            self.state_map(),
            self.descriptor_map(),
        )
    }
}
