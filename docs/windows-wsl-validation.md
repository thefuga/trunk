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

Trunk opens WSL repositories through the **Open Repository** button: on Windows
with WSL available it drops down **Local** plus each installed distro. Picking a
distro opens the native folder picker at that distro's default user's home
directory (`\\wsl.localhost\<Distro>\...`); picking any WSL UNC folder — even
via **Local** — routes the repository through the WSL backend. After opening,
Git operations run through
`wsl.exe -d <Distro> --cd <repo> --exec git ...`, so credentials, SSH keys,
remotes, hooks, Git config, and Git version come from the selected distro.
`--exec` bypasses the distro's default login shell — without it, shells like
zsh re-parse the command line and choke on Git's `%(...)` ref formats, and
user-provided text (commit messages, paths) would be shell-expanded.

## User-facing setup errors

Trunk should surface actionable errors for these cases:

- Missing WSL: install WSL or make sure `wsl.exe` is on the Windows PATH.
- Missing distro: choose an installed distro or install one with
  `wsl --install -d <Distro>`.
- Missing Linux Git: install Git inside the selected distro.
- Invalid repository path: pick a WSL folder that exists and contains a Git
  worktree.
- Failed authentication: fix the distro's SSH keys, credential helper, host key,
  token, or remote access. Authentication errors are reported by Linux Git.

## Manual validation checklist

Use at least one Linux-side repo in one WSL distro. Record the Windows version,
Trunk build, distro name/version, `git --version`, and repository path.

- [ ] Install or build the Windows Trunk desktop app.
- [ ] Launch Trunk on Windows.
- [ ] Open a normal Windows-hosted repository with **Open Repository → Local**.
- [ ] Without restarting Trunk, open the WSL repository with
  **Open Repository → <Distro>** and confirm the folder picker starts in the
  distro's default user's home directory.
- [ ] Confirm both local and WSL repos can remain open in the same app session.
- [ ] Confirm the WSL repo commit graph, branch labels, tags, and stash labels
  load.
- [ ] In a history where a topic branch forks from an older main commit, confirm
  the WSL commit graph renders a fork edge from the parent row to the topic lane.
- [ ] Modify a tracked file in the WSL repo from the distro shell and confirm
  Trunk refreshes the working tree.
- [ ] Create multiple untracked files inside a new nested directory and confirm
  Trunk lists each untracked file instead of only the directory.
- [ ] View unstaged and staged diffs for the WSL repo.
- [ ] Stage, unstage, stage selected lines or hunks, and discard a selected
  change.
- [ ] Commit from Trunk and confirm `git log -1` inside WSL shows the new
  commit.
- [ ] Create and checkout a branch.
- [ ] Checkout an older commit and confirm `git status` inside WSL reports a
  detached HEAD; return to the branch before continuing.
- [ ] Create a lightweight tag and an annotated tag, then confirm both appear in
  Trunk and `git tag` inside WSL. For the annotated tag, confirm the tagger
  identity matches the distro's Git configuration.
- [ ] Delete both tags and confirm they disappear from Trunk and `git tag`
  inside WSL.
- [ ] Merge a branch and verify clean and conflicted merge states if practical.
- [ ] Start, edit, continue, and abort an interactive rebase if the repo history
  supports it.
- [ ] During WSL interactive rebase, verify Trunk's scripted editors are used
  for reword, squash, reorder, and drop actions.
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

To typecheck the Windows-only WSL backend without installing a WSL distro, use a
Linux or macOS runner with the Windows MSVC Rust target and `cargo-xwin`
installed:

```bash
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
cargo xwin check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --no-default-features --features wsl
cargo xwin clippy --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc --no-default-features --features wsl --all-targets -- -D warnings
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
