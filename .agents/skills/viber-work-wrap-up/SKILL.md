---
name: viber-work-wrap-up
description: Finish a Viber-managed run by validating work against issue acceptance criteria, posting meaningful handoff or blocker comments when allowed, and emitting the machine-readable viber-result block that Viber or a workflow wrapper can use to choose the next Project Status. Use this at the end of every Viber work-mode skill.
---

# Viber Work Wrap-Up

Use this skill at the end of every Viber-managed run. The goal is to create a
human-readable handoff and a machine-readable result while keeping lifecycle
ownership explicit. Discover final Project Status ownership from `WORKFLOW.md`,
repo conventions, and task intent; it may belong to Viber, a workflow wrapper,
the human operator, or the agent running the workflow.

## Inputs

Accept the normalized `viber_work_spec`, the selected work mode, and current
workspace state. Inspect git status, relevant diffs, issue comments, PR state,
and validation output needed to verify the run.

If the work spec includes `context_pack`, preserve its source URLs and
truncation notices in the handoff path. Do not cite assembled summaries as facts
unless the relevant `source_url`/`url` was inspected or the handoff clearly marks
the statement as a bounded summary. When dependency context exists, carry
dependency omissions or unresolved blocker-state warnings into the handoff when
they affected planning or validation confidence.

## Outcome Vocabulary

Choose exactly one outcome:

- `ready_for_validation`: the expected artifact exists and is ready for human
  validation or review.
- `needs_reply`: the next step is a human answer, product decision, or discussion
  reply.
- `blocked`: progress depends on unavailable access, hardware, secrets, external
  systems, or missing Project metadata.
- `failed`: the agent attempted the work but could not produce a usable handoff,
  usually due to validation failures or internal tool errors.
- `no_change_needed`: the issue appears already satisfied and needs human
  confirmation.

Viber, the workflow wrapper, or the repository workflow maps `work_mode +
outcome` to a Project Status. Follow `WORKFLOW.md`: some repositories require
the agent to perform the final tracked-issue Status write, while others require
only a `viber-result` handoff or a human-owned transition. Do not hardcode a
default final Status behavior.

## Ordered Procedure

1. Inspect the workspace, issue state, PR state, Project policy, relevant diffs,
   comments, and validation requirements. Determine the work mode, required
   artifacts, closing-keyword policy, and whether the final tracked-issue Status
   write is required, allowed, or forbidden for this run.
2. For PR modes, run `scripts/viber-pr-policy-check` and read its JSON output.
   The script exits 0 when it can inspect the PR, but `policy.status: failed`
   means the PR body or issue-link policy does not satisfy handoff.
3. Run the repository validation discovered from `WORKFLOW.md`, issue
   acceptance criteria, and repo conventions.
4. Draft the planned `viber-result` JSON in a temporary file. If the discovered
   policy requires a handoff comment or final Status write, represent the
   planned final state truthfully in the draft, but do not post the final
   handoff comment or move Status yet.
5. If the draft declares `next_agent_handoffs`, post those target issue comments
   first using the stable marker/template below, then update each entry's
   `posted_url`. These comments must happen before the final issue handoff
   comment and before any Project Status movement. If a required next-agent
   handoff cannot be posted, remove it because it is not actually needed, use
   `blocked` for external access/GitHub/target failures, or use `failed` for
   agent-side assembly, validation, or tooling failures.
6. Run `scripts/viber-result-validate --file ...` with the discovered policy
   flags. If it is invalid, fix the state or JSON and re-run until it is valid,
   or truthfully switch the result to `failed` with the validation errors.
7. Only after validation returns valid, post the final handoff or blocker comment
   when the workflow allows or requires it.
8. Only after the final comment gate, move the tracked issue Status when the
   discovered workflow policy allows or requires the agent to do so.
9. Emit exactly one final fenced `viber-result` block. Update any artifact URLs
   or policy fields changed by the gated mutations before emitting it.

## Checks

1. Compare the result against issue acceptance criteria and scope.
2. Verify required artifacts for the work mode:
   - `issue_comment`: issue comment present, no file changes, no PR.
   - `implementation_pr`: PR present when configured, relevant commits pushed,
     and `policy_checks.repository_artifact_authorized` is true.
   - `design_doc_pr`: design/doc changes present, PR present when configured.
   - `review_followup`: existing PR updated or replied to; no replacement PR.
3. Run or verify required validation commands from `WORKFLOW.md`.
4. Check forbidden actions:
   - no issue closure;
   - Project Status movement matches the discovered workflow policy;
   - no PR closing keywords unless explicitly allowed or required by
     `WORKFLOW.md`;
   - PR body policy passes when `scripts/viber-pr-policy-check` reports
     `policy.status: passed`;
   - no replacement branch/PR unless explicitly authorized.
5. Confirm any `next_agent_handoffs` comments were posted before the final issue
   handoff and before final Status movement.
6. Confirm any final handoff comment or final Status write happened only after
   `viber-result-validate` returned valid.

## Script Support

Use the bundled PR policy script for PR modes before claiming handoff readiness:

```sh
scripts/viber-pr-policy-check --repo OWNER/REPO --issue NUMBER --pr NUMBER
```

Use its JSON output to check for closing keywords, issue references, changed
files, head commit, PR check status, and the durable PR body contract. It exits
0 when it successfully produces a report, even if `policy.status` is `failed`,
checks are failing, or the PR is a draft. It fails only when it cannot inspect
the PR. The script clears `GH_TOKEN` and `GITHUB_TOKEN` for its internal
read-only `gh` calls so a bot, expired, or workspace token does not shadow the
user's keyring-backed account. If `policy.status` is `failed`, fix the PR body
or truthfully use a non-ready outcome before handoff. For this repository, a
passing policy requires non-placeholder `Motivation`, `Outcome`, `Validation`,
and `Continuity` sections plus the workflow-required closing keyword for the
tracked issue.

Before finalizing, write the planned `viber-result` JSON to a temporary file and
validate it:

```sh
scripts/viber-result-validate --file /path/to/result.json --enforce-mode-policy
```

Add `--work-mode`, `--require-pr`, `--forbid-pr`, or
`--require-issue-comment` when the workflow policy is stricter than the default
mode policy. Add `--allow-closing-keywords` when `WORKFLOW.md` explicitly
permits or requires PR closing keywords. Direct Status movement is neutral by
default: use `--forbid-status-move` only when the discovered policy forbids an
agent-owned final Status write. `--allow-status-move` is retained as a
compatibility no-op for older workflows and is not required. Do not emit a final
`viber-result` block until validation passes or until you truthfully report
`failed` with the validation errors.

When `next_agent_handoffs` is present, the validator enforces the initial
same-repository issue-comment scope: `target.kind` must be `issue`,
`target.repository` must match the current issue repository when provided,
`intent` must be one of the supported enum values, `source` may contain only
`issue`, `pull_request`, and `commit`, `post_to` must be `issue_comment`, and
`ready_for_validation` requires each entry to include a posted issue-comment
URL in `posted_url`.

Exit codes:

- `viber-pr-policy-check`: `0` means JSON report produced; inspect
  `policy.status` for pass/fail. `2` means usage, environment, or GitHub
  inspection failure.
- `viber-result-validate`: `0` means valid; `1` means the result is invalid and
  the JSON report explains why; `2` means usage or input failure.

## Gotchas

- Emit exactly one fenced `viber-result` block. Two blocks cause the whole run to
  be rejected by the result consumer.
- Pass `--file` to `viber-result-validate`; reading from stdin is only for pipes,
  and the script exits instead of hanging when stdin is a TTY.
- `viber-pr-policy-check` reports policy pass/fail in JSON. It does not use the
  process exit code for body-policy failures, so read `policy.status` instead
  of relying only on the exit code.
- Run read-only GitHub checks with `GH_TOKEN` and `GITHUB_TOKEN` cleared. The
  bundled PR policy script handles this internally for its `gh` calls.
- Use an iterative validation loop: run the validator, fix invalid state or JSON,
  and re-run until valid or until the truthful outcome is `failed`.

## Handoff Comment Guidance

For successful handoff, include:

- What changed or what was answered.
- Artifact links, such as PR URL or issue comment URL.
- Validation performed.
- Remaining risk or open questions.
- Relevant source URLs and truncation notices from `context_pack` when they
  influenced the work or limit confidence.

For blockers, include:

- What blocked progress.
- What was already tried.
- What human action is needed.
- Current branch/PR state when relevant.

Avoid ceremonial comments that only say the run started or stopped.

## Next-Agent Handoff Guidance

Use `next_agent_handoffs` only for durable context that a later Viber-dispatched
agent needs on another same-repository issue. The daemon does not post these
comments; the agent must post them with `gh-bot issue comment` before the final
issue handoff comment and before Project Status movement, then copy the created
comment URL into `posted_url`. This is the only cross-issue write exception:
the agent may append the marked comment, but must not edit the target issue's
Status, labels, assignees, milestones, title, body, parent/sub-issue links,
dependencies, Project fields, or any other issue field.

Initial scope:

- `target.kind`: `issue`
- `target.repository`: current repository, or omitted to default to it
- `intent`: `shared_context`, `follow_up`, `blocker_unblocked`, `needs_reply`,
  or `risk`
- `source`: one or more of `issue`, `pull_request`, or `commit`
- `post_to`: `issue_comment`
- comment marker: exactly `<!-- viber:next-agent-handoff -->`

Unposted required handoffs cannot be carried as private notes. Remove the entry
if it is not required; otherwise report `blocked` when posting is prevented by
external access, credential, GitHub, or target availability failure, and report
`failed` when the agent cannot assemble or validate a usable handoff comment.

Issue comment template:

```md
<!-- viber:next-agent-handoff -->

Source: SOURCE_URL_OR_COMMIT
Intent: shared_context

Summary: One sentence describing what the next agent needs to know.

Context:
Bounded Markdown with the evidence, decision, caveat, or risk.

Action requested:
Optional next action, or "None".
```

## Required Final Block

End the final response with exactly one fenced `viber-result` JSON block. A
daemon, workflow wrapper, or other configured result consumer should parse this
single block for deterministic status handoff.

```viber-result
{
  "schema_version": "viber-result/v1",
  "work_mode": "implementation_pr",
  "outcome": "ready_for_validation",
  "issue": {
    "identifier": "owner/repo#123",
    "url": "https://github.com/owner/repo/issues/123"
  },
  "summary": "Short human-readable result.",
  "artifacts": {
    "pull_request": "https://github.com/owner/repo/pull/456",
    "issue_comment": "",
    "branch": "123-short-title",
    "head_commit": ""
  },
  "validation": [
    {
      "command": "go test ./...",
      "result": "passed",
      "notes": ""
    }
  ],
  "blockers": [],
  "next_agent_handoffs": [],
  "policy_checks": {
    "created_pr": true,
    "created_replacement_pr": false,
    "changed_files": true,
    "repository_artifact_authorized": true,
    "used_closing_keywords": false,
    "closed_issue": false,
    "moved_project_status": false
  }
}
```

Use empty strings or empty arrays for fields that do not apply. Keep
`schema_version` stable so future Viber parsers can reject unknown shapes safely.

## Result Rules

- The JSON must be valid.
- `work_mode` must be one of `issue_comment`, `implementation_pr`,
  `design_doc_pr`, or `review_followup`.
- `outcome` must be one of the outcome tokens above.
- If `outcome` is `blocked`, include at least one blocker.
- If `outcome` is `ready_for_validation` for a PR mode, include the PR URL unless
  the workflow explicitly says PRs are not used.
- `next_agent_handoffs` is optional. When present, it must be an array of
  same-repository issue-comment handoff entries using the initial scope above.
- If a `ready_for_validation` result declares a `next_agent_handoffs` entry, that
  entry must include a non-empty `posted_url` for the posted target issue
  comment. Unposted required handoffs must not pass a clean ready handoff.
- If any forbidden action occurred, report it truthfully in `policy_checks` and
  use `failed` unless the workflow explicitly permits that action.
