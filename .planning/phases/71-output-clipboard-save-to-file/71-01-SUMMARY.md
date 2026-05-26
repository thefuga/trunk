---
phase: 71-output-clipboard-save-to-file
plan: 01
subsystem: ui
tags: [tauri, clipboard, svelte, ui-affordance, lucide, vitest, tdd]

# Dependency graph
requires:
  - phase: 65-tauri-clipboard-plugin
    provides: clipboard-manager:allow-write-text capability and the
      @tauri-apps/plugin-clipboard-manager writeText callsite pattern
  - phase: 69-comment-management-ui
    provides: @lucide/svelte icon usage pattern in the same component family
  - phase: 70-excerpt-resolution-markdown-render
    provides: ReviewDocPreview.svelte host component with the
      `.preview-spacer` flex dock cell reserved for Phase 71
provides:
  - Copy button on ReviewDocPreview header — writes markdown prop to clipboard
  - Awaited-writeText + try/catch + showToast failure-surfacing pattern
    (intentional divergence from the fire-and-forget .catch(() => {})
    callsites at App.svelte:133, CommitDetail.svelte:72, CommitGraph.svelte:757)
  - First fake-timers usage (vi.useFakeTimers) in the vitest suite —
    template for future timer-driven UI tests
affects:
  - Any future "copy artifact" affordances (e.g., copy commit message,
    copy diff hunk) — the awaited-write + showToast pattern is now the
    house style for product-artifact clipboard writes

# Tech tracking
tech-stack:
  added: []  # No new packages; @tauri-apps/plugin-clipboard-manager@2.3.2 and
             # @lucide/svelte@0.577.0 already installed via Phase 65 / Phase 69
  patterns:
    - "Awaited writeText with try/catch + showToast — for artifact copies
      where silent failure is unacceptable"
    - "instanceof Error narrowing over `as` casts in catch blocks
      (matches coding_style_typescript.md §1)"
    - "clearTimeout-before-setTimeout for re-clickable UI affordances
      (RESEARCH Pitfall 2)"
    - "vi.useFakeTimers + vi.advanceTimersByTime for time-driven UI
      assertions (RESEARCH Pitfall 5)"
    - "Microtask flush via Promise.resolve()+tick() — NEVER
      setTimeout(r, 0) under fake timers"

key-files:
  created:
    - src/components/ReviewDocPreview.test.ts  # 8 unit tests, mock skeleton
  modified:
    - src/components/ReviewDocPreview.svelte  # Copy button + handler + style

key-decisions:
  - "Used Lucide Clipboard icon (not ClipboardCopy). Both exports verified
    against installed @lucide/svelte 0.577.0 (`dist/aliases/index.js`
    lines 444, 453); Clipboard reads cleaner alongside the unicode ✓ in
    the success branch."
  - "Kept the 1500ms revert duration as planned — no tuning needed."
  - "Did NOT modify the pre-existing CommentComposer.svelte noNonNullAssertion
    warnings (3, all on line 43, documented in-source as intentional
    contract assertions from Phase 68-02). Refactoring is out of scope for
    a UI-affordance plan in 71."
  - "Adjusted the test helper accessor from /Copy/ to /Cop(y|ied)/ —
    `Copied` shares only the `Cop` prefix with `Copy` (no `y` in
    `Copied`). The plan's suggested `/Copy/` regex would have silently
    failed the success-state assertions."

patterns-established:
  - "Boundary mocks for clipboard + toast facades: vi.mock with the
    explicit `vi.fn().mockResolvedValue(undefined)` shape (clipboard) and
    `vi.fn()` shape (toast). Reused verbatim from CommitGraph.test.ts
    and ReviewPanel.test.ts."
  - "Renaming-only divergence in style block: copy a sibling button's
    rule, prepend `display: inline-flex; align-items: center; gap: 4px;`
    for the icon-row layout, leave the rest identical."

requirements-completed: [OUT-01]

# Metrics
duration: ~25min
completed: 2026-05-26
---

# Phase 71-01: Copy Button on ReviewDocPreview Summary

**Awaited Tauri clipboard write with in-button ✓ Copied affordance and showToast failure surface; 8 vitest cases cover happy path, rapid re-click, timer cleanup, and three error-coercion paths.**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-05-26T22:28Z
- **Completed:** 2026-05-26T22:38Z
- **Tasks:** 3 (W0 scaffold, W1 TDD GREEN, phase gate)
- **Files modified:** 2 (1 created, 1 modified)

## Accomplishments

- Single Copy button docked into the existing `.preview-spacer` flex cell of `ReviewDocPreview`'s header; flexbox docks it to the right with no positioning hacks.
- Awaited `writeText(markdown)` via `@tauri-apps/plugin-clipboard-manager` — intentional divergence from the three fire-and-forget `.catch(() => {})` sites elsewhere in the app (the rendered review IS the user-facing artifact; silent failure is unacceptable).
- On success: `copied = $state(true)` flips the button to `✓ Copied` for 1500ms; a `clearTimeout` before each new `setTimeout` makes rapid re-clicks extend the window (D-04, RESEARCH Pitfall 2).
- On failure: `showToast(\`Failed to copy: ${msg}\`, "error")` where `msg = e instanceof Error ? e.message : String(e)` — strict-narrowing, no `as` casts.
- 8 vitest cases under `vi.useFakeTimers()` cover every observable behavior; first fake-timers usage in the suite (template for future timer-driven UI tests).
- `just check` exits 0 (fmt + biome + svelte-check + clippy + cargo-test + vitest) with 535/535 vitest cases passing across 50 files.

## Task Commits

Each task was committed atomically:

1. **Task 1 (W0 scaffold):** `e19ea32` (`test`) — test file with mocks, helpers, empty describe shell
2. **Task 2 (W1 RED):** `f49502a` (`test`) — 8 failing test cases added (7 RED, 1 passes by coincidence — back button regression guard)
3. **Task 2 (W1 GREEN):** `7e5cec9` (`feat`) — Copy button implementation in `ReviewDocPreview.svelte` makes all 8 tests pass
4. **Task 3 (Phase Gate):** `c252d9a` (`style`) — biome auto-fix (`organizeImports`) on the new test file; required by `just check`

_Plan metadata commit (this SUMMARY + REQUIREMENTS update) follows separately per the worktree commit protocol._

## Files Created/Modified

- `src/components/ReviewDocPreview.test.ts` (created) — 8 unit tests + boundary mocks + fake-timers lifecycle
- `src/components/ReviewDocPreview.svelte` (modified) — added 3 imports, `copied` state + `copyTimer` handle, `onCopy` handler, Copy button markup in header, `.copy-button` style rule

No `src-tauri/` files were touched. No `package.json` / `package-lock.json` / `bun.lock` changes (verified via `git diff --stat ccdbe27b HEAD -- src-tauri/ package.json package-lock.json bun.lock` showing no diff).

## Decisions Made

- **Lucide icon:** `Clipboard` (planned default). Both `Clipboard` and `ClipboardCopy` exist as named exports in `@lucide/svelte@0.577.0` (`node_modules/@lucide/svelte/dist/aliases/index.js:444,453`). No fallback needed.
- **"Copied" duration:** 1500ms — kept the plan default. Manual tuning deferred to `/gsd:verify-work` per 71-VALIDATION.md.
- **Test helper regex:** Used `/Cop(y|ied)/` instead of the plan's suggested `/Copy/`. See "Issues Encountered" — this is a load-bearing gotcha worth recording in retrospective.
- **Pre-existing biome warnings:** Did NOT modify `CommentComposer.svelte:43`. The three `noNonNullAssertion` warnings there are documented in-source ("the non-null assertions document that contract rather than guard it"); fixing them would require restructuring the `$derived` block introduced in Phase 68-02 and is out of scope.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — Bug] Adjusted test helper regex from `/Copy/` to `/Cop(y|ied)/`**

- **Found during:** Task 2 W1 GREEN — running the 8 tests against the implemented component.
- **Issue:** "Copied" and "Copy" share only the 3-letter prefix `Cop` (no `y` in `Copied`). The plan's suggested `getByRole("button", { name: /Copy/ })` would never match the button in its success state, silently failing tests 71-01-02 / 71-01-03 / 71-01-04. Easy to miss because `/Copy/` *looks* like it should match.
- **Fix:** Single helper `getCopyButton()` uses `/Cop(y|ied)/` to match both states; the inline `name: /Copy/` and `name: /Copied/` assertions throughout the suite continue to work because each is state-specific.
- **Files modified:** `src/components/ReviewDocPreview.test.ts`
- **Verification:** All 8 tests pass after the fix (`8 passed` from `npx vitest run src/components/ReviewDocPreview.test.ts`).
- **Committed in:** `7e5cec9` (rolled into the GREEN commit since it's a test-only adjustment that makes the implementation reachable).

**2. [Rule 3 — Blocking] Applied biome auto-fix (`organizeImports`) to the new test file**

- **Found during:** Task 3 — `just check` reported `2 errors / 3 warnings` from biome. One of the errors was the FIXABLE `assist/source/organizeImports` rule on `ReviewDocPreview.test.ts:13` (my new file ordered imports as written rather than alphabetical).
- **Issue:** The plan didn't specify biome's import-ordering rule. Without the fix, `just check` (and CI) would have stayed red.
- **Fix:** `npx biome check src/components/ReviewDocPreview.test.ts --write` — reordered imports so `@tauri-apps/*` sorts before `@testing-library/*` and `vitest` follows `svelte`.
- **Files modified:** `src/components/ReviewDocPreview.test.ts`
- **Verification:** `npx biome check src/` now reports `0 errors` and `3 warnings` (all pre-existing in `CommentComposer.svelte`); `npx vitest run src/components/ReviewDocPreview.test.ts` still `8 passed`.
- **Committed in:** `c252d9a` (separate `style(71-01)` commit per ownership.md "prefer separate commits for follow-up fixes").

### Worktree Bootstrap

- Worktree's `node_modules/` was empty on agent spawn. Ran `bun install` (lockfile = `bun.lock`) which installed 176 packages in ~408ms. Not a "deviation" per se — it's normal worktree bootstrap — but worth recording: the orchestrator's worktree-spawn step does not seed `node_modules`.

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes were necessary to reach the phase gate. No scope creep — neither expanded the surface area of the plan.

## Issues Encountered

- **Regex gotcha (resolved):** As above, `/Copy/.test("Copied") === false` because `Copied` is spelled `Cop-i-ed`, not `Cop-y-ed`. Verified by `node -e "console.log(/Copy/.test('Copied'))"` after the initial test failure. Worth a memory note for future TDD work involving past-tense button states.
- **Pre-existing biome warnings deferred:** 3 `lint/style/noNonNullAssertion` warnings on `src/components/diff/CommentComposer.svelte:43` (all from Phase 68-02 commit `3a44d6e`). Documented in-source as intentional contract assertions. **Concrete TODO:** Refactor `capturedResult` $derived in `CommentComposer.svelte` to narrow without `!` by either (a) splitting into two code paths (`captured ?? buildDiffAnchor(...)` only when all three diff-path props are non-null), or (b) widening `buildDiffAnchor`'s signature to accept `undefined` and short-circuit internally. Sizing per `continuous_improvement.md §5`: small (10–20 min, low risk). Not done here because it's a Phase 68 concern.
- **Vitest 4 strict suite shape (workaround applied):** vitest 4 reports `No test found in suite` as a failure when a `describe(...)` body has no `it` blocks. The Task 1 W0 scaffold technically fails vitest's "must have ≥1 test" check, but the plan explicitly anticipated this ("vitest may report 'no tests found' — that's the expected RED-baseline state for W0"). Resolved in Task 2 W1 RED when the 8 `it` blocks landed.

## User Setup Required

None — no external service configuration. The Tauri capability (`clipboard-manager:allow-write-text`) was granted in Phase 65 and unchanged by this plan.

## Threat-Model Adherence

| ID | Disposition | Outcome |
|----|-------------|---------|
| T-71-S | N/A | No new identity / auth / principal — verified. |
| T-71-T | N/A | No new IPC, no new persisted state, no new file write — verified by `git diff --stat src-tauri/`. |
| T-71-R | N/A | No new logging beyond `showToast`. |
| T-71-I | accept (LOW) | Error toast surfaces underlying plugin error via `e.message` / `String(e)`. Toast text is bounded by Tauri plugin error conventions; no PII, no filesystem paths in the message. |
| T-71-D | N/A | `clearTimeout` before `setTimeout` bounds timer state; one clipboard write per click. |
| T-71-E | N/A | No new capability. `clipboard-manager:allow-write-text` unchanged. |
| T-71-SC | N/A | Zero new packages. |

## Self-Check: PASSED

- File `src/components/ReviewDocPreview.test.ts` exists.
- File `src/components/ReviewDocPreview.svelte` exists and contains `class="copy-button"`.
- Commit `e19ea32` exists (`git log --oneline ccdbe27b..HEAD`).
- Commit `f49502a` exists.
- Commit `7e5cec9` exists.
- Commit `c252d9a` exists.
- `npx vitest run src/components/ReviewDocPreview.test.ts` → `8 passed`.
- `just check` exits 0.

## Next Phase Readiness

- OUT-01 complete — user can copy the generated markdown to the clipboard with explicit success/failure feedback. Manual verification items (real OS clipboard contents, visual parity, hover styling, duration tuning) handled by `/gsd:verify-work 71`.
- Phase 71 is the only plan in this phase; ready for `/gsd:verify-work 71`.
- OUT-02 (save-to-file) remains descoped per CONTEXT.md decision and the pre-execute descope commit (`940a442`).
- Pre-existing `CommentComposer.svelte:43` non-null assertion warnings remain — concrete TODO in "Issues Encountered" above.

---
*Phase: 71-output-clipboard-save-to-file*
*Completed: 2026-05-26*
