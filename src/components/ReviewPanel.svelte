<script lang="ts">
// Phase 69 — the real review panel (replaces the Phase 65 throwaway stub).
// Renders the accumulated review grouped by commit (D-09), with a per-commit
// "Add note" affordance (D-02), inline edit (D-10), delete-with-confirm (D-05),
// and jump-to-anchor with read-only orphan rows (D-07 / D-08). The panel lives
// in the center pane (UI-SPEC:133); jump is driven by the host via onJump.

import { Clipboard, MessageSquarePlus, Trash2 } from "@lucide/svelte";
import { listen } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { copySha } from "../lib/clipboard.js";
import { isTrunkError, safeInvoke, type TrunkError } from "../lib/invoke.js";
import type { ReviewSessionManager } from "../lib/review-session.svelte.js";
import { showToast } from "../lib/toast.svelte.js";
import type {
	Comment,
	CommentResolution,
	OrphanReason,
	SessionCommit,
	SessionState,
	SessionStatus,
} from "../lib/types.js";

interface Props {
	repoPath: string;
	// The review-session rune (owned by RepoView, threaded in so the panel can
	// drive panel-internal swaps and call the Phase 70 Generate IPC via the rune).
	session: ReviewSessionManager;
	// Resolvable-comment jump: the host (RepoView) binds this to the review-session
	// rune's jumpTo, wiring commit/file selection + scroll-to-range.
	onJump: (comment: Comment) => void;
	// Commit-header jump: select the commit and scroll the graph to it. Same
	// gesture as clicking a line ref, but without a file/line — the panel stays.
	onJumpToCommit: (commitOid: string) => void;
}

let { repoPath, session, onJump, onJumpToCommit }: Props = $props();

let commits = $state<SessionCommit[]>([]);
let comments = $state<Comment[]>([]);
let resolutions = $state<CommentResolution[]>([]);
let canonicalPath = $state<string | null>(null);

// Inline composer / editor state. Only one is open at a time; both reuse the same
// textarea primitive (draftText) and the trim-empty-disables-Save rule.
let addNoteForCommit = $state<string | null>(null);
let editingCommentId = $state<string | null>(null);
let draftText = $state("");

const draftValid = $derived(draftText.trim().length > 0);

// LOCKED OrphanReason → badge label map (UI-SPEC § Copywriting Contract).
const ORPHAN_LABEL: Record<OrphanReason, string> = {
	CommitGone: "commit gone",
	FileGone: "file gone",
	LineOutOfRange: "line out of range",
};

// Resolution lookup by id (D-08): a comment is an orphan when its resolution
// exists and resolvable is false.
const resolutionById = $derived(new Map(resolutions.map((r) => [r.id, r])));

function commitOidForComment(c: Comment): string {
	// Line-anchored comments group under their anchor's commit; commit-level
	// notes (anchor null) group under their own commit_oid (D-09).
	if (c.anchor !== null) return c.anchor.commit_oid;
	return c.commit_oid ?? "";
}

interface CommitGroup {
	oid: string;
	shortOid: string;
	summary: string;
	comments: Comment[];
	isSnapshot: boolean;
}

// Within a group, commit-level comments (anchor === null) sort before
// line-anchored ones — they're notes about the commit as a whole, so they read
// as the lede. Array.prototype.sort is stable on modern engines, so capture
// order is preserved within each class.
function sortGroupComments(list: Comment[]): Comment[] {
	return list.slice().sort((a, b) => {
		if (a.anchor === null && b.anchor !== null) return -1;
		if (a.anchor !== null && b.anchor === null) return 1;
		return 0;
	});
}

// Group comments by commit in the session's commit order; comments on commits
// no longer in the session (e.g. CommitGone) get a fallback group keyed by oid
// so nothing is dropped (D-08). EMPTY snapshot groups (auto-added working-tree /
// staged snapshots with no comments) are filtered out as noise; empty hand-picked
// commit groups stay so their per-commit "Add note" affordance remains (260531-l02d).
const groups = $derived.by<CommitGroup[]>(() => {
	const byOid = new Map<string, Comment[]>();
	for (const c of comments) {
		const oid = commitOidForComment(c);
		const list = byOid.get(oid) ?? [];
		list.push(c);
		byOid.set(oid, list);
	}

	const result: CommitGroup[] = [];
	const seen = new Set<string>();
	for (const commit of commits) {
		result.push({
			oid: commit.oid,
			shortOid: commit.short_oid,
			summary: commit.summary,
			comments: sortGroupComments(byOid.get(commit.oid) ?? []),
			isSnapshot: commit.is_snapshot,
		});
		seen.add(commit.oid);
	}
	// Fallback groups for comments whose commit is gone from the session.
	for (const [oid, list] of byOid) {
		if (seen.has(oid)) continue;
		// The commit isn't in session.commits — either it's actually gone from the
		// repo (the resolver will mark each comment CommitGone and the orphan badge
		// carries the truth) or it's just not added to the review. Either way, the
		// header summary is unknown here — leave it blank and let the per-comment
		// badge speak.
		result.push({
			oid,
			shortOid: oid.slice(0, 7),
			summary: "",
			comments: sortGroupComments(list),
			isSnapshot: false,
		});
	}
	// Drop empty snapshot sections — a snapshot with no comments is noise, not a
	// section to render. Empty hand-picked commits are kept (Add-note affordance).
	return result.filter(
		(group) => !(group.isSnapshot && group.comments.length === 0),
	);
});

const hasAnyComment = $derived(comments.length > 0);

// Phase 72 — Copy state. Pattern carry-forward from the Phase 71 preview
// component (being deleted in Plan 04; this is now the canonical home).
let copied = $state(false);
// Plain handle, not $state — only used to clear; reactivity is on `copied`.
let copyTimer: ReturnType<typeof setTimeout> | null = null;

// Phase 73-01 — Lifecycle state mirrored from get_review_session_status.
// Drives the cold-boot resume branch in reload() and (in Plans 02/03) End-button
// visibility + empty-state gating. Single source of truth; assigned from
// status.state inside reload() before the resume branch reads it.
let sessionState = $state<SessionState>("none");

// Phase 73-02 — End-review two-step confirm. First click flips endConfirming
// true + arms a 3000ms revert timer; second click within the window invokes
// end_review_session and lets the session-changed listener round-trip drive
// the panel back to the cold state (D-08 — no manual array clear). Pattern
// carry-forward from `copied` / `copyTimer` above; the danger is the timer
// leak on unmount (RESEARCH Pitfall 3) — see the $effect teardown below.
let endConfirming = $state(false);
// Plain handle, not $state — only used to clear; reactivity is on `endConfirming`.
let endTimer: ReturnType<typeof setTimeout> | null = null;

// Extract a human-readable message from an unknown catch value. Handles three
// shapes: native Error (writeText / plugin throws), TrunkError (safeInvoke
// throws — plain object with .message), and anything else (coerced via String).
function errorMessage(e: unknown, fallback: string): string {
	if (e instanceof Error) return e.message;
	if (isTrunkError(e)) return e.message;
	return fallback;
}

function isOrphan(c: Comment): boolean {
	const r = resolutionById.get(c.id);
	return r !== undefined && !r.resolvable;
}

function orphanLabel(c: Comment): string | null {
	const r = resolutionById.get(c.id);
	if (r === undefined || r.resolvable || r.reason === null) return null;
	return ORPHAN_LABEL[r.reason];
}

// A line-anchored, resolvable comment is jumpable; commit-level and orphaned
// comments are not (D-07 / D-08).
function isJumpable(c: Comment): boolean {
	return c.anchor !== null && !isOrphan(c);
}

// Parse the comment's cached_excerpt into rendered lines. Diff-source excerpts
// carry +/-/space prefixes per `prefixLine` in diff-anchor.ts; full-file ones
// are plain code with no prefix. Splitting the gutter out (vs. inlining the
// `+/-` into the content span) keeps copy-paste clean.
interface ExcerptLine {
	kind: "add" | "del" | "context" | "plain";
	gutter: string;
	content: string;
}
function parseExcerpt(
	text: string,
	source: "Diff" | "FullFile",
): ExcerptLine[] {
	const lines = text.split("\n");
	if (source === "FullFile") {
		return lines.map((content) => ({ kind: "plain", gutter: " ", content }));
	}
	return lines.map((line) => {
		if (line.startsWith("+")) {
			return { kind: "add", gutter: "+", content: line.slice(1) };
		}
		if (line.startsWith("-")) {
			return { kind: "del", gutter: "-", content: line.slice(1) };
		}
		if (line.startsWith(" ")) {
			return { kind: "context", gutter: " ", content: line.slice(1) };
		}
		// Defensive fallback (e.g. blank line in the source slice).
		return { kind: "plain", gutter: " ", content: line };
	});
}

// Reads. list_session_commits / list_session_comments / resolve_session_comments
// all require an active session; a missing session is a normal state, so swallow
// no_session silently and surface only genuine load failures (UI-SPEC error copy).
async function reload() {
	// Track the canonical path so the session-changed listener can filter to this
	// repo (a missing/inactive session is a normal state — swallow silently).
	// AWAIT the status so canonicalPath is set BEFORE the listener installed by
	// the sibling $effect can fire — otherwise a session-changed event arriving
	// during the cold-start window passes the `canonicalPath && payload !== …`
	// filter (canonicalPath is still null → falsy → short-circuit fails closed,
	// the filter never triggers), and the panel reloads for unrelated repos in
	// a multi-tab session (WR-02).
	try {
		const status = await safeInvoke<SessionStatus>(
			"get_review_session_status",
			{ path: repoPath },
		);
		canonicalPath = status.canonical_path;
		sessionState = status.state;
	} catch {
		// Tolerate — the panel can still try the list reads below; if they fail
		// with no_session that's handled, otherwise a toast surfaces.
	}

	// Phase 73-01 (D-01, D-07): cold-boot resume. When the session exists on
	// disk but not in memory ("resume-available"), promote it before the reads.
	// Resume emits session-changed; our own listener re-fires reload(), which
	// then sees status === "active" and SKIPS this branch (recursion-safe by
	// gating — RESEARCH Pitfall 1). A rejection (e.g. newer_version) surfaces
	// a toast and falls through to the reads, which then return no_session
	// and the existing arm renders the cold empty state.
	if (sessionState === "resume-available") {
		try {
			await safeInvoke("resume_review_session", { path: repoPath });
		} catch (e) {
			// errorMessage extracts e.message (Error or TrunkError); the prefix
			// is added by template literal so a fallback "Failed to resume review"
			// would only fire if the value were neither shape (and the toast then
			// reads "Failed to resume review: Failed to resume review" which is
			// awkward — keep `errorMessage`'s fallback as "Failed to resume review"
			// since the prefix already conveys the action that failed).
			const msg = errorMessage(e, "unknown error");
			showToast(`Failed to resume review: ${msg}`, "error");
		}
	}

	try {
		const [nextCommits, nextComments, nextResolutions] = await Promise.all([
			safeInvoke<SessionCommit[]>("list_session_commits", { path: repoPath }),
			safeInvoke<Comment[]>("list_session_comments", { path: repoPath }),
			safeInvoke<CommentResolution[]>("resolve_session_comments", {
				path: repoPath,
			}),
		]);
		commits = nextCommits;
		comments = nextComments;
		resolutions = nextResolutions;
	} catch (e) {
		if (isTrunkError(e) && e.code === "no_session") {
			commits = [];
			comments = [];
			resolutions = [];
			return;
		}
		showToast(
			"Failed to load review comments. Reload the panel to retry.",
			"error",
		);
	}
}

function openAddNote(oid: string) {
	editingCommentId = null;
	addNoteForCommit = oid;
	draftText = "";
}

function openEdit(c: Comment) {
	addNoteForCommit = null;
	editingCommentId = c.id;
	draftText = c.text;
}

function cancelComposer() {
	addNoteForCommit = null;
	editingCommentId = null;
	draftText = "";
}

async function saveAddNote(oid: string) {
	if (!draftValid) return;
	const text = draftText;
	cancelComposer();
	try {
		await safeInvoke("add_commit_comment", {
			path: repoPath,
			commitOid: oid,
			text,
		});
	} catch (e) {
		showToast(errorMessage(e, "Failed to add note"), "error");
	}
}

async function saveEdit(id: string) {
	if (!draftValid) return;
	const text = draftText;
	cancelComposer();
	try {
		await safeInvoke("edit_comment", { path: repoPath, id, text });
	} catch (e) {
		showToast(errorMessage(e, "Failed to edit comment"), "error");
	}
}

// Phase 72 — Copy handler. The button is disabled by `!hasAnyComment`, so the
// no_comments TrunkError branch (from session.generate) is reachable only by a
// race (the session was emptied by another window between render and click) —
// surface it as a toast. The handler composes session.generate() (IPC, returns
// markdown string) with writeText() (clipboard plugin). Both are awaited inside
// one try/catch so a failure in either step lands in the same showToast call;
// the button never flips to "Copied" on failure. Carry-forward of the Phase 71
// preview component's Copy handler (now-deleted in Plan 04).
async function onCopyClick() {
	try {
		const md = await session.generate(repoPath);
		await writeText(md);
		// Pitfall 2 carry-forward: clear any in-flight revert timer before
		// scheduling a new one. Rapid re-clicks must extend the affordance,
		// not race against it.
		if (copyTimer !== null) clearTimeout(copyTimer);
		copied = true;
		copyTimer = setTimeout(() => {
			copied = false;
			copyTimer = null;
		}, 1500);
	} catch (e) {
		// `unknown` in TS strict; narrow rather than cast. Pitfall 1 + CLAUDE.md.
		const msg = e instanceof Error ? e.message : String(e);
		showToast(`Failed to copy: ${msg}`, "error");
	}
}

// Phase 73-02 — End-review two-step confirm. First click arms the confirming
// state + 3000ms revert; second click fires the IPC and lets the session-changed
// listener round-trip drive the panel back to the cold state (D-08).
function startEndConfirm() {
	// clearTimeout-before-setTimeout discipline (Pattern A): rapid re-clicks
	// extend the confirm window, not race against the previous revert timer.
	if (endTimer !== null) clearTimeout(endTimer);
	endConfirming = true;
	endTimer = setTimeout(() => {
		endConfirming = false;
		endTimer = null;
	}, 3000);
}

async function onEndClick() {
	if (!endConfirming) {
		startEndConfirm();
		return;
	}
	// Second click: clear the auto-revert timer but KEEP endConfirming = true
	// so the label stays "Click again to confirm" (frozen during await — UI-SPEC
	// § End button state machine: In-flight row). On success the session-changed
	// listener round-trip drives reload() → sessionState === "none" → the {#if}
	// gate hides the entire button. On failure we explicitly revert.
	if (endTimer !== null) {
		clearTimeout(endTimer);
		endTimer = null;
	}
	try {
		await safeInvoke("end_review_session", { path: repoPath });
		// No manual mutation of commits/comments/resolutions (D-08) — the
		// session-changed listener at the $effect below is the canonical refresh.
	} catch (e) {
		endConfirming = false;
		// Match Plan 73-01's resume-fail shape: errorMessage() extracts only
		// `.message`; the "Failed to end review: " prefix is added by template
		// literal at the call site (RESEARCH §Pattern 2). The errorMessage
		// fallback fires only when `e` is neither Error nor TrunkError.
		const msg = errorMessage(e, "unknown error");
		showToast(`Failed to end review: ${msg}`, "error");
	}
}

async function deleteComment(id: string) {
	const { ask } = await import("@tauri-apps/plugin-dialog");
	const confirmed = await ask("Delete this comment? This cannot be undone.", {
		title: "Delete comment",
		kind: "warning",
	});
	if (!confirmed) return;
	try {
		await safeInvoke("delete_comment", { path: repoPath, id });
	} catch (e) {
		showToast(errorMessage(e, "Failed to delete comment"), "error");
	}
}

// Initial load when the panel mounts / its repo changes.
$effect(() => {
	void repoPath;
	reload();
});

// Phase 73-02 — Timer cleanup on component destroy (RESEARCH Pitfall 3). If
// the panel unmounts mid-confirm (e.g. tab close, repo switch), the pending
// setTimeout would otherwise fire `endConfirming = false` against a torn-down
// component and Svelte logs an error. This effect's sole purpose is the
// teardown return — no reactive body.
$effect(() => {
	return () => {
		if (endTimer !== null) clearTimeout(endTimer);
	};
});

// Live coordination: reload on session-changed for this repo's canonical path.
// Track cancellation explicitly: if the effect tears down before listen()
// resolves, the cleanup runs with `unlisten === undefined` and the listener
// the promise eventually delivers leaks. Each remount adds another leaked
// listener. Setting `cancelled` lets the .then handler dispose immediately
// when the listener finally arrives (WR-03).
$effect(() => {
	let unlisten: (() => void) | undefined;
	let cancelled = false;
	listen<string>("session-changed", (event) => {
		if (canonicalPath && event.payload !== canonicalPath) return;
		reload();
	}).then((fn) => {
		if (cancelled) fn();
		else unlisten = fn;
	});
	return () => {
		cancelled = true;
		unlisten?.();
	};
});
</script>

<div class="flex flex-col" style="flex: 1; min-height: 0; overflow: hidden;">
  <!-- Panel-level header (Phase 72): hosts the Copy button. Disabled until the
       session has >=1 comment; the disabled tooltip and the hard backstop in
       commands/review.rs (no_comments TrunkError) form the gate. The header
       sits above the scrollable list body so the button is always visible
       while the list scrolls. -->
  <div
    class="flex items-center"
    style="
      gap: 8px;
      padding: 6px 12px;
      background: var(--color-surface);
      border-bottom: 1px solid var(--color-border);
      flex-shrink: 0;
      font-size: 12px;
    "
  >
    <span class="preview-spacer" style="flex: 1;"></span>
    {#if sessionState !== "none"}
      <button
        type="button"
        class="end-button {endConfirming ? 'confirming' : ''} flex items-center"
        onclick={onEndClick}
        title={endConfirming
          ? ""
          : "End the current review and delete the on-disk session"}
      >
        <Trash2 size={14} />
        <span>{endConfirming ? "Click again to confirm" : "End review"}</span>
      </button>
    {/if}
    <button
      type="button"
      class="copy-button flex items-center"
      onclick={onCopyClick}
      disabled={!hasAnyComment}
      title={hasAnyComment ? "" : "Add at least one comment to generate"}
    >
      {#if copied}
        <span aria-hidden="true">✓</span>
        <span>Copied</span>
      {:else}
        <Clipboard size={14} />
        <span>Copy</span>
      {/if}
    </button>
  </div>
  <div
    class="flex flex-col"
    style="
      flex: 1;
      min-height: 0;
      overflow: auto;
      padding: 12px;
      background: var(--color-surface);
      color: var(--color-text);
      font-size: 12px;
      line-height: 1.5;
    "
  >
  <!-- Phase 73-03 — Session summary caption (D-04). Visible whenever a session
       exists (cold branch hides it); sits ABOVE the empty-state block so the
       count is the first thing the eye lands on when the body has content. -->
  {#if sessionState !== "none"}
    <span style="color: var(--color-text-muted); font-size: 11px; padding: 2px 0;">
      {comments.length} comments · {commits.length} commits
    </span>
  {/if}

  <!-- Phase 73-03 — Three-way empty-state branching (D-06). Order is specificity-
       first: cold (no session) → warm-no-commits (existing copy preserved
       verbatim) → warm-with-commits-zero-comments (replaces prior "No comments
       yet." copy). The three branches are mutually exclusive; when the user has
       added at least one comment, none render and the list below takes over. -->
  {#if sessionState === "none"}
    <div class="flex flex-col" style="gap: 4px; padding: 12px;">
      <span>No active review</span>
      <span style="color: var(--color-text-muted); font-size: 11px;">
        Toggle review mode in the toolbar to start.
      </span>
    </div>
  {:else if commits.length === 0 && !hasAnyComment}
    <div class="flex flex-col" style="gap: 4px; padding: 12px;">
      <span>No commits in this review yet.</span>
      <span style="color: var(--color-text-muted); font-size: 11px;">
        Add commits from the graph to start reviewing.
      </span>
    </div>
  {:else if !hasAnyComment}
    <div class="flex flex-col" style="gap: 4px; padding: 12px;">
      <span>Review started.</span>
      <span style="color: var(--color-text-muted); font-size: 11px;">
        Select diff lines or add a commit note to comment.
      </span>
    </div>
  {/if}

  {#if groups.length > 0}
    <ul class="flex flex-col" style="gap: 8px; list-style: none; margin: 0; padding: 0;">
      {#each groups as group (group.oid)}
        <li class="flex flex-col" style="gap: 4px;">
          <!-- Commit group header (focal point): short SHA mono 600 + summary -->
          <div
            class="flex items-center"
            style="gap: 8px; padding: 2px 0; border-bottom: 1px solid var(--color-border);"
          >
            <button
              type="button"
              title="Copy SHA"
              aria-label="Copy SHA {group.shortOid}"
              onclick={() => copySha(group.oid)}
              class="jump-ref font-mono"
              style="
                background: transparent;
                border: none;
                padding: 0;
                cursor: pointer;
                font-size: 13px;
                font-weight: 600;
                color: inherit;
                font-family: inherit;
                flex-shrink: 0;
              "
            >{group.shortOid}</button>
            <button
              type="button"
              aria-label="Jump to commit {group.shortOid}"
              onclick={() => onJumpToCommit(group.oid)}
              class="jump-ref overflow-hidden text-ellipsis whitespace-nowrap"
              style="
                background: transparent;
                border: none;
                padding: 0;
                cursor: pointer;
                text-align: left;
                font-size: 13px;
                font-weight: 600;
                color: inherit;
                font-family: inherit;
                flex: 1;
              "
            >{group.summary}</button>
            <button
              type="button"
              class="flex items-center"
              onclick={() => openAddNote(group.oid)}
              style="
                gap: 4px;
                background: transparent;
                color: var(--color-text-muted);
                border: none;
                border-radius: 4px;
                cursor: pointer;
                padding: 2px 4px;
                flex-shrink: 0;
                font-size: 12px;
              "
              onmouseenter={(e) => (e.currentTarget.style.background = "var(--color-hover)")}
              onmouseleave={(e) => (e.currentTarget.style.background = "transparent")}
            >
              <MessageSquarePlus size={14} />
              <span>Add note</span>
            </button>
          </div>

          <!-- Inline add-note composer for this commit -->
          {#if addNoteForCommit === group.oid}
            <div class="flex flex-col" style="gap: 4px; padding: 4px 0;">
              <textarea
                bind:value={draftText}
                rows="3"
                style="
                  width: 100%;
                  resize: vertical;
                  background: var(--color-bg);
                  color: var(--color-text);
                  border: 1px solid var(--color-border);
                  border-radius: 4px;
                  padding: 4px 6px;
                  font-size: 12px;
                  font-family: inherit;
                "
              ></textarea>
              <div class="flex items-center" style="gap: 4px;">
                <button
                  type="button"
                  onclick={() => saveAddNote(group.oid)}
                  disabled={!draftValid}
                  style="
                    background: transparent;
                    color: var(--color-text);
                    border: 1px solid var(--color-border);
                    border-radius: 4px;
                    cursor: pointer;
                    padding: 2px 8px;
                    font-size: 12px;
                  "
                >Save</button>
                <button
                  type="button"
                  onclick={cancelComposer}
                  style="
                    background: transparent;
                    color: var(--color-text-muted);
                    border: 1px solid var(--color-border);
                    border-radius: 4px;
                    cursor: pointer;
                    padding: 2px 8px;
                    font-size: 12px;
                  "
                >Cancel</button>
              </div>
            </div>
          {/if}

          {#if group.comments.length === 0}
            <span style="color: var(--color-text-muted); font-size: 11px; padding: 2px 0;">
              No comments on this commit.
            </span>
          {:else}
            <ul class="flex flex-col" style="gap: 4px; list-style: none; margin: 0; padding: 0;">
              {#each group.comments as comment (comment.id)}
                <li class="comment-card">
                  <!-- Header: file ref (jump affordance) + orphan badge + actions -->
                  <header class="comment-card-header">
                    {#if comment.anchor !== null}
                      {#if isJumpable(comment)}
                        <button
                          type="button"
                          aria-label="Jump to code"
                          onclick={() => onJump(comment)}
                          class="jump-ref font-mono comment-card-fileref"
                        >{comment.anchor.file_path}:L{comment.anchor.start_line}-L{comment.anchor.end_line}</button>
                      {:else}
                        <span
                          class="font-mono comment-card-fileref"
                          class:comment-card-fileref-dim={isOrphan(comment)}
                        >{comment.anchor.file_path}:L{comment.anchor.start_line}-L{comment.anchor.end_line}</span>
                      {/if}
                    {/if}
                    <span class="comment-card-spacer"></span>
                    {#if orphanLabel(comment)}
                      <span class="orphan-badge">{orphanLabel(comment)}</span>
                    {/if}
                    {#if editingCommentId !== comment.id}
                      <button
                        type="button"
                        class="card-action"
                        onclick={() => openEdit(comment)}
                      >Edit</button>
                      <button
                        type="button"
                        class="card-action card-action-danger"
                        onclick={() => deleteComment(comment.id)}
                      >Delete</button>
                    {/if}
                  </header>

                  <!-- Diff hunk: line-anchored comments only. The cached_excerpt
                       is the canonical body; render with red/green per-line bg
                       for Diff-source +/- lines, plain for full-file content.
                       No syntax highlighting (the project's syntect-based path
                       isn't wired into the panel — deferred). -->
                  {#if comment.anchor !== null && comment.cached_excerpt}
                    <div
                      class="comment-card-diff"
                      class:comment-card-diff-dim={isOrphan(comment)}
                    >
                      {#each parseExcerpt(comment.cached_excerpt, comment.anchor.source) as line, i (i)}
                        <div class="diff-line diff-line-{line.kind}">
                          <span class="diff-gutter">{line.gutter}</span>
                          <span class="diff-content">{line.content}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}

                  <!-- Body: comment text or inline editor (D-10). Comment text
                       stays at full --color-text even when orphaned (D-08). -->
                  <div class="comment-card-body">
                    {#if editingCommentId === comment.id}
                      <textarea
                        bind:value={draftText}
                        rows="3"
                        class="card-textarea"
                      ></textarea>
                      <div class="card-editor-actions">
                        <button
                          type="button"
                          onclick={() => saveEdit(comment.id)}
                          disabled={!draftValid}
                        >Save</button>
                        <button
                          type="button"
                          onclick={cancelComposer}
                        >Cancel</button>
                      </div>
                    {:else}
                      <span class="comment-card-text">{comment.text}</span>
                    {/if}
                  </div>
                </li>
              {/each}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
  </div>
</div>

<style>
  .jump-ref:hover,
  .jump-ref:focus-visible {
    color: var(--color-accent);
    text-decoration: underline;
  }

  /* GitHub-review-style card per comment. */
  .comment-card {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: var(--color-bg);
    overflow: hidden;
  }
  .comment-card-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    font-size: 11px;
  }
  .comment-card-spacer { flex: 1; }
  .comment-card-fileref {
    font-size: 11px;
    line-height: 1.4;
    color: var(--color-text-muted);
    background: transparent;
    border: none;
    padding: 0;
    text-align: left;
    font-family: inherit;
    cursor: pointer;
  }
  .comment-card-fileref-dim { opacity: var(--opacity-dimmed); }

  /* Diff hunk inside the card — line-level red/green backgrounds, no
     syntax highlighting (deferred). */
  .comment-card-diff {
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 11px;
    line-height: 1.5;
    border-bottom: 1px solid var(--color-border);
  }
  .comment-card-diff-dim { opacity: var(--opacity-dimmed); }
  .diff-line {
    display: flex;
  }
  .diff-line-add { background: var(--color-diff-add-bg); }
  .diff-line-del { background: var(--color-diff-delete-bg); }
  .diff-line-context,
  .diff-line-plain { background: transparent; }
  .diff-gutter {
    flex-shrink: 0;
    width: 18px;
    padding: 0 4px;
    text-align: center;
    user-select: none;
    color: var(--color-text-muted);
  }
  .diff-content {
    flex: 1;
    min-width: 0;
    padding-right: 8px;
    white-space: pre-wrap;
    word-break: break-all;
  }

  /* Body */
  .comment-card-body {
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .comment-card-text {
    white-space: pre-wrap;
    word-break: break-word;
  }

  /* Inline action buttons in the header. */
  .card-action {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0 4px;
    font-size: 12px;
    color: var(--color-text-muted);
  }
  .card-action:hover,
  .card-action:focus-visible { color: var(--color-text); }
  .card-action-danger { color: var(--color-danger); }
  .card-action-danger:hover,
  .card-action-danger:focus-visible { color: var(--color-danger); }

  /* Orphan badge */
  .orphan-badge {
    font-size: 11px;
    line-height: 1.4;
    color: var(--color-warning);
    background: var(--color-warning-bg);
    border-radius: 4px;
    padding: 0 6px;
    white-space: nowrap;
  }

  /* Inline editor inside the body. */
  .card-textarea {
    width: 100%;
    resize: vertical;
    background: var(--color-bg);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    padding: 4px 6px;
    font-size: 12px;
    font-family: inherit;
  }
  .card-editor-actions {
    display: flex;
    gap: 4px;
  }
  .card-editor-actions button {
    background: transparent;
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 8px;
    font-size: 12px;
  }
  .card-editor-actions button[disabled] {
    cursor: not-allowed;
    opacity: 0.5;
  }

  /* Phase 72 Copy button — lives in the panel header. Carry-forward from the
     deleted Phase 71 preview component. */
  .copy-button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 8px;
    font-size: 12px;
    font-family: inherit;
  }
  .copy-button:hover:not([disabled]),
  .copy-button:focus-visible:not([disabled]) {
    color: var(--color-text);
    background: var(--color-hover);
  }
  .copy-button[disabled] {
    cursor: not-allowed;
    opacity: 0.5;
  }

  /* Phase 73-02 End-review button — danger-tinted sibling of .copy-button.
     Idle: muted text on transparent (visually subordinate to Copy). Confirming:
     danger-bg + danger-border + on-accent text per UI-SPEC § Interaction States.
     All colors via existing :root tokens in src/app.css (no hex/rgb literals). */
  .end-button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: transparent;
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    padding: 2px 8px;
    font-size: 12px;
    font-family: inherit;
  }
  .end-button:hover:not(.confirming):not([disabled]),
  .end-button:focus-visible:not(.confirming):not([disabled]) {
    color: var(--color-text);
    background: var(--color-hover);
  }
  .end-button.confirming {
    color: var(--fg-1);
    background: var(--color-danger-bg);
    border: 1px solid var(--color-danger-border);
  }
  .end-button.confirming:hover,
  .end-button.confirming:focus-visible {
    background: var(--color-danger-bg-strong);
    border: 1px solid var(--color-danger);
  }
</style>
