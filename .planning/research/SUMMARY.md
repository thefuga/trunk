# Research Summary: Trunk v0.7

**Milestone:** v0.7 Hunk Staging & Search
**Researched:** 2026-03-17

## Key Findings

### Stack
- **Zero new dependencies** — both features are fully implementable with the existing git2 0.19 + Svelte 5 + Tauri 2 stack. No new crates or npm packages needed.
- **git2's `Repository::apply()` with `ApplyOptions::hunk_callback`** is the core API for hunk staging. It supports `ApplyLocation::Index` for staging-area-only modifications, which is the exact equivalent of `git add -p`.
- **Commit search is client-feasible but backend-recommended** — all commit data already lives in `CommitCache` on the Rust side. Backend search avoids pagination gaps (frontend only loads 200 commits per batch) and avoids serializing all commits to JS.

### Features
- **Table stakes for hunk staging:** per-hunk stage/unstage buttons on `@@ @@` headers, discard hunk, visual hunk boundaries, and diff refresh after each operation. Every major GUI (Sublime Merge, VS Code, Fork, GitKraken) has these.
- **Table stakes for search:** Cmd+F shortcut, floating overlay bar (not modal), search by SHA/message/branch, highlight-in-place with prev/next navigation, match count display, Escape to close. Highlight-within-graph (not filter) is the right approach for v0.7.
- **Line-level staging** is the biggest differentiator approaching table-stakes for premium GUIs — but HIGH complexity. Defer to v0.8+. Split diff view similarly deferred.

### Architecture
- **Two parallel implementation tracks:** hunk staging (backend → UI) and search (backend → UI) are fully independent. Phases 1-2 (hunk) and 3-4 (search) can run in parallel.
- **Backend search over `CommitCache` is strictly better** than client-side — Stack research suggested client-side filtering, but Architecture and Pitfalls research identified that the frontend only has paginated batches (not all commits), making backend search necessary for completeness. This is the key disagreement resolved in favor of backend.
- **Existing `scrollToOid(oid)` already handles lazy-load + smooth scroll** — search navigation reuses this directly, making search UI integration low-risk.

### Top Risks
1. **Hunk boundaries invalidate after each staging operation (P2)** — after staging hunk N, all subsequent hunk indices are stale. Every stage/unstage MUST be followed by a full re-diff before the next operation. Never batch hunks with pre-computed line numbers. *Prevention:* atomic single-hunk operations + mandatory diff re-fetch + disable buttons during in-flight ops.
2. **Cmd+F conflicts with macOS WKWebView native find bar (P7)** — `preventDefault()` may not suppress the native find. *Prevention:* test early; fallback to Rust-level event interception or alternate shortcut (Cmd+G / Cmd+K).
3. **Index changes don't trigger filesystem watcher (P11)** — staging modifies `.git/index`, not workdir files. The `repo-changed` event won't fire. *Prevention:* explicit frontend refresh of diffs + status + dirty counts after every staging command return.
4. **Binary files and "no newline at EOF" edge cases (P3)** — binary files have no hunks; `\No newline at end of file` marker requires special handling in blob construction. *Prevention:* guard with `is_binary` check, track NOEOFNL origin in patch builder, test both cases.
5. **Index lock contention with filesystem watcher (P5)** — hunk staging holds `.git/index.lock` longer than whole-file staging. Concurrent `get_status` calls can cause `ELOCKED` errors. *Prevention:* catch + retry, or gate frontend auto-refresh during in-flight staging operations.

## Recommendations

### Hunk Staging Approach
Use `git2::Diff::from_buffer()` + `repo.apply(&diff, ApplyLocation::Index)` (Approach A from Architecture). Build an `extract_hunk_patch` helper that constructs a valid unified diff for a single hunk, then apply it to the index. For unstaging, build a reversed patch. This is pure Rust, testable via the inner-fn pattern, and consistent with all existing local git operations using git2. Avoid CLI subprocess (`git apply --cached`) — it would introduce an inconsistent pattern. For new/untracked files with a single hunk, fall back to `Index::add_path()`.

### Search Approach
Implement as a **backend `search_commits` command** reading from `CommitCache`, returning `Vec<SearchResult>` (OID + match type). This searches all commits regardless of frontend pagination state. For SHA prefixes, check `oid.starts_with()`; for messages, case-insensitive substring on `summary`/`body`; for refs, check `ref.short_name`. Frontend: floating `SearchBar.svelte` inside CommitGraph, debounced input (200ms), result navigation via existing `scrollToOid`. Highlight matching rows via `searchMatches: Set<string>` prop on CommitRow (same pattern as existing `selected` prop).

### Build Order
1. **Phase 1: Hunk Staging Backend** — highest risk (git2 `apply` API + patch construction edge cases). Validate early. Unit test with multi-hunk files, binary files, new files, no-newline-at-EOF.
2. **Phase 2: Hunk Staging UI** — DiffPanel `diffKind` prop + hunk action buttons + App.svelte wiring + refresh flow. Depends on Phase 1.
3. **Phase 3: Search Backend** — low risk (string matching on cached data). Can run in parallel with Phases 1-2. Benchmark with 10k+ commit repo.
4. **Phase 4: Search UI** — SearchBar component + Cmd+F handler + highlight integration. Test Cmd+F/WebView conflict early. Depends on Phase 3.

Phases 1-2 and 3-4 are independent tracks. Hunk staging first because it's higher-value and higher-risk.

## Feature Scope Summary

### In Scope (v0.7)
| Feature | Category | Complexity |
|---------|----------|------------|
| Per-hunk stage button on `@@` headers | Table Stakes | LOW (frontend) |
| Per-hunk unstage button on `@@` headers | Table Stakes | LOW (frontend) |
| `stage_hunk` backend command (git2 apply) | Table Stakes | MEDIUM (backend) |
| `unstage_hunk` backend command (reversed patch) | Table Stakes | MEDIUM (backend) |
| Discard hunk (revert workdir changes) | Table Stakes | MEDIUM (backend) |
| Binary file guard (disable hunk buttons) | Table Stakes | LOW |
| Diff refresh after hunk operation | Table Stakes | LOW-MEDIUM |
| Cmd+F search overlay bar | Table Stakes | MEDIUM (frontend) |
| Search by SHA prefix | Table Stakes | LOW (backend) |
| Search by commit message (case-insensitive) | Table Stakes | LOW (backend) |
| Search by branch/ref name | Table Stakes | LOW (backend) |
| Highlight matching rows in graph | Table Stakes | LOW (frontend) |
| Prev/Next result navigation | Table Stakes | LOW (frontend) |
| Match count display ("3 of 17") | Table Stakes | LOW |
| Escape to close search | Table Stakes | LOW |
| Search by author name | Differentiator | LOW |

### Deferred
| Feature | Reason |
|---------|--------|
| Line-level staging | HIGH complexity — requires selectable diff lines, partial patch computation. Approaching table stakes but not blocking v0.7. Target v0.8. |
| Split (side-by-side) diff view | MEDIUM-HIGH complexity — different rendering mode, line mapping. Pairs with line staging. Target v0.8+. |
| Context expansion (show more lines around hunks) | MEDIUM complexity — requires backend re-diff with different context count. Polish feature. |
| Hunk keyboard navigation (`[`/`]` between hunks) | LOW complexity but not blocking. Add during polish. |
| Scoped search prefixes (`author:`, `sha:`, `branch:`) | LOW-MEDIUM complexity. Nice power-user feature for future. |
| Regex search toggle | LOW complexity but limited value for v0.7. |
| Persist search state across panel switches | LOW complexity. Polish. |

### Out of Scope
| Feature | Reason |
|---------|--------|
| Graph filtering (hide non-matching commits) | Extremely complex — breaks virtual list + SVG overlay + lane algorithm. Years of work in Sublime Merge/Fork. |
| Fuzzy search (fzf-style) | Commit messages and SHAs are not fuzzy targets. Substring match is what every tool uses. Would produce confusing results. |
| Inline editing in diff view | Separate complex feature. Mixing editing with staging creates ambiguous state. |
| Explicit "Split Hunk" button | No GUI tool has this. Line-level staging solves the same need. |
| Search by file path (`path:` scope) | Requires walking commit trees — expensive. Not a core search use case. |
| Saved searches / search history | Over-engineering for v0.7. |
| Cross-panel search | Each panel should have its own search. Commit graph search only for v0.7. |
| Drag-to-expand context lines | Sublime Merge feature — elegant but complex. Defer entirely. |

---
*Synthesized from: STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md*
*Research date: 2026-03-17*
