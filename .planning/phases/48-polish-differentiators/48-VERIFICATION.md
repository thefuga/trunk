---
phase: 48-polish-differentiators
verified: 2026-03-24T23:30:00Z
status: human_needed
score: 6/6 must-haves verified (automated); 48-03 human checkpoint pending
re_verification: false
human_verification:
  - test: "Right-click a tab — native context menu with Close Others, Close All, Copy Path"
    expected: "Native OS context menu appears. Close Others is disabled when only one tab exists. Copy Path is disabled for empty tabs. Clicking each item performs the described action."
    why_human: "Tauri native menu popup cannot be triggered or inspected programmatically in a non-running app."
  - test: "Middle-click a tab to close it"
    expected: "Tab closes with the same behavior as clicking the X button (graceful close with dirty-check)."
    why_human: "Mouse button events (button===1) cannot be simulated via grep/static analysis."
  - test: "Open a repo already open in another tab"
    expected: "No duplicate tab created. The existing tab becomes active. The transient empty tab used to trigger the open is removed."
    why_human: "Dynamic tab state transitions require running app interaction."
  - test: "Directory count badge appears on tree nodes"
    expected: "Each directory node shows '(N)' in muted text next to its name. N is the total recursive file count, including files in subdirectories. Badge is visible when the directory is both expanded and collapsed."
    why_human: "Visual layout and recursive accuracy require visual inspection with real repo data."
  - test: "Stage an entire directory via hover action button"
    expected: "Hovering a directory in the unstaged section reveals a '+' button. Clicking it stages ALL files under that directory path. Similarly '-' on a staged directory unstages all files in it."
    why_human: "File staging state changes require running app with a real git repo."
  - test: "Expand All / Collapse All buttons in staging panel header"
    expected: "Two icon buttons appear beside the tree/list toggle when tree mode is active. They are NOT visible in flat list mode. Clicking Expand All opens all directory nodes across all sections. Clicking Collapse All closes them all."
    why_human: "Visibility toggle conditioned on treeViewEnabled and button effect on multi-section tree state requires visual interaction."
---

# Phase 48: Polish & Differentiators Verification Report

**Phase Goal:** Competitive-parity tab interactions and tree view power features that elevate UX beyond basic functionality
**Verified:** 2026-03-24T23:30:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can right-click a tab to access Close Others, Close All, and Copy Path actions | ? HUMAN | `showTabContextMenu` wired in App.svelte at line 83; Menu.new + 3 items confirmed; TabBar emits oncontextmenu |
| 2 | User can middle-click a tab to close it | ? HUMAN | `onauxclick` handler on `.tab-item` at line 35 of TabBar.svelte checks `e.button === 1` and calls `onauxclose(tab.id)`; wired in App.svelte line 399 to `closeTab(id)` |
| 3 | Opening a repository already open in a tab switches to the existing tab | ? HUMAN | Duplicate detection in `openRepoInTab` at App.svelte lines 111-124: normalizes trailing slashes, finds existing, switches `activeTabId`, removes transient empty tab |
| 4 | User can stage or unstage an entire directory by clicking an action on the directory node | ? HUMAN | `stageDirectory`/`unstageDirectory` in StagingPanel.svelte lines 107-131; `ondirectoryaction` passed to all relevant TreeFileList instances |
| 5 | Directory nodes display file count badges | ? HUMAN | `countFiles` in build-tree.ts lines 130-140; `fileCount = $derived(countFiles(node.children))` in DirectoryRow.svelte line 20; badge rendered at lines 56-61 using `var(--color-text-muted)` |
| 6 | Expand All / Collapse All buttons available in file list header | ? HUMAN | `expandAllSignal`/`collapseAllSignal` state in StagingPanel.svelte lines 44-45; `{#if treeViewEnabled}` block with ChevronsUpDown/ChevronsDownUp buttons at lines 442-485; effects in TreeFileList.svelte lines 78-93 |

**Score:** 6/6 automated checks pass. All truths blocked only by visual/interactive verification (48-03-PLAN.md human checkpoint).

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/components/TabBar.svelte` | Context menu + middle-click events, new callback props | VERIFIED | Props `oncontextmenu` and `onauxclose` defined in interface; `oncontextmenu` and `onauxclick` handlers on `.tab-item` at lines 34-35 |
| `src/App.svelte` | `closeOtherTabs`, `closeAllTabs`, `showTabContextMenu`, duplicate detection | VERIFIED | All four functions present at lines 59, 72, 83, 110; TabBar receives both new props at lines 398-399 |
| `src/lib/build-tree.ts` | `countFiles` and `collectFilePaths` exports | VERIFIED | `countFiles` at line 130, `collectFilePaths` at line 145; both exported, recursive, substantive |
| `src/components/DirectoryRow.svelte` | Count badge, hover action button, `actionLabel`/`onaction` props | VERIFIED | Badge at lines 56-61, action button at lines 63-84, props at lines 12-13 |
| `src/components/TreeFileList.svelte` | `ondirectoryaction`, `expandAllSignal`, `collapseAllSignal` props + effects | VERIFIED | Props at lines 18-20; expand effect lines 78-84; collapse effect lines 86-93; DirectoryRow receives `onaction` at line 193 |
| `src/components/StagingPanel.svelte` | `stageDirectory`, `unstageDirectory`, ChevronsUpDown/ChevronsDownUp buttons, signal state | VERIFIED | Functions at lines 107-131; buttons inside `{#if treeViewEnabled}` at lines 442-485; signals at lines 44-45; all TreeFileList instances wired |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `TabBar.svelte` | `App.svelte` | `oncontextmenu` and `onauxclose` callback props | WIRED | TabBar emits `(tabId, event)` on right-click; App.svelte receives it at line 398 as `oncontextmenu={showTabContextMenu}` |
| `App.svelte` | `@tauri-apps/api/menu` | `Menu.new` + `MenuItem.new` + `menu.popup()` | WIRED | Dynamic import at lines 84-85; `Menu.new` call at line 88; `menu.popup()` at line 107 |
| `DirectoryRow.svelte` | `src/lib/build-tree.ts` | `countFiles` import | WIRED | Import at line 4 of DirectoryRow.svelte; used in `$derived` at line 20 and rendered at line 61 |
| `TreeFileList.svelte` | `DirectoryRow.svelte` | `onaction` prop threading from `ondirectoryaction` | WIRED | Line 192: `actionLabel={ondirectoryaction ? actionLabel : ''}` and line 193: `onaction={ondirectoryaction ? () => ondirectoryaction!(row.node.path) : undefined}` |
| `StagingPanel.svelte` | `TreeFileList.svelte` | `expandAllSignal`/`collapseAllSignal`/`ondirectoryaction` props | WIRED | Unstaged TreeFileList (lines 749-764): `ondirectoryaction`, both signals. Staged (lines 829-831): `ondirectoryaction`, both signals. Rebase conflicted (lines 638-639): both signals only. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `DirectoryRow.svelte` | `fileCount` | `countFiles(node.children)` — pure recursive traversal of live tree structure | Yes — derives from actual git status data passed through component chain | FLOWING |
| `StagingPanel.svelte` | `expandAllSignal` / `collapseAllSignal` | `$state(0)` incremented by button click | Yes — counter increment triggers TreeFileList `$effect` which calls `collectDirPaths(tree)` to get real dir paths | FLOWING |
| `StagingPanel.svelte` | `stageDirectory` / `unstageDirectory` | Filters `status?.unstaged ?? []` / `status?.staged ?? []` by path prefix; calls `safeInvoke('stage_file', ...)` | Yes — reads from live git status, invokes Tauri backend | FLOWING |

### Behavioral Spot-Checks

Step 7b: SKIPPED — behaviors require a running Tauri app; no standalone runnable entry points exist for this code.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| TAB-08 | 48-01-PLAN.md | Right-click tab for context menu (Close Others, Close All, Copy Path) | SATISFIED | `showTabContextMenu` + `Menu.new` with 3 items in App.svelte; TabBar emits `oncontextmenu` |
| TAB-09 | 48-01-PLAN.md | Middle-click a tab to close it | SATISFIED | `onauxclick` handler in TabBar.svelte checking `e.button === 1`; wired to `closeTab(id)` in App.svelte |
| TAB-10 | 48-01-PLAN.md | Opening duplicate repo switches to existing tab | SATISFIED | Duplicate detection with path normalization in `openRepoInTab` (App.svelte lines 110-132) |
| TREE-08 | 48-02-PLAN.md | Stage/unstage entire directory via action on directory node | SATISFIED | `stageDirectory`/`unstageDirectory` in StagingPanel.svelte; `ondirectoryaction` threaded through TreeFileList to DirectoryRow |
| TREE-09 | 48-02-PLAN.md | Directory nodes show file count badges | SATISFIED | `countFiles` exported from build-tree.ts; used in DirectoryRow `$derived`; rendered with `var(--color-text-muted)` and `font-size: 11px` |
| TREE-10 | 48-02-PLAN.md | Expand All / Collapse All buttons in file list header | SATISFIED | ChevronsUpDown/ChevronsDownUp buttons in StagingPanel.svelte under `{#if treeViewEnabled}`; signal effects in TreeFileList.svelte |

No orphaned requirements. All 6 Phase 48 requirements are claimed by plans 48-01 and 48-02.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `StagingPanel.svelte` | 872, 886, 972, 986 | `placeholder=` | Info | HTML textarea placeholder attributes for commit message inputs — not stub code. Pre-existing, not introduced by Phase 48. |

No blockers or warnings found in Phase 48 modified files.

### Human Verification Required

**48-03-PLAN.md is the blocking human checkpoint for this phase.** The following items require running the app:

#### 1. Tab Context Menu (TAB-08)

**Test:** Open two repos in separate tabs. Right-click a tab.
**Expected:** Native OS context menu appears with "Close Others" (disabled if one tab), "Close All", separator, "Copy Path" (disabled for empty tabs). Each action performs correctly.
**Why human:** Tauri native menu popup cannot be inspected via static analysis.

#### 2. Middle-Click Close (TAB-09)

**Test:** Open two tabs, middle-click one.
**Expected:** Tab closes identically to clicking the X button.
**Why human:** Mouse button === 1 events cannot be simulated without a running app.

#### 3. Duplicate Tab Detection (TAB-10)

**Test:** Open repo in tab 1. Cmd+T for new tab. Open the same repo from the welcome screen.
**Expected:** No new tab. Tab 1 becomes active. The transient empty tab is removed.
**Why human:** Tab state transitions require app interaction.

#### 4. Directory Count Badges (TREE-09)

**Test:** In staging panel, switch to tree view. Observe directory nodes.
**Expected:** Nodes show "(N)" in muted text reflecting recursive file count. Visible when expanded and collapsed.
**Why human:** Visual rendering requires running app with a real git repo.

#### 5. Directory Stage/Unstage Action (TREE-08)

**Test:** In tree view, hover a directory in the unstaged section. Click "+".
**Expected:** All files under that directory are staged. Hover "-" on a staged directory unstages all its files.
**Why human:** File staging state changes require a running git repo.

#### 6. Expand All / Collapse All (TREE-10)

**Test:** In tree view, verify two icon buttons appear beside the tree/list toggle. Click each.
**Expected:** Expand All opens all directory nodes in all sections simultaneously. Collapse All closes them. Buttons are hidden in flat list mode.
**Why human:** Multi-section expand/collapse state requires visual inspection.

### Gaps Summary

No automated gaps found. All 6 artifacts exist, are substantive (not stubs), are fully wired to their consumers, and data flows from real git status through the component chain.

The only remaining gate is the human checkpoint (48-03-PLAN.md), which is by design — Phase 48 intentionally deferred visual and interactive verification to a blocking human step. All automated evidence points to a complete, correct implementation.

**Commits verified:**
- `e28396e` — feat(48-01): TabBar context menu and middle-click events
- `1d500a4` — feat(48-01): App.svelte tab context menu, duplicate detection
- `9761995` — feat(48-02): Directory count badges, hover action buttons, expand/collapse signals
- `198a2a1` — feat(48-02): Directory staging logic, Expand All / Collapse All buttons

**Type checking:** No errors in Phase 48 modified files. Pre-existing errors in unrelated files (RebaseEditor.svelte: sortablejs missing type declarations) — not introduced by this phase.

**Unit tests:** 170/170 pass (14 test files, including build-tree.ts tests).

---

_Verified: 2026-03-24T23:30:00Z_
_Verifier: Claude (gsd-verifier)_
