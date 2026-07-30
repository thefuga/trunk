---
name: viber-setup
description: Set up a repository to run Viber autonomous issue workflows. Use this whenever the user asks to bootstrap, initialize, configure, or prepare a repo for Viber, including creating or refining WORKFLOW.md, AGENTS.md, Viber runtime config examples, GitHub Project v2 repository links, board views, Status options, custom fields, labels, or setup checks. This skill should trigger even if the user only says "set this repo up for Viber" or "make this repo ready to vibe".
---

# Viber Setup

Use this skill to prepare the current repository for Viber. Viber setup is
agent-first and CLI-backed: you inspect the repo and make policy choices with
the user, while `viber init` provides deterministic checks and setup helpers.

## Operating rules

- Start by running `viber init check --json` from the repository root if the
  `viber` binary is available.
- Use `viber init plan --json` and `viber init apply --plan PATH` for
  deterministic starter files instead of hand-writing boilerplate from scratch.
- Use `viber init plan --codex-hooks --json` only when the operator explicitly
  opts into optional repo-local Codex hook scaffolding. Review the plan before
  applying it; the CLI must not overwrite existing Codex hook config silently.
- If `viber` is missing, tell the user to install it with
  `go install github.com/thefuga/viber/cmd/viber@latest`, then continue after it
  is available.
- Do not install or remove this skill. Skill lifecycle is owned by the user's
  skill manager or agent.
- Do not commit, push, open PRs, or mutate unrelated GitHub objects unless the
  user explicitly asks.
- Ask before overwriting an existing `WORKFLOW.md`, `AGENTS.md`,
  `.github/ISSUE_TEMPLATE` file, or committed setup file. Prefer editing in place
  when the file already exists.
- Confirm before creating or changing a GitHub Project, Project repository links,
  default repository, Project views, Project fields, Status options, or labels.
- Keep `.viber/config.yaml` machine-local. If you create or recommend it, ensure
  it is ignored. Prefer committed examples such as `.viber/config.example.yaml`.
- Do not silently leave setup dependencies as placeholders. If a dependency is
  missing or ambiguous, ask the user to provide it, authorize you to create or
  configure it, or explicitly defer it. A deferred dependency means setup is not
  ready; report it as an incomplete checklist item.
- Before claiming the repo is ready, confirm every dependency gate below is
  either satisfied or explicitly deferred by the user.

## Dependency gates

After discovery, batch the unresolved setup dependencies into direct questions.
Do not proceed as if a dependency is solved just because local starter files can
be generated.

Required gates:

- Repository identity: a git repo with a canonical GitHub remote or a user-provided
  clone URL for `WORKFLOW.md` `hooks.after_create`.
- GitHub access: `gh` installed and authenticated if Project or repo access needs
  to be inspected or changed.
- GitHub Project choice: use an existing Project, create a new Project, or defer
  Project setup.
- Project owner and number: owner login or `@me`, owner type, and Project number
  for existing Projects.
- Project repository association: link/list the Project on this repository, set
  this repository as the Project default repository when appropriate, or defer.
- Project board view: create or verify a standard board view grouped by `Status`,
  or record a manual/deferred setup item if automation is unavailable.
- Status flow: `review-gated` (`Backlog`, `Todo`, `In Progress`, `Code Review`,
  `Needs Changes`, `Done`) or `autonomous` (`Backlog`, `Todo`, `In Progress`,
  `Done`), plus exact Status option names. Both flows include `Backlog` as an
  initial non-active state kept out of `active_states` by default; ask only if
  the operator wants to drop it.
- Workflow modes: ask whether this repo wants implementation PR runs,
  RFC/design/comment-only runs, exploratory Spike/POC/Investigation routing,
  code-review/needs-changes follow-up runs, or a subset, and how each mode is
  identified from issue type, labels, fields, or status.
- Work-mode skills: ask whether the workflow should reference the reusable
  Viber skills (`viber-work-start`, `viber-issue-comment`,
  `viber-implementation`, `viber-design-doc`, `viber-review-followup`, and
  `viber-work-wrap-up`) and require a final `viber-result` block for handoff.
- Workflow metadata: list existing labels and Project fields, propose useful
  additions, and ask whether work type, area, size, risk, or similar metadata
  should be represented in `WORKFLOW.md` prompt conventions.
- Scheduling constraints: ask whether any labels, fields, assignees, or other
  metadata represent hard pre-dispatch runner requirements such as runner OS,
  harness, hardware, or machine locality. Keep product area separate from runner
  capability: a `Macropad` area can still be `platform:linux` when simulator or
  off-device validation is enough.
- Harness choice: Codex for now unless the user chooses another supported harness.
- Optional Codex hook readiness: ask whether the operator wants setup-owned
  Codex native hook support for `PreToolUse` and `Stop`. Explain that hooks are
  Codex-owned local commands, require Codex project/config trust and non-managed
  hook review, and are never injected per issue, attempt, turn, session, or
  through `WORKFLOW.md`.
- Validation commands: exact test/lint/build commands, or an explicit decision
  that none are available yet.
- Local runtime config: whether `.viber/config.yaml` will be created locally now
  or deferred, and what values the operator must fill in.

If multiple gates are missing, ask in one concise batch with defaults and clear
options. Examples:

- "No GitHub remote is configured. Should I add one, use a clone URL only in
  `WORKFLOW.md`, or defer remote setup?"
- "Which GitHub Project should Viber use: existing owner/number, create a new
  Project, or defer Project setup?"
- "Should I link this Project to the repository and set this repo as the default
  repository for Project-created issues?"
- "Should I create a `Viber Board` view grouped by Status, leave view setup
  manual, or defer it?"
- "Which Status flow should this repo use: `review-gated` (`Backlog`, `Todo`,
  `In Progress`, `Code Review`, `Needs Changes`, `Done`) or `autonomous`
  (`Backlog`, `Todo`, `In Progress`, `Done`, dropping the human-review stages)?"
- "Both flows start with a `Backlog` column kept out of `active_states`, so new
  or unrefined issues are parked off the daemon until a human promotes them, and
  it can optionally be fed by GitHub's built-in auto-add workflow. Should I drop
  `Backlog` for this repo instead?"
- "Which work modes should Viber agents handle here: implementation PRs,
  RFC/design/comment-only issues, exploratory Spike/POC/Investigation routing,
  code review/needs-changes follow-up, or a mix?"
- "Should the workflow delegate those modes to the reusable Viber work-mode
  skills and require a final `viber-result` handoff block?"
- "How should the workflow identify RFC/design/comment-only and exploratory
  Spike/POC/Investigation issues: native issue type, labels such as
  `kind/design`, `type/rfc`, or `type/spike`, a Project field, or issue text
  only?"
- "I found these existing labels: ... Should I add standard Viber labels, define
  custom labels, or leave labels unchanged?"
- "Should work type and labels only affect the workflow prompt, or do any of them
  represent hard runner requirements Viber must enforce before launch?"

## Setup phases

### 1. Discover state

Run:

```sh
viber init check --json
```

Use the JSON report to identify blockers and missing setup. Then inspect the
repo directly for:

- language and repo type;
- package manager and test commands;
- CI workflows;
- existing agent guidance files;
- branch, commit, PR, and release conventions;
- existing GitHub labels, Project references, Project fields, saved Project views,
  and whether the Project is linked to the repository.

If `git`, `gh`, repo remote, or `gh auth` are blocked, stop setup of external
GitHub resources and ask how the user wants to resolve or defer the blocker. You
may still draft local files only after the user explicitly agrees to defer that
external dependency. Do not put a guessed remote or Project into `WORKFLOW.md`.

When local file setup is needed, generate and review a deterministic plan:

```sh
viber init plan --json
```

When the operator opts into optional repo-local Codex hook readiness, generate a
separate reviewed plan:

```sh
viber init plan --codex-hooks --json
```

Prefer repo-local `.codex/hooks.json` for Viber-generated scaffolding. Do not
generate both `.codex/hooks.json` and inline `[hooks]` in `.codex/config.toml` in
the same Codex config layer. If repo-local Codex hook config already exists,
inspect and refine it deliberately instead of overwriting or adding a competing
representation.

If the plan only creates missing starter files or appends the Viber machine-local
runtime artifact ignore rules (`.viber/config.yaml`, `.viber/*.sock`,
`.viber/*.jsonl`, `/viber-workspaces/`), ask for confirmation and apply it with:

```sh
viber init apply --plan PATH
```

Do not apply a plan blindly when it would conflict with files the user already
owns. The CLI refuses to overwrite, but you should still explain conflicts and
edit existing files deliberately.

### 2. Ask for workflow choices

Ask for every dependency gate that is not already satisfied by discovery. Do not
skip GitHub Project setup just because the repository has no remote yet; ask
whether to set up the remote first, use a provided URL, or defer Project setup.

For non-gate policy choices, ask only for details that are not already obvious:

- Use an existing GitHub Project or create a new one?
- Is the Project user-owned or organization-owned?
- Should the Project be linked/listed on this repository and should this
  repository be the default repository for Project-created issues?
- Should setup create a standard `Viber Board` view grouped by Status, use an
  existing board view, or leave the board view as a manual follow-up?
- Use the `review-gated` flow (`Backlog`, `Todo`, `In Progress`, `Code Review`,
  `Needs Changes`, `Done`) or the `autonomous` flow (`Backlog`, `Todo`,
  `In Progress`, `Done`, dropping the human-review stages)?
- Both flows include `Backlog` as an initial non-active state kept out of
  `active_states`, so new or unrefined issues stay off the daemon until
  promoted, optionally fed by GitHub's built-in auto-add workflow. Should this
  repo drop `Backlog` instead?
- Which workflow modes should the repo support: implementation PRs,
  RFC/design/comment-only issues, exploratory Spike/POC/Investigation routing,
  code review/needs-changes follow-up, or all of them?
- Should the workflow delegate work-mode procedure to reusable Viber skills and
  require `viber-work-wrap-up` to emit a final `viber-result` block?
- How should the prompt identify RFC/design/comment-only and exploratory
  Spike/POC/Investigation issues: native issue type, labels such as
  `kind/design`, `type/rfc`, or `type/spike`, a Project field such as
  `Work Type` or `Work Mode`, or issue text only?
- Add standard Viber labels, propose repo-specific custom labels, or leave labels
  unchanged?
- Add optional Project fields such as `Priority`, `Size`, `Area`, `Work Type`,
  `Work Mode`, `Harness`, `Runner Platform`, or `Risk`, define custom fields, or
  keep only `Status`?
- Should work type be represented by native issue types, labels such as
  `kind/bug`, a Project field such as `Work Type`, or prompt text only?
- Do any labels, fields, assignees, or other metadata represent scheduler
  constraints such as runner OS, `Harness`, hardware, or machine locality?
- Should Viber use Codex as the harness for now?
- Should this repo opt into setup-owned Codex `PreToolUse` and `Stop` hook
  readiness, or rely on the portable floor of prompt instructions, skills, and
  scripts for now?
- Are there repo-specific validation commands agents must run before handoff?
- Should I add or refine issue templates so planned issues include problem,
  motivation, acceptance, validation, and review guidance, while preserving this
  repo's preferred layout?

If the user chooses to defer repository remote, Project setup, Status option
setup, Project repository link, board view, label decisions, custom fields, issue
template policy required by the operator, or runtime config, record that deferral
in the final report and do not state that Viber is fully ready.

Recommend the `review-gated` flow for repositories where agents should open PRs
for human review or handle `Needs Changes` follow-up instead of closing issues
directly. In that flow, include merge-conflict checks in the Needs Changes
feedback ledger so a conflicted PR is actionable even when no reviewer comments
or failing checks exist. Recommend the `autonomous` flow for repositories where
agents close issues directly without a human review gate; it drops `Code
Review` and `Needs Changes` from the board and from `WORKFLOW.md`'s `review`
block. Recommend explicit RFC/design/comment-only prompt rules and exploratory
artifact-contract rules for repos where issues may be discussion or research
artifacts rather than implementation tasks.

Both flows default to including `Backlog` as an initial non-active state for
repositories that author issues before they are ready to dispatch — for example
repos that use operator planning (`viber-delivery-planning`) or GitHub's
built-in auto-add workflow to drop new issues onto the board. Keep `Backlog`
out of `active_states` so the daemon ignores parked issues until a human
promotes them into an active state. Only drop `Backlog` if the operator
explicitly says the repo has no pre-dispatch triage step.

### 3. Prepare `WORKFLOW.md`

`WORKFLOW.md` is the autonomous-run contract. If `viber init apply` created a
starter file, refine it instead of replacing it wholesale. It should include YAML
front matter with at least:

```yaml
---
tracker:
  kind: github
  project_owner: OWNER
  project_owner_type: user
  project_number: 1
  active_states:
    - Todo
    - In Progress
  terminal_states:
    - Done
review:
  code_review:
    target_state: Code Review
  needs_changes:
    target_state: Needs Changes
hooks:
  after_create: |
    git clone REPO_URL .
  before_run: |
    git fetch --prune origin
harness:
  kind: codex
---
```

Adjust states to match the user's chosen flow. Omit the `review` block for the
`autonomous` flow.

If the repo adopts an initial non-active state (such as `Backlog`), do **not**
add it to `active_states`: the daemon only claims issues whose Status is active,
so leaving the column out of `active_states` is exactly what parks new or
unrefined issues off the daemon until a human promotes them. The state name is a
repo choice and is never hard-coded in Viber; it is configured the same way as
`Todo`. The Project Status field still needs the option to exist (see Prepare
GitHub Project), and the column can optionally be fed by GitHub's built-in
auto-add workflow.

Do not leave `OWNER`, `PROJECT_NUMBER`, `REPO_URL`, or similar placeholders in
final `WORKFLOW.md` unless the user explicitly deferred that dependency. If a
dependency is deferred, add a short TODO in `WORKFLOW.md` and repeat it in the
final checklist.

The prompt body should cover:

- issue identity, title, URL, state, and description using Liquid variables;
- issue type, labels, and relevant Project fields when the repo uses them to
  select implementation, RFC/design/comment-only, exploratory artifact-contract,
  or review-feedback modes;
- repository-specific instructions and docs to read first;
- exact git branch, commit, rebase, push, and PR expectations;
- work-mode rules for implementation PRs, RFC/design/comment-only issue comments,
  exploratory Spike/POC/Investigation artifact contracts, and
  code-review/needs-changes follow-up when enabled, including merge-conflict
  detection and resolution before returning to review;
- optional work-mode skill delegation and the final `viber-result` handoff
  contract when the repo uses reusable Viber skills;
- validation commands;
- GitHub Project Status transition rules;
- authority boundaries, scoped to the repo's current dispatch model. If the
  workflow forbids mutating sub-issues, parents, or dependencies, say that this
  is the current single-issue model and cite the later parent-owns-children
  deferral or decision if one exists, so the prohibition is not mistaken for a
  permanent rule. If the repo already supports parent-owns-children, spell out
  the exact child scope and the allowed issue, PR, and Project mutations instead
  of leaving both models implicit;
- blocker handling;
- completion gates.

Use the current repository's root `WORKFLOW.md` from the Viber project as the
reference style when available, but keep the target repo's workflow shorter when
it does not need bot identity or review-feedback complexity.

### 4. Prepare `AGENTS.md`

`AGENTS.md` is human-directed and ad hoc agent guidance. If `viber init apply`
created a starter file, refine it instead of replacing it wholesale. It should be
short and point to deeper docs. Include:

- project purpose;
- read order;
- build, test, lint, and formatting commands;
- architecture or dependency boundaries;
- documentation update rules;
- git safety rules for human-directed work;
- where Viber-specific autonomous workflow lives (`WORKFLOW.md`).

Do not duplicate the full autonomous run procedure from `WORKFLOW.md` unless the
repo has no better place for it.

### 5. Recommend issue templates

Issue templates are repository layout policy, not a Viber runtime requirement.
This is a skill/tooling convention, not runtime behavior: do not create or update
runtime design docs for this issue-body convention. Viber recommends templates
that preserve planning substance, but the repo owns section names, order, and
shape. Existing templates should be edited in place only after confirmation; do
not replace a repo's template just because it does not match Viber's preferred
headings.

When the repo has no template or wants a Viber-friendly default, recommend a
generic deliverable template with optional mapping comments. The comments let
`viber-delivery-planning` map canonical content into renamed or reordered
sections; if the repo removes them, the planner falls back to semantic mapping.

Recommended generic template:

```markdown
## Summary
<!-- viber:summary desired_outcome -->

## Problem and motivation
<!-- viber:problem motivation user_impact -->

## Scope
<!-- viber:scope non_goals -->

## Constraints and decisions
<!-- viber:constraints decisions risks assumptions -->

## Acceptance criteria
<!-- viber:acceptance -->
- [ ] <Observable done-check>

## Validation
<!-- viber:validation how_to_test evidence -->

## Review focus
<!-- viber:review_focus what_to_look_for -->

## Implementation notes
<!-- viber:implementation_notes codebase_findings relevant_files -->

## Process checklist
<!-- viber:process_checklist -->
- [ ] Research/confirm approach
- [ ] Implement
- [ ] Validate
- [ ] Open PR / hand off
```

For repos that want multiple templates, suggest small variants rather than one
overloaded form:

- Bug/fix template: include observed behavior, expected behavior, impact,
  evidence/suspected cause, validation, and review focus.
- Docs template: include reader problem, current confusion, required points to
  cover, validation, and review focus.
- Decision/RFC template: include question, context, options considered,
  recommendation, constraints, and expected follow-up.

Do not make template adoption a setup blocker unless the operator explicitly says
issue templates are required repo policy. If deferred, record that Viber planning
will use fallback issue bodies or semantic mapping until templates exist.

### 6. Prepare GitHub Project

Use Viber's read-only project helper before considering raw `gh` commands:

```sh
viber init project view --owner OWNER --number PROJECT_NUMBER --json
viber init project check --owner OWNER --number PROJECT_NUMBER --states "Backlog,Todo,In Progress,Code Review,Needs Changes,Done" --json
```

Use the `review-gated` states above, or `"Backlog,Todo,In Progress,Done"` for
the `autonomous` flow.

Use `@me` as `OWNER` for user-owned Projects when the GitHub CLI expects the
current authenticated user. Use the organization login for organization-owned
Projects.

If the check reports missing Status options or blocked access, explain the
blocker and ask before making any GitHub mutation. Until `viber init project
ensure` exists, use raw `gh project` commands carefully and only after user
confirmation.

Minimum Project requirements:

- GitHub Projects v2 project exists.
- Project is linked/listed on the repository when the Project owner and repo owner
  allow it.
- This repository is the Project default repository when the user wants new
  Project-created issues to land in this repo.
- Status field exists and is single-select.
- Status options include all configured active, review, and terminal states, plus
  any initial non-active state (such as `Backlog`) the repo adopts even though it
  is deliberately absent from `active_states`.
- A standard board view exists or is explicitly deferred/manual. Prefer a view
  named `Viber Board`, layout `Board`, column field `Status`, with columns for
  the configured Status options.
- Issues that Viber should work on are added to the Project.

When creating a new Project, do not stop after creating the Project and Status
options. Also attempt, or explicitly document as manual/deferred, the repository
link/default repository and `Viber Board` view. If `gh` or GraphQL cannot create
or verify saved Project views, say so plainly and give exact manual follow-up
steps instead of claiming Project setup is complete.

Fallback raw checks:

```sh
gh project view PROJECT_NUMBER --owner OWNER --format json
gh project field-list PROJECT_NUMBER --owner OWNER --format json
```

### 7. Prepare workflow metadata and optional fields

Labels and custom Project fields are optional Viber workflow metadata. They are
not required for the daemon to run, so never create them silently. Treat them as
prompt and operator semantics unless the user identifies a hard scheduling
constraint.

List existing labels when a GitHub repo is available:

```sh
gh label list --repo OWNER/REPO --json name,description,color
```

Show the user the existing labels and propose a small set. Standard Viber labels:

- `viber`: work tracked or managed by Viber;
- `agent`: work prepared or changed by an autonomous agent;
- `blocked`: progress is blocked by an external dependency;
- `needs-human`: a human decision is required;
- `needs-review`: review-gated agent work needs human review, only when the repo
  uses the `review-gated` flow;
- `kind/design` or `type/rfc`: RFC/design/comment-only work, only when the repo
  wants agents to handle discussion issues without necessarily opening PRs.
- `kind/spike`, `type/spike`, `kind/poc`, `type/poc`,
  `kind/investigation`, or `type/investigation`: exploratory work, only when the
  repo wants agents to fail closed unless the issue clearly asks for findings in
  a comment, a reusable executable probe/prototype PR, or an ADR/design record
  PR.

Also suggest repo-specific labels if the repo structure makes them obvious, such
as `area/backend`, `area/frontend`, `area/docs`, `kind/bug`, `kind/feature`, or
`kind/documentation`.
Ask whether to create standard labels, custom labels, both, or no labels. Create
or edit labels only after confirmation. Do not delete or rename existing labels
unless the user explicitly asks.

Ask whether work type and workflow mode should be represented by native GitHub
issue types, labels such as `kind/bug`, `kind/design`, or `type/rfc`, Project
fields such as `Work Type` or `Work Mode`, or only by prompt conventions in
`WORKFLOW.md`. Work type does not affect Viber dispatch by itself; capture the
chosen convention in the workflow prompt if it changes agent instructions,
validation, comments, PR expectations, or completion gates.

For Project fields, `Status` is required; all other fields are optional. Show the
existing fields from `viber init project view --json` or `gh project field-list`.
Ask whether to keep only `Status`, add recommended fields (`Priority`, `Size`,
`Area`, `Work Type`, `Work Mode`, `Harness`, `Runner Platform`, `Risk`), or define
custom fields. Create fields only after confirmation, and explain that extra
fields are for planning, prompt behavior, or scheduling constraints rather than
required Viber runtime correctness.

When labels, fields, assignees, or other metadata imply hard pre-dispatch
requirements, distinguish them from ordinary workflow metadata. Examples include
`platform/linux`, `platform/windows`, `Harness=codex`, `Hardware=gpu`, or a
machine-locality requirement. Do not map product areas such as `API`, `UI`, or
`Macropad` into scheduler capabilities unless those values truly mean local
runner requirements.

### 8. Prepare optional Codex hooks

Codex native hooks are optional setup-owned tooling, not Viber runtime config.
Use them only as a determinism upgrade above the portable floor. Missing,
disabled, unsupported, stale, untrusted, or review-needed hooks are warnings or
recommendations, not Viber dispatch blockers.

Use Codex-native locations:

- repo-local `.codex/hooks.json` when the operator opts into Viber scaffolding;
- repo-local `.codex/config.toml` when the repo already owns inline hooks;
- user-level `~/.codex/hooks.json` or `~/.codex/config.toml` when the operator
  owns the behavior globally;
- managed `requirements.toml` only as validate-and-report input unless the
  enterprise administrator owns the policy change.

Codex requires non-managed command hooks to be reviewed and trusted against the
current hook definition. A changed hook may need review again. Tell the operator
to use Codex's `/hooks` UI for trust review when Codex reports review-needed
hooks. Do not scaffold `--dangerously-bypass-hook-trust` by default.

First-slice Viber setup only covers `PreToolUse` and `Stop`. Do not scaffold
`PreCompact` by default. Do not add `harness.codex.hooks` runtime config, do not
put native hook policy in `WORKFLOW.md`, and do not use `WORKFLOW.md` workspace
lifecycle hooks as native Codex hook installers.

When hooks are absent or unavailable, leave the repo usable: Viber agents still
follow prompt instructions, reusable Viber skills, and checked-in scripts.

### 9. Validate and report

Run:

```sh
viber init check --json
viber init validate --json
```

Treat `viber init validate --json` as the final readiness gate. If it reports
`needs_setup` or `blocked`, resolve the findings or record the remaining manual
or deferred items. Do not claim the repo is ready while validation findings
remain.

Also run the repo's narrow validation commands if you changed files. Summarize:

- files created or changed;
- GitHub Project, repository link/default repository, board view, Status choices,
  labels, issue template policy, workflow metadata conventions, optional fields,
  and any deferred scheduling constraints;
- Codex availability/version, hook feature state, configured hook events,
  handler types, source locations, trust/review-needed state where observable,
  and whether optional `PreToolUse`/`Stop` readiness is missing, disabled, or
  deferred;
- remaining blockers;
- exact command to start Viber, usually `viber run` from the repo root;
- whether `.viber/config.yaml` still needs local operator values.

If setup is incomplete, leave a concrete checklist rather than claiming the repo
is ready.
