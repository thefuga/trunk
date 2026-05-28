---
status: resolved
trigger: "Branches that have stashes render the entire branch line as dashed instead of only the stash-to-commit connector being dashed."
created: 2026-03-15T00:00:00Z
updated: 2026-05-28T00:00:00Z
verified_at: 2026-05-28
---

## Current Focus

hypothesis: CONFIRMED - stash_lanes HashSet in Rust graph.rs is never cleaned up when a non-stash commit inherits a stash column, causing all downstream edges at that column to be permanently dashed.
test: Traced through code logic for non-HEAD branch stash scenario.
expecting: N/A - root cause found.
next_action: Return diagnosis.

## Symptoms

expected: Only the line from a stash square to its parent commit dot should be dashed. Lines connecting two regular commits should always be solid.
actual: When a branch has stashes, the entire branch line (including segments between regular commits) becomes dashed.
errors: No errors - purely visual rendering bug.
reproduction: Create stashes on a branch, then view the commit graph. The branch lines become dashed instead of solid.
started: Unknown exact start, present in current code.

## Eliminated

- hypothesis: Bug is in the frontend overlay-paths.ts or active-lanes.ts rendering
  evidence: Frontend faithfully propagates the dashed boolean from backend edges. Coalescing logic correctly breaks segments when dashed changes. The wrong data comes from the Rust backend.
  timestamp: 2026-03-15

- hypothesis: inline_stash_parent mechanism causes the bug
  evidence: inline_stash_parent is properly cleared when the parent commit is processed (line 185-188 of graph.rs). It only affects pass-through edges between the stash row and parent row, which is correct behavior. Additionally, inline stash placement only happens for HEAD-chain stashes where the parent is HEAD tip.
  timestamp: 2026-03-15

## Evidence

- timestamp: 2026-03-15
  checked: Frontend rendering pipeline (overlay-paths.ts, active-lanes.ts, CommitGraph.svelte)
  found: The dashed boolean on edges flows from Rust backend -> GraphEdge.dashed -> OverlayEdge.dashed -> OverlayPath.dashed -> SVG stroke-dasharray. Frontend just passes through whatever the backend sends.
  implication: The bug is in the Rust backend edge generation, not the frontend.

- timestamp: 2026-03-15
  checked: Rust graph.rs lines 335, 351, 374 - first-parent edge dashed logic
  found: All three places use `dashed: stash_lanes.contains(&col) || is_stash`. For a non-stash commit at a column in stash_lanes, this evaluates to true even though the commit is not a stash.
  implication: Any non-stash commit assigned to a column previously marked as a stash lane will have dashed edges.

- timestamp: 2026-03-15
  checked: Rust graph.rs stash_lanes lifecycle (insert at line 242, remove at line 305)
  found: stash_lanes.insert(c) happens when a branched stash gets a new column. stash_lanes.remove(&fc) ONLY happens during fork-in cleanup (line 305). Fork-in requires the occupant to be at a DIFFERENT column than the current commit. When the stash's parent is assigned to the SAME column as the stash (because no other child claimed it first), there is no fork-in, and stash_lanes is never cleaned up for that column.
  implication: This is the root cause. stash_lanes persists forever for columns where a stash and its parent end up in the same column.

- timestamp: 2026-03-15
  checked: When does a stash parent end up in the same column as the stash?
  found: Non-HEAD branch scenario: (1) Stash S on non-HEAD branch tip C_tip. (2) S is processed first (newer timestamp). (3) S gets col N via find_free_column_near, stash_lanes.insert(N). (4) S claims C_tip at col N: pending_parents.insert(C_tip_oid, N). (5) C_tip is processed later: pending_parents.get(&C_tip_oid) = Some(N) -> col = N. (6) C_tip is NOT a stash but occupies a stash-lane column. (7) C_tip emits edges with dashed=true (stash_lanes.contains(&N)==true). (8) C_tip claims its parent at col N. (9) All ancestors at col N get dashed=true. The entire branch line is dashed.
  implication: HEAD-chain branches are protected by pre-reservation (pending_parents has all HEAD chain OIDs at col 0, so HEAD commits never go to the stash column). Non-HEAD branches are NOT protected and inherit the stash column.

- timestamp: 2026-03-15
  checked: Existing test coverage
  found: Tests cover stash on HEAD tip (inline), multiple stashes on HEAD, stash on mid-chain HEAD commit. No test covers stash on a non-HEAD branch. The bug is specific to non-HEAD branches because HEAD chain pre-reservation prevents the parent from going to the stash column.
  implication: The gap in test coverage allowed this regression.

## Resolution

root_cause: In src-tauri/src/git/graph.rs, the `stash_lanes` HashSet tracks columns allocated to branched stashes so their edges are rendered as dashed. However, `stash_lanes` is only removed during fork-in cleanup (line 305), which requires the stash's parent to be at a DIFFERENT column. For non-HEAD branches, when a stash branches to col N and claims its parent at col N (via pending_parents), the parent inherits that column. Since parent and stash are in the same column, no fork-in occurs, and `stash_lanes` is never cleared for col N. All subsequent commits at col N (the entire branch ancestry) get `dashed: true` from the check `stash_lanes.contains(&col)` at lines 335, 351, and 374. HEAD-chain branches are immune because their commits are pre-reserved at col 0 and never assigned to the stash column.
fix: The graph algorithm was rewritten — the offending `stash_lanes` HashSet no longer exists in src-tauri/src/git/graph.rs (0 references). Dashed state is now carried per-slot as `active_lanes[col] = Some((oid, dashed))`, set by the commit that creates or takes over the lane (graph.rs:13-15, 135-136, 248-265). The original lifecycle bug — `stash_lanes` persisting after a stash's parent inherited its column — is structurally impossible under the new representation.
verification: Verified 2026-05-28 against src-tauri/src/git/graph.rs (`grep -c stash_lanes` returns 0; dashed propagation is per-slot).
files_changed:
- src-tauri/src/git/graph.rs
