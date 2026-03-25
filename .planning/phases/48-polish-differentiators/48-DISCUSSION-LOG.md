# Phase 48: Polish & Differentiators - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-24
**Phase:** 48-polish-differentiators
**Areas discussed:** Tab context menu, Middle-click behavior, Duplicate detection, Directory staging, Count badges, Expand All/Collapse All
**Mode:** Auto (--auto flag, all recommended defaults selected)

---

## Tab Context Menu

| Option | Description | Selected |
|--------|-------------|----------|
| Standard trio (Close Others, Close All, Copy Path) | Industry-standard tab context menu actions | ✓ |
| Extended menu (+ Pin, Move Left/Right, Reopen Closed) | Additional power-user actions | |

**User's choice:** [auto] Standard trio — Close Others, Close All, Copy Path
**Notes:** Keep minimal for v0.9. Close Others closes all except right-clicked tab. Close All opens new empty tab. Copy Path copies absolute filesystem path.

---

## Middle-click Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Graceful close (same as X button) | Consistent with Phase 45 D-10, lets running ops finish | ✓ |
| Force close (cancel running ops) | Faster but inconsistent with X button behavior | |

**User's choice:** [auto] Graceful close (recommended, consistent with Phase 45 D-10)
**Notes:** Middle-click is just a faster way to click X — same behavior expected.

---

## Duplicate Detection

| Option | Description | Selected |
|--------|-------------|----------|
| Silent switch to existing tab | Detect by normalized path, switch silently | ✓ |
| Switch with toast notification | Same detection but show "Switched to existing tab" toast | |
| Allow duplicates | No detection, user manages themselves | |

**User's choice:** [auto] Silent switch to existing tab
**Notes:** Tab bar visually confirms the switch. Transient empty "New Tab" that triggered the open gets closed.

---

## Directory Staging

| Option | Description | Selected |
|--------|-------------|----------|
| Frontend loop (call stage_file per file) | No backend changes, loop over matching paths | ✓ |
| New backend command (stage_directory) | Single IPC call, git2 bulk add by path prefix | |

**User's choice:** [auto] Frontend loop — simpler, no Rust changes needed
**Notes:** Hover action button matching FileRow pattern. Section-contextual label (Stage/Unstage).

---

## Count Badges

| Option | Description | Selected |
|--------|-------------|----------|
| Inline muted text "(3)" after name | VS Code style, minimal | ✓ |
| Pill badge with background | More prominent, takes more space | |
| Superscript number | Compact but harder to read | |

**User's choice:** [auto] Inline muted text — recursive file count, visible in both states
**Notes:** Uses --color-text-muted. Shows recursive count (all files under directory, not just direct children).

---

## Expand All / Collapse All

| Option | Description | Selected |
|--------|-------------|----------|
| Two icon buttons next to tree toggle | Clear, always available in tree mode | ✓ |
| Single toggle button | Ambiguous state representation | |
| Keyboard shortcuts only | Hidden discoverability | |

**User's choice:** [auto] Two icon buttons in staging panel header, visible only in tree mode
**Notes:** Affects all sections (unstaged, staged, conflicted) simultaneously. Lucide chevron icons.

---

## Claude's Discretion

- Native Tauri menu API vs custom context menu (native recommended)
- Sequential vs batched directory staging calls
- Exact Lucide icon choices for Expand All / Collapse All
- Propagation mechanism for Expand All / Collapse All to TreeFileList instances
