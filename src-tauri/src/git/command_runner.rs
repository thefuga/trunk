use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use tokio::process::Command as TokioCommand;

use crate::error::TrunkError;
use crate::git::types::RepoDescriptor;
use crate::shell_env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: Option<PathBuf>,
    pub env: Vec<(String, String)>,
}

impl GitCommandSpec {
    pub fn for_repo(repo: &RepoDescriptor, git_args: &[&str]) -> Result<Self, TrunkError> {
        crate::git::backend::resolve_backend(repo.clone())?.command_spec(repo, git_args)
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn with_interactive_rebase_editor_env(
        self,
        repo: &RepoDescriptor,
        seq_editor_path: &str,
        editor_script_path: &str,
    ) -> Result<Self, TrunkError> {
        let spec = self
            .with_env("GIT_SEQUENCE_EDITOR", seq_editor_path)
            .with_env("GIT_EDITOR", editor_script_path);
        Ok(crate::git::backend::resolve_backend(repo.clone())?
            .with_interactive_rebase_editor_env(spec, repo))
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
    GitCommandSpec::for_repo(repo, args)?
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
    let mut command = GitCommandSpec::for_repo(repo, args)?.command();
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

pub fn git_tokio_piped(repo: &RepoDescriptor, args: &[&str]) -> Result<TokioCommand, TrunkError> {
    let mut command = GitCommandSpec::for_repo(repo, args)?.tokio_command();
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_repo(path: &str) -> RepoDescriptor {
        RepoDescriptor::local(path.to_string())
    }

    #[cfg(target_os = "windows")]
    fn wsl_repo() -> RepoDescriptor {
        use crate::git::types::RepoLocator;

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
        let spec = GitCommandSpec::for_repo(&local_repo("/repo"), &["fetch", "--all"]).unwrap();

        assert_eq!(spec.program, "git");
        assert_eq!(spec.args, vec!["fetch", "--all"]);
        assert_eq!(spec.current_dir, Some(PathBuf::from("/repo")));
        assert_eq!(spec.env, Vec::<(String, String)>::new());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn wsl_command_routes_through_selected_distro_and_linux_path() {
        let spec = GitCommandSpec::for_repo(&wsl_repo(), &["status", "--short"]).unwrap();

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
