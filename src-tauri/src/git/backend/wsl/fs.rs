use crate::error::TrunkError;
use crate::git::backend_fs::BackendTempDir;
use crate::git::editor::shell_single_quote;
use crate::git::types::{RepoDescriptor, RepoLocator};
use crate::shell_env;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct WslTempDir {
    distro: String,
    repo_path: String,
    linux_path: String,
}

impl WslTempDir {
    pub fn join_display(&self, child: &str) -> String {
        format!("{}/{child}", self.linux_path)
    }

    pub fn write_file(
        &self,
        child: &str,
        content: &str,
        executable: bool,
    ) -> Result<(), TrunkError> {
        write_temp_file(
            &self.distro,
            &self.repo_path,
            &self.linux_path,
            child,
            content,
            executable,
        )
    }

    pub fn create_dir_all(&self, child: &str) -> Result<(), TrunkError> {
        create_temp_dir_child(&self.distro, &self.repo_path, &self.linux_path, child)
    }

    pub fn cleanup(&self) {
        cleanup_temp_dir(&self.distro, &self.repo_path, &self.linux_path);
    }
}

fn descriptor_parts(repo: &RepoDescriptor) -> Result<(&str, &str), TrunkError> {
    let RepoLocator::Wsl { distro, linux_path } = &repo.locator else {
        return Err(TrunkError::new(
            "backend_descriptor_mismatch",
            "WSL filesystem helper received a non-WSL descriptor",
        ));
    };
    Ok((distro, linux_path))
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

pub fn read_repo_file(repo: &RepoDescriptor, relative_path: &str) -> Result<String, TrunkError> {
    let (distro, linux_path) = descriptor_parts(repo)?;
    let rel = shell_single_quote(relative_path);
    let bytes = wsl_output(distro, linux_path, &format!("cat -- {rel}"), None)?;
    String::from_utf8(bytes).map_err(|e| TrunkError::new("utf8_error", e.to_string()))
}

pub fn write_repo_file(
    repo: &RepoDescriptor,
    relative_path: &str,
    content: &str,
) -> Result<(), TrunkError> {
    let (distro, linux_path) = descriptor_parts(repo)?;
    let rel = shell_single_quote(relative_path);
    wsl_output(
        distro,
        linux_path,
        &format!("mkdir -p -- \"$(dirname -- {rel})\" && cat > {rel}"),
        Some(content.as_bytes()),
    )?;
    Ok(())
}

pub fn read_absolute_file(repo: &RepoDescriptor, path: &str) -> Result<String, TrunkError> {
    let (distro, linux_path) = descriptor_parts(repo)?;
    let path = shell_single_quote(path);
    let bytes = wsl_output(distro, linux_path, &format!("cat -- {path}"), None)?;
    String::from_utf8(bytes).map_err(|e| TrunkError::new("utf8_error", e.to_string()))
}

pub fn write_absolute_file(
    repo: &RepoDescriptor,
    path: &str,
    content: &str,
) -> Result<(), TrunkError> {
    let (distro, linux_path) = descriptor_parts(repo)?;
    let path = shell_single_quote(path);
    wsl_output(
        distro,
        linux_path,
        &format!("mkdir -p -- \"$(dirname -- {path})\" && cat > {path}"),
        Some(content.as_bytes()),
    )?;
    Ok(())
}

pub fn delete_repo_file(repo: &RepoDescriptor, relative_path: &str) -> Result<(), TrunkError> {
    let (distro, linux_path) = descriptor_parts(repo)?;
    let rel = shell_single_quote(relative_path);
    wsl_output(distro, linux_path, &format!("rm -f -- {rel}"), None)?;
    Ok(())
}

pub fn create_temp_dir(repo: &RepoDescriptor, name: &str) -> Result<BackendTempDir, TrunkError> {
    let (distro, linux_path) = descriptor_parts(repo)?;
    let dir = format!("/tmp/{name}");
    let quoted = shell_single_quote(&dir);
    wsl_output(distro, linux_path, &format!("mkdir -p -- {quoted}"), None)?;
    Ok(BackendTempDir::Wsl(WslTempDir {
        distro: distro.to_string(),
        repo_path: linux_path.to_string(),
        linux_path: dir,
    }))
}

fn write_temp_file(
    distro: &str,
    repo_path: &str,
    linux_path: &str,
    child: &str,
    content: &str,
    executable: bool,
) -> Result<(), TrunkError> {
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

fn create_temp_dir_child(
    distro: &str,
    repo_path: &str,
    linux_path: &str,
    child: &str,
) -> Result<(), TrunkError> {
    let dir = shell_single_quote(&format!("{linux_path}/{child}"));
    wsl_output(distro, repo_path, &format!("mkdir -p -- {dir}"), None)?;
    Ok(())
}

fn cleanup_temp_dir(distro: &str, repo_path: &str, linux_path: &str) {
    let dir = shell_single_quote(linux_path);
    let _ = wsl_output(distro, repo_path, &format!("rm -rf -- {dir}"), None);
}

pub fn poll_token(repo: &RepoDescriptor) -> Result<Option<String>, TrunkError> {
    let (distro, linux_path) = descriptor_parts(repo)?;
    let script = "git status --porcelain=v1 -uall && git rev-parse HEAD 2>/dev/null || true";
    let bytes = wsl_output(distro, linux_path, script, None)?;
    Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
}
