---
phase: quick
plan: 260402-wea
type: execute
wave: 1
depends_on: []
files_modified:
  - src/components/TabBar.svelte
  - src/components/TabBar.test.ts
autonomous: true
requirements: [QUICK-260402-wea]

must_haves:
  truths:
    - "Hovering a tab shows the full repo path as a native browser tooltip"
    - "Tabs with no repoPath show the repoName (or 'New Tab') as tooltip"
  artifacts:
    - path: "src/components/TabBar.svelte"
      provides: "title attribute on tab-item div"
      contains: "title="
    - path: "src/components/TabBar.test.ts"
      provides: "Tests for tooltip title attribute"
      contains: "title"
  key_links:
    - from: "src/components/TabBar.svelte"
      to: "tab.repoPath"
      via: "title attribute binding"
      pattern: "title=.*repoPath"
---

<objective>
Add a native HTML `title` attribute to each tab in TabBar so hovering reveals the full repository path.

Purpose: When multiple repos share the same name (e.g. forks), users need to distinguish them by full path. A native tooltip on hover is the simplest solution.
Output: Updated TabBar.svelte with title attribute, updated TabBar.test.ts with regression tests.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@src/components/TabBar.svelte
@src/components/TabBar.test.ts
@src/lib/tab-types.ts
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Add title tooltip to tab items and test</name>
  <files>src/components/TabBar.svelte, src/components/TabBar.test.ts</files>
  <behavior>
    - Test 1: Tab with repoPath shows repoPath as title attribute (e.g. "/path/to/trunk")
    - Test 2: Tab with null repoPath shows repoName as title attribute
    - Test 3: Tab with null repoPath AND empty repoName shows "New Tab" as title
  </behavior>
  <action>
    In TabBar.svelte, add a `title` attribute to the `.tab-item` div (line 66-77):

    ```
    title={tab.repoPath || tab.repoName || 'New Tab'}
    ```

    This uses the native HTML title attribute which renders a browser tooltip on hover. The fallback chain is: full repo path -> repo name -> "New Tab".

    In TabBar.test.ts, add a describe block "tab tooltips" with three tests:
    1. Check that the tab with repoPath="/path/to/trunk" has title="/path/to/trunk"
    2. Add a tab with repoPath=null, repoName="orphan" to verify title="orphan"
    3. Add a tab with repoPath=null, repoName="" to verify title="New Tab"

    Use `.closest('.tab-item')?.getAttribute('title')` or `screen.getByRole('tab', { name })` patterns consistent with existing tests.
  </action>
  <verify>
    <automated>cd /Users/joaofnds/code/trunk && npx vitest run src/components/TabBar.test.ts</automated>
  </verify>
  <done>
    - Tab items have a title attribute showing the full repo path
    - Fallback to repoName when repoPath is null
    - Fallback to "New Tab" when both are empty/null
    - All new and existing TabBar tests pass
  </done>
</task>

</tasks>

<verification>
npx vitest run src/components/TabBar.test.ts
just check
</verification>

<success_criteria>
- Hovering any tab in the app shows a tooltip with the full repo path
- Tabs without a repo path show the repo name or "New Tab"
- All TabBar tests pass including new tooltip tests
- `just check` passes clean
</success_criteria>

<output>
After completion, create `.planning/quick/260402-wea-show-full-repo-path-tooltip-on-tab-hover/260402-wea-SUMMARY.md`
</output>
