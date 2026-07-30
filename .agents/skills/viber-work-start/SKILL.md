---
name: viber-work-start
description: Start a Viber autonomous issue run after the daemon has claimed the issue and prepared the workspace. Use this when a Viber prompt needs to read GitHub issue context, classify the work mode, extract acceptance criteria, and delegate to the correct Viber work-mode skill. This skill is for Viber-managed runs, not ordinary human-directed local work.
---

# Viber Work Start

Use this skill at the beginning of a Viber-managed run, after Viber has selected
the issue and prepared the workspace. The goal is to normalize issue context and
choose the right work-mode skill without guessing from prose alone.

## Inputs

Expect the prompt or workflow to provide:

- GitHub issue identifier, URL, title, state, body, labels, and Project fields.
- Current Project Status and configured active/handoff states.
- Repository `WORKFLOW.md` and repo-local agent instructions.
- Optional linked pull request or review context for follow-up runs.

If any of these are missing, read them with available local files or `gh` before
starting production work. For issue-native discussion work, read issue comments by
default because the conversation is the work surface.

## Script Support

When available, use the bundled context script instead of hand-assembling GitHub
state from multiple ad hoc `gh` calls:

```sh
scripts/viber-gh-context --repo OWNER/REPO --issue NUMBER --project-owner OWNER --project-number NUMBER --terminal-status Done --max-comments 20 --max-comment-chars 4000
```

The script returns normalized issue body, labels, comments, Project item fields,
linked PRs when exposed by the Project item list, and a best-effort work-mode
hint. It also emits `context_pack`, a bounded, typed copy of the assembled
target issue context, incoming handoff notes, linked pull request summaries,
direct blocker summaries, ordinary comments, source URLs, and truncation
notices. Treat comments containing `<!-- viber:next-agent-handoff -->` as
structured dispatch context for the run, not as reviewer feedback, and exclude
comments containing `<!-- viber:telemetry -->` from feedback or discussion
ledgers. It bounds issue body and comment output by default; use
`--max-comments`, `--max-comment-chars`, or `--no-comments` to tune the context
size. For each effective repository terminal state, pass one
`--terminal-status` flag. Read those states from `tracker.terminal_states` in
`WORKFLOW.md` or the resolved Viber config; when the field is absent, pass the
documented Viber default explicitly (`Done`) instead of relying on the helper's
compatibility fallback. Project subcommands clear `GH_TOKEN` and `GITHUB_TOKEN`
so those environment variables do not shadow the authenticated `gh` account used
for user-owned Projects.

The script owns default mode alias normalization. Use `signals.work_mode_hint`,
`signals.work_mode_reason`, `signals.work_mode_outcome_hint`,
`signals.work_mode_needs_clarification`, `signals.artifact_contract`, and
`signals.mode_rules` as the source of truth rather than maintaining a second
mapping table in this skill. Treat the hint as evidence to validate against
repository policy; the consumer workflow remains the lifecycle authority.
`kind/design` and `type/rfc` remain comment-oriented by default, but explicit
requests for an ADR, design document, or repository artifact route as
`design_doc_pr`. Spike, POC, Investigation, and equivalent exploratory Project
or native issue Types are artifact-contract-sensitive: route clear findings
comments to `issue_comment`, reusable executable probes to `implementation_pr`,
explicit ADR/design records to `design_doc_pr`, and unclear or contradictory
artifact intent to one clarification comment with outcome `needs_reply`.

## Work Mode Selection

Select exactly one work mode using this precedence:

1. Explicit Project field such as `Work Mode`, if present.
2. Explicit label such as `mode/issue-comment`, `mode/design-doc`,
   `mode/implementation-pr`, or `mode/review-followup`.
3. Current Status indicating review feedback, such as `Needs Changes`.
4. Project `Type`, native issue type, labels, or title prefix.
5. Repository default from `WORKFLOW.md`.

When using `viber-gh-context`, start from `signals.work_mode_hint` and verify the
reason matches the precedence above. When the script is unavailable, use the
same canonical mode names (`issue_comment`, `implementation_pr`,
`design_doc_pr`, `review_followup`) and only rely on explicit repository policy
or issue metadata you have actually read. When an issue could match both
`issue_comment` and `design_doc_pr`, choose `issue_comment` unless the issue
explicitly asks for a committed repo artifact such as an ADR, design document,
or documentation/design PR. This keeps discussion issues anchored to the issue
without collapsing explicit artifact requests into comment-only work.

Explicit canonical Work Mode fields and `mode/*` labels are artifact contracts
when they are known and do not conflict with the issue's requested deliverable.
Unknown explicit Work Mode values, unknown `mode/*` labels, multiple `mode/*`
labels, and contradictions between explicit mode metadata and requested
deliverable fail closed: post one focused clarification comment and return
`needs_reply`.

## Context Validity Gate

Before delegating, confirm all of these are true:

- A canonical work mode has been selected.
- Project context was genuinely read or cleanly confirmed absent.
- When the dispatched issue has one or more native `blockedBy` dependencies,
  dependency context was genuinely read, complete, and validated.

For `viber-gh-context`, acceptable Project states are
`project.context_status == "read"` or `"confirmed_absent"`. If the repository
or prompt provides a Project owner/number, run the script with those arguments.
If the script exits with `project_context_incomplete`, treat that as a blocker or
`needs_reply`; do not route based on empty Project fields. If Project fields are
empty while warnings are non-empty, treat the read as failed rather than as "no
Work Mode field".

Native dependency context is required whenever the prompt's `issue.blocked_by`
list is non-empty or `context_pack.dependency_context.required == true`. For
that case, `viber-gh-context` must be run successfully before production work.
Treat `dependency_context_incomplete`, `issue_read_failed`, malformed JSON,
missing `context_pack.dependency_context`, `dependency_context.status != "read"`,
`dependency_context.complete != true`, missing
`context_pack.direct_blocker_identities`, or a mismatch between the prompt
`issue.blocked_by` count and
`context_pack.dependency_context.direct_blocker_count` as a blocking incomplete
context outcome. Do not continue into implementation, design, review follow-up,
or issue-comment work until the dependency read can be trusted or a human
resolves the blocker.

After a successful dependency read, validate that
`context_pack.direct_blocker_identities.items` contains every direct blocker
identity, including blockers omitted from `direct_blocker_summaries.items` by
`--max-blockers` or other configured limits. If
`dependency_context.unresolved_blocker_count > 0`, stop before production work:
the run observed a non-terminal or unresolved blocker state, so the prerequisite
outcome is not available. Carry
`dependency_context.summary_omitted_count`,
`direct_blocker_identities.items[].summary_omitted_reason`, and all
`truncation_notices` into the handoff when they affect confidence or validation
scope.

When `signals.work_mode_needs_clarification` is true, use
`signals.work_mode_hint` only to choose the comment skill that asks the
clarifying question. Do not start a branch, create files, commit, push, or open a
PR. Copy `signals.work_mode_clarification_question` into one direct issue
comment, then finish with `work_mode: "issue_comment"` and
`outcome: "needs_reply"`.

## Normalized Handoff

Before delegating, summarize the issue into this shape for the next skill. Copy
Project fields from Viber's `issue.fields` prompt context or from the
`viber-gh-context` output; `project_fields` is a normalized handoff key, not a
Liquid variable name. If `viber-gh-context` was used, copy `context_pack`
through intact. Do not replace it with an unsupported prose summary.

```yaml
handoff_type: viber_work_spec
issue:
  identifier:
  url:
  title:
  status:
  type:
  native_type:
  labels:
  project_fields:
work_mode:
context:
  target_issue_context:
  follow_up_context:
    incoming_handoff_notes:
    linked_pr_summaries:
    direct_blocker_summaries:
    direct_blocker_pr_summaries:
  ordinary_comments:
  truncation_notices:
context_pack:
  schema: viber_work_start_context_pack/v1
  epistemic_status:
  source_url_fields:
  section_order:
  dependency_context:
  target_issue_context:
  direct_blocker_identities:
  incoming_handoff_notes:
  linked_pr_summaries:
  direct_blocker_summaries:
  direct_blocker_pr_summaries:
  ordinary_comments:
  truncation_notices:
problem:
acceptance_criteria:
scope:
out_of_scope:
validation_notes:
linked_pull_request:
handoff_states:
  validation:
  needs_reply:
  blocked:
policy:
  branch_allowed:
  pr_allowed:
  issue_comment_allowed:
  closing_keywords_allowed:
  project_status_updates_allowed:
  allowed_project_status_field:
  target_status:
```

Keep prompt sections visibly separated:

- `target_issue_context`: the issue Viber dispatched on, its body, Project
  fields, labels, status, and ordinary target-issue comments.
- `dependency_context`: whether native dependency context was required, read,
  complete, which terminal states were applied, and whether any blocker outcome
  remained unresolved.
- `direct_blocker_identities`: the complete direct `blockedBy` identity set from
  GitHub, independent of bounded summary limits.
- `follow_up_context`: incoming handoff notes, linked PR summaries, and direct
  blocker summaries that explain prior or prerequisite work.
- `truncation_notices`: every count, scan, body, or omission notice emitted by
  the helper.

Every item copied from `context_pack` keeps its `source_url` or `url`. Downstream
skills may summarize the pack for planning, but source URLs and truncation
notices stay available for citations and verification.

## Delegation

- For `issue_comment`, load and follow `viber-issue-comment`.
- For `implementation_pr`, load and follow `viber-implementation`.
- For `design_doc_pr`, load and follow `viber-design-doc`.
- For `review_followup`, load and follow `viber-review-followup`.

Each delegated skill must finish through `viber-work-wrap-up` and emit a
`viber-result` block.

## Rules

- Do not hard-code a start lifecycle policy. Discover from the prompt, workflow,
  or local repository contract whether moving the issue to `In Progress` is a
  required start gate for this run. If it is required and authorized, complete
  and verify that gate before production work; otherwise leave start lifecycle
  movement to the orchestrator or downstream workflow.
- Follow `WORKFLOW.md` for lifecycle writes. If the workflow authorizes or
  requires the agent to move final Project Status, capture the allowed field and
  target state in the handoff; otherwise do not move final Project Status
  directly.
- Do not close issues or use PR closing keywords unless `WORKFLOW.md` explicitly
  allows or requires that behavior. The default is no closing keywords.
- Do not post ceremonial "starting work" comments. Comment only when the work
  mode requires issue discussion, a blocker, or a meaningful handoff.
- If required context is contradictory, stop with an outcome of `needs_reply` or
  `blocked` rather than guessing.

## Gotchas

- Project reads must clear `GH_TOKEN` and `GITHUB_TOKEN`; otherwise a bot,
  expired, or workspace fallback token can shadow the human `gh` account that can
  read user-owned Projects.
- `gh project item-list --format json` commonly returns camelCase field keys
  such as `workMode` and `linkedPullRequests`; use snake_case or spaced labels
  only as compatibility fallbacks.
- Track the issue Project item, not the linked PR item. PR cards do not drive
  issue dispatch.
- A needs-changes Status outranks Type/RFC-style classification when selecting
  `review_followup`.
- Empty `fields` plus non-empty `warnings` means the Project read failed; it is
  not proof that the Work Mode field is absent.
