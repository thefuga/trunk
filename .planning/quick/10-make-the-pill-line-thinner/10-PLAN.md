---
phase: quick
plan: 10
type: execute
wave: 1
depends_on: []
files_modified:
  - src/components/CommitGraph.svelte
autonomous: true
requirements: [quick-10]
must_haves:
  truths:
    - "Pill connector line from ref pill to commit dot is visually thinner than before"
  artifacts:
    - path: "src/components/CommitGraph.svelte"
      provides: "Thinner pill connector line"
  key_links: []
---

<objective>
Make the ref pill connector line (from the pill to the commit dot) thinner.

Purpose: The connector line between ref pills and commit dots is currently the same thickness as graph edges (1.5px via `displaySettings.edgeStroke`). It should be thinner to be less visually dominant.
Output: Updated CommitGraph.svelte with thinner pill connector stroke width.
</objective>

<execution_context>
@/Users/joaofnds/.config/Claude/get-shit-done/workflows/execute-plan.md
@/Users/joaofnds/.config/Claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/components/CommitGraph.svelte
@src/lib/graph-constants.ts
</context>

<tasks>

<task type="auto">
  <name>Task 1: Reduce pill connector line stroke width</name>
  <files>src/components/CommitGraph.svelte</files>
  <action>
In `src/components/CommitGraph.svelte`, find the connector line SVG element (around line 586-595) that draws from the pill to the commit dot. It currently uses `stroke-width={displaySettings.edgeStroke}` (which is 1.5).

Change the stroke-width to a hardcoded value of `1` to make it thinner:

```svelte
stroke-width={1}
```

This keeps the graph edge strokes at 1.5 but makes the pill connector lines subtler.
  </action>
  <verify>
    <automated>cd /Users/joaofnds/code/trunk && grep -n 'stroke-width' src/components/CommitGraph.svelte | grep -v 'edgeStroke\|mergeStroke\|dotRadius'</automated>
  </verify>
  <done>Pill connector line uses stroke-width of 1 instead of 1.5, making it visually thinner.</done>
</task>

</tasks>

<verification>
- `grep 'stroke-width={1}' src/components/CommitGraph.svelte` shows the connector line with the new value
- Visual: Open app, view a branch with ref pills — connector lines from pills to dots should be noticeably thinner than graph edges
</verification>

<success_criteria>
Pill connector lines render at 1px stroke width, visually thinner than the 1.5px graph edges.
</success_criteria>

<output>
After completion, create `.planning/quick/10-make-the-pill-line-thinner/10-SUMMARY.md`
</output>
