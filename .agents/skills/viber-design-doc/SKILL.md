---
name: viber-design-doc
description: Produce a committed design document or ADR when an issue explicitly requests a repository artifact. Use for design-doc-pr work mode, including generic ADR convention discovery, blocker-decision questions across autonomous/on-demand surfaces, template scaffolding, and validation before PR handoff; not for default RFC discussion issues.
---

# Viber Design Doc

Use this skill when the correct artifact is a committed design document, ADR, or
durable documentation update reviewed through a pull request.

Do not use this skill merely because an issue is an RFC. Default RFC work is
`viber-issue-comment` unless the issue or Project metadata explicitly requests a
repo artifact.

## Inputs

Accept a `viber_work_spec` from `viber-work-start` or equivalent issue context.
The issue must ask for a committed design/doc artifact or be tagged/configured as
`design_doc_pr` work mode.

When the work spec includes `context_pack`, keep it available while drafting.
Treat `target_issue_context` as the dispatched issue,
`dependency_context`/`direct_blocker_identities` as the native dependency gate
result, and the handoff notes, linked PR summaries, direct blocker summaries,
and blocker PR summaries as follow-up context. When dependencies exist, the
blocker identity and summary sections are required prerequisite context for the
design. These sections are bounded dispatch summaries, not primary evidence.
Preserve their `source_url`/`url` values, cite those URLs for material claims,
inspect linked sources before relying on summarized decisions, and carry
`truncation_notices` or dependency omissions into the design doc or PR handoff
when the omitted context could affect the recommendation.

Do not require a repository to have `WORKFLOW.md`. Use it when the governing
workflow names it, but otherwise discover issue, PR, ADR, and review conventions
from the repository's docs, templates, existing design records, and issue thread.

## Bundled Resources

- `assets/adr-template.md`: scaffold new ADRs from this template when the target
  repository has no stronger local template. If the repository already has a
  template, use the local template and ensure the bundled template's required
  concepts are still represented.

## Resolving blocking decisions

A design doc is mostly *decisions* — which alternative, what trade-off, accept vs.
propose. Some are yours to make as the builder; some are not. Split them the way
`viber-product-discovery` does:

- **You decide the technical shape yourself.** Investigate the codebase, the
  existing design records, and the constraints; do not ask the human about the
  technical structure of their own system, and never about their technical
  experience.
- **The human decides vision / preference / priority** — and any fork where the
  design genuinely cannot proceed without their call.

Only a **real blocker** earns a question. A blocker is a decision the doc cannot
be written accurately past without either an answer or an invented assumption — a
true fork (e.g. *persist to a database* vs *keep it in memory*) where each branch
yields a materially different design. Everything below that bar is **not** a
question:

- a non-blocking uncertainty becomes an **Open Questions** entry in the doc
  itself, or a recorded assumption stated in the doc;
- a preference that does not change the design is a default you take and note.

When you hit a real blocker:

- **Ask it answerably in one round.** State the fork, the options, and *your
  recommendation with a one-line why* — never a bare "DB or memory?". A lazy
  question wastes a whole round-trip (a full daemon cycle, in the autonomous
  case), and you bounce again.
- **Then sign off.** Post the question(s) where the work is and stop with
  `needs_reply`. Do not invent the decision and design past it, and do not hold
  open state waiting — the next run picks it up once the human answers. (Viber
  will manage these states more richly later; for now a clean stop is enough.)

This is **one mechanism with two surfaces**, the same whether the run is
autonomous or on-demand:

- **Autonomous (daemon-dispatched):** render the blockers as a single structured
  issue comment, then `needs_reply`. No live human is present.
- **On-demand (operator-invoked):** render them through the runtime's structured
  ask tool (`AskUserQuestion` on Claude Code, `question` on OpenCode; a numbered
  plain-text list elsewhere) when the operator is live — otherwise the same issue
  comment.

**Group blockers, but don't manufacture them.** Ask the blockers visible at one
decision point together rather than one at a time. But a real fork can gate
everything downstream — you often cannot even see the next decision until this one
is answered — so stopping at a genuine blocker the moment you hit it is correct;
do not defer it hunting for siblings. The live ask tool caps at ~4 questions; an
issue comment has none. Treat that 4 as a **smell test**: more than a handful of
"blocking" questions means most are not blockers — route those into the doc's
Open Questions or a single handoff comment, not a gate.

## ADR Convention Discovery

Ground ADR work in the target repository's existing convention before choosing a
path or status. Treat an ADR as a generic architecture decision record: one
durable decision, dated context, selected decision, consequences, and rejected or
deferred alternatives.

Discover the convention in this order:

1. Prefer an issue-named target path, local ADR template, or registry named in the
   issue body, comments, or provided work spec.
2. Search the repository for design/decision-record evidence: docs indexes,
   README guidance, directories or headings containing `ADR`, `architecture
   decision`, `decision record`, or `design record`, and Markdown files with
   status/date/context/decision/consequences structure.
3. Choose the directory with the strongest record convention, not a hardcoded
   path. Infer the index or registry from nearby `index.md`, `README.md`, table
   rows, or a directory-level list of records.
4. Infer numbering from existing filenames and headings. Support numeric
   prefixes such as `0001-title.md`, `001-title.md`, and `1-title.md`; date
   prefixes such as `YYYY-MM-DD-title.md`; and unnumbered slug conventions. Use
   the next number at the discovered width or today's date for date-based
   schemes.
5. If no convention exists, create a conventional `docs/adr/` directory with
   four-digit numeric records, starting at `0001`, plus an `index.md` registry.
   This fallback is generic; do not use this repository's own design-record path
   unless the target repository itself reveals that convention.

Discover the lifecycle vocabulary from existing records and registry rows. For a
new record, use the repository's non-accepted pre-decision status when one exists
(`Proposed` or `Draft` are common). If none is discoverable, use `Proposed`. Do
not mark a record `Accepted`, `Approved`, or equivalent unless the issue thread
or repository policy explicitly records human sign-off.

## Procedure

Use plan-validate-execute:

1. Plan from evidence. Read the issue body and comments to capture the decision
   context, prior disagreement, explicitly requested artifact, and any unresolved
   forks. Read only the existing design records/docs needed to bound the
   decision.
2. Discover the ADR or design-doc convention, including location, numbering,
   template, status vocabulary, and index/registry expectations.
3. Resolve blocking decisions before drafting. If a real blocker exists, ask the
   one-round question(s), stop with `needs_reply`, and do not create a PR.
4. Scaffold new ADRs from `assets/adr-template.md` unless a local template is
   stronger. Adapt heading names to the repository convention, but keep the
   required concepts: Status, Date, Context, Decision, Consequences, and
   Alternatives.
5. Draft the durable artifact. Prefer concise ADR form for a single decision:
   context, decision/recommendation, consequences, alternatives, and open
   questions only when they are useful. Avoid transcript-style docs and
   speculative implementation plans.
6. Update the discovered index or registry. If using the generic fallback, create
   `index.md` with a row/link for the new ADR.
7. Run the validation loop below and fix the draft until it passes.
8. Open or update one pull request only after validation passes and PR handoff is
   configured.
9. Finish through `viber-work-wrap-up` with work mode `design_doc_pr`.

## Validation Loop

Before opening or updating a PR, check the drafted record and any index/registry
changes. Fix every failure and repeat until the checklist passes:

- Required concepts are present: Status, Date, Context, Decision, Consequences,
  and Alternatives. Local heading names may differ, but the content must be
  explicit.
- Status is one of the repository's discovered lifecycle values, or `Proposed`
  when no convention exists. New records are `Proposed` or `Draft` unless human
  sign-off explicitly authorizes an accepted status.
- Filename, heading, and number/date match the discovered convention and do not
  collide with an existing record.
- Relative Markdown links in the changed record and index resolve on disk.
  External links are syntactically valid unless the repository has a stronger
  link-check command.
- The discovered index or registry has a row or entry for the record, and the
  row links to the drafted file with matching title/status.
- Superseding records state what they supersede and why. The accepted record
  being superseded is not edited unless repository policy explicitly allows an
  append-only registry update.
- Any conflict between existing docs and current code is stated explicitly in
  the new record or handoff instead of silently choosing one.
- The planned PR reference follows the repository convention: `Refs #N` or
  `Related to #N` by default; closing keywords only when the governing
  repository workflow or maintainer instruction requires them.

If validation cannot pass without a human decision, stop with `needs_reply`.

## Gotchas

- Treat accepted decision records as append-only. Supersede with a new record;
  do not rewrite accepted history.
- Land new records as `Proposed` or `Draft` unless explicit human sign-off or
  repository policy authorizes acceptance. Do not self-accept the design.
- Update the ADR index or registry when the repository has one; create one when
  using the generic fallback.
- Reference issues with `Refs #N` or `Related to #N` by default. Use `Closes`,
  `Fixes`, or `Resolves` only when the repository convention says to.
- State doc-vs-code conflicts explicitly. A design record should not hide that
  code and documentation disagree.

## PR Rules

- Use `Refs #N` or `Related to #N` by default.
- Use closing keywords only when the governing repository workflow, PR template,
  or maintainer instruction explicitly allows or requires them for PR-based
  work.
- Do not close the issue manually.
- Do not mark the design as accepted unless the repo convention or issue
  explicitly allows the agent to do so. Otherwise use a draft/proposed status or
  leave acceptance for human validation.

## Rules

- Do not implement production behavior unless the issue explicitly asks for a
  small supporting change.
- Keep repository inspection narrow and evidence-driven.
- Do not move final Project Status directly unless the governing workflow
  explicitly authorizes it. If authorized, move only the tracked issue's allowed
  Status field to the workflow's target state.
- If the design needs a human decision before a document can be accurate, stop
  with `needs_reply` instead of inventing the decision — see "Resolving blocking
  decisions" for the blocker test, the one-shot question form, and the
  autonomous/on-demand surfaces.

## Wrap-Up

Load `viber-work-wrap-up` and emit a `viber-result` block. Required policy
checks for this mode:

- `pull_request` is present when PR handoff is configured.
- `used_closing_keywords` matches the workflow policy.
- `moved_project_status` matches the workflow policy.

For PR handoff, use `viber-work-wrap-up`'s `viber-pr-policy-check` script before
claiming `ready_for_validation`.
