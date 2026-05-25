# Feature Research

**Domain:** Local single-user code-review session tool that renders one markdown document framed for an AI coding agent to act on (v0.13 Code Review Mode for Trunk)
**Researched:** 2026-05-25
**Confidence:** HIGH on the markdown-for-AI format (convergent evidence from real artifacts: openai/codex review prompt, reviewprompt tool, multiple practitioner blogs); MEDIUM on session-lifecycle UX details (extrapolated from PR-tool patterns + project decisions).

## Framing That Drives Everything

The recipient of the output is **an AI coding agent**, not a human author. This inverts several conventions of human PR-review tooling:

- **Comment phrasing replaces severity tags.** A human reviewer writes "nit:" or "blocker:" so the author can triage. An AI agent acts on whatever instruction it is given. The signal lives in the verb — "Change X to Y" vs "Consider whether..." — not in a metadata label. Therefore: no severity field; invest in instruction-style phrasing instead.
- **Code excerpts are grounding anchors, not courtesy context.** Practitioner consensus: an agent given a diff/excerpt "behaves like a reviewer" and edits the right lines; given vague references it hallucinates. The excerpt is load-bearing.
- **Location must be machine-locatable AND human-trustworthy.** `path:Lstart-Lend` in a heading is the dominant real-world convention (reviewprompt, raf.xyz) precisely because agents reliably parse it.
- **One-shot static document.** No threading, no resolve/approve state, no posting. The doc is generated, pasted/`@file`-referenced, done.

If this research could be reused for a human-PR-review tool with minor edits, it would be wrong. The format section below is specific to AI consumption.

## Feature Landscape

### Table Stakes (Required for a usable v0.13)

These are non-negotiable. Most are already pinned by PROJECT.md key decisions; restated here as scope.

| Feature | Why Expected | Complexity | Notes / Dependency |
|---------|--------------|------------|--------------------|
| Start/resume one review session per repo, persisted across restarts until render | Core lifecycle; you build a review incrementally, not in one sitting | MEDIUM | Storage in app data dir keyed by repo (PROJECT decision). Mirror LazyStore pattern. Session = its own state, NOT in `.git/` or working tree |
| Seed session from a commit range (base→tip) | Primary way to scope "what am I reviewing" | LOW–MEDIUM | Reuse commit graph selection; resolve range to commit list via git2 |
| Hand-pick individual commits from the graph into the session | Reviews are often a subset, not a contiguous range | MEDIUM | Reuse existing graph context menus (v0.3/v0.5); add "Add to review" action |
| Attach a comment to a code range selected in the **diff view** | The act of reviewing | MEDIUM | Reuse v0.7 per-line selection. Anchor source = `diff` |
| Attach a comment to a code range selected in **full-file-at-commit view** | Reviewing code not in the diff (surrounding logic) | MEDIUM | Reuse v0.12 full-file view. Anchor source = `full_file` |
| Anchor = (commit, file, line-range, source∈{diff,full_file}) | The data model the whole feature hangs on | MEDIUM | PROJECT decision; renderer branches on `source` for fence type |
| Optional commit-level comment with no code anchor | "This whole commit should be split" — common, real | LOW | Renders under the commit, no excerpt |
| Edit a comment | Drafts evolve | LOW | Single comment per anchor (no threading) |
| Delete a comment | Mistakes / reconsideration | LOW | — |
| Session panel listing all comments | See what you've accumulated; the review's working surface | MEDIUM | Group-by-file ordering recommended (see format section); jump-to-anchor from each row |
| Jump-to-anchor from a comment | Verify/revisit while building the review | MEDIUM | Open the commit's diff or full-file view, scroll to line-range, highlight. Depends on diff/full-file view navigation |
| Render ONE markdown document | The entire point of the feature | MEDIUM–HIGH | Format spec below is the deliverable's core |
| Copy-to-clipboard | Dominant paste-into-AI-chat path | LOW | Tauri clipboard plugin |
| Save-to-file | `@file`-reference path (Cursor/Claude Code/Codex `@file`) | LOW | Tauri dialog save. Filename convention below |
| Graceful render of unresolvable anchors | History/state can make an anchor un-renderable; must not crash | MEDIUM | Treated as a feature, not an error — see "Stale-anchor render behavior" |

### The Markdown-for-AI Output Format (the heart of v0.13)

This is where research budget went. Each recommendation is evidence-backed.

**Document skeleton (recommended):**

````markdown
# Code Review: <repo> — <base>..<tip>

> Review feedback for an AI coding agent. Each item gives a file location, a
> code excerpt, and an instruction. Apply the changes described. Treat
> excerpts as the exact code being referenced.

Reviewing commits:
- <short-sha> <subject>
- <short-sha> <subject>

---

## src/auth/login.ts

### L42-L48  (commit a1b2c3d)

```ts
function login(email) {
  return db.users.find(email);
}
```

Validate `email` before the lookup — reject empty/malformed input and return a
typed error rather than passing it straight to the query.

### L70  (commit a1b2c3d)

Add a test covering the failed-login path; there's currently none.

---

## Commit-level notes

### a1b2c3d "add login endpoint"

Split this commit — the migration and the endpoint are unrelated changes.
````

(The example above uses a four-backtick outer fence so the inner triple-backtick code blocks render intact. The actual emitted document uses normal triple-backtick fences.)

**Format decisions, each resolved with evidence:**

1. **Location in the heading, as `path:Lstart-Lend`** — NOT only in a fence info-string, NOT only as a prose sentence. This is the dominant real-world convention (reviewprompt emits `./path:L45-L50` headings; raf.xyz uses `File: <path> Line: <n>`). Agents parse heading-anchored locations reliably. Use repo-relative paths (`./src/...` or `src/...`), consistent within the doc. (In the skeleton above the path lives on the H2 and the H3 carries `Lrange (sha)`; either the full `path:Lrange` on every H3 or this H2/H3 split is acceptable — pick one and be consistent.)

2. **Include the commit SHA next to each anchor.** This tool reviews specific commits, not a flat working tree — the agent (and the human) needs to know which version of the file the excerpt came from. Short SHA in the heading suffix is enough.

3. **Include a code excerpt for every code-anchored comment, kept SHORT.** Codex and multiple practitioners: keep ranges to 5–10 lines, pick the tightest subrange that pinpoints the issue. Long excerpts dilute grounding. The excerpt is what lets the agent "behave like a reviewer" instead of hallucinating.

4. **Fence type branches on `source` (PROJECT decision, confirmed sensible):**
   - `source == diff` → use a ` ```diff ` fence preserving `+`/`-`/context prefixes. This tells the agent *what changed*, which is the relevant frame when the comment is about a change.
   - `source == full_file` → use a **language fence** (` ```ts `, ` ```rust `, …) inferred from extension. The comment is about code as it exists, not a change; a language fence gives the agent clean grounding context. (Reuse v0.12's existing language detection.)
   - Evidence: diff fences make the agent reason about the change; language fences provide clean context. Matching fence to intent is the correct call.

5. **Phrase comments as actionable instructions, verbatim from the user.** Do NOT auto-transform the user's text. But the UI/docs should *encourage* instruction-style phrasing ("Change X to Y", "Add a guard for…", "Extract this into…") because that is the signal the agent acts on. Source consensus: specific imperative instructions → specific actions; vague text → vague results. This replaces severity tags entirely.

6. **Ordering: group by file, then by line ascending within each file.** Strong real-world convergence (raf.xyz explicit: "group by file, then sort by line within each file"; AI-review tools emit file-grouped tables). Rationale for THIS tool: an agent editing files works file-by-file; a file-grouped doc maps onto its edit loop. Commit-level comments (no code anchor) go in a separate trailing "Commit-level notes" section, ordered by commit.
   - Rejected alternative: group-by-commit. Tempting because the tool is commit-centric, but the agent edits *files*, and the same file across commits would scatter. Keep the commit SHA on each anchor instead.
   - Rejected alternative: anchor-insertion order. Non-deterministic, harder for both human and agent to scan.

7. **Top-of-doc preamble: YES, short.** A 1–3 line preamble stating "this is review feedback for an AI agent; apply the described changes; excerpts are the referenced code" measurably improves agent behavior (preambles "set the tone"; structured headers let the model read hierarchy). Keep it terse — it is instruction, not narrative. Also list the commits under review so the agent knows scope.

8. **A `# Code Review: <repo> — <base>..<tip>` H1 title + `---` separators between items.** Markdown hierarchy is something agents read structurally; clear H2 (file) / H3 (anchor) nesting helps. Separators reduce run-together.

### Differentiators (Distinguishing, scope-permitting)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| GitHub-style ` ```suggestion ` / explicit "replace-with" blocks | Lets the user dictate an EXACT change the agent applies verbatim, not just a description | MEDIUM | Real primitive worth offering as an optional comment mode: "suggest exact replacement." Caveat from OpenAI's edit-format research — successful apply-formats show *both* the code-to-replace and the replacement and avoid relying on line numbers for the apply step. So a suggestion block should carry the original excerpt AND the proposed code, not just "replace L42." Strong differentiator for AI-recipient flow |
| Live markdown preview of the rendered doc before export | "See exactly what the agent will receive" — closes the trust gap | MEDIUM | Reuse a markdown renderer; preview pane in the session panel |
| Per-comment toggle: include/exclude from render | Build a big draft, ship a focused subset | LOW | Cheap, high control |
| Auto-trim excerpt to the tightest subrange around the selection | Enforces the 5–10 line guidance automatically; better grounding | MEDIUM | Optional smart-default; user can override |
| Copy a single comment as a mini-prompt | Iterative one-issue-at-a-time agent loops | LOW | Same renderer, single-item scope |
| Configurable filename pattern / output dir memory | Predictable, repeatable exports | LOW | See filename convention |
| Inline `@file` hint in preamble | If saving to file, tell the user the exact `@<path>` to type in their agent | LOW | Nice ergonomic touch for the save-to-file path |

### Anti-Features (Explicitly avoid)

These exist in human-PR-review tools and are wrong for an AI-recipient, single-user, one-shot tool.

| Feature | Why Requested | Why Problematic Here | Alternative |
|---------|---------------|----------------------|-------------|
| Threaded replies / discussion | PR tools have them | No second party; the recipient is an agent that acts once. Pure overhead | Single comment per anchor (PROJECT decision). Edit it |
| Severity tags (nit/blocker/critical) | Standard reviewer triage signal | An agent acts on instructions, not labels; severity adds a field the recipient ignores | Encode urgency in the verb — "Must change…" vs "Consider…". Phrasing carries the signal |
| Approve / Request-changes / review state machine | Gerrit/GitHub gate merges with it | No merge gate; no approver. State is meaningless for a doc you paste into a chat | Session is "draft until rendered," then done. No states |
| Reviewer assignment / multi-reviewer | Team workflows | Single user, local | N/A — out of scope by product definition |
| "Viewed"/seen toggles, file-level checkboxes | Track reviewer progress through large PRs | Progress tracking for one person reviewing locally is low value vs build cost | If anything, the include/exclude toggle covers the real need |
| Real-time collaboration / shared sessions | Team tools | Single-user desktop app; storage is private app-data | N/A |
| Posting to GitHub/GitLab | "Submit review" muscle memory | Explicitly out of scope (PROJECT decision); the output target is an AI agent, not a forge | Local markdown only — copy + save |
| Re-anchoring comments on history rewrite | Comments drift when commits are amended/rebased | PROJECT decision: sessions assume a stable range; re-anchoring is a deep, bug-prone algorithm with no payoff for a one-shot personal doc | Detect unresolvable anchors at render time and surface them (below) |
| Inferred / auto-rewritten comment text ("improve this for the AI") | Sounds helpful | Silently changing the user's instruction is a trust violation; the user owns the words the agent will obey | Render the user's text verbatim; only *encourage* good phrasing in UI |
| Auto-generated review findings (AI reviews the code for you) | "Let the tool find issues" | Different product (AI reviewer). v0.13 is a human-authored review captured for an AI to *fix* | Out of scope; the human is the reviewer here |

### Stale-Anchor Render Behavior (a feature, not an error path)

PROJECT.md says "render-time surfaces unresolvable anchors gracefully" but does not specify how. Concrete failure modes and recommended treatment (all MEDIUM complexity, mostly in the renderer):

| Failure mode | Detection (via git2 at render time) | Recommended render treatment |
|--------------|-------------------------------------|------------------------------|
| Commit not reachable (rebased/GC'd) | `revparse_single`/`find_commit` fails | Emit the comment under a trailing `## Unresolvable anchors` section with raw anchor info (sha, path, line-range, the comment text). Never drop the user's words |
| File absent at that commit | `tree.get_path` fails | Same section. Include the comment + "file not found at `<sha>`" note |
| Line range out of bounds (file shorter than range at that commit) | Compare range to blob line count | Two options: (a) clamp to file end and render what exists with a "range truncated" note, or (b) move to the unresolvable section. Recommend (a) when partial overlap exists, (b) when zero overlap |
| Binary file at anchor | blob `is_binary()` | Render the comment with "(binary file, no excerpt)" — comment still actionable |

Principle: **never silently drop a comment.** Worst case it lands in an "Unresolvable anchors" section with enough raw context (sha + path + line range + text) that the user — or even the agent — can still act. This protects against the no-re-anchoring decision biting the user invisibly.

### Filename Convention

Recommend: `trunk-review-<repo-name>-<YYYYMMDD-HHMM>.md`

- Predictable + lexically sortable beats clever. Prefix `trunk-review-` so the file is identifiable among others. `<repo-name>` disambiguates multi-repo work. Timestamp avoids clobbering and orders chronologically.
- Default save dir should remember the last-used location (LazyStore), defaulting to the repo root or Downloads.
- This is a one-sentence decision; do not over-engineer a template DSL for v0.13 (that's the differentiator row above, defer).

## Feature Dependencies

```
Review session (per-repo, persisted)
    └──requires──> Anchor model (commit, file, line-range, source)
                       ├──requires──> Diff line-selection (EXISTS, v0.7)
                       └──requires──> Full-file-at-commit view (EXISTS, v0.12)

Session seeding (range + hand-picked commits)
    └──requires──> Commit graph + context menus (EXISTS, v0.3/v0.5)

Comment management (edit/delete/list/jump)
    └──requires──> Session panel UI
    └──requires──> Anchor model

Jump-to-anchor ──requires──> Diff/full-file view navigation (EXISTS, v0.7/v0.12)

Markdown render ──requires──> Anchor model + comments
    └──requires──> Syntax/language detection (EXISTS, v0.12) for language fences
    └──requires──> Stale-anchor resolution (git2 at render time)

Copy-to-clipboard / Save-to-file ──requires──> Markdown render

Suggestion blocks (differentiator) ──enhances──> Markdown render
Live preview (differentiator) ──enhances──> Session panel

Re-anchoring ──CONFLICTS WITH──> "stable range" assumption (excluded by design)
Severity tags ──CONFLICTS WITH──> instruction-phrasing model (excluded by design)
```

### Dependency Notes

- **Anchor model is the keystone.** Everything (selection, listing, jump, render) reads/writes it. Build and lock its schema first; the `source` field is what makes the renderer's diff-vs-language fence branch possible.
- **Reuse is the dominant theme.** Diff line-selection (v0.7), full-file view (v0.12), commit graph context menus (v0.3/v0.5), and language detection (v0.12) all already exist. The new surface area is: session persistence, the session panel, the renderer, and output mechanics. This keeps v0.13 scoped.
- **Renderer + stale-anchor handling are the two genuinely new backend pieces.** Both touch git2 at render time (read blobs/trees at specific commits). Plan them together.

## MVP Definition

### Launch With (v0.13)

- [ ] One persisted review session per repo (start/resume) — core lifecycle
- [ ] Seed from commit range + hand-pick commits — defines scope
- [ ] Comment on diff selection (`source=diff`) — reuses v0.7
- [ ] Comment on full-file selection (`source=full_file`) — reuses v0.12
- [ ] Commit-level comment (no anchor) — common, cheap
- [ ] Edit / delete comment — table stakes
- [ ] Session panel: list, jump-to-anchor — the working surface
- [ ] Markdown render with the format spec above (preamble, file-grouped, line-sorted, location heading, short excerpt, diff/language fence by source, verbatim instruction text) — THE deliverable
- [ ] Graceful unresolvable-anchor section at render — protects no-re-anchor decision
- [ ] Copy-to-clipboard + save-to-file with the filename convention — both export paths

### Add After Validation (v0.13.x or next milestone)

- [ ] Live markdown preview pane — trigger: user wants to verify output before export
- [ ] Suggestion / exact-replacement blocks — trigger: user wants the agent to apply verbatim changes
- [ ] Per-comment include/exclude toggle — trigger: drafts grow beyond what should ship
- [ ] Copy-single-comment-as-prompt — trigger: iterative one-issue agent loops
- [ ] Auto-trim excerpt to tightest subrange — trigger: excerpts feel noisy

### Future Consideration (later)

- [ ] Configurable filename/output template — defer; predictable default suffices
- [ ] Multiple concurrent sessions per repo — defer; PROJECT pins one active session
- [ ] `@file` hint ergonomics in preamble — defer; minor polish

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Anchor model (commit,file,line-range,source) | HIGH | MEDIUM | P1 |
| Markdown render (format spec) | HIGH | MEDIUM–HIGH | P1 |
| Comment on diff + full-file selection | HIGH | MEDIUM | P1 |
| Session persistence (per repo) | HIGH | MEDIUM | P1 |
| Seed from range + hand-pick commits | HIGH | MEDIUM | P1 |
| Session panel (list + jump) | HIGH | MEDIUM | P1 |
| Edit/delete + commit-level comment | MEDIUM | LOW | P1 |
| Copy + save export | HIGH | LOW | P1 |
| Unresolvable-anchor handling | MEDIUM | MEDIUM | P1 (correctness/trust) |
| Live preview | MEDIUM | MEDIUM | P2 |
| Suggestion/exact-replacement blocks | MEDIUM–HIGH | MEDIUM | P2 |
| Include/exclude toggle | MEDIUM | LOW | P2 |
| Copy-single-as-prompt | LOW–MEDIUM | LOW | P3 |
| Filename template DSL | LOW | LOW | P3 |

## Competitor / Comparable Analysis

Pulled only the *session-lifecycle* idea from PR tools; their multi-reviewer machinery is irrelevant to a single-user AI-recipient tool.

| Aspect | GitHub PR review | Gerrit / Reviewable / CodeStream | reviewprompt / Codex review / practitioner prompts | Our approach (v0.13) |
|--------|------------------|----------------------------------|----------------------------------------------------|----------------------|
| Session model | Draft comments → batched "Submit review" | Draft → publish; change-centric | One-shot: collect → render markdown | One persisted draft session per repo → render once (static) |
| Comment anchor | file + line on a diff | file + line, re-anchors across patchsets | `path:Lstart-Lend` heading + excerpt | (commit, file, line-range, source) → `path:Lrange (sha)` heading + fenced excerpt |
| Severity | labels/prefixes | labels, scores (+2/-1) | instruction phrasing | instruction phrasing (no severity field) |
| Stale anchors | re-anchors / marks outdated | re-anchors across patchsets | N/A (one-shot) | NO re-anchor; render-time "Unresolvable anchors" section |
| Output | posted to forge | posted to forge | markdown for AI agent | markdown for AI agent (copy + save), never posted |
| Exact-change primitive | ` ```suggestion ` blocks | inline edit | "show code-to-replace + replacement" | optional suggestion block carrying excerpt + proposed code (differentiator) |
| Code presentation | diff in UI | diff in UI | diff-fence behaves like reviewer; short excerpts ground the agent | diff-fence for `diff` source, language-fence for `full_file` source; short excerpts |

## Sources

- [openai/codex review_prompt.md](https://github.com/openai/codex/blob/main/codex-rs/core/review_prompt.md) — HIGH. Structured finding format: imperative title ≤80 chars, markdown body citing files/lines, `absolute_file_path` + `line_range {start,end}`, "line ranges as short as possible (avoid >5–10 lines, pick tightest subrange)."
- [dyoshikawa/reviewprompt](https://github.com/dyoshikawa/reviewprompt) — HIGH. Tool that converts review comments into AI-agent prompts. Emits `./path:Lstart-Lend` heading + comment text. Directly analogous use case.
- [How I keep up with AI-generated PRs — raf.xyz](https://www.raf.xyz/blog/03-how-i-keep-up-with-ai-generated-prs) — MEDIUM/HIGH. "Group by file, then sort by line within each file." Casual lowercase imperative phrasing. Short review body. Reviewing AI-generated code specifically.
- [AI PR Review Prompt gist (shamashel)](https://gist.github.com/shamashel/7401403a1c663bb42777061cf49a3991) — MEDIUM. Nested bullet format: File `<path>:<line-range>` / Issue / Fix.
- [Cursor + Claude AI code review checklist — dev.to](https://dev.to/sathish_daggula/cursor-claude-my-ai-code-review-checklist-hm5) — MEDIUM. "Paste a diff → behaves like a reviewer; paste a full file → hallucinates context." Output grouped by risk with line refs + minimal patch.
- [Code Surgery: how AI assistants make precise edits — fabianhertwig.com](https://fabianhertwig.com/blog/coding-assistants-file-edits/) — MEDIUM. OpenAI edit-format findings: successful apply formats show both code-to-replace AND replacement, avoid relying on line numbers for the apply step; context-matching with whitespace-trim fallbacks.
- [Spec-driven development / markdown for agents — github.blog](https://github.blog/ai-and-ml/generative-ai/spec-driven-development-using-markdown-as-a-programming-language-when-building-with-ai/) and [Markdown is the new source code — hartleybrody.com](https://blog.hartleybrody.com/markdown-research-planning/) — MEDIUM. Agents read markdown hierarchy structurally; fenced blocks are treated as code; preambles set tone.

---
*Feature research for: local AI-recipient code-review session tool (Trunk v0.13)*
*Researched: 2026-05-25*
