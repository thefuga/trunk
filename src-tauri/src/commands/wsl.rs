use crate::error::TrunkError;
use crate::git::types::RepoDescriptor;
#[cfg(target_os = "windows")]
use crate::git::types::RepoLocator;
use serde::Serialize;

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

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
#[cfg(any(target_os = "windows", test))]
fn clean_wsl_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .replace(['\0', '\r'], "")
        .trim()
        .to_string()
}

#[cfg(any(target_os = "windows", test))]
fn parse_default_distro(status_text: &str) -> Option<String> {
    status_text.lines().find_map(|line| {
        line.strip_prefix("Default Distribution:")
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    })
}

#[cfg(any(target_os = "windows", test))]
fn parse_wsl_distros(list_text: &str, default_name: Option<&str>) -> Vec<WslDistro> {
    list_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|name| WslDistro {
            name: name.to_string(),
            default: default_name == Some(name),
        })
        .collect()
}

#[cfg(any(target_os = "windows", test))]
fn classify_wsl_repo_error(distro: &str, linux_path: &str, stderr: &str) -> TrunkError {
    let trimmed = stderr.trim();
    let lower = trimmed.to_lowercase();

    if lower.contains("there is no distribution with the supplied name")
        || lower.contains("specified distribution does not exist")
        || lower.contains("distribution not found")
        || lower.contains("no installed distributions")
    {
        return TrunkError::new(
            "wsl_missing_distro",
            format!(
                "The WSL distro `{}` is not installed. Choose an installed distro or install it with `wsl --install -d <Distro>`.",
                distro
            ),
        );
    }

    if lower.contains("execvpe(git) failed")
        || lower.contains("git: command not found")
        || lower.contains("git: not found")
        || lower.contains("the command 'git' could not be found")
    {
        return TrunkError::new(
            "wsl_missing_git",
            format!(
                "Git is not installed in the `{}` WSL distro. Install Linux Git inside that distro, then try again.",
                distro
            ),
        );
    }

    if lower.contains("no such file or directory")
        || lower.contains("cannot change to")
        || lower.contains("not a git repository")
        || lower.contains("not a git repo")
    {
        return TrunkError::new(
            "wsl_repo_invalid",
            format!(
                "`{}` is not a valid Git repository path in the `{}` WSL distro. Use an absolute Linux path such as `/home/me/project`.",
                linux_path, distro
            ),
        );
    }

    TrunkError::new(
        "wsl_repo_invalid",
        if trimmed.is_empty() {
            format!("`{}` is not a Git repository in {}.", linux_path, distro)
        } else {
            trimmed.to_string()
        },
    )
}

pub fn unc_path(distro: &str, linux_path: &str) -> String {
    let path = linux_path.trim_start_matches('/').replace('/', "\\");
    if path.is_empty() {
        format!(r"\\wsl.localhost\{}", distro)
    } else {
        format!(r"\\wsl.localhost\{}\{}", distro, path)
    }
}

#[cfg(target_os = "windows")]
fn wsl_command(args: &[&str]) -> std::io::Result<std::process::Output> {
    std::process::Command::new("wsl.exe").args(args).output()
}

pub fn availability_inner() -> WslAvailability {
    #[cfg(not(target_os = "windows"))]
    {
        WslAvailability {
            available: false,
            supported_platform: false,
            message: Some("WSL repository opening is only available on Windows.".to_string()),
        }
    }

    #[cfg(target_os = "windows")]
    match wsl_command(&["--status"]) {
        Ok(output) if output.status.success() => WslAvailability {
            available: true,
            supported_platform: true,
            message: None,
        },
        Ok(output) => WslAvailability {
            available: false,
            supported_platform: true,
            message: Some(clean_wsl_output(&output.stderr).if_empty(
                "WSL is installed but not available. Run `wsl --status` in Windows Terminal.",
            )),
        },
        Err(_) => WslAvailability {
            available: false,
            supported_platform: true,
            message: Some("WSL is not installed or `wsl.exe` is not on PATH.".to_string()),
        },
    }
}

#[cfg(target_os = "windows")]
trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

#[cfg(target_os = "windows")]
impl IfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

#[cfg(target_os = "windows")]
fn ensure_available() -> Result<(), TrunkError> {
    let availability = availability_inner();
    if availability.available {
        Ok(())
    } else {
        Err(TrunkError::new(
            "wsl_unavailable",
            availability
                .message
                .unwrap_or_else(|| "WSL is not available on this machine.".to_string()),
        ))
    }
}

#[cfg(target_os = "windows")]
pub fn list_distros_inner() -> Result<Vec<WslDistro>, TrunkError> {
    ensure_available()?;
    let output = wsl_command(&["--list", "--quiet"]).map_err(|e| {
        TrunkError::new(
            "wsl_unavailable",
            format!("Could not run `wsl --list --quiet`: {}", e),
        )
    })?;
    if !output.status.success() {
        return Err(TrunkError::new(
            "wsl_list_failed",
            clean_wsl_output(&output.stderr).if_empty("Could not list WSL distros."),
        ));
    }

    let default_output = wsl_command(&["--status"]).ok();
    let status_text = default_output
        .as_ref()
        .map(|output| clean_wsl_output(&output.stdout))
        .unwrap_or_default();
    let default_name = parse_default_distro(&status_text);
    let distros = parse_wsl_distros(&clean_wsl_output(&output.stdout), default_name.as_deref());

    Ok(distros)
}

#[cfg(target_os = "windows")]
pub fn validate_repo_inner(
    distro: String,
    linux_path: String,
) -> Result<WslRepoValidation, TrunkError> {
    ensure_available()?;
    let distro = distro.trim().to_string();
    let linux_path = linux_path.trim().to_string();
    if distro.is_empty() {
        return Err(TrunkError::new(
            "wsl_missing_distro",
            "Choose a WSL distro.",
        ));
    }
    if !linux_path.starts_with('/') {
        return Err(TrunkError::new(
            "wsl_invalid_path",
            "Enter an absolute Linux path, for example `/home/me/project`.",
        ));
    }

    let output = wsl_command(&[
        "--distribution",
        &distro,
        "--",
        "git",
        "-C",
        &linux_path,
        "rev-parse",
        "--show-toplevel",
    ])
    .map_err(|e| TrunkError::new("wsl_unavailable", format!("Could not run WSL: {}", e)))?;

    if !output.status.success() {
        let stderr = clean_wsl_output(&output.stderr);
        return Err(classify_wsl_repo_error(&distro, &linux_path, &stderr));
    }

    let repo_root = clean_wsl_output(&output.stdout);
    if repo_root.is_empty() {
        return Err(TrunkError::new(
            "wsl_repo_invalid",
            "Git did not return a repository root for that WSL path.",
        ));
    }

    let display_name = repo_root
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(&repo_root)
        .to_string();
    let display_path = format!("{}:{}", distro, repo_root);
    let locator = RepoLocator::Wsl {
        distro: distro.clone(),
        linux_path: repo_root.clone(),
    };
    let descriptor = RepoDescriptor {
        id: locator.stable_id(),
        display_name,
        display_path,
        locator,
    };

    Ok(WslRepoValidation {
        distro,
        linux_path,
        repo_root,
        descriptor,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn list_distros_inner() -> Result<Vec<WslDistro>, TrunkError> {
    Err(crate::git::backend::wsl_unsupported_platform())
}

#[cfg(not(target_os = "windows"))]
pub fn validate_repo_inner(
    _distro: String,
    _linux_path: String,
) -> Result<WslRepoValidation, TrunkError> {
    Err(crate::git::backend::wsl_unsupported_platform())
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

#[cfg(test)]
mod tests {
    use super::{classify_wsl_repo_error, parse_default_distro, parse_wsl_distros, unc_path};

    #[test]
    fn builds_unc_path_from_linux_path() {
        assert_eq!(
            unc_path("Ubuntu", "/home/me/trunk"),
            r"\\wsl.localhost\Ubuntu\home\me\trunk"
        );
    }

    #[test]
    fn parses_default_distro_from_status_output() {
        assert_eq!(
            parse_default_distro("Default Distribution: Ubuntu\nDefault Version: 2"),
            Some("Ubuntu".to_string())
        );
    }

    #[test]
    fn parses_wsl_distros_and_marks_default() {
        let distros = parse_wsl_distros("Ubuntu\nDebian\n", Some("Debian"));

        assert_eq!(distros.len(), 2);
        assert_eq!(distros[0].name, "Ubuntu");
        assert!(!distros[0].default);
        assert_eq!(distros[1].name, "Debian");
        assert!(distros[1].default);
    }

    #[test]
    fn classifies_missing_wsl_distro() {
        let error = classify_wsl_repo_error(
            "Ubuntu",
            "/home/me/trunk",
            "There is no distribution with the supplied name.",
        );

        assert_eq!(error.code, "wsl_missing_distro");
        assert!(error.message.contains("Ubuntu"));
        assert!(error.message.contains("not installed"));
    }

    #[test]
    fn classifies_missing_linux_git() {
        let error = classify_wsl_repo_error(
            "Debian",
            "/home/me/trunk",
            "execvpe(git) failed: No such file or directory",
        );

        assert_eq!(error.code, "wsl_missing_git");
        assert!(error.message.contains("Git is not installed"));
        assert!(error.message.contains("Debian"));
    }

    #[test]
    fn classifies_invalid_wsl_repo_path() {
        let error = classify_wsl_repo_error(
            "Ubuntu",
            "/home/me/missing",
            "fatal: cannot change to '/home/me/missing': No such file or directory",
        );

        assert_eq!(error.code, "wsl_repo_invalid");
        assert!(error.message.contains("/home/me/missing"));
        assert!(error.message.contains("absolute Linux path"));
    }
}
