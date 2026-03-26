use crate::common::context::TestContext;
use trunk_lib::commands::staging;
use trunk_lib::error::TrunkError;
use trunk_lib::git::types::WorkingTreeStatus;

impl TestContext {
    pub fn get_status(&self) -> Result<WorkingTreeStatus, TrunkError> {
        staging::get_status_inner(self.path(), self.state_map())
    }

    pub fn stage_file(&self, file_path: &str) -> Result<(), TrunkError> {
        staging::stage_file_inner(self.path(), file_path, self.state_map())
    }

    pub fn unstage_file(&self, file_path: &str) -> Result<(), TrunkError> {
        staging::unstage_file_inner(self.path(), file_path, self.state_map())
    }

    pub fn discard_file(&self, file_path: &str) -> Result<(), TrunkError> {
        staging::discard_file_inner(self.path(), file_path, self.state_map())
    }

    pub fn discard_all(&self) -> Result<(), TrunkError> {
        staging::discard_all_inner(self.path(), self.state_map())
    }

    pub fn stage_all(&self) -> Result<(), TrunkError> {
        staging::stage_all_inner(self.path(), self.state_map())
    }

    pub fn stage_hunk(&self, file_path: &str, hunk_index: u32) -> Result<(), TrunkError> {
        staging::stage_hunk_inner(self.path(), file_path, hunk_index, self.state_map())
    }

    pub fn unstage_hunk(&self, file_path: &str, hunk_index: u32) -> Result<(), TrunkError> {
        staging::unstage_hunk_inner(self.path(), file_path, hunk_index, self.state_map())
    }

    pub fn discard_hunk(&self, file_path: &str, hunk_index: u32) -> Result<(), TrunkError> {
        staging::discard_hunk_inner(self.path(), file_path, hunk_index, self.state_map())
    }

    pub fn unstage_all(&self) -> Result<(), TrunkError> {
        staging::unstage_all_inner(self.path(), self.state_map())
    }

    pub fn stage_lines(
        &self,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
    ) -> Result<(), TrunkError> {
        staging::stage_lines_inner(
            self.path(),
            file_path,
            hunk_index,
            line_indices,
            self.state_map(),
        )
    }

    pub fn unstage_lines(
        &self,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
    ) -> Result<(), TrunkError> {
        staging::unstage_lines_inner(
            self.path(),
            file_path,
            hunk_index,
            line_indices,
            self.state_map(),
        )
    }

    pub fn discard_lines(
        &self,
        file_path: &str,
        hunk_index: u32,
        line_indices: Vec<u32>,
    ) -> Result<(), TrunkError> {
        staging::discard_lines_inner(
            self.path(),
            file_path,
            hunk_index,
            line_indices,
            self.state_map(),
        )
    }
}
