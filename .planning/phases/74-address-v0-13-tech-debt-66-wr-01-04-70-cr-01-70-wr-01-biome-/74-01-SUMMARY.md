---
phase: 74-address-v0-13-tech-debt-66-wr-01-04-70-cr-01-70-wr-01-biome-
plan: 01
subsystem: review-session / CommitGraph listener machinery
tags: [tdd, listener-leak, error-branching, pattern-replication]
requires: [phase-73-canonicalPath-pattern]
provides:
  - "CommitGraph canonicalPath state + fail-closed listener filter"
  - "CommitGraph reloadSession branched catch (no_session/not_open silent, others toast)"
  - "lib/invoke isTrunkError export (promoted from ReviewPanel local)"
affects: [src/components/CommitGraph.svelte, src/components/ReviewPanel.svelte, src/lib/invoke.ts]
tech-stack:
  added: []
  patterns: [pattern-replication-of-ReviewPanel-73-01]
key-files:
  created:
    - .planning/phases/74-address-v0-13-tech-debt-66-wr-01-04-70-cr-01-70-wr-01-biome-/74-01-SUMMARY.md
  modified:
    - src/components/CommitGraph.svelte
    - src/components/CommitGraph.test.ts
    - src/components/ReviewPanel.svelte
    - src/components/ReviewPanel.test.ts
    - src/lib/invoke.ts
decisions:
  - "Fail-closed null check: !canonicalPath || payload !== canonicalPath. The plan's literal `if (canonicalPath && payload !== canonicalPath) return` would only filter when canonicalPath is known and re-introduce the WR-01 bug, because CommitGraph's reload-and-listener effects run in parallel on mount (ReviewPanel sequences them via await)."
  - "Promote isTrunkError to lib/invoke.ts rather than duplicate the type guard locally. ReviewPanel's local copy was deleted; the symbol now has one canonical home."
metrics:
  duration_min: 12
  completed: 2026-05-27
---

# Phase 74 Plan 01: Fix CommitGraph WR-01 null-permissive listener + WR-02 swallowed IPC failures

Pattern-replicated the Phase 73-01 ReviewPanel fix into `CommitGraph.svelte`: canonical_path is now tracked separately from sessionStatus and the session-changed listener fails closed when canonical is unknown; reloadSession branches its catch on `isTrunkError(e) && (code === "no_session" || code === "not_open")` to keep cold-repo paths silent while surfacing real backend failures via toast.

## What changed

- **`src/lib/invoke.ts`** — exported new `isTrunkError(e: unknown): e is TrunkError` type guard. Previously a local function inside `ReviewPanel.svelte`; promoted so any consumer of `safeInvoke` can branch on the error code without duplicating the guard.
- **`src/components/ReviewPanel.svelte`** — switched to the promoted import; deleted the local copy of `isTrunkError`.
- **`src/components/CommitGraph.svelte`** —
  - Added `let canonicalPath = $state<string | null>(null);` next to `sessionStatus`.
  - `reloadSession()` now sets `canonicalPath = status.canonical_path;` BEFORE the active-branch so cold (resume-available / none) sessions still track canonical for the listener.
  - Replaced `catch { sessionStatus = null; sessionOids = new Set(); }` with branched form: state always resets, then `if (isTrunkError(e) && (e.code === "no_session" || e.code === "not_open")) return;` else `showToast("Failed to load review session. Try again or reopen the repo.", "error");`.
  - Session-changed listener filter is now `if (!canonicalPath || event.payload !== canonicalPath) return;` (fail-closed). Added the `cancelled` flag + `if (cancelled) fn(); else unlisten = fn;` shape to prevent listen() promise leakage on tear-down.
  - Added `isTrunkError` to the import list from `../lib/invoke.js`.
- **`src/components/CommitGraph.test.ts`** — rewired to the ReviewPanel-style `installReads()` dispatcher (mock `safeInvoke` at the wrapper layer, not `invoke` at the Tauri layer); added `fireSessionChanged(payload)` that captures the session-changed handler by event-name filter (so the search-toggle listener doesn't shadow it). Five new tests under `describe("session-changed listener (66/WR-01)")` and `describe("reloadSession error branching (66/WR-02)")`.
- **`src/components/ReviewPanel.test.ts`** — switched the `vi.mock("../lib/invoke.js")` stub to `importActual + spread` so the newly-promoted `isTrunkError` export remains available to the panel's `errorMessage()` at runtime. The previous stub-shaped mock dropped the export and caused unhandled rejections in ReviewPanel tests once the symbol started being imported there.

## Test count delta

| Suite | Before | After | Delta |
|-------|-------:|------:|------:|
| CommitGraph.test.ts | 5 | 10 | +5 |
| Whole project (vitest) | 547 | 552 | +5 |

## Diff size

```
4 commits, ~95 net LOC added across 5 source files
```

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking import] `isTrunkError` was not exported from `lib/invoke.ts`**
- **Found during:** Task 1 prep
- **Issue:** The plan's `interfaces` section directed `import { isTrunkError } from "../lib/invoke.js"`, but the symbol lived as a local function in `ReviewPanel.svelte:156`.
- **Fix:** Promoted to `lib/invoke.ts` as an exported type guard. Deleted the local copy in `ReviewPanel.svelte` and added the new import there too. Eliminates duplication and makes the plan's intent achievable.
- **Files modified:** `src/lib/invoke.ts`, `src/components/ReviewPanel.svelte`.
- **Commit:** `47811e7`.

**2. [Rule 1 - Bug] `ReviewPanel.test.ts` mock dropped the new `isTrunkError` export**
- **Found during:** Task 3 (just check)
- **Issue:** `vi.mock("../lib/invoke.js", () => ({ safeInvoke: vi.fn() }))` shadowed the module entirely. Once `isTrunkError` was added there and consumed by `ReviewPanel.svelte`'s `errorMessage()`, the panel's End-review handler threw at runtime ("No 'isTrunkError' export is defined on the mock"). Manifested as 2 unhandled rejections in the ReviewPanel test suite.
- **Fix:** Replaced the stub with `vi.mock(..., async () => { const actual = await vi.importActual(...); return { ...actual, safeInvoke: vi.fn() }; })`. Same pattern adopted in `CommitGraph.test.ts` from the start.
- **Files modified:** `src/components/ReviewPanel.test.ts`.
- **Commit:** `760e701`.

### Acceptance-criterion mismatches (documented, not auto-fixed)

**3. Task 1 acceptance criterion #1: "at least 5 newly-failing tests"** — Only 2 of the 5 new tests are RED against the current code. The other 3 (own-repo filter, no_session silence, not_open silence) happen to be green today because the buggy code does the right thing for those specific inputs by accident: the null-permissive guard correctly drops cross-repo events when sessionStatus is non-null, and the bare `catch {}` correctly stays silent for no_session/not_open (it stays silent for ALL errors — which is the WR-02 bug, but it's incidentally right for the benign codes). The 3 green tests serve as regression markers pinning the post-fix behavior. The pivotal RED tests (cross-repo event during null window; toast on unexpected error code) both flipped to GREEN after Task 2.

**4. Task 2 acceptance criterion #2: literal `if (canonicalPath && event.payload !== canonicalPath) return`** — Implemented as `if (!canonicalPath || event.payload !== canonicalPath) return;` instead. The literal form ONLY filters when canonical is known, and would re-introduce the WR-01 null-permissive bug because CommitGraph's reload `$effect` runs in parallel with the listener `$effect` on mount (ReviewPanel.svelte avoids this by awaiting the status read inside the reload() that the listener calls, which sequences canonical assignment before the next event). The plan's `must_haves.truths[0]` ("fails closed when canonicalPath is null") and the truth-criterion override the literal grep-criterion. Documented in the GREEN commit message.

## Project gate

`just check` exits 0 after the final commit (`760e701`). 6/6 sub-checks green: fmt, biome, svelte-check, clippy, cargo-test, vitest.

## One-liner

This fix closes the second of the two ReviewPanel-pattern replications — the first being Phase 73-01's introduction of `canonicalPath` to ReviewPanel itself.

## TDD Gate Compliance

- **RED:** `70a5da1` — `test(74-01): add failing tests for canonicalPath listener filter and reloadSession error branching`.
- **GREEN:** `3d0182b` — `fix(74-01): pattern-replicate canonicalPath listener filter and branched reload catch`.
- **REFACTOR:** not needed (the catch branches' shared state-reset triple was 3 short lines; extracting a helper would have been speculative generality).

## Self-Check: PASSED

- `src/components/CommitGraph.svelte` — modified, commits `3d0182b` and `760e701` (no source changes in 760e701).
- `src/components/CommitGraph.test.ts` — modified, commits `70a5da1` (full rewrite to dispatcher) and `760e701` (biome fmt).
- `src/lib/invoke.ts` — modified, commit `47811e7`.
- `src/components/ReviewPanel.svelte` — modified, commit `47811e7`.
- `src/components/ReviewPanel.test.ts` — modified, commit `760e701`.
- `74-01-SUMMARY.md` — created (this file).
- All 4 commit hashes verified present in `git log`.
