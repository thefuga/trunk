//! Single-shot temp-editor helper for `GIT_EDITOR` plumbing.
//!
//! Builds a shell script + message file pair under `std::env::temp_dir()` so
//! Phase 76 callers (`merge --continue`, `merge <branch>`, `revert <oid>`) can
//! hand git a pre-filled commit message via `GIT_EDITOR=<script>`. The
//! returned [`EditorHandle`] owns both files and removes them on `Drop` —
//! including the early-return path where the handle leaves scope on `?`.
//!
//! This is the single-shot extract of the queue-based pattern in
//! `src-tauri/src/commands/interactive_rebase.rs:131-179` (per D-08 the queue
//! stays inline; only the single-shot is extracted here per D-07).
//!
//! # Usage (Phase 76)
//!
//! ```ignore
//! let handle = trunk_lib::git::editor::prepare("Merge branch 'foo'")?;
//! std::process::Command::new("git")
//!     .env("GIT_EDITOR", handle.script_path())
//!     .args(["merge", "--continue"])
//!     .status()?;
//! // handle drops here; both temp files are removed.
//! ```

use std::path::{Path, PathBuf};

use crate::error::TrunkError;

/// Owns the two temp files (`script` + `msg`) backing one `GIT_EDITOR` invocation.
///
/// `Drop` removes both files unconditionally so the early-return path
/// (`prepare()?` followed by an immediate `?` on the next call) leaves no
/// orphan temp files (MSG-05, D-07).
pub struct EditorHandle {
    script_path: PathBuf,
    msg_path: PathBuf,
}

impl EditorHandle {
    /// Path callers pass to `Command::env("GIT_EDITOR", _)`.
    pub fn script_path(&self) -> &Path {
        &self.script_path
    }

    /// Internal accessor for the message file. Tests assert on its contents.
    /// Not part of the Phase 76 public API.
    #[cfg(test)]
    pub(crate) fn msg_path(&self) -> &Path {
        &self.msg_path
    }
}

impl Drop for EditorHandle {
    fn drop(&mut self) {
        // Best-effort cleanup — Drop cannot return Result, and any leftover
        // file in temp_dir() is reaped by the OS eventually. Matches the
        // `let _ = std::fs::create_dir_all(...)` pattern at
        // src-tauri/src/commands/interactive_rebase.rs:144.
        let _ = std::fs::remove_file(&self.script_path);
        let _ = std::fs::remove_file(&self.msg_path);
    }
}

/// Build the script + message file pair, returning a handle that owns cleanup.
///
/// The message file contains `message` verbatim — no whitespace stripping, no
/// comment-line removal. Phase 76 callers build the default backend-side and
/// hand it to git via the shell script's `cp <msg> "$1"` invocation (MSG-04).
///
/// On any internal failure, partial state is cleaned up before returning
/// `Err` — the `Drop` impl only runs once `EditorHandle` is constructed, so
/// the pre-construction window is handled by hand inside this function.
pub fn prepare(message: &str) -> Result<EditorHandle, TrunkError> {
    // Reserve both temp paths up-front via `tempfile::Builder` — non-
    // predictable paths under `std::env::temp_dir()` with O_EXCL semantics
    // (T-75-T01 TOCTOU defence). `.keep()` hands ownership of the path back
    // to us so the auto-cleanup of `NamedTempFile` is replaced by our
    // explicit `Drop`. Default Unix permissions on both files are 0o600,
    // sufficient for T-75-T02 since only this process and its git child
    // (same uid) need to read them.
    let msg_path = reserve_temp_path("trunk-editor-msg-", "")?;
    let script_path = match reserve_temp_path("trunk-editor-", ".sh") {
        Ok(p) => p,
        Err(e) => {
            let _ = std::fs::remove_file(&msg_path);
            return Err(e);
        }
    };

    // From here on, both paths exist on disk; any failure must clean both.
    if let Err(e) = write_artifacts(&msg_path, &script_path, message) {
        let _ = std::fs::remove_file(&msg_path);
        let _ = std::fs::remove_file(&script_path);
        return Err(e);
    }

    Ok(EditorHandle {
        script_path,
        msg_path,
    })
}

/// Reserve a unique temp path with the given prefix + suffix and return it,
/// leaving the underlying file in place for the caller to write into.
fn reserve_temp_path(prefix: &str, suffix: &str) -> Result<PathBuf, TrunkError> {
    let tf = tempfile::Builder::new()
        .prefix(prefix)
        .suffix(suffix)
        .tempfile()
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    let (_file, path) = tf
        .keep()
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    Ok(path)
}

/// Write the message verbatim, then the shell script, then chmod 0o755 on
/// Unix. Caller owns cleanup on failure — this helper does NOT remove files.
fn write_artifacts(msg_path: &Path, script_path: &Path, message: &str) -> Result<(), TrunkError> {
    // MSG-04: payload flows through verbatim — no stripping, no normalisation.
    std::fs::write(msg_path, message).map_err(|e| TrunkError::new("io_error", e.to_string()))?;

    // T-75-T04: cp arguments are ALWAYS quoted. Mirrors interactive_rebase.rs:133.
    let script_body = format!("#!/bin/sh\ncp \"{}\" \"$1\"\n", msg_path.display());
    std::fs::write(script_path, &script_body)
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;

    // D-09: chmod 0o755 — gated on unix, matching interactive_rebase.rs:136-141.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(script_path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn script_file_is_created_with_executable_permissions() {
        let handle = prepare("hello").expect("prepare() must succeed on happy path");

        assert!(
            handle.script_path().exists(),
            "script file must exist after prepare()",
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(handle.script_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o755, "script file must be chmod 0o755 (D-09)");
        }
    }

    #[test]
    fn script_file_contains_shebang_and_cp_invocation() {
        let handle = prepare("ignored").expect("prepare() must succeed");
        let script = fs::read_to_string(handle.script_path()).unwrap();

        assert!(
            script.starts_with("#!/bin/sh\n"),
            "script must start with shebang, got: {script:?}",
        );

        let msg_path_str = handle.msg_path().display().to_string();
        let expected_cp = format!("cp \"{msg_path_str}\" \"$1\"");
        assert!(
            script.contains(&expected_cp),
            "script must contain quoted cp pattern {expected_cp:?} (T-75-T04 mitigation), got: {script:?}",
        );
    }

    #[test]
    fn message_file_contains_payload_verbatim_including_newlines_and_pound_signs() {
        let payload = "Merge branch 'foo'\n\n# preserved verbatim, no comment stripping\n";
        let handle = prepare(payload).expect("prepare() must succeed");

        let written = fs::read_to_string(handle.msg_path()).unwrap();
        assert_eq!(
            written, payload,
            "message file must contain payload verbatim (MSG-04: defaults flow through unchanged)",
        );
    }

    #[test]
    fn message_file_contains_empty_string_when_prepared_with_empty_payload() {
        let handle = prepare("").expect("prepare() must succeed even with empty payload");
        let written = fs::read_to_string(handle.msg_path()).unwrap();
        assert_eq!(written, "", "empty payload must yield empty msg file");
    }

    #[test]
    fn drop_removes_both_files_on_happy_path() {
        let handle = prepare("x").expect("prepare() must succeed");
        let script_path = handle.script_path().to_path_buf();
        let msg_path = handle.msg_path().to_path_buf();

        assert!(script_path.exists(), "script file must exist before drop");
        assert!(msg_path.exists(), "msg file must exist before drop");

        drop(handle);

        assert!(
            !script_path.exists(),
            "script file must be removed on Drop (MSG-05)",
        );
        assert!(
            !msg_path.exists(),
            "msg file must be removed on Drop (MSG-05)",
        );
    }

    #[test]
    fn drop_removes_both_files_when_handle_leaves_scope() {
        // Models Phase 76's `?` early-return path: the handle is constructed
        // and immediately goes out of scope when the next call returns Err.
        let (script_path, msg_path) = {
            let handle = prepare("x").expect("prepare() must succeed");
            (
                handle.script_path().to_path_buf(),
                handle.msg_path().to_path_buf(),
            )
        };

        assert!(
            !script_path.exists(),
            "script file must be removed when handle leaves scope",
        );
        assert!(
            !msg_path.exists(),
            "msg file must be removed when handle leaves scope",
        );
    }

    #[test]
    fn temp_paths_live_under_system_temp_dir() {
        let handle = prepare("x").expect("prepare() must succeed");
        let temp_dir = std::env::temp_dir();

        // Canonicalize to handle symlinked /tmp → /private/tmp on macOS.
        let temp_canonical = fs::canonicalize(&temp_dir).unwrap_or(temp_dir.clone());
        let script_canonical =
            fs::canonicalize(handle.script_path()).unwrap_or(handle.script_path().to_path_buf());
        let msg_canonical =
            fs::canonicalize(handle.msg_path()).unwrap_or(handle.msg_path().to_path_buf());

        assert!(
            script_canonical.starts_with(&temp_canonical),
            "script path must live under temp_dir() (T-75-T01 TOCTOU mitigation): \
             script={script_canonical:?} temp_dir={temp_canonical:?}",
        );
        assert!(
            msg_canonical.starts_with(&temp_canonical),
            "msg path must live under temp_dir() (T-75-T01 TOCTOU mitigation): \
             msg={msg_canonical:?} temp_dir={temp_canonical:?}",
        );
    }

    #[test]
    fn multiple_prepare_calls_produce_distinct_paths() {
        let a = prepare("a").expect("first prepare() must succeed");
        let b = prepare("b").expect("second prepare() must succeed");

        assert_ne!(
            a.script_path(),
            b.script_path(),
            "distinct calls must yield distinct script paths (no fixed names)",
        );
        assert_ne!(
            a.msg_path(),
            b.msg_path(),
            "distinct calls must yield distinct msg paths (no fixed names)",
        );
    }

    // Note on the partial-cleanup invariant (D-07):
    //
    // `prepare()` must clean up partially-created files if it fails mid-way
    // (e.g. msg write succeeds but script write fails). The `Drop` impl only
    // runs once `EditorHandle` has been constructed; the pre-construction
    // window is the helper's own responsibility.
    //
    // Injecting a deterministic failure (e.g. blocking chmod by mounting the
    // path as a read-only directory) requires more test scaffolding than the
    // cleanup branch itself, and OS behaviour varies across CI runners. The
    // invariant is documented in code via the per-step cleanup arms inside
    // `prepare()` and the `T-75-T05` row of the plan's threat model. Skipping
    // the test here per the executor decision called out in the PLAN
    // (Task 1 behavior, last bullet).
}
