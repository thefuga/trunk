use std::path::{Component, Path, PathBuf};

use uuid::Uuid;

use crate::error::TrunkError;
use crate::git::command_runner;
use crate::git::types::RepoDescriptor;

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

pub fn repo_identity(repo: &RepoDescriptor) -> String {
    repo.locator.stable_id()
}

pub fn read_repo_file(repo: &RepoDescriptor, relative_path: &str) -> Result<String, TrunkError> {
    validate_repo_relative(relative_path)?;
    crate::git::backend::resolve_backend(repo.clone())?.read_repo_file(repo, relative_path)
}

pub fn write_repo_file(
    repo: &RepoDescriptor,
    relative_path: &str,
    content: &str,
) -> Result<(), TrunkError> {
    validate_repo_relative(relative_path)?;
    crate::git::backend::resolve_backend(repo.clone())?.write_repo_file(
        repo,
        relative_path,
        content,
    )
}

fn resolve_git_path(repo: &RepoDescriptor, git_relative_path: &str) -> Result<String, TrunkError> {
    validate_repo_relative(git_relative_path)?;
    let output = command_runner::git_output(
        repo,
        &[
            "rev-parse",
            "--path-format=absolute",
            "--git-path",
            git_relative_path,
        ],
        "git_path_error",
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TrunkError::new("git_path_error", stderr.trim().to_string()));
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err(TrunkError::new(
            "git_path_error",
            format!("Git path not found: {git_relative_path}"),
        ));
    }
    Ok(path)
}

pub fn read_git_file(repo: &RepoDescriptor, git_relative_path: &str) -> Result<String, TrunkError> {
    let backend = crate::git::backend::resolve_backend(repo.clone())?;
    let git_path = resolve_git_path(repo, git_relative_path)?;
    backend.read_absolute_file(repo, &git_path)
}

pub fn write_git_file(
    repo: &RepoDescriptor,
    git_relative_path: &str,
    content: &str,
) -> Result<(), TrunkError> {
    let backend = crate::git::backend::resolve_backend(repo.clone())?;
    let git_path = resolve_git_path(repo, git_relative_path)?;
    backend.write_absolute_file(repo, &git_path, content)
}

pub fn delete_repo_file(repo: &RepoDescriptor, relative_path: &str) -> Result<(), TrunkError> {
    validate_repo_relative(relative_path)?;
    crate::git::backend::resolve_backend(repo.clone())?.delete_repo_file(repo, relative_path)
}

pub enum BackendTempDir {
    Local(PathBuf),
    #[cfg(target_os = "windows")]
    Wsl(crate::git::backend::wsl::fs::WslTempDir),
}

impl BackendTempDir {
    pub fn create(repo: &RepoDescriptor, prefix: &str) -> Result<Self, TrunkError> {
        let name = format!("{prefix}-{}-{}", std::process::id(), Uuid::new_v4());
        crate::git::backend::resolve_backend(repo.clone())?.create_temp_dir(repo, &name)
    }

    pub fn join_display(&self, child: &str) -> String {
        match self {
            Self::Local(path) => path.join(child).display().to_string(),
            #[cfg(target_os = "windows")]
            Self::Wsl(temp_dir) => temp_dir.join_display(child),
        }
    }

    pub fn local_path(&self) -> Option<&Path> {
        match self {
            Self::Local(path) => Some(path),
            #[cfg(target_os = "windows")]
            Self::Wsl(_) => None,
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
            #[cfg(target_os = "windows")]
            Self::Wsl(temp_dir) => temp_dir.write_file(child, content, executable),
        }
    }

    pub fn create_dir_all(&self, child: &str) -> Result<(), TrunkError> {
        match self {
            Self::Local(path) => std::fs::create_dir_all(path.join(child))
                .map_err(|e| TrunkError::new("io_error", e.to_string())),
            #[cfg(target_os = "windows")]
            Self::Wsl(temp_dir) => temp_dir.create_dir_all(child),
        }
    }

    pub fn cleanup(&self) {
        match self {
            Self::Local(path) => {
                let _ = std::fs::remove_dir_all(path);
            }
            #[cfg(target_os = "windows")]
            Self::Wsl(temp_dir) => temp_dir.cleanup(),
        }
    }
}

pub fn poll_token(repo: &RepoDescriptor) -> Result<Option<String>, TrunkError> {
    crate::git::backend::resolve_backend(repo.clone())?.poll_token(repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn run_git(dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("git command should spawn");
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn git_file_helpers_resolve_gitfile_worktree_paths() {
        let root = tempfile::tempdir().unwrap();
        let main_path = root.path().join("main");
        let worktree_path = root.path().join("linked");
        fs::create_dir(&main_path).unwrap();

        run_git(&main_path, &["init", "-b", "main"]);
        run_git(&main_path, &["config", "user.name", "Test"]);
        run_git(&main_path, &["config", "user.email", "test@example.com"]);
        fs::write(main_path.join("README.md"), "initial\n").unwrap();
        run_git(&main_path, &["add", "README.md"]);
        run_git(&main_path, &["commit", "-m", "initial"]);
        run_git(
            &main_path,
            &[
                "worktree",
                "add",
                "-b",
                "linked-branch",
                worktree_path.to_str().unwrap(),
            ],
        );

        let descriptor = RepoDescriptor::local(worktree_path.to_string_lossy().into_owned());
        assert!(
            worktree_path.join(".git").is_file(),
            "linked worktree should use a .git file"
        );

        write_git_file(&descriptor, "MERGE_MSG", "Merge branch 'feature'\n").unwrap();

        assert_eq!(
            read_git_file(&descriptor, "MERGE_MSG").unwrap(),
            "Merge branch 'feature'\n"
        );
        assert!(
            !worktree_path.join(".git").join("MERGE_MSG").exists(),
            "helper must not assume .git is a directory"
        );
    }
}
