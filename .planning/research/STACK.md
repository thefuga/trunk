# Stack Research

**Domain:** v0.6 UI Polish & Core Ops additions to Tauri 2 + Svelte 5 Git GUI
**Researched:** 2026-03-15
**Confidence:** HIGH

## Core Finding: One New npm Package + Zero New Rust Crates

v0.6 needs exactly **one new dependency** (`@lucide/svelte` for icons). All git operations (discard, branch delete, tag delete) use existing `git2 = "0.19"` APIs. The dialog/notification system uses existing `@tauri-apps/plugin-dialog` plus a custom ~30 LOC toast component.

---

## Recommended Stack Additions

### Icon Library: `@lucide/svelte`

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `@lucide/svelte` | `^0.577.0` | SVG icon components throughout the app | Svelte 5-native package (separate from `lucide-svelte` which targets Svelte 4). 1500+ icons, fully tree-shakable, renders inline SVG matching the app's existing SVG-heavy architecture. Each icon is a Svelte component with `size`, `color`, `strokeWidth` props. |

**Why `@lucide/svelte` over alternatives:**

| Recommended | Alternative | Why Not |
|-------------|-------------|---------|
| `@lucide/svelte` | `lucide-svelte` | `lucide-svelte` is the Svelte 4 compatible version. `@lucide/svelte` is the official Svelte 5 package using `$props()` runes natively. Same icon set, different component internals. |
| `@lucide/svelte` | `@iconify/svelte` | Iconify fetches icons from a remote API at runtime by default — unacceptable for a desktop app that must work offline. Offline mode requires importing individual icon packages which adds complexity. Also, 216 KB unpacked vs Lucide's per-icon tree-shaking. |
| `@lucide/svelte` | `svelte-radix` | Only ~310 Radix icons. Lucide has 1500+. Radix icons have a distinct minimal style that doesn't include git-specific icons (no git-branch, git-commit, tag, etc.). |
| `@lucide/svelte` | Unicode symbols (current) | Current toolbar uses `&#8617;`, `&#8595;`, `&#128230;` etc. These render inconsistently across platforms, can't be sized/colored independently, and look amateur. Lucide provides proper visual consistency. |

**Relevant Lucide icons for v0.6 features:**
- Toolbar: `Undo2`, `Redo2`, `ArrowDown` (pull), `ArrowUp` (push), `GitBranch`, `Package` (stash), `PackageOpen` (pop)
- Staging: `Plus` / `Minus` (stage/unstage), `CirclePlus` / `CircleMinus` (stage all/unstage all), `Trash2` (discard)
- Refs: `GitBranch`, `Tag`, `FolderGit2` (remote), `Layers` (stash)
- Commit form: `GitCommit`, `PenLine` (amend), `Package` (stash) — for 3-way selector
- Dialog/toast: `AlertTriangle`, `Info`, `CheckCircle`, `XCircle`
- Navigation: `ChevronDown`, `ChevronRight` (section expand/collapse, replacing `▼`/`▶`)
- Tag pill: `Tag` icon (addresses "find a better icon for the tag pill" requirement)

**Usage pattern:**
```svelte
<script lang="ts">
  import { GitBranch, Tag, Trash2 } from '@lucide/svelte';
  // Or for faster builds, direct imports:
  import GitBranch from '@lucide/svelte/icons/git-branch';
</script>

<GitBranch size={14} color="currentColor" strokeWidth={2} />
```

**TypeScript support:**
```typescript
import { type Icon as IconType } from '@lucide/svelte';
// Use `typeof IconType` for component type references in menus/configs
```

**Confidence:** HIGH — Official Lucide docs explicitly document `@lucide/svelte` as the Svelte 5 package. Version 0.577.0 confirmed on npm (2026-03-04). 308K weekly downloads for the lucide-svelte ecosystem.

---

### Dialog / Notification System: No New Dependencies

The app already has everything needed. **Do not add a toast/notification library.**

| Approach | What to Use | Why |
|----------|-------------|-----|
| **Destructive confirmations** | `@tauri-apps/plugin-dialog` `ask()` | Already used for stash drop (BranchSidebar.svelte:207). Native OS dialog, correct for "are you sure?" prompts (discard, branch delete, tag delete). |
| **Error messages** | `@tauri-apps/plugin-dialog` `message()` | Already used for 9+ error cases in CommitGraph.svelte. Native modal, blocks until dismissed, appropriate for errors. |
| **In-app notifications/toasts** | Custom Svelte component (build it) | For non-blocking feedback ("Branch deleted", "Changes discarded"). Build a simple `<Toast>` component using the existing `$state` rune pattern. ~30 LOC. Not worth a dependency. |
| **Input dialogs** | Existing `InputDialog.svelte` | Already handles branch/tag creation with field validation, Escape/Enter handling, backdrop click. Extend for any future input needs. |

**Why not add a toast library:**
- The app has ~4,400 LOC Svelte. A toast is ~30 LOC.
- Libraries like `svelte-sonner` or `svelte-french-toast` add SSR handling, portal management, and animation systems designed for web apps — unnecessary overhead for a Tauri desktop app.
- The existing `$state` rune + shared module pattern (see `remote-state.svelte.ts`) is the proven cross-component state pattern.

**Recommended toast implementation pattern:**
```typescript
// lib/toast-state.svelte.ts
type Toast = { id: number; message: string; kind: 'success' | 'error' | 'info'; };
export const toastState = $state<{ toasts: Toast[] }>({ toasts: [] });
export function showToast(message: string, kind: Toast['kind'] = 'info') {
  const id = Date.now();
  toastState.toasts = [...toastState.toasts, { id, message, kind }];
  setTimeout(() => {
    toastState.toasts = toastState.toasts.filter(t => t.id !== id);
  }, 3000);
}
```

**Confidence:** HIGH — existing patterns verified in codebase. `@tauri-apps/plugin-dialog` already in Cargo.toml and package.json with 9+ existing usage sites.

---

### git2 APIs for New Operations

No new Rust crate dependencies needed. All operations use existing `git2 = "0.19"`.

#### Discard Changes (revert working tree files)

**Two cases, two approaches:**

| File State | git2 API | Equivalent git command |
|------------|----------|----------------------|
| **Tracked modified/deleted** | `repo.checkout_head()` with `CheckoutBuilder::force().path(file)` | `git checkout -- <file>` |
| **Untracked new files** | `std::fs::remove_file()` / `std::fs::remove_dir_all()` | `rm <file>` |

**Implementation sketch:**
```rust
pub fn discard_file_inner(
    path: &str,
    file_path: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let status = repo.status_file(std::path::Path::new(file_path))?;

    if status.contains(git2::Status::WT_NEW) {
        // Untracked file — just delete it
        let full_path = repo.workdir()
            .ok_or_else(|| TrunkError::new("bare_repo", "Cannot discard in bare repo"))?
            .join(file_path);
        if full_path.is_dir() {
            std::fs::remove_dir_all(&full_path)
                .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
        } else {
            std::fs::remove_file(&full_path)
                .map_err(|e| TrunkError::new("io_error", e.to_string()))?;
        }
    } else {
        // Tracked file — checkout from HEAD
        repo.checkout_head(Some(
            git2::build::CheckoutBuilder::new()
                .force()
                .path(file_path)
        ))?;
    }
    Ok(())
}
```

**Key APIs (all confirmed in git2 0.19.0 docs):**
- `repo.status_file(path)` — returns `Status` bitflags for one file
- `repo.checkout_head(opts)` — already used in `create_branch_inner` (branches.rs:279)
- `CheckoutBuilder::force()` — "take any action necessary to get the working directory to match"
- `CheckoutBuilder::path(file)` — scopes checkout to a single file instead of entire tree

**Discard all:** Same pattern but use `checkout_head(force)` without path filter (discards all tracked), then iterate `get_status_inner` result to delete remaining `WT_NEW` files.

**Confidence:** HIGH — `checkout_head`, `CheckoutBuilder::force()`, and `CheckoutBuilder::path()` all verified in git2 0.19.0 docs.

#### Branch Delete

**API:** `Branch::delete(&mut self) -> Result<(), Error>`

```rust
pub fn delete_branch_inner(
    path: &str,
    branch_name: &str,
    force: bool,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    let mut branch = repo.find_branch(branch_name, git2::BranchType::Local)?;

    if branch.is_head() {
        return Err(TrunkError::new(
            "cannot_delete_head",
            "Cannot delete the currently checked out branch",
        ));
    }

    // Optional: check if merged (for non-force delete)
    if !force {
        if let (Ok(head_ref), Some(branch_oid)) = (repo.head(), branch.get().target()) {
            if let Ok(head_oid) = head_ref.target()
                .ok_or(())
                .and_then(|h| repo.merge_base(branch_oid, h).map_err(|_| ())) {
                if head_oid != branch_oid {
                    return Err(TrunkError::new(
                        "branch_not_merged",
                        format!("Branch '{}' is not fully merged. Use force delete.", branch_name),
                    ));
                }
            }
        }
    }

    branch.delete()?;
    Ok(())
}
```

**Key details:**
- `repo.find_branch(name, BranchType::Local)` — already used in test code (branches.rs)
- `Branch::delete()` takes `&mut self` — directly deletes the ref
- Must check `branch.is_head()` to prevent deleting the current branch
- Merge check is optional — let the frontend decide via `ask()` dialog whether to force

**Confidence:** HIGH — `Branch::delete()` verified in git2 0.19.0 docs. `find_branch()` already used in existing tests.

#### Tag Delete

**API:** `Repository::tag_delete(&self, name: &str) -> Result<(), Error>`

```rust
pub fn delete_tag_inner(
    path: &str,
    tag_name: &str,
    state_map: &HashMap<String, PathBuf>,
) -> Result<(), TrunkError> {
    let repo = open_repo_from_state(path, state_map)?;
    repo.tag_delete(tag_name)?;
    Ok(())
}
```

**Key details:**
- `tag_delete` takes the short name (e.g., "v1.0.0"), NOT the full ref ("refs/tags/v1.0.0")
- Already have `repo.tag()` in `create_tag_inner` (commit_actions.rs:75) — symmetric operation
- Confirmation dialog in frontend before calling (tags may be pushed to remotes)

**Confidence:** HIGH — `Repository::tag_delete()` verified in git2 0.19.0 docs.

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `svelte-sonner` / `svelte-french-toast` | SSR-oriented, portal complexity, overkill for desktop app | Custom `<Toast>` with `$state` rune (~30 LOC) |
| `@iconify/svelte` | Fetches icons from remote API by default — fails offline in desktop app | `@lucide/svelte` (bundled, tree-shakable) |
| `lucide-svelte` (without @lucide scope) | Targets Svelte 4, not Svelte 5 runes-native | `@lucide/svelte` (Svelte 5 native) |
| Any animation library | Existing CSS transitions + Svelte `transition:fade` sufficient for toast | `svelte/transition` (built-in) |
| Any new Rust crates | All git operations covered by existing `git2 = "0.19"` | `Branch::delete()`, `tag_delete()`, `checkout_head()` with `force().path()` |
| Custom icon font | Adds build complexity, can't tree-shake, can't color individual strokes | SVG icon components from `@lucide/svelte` |
| `tauri-plugin-notification` | System-level notifications are too heavy for in-app feedback | Custom toast or `message()` dialog |

---

## Installation

```bash
# One new npm dependency (icons)
npm install @lucide/svelte

# No new Rust dependencies — all operations use existing git2 = "0.19"
# No Cargo.toml changes
# No tauri.conf.json changes (dialog plugin already configured)
```

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `@lucide/svelte@^0.577.0` | `svelte@^5.0.0` | Explicitly designed for Svelte 5. Uses `$props()` runes internally. |
| `@lucide/svelte@^0.577.0` | `vite@^6.0.0` | Pure ES modules, works with Vite's tree-shaking out of the box. |
| `git2@0.19` | Existing — no changes | `Branch::delete()`, `tag_delete()`, `checkout_head(force)` all in 0.19.0. |
| `@tauri-apps/plugin-dialog@^2.6.0` | Existing — no changes | `ask()`, `message()` already used for confirmations and errors. |

---

## Integration Points

### Icon Integration with Existing Components

| Component | Current Icons | Replace With |
|-----------|--------------|--------------|
| `Toolbar.svelte` | Unicode: `↩ ↪ ↓ ↑ ⏗ 📦 📥` | Lucide: `Undo2`, `Redo2`, `ArrowDown`, `ArrowUp`, `GitBranch`, `Package`, `PackageOpen` |
| `FileRow.svelte` | Text symbols: `+`, `✎`, `−`, `→`, `⇄`, `!` | **Keep as-is** — these are single-character status badges, not icons. Text symbols are the correct convention for file status indicators (Git, VS Code, and GitKraken all use similar). |
| `StagingPanel.svelte` | Text: "Stage All Changes", "Unstage All" with `▼`/`▶` chevrons | Add Lucide `ChevronDown`/`ChevronRight` for expand toggles. Add `Plus`/`Minus` icons to "Stage All" / "Unstage All" buttons. |
| `BranchSection.svelte` | No icons, `▶`/`▼` for expand | Add section header icons: `GitBranch` (Local), `Globe` (Remote), `Tag` (Tags), `Layers` (Stashes). Replace `▶`/`▼` with `ChevronRight`/`ChevronDown`. |
| `CommitForm.svelte` | No icons, checkbox for amend | Replace checkbox with 3-way selector using icons: `GitCommit` (commit), `PenLine` (amend), `Package` (stash) |
| `TabBar.svelte` | `×` text close button | Lucide `X` icon (14px, consistent with other icons) |
| SVG ref pills | Text-only tag pill | Lucide `Tag` icon rendered as inline SVG `<path>` data (not as component — pills are in SVG overlay, need raw path data) |
| `WelcomeScreen.svelte` | No icons | Lucide `FolderOpen` for the "Open Repository" button |

### Dialog Integration for New Operations

| Operation | Dialog Type | Existing Pattern |
|-----------|-------------|------------------|
| Discard single file | `ask()` — "Discard changes to {file}?" | Same as stash drop in BranchSidebar.svelte |
| Discard all files | `ask()` with `kind: 'warning'` — "Discard all changes?" | Same pattern |
| Delete branch | `ask()` — "Delete branch {name}?"; warn if unmerged | Same pattern |
| Delete tag | `ask()` — "Delete tag {name}?" | Same pattern |
| Success feedback | Custom `<Toast>` — "Branch deleted" | New pattern (described above) |
| Error feedback | `message()` with `kind: 'error'` | 9+ existing usages in CommitGraph.svelte |

### Rust Command Pattern for New Operations

Follow the established `inner-fn` pattern (e.g., `create_tag_inner` → `create_tag`):

```
discard_file_inner   → discard_file    (Tauri command)
discard_all_inner    → discard_all     (Tauri command)
delete_branch_inner  → delete_branch   (Tauri command)
delete_tag_inner     → delete_tag      (Tauri command)
```

Each new command should:
1. Clone `state_map` from `RepoState` (established pattern)
2. Run in `tauri::async_runtime::spawn_blocking` (established pattern)
3. Rebuild graph cache after mutation for branch/tag delete (changes ref labels in graph)
4. Emit `repo-changed` event via `app.emit()` (established pattern)
5. Return `Result<(), String>` with `TrunkError` serialization (established pattern)

**Where to place new commands:**
- `discard_file` / `discard_all` → `staging.rs` (alongside stage/unstage)
- `delete_branch` → `branches.rs` (alongside create/checkout branch)
- `delete_tag` → `commit_actions.rs` (alongside create_tag)

---

## Sources

- [git2 0.19.0 Repository docs](https://docs.rs/git2/0.19.0/git2/struct.Repository.html) — `tag_delete()`, `checkout_head()`, `status_file()` (HIGH confidence)
- [git2 0.19.0 Branch docs](https://docs.rs/git2/0.19.0/git2/struct.Branch.html) — `Branch::delete()`, `find_branch()`, `is_head()` (HIGH confidence)
- [git2 0.19.0 CheckoutBuilder docs](https://docs.rs/git2/0.19.0/git2/build/struct.CheckoutBuilder.html) — `force()`, `path()` (HIGH confidence)
- [Lucide Svelte docs](https://lucide.dev/guide/packages/lucide-svelte) — `@lucide/svelte` for Svelte 5, import patterns, props API (HIGH confidence)
- [npm: @lucide/svelte](https://www.npmjs.com/package/@lucide/svelte) — version 0.577.0, confirmed on npm (HIGH confidence)
- [npm: lucide-svelte](https://www.npmjs.com/package/lucide-svelte) — Svelte 4 version, 308K weekly downloads (HIGH confidence)
- [npm: @iconify/svelte](https://www.npmjs.com/package/@iconify/svelte) — API-dependent, not suitable for offline desktop (HIGH confidence)
- [npm: svelte-radix](https://www.npmjs.com/package/svelte-radix) — Only 310 icons, no git-specific icons (HIGH confidence)
- Existing codebase: `branches.rs`, `commit_actions.rs`, `staging.rs`, `Toolbar.svelte`, `BranchSidebar.svelte`, `CommitGraph.svelte`, `InputDialog.svelte`, `FileRow.svelte`, `CommitForm.svelte`, `StagingPanel.svelte` (HIGH confidence)

---
*Stack research for: Trunk v0.6 UI Polish & Core Ops*
*Researched: 2026-03-15*
