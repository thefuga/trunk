---
status: resolved
trigger: "Display option toggle buttons (Space, Pilcrow, TextWrap) flicker when opening a diff"
created: 2026-03-30T00:00:00Z
updated: 2026-03-30T00:00:00Z
---

## Current Focus

hypothesis: Toggle state is initialized synchronously as false (line 47-48 DiffPanel.svelte), then overwritten asynchronously by LazyStore $effect (lines 60-71), causing a visible flash of inactive state.
test: Trace the initialization path and confirm the timing gap
expecting: The $state defaults render one frame before the $effect Promise resolves
next_action: Confirm root cause by reading the full initialization flow

## Symptoms

expected: Toggle buttons should appear in their persisted state immediately when a diff opens
actual: Buttons briefly flash as inactive/unselected, then snap to their persisted active state
errors: none (visual flicker only)
reproduction: Open a diff when any display toggle (invisibles, word wrap) is persisted as active
started: Phase 63 (when these toggles were added)

## Eliminated

(none yet)

## Evidence

- timestamp: 2026-03-30T00:01:00Z
  checked: DiffPanel.svelte state initialization (lines 45-48)
  found: viewMode, ignoreWhitespace, showInvisibles, wordWrap all initialized synchronously with hardcoded defaults (hunk, false, false, false)
  implication: First render will always show these defaults regardless of persisted state

- timestamp: 2026-03-30T00:02:00Z
  checked: DiffPanel.svelte $effect (lines 60-71)
  found: Persisted values loaded via Promise.all inside $effect, which runs AFTER first render. The resolved values overwrite the $state vars.
  implication: There is always at least one render frame with the default values before the async load completes

- timestamp: 2026-03-30T00:03:00Z
  checked: DiffToolbar.svelte toggle rendering (lines 60-83)
  found: Toggle buttons use class:active={showInvisibles}, class:active={wordWrap} etc. These are purely reactive to the props passed from DiffPanel.
  implication: DiffToolbar is a dumb display component — the flicker originates entirely from DiffPanel's async initialization pattern

- timestamp: 2026-03-30T00:04:00Z
  checked: store.ts LazyStore functions (lines 302-322)
  found: getDiffShowInvisibles and getDiffWordWrap are async functions returning Promise<boolean>. LazyStore lazily initializes on first access, adding additional latency on the very first call.
  implication: The async nature of these store functions makes synchronous initialization impossible without a different pattern

## Resolution

root_cause: DiffPanel.svelte initializes toggle state synchronously with hardcoded defaults (false) on lines 45-48, then loads persisted values asynchronously in a $effect on lines 60-71. Since $effect runs after the first render, there is always at least one rendered frame showing the default (inactive) state before the Promise resolves and updates the $state variables. This creates the visible flicker.
fix:
verification:
files_changed: []
