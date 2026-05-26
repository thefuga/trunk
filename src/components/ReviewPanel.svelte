<script lang="ts">
// Phase 69 — the real review panel (replaces the Phase 65 throwaway stub).
// Renders the accumulated review grouped by commit (D-09), with a per-commit
// "Add note" affordance (D-02), inline edit (D-10), delete-with-confirm (D-05),
// and jump-to-anchor with read-only orphan rows (D-07 / D-08). The panel lives
// in the center pane (UI-SPEC:133); jump is driven by the host via onJump.

import { MessageSquarePlus } from "@lucide/svelte";
import { listen } from "@tauri-apps/api/event";
import { safeInvoke, type TrunkError } from "../lib/invoke.js";
import { showToast } from "../lib/toast.svelte.js";
import type {
	Comment,
	CommentResolution,
	OrphanReason,
	SessionCommit,
	SessionStatus,
} from "../lib/types.js";

interface Props {
	repoPath: string;
	// Resolvable-comment jump: the host (RepoView) binds this to the review-session
	// rune's jumpTo, wiring commit/file selection + scroll-to-range.
	onJump: (comment: Comment) => void;
	// Commit-header jump: select the commit and scroll the graph to it. Same
	// gesture as clicking a line ref, but without a file/line — the panel stays.
	onJumpToCommit: (commitOid: string) => void;
}

let { repoPath, onJump, onJumpToCommit }: Props = $props();

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
// so nothing is dropped (D-08).
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
		});
	}
	return result;
});

const hasAnyComment = $derived(comments.length > 0);

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
	} catch {
		// Tolerate — the panel can still try the list reads below; if they fail
		// with no_session that's handled, otherwise a toast surfaces.
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
		const err = e as TrunkError;
		if (err.code === "no_session") {
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
		showToast((e as TrunkError).message ?? "Failed to add note", "error");
	}
}

async function saveEdit(id: string) {
	if (!draftValid) return;
	const text = draftText;
	cancelComposer();
	try {
		await safeInvoke("edit_comment", { path: repoPath, id, text });
	} catch (e) {
		showToast((e as TrunkError).message ?? "Failed to edit comment", "error");
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
		showToast((e as TrunkError).message ?? "Failed to delete comment", "error");
	}
}

// Initial load when the panel mounts / its repo changes.
$effect(() => {
	void repoPath;
	reload();
});

// Live coordination: reload on session-changed for this repo's canonical path.
$effect(() => {
	let unlisten: (() => void) | undefined;
	listen<string>("session-changed", (event) => {
		if (canonicalPath && event.payload !== canonicalPath) return;
		reload();
	}).then((fn) => {
		unlisten = fn;
	});
	return () => {
		unlisten?.();
	};
});
</script>

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
  {#if groups.length === 0}
    <div class="flex flex-col" style="gap: 4px; padding: 12px;">
      <span>No commits in this review yet.</span>
      <span style="color: var(--color-text-muted); font-size: 11px;">
        Add commits from the graph to start reviewing.
      </span>
    </div>
  {:else if !hasAnyComment}
    <div class="flex flex-col" style="gap: 4px; padding: 12px;">
      <span>No comments yet.</span>
      <span style="color: var(--color-text-muted); font-size: 11px;">
        Select lines in a diff to comment, or add a note to a commit above.
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
              aria-label="Jump to commit {group.shortOid}"
              onclick={() => onJumpToCommit(group.oid)}
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
            <span
              class="overflow-hidden text-ellipsis whitespace-nowrap"
              style="font-size: 13px; font-weight: 600; flex: 1;"
            >{group.summary}</span>
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
</style>
