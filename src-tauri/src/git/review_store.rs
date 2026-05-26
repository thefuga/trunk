//! Per-repo review-session persistence (Phase 65, Plan 02).
//!
//! This is the one file in the phase with no codebase analog: it owns all
//! session file I/O and the security-relevant recovery behavior.
//!
//! - Writes are atomic via tmp-in-same-dir + `sync_all` + `rename` (D-10).
//! - The on-disk filename is a build-stable FNV-1a hash of the canonical path
//!   (D-11) — also the path-traversal mitigation, since the hash can contain no
//!   `..`, separators, or OS-specific verbatim prefixes.
//! - `load_session` peeks `schema_version` before a full deserialize so it can
//!   refuse a newer-schema file untouched (D-16) and quarantine an unparseable
//!   file to a `.corrupt` sidecar rather than destroying it (D-15).
//!
//! Every function takes `data_dir: &Path` so tests inject a `tempfile::TempDir`
//! instead of resolving the real `app_data_dir` (the testability wedge).

use crate::error::TrunkError;
use crate::git::types::ReviewSession;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The outcome of attempting to load a session from disk.
pub enum LoadOutcome {
    /// A valid, same-or-older-schema session was read.
    Loaded(ReviewSession),
    /// No session file exists for this repo.
    None,
    /// The file was unparseable and has been quarantined to a `.corrupt`
    /// sidecar; the caller should start a fresh session and warn (D-15).
    RecoveredCorrupt,
    /// The file declares a newer `schema_version` than this build supports and
    /// has been left untouched; the caller must NOT auto-create a fresh
    /// session, so a downgrade cannot wipe newer data (D-16).
    RefusedNewer,
}

fn sessions_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("sessions")
}

/// Build-stable FNV-1a 64-bit hash of the canonical path → filename-safe token.
///
/// PRIVATE by design (encapsulation). NEVER `std::hash::DefaultHasher`: it is
/// not stable across Rust versions, so a toolchain bump would orphan sessions.
/// Hashing the canonical path is also the path-traversal mitigation (D-11).
fn session_filename(canonical: &Path) -> String {
    let s = canonical.to_string_lossy();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a 64-bit offset basis
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3); // FNV prime
    }
    format!("{:016x}.json", hash)
}

fn session_path(data_dir: &Path, canonical: &Path) -> PathBuf {
    sessions_dir(data_dir).join(session_filename(canonical))
}

/// Atomic write: tmp-in-same-dir + `sync_all` + `rename` (D-10, Pitfall 5).
/// `rename` is only atomic within a filesystem, so the tmp file lives next to
/// the target. `create_dir_all` covers the first-write case (Pitfall 2).
fn atomic_write_json(final_path: &Path, json: &str) -> Result<(), TrunkError> {
    let dir = final_path
        .parent()
        .ok_or_else(|| TrunkError::new("bad_path", "session path has no parent dir"))?;
    fs::create_dir_all(dir).map_err(|e| TrunkError::new("io", e.to_string()))?;

    let tmp_path = final_path.with_extension("json.tmp");
    {
        let mut f = File::create(&tmp_path).map_err(|e| TrunkError::new("io", e.to_string()))?;
        f.write_all(json.as_bytes())
            .map_err(|e| TrunkError::new("io", e.to_string()))?;
        f.sync_all()
            .map_err(|e| TrunkError::new("io", e.to_string()))?;
    }
    fs::rename(&tmp_path, final_path).map_err(|e| TrunkError::new("io", e.to_string()))?;
    Ok(())
}

/// Rename a file we cannot read to a `.corrupt` sidecar — never delete it (D-15).
fn quarantine_corrupt(final_path: &Path) -> Result<(), TrunkError> {
    let corrupt = final_path.with_extension("json.corrupt");
    fs::rename(final_path, corrupt).map_err(|e| TrunkError::new("io", e.to_string()))
}

/// Persist a session atomically for the given canonical repo path.
pub fn save_session(
    data_dir: &Path,
    canonical: &Path,
    session: &ReviewSession,
) -> Result<(), TrunkError> {
    let json = serde_json::to_string_pretty(session)
        .map_err(|e| TrunkError::new("serialize", e.to_string()))?;
    atomic_write_json(&session_path(data_dir, canonical), &json)
}

/// Load the session for a canonical repo path, applying the recovery state
/// machine: missing → `None`; newer schema → `RefusedNewer` (file untouched,
/// D-16); unparseable or wrong-shape → quarantine + `RecoveredCorrupt` (D-15).
pub fn load_session(data_dir: &Path, canonical: &Path) -> Result<LoadOutcome, TrunkError> {
    let final_path = session_path(data_dir, canonical);
    let raw = match fs::read_to_string(&final_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(LoadOutcome::None),
        Err(e) => return Err(TrunkError::new("io", e.to_string())),
    };

    let value = match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(v) => v,
        Err(_) => {
            quarantine_corrupt(&final_path)?;
            return Ok(LoadOutcome::RecoveredCorrupt);
        }
    };

    let version = value
        .get("schema_version")
        .and_then(|x| x.as_u64())
        .unwrap_or(0) as u32;
    if version > CURRENT_SCHEMA_VERSION {
        return Ok(LoadOutcome::RefusedNewer);
    }

    match serde_json::from_value::<ReviewSession>(value) {
        Ok(session) => Ok(LoadOutcome::Loaded(session)),
        Err(_) => {
            quarantine_corrupt(&final_path)?;
            Ok(LoadOutcome::RecoveredCorrupt)
        }
    }
}

/// Hard-delete the per-repo session file (SESS-03 / D-13). NotFound is treated
/// as idempotent success, so end-and-clear is safe to call repeatedly.
pub fn delete_session(data_dir: &Path, canonical: &Path) -> Result<(), TrunkError> {
    match fs::remove_file(session_path(data_dir, canonical)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(TrunkError::new("io", e.to_string())),
    }
}

/// Whether a session file is present for the canonical repo path (drives D-14
/// resume detection).
pub fn session_exists(data_dir: &Path, canonical: &Path) -> bool {
    session_path(data_dir, canonical).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_canonical_path_same_file() {
        let path = Path::new("/home/user/repos/trunk");
        let other = Path::new("/home/user/repos/other");

        assert_eq!(
            session_filename(path),
            session_filename(path),
            "the same canonical path must map to the same filename (build-stable hash)",
        );
        assert_ne!(
            session_filename(path),
            session_filename(other),
            "different canonical paths must map to different filenames",
        );
    }

    // ── v1→v2 migration / load-recovery state machine ────────────────────────

    /// Write raw bytes to the session path a given canonical repo maps to,
    /// creating the sessions dir first (mirrors atomic_write_json's setup).
    fn seed_session_file(data_dir: &Path, canonical: &Path, raw: &str) {
        let path = session_path(data_dir, canonical);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, raw).unwrap();
    }

    fn corrupt_sidecar_path(data_dir: &Path, canonical: &Path) -> PathBuf {
        session_path(data_dir, canonical).with_extension("json.corrupt")
    }

    #[test]
    fn v1_session_loads_and_backfills_ids_without_corrupting() {
        let dir = tempfile::tempdir().unwrap();
        let canonical = Path::new("/repos/trunk");
        let v1 = r#"{
            "schema_version": 1,
            "commits": ["abc"],
            "comments": [
                { "text": "one", "anchor": null, "cached_excerpt": null },
                { "text": "two", "anchor": null, "cached_excerpt": null }
            ],
            "draft_comment": null
        }"#;
        seed_session_file(dir.path(), canonical, v1);

        let outcome = load_session(dir.path(), canonical).unwrap();

        let session = match outcome {
            LoadOutcome::Loaded(s) => s,
            _ => panic!("v1 file must load, not corrupt/refuse"),
        };
        assert!(
            session.comments.iter().all(|c| !c.id.is_empty()),
            "every comment must have a non-empty backfilled id",
        );
        assert!(
            !corrupt_sidecar_path(dir.path(), canonical).exists(),
            "a valid v1 load must NOT create a .corrupt sidecar (D-15 false positive)",
        );
    }

    #[test]
    fn backfilled_ids_are_stable_across_reloads() {
        let dir = tempfile::tempdir().unwrap();
        let canonical = Path::new("/repos/trunk");
        let v1 = r#"{
            "schema_version": 1,
            "commits": [],
            "comments": [
                { "text": "one", "anchor": null, "cached_excerpt": null }
            ],
            "draft_comment": null
        }"#;
        seed_session_file(dir.path(), canonical, v1);

        let first = match load_session(dir.path(), canonical).unwrap() {
            LoadOutcome::Loaded(s) => s,
            _ => panic!("first load must succeed"),
        };
        let second = match load_session(dir.path(), canonical).unwrap() {
            LoadOutcome::Loaded(s) => s,
            _ => panic!("second load must succeed"),
        };

        assert_eq!(
            first.comments[0].id, second.comments[0].id,
            "backfilled ids must persist (re-saved as v2), not re-mint each reload",
        );
        assert_eq!(
            second.schema_version, 2,
            "the on-disk file must have been re-saved as schema_version 2",
        );
    }

    #[test]
    fn v2_session_loads_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let canonical = Path::new("/repos/trunk");
        let v2 = r#"{
            "schema_version": 2,
            "commits": [],
            "comments": [
                { "id": "keep-me", "text": "one", "anchor": null, "cached_excerpt": null, "commit_oid": null }
            ],
            "draft_comment": null
        }"#;
        seed_session_file(dir.path(), canonical, v2);

        let session = match load_session(dir.path(), canonical).unwrap() {
            LoadOutcome::Loaded(s) => s,
            _ => panic!("v2 file must load"),
        };
        assert_eq!(session.comments[0].id, "keep-me", "existing ids preserved");
        assert_eq!(session.schema_version, 2);
    }

    #[test]
    fn newer_schema_refused_and_file_left_byte_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let canonical = Path::new("/repos/trunk");
        let v3 = r#"{ "schema_version": 3, "commits": [], "comments": [], "draft_comment": null }"#;
        seed_session_file(dir.path(), canonical, v3);
        let before = fs::read(session_path(dir.path(), canonical)).unwrap();

        let outcome = load_session(dir.path(), canonical).unwrap();

        assert!(
            matches!(outcome, LoadOutcome::RefusedNewer),
            "a schema_version > CURRENT must return RefusedNewer (D-16)",
        );
        let after = fs::read(session_path(dir.path(), canonical)).unwrap();
        assert_eq!(before, after, "the newer file must be left byte-unchanged (D-16)");
        assert!(
            !corrupt_sidecar_path(dir.path(), canonical).exists(),
            "a refused file must NOT be quarantined",
        );
    }

    #[test]
    fn garbage_json_quarantines_to_corrupt_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let canonical = Path::new("/repos/trunk");
        seed_session_file(dir.path(), canonical, "{ this is not valid json ]");

        let outcome = load_session(dir.path(), canonical).unwrap();

        assert!(
            matches!(outcome, LoadOutcome::RecoveredCorrupt),
            "malformed JSON must return RecoveredCorrupt (D-15)",
        );
        assert!(
            corrupt_sidecar_path(dir.path(), canonical).exists(),
            "a .corrupt sidecar must be created (D-15)",
        );
    }
}
