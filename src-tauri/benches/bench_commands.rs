use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

struct BenchRepo {
    _dir: tempfile::TempDir,
    path: std::path::PathBuf,
}

/// Create a repo with an initial commit on main, then `branch_count` additional branches
/// each with 2 extra commits. Produces a repo with many refs for `list_refs_inner` to enumerate.
fn make_repo_with_branches(branch_count: usize) -> BenchRepo {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("Bench", "bench@test.com").unwrap();

    // Initial commit on main
    let blob_oid = repo.blob(b"initial").unwrap();
    let mut tb = repo.treebuilder(None).unwrap();
    tb.insert("README.md", blob_oid, 0o100644).unwrap();
    let tree_oid = tb.write().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let initial_oid = repo
        .commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();
    let initial_commit = repo.find_commit(initial_oid).unwrap();

    // Create branches, each with 2 additional commits
    for b in 0..branch_count {
        let branch = repo
            .branch(&format!("branch-{}", b), &initial_commit, false)
            .unwrap();
        let branch_ref = branch.into_reference();
        let ref_name = branch_ref.name().unwrap().to_owned();

        let mut parent_oid = initial_oid;
        for c in 0..2 {
            let blob = repo
                .blob(format!("branch-{}-commit-{}", b, c).as_bytes())
                .unwrap();
            let mut tb = repo.treebuilder(None).unwrap();
            tb.insert(format!("file-{}-{}.txt", b, c), blob, 0o100644)
                .unwrap();
            let tree_oid = tb.write().unwrap();
            let tree = repo.find_tree(tree_oid).unwrap();
            let parent = repo.find_commit(parent_oid).unwrap();
            let oid = repo
                .commit(
                    Some(&ref_name),
                    &sig,
                    &sig,
                    &format!("Branch {} commit {}", b, c),
                    &tree,
                    &[&parent],
                )
                .unwrap();
            parent_oid = oid;
        }
    }

    BenchRepo {
        path: dir.path().to_path_buf(),
        _dir: dir,
    }
}

/// Create a repo with an initial commit containing README.md, then modify
/// README.md on the filesystem to produce unstaged changes for diff and status benchmarks.
fn make_repo_with_unstaged_changes() -> BenchRepo {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("Bench", "bench@test.com").unwrap();

    // Write README.md to filesystem and commit it
    std::fs::write(dir.path().join("README.md"), "initial content").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    repo.commit(
        Some("refs/heads/main"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    )
    .unwrap();

    // Modify README.md to produce unstaged changes
    std::fs::write(dir.path().join("README.md"), "modified content").unwrap();

    BenchRepo {
        path: dir.path().to_path_buf(),
        _dir: dir,
    }
}

/// Create a fresh repo with an unstaged hunk for `stage_hunk_inner` (mutating operation).
/// Returns (dir, path_string, state_map) -- dir must live until the iteration ends.
fn make_repo_for_stage_hunk() -> (tempfile::TempDir, String, HashMap<String, PathBuf>) {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("Bench", "bench@test.com").unwrap();

    // Write README.md and commit
    std::fs::write(dir.path().join("README.md"), "initial content\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("README.md")).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    repo.commit(
        Some("refs/heads/main"),
        &sig,
        &sig,
        "Initial commit",
        &tree,
        &[],
    )
    .unwrap();

    // Modify README.md to produce an unstaged hunk
    std::fs::write(dir.path().join("README.md"), "modified content\n").unwrap();

    let path = dir.path().display().to_string();
    let mut state_map: HashMap<String, PathBuf> = HashMap::new();
    state_map.insert(path.clone(), dir.path().to_path_buf());

    (dir, path, state_map)
}

// OnceLock fixtures for read-only benchmarks
static REPO_BRANCHES: OnceLock<BenchRepo> = OnceLock::new();
static REPO_UNSTAGED: OnceLock<BenchRepo> = OnceLock::new();

fn bench_list_refs(c: &mut Criterion) {
    let bench_repo = REPO_BRANCHES.get_or_init(|| make_repo_with_branches(50));
    let path = bench_repo.path.display().to_string();
    let mut state_map: HashMap<String, PathBuf> = HashMap::new();
    state_map.insert(path.clone(), bench_repo.path.clone());

    c.bench_function("list_refs_inner", |b| {
        b.iter(|| {
            trunk_lib::commands::branches::list_refs_inner(&path, &state_map).unwrap();
        });
    });
}

fn bench_diff_unstaged(c: &mut Criterion) {
    let bench_repo = REPO_UNSTAGED.get_or_init(make_repo_with_unstaged_changes);
    let path = bench_repo.path.display().to_string();
    let mut state_map: HashMap<String, PathBuf> = HashMap::new();
    state_map.insert(path.clone(), bench_repo.path.clone());

    c.bench_function("diff_unstaged_inner", |b| {
        b.iter(|| {
            trunk_lib::commands::diff::diff_unstaged_inner(&path, "README.md", &state_map, &trunk_lib::git::types::DiffRequestOptions::default()).unwrap();
        });
    });
}

fn bench_get_status(c: &mut Criterion) {
    // Reuse REPO_UNSTAGED -- get_status reads but doesn't mutate
    let bench_repo = REPO_UNSTAGED.get_or_init(make_repo_with_unstaged_changes);
    let path = bench_repo.path.display().to_string();
    let mut state_map: HashMap<String, PathBuf> = HashMap::new();
    state_map.insert(path.clone(), bench_repo.path.clone());

    c.bench_function("get_status_inner", |b| {
        b.iter(|| {
            trunk_lib::commands::staging::get_status_inner(&path, &state_map).unwrap();
        });
    });
}

fn bench_stage_hunk(c: &mut Criterion) {
    c.bench_function("stage_hunk_inner", |b| {
        b.iter_batched(
            || make_repo_for_stage_hunk(),
            |(_dir, path, state_map)| {
                trunk_lib::commands::staging::stage_hunk_inner(&path, "README.md", 0, &state_map)
                    .unwrap();
                // _dir dropped here, cleaning up temp directory
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_list_refs,
    bench_diff_unstaged,
    bench_get_status,
    bench_stage_hunk
);
criterion_main!(benches);
