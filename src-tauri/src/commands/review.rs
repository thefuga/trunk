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
use crate::git::types::{Comment, DraftComment, ReviewSession};
use crate::state::{CommitCache, RepoState, ReviewSessionsState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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

/// A single commit in the review session, rendered by the panel (D-05) and
/// consumed as a membership set by the graph (D-04/D-06). Serialize-default
/// snake_case matches `GraphCommit`, whose fields it copies 1:1.
#[derive(Debug, Serialize, Clone)]
pub struct SessionCommit {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
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
    if !repo
        .graph_descendant_of(tip, base)
        .map_err(TrunkError::from)?
    {
        return Err(TrunkError::new(
            "bad_range",
            "Base is not an ancestor of tip",
        ));
    }
    Ok(())
}

/// Compute the OIDs in the inclusive range `[base..tip]` (SEL-01, D-02).
///
/// Walks `push(tip)` then hides EVERY parent of `base` so `base` itself stays in
/// the set while none of its ancestors do. Hiding all parents (not just the
/// first) matters when `base` is a merge commit: hiding only `parent(0)` would
/// leave the second-parent side branch reachable from `tip` and leak it into the
/// selection (CR-01). A root-commit base (`parent_count() == 0`) hides nothing,
/// mirroring the verified `interactive_rebase.rs` fallback, so it never panics.
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
    for i in 0..base_commit.parent_count() {
        revwalk
            .hide(base_commit.parent_id(i).map_err(TrunkError::from)?)
            .map_err(TrunkError::from)?;
    }
    // Root commit base: hide nothing — the whole ancestry through tip is included.

    revwalk
        .map(|oid| oid.map(|o| o.to_string()).map_err(TrunkError::from))
        .collect()
}

/// Add `oid` to the selection if absent — idempotent (SEL-02, D-06).
pub fn apply_add(commits: &mut Vec<String>, oid: &str) {
    if !commits.iter().any(|c| c == oid) {
        commits.push(oid.to_string());
    }
}

/// Remove every occurrence of `oid` from the selection; missing is a no-op (SEL-03).
pub fn apply_remove(commits: &mut Vec<String>, oid: &str) {
    commits.retain(|c| c != oid);
}

/// Union `incoming` into `existing`, preserving hand-picked commits and deduping
/// (D-03). Store order is irrelevant — `intersect_graph_order` re-imposes graph
/// order on read, so the set is the only thing that matters here.
pub fn union_dedup(existing: &[String], incoming: Vec<String>) -> Vec<String> {
    let mut set: std::collections::HashSet<String> = existing.iter().cloned().collect();
    set.extend(incoming);
    set.into_iter().collect()
}

/// Order the session set by the full cached graph order, deduped, as the SEL-04
/// list. OIDs present in the cached `graph` come first in graph order; any
/// selected OID absent from the graph is appended via `repo.find_commit`, and an
/// OID that even `find_commit` can't resolve is included with an `(unavailable)`
/// summary rather than silently dropped (Phase 65 "never silently destroy").
pub fn intersect_graph_order(
    commits: &[String],
    graph: &crate::git::types::GraphResult,
    repo: &git2::Repository,
) -> Vec<SessionCommit> {
    let want: std::collections::HashSet<&String> = commits.iter().collect();
    let mut out: Vec<SessionCommit> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for c in graph.commits.iter().filter(|c| want.contains(&c.oid)) {
        if seen.insert(c.oid.clone()) {
            out.push(SessionCommit {
                oid: c.oid.clone(),
                short_oid: c.short_oid.clone(),
                summary: c.summary.clone(),
            });
        }
    }

    // Fallback: selected OIDs not in the cached graph (orphaned/force-pushed).
    for oid_str in commits {
        if !seen.insert(oid_str.clone()) {
            continue;
        }
        let summary = git2::Oid::from_str(oid_str)
            .ok()
            .and_then(|oid| repo.find_commit(oid).ok())
            .and_then(|c| c.summary().map(|s| s.to_owned()))
            .unwrap_or_else(|| "(unavailable)".to_string());
        out.push(SessionCommit {
            oid: oid_str.clone(),
            short_oid: oid_str.chars().take(7).collect(),
            summary,
        });
    }

    out
}

// ── Selection RMW core (Phase 66, Plan 02): mutex-serialized read-modify-write ─
// Add/remove/seed mutate the PERSISTED session — unlike the Phase 65 create/delete
// `_inner`s, which read no prior state. Two rapid clicks racing the read-mutate-
// write would lose a write (Pitfall 2), so the ENTIRE
// read→mutate→save_session→map-write runs under the `ReviewSessionsState` mutex.
// These free functions take the raw `&Mutex<..>` (not Tauri `State`) so the
// serialization is unit-testable without a Tauri runtime (`selection_rmw_serialized`).

/// Read-modify-write the session's commit set under the sessions mutex.
///
/// The lock is held across read → `mutate` → `save_session` → map-write so disk
/// and memory never diverge and no concurrent caller observes a stale set. The
/// blocking disk write inside the critical section is fine (small atomic
/// tmp+rename); the lock is never held across an `.await`. Returns `no_session`
/// when there is no in-memory session for `canonical` (distinct from `not_open`).
fn mutate_session_rmw<F>(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    mutate: F,
) -> Result<(), TrunkError>
where
    F: FnOnce(&mut ReviewSession),
{
    let mut map = sessions.lock().unwrap();
    let session = map.get_mut(canonical).ok_or_else(|| {
        TrunkError::new("no_session", "No active review session for this repository")
    })?;
    mutate(session);
    review_store::save_session(data_dir, canonical, session)?;
    Ok(())
}

/// Union `range_oids` into the session set, deduped (SEL-01, D-03). One walk, one
/// save, one map-write — never decomposed into N adds.
fn seed_review_range_rmw(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    range_oids: Vec<String>,
) -> Result<(), TrunkError> {
    mutate_session_rmw(data_dir, canonical, sessions, |session| {
        session.commits = union_dedup(&session.commits, range_oids);
    })
}

/// Add `oid` to the session set if absent (SEL-02, idempotent).
fn add_review_commit_rmw(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    oid: &str,
) -> Result<(), TrunkError> {
    mutate_session_rmw(data_dir, canonical, sessions, |session| {
        apply_add(&mut session.commits, oid);
    })
}

/// Remove every occurrence of `oid` from the session set (SEL-03, no-op if absent).
fn remove_review_commit_rmw(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    oid: &str,
) -> Result<(), TrunkError> {
    mutate_session_rmw(data_dir, canonical, sessions, |session| {
        apply_remove(&mut session.commits, oid);
    })
}

// ── Comment capture (Phase 67, Plan 02): shared dumb writers ──────────────────
// `add_comment_inner` and `save_draft_comment_inner` reuse the SAME serialized
// `mutate_session_rmw` as the selection RMW functions — the comment writer pushes
// a fully-formed `Comment` (the `Anchor` already carries source/side from the TS
// adapter) and clears the single draft slot; the draft writer replaces that slot.

/// Argument bundle for `add_comment_inner` (the testable core). The thin command
/// takes flat named args off the wire and assembles this; `_inner` is the wedge so
/// disk behavior is provable with a `TempDir` (no Tauri runtime).
#[derive(Debug)]
pub struct AddCommentRequest {
    pub path: String,
    pub text: String,
    pub anchor: crate::git::types::Anchor,
    pub cached_excerpt: String,
}

/// Argument bundle for `save_draft_comment_inner`. `DraftComment` has NO
/// `cached_excerpt` (schema asymmetry, Pitfall 5) — the draft carries text+anchor.
#[derive(Debug)]
pub struct SaveDraftCommentRequest {
    pub path: String,
    pub text: String,
    pub anchor: Option<crate::git::types::Anchor>,
}

/// Submit a comment: push the fully-formed `Comment` (the `Anchor` already carries
/// source/side from the TS adapter — L-08 dumb writer) and clear the single draft
/// slot so the composer never reopens with stale text. The push + clear + save run
/// inside the serialized RMW critical section, so concurrent submits never lose a
/// write and disk/memory never diverge.
fn add_comment_inner(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    req: AddCommentRequest,
) -> Result<(), TrunkError> {
    mutate_session_rmw(data_dir, canonical, sessions, |session| {
        session.comments.push(Comment {
            id: uuid::Uuid::new_v4().to_string(),
            text: req.text,
            anchor: Some(req.anchor),
            cached_excerpt: Some(req.cached_excerpt),
            commit_oid: None,
        });
        session.draft_comment = None;
    })
}

/// Write/replace the single draft-comment slot (per-keystroke). `DraftComment` has
/// NO `cached_excerpt` (schema asymmetry, Pitfall 5) — only text + anchor persist;
/// the excerpt is computed at submit. Does NOT emit `session-changed` (drafts are
/// not panel-visible until Phase 69; per-keystroke emits would cause reload storms).
fn save_draft_comment_inner(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    req: SaveDraftCommentRequest,
) -> Result<(), TrunkError> {
    mutate_session_rmw(data_dir, canonical, sessions, |session| {
        session.draft_comment = Some(DraftComment {
            text: req.text,
            anchor: req.anchor,
        });
    })
}

/// Seed the session from an inclusive commit range `[base..tip]` (SEL-01, D-02/D-03).
///
/// git2 work (open repo, parse OIDs, validate, walk) runs in `spawn_blocking` on a
/// cloned `RepoState` snapshot and the RAW path; the resulting range is then
/// unioned into the session under the sessions mutex (one emit per gesture).
#[tauri::command]
pub async fn seed_review_range(
    path: String,
    base_oid: String,
    tip_oid: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;

    let path_for_blocking = path.clone();
    let (canonical, range_oids) = tauri::async_runtime::spawn_blocking(
        move || -> Result<(PathBuf, Vec<String>), TrunkError> {
            let canonical = canonical_repo_path(&path_for_blocking, &state_map)?;
            let repo = git2::Repository::open(&path_for_blocking).map_err(TrunkError::from)?;
            let base = git2::Oid::from_str(&base_oid).map_err(TrunkError::from)?;
            let tip = git2::Oid::from_str(&tip_oid).map_err(TrunkError::from)?;
            validate_range(&repo, base, tip)?;
            let range_oids = compute_range_oids(&repo, base, tip)?;
            Ok((canonical, range_oids))
        },
    )
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;

    seed_review_range_rmw(&data_dir, &canonical, &sessions.0, range_oids)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}

/// Add a single hand-picked commit to the session (SEL-02).
#[tauri::command]
pub async fn add_review_commit(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let canonical =
        canonical_repo_path(&path, &state_map).map_err(|e| serde_json::to_string(&e).unwrap())?;

    add_review_commit_rmw(&data_dir, &canonical, &sessions.0, &oid)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}

/// Remove a single commit from the session (SEL-03).
#[tauri::command]
pub async fn remove_review_commit(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let canonical =
        canonical_repo_path(&path, &state_map).map_err(|e| serde_json::to_string(&e).unwrap())?;

    remove_review_commit_rmw(&data_dir, &canonical, &sessions.0, &oid)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}

/// Submit a comment to the active session (ANCH-01). Dumb writer: the `Anchor`
/// already carries source/side from the TS adapter (L-08). Emits `session-changed`
/// because a submitted comment is panel-visible state.
///
/// Flat named args (not a single struct param) mirror the sibling commands and the
/// `safeInvoke("add_comment", { path, text, anchor, cachedExcerpt })` flat wire
/// shape; Tauri maps the camelCase JS key `cachedExcerpt` to `cached_excerpt`.
#[tauri::command]
pub async fn add_comment(
    path: String,
    text: String,
    anchor: crate::git::types::Anchor,
    cached_excerpt: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let canonical =
        canonical_repo_path(&path, &state_map).map_err(|e| serde_json::to_string(&e).unwrap())?;

    let req = AddCommentRequest {
        path,
        text,
        anchor,
        cached_excerpt,
    };
    add_comment_inner(&data_dir, &canonical, &sessions.0, req)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}

/// Persist the in-progress draft comment on keystroke (ANCH-01). Does NOT emit
/// `session-changed`: drafts are not panel-visible until Phase 69, and per-keystroke
/// emits would cause needless reload storms (RESEARCH Q3).
///
/// Flat named args mirror `add_comment` and the
/// `safeInvoke("save_draft_comment", { path, text, anchor })` flat wire shape.
#[tauri::command]
pub async fn save_draft_comment(
    path: String,
    text: String,
    anchor: Option<crate::git::types::Anchor>,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let canonical =
        canonical_repo_path(&path, &state_map).map_err(|e| serde_json::to_string(&e).unwrap())?;

    let req = SaveDraftCommentRequest { path, text, anchor };
    save_draft_comment_inner(&data_dir, &canonical, &sessions.0, req)
        .map_err(|e| serde_json::to_string(&e).unwrap())?;
    Ok(())
}

/// List the session's commits in graph order (SEL-04). No mutation, no emit.
///
/// Dual path-keying (Pitfall 3): the session set is read by CANONICAL key from the
/// in-memory map; the graph order comes from `CommitCache` by RAW path. A missing
/// in-memory session is `no_session` (distinct from `canonical_repo_path`'s
/// `not_open`) so the frontend can branch on session-active vs repo-not-open.
#[tauri::command]
pub async fn list_session_commits(
    path: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    cache: State<'_, CommitCache>,
) -> Result<Vec<SessionCommit>, String> {
    let state_map = state.0.lock().unwrap().clone();
    let canonical =
        canonical_repo_path(&path, &state_map).map_err(|e| serde_json::to_string(&e).unwrap())?;

    // Read the session set by CANONICAL key; a missing entry is `no_session`.
    let commits = {
        let map = sessions.0.lock().unwrap();
        map.get(&canonical)
            .ok_or_else(|| {
                serde_json::to_string(&TrunkError::new(
                    "no_session",
                    "No active review session for this repository",
                ))
                .unwrap()
            })?
            .commits
            .clone()
    };

    // Read the full graph order from CommitCache by RAW path (Pitfall 3).
    let graph = {
        let map = cache.0.lock().unwrap();
        map.get(&path)
            .ok_or_else(|| {
                serde_json::to_string(&TrunkError::new("not_open", "Repository not open")).unwrap()
            })?
            .clone()
    };

    // Open the repo fresh in spawn_blocking (orphan fallback needs find_commit);
    // never hold the RepoState lock across git2 work.
    let result =
        tauri::async_runtime::spawn_blocking(move || -> Result<Vec<SessionCommit>, TrunkError> {
            let repo = git2::Repository::open(&path).map_err(TrunkError::from)?;
            Ok(intersect_graph_order(&commits, &graph, &repo))
        })
        .await
        .map_err(|e| {
            serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap()
        })?
        .map_err(|e| serde_json::to_string(&e).unwrap())?;

    Ok(result)
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
        assert!(
            oids.contains(&t.c.to_string()),
            "intermediate C must be included"
        );
        assert!(
            !oids.contains(&t.a.to_string()),
            "A is below base, excluded"
        );
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
        assert_eq!(
            oids,
            vec![t.c.to_string()],
            "base==tip yields exactly {{base}}"
        );
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
        assert!(
            oids.contains(&t.side.to_string()),
            "merge brings in side branch"
        );
    }

    #[test]
    fn seed_range_merge_base_excludes_side_branch() {
        let t = make_repo();
        // D-02 + D-08: when the range BASE is a merge commit, [base..tip] includes
        // base but excludes ALL of base's ancestors — including the second-parent
        // side branch. Regression for CR-01 (hiding only parent_id(0) leaked the
        // side branch into the selection).
        let top = commit(&t.repo, "top of merge", &[t.merge]);
        let oids = compute_range_oids(&t.repo, t.merge, top).unwrap();
        assert!(oids.contains(&top.to_string()), "tip must be included");
        assert!(
            oids.contains(&t.merge.to_string()),
            "merge base must be included"
        );
        assert!(
            !oids.contains(&t.side.to_string()),
            "second-parent side branch must NOT leak when base is a merge"
        );
        assert!(
            !oids.contains(&t.d.to_string()),
            "first-parent ancestor must be excluded"
        );
    }

    // ── Task 2: Set union / add / remove / dedup ─────────────────────────────

    #[test]
    fn add_commit_idempotent() {
        let mut commits = vec!["aaa".to_string()];
        apply_add(&mut commits, "bbb");
        assert_eq!(commits, vec!["aaa".to_string(), "bbb".to_string()]);
        // SEL-02: a second add of the same oid is a no-op (no duplicate).
        apply_add(&mut commits, "bbb");
        assert_eq!(commits, vec!["aaa".to_string(), "bbb".to_string()]);
    }

    #[test]
    fn remove_commit() {
        let mut commits = vec!["aaa".to_string(), "bbb".to_string(), "ccc".to_string()];
        apply_remove(&mut commits, "bbb");
        assert_eq!(commits, vec!["aaa".to_string(), "ccc".to_string()]);
        // SEL-03: removing an oid not in the set is a safe no-op.
        apply_remove(&mut commits, "zzz");
        assert_eq!(commits, vec!["aaa".to_string(), "ccc".to_string()]);
    }

    #[test]
    fn seed_range_unions_dedups() {
        // D-03: hand-picked commits survive a range seed; the range unions in;
        // overlapping oids are deduped.
        let existing = vec!["picked".to_string(), "shared".to_string()];
        let incoming = vec![
            "shared".to_string(),
            "range1".to_string(),
            "range2".to_string(),
        ];
        let result = union_dedup(&existing, incoming);
        for oid in ["picked", "shared", "range1", "range2"] {
            assert!(
                result.contains(&oid.to_string()),
                "union must contain {oid}"
            );
        }
        assert_eq!(result.len(), 4, "no duplicates after union");
    }

    // ── Task 3: Graph-ordered intersection (SEL-04) ──────────────────────────

    /// A minimal `GraphCommit` for fixtures — only the fields `SessionCommit`
    /// copies (oid, short_oid, summary) carry meaning; the rest are inert.
    fn graph_commit(oid: &str, summary: &str) -> crate::git::types::GraphCommit {
        crate::git::types::GraphCommit {
            oid: oid.to_string(),
            short_oid: oid.chars().take(7).collect(),
            summary: summary.to_string(),
            body: None,
            author_name: String::new(),
            author_email: String::new(),
            author_timestamp: 0,
            parent_oids: vec![],
            column: 0,
            color_index: 0,
            edges: vec![],
            refs: vec![],
            is_head: false,
            is_merge: false,
            is_branch_tip: false,
            is_stash: false,
        }
    }

    #[test]
    fn list_session_commits_graph_order() {
        let t = make_repo();
        // Graph order: D, C, B (newest-first slice of the cached graph).
        let graph = crate::git::types::GraphResult {
            commits: vec![
                graph_commit(&t.d.to_string(), "D"),
                graph_commit(&t.c.to_string(), "C"),
                graph_commit(&t.b.to_string(), "B"),
            ],
            max_columns: 1,
        };
        // Session set given in a DIFFERENT order, with a duplicate.
        let session = vec![
            t.b.to_string(),
            t.d.to_string(),
            t.b.to_string(), // dup — must collapse
        ];
        let out = intersect_graph_order(&session, &graph, &t.repo);
        let oids: Vec<String> = out.iter().map(|c| c.oid.clone()).collect();
        // Re-imposed graph order (D before B), deduped, C excluded (not selected).
        assert_eq!(oids, vec![t.d.to_string(), t.b.to_string()]);
        assert_eq!(out[0].summary, "D");
    }

    #[test]
    fn list_session_commits_orphan_fallback() {
        let t = make_repo();
        // Graph contains only D; the session also selects A (absent from graph but
        // resolvable via find_commit) and a bogus OID (truly unresolvable).
        let graph = crate::git::types::GraphResult {
            commits: vec![graph_commit(&t.d.to_string(), "D")],
            max_columns: 1,
        };
        let bogus = "0".repeat(40);
        let session = vec![t.d.to_string(), t.a.to_string(), bogus.clone()];
        let out = intersect_graph_order(&session, &graph, &t.repo);
        let oids: Vec<String> = out.iter().map(|c| c.oid.clone()).collect();
        // D from the graph, then the appended fallbacks — none silently dropped.
        assert!(oids.contains(&t.d.to_string()), "in-graph commit present");
        assert!(
            oids.contains(&t.a.to_string()),
            "orphan resolvable via find_commit must be appended"
        );
        assert!(
            oids.contains(&bogus),
            "unresolvable orphan must still appear (never dropped)"
        );
        let unresolved = out.iter().find(|c| c.oid == bogus).unwrap();
        assert_eq!(unresolved.summary, "(unavailable)");
    }

    // ── Task 1: RMW serialization (SEL-02/03, Pitfall 2) ─────────────────────

    #[test]
    fn selection_rmw_serialized() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let data_dir = TempDir::new().unwrap();
        let canonical = data_dir.path().join("repo-canonical");
        let sessions: Arc<Mutex<HashMap<PathBuf, ReviewSession>>> =
            Arc::new(Mutex::new(HashMap::new()));
        sessions.lock().unwrap().insert(
            canonical.clone(),
            ReviewSession {
                schema_version: 1,
                commits: vec![],
                comments: vec![],
                draft_comment: None,
            },
        );

        // 50 threads each add a distinct oid concurrently. Without serialization
        // the read-modify-write races and writes clobber each other; with the
        // mutex held across read→mutate→save→map-write, every add survives.
        let n = 50;
        let mut handles = Vec::new();
        for i in 0..n {
            let sessions = Arc::clone(&sessions);
            let data_dir = data_dir.path().to_path_buf();
            let canonical = canonical.clone();
            handles.push(thread::spawn(move || {
                let oid = format!("oid-{i:04}");
                add_review_commit_rmw(&data_dir, &canonical, &sessions, &oid).unwrap();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        // In-memory set holds all N adds (no lost write).
        {
            let in_memory = sessions.lock().unwrap();
            let commits = &in_memory.get(&canonical).unwrap().commits;
            assert_eq!(
                commits.len(),
                n,
                "every concurrent add must survive in memory"
            );
            for i in 0..n {
                assert!(
                    commits.contains(&format!("oid-{i:04}")),
                    "oid-{i:04} lost under concurrent RMW"
                );
            }
        }

        // Disk reflects the same set (save_session ran inside the critical section).
        match review_store::load_session(data_dir.path(), &canonical).unwrap() {
            LoadOutcome::Loaded(s) => {
                assert_eq!(s.commits.len(), n, "disk must hold every concurrent add");
            }
            _ => panic!("expected a loadable session on disk"),
        }

        // Removing one oid through the same serialized path leaves a stable set.
        remove_review_commit_rmw(data_dir.path(), &canonical, &sessions, "oid-0000").unwrap();
        let in_memory = sessions.lock().unwrap();
        let commits = &in_memory.get(&canonical).unwrap().commits;
        assert_eq!(commits.len(), n - 1, "remove drops exactly one");
        assert!(
            !commits.contains(&"oid-0000".to_string()),
            "removed oid must be gone"
        );
    }

    #[test]
    fn rmw_missing_session_is_no_session_error() {
        use std::sync::Mutex;
        let data_dir = TempDir::new().unwrap();
        let canonical = data_dir.path().join("absent");
        let sessions: Mutex<HashMap<PathBuf, ReviewSession>> = Mutex::new(HashMap::new());
        // No in-memory session for `canonical` → RMW must reject with `no_session`.
        let err = add_review_commit_rmw(data_dir.path(), &canonical, &sessions, "x").unwrap_err();
        assert_eq!(err.code, "no_session");
    }

    // ── Phase 67 Plan 02: comment capture (add_comment / save_draft_comment) ──
    // `Comment` + `DraftComment` already come through `use super::*`.
    use crate::git::types::{Anchor, Side, Source};

    /// A `TempDir` data dir + a sessions map seeded with one empty session keyed
    /// by a synthetic canonical path. No git repo is needed — these writers only
    /// touch the persisted JSON store (mirrors `selection_rmw_serialized:940-952`).
    fn seeded_sessions(data_dir: &TempDir) -> (PathBuf, Mutex<HashMap<PathBuf, ReviewSession>>) {
        let canonical = data_dir.path().join("repo-canonical");
        let mut map = HashMap::new();
        map.insert(
            canonical.clone(),
            ReviewSession {
                schema_version: 1,
                commits: vec![],
                comments: vec![],
                draft_comment: None,
            },
        );
        (canonical, Mutex::new(map))
    }

    /// A non-trivial anchor with all six fields distinct (side=Old, source=Diff).
    fn distinct_anchor() -> Anchor {
        Anchor {
            commit_oid: "abc123def456".to_string(),
            file_path: "src/lib/foo.rs".to_string(),
            source: Source::Diff,
            side: Side::Old,
            start_line: 12,
            end_line: 34,
        }
    }

    fn loaded(data_dir: &TempDir, canonical: &Path) -> ReviewSession {
        match review_store::load_session(data_dir.path(), canonical).unwrap() {
            LoadOutcome::Loaded(s) => s,
            _ => panic!("expected a loadable session on disk"),
        }
    }

    // Test 1 (SC-1): add_comment_inner pushes a Comment with anchor+excerpt, persists.
    #[test]
    fn add_comment_persists_comment_with_anchor_and_excerpt() {
        let data_dir = TempDir::new().unwrap();
        let (canonical, sessions) = seeded_sessions(&data_dir);
        let req = AddCommentRequest {
            path: "ignored".to_string(),
            text: "looks good".to_string(),
            anchor: distinct_anchor(),
            cached_excerpt: "let x = 1;".to_string(),
        };
        add_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap();

        let s = loaded(&data_dir, &canonical);
        assert_eq!(s.comments.len(), 1);
        assert_eq!(s.comments[0].text, "looks good");
        assert_eq!(s.comments[0].cached_excerpt.as_deref(), Some("let x = 1;"));
        assert!(
            s.comments[0].anchor.is_some(),
            "comment must carry its anchor"
        );
    }

    // Test 2: submit clears the single draft_comment slot.
    #[test]
    fn add_comment_clears_draft_slot() {
        let data_dir = TempDir::new().unwrap();
        let (canonical, sessions) = seeded_sessions(&data_dir);
        // Seed a pre-existing draft that submit must clear.
        sessions
            .lock()
            .unwrap()
            .get_mut(&canonical)
            .unwrap()
            .draft_comment = Some(DraftComment {
            text: "half-typed".to_string(),
            anchor: Some(distinct_anchor()),
        });

        let req = AddCommentRequest {
            path: "ignored".to_string(),
            text: "done".to_string(),
            anchor: distinct_anchor(),
            cached_excerpt: "x".to_string(),
        };
        add_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap();

        let s = loaded(&data_dir, &canonical);
        assert!(
            s.draft_comment.is_none(),
            "submit must clear the draft slot so the composer never reopens with stale text"
        );
    }

    // Test 3 (no-session): missing in-memory session → "no_session".
    #[test]
    fn add_comment_missing_session_is_no_session_error() {
        let data_dir = TempDir::new().unwrap();
        let canonical = data_dir.path().join("absent");
        let sessions: Mutex<HashMap<PathBuf, ReviewSession>> = Mutex::new(HashMap::new());
        let req = AddCommentRequest {
            path: "ignored".to_string(),
            text: "t".to_string(),
            anchor: distinct_anchor(),
            cached_excerpt: "e".to_string(),
        };
        let err = add_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap_err();
        assert_eq!(err.code, "no_session");
    }

    // Test 4 (L-08): a Source::FullFile anchor persists unchanged (Phase-68 contract).
    #[test]
    fn add_comment_persists_full_file_source_unchanged() {
        let data_dir = TempDir::new().unwrap();
        let (canonical, sessions) = seeded_sessions(&data_dir);
        let mut anchor = distinct_anchor();
        anchor.source = Source::FullFile;
        let req = AddCommentRequest {
            path: "ignored".to_string(),
            text: "whole-file note".to_string(),
            anchor,
            cached_excerpt: "e".to_string(),
        };
        add_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap();

        let s = loaded(&data_dir, &canonical);
        let stored = s.comments[0].anchor.as_ref().unwrap();
        assert_eq!(
            stored.source,
            Source::FullFile,
            "add_comment must persist Source::FullFile verbatim (L-08 dumb-writer contract)"
        );
    }

    // Test 5 (SC-2): a non-trivial anchor round-trips with every field identical.
    #[test]
    fn add_comment_anchor_round_trips_all_six_fields() {
        let data_dir = TempDir::new().unwrap();
        let (canonical, sessions) = seeded_sessions(&data_dir);
        let anchor = distinct_anchor();
        let req = AddCommentRequest {
            path: "ignored".to_string(),
            text: "t".to_string(),
            anchor: anchor.clone(),
            cached_excerpt: "e".to_string(),
        };
        add_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap();

        let s = loaded(&data_dir, &canonical);
        let stored = s.comments[0].anchor.as_ref().unwrap();
        // Anchor derives no PartialEq (frozen schema) — assert field-by-field.
        assert_eq!(stored.commit_oid, anchor.commit_oid);
        assert_eq!(stored.file_path, anchor.file_path);
        assert_eq!(stored.source, anchor.source);
        assert_eq!(stored.side, anchor.side);
        assert_eq!(stored.start_line, anchor.start_line);
        assert_eq!(stored.end_line, anchor.end_line);
    }

    // Test 6 (concurrency): N concurrent submits all survive on disk.
    #[test]
    fn add_comment_concurrent_submits_all_survive() {
        use std::sync::Arc;
        use std::thread;

        let data_dir = TempDir::new().unwrap();
        let canonical = data_dir.path().join("repo-canonical");
        let sessions: Arc<Mutex<HashMap<PathBuf, ReviewSession>>> =
            Arc::new(Mutex::new(HashMap::new()));
        sessions.lock().unwrap().insert(
            canonical.clone(),
            ReviewSession {
                schema_version: 1,
                commits: vec![],
                comments: vec![],
                draft_comment: None,
            },
        );

        let n = 50;
        let mut handles = Vec::new();
        for i in 0..n {
            let sessions = Arc::clone(&sessions);
            let data_dir = data_dir.path().to_path_buf();
            let canonical = canonical.clone();
            handles.push(thread::spawn(move || {
                let req = AddCommentRequest {
                    path: "ignored".to_string(),
                    text: format!("comment-{i:04}"),
                    anchor: distinct_anchor(),
                    cached_excerpt: "e".to_string(),
                };
                add_comment_inner(&data_dir, &canonical, &sessions, req).unwrap();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        match review_store::load_session(data_dir.path(), &canonical).unwrap() {
            LoadOutcome::Loaded(s) => {
                assert_eq!(s.comments.len(), n, "every concurrent submit must survive");
            }
            _ => panic!("expected a loadable session on disk"),
        }
    }

    // Test 7 (T-67-02): a traversal-shaped file_path round-trips verbatim AND does
    // not influence the on-disk session filename (filename is the FNV-1a hash).
    #[test]
    fn add_comment_path_traversal_round_trips_without_affecting_filename() {
        let data_dir = TempDir::new().unwrap();
        let (canonical, sessions) = seeded_sessions(&data_dir);
        let mut anchor = distinct_anchor();
        anchor.file_path = "../../etc/passwd".to_string();
        let req = AddCommentRequest {
            path: "ignored".to_string(),
            text: "t".to_string(),
            anchor,
            cached_excerpt: "e".to_string(),
        };
        add_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap();

        // The anchor metadata round-trips verbatim.
        let s = loaded(&data_dir, &canonical);
        assert_eq!(
            s.comments[0].anchor.as_ref().unwrap().file_path,
            "../../etc/passwd",
            "traversal-shaped file_path is metadata and must round-trip unchanged"
        );

        // The on-disk filename is the FNV-1a hash of the canonical path, never the
        // anchor: exactly one session file, named `<16 hex>.json`, no traversal.
        let entries: Vec<_> = std::fs::read_dir(data_dir.path().join("sessions"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries.len(), 1, "exactly one session file on disk");
        let name = &entries[0];
        assert!(
            name.len() == "0123456789abcdef.json".len()
                && name.ends_with(".json")
                && name[..16].chars().all(|c| c.is_ascii_hexdigit()),
            "session filename must be the 16-hex FNV-1a hash, got {name}"
        );
        assert!(
            !name.contains("..") && !name.contains("etc") && !name.contains("passwd"),
            "anchor file_path must never leak into the session filename"
        );
    }

    // Test 8: save_draft_comment_inner writes the draft slot (no cached_excerpt).
    #[test]
    fn save_draft_comment_persists_text_and_anchor() {
        let data_dir = TempDir::new().unwrap();
        let (canonical, sessions) = seeded_sessions(&data_dir);
        let req = SaveDraftCommentRequest {
            path: "ignored".to_string(),
            text: "typing...".to_string(),
            anchor: Some(distinct_anchor()),
        };
        save_draft_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap();

        let s = loaded(&data_dir, &canonical);
        let draft = s.draft_comment.as_ref().expect("draft must be persisted");
        assert_eq!(draft.text, "typing...");
        assert_eq!(
            draft.anchor.as_ref().unwrap().file_path,
            "src/lib/foo.rs",
            "draft must carry its anchor"
        );
        // No comment was added — drafts are not comments.
        assert!(s.comments.is_empty(), "a draft does not append a comment");
    }

    // Test 9: save_draft_comment_inner no-session → "no_session".
    #[test]
    fn save_draft_comment_missing_session_is_no_session_error() {
        let data_dir = TempDir::new().unwrap();
        let canonical = data_dir.path().join("absent");
        let sessions: Mutex<HashMap<PathBuf, ReviewSession>> = Mutex::new(HashMap::new());
        let req = SaveDraftCommentRequest {
            path: "ignored".to_string(),
            text: "t".to_string(),
            anchor: None,
        };
        let err =
            save_draft_comment_inner(data_dir.path(), &canonical, &sessions, req).unwrap_err();
        assert_eq!(err.code, "no_session");
    }
}
