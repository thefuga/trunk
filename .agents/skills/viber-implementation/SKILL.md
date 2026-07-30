---
name: viber-implementation
description: Implement a Viber-managed GitHub issue through code or documentation changes that are intended for pull request handoff. Use this only for implementation_pr work mode after Viber has claimed the issue and viber-work-start has normalized the issue context.
---

# Viber Implementation

Use this skill for Viber-managed implementation work where a branch, commits, and
a pull request are expected artifacts.

## Inputs

Accept a `viber_work_spec` from `viber-work-start` or equivalent issue context.
The issue should have implementation acceptance criteria or be clear enough to
investigate and implement safely.

The work spec must authorize a repository artifact. Authorization may come from
an explicit `implementation_pr` Work Mode, an agreeing `mode/implementation-pr`
label, or a clear issue deliverable such as a reusable executable probe,
prototype, production code change, docs change, or testable repository change.
Exploratory issues that only ask for findings, research, comparison, or a
recommendation without a repo artifact belong in `issue_comment`, usually with
`outcome: "needs_reply"` when the artifact contract is unclear.

When the work spec includes `context_pack`, keep it in the working context. Treat
its `target_issue_context` as the dispatched issue, its `dependency_context` and
`direct_blocker_identities` as the native dependency gate result, and its
`incoming_handoff_notes`, `linked_pr_summaries`, `direct_blocker_summaries`, and
`direct_blocker_pr_summaries` as follow-up context. When dependencies exist,
the blocker identity and summary sections are required prerequisite context for
planning the implementation. These are bounded summaries, not unsupported truth:
preserve `source_url`/`url` values, inspect the linked source before relying on a
claim, and carry any `truncation_notices` or dependency omissions into the
handoff when they affect confidence or validation scope.

## Procedure

1. Read `WORKFLOW.md`, `AGENTS.md`, and the repo docs required by the workflow.
2. Re-read the issue acceptance criteria, scope, and out-of-scope sections.
3. Check the workspace git state before editing.
4. Use the configured issue branch when the workflow provides one.
5. For non-trivial issues, write a short plan before editing. Include the
   files or areas you expect to touch, the validation you expect to run, and
   the discovered PR/status policy you will follow.
6. Confirm the normalized work spec authorizes a repository artifact. If it does
   not, stop before editing files and return to `viber-issue-comment` for one
   clarification comment with outcome `needs_reply`.
7. Make the smallest correct change.
8. Update docs when behavior, configuration, protocol, or operator workflow
   changes.
9. Follow the validation loop below: targeted checks first, broader checks
   after the focused checks are green.
10. Draft the PR title/body and status handoff, run the workflow policy check,
   then open or update one pull request only when PR handoff is permitted.
11. Finish through `viber-work-wrap-up` with work mode `implementation_pr`.

## Implementation Checklist

- Read repository instructions and conventions before editing: workflow files,
  agent instructions, contributing/style docs, PR templates, package manifests,
  build files, and CI definitions that apply to the task.
- Create or reuse the workflow-designated issue branch and preserve unrelated
  local changes.
- For non-trivial work, plan before editing and keep the plan scoped to the
  acceptance criteria.
- Make the smallest correct code or documentation change; do not bundle
  adjacent cleanup.
- Discover and run format, lint, test, and build validation from the repository,
  moving from narrow checks to broad checks.
- Fix failures in scope and rerun the failing command before proceeding to
  broader validation.
- Commit, push, draft the PR body, and choose closing/status behavior according
  to the discovered workflow and repository policy.
- Run the PR policy check before opening or updating the PR, then wrap up with
  the required `viber-result`.

## Validation Loop

Discover validation commands from the consumer repository before running them.
Prefer explicit project entrypoints over language assumptions:

- Repository guidance: `WORKFLOW.md`, `AGENTS.md`, `README`, `CONTRIBUTING`,
  `docs/`, PR templates, CI workflows, and checked-in scripts.
- Build tool entrypoints: `Makefile` targets, `package.json` scripts, task
  runners, language-specific project files, formatter/linter config, and test
  config.
- Language defaults only when the repository has no clearer convention.

Build a validation plan that covers the commands the project exposes for:

- formatting or generated-file checks,
- lint/static analysis,
- focused tests for the changed area,
- broader test suites, and
- build/typecheck/package verification when available.

Run narrow to broad:

1. Start with the fastest command that exercises the changed file, package,
   module, fixture, or behavior.
2. Run the repository's format and lint commands before relying on broad tests.
3. Expand to package/module-level tests, then repository-level tests/builds
   when feasible.
4. If a command fails, inspect the diagnostic, make the smallest in-scope fix,
   rerun the failing command, and continue this fix-rerun loop until green or
   blocked.
5. Record every command run, its result, and any skipped check with the reason
   in the handoff.

Do not carry commands from one repository into another. For example, a
repository with package scripts should use those scripts; a repository with
Makefile targets should use the relevant targets; a repository with neither
should fall back to the applicable language or tool defaults.

## Gotchas

- Conventions are discovered, not assumed. Check the repository's own style,
  contributing, architecture, design, workflow, and PR docs before relying on
  generic skill guidance.
- Local agent instructions can be layered. Follow the closest applicable
  instructions for the files you edit, and let explicit workflow policy override
  generic defaults.
- Validation names are not portable. Similar projects may expose checks through
  different task runners, package scripts, CI jobs, or language tools.
- Closing keywords can trigger automation. Discover whether the repository wants
  auto-closing PR bodies, non-closing issue references, or no issue reference
  until human review.
- Status transitions are repository policy, not skill policy. Move only the
  allowed tracked item and field when the workflow explicitly authorizes it.
- Generated files, lockfiles, snapshots, and vendored docs may have local rules;
  inspect the nearby docs and build scripts before updating them.

## PR Rules

- Discover the issue-linking and status policy from `WORKFLOW.md`, repo
  conventions, PR templates, and the task intent before drafting the PR body.
- If discovered policy requires auto-close for completed implementation work,
  use the repository's accepted closing keyword form.
- If discovered policy forbids auto-close or the task intent is exploratory,
  partial, blocked, or not meant to complete the issue, use the repository's
  accepted non-closing reference form.
- Generic fallback when policy is silent: use a non-closing issue reference and
  do not move final status unless the workflow explicitly authorizes it; call
  out the uncertainty in the handoff rather than inventing automation behavior.
- Draft the PR title/body and intended status transition before opening the PR,
  run the workflow's policy check when available, then execute the PR action
  only if the draft matches the discovered policy.
- Do not close the issue manually.
- Do not create replacement PRs or branches when an existing issue PR should be
  updated.

## Blockers

Use outcome `blocked` when required access, hardware, secrets, external systems,
or Project fields are missing. Leave a useful blocker comment if the workflow
allows issue comments, then report the blocker in `viber-result`.

## Rules

- Do not move final Project Status directly unless `WORKFLOW.md` explicitly
  authorizes it. If authorized, move only the tracked issue's allowed Status
  field to the workflow's target state.
- Do not broaden scope to adjacent cleanup or refactors.
- Preserve unrelated local changes.
- If acceptance criteria are ambiguous enough that implementation would be a
  product decision, stop with `needs_reply` instead of inventing behavior.
- If no repository artifact is authorized, do not manufacture one. In
  particular, do not create a findings, research, spike, or investigation
  Markdown file merely to satisfy this mode's PR completion contract.
- A `ready_for_validation` result for this mode must set
  `policy_checks.repository_artifact_authorized` to `true`; otherwise use
  `needs_reply` or the appropriate non-ready outcome.

## Wrap-Up

Load `viber-work-wrap-up` and emit a `viber-result` block. Required policy
checks for this mode:

- `pull_request` is present when PR handoff is configured.
- `used_closing_keywords` matches the workflow policy.
- `moved_project_status` matches the workflow policy.

For PR handoff, use `viber-work-wrap-up`'s `viber-pr-policy-check` script before
claiming `ready_for_validation`.
