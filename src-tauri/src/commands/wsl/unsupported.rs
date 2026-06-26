use super::{WslAvailability, WslDistro, WslRepoValidation};
use crate::error::TrunkError;

pub fn availability_inner() -> WslAvailability {
    WslAvailability {
        available: false,
        supported_platform: false,
        message: Some("WSL repository opening is only available on Windows.".to_string()),
    }
}

pub fn list_distros_inner() -> Result<Vec<WslDistro>, TrunkError> {
    Err(crate::git::backend::wsl_unsupported_platform())
}

pub fn validate_repo_inner(
    _distro: String,
    _linux_path: String,
) -> Result<WslRepoValidation, TrunkError> {
    Err(crate::git::backend::wsl_unsupported_platform())
}
