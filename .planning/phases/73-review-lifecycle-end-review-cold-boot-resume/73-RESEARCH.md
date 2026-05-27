# Phase 73: Review Lifecycle (End-review + cold-boot resume) - Research

**Researched:** 2026-05-27
**Domain:** Svelte 5 runes UI wiring + Tauri 2 IPC composition (no backend schema changes)
**Confidence:** HIGH (all backend primitives + frontend integration points verified by file read)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01: Resume trigger** — `ReviewPanel.reload()` acts on `status.state`. When `state === 'ResumeAvailable'`, call `resume_review_session` before the parallel `list_session_*` reads. No new IPC, no backend schema change.
- **D-02: Toolbar active-state sync (lag)** — Default: ignore the lag. `reviewActive` reflects runtime, not disk; the Toolbar Review button is a toggle. If plan-phase considers an eager `resume-on-repo-open` path, the plan **MUST benchmark the cost first** (per `[[measure-before-assuming]]`).
- **D-03: End placement** — ReviewPanel header, next to the existing Copy button. Use a visually distinct icon (proposed: `Trash2`). No new toolbar entry, no new menu item.
- **D-04: Session summary line** — Small caption above the comments list: "N comments · M commits" (muted text). Uses `comments.length` and `commits.length` from existing rune state — no new IPC.
- **D-05: Confirmation pattern** — Inline two-step button. First click swaps label to "Click again to confirm" with a clear danger color; auto-reverts after ~3s. No modal. Extends the `setTimeout`/`clearTimeout` pattern already used for the Copy ✓ Copied affordance (`ReviewPanel.svelte:130-133`, same `clearTimeout` before `setTimeout` discipline from Phase 71).
- **D-06: Distinct empty-state copy, no inline CTA button**
  - Cold (`status.state === 'None'` and no runtime session): "No active review. Toggle review mode in the toolbar to start."
  - Warm-empty (session active, zero comments): "Review started. Select diff lines or add a commit note to comment."
  - No inline [Start review] button. Word-level wording is plan-phase polish; structural decision (two distinct copies) is locked.
- **D-07: LoadOutcome handling on cold-boot resume** — Reuse the existing branching in `resume_review_session` (Loaded/RecoveredCorrupt → toast; RefusedNewer → warn). Cold-boot resume MUST NOT crash the panel on a non-`Loaded` outcome.
- **D-08: Post-End behavior** — On confirmed End: call `end_review_session` (already hard-deletes the file + clears runtime state via `_inner` at `review.rs:110-118`). Panel re-renders into the cold empty state.
- **D-09: Multi-tab live coordination** — Reuse existing `session-changed` event. Tab A's End emits → tab B's `ReviewPanel.reload()` sees `status.state === 'None'` and renders the cold empty state. No new event type.

### Claude's Discretion
- Exact Lucide icons (Trash2 for End / MessageSquareOff for cold-empty / etc.) — pick during implementation by visual fit; all proposed icons should be verified present in `@lucide/svelte` (Phase 72 did this same check for `MessagesSquare`).
- Exact copy wording for the empty states, the two-step button label ("Click again to confirm" vs. "Confirm end" etc.), and the danger color treatment.
- Auto-revert duration for the two-step End button (~3s suggested) — tune for testability and feel.
- Whether to surface a "Review was ended in another window" toast for tab B specifically, or let the empty state speak for itself.

### Deferred Ideas (OUT OF SCOPE)
- Keyboard shortcut for End-review (REQ-72-1b retracted; do not re-bind `Cmd+Shift+R`).
- Review history / archival of past sessions (Phase 74 candidate).
- Combined "Copy & End" single-button affordance (intentionally separated — destructive vs. non-destructive).
- Eager-resume on repo-open (deferred unless benchmark proves cost; default ships "Toolbar reflects runtime").
- Explicit "Review was ended in another window" toast (Claude's discretion if usability warrants).
</user_constraints>

<phase_requirements>
## Phase Requirements

The phase brief carries forward from `.planning/todos/pending/phase-73-review-lifecycle.md` and `72-VERIFICATION.md` Bug 3. Two named outcomes drive every task:

| ID | Description | Research Support |
|----|-------------|------------------|
| REQ-73-RESUME | Cold boot → open ReviewPanel → on-disk comments appear without any mutation (closes Bug 3 from 72-VERIFICATION.md) | §"Cold-boot resume wiring" + §"Validation Architecture" lifecycle cases |
| REQ-73-END | ReviewPanel has a visible End-review affordance; inline two-step confirm; on confirm calls `end_review_session`; panel re-renders into cold empty state (D-03/D-05/D-08) | §"Two-step End button state model" + §"End-review IPC composition" |
| REQ-73-EMPTY | Empty state distinguishes "no session" (cold) from "session active, no comments" (warm-empty) (D-06) | §"Empty-state gating" |
| REQ-73-SUMMARY | "N comments · M commits" caption above the comments list, sourced from existing rune state (D-04) | §"Session summary line" |
| REQ-73-MULTITAB | Tab A's End emits `session-changed`; tab B sees cold empty state on its next reload (D-09) | §"Multi-tab coordination" + §"Validation Architecture" multi-tab cases |
| REQ-73-NYQUIST | All five lifecycle corner cases are covered by automated tests at the test framework that already exists (`vitest` + `vi.useFakeTimers`) | §"Validation Architecture" |
| REQ-73-CHECK | `just check` exits 0 | Existing rule from CLAUDE.md; no special research |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

The following are non-negotiable for every task in this phase. Plans must verify compliance, not work around it.

| Constraint | Source | Application to Phase 73 |
|------------|--------|------------------------|
| Never inline colors — use CSS custom properties | CLAUDE.md "Rules" | Two-step End "danger" state MUST use `var(--color-danger)` (already in the theme, used in `.card-action-danger` at `ReviewPanel.svelte:728-730`). No hex/rgb literals. |
| Never fight layout with positioning hacks | CLAUDE.md "Rules" | End button goes in the existing flex header row (`ReviewPanel.svelte:376-403`), not absolutely positioned. Summary caption uses existing flex column. |
| All git ops via git2 (no shelling) | CLAUDE.md "Rules" | N/A — Phase 73 has no git operations; pure session-lifecycle IPC composition. |
| `just check` before every commit and push | CLAUDE.md "Commands" | Plans must include `just check` in the verification step of every task that touches code. |
| TS 5.6 strict; never `as any` / `as unknown` | TS coding_style.md §1 | Catch blocks use `instanceof Error` narrowing (carry-forward Phase 71/72); no `as TrunkError` cast (use the `isTrunkError` guard already defined at `ReviewPanel.svelte:139-147`). |
| TDD-mode is ON (config.json `tdd_mode: true`) | `.planning/config.json` | UI/glue stays standard; component behavior with defined IPC I/O is TDD-eligible (see §"TDD Eligibility"). |

## Summary

**This phase is pure frontend wiring.** Every backend primitive needed already exists, is verified by reading `src-tauri/src/commands/review.rs`, and requires no schema change:

- `get_review_session_status` already returns the `SessionState` discriminator (`Active | ResumeAvailable | None`) at `review.rs:1147-1168`. The cold-boot resume wiring branches on this exact field.
- `resume_review_session` already handles all three `LoadOutcome` variants (`Loaded` / `RecoveredCorrupt` / `RefusedNewer`) in the thin command at `review.rs:1070-1122`. `RefusedNewer` surfaces as a `newer_version` `TrunkError`; `RecoveredCorrupt` silently persists a fresh session and emits `session-changed`; `Loaded` inserts into memory and emits.
- `end_review_session` already hard-deletes the on-disk file via `_inner` (`review.rs:110-118`), drops the in-memory entry, and emits `session-changed` (`review.rs:1124-1144`). End-review is purely a UI affordance + confirmation pattern over this command.
- `session-changed` event payload is the canonical path string (`review.rs:1142`), already consumed by `ReviewPanel`'s `$effect` listener at `ReviewPanel.svelte:347-367` with cancellation-leak protection (WR-03) in place.

The change set is concentrated in `ReviewPanel.svelte` and its test file. `ReviewPanel.svelte`'s `reload()` (lines 210-253) is the single chokepoint where cold-boot resume + post-End cleanup + multi-tab session-changed all converge.

**Primary recommendation:** Stage the work in three small tasks: (1) cold-boot resume in `reload()` + LoadOutcome handling; (2) header layout — End button + summary caption + two-step confirm rune state; (3) empty-state gating with distinct copy. Each task is independently TDD-able against the existing `installReads` test harness in `ReviewPanel.test.ts`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Cold-boot resume decision (branch on `status.state`) | Frontend component (`ReviewPanel.svelte:reload()`) | — | Backend already surfaces the discriminator; the *decision* to call `resume_review_session` is a UI policy choice (D-01 chose "panel mount", not "repo open"). |
| Session lifecycle primitives (start/resume/end + status) | Tauri command layer (`commands/review.rs`) | git2/std (none in Phase 73) | All four `_inner` fns + their thin commands exist and emit `session-changed`; no additions needed. |
| Two-step End button state | Frontend component (rune-local `$state`) | — | UI-only confirmation; never persisted. Same architectural tier as the existing `copied` rune at `ReviewPanel.svelte:132-134`. |
| Multi-tab live coordination | Tauri event bus (`session-changed` emit/listen) | Frontend component reload | Backend emits on every mutation (start/resume/end/comment/seed); frontend's `$effect` listener at `ReviewPanel.svelte:353-367` filters by canonical path and triggers `reload()`. |
| Empty-state copy gating | Frontend component (derived state) | — | `commits.length === 0 && comments.length === 0` is a derivable view-state, not a persisted one. The cold-vs-warm distinction is the `status.state === 'None'` vs anything else discriminator surfaced by the backend's existing IPC. |
| Session summary line ("N comments · M commits") | Frontend component (derived from existing reactive arrays) | — | `comments.length` and `commits.length` are already `$state` arrays in `ReviewPanel.svelte:37-38`. No new IPC; pure derivation. |

## Standard Stack

### Core (already in the project — no installs)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Svelte | 5 (runes mode: `$state`, `$derived`, `$effect`) | UI reactivity | Phase 72 simplified the panel state machine to two axes (`reviewActive × rightPaneMode`); Phase 73's new state stays inside `ReviewPanel.svelte` as local runes [VERIFIED: `package.json` + `src/lib/review-session.svelte.ts`]. |
| `@lucide/svelte` | 0.577.0 | Icon set | Already used (`Clipboard`, `MessageSquarePlus`, `MessagesSquare`). `Trash2` exported by `node_modules/@lucide/svelte/dist/icons/index.d.ts`; `MessageSquareOff` also exported (candidate for cold-empty illustration if desired) [VERIFIED: grep on `index.d.ts` + file existence check]. |
| `@tauri-apps/api` | (per package.json) | `listen` for `session-changed` | Existing pattern in `ReviewPanel.svelte:9`, `:356-367` (with cancellation-leak fix from WR-03). |
| `@tauri-apps/plugin-dialog` | (per package.json) | `ask()` for delete-comment confirms | Note: NOT used for End-review. D-05 locks an inline two-step pattern, not a modal. The existing `ask`-based delete pattern (`ReviewPanel.svelte:327-339`) is intentionally distinct. |
| `vitest` + `@testing-library/svelte` + `vi.useFakeTimers` | (per package.json) | Tests | Phase 72's Copy describe block at `ReviewPanel.test.ts:641-813` is the template; the timer-scoping pattern (fake timers per `describe`, microtask flush via `flushFake`) extends 1:1 to the two-step End button's `setTimeout` revert. |

### Supporting

| Library | Purpose | When to Use |
|---------|---------|-------------|
| `safeInvoke` (`src/lib/invoke.ts`) | Wraps `invoke<T>` and parses string-encoded `TrunkError`. | Every Tauri IPC call in this phase: `get_review_session_status`, `resume_review_session`, `end_review_session`. Already used everywhere in `ReviewPanel.svelte`. |
| `showToast` (`src/lib/toast.svelte.ts`) | Two kinds: `"success"` and `"error"`. **No "warning" / "info" level exists.** | Surface failures of `end_review_session` IPC and `newer_version` from `resume_review_session`. For `RecoveredCorrupt`, no toast emitted from the frontend in this phase — the backend's `LoadOutcome::RecoveredCorrupt` arm silently persists a fresh session and emits `session-changed`; surfacing a "we recovered a corrupt file" message would require a backend signaling change out of Phase 73's scope. [VERIFIED: read of `toast.svelte.ts` — only `"success"` and `"error"` exist.] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff | Why Rejected |
|------------|-----------|----------|--------------|
| Inline two-step End (D-05) | `@tauri-apps/plugin-dialog` `ask()` modal | Heavier confirmation friction; consistent with `deleteComment` | D-05 locked. The two-step inline pattern matches the lightweight character of the Copy affordance and avoids modal context switching. |
| Cold-boot resume from `reload()` (D-01) | New IPC `resume_or_start_review_session` | Single round-trip; fewer frontend branches | D-01 locked. Adding an IPC for a one-line frontend branch violates YAGNI; the discriminator we need is already in `SessionStatus`. |
| Toast on Recovered/Refused | Silent on Recovered + toast on Refused | More info to user | Toast level mismatch (`showToast` has no `warn`); RefusedNewer already throws a `TrunkError` from the thin command, so the frontend's catch arm is the natural surface. |

**No installs required for this phase.** Skipping Package Legitimacy Audit (no external packages added).

## Architecture Patterns

### System Architecture Diagram (data flow for the three new lifecycle paths)

```
COLD-BOOT RESUME (REQ-73-RESUME)
─────────────────────────────────────────────────────────────────
  User opens ReviewPanel (first time after app boot)
    │
    ▼
  ReviewPanel.svelte: $effect → reload()
    │
    ▼
  safeInvoke("get_review_session_status", { path })
    │              [Rust: get_review_session_status_inner +
    │               merge_status(file_exists, in_memory_present)]
    ▼
  status.state ∈ { Active | ResumeAvailable | None }
    │
    ├── 'ResumeAvailable' ───► safeInvoke("resume_review_session", { path })
    │                            │  [Rust: LoadOutcome match: Loaded → insert,
    │                            │   RecoveredCorrupt → fresh + save,
    │                            │   RefusedNewer → return newer_version error]
    │                            ▼
    │                          on success: continue to Promise.all reads
    │                          on newer_version: showToast(...) + skip reads
    │
    ├── 'Active' ────────────► Promise.all reads (already-in-memory path)
    │
    └── 'None' ──────────────► reads return no_session (already swallowed by
                                catch arm at ReviewPanel.svelte:242-247);
                                empty arrays render cold empty state.

END-REVIEW (REQ-73-END)
─────────────────────────────────────────────────────────────────
  User clicks "End review" (first click)
    │
    ▼
  endConfirming = true; clearTimeout(endTimer); endTimer = setTimeout(revert, 3000)
    │
    ▼
  Button re-renders as "Click again to confirm" (danger color)
    │
    ▼
  Second click within 3s ───► safeInvoke("end_review_session", { path })
    │                          [Rust: delete file + drop in-memory + emit session-changed]
    │                          │
    │                          ▼
    │                        on success: clear all local arrays, reset endConfirming
    │                        on failure: showToast("Failed to end review: ...", "error")
    │                                    DO NOT clear arrays (no partial state)
    │
    Otherwise (3s elapses): endConfirming = false; button reverts to "End review"

MULTI-TAB COORDINATION (REQ-73-MULTITAB)
─────────────────────────────────────────────────────────────────
  Tab A: end_review_session → emit("session-changed", canonical)
    │
    ▼
  Tauri event bus
    │
    ▼
  Tab B: ReviewPanel.svelte:353-367 listener fires
    │
    ▼
  payload === canonicalPath → reload()
    │
    ▼
  get_review_session_status → state === 'None'
    │
    ▼
  Empty arrays → cold empty state renders (REQ-73-EMPTY)
```

### Recommended Component Structure (changes within `ReviewPanel.svelte`)

```
ReviewPanel.svelte
├── <script>
│   ├── existing state (commits, comments, resolutions, canonicalPath, draft state)
│   ├── existing Copy state (copied, copyTimer) — lines 130-134
│   ├── NEW: endConfirming = $state(false)
│   ├── NEW: endTimer = null (plain handle, not $state — mirrors copyTimer at line 134)
│   ├── NEW: derived(commentsCount, commitsCount, isColdEmpty) — small reactive surface
│   ├── reload() — MODIFY to insert resume branch (see "Cold-boot resume — exact wiring")
│   ├── onCopyClick() — unchanged
│   ├── NEW: onEndClick() — two-step confirm handler (see "Two-step End button state model")
│   ├── NEW: cancelEndConfirm() — clear timer + flag (called on success/timeout/destroy)
│   └── existing $effects (initial load, session-changed listener)
└── <template>
    ├── header row (lines 376-403) — ADD: End button + summary caption above list
    ├── existing list-render path (lines 417-629) — MODIFY empty-state copy gating
    └── existing <style> — ADD: .end-button + .end-button-confirming classes
```

### Pattern 1: Two-step button rune state (D-05)

**What:** A boolean `$state` flag flips the button label/color on first click and auto-reverts after a timeout. Second click within the window invokes the destructive IPC.

**When to use:** Inline confirmation of a destructive action without a modal dialog (D-05 locked).

**Carry-forward from:** The `copied` state at `ReviewPanel.svelte:130-134` is the exact template — same `$state` boolean, same plain timer handle, same `clearTimeout` before `setTimeout` discipline (Phase 71 rule).

**Example (sketch — verify final form during TDD):**

```typescript
// Source: ReviewPanel.svelte:130-134 pattern + Phase 71's clearTimeout discipline
let endConfirming = $state(false);
let endTimer: ReturnType<typeof setTimeout> | null = null;

function startEndConfirm() {
  if (endTimer !== null) clearTimeout(endTimer);
  endConfirming = true;
  endTimer = setTimeout(() => {
    endConfirming = false;
    endTimer = null;
  }, 3000);
}

function cancelEndConfirm() {
  if (endTimer !== null) clearTimeout(endTimer);
  endTimer = null;
  endConfirming = false;
}

async function onEndClick() {
  if (!endConfirming) {
    startEndConfirm();
    return;
  }
  cancelEndConfirm();
  try {
    await safeInvoke("end_review_session", { path: repoPath });
    // session-changed listener will fire reload(); no manual array clear needed
    // because the listener round-trip is the canonical refresh.
  } catch (e) {
    const msg = e instanceof Error ? e.message : isTrunkError(e) ? e.message : String(e);
    showToast(`Failed to end review: ${msg}`, "error");
  }
}
```

### Pattern 2: Cold-boot resume — exact wiring in `reload()`

**What:** Branch on `status.state` from `get_review_session_status` to insert a `resume_review_session` call before the parallel reads.

**When to use:** This phase only — the existing `reload()` is the chokepoint and D-01 locks the location.

**Example (sketch — verify final form during TDD against installReads):**

```typescript
// Source: existing reload() at ReviewPanel.svelte:210-253 + D-01/D-07
async function reload() {
  let status: SessionStatus | null = null;
  try {
    status = await safeInvoke<SessionStatus>("get_review_session_status", {
      path: repoPath,
    });
    canonicalPath = status.canonical_path;
  } catch {
    // Tolerate (existing behavior); the catch arm below handles no_session.
  }

  // NEW (D-01): cold-boot resume before reads. Only fires if the session is on
  // disk but not in memory — i.e. first open after app boot.
  if (status?.state === "resume-available") {
    try {
      await safeInvoke("resume_review_session", { path: repoPath });
      // session-changed emits inside resume; the listener may fire reload() again,
      // which is idempotent (re-read of three lists).
    } catch (e) {
      // D-07: newer_version is the only error we expect; surface as toast,
      // continue to reads (which will then return no_session → cold empty state).
      const msg = e instanceof Error ? e.message : isTrunkError(e) ? e.message : String(e);
      showToast(`Failed to resume review: ${msg}`, "error");
    }
  }

  // Existing reads — unchanged.
  try {
    const [nextCommits, nextComments, nextResolutions] = await Promise.all([...]);
    commits = nextCommits; comments = nextComments; resolutions = nextResolutions;
  } catch (e) {
    if (isTrunkError(e) && e.code === "no_session") {
      commits = []; comments = []; resolutions = [];
      return;
    }
    showToast("Failed to load review comments. Reload the panel to retry.", "error");
  }
}
```

**Recursion-safety note:** `resume_review_session` emits `session-changed`, which fires our own listener which calls `reload()` again. The second `reload()` finds `status.state === 'active'`, skips the resume branch, and re-reads the three lists. This is idempotent and produces no observable double-flicker (the second read overwrites identical data). The plan should verify this with a test that asserts `resume_review_session` is called **exactly once** on a cold boot even though `session-changed` fires.

### Pattern 3: Empty-state gating — cold vs. warm

**What:** Distinguish "no session exists" (cold) from "session active but zero comments" (warm-empty).

**When to use:** D-06 locks distinct copy for these two states.

**Discriminator (no race):**

```typescript
// status.state read inside reload() is the source of truth. Cache it as local
// $state so empty-state copy can branch on it:
let sessionState = $state<SessionState>("none");
// Inside reload(): sessionState = status?.state ?? "none";

const isCold = $derived(
  sessionState === "none" && commits.length === 0 && comments.length === 0
);
const isWarmEmpty = $derived(
  (sessionState === "active" || sessionState === "resume-available") &&
  comments.length === 0
);
```

**Race analysis:** `reload()` is async and sequential — `status` resolves before the resume call and before the reads. By the time the reads finish, `sessionState` is already set. The template re-renders once when `sessionState` is assigned and again when `comments`/`commits` are assigned. There is no observable window where `sessionState === "active"` and `comments === []` simultaneously appears as "cold" — the gates use AND on the live values, so the false intermediate state (sessionState=none, comments=[]) is precisely "cold", which is correct. No `loading` flag needed beyond the implicit one (the empty list rendering during the first paint already matches cold; transitioning to warm after the reads is a positive update).

### Pattern 4: Session summary line (D-04)

**What:** A muted caption "N comments · M commits" above the comments list.

**Where:** Inside the existing scroll body, just above the `{#if groups.length === 0}` block at `ReviewPanel.svelte:417`. NOT inside the header row (which hosts Copy + End).

**Visual analog already in the codebase:** Muted captions exist in the empty-state hints — see `ReviewPanel.svelte:420-422` (`color: var(--color-text-muted); font-size: 11px`) and `:427-429`. Reuse that exact style triplet.

**Implementation:**

```svelte
{#if sessionState === "active" || sessionState === "resume-available"}
  <span style="color: var(--color-text-muted); font-size: 11px; padding: 2px 0;">
    {comments.length} comments · {commits.length} commits
  </span>
{/if}
```

No pluralization (the codebase has no precedent for it; "1 comments" is acceptable per existing muted-caption style — but plan-phase may polish with a `commentsLabel = $derived(...)` if desired, that's a copywriting nit).

### Anti-Patterns to Avoid

- **Reintroducing `panelMode`:** Phase 72 deleted the panel-internal mode swap. The summary line and empty-state copy MUST live in the existing list-render path, not a swapped face. (Source: `72-CONTEXT.md` §"State-machine simplification" + `72-VERIFICATION.md` REQ-72-4.)
- **Adding states to `_inner`:** `merge_status` happens in the thin command, never in `_inner` (`review.rs:120-122` docstring + `:145-151`). Do not propose new `SessionState` variants; the existing three are sufficient.
- **Treating `reload()`'s `no_session` arm as an error:** A missing session is a normal state. The catch arm at `:242-247` swallows `no_session` silently and clears arrays. Preserve this — do not toast on cold boot when there's genuinely no session.
- **Modal confirmation for End:** D-05 locks inline two-step. `@tauri-apps/plugin-dialog` `ask()` is used for `deleteComment` and stays scoped to per-comment deletes.
- **Awaiting `end_review_session` without try/catch:** Carry forward Phase 71's `instanceof Error` narrowing + `showToast` pattern. Half-success states are not allowed (the canonical reload happens via `session-changed` listener — if End fails, no list mutation should occur in the UI).
- **Rebinding `Cmd+Shift+R`:** REQ-72-1b was retracted. Out of scope.
- **`as TrunkError` cast:** Use the existing `isTrunkError` guard at `ReviewPanel.svelte:139-147`. Note `DiffPanel.svelte:145` still uses `(e as TrunkError).message` — do NOT copy that style; it's a pre-existing wart, not a pattern to extend.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Session lifecycle persistence | A new IPC or schema field | Existing `start/resume/end_review_session_inner` + `get_review_session_status` + `merge_status` (all in `review.rs`) | Phase 65 designed the lifecycle. Every primitive needed for cold-boot resume + End-review is present. |
| Cross-tab live coordination | A custom Tauri event or polling loop | Existing `session-changed` emit/listen (`review.rs:1142` emit + `ReviewPanel.svelte:353-367` listen) | The listener is already wired and has WR-03 cancellation-leak protection. |
| Confirmation timer | A bespoke Promise-with-timeout combinator | The exact `copied` + `copyTimer` + `clearTimeout`/`setTimeout` pattern at `ReviewPanel.svelte:130-134, 311-320` | Same shape, same discipline; carry-forward keeps the code uniform and the test pattern (vi.useFakeTimers) reusable. |
| Empty-state copy detection | A backend "is this a fresh repo?" call | The existing `SessionStatus.state` discriminator | The backend already classifies disk + memory into three states; the frontend just reads the field. |
| Pluralization / i18n | A custom format helper | Inline string template (`{n} comments · {m} commits`) | No i18n infrastructure exists in the project; speculative. YAGNI. |

**Key insight:** Phase 73 is *integration*, not *new capability*. Every backend primitive landed in Phase 65 (with `session-changed` augmentations through Phase 69). The work is connecting the existing wires through the panel's existing single chokepoint (`reload()`) and adding two small UI affordances (End button + summary caption) with one tiny piece of new local rune state (`endConfirming`). Treat any task that proposes new IPC or new backend code as scope creep.

## Common Pitfalls

### Pitfall 1: `reload()` recursion via `session-changed`
**What goes wrong:** Calling `resume_review_session` inside `reload()` emits `session-changed`, which fires the listener which calls `reload()` again, which sees `status.state === 'active'` and re-reads.
**Why it happens:** Backend emits `session-changed` from every mutating thin command; the frontend listener auto-reloads on payload match.
**How to avoid:** This is *self-stabilizing* (the recursive call doesn't re-resume because state is now `'active'`), but the test plan must explicitly assert `resume_review_session` is called **exactly once** on a cold boot. If a future change adds emission on `get_review_session_status` itself (it does not today — read of `review.rs:1147-1168` confirms no emit), infinite loop.
**Warning signs:** Watch for `resume_review_session` being called twice in any test; that's the regression signal.

### Pitfall 2: Stale `canonicalPath` after End
**What goes wrong:** Tab B's listener filter is `if (canonicalPath && event.payload !== canonicalPath) return;` (`ReviewPanel.svelte:357`). After tab A ends, tab B's `canonicalPath` is still set to the old path; the filter passes (payload matches), `reload()` fires, status returns `'None'`, all good. But if the user then *also* closes the repo in tab B, `canonicalPath` should be cleared, which doesn't happen automatically.
**Why it happens:** End-in-this-tab doesn't currently clear `canonicalPath`. Today's code only sets it inside `reload()`. After a successful End in tab B itself, `canonicalPath` retains the canonical path string until the next `reload()` resets it (which is what `session-changed` triggers).
**How to avoid:** After a successful End-in-this-tab, *do not* manually clear `canonicalPath` — let the `session-changed` listener's `reload()` round-trip update it from `get_review_session_status` (which always returns the canonical path). The subsequent reads return `no_session` → arrays clear → cold empty state renders. This is the same code path as multi-tab, so testing one covers both.
**Warning signs:** Any test where tab B's `canonicalPath` somehow gets cleared *before* the listener fires would indicate a manual mutation that breaks the multi-tab filter.

### Pitfall 3: Timer-leak on component destroy
**What goes wrong:** The `endTimer` setTimeout fires after the component unmounts → assigns `$state` on a torn-down instance → either no-op or warning.
**Why it happens:** Svelte does not auto-clear timers on destroy.
**How to avoid:** Add a `$effect` cleanup (or rely on the existing `$effect`'s teardown function pattern) that calls `clearTimeout(endTimer)` if non-null. Mirror the cancellation-leak protection already in `ReviewPanel.svelte:353-367` for the listener.
**Warning signs:** Test fakes warning "tried to update component after unmount" — or a fake-timers test where advanceTimersByTime after unmount triggers an assertion.

### Pitfall 4: Hardcoded color literal for "danger" state
**What goes wrong:** CLAUDE.md prohibits inline colors. A naive "make it red" implementation uses `style="color: red"` or `style="background: #d00"`.
**Why it happens:** The two-step confirming state visually needs to signal danger.
**How to avoid:** The theme already has `--color-danger` (used at `ReviewPanel.svelte:728-730` for `.card-action-danger`). Reuse `var(--color-danger)` in the End button's `:not(.end-button-confirming)` and a stronger contrast (`var(--color-danger-bg)` if it exists, or just `background: var(--color-danger); color: var(--color-on-accent)`) for the confirming state.
**Warning signs:** Any new CSS variable proposal or any hex/rgb literal in the diff.

### Pitfall 5: Showing the End button on cold empty state
**What goes wrong:** End button visible when there's no session to end → clicking it surfaces a `no_session` error toast.
**Why it happens:** Disabled-state oversight.
**How to avoid:** Gate the End button on the existence of a session: `disabled={sessionState === 'none'}` (or hide entirely with `{#if sessionState !== 'none'}`). Visible-but-disabled is consistent with the Copy button's `disabled={!hasAnyComment}` pattern at `:392`.
**Warning signs:** A test where clicking End on a cold panel produces an IPC call — that's the regression.

### Pitfall 6: Race between mount $effect and session-changed listener
**What goes wrong:** Cold-boot resume's `reload()` is in-flight; meanwhile another mutation in another tab fires `session-changed`; the listener calls `reload()` concurrently → two overlapping async chains write to the same `$state`. Last-write-wins is fine *if* both writes produce the same data, but interleaved reads can produce a brief incorrect state.
**Why it happens:** No cancellation between two concurrent `reload()` invocations.
**How to avoid:** This was identified for Phase 69 (WR-02 — the `canonicalPath` AWAIT-before-listener-fires fix at `:217-228`). The current code already does the await. For Phase 73, the new resume call adds one more sequential IPC; the same "set `canonicalPath` early, then proceed" invariant holds — `canonicalPath` is assigned at `:224` before the resume call. No new race.
**Warning signs:** A test where two `reload()` invocations interleave and produce a different final state than either alone.

## Runtime State Inventory

> Not a rename/refactor/migration phase. **Section omitted.**

## Code Examples

Verified patterns from existing code in this repo:

### Carry-forward: Copy button rune state + clearTimeout discipline (template for End)
```typescript
// Source: src/components/ReviewPanel.svelte:130-134, 307-325 (Phase 72)
let copied = $state(false);
let copyTimer: ReturnType<typeof setTimeout> | null = null;

async function onCopyClick() {
  try {
    const md = await session.generate(repoPath);
    await writeText(md);
    if (copyTimer !== null) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => {
      copied = false;
      copyTimer = null;
    }, 1500);
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    showToast(`Failed to copy: ${msg}`, "error");
  }
}
```

### Carry-forward: TrunkError guard + errorMessage extractor
```typescript
// Source: src/components/ReviewPanel.svelte:139-156
function isTrunkError(e: unknown): e is TrunkError {
  return typeof e === "object" && e !== null && "code" in e && "message" in e
    && typeof (e as { message: unknown }).message === "string";
}

function errorMessage(e: unknown, fallback: string): string {
  if (e instanceof Error) return e.message;
  if (isTrunkError(e)) return e.message;
  return fallback;
}
```
End-review's catch block should use `errorMessage(e, "Failed to end review")` for symmetry with the rest of `ReviewPanel.svelte`.

### Carry-forward: session-changed listener with cancellation-leak protection
```typescript
// Source: src/components/ReviewPanel.svelte:353-367 (WR-03 fix)
$effect(() => {
  let unlisten: (() => void) | undefined;
  let cancelled = false;
  listen<string>("session-changed", (event) => {
    if (canonicalPath && event.payload !== canonicalPath) return;
    reload();
  }).then((fn) => {
    if (cancelled) fn();
    else unlisten = fn;
  });
  return () => {
    cancelled = true;
    unlisten?.();
  };
});
```
This effect is **untouched** by Phase 73. The new behavior (multi-tab End → reload → cold empty) is achieved by reusing it as-is.

### Carry-forward: Vitest fake-timers per describe-block
```typescript
// Source: src/components/ReviewPanel.test.ts:641-657 (Phase 72 Copy describe)
describe("Copy", () => {
  beforeEach(() => { vi.useFakeTimers(); });
  afterEach(() => { vi.useRealTimers(); });
  async function flushFake() { await Promise.resolve(); await tick(); }
  // ... tests use vi.advanceTimersByTime() ...
});
```
The new "End" describe block must follow this exact scoping discipline — fake timers MUST NOT leak into other describe blocks (the file-global `flush` uses `setTimeout(r, 0)` and deadlocks under fake timers).

### Verified: command-aware safeInvoke dispatcher (`installReads`)
```typescript
// Source: src/components/ReviewPanel.test.ts:100-121
function installReads(opts: {
  commits?: SessionCommit[];
  comments?: Comment[];
  resolutions?: CommentResolution[];
  generateDoc?: string;
  // NEW for Phase 73: status?: SessionStatus; resumeRejection?: unknown; endRejection?: unknown;
}) {
  vi.mocked(safeInvoke).mockReset();
  vi.mocked(safeInvoke).mockImplementation((cmd: string) => {
    switch (cmd) {
      case "list_session_commits": return Promise.resolve(opts.commits ?? []);
      case "list_session_comments": return Promise.resolve(opts.comments ?? []);
      case "resolve_session_comments": return Promise.resolve(opts.resolutions ?? []);
      case "generate_review_doc": return Promise.resolve(opts.generateDoc ?? "# stub\n");
      // NEW: case "get_review_session_status": return Promise.resolve(opts.status ?? defaultStatus);
      // NEW: case "resume_review_session": return opts.resumeRejection ? Promise.reject(...) : Promise.resolve(undefined);
      // NEW: case "end_review_session": return opts.endRejection ? Promise.reject(...) : Promise.resolve(undefined);
      default: return Promise.resolve(undefined);
    }
  });
}
```
**Important:** the current `installReads` does NOT handle `get_review_session_status` — the default branch returns `undefined`, which `ReviewPanel.svelte`'s reload catches silently (line 225-228). Adding the case is a one-line extension; the test plan must do this in Wave 0.

## State of the Art

No State of the Art shift relevant to Phase 73. The backend lifecycle from Phase 65 + the multi-tab `session-changed` coordination from Phase 69 are the current canonical patterns and remain unchanged.

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Implicit resume via DiffPanel comment-add (Bug 3) | Explicit resume from `ReviewPanel.reload()` on cold boot | Phase 73 (this phase) | Comments appear without user mutation. |
| Three-axis state machine (`reviewActive × rightPaneMode × panelMode`) | Two-axis (`reviewActive × rightPaneMode`) | Phase 72 | `panelMode` deleted; summary line and empty-state copy live in existing list-render path. |
| Modal confirm via `@tauri-apps/plugin-dialog` for ALL destructive ops | Inline two-step for End (D-05); modal stays for per-comment delete | Phase 73 (this phase) | Lighter UX for the more-frequent End action. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Trash2` is the right Lucide icon for End-review per D-03's proposal | Standard Stack | LOW — Claude's discretion per CONTEXT; any visually distinct destructive-action icon works. `MessageSquareOff` is also exported and an equally valid candidate. [VERIFIED present in `@lucide/svelte/dist/icons/index.d.ts`; UX fit is judgment.] |
| A2 | The cold-vs-warm empty-state copy strings from D-06 are the final wording | Pattern 3 / template sketch | LOW — D-06 explicitly defers word-level wording to plan-phase as a copywriting nit. |
| A3 | Auto-revert duration of 3s is the right feel for the two-step button | Pattern 1 | LOW — D-05 says "~3s suggested" and explicitly leaves it as Claude's discretion to tune. |
| A4 | The `session-changed` listener's `reload()` round-trip is sufficient to update `canonicalPath` after End-in-this-tab (no manual clear needed) | Pitfall 2 | LOW — verified by reading `:222-224`; `canonicalPath` is set on every `reload()` from the status response, which is `'none'` after End. |
| A5 | `showToast` having no "warning" level means `RecoveredCorrupt` cannot be surfaced from the frontend in Phase 73 | Standard Stack §Supporting | LOW — `RecoveredCorrupt` is silently handled by the backend (`review.rs:1097-1109`); user sees a fresh session, no error. Adding a warn level would be a separate widget change. |

## Open Questions

None — all D-01..D-09 decisions are locked in CONTEXT.md, all backend primitives are verified by reading `review.rs`, all frontend integration points are verified by reading `ReviewPanel.svelte`, and all carry-forward patterns are verified by reading `ReviewPanel.test.ts`. The plan-phase agent has everything needed.

The single semi-open question (whether to surface a "Review was ended in another window" toast for tab B) is explicitly Claude's discretion per CONTEXT.md and tracked as "Deferred Ideas". Default recommendation: skip the toast — the cold empty-state copy carries enough signal.

## Environment Availability

> No external dependencies. Phase 73 is pure frontend code + tests. **Section content N/A.**

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node + pnpm + vitest | Test runs | ✓ | Per `package.json` | — |
| `@lucide/svelte` `Trash2` export | End button icon | ✓ | 0.577.0 | `Trash`, `CircleX`, `OctagonX` also exported |
| `@lucide/svelte` `MessageSquareOff` (optional, cold-empty illustration) | Empty state visual | ✓ | 0.577.0 | Plain text, no icon |
| `--color-danger` CSS custom property | Two-step End danger color | ✓ | Already in theme | — (verified used at `:728`) |
| `var(--color-text-muted)`, `var(--color-surface)`, `var(--color-border)`, `var(--color-bg)` | Summary caption + button styling | ✓ | Already in theme | — (all used throughout `ReviewPanel.svelte`) |

**No missing dependencies; no fallbacks needed.**

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `vitest` + `@testing-library/svelte` + `jsdom` |
| Config file | `vitest.config.*` (per project convention; check `vite.config.ts` for the test block) |
| Quick run command | `pnpm vitest run src/components/ReviewPanel.test.ts` (or `just check` for full sweep) |
| Full suite command | `just check` (runs fmt + biome + svelte-check + clippy + cargo-test + vitest) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| REQ-73-RESUME | Cold boot (`status.state === 'resume-available'`) → `reload()` calls `resume_review_session` exactly once before the parallel reads | unit (component) | `pnpm vitest run src/components/ReviewPanel.test.ts -t "cold-boot resume"` | ❌ Wave 0 (new describe block) |
| REQ-73-RESUME | `status.state === 'active'` → `reload()` does NOT call `resume_review_session` (skip path) | unit | `pnpm vitest ... -t "skips resume when already active"` | ❌ Wave 0 |
| REQ-73-RESUME | `status.state === 'none'` → `reload()` does NOT call `resume_review_session`; arrays stay empty; cold empty state renders | unit | `pnpm vitest ... -t "no session means no resume call"` | ❌ Wave 0 |
| REQ-73-RESUME | `resume_review_session` rejects with `newer_version` `TrunkError` → showToast with "error" kind; reads still attempted but yield no_session | unit | `pnpm vitest ... -t "newer_version surfaces toast"` | ❌ Wave 0 |
| REQ-73-RESUME | `resume_review_session` rejects with arbitrary IPC failure → showToast with extracted message; no half-success state | unit | `pnpm vitest ... -t "resume IPC failure surfaces toast"` | ❌ Wave 0 |
| REQ-73-END | First click on End button: button label/state flips to confirming; `end_review_session` NOT yet invoked | unit + vi.useFakeTimers | `pnpm vitest ... -t "first click enters confirming state"` | ❌ Wave 0 |
| REQ-73-END | Second click within 3s: `end_review_session` invoked with `{ path }`; on success, no manual array clear (listener handles refresh) | unit + vi.useFakeTimers | `pnpm vitest ... -t "second click invokes end_review_session"` | ❌ Wave 0 |
| REQ-73-END | No second click within 3s → `vi.advanceTimersByTime(3000)` → button reverts to "End review"; `end_review_session` never invoked | unit + vi.useFakeTimers | `pnpm vitest ... -t "auto-reverts after 3s"` | ❌ Wave 0 |
| REQ-73-END | `clearTimeout` before `setTimeout` discipline: repeated first-clicks within window do not race-fire revert | unit + vi.useFakeTimers | `pnpm vitest ... -t "re-arm timer on repeated first click"` | ❌ Wave 0 |
| REQ-73-END | End IPC rejection → showToast with extracted message; button reverts; no array clear | unit | `pnpm vitest ... -t "end IPC failure surfaces toast"` | ❌ Wave 0 |
| REQ-73-END | Component unmount during confirming window → `clearTimeout` called; no leaked timer | unit + vi.useFakeTimers | `pnpm vitest ... -t "timer cleared on destroy"` | ❌ Wave 0 |
| REQ-73-EMPTY | Cold (`status.state === 'none'`, no comments, no commits) → cold copy visible; warm copy NOT in DOM | unit | `pnpm vitest ... -t "renders cold empty state when no session"` | ❌ Wave 0 |
| REQ-73-EMPTY | Warm-empty (`status.state === 'active'`, zero comments) → warm copy visible; cold copy NOT in DOM | unit | `pnpm vitest ... -t "renders warm empty state when session active and zero comments"` | ❌ Wave 0 |
| REQ-73-EMPTY | End button hidden/disabled when `sessionState === 'none'` | unit | `pnpm vitest ... -t "end button gated by session presence"` | ❌ Wave 0 |
| REQ-73-SUMMARY | When session active and N comments + M commits exist → "N comments · M commits" visible in DOM | unit | `pnpm vitest ... -t "renders session summary line"` | ❌ Wave 0 |
| REQ-73-SUMMARY | Summary line hidden when no session (cold) | unit | `pnpm vitest ... -t "no summary when cold"` | ❌ Wave 0 |
| REQ-73-MULTITAB | Simulate `session-changed` event with matching canonical path → listener fires reload → if backend now reports `'none'`, panel renders cold empty state | unit (mock listen) | `pnpm vitest ... -t "multi-tab end clears panel"` | ❌ Wave 0 |
| REQ-73-MULTITAB | `session-changed` event with mismatched canonical path → listener filters out, no reload, no IPC | unit | This test already exists in spirit at the panel's listener filter; extend with an end-of-other-repo scenario | ❌ Wave 0 (may augment existing test) |
| REQ-73-CHECK | `just check` exits 0 after the phase | manual / CI | `just check` | ✓ (existing) |

### Sampling Rate (Nyquist Coverage)

The lifecycle has five distinct paths; each MUST be sampled at least once in the automated suite. The cases above cover:
- **Cold-boot resume path:** Loaded (REQ-73-RESUME #1), RefusedNewer/newer_version (#4), arbitrary failure (#5), no-op skip paths (#2, #3). `RecoveredCorrupt` is handled silently by the backend; no frontend-visible behavior to assert in Phase 73 → not a separate test.
- **End-review path:** confirm flow (REQ-73-END #1–4), failure (#5), unmount safety (#6).
- **Empty-state transitions:** cold (REQ-73-EMPTY #1), warm-empty (#2), gating (#3).
- **Summary line:** present + hidden (REQ-73-SUMMARY).
- **Multi-tab:** matching payload reload (REQ-73-MULTITAB #1), mismatched filter (#2).

**Per task commit:** `pnpm vitest run src/components/ReviewPanel.test.ts`
**Per wave merge:** `pnpm vitest run` (whole frontend suite — should take seconds)
**Phase gate:** `just check` green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `installReads` in `src/components/ReviewPanel.test.ts` — extend the dispatcher to handle `get_review_session_status`, `resume_review_session`, `end_review_session`. Add `status?: SessionStatus` and rejection overrides to the `opts` type. (~10 lines.)
- [ ] New describe block `describe("cold-boot resume", ...)` — 5 tests (REQ-73-RESUME cases above).
- [ ] New describe block `describe("End review", ...)` with `vi.useFakeTimers()` scoping per Phase 72 pattern — 6 tests (REQ-73-END cases).
- [ ] New tests in the existing top-level describe (or a new `describe("empty states", ...)`) — 3 tests (REQ-73-EMPTY).
- [ ] New test in a new `describe("summary line", ...)` — 2 tests.
- [ ] Augment existing session-changed listener test (or add) — 2 tests (REQ-73-MULTITAB).
- [ ] No framework install needed; no new test config; vitest + fake timers are already in use.

Total new tests: ~18 unit tests, all in `ReviewPanel.test.ts`. No new test files.

## TDD Eligibility (config.json `tdd_mode: true`)

Each new behavior in this phase has defined IPC I/O (mocked via `installReads`) and produces observable DOM changes or `safeInvoke` call records. TDD applies as follows:

| Behavior | TDD-eligible | Notes |
|----------|--------------|-------|
| Cold-boot resume branch in `reload()` | ✓ Yes | Write failing test asserting `resume_review_session` is called when `status.state === 'resume-available'`. Implement minimum change in `reload()`. Refactor on green. |
| LoadOutcome error handling (newer_version toast) | ✓ Yes | Write failing test that rejects `resume_review_session` with TrunkError shape; assert toast call. |
| Two-step End button (first click) | ✓ Yes | Write failing test asserting button text changes after one click; no IPC. Implement `endConfirming` rune toggle. |
| Two-step End button (second click invokes IPC) | ✓ Yes | Write failing test asserting `end_review_session` called only after two clicks. |
| Auto-revert after 3s | ✓ Yes | Fake-timers test asserting button reverts after `vi.advanceTimersByTime(3000)`. |
| Empty-state copy gating (cold vs warm) | ✓ Yes | DOM assertion tests with two different `installReads` setups. |
| Summary line presence/absence | ✓ Yes | DOM assertion tests gated on `installReads` status. |
| Multi-tab session-changed → cold state | ✓ Yes | Invoke the mocked `listen` callback directly (see DiffPanel test patterns for how this is done) and assert subsequent state. |
| Lucide icon choice (Trash2 vs alternatives) | ✗ No | Pure visual; UI/glue; no defined I/O. Standard implementation, not TDD. |
| CSS class names + style props | ✗ No | UI/glue; covered by visual UAT, not unit tests. |

**Plan-phase guidance:** Group the ~18 tests into TDD pairs (Red → simplest Green → Refactor) keyed off the behaviors above. The header layout change (adding the End button + summary caption) is best implemented after the behavior tests drive out the rune state, so the template renders the new state shape on first arrival.

## Security Domain

> `security_enforcement` is not explicitly disabled in `.planning/config.json`. Treating as enabled for completeness; risk is low (no new IPC, no new principal, no new persisted state).

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | No new identity in this phase. |
| V3 Session Management | no | "Session" here is a review session, not an auth session. No principal binding. |
| V4 Access Control | no | All operations are on the local user's own filesystem; no privileged escalation surface. |
| V5 Input Validation | yes | `repoPath` is forwarded to existing IPCs; backend validates via `canonical_repo_path` (`review.rs:61-69`) which rejects unknown repos with `not_open`. No new input. |
| V6 Cryptography | no | No crypto. |

### Known Threat Patterns for {Tauri 2 + Svelte 5 lifecycle wiring}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Accidental destructive op via double-click | Tampering | Two-step inline confirm (D-05) — explicit by design. |
| Race deleting session file mid-resume | Tampering | Backend serializes through `Mutex<HashMap<...>>` (`review.rs:425-447` mutate_session_rmw); File ops are atomic tmp+rename (Phase 65 D-10). No new race surface in Phase 73. |
| Multi-tab data loss (tab A ends, tab B writes a comment after) | Tampering / DoS | `session-changed` event + `reload()` round-trip syncs tab B; tab B's subsequent comment write fails with `no_session` → existing `errorMessage` catch arm surfaces toast. Existing behavior; not introduced by Phase 73. |
| Markdown/HTML injection in error toast | Info disclosure | `showToast` renders plain text only (`toast.svelte.ts`). No HTML interpolation. |
| Timer-leak DoS | DoS | `clearTimeout` discipline (Phase 71 carry-forward) + component-destroy cleanup. Documented as Pitfall 3. |

**Threat-model verdict:** Phase 73 introduces **zero new IPC handlers, zero new capabilities, zero new persisted state shapes**. The threat surface is identical to Phase 72 + Phase 65 baseline. Accept LOW; no special mitigations required beyond the carry-forward patterns already in code.

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/commands/review.rs` — read in full (lines 1-1373); all backend primitives verified, including `merge_status` semantics, LoadOutcome variants, `session-changed` emit sites, and the `_inner`/thin-command split rule.
- `src/components/ReviewPanel.svelte` — read in full (lines 1-797); `reload()`, Copy state, listener wiring, error narrowing helpers, and CSS theme variable usage all verified.
- `src/components/ReviewPanel.test.ts` — read in full (lines 1-814); test patterns including `installReads` dispatcher, fake-timers scoping in Copy describe, and `flushFake` discipline all verified.
- `src/lib/review-session.svelte.ts` — read in full (86 lines); rune shape, `generate` signature, and Phase 72 simplification confirmed.
- `src/lib/types.ts:336-342` — `SessionState` is the lowercase kebab-case TS type (`"active" | "resume-available" | "none"`), `SessionStatus.state` is `SessionState`. **Important:** the comparison values in JS/TS code are lowercase kebab-case strings, NOT PascalCase (the PascalCase variants are Rust enum names; the wire serializes kebab-case via `#[serde(rename_all = "kebab-case")]` at `review.rs:27`). The example snippets in this RESEARCH.md use the correct lowercase form.
- `src/lib/invoke.ts` — `TrunkError` shape verified (plain object with `code` + `message`, NOT an Error subclass).
- `src/lib/toast.svelte.ts` — `ToastKind = "success" | "error"` only.
- `src/components/DiffPanel.svelte:140-167` — current `ensureActiveSession()` flow showing how `resume_review_session` is currently called implicitly on comment-add (the path that masks Bug 3).
- `node_modules/@lucide/svelte/dist/icons/index.d.ts` — Trash2 and MessageSquareOff exports verified.
- `.planning/phases/72-review-pane-ux-integration/72-VERIFICATION.md` — Bug 3 source-of-truth and retracted REQ-72-1b context.
- `.planning/phases/72-review-pane-ux-integration/72-CONTEXT.md` — post-Phase-72 state machine.
- `.planning/phases/65-data-model-persistence-session-lifecycle/65-CONTEXT.md` — lifecycle foundation, SC #4 contract for End-review.
- `.planning/todos/pending/phase-73-review-lifecycle.md` — carry-forward bundle and acceptance hints.

### Secondary (MEDIUM confidence)
- `.planning/config.json` — confirmed `tdd_mode: true`, `nyquist_validation: true`, `commit_docs: true`.

### Tertiary (LOW confidence)
- None. Every claim in this research traces to a primary source read in this session.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — every library cited is already installed and used in the file being modified; versions read from `package.json` and `index.d.ts`.
- Architecture: HIGH — every backend primitive read end-to-end in `review.rs`; every frontend integration point read end-to-end in `ReviewPanel.svelte`.
- Pitfalls: HIGH for #1-5 (each tied to a specific line of existing code); MEDIUM for #6 (race analysis is reasoned from existing WR-02 code, not from a failing test).
- Validation Architecture: HIGH — Phase 72's Copy describe block is the exact template; the same scoping discipline applies.

**Research date:** 2026-05-27
**Valid until:** 2026-06-27 (30 days — backend lifecycle is mature and stable since Phase 65; frontend integration points changed last in Phase 72, no churn expected).
