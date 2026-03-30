---
phase: 64-split-view
plan: 01
subsystem: ui
tags: [svelte, typescript, diff-viewer, segmented-control, css-custom-properties]

requires:
  - phase: 63-full-file-view
    provides: DiffToolbar ViewMode segmented control, DiffViewer dispatch, store persistence
provides:
  - ContentMode and LayoutMode type unions replacing ViewMode
  - getDiffContentMode/setDiffContentMode and getDiffLayoutMode/setDiffLayoutMode store functions
  - Legacy ViewMode key migration in store
  - Two-segmented-control toolbar (content + layout)
  - 2D dispatch in DiffViewer (layoutMode x contentMode)
  - --color-diff-phantom-bg CSS custom property
  - SplitView stub with full prop interface
affects: [64-02-PLAN, 64-03-PLAN]

tech-stack:
  added: []
  patterns:
    - "2D mode dispatch: layoutMode x contentMode independent toggles"
    - "Legacy store key migration with fallback priority"
    - "Icon-only segmented control buttons with Lucide Rows2/Columns2"

key-files:
  created: []
  modified:
    - src/lib/types.ts
    - src/lib/store.ts
    - src/app.css
    - src/lib/store.test.ts
    - src/components/DiffPanel.svelte
    - src/components/diff/DiffToolbar.svelte
    - src/components/diff/DiffViewer.svelte
    - src/components/diff/SplitView.svelte
    - src/components/DiffPanel.test.ts

key-decisions:
  - "ContentMode ('hunk'|'full') and LayoutMode ('inline'|'split') as independent type unions replacing 3-way ViewMode"
  - "Legacy store key 'diff_view_mode' migration: 'full' maps to contentMode='full', 'split' maps to layoutMode='split'"
  - "Lucide Rows2/Columns2 icons for layout toggle instead of text labels"
  - "getDiffLayoutMode mock resets required in tests after split-view test mutates stateful mock"

patterns-established:
  - "2D mode dispatch: DiffViewer uses layoutMode x contentMode for 4-way rendering"
  - "Independent store persistence: contentMode and layoutMode saved/loaded separately"

requirements-completed: [VIEW-02, VIEW-03]

duration: 7min
completed: 2026-03-30
---

# Phase 64 Plan 01: Type Refactor & Toolbar Summary

**Refactored ViewMode into ContentMode + LayoutMode with 2D dispatch, two-control toolbar with Lucide icons, legacy store migration, and phantom row CSS variable**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-30T15:35:37Z
- **Completed:** 2026-03-30T15:42:37Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Replaced 3-way ViewMode with two orthogonal dimensions (ContentMode and LayoutMode)
- Toolbar shows [Hunk|Full] text control + [Rows2|Columns2] icon control independently
- DiffViewer dispatches rendering based on all 4 combinations of content x layout
- Store persists both modes independently with transparent migration from legacy key
- All 64 tests pass (24 store + 40 DiffPanel)
- svelte-check passes with zero errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Refactor types, store, and CSS custom property** - `c71041c` (feat)
2. **Task 2: Refactor DiffPanel, DiffToolbar, DiffViewer, and update DiffPanel tests** - `09e0442` (feat)

## Files Created/Modified
- `src/lib/types.ts` - Replaced ViewMode with ContentMode + LayoutMode type unions
- `src/lib/store.ts` - Four new store functions with legacy migration from diff_view_mode key
- `src/app.css` - Added --color-diff-phantom-bg CSS custom property
- `src/lib/store.test.ts` - 7 new tests for content/layout mode store functions
- `src/components/DiffPanel.svelte` - Two independent state variables and handlers for content/layout
- `src/components/diff/DiffToolbar.svelte` - Two segmented controls with Lucide Rows2/Columns2 icons
- `src/components/diff/DiffViewer.svelte` - 2D dispatch (inline+hunk, inline+full, split+*)
- `src/components/diff/SplitView.svelte` - Stub updated with full Props interface for TypeScript
- `src/components/DiffPanel.test.ts` - Updated all mocks and assertions for new type system

## Decisions Made
- ContentMode ('hunk'|'full') and LayoutMode ('inline'|'split') as independent type unions replacing 3-way ViewMode -- enables future 4-combination rendering
- Legacy store key migration: old 'diff_view_mode' value of 'full' maps to contentMode='full', 'split' maps to layoutMode='split' -- transparent upgrade for existing users
- Lucide Rows2/Columns2 icons for layout toggle (4px horizontal padding) -- compact visual distinction per UI-SPEC
- getDiffLayoutMode mock resets required in tests -- stateful mock persists layout mode across test boundaries

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added getDiffLayoutMode mock resets to 10 tests**
- **Found during:** Task 2 (DiffPanel test updates)
- **Issue:** After the "split view stub" test changes layoutMode to "split" via stateful mock, subsequent tests rendered in split view instead of inline, causing 7 test failures
- **Fix:** Added `vi.mocked(storeMock.getDiffLayoutMode).mockImplementation(() => Promise.resolve("inline"))` to all tests that reset getDiffContentMode
- **Files modified:** src/components/DiffPanel.test.ts
- **Verification:** All 40 DiffPanel tests pass
- **Committed in:** 09e0442 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Fix necessary for test isolation correctness. No scope creep.

## Issues Encountered
- Pre-existing TabBar.test.ts failure ("calls onactivate when tab clicked") confirmed unrelated to this plan's changes -- out of scope per deviation rules

## Known Stubs
- `src/components/diff/SplitView.svelte` - Displays "Split view -- coming soon" placeholder; intentional stub to be implemented in Plan 02

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ContentMode and LayoutMode types ready for SplitView implementation (Plan 02)
- SplitView stub accepts full prop interface including contentMode
- DiffViewer's {:else} branch routes all split layout traffic to SplitView
- --color-diff-phantom-bg CSS variable available for phantom row styling

## Self-Check: PASSED

All 9 files verified present. Both task commits (c71041c, 09e0442) verified in git log.

---
*Phase: 64-split-view*
*Completed: 2026-03-30*
