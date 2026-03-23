# Phase 44: Backend State Scoping - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 44-backend-state-scoping
**Areas discussed:** Same-repo op policy, Close-repo cleanup, Progress event scoping

---

## Same-repo op policy

| Option | Description | Selected |
|--------|-------------|----------|
| One op per repo | Keep mutual exclusion per repo — second op on same repo gets 'op_in_progress' error. Same behavior as today but scoped per-repo instead of global. | ✓ |
| Allow concurrent per repo | Let fetch + push run simultaneously on the same repo. More flexible but git itself may conflict on lock files. | |

**User's choice:** One op per repo (recommended)
**Notes:** User pointed out this is moot in practice since TAB-10 (Phase 48) enforces one tab per repo — "why are you even asking me this? Won't we have just one tab per repo?"

---

## Close-repo cleanup

| Option | Description | Selected |
|--------|-------------|----------|
| Cancel on close | Kill the running git subprocess when the tab closes. Clean and predictable. | |
| Let it finish | Remove watcher/cache but let the remote op complete in background. | ✓ |

**User's choice:** Let it finish
**Notes:** User added important nuance: normal close (X button) lets ops finish, but Shift+click on tab X should force-close and cancel any in-flight operations. This creates a two-tier close behavior that the backend must support.

---

## Progress event scoping

| Option | Description | Selected |
|--------|-------------|----------|
| Add path now | Include repo path in remote-progress event payload now. Prepares contract for Phase 45 tabs. | ✓ |
| Defer to Phase 45 | Leave events as-is. Phase 45 modifies events when building tab architecture. | |

**User's choice:** Add path now (recommended)
**Notes:** None

---

## Claude's Discretion

- Internal structure of per-repo RunningOp
- Whether cancel uses path param or separate force_close_repo command
- Error message wording

## Deferred Ideas

None
