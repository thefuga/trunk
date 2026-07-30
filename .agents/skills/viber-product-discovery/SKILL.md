---
name: viber-product-discovery
description: "Operator-invoked product discovery for Viber-capable or plain repositories. Use when the user brings a rough, vague, fuzzy, or half-written product idea and wants to think through what to build before issues, code, docs, or ADRs are authored. The skill turns undecided product intent into a confirmed decided understanding: problem, audience, direction, scope, constraints, and codebase/system findings. It asks only vision/preference/priority questions, investigates technical details itself, stays read-only, writes nothing by default, and asks the operator to choose a follow-up route. Do not use for already-decided briefs that need issue decomposition, existing-issue refinement, implementation, PR review, or daemon-dispatched work; use viber-delivery-planning or the relevant work-mode skill instead."
---

# Viber Product Discovery

Use this skill at the front of the funnel — when there is an idea, a brief, or a
vague want, and the problem and solution are not yet decided. It owns
**problem/solution understanding**; turning a *decided* thing into well-formed,
ordered issues is `viber-delivery-planning`'s job, not this skill's.

This is a conversation, not a wizard. No forms, no DAGs, no identity guards, and
no issue-authoring ceremony — that belongs to `viber-delivery-planning`.
The output is a *decided product understanding* you present back to the operator,
held in the session and in the operator's head — not a file. Per
[0023 - Spec-framework evaluation and workflow model](../../docs/design/0023-spec-framework-evaluation-and-workflow-model.md),
decisions live in docs and work lives in issues; this skill persists neither on its
own (see "Writing nothing" below).

The front of the funnel is the highest-leverage moment: every issue authored
later is bounded by how well the problem and solution are understood now. Spend
the effort here.

## Staying in discovery across a long session

Discovery is long and conversational, so the session can fill the context window
and trigger the runtime's auto-compaction. Compaction preserves *what you
understood* (the feature) but tends to drop *the frame* — that you are in
discovery and must write nothing. After a compaction the summary can read like an
ordinary, well-specified feature request, and the default reaction to that is to
start building. Preventing exactly that is what this section is for.

Keep the frame durable:

- **Restate the mode.** At the top of every understanding recap, and before any
  tool call that could mutate state, re-emit one line:
  `MODE: DISCOVERY — investigate read-only; write nothing until the operator routes a follow-up.`
  Recently-restated text survives compaction far better than instructions read
  once when the skill first loaded.
- **When in doubt, you are still discovering.** If you are ever unsure whether
  this skill is still active — it is. A fleshed-out, decided understanding is
  **not** permission to build. Only an explicit operator follow-up choice (issues
  / doc / ADR) authorizes any write.
- **Keep a rolling distilled recap.** Treat the compact "decided product
  understanding" structure below as the live source of truth and refresh it as the
  conversation moves. A small living summary degrades gracefully under compaction;
  a long Q&A transcript does not.

## Gotchas

- Compaction can silently re-frame discovery as a build ticket. Re-emit
  `MODE: DISCOVERY — investigate read-only; write nothing until the operator routes a follow-up.`
  after compaction, in every recap, and before mutating-capable tool calls; run
  the pre-write gate before any mutation.
- A *decided product understanding* is not a write trigger. It only means you can
  ask which follow-up route, if any, the operator wants.
- The exploration subagent stays read-only and returns concise bullets with
  `file:line` pointers. It must not write, edit, or dump transcripts.
- Operate in the current repository. Do not broad-search the filesystem.
- Never interrogate the founder about their own tech stack, system, or technical
  experience. Investigate that yourself.
- No subagent facility means investigate inline. It does not mean skip
  investigation.

## When to use it

- An operator hands you a one-line idea, a pasted brief, a doc pointer, or a
  half-formed want, and wants to think it through before committing to work.
- Use it for both a **brief idea** (little to go on — question deeply) and a
  **solid one** (a clear prompt or a written doc — extract from it and skip
  questions it already answers). Match the depth to the input; do not re-ask what
  the operator already told you.

## When not to use it

- The thing is already decided and the operator wants issues — go straight to
  `viber-delivery-planning`.
- The operator wants to improve one existing rough issue in place — that is
  `viber-delivery-planning` refine mode.
- Daemon/autonomous work — this is operator-invoked tooling, never dispatched.

## The founder/builder split

The operator is the founder/visionary. You are the builder. This split decides
who answers what, and it is not negotiable:

- **Ask the operator only about vision, preference, and priority** — what problem
  this solves, who it is for, what "good" means, what matters most, what is in or
  out of scope, which trade-off they want.
- **You figure out the rest yourself** — codebase patterns, where the work lands,
  technical risk, integration points, the implementation approach. Investigate;
  do not interrogate the operator about the technical shape of their own system.
- **Never ask the operator about their own technical experience.** Not "are you
  familiar with X," not "do you know how Y works." You build; their job is to
  want. If you need technical context, find it in the repo or research it.

## Two entry paths

Read the input first and pick the depth — do not run a fixed script.

- **Brief idea (deep questioning).** Little signal in the prompt. Open the
  conversation, let the operator dump their mental model, then follow the thread
  with questions until the understanding is decided. This is the questioning
  discipline below.
- **Solid idea (fast path).** A clear prompt or an attached doc/design record
  already carries much of the intent. Extract everything it states, investigate
  the codebase to ground it, and ask only the questions the document genuinely
  leaves open. Do not march the operator back through what they already wrote.

## How to question

Use a dream-extraction questioning style. You are a thinking partner, not an
interviewer — help the operator sharpen a fuzzy idea into a decided one.

- **One question at a time.** Never fire a list. Follow the answer you just got.
- **Start open, then follow energy.** Let them describe it in their words; dig
  into whatever they emphasized or got excited about.
- **Challenge vagueness.** "Good" means what? "Users" means who? "Fast" how?
  Never accept a fuzzy answer and move on.
- **Make the abstract concrete.** "Walk me through using this." "What does that
  actually look like?" "Give me an example."
- **Clarify ambiguity.** "When you say X, do you mean A or B?"
- **Surface and record assumptions** instead of asking low-value questions — but
  say which they are, and let the operator correct them.
- **Know when to stop.** When you can state the problem, who it is for, the
  decided direction, and what "done" looks like, stop questioning and present the
  understanding back. Do not pad the conversation.

Anti-patterns to avoid: checklist-walking fixed domains regardless of the thread;
corporate-speak ("what are your success criteria"); interrogating without building
on answers; rushing to minimize questions; accepting vague answers; asking about
tech stack before you understand the idea; and — the hard rule — asking about the
operator's technical skills.

### Asking with options

Presenting concrete options to react to is a strong way to sharpen a fuzzy idea —
interpretations of what they might mean, specific examples to confirm or deny,
trade-offs that reveal priorities. Use it where it helps; plain conversation is
always a valid fallback.

When the runtime exposes a structured multiple-choice ask tool, prefer it for these
react-to-options moments. Known adapters:

- **Claude Code** → `AskUserQuestion`
- **OpenCode** → `question`
- **Codex / Gemini / any runtime without a structured ask tool** → ask in plain
  text, offering the options as a short numbered list.

This adapter list is expected to grow; when a runtime you're on isn't listed, fall
back to the vendor-agnostic plain-text form rather than guessing at a tool. Whatever
the affordance:

- Keep it to 2–4 concrete, non-leading options with a short header.
- Always leave a freeform escape ("type your own" / "let me explain"). The moment
  the operator wants to describe something in their own words, **drop the options
  and continue in plain conversation** — resume structured asks only afterward.
- Never encode an option that asks about the operator's technical experience; the
  founder/builder rule holds regardless of the affordance.

## Progressive, adaptive depth

Start high-level — business and product intent — and drill down only as far as
the task actually demands. Let the work pull the depth:

- A small product tweak may need no more than the product intent and one scope
  boundary.
- "Marketing wants to track user clicks" → "clicks from where?" → and only if the
  task lands in a backend repo do you go down to the request/endpoint and
  network level. Do not pre-emptively spec infrastructure the task does not reach.

**Investigate what you can; ask for what you cannot.** Build your own
understanding of the codebase and the relevant systems — find the patterns, locate
where this would land, name the technical risk — before and while you ask. When you
hit context the repo cannot give you and you cannot safely infer, ask for it rather
than guess. Operate in the current repository; do not broad-search the filesystem.

### Dispatch codebase exploration to a subagent

Dispatch the codebase investigation to a **read-only exploration subagent** rather
than reading the repo inline. This keeps the discovery conversation's context clean
and focused on the operator while the exploration runs off to the side — the builder
half of the split. The point is to *dispatch exploration*, not to depend on any
particular custom agent type: use whatever subagent facility the harness exposes.

- Dispatch it with the harness's subagent/Task facility, whatever that is. Give it a
  tight question — "where would click-tracking land in this repo, what patterns and
  integration points exist, what's the technical risk" — scoped to the current depth,
  not "understand the whole system."
- The subagent reads and reports; it does **not** write, edit, or run mutating
  commands. It returns **concise findings only** — bullets and `file:line`
  pointers, never file dumps or a transcript. Verbose returns defeat the purpose:
  they bloat the discovery context and bring the next compaction closer.
- Keep the main discovery thread lean: do not read large files inline in it.
  Every large read you pull into the conversation is context spent toward the next
  compaction — push that work into the subagent instead.
- Dispatch one when a question genuinely needs the codebase (where work lands,
  feasibility, technical risk), and re-dispatch as the conversation drills deeper.
  Don't dispatch one for product/vision questions only the operator can answer.
- **If the harness has no subagent facility, investigate inline** in the main
  context instead — accept the added context cost; the exploration still happens, it
  just is not isolated off to the side. Don't skip investigation for lack of a
  subagent.
- Fold the findings into the decided understanding's "Codebase & system findings" —
  these are things you discovered, never things you asked the operator.

## Reaching the decided product understanding

You are done discovering when you can present back a coherent, decided
understanding — the handoff contract this skill produces. This is a skill/tooling
contract, not Viber runtime behavior: do not create or update runtime design docs
for this issue-body convention. The handoff is a structured planning packet, not
just a loose recap, because `viber-delivery-planning` must preserve its substance
into GitHub issues for a fresh human or agent who did not read the discovery chat.

Present it conversationally, but keep these content units explicit:

- **Problem & intent** — the why (business/product motivation) and who it is for.
- **User/operator impact** — what gets better, clearer, faster, less risky, or
  less frustrating when this exists.
- **Decided direction** — the settled what and how, at the depth the task reached:
  the solution shape, the tools/systems involved, the approach.
- **Scope** — what is in, and explicitly what is out.
- **Constraints & considerations** — only those that materially bear: technical
  constraints, security/privacy if the work plausibly touches either, UX/interface
  if it changes a user- or operator-facing surface.
- **Codebase & system findings** — what you discovered yourself: relevant
  patterns, where the work lands, integration points, technical risk. (The builder
  half of the split — not things you asked the operator.)
- **Acceptance seeds** — observable outcomes or done-checks already implied by the
  discovery, before delivery planning decomposes them into per-issue criteria.
- **Validation expectations** — automated checks, manual smoke tests, evidence, or
  environments likely needed to prove the work.
- **Review focus** — what a human reviewer should inspect when automation cannot
  prove everything: UX, docs clarity, edge cases, migrations, security/privacy,
  release risk, or similar judgment-heavy areas.
- **Open questions & assumptions** — what is still undecided, and what you assumed
  rather than confirmed, flagged so the operator can correct it.

"Decided" means there is enough settled understanding that the next stage does not
have to re-derive product intent — not that every detail is nailed down. Validate
the recap with an explicit present -> correct -> re-present loop:

1. Present the recap with the `MODE: DISCOVERY` line.
2. Ask the operator to confirm it or correct anything wrong, vague, or missing.
3. Fold in corrections and re-present the updated recap.
4. Repeat until the operator confirms the understanding or chooses to stop.

This confirmed recap is the input `viber-delivery-planning` consumes; it lets that
skill skip product elicitation and go straight to decomposition, sequencing, and
grouping.

### Planning handoff packet

When the operator chooses the Issues route, pass the latest decided understanding
to `viber-delivery-planning` as a compact packet with these headings. Do not rely
on the downstream planner to reconstruct them from the chat transcript:

```markdown
## Problem and motivation

## User/operator impact

## Desired outcome

## Decided direction

## Scope

## Constraints and decisions

## Codebase and system findings

## Acceptance seeds

## Validation expectations

## Review focus

## Risks

## Open questions and assumptions
```

Use the headings as a handoff contract, not as a GitHub issue layout mandate. The
delivery planner maps this content into the repository's issue template or layout.

## Writing nothing

This skill writes nothing by itself — no issues, no docs, no files, no flat-file
ledger, no code. Reaching the decided understanding is the whole job; persisting
it is a separate, user-directed follow-up. This is a hard gate, not a closing
note: it holds for the entire session, including after a compaction.

**Pre-write gate.** Before any Write / Edit / file creation / commit / mutating
command:

1. Re-emit
   `MODE: DISCOVERY — investigate read-only; write nothing until the operator routes a follow-up.`
2. Check whether the operator explicitly chose a follow-up route that requires a
   write.
3. If no route was chosen, do not write. Present or refresh the understanding and
   ask how to follow up instead.
4. If a route was chosen, name the route and destination before the mutating
   action. For docs, ADRs, or wiki pages, draft first and ask for explicit
   confirm-before-write approval.

Read-only investigation is always allowed; mutating anything never is until the
operator routes you.

When the understanding is decided, **always ask how to follow up** and route per
the operator's choice — the operator decides when they are ready; you do not
auto-chain:

- **Issues** → load `viber-delivery-planning` and hand it the decided product
  understanding as input. This is the default when the outcome is GitHub work.
- **A doc or ADR / wiki** → draft it only if asked, present the target path or
  destination, and get explicit confirm-before-write approval before creating or
  editing anything. Design records are not a Viber requirement and not every repo
  uses them; never produce one by default.
- **Nothing** → end cleanly; the operator just wanted to think it through.

If the delivery destination is not inferable from the initial prompt, **ask the
operator where this should land** (issues / a doc / an ADR / a wiki / nothing)
rather than assuming. Loading `viber-delivery-planning` happens only when the
operator asks to create issues.

## Rules

- Stay in the current repository; do not broad-search the filesystem.
- Ask only about vision, preference, and priority.
- Never ask about the operator's technical experience or their own tech stack.
- Investigate technical shape yourself.
- Use a read-only exploration subagent when available; investigate inline when
  none exists.
- Keep exploration returns to concise bullets with `file:line` pointers.
- Prefer structured ask tools for react-to-options; always offer a freeform
  escape.
- Match depth to the input and task.
- Re-emit the `MODE: DISCOVERY` line in every recap and before
  mutating-capable tool calls.
- Run the pre-write gate before any mutation.
- Write nothing unless the operator confirms a write-capable follow-up route.
- Do not auto-chain into `viber-delivery-planning`.
- Keep ceremony low; do not add identity guards, dependency graphs, or issue
  decomposition here.

## Output

A decided product understanding, presented back to the operator in-session as the
planning handoff packet above, followed by the follow-up question and the routing
the operator chose. Nothing is written to GitHub or to disk unless the operator
directs a follow-up that does so.
