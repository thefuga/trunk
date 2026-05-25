# Phase 66: Commit Selection - Research

**Researched:** 2026-05-25
**Domain:** git2 revwalk / set semantics, Tauri command wiring, Svelte 5 graph-row rendering
**Confidence:** HIGH (all claims verified against the live codebase and the installed git2 0.19 source)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Range is seeded by **two right-clicks on the graph** — right-click a commit → "Set as review base", then right-click another → "Add range to review". Reuses the existing native context menu (`showCommitContextMenu`, `CommitGraph.svelte:567`). No new modal. A **transient base highlight** marks the pending base between the two clicks (cleared after the range is added or the action is cancelled).
- **D-02:** Range is **inclusive of both base and tip** — `[base..tip]`, not `base..tip`. Implement via revwalk `push(tip)` + `hide(base.parent(0))`, with the root-commit fallback (don't hide anything) demonstrated in `interactive_rebase.rs:53`.
- **D-03:** Seeding a range **unions** its commits into the existing selection — never replaces. The session is a set. Dedup keeps it clean (success criterion #4).
- **D-04:** In-session commits are **visually marked in the graph** (left-gutter accent / dot or tinted row). Use a **theme CSS custom property — no inline color** (project rule).
- **D-05:** The panel commit list is **minimal**: short SHA + commit summary per row, rendered in **graph order**, dedup'd. The review panel is Phase 65's throwaway stub (replaced in Phase 69). Do not over-invest.
- **D-06:** The graph context menu carries a **single toggle item**: "Add to review" when the commit is not in the session, "Remove from review" when it is. Covers SEL-02 and SEL-03 from the graph.
- **D-07:** Each row in the panel list also gets an **× / remove button**.
- **D-08:** Merge commits are **selectable like any other commit** (add, range-seed). The diff-source-only-on-non-merges restriction is **deferred to Phase 67** (anchor capture), NOT enforced at selection time. (Contrast: Cherry-pick/Revert ARE disabled for merges at `CommitGraph.svelte:682` — that constraint does NOT apply here.)

### Claude's Discretion (resolved in this research — see the dedicated sections below)
- Invalid-range handling → see **Pitfall 1** + **Code Examples**.
- Selected-but-not-loaded commits → see **Open Question 1** (resolved: it is NOT a pagination problem).
- Command surface & naming → see **Standard Stack / Command Surface**.
- Persistence/event wiring → see **Architecture Patterns / Persistence & Events**.
- The in-session graph marker (D-04) → see **Architecture Patterns / D-04 Marker**.

### Deferred Ideas (OUT OF SCOPE)
- Enforcing diff-source-only-on-non-merges (Phase 67).
- Richer commit list — author, date, etc. (Phase 69).
- No comment content capture happens this phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SEL-01 | Seed a review session from a commit range (base → tip) | `seed_review_range` command: git2 revwalk `push(tip)`+`hide(base.parent(0))` (interactive_rebase.rs:40-56 pattern), union into `commits`, dedup, single `save_session`, single emit. Invalid-range detection via `merge_base`/`graph_descendant_of`. |
| SEL-02 | Add individual commits from the graph context menu | `add_review_commit` command + D-06 toggle item in `showCommitContextMenu`. Read-modify-write under mutex. |
| SEL-03 | Remove a commit from the session | `remove_review_commit` command + D-06 toggle (graph) and D-07 × button (panel). |
| SEL-04 | See the list of commits in the session | `list_session_commits` command: intersect `session.commits` with the FULL cached graph order (`CommitCache`), dedup, return `[{oid, short_oid, summary}]`. Rendered by ReviewPanel (D-05) and consumed as a `Set` by CommitGraph (D-04 marker, D-06 toggle state). |
</phase_requirements>

## Summary

Phase 66 is a thin feature layered on a complete keystone. `ReviewSession.commits: Vec<String>` already exists (`types.rs:333`), the atomic store (`review_store.rs`) and the `_inner(data_dir, …)` + thin-command + `session-changed` emit pattern (`review.rs`) are all done. This phase adds four selection commands that mutate `commits` and reuses the existing persistence/event machinery — no schema change, no new persistence code.

The one **non-obvious architectural fact** that resolves the hardest discretion item: the backend already caches the *full, graph-ordered* commit list. `open_repo` (`repo.rs:32,45`) runs `walk_commits(.., 0, usize::MAX)` and stores the whole `GraphResult` in `CommitCache` keyed by raw path. Frontend pagination (`get_commit_graph` offset slices, `history.rs:28-34`) is purely a *display* slice; it does not constrain what the backend knows. So "selected-but-not-loaded commits" is **not a pagination problem** — SEL-04's graph-ordered list is produced server-side by intersecting `session.commits` with the cached order. No frontend reconciliation logic is needed.

The D-04 marker also has a clean, low-risk home: `CommitRow.svelte` is a **plain HTML flex row** (the SVG lane pipeline is a separate absolutely-positioned overlay). Row-background tinting already exists there for `selected` / `isSearchMatch` / `isCurrentMatch` (`CommitRow.svelte:66-68`). A new `inSession` prop driving a theme-variable background or `border-left` accent follows that exact precedent and never touches the SVG graph pipeline — satisfying both D-04 and the `.claude/rules/commit-graph.md` "never post-process the SVG layers" rule.

**Primary recommendation:** Add four commands to `commands/review.rs` — `seed_review_range`, `add_review_commit`, `remove_review_commit`, `list_session_commits` — each mutating `commits` under the `ReviewSessionsState` mutex (read-modify-write must be serialized), persisting via `review_store::save_session`, and emitting `session-changed`. Surface membership to the frontend via `list_session_commits` (NOT by bloating `SessionStatus`). Mark in-graph membership via a new `inSession` prop on `CommitRow`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Range walk (base→tip OIDs) | API/Backend (Rust, git2 revwalk) | — | git2-only rule; revwalk is a libgit2 API. Mirrors `interactive_rebase.rs:40`. |
| Invalid-range detection | API/Backend (Rust, git2) | — | `merge_base`/`graph_descendant_of` are libgit2; validation must run where the repo lives. |
| Set membership / dedup | API/Backend (Rust) | — | `commits` is the canonical store; the set is authoritative on disk, not in the UI. |
| Persistence (atomic write) | Storage (Rust `review_store`) | — | Reuses Phase 65 atomic tmp+rename. Every mutation goes through it. |
| Graph-ordered list (SEL-04) | API/Backend (intersect with `CommitCache`) | — | Backend holds the full ordered list; ordering authority is `walk_commits`. |
| In-graph marker (D-04) | Frontend (Svelte `CommitRow` HTML row) | — | Plain HTML row tint; must NOT touch the SVG overlay tiers. |
| Add/Remove toggle (D-06) | Frontend (CommitGraph context menu) | API/Backend (command) | Menu item is UI; the mutation is a backend command. |
| Panel list + × button (D-05/07) | Frontend (ReviewPanel) | API/Backend (`list_session_commits`, `remove_review_commit`) | Display + remove affordance; data and mutation are backend. |
| Live multi-tab sync | Frontend (`session-changed` listener) | API/Backend (emit) | Reuses Phase 65 DP-01 event. |

## Standard Stack

This phase introduces **no new dependencies**. Everything is already in the tree.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 (libgit2) | 0.19, vendored | revwalk range, `merge_base`, `graph_descendant_of`, `find_commit` | Project rule: all git ops go through git2, no shelling out. [VERIFIED: `src-tauri/Cargo.toml:26`] |
| Tauri | 2 | `#[tauri::command]`, `app.emit`, managed state, native `Menu`/`MenuItem` | Existing IPC + menu mechanism. [VERIFIED: codebase] |
| Svelte 5 | runes | `CommitRow`/`ReviewPanel` rendering, `$state`/`$derived` | Existing frontend stack. [VERIFIED: codebase] |
| serde / serde_json | (in tree) | command request structs + session (de)serialization | Existing serde conventions. [VERIFIED: codebase] |

### Command Surface (resolves the "Command surface & naming" discretion item)

Live in the existing `src-tauri/src/commands/review.rs` (recommended — per-domain file already exists, mirrors Phase 65). Register in `lib.rs` `invoke_handler` alongside `commands::review::*` (currently `lib.rs:124-127`).

| Command | Args (camelCase via serde) | Returns | Requirement |
|---------|----------------------------|---------|-------------|
| `seed_review_range` | `path: String, baseOid: String, tipOid: String` | `()` (emits) | SEL-01 |
| `add_review_commit` | `path: String, oid: String` | `()` (emits) | SEL-02 |
| `remove_review_commit` | `path: String, oid: String` | `()` (emits) | SEL-03 |
| `list_session_commits` | `path: String` | `Vec<SessionCommit>` | SEL-04 |

`SessionCommit` (new Serialize struct, snake_case default to match `GraphCommit`):
```rust
#[derive(Debug, Serialize, Clone)]
pub struct SessionCommit {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
}
```

**Naming/serde convention** [VERIFIED: `review.rs`, `interactive_rebase.rs:11`, `history.rs:11`]:
- Serialize-default structs are snake_case (`SessionStatus`, `GraphCommit`, `GraphResponse`).
- Frontend-facing **request** types use `#[serde(rename_all = "camelCase")]` (e.g. `RebaseTodoAction` at `interactive_rebase.rs:11`). Command function *parameters* are received as camelCase from JS automatically by Tauri (e.g. `baseOid` ⇄ `base_oid`), as `get_rebase_todo(base_oid: String, …)` demonstrates — no wrapper struct needed for a few scalar args.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| New `list_session_commits` command | Add `commits` to `SessionStatus` | Bloats the status payload that ReviewPanel polls; SEL-04 list needs `short_oid`+`summary` which require repo access the status command doesn't do. Keep them separate. |
| Backend-computed graph order | Frontend reorders `commits` against loaded slice | Frontend only has a paginated slice; selected commits outside it would be unorderable. Backend has the full cached order. Reject frontend ordering. |
| Atomic `seed_review_range` (one walk, one save, one emit) | N× `add_review_commit` | N disk writes + N events for one user gesture. Reject decomposition. |

**Installation:** None — no `cargo add` / `npm install` this phase.

## Package Legitimacy Audit

Not applicable — Phase 66 installs **no external packages**. All capabilities use crates already vendored in `src-tauri/Cargo.toml` (git2 0.19, tauri 2, serde) and existing frontend deps. slopcheck/registry verification is moot.

## Architecture Patterns

### System Architecture Diagram

```
  User right-clicks commit in graph                 User clicks × in panel list
            │                                                 │
            ▼                                                 ▼
  CommitGraph.showCommitContextMenu              ReviewPanel row × button
   (D-06 toggle: Add / Remove)                    (D-07 remove)
            │                                                 │
            └───────────────┬─────────────────────────────────┘
                            ▼
        safeInvoke("add_review_commit" | "remove_review_commit"
                  | "seed_review_range", {path, oid|baseOid,tipOid})
                            │
                            ▼  (Rust, spawn_blocking)
   ┌─────────────────────────────────────────────────────────────────┐
   │ 1. lock ReviewSessionsState (canonical key)                       │
   │ 2. read current ReviewSession from in-memory map                  │
   │ 3. mutate commits:  range → revwalk push(tip)/hide(base.parent)   │
   │                     add   → push if absent                        │
   │                     remove→ retain != oid                         │
   │    (range: validate first via merge_base/graph_descendant_of)     │
   │ 4. save_session(data_dir, canonical, &new)   [atomic tmp+rename]  │
   │ 5. update in-memory map ; release lock                            │
   │ 6. app.emit("session-changed", canonical)                        │
   └─────────────────────────────────────────────────────────────────┘
                            │ event
            ┌───────────────┴───────────────┐
            ▼                                ▼
  ReviewPanel listener                CommitGraph listener
  → list_session_commits(path)        → list_session_commits(path)
  → render D-05 list                  → build Set<oid>
                                      → CommitRow inSession marker (D-04)
                                      → context-menu toggle label (D-06)

  list_session_commits (Rust):
     session.commits  ∩  CommitCache[path] (FULL walk_commits order)
        → graph-ordered, dedup'd  [{oid, short_oid, summary}]
     OID not in cache → repo.find_commit fallback (append/flag)
```

### Recommended structure (where each piece lands)
```
src-tauri/src/commands/review.rs   # + seed_review_range / add / remove / list_session_commits
                                   #   (mutating fns + thin #[tauri::command] wrappers)
src-tauri/src/lib.rs               # register the 4 new commands in invoke_handler (after :127)
src/components/CommitGraph.svelte  # D-06 toggle item in showCommitContextMenu; transient base
                                   #   highlight (D-01); fetch + Set of session OIDs; pass inSession
src/components/CommitRow.svelte    # + inSession prop → background/border-left via theme var (D-04)
src/components/ReviewPanel.svelte  # + commit list (D-05) + per-row × (D-07); listen session-changed
src/lib/types.ts                   # + SessionCommit interface
```

### Pattern 1: Range walk (SEL-01, D-02 inclusive)
**What:** revwalk `push(tip)` + `hide(base.parent(0))`, with root-commit fallback.
**When to use:** `seed_review_range` only.
**Example:**
```rust
// Source: derived from src-tauri/src/commands/interactive_rebase.rs:40-56 (VERIFIED)
// D-02 inclusive [base..tip]: hide base's PARENT, not base itself, so base is included.
let mut revwalk = repo.revwalk()?;
revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
revwalk.push(tip)?;                 // include tip and its ancestors
let base_commit = repo.find_commit(base)?;
if base_commit.parent_count() > 0 {
    revwalk.hide(base_commit.parent_id(0)?)?;   // stop just below base → base included
}
// else: root commit base — hide nothing (interactive_rebase.rs:53 fallback)
let range_oids: Vec<String> = revwalk
    .filter_map(|r| r.ok())
    .map(|oid| oid.to_string())
    .collect();
```
**Note on first-parent vs full ancestry:** `hide(base.parent_id(0))` hides only the first-parent line below base. For a linear range this is exactly `git log base^..tip`. For ranges crossing merges the semantics match `interactive_rebase.rs`'s established behavior — acceptable and consistent with the existing codebase. The union+dedup into the set (D-03) makes exact walk shape non-critical: anything reachable in `[base..tip]` is what the user asked to review.

### Pattern 2: Set union + dedup (D-03, SEL-04 criterion #4)
**What:** treat `commits` as a set; union new OIDs, dedup, never reorder on disk (order is imposed at read time by `list_session_commits`).
**Example:**
```rust
// Union range into existing selection, preserving set semantics. Store order is
// irrelevant — list_session_commits re-imposes graph order on read.
let mut set: std::collections::HashSet<String> = session.commits.iter().cloned().collect();
for oid in range_oids { set.insert(oid); }
session.commits = set.into_iter().collect();   // unordered on disk; graph-ordered on read
```

### Pattern 3: Graph-ordered read (SEL-04)
**What:** intersect the session set with the FULL cached graph order; fall back to on-demand resolution for OIDs not in cache.
**Example:**
```rust
// CommitCache holds the FULL ordered GraphResult (repo.rs:32,45 → walk_commits(.., 0, usize::MAX)).
// Keyed by RAW path (history.rs / repo.rs convention), NOT canonical.
let graph = cache_map.get(path).ok_or_else(|| TrunkError::new("repo_not_open", "..."))?;
let want: std::collections::HashSet<&String> = session.commits.iter().collect();
let mut out: Vec<SessionCommit> = graph.commits.iter()
    .filter(|c| want.contains(&c.oid))
    .map(|c| SessionCommit { oid: c.oid.clone(), short_oid: c.short_oid.clone(), summary: c.summary.clone() })
    .collect();
// Fallback: selected OIDs not present in the cached graph (orphaned/force-pushed since selection).
// Resolve summary on demand and append at the end (never silently drop).
let seen: std::collections::HashSet<&String> = out.iter().map(|c| &c.oid).collect();
for oid_str in &session.commits {
    if !seen.contains(oid_str) {
        if let Ok(oid) = git2::Oid::from_str(oid_str) {
            if let Ok(c) = repo.find_commit(oid) {
                out.push(SessionCommit {
                    oid: oid_str.clone(),
                    short_oid: oid_str.chars().take(7).collect(),
                    summary: c.summary().unwrap_or("").to_owned(),
                });
            }
        }
    }
}
```

### Pattern 4: D-04 marker on the HTML row (NOT the SVG overlay)
**What:** add `inSession?: boolean` to `CommitRow`, drive a background tint or `border-left` from a theme variable, mirroring the existing `selected`/`isSearchMatch` inline-style precedent (`CommitRow.svelte:66-68`).
**Example:**
```svelte
<!-- CommitRow.svelte: extend the existing root-div style expression.
     Theme variable only (project rule: never inline colors).
     Reuse --color-selected-row, or add a dedicated --color-review-row to the theme. -->
<div
  ...
  style="color: var(--color-text);
    {inSession ? 'box-shadow: inset 3px 0 0 var(--color-accent);' : ''}
    {isCurrentMatch ? 'background: ...' : ... : selected ? 'background: var(--color-selected-row);' : ''} ...">
```
`CommitGraph.svelte:1806` passes the prop: `inSession={sessionOids.has(commit.oid)}` where `sessionOids` is a `$state<Set<string>>` refreshed from `list_session_commits` on mount and on `session-changed`.

### Pattern 5: D-06 toggle in the native context menu
**What:** in `showCommitContextMenu`, append ONE `MenuItem` whose label/action depend on membership. Only show it when a session is active (no session → no item).
```ts
// In showCommitContextMenu (CommitGraph.svelte:643 items array). NO merge gating (D-08):
// do NOT copy `enabled: !commit.is_merge` from Cherry-pick (line 682).
const inSession = sessionOids.has(commit.oid);
const reviewItems = sessionActive ? [
  await MenuItem.new({
    text: inSession ? "Remove from review" : "Add to review",
    action: () => {
      safeInvoke(inSession ? "remove_review_commit" : "add_review_commit",
        { path: repoPath, oid: commit.oid }).catch(() => {});
    },
  }),
  await PredefinedMenuItem.new({ item: "Separator" }),
] : [];
```

### Anti-Patterns to Avoid
- **Putting the D-04 marker in the SVG pipeline.** Do NOT add membership styling in `overlay-paths.ts`, `active-lanes.ts`, or the `<g class="overlay-dots">` block. Violates `.claude/rules/commit-graph.md` ("never post-process the SVG layers"). The marker belongs on the HTML `CommitRow`.
- **Read-modify-write without holding the mutex** (see Pitfall 2). Mirroring the start/end `_inner` pattern blindly is wrong: those create/delete; add/remove/seed *mutate existing* state and must serialize.
- **Decomposing range seeding into N adds** (N writes, N events).
- **Gating add/range on `is_merge`** — D-08 explicitly makes merges selectable. The `enabled: !commit.is_merge` precedent (line 682) is for Cherry-pick/Revert only.
- **Bloating `SessionStatus` with commits** — keep `list_session_commits` separate.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Range walk base→tip | Manual parent-chain traversal | git2 `revwalk` push/hide | Handles merges, topo sort, multiple roots; already the codebase pattern. |
| "Is base an ancestor of tip?" | Custom DAG walk | git2 `graph_descendant_of(tip, base)` + `merge_base` | libgit2-correct; handles unrelated histories. |
| Atomic persistence | New file writer | `review_store::save_session` | Atomic tmp+rename + corrupt recovery already built (Phase 65). |
| Graph ordering | Sort by timestamp in JS | Intersect with cached `walk_commits` order | Timestamp ≠ topological order; backend already has the authoritative order. |
| Multi-tab sync | Polling | existing `session-changed` emit + listener | DP-01 mechanism already wired in ReviewPanel. |

**Key insight:** Phase 65 already paid for the hard parts (atomic store, recovery, event sync, the testable `_inner` shape). Phase 66 is mostly *wiring* existing primitives; the only genuinely new logic is the revwalk range + invalid-range validation + the set union/dedup.

## Common Pitfalls

### Pitfall 1: Invalid-range edge cases produce a surprising or empty set
**What goes wrong:** Seeding with `base` not an ancestor of `tip`, unrelated histories, or `base == tip` either errors opaquely or silently produces a weird/empty selection.
**Why it happens:** revwalk `hide(base.parent)` does not validate ancestry; it just hides whatever is reachable. `graph_descendant_of(c, c)` returns **false** (a commit is not its own descendant — [VERIFIED: git2 0.19 `repo.rs` doc comment], in contrast to `git merge-base --is-ancestor`), so naive ancestry checks misclassify the valid `base == tip` case.
**How to avoid (resolves the "Invalid-range handling" discretion item — lean: validate + toast, never silent):**
- `base == tip`: **valid** under D-02 inclusive semantics → set = `{base}`. Detect by OID equality *before* the descendant check.
- `base` not an ancestor of `tip`: `repo.graph_descendant_of(tip, base)?` is false → reject with a toast ("Base is not an ancestor of tip"). Order the args correctly: `graph_descendant_of(descendant=tip, ancestor=base)`.
- Unrelated histories: `repo.merge_base(base, tip)` returns `Err` (`NotFound`) → reject with a toast ("These commits share no history").
- Root-commit base: `parent_count()==0` → push tip, hide nothing (Pattern 1 fallback).
- Empty result after a valid walk: should not happen given the above, but if `range_oids` is empty, reject rather than emit a no-op.
**Warning signs:** a "successful" seed that adds zero commits, or a panic on `parent_id(0)`.

### Pitfall 2: Concurrent add/remove races (read-modify-write)
**What goes wrong:** Two rapid "Add to review" clicks (or add+remove) read the same stale session and one write clobbers the other; a commit silently vanishes from or duplicates in the set.
**Why it happens:** `start_review_session_inner`/`end_..._inner` are create/delete and effectively idempotent, so the existing pattern reads no prior state. Add/remove/seed are **read-modify-write on the persisted session** — if each `spawn_blocking` independently reads the in-memory map, computes, and writes back, concurrent calls race.
**How to avoid:** serialize the whole RMW under the `ReviewSessionsState` mutex:
1. lock `sessions.0` → 2. clone/read the current `ReviewSession` for the canonical key → 3. compute the new session (walk/union/retain) → 4. `review_store::save_session(...)` **while still holding the lock** → 5. write the new session back into the map → 6. release → 7. `app.emit`.
This means the `_inner` for these commands takes the *current session* as an argument (or the command does the RMW inline rather than re-reading disk inside `_inner`). Keep the disk write inside the critical section so disk and memory never diverge (consistent with Phase 65's disk-first D-10 intent, adapted for mutation).
**Note on blocking in async:** the existing commands wrap disk work in `tauri::async_runtime::spawn_blocking`. Holding a `std::sync::Mutex` across a blocking `save_session` is fine here (writes are small, fast, single-machine); do NOT hold it across an `.await`.
**Warning signs:** flaky tests where a rapid add+add yields one commit; a removed commit reappearing after a near-simultaneous add elsewhere.
**Confidence:** MEDIUM (pattern is sound and matches the codebase's mutex usage; the exact `_inner` signature is the planner's call).

### Pitfall 3: Canonical vs raw path key mismatch
**What goes wrong:** A selection command writes to one session file but reads graph order from the wrong cache entry, or `list_session_commits` can't find the cache.
**Why it happens:** Phase 65's session store keys by **canonical** path (`std::fs::canonicalize`, D-11), but `RepoState`/`CommitCache` key by the **raw** path string (`repo.rs:34`, unchanged by Phase 65 — D-11 explicitly does not retrofit canonicalization onto them). A selection command touches BOTH worlds: it canonicalizes for `review_store`, and uses the raw `path` for `CommitCache`/`RepoState`.
**How to avoid:** follow `review.rs`'s `canonical_repo_path(path, state_map)` for the store side; use the raw `path` directly for `CommitCache.get(path)`. `list_session_commits` reads the session by canonical key and the graph by raw key — both derived from the same incoming `path` arg.
**Warning signs:** `repo_not_open` from `list_session_commits` even though the repo is open; or selection writes that don't show up because the wrong file was read.

### Pitfall 4: D-06 toggle shown with no active session
**What goes wrong:** "Add to review" appears even when there is no session, inviting a `not_open`/no-session error path.
**How to avoid:** CommitGraph must know whether a session is active (it already can via `get_review_session_status` or by whether `list_session_commits` succeeds / panel open state). Only inject the toggle item when active. The panel-open flag (`reviewPanelOpen`, App.svelte:592) is a reasonable proxy but confirm it tracks *session active*, not just panel visibility.

### Pitfall 5: Stale `sessionOids` Set after a mutation
**What goes wrong:** Graph marker (D-04) and toggle label (D-06) lag behind the actual set after an add/remove.
**How to avoid:** CommitGraph holds `sessionOids` as `$state<Set<string>>`, (re)loaded from `list_session_commits` on mount and on every `session-changed` for this repo's canonical path — exactly the listener pattern ReviewPanel already uses (`ReviewPanel.svelte:55-66`). Reassign the Set (`sessionOids = new Set(...)`) so Svelte 5 reactivity fires.

## Runtime State Inventory

Phase 66 is **not** a rename/refactor/migration phase — it adds behavior, it does not rewrite identifiers or move stored data. Per the protocol, each category is answered explicitly:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None new — `commits` field already exists in the session JSON (Phase 65). Existing session files have `"commits": []`; populating them is normal use, not migration. | None — verified by reading `types.rs:333` and `review_store.rs` (no schema change, `schema_version` stays 1). |
| Live service config | None — purely local desktop app. | None. |
| OS-registered state | None. | None. |
| Secrets/env vars | None. | None. |
| Build artifacts | None — no package rename; existing build outputs unaffected. | None. |

## Code Examples

### Detecting an invalid range (SEL-01 validation)
```rust
// Source: git2 0.19 repo.rs (VERIFIED signatures + doc comments).
// Returns Ok(()) if [base..tip] is a valid inclusive range; Err(TrunkError) otherwise.
fn validate_range(repo: &git2::Repository, base: git2::Oid, tip: git2::Oid) -> Result<(), TrunkError> {
    if base == tip { return Ok(()); }                       // D-02: {base} is valid
    // Unrelated histories → merge_base errors (NotFound).
    repo.merge_base(base, tip)
        .map_err(|_| TrunkError::new("unrelated_history", "These commits share no history"))?;
    // base must be an ancestor of tip. Note: graph_descendant_of(x,x)==false, handled above.
    let ok = repo.graph_descendant_of(tip, base)
        .map_err(TrunkError::from)?;
    if !ok {
        return Err(TrunkError::new("bad_range", "Base is not an ancestor of tip"));
    }
    Ok(())
}
```

### Thin command shape (mirrors review.rs, adapted for RMW)
```rust
// Source: pattern from src-tauri/src/commands/review.rs (VERIFIED) + Pitfall 2 serialization.
#[tauri::command]
pub async fn add_review_commit(
    path: String,
    oid: String,
    state: State<'_, RepoState>,
    sessions: State<'_, ReviewSessionsState>,
    app: AppHandle,
) -> Result<(), String> {
    let state_map = state.0.lock().unwrap().clone();
    let data_dir = resolve_data_dir(&app)?;
    let canonical = /* canonical_repo_path(&path, &state_map) → JSON-stringify error */;
    // RMW under the sessions mutex (Pitfall 2):
    {
        let mut map = sessions.0.lock().unwrap();
        let session = map.get(&canonical).cloned()
            .ok_or_else(|| /* "no_session" JSON */)?;
        let mut next = session;
        if !next.commits.contains(&oid) { next.commits.push(oid.clone()); }
        review_store::save_session(&data_dir, &canonical, &next)
            .map_err(|e| serde_json::to_string(&e).unwrap())?;
        map.insert(canonical.clone(), next);
    }
    let _ = app.emit("session-changed", canonical.to_string_lossy().into_owned());
    Ok(())
}
```
*(Planner: decide whether to keep a testable `_inner` that takes `(data_dir, current_session, oid) -> ReviewSession` for pure set logic, with the mutex/IO orchestration in the thin command. This keeps the union/dedup/walk unit-testable without Tauri state — matching the Phase 65 testability ethos while respecting Pitfall 2.)*

### Frontend: refresh session Set on change (CommitGraph)
```ts
// Mirrors ReviewPanel.svelte:55-66 listener pattern (VERIFIED).
let sessionOids = $state<Set<string>>(new Set());
async function reloadSessionOids() {
  try {
    const list = await safeInvoke<SessionCommit[]>("list_session_commits", { path: repoPath });
    sessionOids = new Set(list.map((c) => c.oid));
  } catch { sessionOids = new Set(); }  // no session / repo not open
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| n/a (new feature) | git2 revwalk push/hide for ranges | established in codebase (interactive_rebase.rs) | Reuse, don't reinvent. |

**Deprecated/outdated:** none relevant. git2 0.19 is the in-tree version; `merge_base`, `graph_descendant_of`, `revwalk` are all stable, long-standing APIs.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `reviewPanelOpen` (App.svelte:592) can serve as / be extended to a "session active" signal for D-06 gating | Pitfall 4 | If it tracks only panel visibility, the toggle could show with no session; mitigated by calling `list_session_commits`/status. LOW risk — planner verifies during plan. |

*All other claims in this research were VERIFIED against the codebase or the installed git2 0.19 source, or follow directly from locked CONTEXT decisions.*

## Open Questions

1. **Selected-but-not-loaded commits (the discretion item) — RESOLVED, recorded for the planner.**
   - What we know: `open_repo` populates `CommitCache` with the FULL `walk_commits(.., 0, usize::MAX)` (repo.rs:32,45). Frontend pagination is a display-only slice. So a selected commit "outside the loaded slice" is still in the backend's cached order.
   - What's unclear: only the genuine edge case where a selected OID is not in the cached graph at all (e.g. it became unreachable after a force-push/rebase between selection and read).
   - Recommendation: `list_session_commits` intersects with the cached order (graph order, dedup), then **appends** any unresolved-in-cache OIDs via `repo.find_commit` fallback (Pattern 3) so SEL-04 never silently drops a selected commit. If even `find_commit` fails (OID truly gone), include it with a placeholder summary or omit-and-flag — planner's call (lean: include with `(unavailable)` summary, consistent with Phase 65 "never silently destroy").

2. **`_inner` signature for RMW commands.**
   - What we know: Phase 65's `_inner` takes `(data_dir, path, state_map)` and is pure-ish (create/delete). Add/remove/seed need the *current* session.
   - Recommendation: extract pure set logic into a testable helper (`apply_add`/`apply_remove`/`compute_range`) and keep mutex+IO orchestration in the thin command (see Code Examples note). Preserves unit-testability without Tauri state.

3. **Transient base highlight (D-01).**
   - What we know: D-01 wants a pending-base highlight between the two right-clicks, cleared on completion/cancel. This is pure frontend state in CommitGraph (a `$state<string|null> pendingBase`), distinct from `sessionOids`.
   - Recommendation: reuse the same `CommitRow` marker mechanism with a *second* prop (`isPendingBase`) and a distinct theme variable, OR a transient class — planner decides. No backend involvement.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| git2 / libgit2 | revwalk, merge_base, descendant | ✓ (vendored) | 0.19 | — |
| Rust toolchain | backend build | ✓ | (project) | — |
| Node/Vite/Svelte | frontend build | ✓ | (project) | — |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** none. Phase 66 has no external/runtime dependencies beyond the existing toolchain.

## Validation Architecture

`workflow.nyquist_validation` is not disabled in config, so this section is included.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust: `cargo test --lib` (in-process tempfile/test-repo unit tests). Frontend: `vitest` (`npx vitest run`), component tests via `@testing-library/svelte` (see `ReviewPanel.test.ts`). |
| Config file | Rust: Cargo defaults. Frontend: existing vitest config (ReviewPanel.test.ts already runs). |
| Quick run command | `cd src-tauri && cargo test --lib review` (selection tests) / `npx vitest run ReviewPanel CommitRow` |
| Full suite command | `just check` (fmt, biome, svelte-check, clippy, cargo-test, vitest) |

### Phase Requirements → Test Map
| Req | Behavior | Test Type | Automated Command | File Exists? |
|-----|----------|-----------|-------------------|-------------|
| SEL-01 | `[base..tip]` inclusive: both endpoints in result | unit (Rust) | `cargo test --lib seed_range_inclusive` | ❌ Wave 0 |
| SEL-01 | root-commit base: walk without hiding | unit (Rust) | `cargo test --lib seed_range_root_base` | ❌ Wave 0 |
| SEL-01 | `base == tip` → single-commit set | unit (Rust) | `cargo test --lib seed_range_base_eq_tip` | ❌ Wave 0 |
| SEL-01 | base not ancestor of tip → Err, no mutation | unit (Rust) | `cargo test --lib seed_range_rejects_non_ancestor` | ❌ Wave 0 |
| SEL-01 | unrelated histories → Err (merge_base NotFound) | unit (Rust) | `cargo test --lib seed_range_rejects_unrelated` | ❌ Wave 0 |
| SEL-01/D-03 | range unions, does not replace; dedup | unit (Rust) | `cargo test --lib seed_range_unions_dedups` | ❌ Wave 0 |
| SEL-02 | add appends, idempotent (no dup) | unit (Rust) | `cargo test --lib add_commit_idempotent` | ❌ Wave 0 |
| SEL-03 | remove deletes exactly one; missing oid is no-op | unit (Rust) | `cargo test --lib remove_commit` | ❌ Wave 0 |
| SEL-02/03 | concurrent add/remove serialized (no lost write) | unit (Rust) | `cargo test --lib selection_rmw_serialized` | ❌ Wave 0 |
| SEL-04 | list returned in graph order, dedup'd | unit (Rust) | `cargo test --lib list_session_commits_graph_order` | ❌ Wave 0 |
| SEL-04 | selected OID absent from cache → fallback resolves/append | unit (Rust) | `cargo test --lib list_session_commits_orphan_fallback` | ❌ Wave 0 |
| D-08 | merge commit IS selectable (contrast interactive_rebase merge handling) | unit (Rust) | `cargo test --lib merge_commit_selectable` | ❌ Wave 0 |
| D-05/07 | panel renders list + × triggers remove | component (vitest) | `npx vitest run ReviewPanel` | ⚠️ extend existing `ReviewPanel.test.ts` |
| D-04 | `inSession` prop tints row via theme var | component (vitest) | `npx vitest run CommitRow` | ❌ Wave 0 (`CommitRow.test.ts`) |
| D-06 | toggle label flips Add/Remove on membership | component (vitest) | `npx vitest run CommitGraph` | ⚠️ if a CommitGraph test harness exists; else manual |
| SEL-04/sync | `session-changed` triggers list reload | component (vitest) | `npx vitest run ReviewPanel` | ⚠️ pattern exists in `ReviewPanel.test.ts:84` |

### Sampling Rate
- **Per task commit:** `cd src-tauri && cargo test --lib review` (or the touched test file's vitest subset).
- **Per wave merge:** `just check`.
- **Phase gate:** `just check` green before `/gsd:verify-work`.

### Wave 0 Gaps
- [ ] Rust selection unit tests in `commands/review.rs` `#[cfg(test)]` — need an in-process test repo with a known linear+merge topology. Check whether a test-repo helper already exists (grep for `tempfile`/`Repository::init` in `src-tauri` tests) before hand-rolling one; the revwalk tests need real commits.
- [ ] `src/components/CommitRow.test.ts` — covers D-04 `inSession` marker (none exists today).
- [ ] Extend `src/components/ReviewPanel.test.ts` — list rendering (D-05) + × remove (D-07) + reload-on-`session-changed`.
- [ ] (Optional) CommitGraph context-menu toggle test — Tauri `Menu` mocking is heavier; manual verification acceptable for D-06 label flip if no harness exists.

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/commands/review.rs` — Phase 65 command pattern, `SessionStatus`, `canonical_repo_path`, `_inner`+thin+emit shape.
- `src-tauri/src/git/review_store.rs` — `save_session`/`load_session`/`delete_session`, atomic tmp+rename, recovery.
- `src-tauri/src/commands/interactive_rebase.rs:40-56` — revwalk push/hide range + root-commit fallback (D-02 source pattern).
- `src-tauri/src/git/graph.rs:52` + `src-tauri/src/commands/history.rs:17-67` + `src-tauri/src/commands/repo.rs:32,45` — full graph cached at open; pagination is display-only (resolves Open Question 1).
- `src/components/CommitRow.svelte:61-115` — plain HTML row, existing `selected`/`isSearchMatch` tint precedent (D-04 home).
- `src/components/CommitGraph.svelte:567-721,1806` — `showCommitContextMenu` insertion point, `CommitRow` prop wiring, merge-gating precedent (line 682, NOT applied per D-08).
- `src/components/ReviewPanel.svelte` + `ReviewPanel.test.ts` — `session-changed` listener pattern; component test harness.
- `src-tauri/src/git/types.rs:330-336` — `ReviewSession` shape (unchanged).
- git2 0.19 installed source `repo.rs:2442,2533` — `merge_base`, `graph_descendant_of` signatures + doc note that `graph_descendant_of(x,x)==false`.
- `.planning/COMMIT-GRAPH-ARCHITECTURE.md` + `.claude/rules/commit-graph.md` — SVG-pipeline boundary (D-04 anti-pattern).

### Secondary (MEDIUM confidence)
- Concurrency mutex-ordering recommendation (Pitfall 2) — sound and consistent with codebase mutex usage; exact `_inner` signature deferred to planner.

### Tertiary (LOW confidence)
- A1 (`reviewPanelOpen` as session-active proxy) — flag for planner verification.

## Metadata

**Confidence breakdown:**
- Standard stack / command surface: HIGH — verified against `review.rs`, `interactive_rebase.rs`, git2 0.19 source; no new deps.
- Architecture (graph-order via cache, D-04 row marker): HIGH — verified `repo.rs`/`history.rs`/`CommitRow.svelte` directly.
- Invalid-range handling: HIGH — git2 API signatures + `graph_descendant_of` self-descendant semantics verified in installed source.
- Concurrency pitfall: MEDIUM — pattern is correct; precise `_inner` shape is the planner's design choice.

**Research date:** 2026-05-25
**Valid until:** 2026-06-24 (stable — in-tree deps, no fast-moving externals)
