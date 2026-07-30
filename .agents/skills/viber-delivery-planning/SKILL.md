---
name: viber-delivery-planning
description: "Turn a decided product understanding into an issue-ready GitHub delivery plan (optional milestone plus dependency-ordered issues) or refine one rough existing issue in place from a number, OWNER/REPO#N, or URL. Use after product discovery or another decision point when the operator wants delivery planning in a Viber or plain GitHub repo. Runs bounded decomposition/sequencing/grouping, goal-backward readiness checks, native Project/field discovery and verification when a Project is used, confirmation-gated writes through the operator's own gh, and issue-only mode when no Project is selected. Not daemon-dispatched work and not product discovery."
---

# Viber Delivery Planning

Use this skill to turn a *decided* product understanding into GitHub delivery
objects: zero-or-one milestone plus one-or-more dependency-ordered deliverable
issues. It also refines one rough existing issue in place to the same standard.

This is the **delivery** half of the funnel. Product discovery decides what to
build and why; this skill turns an already-decided direction into well-formed,
ordered issues. If the input is still an undecided idea, stop and point the
operator to `viber-product-discovery`.

## Modes

- **Create mode:** author new GitHub objects from a decided brief or discovery
  handoff. Create a milestone only when several deliverable issues need shared
  grouping context. For Viber-backed destinations, create in-deliverable process
  steps as native GitHub sub-issues under their deliverable parent instead of
  body-only checklist items.
- **Refine mode:** when the input is a bare issue number, `OWNER/REPO#N`, or a
  GitHub issue URL for the current repo, read that existing issue, recover its
  real intent, and rewrite the issue body in place. Refine mode does not create
  issues, create milestones, edit dependencies, or move board state.

Use create mode unless the operator clearly supplied an existing issue to
improve. If a referenced issue is only context for new sibling work, keep create
mode and say so.

## References

Load these bundled references only when their condition applies:

- `references/destinations.md`: load when choosing or verifying a destination
  adapter, when a Project is present, when Viber config exists, or when board
  Status options must be classified. It owns the adapter ladder and generic
  board-state discovery.
- `references/github-writes.md`: load after the plan passes self-check and
  before confirmation/writes. It owns `gh` commands, Project write mechanics,
  read-back verification, and the final created-vs-confirmed diff.

## CLI helpers

- `viber plan taxonomy`: run during context loading and use its JSON output as
  the enumerated interview taxonomy, readiness finding set, body-template
  skeleton, and plan JSON contract. Do not invent parallel question categories
  when the taxonomy already covers the decision.
- `viber plan validate`: run after drafting and before the confirmation preview
  in create mode. Feed it a `viber-delivery-plan/v1` JSON plan and do not write
  to GitHub unless it exits 0. A non-zero result is a BLOCKER report to revise
  or show the operator.

## Gotchas

- Project v2 writes need a clean token environment:
  `env -u GH_TOKEN -u GITHUB_TOKEN`.
- Blockers only gate dispatch as *native* `blockedBy` dependencies, not prose in
  an issue body.
- Viber-backed process-step sub-issues are kept non-dispatchable by their native
  parent relationship. The daemon excludes any candidate whose `issue.parent` is
  set (design 0035), even if a Project Status later looks active.
- Do not guess the Project from its name; discover linked/available Projects and
  ask when more than one plausible target exists.
- Native fields are not body metadata: relationships, milestone, priority, size,
  and Project fields live in GitHub structures, never duplicated in body text.
  Never patch a failed native Project write with body prose.
- Description front-matter must stay quoted; regression `341fc24` broke this.

## Operating Rules

- Operate only in the current repository. Resolve the target from
  `git remote get-url origin`, confirm it with `gh repo view`, and write only to
  that repo.
- Write nothing to GitHub before explicit confirmation. Reading context is fine.
- Keep the interview bounded to delivery structure: issue boundaries,
  sequencing, grouping, destination choice, and testable done checks.
- Do not invent acceptance criteria as a formality. If no testable done-check can
  be described, stop with a BLOCKER.
- Treat `research`, `investigate`, `spike`, and `document findings` as process
  language. For exploratory work, require the issue contract to say what durable
  product will be produced, where it lands, what evidence it contains, and how
  completion is judged.
- Preserve the issue content contract as a skill/tooling convention, not Viber
  runtime behavior: problem/motivation, outcome, scope/non-goals,
  constraints/decisions, acceptance, validation, review focus, implementation
  notes, risks, and assumptions must survive rendering into the repo's layout. Do
  not create or update runtime design docs for this issue-body convention.
- Prefer the fewest deliverables that cover the work. Scope guard: every issue
  must stand alone as a real deliverable, never a hollow umbrella whose meaning
  depends on children.
- `blockedBy` edges must form a directed acyclic graph; reject cycles,
  self-blocks, duplicate edges, and missing endpoints before any write.
- Milestone guard: a milestone is repo-scoped grouping for a multi-issue plan
  only; it is independent of Project board membership and dispatch state.
- Priority and size are optional workflow metadata. Omit them rather than
  guessing. When used, set them through confirmed native fields; never include
  them in the issue body as a Metadata section.

## Inputs

Create mode accepts a decided product understanding: problem, motivation,
direction, desired outcome, scope in/out, constraints and decisions, codebase
findings, validation expectations, review focus, risks, assumptions, and any
operator-provided priority, size, acceptance criterion, or intended decomposition.
Treat those as given.

Refine mode accepts an existing issue reference for the current repository plus
optional refinement instructions. The issue's labels, milestone, dependencies,
assignees, and board state are context to preserve.

## Procedure

### 1. Read Context

Resolve and verify the repository identity. Read issue templates from local
`.github/ISSUE_TEMPLATE/`, then the remote template path through `gh api`, then
fall back to the normalized body shape below.

Templates control layout, not substance. Capture section names, required prompts,
and optional Viber mapping comments such as
`<!-- viber:problem motivation constraints -->`. A repository may rename, remove,
combine, or reorder sections; map the canonical content model into that layout
without dropping required content.

Run `viber plan taxonomy` and keep the emitted JSON in the working context. Its
`interview_axes` order is the default interview order, its
`readiness_findings` keys are the allowed BLOCKER/WARNING categories, and its
`body_templates` are the fallback shape when no repository template applies, and
its `native_metadata` section names fields that must stay out of body prose.

Select the destination through `references/destinations.md`. Treat `WORKFLOW.md`
as one optional source among repo-local config, Project field discovery, existing
Project items, and operator input. Discover the board's Status options and a
non-active landing option from the selected Project; do not depend on a
hardcoded config key or a particular column name.

Refine-only: read the target issue body, title, comments, and native fields with
`gh issue view --repo OWNER/REPO`. Capture the current body verbatim so the
confirmation preview can show the before/after change.

### 2. Interview

Ask one question at a time and stop as soon as the delivery structure is clear.
Prioritize:

1. Scope-to-issues: what self-contained deliverable or deliverables must exist?
2. Exploratory deliverable: when the work is a spike/investigation, what product
   will exist, where will it land, what evidence will it contain, and what makes
   it complete?
3. Decomposition: is this one issue or several separately testable outcomes?
4. Sequencing: which deliverables must finish before others can start?
5. Grouping: does a multi-issue plan warrant a milestone?

Prefer concrete options when they help the operator react, but always leave a
freeform escape. Record low-value assumptions instead of asking about them.
Do not re-elicit product intent, motivation, constraints, validation, or review
focus; pull them from the decided understanding. If a missing product decision or
validation/review constraint blocks a well-formed plan, send the operator back to
discovery.

Refine-only: seed the interview from the existing issue text and ask only what
is needed to close gaps in outcome, scope, or testability. If the issue is really
several deliverables, recommend a create-mode run instead of silently splitting
it.

### 3. Draft

Each deliverable issue is drafted from a canonical content model, then rendered
through the selected layout. The normalized shape is content-only: issue bodies
carry delivery context and checks, while native metadata stays in GitHub
fields and relationships. Use these as content units, not mandatory headings:

- problem and motivation;
- user/operator impact;
- desired outcome;
- expected deliverable and destination;
- expected evidence and observable completion condition for exploratory work;
- scope and material non-goals;
- constraints and decisions;
- acceptance criteria;
- validation / how to test;
- review focus / what to look for when human judgment is involved;
- implementation notes, relevant files, risks, and assumptions when known;
- process checklist.

Layout selection order is: explicit operator instruction for this run,
repository template with Viber mapping markers, semantic mapping to existing
headings, Viber recommended setup template, then this fallback layout when no
template exists:

```markdown
## Summary
<One-paragraph issue summary.>

## Problem and motivation
<The user/operator problem, why it matters, and the impact of solving it.>

## Desired outcome
<The observable end state for this deliverable.>

## Deliverable
<What will be produced, where it lands, expected evidence, and how completion is
observed. For exploratory work, this must be a concrete product such as an issue
comment, reusable probe, design record, delivery plan, PR, or another
natural-language destination the operator chose.>

## Scope
<What is in scope and any material non-goals.>

## Constraints and decisions
<Decisions, constraints, compatibility requirements, or policy boundaries carried
from discovery/planning.>

## Acceptance criteria
- [ ] <Testable, observable done-check.>

## Validation
- <Automated checks, manual smoke tests, evidence expected in the PR, or
  environment notes needed to prove the acceptance criteria.>

## Review focus
- <What a human reviewer should inspect, especially areas automation cannot fully
  prove.>

## Implementation notes
- <Relevant files, existing patterns, risks, and assumptions useful to the assignee.>

## Process checklist
- [ ] <In-deliverable progress step such as research, implement, test, PR.>
```

If a template has no obvious home for required content, fold it into the nearest
sensible section, add a minimal section, or ask whether adding a section violates
repo policy. Do not omit required content.

Optional informed assumptions may be appended only when they materially affect
execution:

```markdown
## Assumptions
- <Recorded informed default or unresolved non-critical detail.>
```

Map this normalized shape into the repo template when one exists while preserving
the same content-only rule. Relationships, milestone, size, priority, Status,
and other Project fields are native metadata. Preview and write them through the
destination adapter and `blockedBy` APIs, not as body sections or prose.

Acceptance criteria describe what must be true; validation explains how to prove
it; review focus tells a human what to inspect. Review focus is required when the
work touches UI/UX, docs clarity, behavior changes, releases, migrations,
security/privacy, data handling, or manual-only validation. Use the process
checklist for work inside a deliverable, not for separate deliverables. In
Viber-backed create mode, those process steps are materialized as native child
issues under the deliverable parent and the parent body may link to the
resulting sub-issues instead of carrying checkbox-only progress. Do not add a
Metadata section for size, priority, phase, status, Project, milestone, or
dependencies. If issue-only mode needs human-readable planning notes, put them
under Implementation notes or Assumptions without duplicating native metadata.

Exploratory issues need an explicit deliverable contract. A findings-only issue
comment is a valid product when the issue says that the comment is the
deliverable, names the target issue or thread where it lands, states the
evidence the comment must contain, and gives an observable completion condition.
Executable probes, design records, delivery plans, or other durable products are
equally valid when described with the same clarity. Do not let `research X`,
`investigate Y`, or `document findings` stand alone as the deliverable or as the
only acceptance criterion.

Create mode: draft the optional milestone, issue bodies, optional process
sub-issues, edge set, and topological create-order. Refine mode: draft one
coherent replacement body that keeps the original's concrete requirements,
constraints, links, and criteria while removing redundancy and adding the missing
structure. Do not append a "Refined" section.

### 4. Self-Check

Run a goal-backward readiness check before writing. Credit only what is present
in the draft, not intent.

Per issue, BLOCKER findings include:

- missing or unclear outcome;
- missing problem/motivation or user/operator impact needed by a fresh reader;
- missing required discovery constraints, decisions, non-goals, risks, or
  assumptions from the rendered issue body;
- exploratory work with a missing or ambiguous deliverable product, destination,
  expected evidence, or observable completion condition;
- no testable acceptance criterion;
- acceptance criteria that are only process steps or validation commands;
- missing validation / how-to-test guidance;
- missing review focus when human judgment is required;
- unbounded scope;
- violation of the scope guard;
- contradiction with locked repo policy or selected destination contract.

Plan-level BLOCKER findings include:

- dependency graph is not a DAG;
- edge endpoint is neither in the plan nor an existing tracked issue;
- violation of the milestone guard.

Refine-only: run the per-issue check on the replacement body. Plan-level checks
do not apply.

Revise and re-check up to three passes. If any BLOCKER remains after pass three,
write nothing and report what is missing.

Create mode: serialize the draft as `viber-delivery-plan/v1` JSON and run
`viber plan validate` before the confirmation preview. At
minimum include `repo`, optional `milestone`, and every issue's stable `id`,
`title`, drafted `body`, and `blockedBy` array so readiness validation can check
exploratory deliverable contracts. Treat a non-zero exit exactly like a
remaining BLOCKER finding. There is intentionally no reconcile or idempotency
script.

### 5. Confirm And Write

After the self-check passes, load `references/github-writes.md`.

Show one confirmation preview:

- target repo and selected destination adapter;
- optional milestone, every deliverable issue title/body, and every process
  sub-issue title/body when sub-issues will be created;
- native dependency edges and create-order;
- every Project field/value that will be set and every intentionally unset field;
- the selected non-dispatchable mechanism for process sub-issues: native parent
  relationship, with daemon candidate exclusion for `issue.parent`;
- issue-only absence of Project writes when that mode was selected;
- assumptions and any material change from the original issue in refine mode.

If the operator declines, create or edit nothing. Revise and re-confirm, or stop
cleanly with a zero-write result.

On confirmation, write through the operator's own `gh` following
`references/github-writes.md`: create objects in dependency order, create and
attach process sub-issues when planned, apply native Project fields for Project
adapters, read everything back, and run the closing assertion by diffing created
issues/sub-issues/edges/fields against the confirmed plan.

### 6. Output

Create mode output:

- created milestone URL when used;
- created issues in create-order with URLs;
- created process sub-issues with parent issue URLs when used;
- native `blockedBy` edge verification;
- native parent/sub-issue verification for process sub-issues;
- selected destination adapter and Project read-back result, or explicit
  issue-only confirmation.

Refine mode output:

- refined issue URL;
- summary of body/title changes and anything materially dropped or altered;
- explicit no-op statement when the existing issue was already well-formed.

All outputs include assumptions, stop reasons before writes, or exact
partial-failure state when a write/read-back fails.

## Final Rules

- Do not commit, push, open PRs, edit labels, close/reopen issues, or create
  follow-up issues as part of a planning run.
- In Viber-backed destinations, do not move created issues into an active
  dispatch state.
- In refine mode, edit only the issue body and, when genuinely unclear, the
  title.
