# Phase 76: Wire MessageEditor into merge/continue, merge, and revert - Pattern Map

**Mapped:** 2026-05-29
**Files analyzed:** 9 (4 backend commands/registration, 5 frontend)
**Analogs found:** 8 / 9 (one new tagged enum has no codebase analog)

> **editor.rs is intentionally UNUSED (D-01a).** `src-tauri/src/git/editor.rs` is dead code this phase by decision. Do NOT map any new file to it as an analog to "wire in." The mechanism is direct `git commit -m --cleanup=strip`, not the GIT_EDITOR script.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/commands/operation_state.rs` — `get_merge_message` (NEW) | command (query) | request-response (file read) | `get_operation_state` (read wrapper) + `operation_state.rs:59` (MERGE_MSG read) | role-match (query, not mutation) |
| `src-tauri/src/commands/operation_state.rs` — `merge_branch_begin` (NEW, replaces `merge_branch`) | command (mutation) | request-response / 2-step | `merge_branch_inner` + `merge_branch` wrapper (:292-318, :494-513) | exact-shape, deviates at cache.insert (see assignment) |
| `src-tauri/src/commands/operation_state.rs` — `merge_continue` finish path (MODIFIED) | command (mutation) | event-driven (finalize commit) | itself, `Some(msg)` arm `merge_continue_inner:158-165` | exact (in-place edit) |
| `src-tauri/src/commands/operation_state.rs` — `MergeBeginResult` enum (NEW) | type | — | **none** (no internally-tagged enum in codebase) | **no analog** |
| `src-tauri/src/commands/commit_actions.rs` — `revert_commit_begin` (NEW, replaces `revert_commit`) | command (mutation) | request-response / 2-step | `revert_commit_inner` + `revert_commit` wrapper (:143-171, :320-340) | exact-shape, deviates at cache.insert |
| `src-tauri/src/commands/commit_actions.rs` — `revert_continue` (NEW) | command (mutation) | event-driven (finalize commit) | `merge_continue_inner:158-165` (`git commit -m` path) | exact (sibling op) |
| `src-tauri/src/commands/commit_actions.rs` — `revert_abort` (NEW) | command (mutation) | request-response | `merge_abort_inner` + `merge_abort` wrapper (operation_state.rs:183-202, :410-429) | exact |
| `src-tauri/src/lib.rs` — handler registration (MODIFIED) | config | — | existing `invoke_handler!` list (:144-163) | exact |
| `src/components/RepoView.svelte` — host `<MessageEditor>`, thread `onopenmessageeditor` (MODIFIED) | provider/host | event-driven (callback) | `onopenrebaseeditor` host+thread (`handleOpenRebaseEditor:567`, props :805/:888) | exact |
| `src/components/CommitGraph.svelte` — route `handleRevert` (:541) + `handleMergeBranch` (:590) (MODIFIED) | component | request-response | local `handleInteractiveRebaseBranch:610` (already routes via `onopenrebaseeditor`) | exact |
| `src/components/BranchSidebar.svelte` — route `handleMergeBranch` (:395) (MODIFIED) | component | request-response | CommitGraph `handleInteractiveRebaseBranch:610` | exact |
| `src/components/StagingPanel.svelte` — REPLACE inline merge form (MODIFIED) | component | event-driven | the inline form itself (state :573-585, `commitMerge:587`, UI :1302-1362) — to be removed/rerouted | self (deletion target) |
| `src/components/OperationBanner.svelte` — add Revert Continue/Abort (MODIFIED) | component | request-response | the `{#if isRebase}` button block (:135-180), `handleAbort:58`, `handleContinue:30` | role-match |

## Pattern Assignments

### `get_merge_message` (operation_state.rs, NEW — QUERY)

**Analog:** `get_operation_state` wrapper (operation_state.rs:348-372) — this is a READ command. It does NOT `cache.insert` and does NOT `app.emit("repo-changed")`. Do not copy the mutation-wrapper shape here.

**The actual file read** (already present at operation_state.rs:59 inside `get_operation_state_inner`):
```rust
let merge_msg = std::fs::read_to_string(git_dir.join("MERGE_MSG")).ok();
```

**Wrapper shape to copy** (read-only — no AppHandle, no cache):
```rust
// Source shape: get_operation_state (operation_state.rs:348-372)
#[tauri::command]
pub async fn get_merge_message(
    path: String,
    state: State<'_, RepoState>,
) -> Result<Option<String>, String> {
    let state_map = state.0.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let repo = open_repo(&path, &state_map)?;
        Ok(std::fs::read_to_string(repo.path().join("MERGE_MSG")).ok())
    })
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e: TrunkError| serde_json::to_string(&e).unwrap())
}
```
Strip nothing here — `--cleanup=strip` handles `# Conflicts:` at commit time (RESEARCH OQ-3/Pitfall 1).

---

### `merge_branch_begin` (operation_state.rs, NEW — replaces `merge_branch`, MUTATION 2-step)

**Analog:** `merge_branch_inner` (operation_state.rs:292-318) for the subprocess body; `merge_branch` wrapper (:494-513) for the async shape.

**ff probe + no-commit body** (restructure `merge_branch_inner:300-318`; remove `--no-edit` :301 and `GIT_EDITOR` :304):
```rust
// 1. probe ff (RESEARCH OQ-1): exit 0 => fast-forwarded, no editor
let probe = std::process::Command::new("git")
    .args(["merge", "--ff-only", branch])
    .current_dir(path_buf)
    .env("PATH", shell_env::system_path())
    .output()
    .map_err(|e| TrunkError::new("merge_error", e.to_string()))?;
if probe.status.success() {
    let mut repo = git2::Repository::open(path_buf)?;
    let graph = graph::walk_commits(&mut repo, 0, usize::MAX)?;
    return Ok(MergeBeginResult::FastForwarded { graph });
}
// 2. non-ff: run --no-commit, then read MERGE_MSG / detect conflict
```

**Conflict-branch analog — emit-on-all-paths is already proven here** (merge_branch_inner:307-315): on conflict the inner returns `Ok(graph)` (NOT `Err`) so the single wrapper emit fires. Mirror exactly — return `MergeBeginResult::Conflicts { graph }`, never an error, on conflict.

**Wrapper deviation — DO NOT copy `cache.insert` verbatim.** The mutation analog inserts `graph_result` directly:
```rust
// merge_branch wrapper (:510) — graph_result IS a GraphResult here
cache.0.lock().unwrap().insert(path.clone(), graph_result);
```
The begin inner now returns `MergeBeginResult` (not `GraphResult`), so extract `.graph` from the variant FIRST, then insert + emit for **every** variant (ff / conflicts / ready — RESEARCH Pitfall 4 / finding 7). The `emit("repo-changed")` (:511) must fire on all three:
```rust
let graph = match &result { // borrow graph out of whichever variant
    MergeBeginResult::FastForwarded { graph }
    | MergeBeginResult::Conflicts { graph }
    | MergeBeginResult::Ready { graph, .. } => graph.clone(),
};
cache.0.lock().unwrap().insert(path.clone(), graph);
let _ = app.emit("repo-changed", path); // EVERY variant — never skip
Ok(result)
```

---

### `merge_continue` finish path (operation_state.rs, MODIFIED in place)

**Analog:** itself. Keep the `Some(msg)` arm (`merge_continue_inner:158-165`); **remove the `None` else-branch** (:166-174) that sets `GIT_EDITOR=true`. Add `--cleanup=strip` (RESEARCH OQ-3, Pitfall 1):
```rust
// merge_continue_inner:160-165 — keep this arm, add --cleanup=strip
std::process::Command::new("git")
    .args(["commit", "-m", msg, "--cleanup=strip"])
    .current_dir(path_buf)
    .env("PATH", shell_env::system_path())
    .output()
    .map_err(|e| TrunkError::new("merge_error", e.to_string()))?
```
After removing the else-branch, `message: Option<&str>` always carries `Some` in practice (frontend aborts on null and never invokes). The async wrapper (`merge_continue:389-408`) is unchanged — it already does `cache.insert` + `emit`. This same command finishes both merge --continue (StagingPanel) and the merge `<branch>` `Ready` flow.

---

### `revert_commit_begin` (commit_actions.rs, NEW — replaces `revert_commit`, MUTATION 2-step)

**Analog:** `revert_commit_inner` (commit_actions.rs:143-171). Change `["revert", oid, "--no-edit"]` (:153) → `["revert", "--no-commit", oid]`. **Keep the `conflict_state` error branch (:159-166) exactly** — conflicted revert returns `Err(conflict_state)`, existing handling takes over (RESEARCH OQ-4 / diagram).

On clean success, return the default message read from MERGE_MSG (same read as `get_merge_message`):
```rust
let mut repo = git2::Repository::open(path_buf)?;
let message = std::fs::read_to_string(repo.path().join("MERGE_MSG")).ok();
let graph = graph::walk_commits(&mut repo, 0, usize::MAX)?;
Ok((graph, message)) // or a small struct; wrapper emits repo-changed (REVERT_HEAD now set)
```

**Wrapper:** copy `revert_commit` (commit_actions.rs:320-340). It already does `cache.insert` + `emit("repo-changed")`. Because begin mutates (`REVERT_HEAD` set + staged) BEFORE the editor opens, the emit on begin is mandatory so the banner surfaces even if the user cancels (RESEARCH Pitfall 2/4). Extract `.0`/graph for the insert, return message to frontend.

---

### `revert_continue` (commit_actions.rs, NEW — MUTATION finalize)

**Analog:** `merge_continue_inner`'s `Some(msg)` arm (operation_state.rs:158-165) — symmetric sibling. RESEARCH Anti-Patterns: do NOT reuse `merge_continue` to finish a revert; add a symmetric command (matches the `merge_continue`/`rebase_continue` family).
```rust
std::process::Command::new("git")
    .args(["commit", "-m", msg, "--cleanup=strip"])
    .current_dir(path_buf)
    .env("PATH", shell_env::system_path())
    .output()
    .map_err(|e| TrunkError::new("revert_error", e.to_string()))?
```
`git commit -m` clears `REVERT_HEAD` (RESEARCH OQ-3, verified). Wrapper: copy `merge_continue` (operation_state.rs:389-408), rename, `revert_error` code.

---

### `revert_abort` (commit_actions.rs, NEW — MUTATION)

**Analog:** `merge_abort` — near-exact. Inner = `merge_abort_inner` (operation_state.rs:183-202); wrapper = `merge_abort` (:410-429).
```rust
// inner: copy merge_abort_inner:190-195, swap args + error code
let output = std::process::Command::new("git")
    .args(["revert", "--abort"])
    .current_dir(path_buf)
    .env("PATH", shell_env::system_path())
    .output()
    .map_err(|e| TrunkError::new("revert_error", e.to_string()))?;
if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(TrunkError::new("revert_error", stderr.to_string()));
}
let mut repo = git2::Repository::open(path_buf)?;
graph::walk_commits(&mut repo, 0, usize::MAX)
```
Wrapper: copy `merge_abort:410-429` (the no-arg-besides-path `spawn_blocking → cache.insert → emit` shape). `git revert --abort` clears `REVERT_HEAD` + clean tree (RESEARCH "Don't Hand-Roll").

---

### `lib.rs` handler registration (MODIFIED — CONFIG)

**Analog:** existing `invoke_handler![...]` rows (lib.rs:144-163). Add the new commands as sibling lines. Backend commands are invisible to `invoke(...)` until registered:
```rust
// after operation_state::merge_branch (:162) and commit_actions block (:145):
commands::operation_state::get_merge_message,
commands::operation_state::merge_branch_begin,   // replaces merge_branch (:162) if old removed
commands::commit_actions::revert_commit_begin,    // replaces revert_commit (:145) if old removed
commands::commit_actions::revert_continue,
commands::commit_actions::revert_abort,
```
Decide per-command whether the old `merge_branch`/`revert_commit` registration lines are removed or kept (they become unused once trigger sites move to `*_begin`).

---

### `RepoView.svelte` — host MessageEditor + thread callback (MODIFIED)

**Analog:** `onopenrebaseeditor` host+thread, EXACTLY.
- Host handler: `handleOpenRebaseEditor` (RepoView.svelte:567) — the RepoView-owned async function the children call back into.
- Prop threading: `onopenrebaseeditor={handleOpenRebaseEditor}` passed to `BranchSidebar` (:805) and `CommitGraph` (:888).

Mirror it:
1. Import + host a single instance (NOT yet imported — confirmed `MessageEditor` absent from RepoView): `import MessageEditor from "./MessageEditor.svelte";` and `<MessageEditor bind:this={messageEditorRef} title={editorTitle} />` near the end of the template.
2. `title` is a `$props()` set BEFORE `open()` (verified `MessageEditor.svelte:1-6`; A1). Use a `$state` var:
```svelte
let messageEditorRef = $state<MessageEditor | null>(null);
let editorTitle = $state("Merge commit message");
async function handleOpenMessageEditor(defaultValue: string, title: string): Promise<string | null> {
  editorTitle = title;                       // set reactive title first (D-03)
  return (await messageEditorRef?.open(defaultValue)) ?? null;
}
```
3. Thread `onopenmessageeditor={handleOpenMessageEditor}` to `CommitGraph` (:888), `BranchSidebar` (:805), and `StagingPanel` (:906) — mirroring the `onopenrebaseeditor` rows. StagingPanel re-threads it down to OperationBanner (Open Question 2 / finding for Revert Continue).

The `MessageEditor.open()` contract (frozen Phase 75, `MessageEditor.svelte:14-23`): `open(default) → Promise<string|null>`, returns `null` on Cancel/Esc/backdrop/empty/whitespace (`:33-35`). Single null check covers all aborts (D-02): `if (result === null) return;` — begin already emitted, so the recoverable state is visible.

---

### `CommitGraph.svelte` — route revert + merge through editor (MODIFIED)

**Analog:** `handleInteractiveRebaseBranch` (CommitGraph.svelte:610) — already fetches a backend value then calls `onopenrebaseeditor?.(...)`. Same pattern, new callback.

Current direct calls to rewrite:
- `handleRevert` (:541-556) currently `await safeInvoke("revert_commit", {...})` (:544). New: `revert_commit_begin` → on clean, `await onopenmessageeditor?.(defaultMsg, "Revert commit message")` → if non-null `revert_continue`; if null return (recoverable).
- `handleMergeBranch` (:590-598) currently `await safeInvoke("merge_branch", {...})` (:592). New: `merge_branch_begin` → discriminate on `result.kind`.

**`kind` discrimination analog:** `OperationBanner.svelte:16` — `info.op_type === "Merge"` switches on a backend-returned discriminant string. Mirror for the begin result:
```ts
const result = await safeInvoke<MergeBeginResult>("merge_branch_begin", { path: repoPath, branch });
if (result.kind === "ready") {
  const msg = await onopenmessageeditor?.(result.message, "Merge commit message");
  if (msg == null) return;            // D-02 recoverable; begin already emitted
  await safeInvoke("merge_continue", { path: repoPath, message: msg });
}
// "fast_forwarded" / "conflicts": no editor — repo-changed from begin drives the UI
```
Add `onopenmessageeditor?: (def: string, title: string) => Promise<string | null>` to the component `Props` interface (mirror existing `onopenrebaseeditor` prop decl).

---

### `BranchSidebar.svelte` — route merge through editor (MODIFIED)

**Analog:** identical to CommitGraph's `handleMergeBranch`. BranchSidebar `handleMergeBranch` (:395-405) currently `await safeInvoke("merge_branch", {...})` (:397). Same `merge_branch_begin` → `kind` discrimination → editor → `merge_continue` flow. Keep its existing `loadRefs(repoPath); onrefreshed?.()` post-step (:399-400). Add the `onopenmessageeditor` prop (mirror `onopenrebaseeditor` decl). **Both merge sites must route — no second silent path** (CONTEXT specifics / D-04).

---

### `StagingPanel.svelte` — REPLACE inline merge form (MODIFIED)

**Analog:** the inline form is the deletion target, not a copy source. Remove:
- state `mergeLoading`/`mergeSubject`/`mergeBody` (:573-575) + the pre-fill `$effect` (:578-585) that hardcodes `Merge branch '<src>' into <tgt>` (the divergence RESEARCH finding 5 / :582 fixes).
- `commitMerge` (:587-607).
- the `{:else if isMerge}` UI block (:1302-1362): subject input, body textarea, "Commit and Merge" button gated `disabled={!allResolved}`.

Replace with a merge-continue path that, when conflicts are resolved (`allResolved`), calls `get_merge_message` → `onopenmessageeditor?.(default, "Merge commit message")` → on non-null `merge_continue` with the edited message; on null returns (recoverable). This is the **single live** merge-continue editor (RESEARCH finding 5/6). Thread `onopenmessageeditor` in from RepoView and down to OperationBanner.

> Do NOT wire `OperationBanner:33`'s `isMerge ? "merge_continue"` branch — it is DEAD for merge (its button block is `{#if isRebase}`, never renders for Merge). RESEARCH finding 6 / Anti-Pattern.

---

### `OperationBanner.svelte` — add Revert Continue/Abort (MODIFIED)

**Analog:** the `{#if isRebase}` button block (:135-180) — the three buttons (Continue/Skip/Abort) with `var(--color-success-bg)` / `var(--color-warning-bg)` / `var(--color-danger-bg)` styling are the template. Currently a Revert state falls into the `{:else}` label-only branch (:131-133) → zero buttons (RESEARCH finding 4).

Add an `isRevert` derived (mirror `isMerge`/`isRebase` :16-17) and a button block for it:
- **Abort** — self-contained, mirror `handleAbort` (:58-84): confirm dialog → `safeInvoke("revert_abort", { path: repoPath })` → toast → `onaction?.()`.
- **Continue** — must reach RepoView's `handleOpenMessageEditor`. OperationBanner is rendered inside StagingPanel (StagingPanel.svelte:909 with `info`/`repoPath`/`onaction` props). Add an `onopenmessageeditor` (or `oncontinuerevert`) callback prop, threaded RepoView → StagingPanel → OperationBanner (Open Question 2). Continue handler: `get_merge_message` → editor → `revert_continue` on non-null.

Continue handler `handleContinue` (:30-43) currently branches `isMerge ? "merge_continue" : "rebase_continue"` — extend for the revert case (route through the modal callback, not a direct `safeInvoke`).

## Shared Patterns

### Backend mutation-command wrapper (`spawn_blocking → cache.insert → emit`)
**Source:** every wrapper in `operation_state.rs` / `commit_actions.rs`; canonical example `merge_abort` (operation_state.rs:410-429), `merge_continue` (:389-408).
**Apply to:** `merge_branch_begin`, `revert_commit_begin`, `revert_continue`, `revert_abort`.
```rust
let state_map = state.0.lock().unwrap().clone();
let path_clone = path.clone();
let graph_result = tauri::async_runtime::spawn_blocking(move || xxx_inner(&path_clone, &state_map))
    .await
    .map_err(|e| serde_json::to_string(&TrunkError::new("spawn_error", e.to_string())).unwrap())?
    .map_err(|e| serde_json::to_string(&e).unwrap())?;
cache.0.lock().unwrap().insert(path.clone(), graph_result);
let _ = app.emit("repo-changed", path);
Ok(())
```
**Deviation for the begin commands:** the inner returns `MergeBeginResult` / `(GraphResult, message)`, NOT a bare `GraphResult`. Extract the graph for `cache.insert`, return the message/variant to the frontend, and emit `repo-changed` on **every** outcome (begin mutates the repo before the editor opens — RESEARCH findings 4/7). `get_merge_message` is the exception: it is a QUERY (analog `get_operation_state:348-372`) — no cache.insert, no emit.

### Subprocess git (CLAUDE.md sanctioned exception)
**Source:** `merge_continue_inner:160-173`, `revert_commit_inner:152-157`, `merge_abort_inner:190-195`.
**Apply to:** all new subprocess calls (`merge --ff-only`, `merge --no-commit`, `commit -m --cleanup=strip`, `revert --no-commit`, `revert --abort`).
```rust
std::process::Command::new("git")
    .args([...])
    .current_dir(path_buf)
    .env("PATH", shell_env::system_path())   // PATH-hijack mitigation — preserve on every call
    .output()
    .map_err(|e| TrunkError::new("<op>_error", e.to_string()))?;
```
Message text passes as a single argv element (`["commit", "-m", msg, ...]`) — no shell, no injection (RESEARCH Security V5).

### TrunkError with typed `code`
**Source:** `merge_error` (operation_state.rs), `revert_error` / `conflict_state` (commit_actions.rs:161-166).
**Apply to:** all new commands. Preserve `conflict_state` on conflicted revert; use `merge_error`/`revert_error` otherwise.

### Frontend callback-thread to a RepoView-owned modal
**Source:** `onopenrebaseeditor` — `handleOpenRebaseEditor` (RepoView.svelte:567), threaded at :805/:888.
**Apply to:** `onopenmessageeditor` threaded to CommitGraph, BranchSidebar, StagingPanel (→ OperationBanner). Single `<MessageEditor bind:this>` host in RepoView; reactive `$state` title set before `open()` (D-03/D-04, A1).

### Frontend discriminant-string switching
**Source:** `OperationBanner.svelte:16` (`info.op_type === "Merge"`).
**Apply to:** `merge_branch_begin` result — `result.kind === "ready" | "fast_forwarded" | "conflicts"`.

### CSS custom properties only (no inline colors)
**Source:** MessageEditor.svelte (already compliant), OperationBanner button styles (:140-178 use `var(--color-success-bg)` etc.).
**Apply to:** the new Revert button block in OperationBanner — reuse the rebase button `var(--color-*)` tokens; never inline a hex.

## No Analog Found

| File / Item | Role | Reason |
|------|------|--------|
| `MergeBeginResult` tagged enum (operation_state.rs) | type | `grep -rn 'serde(tag' src-tauri/src/` returns **nothing**. Existing enums (`OperationType`, `RefType`, types.rs:8/26/253) are plain unit enums with `#[derive(Serialize)]` — none is an internally-tagged struct-variant enum. Use RESEARCH 76-RESEARCH.md lines 257-268 verbatim (`#[serde(tag = "kind", rename_all = "snake_case")]`, variants `FastForwarded`/`Conflicts`/`Ready`, all carrying `graph: GraphResult`; `Ready` adds `message: String`). Place near `GraphResult` (types.rs:72) or in operation_state.rs. Do NOT force a unit enum as its analog. |
| `src-tauri/src/git/editor.rs` | — | **Intentionally unused (D-01a).** Do NOT map any new file to it as a wire-in target. Mechanism is direct `git commit -m --cleanup=strip`. A reviewer must not "fix" this phase by routing through editor.rs; do not delete it either (preserves Phase 75 tests). |

## Metadata

**Analog search scope:** `src-tauri/src/commands/` (operation_state.rs, commit_actions.rs read in full), `src-tauri/src/lib.rs` (handler list), `src-tauri/src/git/types.rs` (enum shapes), `src/components/` (MessageEditor, OperationBanner read in full; RepoView, CommitGraph, BranchSidebar, StagingPanel read at trigger sites).
**Files scanned:** 9 source files + global tagged-enum grep.
**Pattern extraction date:** 2026-05-29
</content>
</invoke>
