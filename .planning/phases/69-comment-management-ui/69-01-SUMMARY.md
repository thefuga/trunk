---
phase: 69-comment-management-ui
plan: 01
subsystem: database
tags: [serde, uuid, schema-migration, review-session, rust, git2]

# Dependency graph
requires:
  - phase: 65-data-model-persistence-session-lifecycle
    provides: "frozen review-session schema (Comment/Anchor/ReviewSession), the load recovery state machine (D-15 corrupt-quarantine, D-16 newer-refusal), and the atomic tmp+sync_all+rename writer"
provides:
  - "Comment v2 shape: stable `id: String` (D-03) + sibling `commit_oid: Option<String>` (D-01)"
  - "CURRENT_SCHEMA_VERSION = 2; fresh sessions mint v2 (D-04)"
  - "lazy v1->v2 load-path migration: id backfill + re-save, no .corrupt false-positive"
  - "uuid v4 dependency for stable id generation"
affects: [69-02 commit-level comments, 69-03 edit/delete by id, 69-04 panel rendering]

# Tech tracking
tech-stack:
  added: [uuid v1 (v4 feature)]
  patterns:
    - "Migration shape A: #[serde(default)] empty-string sentinel on a new required field, backfilled + re-saved at load time"
    - "Version gate stays BEFORE from_value and migration so a newer file is refused untouched (D-16)"

key-files:
  created: []
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/git/types.rs
    - src-tauri/src/git/review_store.rs
    - src-tauri/src/commands/review.rs

key-decisions:
  - "Comment.id uses #[serde(default)] so v1 files deserialize to '' (the migration sentinel) instead of quarantining; commit_oid: Option<String> needs no default (serde maps missing -> None)"
  - "load_session backfills uuid ids on empty-id comments, sets schema_version=2, and re-saves only when migrating (version<2) or any id was backfilled — ids persist, so they are stable across reloads"
  - "Line-anchored comments are minted with a real uuid id at WRITE time (add_comment_inner), not an empty string — edit/delete target-by-id never misses within the same session before a reload"
  - "Fresh-session production paths (start + corrupt-recovery) write schema_version=2 directly (D-04), so a fresh session never round-trips through the migration"

patterns-established:
  - "Empty-string serde sentinel + load-time backfill + conditional atomic re-save for additive schema migrations"
  - "Newer-version refusal gate precedes deserialization and migration; byte-unchanged is asserted via read-before/read-after equality"

requirements-completed: [CMT-02, CMT-03]

# Metrics
duration: ~18min
completed: 2026-05-26
---

# Phase 69 Plan 01: Review Schema v2 (id + commit_oid) Summary

**Extended the frozen Phase 65 review schema to v2 — added a stable `id: String` and a sibling `commit_oid: Option<String>` to Comment, bumped CURRENT_SCHEMA_VERSION 1→2, and added a lazy v1→v2 load-path migration that backfills uuid ids without tripping the corrupt quarantine (D-15) or the newer-version refusal (D-16).**

## Performance

- **Duration:** ~18 min
- **Started:** 2026-05-26T02:30Z (approx)
- **Completed:** 2026-05-26T02:48Z (approx)
- **Tasks:** 2 (both TDD)
- **Files modified:** 7 (incl. Cargo.lock + two pre-existing test files)

## Accomplishments

- Comment carries `id: String` (#[serde(default)] v1 sentinel) and `commit_oid: Option<String>`; round-trips both unchanged.
- `uuid` v1 (v4 feature) added as the approved stable id generator.
- `CURRENT_SCHEMA_VERSION` is 2; v1 sessions migrate-on-load with non-empty backfilled ids that are stable across reloads (re-saved as v2 via the existing atomic writer).
- D-16 (refuse newer untouched) and D-15 (quarantine corrupt) both preserved and verified by regression tests, including a byte-unchanged assertion on the refused v3 file.
- Line-anchored comments are minted with a real uuid id at write time so edit/delete-by-id never silently miss before a reload.

## Task Commits

Each task followed the TDD RED → GREEN gate:

1. **Task 1 (RED): v1 round-trip test for Comment v2 fields** - `3f85fd9` (test)
2. **Task 1 (GREEN): extend Comment schema to v2 with id and commit_oid** - `6b4b06a` (feat)
3. **Task 2 (RED): v1→v2 migration regression tests** - `a4c5ced` (test)
4. **Task 2 (GREEN): bump schema to v2 with load-path id backfill migration** - `58148c2` (feat)

No REFACTOR commits — both implementations were minimal and clean as written.

## Files Created/Modified

- `src-tauri/Cargo.toml` - Added `uuid = { version = "1", features = ["v4"] }`.
- `src-tauri/Cargo.lock` - uuid v1.21.0 resolved.
- `src-tauri/src/git/types.rs` - Added `#[serde(default)] pub id: String` and `pub commit_oid: Option<String>` to `Comment`; added a serde round-trip test mod proving v1 JSON deserializes with id=="" and commit_oid==None.
- `src-tauri/src/git/review_store.rs` - `CURRENT_SCHEMA_VERSION` 1→2; load_session backfills uuid ids + conditionally re-saves; five new regression tests (v1 backfill, id stability, v2 load, v3 RefusedNewer byte-unchanged, garbage RecoveredCorrupt).
- `src-tauri/src/commands/review.rs` - BUILD-FIX: `add_comment_inner` struct literal sets `id: uuid::Uuid::new_v4().to_string()` and `commit_oid: None`; fresh-session production paths (start + corrupt-recovery) write schema_version=2.
- `src-tauri/tests/test_review.rs` - Rule 3 build-fix: fresh-session helper → v2, newer-refusal probe JSON → v3, start assertion → v2.
- `src-tauri/tests/test_integ_serde.rs` - Rule 3 build-fix: `anchored_session` Comment literal gets id + commit_oid; helper schema_version → 2; serde-shape assertion → 2.

## Decisions Made

- `Comment.id` uses `#[serde(default)]` (empty-string sentinel) so a v1 file lacking the field deserializes (not quarantines); `commit_oid: Option<String>` needs no default since serde maps a missing field to None automatically.
- Re-save predicate is `any_id_backfilled || on_disk_version < 2`; the in-memory `schema_version` is set to 2 unconditionally before returning Loaded. This keeps ids stable across reloads and avoids needless churn on already-v2 files with all ids present.
- Line-anchored comments mint a real uuid id at write time (not empty), consistent with the v4 pin, so id-targeted edit/delete works within the same session before the next load_session backfill re-saves.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] v1-era Phase 65 tests + production fresh-session paths broke on the v2 bump**
- **Found during:** Task 2 (GREEN, surfaced by `just check`)
- **Issue:** Bumping `CURRENT_SCHEMA_VERSION` to 2 broke pre-existing Phase 65 expectations that hardcoded v1 as "current" and v2 as "newer": `test_review.rs::newer_version_refused` wrote `schema_version:2` expecting RefusedNewer (now loads); `test_review.rs::session_round_trips` + `start_creates_session` asserted v1; `test_integ_serde.rs::session_serde_shape` asserted v1. The plan's must-have truth #2 also requires fresh sessions to be v2 (D-04), but `start_review_session_inner` and the corrupt-recovery path still wrote v1.
- **Fix:** Production fresh-session paths (`review.rs:87`, `review.rs:681`) now write `schema_version: 2`. Stale v1-era test expectations updated to the v2 reality: fresh-session helpers → v2, newer-refusal probe → v3 (genuinely newer), serde-shape/start asserts → v2. Also added id + commit_oid to the `test_integ_serde::anchored_session` Comment literal (required to compile after the struct change).
- **Files modified:** src-tauri/src/commands/review.rs, src-tauri/tests/test_review.rs, src-tauri/tests/test_integ_serde.rs
- **Verification:** `just check` exits 0 (fmt, biome, svelte-check, clippy, cargo-test, vitest 507 passing).
- **Committed in:** `58148c2` (Task 2 GREEN commit)

**2. [Rule 3 - Blocking] add_comment_inner struct literal — the planned BUILD-FIX**
- **Found during:** Task 1 (GREEN)
- **Issue:** Adding a non-optional `id: String` to Comment breaks the only `Comment {…}` literal at review.rs:397 at compile time. (Anticipated by the plan as a required BUILD-FIX.)
- **Fix:** Set `id: uuid::Uuid::new_v4().to_string()` and `commit_oid: None` on that literal only; scoped to struct-literal fields — no change to add_comment / AddCommentRequest / signature / line-anchored tests.
- **Files modified:** src-tauri/src/commands/review.rs
- **Verification:** Full lib test suite (48 tests) passes.
- **Committed in:** `6b4b06a` (Task 1 GREEN commit)

---

**Total deviations:** 2 auto-fixed (both Rule 3 - Blocking). Deviation 2 was an explicitly-planned BUILD-FIX. Deviation 1 was the necessary fallout of the v2 bump on v1-era tests and the D-04 fresh-session requirement.
**Impact on plan:** All fixes required for the code to compile and for the suite to reflect v2 reality. No scope creep — only `schema_version` literals and the one mandated struct literal were touched.

## Issues Encountered

- The initial `rg 'Comment\s*\{'` scan only searched `src-tauri/src/`, missing a `Comment {}` literal in `src-tauri/tests/test_integ_serde.rs`. Caught by `just check` and fixed as part of Deviation 1. Lesson: scan `tests/` as well when changing a public struct's required fields.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The v2 keystone schema is in place; Plan 02 (commit-level comments) can write `commit_oid`, and Plan 03 (edit/delete by id) can rely on every comment carrying a stable non-empty id.
- No blockers.

---
*Phase: 69-comment-management-ui*
*Completed: 2026-05-26*

## Self-Check: PASSED

- All modified files present on disk (types.rs, review_store.rs, review.rs, Cargo.toml, SUMMARY.md).
- All four task commits found in git history (3f85fd9, 6b4b06a, a4c5ced, 58148c2).
- Key literals verified: `pub id: String` in types.rs, `const CURRENT_SCHEMA_VERSION: u32 = 2` in review_store.rs.
- `just check` exits 0 (fmt, biome, svelte-check, clippy, cargo-test, vitest 507 passing).
