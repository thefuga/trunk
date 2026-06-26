use crate::error::TrunkError;
use crate::git::types::RepoDescriptor;
use serde::Serialize;

#[cfg(not(target_os = "windows"))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(target_os = "windows"))]
use unsupported as platform;
#[cfg(target_os = "windows")]
use windows as platform;

pub use platform::{availability_inner, list_distros_inner, validate_repo_inner};

#[derive(Debug, Serialize)]
pub struct WslAvailability {
    pub available: bool,
    pub supported_platform: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WslDistro {
    pub name: String,
    pub default: bool,
}

#[derive(Debug, Serialize)]
pub struct WslRepoValidation {
    pub distro: String,
    pub linux_path: String,
    pub repo_root: String,
    pub descriptor: RepoDescriptor,
}

pub fn unc_path(distro: &str, linux_path: &str) -> String {
    let path = linux_path.trim_start_matches('/').replace('/', "\\");
    if path.is_empty() {
        format!(r"\\wsl.localhost\{}", distro)
    } else {
        format!(r"\\wsl.localhost\{}\{}", distro, path)
    }
}

#[tauri::command]
pub async fn wsl_availability() -> Result<WslAvailability, String> {
    Ok(availability_inner())
}

#[tauri::command]
pub async fn list_wsl_distros() -> Result<Vec<WslDistro>, String> {
    tauri::async_runtime::spawn_blocking(list_distros_inner)
        .await
        .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
        .map_err(|e| e.to_json())
}

#[tauri::command]
pub async fn validate_wsl_repo(
    distro: String,
    linux_path: String,
) -> Result<WslRepoValidation, String> {
    tauri::async_runtime::spawn_blocking(move || validate_repo_inner(distro, linux_path))
        .await
        .map_err(|e| TrunkError::new("spawn_error", e.to_string()).to_json())?
        .map_err(|e| e.to_json())
}
