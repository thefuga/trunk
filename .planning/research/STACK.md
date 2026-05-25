# Stack Research

**Domain:** Desktop Git GUI — v0.13 Code Review Mode (anchored-comment collection → single AI-targeted markdown artifact)
**Researched:** 2026-05-25
**Confidence:** HIGH (verified against installed crate/package sources)

## Bottom Line

**Zero new dependencies are needed.** Every capability the feature requires is already in `Cargo.toml` or `package.json`, and the remaining pieces (markdown generation, file write, line-excerpt extraction) are inline logic that fits existing patterns (inner-fn Tauri commands, `serde_json`, git2 blob reads). The project's default-to-inline stance is the correct one here — adding a markdown crate, a path crate, or a filesystem plugin would all be unjustified.

Verified against installed sources (not just training data): Tauri 2.10.2, `@tauri-apps/plugin-clipboard-manager@2.3.2`, `@tauri-apps/plugin-dialog@2.6.0`, `@tauri-apps/plugin-store@2.4.2`.

## Existing Stack (DO NOT RE-ADD)

Tauri 2, Svelte 5 (runes), Vite 6, TypeScript 5.6, Tailwind CSS 4, Rust (`git2` 0.19, `serde`/`serde_json` 1, `notify` 7, `tokio` 1, `syntect` 5, `similar` 2.7), `tauri-plugin-clipboard-manager` 2, `tauri-plugin-dialog` 2, `tauri-plugin-store` 2.4.2 (LazyStore), Vitest, GOOS test harness, inner-fn command pattern, `safeInvoke<T>` IPC wrapper.

## Recommended Stack

### Core Technologies (ALL ALREADY INSTALLED)

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `git2` | 0.19 (vendored libgit2) | Read file content at a commit; extract line excerpts via blob lookup by tree path | Already the mandated git backend. A blob read (`commit.tree()?.get_path(p)?.to_object(repo)?.peel_to_blob()?.content()`) gives the exact bytes at any commit, independent of diff hunk boundaries — exactly what arbitrary `(commit, file, line-range)` anchors need. No new crate. |
| `serde` / `serde_json` | 1 / 1 | Serialize/deserialize the per-repo review session (anchors + comments) to a JSON file | Already used project-wide for IPC and round-trip tests. Derive `Serialize`/`Deserialize` on the session struct; that's it. |
| `tauri` (Manager `path()`) | 2.10.2 | Resolve the app data directory for session storage | `app.path().app_data_dir()` — the `Manager` trait `path()` accessor (confirmed in `tauri-2.10.2/src/lib.rs:772` and `path/desktop.rs:247`) returns the OS-correct per-app data dir. Removes any need for the `dirs` crate. |
| `tauri-plugin-clipboard-manager` (Rust) | 2 | Backend half of clipboard plugin | Already a dependency; nothing to add. Frontend does the actual write. |
| `tauri-plugin-dialog` (Rust) | 2 | Backend half of the native save dialog | Already a dependency; the `save()` picker is called from the frontend. |

### Supporting Libraries — Frontend (ALL ALREADY INSTALLED)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `@tauri-apps/plugin-clipboard-manager` | 2.3.2 | Write rendered markdown to the system clipboard | `import { writeText } from '@tauri-apps/plugin-clipboard-manager'; await writeText(markdown)`. Export confirmed in installed `dist-js/index.d.ts` (`writeText, readText, writeHtml, clear, readImage, writeImage`). Same library already used for copy-SHA/message. |
| `@tauri-apps/plugin-dialog` | 2.6.0 | Native "Save As…" picker for the artifact | `import { save } from '@tauri-apps/plugin-dialog'`. Signature confirmed: `save(options?): Promise<string \| null>` — returns chosen path, or `null` on cancel. Pass `{ defaultPath, filters: [{ name: 'Markdown', extensions: ['md'] }] }`. |
| `@tauri-apps/plugin-store` (LazyStore) | 2.4.2 | UI prefs only — NOT session data | Keep for UI prefs (panel open/closed, last-used export dir). Do NOT store the review session here (see persistence note). |

### Development Tools

| Tool | Purpose | Notes |
|------|---------|-------|
| `just check` | Full gate (fmt, biome, svelte-check, clippy, cargo-test, vitest) | Existing. New Rust logic uses the inner-fn pattern so the markdown renderer, session (de)serialization, and blob-excerpt extraction are all unit-testable without a Tauri runtime — same as the GOOS harness. No new tooling. |

## Per-Question Recommendations

### 1. Markdown generation → Rust, hand-rolled, no crate

Generate in **Rust**, not TypeScript. The git blob bytes already live in Rust (git2); shipping raw file content to the frontend purely to template strings is wasteful IPC and splits rendering from the data it reads. The renderer must also handle unresolvable anchors gracefully (commit/file/line gone after history rewrite) — domain logic best kept next to git2.

**No markdown crate.** Excerpts are emitted as fenced code blocks; the only real concern is fence collision (an excerpt containing a ``` run). Handle inline: scan the excerpt for the longest run of backticks and open/close the fence with one more backtick than that (CommonMark allows fences of length ≥ 3). ~10 lines. Crates like `pulldown-cmark`/`comrak`/`markdown-it` are **parsers**, not generators — wrong tool. Template engines (`handlebars`/`tera`/`askama`) are overkill for a single fixed output shape.

**Source-aware fencing (integration point):** the anchor carries `source ∈ {diff, full_file}`. For `source = diff`, emit a ` ```diff ` block. For `source = full_file`, emit a language-fenced block whose language tag comes from the **existing** extension→language mapping in `src-tauri/src/git/syntax.rs` (`extension_from_path` / `has_syntax_for_extension`). Reuse that identifier string — do not re-detect language a second way.

### 2. Clipboard write → `@tauri-apps/plugin-clipboard-manager` `writeText` (already installed)

```ts
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
await writeText(renderedMarkdown);
```

Version 2.3.2 (installed). Frontend-only call; the Rust plugin half is already registered and its capability already granted (copy-SHA/message uses it today).

### 3. File save → plugin-dialog `save()` + a Rust write command (do NOT add plugin-fs)

```ts
import { save } from "@tauri-apps/plugin-dialog";
const path = await save({
  defaultPath: "review.md",
  filters: [{ name: "Markdown", extensions: ["md"] }],
});
if (path) await safeInvoke("write_review_artifact", { path, content });
```

Do the **picker** with plugin-dialog (2.6.0, installed; returns `string | null`). Do the **write** with a Rust `#[tauri::command]` using `std::fs` — consistent with the codebase's "Rust command + git2/std::fs, no fs plugin" approach and matching the dynamic-import dialog pattern already established. Prefer atomic write (temp file + `rename`) so a crash mid-write can't leave a truncated artifact. **Do not add `tauri-plugin-fs`** — a new dependency and a new capability surface for what `std::fs::write` does in one line, testable via inner-fn.

**Verify during execution:** confirm the `dialog:allow-save` permission is present in `capabilities/` — earlier dialog usage may have granted only `open`/`message`/`ask`. If `save` isn't yet permitted, add it.

### 4. Session persistence → Rust-side JSON in the app data dir (NOT LazyStore)

PROJECT.md already records this decision ("Session storage in app data dir, keyed by repo… not `.git/`, not the working tree"). Mechanics, confirmed:

- **Where:** `app.path().app_data_dir()?` (Tauri 2 `Manager::path()`, verified in installed `tauri-2.10.2`). Create a `review-sessions/` subdir; one file per repo.
- **Repo key / filename:** derive a stable filename by **canonicalizing the repo path and percent-encoding it** (reversible, deterministic across Rust versions). Do NOT key by `std::collections::hash_map::DefaultHasher` — its output is explicitly not stable across Rust versions or runs, so a toolchain bump would orphan every existing session file. If you ever hit a filename-length limit (Windows is 260 chars unless long-paths are enabled), truncate the encoded path with a short disambiguating suffix rather than reaching for a hash crate. **Do not add `sha2`/`blake3`/`uuid`** for this.
- **Format:** `serde_json` (installed). Derive `Serialize`/`Deserialize` on the session struct.
- **Durability:** atomic write (temp + rename) on every mutation, so a crash never corrupts an in-progress session.
- **IDs:** a monotonic counter persisted in the session file gives stable anchor/comment IDs across edit/delete — no `uuid` crate needed.

**Why not LazyStore?** It is right for small UI prefs (already used for column widths, diff toggles). A per-repo review session is different in kind: N anchors each with a commit oid, file path, line range, source flag, and free-text comment — the feature's primary domain data, not a UI preference. A dedicated per-repo file gives clean separation from UI state, atomic write, trivial on-disk debugging, and avoids cramming growing structured data into the shared store. LazyStore would technically work but conflates concerns and offers weaker write-atomicity guarantees.

### 5. File content + line excerpts at a commit → pure git2, no new crate

```rust
let commit = repo.find_commit(oid)?;
let blob = commit.tree()?.get_path(Path::new(file))?.to_object(repo)?.peel_to_blob()?;
let text = std::str::from_utf8(blob.content())?;   // guard non-UTF8 / binary
let excerpt: Vec<&str> = text.lines().skip(start - 1).take(end - start + 1).collect();
```

Pure git2 (already mandated) plus std `str`/`lines`. No new crate. Note: the existing full-file diff view (`src-tauri/src/commands/diff.rs`) renders a full file by setting a very large `context_lines` (100_000) on a libgit2 diff — correct for *rendering a diff*, but for arbitrary review line-range excerpts a **direct blob read** is cleaner because it is independent of diff hunk structure and works for the `source = full_file` case where there may be no diff at all. Surface binary/non-UTF8 blobs as unresolvable/excluded at render time (consistent with the graceful-degradation requirement for unresolvable anchors).

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| Markdown in Rust, hand-rolled | Markdown in TS template strings | Only if all excerpt content were already on the frontend — it isn't; git blob bytes originate in Rust |
| `std::fs` write via Rust command | `tauri-plugin-fs` | Only if you needed sandboxed scoped FS access from JS across many paths; here it's one user-chosen path → unjustified plugin |
| `app.path().app_data_dir()` | `dirs` crate | Only outside a Tauri context; inside Tauri the Manager already resolves the platform-correct dir |
| Rust JSON file for session | LazyStore (plugin-store) | Acceptable for *small* sessions, but loses clean separation and atomic-write guarantees; not recommended |
| Canonicalize + percent-encode repo path for filename | `sha2` / `blake3` hash | Only if you needed cryptographic collision resistance or fixed-length keys — you don't, this is a local filename |
| Monotonic counter for IDs | `uuid` | Only if IDs had to be globally unique across machines — they're per-session-file, so a counter suffices |

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `pulldown-cmark` / `comrak` / `markdown-it` | Markdown *parsers*; the feature *emits* markdown, never parses it | Hand-rolled string building with backtick-run fence-length detection |
| `handlebars` / `tera` / `askama` | Template engine for a single fixed output shape — heavy, new dependency, new build surface | Inline Rust string formatting |
| `tauri-plugin-fs` | New dependency + new capability surface for a single-file write | Rust `#[tauri::command]` + `std::fs` (atomic temp+rename) |
| `dirs` crate | Tauri's `Manager::path().app_data_dir()` already provides it | `app.path().app_data_dir()` |
| `std::collections::hash_map::DefaultHasher` for the filename key | Output not stable across Rust versions/runs → a toolchain bump orphans existing session files | Canonicalize + percent-encode the repo path |
| `uuid` | Per-session IDs don't need global uniqueness | Persisted monotonic counter in the session file |
| `sha2` / `blake3` | Repo-path filename keying needs no crypto strength | Canonicalize + percent-encode the repo path |
| Storing the session in LazyStore | Conflates primary domain data with UI prefs; weaker atomicity | Dedicated per-repo JSON file in app data dir |

## Integration Points With Existing Code

| New capability | Hooks into existing code |
|----------------|--------------------------|
| Line-range selection (diff source) | Reuses v0.7 per-line selection in the diff viewer |
| Line-range selection (full-file source) | Reuses v0.12 full-file-at-commit view |
| Language tag for full-file fences | Reuses `src-tauri/src/git/syntax.rs` extension→language mapping (the identifier string, not the highlighted tokens) |
| Save picker | Reuses the dynamic-import `@tauri-apps/plugin-dialog` pattern already established |
| Clipboard copy | Reuses existing clipboard-manager `writeText` usage (copy-SHA/message) |
| New Rust commands (render, save, load/store session, read-blob-excerpt) | Follow inner-fn pattern → unit-testable in GOOS harness; `safeInvoke<T>` on the frontend |
| Commit seeding from graph | Reuses commit selection + right-click context menu in the graph |

## Version Compatibility

| Package A | Compatible With | Notes |
|-----------|-----------------|-------|
| `tauri` 2.10.2 | `tauri-plugin-clipboard-manager` 2, `tauri-plugin-dialog` 2 | All on the Tauri 2.x line; already coexisting in the build |
| `@tauri-apps/api` ^2 | plugin-clipboard-manager 2.3.2 / plugin-dialog 2.6.0 | Frontend plugins track the 2.x JS API already in use |
| `git2` 0.19 | vendored libgit2 + vendored OpenSSL | Already pinned; blob reads need no feature changes |

## Sources

- `node_modules/@tauri-apps/plugin-clipboard-manager/dist-js/index.d.ts` + `package.json` — confirmed `writeText` export, v2.3.2 (HIGH, installed source)
- `node_modules/@tauri-apps/plugin-dialog/dist-js/index.d.ts` + `package.json` — confirmed `save(options?): Promise<string \| null>`, v2.6.0 (HIGH, installed source)
- `~/.cargo/registry/.../tauri-2.10.2/src/path/desktop.rs:247` + `src/lib.rs:772` — confirmed `app_data_dir()` on `Manager::path()`, Tauri 2.10.2 (HIGH, installed source)
- `src-tauri/Cargo.toml`, `package.json` — confirmed all dependencies already present (HIGH)
- `src-tauri/src/commands/diff.rs`, `src-tauri/src/git/syntax.rs` — confirmed existing full-file diff handling and extension→language mapping for integration points (HIGH)
- Rust std docs — `DefaultHasher` output explicitly not guaranteed stable across versions/runs (HIGH)
- `.planning/PROJECT.md` — recorded v0.13 decisions (session in app data dir keyed by repo; anchor = commit/file/line-range/source) (HIGH)

---
*Stack research for: Trunk v0.13 Code Review Mode*
*Researched: 2026-05-25*
