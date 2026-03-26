use crate::common::context::TestContext;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct TestContextBuilder {
    steps: Vec<BuildStep>,
}

enum BuildStep {
    WriteFile { path: String, content: Vec<u8> },
    WriteBinaryFile { path: String, content: Vec<u8> },
    Commit { message: String },
    Branch { name: String },
    Checkout { name: String },
    Merge { branch: String },
    Conflict { branch: String },
    Tag { name: String },
    Stash { message: Option<String> },
    Remote { name: String },
}

impl TestContextBuilder {
    pub fn new() -> Self {
        TestContextBuilder { steps: Vec::new() }
    }

    pub fn with_file(&mut self, path: &str, content: &str) -> &mut Self {
        self.steps.push(BuildStep::WriteFile {
            path: path.to_string(),
            content: content.as_bytes().to_vec(),
        });
        self
    }

    pub fn with_binary_file(&mut self, path: &str, content: &[u8]) -> &mut Self {
        self.steps.push(BuildStep::WriteBinaryFile {
            path: path.to_string(),
            content: content.to_vec(),
        });
        self
    }

    pub fn with_commit(&mut self, message: &str) -> &mut Self {
        self.steps.push(BuildStep::Commit {
            message: message.to_string(),
        });
        self
    }

    pub fn with_branch(&mut self, name: &str) -> &mut Self {
        self.steps.push(BuildStep::Branch {
            name: name.to_string(),
        });
        self
    }

    pub fn checkout(&mut self, name: &str) -> &mut Self {
        self.steps.push(BuildStep::Checkout {
            name: name.to_string(),
        });
        self
    }

    pub fn merge(&mut self, branch: &str) -> &mut Self {
        self.steps.push(BuildStep::Merge {
            branch: branch.to_string(),
        });
        self
    }

    pub fn with_conflict(&mut self, branch: &str) -> &mut Self {
        self.steps.push(BuildStep::Conflict {
            branch: branch.to_string(),
        });
        self
    }

    pub fn with_tag(&mut self, name: &str) -> &mut Self {
        self.steps.push(BuildStep::Tag {
            name: name.to_string(),
        });
        self
    }

    pub fn with_stash(&mut self, message: Option<&str>) -> &mut Self {
        self.steps.push(BuildStep::Stash {
            message: message.map(|s| s.to_string()),
        });
        self
    }

    pub fn with_remote(&mut self, name: &str) -> &mut Self {
        self.steps.push(BuildStep::Remote {
            name: name.to_string(),
        });
        self
    }

    pub fn build(&mut self) -> TestContext {
        let dir = tempfile::tempdir().expect("failed to create tempdir");
        let repo = git2::Repository::init(dir.path()).expect("failed to init repo");

        // Configure user identity
        let mut cfg = repo.config().expect("failed to get config");
        cfg.set_str("user.name", "Test User").unwrap();
        cfg.set_str("user.email", "test@example.com").unwrap();
        drop(cfg);

        // Set HEAD to point at main branch
        repo.set_head("refs/heads/main").unwrap();

        // Track files written since the last commit so Commit knows what to stage
        let mut pending_files: Vec<String> = Vec::new();

        for step in &self.steps {
            match step {
                BuildStep::WriteFile { path, content } | BuildStep::WriteBinaryFile { path, content } => {
                    let full_path = dir.path().join(path);
                    if let Some(parent) = full_path.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }
                    std::fs::write(&full_path, content).unwrap();
                    pending_files.push(path.clone());
                }

                BuildStep::Commit { message } => {
                    let sig = repo.signature().unwrap();
                    let mut index = repo.index().unwrap();

                    for file in &pending_files {
                        index
                            .add_path(std::path::Path::new(file))
                            .unwrap();
                    }
                    index.write().unwrap();
                    pending_files.clear();

                    let tree_oid = index.write_tree().unwrap();
                    let tree = repo.find_tree(tree_oid).unwrap();

                    // Get current HEAD as parent (if it exists)
                    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
                    let parents: Vec<&git2::Commit> =
                        parent.as_ref().map(|p| vec![p]).unwrap_or_default();

                    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
                        .unwrap();
                }

                BuildStep::Branch { name } => {
                    let head = repo.head().unwrap().peel_to_commit().unwrap();
                    repo.branch(name, &head, false).unwrap();
                }

                BuildStep::Checkout { name } => {
                    repo.set_head(&format!("refs/heads/{}", name)).unwrap();
                    repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::default().force(),
                    ))
                    .unwrap();
                }

                BuildStep::Merge { branch } => {
                    let sig = repo.signature().unwrap();

                    // Find the branch tip
                    let branch_ref = repo
                        .find_branch(branch, git2::BranchType::Local)
                        .unwrap();
                    let their_commit = branch_ref.get().peel_to_commit().unwrap();

                    // Get current HEAD
                    let our_commit = repo.head().unwrap().peel_to_commit().unwrap();

                    // Merge the two trees
                    let ancestor = repo
                        .find_commit(
                            repo.merge_base(our_commit.id(), their_commit.id()).unwrap(),
                        )
                        .unwrap();
                    let ancestor_tree = ancestor.tree().unwrap();
                    let our_tree = our_commit.tree().unwrap();
                    let their_tree = their_commit.tree().unwrap();

                    let mut merge_index = repo
                        .merge_trees(&ancestor_tree, &our_tree, &their_tree, None)
                        .unwrap();

                    let tree_oid = merge_index.write_tree_to(&repo).unwrap();
                    let tree = repo.find_tree(tree_oid).unwrap();

                    let msg = format!("Merge branch '{}'", branch);
                    repo.commit(
                        Some("HEAD"),
                        &sig,
                        &sig,
                        &msg,
                        &tree,
                        &[&our_commit, &their_commit],
                    )
                    .unwrap();
                }

                BuildStep::Conflict { branch } => {
                    let branch_ref = repo
                        .find_branch(branch, git2::BranchType::Local)
                        .unwrap();
                    let their_commit = branch_ref.get().peel_to_commit().unwrap();
                    let annotated =
                        repo.find_annotated_commit(their_commit.id()).unwrap();

                    repo.merge(&[&annotated], None, None).unwrap();
                    // Leave the repo in merge/conflict state -- do NOT commit
                }

                BuildStep::Tag { name } => {
                    let head = repo.head().unwrap().peel_to_commit().unwrap();
                    let obj = head.as_object();
                    repo.tag_lightweight(name, obj, false).unwrap();
                }

                BuildStep::Stash { message } => {
                    let sig = repo.signature().unwrap();

                    // Need a tracked file that is modified to create a stash
                    let stash_marker = dir.path().join(".stash_marker");
                    if !stash_marker.exists() {
                        // Create and commit the marker file first
                        std::fs::write(&stash_marker, "initial").unwrap();
                        let mut index = repo.index().unwrap();
                        index
                            .add_path(std::path::Path::new(".stash_marker"))
                            .unwrap();
                        index.write().unwrap();

                        let tree_oid = index.write_tree().unwrap();
                        let tree = repo.find_tree(tree_oid).unwrap();
                        let parent =
                            repo.head().ok().and_then(|h| h.peel_to_commit().ok());
                        let parents: Vec<&git2::Commit> =
                            parent.as_ref().map(|p| vec![p]).unwrap_or_default();
                        repo.commit(
                            Some("HEAD"),
                            &sig,
                            &sig,
                            "Add stash marker",
                            &tree,
                            &parents,
                        )
                        .unwrap();
                    }

                    // Modify the tracked file to create something to stash
                    std::fs::write(
                        &stash_marker,
                        format!("modified-{}", repo.stash_count()),
                    )
                    .unwrap();

                    let msg = message.as_deref();
                    repo.stash_save(&sig, msg.unwrap_or("stash"), None)
                        .unwrap();
                }

                BuildStep::Remote { name } => {
                    // Create a bare repo as the remote
                    let bare_path = dir.path().join(format!("{}.git", name));
                    git2::Repository::init_bare(&bare_path).unwrap();

                    let bare_url = bare_path.display().to_string();
                    repo.remote(name, &bare_url).unwrap();
                }
            }
        }

        drop(repo);

        let path = dir.path().display().to_string();
        let mut state_map = HashMap::new();
        state_map.insert(path.clone(), dir.path().to_path_buf());

        TestContext::from_parts(dir, path, state_map)
    }
}
