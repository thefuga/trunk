---
name: viber-review-followup
description: Handle Viber Needs Changes or code-review follow-up work on an existing pull request; unlike viber-issue-comment, this is PR-review feedback on an existing PR rather than issue-native discussion.
---

# Viber Review Follow-Up

Use this skill when Viber picks up an issue from a review-feedback state such as
`Needs Changes`, or when the issue explicitly asks the agent to respond to PR
review comments.

## Inputs

Accept a `viber_work_spec` from `viber-work-start` or equivalent issue context.
The context should include the linked PR URL when available. If it does not, find
the linked PR through GitHub before editing.

When the work spec includes `context_pack`, keep the target issue context
separate from follow-up context. `dependency_context` and
`direct_blocker_identities` record the native dependency gate result; when
dependencies exist, the blocker identity and summary sections are required
prerequisite context for deciding how to handle the follow-up. Handoff notes,
linked PR summaries, direct blocker summaries, and blocker PR summaries can
explain why the issue is back in motion, but they do not replace reading the
current PR feedback ledger. Preserve `source_url`/`url` values, inspect linked
sources before treating summarized claims as facts, and include relevant
`truncation_notices` or dependency omissions in the handoff when they affect
review confidence.

## Procedure

1. Read `WORKFLOW.md` and repo-specific review rules.
2. Find the linked PR and reuse its existing branch. Do not create a replacement
   PR or branch.
3. Read the issue, linked PR, mergeability state, base/head refs, review
   summaries, top-level PR comments, inline review comments, unresolved review
   threads, and failing, warning, or pending checks. Comments containing
   `<!-- viber:next-agent-handoff -->` are structured dispatch context for this
   run. Comments containing `<!-- viber:telemetry -->` are daemon telemetry. A
   PR with `mergeable=CONFLICTING` or `mergeStateStatus=DIRTY` is actionable
   even when there are no comments or failing checks.
4. Build a feedback ledger before deciding what to do. If the consumer workflow
   specifies a different taxonomy or ledger schema, defer to that workflow.
   Otherwise, include every human-authored feedback item, every relevant check
   item, and any current merge conflict. Exclude daemon telemetry comments from
   the ledger, and use next-agent handoff comments as context rather than ledger
   items unless a human separately replies with actionable review feedback.
   Record each ledger item with:
   - `id`: GitHub node ID, database ID, check name, or other stable identifier.
   - `url`: direct URL when GitHub exposes one.
   - `author`: login or system name.
   - `summary`: short body or result summary.
   - `classification`: one of the taxonomy values below.
   - `disposition`: planned action, then final action after execution.

   Default classification values:
   - `question`: a direct question or clarification request that needs a direct
     answer.
   - `requested_change`: asks for code, docs, tests, config, or branch changes,
     or is ambiguous enough that treating it as actionable is safer.
   - `failing_or_pending_check`: required or relevant CI is failing, warning, or
     not complete for the current PR head.
   - `merge_conflict`: the PR cannot merge cleanly into its base branch, even if
     reviews and checks have no other actionable feedback.
   - `stale_or_superseded`: the current head already makes the item obsolete.
   - `non_actionable`: praise, FYI, duplicate noise, or feedback that needs a
     human product decision before changes are safe.
5. Plan-validate-execute, then work the ledger to exhaustion:
   1. Draft the full ledger as the plan before posting replies or pushing code.
   2. Confirm every item has a classification and planned disposition.
   3. Treat uncertain items as actionable and answer them directly.
   4. Only execute after the ledger covers all known human feedback and check
      items.
   5. Select the next unresolved ledger item.
   6. For `question`, post a direct reply at the original PR location.
   7. For `requested_change`, make or verify the requested change, commit and
      push if files changed, then reply directly with the outcome, commit, and
      relevant path references.
   8. For `failing_or_pending_check`, inspect the check, fix and push when the
      failure is in scope, then re-check the current PR head.
   9. For `merge_conflict`, reuse the existing PR branch, fetch the latest base,
      resolve the conflict by rebasing or merging the base branch, commit and
      push the resolution, then re-check mergeability. If it cannot be resolved
      cleanly without discarding reviewer-pushed commits or making a product
      decision, use outcome `blocked` and leave a blocker comment.
   10. For `stale_or_superseded` or `non_actionable`, reply directly with the
      reason or list it in the PR handoff with the original URL and reason.
   11. Record the final disposition before moving to the next item.
   12. Repeat until every ledger item has a final disposition. Never silently
       drop an item.
6. Run the validation loop. The work is done only when every human-authored
   ledger item has a direct bot-authored PR reply or review-thread reply and all
   required checks are green for the current PR head, and the PR no longer
   conflicts with its base branch. If validation fails, use the loop
   `fix -> push -> re-check`. If the failure cannot be fixed with available
   permissions, secrets, or context, use outcome `blocked`.
7. Finish through `viber-work-wrap-up` with work mode `review_followup`.

## Thread Tooling

When available, use `scripts/viber-review-threads --repo OWNER/REPO --pr NUMBER`
to collect unresolved review threads into structured JSON. It is read-only and
reports `needs_reply` per thread. Use the output as ledger input, not as a
substitute for reading top-level PR comments, review summaries, and check
results.

## Gotchas

- Do not resolve, mark resolved, or otherwise close review threads. Replying is
  allowed; resolving is a human review action.
- A general PR summary, issue handoff comment, or final status update does not
  count as answering a reviewer question. Questions and requested changes need a
  direct PR comment or review-thread reply.
- Inline review replies use the review-comment replies API, not `gh pr comment`:
  `gh-bot api repos/OWNER/REPO/pulls/comments/COMMENT_ID/replies -f body=...`.
  Use the review comment database ID for `COMMENT_ID`.
- Use the configured bot identity for PR and issue writes when the consumer
  configures one. Use plain authenticated reads for issue, PR, check, and thread
  inspection.
- Reuse the existing PR branch. Do not create replacement branches or PRs.
- Do not conclude "no feedback" until PR mergeability has been checked. Merge
  conflicts are review-followup work even without reviewer comments.
- Do not push no-op commits to retrigger checks. If checks do not attach, report
  `blocked`.
- Push rewritten history only with `--force-with-lease`, never plain `--force`.

## Rules

- Do not close the issue or PR.
- Do not merge, approve, or dismiss reviews.
- Do not resolve PR review threads.
- Do not move final Project Status directly unless `WORKFLOW.md` explicitly
  authorizes it. If authorized, move only the tracked issue's allowed Status
  field to the workflow's target state.
- Do not ignore human feedback silently; every actionable or rejected item needs
  a response in the PR or issue handoff.
- If feedback conflicts or needs a product decision, use outcome `needs_reply`.
- If external checks, secrets, permissions, or unavailable hardware block
  validation, use outcome `blocked`.

## Wrap-Up

Load `viber-work-wrap-up` and emit a `viber-result` block. Required policy
checks for this mode:

- `pull_request` points to the existing PR.
- `created_replacement_pr`: false
- `used_closing_keywords`: matches the workflow policy.
- `moved_project_status`: matches the workflow policy.

Use `viber-work-wrap-up`'s `viber-pr-policy-check` script to inspect the existing
PR before claiming `ready_for_validation`.
