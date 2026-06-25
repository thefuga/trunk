use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use tokio::process::Command as TokioCommand;

use crate::error::TrunkError;
use crate::git::types::{RepoDescriptor, RepoLocator};
use crate::shell_env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    pub env: Vec<(String, String)>,
}

impl GitCommandSpec {
    pub fn for_repo(repo: &RepoDescriptor, git_args: &[&str]) -> Self {
        match &repo.locator {
            RepoLocator::Local { path } => Self {
                program: "git".to_string(),
                args: git_args.iter().map(|arg| arg.to_string()).collect(),
                current_dir: Some(PathBuf::from(path)),
                env: Vec::new(),
            },
            RepoLocator::Wsl { distro, linux_path } => {
                let mut args = vec![
                    "-d".to_string(),
                    distro.clone(),
                    "--cd".to_string(),
                    linux_path.clone(),
                    "git".to_string(),
                ];
                args.extend(git_args.iter().map(|arg| arg.to_string()));
                Self {
                    program: "wsl.exe".to_string(),
                    args,
                    current_dir: None,
                    env: Vec::new(),
                }
            }
        }
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .env("PATH", shell_env::system_path());
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        command
    }

    pub fn tokio_command(&self) -> TokioCommand {
        let mut command = TokioCommand::new(&self.program);
        command
            .args(&self.args)
            .env("PATH", shell_env::system_path());
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        command
    }
}

pub fn git_output(
    repo: &RepoDescriptor,
    args: &[&str],
    spawn_error_code: &str,
) -> Result<Output, TrunkError> {
    GitCommandSpec::for_repo(repo, args)
        .command()
        .output()
        .map_err(|e| TrunkError::new(spawn_error_code, e.to_string()))
}

pub fn git_output_owned(
    repo: &RepoDescriptor,
    args: &[String],
    spawn_error_code: &str,
) -> Result<Output, TrunkError> {
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    git_output(repo, &borrowed, spawn_error_code)
}

pub fn git_output_with_stdin(
    repo: &RepoDescriptor,
    args: &[&str],
    stdin: &[u8],
    spawn_error_code: &str,
) -> Result<Output, TrunkError> {
    let mut command = GitCommandSpec::for_repo(repo, args).command();
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|e| TrunkError::new(spawn_error_code, e.to_string()))?;
    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| TrunkError::new(spawn_error_code, "failed to open git subprocess stdin"))?;
    child_stdin
        .write_all(stdin)
        .map_err(|e| TrunkError::new(spawn_error_code, e.to_string()))?;
    drop(child_stdin);
    child
        .wait_with_output()
        .map_err(|e| TrunkError::new(spawn_error_code, e.to_string()))
}

pub fn git_tokio_piped(repo: &RepoDescriptor, args: &[&str]) -> TokioCommand {
    let mut command = GitCommandSpec::for_repo(repo, args).tokio_command();
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_repo(path: &str) -> RepoDescriptor {
        RepoDescriptor::local(path.to_string())
    }

    fn wsl_repo() -> RepoDescriptor {
        let locator = RepoLocator::Wsl {
            distro: "Ubuntu".to_string(),
            linux_path: "/home/me/project".to_string(),
        };
        RepoDescriptor {
            id: locator.stable_id(),
            display_name: "project".to_string(),
            display_path: r"\\wsl.localhost\Ubuntu\home\me\project".to_string(),
            locator,
        }
    }

    #[test]
    fn local_command_uses_host_git_and_current_dir() {
        let spec = GitCommandSpec::for_repo(&local_repo("/repo"), &["fetch", "--all"]);

        assert_eq!(spec.program, "git");
        assert_eq!(spec.args, vec!["fetch", "--all"]);
        assert_eq!(spec.current_dir, Some(PathBuf::from("/repo")));
        assert_eq!(spec.env, Vec::<(String, String)>::new());
    }

    #[test]
    fn wsl_command_routes_through_selected_distro_and_linux_path() {
        let spec = GitCommandSpec::for_repo(&wsl_repo(), &["status", "--short"]);

        assert_eq!(spec.program, "wsl.exe");
        assert_eq!(
            spec.args,
            vec![
                "-d",
                "Ubuntu",
                "--cd",
                "/home/me/project",
                "git",
                "status",
                "--short"
            ]
        );
        assert_eq!(spec.current_dir, None);
        assert_eq!(spec.env, Vec::<(String, String)>::new());
    }
}
