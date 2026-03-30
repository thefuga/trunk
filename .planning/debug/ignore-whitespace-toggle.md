---
status: diagnosed
trigger: "Toggling Ignore Whitespace on does NOT change the diff content"
created: 2026-03-30T00:00:00Z
updated: 2026-03-30T00:00:00Z
---

## Current Focus

hypothesis: CONFIRMED -- Wrong git2 API method used: `ignore_whitespace_change` instead of `ignore_whitespace`
test: Traced full data flow from toggle to git2 call
expecting: N/A -- root cause found
next_action: Return diagnosis

## Symptoms

expected: When "Ignore Whitespace" is toggled on, whitespace-only changes should disappear from the diff
actual: Diff shows the same content whether Ignore Whitespace is on or off
errors: None reported
reproduction: Indent lines in a file, toggle Ignore Whitespace on/off, diff unchanged
started: Phase 63

## Eliminated

- hypothesis: ondiffoptionschange callback not triggering re-fetch
  evidence: Traced the flow -- DiffPanel.svelte line 86 calls ondiffoptionschange, RepoView.svelte line 662-665 handler calls refetchFileDiff which calls buildDiffOptions (reads from store) and re-invokes the Rust diff command with updated options. The callback chain is correct for staging diffs.
  timestamp: 2026-03-30

- hypothesis: ignoreWhitespace not being passed to Rust backend
  evidence: DiffRequestOptions in types.rs uses #[serde(rename_all = "camelCase")] so TS camelCase maps correctly to Rust snake_case. buildDiffOptions() reads getDiffIgnoreWhitespace() from store and passes it in the options object to the Rust command. Confirmed correct.
  timestamp: 2026-03-30

## Evidence

- timestamp: 2026-03-30
  checked: DiffPanel.svelte handleIgnoreWhitespaceChange (line 83-87)
  found: Correctly calls setDiffIgnoreWhitespace(value) then ondiffoptionschange?.()
  implication: Frontend toggle and callback chain works correctly

- timestamp: 2026-03-30
  checked: RepoView.svelte ondiffoptionschange handler (line 662-665)
  found: Calls refetchFileDiff -> buildDiffOptions -> getDiffIgnoreWhitespace from store -> passes to Rust
  implication: Options are correctly read from store and sent to backend

- timestamp: 2026-03-30
  checked: Rust DiffRequestOptions serde (types.rs line 162-171)
  found: Uses #[serde(rename_all = "camelCase")] with field ignore_whitespace: bool
  implication: Deserialization is correct, TS ignoreWhitespace maps to Rust ignore_whitespace

- timestamp: 2026-03-30
  checked: apply_request_options in diff.rs line 32-40
  found: Uses opts.ignore_whitespace_change(req.ignore_whitespace) -- THIS IS THE WRONG METHOD
  implication: ignore_whitespace_change = git -b = only ignores changes in AMOUNT of whitespace. For indentation (adding whitespace where none existed), need ignore_whitespace = git -w = ignore ALL whitespace

- timestamp: 2026-03-30
  checked: Existing test (test_diff.rs line 293-339)
  found: Test uses "hello world" -> "hello  world  " (extra spaces between existing words). This is a change in AMOUNT of whitespace, which ignore_whitespace_change catches. Test passes but doesn't cover the indentation case.
  implication: The test gave false confidence -- it tested a case that works with the weaker flag

- timestamp: 2026-03-30
  checked: Secondary issue -- ondiffoptionschange for commit diffs
  found: RepoView.svelte line 662-665 only re-fetches when selectedFile is set (staging). When viewing commit diffs (selectedCommitFile), the callback does nothing because selectedFile is null
  implication: Ignore whitespace toggle has no effect on commit diffs at all (separate bug)

## Resolution

root_cause: Wrong git2 API method in apply_request_options (diff.rs line 39). Code calls opts.ignore_whitespace_change() which maps to git -b (only ignores changes in AMOUNT of whitespace). For the user's scenario (indenting lines = adding new leading whitespace), the correct method is opts.ignore_whitespace() which maps to git -w (ignores ALL whitespace differences).
fix:
verification:
files_changed: []
