use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use uuid::Uuid;

use crate::error::TrunkError;
use crate::git::editor::shell_single_quote;
use crate::git::types::{RepoDescriptor, RepoLocator};
use crate::shell_env;

fn validate_repo_relative(path: &str) -> Result<(), TrunkError> {
    let rel = Path::new(path);
    if rel.is_absolute() {
        return Err(TrunkError::new(
            "bad_path",
            format!("Repository-relative path must not be absolute: {path}"),
        ));
    }
    for component in rel.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(TrunkError::new(
                "bad_path",
                format!("Repository-relative path escapes the repository: {path}"),
            ));
        }
    }
    Ok(())
}

fn wsl_sh(distro: &str, cd: &str, script: &str) -> Command {
    let mut command = Command::new("wsl.exe");
    command
        .args(["-d", distro, "--cd", cd, "sh", "-lc", script])
        .env("PATH", shell_env::system_path());
    command
}

fn wsl_output(
    distro: &str,
    cd: &str,
    script: &str,
    stdin: Option<&[u8]>,
) -> Result<Vec<u8>, TrunkError> {
    let mut command = wsl_sh(distro, cd, script);
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|e| TrunkError::new("wsl_spawn_error", e.to_string()))?;
    if let Some(input) = stdin {
        let mut child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| TrunkError::new("wsl_io_error", "failed to open WSL stdin"))?;
        child_stdin
            .write_all(input)
            .map_err(|e| TrunkError::new("wsl_io_error", e.to_string()))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|e| TrunkError::new("wsl_io_error", e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(TrunkError::new(
            "wsl_io_error",
            if stderr.is_empty() {
                "WSL filesystem command failed".to_string()
            } else {
                stderr
            },
        ));
    }
    Ok(output.stdout)
}

pub fn repo_identity(repo: &RepoDescriptor) -> String {
    repo.locator.stable_id()
}

pub fn read_repo_file(repo: &RepoDescriptor, relative_path: &str) -> Result<String, TrunkError> {
    validate_repo_relative(relative_path)?;
    match &repo.locator {
        RepoLocator::Local { path } => std::fs::read_to_string(Path::new(path).join(relative_path))
            .map_err(|e| TrunkError::new("io_error", e.to_string())),
        RepoLocator::Wsl { distro, linux_path } => {
            let rel = shell_single_quote(relative_path);
            let bytes = wsl_output(distro, linux_path, &format!("cat -- {rel}"), None)?;
            String::from_utf8(bytes).map_err(|e| TrunkError::new("utf8_error", e.to_string()))
        }
    }
}

pub fn write_repo_file(
    repo: &RepoDescriptor,
    relative_path: &str,
    content: &str,
) -> Result<(), TrunkError> {
    validate_repo_relative(relative_path)?;
    match &repo.locator {
        RepoLocator::Local { path } => std::fs::write(Path::new(path).join(relative_path), content)
            .map_err(|e| TrunkError::new("write_error", e.to_string())),
        RepoLocator::Wsl { distro, linux_path } => {
            let rel = shell_single_quote(relative_path);
            wsl_output(
                distro,
                linux_path,
                &format!("mkdir -p -- \"$(dirname -- {rel})\" && cat > {rel}"),
                Some(content.as_bytes()),
            )?;
            Ok(())
        }
    }
}

pub fn delete_repo_file(repo: &RepoDescriptor, relative_path: &str) -> Result<(), TrunkError> {
    validate_repo_relative(relative_path)?;
    match &repo.locator {
        RepoLocator::Local { path } => {
            match std::fs::remove_file(Path::new(path).join(relative_path)) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(TrunkError::new("io_error", e.to_string())),
            }
        }
        RepoLocator::Wsl { distro, linux_path } => {
            let rel = shell_single_quote(relative_path);
            wsl_output(distro, linux_path, &format!("rm -f -- {rel}"), None)?;
            Ok(())
        }
    }
}

pub enum BackendTempDir {
    Local(PathBuf),
    Wsl {
        distro: String,
        repo_path: String,
        linux_path: String,
    },
}

impl BackendTempDir {
    pub fn create(repo: &RepoDescriptor, prefix: &str) -> Result<Self, TrunkError> {
        let name = format!("{prefix}-{}-{}", std::process::id(), Uuid::new_v4());
        match &repo.locator {
            RepoLocator::Local { .. } => {
                let path = std::env::temp_dir().join(name);
                std::fs::create_dir_all(&path)
                    .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
                Ok(Self::Local(path))
            }
            RepoLocator::Wsl { distro, linux_path } => {
                let dir = format!("/tmp/{name}");
                let quoted = shell_single_quote(&dir);
                wsl_output(distro, linux_path, &format!("mkdir -p -- {quoted}"), None)?;
                Ok(Self::Wsl {
                    distro: distro.clone(),
                    repo_path: linux_path.clone(),
                    linux_path: dir,
                })
            }
        }
    }

    pub fn join_display(&self, child: &str) -> String {
        match self {
            Self::Local(path) => path.join(child).display().to_string(),
            Self::Wsl { linux_path, .. } => format!("{linux_path}/{child}"),
        }
    }

    pub fn local_path(&self) -> Option<&Path> {
        match self {
            Self::Local(path) => Some(path),
            Self::Wsl { .. } => None,
        }
    }

    pub fn write_file(
        &self,
        child: &str,
        content: &str,
        executable: bool,
    ) -> Result<(), TrunkError> {
        match self {
            Self::Local(path) => {
                let file = path.join(child);
                std::fs::write(&file, content)
                    .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
                if executable {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755))
                            .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
                    }
                }
                Ok(())
            }
            Self::Wsl {
                distro,
                repo_path,
                linux_path,
            } => {
                let target = shell_single_quote(&format!("{linux_path}/{child}"));
                let chmod = if executable {
                    " && chmod 755 -- $TARGET"
                } else {
                    ""
                };
                wsl_output(
                    distro,
                    repo_path,
                    &format!("TARGET={target}; mkdir -p -- \"$(dirname -- \"$TARGET\")\" && cat > \"$TARGET\"{chmod}"),
                    Some(content.as_bytes()),
                )?;
                Ok(())
            }
        }
    }

    pub fn create_dir_all(&self, child: &str) -> Result<(), TrunkError> {
        match self {
            Self::Local(path) => std::fs::create_dir_all(path.join(child))
                .map_err(|e| TrunkError::new("io_error", e.to_string())),
            Self::Wsl {
                distro,
                repo_path,
                linux_path,
            } => {
                let dir = shell_single_quote(&format!("{linux_path}/{child}"));
                wsl_output(distro, repo_path, &format!("mkdir -p -- {dir}"), None)?;
                Ok(())
            }
        }
    }

    pub fn cleanup(&self) {
        match self {
            Self::Local(path) => {
                let _ = std::fs::remove_dir_all(path);
            }
            Self::Wsl {
                distro,
                repo_path,
                linux_path,
            } => {
                let dir = shell_single_quote(linux_path);
                let _ = wsl_output(distro, repo_path, &format!("rm -rf -- {dir}"), None);
            }
        }
    }
}

pub fn wsl_poll_token(repo: &RepoDescriptor) -> Result<Option<String>, TrunkError> {
    match &repo.locator {
        RepoLocator::Local { .. } => Ok(None),
        RepoLocator::Wsl { distro, linux_path } => {
            let script =
                "git status --porcelain=v1 -uall && git rev-parse HEAD 2>/dev/null || true";
            let bytes = wsl_output(distro, linux_path, script, None)?;
            Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
        }
    }
}
