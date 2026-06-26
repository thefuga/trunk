use crate::git::command_runner::GitCommandSpec;
use crate::git::types::{RepoDescriptor, RepoLocator};

pub fn spec_for_repo(repo: &RepoDescriptor, git_args: &[&str]) -> GitCommandSpec {
    let RepoLocator::Wsl { distro, linux_path } = &repo.locator else {
        unreachable!("WSL command spec requested for non-WSL descriptor");
    };
    let mut args = vec![
        "-d".to_string(),
        distro.clone(),
        "--cd".to_string(),
        linux_path.clone(),
        "git".to_string(),
    ];
    args.extend(git_args.iter().map(|arg| arg.to_string()));
    GitCommandSpec {
        program: "wsl.exe".to_string(),
        args,
        current_dir: None,
        env: Vec::new(),
    }
}

pub fn with_interactive_rebase_editor_env(spec: GitCommandSpec) -> GitCommandSpec {
    spec.with_env("WSLENV", "GIT_SEQUENCE_EDITOR:GIT_EDITOR")
}
