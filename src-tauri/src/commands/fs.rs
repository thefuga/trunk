use crate::error::TrunkError;
use crate::git::repository;
use crate::git::types::{RepoDescriptor, RepoLocator};

/// Best-effort check that a recents-list entry is still a valid git repo.
///
/// Returns `Ok(true)` iff the path exists AND opens cleanly via `git2`.
/// Any failure (missing path, not a repo, permission denied) maps to `Ok(false)` —
/// callers only care about whether to keep the entry.
#[tauri::command]
pub async fn validate_recent_path(path: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path_buf = std::path::PathBuf::from(&path);
        repository::validate_and_open(&path_buf).is_ok()
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())
}

fn is_definitive_wsl_validation_error(code: &str) -> bool {
    matches!(
        code,
        "wsl_invalid_path" | "wsl_missing_distro" | "wsl_repo_invalid"
    )
}

/// Backend-aware recents validation. Definitive absence returns `false`; transient
/// backend failures remain errors so the frontend keeps the recent entry.
#[tauri::command]
pub async fn validate_recent_repo(repo: RepoDescriptor) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || match repo.locator {
        RepoLocator::Local { path } => {
            Ok(repository::validate_and_open(std::path::Path::new(&path)).is_ok())
        }
        RepoLocator::Wsl { distro, linux_path } => {
            match crate::commands::wsl::validate_repo_inner(distro, linux_path) {
                Ok(_) => Ok(true),
                Err(error) if is_definitive_wsl_validation_error(&error.code) => Ok(false),
                Err(error) => Err(error.to_json()),
            }
        }
    })
    .await
    .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
}

#[cfg(test)]
mod tests {
    use super::is_definitive_wsl_validation_error;

    #[test]
    fn only_absence_errors_are_definitive_for_wsl_recents() {
        assert!(is_definitive_wsl_validation_error("wsl_repo_invalid"));
        assert!(is_definitive_wsl_validation_error("wsl_missing_distro"));
        assert!(is_definitive_wsl_validation_error("wsl_invalid_path"));
        assert!(!is_definitive_wsl_validation_error("wsl_unavailable"));
        assert!(!is_definitive_wsl_validation_error("wsl_io_error"));
        assert!(!is_definitive_wsl_validation_error(
            "wsl_unsupported_platform"
        ));
    }
}
