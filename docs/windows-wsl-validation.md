# Windows WSL repository validation

This checklist validates Trunk's Windows-installed desktop app against a
repository stored inside WSL and operated with Linux Git.

## Supported setup

- Windows 11 or Windows 10 with WSL 2 installed.
- At least one installed WSL distro, validated with `wsl --list --verbose`.
- Linux Git installed inside the selected distro, validated with
  `wsl -d <Distro> -- git --version`.
- A Git repository stored on the Linux filesystem, for example
  `/home/me/projects/trunk`. Prefer the Linux filesystem over `/mnt/c/...` so
  Trunk exercises the WSL backend instead of a Windows path.

Trunk opens WSL repositories from the welcome screen with a distro selector and
an absolute Linux path. After opening, Git operations run through
`wsl.exe -d <Distro> --cd <repo> git ...`, so credentials, SSH keys, remotes,
hooks, Git config, and Git version come from the selected distro.

## User-facing setup errors

Trunk should surface actionable errors for these cases:

- Missing WSL: install WSL or make sure `wsl.exe` is on the Windows PATH.
- Missing distro: choose an installed distro or install one with
  `wsl --install -d <Distro>`.
- Missing Linux Git: install Git inside the selected distro.
- Invalid repository path: enter an absolute Linux path that exists and contains
  a Git worktree.
- Failed authentication: fix the distro's SSH keys, credential helper, host key,
  token, or remote access. Authentication errors are reported by Linux Git.

## Manual validation checklist

Use at least one Linux-side repo in one WSL distro. Record the Windows version,
Trunk build, distro name/version, `git --version`, and repository path.

- [ ] Install or build the Windows Trunk desktop app.
- [ ] Launch Trunk on Windows.
- [ ] Open a normal Windows-hosted repository with **Open Repository**.
- [ ] Without restarting Trunk, open the WSL repository from **Open from WSL**.
- [ ] Confirm both local and WSL repos can remain open in the same app session.
- [ ] Confirm the WSL repo commit graph, branch labels, tags, and stash labels
  load.
- [ ] Modify a tracked file in the WSL repo from the distro shell and confirm
  Trunk refreshes the working tree.
- [ ] View unstaged and staged diffs for the WSL repo.
- [ ] Stage, unstage, stage selected lines or hunks, and discard a selected
  change.
- [ ] Commit from Trunk and confirm `git log -1` inside WSL shows the new
  commit.
- [ ] Create and checkout a branch.
- [ ] Merge a branch and verify clean and conflicted merge states if practical.
- [ ] Start, edit, continue, and abort an interactive rebase if the repo history
  supports it.
- [ ] Create and pop a stash.
- [ ] Fetch, pull, and push with the distro's configured remote credentials.
- [ ] Trigger an authentication failure with an invalid remote or credential and
  confirm the error explains the failed authentication.
- [ ] Close and reopen Trunk, then reopen the WSL repo from recents.
- [ ] Validate the documented setup errors: unavailable WSL, missing distro,
  missing Linux Git, and invalid repo path.

## Windows packaging checks

When running on a Windows machine with the required toolchains:

```bash
just check
just build
```

If full Windows validation is not feasible from the current runner, run the
closest available subset and record the gap in the issue or PR handoff. Linux or
macOS CI can still validate formatting, TypeScript, Svelte, Rust linting, Rust
tests, and frontend tests, but it cannot prove a packaged Windows app can invoke
`wsl.exe`.

## Current limitations

- WSL support is available only in the Windows desktop app.
- Trunk does not install WSL, distros, Git, credentials, SSH keys, or host keys.
- WSL file watching uses polling, so refreshes can lag behind local filesystem
  watcher updates.
- Remote authentication is delegated to Linux Git inside the selected distro.
- Repositories under `/mnt/c/...` are Windows filesystems mounted in WSL; prefer
  Linux-side paths under `/home/...` for parity validation.
