# Evaluation: should trunk adopt `@pierre/diffs` to improve diffs?

Status: **Phase 0 complete + Phase 1 fork resolved** — reject `@pierre/diffs`;
close the highlighting gap **natively in Rust via the `two-face` syntect crate**
(not frontend Shiki). See "Phase 0 results" and "Phase 1 fork decision" below.
Date: 2026-06-20

## TL;DR

Do **not** adopt `@pierre/diffs` wholesale. The cost (inverting our Rust diff
pipeline + React-in-Svelte friction + rebuilding our staging interactivity)
dwarfs the benefit, and we'd be paying it to replace capabilities we already
have. Instead, run a throwaway **spike** to settle the library question
empirically, then close the real gaps (syntax-highlighting coverage, visual
polish, large-file perf) **natively** regardless of how the spike lands.

The likely outcome: "steal Shiki for highlighting, drop the rest" — but the
spike, not intuition, decides.

## What `@pierre/diffs` is

Open-source JS diff-rendering library (Pierre Computer Company), built on
**Shiki** for syntax highlighting. Ships as **React + vanilla-JS** components
(no Svelte adapter). Lower-level APIs render **HTML strings**; higher-order
components consume them. Recent + active (v1.2.x, published days ago).

Its model: you feed it raw file contents (`FileContents`) or a git patch string
(`parsePatchFiles` / `parseDiffFromFile`); **it** owns syntax highlighting, word
diff, layout (split/unified), virtualization (`CodeView`), merge-conflict UI,
and a comments/annotations framework.

- npm: https://www.npmjs.com/package/@pierre/diffs
- docs: https://diffs.com/docs
- architecture writeup: https://pierre.computer/writing/on-rendering-diffs
- source: https://github.com/pierrecomputer/pierre/tree/main/packages/diffs

## Why wholesale adoption is a poor fit

The two architectures are mirror images, and ours is the deliberate one:

| | trunk today | `@pierre/diffs` wants |
|---|---|---|
| Syntax highlighting | syntect, **in Rust**, precomputed | Shiki, **in JS**, async workers |
| Word / intra-line diff | `similar` crate, in Rust | its own, in JS |
| Data shipped to UI | enriched `MergedSpan[]` | raw file content / patch strings |
| Framework | **Svelte 5** | **React / vanilla** |

Adopting it means inverting the entire diff pipeline — tearing out `diff.rs`
enrichment, shipping raw file bodies to the client, redoing highlighting in JS.
And our diff is **not a read-only viewer**: it's an interactive **staging
surface** (per-line selection, drag-to-select, stage / unstage / discard
hunks-or-lines, full-file mode, comment composer). Pierre's accept/reject UI is
for merge conflicts and review comments, **not git-index staging** — so the
interactivity we care about most we'd rebuild on their event model anyway, from
a React-first lib, inside a Svelte app. Engineering-judgment red flags: "every
new dependency needs a strong case" and "don't fight your tools," both failing.

Most of what it sells, we already have: split + unified, syntax highlighting,
word-level emphasis, invisible-char rendering.

## Real gaps it exposes (worth fixing without the library)

1. **Syntax-highlighting coverage.** `src-tauri/src/git/syntax.rs` maps
   `ts/tsx/jsx/svelte → js`. A Git GUI for a Svelte+TS codebase highlighting its
   own files as plain JS is the weakest link.
2. **Large-file performance.** We render every line into the DOM; no
   virtualizer. `CodeView`'s pitch is zero-blanking at any scale.

## Current implementation map (baseline)

- Backend: `src-tauri/src/commands/diff.rs` (~583 lines) — git2 diff →
  `walk_diff()` → `enrich_file_diffs()` (word diff via `similar`, syntax via
  syntect, span merge). Returns `Vec<FileDiff>` with fully-populated
  `MergedSpan[]` per line.
- Syntax: `src-tauri/src/git/syntax.rs` — syntect, base16-ocean.dark → CSS
  classes (`syn-keyword`, …). Fallback `ts/tsx/jsx/svelte → js`.
- Types: `src-tauri/src/git/types.rs` ↔ `src/lib/types.ts` (mirrored serde).
- Frontend: `src/components/diff/` — `DiffPanel` → `DiffViewer` →
  `HunkView` (unified) / `SplitView` (side-by-side) / `FullFileView`.
  Utilities in `src/lib/diff-utils.ts` (`pairLines`, `splitInvisibles`).
- No diff UI library in `package.json`; backend deps `git2`, `similar`,
  `syntect`.

## Plan

### Phase 0 — Spike the library (throwaway, ~1 day) — DECISION GATE

Convert the verdict into evidence before committing either way. Build a
throwaway Svelte route rendering one real `FileDiff` through Pierre's **vanilla**
API (feed a git patch via `parsePatchFiles`). Measure four things, each tied to
a stated goal:

1. **Interactivity bridge (make-or-break)** — can per-line selection / stage /
   unstage / discard / comment-anchor ride on Pierre's selection + comment
   callbacks without rebuilding everything?
2. **Shiki highlighting** quality on `.svelte` and `.tsx` vs our syntect-as-JS
   fallback.
3. **`CodeView` large-file perf** — load a multi-thousand-line diff; check
   blanking / jank.
4. **Bundle cost** — Shiki + grammars + worker added to the Tauri bundle.

**Gate output:** `adopt for viewing-only` / `steal Shiki, drop the rest` /
`reject`. The spike is throwaway — do **not** merge it; produce a written
recommendation. Prior: "steal Shiki, drop the rest."

### Phase 1 — Syntax highlighting (biggest real gap)

> **Resolved 2026-06-20 → "Stay in Rust" via the `two-face` crate.** See the
> "Phase 1 fork decision" section at the end of this doc. The two options below
> are kept for context; the backend path won decisively.

Defect: `syntax.rs` fallback `ts/tsx/jsx/svelte → js`. Path chosen by the gate:

- **Stay in Rust (preferred if spike rejects Shiki):** load richer grammars into
  syntect (bat / two-face syntax set has real TS/TSX; `.svelte` is the hard one,
  may need a custom `.sublime-syntax`). Keeps the stateful per-file highlighter —
  full-file context, which per-line client highlighting *loses*.
- **Move to Shiki in JS (only if spike proves the jump is large):** we already
  ship `content` per line, but per-line highlighting drops multi-line
  string/comment context — so this also means shipping full file bodies (the
  partial architecture inversion). Only worth it if quality gain is decisive.

### Phase 2 — Visual polish (low risk, pure frontend, library-independent)

CSS/markup pass across `HunkView` / `SplitView` / `FullFileView`, all through
theme `--color-*` variables (no inline colors, per project rules): change-
indicator style options (+/− vs background vs vertical bar), gutter
spacing/alignment, line-height, word-diff emphasis treatment. Shippable on its
own.

### Phase 3 — Large-file performance (conditional, gated on measurement)

We render every line into the DOM. If Phase 0's `CodeView` test shows a real gap
on big diffs, virtualize the Svelte diff list natively (windowed rendering of
`hunk.lines`) — keeps our enriched-span architecture, no React. Gate on an
actual measurement, not an assumption.

## Sequencing

Phase 0 first (informs Phase 1). Phase 2 can run anytime in parallel (pure CSS,
no dependencies). Phase 3 last and conditional.

## Phase 0 results

Run 2026-06-20. Throwaway spike executed in an isolated git worktree
(`/Users/joaofnds/code/trunk-pierre-spike`, branch `spike/pierre-diffs`) — **not
merged**, never to be merged. `@pierre/diffs@1.2.11` + `shiki@4.2.0` installed
there via bun. Four real measurements driven against real fixtures: a real
`.svelte` patch + real `.rs` patch (both from commit `0f625f2`), the combined
two-file patch, and a synthetic 4,400-line all-addition patch. Only the
**vanilla `.` export** of `@pierre/diffs` was used (its `./react` entry was never
imported); the library's React adapter is out of scope by design.

### Gate decision

**Reject `@pierre/diffs` as a library — in every form (not wholesale, not
viewing-only). Of the three required gate labels, the closest fit is "steal
Shiki, drop the rest" — but with a hard qualification: "steal Shiki" is the
*frontend fallback*, not a committed path. Phase 1 must first weigh backend
syntect-grammar enrichment against frontend Shiki.**

Why the firm part (reject the library) is unambiguous:

- **The make-or-break (M1) fails on the load-bearing path.** Trunk's diff is a
  git *staging surface*, and its staging commands (`stage_lines` /
  `unstage_lines` / `discard_lines`) are keyed by a **flat libgit2 within-hunk
  line index** — verified firsthand at `src-tauri/src/commands/staging.rs:822-829`
  (`for line_idx in 0..num_lines { patch.line_in_hunk(hunk_idx, line_idx) … }`,
  context lines included). Pierre's vanilla API **never emits that index**: its
  public line events carry only `(lineNumber, side, coarse lineType)` over a
  *grouped block* model (`hunkContent: (ContextContent | ChangeContent)[]`,
  `types.d.ts:154`) with no flat per-line array. The one internal API that does
  carry `(hunkIndex, lineIndex)` — `DiffHunksRenderer.getUnified/SplitInjectedRowsForLine`
  — is `protected` and unreachable without subclassing. Adopting Pierre forces a
  custom, **correctness-critical** coordinate adapter that bets Pierre's
  block-to-line ordering reproduces libgit2's `line_in_hunk` ordering byte-for-byte.
  A wrong bet **silently stages the wrong lines** — the worst failure for a git
  GUI. So the part that makes trunk's diff a staging surface does not ride; it
  gets rebuilt on top.
- **"Viewing-only" is killed by the architecture.** `RepoView.svelte:886` — a
  *single* `DiffPanel` mount is shared by both read-only commit views
  (`diffKind="commit"`) and staging. There is no separate viewer to swap cheaply;
  viewing-only adoption means forking the diff surface in two and maintaining both,
  while still inheriting Pierre's bundle cost (M4) and a coordinate adapter for
  comment anchors.
- **Nothing else load-bearing rides.** Pierre's accept/reject-hunk API
  (`diffAcceptRejectHunk`) is **merge-conflict resolution** (ours/theirs/both —
  it returns a transformed in-memory `FileDiffMetadata`, performs no git write),
  not git-index staging — confirmed from both the type and `normalizeDiffResolution.js`.

Why "steal Shiki" is the qualifier and not a blind adopt:

- Shiki is an **independently installable** package (`@pierre/diffs`'s own
  `package.json` lists `shiki: "^3 || ^4"` as an ordinary dependency). Capturing
  the highlighting win needs **zero `@pierre/diffs` code** — `bun add shiki` + ~20
  lines of extension routing. So the grammar/WASM bundle cost rides with *Shiki*,
  not with the rejected Pierre machinery (CodeView/InteractionManager/virtualizer).
- **The strongest dissent (toward full "reject") is real and reframes Phase 1.**
  Trunk highlights *once in Rust, on the backend, off the UI thread*. Pulling
  Shiki into the *frontend* moves highlighting onto the webview (and, unless wired
  through a worker, the main thread), which then makes virtualization a
  prerequisite rather than a bonus. The cheaper, lower-risk fix for the `.svelte`
  gap may be to add a `.svelte`/`.tsx` grammar to **syntect on the backend** —
  keeping highlighting in Rust. The spike did **not** measure whether syntect can
  get a usable `.svelte` grammar; that is the first Phase 1 question. (Two
  corrections to the dissent: frontend Shiki *can* run in a worker, and
  virtualization is warranted natively regardless of the highlighting choice — see
  M3.)

Net: drop `@pierre/diffs`. Pursue the highlighting gap (Phase 1) and big-file
virtualization (Phase 3) **natively**; Shiki is the leading frontend candidate
for grammars but must lose a head-to-head against backend syntect enrichment
before it ships.

### M1 — Interactivity bridge (make-or-break): PARTIAL, leaning REBUILD

The selection/comment *UI* rides; the selection→`stage_lines` *coordinate bridge*
does not.

- Rides for free: drag-to-paint range selection
  (`InteractionManager`: `onLineSelectionStart/Change/End`, `pointerdown` →
  `handleDocumentPointerMove`), a controlled selection-write API
  (`CodeView.setSelectedLines` / `InteractionManager.setSelection`), and inline DOM
  row injection for a comment composer (`CodeViewDiffItem.annotations` +
  `renderAnnotation(annotation): HTMLElement`, `FileDiff.d.ts:43`).
- Does **not** ride: every Pierre callback/selection is in `(lineNumber, side)`
  display space; trunk needs `(hunkIndex, within-hunk lineIndex)`. `DiffLineEventBaseProps`
  (`types.d.ts:456-460`) + inherited `LineEventBaseProps` (`:449-455`) expose
  `lineNumber`, `annotationSide`, `lineType` — **no `hunkIndex`, no within-hunk
  index**. Confirmed in built JS (`InteractionManager.js:805-812`).
- Two further mismatches: Pierre's `SelectedLineRange` is a single **contiguous**
  `start..end`; trunk's selection is an arbitrary **`Set<number>`**
  (`DiffPanel.svelte:86`). And the inline-comment anchor is keyed by
  `lineNumber+side` only, so old/new line numbers + origin must be reconstructed
  from `Hunk.additionStart/deletionStart` ranges.
- Caveat (honest): nothing was staged end-to-end through Pierre. The critical
  unverified risk — that Pierre's `hunkContent` ordering matches libgit2's
  `line_in_hunk` byte-for-byte on **interleaved +/- hunks, no-newline-at-EOF, and
  CRLF** — was exercised only on an all-additions/context hunk. This gap is moot
  for the chosen "drop the rest" verdict but would be a hard blocker for any
  Pierre-staging path. Also untested: whether a live Svelte component injected via
  `renderAnnotation` survives Pierre's element pooling on scroll (could unmount
  mid-edit).

### M2 — Shiki highlighting quality: DECISIVE for `.svelte`, MODERATE for `.tsx`

Token-level evidence (Shiki tokenizer, scripts at
`spike-fixtures/shiki-quality.mjs` + `shiki-svelte-swallow.mjs`):

- Trunk today maps `ts|mts|cts|tsx|jsx|svelte|vue → js` (`syntax.rs:41-46`) then
  collapses every color into **7 fixed CSS classes** by RGB-matching
  base16-ocean.dark (`syntax.rs:16-37`). Its own test
  `highlight_svelte_uses_js_fallback` (`syntax.rs:356`) bakes this in.
- Shiki ships **real distinct `svelte`, `tsx`, `vue` grammars**; `@pierre/diffs`'s
  `getFiletypeFromFileName` routes `.svelte→"svelte"`, `.tsx→"tsx"` (no `js`
  collapse).
- **The `.svelte` `<script>` body is genuinely broken under the JS fallback:**
  `  const doubled = $derived(count * 2);` → **13** correctly-colored tokens under
  the `svelte` grammar vs **1** flat uncolored default-foreground span under the
  `js` grammar. Svelte block/directive syntax degrades too: `{#if}` loses
  `keyword.control.conditional.svelte`; `on:click` is mis-scoped as a namespaced
  HTML attribute instead of `meta.directive.on.svelte`.
- Scope-richness on the Svelte snippet: **63 distinct scopes (svelte) vs 30 (js)**
  — 2.1×, with 63 scopes exclusive to the correct grammar (incl. embedded
  `source.ts` in `<script>`, `keyword.operator.type.annotation.ts`).
- `.tsx` is more moderate: scope counts **tie 52 = 52** because Shiki's
  `javascript` grammar already embeds TS+JSX; the real `.tsx` win is escaping
  trunk's 7-bucket collapse, not a better grammar.
- Caveats: token/scope-level, **not** pixels in trunk's running GUI; the 7-bucket
  claim is read from source, not screenshotted. The JS-fallback simulation used
  Shiki's `javascript` grammar, not syntect's exact `JavaScript.sublime-syntax`
  — so the `.tsx` tie is a *conservative, best-case-for-trunk* estimate; real
  syntect `.tsx` could be worse.

### M3 — Large-file performance: DECISIVE gap; trunk does NOT virtualize its diff

- Trunk's `HunkView` / `SplitView` / `FullFileView` render **every** line into the
  DOM (`HunkView.svelte:446 {#each hunk.lines}`, etc.); grep for
  `IntersectionObserver|scrollTop|overscan|renderRange` across all four diff files
  = **zero** matches.
- The `@humanspeak/svelte-virtual-list` dep is wired **only** into
  `CommitGraph.svelte` (commit list); `MergeEditor` has its own hand-rolled
  virtualizer. **The diff path imports neither.**
- Pierre's `CodeView` windows the DOM to ~visible viewport rounded to
  `hunkLineCount` (50) + overscan (`overscrollSize: 200px`,
  `DEFAULT_VIRTUAL_FILE_METRICS`, `constants.js:38-43`) — ~150 lines regardless of
  file size.
- On the 4,400-line synthetic patch: trunk ≈ **50,600 elements (~88k nodes)** all
  mounted at once vs a ~150-line window ≈ **1,725** → **~29× DOM reduction**. The
  4-element scaffold-per-line floor (≈17,600) is exact and span-count-independent.
- Real timings (Node v26, `@pierre/diffs@1.2.11`, `shiki@4.2.0`,
  `spike-fixtures/measure-parse.mjs`): parsing the 4,400-line patch is cheap
  (**~0.71 ms**); the cost is **highlighting** — full-file Shiki ≈ **395 ms**
  median vs a 150-line window ≈ **15 ms** (**~26×**).
- Caveats: **visual blanking/jank needs a human eyeball** in the running Tauri
  webview — characterized as architecture (windowed vs all-in-DOM) + proxies (node
  count, highlight ms) only. The 395 ms is *Pierre's own* full-file Shiki cost
  (showing why Pierre windows), **not** trunk's current Rust/syntect highlight
  cost. Virtualization is independently warranted by trunk's ~50k-node DOM blowup
  regardless of where highlighting runs.

### M4 — Bundle cost + integration proof: mounts clean with zero React; HEAVY but mostly LAZY

- **Integration proof:** the vanilla `CodeView` mounts in a Svelte 5 runes
  component with **zero React** — `new CodeView({theme})` → `setup(div)` →
  `setItems(parsePatchFiles(...).flatMap(p => p.files))` → `render(true)`.
  `parsePatchFiles` correctly parsed all 4 real git-style fixtures; the mount code
  **type-checks clean** (`svelte-check`: 0 errors), and `vite build` of both
  entries succeeds.
- **Eager cost** added to the first page importing `CodeView`: **~528 KB raw /
  ~148 KB gzip** (vanilla core + worker manager + Shiki core), measured against a
  real no-Pierre spike build.
- **Lazy cost** (dynamic-import chunks, none preloaded): **239 grammars + 67
  themes + a 608 KB base64-inlined oniguruma WASM** ≈ **9.4 MB raw / 1.7 MB gzip**.
  This is **Shiki's** footprint, not Pierre-unique — and it pulls Shiki's *full*
  bundled registry; pinning only the languages trunk shows would shrink it
  drastically (not attempted, should be re-measured before any frontend-Shiki
  ship).
- This build did **not** split a Web Worker file — `WorkerPoolManager` was bundled
  into the eager entry, so highlighting would run on the main thread unless a
  worker is wired explicitly. CSS added is tiny (~1 KB raw).
- Verdict: ACCEPTABLE for a desktop Tauri app (assets ship on-disk, no per-user
  download; eager cost paid once when the diff view first opens) — **but the entire
  eager Pierre core is what "drop the rest" sheds**, and the lazy Shiki cost is the
  price of "steal Shiki" specifically.
- Caveat: **no pixels were rendered/observed** — mount sequence + type-check +
  build are proven; actual on-screen rendering, colors, and scroll behavior need a
  Playwright/WebView run a human can watch.

### Eyeball confirmation (2026-06-20, human)

The one thing the spike could not measure headlessly — perceived visual quality —
was checked by opening the spike route: **the rendered Pierre diff is judged
substantially better than trunk's current diff.** This confirms M2 perceptually
(it's no longer just a token-count argument). It does **not** change the gate.
The visible quality is the sum of (a) **Shiki syntax highlighting** — full themed
colors + correct `.svelte`/`.tsx`/`.rust` grammars, the M2 win — and (b) **CSS
polish** (gutter, spacing, word-diff treatment), the Phase 2 work. **Both are
capturable without the library**; neither is the rejected value (interactive
staging). So the eyeball result *raises the priority of Phases 1–2* — it does not
argue for adopting `@pierre/diffs`. Open question worth pinning down before
Phase 1: how much of "better" is highlighting (Phase 1) vs layout/typography
(Phase 2), since they have different owners and the highlighting path is the one
real backend-vs-frontend decision.

### What this means for Phases 1–3

- **Phase 1 (highlighting) — the real open decision.** Run a *cheap* follow-up
  spike: can syntect load a working `.svelte`/`.tsx` grammar on the backend
  (two-face/bat syntax set for TS/TSX; a custom `.sublime-syntax` for `.svelte`)?
  If yes → close the gap in Rust, keep highlighting off the UI thread, no Shiki.
  If no (or quality is poor) → adopt **Shiki directly** (no `@pierre/diffs`), pin a
  fixed language set, and run it through a worker. Either way, also widen the
  7-bucket color collapse.
- **Phase 2 (visual polish)** — unaffected; pure CSS, ship anytime.
- **Phase 3 (virtualization)** — confirmed warranted: trunk's diff path mounts
  ~50k elements on a 4,400-line diff with no windowing. Virtualize the Svelte diff
  list natively (the project already owns `VirtualList`), keeping enriched spans
  and no React. (A human should still eyeball the jank in the running app to
  confirm priority.)

### Spike artifacts & cleanup

- Worktree: `/Users/joaofnds/code/trunk-pierre-spike` (branch `spike/pierre-diffs`).
  Throwaway route: `spike.html` + `src/spike/{main.ts,Spike.svelte}`. Measurement
  scripts + fixtures under `spike-fixtures/`.
- To eyeball the one thing the spike could not measure headlessly (rendered
  highlighting + scroll/jank), in the worktree: `bun run dev` and open
  `/spike.html`, or `bun run build` then preview `dist/spike.html`.
- To remove when done: `git worktree remove --force /Users/joaofnds/code/trunk-pierre-spike`
  then `git branch -D spike/pierre-diffs`. Nothing from the spike touches `main` or
  the main checkout's `package.json`. (The worktree now also holds the Phase 1
  probes: `spike-fixtures/syntect-probe/` (Rust crate) and `shiki-pinned`
  build dirs — all throwaway, same removal.)

## Phase 1 fork decision

Resolved 2026-06-20 by a second throwaway spike (two parallel probes: backend
syntect capability via `two-face`, and frontend Shiki cost with a *pinned*
language set). Both arms ran real builds against the worktree.

**Decision: close the highlighting gap on the BACKEND, in Rust, via the
`two-face` syntect crate. Do NOT adopt frontend Shiki.**

### Why backend won (both quality and cost)

- **`two-face` is drop-in.** Version `0.5.1` compiles against trunk's **exact**
  `syntect v5.3.0` (`src-tauri/Cargo.toml:35`, `default-features=false`,
  `default-onig`) — `cargo tree` shows a single shared, deduped `syntect`, no
  version conflict. The change is essentially **one line**: swap
  `SyntaxSet::load_defaults_newlines()` (`syntax.rs:8`) for
  `two_face::syntax::extra_newlines()`.
- **It ships the real grammars trunk is missing.** `find_syntax_by_extension`:
  default set → `NONE` for `.svelte/.tsx/.ts/.jsx/.vue`; two-face →
  `Svelte / TypeScriptReact / TypeScript / JavaScript (Babel) / Vue Component`.
  Those are exactly the five extensions `syntax.rs:41-46` collapses to `"js"`.
- **`.svelte` quality is DECISIVE.** Token-level proof: the JS fallback emits
  `"{#if count > 0}\n"` and the whole `<button on:click={…} class="btn">{label}:
  {doubled}<` markup each as **one** swallowed token; the real Svelte grammar
  splits them into `{` / `#if`(keyword) / `count`(var) / `0`(number) / `}` and
  tokenizes `on:click`, `{#each … as …}`, `:else`, `/if` distinctly. `.tsx` is
  partial-to-decisive (keeps JSX structured, colors JSX attrs; the established
  win — escaping the 7-bucket collapse — lands).
- **The fix survives trunk's existing CSS layer.** Two-face's Svelte/TSX tokens
  land on RGBs already in `color_to_css_class` (`syntax.rs:16-37`) — keyword
  `180,142,173`, string `163,190,140`, number `208,135,112`, variable
  `191,97,106` — so the structural win shows up with **zero** CSS-map changes;
  unmapped extras degrade gracefully to default fg via the existing `_ => ""` arm.
- **It keeps the architecture trunk already owns.** Highlighting stays 100% in
  Rust, **off the UI thread**, precomputed per-file with full multi-line context
  (`enrich_file_diffs` already builds one highlighter per file). **No** Shiki,
  **no** main-thread highlight cost, **no** forced virtualization, **no** new JS
  dependency.

### What frontend Shiki would have cost (the rejected arm)

Pinned to ~18 git-GUI languages + 2 themes via Shiki's fine-grained core API:
Engine B (pure-JS regex, **no WASM**) = **55 KB gzip eager + 141 KB gzip lazy
(196 KB total)**; Engine A (oniguruma WASM) = 37 KB eager + 367 KB lazy (404 KB
total, dominated by the 608 KB inlined WASM blob). Pinning cuts 82–87% off the
full-registry footprint measured in M4 — genuinely well-bounded. But it's cost
paid to buy a `.svelte` quality the backend already reaches **for free and
off-thread**, while importing permanent structural taxes (main-thread
highlighting ≈395 ms on a 4,400-line file, virtualization as a hard prerequisite,
worker plumbing to avoid jank). Not justified when the cheaper arm matches the
outcome.

### Tradeoff summary

| Dimension | Backend syntect (two-face) | Frontend Shiki (pinned) |
|---|---|---|
| `.svelte` quality | DECISIVE, zero CSS-map changes | DECISIVE (no edge over backend) |
| `.tsx` quality | Partial-to-decisive (escapes 7-bucket) | Comparable; theme-dependent, no edge |
| Bundle cost | ~0 frontend bytes | +196 KB gzip (Engine B) / 404 KB (A) |
| Thread | OFF the UI thread (Rust) | ON the webview main thread (unless worker) |
| Virtualization forced? | No | Yes (big-file highlight on main thread) |
| Implementation lift | LOW (1-line swap + delete fallback) | HIGH (dep + engine + worker + virtualization) |

### Next action (when Phase 1 is approved for execution)

1. Add `two-face = "0.5"` to `src-tauri/Cargo.toml`.
2. `syntax.rs:8` — swap `load_defaults_newlines` → `two_face::syntax::extra_newlines()`.
3. `syntax.rs:41-46` — delete the `ts/tsx/jsx/svelte/vue → js` fallback (now
   unnecessary; keep any genuinely-needed non-JS aliases if present).
4. Gate on `just check` **plus two new tests** (the spike did NOT cover these):
   - **`merge_spans` byte-offset coverage** on real two-face Svelte+TSX output —
     assert spans cover `0..len` with no gaps (two-face tokenizes multi-char
     tokens like `</`, `:` differently; `highlight_line_with`'s offset accounting
     must survive).
   - **`color_to_css_class` audit** against the RGBs two-face actually emits —
     add entries for any high-frequency scope currently falling through to `""`
     (e.g. interpolation-brace `171,121,103`, JSX-attr `150,181,180`).
5. Sanity-check startup cost: `extra_newlines()` deserializes a larger SyntaxSet
   than the default — measure app-startup load + per-file precompute (likely fine
   for a desktop app, but unbenchmarked in the spike).

### Open risks (carried from the spike — must be closed during execution)

- CSS-class-map gap (some two-face RGBs unmapped → degrade to default fg until
  audited). Graceful, but the fix isn't *fully* realized until the map is extended.
- `merge_spans` byte-offset survival unverified on two-face's different
  tokenization — needs the direct coverage test above before shipping.
- `two-face` SyntaxSet load-time/memory unmeasured.
- Theme-palette ceiling persists on **both** arms: under base16-ocean.dark, TS
  type positions and `satisfies` won't get distinct colors regardless of grammar
  — a separate theme/CSS-map change if richer TS coloring is later wanted.
