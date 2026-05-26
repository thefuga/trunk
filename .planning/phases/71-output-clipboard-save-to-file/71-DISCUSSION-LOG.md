# Phase 71: Output (Clipboard) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-26
**Phase:** 71-Output (Clipboard) — descoped from "Output (Clipboard + Save-to-File)" mid-discussion
**Areas discussed:** Button design, Save dialog defaults (descoped), Keyboard shortcuts, Toast feedback + edge cases

---

## Button design

### Q1: How should Copy and Save be presented in the preview header?

| Option | Description | Selected |
|--------|-------------|----------|
| Two icon+label buttons | Side-by-side `📋 Copy` and `💾 Save…`, most discoverable | ✓ |
| Two icon-only buttons | Just icons with tooltips; minimal | |
| Split / dropdown | Primary Copy with chevron for Save | |

**User's choice:** Two icon+label buttons. (Later collapsed to a single Copy button after the save-to-file descope.)

### Q2: Which is the primary action?

| Option | Description | Selected |
|--------|-------------|----------|
| Copy is primary | Lower-friction, higher-frequency (paste into AI) | ✓ |
| Save is primary | Durable artifact | |
| Equal weight | Peers, no visual primary | |

**User's choice:** Copy is primary. (Moot after descope; only Copy remains.)

### Q3: Should the buttons match the existing back-button style or stand out?

| Option | Description | Selected |
|--------|-------------|----------|
| Match back-button | Border + transparent bg + hover, calm | ✓ |
| Primary filled style | Filled accent color, draws the eye | |

**User's choice:** Match back-button.

### Q4: Which icon library?

| Option | Description | Selected |
|--------|-------------|----------|
| @lucide/svelte | Already used throughout | ✓ |
| No icons — text only | Just labels | |

**User's choice:** Lucide.

---

## Save dialog defaults

### Q1: Default directory?

| Option | Description | Selected |
|--------|-------------|----------|
| OS default (Downloads) | Don't set defaultPath, OS remembers | |
| Repo parent directory | `dirname(repoPath)` | ✓ |
| Home directory | `~` | |

**User's choice:** Repo parent directory.

### Q2: How to derive `<repo-name>`?

| Option | Description | Selected |
|--------|-------------|----------|
| basename + lowercased + slugged | FS-safe everywhere | |
| basename as-is | Preserves casing; risks weird chars | |
| Strip trailing `.git` too | Slugged AND strip `.git` | |
| Free-text | "Don't include repo name — `trunk-review-<YYYYMMDD-HHMM>.md` (storing in repo folder already disambiguates)" | ✓ |

**User's choice:** Free-text — drop `<repo-name>` entirely; filename is `trunk-review-<YYYYMMDD-HHMM>.md`.

### Q3: Local time or UTC?

| Option | Description | Selected |
|--------|-------------|----------|
| Local time | Matches `ls -l` mtime intuition | ✓ |
| UTC | Timezone-stable across machines | |

**User's choice:** Local time.

### Q4: Overwrite handling?

| Option | Description | Selected |
|--------|-------------|----------|
| Trust the dialog | Native dialog handles "Replace?" | ✓ |
| App-level confirm | Re-check exists() + `ask()` | |

**User's choice:** Trust the dialog.

**⚠ All of the above superseded:** Mid-discussion (after the Keyboard shortcuts area), the user decided the save-to-file path was carrying too much accidental complexity (default dir, filename, conflicts, atomic write, permissions, release verification) for marginal value vs. `Cmd+V` into their own editor. Save-to-file (OUT-02) was **dropped entirely**, not deferred. Decisions preserved in CONTEXT.md → Deferred Ideas in case the requirement ever comes back.

---

## Keyboard shortcuts

### First batch (Cmd+S, Cmd+C, scope, tooltips)

Four-question batch presented before the descope. User dismissed all four — they were second-guessing whether the save complexity was worth it, which surfaced the descope decision.

### Post-descope re-asked: Cmd/Ctrl+C to copy the whole doc when nothing is selected?

| Option | Description | Selected |
|--------|-------------|----------|
| Only when no text selection | Selection → native; empty → whole doc | |
| Never — button only | Cmd+C is purely native text selection | ✓ |
| No shortcut at all | Drop the entire area | |

**User's choice:** Never — button only. Cmd+C is reserved for native text selection inside the `<pre>`.

---

## Toast feedback + edge cases

### Q1: Success toast text?

| Option | Description | Selected |
|--------|-------------|----------|
| "Review copied to clipboard" | Specific noun | (chosen, then superseded — see Q3) |
| "Copied to clipboard" | Generic, shorter | |
| "Copied — paste into your AI agent" | Hints at intended use | |

**User's choice:** "Review copied to clipboard" — but superseded when user opted for in-button affordance instead of a success toast (Q3). No success toast will be shown.

### Q2: Failure handling?

| Option | Description | Selected |
|--------|-------------|----------|
| Error toast with reason | `Failed to copy: <error message>` | ✓ |
| Error toast, generic | `Failed to copy to clipboard` | |
| Error toast + fallback modal | Show error AND open modal with raw md | |

**User's choice:** Error toast with reason.

### Q3: Transient "Copied" state on the button?

| Option | Description | Selected |
|--------|-------------|----------|
| No — toast is the only feedback | Simplest | |
| Yes — swap label+icon for ~1.5s | Local affordance | ✓ |

**User's choice:** Yes — and use toast ONLY for the failure case. Success path is button-state-only.

### Q4: Copy enabled when doc is empty?

| Option | Description | Selected |
|--------|-------------|----------|
| Not reachable — ignore | Phase 70 D-11 gates Generate on >0 comments | ✓ |
| Disable button | Belt-and-suspenders | |

**User's choice:** Not reachable — ignore.

---

## Scope change — Save-to-file (OUT-02) descoped

| Option | Description | Selected |
|--------|-------------|----------|
| Drop entirely | Remove from REQUIREMENTS.md | ✓ |
| Move to Future Requirements | Park in `## Future Requirements` section | |

**User's choice:** Drop entirely.

**Triggers follow-up edits** (captured in CONTEXT.md → Deferred Ideas → Follow-up edits):
- `.planning/REQUIREMENTS.md`: remove OUT-02
- `.planning/ROADMAP.md` Phase 71: rename, drop success criteria #2/#3, prune Notes
- Optional: rename phase directory slug

---

## Claude's Discretion

- Exact Lucide icon name (`Clipboard` vs `ClipboardCopy`)
- "Copied" revert duration (~1.5s target, tune if it feels off)
- Promise handling style (`await` + try/catch vs `.then/.catch`)

## Deferred Ideas

- **OUT-02 (Save to file):** dropped from REQUIREMENTS.md. Full design ground (default dir, filename, timezone, overwrite, Rust write, capability, release verification) preserved in CONTEXT.md → Deferred Ideas for if it ever comes back.
