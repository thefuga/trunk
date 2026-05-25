//! Review-session lifecycle commands (Phase 65, Plan 03).
//!
//! Four thin `#[tauri::command]`s over testable `_inner(data_dir: &Path, ...)`
//! functions, mirroring `stash.rs`. The `_inner` wedge takes plain args (no Tauri
//! state) so disk behavior is provable with a `tempfile::TempDir`.
//!
//! Canonical-path keying (D-11): the repo's `PathBuf` is canonicalized so a repo
//! opened via a symlink or alias resumes the SAME session.
//!
//! Disk-first mutation ordering (D-10): `_inner` writes the file → the thin
//! command then updates `ReviewSessionsState` → then emits `session-changed`, so
//! a failed write can never leave memory and disk diverged.

use crate::error::TrunkError;
use crate::git::review_store::{self, LoadOutcome};
use crate::git::types::ReviewSession;
use crate::state::{RepoState, ReviewSessionsState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager, State};

/// The three review-session states the frontend renders (D-12). Serializes
/// kebab-case to match the stub strings `active` / `resume-available` / `none`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SessionState {
    /// File on disk AND in-memory entry present (only the thin command derives this).
    Active,
    /// File on disk but no in-memory entry — the user can resume.
    ResumeAvailable,
    /// No file and no in-memory entry.
    None,
}

/// Status payload for `get_review_session_status`. `_inner` fills the DISK half
/// (`file_exists` + `state` = ResumeAvailable/None); the thin command promotes to
/// `Active` after locking `ReviewSessionsState`. `canonical_path` is the
/// canonicalized path as a String so the frontend can match `session-changed`
/// payloads without re-canonicalizing (it cannot call `std::fs::canonicalize`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionStatus {
    pub state: SessionState,
    pub file_exists: bool,
    pub canonical_path: String,
}

/// Look the repo up in `RepoState`'s map and canonicalize its `PathBuf`.
/// Returns `not_open` when the path is not a currently-open repo (SESS-01).
fn canonical_repo_path(
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<PathBuf, TrunkError> {
    let path_buf = state_map
        .get(path)
        .ok_or_else(|| TrunkError::new("not_open", format!("Repository not open: {}", path)))?;
    std::fs::canonicalize(path_buf).map_err(|e| TrunkError::new("io", e.to_string()))
}

/// Start a fresh review session for a currently-open repo (SESS-01 / D-08).
/// Rejects with `session_exists` if a file is already present — the client must
/// Resume or End-and-clear first (RESEARCH Open Question 2).
pub fn start_review_session_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(PathBuf, ReviewSession), TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    if review_store::session_exists(data_dir, &canonical) {
        return Err(TrunkError::new(
            "session_exists",
            "A review session already exists for this repository — resume or end it first",
        ));
    }
    let session = ReviewSession {
        schema_version: 1,
        commits: vec![],
        comments: vec![],
        draft_comment: None,
    };
    review_store::save_session(data_dir, &canonical, &session)?;
    Ok((canonical, session))
}

/// Load an existing session from disk for a currently-open repo (SESS-02 / D-14).
/// Returns the canonical path + the `LoadOutcome` so the command layer can branch
/// (Loaded → insert + emit; RecoveredCorrupt → fresh + toast; RefusedNewer → warn).
pub fn resume_review_session_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(PathBuf, LoadOutcome), TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    let outcome = review_store::load_session(data_dir, &canonical)?;
    Ok((canonical, outcome))
}

/// Hard-delete the session file for a currently-open repo (SESS-03 / D-13).
pub fn end_review_session_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<PathBuf, TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    review_store::delete_session(data_dir, &canonical)?;
    Ok(canonical)
}

/// Report the DISK half of the session status (D-14). `_inner` has no Tauri state
/// so it NEVER returns `Active` — it sets `ResumeAvailable` if the file exists,
/// else `None`. The thin command promotes to `Active` after locking the in-memory map.
pub fn get_review_session_status_inner(
    data_dir: &Path,
    path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<SessionStatus, TrunkError> {
    let canonical = canonical_repo_path(path, state_map)?;
    let file_exists = review_store::session_exists(data_dir, &canonical);
    let state = if file_exists {
        SessionState::ResumeAvailable
    } else {
        SessionState::None
    };
    Ok(SessionStatus {
        state,
        file_exists,
        canonical_path: canonical.to_string_lossy().into_owned(),
    })
}

/// Derive the final three-state status from disk presence + in-memory presence.
/// This is the merge `_inner` structurally cannot do (it has no Tauri state).
/// `Active` is produced ONLY here, when both halves are present.
fn merge_status(file_exists: bool, in_memory_present: bool) -> SessionState {
    match (file_exists, in_memory_present) {
        (true, true) => SessionState::Active,
        (true, false) => SessionState::ResumeAvailable,
        (false, _) => SessionState::None,
    }
}

/// Resolve `app_data_dir`, JSON-stringifying the error like the other commands.
fn resolve_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| {
        serde_json::to_string(&TrunkError::new("app_data_dir", e.to_string())).unwrap()
    })
}

// ── Selection core (Phase 66, Plan 01): pure, testable helpers ───────────────
// These take a `&git2::Repository` (no Tauri state) so the range/validation logic
// is provable against an in-process test repo. Plan 02 wraps them in commands.

/// Validate that `[base..tip]` is a meaningful inclusive range (SEL-01).
///
/// Order matters: `graph_descendant_of(x, x)` is `false`, so the `base == tip`
/// case (valid under D-02 inclusive semantics → set `{base}`) MUST short-circuit
/// before the descendant check. Unrelated histories surface as a `merge_base`
/// error; a base that is not an ancestor of the tip is a `bad_range`.
pub fn validate_range(
    repo: &git2::Repository,
    base: git2::Oid,
    tip: git2::Oid,
) -> Result<(), TrunkError> {
    if base == tip {
        return Ok(());
    }
    repo.merge_base(base, tip)
        .map_err(|_| TrunkError::new("unrelated_history", "These commits share no history"))?;
    if !repo.graph_descendant_of(tip, base).map_err(TrunkError::from)? {
        return Err(TrunkError::new("bad_range", "Base is not an ancestor of tip"));
    }
    Ok(())
}

/// Compute the OIDs in the inclusive range `[base..tip]` (SEL-01, D-02).
///
/// Walks `push(tip)` then `hide(base.parent(0))` so `base` itself stays in the
/// set. A root-commit base (`parent_count() == 0`) hides nothing, mirroring the
/// verified `interactive_rebase.rs` fallback, so it never panics on `parent_id(0)`.
pub fn compute_range_oids(
    repo: &git2::Repository,
    base: git2::Oid,
    tip: git2::Oid,
) -> Result<Vec<String>, TrunkError> {
    let mut revwalk = repo.revwalk().map_err(TrunkError::from)?;
    revwalk
        .set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)
        .map_err(TrunkError::from)?;
    revwalk.push(tip).map_err(TrunkError::from)?;

    let base_commit = repo.find_commit(base).map_err(TrunkError::from)?;
    if base_commit.parent_count() > 0 {
        revwalk
            .hide(base_commit.parent_id(0).map_err(TrunkError::from)?)
            .map_err(TrunkError::from)?;
    }
    // Root commit base: hide nothing — the whole ancestry through tip is included.

    revwalk
        .map(|oid| oid.map(|o| o.to_string()).map_err(TrunkError::from))
        .collect()
}

#[tauri::command]
pub async fn start_review_session(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let (canonical, session) = tauri::async_runtime::spawn_blocking(move || {
        start_review_session_inner(&data_dir, &path, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    // Disk-first ordering (D-10): _inner already wrote the file → in-memory → emit.
    sessions
        .0
        .lock()
        .unwrap()
        .insert(canonical.clone(), session);
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}

#[tauri::command]
pub async fn resume_review_session(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let data_dir_for_save = data_dir.clone();
    let (canonical, outcome) = tauri::async_runtime::spawn_blocking(move || {
        resume_review_session_inner(&data_dir, &path, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    match outcome {
        LoadOutcome::Loaded(session) => {
            sessions
                .0
                .lock()
                .unwrap()
                .insert(canonical.clone(), session);
        }
        LoadOutcome::None => {
            // No file to resume — nothing to load, nothing to insert.
        }
        LoadOutcome::RecoveredCorrupt => {
            // D-15: the corrupt file was quarantined; start a fresh session, persist
            // it (disk-first), cache it, and let the frontend toast the warning.
            let fresh = ReviewSession {
                schema_version: 1,
                commits: vec![],
                comments: vec![],
                draft_comment: None,
            };
            review_store::save_session(&data_dir_for_save, &canonical, &fresh)
                .map_err(|e| serde_json::to_string(&e).unwrap())?;
            sessions.0.lock().unwrap().insert(canonical.clone(), fresh);
        }
        LoadOutcome::RefusedNewer => {
            // D-16: a newer-schema file is left untouched; do NOT create a fresh
            // session, so a downgrade cannot wipe newer data.
            return Err(serde_json::to_string(&TrunkError::new(
                "newer_version",
                "This review session was written by a newer version of Trunk and cannot be opened",
            ))
            .unwrap());
        }
    }
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}

#[tauri::command]
pub async fn end_review_session(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let canonical = tauri::async_runtime::spawn_blocking(move || {
        end_review_session_inner(&data_dir, &path, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    // Disk-first ordering (D-10): _inner deleted the file → drop in-memory → emit.
    sessions.0.lock().unwrap().remove(&canonical);
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}

#[tauri::command]
pub async fn get_review_session_status(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<SessionStatus, String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let mut status = tauri::async_runtime::spawn_blocking(move || {
        get_review_session_status_inner(&data_dir, &path, &state_map)
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    // THREE-STATE MERGE: _inner returned the disk half; promote to Active here by
    // checking the canonical key in the in-memory map (the only place Active is born).
    let canonical = PathBuf::from(&status.canonical_path);
    let in_memory_present = sessions.0.lock().unwrap().contains_key(&canonical);
    status.state = merge_status(status.file_exists, in_memory_present);
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Oid, Repository, Signature};
    use tempfile::TempDir;

    #[test]
    fn merge_active_requires_both_halves() {
        assert_eq!(merge_status(true, true), SessionState::Active);
    }

    #[test]
    fn merge_resume_available_when_file_only() {
        assert_eq!(merge_status(true, false), SessionState::ResumeAvailable);
    }

    #[test]
    fn merge_none_when_no_file() {
        assert_eq!(merge_status(false, false), SessionState::None);
        assert_eq!(merge_status(false, true), SessionState::None);
    }

    // ── In-process test-repo helper (Wave 0) ─────────────────────────────────
    // tempfile::TempDir + git2::Repository::init builds a known topology so the
    // revwalk/validation helpers run against REAL commits (classical TDD: real
    // collaborator, no mocks). The TempDir is returned alongside the repo so the
    // caller keeps it alive for the test's duration (drop deletes the dir).

    /// A deterministic signature so commits are reproducible (F.I.R.S.T.: no clock).
    fn sig() -> Signature<'static> {
        Signature::new("Test", "test@example.com", &git2::Time::new(0, 0)).unwrap()
    }

    /// Commit a single empty-tree commit with the given parents, returning its OID.
    fn commit(repo: &Repository, message: &str, parents: &[Oid]) -> Oid {
        let tree_oid = repo.treebuilder(None).unwrap().write().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let parent_commits: Vec<_> = parents
            .iter()
            .map(|oid| repo.find_commit(*oid).unwrap())
            .collect();
        let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();
        let s = sig();
        repo.commit(None, &s, &s, message, &tree, &parent_refs)
            .unwrap()
    }

    /// A linear chain A→B→C→D plus a merge commit M (side branch off B, merged
    /// into the tip) so range walks can exercise both linear and merge topologies.
    struct TestRepo {
        _dir: TempDir,
        repo: Repository,
        a: Oid, // root
        b: Oid,
        c: Oid,
        d: Oid,
        side: Oid,  // off B
        merge: Oid, // merge of D and side
    }

    fn make_repo() -> TestRepo {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let a = commit(&repo, "A (root)", &[]);
        let b = commit(&repo, "B", &[a]);
        let c = commit(&repo, "C", &[b]);
        let d = commit(&repo, "D", &[c]);
        let side = commit(&repo, "side off B", &[b]);
        let merge = commit(&repo, "merge", &[d, side]);
        TestRepo {
            _dir: dir,
            repo,
            a,
            b,
            c,
            d,
            side,
            merge,
        }
    }

    /// A second, unrelated repository with its own root — for the
    /// unrelated-history rejection case (merge_base across these errors).
    fn make_unrelated_repo() -> (TempDir, Repository, Oid) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        let root = commit(&repo, "unrelated root", &[]);
        (dir, repo, root)
    }

    // ── Task 1: Range walk + validation ──────────────────────────────────────

    #[test]
    fn seed_range_inclusive() {
        let t = make_repo();
        let oids = compute_range_oids(&t.repo, t.b, t.d).unwrap();
        // [B..D] inclusive: both endpoints present, plus C between them.
        assert!(oids.contains(&t.b.to_string()), "base B must be included");
        assert!(oids.contains(&t.d.to_string()), "tip D must be included");
        assert!(oids.contains(&t.c.to_string()), "intermediate C must be included");
        assert!(!oids.contains(&t.a.to_string()), "A is below base, excluded");
    }

    #[test]
    fn seed_range_root_base() {
        let t = make_repo();
        // Root commit base: walk hides nothing, full ancestry through tip included.
        let oids = compute_range_oids(&t.repo, t.a, t.d).unwrap();
        for oid in [t.a, t.b, t.c, t.d] {
            assert!(
                oids.contains(&oid.to_string()),
                "root-base range must include {oid}"
            );
        }
    }

    #[test]
    fn seed_range_base_eq_tip() {
        let t = make_repo();
        assert!(validate_range(&t.repo, t.c, t.c).is_ok());
        let oids = compute_range_oids(&t.repo, t.c, t.c).unwrap();
        assert_eq!(oids, vec![t.c.to_string()], "base==tip yields exactly {{base}}");
    }

    #[test]
    fn seed_range_rejects_non_ancestor() {
        let t = make_repo();
        // side is NOT an ancestor of D (it forks off B onto its own line).
        let err = validate_range(&t.repo, t.side, t.d).unwrap_err();
        assert_eq!(err.code, "bad_range");
    }

    #[test]
    fn seed_range_rejects_unrelated() {
        let t = make_repo();
        let (_other_dir, _other_repo, other_root) = make_unrelated_repo();
        // other_root lives in a different repo, so it shares no history with D.
        let err = validate_range(&t.repo, other_root, t.d).unwrap_err();
        assert_eq!(err.code, "unrelated_history");
    }

    #[test]
    fn merge_commit_selectable() {
        let t = make_repo();
        // D-08: a merge commit can be the tip and appears in the computed range,
        // with no is_merge gate filtering it out.
        let oids = compute_range_oids(&t.repo, t.b, t.merge).unwrap();
        assert!(
            oids.contains(&t.merge.to_string()),
            "merge commit must be selectable as tip"
        );
        assert!(oids.contains(&t.side.to_string()), "merge brings in side branch");
    }
}
