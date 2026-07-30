---
name: viber-issue-comment
description: Handle Viber issue-native discussion work where the artifact is one GitHub issue comment, not file changes, branches, commits, or PRs. Use for RFCs, design discussions, issue clarification Q&A, and comment-only handoffs. Do not use for PR review replies or reviewer-response work on an existing PR; use viber-review-followup.
---

# Viber Issue Comment

Use this skill when the issue itself is the work surface. This includes RFCs,
design discussions, clarification replies, and follow-up comments from human
handoff states.

For exploratory Spike, POC, Investigation, or similar issues where
`viber-work-start` reports `signals.work_mode_needs_clarification`, the comment
artifact is a focused artifact-contract question. Ask whether the expected
deliverable is findings in an issue comment, a reusable executable probe or
prototype in an implementation PR, or an ADR/design record PR. Finish with
`outcome: "needs_reply"` after posting that one comment.

Do not use this skill for PR review replies, unresolved review threads, or
reviewer-response work on an existing pull request. Use `viber-review-followup`
for that mode.

## Inputs

Accept a `viber_work_spec` from `viber-work-start` or equivalent issue context.
Read the issue body and comments before writing a response. For RFCs, comments
are required context, not optional background.

When the work spec includes `context_pack`, use its `target_issue_context` as the
discussion target, its `dependency_context`/`direct_blocker_identities` as the
native dependency gate result, and its handoff notes, linked PR summaries,
direct blocker summaries, and blocker PR summaries as follow-up context. When
dependencies exist, the blocker identity and summary sections are required
prerequisite context for the comment. The pack is a bounded summary with source
URLs and possible truncation. Preserve `source_url`/`url` values in your notes,
verify material claims against linked sources before presenting them as facts,
and mention relevant `truncation_notices` or dependency omissions in the final
comment when they limit confidence.

## Procedure

1. Read `WORKFLOW.md` and repo guidance only as needed to answer the issue.
2. Read the GitHub issue body and all relevant comments. Treat comments
   containing `<!-- viber:next-agent-handoff -->` as structured dispatch
   context, and exclude comments containing `<!-- viber:telemetry -->` from any
   feedback or discussion ledger. Reads are read-only and may use the plain
   authenticated GitHub CLI/API unless the workflow says otherwise.
3. Identify the concrete question, decision, or discussion thread to advance.
4. Inspect repository files only when a specific claim needs verification.
5. Draft one focused issue comment.
6. Validate the draft shape before posting:
   - for RFCs, design answers, or final substantive recommendations, confirm the
     required sections are present, including `Alternatives Considered` and
     `Risks`;
   - for a short clarification or one-line answer, confirm it is direct and do
     not force the full template when it would add noise;
   - confirm the target repository, issue number, and issue URL.
7. Post the comment only after the draft passes shape validation. Posting is the
   artifact and should be treated as irreversible.
8. Re-read the issue after posting and confirm the new comment landed on the
   intended issue.
9. Finish through `viber-work-wrap-up` with work mode `issue_comment`.

## Post Path and Identity

Use the consumer-configured bot identity for the mutating comment write when one
is present, such as a workflow-provided wrapper around `gh issue comment`. If no
bot identity is configured, use the plain authenticated GitHub CLI/API.

Do not use the posting identity for read-only discovery unless the workflow
requires it. If a configured bot path exists but cannot post, treat that as a
blocker instead of silently changing the author identity.

## Plan, Validate, Execute

Before mutating GitHub, make the irreversible step explicit:

1. Plan the response target and intended outcome.
2. Draft the full comment text.
3. Validate the comment shape and target issue.
4. Post the comment with the correct identity.
5. Re-read the issue to confirm the comment landed correctly.

## Gotchas

- Posting the issue comment is the artifact. If you changed a repository file,
  created a branch, committed, pushed, or opened a PR, you are in the wrong work
  mode.
- Do not force the full template onto a one-line clarification. A final RFC or
  design answer should still carry `Alternatives Considered` and `Risks`.
- Status moves, when the workflow authorizes them, are the last action and must
  be discovered from the workflow or Project metadata rather than assumed.
- An issue-native RFC stays an issue comment unless the issue explicitly asks for
  a committed artifact such as an ADR, design document, or docs PR.

## Comment Shape

For RFCs, design discussions, or final substantive recommendations, prefer this
shape as a generic quality default:

```markdown
## Recommendation

<direct recommendation or answer>

## Reasoning

- <tradeoff or evidence>
- <tradeoff or evidence>

## Alternatives Considered

- <alternative and why it was not chosen>

## Risks

- <risk, mitigation, or "None identified">

## Open Questions

- <question, or "None" when complete>

## Validation

- <issue comments/docs/files reviewed>
```

This is not a hard external approval gate; it is the default shape for comments
that answer an RFC, make a recommendation, or close a substantive discussion
loop. For a short follow-up question or answer, keep the comment shorter.

## Rules

- Do not create a branch, commit, push, or open a pull request.
- Do not edit repository files.
- Do not close the issue.
- Do not handle PR review replies or reviewer-response work; use
  `viber-review-followup`.
- Do not move final Project Status directly unless `WORKFLOW.md` explicitly
  authorizes it. If authorized, move only the tracked issue's allowed Status
  field to the workflow's target state.
- Do not convert an issue-native RFC into an ADR unless the issue explicitly asks
  for a committed repo artifact.
- Prefer `needs_reply` when the next useful step is a human answer.
- Prefer `needs_reply` for exploratory issues whose artifact contract is
  missing, unclear, unknown, or contradictory; do not create a branch or file to
  answer that ambiguity.
- Prefer `ready_for_validation` when the comment fully answers the issue and a
  human should validate or accept it.

## Wrap-Up

Load `viber-work-wrap-up` and emit a `viber-result` block. Required policy
checks for this mode:

- `created_pr`: false
- `changed_files`: false
- `used_closing_keywords`: false
- `moved_project_status`: matches the workflow policy.
