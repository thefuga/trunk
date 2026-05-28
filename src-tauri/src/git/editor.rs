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
    // 1. Reserve a non-predictable temp path for the message file.
    //    `tempfile::Builder` returns a path under `std::env::temp_dir()` with
    //    enough entropy + O_EXCL semantics to defend the create-then-write
    //    race (T-75-T01). We take ownership of the path via `.keep()` so the
    //    auto-cleanup of `NamedTempFile` is replaced by our explicit `Drop`.
    //    The default Unix permissions are 0o600 (owner read/write only) —
    //    sufficient for T-75-T02 since only this process and its git child
    //    (same uid) need to read it. We do NOT chmod it more permissively.
    let msg_tempfile = tempfile::Builder::new()
        .prefix("trunk-editor-msg-")
        .tempfile()
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
    let (_msg_file, msg_path) = msg_tempfile
        .keep()
        .map_err(|e| TrunkError::new("io_error", e.to_string()))?;

    // 2. Write the payload verbatim. MSG-04: no stripping, no normalisation.
    if let Err(e) = std::fs::write(&msg_path, message) {
        let _ = std::fs::remove_file(&msg_path);
        return Err(TrunkError::new("io_error", e.to_string()));
    }

    // 3. Reserve a non-predictable temp path for the script. Same TOCTOU
    //    defence as the msg file. `.sh` suffix is cosmetic — git execs by
    //    path, not extension.
    let script_path = match tempfile::Builder::new()
        .prefix("trunk-editor-")
        .suffix(".sh")
        .tempfile()
    {
        Ok(tf) => match tf.keep() {
            Ok((_, p)) => p,
            Err(e) => {
                let _ = std::fs::remove_file(&msg_path);
                return Err(TrunkError::new("io_error", e.to_string()));
            }
        },
        Err(e) => {
            let _ = std::fs::remove_file(&msg_path);
            return Err(TrunkError::new("io_error", e.to_string()));
        }
    };

    // 4. Write the shell script. The cp arguments are ALWAYS quoted — even
    //    though `msg_path` comes from `tempfile::Builder` and has no shell
    //    metacharacters in practice, the quoting is the explicit defence
    //    against T-75-T04 command-injection via path content. Mirrors
    //    `interactive_rebase.rs:133`.
    let script_body = format!("#!/bin/sh\ncp \"{}\" \"$1\"\n", msg_path.display());
    if let Err(e) = std::fs::write(&script_path, &script_body) {
        let _ = std::fs::remove_file(&msg_path);
        let _ = std::fs::remove_file(&script_path);
        return Err(TrunkError::new("io_error", e.to_string()));
    }

    // 5. Make the script executable. Tauri is Unix-only for the editor path
    //    today (matches `interactive_rebase.rs:136-141` gating).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) =
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))
        {
            let _ = std::fs::remove_file(&msg_path);
            let _ = std::fs::remove_file(&script_path);
            return Err(TrunkError::new("io_error", e.to_string()));
        }
    }

    Ok(EditorHandle {
        script_path,
        msg_path,
    })
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
