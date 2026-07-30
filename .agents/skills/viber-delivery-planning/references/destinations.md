# Destination and Board Discovery

Load this reference when selecting a destination, inspecting a GitHub Project, or
classifying board Status options. Keep `SKILL.md` as the source for write gates
and guards.

## Adapter Ladder

Make the chosen adapter visible in the confirmation preview.

1. **Viber-backed GitHub Project.** Use when repo-local Viber context identifies
   a Project or dispatch lifecycle. `WORKFLOW.md` is one optional source, not the
   only source; also consider runtime config, setup docs, existing Project items,
   and operator-provided coordinates. This adapter reasons about dispatch safety,
   but still verifies the live Project fields before writes.
2. **Discovered GitHub Project.** Use when no Viber context is available but
   `gh` can discover a single plausible Project linked to the repo owner,
   organization, or repository. If several Projects are plausible, ask the
   operator to choose.
3. **Manual GitHub Project.** Use when the operator supplies Project owner/number
   or chooses one during the run. Read fields and options before previewing
   writes.
4. **Issue-only.** Use when no Project is available or the operator explicitly
   chooses not to use one. Preview that no Project membership or Project fields
   will be written.

## Project Discovery

Resolve candidate owner/number pairs from the highest-confidence available
sources:

- repo-local Viber workflow/config fields;
- repository, owner, or organization Projects visible through `gh`;
- Project items already attached to a refine-mode target issue;
- explicit operator input.

Do not pick a Project because its title sounds right. Names like "Backlog",
"Todo", "Roadmap", or "Planning" are only labels after the owner/number and repo
fit are verified.

For Viber-shaped repos, reuse `.skills/viber-work-start/scripts/viber-gh-context`
when a target issue exists and the script is available. Otherwise use direct
`gh project` reads.

## Board-State Discovery

For any Project adapter:

1. Read the selected Project's fields and options.
2. Identify the Status field by exact field name when configured, otherwise by
   single-select field semantics and operator confirmation.
3. List every Status option on the live board.
4. Classify options as active, handoff/review, terminal, or non-active landing
   using available Viber lifecycle config, existing item states, repository
   conventions, and operator input.
5. Choose an existing non-active landing option. If the classification leaves no
   unique answer, ask the operator.

Do not depend on a hardcoded `active_states` key. If a config exposes active
states under any schema, treat that as one hint and still verify the board's live
columns/options.

## Field Mapping

Capture exact field names, option names, and IDs before confirmation. At minimum
record:

- Project owner and number;
- Project ID;
- Status field ID and target option ID when Status will be set;
- optional native fields such as Phase, Size, Priority, or repo-specific
  equivalents.

If a required field cannot be resolved, ask the operator or switch to explicit
issue-only mode. Keep issue-only mode explicit; it is a valid destination, not a
silent fallback.
