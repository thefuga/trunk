---
phase: 74-address-v0-13-tech-debt-66-wr-01-04-70-cr-01-70-wr-01-biome
plan: 05
subsystem: src-tauri/src/commands/review.rs
tags: [refactor, observability, 66/WR-04]
requires: [74-04]
provides: [emit_session_changed-helper]
affects: [review-session-lifecycle]
tech-stack:
  added: []
  patterns: [logged-tauri-emit-failure]
key-files:
  created: []
  modified:
    - src-tauri/src/commands/review.rs
decisions:
  - "Use eprintln! (not log::warn!) — codebase has no logger crate; RESEARCH A4 falsified at plan time"
  - "Helper takes &Path so callers pass &canonical zero-cost from PathBuf"
metrics:
  duration: ~6m
  completed: 2026-05-27
  tasks: 2/2
---

# Phase 74 Plan 05: emit_session_changed Helper Summary

One-liner: Replaced 10 silent `let _ = app.emit("session-changed", ...)` sites in `commands/review.rs` with a single `emit_session_changed(&app, &canonical)` helper that logs emit failures via `eprintln!` (66/WR-04 closed).

## Grep Gate (before → after)

Before (Plan 74-04 had shifted line numbers since planning; re-enumerated):
```
725, 745, 765, 799, 853, 879, 903, 1129, 1184, 1206   (10 sites)
```

After:
```
$ grep -n 'let _ = app.emit' src-tauri/src/commands/review.rs
(empty — 0 matches)

$ grep -c 'emit_session_changed(&app' src-tauri/src/commands/review.rs
10
```

Gate: PASS.

## Tasks

| # | Task | Commit | Files |
| - | ---- | ------ | ----- |
| 1 | Add emit_session_changed helper | bf1f964 | src-tauri/src/commands/review.rs |
| 2 | Replace all 10 call sites | a224797 | src-tauri/src/commands/review.rs |

## Helper Definition

Placed after `resolve_data_dir` near the top of the file, before any `_inner`/command function:

```rust
fn emit_session_changed(app: &AppHandle, canonical: &Path) {
    if let Err(e) = app.emit("session-changed", canonical.to_string_lossy().into_owned()) {
        eprintln!(
            "session-changed emit failed for {}: {}",
            canonical.display(),
            e
        );
    }
}
```

## Project Gate

`just check` exit 0 (fmt + biome + svelte-check + clippy + cargo test + vitest, 552 frontend tests + all backend tests pass).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Adjusted helper doc-comment to avoid literal `let _ = app.emit(...)` substring**
- **Found during:** Task 2 grep gate
- **Issue:** After replacing all real call sites, the file still had one match for `let _ = app.emit` — inside the helper's own doc comment ("the previous `let _ = app.emit(...)` pattern violated it"). The plan's truths require zero matches in the file unconditionally.
- **Fix:** Edited the doc comment from "the previous \`let _ = app.emit(...)\` pattern violated it" to "the previous silent-discard pattern violated it" — preserves the explanatory intent, removes the literal substring, gate passes.
- **Files modified:** src-tauri/src/commands/review.rs (same commit a224797)

No other deviations. All 10 sites had the exact verified shape — straight `replace_all` worked.

## Known Stubs

None.

## Threat Flags

None — refactor only, no new attack surface. The eprintln payload includes a local filesystem path on stderr in a desktop app (T-74-05-01 in plan threat register, disposition `accept`).

## Self-Check: PASSED

- `src-tauri/src/commands/review.rs` exists and contains `fn emit_session_changed`: FOUND
- Commit bf1f964: FOUND
- Commit a224797: FOUND
- `grep -c 'let _ = app.emit' src-tauri/src/commands/review.rs` = 0: PASS
- `grep -c 'emit_session_changed(&app' src-tauri/src/commands/review.rs` = 10: PASS
- `just check` exit 0: PASS
