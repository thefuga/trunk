---
phase: 72-review-pane-ux-integration
milestone: v0.13 Code Review Mode
status: design-locked
created: 2026-05-26
origin:
  - .planning/todos/pending/2026-05-26-relocate-copy-action-off-preview-pane.md
  - .planning/todos/pending/2026-05-26-review-pane-navigation-and-dead-review-button.md
gaps_closed: [G-71-A, G-71-B]
depends_on:
  - Phase 71 (Copy mechanism — patterns carry forward; preview pane is deleted)
  - Phase 70 (preview-pane infrastructure is being walked back; markdown generation IPC is kept)
  - Phase 65 (clipboard-manager:allow-write-text capability — already granted, unchanged)
---

# Phase 72: Review-Pane UX Integration

**Goal:** Reviewing is a first-class, in-window mode — enter and exit it from a persistent toolbar button (or keyboard shortcut), copy the generated markdown directly from the comments view with one click, and remove the leftover UI from the preview-pane detour.

This phase walks back the markdown preview surface introduced in Phase 70 (panel-internal swap to `ReviewDocPreview`) and the temporary "Review" toggle strip introduced in Phase 65/69, replacing both with a tighter integration into the surrounding application chrome.

## Why this phase exists

Phase 71's UAT (`.planning/phases/71-output-clipboard-save-to-file/71-UAT.md`) closed as validated for the clipboard mechanism itself, but surfaced two design-level gaps the user explicitly wants to address as a follow-up phase:

- **G-71-A:** Copy lives on the markdown preview pane (`ReviewDocPreview.svelte`), forcing the user through a Generate → preview → Copy path. The user wants Copy directly on the comments view, no preview detour.
- **G-71-B:** Entering review mode requires the native macOS menu item "View → Start/End Code Review" (no keyboard shortcut, no in-window UI). Inside review mode, a blue "Review" button sits in a header strip above the panel/diff content (`RepoView.svelte:815-827`) that appears clickable but is a no-op when already on the panel (the default state). User feedback verbatim: *"That menu option is too clunky, and for some reason we have a blue review button in the review pane that doesn't do anything."*

Solving both at once is right because they share a single design question: *how does review mode integrate with the surrounding UI?*

## Root cause read (engineering_judgment.md §1)

The three issues are surface symptoms of one structural problem: **review mode has three orthogonal state axes** (`reviewActive` × `rightPaneMode: panel|diff` × `panelMode: list|preview`) all gated by a single hidden entry point (the menu). The UI doesn't make any of these axes legible:

- `reviewActive` has no in-window indicator → "is review on?" is invisible.
- `rightPaneMode` has the blue button as an attempted breadcrumb, but it doubles as an action button while only being functional in one sub-state, so it reads as broken.
- `panelMode` exists *because* Phase 70 needed somewhere to host the preview, but the preview is itself the unnecessary detour.

Collapsing the state machine (drop `panelMode`) and externalizing the remaining axes to clearer affordances (toolbar button for `reviewActive`, panel-vs-diff handled by each component's own close affordance) solves all three reported symptoms at the root.

## Design

### Shape

Pull review-mode entry into the in-window toolbar, fold the markdown preview surface entirely into a single Copy action on the comments view, and delete the redundant blue-button header strip.

### State-machine simplification

Before:

```
reviewActive: boolean
rightPaneMode: "panel" | "diff"
panelMode: "list" | "preview"     ← removed
previewMarkdown: string | null     ← removed
```

After:

```
reviewActive: boolean
rightPaneMode: "panel" | "diff"
```

`generate(repoPath)` becomes a pure async function returning the markdown string. The caller (Copy handler in ReviewPanel) consumes the return value directly and writes it to the clipboard — no state caching, no panelMode swap.

### Component changes

| Component | Change |
|-----------|--------|
| `src/components/Toolbar.svelte` | **Modify** — add a new `.toolbar-btn` for "Review" with active-state styling (when `reviewActive === true`). Icon: `MessagesSquare` from `@lucide/svelte` (matches comment-review semantics). On click: emits `review-toggle` via the Tauri event bus so behavior matches the menu + shortcut path. Lives in a new `.toolbar-group` appended after the existing Branch/Stash/Pop group (currently the rightmost group; new Review group becomes the new right edge). |
| `src/components/ReviewPanel.svelte` | **Modify** — rename header "Generate" button to "Copy" (icon: `Clipboard`). Click handler: `await session.generate(repoPath)` → `await writeText(markdown)` → flip `copied` state for 1500ms. On error in either step: `showToast(\`Failed to copy: ${msg}\`, "error")`. Remove all `panelMode === "preview"` rendering (the early-return branch at lines 332-339). |
| `src/components/ReviewDocPreview.svelte` | **Delete entirely.** Including `ReviewDocPreview.test.ts`. |
| `src/components/RepoView.svelte` | **Modify** — delete lines 813-828 (the header strip + blue button). Inside `{#if reviewSession.state.reviewActive}` the body renders `ReviewPanel` or `DiffPanel` directly with no surrounding strip. |
| `src/lib/review-session.svelte.ts` | **Simplify** — remove `panelMode: PanelMode`, `previewMarkdown`, `showList()`, `showPreview()`. Adjust `generate(repoPath)` to *return* the markdown string rather than mutate state. `setReviewActive(false)` no longer needs to clear the preview-related fields. |
| `src-tauri/src/lib.rs` | **Modify** — add `.accelerator("CmdOrCtrl+Shift+R")` to the `review-toggle` MenuItemBuilder (line 28). Direct mirror of how `find` got `CmdOrCtrl+F` at line 22. |

### Data flow (the new Copy path)

```
User clicks "Copy" in ReviewPanel header
  → await session.generate(repoPath)
     [IPC: generate_review_doc → markdown: string]
  → await writeText(markdown)
     [@tauri-apps/plugin-clipboard-manager]
  → copied = true; setTimeout(reset, 1500); button shows "✓ Copied"
  → on any throw: showToast(`Failed to copy: ${msg}`, "error")
```

No state caching between clicks — each Copy click re-generates. The IPC is local, comment-count-bounded, and cheap; YAGNI says don't cache until measurement shows a problem.

### Error handling

Single try/catch wrapping the full `generate → writeText` sequence. `instanceof Error` narrowing in catch (carry-forward from Phase 71). Either step failing → error toast + button stays in "Copy" state (no half-success affordance).

### Testing

| Test | What it verifies |
|------|------------------|
| `ReviewPanel.test.ts` (extend) | New: Copy click invokes `generate` + `writeText` with returned markdown; ✓ Copied affordance; 1500ms timer; `clearTimeout` before `setTimeout` on re-click; `instanceof Error` narrowing in catch; non-`Error` rejection coercion via `String(e)`. Pattern carry-forward from `ReviewDocPreview.test.ts`. |
| `Toolbar.test.ts` (extend) | New: Review button renders; click emits `review-toggle` event; active-state styling when `reviewActive === true` (mock the rune or the prop). |
| `ReviewDocPreview.test.ts` | **Delete** (component deleted). |
| `review-session.svelte.ts` tests (if present) | Update for the simplified state machine — drop `panelMode`/`previewMarkdown` assertions; verify `generate` returns the string rather than mutating state. |
| `src-tauri/` | No new automated tests — the accelerator addition is one-line menu config. Manual UAT: hit Cmd+Shift+R, confirm toggle. |

### Carry-forward patterns from Phase 71

- 1500ms ✓ Copied affordance — preserved, just on `ReviewPanel.svelte` header
- `clearTimeout` before `setTimeout` for re-clickable affordance — preserved
- Awaited `writeText` with try/catch + `showToast` — preserved (intentional divergence from fire-and-forget callsites elsewhere in the app remains the house style for product-artifact copies)
- `instanceof Error` narrowing (not `as` casts) — preserved
- `vi.useFakeTimers` test pattern — moves into `ReviewPanel.test.ts`'s new Copy block

## Open implementation choices (design-doc punts to plan)

1. **Exact Lucide icon** — `MessagesSquare` proposed (multi-comment, matches review semantics). Alternatives: `MessageSquareText`, `BookOpenCheck`, `ClipboardList`. All four verified present in `@lucide/svelte@0.577.0` (`node_modules/@lucide/svelte/dist/icons/index.js` lines 241/445/1038/1042) — pick during implementation by visual fit.
2. **Active-state styling for Toolbar's Review button** — Toolbar has no precedent for toggle buttons. Proposed: `background: var(--color-accent); color: var(--color-bg);` when active. Validate against theme tokens and contrast in both light/dark mode during implementation.
3. **Toolbar position** — proposed: a new `.toolbar-group` on the right edge after the existing Branch/Stash/Pop group. Confirm visual fit and that it doesn't crowd the right edge with window controls.

## Threat model

| ID | Threat | Disposition |
|----|--------|-------------|
| T-72-S | New principal / auth path | N/A — no new identity, no new IPC handler, no new capability |
| T-72-T | Tamper with IPC / persisted state | N/A — `generate_review_doc` and `writeText` are unchanged; only their composition changes |
| T-72-R | New logging surface | N/A — error toast text is unchanged from Phase 71 (`Failed to copy: ${msg}`) |
| T-72-I | Info disclosure via UI / logs | accept (LOW) — same error toast surface as 71, no new disclosure |
| T-72-D | Denial via runaway timer / IPC | N/A — `clearTimeout` before each new `setTimeout` (carry-forward from 71); generation IPC is bounded by session comment count |
| T-72-E | Privilege escalation | N/A — no new capability, no new tauri command |
| T-72-SC | Supply chain | N/A — zero new packages |

## Net effect

| Metric | Change |
|--------|--------|
| Files deleted | 2 (`ReviewDocPreview.svelte`, `ReviewDocPreview.test.ts`) |
| Files modified | 5 (`Toolbar.svelte`, `ReviewPanel.svelte`, `RepoView.svelte`, `review-session.svelte.ts`, `src-tauri/src/lib.rs`) |
| Files added | 0 |
| State-machine axes | 3 → 2 (`panelMode` dropped) |
| User clicks to copy | 4 (enter via menu → Generate → wait for preview → Copy) → 2 (enter via toolbar/shortcut → Copy) |
| Dead UI removed | 1 (blue Review button + its host strip) |

## Success criteria

What must be TRUE after this phase:

1. User can toggle review mode from the in-window Toolbar (click) or by pressing Cmd+Shift+R, in addition to the existing macOS menu item.
2. The Toolbar's Review button shows an active state when review is on, so the user can see review state at a glance without entering it.
3. From the comments view, a single click on the Copy button generates the review doc and writes it to the OS clipboard, with the same ✓ Copied / error-toast affordances as Phase 71.
4. The markdown preview pane no longer exists — `ReviewDocPreview.svelte` is deleted; `panelMode` is removed from the review-session rune; the panel never swaps to a preview face.
5. The blue Review button at `RepoView.svelte:815-827` and its host header strip are removed. Returning from a jumped-to diff back to the panel is handled by `DiffPanel`'s existing close affordance (already wired to `reviewSession.showPanel()` at `RepoView.svelte:839`).
6. `just check` exits 0 with all updated/new tests passing.

---

*Design locked: 2026-05-26 via brainstorming conversation (transcript: this session)*
*Ready for: `/gsd:plan-phase 72`*
