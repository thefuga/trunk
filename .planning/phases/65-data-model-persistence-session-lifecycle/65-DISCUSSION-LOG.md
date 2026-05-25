# Phase 65: Data Model + Persistence + Session Lifecycle - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-25
**Phase:** 65-data-model-persistence-session-lifecycle
**Areas discussed:** Lifecycle surface (scope), End-and-clear + resume, Corrupt-file recovery

---

## Gray-Area Selection

The roadmap pre-locked most of this phase (anchor schema, Rust-owned persistence, atomic writes, `schema_version: 1`). Advisor review trimmed a candidate "record metadata" area (timestamps/IDs have no user-visible consequence — render uses `path:Lstart-Lend (sha)` headings, no timestamps) and confirmed the two roadmap-flagged "DECIDE in planning" items (multi-tab coordination, draft-comment location) belong to the planner, not the user. Three areas presented; user selected all three.

---

## Lifecycle surface (scope)

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal temp trigger + stub | View-menu trigger + bare review-panel stub (active/empty/resume states); replaced by real panel in Phase 69. Makes criteria directly testable. | ✓ |
| Backend-only + tests | Commands + Rust state + integration tests only; real UI wired in Phase 69. Criterion #2 reinterpreted as a backend assertion. | |
| Menu trigger, no panel | Lifecycle trigger + small status badge, no panel scaffolding. | |

**User's choice:** Minimal temp trigger + stub
**Notes:** Drives what "verified" means this phase — SESS-01/02/03 hand-verifiable end-to-end. Stub is throwaway, replaced in Phase 69.

---

## End-and-clear + resume

### Sub-decision A — what "end and clear" does

| Option | Description | Selected |
|--------|-------------|----------|
| Hard-delete the file | Remove the per-repo JSON entirely; criterion #4 trivially satisfied, no stale state. | ✓ |
| Soft-archive | Keep file marked ended / move to archive; no requirement consumes it, adds resume/cleanup edge cases. | |

**User's choice:** Hard-delete the file

### Sub-decision B — resume on open

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-load on open | Session loads into state automatically on repo open; no prompt. | |
| Prompt to resume | Detect file on open, surface indicator; user clicks Resume to load. | ✓ |

**User's choice:** Prompt to resume
**Notes:** Opening a repo does not silently enter review mode. The lifecycle stub must surface a "resume available" state to back this.

---

## Corrupt-file recovery

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve & warn (split) | Corrupt JSON → `.corrupt` sidecar + fresh + toast; newer `schema_version` → refuse, leave untouched, "newer version" message. | ✓ |
| Fresh + warn | Any unreadable/incompatible file → fresh session + warn; old file overwritten on next save (data loss). | |
| Refuse for everything | Any read failure → error, load no session; user must manually fix/delete. | |

**User's choice:** Preserve & warn (split)
**Notes:** Two distinct failure cases handled differently — never destroy an unreadable file; never let a downgrade silently wipe data written by a newer build.

---

## Claude's Discretion

- Exact Rust struct/enum names and serde field-casing (follow existing codebase conventions).
- JSON filename scheme and on-disk layout under `app_data_dir` (must use atomic tmp+rename).
- New `src-tauri/src/commands/review.rs` for lifecycle commands (recommended).

## Deferred Ideas

- Soft-archive / past-session history — considered and rejected for SESS-03.
- Multiple concurrent sessions per repo — already tracked as Future Requirement SESS-F1.
- Multi-tab live coordination (`session-changed` event vs tab-reload) — deferred to planner (DP-01).
- Draft-comment storage location (`draft_comment` field vs component-level) — deferred to planner (DP-02).
