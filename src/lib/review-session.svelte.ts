import { safeInvoke } from "./invoke.js";
import type { Comment, Side } from "./types";

/**
 * Owns Review-mode center-pane state and the jump action (D-07).
 *
 * Per UI-SPEC:133 the review panel claims the CENTER pane; a jump is a
 * single-pane swap (panel -> diff -> back via the "Review" toggle), NOT a
 * relocation to the right pane. `rightPaneMode` names which of {panel, diff}
 * the center pane currently shows; `reviewActive` tracks whether the review
 * session is the active center-pane context.
 *
 * This module is a pure state+action factory: it never imports RepoView and
 * never reaches into its internals. `jumpTo` composes injected callbacks
 * (`JumpDeps`) that Plan 05 binds to RepoView's concrete selection/scroll
 * machinery, keeping the rune decoupled from the component.
 */

export type RightPaneMode = "panel" | "diff";

// Phase 70 D-02: the ReviewPanel swaps in-place between the comment list and a
// markdown preview of the generated review doc. The preview slot is panel-
// internal — sibling to `rightPaneMode`'s panel-vs-diff dimension, not a
// replacement for it.
export type PanelMode = "list" | "preview";

export interface ReviewSessionState {
	reviewActive: boolean;
	rightPaneMode: RightPaneMode;
	// Phase 70 D-02: which face the ReviewPanel currently shows.
	panelMode: PanelMode;
	// Phase 70 D-02: cached generated doc; null when no preview has been
	// generated for this session. Re-generating is the only invalidation path
	// (swap-back to list preserves the cache).
	previewMarkdown: string | null;
}

// The navigation seams jumpTo composes. Plan 05 binds these to RepoView's
// commit/file selection and scroll machinery; each may be sync or async.
// `side` is threaded through so the diff scroll target picks the right
// hunk-line coordinate (Side::Old indexes the parent tree's line numbers,
// Side::New the commit's own tree). Without it, an Old-side anchor's
// line number is checked against new_start/new_lines and misses the hunk.
export interface JumpDeps {
	selectCommit(oid: string): void | Promise<void>;
	selectFile(path: string): void | Promise<void>;
	scrollToRange(startLine: number, endLine: number, side: Side): void;
}

export interface ReviewSessionManager {
	state: ReviewSessionState;
	setReviewActive(active: boolean): void;
	showPanel(): void;
	showDiff(): void;
	jumpTo(comment: Comment, deps: JumpDeps): Promise<void>;
	// Phase 70 D-02: ReviewPanel panel-internal view-swap actions.
	showList(): void;
	showPreview(md: string): void;
	// Phase 70 DOC-01: calls `generate_review_doc` IPC and, on success, stores
	// the returned markdown and swaps the panel to preview. Rejection propagates
	// to the caller for toast surfacing; state is NOT partially updated.
	generate(repoPath: string): Promise<void>;
}

export function createReviewSession(): ReviewSessionManager {
	const state: ReviewSessionState = $state({
		reviewActive: false,
		rightPaneMode: "panel" as RightPaneMode,
		panelMode: "list" as PanelMode,
		previewMarkdown: null,
	});

	return {
		state,
		setReviewActive(active: boolean) {
			state.reviewActive = active;
			// Deactivation ends the session — the cached preview belongs to that
			// session's snapshot and is no longer meaningful. Reactivation does NOT
			// touch the preview fields (the rune is reused across activate-toggles
			// within the same logical session; only the explicit teardown clears).
			if (!active) {
				state.previewMarkdown = null;
				state.panelMode = "list";
			}
		},
		showList() {
			state.panelMode = "list";
		},
		showPreview(md: string) {
			state.previewMarkdown = md;
			state.panelMode = "preview";
		},
		async generate(repoPath: string) {
			// IMPORTANT: the assignment order is "await then write." A rejection
			// from safeInvoke (e.g. TrunkError code "no_comments") propagates
			// before any state mutation, so the panel stays on the list view and
			// the cached previewMarkdown (if any) is untouched.
			const md = await safeInvoke<string>("generate_review_doc", {
				path: repoPath,
			});
			state.previewMarkdown = md;
			state.panelMode = "preview";
		},
		showPanel() {
			state.rightPaneMode = "panel";
		},
		showDiff() {
			state.rightPaneMode = "diff";
		},
		async jumpTo(comment: Comment, deps: JumpDeps) {
			// Commit-level or orphaned comments have no line anchor and thus no
			// jump target (D-08) — stay on the panel, navigate nowhere.
			if (comment.anchor === null) return;

			const anchor = comment.anchor;
			await deps.selectCommit(anchor.commit_oid);
			// The file's Source decides diff vs full-file view downstream.
			await deps.selectFile(anchor.file_path);
			state.rightPaneMode = "diff";
			deps.scrollToRange(anchor.start_line, anchor.end_line, anchor.side);
		},
	};
}
