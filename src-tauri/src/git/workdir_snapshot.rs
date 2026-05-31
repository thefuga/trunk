//! Capture the current working tree as a REAL but DANGLING commit so it can be
//! reviewed exactly like a hand-picked commit. The snapshot commit's own tree is
//! the working tree (staged + unstaged + untracked-not-ignored); its parent is
//! HEAD. The existing review pipeline then resolves `Side::Old` against the
//! parent tree (= HEAD) and `Side::New` against the snapshot tree (= working
//! tree) — precisely "before vs after" for uncommitted work, with no new
//! Source/Side variant.

use crate::error::TrunkError;

/// Build the current working-tree TREE (staged + unstaged + untracked-not-ignored)
/// and write it to the ODB, returning its Oid — WITHOUT creating a commit and
/// WITHOUT persisting the on-disk `.git/index`.
///
/// No-clobber rationale: the working tree is captured through a THROWAWAY index
/// built with `git2::Index::new()` + `repo.set_index(..)`. `set_index` only
/// associates the in-memory index with the repo so `add_all` can resolve the
/// workdir; `write_tree_to` only writes tree objects to the ODB. Neither calls
/// `index.write()` — the ONLY call that persists `.git/index` to disk. We never
/// call `index.write()`, so the user's real index is byte-for-byte untouched.
///
/// Deterministic by construction: two calls on an unchanged workdir produce the
/// same tree content → the same Oid. This is what lets `decide_snapshot` reuse a
/// prior snapshot whose commit tree equals the current workdir tree.
pub fn workdir_tree_oid(repo: &git2::Repository) -> Result<git2::Oid, TrunkError> {
    // 1. Associate an EMPTY in-memory index with the repo. Starting from empty +
    //    add_all("*") captures the full current workdir (staged + unstaged +
    //    untracked-not-ignored) in one shot, independent of what is staged.
    let mut idx = git2::Index::new()?;
    repo.set_index(&mut idx)?;

    // 2. Re-fetch the now-associated index and add the whole workdir.
    //    IndexAddOption::DEFAULT respects .gitignore: it includes
    //    untracked-but-not-ignored files and excludes ignored ones (identical to
    //    the shipped call at commands/staging.rs:344). NEVER call idx.write().
    let mut idx = repo.index()?;
    idx.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;

    // 3. Write the tree objects to the ODB (does NOT persist the on-disk index).
    Ok(idx.write_tree_to(repo)?)
}

/// Get-or-create the working-tree snapshot for a session (the reuse-vs-create
/// decision in a pure, unit-testable surface — mirrors the validate_range /
/// compute_range_oids pattern: takes `&Repository`, no Tauri state).
///
/// Returns `(oid, created)`:
/// - When `prior` is `Some` and its COMMIT TREE equals the current workdir tree,
///   the workdir is unchanged → reuse the prior snapshot, `(prior, false)`. The
///   comparison is tree-vs-tree: `workdir_tree_oid(repo)` against
///   `repo.find_commit(prior)?.tree_id()`. `prior` itself is a COMMIT oid and is
///   never compared against the tree oid directly.
/// - Otherwise (changed workdir, or `prior` is `None`) create a fresh snapshot
///   commit → `(new_oid, true)`.
pub fn decide_snapshot(
    repo: &git2::Repository,
    prior: Option<git2::Oid>,
) -> Result<(git2::Oid, bool), TrunkError> {
    let current_tree = workdir_tree_oid(repo)?;
    if let Some(prior_oid) = prior {
        let prior_tree = repo.find_commit(prior_oid)?.tree_id();
        if prior_tree == current_tree {
            return Ok((prior_oid, false));
        }
    }
    let new_oid = snapshot_working_tree(repo)?;
    Ok((new_oid, true))
}

/// Snapshot the current working tree into a dangling commit (parent = HEAD) and
/// return its Oid. Builds the tree via `workdir_tree_oid` (which never persists
/// the real `.git/index`) then commits it parent = HEAD.
pub fn snapshot_working_tree(repo: &git2::Repository) -> Result<git2::Oid, TrunkError> {
    // 1–3. Build the workdir tree (no idx.write(), real index untouched).
    let tree = repo.find_tree(workdir_tree_oid(repo)?)?;

    // 4. Resolve the parent: HEAD's commit, or none when HEAD is unborn (a fresh
    //    repo with zero commits still snapshots fine — a parent-less commit).
    let head_commit = if is_head_unborn(repo) {
        None
    } else {
        Some(repo.head()?.peel_to_commit()?)
    };
    let parents: Vec<&git2::Commit> = head_commit.iter().collect();

    // 5. Descriptive message only — the snapshot is tracked by OID in the session
    //    field, never by parsing this string.
    let sig = git2::Signature::now("Trunk", "review@trunk.local")?;
    let msg = format!("Uncommitted changes — {}", sig.when().seconds());

    // 6. `None` target ref => the commit is created dangling. The session command
    //    (ensure_working_tree_snapshot) then pins it via keep_snapshot_ref so gc can't
    //    prune a snapshot that still has comments anchored to it (260531-l02 C3).
    let oid = repo.commit(None, &sig, &sig, &msg, &tree, &parents)?;
    Ok(oid)
}

/// HEAD is unborn when the repo has no commits yet (freshly init'd). Mirrors the
/// probe at commands/diff.rs:25.
fn is_head_unborn(repo: &git2::Repository) -> bool {
    match repo.head() {
        Err(e) => e.code() == git2::ErrorCode::UnbornBranch,
        Ok(_) => false,
    }
}

/// Ref namespace that pins working-tree review snapshots so `git gc` can't prune them.
/// Deliberately NOT under refs/heads|remotes|tags, so these keepalive refs stay out of
/// the branch/commit graph.
pub const SNAPSHOT_REF_PREFIX: &str = "refs/trunk/review-snapshots/";

/// Pin a snapshot commit with a keepalive ref (260531-l02 C3). Without it the snapshot
/// is a dangling commit that gc prunes, silently orphaning every comment anchored to
/// it. Named by the oid so re-pinning a reused snapshot is idempotent; `force = true`
/// tolerates an already-present ref.
pub fn keep_snapshot_ref(repo: &git2::Repository, oid: git2::Oid) -> Result<(), TrunkError> {
    let name = format!("{SNAPSHOT_REF_PREFIX}{oid}");
    repo.reference(&name, oid, true, "trunk working-tree review snapshot")?;
    Ok(())
}

/// Drop all snapshot keepalive refs — called on End Review. Afterward gc may prune the
/// snapshots, which is correct: the session and its comments are gone.
pub fn clear_snapshot_refs(repo: &git2::Repository) -> Result<(), TrunkError> {
    let glob = format!("{SNAPSHOT_REF_PREFIX}*");
    // Collect names first: the glob iterator borrows `repo`, and deleting inside the
    // loop also needs `repo`, so the borrow must be released before the deletes.
    let names: Vec<String> = repo
        .references_glob(&glob)?
        .filter_map(|r| r.ok())
        .filter_map(|r| r.name().map(str::to_owned))
        .collect();
    for name in names {
        if let Ok(mut reference) = repo.find_reference(&name) {
            let _ = reference.delete();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn sig() -> git2::Signature<'static> {
        git2::Signature::new("Test", "test@example.com", &git2::Time::new(0, 0)).unwrap()
    }

    /// Init a repo and write one committed file on HEAD so the snapshot has a
    /// real parent. Returns the TempDir (keep alive) and the open repo.
    fn repo_with_initial_commit() -> (TempDir, git2::Repository) {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("committed.txt"), b"original\n").unwrap();
        {
            let mut index = repo.index().unwrap();
            index
                .add_path(std::path::Path::new("committed.txt"))
                .unwrap();
            index.write().unwrap();
            let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
            let s = sig();
            repo.commit(Some("HEAD"), &s, &s, "initial", &tree, &[])
                .unwrap();
        }
        (dir, repo)
    }

    fn tree_contains(repo: &git2::Repository, oid: git2::Oid, path: &str) -> bool {
        let commit = repo.find_commit(oid).unwrap();
        let tree = commit.tree().unwrap();
        tree.get_path(std::path::Path::new(path)).is_ok()
    }

    // Test A: an untracked-but-not-ignored workdir file IS present in the snapshot.
    #[test]
    fn untracked_file_is_included_in_snapshot() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();

        let oid = snapshot_working_tree(&repo).unwrap();

        assert!(
            tree_contains(&repo, oid, "new.txt"),
            "untracked-not-ignored file must appear in the snapshot tree"
        );
    }

    // C3: keep_snapshot_ref pins the snapshot under refs/trunk/review-snapshots/ so gc
    // can't prune it; it is idempotent; clear_snapshot_refs removes the namespace.
    #[test]
    fn keepalive_ref_pins_snapshot_then_clears() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();
        let oid = snapshot_working_tree(&repo).unwrap();

        keep_snapshot_ref(&repo, oid).unwrap();
        let name = format!("{SNAPSHOT_REF_PREFIX}{oid}");
        assert_eq!(
            repo.find_reference(&name).unwrap().target().unwrap(),
            oid,
            "keepalive ref must point at the snapshot commit"
        );

        // Idempotent: re-pinning a reused snapshot must not error.
        keep_snapshot_ref(&repo, oid).unwrap();

        clear_snapshot_refs(&repo).unwrap();
        assert!(
            repo.find_reference(&name).is_err(),
            "clear_snapshot_refs must remove the keepalive ref"
        );
    }

    // Test B: a file matching a .gitignore pattern is NOT present in the snapshot.
    #[test]
    fn ignored_file_is_excluded_from_snapshot() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join(".gitignore"), b"secret.txt\n").unwrap();
        fs::write(dir.path().join("secret.txt"), b"do not capture\n").unwrap();

        let oid = snapshot_working_tree(&repo).unwrap();

        assert!(
            !tree_contains(&repo, oid, "secret.txt"),
            "a .gitignore-matched file must NOT appear in the snapshot tree"
        );
        assert!(
            tree_contains(&repo, oid, ".gitignore"),
            "the .gitignore itself (not ignored) should still be captured"
        );
    }

    // Test C: the user's real .git/index is byte-for-byte unchanged by the call.
    #[test]
    fn real_index_is_untouched() {
        let (dir, repo) = repo_with_initial_commit();
        // The initial commit wrote a real .git/index. Capture its bytes.
        let index_path = dir.path().join(".git").join("index");
        let before = fs::read(&index_path).expect("repo with a commit has a .git/index");

        // Stage nothing extra; just snapshot a dirty workdir.
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();
        snapshot_working_tree(&repo).unwrap();

        let after = fs::read(&index_path).expect(".git/index must still exist");
        assert_eq!(
            before, after,
            "snapshot must NOT persist anything to the real .git/index"
        );
    }

    // workdir_tree_oid: two calls on an UNCHANGED workdir return the SAME oid,
    // and it equals the tree of a snapshot_working_tree commit (deterministic).
    #[test]
    fn workdir_tree_oid_is_stable_and_matches_snapshot_tree() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();

        let first = workdir_tree_oid(&repo).unwrap();
        let second = workdir_tree_oid(&repo).unwrap();
        assert_eq!(
            first, second,
            "two calls on an unchanged workdir must return the same tree oid"
        );

        let snap = snapshot_working_tree(&repo).unwrap();
        let snap_tree = repo.find_commit(snap).unwrap().tree_id();
        assert_eq!(
            first, snap_tree,
            "workdir_tree_oid must equal the snapshot commit's tree id"
        );
    }

    // decide_snapshot REUSE: prior is the existing snapshot and the workdir is
    // unchanged → returns (prior, false). Asserts BOTH the not-created flag and
    // that the returned oid IS the prior commit oid (catches a tree-vs-commit mixup).
    #[test]
    fn decide_snapshot_reuses_unchanged_workdir() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();

        let prior = snapshot_working_tree(&repo).unwrap();
        let (oid, created) = decide_snapshot(&repo, Some(prior)).unwrap();

        assert!(
            !created,
            "an unchanged workdir must NOT create a new snapshot"
        );
        assert_eq!(
            oid, prior,
            "reuse must return the prior commit oid unchanged"
        );
    }

    // decide_snapshot CREATE: the workdir changed after the prior snapshot →
    // returns (new_oid, true) with new_oid != prior.
    #[test]
    fn decide_snapshot_creates_on_changed_workdir() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();
        let prior = snapshot_working_tree(&repo).unwrap();

        fs::write(dir.path().join("new.txt"), b"changed contents\n").unwrap();
        let (oid, created) = decide_snapshot(&repo, Some(prior)).unwrap();

        assert!(created, "a changed workdir must create a new snapshot");
        assert_ne!(
            oid, prior,
            "the new snapshot oid must differ from the prior"
        );
    }

    // decide_snapshot FIRST snapshot: prior is None → always creates.
    #[test]
    fn decide_snapshot_creates_first_snapshot() {
        let (dir, repo) = repo_with_initial_commit();
        fs::write(dir.path().join("new.txt"), b"uncommitted\n").unwrap();

        let (oid, created) = decide_snapshot(&repo, None).unwrap();

        assert!(created, "first snapshot (prior=None) must create");
        assert!(
            tree_contains(&repo, oid, "new.txt"),
            "the first snapshot must reflect the workdir"
        );
    }

    // Test D: a freshly init'd repo with zero commits (unborn HEAD) snapshots
    // without error and yields a parent-less commit reflecting the workdir.
    #[test]
    fn unborn_head_snapshots_without_parent() {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        fs::write(dir.path().join("first.txt"), b"hello\n").unwrap();

        let oid = snapshot_working_tree(&repo).unwrap();

        let commit = repo.find_commit(oid).unwrap();
        assert_eq!(
            commit.parent_count(),
            0,
            "unborn-HEAD snapshot must have no parent"
        );
        assert!(
            tree_contains(&repo, oid, "first.txt"),
            "the snapshot tree must reflect the workdir even with no commits yet"
        );
    }
}
