# Phase 69: Comment Management UI - Research

**Researched:** 2026-05-26
**Domain:** Tauri 2 + Svelte 5 review-session UI; git2 anchor resolvability; serde schema migration (v1→v2)
**Confidence:** HIGH (all claims grounded in files read this session; no new external deps required)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Commit-level comment is tied to a specific commit (no file/line), renders with the commit SHA heading, MUST store `commit_oid`, MUST be distinguishable from line-anchored comments. Rides the v2 bump.
- **D-02:** Creation surface is a per-commit "Add note" affordance in the panel's commit list.
- **D-03:** Add a stable `id` field to each `Comment`, generated on write. `edit_comment`/`delete_comment` target by `id`. Index-by-position rejected.
- **D-04:** Bump `schema_version` 1 → 2. Both `id` (D-03) and commit-level representation (D-01) land in v2 — one bump. Backfill ids on load for v1 sessions. Honor D-16 (newer refused untouched); this build now supports v2.
- **D-05:** Delete uses the v0.6 confirmation-dialog pattern (`@tauri-apps/plugin-dialog` `ask`), consistent with `DiffPanel.handleDiscardLines`.
- **D-06:** Resolvability checked eagerly at panel load via a Rust git2-backed command resolving every comment (commit exists? file exists at that commit on the anchor's side? line range within bounds?). Frontend "is the commit in `session.commits`" check is insufficient.
- **D-07:** Jump reveals the diff: selects commit + file, switches the right pane back to diff/full-file view per `Source` (`Source::Diff` → commit's diff of that file; `Source::FullFile` → full-file-at-commit view), scrolls to and highlights the range. Session stays active; a persistent "Review" toggle returns to the panel.
- **D-08:** An orphaned comment is read-only — jump disabled, shows a reason badge (commit gone / file gone / line out of range). Comment text + cached excerpt stay visible (Phase 65 D-04).
- **D-09:** Comment list grouped by commit. Independent of Phase 70 render-by-line ordering.
- **D-10:** Edit is inline in the panel — click edit → a textarea appears in place; save/cancel there. Works uniformly for line-anchored, commit-level, AND orphaned comments. Reusing the Phase 67 diff composer rejected (couples edit to a resolvable anchor).

### Claude's Discretion
- Commit-level storage mechanism (D-01): least-disruptive v2 representation. Candidates: `Source::Commit` enum variant; optional Anchor file/line fields; dedicated commit-comment shape. Keep Phase 67/68 anchors unchanged.
- `id` generation: uuid vs monotonic counter vs content hash. Must be stable for the comment's lifetime.
- Command surface & naming: read-comments (none exists), `edit_comment`, `delete_comment`, commit-level writer (extend `add_comment` OR sibling). Whether resolvability is its own command or folded into read-comments.
- `review-session.svelte.ts` rune module shape and how it drives Review-mode right-pane swapping.
- Attach-success/save feedback: rely on `session-changed` → reload, mostly silent; toasts optional.
- Empty-text validation: zero-text comments disabled; exact validation point is the planner's.

### Deferred Ideas (OUT OF SCOPE)
- In-diff comment markers / browser (gutter badges, click-to-edit) — deferred again.
- Render-time excerpt re-resolution + "unresolvable" render section — Phase 70 (DOC-04). The eager check here is panel-only.
- Comment metadata (severity, author, threading) — out per REQUIREMENTS. The `id` is identity only.
- Clipboard / save-to-file output — Phase 71.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ANCH-03 | Attach a commit-level comment with no code anchor | §"Commit-level storage" — recommend `commit_oid: Option<String>` sibling field on `Comment` (candidate c); sibling `add_commit_comment` command; the discriminator `match (anchor, commit_oid)`. |
| CMT-01 | View all comments in the active session in a review panel | §"Command surface" — new `list_session_comments` read command (none exists today); §"Panel" — grouped-by-commit layout reusing `list_session_commits` + the panel's existing `session-changed` listener. |
| CMT-02 | Edit a comment's text | §"Command surface" — `edit_comment(path, id, text)` targeting by `id` (D-03); §"id generation"; §"Frontend" inline textarea (D-10). |
| CMT-03 | Delete a comment, with confirmation | §"Command surface" — `delete_comment(path, id)`; §"Frontend" — `@tauri-apps/plugin-dialog` `ask` confirm (D-05, mirrors `DiffPanel.handleDiscardLines`). |
| CMT-04 | Jump from a comment to its anchored code location | §"git2 resolvability" — eager `resolve_session_comments` command; §"Jump wiring" — `handleCommitSelect` → `handleCommitFileSelect` → `DiffPanel.scrollToHunk`; orphan → read-only badge (D-08). |
</phase_requirements>

## Summary

This is a brownfield phase that extends a deliberately frozen, well-tested schema (Phase 65 keystone) and replaces a throwaway stub. The backend conventions are fully established in `commands/review.rs`: testable `_inner(data_dir, …)` cores wrapped by thin `#[tauri::command]`s, all mutations serialized through `mutate_session_rmw` holding the `ReviewSessionsState` mutex across read→mutate→`save_session`→map-write, with `emit("session-changed", canonical)` after panel-visible writes. Every new command in this phase mirrors that shape exactly.

The single highest-risk item is the **v1→v2 serde migration**: `review_store.rs:128` quarantines a session to `.corrupt` on **any** `from_value::<ReviewSession>` failure. Adding a non-defaulted field to `Comment` and bumping the version would corrupt every existing v1 file — converting D-04's intended migration into D-15's bulldozer. The migration must inject ids (and tolerate the missing commit-level field) **before or during** deserialization, via `#[serde(default)]` + a normalize pass, or via JSON-`Value` rewriting between the version check (line 126) and `from_value` (line 128).

**Primary recommendation:** Store commit-level comments as a sibling `commit_oid: Option<String>` field on `Comment` (NOT a new `Source::Commit` variant, NOT optional Anchor line fields) — this leaves the Phase 67/68 `Anchor` invariants completely intact. Add `id: String` to `Comment`. Use `#[serde(default)]` on both new fields plus a one-pass normalize-on-load that backfills empty ids and re-saves. Generate ids with the `uuid` crate (v4). Add a sibling `add_commit_comment` command (do not change `add_comment`'s wire shape). Add `list_session_comments`, `edit_comment`, `delete_comment`, and a git2-backed `resolve_session_comments`. Create a `review-session.svelte.ts` rune module that owns Review-mode right-pane state and the jump action.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Comment persistence (add/edit/delete, id, commit-level) | API/Backend (`commands/review.rs` + `review_store.rs`) | — | Authoritative store; atomic writes + RMW serialization already live here. |
| v1→v2 migration + id backfill | API/Backend (`review_store.rs` load path) | — | Must run before the in-memory session exists; pure, unit-testable. |
| Anchor resolvability check | API/Backend (git2) | — | Requires libgit2 tree/blob reads; cannot be done from the frontend (D-06). |
| Comment list rendering + grouping | Frontend (`ReviewPanel.svelte`) | — | Pure presentation of backend data. |
| Right-pane Review-mode swap + jump action | Frontend (`review-session.svelte.ts` rune + `RepoView.svelte`) | — | Owns transient UI state (panel vs diff); composes existing selection/scroll machinery. |
| Delete confirmation | Frontend (`@tauri-apps/plugin-dialog`) | — | Native dialog; same pattern as `DiffPanel.handleDiscardLines`. |

## Standard Stack

### Core (already in repo — reuse, do not add)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| git2 | 0.19 (vendored libgit2) | Resolvability: `find_commit`, `commit.tree()`, `Tree::get_path`, `repo.find_blob`, blob line count | All local git reads go through git2 (CLAUDE.md rule); `find_blob(entry.id)` already used at `merge_editor.rs:57`. [VERIFIED: src-tauri/Cargo.toml:26, merge_editor.rs:57] |
| serde / serde_json | 1 | Schema (de)serialization, `Value`-level migration | Whole review schema is serde; `from_value`/`from_str` recovery state machine in `review_store.rs:104-135`. [VERIFIED: Cargo.toml:24-25] |
| @tauri-apps/plugin-dialog | 2 | Delete confirmation `ask()` | D-05; identical pattern at `DiffPanel.svelte:385,508`. [VERIFIED: DiffPanel.svelte:385] |
| Svelte 5 runes | — | `review-session.svelte.ts` rune module; panel reactivity | Project uses runes only; existing `.svelte.ts` modules: `toast`, `undo-redo`, `remote-state`. [VERIFIED: src/lib/*.svelte.ts] |

### Supporting (NEW dependency — one)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| uuid | latest 1.x | Stable `id` generation (D-03) | Used in the comment writers (`add_commit_comment`, the line-anchored `add_comment`) and the v1→v2 backfill. [ASSUMED — see Assumptions Log A1; version not pinned this session] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `uuid` crate (v4) | Monotonic counter | Counter needs persistent state in the session and is fragile under the multi-tab story (two tabs would both mint the same next-id between read and write). Rejected. |
| `uuid` crate (v4) | Content hash (text+anchor) | Collides on identical-text comments — a real case: two "looks good" notes on different files would share an id. Rejected: id must be unique per comment, not per content. |
| `uuid` crate (v4) | Hand-rolled (e.g. FNV of a timestamp+counter) | Engineering-judgment §2 "every dependency needs a strong case" — but a correct unique-id generator is exactly the kind of deceptively-simple thing not to hand-roll. `uuid` v4 is tiny, audited, zero-maintenance. Strong case to add. |

**Installation:**
```bash
# Backend — add to src-tauri/Cargo.toml [dependencies]:
#   uuid = { version = "1", features = ["v4"] }
# (verify current version + features before pinning — see Package Legitimacy Audit)
```
The TS side needs no new package — `@tauri-apps/plugin-dialog` is already present (used by `DiffPanel.svelte`).

**Version verification (REQUIRED before the planner pins):** run `cargo search uuid` / check crates.io for the current 1.x and confirm the `v4` feature flag name. Training data may be stale.

## Package Legitimacy Audit

> One new dependency this phase: `uuid` (Rust / crates.io). slopcheck not run this session (no Rust ecosystem support in slopcheck; it targets npm/PyPI). `uuid` is one of the most-downloaded crates in the ecosystem (rust-lang adjacent, source at github.com/uuid-rs/uuid) — extremely low slop risk, but treat the exact version as `[ASSUMED]` until the planner runs `cargo search uuid`.

| Package | Registry | Age | Downloads | Source Repo | slopcheck | Disposition |
|---------|----------|-----|-----------|-------------|-----------|-------------|
| uuid | crates.io | ~10 yrs | very high (top-tier) | github.com/uuid-rs/uuid | n/a (Rust) | Approved — pin version after `cargo search` |

**Packages removed due to slopcheck [SLOP] verdict:** none
**Packages flagged as suspicious [SUS]:** none

*The planner should add a `checkpoint:human-verify` (or just a `cargo search uuid` step) before the first build to confirm the version/feature, per the [ASSUMED] tag.*

## Architecture Patterns

### System Architecture Diagram

```
                          ┌──────────────────────────── Frontend (Svelte 5) ───────────────────────────┐
  panel mount / session-  │                                                                              │
  changed event           │   ReviewPanel.svelte (real, replaces stub)                                   │
        │                 │     ├── list_session_commits ──────────────┐                                 │
        ▼                 │     ├── list_session_comments ─────────────┤  group-by-commit render (D-09)  │
  safeInvoke<T>           │     └── resolve_session_comments ──────────┘  → orphan badges (D-08)         │
        │                 │                                                                              │
        │   per row:      │   review-session.svelte.ts  (NEW rune module — owns Review-mode pane state)  │
        │   add note      │     reviewActive: boolean      rightPaneMode: "panel" | "diff"               │
        │   edit / delete │     jumpTo(comment)  ──► sets selectedCommitOid + selectedCommitFile         │
        │   jump          │                          + rightPaneMode="diff" + scroll target              │
        └─────────────────┤                                                                              │
                          │   RepoView.svelte: handleCommitSelect(oid) → handleCommitFileSelect(path)    │
                          │     → DiffPanel renders → scrollToHunk / scrollIntoView + highlight (D-07)   │
                          └───────────────────────────────────┬──────────────────────────────────────┘
                                                              │ invoke (camelCase args → snake_case)
                          ┌───────────────────────────────────▼──────── Backend (Rust/Tauri 2) ────────┐
                          │  commands/review.rs                                                          │
                          │   thin #[tauri::command]  →  _inner(data_dir, …)                             │
                          │     add_commit_comment   ─┐                                                  │
                          │     add_comment (unchanged)├─► mutate_session_rmw (mutex held across        │
                          │     edit_comment          ─┤      read → mutate → save_session → map-write)  │
                          │     delete_comment        ─┘   → emit("session-changed", canonical)          │
                          │     list_session_comments  (read-only, no emit)                              │
                          │     resolve_session_comments  (git2 spawn_blocking; read-only)               │
                          │                                                                              │
                          │  git/review_store.rs                                                         │
                          │     load_session: read → version-gate (D-16) → migrate v1→v2 (D-04) →        │
                          │                   from_value → normalize/backfill ids → (re-save)            │
                          │     save_session: atomic tmp + sync_all + rename                             │
                          │                                                                              │
                          │  git/types.rs : Comment{ id, text, anchor, commit_oid, cached_excerpt }      │
                          └──────────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure (files touched / created)
```
src-tauri/src/
├── git/types.rs              # MODIFY: add Comment.id, Comment.commit_oid; bump default schema_version
├── git/review_store.rs       # MODIFY: CURRENT_SCHEMA_VERSION 1→2; migrate_v1_to_v2 + normalize/backfill on load
└── commands/review.rs        # MODIFY: add_commit_comment, list_session_comments, edit_comment,
                              #         delete_comment, resolve_session_comments (+ register in lib.rs)
src-tauri/src/lib.rs          # MODIFY: invoke_handler — register the 5 new commands
src/
├── lib/types.ts              # MODIFY: Comment gets id + commit_oid?; add ResolvedComment/orphan-reason types
├── lib/review-session.svelte.ts  # CREATE: rune module — Review-mode right-pane state + jump action
├── components/ReviewPanel.svelte # REWRITE: real panel (keeps session-changed listener + canonical filter)
├── components/RepoView.svelte    # MODIFY: wire panel into a pane; jump composes selection/scroll machinery
└── App.svelte                    # MODIFY: relocate panel from thin-bar to the chosen pane
```

### Pattern 1: Testable `_inner` + thin command + serialized RMW + emit
**What:** Every mutating review command is a thin `#[tauri::command]` that resolves the canonical path + data dir, calls a free `*_rmw`/`*_inner` that holds the sessions mutex across the whole read-modify-write, then emits `session-changed`.
**When to use:** All of `add_commit_comment`, `edit_comment`, `delete_comment`.
**Example:**
```rust
// Source: src-tauri/src/commands/review.rs:390-404 (add_comment_inner, the canonical writer)
fn add_comment_inner(
    data_dir: &Path,
    canonical: &Path,
    sessions: &Mutex<HashMap<PathBuf, ReviewSession>>,
    req: AddCommentRequest,
) -> Result<(), TrunkError> {
    mutate_session_rmw(data_dir, canonical, sessions, |session| {
        session.comments.push(Comment {
            text: req.text,
            anchor: Some(req.anchor),
            cached_excerpt: Some(req.cached_excerpt),
        });
        session.draft_comment = None;
    })
}
```
`edit_comment`/`delete_comment` reuse `mutate_session_rmw` with `session.comments.iter_mut().find(|c| c.id == id)` / `session.comments.retain(|c| c.id != id)`. Missing id is a no-op or a `not_found` `TrunkError` (planner's call; prefer no-op delete for idempotency parity with `apply_remove` at line 232).

### Pattern 2: Read-only command (no mutation, no emit)
**What:** `list_session_comments` mirrors `list_session_commits` (`review.rs:569-619`): read the session by CANONICAL key from the in-memory map (`no_session` if absent), clone out the data, return. No `save_session`, no `emit`.
**Example:**
```rust
// Source: src-tauri/src/commands/review.rs:580-593 (the canonical read pattern)
let comments = {
    let map = sessions.0.lock().unwrap();
    map.get(&canonical)
        .ok_or_else(|| serde_json::to_string(&TrunkError::new(
            "no_session", "No active review session for this repository")).unwrap())?
        .comments
        .clone()
};
```

### Pattern 3: git2 work in `spawn_blocking` on a fresh repo handle
**What:** Never hold the `RepoState` lock across git2 work; open the repo fresh inside `spawn_blocking`. `resolve_session_comments` follows `list_session_commits:605-616`.
**Example:**
```rust
// Source: src-tauri/src/commands/review.rs:607-611
let result = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<…>, TrunkError> {
    let repo = git2::Repository::open(&path).map_err(TrunkError::from)?;
    Ok(resolve_all(&comments, &repo))   // pure resolver — testable against an in-process repo
})
.await…;
```

### Pattern 4: Native confirm dialog (delete)
```typescript
// Source: src/components/DiffPanel.svelte:508-510 (handleDiscardLines)
const { ask } = await import("@tauri-apps/plugin-dialog");
const confirmed = await ask("Delete this comment? This cannot be undone.", { /* opts */ });
if (!confirmed) return;
await safeInvoke("delete_comment", { path: repoPath, id });
```

### Anti-Patterns to Avoid
- **Adding a non-defaulted field to `Comment` + bumping the version.** This corrupts every v1 file via `review_store.rs:128`. See the Migration pitfall below. Use `#[serde(default)]` or `Value`-level migration.
- **Index-by-position edit/delete.** Explicitly rejected (D-03) — a second same-repo tab mutating the list between read and action edits the wrong comment. Target by `id`.
- **A frontend-only resolvability check** ("is the oid in `session.commits`"). Cannot detect renamed/deleted files or out-of-bounds line ranges (D-06). Must be git2-backed.
- **Changing `add_comment`'s wire shape** to make the anchor optional. Breaks every Phase 67/68 caller and its 7 tests (`review.rs:1187-1394`). Add a sibling command instead.
- **Inline raw colors / positioning hacks** (CLAUDE.md + CONVENTIONS.md): orphan badges and group headers use `--color-*` vars; layout via flex/grid.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Unique comment id | Custom timestamp+counter generator | `uuid` v4 | Collision-free, audited, no persistent counter state to race across tabs. |
| Confirm-before-delete | Custom modal component | `@tauri-apps/plugin-dialog` `ask` | D-05; native, already the app-wide pattern (`DiffPanel`). |
| Blob-by-path-at-commit | Manual tree walking | `commit.tree()?.get_path(Path::new(file_path))?` then `repo.find_blob(entry.id())` | git2 0.19 provides `Tree::get_path(&Path) -> Result<TreeEntry, Error>`; blob read pattern is already `merge_editor.rs:57`. |
| Atomic session write | New persistence code | `review_store::save_session` (via `mutate_session_rmw`) | tmp+`sync_all`+rename already implemented & tested. |
| Concurrency safety on writes | Per-command ad-hoc locking | `mutate_session_rmw` | Mutex held across the whole RMW; proven by `selection_rmw_serialized` (50-thread test, `review.rs:1060`). |

**Key insight:** Almost everything this phase needs already exists and is tested. The genuinely new logic is small and pure: id generation, the v1→v2 normalize, and the resolvability classifier — exactly the three units to TDD.

## Runtime State Inventory

> This is a schema-migration phase (v1→v2). The "runtime state" is on-disk v1 session files written by Phases 65–68.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | Per-repo session JSON at `<app_data_dir>/sessions/<16-hex FNV-1a>.json`, `schema_version: 1`, with `comments[]` that have NO `id` and NO `commit_oid` field. Filename keyed by FNV-1a hash of canonical path (`review_store.rs:49-57`). | **Data migration** on load: backfill `id` for each comment, tolerate missing `commit_oid`, re-save as v2. MUST NOT trip the `.corrupt` quarantine. |
| Live service config | None — no external services; session files are the only persisted state. | None — verified: only `review_store.rs` writes session state. |
| OS-registered state | None — verified: no OS-level registrations reference comment shape. | None. |
| Secrets/env vars | None — verified: no secret/env names involved in this schema. | None. |
| Build artifacts | Rust rebuild required after adding the `uuid` dependency (`cargo` picks it up). No stale egg-info/binaries to clean. | `cargo build` / `just check`. |

**The canonical question — after every file in the repo is updated, what runtime systems still have the old shape?** Only the on-disk v1 session files of users who reviewed in Phases 65–68. The load-path migration (below) is the one and only place that must handle them. There is no way to "migrate all files at once" because filenames are per-repo hashes the app only resolves when that repo is opened — so migration MUST be lazy, on the `load_session` path (which is what D-04 specifies).

## Common Pitfalls

### Pitfall 1: v1→v2 deserialization corrupts existing sessions (CRITICAL)
**What goes wrong:** `review_store.rs:128` does `serde_json::from_value::<ReviewSession>(value)`; on ANY error it calls `quarantine_corrupt` (renames to `.corrupt`) and returns `RecoveredCorrupt`. If `Comment` gains a non-`#[serde(default)]` field (`id`), a v1 file — whose comments lack that field — fails `from_value` and is quarantined. The user's review silently moves to `.corrupt` and a blank session is created. That is D-15's recovery, not D-04's migration.
**Why it happens:** serde requires all non-optional, non-defaulted fields to be present in the input.
**How to avoid (two viable shapes — planner picks ONE):**
- **(A) `#[serde(default)]` + normalize pass.** Mark `id: String` and `commit_oid: Option<String>` with `#[serde(default)]`. `id` deserializes to `""` for v1 files. After a successful `from_value`, run a normalize step that assigns a fresh uuid to any comment with empty `id`, then re-save if anything changed. Simplest; downside: the `""`-means-unbackfilled sentinel lives in the type forever, so every read site must trust the backfill ran first.
- **(B) `Value`-level migration before `from_value`.** Between the version check (`line 126`) and `from_value` (`line 128`): if `version == 1`, walk `value["comments"]` as a `serde_json::Value`, inject a uuid into each element missing `id`, then `from_value` into a strict (non-defaulted-`id`) v2 struct. No sentinel pollution; explicit and unit-testable; slightly more code. `commit_oid` still needs `#[serde(default)]` (or `Option`, which already defaults to `None`) since v1 comments simply omit it — `Option<T>` deserializes a missing field to `None` without `#[serde(default)]`.
**Recommendation:** **(A)** for the smallest diff, OR **(B)** if the team prefers no sentinel. Note: `commit_oid: Option<String>` needs NO `#[serde(default)]` — serde treats a missing field as `None` for `Option` automatically — so under (B) only the `id` field needs migration. Under (A), `id: String` (non-Option) is the only field needing `#[serde(default)]`.
**Warning signs:** a `.corrupt` sidecar appears after opening a pre-Phase-69 session; the panel shows an empty session for a repo that had comments. Write a regression test: persist a hand-built v1 JSON (no `id`), `load_session`, assert `Loaded` (not `RecoveredCorrupt`) and that every comment has a non-empty `id`.

### Pitfall 2: Breaking D-16 (newer-version refusal) with the bump
**What goes wrong:** Bumping `CURRENT_SCHEMA_VERSION` to 2 is correct, but the version gate (`review_store.rs:124`) must keep refusing version > 2 untouched. If the migration is added before the `version > CURRENT_SCHEMA_VERSION` check, a future v3 file could be mangled.
**How to avoid:** Keep the order: read → `version > CURRENT_SCHEMA_VERSION` ⇒ `RefusedNewer` (untouched) → only THEN migrate v1→v2 → `from_value`. The migration runs strictly for `version <= 2`. The existing `resume_review_session` branch already maps `RefusedNewer` to a `newer_version` error (`review.rs:688-696`) — leave it intact.
**Warning signs:** a v3 file gets rewritten/quarantined instead of refused.

### Pitfall 3: Backfilled ids not persisted → unstable across reloads
**What goes wrong:** If ids are backfilled only in memory and not re-saved, every reload mints new ids, so an `edit_comment`/`delete_comment` issued against a previously-shown id misses after a reload (D-03 demands stability for the comment's lifetime).
**How to avoid:** After backfill, re-`save_session` (the migration writes v2 back to disk, so the next load is a clean v2 read). This also "completes" the migration: the file is upgraded once.
**Warning signs:** edit/delete intermittently does nothing after a tab reload.

### Pitfall 4: Side semantics in the resolvability bound check
**What goes wrong:** The line-range bound check reads the wrong tree, giving false orphans or false positives.
**Why it happens:** Anchor line numbers are SOURCE coordinates with a `side` discriminator (Phase 67). `Source::FullFile` is ALWAYS `Side::New` (`full-file-anchor.ts:60`), so it reads the commit's own tree. `Source::Diff` can be `Side::Old` (read the **parent's** tree) or `Side::New` (the commit's tree).
**How to avoid:**
- `Side::New` → blob at `commit.tree()?.get_path(file_path)`.
- `Side::Old` → blob at `commit.parent(0)?.tree()?.get_path(file_path)`; a root commit (no parent) ⇒ file gone on the old side.
- Line count: `String::from_utf8_lossy(blob.content()).lines().count()`. Note `str::lines()` does NOT count a trailing newline as a final empty line (a 10-line file with trailing `\n` → 10). Treat the comment resolvable when `start_line >= 1 && end_line <= lines_count` (1-based, matching capture). Confirm against Phase 67/68 capture semantics when implementing.
**Warning signs:** a comment on the last line of a file shows "line out of range"; an old-side deletion comment never resolves.

### Pitfall 5: Per-keystroke or per-edit emit storms
**What goes wrong:** Emitting `session-changed` on every keystroke of the inline editor would cause reload storms (the same reason `save_draft_comment` does NOT emit — `review.rs:537-539`).
**How to avoid:** Only `add_commit_comment`, `edit_comment` (on save), and `delete_comment` emit. Inline editing is local component state until "save". `list_session_comments` and `resolve_session_comments` are reads — never emit.

## Code Examples

### Resolvability classifier (pure, TDD this)
```rust
// New, in commands/review.rs — pure resolver over an in-process git2::Repository.
// Mirrors the spawn_blocking + fresh-repo pattern of list_session_commits.
// Returns, per comment id, a status the panel maps to a badge (D-08).
pub enum OrphanReason { CommitGone, FileGone, LineOutOfRange }   // serialize PascalCase, NO rename_all

pub struct CommentResolution {                                   // Serialize, snake_case fields
    pub id: String,
    pub resolvable: bool,
    pub reason: Option<OrphanReason>,   // None when resolvable
}

// for each comment:
//   commit-level (commit_oid: Some, anchor: None)  → resolvable iff find_commit(commit_oid) ok, else CommitGone
//   line-anchored (anchor: Some)                   → find_commit → tree-by-side → get_path → line bound
// (commit-level comments only need the commit to exist; no file/line check.)
```

### git2 blob-by-path + line count
```rust
// Source patterns: diff.rs:406-407 (find_commit + tree()), merge_editor.rs:57 (find_blob),
//                  git2 0.19 Tree::get_path (docs.rs/git2/0.19.0).
let commit = repo.find_commit(oid)?;                 // CommitGone if Err
let tree = match side { Side::New => commit.tree()?,
                        Side::Old => commit.parent(0)?.tree()? };  // root → no parent → FileGone
let entry = tree.get_path(std::path::Path::new(&file_path))?;       // FileGone if Err
let blob = repo.find_blob(entry.id())?;
let n = String::from_utf8_lossy(blob.content()).lines().count() as u32;
let ok = start_line >= 1 && end_line <= n;            // LineOutOfRange otherwise
```

### Jump wiring (frontend — compose existing machinery)
```typescript
// review-session.svelte.ts owns rightPaneMode; jump composes the SAME functions
// RepoView already uses for commit/file selection + scroll (no new backend needed).
// Source: RepoView.svelte:285-318 (handleCommitSelect), :348-369 (handleCommitFileSelect),
//         DiffPanel.svelte:257-265 (scrollToHunk → scrollIntoView + highlight),
//         CommitGraph.svelte scrollToOid (graph scroll).
async function jumpTo(c: Comment) {
  if (!c.anchor) return;                       // commit-level / orphaned → disabled (D-08)
  await handleCommitSelect(c.anchor.commit_oid);     // loads files + detail
  await handleCommitFileSelect(c.anchor.file_path);  // Source::Diff → diff; Source::FullFile → full-file view
  // then scroll DiffPanel to [start_line, end_line] and highlight (reuse scrollToHunk/scrollIntoView)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Comments identified by list position | Stable `id` per comment (D-03) | This phase (v2) | Multi-tab-safe edit/delete. |
| Schema frozen at v1 (no id, anchor-only) | v2: `id` + commit-level `commit_oid` | This phase | One version bump; lazy migration on load. |
| ReviewPanel = throwaway stub (thin bar, lifecycle only) | Real panel: view/add/edit/delete/jump in the right pane | This phase | Replaces `ReviewPanel.svelte` + relocates from `App.svelte:592-594`. |

**Deprecated/outdated:**
- The `review-toggle` menu item + `reviewPanelOpen` thin-bar gating (`lib.rs:27,67`, `App.svelte:592`) is the stub's trigger. Reconcile with the right-pane placement — the planner decides whether the menu item stays as the persistent "Review" toggle (D-07) or is reworked.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `uuid` v4 crate is the right id generator and a `v4`-featured 1.x is current on crates.io | Standard Stack / Don't Hand-Roll | Low — `uuid` is a top-tier crate; only the exact version/feature string needs `cargo search` confirmation before pinning. |
| A2 | Line-range bound semantics are 1-based with `end_line <= lines_count` matching Phase 67/68 capture | Pitfall 4 / git2 example | Medium — if capture used a different convention, off-by-one false orphans. Confirm against `buildDiffAnchor`/`buildFullFileAnchor` and the diff-line numbering when implementing. |
| A3 | The "right pane" the panel claims is resolvable without a large RepoView refactor | Open Question 1 | Medium — ROADMAP says "replaces CommitDetail/DiffPanel" but those live in DIFFERENT panes; see Open Question 1. |

**Not assumed (verified this session):** the serde corruption trap (read `review_store.rs:104-135`), `Tree::get_path` existence (docs.rs git2 0.19), the absence of a read-comments command and of a `uuid` dep, the `_inner`+RMW+emit pattern, the confirm-dialog pattern, the selection/scroll machinery.

## Open Questions

1. **Which pane does the real panel claim?** (HIGH importance — planner must resolve.)
   - What we know: `RepoView.svelte` has a **center pane** (`flex-1`, line 721) rendering `DiffPanel`/`MergeEditor`/`CommitGraph`, and a **rightmost pane** (line 781) rendering `CommitDetail`/`StagingPanel`. ROADMAP §69 says the panel "replaces CommitDetail/DiffPanel content when Review mode active" — but those are in two different panes.
   - What's unclear: does the panel claim the center pane (so jump swaps it back to `DiffPanel`, and `CommitGraph` must go somewhere), the right pane (replacing `CommitDetail`, smaller canvas), or become its own Review mode that hides one pane?
   - Recommendation: The panel claims the **center pane** in Review mode (where `DiffPanel` already lives), because D-07's jump is exactly "swap THIS pane from panel → diff and back". `CommitGraph` continues to occupy the center pane when Review mode is off / no file selected; the `review-session.svelte.ts` rune gates which of {panel, diff, graph} the center pane shows. This keeps jump a single-pane swap and avoids touching the rightmost `CommitDetail`/`StagingPanel` pane. Confirm in discuss-phase or let the planner lock it as a plan decision.

2. **Commit-level write: extend `add_comment` or sibling command?**
   - Recommendation: **Sibling `add_commit_comment(path, commit_oid, text)`**. Extending `add_comment` to optional anchor changes the wire shape of a shipped command and forces edits to all 7 Phase 67/68 tests (`review.rs:1187-1394`) and the TS `AddCommentRequest`. A sibling is purely additive (CONTEXT code_context: "must NOT break existing line-anchored callers"). Both route through `mutate_session_rmw` + emit.

3. **Edit/delete of a missing id: error or no-op?**
   - Recommendation: delete = idempotent no-op (parity with `apply_remove`, `review.rs:232`); edit of a missing id = `not_found` `TrunkError` (the user expected a specific comment to exist). Planner's call; low risk.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| git2 (vendored libgit2) | resolvability check | ✓ | 0.19 | — |
| @tauri-apps/plugin-dialog | delete confirm | ✓ | 2 | — |
| uuid (Rust) | id generation | ✗ (not yet a dep) | — | Hand-rolled FNV(timestamp+counter) — NOT recommended (see Don't Hand-Roll); add the crate. |
| cargo / just | build + `just check` | ✓ | — | — |

**Missing dependencies with no fallback:** none.
**Missing dependencies with fallback:** `uuid` — add to Cargo.toml (strong case; do not hand-roll).

## Validation Architecture

> nyquist_validation enabled. The three pure units below are the RED/GREEN/REFACTOR targets; glue/UI is verified by behavior.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust: `cargo test` (in-process `tempfile::TempDir` + `git2::Repository::init`, no mocks — see `review.rs:770-836`). Frontend: Vitest + Testing Library (`*.test.ts`, e.g. `DiffPanel.test.ts`). |
| Config file | `src-tauri/Cargo.toml` (`[dev-dependencies] tempfile`, `tauri test` feature); `vitest` via repo `package.json` / biome. |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml review` ; `npm run test -- ReviewPanel` (adjust to repo script) |
| Full suite command | `just check` (fmt, biome, svelte-check, clippy, cargo-test, vitest) |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| (migration) | v1 session (no `id`) loads as `Loaded` with backfilled ids, NOT `RecoveredCorrupt` | unit | `cargo test --manifest-path src-tauri/Cargo.toml review_store` | ❌ Wave 0 |
| (migration) | v3 (newer) session still `RefusedNewer` after the bump (D-16) | unit | `cargo test … review_store` | ⚠️ extend existing recovery tests |
| ANCH-03 | `add_commit_comment` persists a comment with `commit_oid: Some`, `anchor: None`, fresh `id` | unit | `cargo test … review` | ❌ Wave 0 |
| CMT-01 | `list_session_comments` returns all comments incl. ids; `no_session` when absent | unit | `cargo test … review` | ❌ Wave 0 |
| CMT-02 | `edit_comment` updates text by id; missing id → `not_found`; survives concurrent edits | unit | `cargo test … review` | ❌ Wave 0 |
| CMT-03 | `delete_comment` removes by id; missing id is no-op; emits/persists | unit | `cargo test … review` | ❌ Wave 0 |
| CMT-04 | resolvability classifier: commit gone / file gone / line out of range / resolvable, per `side` | unit (in-process repo) | `cargo test … review` | ❌ Wave 0 |
| CMT-03 | delete shows `ask` confirm; cancel aborts; confirm invokes `delete_comment` | component | vitest `ReviewPanel.test.ts` (mock dialog + invoke) | ❌ Wave 0 |
| CMT-04 | jump on resolvable selects commit+file & swaps pane; orphan row disables jump & shows badge | component | vitest `ReviewPanel.test.ts` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test … review` (+ relevant vitest file)
- **Per wave merge:** `just check`
- **Phase gate:** `just check` green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/git/review_store.rs` tests — v1→v2 migration + id backfill + D-16 preserved (extend existing `#[cfg(test)]` mod).
- [ ] `src-tauri/src/commands/review.rs` tests — `add_commit_comment`, `list_session_comments`, `edit_comment`, `delete_comment`, resolvability classifier (reuse `seeded_sessions`, `make_repo` helpers at `review.rs:808,1151`).
- [ ] `src/components/ReviewPanel.test.ts` — panel render, group-by-commit, inline edit, delete-confirm, jump vs orphan (new file).
- [ ] No framework install needed — `tempfile`, `tauri test`, vitest all present.

## Security Domain

> security_enforcement default (enabled). This phase adds no auth/network/crypto surface; it persists local JSON and reads local git objects.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Single-user desktop app, no auth. |
| V3 Session Management | no | "Session" here is a review session, not an auth session. |
| V4 Access Control | no | Local files only; no multi-user boundary. |
| V5 Input Validation | yes | `commit_oid`/`file_path`/`id` come from the frontend → backend. `commit_oid` parsed via `git2::Oid::from_str` (rejects malformed). `file_path` is metadata used in `Tree::get_path` (a tree lookup, NOT a filesystem path) — cannot escape the repo object DB. Session filename is the FNV-1a hash of the canonical path, never the `file_path` (path-traversal mitigation already tested at `review.rs:1352-1394`). Empty-text comments disabled (CONTEXT discretion). |
| V6 Cryptography | no | uuid v4 is identity, not a security token; no crypto requirement. |

### Known Threat Patterns for Tauri desktop + git2
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `file_path` into the filesystem | Tampering / EoP | `file_path` is only ever passed to `Tree::get_path` (object-DB lookup), never `std::fs`. Session filename = FNV-1a hash, not user path. Already mitigated + tested. |
| Malformed `commit_oid` crashing the resolver | DoS | Parse with `git2::Oid::from_str` → `invalid_oid` `TrunkError`; classify unparseable/unknown as `CommitGone`, never panic (mirror `intersect_graph_order`'s never-drop fallback, `review.rs:269-285`). |
| Concurrent multi-tab writes losing data | Tampering | `mutate_session_rmw` mutex + atomic write (existing, proven by 50-thread test). |

## Sources

### Primary (HIGH confidence)
- `src-tauri/src/git/types.rs:288-336` — review schema (`Source`, `Side`, `Anchor`, `Comment`, `DraftComment`, `ReviewSession`); serde conventions in the header comment.
- `src-tauri/src/git/review_store.rs` — `save_session`/`load_session`/`delete_session`, `CURRENT_SCHEMA_VERSION`, the version-gate + `.corrupt` quarantine state machine (the migration hook point, lines 104-135).
- `src-tauri/src/commands/review.rs` — `_inner`+RMW+emit pattern (`add_comment_inner:390`, `mutate_session_rmw:304`), `list_session_commits:569` (only read), `add_comment:511`, `AddCommentRequest:368`, recovery branches `resume_review_session:647-700`, the full `#[cfg(test)]` suite + helpers (`make_repo:808`, `seeded_sessions:1151`).
- `src-tauri/src/lib.rs:79-162` — `invoke_handler` (where the 5 new commands register); `review-toggle` menu + emit (27,67).
- `src-tauri/src/commands/merge_editor.rs:54-62` — `repo.find_blob(e.id)` blob-read pattern.
- `src-tauri/src/commands/diff.rs:406-407,501` — `find_commit` + `commit.tree()` patterns.
- `src/components/ReviewPanel.svelte` — the stub to replace; `session-changed` listener + canonical filter (81-93).
- `src/components/RepoView.svelte:86-131,285-369,685-799` — selection state, `handleCommitSelect`/`handleCommitFileSelect`, the two-pane render.
- `src/components/DiffPanel.svelte:257-265,385-386,505-510` — `scrollToHunk`/`scrollIntoView`+highlight; `ask` confirm.
- `src/lib/full-file-anchor.ts:9-66` — `Source::FullFile` is always `Side::New`, absolute 1-based new-side line numbers.
- `src/lib/types.ts:288-353` — TS DTO mirrors (`Comment`, `Anchor`, request shapes).
- `src/App.svelte:587-615` — `reviewPanelOpen` thin-bar render (to relocate).
- `.planning/codebase/CONVENTIONS.md` — serde/naming/styling/error-handling conventions.

### Secondary (MEDIUM confidence)
- docs.rs/git2/0.19.0 `Tree` — confirmed `get_path(&Path) -> Result<TreeEntry, Error>` exists (blob conversion via `find_blob(entry.id())`, matching in-repo usage).

### Tertiary (LOW confidence)
- `uuid` crate current version/feature string — needs `cargo search uuid` before pinning (A1).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all reuse is verified in-repo; only `uuid` version needs a registry check.
- Architecture/patterns: HIGH — every pattern cited to a concrete file:line already shipped & tested.
- Migration/serde trap: HIGH — verified against the exact `review_store.rs:128` quarantine behavior.
- Pane placement: MEDIUM — genuine ROADMAP ambiguity, surfaced as Open Question 1 with a recommendation.
- Line-bound semantics: MEDIUM — recommend confirming the 1-based convention against capture code during implementation (A2).

**Research date:** 2026-05-26
**Valid until:** 2026-06-25 (stable in-repo brownfield; the only external moving part is the `uuid` crate version).
