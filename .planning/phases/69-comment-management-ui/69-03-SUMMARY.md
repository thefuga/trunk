---
phase: 69-comment-management-ui
plan: 03
subsystem: review-commands
tags: [tauri-command, review-session, comment-resolution, git2, orphan-detection, rust, tdd]

# Dependency graph
requires:
  - phase: 69-comment-management-ui
    plan: 01
    provides: "Comment v2 shape — id: String (D-03), commit_oid: Option<String> (D-01), anchor: Option<Anchor>, cached_excerpt: Option<String>"
  - phase: 69-comment-management-ui
    plan: 04
    provides: "TS OrphanReason union + CommentResolution interface — the wire contract this backend mirrors string-for-string"
provides:
  - "list_session_comments command (CMT-01): read-only clone of the session's comments incl. stable ids; no_session when absent; no save, no emit"
  - "resolve_session_comments command (CMT-04 / D-06): eager git2-backed resolvability check; Vec<CommentResolution> one-per-comment; runs resolve_all in spawn_blocking on a fresh repo handle, off-lock; no save, no emit"
  - "OrphanReason { CommitGone, FileGone, LineOutOfRange } + CommentResolution { id, resolvable, reason } DTOs (D-08) matching the TS wire shape"
  - "pure resolve_all(comments, repo) classifier — never drops an input, never panics, side-aware bound check"
affects: [69-05 panel rendering (orphan badges + comment list)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pure never-drop git2 classifier (resolve_all) modeled on intersect_graph_order: one CommentResolution per input, Err(reason) folded to Option, never errors-as-result"
    - "Side-aware tree selection: Side::New reads commit.tree(); Side::Old reads commit.parent(0)?.tree() with root-commit-on-Old classified FileGone"
    - "Read command = clone-under-lock + (optional) spawn_blocking fresh-repo git2 off-lock; never emit (mirrors list_session_commits)"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/review.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "resolve_all reads the ANCHOR's own commit_oid for line-anchored comments (the commit the line numbers index into), and the top-level comment.commit_oid only for commit-level comments (anchor None) — a `match (&anchor, &commit_oid)` makes the two paths explicit."
  - "The defensive (None, None) arm — neither anchor nor commit target — classifies CommitGone rather than panicking, keeping the never-panic guarantee honest even though v1 backfill should make it unreachable."
  - "1-based inclusive line bounds (start_line >= 1 && end_line <= line_count): a comment on the exact last line resolves; str::lines() does NOT count a trailing newline as a final empty line, matching Phase 67/68 capture (RESEARCH A2)."
  - "No wrapper-level integration test for resolve_session_comments: the existing review test harness tests pure cores / _inner functions only (no Tauri-runtime async-command harness, same as list_session_commits which has no wrapper test). The wrapper is thin glue over the unit-tested resolve_all; coverage is the 9 resolve_all tests + the compile/registration check (plan's explicit escape clause)."
  - "classify_anchor extracted as a Result-returning helper so resolve_all reads as a flat per-comment match (Ok -> resolvable, Err(reason) -> orphan); reason: Option<OrphanReason> serializes to JSON null when resolvable (no skip_serializing_if — the TS interface expects the field present-and-null)."

patterns-established:
  - "Pure git2 resolvability classifier with Err(reason)->Option folding, side-aware tree selection, and a defensive no-target arm"

requirements-completed: [CMT-01, CMT-04]

# Metrics
duration: ~25min
completed: 2026-05-26
---

# Phase 69 Plan 03: Comment Read Commands (list + git2 resolvability) Summary

**Added the two read commands the panel needs: `list_session_comments` (a read-only clone of the session's comments incl. stable ids, CMT-01) and `resolve_session_comments` (an eager git2-backed resolvability check, CMT-04 / D-06) returning one `CommentResolution { id, resolvable, reason }` per comment. The pure `resolve_all` classifier distinguishes `CommitGone` / `FileGone` / `LineOutOfRange` (D-08) side-aware — Side::New reads the commit's own tree, Side::Old reads the parent's — never dropping an input and never panicking. The wire DTOs match the TS contract Plan 04 already declared.**

## Performance

- **Duration:** ~25 min
- **Tasks:** 2 (Task 1 TDD RED -> GREEN; Task 2 thin-glue wrapper + registration)
- **Files modified:** 2

## Accomplishments

- `resolve_all(comments, repo)` returns exactly one `CommentResolution` per input (count in == count out), never dropping or panicking. For line-anchored comments it parses the anchor's own oid (`CommitGone` on unparseable/unknown), selects the tree by side (`Side::New` -> `commit.tree()`; `Side::Old` -> `commit.parent(0)?.tree()`, root-on-Old -> `FileGone`), reads the blob by path (`FileGone` if absent), and bound-checks the 1-based inclusive range (`LineOutOfRange` otherwise). Commit-level comments (anchor None) resolve iff `find_commit` succeeds.
- `list_session_comments`: read-only, clones `.comments` by canonical key, `no_session` if the in-memory map has no entry, no `save_session`, no emit.
- `resolve_session_comments`: clones `.comments` under the `ReviewSessionsState` lock, then opens the repo fresh inside `spawn_blocking` and runs `resolve_all` off-lock; read-only, no emit (mirrors `list_session_commits`).
- `OrphanReason` (PascalCase, no `rename_all`) and `CommentResolution` (snake_case fields, `reason: Option<OrphanReason>` serializing to JSON `null`) match the TS `OrphanReason` / `CommentResolution` wire shape from Plan 04 string-for-string.
- Both commands registered in `lib.rs` `invoke_handler` alongside the existing `commands::review::*` entries.

## Task Commits

Task 1 followed the TDD RED -> GREEN gate; Task 2 is thin glue over the tested core (no separate RED, per the plan's escape clause).

1. **Task 1 (RED): failing tests for resolve_all** - `ca18cb7` (test) — 9 tests against an empty-Vec stub, all failing on assertions
2. **Task 1 (GREEN): resolve_all classifier + list_session_comments** - `ab35921` (feat)
3. **Task 2: resolve_session_comments wrapper + register both reads** - `406b5fb` (feat)

No REFACTOR commits — both implementations were minimal and clean (rustfmt applied before each commit, so no separate `style` fallout this time).

## Files Created/Modified

- `src-tauri/src/commands/review.rs` - Added `OrphanReason` enum, `CommentResolution` struct, `classify_anchor` helper, pure `resolve_all`; `list_session_comments` + `resolve_session_comments` read commands; a `make_file_repo` test helper (commits a 3-line `foo.rs` blob via real git2) and 9 `resolve_all` unit tests covering every orphan condition + side semantics + the last-line boundary + the unparseable-oid never-panic path.
- `src-tauri/src/lib.rs` - Registered `commands::review::list_session_comments` and `commands::review::resolve_session_comments` in `invoke_handler`.

## Decisions Made

- **Anchor oid vs top-level oid:** for a line-anchored comment the relevant commit is `anchor.commit_oid` (where the line numbers live), not the top-level `comment.commit_oid` (which is `Some` only for commit-level notes). A `match (&c.anchor, &c.commit_oid)` makes the two paths and the defensive no-target arm explicit.
- **Defensive (None, None) arm -> CommitGone:** a comment with neither anchor nor commit target has no resolvable target; classifying it `CommitGone` (rather than `unwrap`/panic) keeps the never-panic guarantee honest even though v1 backfill should make this unreachable.
- **1-based inclusive last-line convention:** `start_line >= 1 && end_line <= line_count`. `str::lines()` does not count a trailing newline as a final empty line (a 3-line file with trailing `\n` -> 3), so a comment on the exact last line resolves and `end_line == line_count + 1` is `LineOutOfRange` — confirmed against the Phase 67/68 capture convention (RESEARCH A2). A dedicated test pins both halves.
- **No wrapper-level integration test:** the existing review test harness exercises pure cores / `_inner` functions only (there is no Tauri-runtime async-command harness; `list_session_commits` itself has no wrapper test). `resolve_session_comments` is thin glue over the unit-tested `resolve_all`, so coverage is the 9 `resolve_all` tests plus the compile + registration check — exactly the plan's stated escape clause.
- **`reason` serializes present-and-null:** `Option<OrphanReason>` with default serde gives JSON `null` (field present) when resolvable, matching the TS `reason: OrphanReason | null` field. No `skip_serializing_if`, which would drop the field and break the interface.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] TreeBuilder rejects nested paths in the test helper**
- **Found during:** Task 1 RED (first test run)
- **Issue:** `make_file_repo`'s `commit_with_file` inserted `"src/foo.rs"` into a `git2::TreeBuilder`, which only handles single-level entries and errors with "invalid name for a tree entry" — the RED failures were panics in the helper, not clean assertion failures.
- **Fix:** Changed the seeded path to a top-level `foo.rs` (sufficient for `Tree::get_path` + the bound check) and updated the test bodies accordingly. No production code involved — test scaffolding only.
- **Files modified:** src-tauri/src/commands/review.rs (test code only)
- **Committed in:** `ca18cb7` (the corrected RED commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug, test scaffolding only). No scope creep, no production-logic deviation.
**Impact on plan:** None — the production `resolve_all` matches the RESEARCH code example exactly; the fix was a one-level path constraint of `git2::TreeBuilder` discovered while building the RED test repo.

## Issues Encountered

- `git2::TreeBuilder::insert` rejects a `/`-containing entry name (it builds one tree level, not a nested path). The test helper now seeds a top-level file, which is all `Tree::get_path` and the line-count bound check need. Building a nested tree would have added no coverage.
- Per-task `cargo test review` does not run rustfmt/clippy. Following the Plan 01/02 lesson, ran `cargo fmt` + `cargo clippy --lib` before each commit and the full `just check` before finalizing, so no `style` fallout commit this time.

## Threat Model Compliance

All `mitigate` dispositions from the plan's STRIDE register are satisfied:

- **T-69-09 (Tampering/EoP — path traversal via `file_path`):** `file_path` is only ever passed to `Tree::get_path` (a git object-DB tree lookup), never to `std::fs`; it cannot escape the repo object DB. Session keying remains the canonical-path hash (unchanged).
- **T-69-10 (DoS — malformed `commit_oid` crashing the resolver):** parsed via `git2::Oid::from_str`; on failure classified `CommitGone` and the iteration continues — never panics, never bubbles. Pinned by `resolve_all_classifies_unparseable_oid_as_commit_gone_without_panicking`.
- **T-69-11 (DoS — large blob inflating line-count work):** accepted by design (local single-user repo; reading blob content is the cost git already pays, no amplification).
- **T-69-12 (Information Disclosure — resolver leaking content):** `resolve_all` returns only `{ id, resolvable, reason }` — no file content or line text crosses the IPC boundary; excerpts come from the independently-stored `cached_excerpt`, not this command.

No new threat surface beyond the plan's register — two read-only IPC commands on the already-established frontend->IPC trust boundary; the only git2 work is object-DB reads inside `spawn_blocking` on a fresh handle.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The backend read half of the comment panel is complete and `just check`-clean. Plan 05 can:
  - call `list_session_comments` to render the full review (line-anchored vs commit-level by branching on `comment.anchor === null`);
  - call `resolve_session_comments` at panel load and badge orphans by `CommentResolution.reason` (`CommitGone` / `FileGone` / `LineOutOfRange`), keeping the comment text + `cached_excerpt` visible (D-08);
  - disable jump for non-resolvable / commit-level comments.
- No blockers.

---
*Phase: 69-comment-management-ui*
*Completed: 2026-05-26*

## Self-Check: PASSED

- All modified files present on disk (review.rs, lib.rs, 69-03-SUMMARY.md).
- All three task commits found in git history (ca18cb7 RED, ab35921 GREEN Task 1, 406b5fb Task 2).
- Symbols verified: `fn resolve_all`, `enum OrphanReason` (CommitGone/FileGone/LineOutOfRange), `struct CommentResolution`, `pub async fn list_session_comments`, `pub async fn resolve_session_comments` in review.rs; `commands::review::list_session_comments` + `commands::review::resolve_session_comments` registered in lib.rs.
- `just check` exits 0 (fmt, biome, svelte-check, clippy, cargo-test — 50 review lib tests incl. 9 new resolve_all tests, vitest 507 passing).
- TDD gate: `test(69-03)` commit (ca18cb7) precedes the `feat(69-03)` GREEN commit (ab35921).
